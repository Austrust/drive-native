"""Freeze text/static artifacts into a new evidence bundle, excluding software binaries."""
from pathlib import Path
import hashlib
import json
import re
import shutil

root = Path(__file__).resolve().parent
bundle = root / "evidence-bundle-002"
if bundle.exists():
    raise SystemExit("Bundle already exists; preserve it and use a new numbered bundle.")
files = sorted(p for p in root.iterdir() if p.is_file() and p.suffix in {".md", ".py", ".json", ".txt"} and p.name not in {"SHA256SUMS.txt", "evidence-manifest.json"})
for name in ("acl-boundary", "dispatch-boundary", "control-details", "flow", "filter-boundary", "core-capability", "acl-path", "filter-source"):
    files += sorted((root / name).glob("*.asm"))
    files.append(root / name / "analysis.json")
files.append(root / "symbols/symbols.json")
files = sorted(set(files))
records = [{"path": p.relative_to(root).as_posix(), "bytes": p.stat().st_size, "sha256": hashlib.sha256(p.read_bytes()).hexdigest()} for p in files]
manifest = {"purpose": "public package static permission evidence", "product_code_changed": False, "product_tests_run": False, "nas_contacted": False, "synology_code_executed": False, "binary_artifacts_included": False, "files": records}
(root / "evidence-manifest.json").write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
files.append(root / "evidence-manifest.json")
checksums = "".join(hashlib.sha256(p.read_bytes()).hexdigest() + "  " + p.relative_to(root).as_posix() + "\n" for p in sorted(files))
(root / "SHA256SUMS.txt").write_text(checksums, encoding="utf-8")
files.append(root / "SHA256SUMS.txt")
bundle.mkdir()
for file in files:
    output = bundle / file.relative_to(root)
    output.parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(file, output)
    assert output.read_bytes() == file.read_bytes()
links, broken = [], []
for doc in bundle.glob("*.md"):
    for target in re.findall(r"\]\(([^)]+)\)", doc.read_text(encoding="utf-8")):
        if "://" in target or target.startswith("#"):
            continue
        links.append((doc.name, target))
        if not (doc.parent / target.split("#", 1)[0]).is_file():
            broken.append((doc.name, target))
assert not broken, broken
summary = {"bundle": str(bundle), "files_including_checksums": len(files), "hashed_files": len(files) - 1, "local_markdown_links": len(links), "broken_links": len(broken), "report_sha256": hashlib.sha256((bundle / "REPORT.md").read_bytes()).hexdigest(), "checksums_sha256": hashlib.sha256((bundle / "SHA256SUMS.txt").read_bytes()).hexdigest()}
print(json.dumps(summary, indent=2))
