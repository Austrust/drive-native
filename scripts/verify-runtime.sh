#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
test_log=$(mktemp /tmp/drive-native-runtime-tests.XXXXXX)
set +e
cargo test --workspace --all-features --locked 2>&1 | tee "$test_log"
test_status=${PIPESTATUS[0]}
set -e
python3 - "$test_log" evidence/runtime-tests.log <<'PY'
from pathlib import Path
import os,sys,tempfile
source,target=map(Path,sys.argv[1:])
data=source.read_bytes()
if b'\0' in data:
    raise SystemExit('NUL in captured test log; preserve temporary input for diagnosis')
with tempfile.NamedTemporaryFile(dir=target.parent,prefix='.complete-log-',delete=False) as f:
    temporary=Path(f.name);f.write(data);f.flush();os.fsync(f.fileno())
os.replace(temporary,target)
if target.read_bytes()!=data:
    raise SystemExit('Archived log failed readback verification')
source.unlink()
PY
exit "$test_status"
