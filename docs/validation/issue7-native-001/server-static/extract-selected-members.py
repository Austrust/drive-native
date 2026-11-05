"""Extract named software members only; never execute package contents."""
from pathlib import Path
import hashlib
import json
import tarfile

root = Path(__file__).resolve().parent
expected = {
    "sbin/syncd": ("syncd.elf", "e2288b191e8db97673c3468595e3a2d441579d2da365927c1108c000549fdf9c"),
    "lib/libsynosyncserver.so": ("libsynosyncserver.so.elf", "1677fcc08004cb9128ebd4efccd0d7053586de87042dd54880bc3e2e124b617c"),
    "lib/libsynosyncservercore.so": ("libsynosyncservercore.so.elf", "cfcad81883c49285b34a986eec78dc4702bf8312d4b1d847a07b095fbb93ab3b"),
    "lib/libsynodrivecommon.so.7.0": ("libsynodrivecommon.so.7.0.elf", "9f213f24bd8c4a3e9e3bbfbd5033e27086f98176b01cd824196e1846a057f32e"),
}
archive = root / "package.tgz"
assert hashlib.sha256(archive.read_bytes()).hexdigest() == "8465acfa404b662dd13a2759f7d5226e2095ed39c4624a90e620ebc9454df9e4"
records = []
with tarfile.open(archive, "r:*") as package:
    for member_name, (output_name, expected_sha) in expected.items():
        member = package.getmember(member_name)
        assert member.isfile() and member.size < 10_000_000
        data = package.extractfile(member).read()
        digest = hashlib.sha256(data).hexdigest()
        assert digest == expected_sha
        output = root / output_name
        if output.exists():
            assert output.read_bytes() == data
        else:
            output.write_bytes(data)
        records.append({"member": member_name, "file": output_name, "bytes": len(data), "sha256": digest, "executed": False})
(root / "selected-members-manifest.json").write_text(json.dumps(records, indent=2) + "\n", encoding="utf-8")
print(json.dumps(records, indent=2))
