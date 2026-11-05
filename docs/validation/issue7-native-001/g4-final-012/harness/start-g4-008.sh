#!/usr/bin/env bash
set -euo pipefail
source /mnt/c/Users/a_tas/Desktop/MFMLAB/Personal_Sync/20-Coding/P032-群晖Drive按需同步/work/fsearch-perf-runs/env.sh
harness="$P032_WORK/issue7-validation"
python3 - <<'PY'
import json,pathlib,time
path=pathlib.Path('/home/drive-native-test/.local/state/drive-native-ultra-validation/g4-007/fresh-001/summary.json')
print('Waiting for existing fresh-001 to finish before copying its target.',flush=True)
while True:
    data=json.loads(path.read_text())
    if data.get('complete'):break
    if data.get('failure_type'):raise SystemExit('Previous run ended unexpectedly; inspect originals before continuing')
    time.sleep(5)
assert not data['changed_source_files'] and not data['residual_owned_mounts']
PY
exec python3 "$harness/run-g4.py" --archives "$harness/prepared-g4-008" \
  --run /home/drive-native-test/.local/state/drive-native-ultra-validation/g4-008 \
  --candidate-commit 99aa587e83fc409883ecd3e0d3f60fe8640b2e06 \
  --copy-target /home/drive-native-test/.local/state/drive-native-ultra-validation/g4-007/target \
  --nautilus-private-prefix /home/drive-native-test/.local/state/drive-native-ultra-validation/g4-002/nautilus-private/prefix \
  --g4-authorized
