#!/bin/bash
set -euo pipefail
w=/mnt/c/Users/a_tas/Documents/Codex/2026-10-28/dui-yu/work
r=/mnt/c/Users/a_tas/Documents/Codex/2026-10-28/dui-yu/outputs/drive-native-github
b=$(mktemp -d /tmp/drive-p0-durability-build.XXXXXX)
printf '%s' "$b" > "$w/p0-durability-build-location.txt"
git -C "$r" archive HEAD | tar -x -C "$b"
export RUSTUP_HOME="$w/toolchains/rustup" CARGO_HOME="$w/toolchains/cargo"
export CARGO_TARGET_DIR=/home/drive-native-test/.cache/drive-native-build/target
export PATH="$CARGO_HOME/bin:/usr/local/bin:/usr/bin:/bin"
cd "$b"
cargo build --release --locked -p drive-core --features native-research --example durability_fixture > "$w/p0-durability-build.log" 2>&1
printf 'fixture builder compiled\n'
