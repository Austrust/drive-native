#!/usr/bin/env bash
set -euo pipefail
source /mnt/c/Users/a_tas/Desktop/MFMLAB/Personal_Sync/20-Coding/P032-群晖Drive按需同步/work/fsearch-perf-runs/env.sh
harness="$P032_WORK/ultra-benchmark"
run=/home/drive-native-test/.local/state/drive-native-ultra-validation/schema-g4-001
test ! -e "$run"
mkdir -m700 -p "$run"/{source23,source24,probe23/src,probe24/src,probe25/src,logs,bins,state}
tar -xf "$harness/prepared-g4-001/baseline.tar" -C "$run/source23"
tar -xf "$harness/prepared-g4-001/history24.tar" -C "$run/source24"
export CARGO_TARGET_DIR="$run/target" CARGO_NET_OFFLINE=true CARGO_BUILD_JOBS=2
unset RUSTFLAGS CARGO_ENCODED_RUSTFLAGS
for schema in 23 24 25; do
    if test "$schema" = 25; then
        source_path=/home/drive-native-test/.local/state/drive-native-ultra-validation/g4-001/source
    else
        source_path="$run/source$schema"
    fi
    cp "$harness/schema-probe.rs" "$run/probe$schema/src/main.rs"
    python3 - "$run/probe$schema/Cargo.toml" "$source_path" "$schema" <<'PY'
import pathlib,sys
dest,source,schema=sys.argv[1:]
pathlib.Path(dest).write_text('[package]\nname="schema-probe-'+schema+'"\nversion="0.0.0"\nedition="2024"\n[dependencies]\ndrive-core={path="'+source+'/crates/drive-core"}\n')
PY
    cargo generate-lockfile --offline --manifest-path "$run/probe$schema/Cargo.toml" > "$run/logs/lock$schema.log" 2>&1
    cargo build --offline --locked --manifest-path "$run/probe$schema/Cargo.toml" > "$run/logs/build$schema.log" 2>&1
    cp "$run/target/debug/schema-probe-$schema" "$run/bins/schema$schema"
done
python3 "$harness/verify-schema.py" "$run"
