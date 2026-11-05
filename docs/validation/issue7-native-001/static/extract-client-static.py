"""Read an already downloaded official ELF; never execute it or use the network.

Writes only the selected static evidence into this script's directory.
"""
from pathlib import Path
import argparse
import hashlib
import json
import struct
import sys

ROOT = Path(__file__).resolve().parent
parser = argparse.ArgumentParser()
parser.add_argument("--binary", type=Path, required=True)
parser.add_argument("--symbols", type=Path, required=True)
parser.add_argument("--pylibs", type=Path, required=True)
args = parser.parse_args()
sys.path.insert(0, str(args.pylibs))
import capstone

blob = args.binary.read_bytes()
digest = hashlib.sha256(blob).hexdigest()
expected = "ea01d634762060424f4a5fd3fccb3f5c0ba7c10a81e90e144706fb2b8406314a"
if digest != expected:
    raise SystemExit("Unexpected ELF SHA256; do not reuse the recorded addresses")
header = struct.unpack_from("<16sHHIQQQIHHHHHH", blob)
sections = [struct.unpack_from("<IIQQQQIIQQ", blob, header[6] + i * header[11])
            for i in range(header[12])]
symbols = json.loads(args.symbols.read_text(encoding="utf-8"))
names = {v["va"]: k for k, v in symbols.items() if v["type"] == 2 and v["va"]}

def offset(address):
    for section in sections:
        if section[1] != 8 and section[3] <= address < section[3] + section[5]:
            return section[4] + address - section[3]
    raise ValueError(f"Unmapped address {address:x}")

def string(address):
    try:
        start = offset(address)
    except ValueError:
        return ""
    raw = blob[start:start + 600].split(b"\0", 1)[0]
    if raw and all(32 <= x < 127 or x in (9, 10, 13) for x in raw):
        return raw.decode("ascii")
    return ""

selected = [
    "_Z19ProtoResp2ErrorCodej",
    "_ZN13HandlerHelper34ConvertResponseErrorToHandlerErrorEj",
    "_ZN21DownloadRemoteHandler27HandleDownloadResponseErrorEj",
    "_ZN18UploadLocalHandler25HandleUploadResponseErrorEj",
    "_ZN19RemoveRemoteHandler25HandleRemoveResponseErrorEjRK10DBFileInfo",
    "_ZN19RenameRemoteHandler25HandleRenameResponseErrorERK7PObject",
    "_ZN7Channel5CloseEv",
    "_ZN14ProtocolClient23RecvResponseFromChannelER7ChannelR7PObjectS3_",
    "_ZN22UpdatePrivilegeHandler16IsProfileChangedEv",
    "_ZN22UpdatePrivilegeHandler13ReloadProfileEv",
]
dis = capstone.Cs(capstone.CS_ARCH_X86, capstone.CS_MODE_64)
dis.detail = True
functions = []
for name in selected:
    item = symbols[name]
    va, size = item["va"], item["size"]
    lines = [name, f"address=0x{va:x} size={size}"]
    registers = {}
    for ins in dis.disasm(blob[offset(va):offset(va) + size], va):
        refs = []
        old = registers.copy()
        _, written = ins.regs_access()
        for register in written:
            registers.pop(register, None)
        if ins.mnemonic == "lea" and len(ins.operands) == 2:
            operand = ins.operands[1]
            if operand.type == capstone.x86.X86_OP_MEM:
                memory = operand.mem
                target = None
                if memory.base == capstone.x86.X86_REG_RIP:
                    target = ins.address + ins.size + memory.disp
                elif memory.base in old and not memory.index:
                    target = old[memory.base] + memory.disp
                if target is not None:
                    registers[ins.operands[0].reg] = target
                    text = string(target)
                    if text:
                        refs.append(f"0x{target:x} {text!r}")
        for operand in ins.operands:
            if operand.type == capstone.x86.X86_OP_IMM and ins.mnemonic == "call":
                if operand.imm in names:
                    refs.append(names[operand.imm])
        lines.append(f"{ins.address:x}: {ins.mnemonic} {ins.op_str} ; {' | '.join(refs)}")
        if ins.mnemonic in ("call", "ret", "jmp"):
            registers.clear()
    filename = f"function-{va:x}.asm"
    (ROOT / filename).write_text("\n".join(lines) + "\n", encoding="utf-8")
    functions.append({"symbol": name, "address": hex(va), "bytes": size, "file": filename})

table_symbol = symbols["_ZZL14GetErrorStringiE3map"]
table = []
for i in range(table_symbol["size"] // 8):
    pointer = struct.unpack_from("<Q", blob, offset(table_symbol["va"]) + i * 8)[0]
    table.append({"internal_error": -i, "string_address": hex(pointer), "text": string(pointer)})
(ROOT / "internal-errors.json").write_text(json.dumps(table, indent=2) + "\n", encoding="utf-8")
manifest = {
    "input_binary": str(args.binary.resolve()), "sha256": digest, "bytes": len(blob),
    "symbol_source": str(args.symbols.resolve()),
    "symbol_source_sha256": hashlib.sha256(args.symbols.read_bytes()).hexdigest(),
    "method": "ELF section mapping and Capstone static disassembly; no binary execution",
    "capstone_version": capstone.__version__, "functions": functions,
    "error_table_address": hex(table_symbol["va"]),
    "index_evidence": "Channel::Close at 0x2baca4 negates the internal code before indexing the table",
    "limitations": ["No live server probe", "No verification of server mutation commit boundaries",
                    "Static client version does not establish every server version's behavior",
                    "String annotations are evidence aids, not wire schema definitions"],
}
(ROOT / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
print(json.dumps({"sha256": digest, "functions": len(functions), "internal_error_strings": len(table)}))
