#!/usr/bin/env bash
set -euo pipefail
source /mnt/c/Users/a_tas/Desktop/MFMLAB/Personal_Sync/20-Coding/P032-群晖Drive按需同步/work/fsearch-perf-runs/env.sh
harness="$P032_WORK/ultra-benchmark"
python3 - <<'PY'
import json,pathlib
root=pathlib.Path('/home/drive-native-test/.local/state/drive-native-ultra-validation')
for relative in ['g4-002/summary.json','g4-002/supplement/summary.json','functional-g4-002/result.json']:
    assert json.loads((root/relative).read_text())['complete'], 'prior G4 must finish before copying target: '+relative
PY
exec python3 "$harness/run-g4.py" --archives "$harness/prepared-g4-003" \
  --run /home/drive-native-test/.local/state/drive-native-ultra-validation/g4-003 \
  --candidate-commit c744729141ca8b23bde63c61ca8a644bdfdaee81 \
  --copy-target /home/drive-native-test/.local/state/drive-native-ultra-validation/g4-002/target \
  --nautilus-private-prefix /home/drive-native-test/.local/state/drive-native-ultra-validation/g4-002/nautilus-private/prefix \
  --g4-authorized
