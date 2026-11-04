#!/usr/bin/env bash
# No execution before the coordinator supplies a completed G3 hash and G4 authorization.
set -euo pipefail
test "${1:-}" = g4-authorized
prepared=${2:?prepared source archives}
run_root=${3:?new isolated G4 root}
candidate=${4:?exact candidate commit}
source /mnt/c/Users/a_tas/Desktop/MFMLAB/Personal_Sync/20-Coding/P032-群晖Drive按需同步/work/fsearch-perf-runs/env.sh
script_dir=$(CDPATH='' cd -- "$(dirname -- "$0")" && pwd)
exec python3 "$script_dir/run-g4.py" --archives "$prepared" --run "$run_root" \
  --candidate-commit "$candidate" --g4-authorized
