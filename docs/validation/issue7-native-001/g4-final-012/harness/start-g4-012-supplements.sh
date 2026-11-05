#!/usr/bin/env bash
set -euo pipefail
source /mnt/c/Users/a_tas/Desktop/MFMLAB/Personal_Sync/20-Coding/P032-群晖Drive按需同步/work/fsearch-perf-runs/env.sh
export CARGO_INCREMENTAL=0
exec python3 -B "$P032_WORK/issue7-validation/run-g4-012-supplements.py"
