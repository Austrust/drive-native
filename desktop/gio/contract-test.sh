#!/usr/bin/env bash
set -euo pipefail
source_dir=$(CDPATH='' cd -- "$(dirname -- "$0")" && pwd)
modules=${1:?Supply the built isolated module directory}
output=${2:?Supply an isolated test output directory}
mkdir -p -- "$output"
export GIO_MODULE_DIR="$modules"
bash "$source_dir/local-test.sh" "$modules" "$output"
export GIO_EXTRA_MODULES="$modules"
export GIO_USE_VFS=drive-native-probe
G_DEBUG=fatal-criticals /usr/bin/python3 "$source_dir/enumerator_lifecycle_test.py" | tee "$output/enumerator-lifecycle.json"
/usr/bin/python3 "$source_dir/write_test.py" | tee "$output/writes.json"
strace -f -e trace=openat -o "$output/monitor-open.trace" /usr/bin/python3 "$source_dir/monitor_test.py" | tee "$output/monitor.json"
/usr/bin/python3 - "$output" <<'PY'
import json,pathlib,sys
out=pathlib.Path(sys.argv[1])
rows=(out/'monitor-open.trace').read_text().splitlines()
reads=[r for r in rows if 'drive-native-gio-monitor-' in r and '.doc"' in r and ('O_RDONLY' in r or 'O_RDWR' in r)]
assert not reads, reads
(out/'monitor-io.json').write_text(json.dumps({'metadata_body_opens':len(reads),'passed':True})+'\n')
print('PASS monitor metadata queries opened zero document bodies')
PY
