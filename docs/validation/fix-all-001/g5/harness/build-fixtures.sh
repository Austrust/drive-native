#!/usr/bin/env bash
# Build only after the coordinator authorizes the post-G4 preparation window.
set -euo pipefail
test "${1:-}" = build-authorized
archives=${2:?prepared source directory}
build_root=${3:?new Linux build directory}
case "$build_root" in /home/drive-native-test/.local/state/drive-native-ultra-benchmark/build-*) ;; *) echo 'invalid build root' >&2; exit 2;; esac
test ! -e "$build_root"
source /mnt/c/Users/a_tas/Desktop/MFMLAB/Personal_Sync/20-Coding/P032-群晖Drive按需同步/work/fsearch-perf-runs/env.sh
unset RUSTFLAGS CARGO_ENCODED_RUSTFLAGS DRIVE_NATIVE_FS_DIAGNOSTICS DRIVE_NATIVE_FS_INDEX DRIVE_NATIVE_FS_TTL_MS
mkdir -p "$build_root"/{baseline,candidate,bin,logs}
python3 - "$archives" <<'PY'
import hashlib,json,pathlib,sys
p=pathlib.Path(sys.argv[1]);m=json.loads((p/'source.json').read_text())
for role in ('baseline','candidate'):
    h=hashlib.file_digest((p/f'{role}.tar').open('rb'),'sha256').hexdigest()
    assert h==m[f'{role}_archive_sha256'],role
assert m['baseline_commit'].startswith('d555944')
PY
tar -xf "$archives/baseline.tar" -C "$build_root/baseline"
tar -xf "$archives/candidate.tar" -C "$build_root/candidate"
cp "$build_root/candidate/crates/drive-fs/examples/fsearch_fixture.rs" "$build_root/baseline/crates/drive-fs/examples/fsearch_fixture.rs"
cp "$archives/source.json" "$build_root/source.json"
cargo --version > "$build_root/logs/toolchain.log"
rustc --version >> "$build_root/logs/toolchain.log"
for role in baseline candidate; do
    export CARGO_TARGET_DIR="$build_root/target-$role"
    cd "$build_root/$role"
    if test "$role" = baseline; then
        cargo rustc --release --locked --offline -j2 -p drive-fs --example fsearch_fixture -- --cfg drive_fs_baseline > "$build_root/logs/$role-ordinary.log" 2>&1
    else
        cargo build --release --locked --offline -j2 -p drive-fs --example fsearch_fixture > "$build_root/logs/$role-ordinary.log" 2>&1
    fi
    cp "$CARGO_TARGET_DIR/release/examples/fsearch_fixture" "$build_root/bin/$role"
    if test "$role" = baseline; then
        cargo rustc --release --locked --offline -j2 -p drive-fs --example fsearch_fixture -- --cfg drive_fs_baseline --cfg drive_fs_diagnostics > "$build_root/logs/$role-diagnostic.log" 2>&1
    else
        cargo rustc --release --locked --offline -j2 -p drive-fs --example fsearch_fixture -- --cfg drive_fs_diagnostics > "$build_root/logs/$role-diagnostic.log" 2>&1
    fi
    cp "$CARGO_TARGET_DIR/release/examples/fsearch_fixture" "$build_root/bin/$role-diagnostic"
done
python3 - "$build_root" <<'PY'
import hashlib,json,pathlib,sys,time
p=pathlib.Path(sys.argv[1]);m=json.loads((p/'source.json').read_text())
sha=lambda f:hashlib.file_digest(f.open('rb'),'sha256').hexdigest()
fixture='crates/drive-fs/examples/fsearch_fixture.rs'
assert sha(p/'baseline'/fixture)==sha(p/'candidate'/fixture)
m.update(build_complete=True,executed='offline fixture builds only; no tests or benchmark scans',built_at_utc_unix=time.time(),fixture_sha256=sha(p/'candidate'/fixture),
         binaries={name:{'path':str(p/'bin'/name),'sha256':sha(p/'bin'/name)}
                   for name in ('baseline','candidate','baseline-diagnostic','candidate-diagnostic')},
         logs={f.name:sha(f) for f in sorted((p/'logs').iterdir())},
         baseline_example_cfg='drive_fs_baseline',diagnostic_example_cfg='drive_fs_diagnostics',
         product_rustflags=None)
(p/'build.json').write_text(json.dumps(m,indent=2)+'\n')
print(json.dumps({'build_complete':True,'manifest':str(p/'build.json')}))
PY
