"""Save pinned public archive-format reference source as inert text."""
from pathlib import Path
import hashlib
import json
import urllib.request
import urllib.error

ROOT = Path(__file__).resolve().parent
refs = [("synacktiv/synodecrypt", "synodecrypt.py"),
        ("prt1999/SynoXtract", "main.cpp")]
manifest = []
for repository, filename in refs:
    commit_url = f"https://api.github.com/repos/{repository}/commits/main"
    try:
        with urllib.request.urlopen(commit_url, timeout=30) as response:
            commit = json.load(response)["sha"]
    except urllib.error.HTTPError as error:
        if error.code != 403:
            raise
        commit = None  # unauthenticated API rate limit; preserve fetched bytes by SHA256
    url = f"https://raw.githubusercontent.com/{repository}/{commit or 'main'}/{filename}"
    with urllib.request.urlopen(url, timeout=30) as response:
        source = response.read(1024 * 1024)
    target = ROOT / (repository.split("/")[0] + "-" + filename + ".txt")
    if target.exists():
        raise SystemExit("Refusing to replace reference evidence")
    target.write_bytes(source)
    manifest.append({"repository": repository, "commit": commit, "source_url": url,
                     "file": target.name, "bytes": len(source),
                     "sha256": hashlib.sha256(source).hexdigest(), "executed": False})
(ROOT / "reference-sources.json").write_text(json.dumps(manifest, indent=2)+"\n", encoding="utf-8")
print(json.dumps([{k:v for k,v in item.items() if k in ['repository','commit','bytes','sha256']} for item in manifest]))
