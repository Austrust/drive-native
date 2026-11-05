#!/usr/bin/env bash
# Prepared only. Start after the coordinator confirms host space and authorization.
set -euo pipefail
source /mnt/c/Users/a_tas/Desktop/MFMLAB/Personal_Sync/20-Coding/P032-群晖Drive按需同步/work/fsearch-perf-runs/env.sh
harness="$P032_WORK/issue7-validation"
test ! -e "$harness/prepared-g4-011/launcher.log"
export CARGO_INCREMENTAL=0
python3 -B - <<'PY'
import json,pathlib
path=pathlib.Path('/home/drive-native-test/.local/state/drive-native-ultra-validation/g4-009/summary.json')
data=json.loads(path.read_text())
assert data['complete'] and not data['changed_source_files'] and not data['residual_owned_mounts']
PY
# Keep even pre-summary preparation errors outside the guest filesystem.
python3 -B "$harness/run-g4.py" --archives "$harness/prepared-g4-011" \
  --run /home/drive-native-test/.local/state/drive-native-ultra-validation/g4-011 \
  --candidate-commit 41365a650eb7d7df696046e21b7671697231f851 \
  --registry-seed /home/drive-native-test/.local/state/drive-native-ultra-validation/g4-009/target \
  --nautilus-private-prefix /home/drive-native-test/.local/state/drive-native-ultra-validation/g4-002/nautilus-private/prefix \
  --g4-authorized 2>&1 | tee "$harness/prepared-g4-011/launcher.log"
