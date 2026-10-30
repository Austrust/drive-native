#!/bin/bash
set -euo pipefail
w=/mnt/c/Users/a_tas/Documents/Codex/2026-10-28/dui-yu/work
b=$(cat "$w/p0-durability-build-location.txt")
case "$b" in /tmp/drive-p0-durability-build.*) ;; *) exit 2;; esac
export RUSTUP_HOME="$w/toolchains/rustup" CARGO_HOME="$w/toolchains/cargo"
export CARGO_TARGET_DIR=/home/drive-native-test/.cache/drive-native-build/target
export PATH="$CARGO_HOME/bin:/usr/local/bin:/usr/bin:/bin"
cd "$b"
cargo test --release --locked -p drive-fs --test mounted --no-run > "$w/p0-kernel-boundary-build.log" 2>&1
timeout --signal=TERM --kill-after=10s 120s cargo test --release --locked -p drive-fs --test mounted -- --ignored --test-threads=1 > "$w/p0-kernel-boundary-tests.log" 2>&1
printf 'six mounted kernel tests completed\n'
