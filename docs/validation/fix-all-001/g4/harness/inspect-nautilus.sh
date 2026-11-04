#!/usr/bin/env bash
set -u
for path in /usr/lib/girepository-1.0 /usr/lib/x86_64-linux-gnu/girepository-1.0 /usr/lib/x86_64-linux-gnu /var/cache/apt/archives /opt/drive-native-build; do
    if test -d "$path"; then
        rg --files "$path" -g '*Nautilus*' -g '*nautilus*' 2>/dev/null || true
    fi
done
dpkg-query -W 'gir1.2-nautilus*' 'libnautilus-extension*' 2>/dev/null || true
