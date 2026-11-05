#!/usr/bin/env bash
set -euo pipefail
source /mnt/c/Users/a_tas/Desktop/MFMLAB/Personal_Sync/20-Coding/P032-群晖Drive按需同步/work/fsearch-perf-runs/env.sh
harness="$P032_WORK/issue7-validation"
python3 - <<'PY'
import json,pathlib
path=pathlib.Path('/home/drive-native-test/.local/state/drive-native-ultra-validation/g4-007/fresh-001/summary.json')
data=json.loads(path.read_text())
assert data['complete'] and not data['changed_source_files'] and not data['residual_owned_mounts']
PY
exec python3 "$harness/run-g4.py" --archives "$harness/prepared-g4-009" \
  --run /home/drive-native-test/.local/state/drive-native-ultra-validation/g4-009 \
  --candidate-commit 4e6aaf7166fd370076adf43a0dad3e6ecdca36d6 \
  --copy-target /home/drive-native-test/.local/state/drive-native-ultra-validation/g4-007/target \
  --nautilus-private-prefix /home/drive-native-test/.local/state/drive-native-ultra-validation/g4-002/nautilus-private/prefix \
  --g4-authorized
