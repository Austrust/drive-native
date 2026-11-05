#!/usr/bin/env bash
# T11 independent verifier supplement driver launcher (G4-014).
set -euo pipefail
source /mnt/c/Users/a_tas/Desktop/MFMLAB/Personal_Sync/20-Coding/P032-群晖Drive按需同步/work/fsearch-perf-runs/env.sh
export CARGO_INCREMENTAL=0
BUNDLE="$P032_WORK/issue7-validation/G4-evidence-bundle-014-final"
mkdir -p "$BUNDLE/environment"
python3 -B "$BUNDLE/harness/run-g4-014-supplements.py" 2>&1 | tee "$BUNDLE/environment/supplemental-launcher.log"