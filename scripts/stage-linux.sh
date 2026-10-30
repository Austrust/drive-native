#!/usr/bin/env bash
set -euo pipefail
build=${1:?Supply absolute build output}
stage=${2:?Supply absolute package staging directory}
case "$build:$stage" in /*:/*) ;; *) exit 2;; esac
test "$(cat "$build/gio/scope-mode.txt")" = registry
project=$(CDPATH='' cd -- "$(dirname -- "$0")/.." && pwd)
cd "$project"
install -d "$stage/usr/lib/drive-native" "$stage/usr/bin"
install -m755 "$build"/native/drive-native-* "$stage/usr/lib/drive-native/"
cp -a "$build/gui-dist/drive-native-gui" "$stage/usr/lib/drive-native/gui"
printf '#!/bin/sh
exec /usr/lib/drive-native/gui/drive-native-gui "$@"
' > "$stage/usr/bin/drive-native"
chmod 755 "$stage/usr/bin/drive-native"
gio_dir=$(pkg-config --variable=giomoduledir gio-2.0)
test "$gio_dir" = /usr/lib/x86_64-linux-gnu/gio/modules
install -Dm755 "$build/gio/libdrive-native-probe.so" "$stage$gio_dir/libdrive-native-probe.so"
install -Dm644 desktop/nemo/drive_native_nemo.py "$stage/usr/share/nemo-python/extensions/drive_native_nemo.py"
install -d "$stage/usr/share/icons/hicolor/scalable/emblems"
install -m644 desktop/nemo/icons/*.svg "$stage/usr/share/icons/hicolor/scalable/emblems/"
install -Dm644 desktop/nemo/icons/drive-native-sync.svg "$stage/usr/share/icons/hicolor/scalable/apps/drive-native.svg"
install -Dm644 desktop/drive-native.desktop "$stage/usr/share/applications/drive-native.desktop"
install -Dm644 desktop/drive-native.service "$stage/usr/lib/systemd/user/drive-native.service"
install -Dm644 desktop/drive-native-unlock.service "$stage/usr/lib/systemd/user/drive-native-unlock.service"
doc="$stage/usr/share/doc/drive-native"
install -d "$doc"
install -m644 LICENSE DELIVERY_GATES.md "$doc/"
install -m644 crates/sproto/LICENSE "$doc/sproto-LICENSE"
install -m644 "$build/gui-dependencies.txt" scripts/gui-requirements.lock "$doc/"
# No giomodule.cache is shipped: maintainer scripts update the shared cache.

install -Dm644 desktop/drive-native@.service "$stage/usr/lib/systemd/user/drive-native@.service"
install -Dm644 desktop/drive-native-unlock@.service "$stage/usr/lib/systemd/user/drive-native-unlock@.service"
