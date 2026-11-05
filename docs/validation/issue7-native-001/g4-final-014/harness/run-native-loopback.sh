#!/usr/bin/env bash
# T11 verifier native loopback matrix (G4-013 corrected harness carried forward).
# Writes the run-g4.py contract output path ($run/native-loopback-results.json) so
# the main matrix row and the G4-013-baseline comparison use the same artifact.
#
# The probe package is renamed so cargo cannot reuse a stale unit artifact of the
# same name/version from another crate source in the shared target dir.
set -euo pipefail
run=${1:?private run path}
candidate=${2:?candidate full hash}
source /mnt/c/Users/a_tas/Desktop/MFMLAB/Personal_Sync/20-Coding/P032-群晖Drive按需同步/work/fsearch-perf-runs/env.sh
harness="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/native-loopback-014"
probe="$run/native-loopback-crate-014"
work="$run/native-loopback-scratch"
test ! -e "$probe"
mkdir -m 700 -p "$work"
python3 "$harness/prepare.py" --source "$run/source" --output "$probe" --candidate "$candidate"
sed -i 's/^name = "native-loopback-probe"$/name = "native-loopback-probe-014"/' "$probe/Cargo.toml"
grep -q '^name = "native-loopback-probe-014"$' "$probe/Cargo.toml"
cargo metadata --offline --manifest-path "$probe/Cargo.toml" --format-version 1 > "$probe/cargo-metadata.json"
python3 "$harness/verify-lock.py" --crate "$probe" --source "$run/source" --metadata "$probe/cargo-metadata.json"
cargo build --offline --locked --manifest-path "$probe/Cargo.toml" -j2
test -x "$CARGO_TARGET_DIR/debug/native-loopback-probe-014"
sha256sum "$CARGO_TARGET_DIR/debug/native-loopback-probe-014" > "$run/native-loopback-elf.sha256"
exec "$CARGO_TARGET_DIR/debug/native-loopback-probe-014" --work-dir "$work" --output "$run/native-loopback-results.json"