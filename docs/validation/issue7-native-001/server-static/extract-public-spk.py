"""Offline selective extraction of the verified public software package.

Format references: reference-sources.json and https://doc.libsodium.org/key_derivation
Only inert INFO and package.tgz are emitted; no archive pathname is trusted.
"""
from pathlib import Path
import hashlib
import io
import json
import re
import struct
import sys
import tarfile

ROOT = Path(__file__).resolve().parent
sys.path.insert(0, str(ROOT / "python-libs"))
import msgpack
import nacl
from nacl.signing import VerifyKey
from nacl.bindings import (
    crypto_secretstream_xchacha20poly1305_state as State,
    crypto_secretstream_xchacha20poly1305_init_pull as init_pull,
    crypto_secretstream_xchacha20poly1305_pull as pull,
)

source = ROOT / "SynologyDrive-x86_64-4.0.3-27892.spk"
blob = source.read_bytes()
assert hashlib.sha256(blob).hexdigest() == "58417bbeb433d303b18715649b7bac07efbcef686468378456741eff8ef060fc"
assert blob[1:4] == bytes.fromhex("adbeef")
reference = (ROOT / "prt1999-main.cpp.txt").read_text(encoding="utf-8")
match = re.search(r'keys_\[3\]\s*=\s*\{hex_to_bytes\("([0-9A-F]+)"\),\s*hex_to_bytes\("([0-9A-F]+)"\)', reference)
assert match, "Public SPK format constants absent from saved reference"
verification, master = map(bytes.fromhex, match.groups())
header_len = struct.unpack_from("<I", blob, 4)[0]
header = blob[8:8 + header_len]
VerifyKey(verification).verify(header, blob[8 + header_len:72 + header_len])
metadata = msgpack.unpackb(header)
seed, entries = metadata[:2]
context = seed[24:31] + b"\0"
key = hashlib.blake2b(b"", digest_size=32, key=master,
                     salt=seed[16:24] + b"\0" * 8, person=context + b"\0" * 8).digest()

def start_stream(data):
    state = State()
    init_pull(state, data[:24], key)
    return state

cursor = 72 + header_len
records = []
for encoded_size, expected_digest in entries:
    part = blob[cursor:cursor + encoded_size]
    assert len(part) == encoded_size
    assert hashlib.blake2b(part, digest_size=32).digest() == expected_digest
    state = start_stream(part)
    tar_header, _ = pull(state, part[24:24 + 0x193])
    info = tarfile.TarInfo.frombuf(tar_header.ljust(512, b"\0"), "utf-8", "strict")
    record = {"member": info.name, "size": info.size, "encoded_offset": cursor,
              "encoded_bytes": encoded_size, "entry_digest_verified": True}
    normalized = info.name.removeprefix("./")
    if normalized in {"INFO", "package.tgz"}:
        assert info.isfile(), "Selected entry must be a regular file"
        output = ROOT / ("package-INFO.txt" if normalized == "INFO" else "package.tgz")
        if output.exists():
            raise SystemExit("Refusing to replace extracted evidence")
        state = start_stream(part[512:])
        position, remaining = 536, info.size
        digest = hashlib.sha256()
        with output.open("xb") as stream:
            while remaining:
                take = min(4 * 1024 * 1024, remaining)
                plaintext, _ = pull(state, part[position:position + take + 17])
                assert len(plaintext) == take
                stream.write(plaintext)
                digest.update(plaintext)
                position += take + 17
                remaining -= take
        record.update({"output": output.name, "sha256": digest.hexdigest()})
    records.append(record)
    cursor += encoded_size
assert cursor == len(blob), (cursor, len(blob))
manifest = {"header_signature_verified": True, "entry_digests_verified": len(records),
            "pynacl_version": nacl.__version__, "msgpack_version": msgpack.__version__,
            "binary_executed": False, "entries": records}
(ROOT / "spk-extraction.json").write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
with tarfile.open(ROOT / "package.tgz", "r:*") as archive:
    members = [{"name": m.name, "size": m.size, "is_file": m.isfile()} for m in archive]
(ROOT / "package-members.json").write_text(json.dumps(members, indent=2) + "\n", encoding="utf-8")
print(json.dumps({"entry_digests_verified": len(records), "selected": [r for r in records if "output" in r],
                  "syncd_candidates": [m for m in members if Path(m["name"]).name == "syncd"]}))
