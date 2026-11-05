#!/usr/bin/env bash
set -euo pipefail
source /mnt/c/Users/a_tas/Desktop/MFMLAB/Personal_Sync/20-Coding/P032-群晖Drive按需同步/work/fsearch-perf-runs/env.sh
export P032_CHAIN_HARNESS="$P032_WORK/issue7-validation"
exec python3 -B - <<'PY'
import json,os,pathlib,subprocess
run=pathlib.Path('/home/drive-native-test/.local/state/drive-native-ultra-validation/g4-011')
h=pathlib.Path(os.environ['P032_CHAIN_HARNESS'])
build=json.loads((run/'native-worker-chain-build-001/build.json').read_text())
assert build['passed'] and build['candidate_commit']=='41365a650eb7d7df696046e21b7671697231f851'
argv=['python3','-B',str(h/'native-worker-chain/run.py'),'--binary',build['binary']['path'],
 '--worker',str(run/'target/debug/drive-native-worker'),'--crate',build['crate'],
 '--candidate',build['candidate_commit'],'--output',str(run/'native-worker-chain-other-001'),
 '--host-space-helper',str(h/'prepared-g4-011/run-g4.py')]
for name in ['bridge_profile_false_empty','bridge_profile_true_empty','bridge_profile_true_other_view',
 'bridge_late_read_after_profile','worker_rpc_generic_3004','worker_rpc_generic_8004','worker_rpc_generic_9005','worker_rpc_generic_a005']:
 argv.extend(['--case',name])
raise SystemExit(subprocess.run(argv).returncode)
PY
