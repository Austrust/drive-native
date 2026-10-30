"""Reproducible static evidence; does not execute or modify Synology binaries.

Requires pefile and capstone. Pass --python-libs for the unpacking environment.
RVA selections are specific to the hashed sample, not stable public interfaces.
"""
import argparse
import bisect
import hashlib
import json
from pathlib import Path
import sys

p = argparse.ArgumentParser()
p.add_argument("--package", required=True, type=Path)
p.add_argument("--out", required=True, type=Path)
p.add_argument("--python-libs", type=Path)
p.add_argument("--focus", choices=["placeholder", "save"], default="placeholder")
a = p.parse_args()
if a.python_libs:
    sys.path.insert(0, str(a.python_libs))
import pefile
import capstone

manifest = {"method": "PE exception ranges, exports, normal and delay IAT; Capstone x86-64", "samples": []}
lines = []
for filename in (["cloud-drive-daemon.exe"] if a.focus == "save" else ["WinCFWrapper.dll", "cloud-drive-daemon.exe"]):
    path = a.package / filename
    pe = pefile.PE(str(path))
    base = pe.OPTIONAL_HEADER.ImageBase
    ranges = sorted((x.struct.BeginAddress, x.struct.EndAddress) for x in pe.DIRECTORY_ENTRY_EXCEPTION)
    starts = [s for s, _ in ranges]
    imports = {s.address: s.name.decode() if s.name else str(s.ordinal)
               for attr in ("DIRECTORY_ENTRY_IMPORT", "DIRECTORY_ENTRY_DELAY_IMPORT")
               for e in getattr(pe, attr, []) for s in e.imports}
    md = capstone.Cs(capstone.CS_ARCH_X86, capstone.CS_MODE_64)
    md.detail = True
    def notes(ins):
        result = []
        for op in ins.operands:
            if op.type == capstone.x86.X86_OP_MEM and op.mem.base == capstone.x86.X86_REG_RIP:
                addr = ins.address + ins.size + op.mem.disp
                if addr in imports:
                    result.append(imports[addr])
                else:
                    raw = pe.get_data(addr - base, 220).split(b"\0")[0]
                    if len(raw) > 5 and all(32 <= c < 127 for c in raw):
                        result.append(raw.decode())
            elif op.type == capstone.x86.X86_OP_IMM and ins.mnemonic in ("call", "jmp"):
                raw = pe.get_data(op.imm - base, 6)
                if len(raw) == 6 and raw[:2] == b"\xff\x25":
                    addr = op.imm + 6 + int.from_bytes(raw[2:], "little", signed=True)
                    if addr in imports:
                        result.append(imports[addr])
        return result
    selections = []
    if filename.endswith("dll"):
        for x in pe.DIRECTORY_ENTRY_EXPORT.symbols:
            name = x.name.decode() if x.name else ""
            if any(s in name for s in ("PlaceholderUnpin", "PlaceholderForceUnpin", "PlaceholderPinToLocal", "PlaceholderSetInSync", "PlaceholderSetNotInSync", "ProactiveTransfer", "PlaceholderDehydrateAll", "PlaceholderHydrateAll")):
                selections.append((name, x.address, None))
        selections += [("shared pin implementation", 0x6f00, None), ("dehydrate-all implementation", 0x6620, None)]
    else:
        selections = [("callback registration", 0x6887d, 50), ("hydrated/identity check", 0x103685, 40), ("background transfer", 0x1296a0, 25), ("foreground transfer", 0x1442e0, 25), ("download sync mark", 0x11482c, 20), ("upload sync mark", 0x18b0aa, 20), ("rename acknowledgement", 0x8ddc8, 25), ("delete acknowledgement", 0x80ce4, 20)]
    if a.focus == "save":
        selections = [("complete upload sync-state function", 0x18b0aa, None), ("complete rename acknowledgement function", 0x8ddc8, None)]
    sample = {"name": filename, "sha256": hashlib.sha256(path.read_bytes()).hexdigest(), "size": path.stat().st_size, "selections": []}
    lines += ["\nSAMPLE " + filename + " SHA256 " + sample["sha256"]]
    for name, rva, window in selections:
        idx = bisect.bisect_right(starts, rva) - 1
        start, end = ranges[idx]
        if not start <= rva < end:
            raise RuntimeError(f"RVA {rva:x} outside function; sample mismatch")
        instructions = list(md.disasm(pe.get_data(start, end-start), base+start))
        center = next(i for i, x in enumerate(instructions) if x.address == base+rva)
        if window:
            instructions = instructions[max(0, center-window):center+window+1]
        sample["selections"].append({"label": name, "rva": hex(rva), "function": [hex(start), hex(end)], "bounded_window": window})
        lines.append(f"\n{name} RVA {rva:#x}; function {start:#x}..{end:#x}")
        lines.extend(f"{x.address-base:#x} {x.mnemonic} {x.op_str} | {' | '.join(notes(x))}" for x in instructions)
    manifest["samples"].append(sample)
a.out.mkdir(parents=True, exist_ok=True)
(a.out / ("windows-save-disassembly.txt" if a.focus == "save" else "windows-contract-disassembly.txt")).write_text("\n".join(lines)+"\n", encoding="utf8")
(a.out / ("windows-save-samples.json" if a.focus == "save" else "windows-contract-samples.json")).write_text(json.dumps(manifest, indent=2)+"\n", encoding="utf8")
print(json.dumps({"samples": len(manifest["samples"]), "lines": len(lines), "out": str(a.out)}))
