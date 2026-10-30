"""Read-only Windows download evidence. Requires pefile and Capstone.

Addresses apply only to the sample recorded in the output SHA-256 manifest.
Does not execute the official client. Conditional branches require separate
data-flow review; an instruction reference alone is not a wire contract.
"""

import argparse
import bisect
import hashlib
import json
from pathlib import Path
import capstone
import pefile


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("daemon", type=Path)
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    pe = pefile.PE(str(args.daemon))
    base = pe.OPTIONAL_HEADER.ImageBase
    ranges = sorted(
        (x.struct.BeginAddress, x.struct.EndAddress) for x in pe.DIRECTORY_ENTRY_EXCEPTION
    )
    starts = [s for s, _ in ranges]
    md = capstone.Cs(capstone.CS_ARCH_X86, capstone.CS_MODE_64)
    md.detail = True
    result = {
        "sample_sha256": hashlib.sha256(args.daemon.read_bytes()).hexdigest(),
        "capstone_version": capstone.__version__,
        "pefile_version": pefile.__version__,
        "method": "Bounded PE functions; RIP strings and direct jump thunks; no execution",
        "functions": [],
    }
    # Verified against the prior Windows sample manifest, fail on other samples.
    expected = "f6c23a89cbbc145542d5169281ef1858ab9ebc8daa7d35a8fe6042d89b84fb26"
    if result["sample_sha256"] != expected:
        raise ValueError("sample mismatch: recover addresses for this binary first")
    lines = []
    for rva, label in [
        (0x11F320, "ordinary download request"),
        (0x141C90, "FetchData handler calls download helper"),
        (0x1AE2E0, "download helper with conditional current-version branch"),
    ]:
        start, end = ranges[bisect.bisect_right(starts, rva) - 1]
        if start != rva:
            raise ValueError("function boundary mismatch")
        refs = []
        lines.append(f"\nFUNCTION {label} RVA {start:x}..{end:x}")
        for ins in md.disasm(pe.get_data(start, end - start), base + start):
            notes = []
            for op in ins.operands:
                if op.type == capstone.x86.X86_OP_MEM and op.mem.base == capstone.x86.X86_REG_RIP:
                    addr = ins.address + ins.size + op.mem.disp
                    raw = pe.get_data(addr - base, 240).split(b"\0")[0]
                    if len(raw) > 2 and all(32 <= c < 127 for c in raw):
                        notes.append(raw.decode())
                if op.type == capstone.x86.X86_OP_IMM and ins.mnemonic == "call":
                    raw = pe.get_data(op.imm - base, 5)
                    if raw[:1] == b"\xe9":
                        notes.append(
                            "thunk->RVA "
                            + hex(
                                op.imm + 5 + int.from_bytes(raw[1:], "little", signed=True) - base
                            )
                        )
            line = f"{ins.address-base:x}: {ins.mnemonic} {ins.op_str} ; " + " | ".join(notes)
            lines.append(line)
            if notes:
                refs.append(line)
        result["functions"].append(
            {"label": label, "start_rva": hex(start), "end_rva": hex(end), "references": refs}
        )
    args.out.mkdir(parents=True, exist_ok=True)
    (args.out / "windows-download-samples.json").write_text(
        json.dumps(result, indent=2) + "\n", encoding="utf8"
    )
    (args.out / "windows-download-disassembly.txt").write_text(
        "\n".join(lines) + "\n", encoding="utf8"
    )
    print(json.dumps({"functions": len(result["functions"]), "sha256": result["sample_sha256"]}))


if __name__ == "__main__":
    main()
