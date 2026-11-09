#!/usr/bin/env bash
set -euo pipefail
: "${GUI_PYTHON:?Set GUI_PYTHON to the locked build venv python}"
project=$(CDPATH='' cd -- "$(dirname -- "$0")/.." && pwd)
test_root=$(mktemp -d /tmp/drive-native-gui-tests.XXXXXXXX)
trap 'rm -rf -- "$test_root"' EXIT
export XDG_CONFIG_HOME="$test_root/config" XDG_DATA_HOME="$test_root/data"
export XDG_CACHE_HOME="$test_root/cache" XDG_RUNTIME_DIR="$test_root/runtime"
mkdir -p "$XDG_CONFIG_HOME" "$XDG_DATA_HOME" "$XDG_CACHE_HOME" "$XDG_RUNTIME_DIR"
chmod 700 "$XDG_RUNTIME_DIR"
export PYTHONDONTWRITEBYTECODE=1 QT_QPA_PLATFORM=offscreen
export PYTHONPATH="$project/desktop"
# Each module owns its QApplication and runs in a separate process. These are
# synthetic tests; installed desktop/NAS probes remain explicit release gates.
run_module() {
    local module=$1
    printf 'GUI regression: %s\n' "$module"
    if [ "$module" = gui_instance_test ] && [ -z "${DBUS_SESSION_BUS_ADDRESS:-}" ] \
            && command -v dbus-run-session >/dev/null 2>&1; then
        # The activation test needs a real session bus; skip only when none can run.
        dbus-run-session -- env DRIVE_NATIVE_GUI_REQUIRE_BUS=1 \
            "$GUI_PYTHON" "$project/desktop/tests/$module.py"
    else
        "$GUI_PYTHON" "$project/desktop/tests/$module.py"
    fi
}

for module in gui_unit_tests cache_gui_tests conflict_gui_tests hidden_dialog_tests \
    lazy_gui_tests multi_mount_tests trust_gui_tests transfer_gui_tests theme_tests \
    selector_gui_tests backend_upgrade_test pause_handover_test initialization_gui_tests \
    unified_manager_tests gui_instance_test queue_gui_tests history_gui_tests \
    permission_gui_tests background_banner_tests notification_tests \
    shutdown_diagnostic_tests keyboard_gui_tests window_transfer_tests \
    release_gui_tests window_state_tests; do
    run_module "$module"
done
