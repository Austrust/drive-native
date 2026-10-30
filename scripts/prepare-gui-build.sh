#!/bin/sh
# Build-time only. The eventual .deb must never run pip at installation time.
set -eu
build_env=${1:?usage: prepare-gui-build.sh /absolute/path/to/build-venv}
case "$build_env" in /*) ;; *) printf '%s\n' 'Build environment path must be absolute' >&2; exit 2;; esac
script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
python3 -m venv "$build_env"
"$build_env/bin/python" -m pip install --requirement "$script_dir/gui-requirements.lock"
"$build_env/bin/python" -c 'from PySide6 import QtCore, QtWidgets, QtDBus; print("Qt", QtCore.qVersion(), "Widgets / D-Bus ready")'
