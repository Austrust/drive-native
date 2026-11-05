#!/usr/bin/env bash
set -euo pipefail
source /mnt/c/Users/a_tas/Desktop/MFMLAB/Personal_Sync/20-Coding/P032-群晖Drive按需同步/work/fsearch-perf-runs/env.sh
export CARGO_INCREMENTAL=0
unset RUSTFLAGS CARGO_ENCODED_RUSTFLAGS
harness="$P032_WORK/issue7-validation"
run=/home/drive-native-test/.local/state/drive-native-ultra-validation/g4-011
python3 -B - <<'PY'
import json,pathlib
run=pathlib.Path('/home/drive-native-test/.local/state/drive-native-ultra-validation/g4-011')
summary=json.loads((run/'summary.json').read_text())
assert summary['complete'] and summary['candidate_commit']=='41365a650eb7d7df696046e21b7671697231f851'
assert not summary['changed_source_files'] and not summary['residual_owned_mounts']
PY
exec python3 -B "$harness/native-worker-chain/build.py" \
  --source "$run/source" --candidate 41365a650eb7d7df696046e21b7671697231f851 \
  --target "$run/native-worker-chain-target" --output "$run/native-worker-chain-build-001" \
  --host-space-helper "$harness/prepared-g4-011/run-g4.py"
