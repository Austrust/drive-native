"""Static vtable and four-immediate audit. No imported or executed Synology code."""
from pathlib import Path
import argparse
import json
import struct
import sys

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("--python-libs", type=Path, required=True)
args = parser.parse_args()
sys.path.insert(0, str(args.python_libs))
import capstone

root = Path(__file__).resolve().parent
binary = (root / "syncd.elf").read_bytes()
header = struct.unpack_from("<16sHHIQQQIHHHHHH", binary)
segments = [struct.unpack_from("<IIQQQQQQ", binary, header[5] + i * header[9]) for i in range(header[10])]

def read(address, size):
    for kind, flags, offset, virtual, physical, file_size, memory_size, alignment in segments:
        if kind == 1 and virtual <= address and address + size <= virtual + file_size:
            return binary[offset + address - virtual:offset + address - virtual + size]
    raise ValueError(hex(address))

symbols = json.loads((root / "symbols/symbols.json").read_text())
names = {s["address"]: s["name"] for s in symbols if s["address"] and s["defined"]}
selected = {"_ZTV13CreateACLTest", "_ZTV13RemoveACLTest", "_ZTV19RenameSourceACLTest", "_ZTV17RenameDestACLTest", "_ZTV19TestPhaseMiddleware", "_ZTV13UploadHandler", "_ZTV13RemoveHandler", "_ZTV13RenameHandler", "_ZTV7ACLTest"}
lines = []
for symbol in symbols:
    if symbol["name"] not in selected:
        continue
    lines.append(symbol["name"] + " " + hex(symbol["address"]))
    for index in range(0, symbol["size"], 8):
        value = struct.unpack("<Q", read(symbol["address"] + index, 8))[0]
        lines.append(f"  address-point-slot {index - 16:+#x}: {value:#x} {names.get(value, '')}")
(root / "vtables.txt").write_text("\n".join(lines) + "\n", encoding="utf-8")

decoder = capstone.Cs(capstone.CS_ARCH_X86, capstone.CS_MODE_64)
decoder.detail = True
matches, seen = [], set()
for symbol in symbols:
    if not symbol["defined"] or symbol["kind"] != 2 or not symbol["size"] or symbol["address"] in seen:
        continue
    seen.add(symbol["address"])
    for instruction in decoder.disasm(read(symbol["address"], symbol["size"]), symbol["address"]):
        if any(operand.type == capstone.x86.X86_OP_IMM and operand.imm in (0x9003, 0x9004, 0xa003, 0xa004) for operand in instruction.operands):
            matches.append({"function": symbol["name"], "start": hex(symbol["address"]), "address": hex(instruction.address), "instruction": instruction.mnemonic + " " + instruction.op_str})
result = {"scope": "all defined function dynamic-symbol extents; not a proof against computed codes or unexported code", "function_extents": len(seen), "matches": matches}
(root / "permission-code-immediates.json").write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
print(json.dumps({"function_extents": len(seen), "immediate_matches": len(matches), "vtable_count": len(selected)}))
