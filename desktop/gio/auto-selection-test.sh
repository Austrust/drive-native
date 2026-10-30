#!/usr/bin/env bash
set -euo pipefail
source_dir=$(CDPATH='' cd -- "$(dirname -- "$0")" && pwd)
modules=${1:?Supply registry-mode modules}
test_modules=${2:?Supply test-scope modules}
output=${3:?Supply scratch output}
test "$(cat "$modules/scope-mode.txt")" = registry
test "$(cat "$test_modules/scope-mode.txt")" = test
mkdir -p -- "$output"
dbus-run-session -- env -u GIO_USE_VFS GIO_EXTRA_MODULES="$modules" \
  timeout 20 /usr/bin/python3 "$source_dir/auto-selection-test.py" --expect automatic | tee "$output/automatic.json"
dbus-run-session -- env GIO_USE_VFS=local GIO_EXTRA_MODULES="$modules" \
  timeout 20 /usr/bin/python3 "$source_dir/auto-selection-test.py" --expect local | tee "$output/explicit-local.json"
dbus-run-session -- env -u GIO_USE_VFS GIO_EXTRA_MODULES="$test_modules" \
  timeout 20 /usr/bin/python3 "$source_dir/auto-selection-test.py" --expect system | tee "$output/test-mode-not-automatic.json"
dbus-run-session -- env -u GIO_USE_VFS GIO_EXTRA_MODULES="$modules" \
  timeout 25 /usr/bin/python3 "$source_dir/gvfs_test.py" --ordinary-only | tee "$output/ordinary-gvfs.json"
dbus-run-session -- env -u GIO_USE_VFS GIO_EXTRA_MODULES="$modules" \
  timeout 35 /usr/bin/python3 "$source_dir/backend_io_test.py" --report "$output/backend-io.json" | tee "$output/backend-io.log"
