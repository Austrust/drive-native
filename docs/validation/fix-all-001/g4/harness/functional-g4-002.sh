#!/usr/bin/env bash
set -euo pipefail
source /mnt/c/Users/a_tas/Desktop/MFMLAB/Personal_Sync/20-Coding/P032-群晖Drive按需同步/work/fsearch-perf-runs/env.sh
harness="$P032_WORK/ultra-benchmark"
run=/home/drive-native-test/.local/state/drive-native-ultra-validation/g4-002
python3 - "$run/summary.json" <<'PY'
import json,pathlib,sys
assert json.loads(pathlib.Path(sys.argv[1]).read_text())['complete'],'main matrix must complete first'
PY
export CARGO_TARGET_DIR="$run/target" CARGO_NET_OFFLINE=true CARGO_BUILD_JOBS=2
export TMPDIR="$run/tmp" DRIVE_NATIVE_FS_INDEX=1 DRIVE_NATIVE_FS_TTL_MS=1000
unset RUSTFLAGS CARGO_ENCODED_RUSTFLAGS
mkdir "$run/functional-bin"
cd "$run/source"
cargo rustc --offline --locked -p drive-fs --example fsearch_fixture -- --cfg drive_fs_diagnostics > "$run/logs/functional-fixture-build.log" 2>&1
cp "$CARGO_TARGET_DIR/debug/examples/fsearch_fixture" "$run/functional-bin/fsearch_fixture-diagnostic"
python3 "$harness/verify-fsearch-functional.py" --g4-functional-authorized \
  --candidate-commit 652eabe741396c8332a7182599aaf4e53d5af48b \
  --run /home/drive-native-test/.local/state/drive-native-ultra-validation/functional-g4-002 \
  --fixture "$run/functional-bin/fsearch_fixture-diagnostic" \
  --fsearch /home/drive-native-test/fsearch-bench/bin/fsearch-0.3.1-canonical \
  --audit /home/drive-native-test/fsearch-bench/bin/fsearch-index-audit > "$run/logs/functional-fsearch.log" 2>&1
