#!/usr/bin/env bash
set -euo pipefail
run=${1:?private run path}
candidate=${2:?candidate full hash}
source /mnt/c/Users/a_tas/Desktop/MFMLAB/Personal_Sync/20-Coding/P032-群晖Drive按需同步/work/fsearch-perf-runs/env.sh
harness="$P032_WORK/issue7-validation/native-loopback"
probe="$run/native-loopback-crate"
python3 "$harness/prepare.py" --source "$run/source" --output "$probe" --candidate "$candidate"
cargo metadata --offline --manifest-path "$probe/Cargo.toml" --format-version 1 > "$probe/cargo-metadata.json"
python3 "$harness/verify-lock.py" --crate "$probe" --source "$run/source" --metadata "$probe/cargo-metadata.json"
cargo build --offline --locked --manifest-path "$probe/Cargo.toml" -j2
sha256sum "$CARGO_TARGET_DIR/debug/native-loopback-probe" > "$run/native-loopback-elf.sha256"
exec "$CARGO_TARGET_DIR/debug/native-loopback-probe" --work-dir "$run/native-loopback-scratch" --output "$run/native-loopback-results.json"
