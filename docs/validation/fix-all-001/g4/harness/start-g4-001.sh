#!/usr/bin/env bash
set -euo pipefail
source /mnt/c/Users/a_tas/Desktop/MFMLAB/Personal_Sync/20-Coding/P032-群晖Drive按需同步/work/fsearch-perf-runs/env.sh
harness="$P032_WORK/ultra-benchmark"
python3 - "$harness/run-g4.py" <<'PY'
import ast,pathlib,sys
ast.parse(pathlib.Path(sys.argv[1]).read_text())
print('G4 harness syntax parsed')
PY
exec python3 "$harness/run-g4.py" --archives "$harness/prepared-g4-001" \
  --run /home/drive-native-test/.local/state/drive-native-ultra-validation/g4-001 \
  --candidate-commit cb22500970c139f104dc8fa9b6760dc8235c34e2 \
  --copy-target /home/drive-native-test/.cache/ultra-integration-target --g4-authorized
