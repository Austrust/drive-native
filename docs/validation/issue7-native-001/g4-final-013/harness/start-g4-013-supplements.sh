#!/usr/bin/env bash
# T4 independent verifier supplement driver launcher.
set -euo pipefail
source /mnt/c/Users/a_tas/Desktop/MFMLAB/Personal_Sync/20-Coding/P032-群晖Drive按需同步/work/fsearch-perf-runs/env.sh
export CARGO_INCREMENTAL=0
BUNDLE="$P032_WORK/issue7-validation/G4-evidence-bundle-013-final"
mkdir -p "$BUNDLE/environment"
python3 -B "$BUNDLE/harness/run-g4-013-supplements.py" 2>&1 | tee "$BUNDLE/environment/supplemental-launcher.log"