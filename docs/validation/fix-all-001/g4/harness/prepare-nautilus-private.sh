#!/usr/bin/env bash
set -euo pipefail
run=/home/drive-native-test/.local/state/drive-native-ultra-validation/g4-002
deps="$run/nautilus-private"
test ! -e "$deps"
mkdir -m700 "$deps"
mkdir "$deps/debs" "$deps/prefix"
cd "$deps/debs"
# Download/extract only. No system install, maintainer scripts or privileged apt.
apt-get download 'gir1.2-nautilus-4.0=1:46.4-0ubuntu0.2' 'libnautilus-extension4=1:46.4-0ubuntu0.2' > "$deps/download.log" 2>&1
for package in ./*.deb; do dpkg-deb -x "$package" "$deps/prefix"; done
sha256sum ./*.deb > "$deps/packages.sha256"
export GI_TYPELIB_PATH="$deps/prefix/usr/lib/x86_64-linux-gnu/girepository-1.0"
export LD_LIBRARY_PATH="$deps/prefix/usr/lib/x86_64-linux-gnu"
/usr/bin/python3 - <<'PY' > "$deps/import.log" 2>&1
import gi
gi.require_version('Nautilus','4.0')
from gi.repository import Nautilus
print('Nautilus 4.0 available from private prefix')
PY
