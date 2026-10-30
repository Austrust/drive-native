#!/usr/bin/env bash
set -euo pipefail
source_dir=$(CDPATH='' cd -- "$(dirname -- "$0")" && pwd)
modules=${1:?Supply an absolute built module directory}
output=${2:?Supply an isolated output directory}
test -f "$modules/libdrive-native-probe.so"
grep -Fxq 'libdrive-native-probe.so: gio-vfs' "$modules/giomodule.cache"
mkdir -p -- "$output"
dbus-run-session -- env GIO_USE_VFS=drive-native-probe GIO_EXTRA_MODULES="$modules" \
    timeout 25 /usr/bin/python3 "$source_dir/gvfs_test.py" | tee "$output/gvfs.json"
dbus-run-session -- env GIO_USE_VFS=drive-native-probe GIO_EXTRA_MODULES="$modules" \
    timeout 35 /usr/bin/python3 "$source_dir/backend_io_test.py" --report "$output/backend-io.json" | tee "$output/backend-io.log"
# Missing module cache is a rejected activation, never an adapter acceptance.
uncached=$(mktemp -d /tmp/drive-native-gvfs-cache.XXXXXX)
trap 'rm -f -- "$uncached/libdrive-native-probe.so"; rmdir -- "$uncached"' EXIT
cp -- "$modules/libdrive-native-probe.so" "$uncached/libdrive-native-probe.so"
dbus-run-session -- env GIO_USE_VFS=drive-native-probe GIO_EXTRA_MODULES="$uncached" \
    timeout 25 /usr/bin/python3 "$source_dir/gvfs_test.py" --expect-system-fallback | tee "$output/missing-cache.json"
