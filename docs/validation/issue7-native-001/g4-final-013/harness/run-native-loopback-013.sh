#!/usr/bin/env bash
# T4 verifier rerun of the native loopback matrix with the two T1-intended
# expectation changes plus four adversarial additions (harness/native-loopback-013;
# see patch-native-loopback-013.py). The frozen 012 harness copy (native-loopback/)
# and its main-matrix result stay untouched.
#
# The probe package is renamed so cargo cannot reuse the stale unit artifact of the
# same name/version from the main-matrix build in the shared target dir (verifier
# observation: identical package name+version in the same CARGO_TARGET_DIR made
# cargo reuse the old binary, which was caught by the expectation mismatch).
set -euo pipefail
run=${1:?private run path}
candidate=${2:?candidate full hash}
source /mnt/c/Users/a_tas/Desktop/MFMLAB/Personal_Sync/20-Coding/P032-群晖Drive按需同步/work/fsearch-perf-runs/env.sh
harness="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/native-loopback-013"
probe="$run/native-loopback-crate-013"
out="$run/native-loopback-updated-001"
test ! -e "$probe"
test ! -e "$out"
mkdir -m 700 -p "$out"
python3 "$harness/prepare.py" --source "$run/source" --output "$probe" --candidate "$candidate"
sed -i 's/^name = "native-loopback-probe"$/name = "native-loopback-probe-013"/' "$probe/Cargo.toml"
grep -q '^name = "native-loopback-probe-013"$' "$probe/Cargo.toml"
cargo metadata --offline --manifest-path "$probe/Cargo.toml" --format-version 1 > "$probe/cargo-metadata.json"
python3 "$harness/verify-lock.py" --crate "$probe" --source "$run/source" --metadata "$probe/cargo-metadata.json"
cargo build --offline --locked --manifest-path "$probe/Cargo.toml" -j2
test -x "$CARGO_TARGET_DIR/debug/native-loopback-probe-013"
sha256sum "$CARGO_TARGET_DIR/debug/native-loopback-probe-013" > "$out/elf.sha256"
exec "$CARGO_TARGET_DIR/debug/native-loopback-probe-013" --work-dir "$out/scratch" --output "$out/results.json"