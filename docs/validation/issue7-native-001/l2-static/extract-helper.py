"""Bounded static bytes for GetAdvSharePrivilege's direct list matcher.

No target binary is executed. The unnamed helper has no dynamic symbol size;
the preserved window ends at the next independently visible prologue.
"""
from pathlib import Path
import argparse
import hashlib
import json
import sys

p = argparse.ArgumentParser(description=__doc__)
p.add_argument("library", type=Path)
p.add_argument("--reader", type=Path, required=True)
p.add_argument("--out", type=Path, required=True)
p.add_argument("--python-libs", type=Path, required=True)
a = p.parse_args()
saved = sys.argv
sys.argv = [str(a.reader), str(a.library), "--out", "unused-read-only", "--python-libs", str(a.python_libs)]
ns = {"__name__": "static_reader"}
exec(a.reader.read_text(encoding="utf-8").split("args.out.mkdir")[0], ns)
sys.argv = saved
assert hashlib.sha256(ns["b"]).hexdigest() == "1677fcc08004cb9128ebd4efccd0d7053586de87042dd54880bc3e2e124b617c"
start, end = 0x10ae80, 0x10b120
lines = ["Unnamed direct callee of SDK::GetAdvSharePrivilege; bounded window [0x10ae80, 0x10b120)."]
for ins in ns["md"].disasm(ns["read"](start, end-start), start):
    notes = []
    for op in ins.operands:
        if op.type == 3 and op.mem.base == ns["capstone"].x86.X86_REG_RIP:
            va = ins.address + ins.size + op.mem.disp
            if val := ns["cstr"](va):
                notes.append(val)
        if op.type != 2 or ins.mnemonic not in ("call", "jmp"):
            continue
        if op.imm in ns["names"]:
            notes.append(ns["names"][op.imm])
            continue
        try:
            first = next(ns["md"].disasm(ns["read"](op.imm, 16), op.imm))
            if first.mnemonic == "jmp" and first.operands[0].type == 3:
                va = first.address + first.size + first.operands[0].mem.disp
                if va in ns["got"]:
                    notes.append("PLT:" + ns["got"][va])
        except (ValueError, StopIteration):
            pass
    lines.append(f"{ins.address:x}: {ins.mnemonic} {ins.op_str} ; " + " | ".join(notes))
a.out.mkdir(parents=True, exist_ok=True)
(a.out / "window-10ae80-10b120.asm").write_text("\n".join(lines)+"\n", encoding="utf-8")
(a.out / "helper-window.json").write_text(json.dumps({
    "binary_sha256": hashlib.sha256(ns["b"]).hexdigest(),
    "start": hex(start), "end_exclusive": hex(end),
    "window_sha256": hashlib.sha256(ns["read"](start, end-start)).hexdigest(),
    "symbol_size_known": False,
    "selection": "Direct target of calls 0x10fcb5, 0x10fcf1, 0x10ff16 in SDK::GetAdvSharePrivilege; stop before next prologue at 0x10b120",
    "binary_executed": False,
}, indent=2)+"\n", encoding="utf-8")
print("Saved bounded helper window; target binary was not executed.")
