#!/usr/bin/env bash
set -euo pipefail
source /mnt/c/Users/a_tas/Desktop/MFMLAB/Personal_Sync/20-Coding/P032-群晖Drive按需同步/work/fsearch-perf-runs/env.sh
harness="$P032_WORK/ultra-benchmark"
python3 - <<'PY'
import json,pathlib
root=pathlib.Path('/home/drive-native-test/.local/state/drive-native-ultra-validation')
for relative in ['g4-003/summary.json','g4-003/supplement/summary.json','functional-g4-003/result.json']:
    assert json.loads((root/relative).read_text())['complete'], 'prior G4 must finish before copying target: '+relative
assert json.loads((root/'g4-003/final-integrity.json').read_text())['passed']
PY
exec python3 "$harness/run-g4.py" --archives "$harness/prepared-g4-004" \
  --run /home/drive-native-test/.local/state/drive-native-ultra-validation/g4-004 \
  --candidate-commit b3dfff5992e2421c3898ff7e79203a2d23c2f5a3 \
  --copy-target /home/drive-native-test/.local/state/drive-native-ultra-validation/g4-003/target \
  --nautilus-private-prefix /home/drive-native-test/.local/state/drive-native-ultra-validation/g4-002/nautilus-private/prefix \
  --g4-authorized
