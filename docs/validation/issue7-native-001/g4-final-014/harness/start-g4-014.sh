#!/usr/bin/env bash
# T11 independent verifier entry point for the full G4-014 matrix (formal rerun).
# Attempt 3 (run g4-014, complete=false, 28 commands) was interrupted during
# workspace-default; it is preserved as g4-014-aborted-partial and its target is
# used only as a registry-dependency seed (Cargo.lock registry filter, no
# workspace/path package, test ELF or incremental artifact copied).
#
# Nautilus private dependency: the old g4-002 prepared prefix was removed from the
# WSL state dir by external cleanup; this run reuses the verifier-owned copy
# carried inside the G4-013 run output (same Ubuntu-package private extraction,
# not a product build artifact).
set -euo pipefail
# Attempt 4 (launched as root) is invalid: permissions_kernel asserts getuid != 0
# (kernel-parent-actions-preflight, core-history-probe); preserved as
# g4-014-root-invalid-partial and excluded from all product conclusions.
test "$(id -u)" -ne 0 || { echo 'refuse root: FUSE/kernel group requires non-root (drive-native-test)' >&2; exit 9; }
source /mnt/c/Users/a_tas/Desktop/MFMLAB/Personal_Sync/20-Coding/P032-群晖Drive按需同步/work/fsearch-perf-runs/env.sh
export CARGO_INCREMENTAL=0
BUNDLE="$P032_WORK/issue7-validation/G4-evidence-bundle-014-final"
PREP="$P032_WORK/issue7-validation/prepared-g4-014"
BASE=/home/drive-native-test/.local/state/drive-native-ultra-validation
test ! -e "$PREP/launcher-rerun.log"
test -d "$BASE/g4-013/nautilus-prefix"
test -d "$BASE/g4-014-rerun-target-seed"
test ! -e "$BASE/g4-014-rerun"
python3 -B "$BUNDLE/harness/run-g4-014.py" \
  --archives "$PREP" \
  --run "$BASE/g4-014-rerun" \
  --candidate-commit b303ac6bf01f6ff66f48d1e74cb94740561a841c \
  --registry-seed "$BASE/g4-014-rerun-target-seed" \
  --nautilus-private-prefix "$BASE/g4-013/nautilus-prefix" \
  --g4-authorized 2>&1 | tee "$PREP/launcher-rerun.log"
