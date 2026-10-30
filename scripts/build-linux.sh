#!/usr/bin/env bash
set -euo pipefail
output=${1:?Supply absolute build output}
case "$output" in /*) ;; *) exit 2;; esac
: "${GUI_PYTHON:?Set GUI_PYTHON to the locked build venv python}"
project=$(CDPATH='' cd -- "$(dirname -- "$0")/.." && pwd)
cd "$project"
mkdir -p "$output"
# Refuse a silently different Python dependency set.
"$GUI_PYTHON" - "$project/scripts/gui-requirements.lock" <<'PY'
import importlib.metadata, pathlib, sys
for line in pathlib.Path(sys.argv[1]).read_text().splitlines():
    if line and not line.startswith('#'):
        name, version = line.split('==')
        assert importlib.metadata.version(name) == version, name
PY
cargo build -p drive-runtime --all-features --release --bins --locked
target=${CARGO_TARGET_DIR:-$project/target}
mkdir -p "$output/native"
for program in drive-native-fsd drive-native-worker drive-native-setup; do
    install -m755 "$target/release/$program" "$output/native/$program"
done
bash desktop/gio/build.sh "$output/gio"
test "$(cat "$output/gio/scope-mode.txt")" = registry
"$GUI_PYTHON" -m PyInstaller --noconfirm --clean --onedir --name drive-native-gui     --distpath "$output/gui-dist" --workpath "$output/gui-work" --specpath "$output"     --paths "$project/desktop" scripts/gui-entry.py
"$GUI_PYTHON" -m pip freeze --all > "$output/gui-dependencies.txt"
