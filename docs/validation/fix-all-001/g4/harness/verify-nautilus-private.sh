#!/usr/bin/env bash
set -euo pipefail
run=/home/drive-native-test/.local/state/drive-native-ultra-validation/g4-002
deps="$run/nautilus-private"
export GI_TYPELIB_PATH="$deps/prefix/usr/lib/x86_64-linux-gnu/girepository-1.0"
export LD_LIBRARY_PATH="$deps/prefix/usr/lib/x86_64-linux-gnu"
export DRIVE_NATIVE_TEST_FILEMANAGER=nautilus PYTHONPATH="$run/source/desktop" PYTHONDONTWRITEBYTECODE=1
export XDG_CONFIG_HOME="$deps/config" XDG_DATA_HOME="$deps/data" XDG_CACHE_HOME="$deps/cache" XDG_RUNTIME_DIR="$deps/runtime" TMPDIR="$run/tmp"
mkdir -m700 "$XDG_CONFIG_HOME" "$XDG_DATA_HOME" "$XDG_CACHE_HOME" "$XDG_RUNTIME_DIR"
unset DBUS_SESSION_BUS_ADDRESS DISPLAY WAYLAND_DISPLAY
set +e
dbus-run-session -- /usr/bin/python3 "$run/source/desktop/tests/filemanager_provider_tests.py" -v > "$deps/provider.log" 2>&1
result=$?
set -e
python3 - "$deps" "$result" <<'PY'
import hashlib,json,pathlib,re,sys
p=pathlib.Path(sys.argv[1]);code=int(sys.argv[2]);text=(p/'provider.log').read_text()
names=re.findall(r'^(test_\S+) \(([^)]+)\) \.\.\. (ok|FAIL|ERROR|skipped.*)$',text,re.M)
counts={status:sum(row[2]==status for row in names) for status in ('ok','FAIL','ERROR')}
result={'candidate_commit':'652eabe741396c8332a7182599aaf4e53d5af48b','exit_code':code,
        'passed':counts['ok'],'failed':counts['FAIL']+counts['ERROR'],'ignored':sum(row[2].startswith('skipped') for row in names),
        'tests':names,'log_sha256':hashlib.sha256((p/'provider.log').read_bytes()).hexdigest(),
        'dependency_mode':'Two Ubuntu packages downloaded and extracted to private prefix; system installation unchanged',
        'packages_sha256':(p/'packages.sha256').read_text(),'complete':code==0 and len(names)==10}
(p/'result.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result))
PY
exit "$result"
