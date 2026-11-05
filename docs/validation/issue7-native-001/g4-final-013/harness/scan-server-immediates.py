#!/usr/bin/env python3
"""Independent immediate-code scan of the public Drive Server ELF set.

Verifier-authored check of the T1 "0x9003 has one proven producer" claim and of
the codes kept as hints. Differs from the author's scan (`server-static/
permission-code-immediates.json`, syncd-only, function extents from dynamic
symbols): this scans every executable PT_LOAD segment of all four extracted
library/executable inputs with a linear capstone sweep and reports any
instruction whose immediate operand equals the target code.

Scope limits: linear sweep can desynchronise inside data embedded in executable
segments (results are checked by mnemonic/address); computed/encoded values and
cross-version binaries are out of scope. No binary is executed.
"""
import argparse
import hashlib
import json
import struct
import sys
from pathlib import Path

CODES = [0x9003, 0x9004, 0xa003, 0xa004]
PT_LOAD = 1
PF_X = 1


def parse_segments(data):
    assert data[:4] == b'\x7fELF' and data[4] == 2 and data[5] == 1, 'x86-64 ELF required'
    machine = struct.unpack_from('<H', data, 18)[0]
    assert machine == 62, machine
    phoff = struct.unpack_from('<Q', data, 32)[0]
    phentsize = struct.unpack_from('<H', data, 54)[0]
    phnum = struct.unpack_from('<H', data, 56)[0]
    segments = []
    for index in range(phnum):
        kind, flags, offset, vaddr, paddr, filesz, memsz, align = struct.unpack_from(
            '<IIQQQQQQ', data, phoff + index * phentsize)
        if kind == PT_LOAD and flags & PF_X and filesz:
            segments.append({'offset': offset, 'vaddr': vaddr, 'filesz': filesz,
                             'flags': flags})
    return segments


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--inputs', type=Path, required=True)
    parser.add_argument('--python-libs', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    sys.path.insert(0, str(args.python_libs))
    import capstone
    md = capstone.Cs(capstone.CS_ARCH_X86, capstone.CS_MODE_64)
    md.detail = True
    md.skipdata = False
    result = {'scope': ('linear capstone sweep of every PF_X PT_LOAD segment in the four '
                        'public extracted ELFs; no execution, no NAS'),
              'inputs': str(args.inputs), 'codes': [hex(code) for code in CODES],
              'files': [], 'findings': {hex(code): [] for code in CODES}}
    for name in ('syncd.elf', 'libsynosyncserver.so.elf', 'libsynosyncservercore.so.elf',
                 'libsynodrivecommon.so.7.0.elf'):
        path = args.inputs / name
        data = path.read_bytes()
        entry = {'file': name, 'bytes': len(data),
                 'sha256': hashlib.sha256(data).hexdigest(), 'exec_segments': {}}
        for segment in parse_segments(data):
            size = segment['filesz']
            blob = data[segment['offset']:segment['offset'] + size]
            found = 0
            instructions = 0
            for insn in md.disasm(blob, segment['vaddr']):
                instructions += 1
                for operand in insn.operands:
                    if operand.type == capstone.x86.X86_OP_IMM and operand.imm in CODES:
                        found += 1
                        result['findings'][hex(operand.imm)].append({
                            'file': name, 'address': hex(insn.address),
                            'mnemonic': insn.mnemonic, 'op_str': insn.op_str,
                            'immediate': hex(operand.imm)})
                        break
            entry['exec_segments'][hex(segment['vaddr'])] = {
                'vaddr': hex(segment['vaddr']), 'bytes': size,
                'decoded_instructions': instructions, 'matched_instructions': found}
        result['files'].append(entry)
    producer_mnemonics = {'mov', 'push'}
    consumer_mnemonics = {'cmp', 'sub', 'add', 'test', 'and', 'or', 'xor', 'lea'}
    for row in (row for rows in result['findings'].values() for row in rows):
        if row['mnemonic'] in producer_mnemonics:
            row['role'] = 'producer_immediate'
        elif row['mnemonic'] in consumer_mnemonics:
            row['role'] = 'consumer_or_compare'
        else:
            row['role'] = 'other'
    syncd = {hex(code): [row for row in result['findings'][hex(code)]
                         if row['file'] == 'syncd.elf'] for code in CODES}
    result['syncd_counts'] = {code: len(rows) for code, rows in syncd.items()}
    result['all_file_counts'] = {code: len(rows) for code, rows in result['findings'].items()}
    # The frozen author JSON has exactly 13 syncd references with this per-code
    # split: 0x9003 x1 (CreateACLTest::HandleError), 0x9004 x7, 0xa003 x1,
    # 0xa004 x4. Reproducing it with a different scanner independently validates
    # the scan, not only the report text.
    expected = {hex(0x9003): 1, hex(0x9004): 7, hex(0xa003): 1, hex(0xa004): 4}
    result['author_json_expected_syncd_counts'] = expected
    result['syncd_counts_reproduced'] = result['syncd_counts'] == expected
    result['other_library_hits'] = {
        code: [row for row in rows if row['file'] != 'syncd.elf']
        for code, rows in result['findings'].items()}
    result['other_library_hit_count'] = sum(len(rows) for rows in result['other_library_hits'].values())
    result['producer_counts_all_files'] = {
        code: sum(1 for row in rows if row.get('role') == 'producer_immediate')
        for code, rows in result['findings'].items()}
    result['producer_counts_syncd'] = {
        code: sum(1 for row in syncd[hex(code)] if row.get('role') == 'producer_immediate')
        for code in CODES}
    result['other_library_producer_hits'] = {
        code: [row for row in rows if row['file'] != 'syncd.elf'
               and row.get('role') == 'producer_immediate']
        for code, rows in result['findings'].items()}
    result['other_library_producer_count'] = sum(
        len(rows) for rows in result['other_library_producer_hits'].values())
    # Claim under test: 0x9003 is produced by exactly one immediate site, in
    # syncd (CreateACLTest::HandleError). 0x9004 is expected to have several
    # producers (the filter ambiguity documented by T1) and must not be unique.
    result['claim_9003_unique_producer'] = result['producer_counts_all_files'][hex(0x9003)] == 1
    result['claim_9004_multiple_producers'] = result['producer_counts_all_files'][hex(0x9004)] > 1
    result['passed'] = (result['syncd_counts_reproduced']
                        and result['claim_9003_unique_producer']
                        and result['claim_9004_multiple_producers'])
    args.output.write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps({'syncd_counts': result['syncd_counts'],
                      'syncd_reproduced': result['syncd_counts_reproduced'],
                      'other_library_hits': result['other_library_hit_count'],
                      'passed': result['passed']}))


if __name__ == '__main__':
    main()