#!/bin/bash
set -euo pipefail
w=/mnt/c/Users/a_tas/Documents/Codex/2026-10-28/dui-yu/work
r=/mnt/c/Users/a_tas/Documents/Codex/2026-10-28/dui-yu/outputs/drive-native-github
test -z "$(git -C "$r" status --porcelain)"
b=$(mktemp -d /tmp/drive-p0-wait-build.XXXXXX)
printf '%s' "$b" > "$w/p0-wait5-build-location.txt"
git -C "$r" rev-parse HEAD > "$w/p0-wait5-base-commit.txt"
git -C "$r" archive HEAD | tar -x -C "$b"
python3 "$w/p0-instrument-wait5.py" "$b"
export RUSTUP_HOME="$w/toolchains/rustup" CARGO_HOME="$w/toolchains/cargo"
export CARGO_TARGET_DIR=/home/drive-native-test/.cache/drive-native-build/target
export PATH="$CARGO_HOME/bin:/usr/local/bin:/usr/bin:/bin"
cd "$b"
cargo build --release --all-features --locked -p drive-runtime --bins > "$w/p0-wait5-diag-build.log" 2>&1
mkdir "$b/diag-bin"
cp "$CARGO_TARGET_DIR/release/drive-native-fsd" "$CARGO_TARGET_DIR/release/drive-native-worker" "$b/diag-bin/"
cargo test --locked --all-features -p sproto > "$w/p0-wait5-diag-protocol-tests.log" 2>&1
sha256sum "$b/diag-bin/"* > "$w/p0-wait5-diag-binaries.sha256"
printf 'Isolated diagnostic binaries compiled; protocol regressions passed\n'
