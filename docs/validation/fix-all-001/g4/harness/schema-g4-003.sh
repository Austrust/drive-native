#!/usr/bin/env bash
set -euo pipefail
source /mnt/c/Users/a_tas/Desktop/MFMLAB/Personal_Sync/20-Coding/P032-群晖Drive按需同步/work/fsearch-perf-runs/env.sh
harness="$P032_WORK/ultra-benchmark"
run=/home/drive-native-test/.local/state/drive-native-ultra-validation/schema-g4-003
old=/home/drive-native-test/.local/state/drive-native-ultra-validation/schema-g4-001
source_path=/home/drive-native-test/.local/state/drive-native-ultra-validation/g4-003/source
test ! -e "$run"
mkdir -m700 -p "$run"/{probe25/src,logs,bins,state}
cp "$harness/schema-probe.rs" "$run/probe25/src/main.rs"
python3 - "$run" "$old" "$source_path" <<'PY'
import hashlib,json,pathlib,shutil,sys
run,old,source=map(pathlib.Path,sys.argv[1:])
assert json.loads((source.parent/'source.json').read_text())['candidate_commit']=='c744729141ca8b23bde63c61ca8a644bdfdaee81'
for version in (23,24):shutil.copy2(old/'bins'/f'schema{version}',run/'bins'/f'schema{version}')
shutil.copytree(old/'target',run/'target',symlinks=True)
(run/'probe25/Cargo.toml').write_text('[package]\nname="schema-probe-25"\nversion="0.0.0"\nedition="2024"\n[dependencies]\ndrive-core={path="'+str(source/'crates/drive-core')+'"}\n')
(run/'provenance.json').write_text(json.dumps({'candidate_commit':'c744729141ca8b23bde63c61ca8a644bdfdaee81','old_readers_copied_from':str(old/'bins'),'old_reader_sha256':{str(v):hashlib.sha256((run/'bins'/f'schema{v}').read_bytes()).hexdigest() for v in (23,24)},'old_schema23_source':'d555944bfe3058cb223bf7d8ebc71244f1e4d7a3','old_schema24_source':'ed1fabf91ec54bc1ba09d3537128b7d229a12936'},indent=2)+'\n')
PY
export CARGO_TARGET_DIR="$run/target" CARGO_NET_OFFLINE=true CARGO_BUILD_JOBS=2
unset RUSTFLAGS CARGO_ENCODED_RUSTFLAGS
cargo generate-lockfile --offline --manifest-path "$run/probe25/Cargo.toml" > "$run/logs/lock25.log" 2>&1
cargo build --offline --locked --manifest-path "$run/probe25/Cargo.toml" > "$run/logs/build25.log" 2>&1
cp "$run/target/debug/schema-probe-25" "$run/bins/schema25"
python3 "$harness/verify-schema.py" "$run"
