#!/usr/bin/env bash
set -euo pipefail
source_dir=$(CDPATH='' cd -- "$(dirname -- "$0")" && pwd)
modules=${1:?Supply the built isolated module directory}
output=${2:?Supply an isolated test output directory}
mkdir -p -- "$output"
root=$(mktemp -d /tmp/drive-native-gio-probe.XXXXXX)
trap 'rm -f -- "$root/A.doc"; rmdir -- "$root"' EXIT
printf 'unknown doc content' > "$root/A.doc"
export GIO_EXTRA_MODULES="$modules"
export GIO_USE_VFS=drive-native-probe
export DRIVE_NATIVE_METADATA_ROOT="$root"
strace -f -e trace=openat -o "$output/open.trace" /usr/bin/python3 "$source_dir/local_test.py" | tee "$output/local.json"
if grep -F "$root/A.doc" "$output/open.trace"; then exit 1; fi
gio cat "$root/A.doc" > "$output/read.txt"
cmp "$root/A.doc" "$output/read.txt"
printf '{"metadata_body_opens":0,"explicit_read_matches":true}\n' > "$output/io-check.json"
