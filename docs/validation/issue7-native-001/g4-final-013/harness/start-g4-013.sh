#!/usr/bin/env bash
# T4 independent verifier entry point for the full G4-013 matrix (attempt 2).
# The runner sits next to its helper scripts (run-native-loopback.sh,
# verify-worker-versions.py, prepare-core-probe.py, verify-history-labels.py,
# native-loopback/) because run-g4.py resolves them next to itself.
# Attempt 1 failed before product tests because the runner was launched from
# harness/prepared-g4-013/ where those helpers are absent; that attempt is kept
# as evidence and is not a product result.
# Registry seed: only debug/release registry dependency artifacts copied by
# run-g4.py's Cargo.lock filter from this verifier's own attempt-1 target; no
# author target and no product artifact is copied.
set -euo pipefail
source /mnt/c/Users/a_tas/Desktop/MFMLAB/Personal_Sync/20-Coding/P032-群晖Drive按需同步/work/fsearch-perf-runs/env.sh
export CARGO_INCREMENTAL=0
BUNDLE="$P032_WORK/issue7-validation/G4-evidence-bundle-013-final"
PREP="$P032_WORK/issue7-validation/prepared-g4-013"
BASE=/home/drive-native-test/.local/state/drive-native-ultra-validation
test ! -e "$PREP/launcher.log"
python3 -B "$BUNDLE/harness/run-g4-013.py" \
  --archives "$PREP" \
  --run "$BASE/g4-013" \
  --candidate-commit fefb8ee0b02f68ec0e76d9cd89b6f37172d645b4 \
  --registry-seed "$BASE/g4-013-target-retry-seed" \
  --nautilus-private-prefix "$BASE/g4-002/nautilus-private/prefix" \
  --g4-authorized 2>&1 | tee "$PREP/launcher.log"