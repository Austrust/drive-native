"""Download one unauthenticated public official software package; do not execute it."""
from pathlib import Path
import datetime
import hashlib
import json
import urllib.request

ROOT = Path(__file__).resolve().parent
INDEX = "https://archive.synology.com/download/Package/SynologyDrive/4.0.3-27892"
URL = "https://global.synologydownload.com/download/Package/spk/SynologyDrive/4.0.3-27892/SynologyDrive-x86_64-4.0.3-27892.spk"
target = ROOT / "SynologyDrive-x86_64-4.0.3-27892.spk"
if target.exists():
    raise SystemExit("Package already exists; inspect the existing evidence instead of replacing it")
start = datetime.datetime.now(datetime.timezone.utc).isoformat()
with urllib.request.urlopen(URL, timeout=60) as response:
    final_url = response.url
    if not final_url.startswith("https://global.synologydownload.com/"):
        raise SystemExit(f"Unexpected redirect: {final_url}")
    headers = dict(response.headers)
    status = response.status
    size = 0
    digest = hashlib.sha256()
    with target.open("xb") as output:
        while chunk := response.read(1024 * 1024):
            size += len(chunk)
            if size > 80 * 1024 * 1024:
                raise SystemExit("Unexpected package size")
            output.write(chunk)
            digest.update(chunk)
manifest = {"index_url": INDEX, "source_url": URL, "final_url": final_url,
            "http_status": status, "response_headers": headers,
            "started_utc": start, "completed_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(),
            "filename": target.name, "bytes": size, "sha256": digest.hexdigest(),
            "executed": False, "authentication": "none", "nas_contacted": False}
(ROOT / "download-manifest.json").write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
print(json.dumps({"status": status, "bytes": size, "sha256": digest.hexdigest()}))
