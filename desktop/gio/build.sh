#!/usr/bin/env bash
set -euo pipefail
source_dir=$(CDPATH='' cd -- "$(dirname -- "$0")" && pwd)
output=${1:?Supply an isolated module output directory}
mkdir -p -- "$output"
extra=()
mode=registry
case "${2:-}" in
    "") ;;
    --test-scopes) extra=(-DDRIVE_NATIVE_GIO_TEST_SCOPES=1); mode=test ;;
    *) printf 'Unknown build option\n' >&2; exit 2 ;;
esac
cc -shared -fPIC -Wall -Wextra -Werror "${extra[@]}" -o "$output/libdrive-native-probe.so" "$source_dir/metadata-probe.c" "$source_dir/mount-registry.c" $(pkg-config --cflags --libs gio-2.0)
gio-querymodules "$output"
grep -Fxq "libdrive-native-probe.so: gio-vfs" "$output/giomodule.cache"
printf '%s\n' "$mode" > "$output/scope-mode.txt"
