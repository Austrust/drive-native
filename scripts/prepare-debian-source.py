#!/usr/bin/env python3
"""Prepare an isolated Debian build tree, including compile-time protocol fixtures."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import shutil


def prepare(source: Path, target: Path):
    source = source.resolve(strict=True)
    target = target.resolve(strict=False)
    if target.exists() or source == target or source in target.parents:
        raise ValueError("build destination must be new and outside the project")
    target.parent.resolve(strict=True)
    if not (source / "Cargo.lock").is_file():
        raise ValueError("source is not the locked Drive Native workspace")
    fixtures = set()
    for code in (source / "crates").rglob("*.rs"):
        for literal in re.findall(
            r'include_(?:str|bytes)!\(\s*"([^"\n]+)"', code.read_text(encoding="utf-8")
        ):
            value = (code.parent / literal).resolve(strict=True)
            relative = value.relative_to(source)
            if relative.parts[0] == "evidence":
                fixtures.add(relative)
            elif relative.parts[0] != "crates":
                raise ValueError(f"unregistered external compile-time input: {relative}")
    target.mkdir()
    directories = ["crates", "desktop", "scripts", "debian"]
    if (source / "vendor").is_dir():
        directories.append("vendor")
    for name in directories:
        for item in (source / name).rglob("*"):
            if item.is_symlink():
                raise ValueError(
                    f"source symlink requires explicit review: {item.relative_to(source)}"
                )
        shutil.copytree(
            source / name,
            target / name,
            ignore=shutil.ignore_patterns(
                "__pycache__", "*.pyc", "target", "build", ".git", ".venv"
            ),
        )
    for name in ("Cargo.toml", "Cargo.lock", "LICENSE", "README.md", "DELIVERY_GATES.md"):
        shutil.copyfile(source / name, target / name)
    for relative in sorted(fixtures):
        (target / relative).parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(source / relative, target / relative)
    files = [
        {
            "path": f.relative_to(target).as_posix(),
            "bytes": f.stat().st_size,
            "sha256": hashlib.sha256(f.read_bytes()).hexdigest(),
        }
        for f in sorted(target.rglob("*"))
        if f.is_file()
    ]
    manifest = {
        "purpose": "isolated Debian build inputs; not a full research source archive",
        "compile_time_fixtures": [f.as_posix() for f in sorted(fixtures)],
        "files": files,
    }
    (target / "build-source-manifest.json").write_text(
        json.dumps(manifest, indent=2) + "\n", encoding="utf-8"
    )
    print(json.dumps({"files": len(files), "protocol_fixtures": manifest["compile_time_fixtures"]}))


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("destination", type=Path)
    parser.add_argument("--source", type=Path, default=Path(__file__).resolve().parent.parent)
    options = parser.parse_args()
    prepare(options.source, options.destination)
