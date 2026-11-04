#!/usr/bin/env bash
set -euo pipefail
source /mnt/c/Users/a_tas/Desktop/MFMLAB/Personal_Sync/20-Coding/P032-群晖Drive按需同步/work/fsearch-perf-runs/env.sh
harness="$P032_WORK/ultra-benchmark"
python3 - /home/drive-native-test/.local/state/drive-native-ultra-validation/g4-001/summary.json <<'PY'
import json,pathlib,sys
assert json.loads(pathlib.Path(sys.argv[1]).read_text())['complete'],'original run must finish before copying its target'
PY
exec python3 "$harness/run-g4.py" --archives "$harness/prepared-g4-002-final" \
  --run /home/drive-native-test/.local/state/drive-native-ultra-validation/g4-002 \
  --candidate-commit 652eabe741396c8332a7182599aaf4e53d5af48b \
  --copy-target /home/drive-native-test/.local/state/drive-native-ultra-validation/g4-001/target --g4-authorized
