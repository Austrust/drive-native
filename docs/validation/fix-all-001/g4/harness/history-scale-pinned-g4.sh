#!/usr/bin/env bash
set -euo pipefail
run_id=${1:?g4-NNN}
candidate=${2:?full frozen commit}
source /mnt/c/Users/a_tas/Desktop/MFMLAB/Personal_Sync/20-Coding/P032-群晖Drive按需同步/work/fsearch-perf-runs/env.sh
harness="$P032_WORK/ultra-benchmark"
run="/home/drive-native-test/.local/state/drive-native-ultra-validation/$run_id"
label=${3:-history-scale-verified}
[[ "$label" =~ ^history-scale-[a-z0-9-]+$ ]]
root="$run/$label"
python3 - "$run" "$candidate" <<'PY'
import json,pathlib,re,sys
run=pathlib.Path(sys.argv[1]);commit=sys.argv[2]
assert re.fullmatch(r'g4-\d{3}',run.name)
summary=json.loads((run/'summary.json').read_text())
assert summary['complete'] and summary['candidate_commit']==commit
PY
test ! -e "$root"
mkdir -m700 -p "$root"/{probe/src,config,data,cache,runtime}
cp "$harness/history-ipc-server.rs" "$root/probe/src/main.rs"
cp "$harness/verify-history-scale.py" "$root/verify-history-scale.py"
python3 - "$root" "$run/source" "$candidate" <<'PY'
import hashlib,json,pathlib,sys
root,source=map(pathlib.Path,sys.argv[1:3])
manifest='''[package]
name="history-ipc-probe"
version="0.0.0"
edition="2024"
[dependencies]
drive-core={path="SOURCE/crates/drive-core",features=["native"]}
drive-runtime={path="SOURCE/crates/drive-runtime"}
tokio={version="1",features=["rt-multi-thread","macros","signal","time"]}
zbus={version="=6.1.0",default-features=false,features=["tokio","proxy","service"]}
[patch.crates-io]
fuser={path="SOURCE/vendor/fuser"}
'''.replace('SOURCE',str(source))
(root/'probe/Cargo.toml').write_text(manifest)
(root/'provenance.json').write_text(json.dumps({'candidate_commit':sys.argv[3],
 'source':str(source),'target':'own G4 run target; sequential after main matrix; never shared with root/G5',
 'server_source_sha256':hashlib.sha256((root/'probe/src/main.rs').read_bytes()).hexdigest(),
 'python_source_sha256':hashlib.sha256((root/'verify-history-scale.py').read_bytes()).hexdigest()},indent=2)+'\n')
PY
export CARGO_TARGET_DIR="$run/target" CARGO_NET_OFFLINE=true CARGO_BUILD_JOBS=2
unset RUSTFLAGS CARGO_ENCODED_RUSTFLAGS DBUS_SESSION_BUS_ADDRESS DISPLAY WAYLAND_DISPLAY
cp "$run/source/Cargo.lock" "$root/probe/Cargo.lock"
cargo metadata --offline --format-version 1 --manifest-path "$root/probe/Cargo.toml" > "$root/metadata.json" 2> "$root/lock.log"
python3 "$harness/verify-history-lock.py" "$root" "$run/source"
cargo build --offline --locked --manifest-path "$root/probe/Cargo.toml" > "$root/build.log" 2>&1
cp "$run/target/debug/history-ipc-probe" "$root/history-ipc-probe"
export PYTHONPATH="$run/source/desktop" PYTHONDONTWRITEBYTECODE=1 QT_QPA_PLATFORM=offscreen
export XDG_CONFIG_HOME="$root/config" XDG_DATA_HOME="$root/data" XDG_CACHE_HOME="$root/cache" XDG_RUNTIME_DIR="$root/runtime"
dbus-run-session -- /opt/drive-native-build/gui-venv/bin/python "$root/verify-history-scale.py" "$root" "$root/history-ipc-probe" > "$root/validation.log" 2>&1
python3 - "$root/results.json" <<'PY'
import json,pathlib,sys
s=json.loads(pathlib.Path(sys.argv[1]).read_text())
print(json.dumps({k:s[k] for k in ('passed','passed_count','failed_count','ignored_count')}))
PY
