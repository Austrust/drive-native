"""Explicit WSLg test of the real Qt client against an existing authorized fixture.

No NAS writes: login, metadata, native downloads and local pin/pause controls only.
The fixture must have A.doc and Pinned/B.txt, Pinned/C.txt from native_service_roundtrip.
"""

import getpass, json, os, signal, sys, tempfile, time, resource, argparse
from pathlib import Path

resource.setrlimit(resource.RLIMIT_CORE, (0, 0))

parser = argparse.ArgumentParser()
parser.add_argument("scope")
parser.add_argument("screenshot")
parser.add_argument("report")
parser.add_argument("--provider-rounds", type=int, default=1)
parser.add_argument("--without-host", action="store_true")
parser.add_argument(
    "--read-retry", action="store_true", help="explicit mounted read retry scenario"
)
parser.add_argument(
    "--gio-probe-module", help="isolated research GIO module directory; not production acceptance"
)
options = parser.parse_args()
if not 1 <= options.provider_rounds <= 20:
    parser.error("provider rounds must be between 1 and 20")
if options.gio_probe_module:
    if options.without_host:
        parser.error("GIO probe requires the Nemo host test")
    module_dir = Path(options.gio_probe_module).resolve(strict=True)
    if not (module_dir / "libdrive-native-probe.so").is_file():
        parser.error("missing built GIO research module")
    if (
        not (module_dir / "giomodule.cache").is_file()
        or "libdrive-native-probe.so: gio-vfs"
        not in (module_dir / "giomodule.cache").read_text().splitlines()
    ):
        parser.error("missing GIO module cache; rebuild the isolated module")
scope = options.scope
parts = scope.split("/")
if (
    len(parts) != 4
    or parts[1] != "DriveNative-Lab"
    or not parts[2]
    or not parts[3].startswith("service-")
    or any(p in (".", "..") for p in parts)
):
    raise SystemExit("expected existing authorized service fixture")
password = getpass.getpass("Test account password (not saved): ")
scratch = tempfile.TemporaryDirectory(prefix="drive-native-gui-")
base = Path(scratch.name)
os.environ["XDG_CONFIG_HOME"] = str(base / "config")
os.environ["XDG_DATA_HOME"] = str(base / "data")
from PySide6.QtWidgets import QApplication, QMenu
from PySide6.QtCore import Qt, QTimer
from PySide6.QtTest import QTest
from drive_native_gui.window import MainWindow
from drive_native_gui.config import config_path

app = QApplication([])
window = MainWindow()
window.show()
report = {
    "passed": False,
    "scope": scope,
    "qt_platform": app.platformName(),
    "stage": "native_login",
    "secret_service_tested": False,
    "nemo_provider_tested": False,
    "nemo_window_tested": False,
}


def wait(predicate, label, timeout=90):
    deadline = time.monotonic() + timeout
    while not predicate():
        if time.monotonic() > deadline:
            window.grab().save(str(Path(sys.argv[2]).with_name("gui-failure.png")))
            print(
                json.dumps(
                    {
                        "folder": window.folder,
                        "rows": [r.get("path") for r in window.rows],
                        "loading": window.loading,
                        "banner": window.banner.text(),
                        "overview": window.overview,
                    },
                    ensure_ascii=False,
                ),
                flush=True,
            )
            raise AssertionError(f"{label}: {window.banner.text()}")
        QTest.qWait(50)


def row(path):
    for index in range(window.table.topLevelItemCount()):
        item = window.table.topLevelItem(index)
        if item.data(0, Qt.UserRole)["path"] == path:
            return item
    return None


def menu_action(item, index):
    window.table.clearSelection()
    item.setSelected(True)

    def trigger():
        popup = QApplication.activePopupWidget()
        assert isinstance(popup, QMenu)
        popup.actions()[index].trigger()
        popup.close()

    QTimer.singleShot(100, trigger)
    window.context_menu(window.table.visualItemRect(item).center())


try:
    wait(lambda: window.connection_value.text() == "尚未连接", "initial screen")
    QTest.mouseClick(window.configure, Qt.LeftButton)
    wizard = window.wizard
    wizard.host.setText("192.0.2.10")
    wizard.tls.setText("nas.example.invalid")
    wizard.username.setText("test-user")
    wizard.password.setText(password)
    QTest.mouseClick(wizard.next, Qt.LeftButton)
    wait(lambda: wizard.pages.currentIndex() == 1, "native login selector")
    root = wizard.tree.topLevelItem(0)
    root_identity_observed = bool(root.data(0, Qt.UserRole).get("scope"))
    selected = root
    for segment in parts[1:]:
        selected.setExpanded(True)
        wait(lambda item=selected: item.data(0, Qt.UserRole).get("loaded"), "folder expansion")
        found = [
            selected.child(i)
            for i in range(selected.childCount())
            if selected.child(i).text(0) == segment
        ]
        assert len(found) == 1, segment
        selected = found[0]
    wizard.tree.setCurrentItem(selected)
    wizard.mount.setText(str(base / "mount"))
    wizard.remember.setChecked(False)
    QTest.mouseClick(wizard.next, Qt.LeftButton)
    wait(
        lambda: window.overview.get("connection") == "online" and row("A.doc") is not None,
        "online file browser",
    )
    assert window.overview["downloads_started"] == 0
    assert row("A.doc").data(0, Qt.UserRole)["status"] == "online_only"
    queried = []
    window.bus.call(
        "GetStates", [["A.doc", "Pinned"]], lambda data, error: queried.append((data, error))
    )
    wait(lambda: bool(queried), "Qt typed string-array call")
    assert queried[0][1] is None and len(queried[0][0]["entries"]) == 2, queried[0][1]
    profile = json.loads(config_path().read_text())
    assert "password" not in profile and profile["scope"]["path"] == scope
    assert config_path().stat().st_mode & 0o077 == 0
    report.update(
        root_identity_observed=root_identity_observed,
        real_native_login_wizard=True,
        private_profile_without_password=True,
        metadata_browser_zero_downloads=True,
        stage="qt_actions",
    )
    print(
        "PASS real Qt login, native folder selection, private profile and metadata-only file list",
        flush=True,
    )
    folder = row("Pinned")
    point = window.table.visualItemRect(folder).center()
    QTest.mouseClick(window.table.viewport(), Qt.LeftButton, pos=point)
    QTest.qWait(100)
    QTest.mouseDClick(window.table.viewport(), Qt.LeftButton, pos=point)
    wait(
        lambda: window.folder == "Pinned" and row("Pinned/B.txt") is not None, "enter child folder"
    )
    assert window.overview["downloads_started"] == 0
    QTest.mouseClick(window.up, Qt.LeftButton)
    wait(lambda: window.folder == "" and row("Pinned") is not None, "back to root")
    menu_action(row("Pinned"), 0)
    wait(
        lambda: row("Pinned") and row("Pinned").data(0, Qt.UserRole)["status"] == "pinned_complete",
        "live pin status signal",
    )
    screenshot = Path(sys.argv[2])
    window.grab().save(str(screenshot))
    QTest.mouseClick(window.pause, Qt.LeftButton)
    wait(lambda: window.overview.get("connection") == "paused", "pause from Qt")
    assert (base / "mount/Pinned/B.txt").read_bytes() == b"pinned-v1"
    QTest.mouseClick(window.pause, Qt.LeftButton)
    wait(lambda: window.overview.get("connection") == "online", "resume from Qt")
    menu_action(row("Pinned"), 1)
    wait(
        lambda: row("Pinned") and row("Pinned").data(0, Qt.UserRole)["status"] == "local_complete",
        "unpin retains complete content",
    )
    menu_action(row("Pinned"), 2)
    wait(
        lambda: row("Pinned") and row("Pinned").data(0, Qt.UserRole)["status"] == "online_only",
        "release from context menu",
    )
    print(
        "PASS live file status refresh, pin/unpin/release menus and pause/resume buttons",
        flush=True,
    )
    report.update(
        dbus_state_signal_refresh=True,
        pin_unpin_release_menus=True,
        pause_resume_buttons=True,
        stage="nemo_provider",
    )
    from PySide6.QtCore import QProcess

    report["test_scope"] = "gui_provider" if options.without_host else "full_desktop"
    if options.gio_probe_module:
        report["test_scope"] = "experimental_gio_nemo_host"
    if options.read_retry:
        reader = QProcess()
        reader.start(
            "/usr/bin/python3", [str(Path(__file__).with_name("mounted_read_retry_test.py"))]
        )
        wait(
            lambda: reader.state() == QProcess.NotRunning,
            "native mounted foreground read retry",
            60,
        )
        if reader.exitCode() != 0:
            raise AssertionError(
                "mounted read retry failed: " + bytes(reader.readAllStandardError()).decode("utf8")
            )
        retry_result = json.loads(bytes(reader.readAllStandardOutput()).decode("utf8"))
        assert retry_result["passed"]
        Path(options.report).with_name("mounted-read-retry.json").write_text(
            json.dumps(retry_result, indent=2) + "\n", encoding="utf8"
        )
        report["foreground_read_retry_tested"] = True
        print("PASS native mounted foreground retry and shared concurrent readers", flush=True)
    report["provider_rounds_completed"] = 0
    for round_index in range(options.provider_rounds):
        helper = QProcess()
        helper.start(
            "/usr/bin/python3",
            [
                str(Path(__file__).with_name("nemo_provider_test.py")),
                str(
                    Path(__import__("drive_native_gui").__file__).parent.parent
                    / "nemo/drive_native_nemo.py"
                ),
            ],
        )
        wait(lambda: helper.state() == QProcess.NotRunning, "Nemo provider integration")
        provider_output = bytes(helper.readAllStandardOutput()).decode("utf8")
        provider_errors = bytes(helper.readAllStandardError()).decode("utf8")
        if helper.exitCode() != 0:
            raise AssertionError("Nemo provider test failed: " + provider_errors)
        nemo_result = json.loads(provider_output)
        assert nemo_result["passed"]
        Path(sys.argv[3]).with_name("nemo-provider-roundtrip.json").write_text(
            json.dumps(nemo_result, indent=2) + "\n", encoding="utf8"
        )
        print(
            "PASS actual GI provider, live emblems and Nemo pin/unpin/release menu actions",
            flush=True,
        )
        report.update(nemo_provider_tested=True, stage="nemo_host")
        report["provider_rounds_completed"] = round_index + 1
        print(f"PASS provider round {round_index+1}", flush=True)
    if not options.without_host:
        host_helper = QProcess()
        if options.gio_probe_module:
            from PySide6.QtCore import QProcessEnvironment

            host_env = QProcessEnvironment.systemEnvironment()
            host_env.insert("DRIVE_NATIVE_TEST_GIO_MODULES", str(module_dir))
            host_helper.setProcessEnvironment(host_env)
        host_helper.start(
            "/usr/bin/python3",
            [
                str(Path(__file__).with_name("nemo_host_test.py")),
                str(
                    Path(__import__("drive_native_gui").__file__).parent.parent
                    / "nemo/drive_native_nemo.py"
                ),
                str(Path(sys.argv[3]).with_name("nemo-host-observation.json")),
            ],
        )
        wait(lambda: host_helper.state() == QProcess.NotRunning, "Nemo real window integration")
        host_output = bytes(host_helper.readAllStandardOutput()).decode("utf8")
        host_errors = bytes(host_helper.readAllStandardError()).decode("utf8")
        if host_helper.exitCode() != 0:
            raise AssertionError("Nemo window test failed: " + host_errors)
        host_result = json.loads(host_output)
        assert host_result["passed"]
        Path(sys.argv[3]).with_name("nemo-host-roundtrip.json").write_text(
            json.dumps(host_result, indent=2) + "\n", encoding="utf8"
        )
        print("PASS actual Nemo window, status column, fixture browse and clean exit", flush=True)
    window.close()
    QTest.qWait(200)
    result = []
    window.bus.call("GetOverview", callback=lambda data, error: result.append((data, error)))
    wait(lambda: bool(result), "service after GUI closed", 10)
    assert result[0][1] is None and result[0][0]["mounted"]
    report["download_diagnostics"] = result[0][0].get("download_diagnostics")
    report["download_attempts"] = result[0][0]["downloads_started"]
    report["outstanding_download_errors"] = result[0][0].get("download_errors")
    report.update(
        gui_exit_retains_mount=True, nemo_window_tested=not options.without_host, stage="stop_mount"
    )
    Path(sys.argv[3]).write_text(
        json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf8"
    )
    result = []
    window.bus.call("StopMount", callback=lambda data, error: result.append((data, error)))
    wait(lambda: bool(result), "explicit stop mount", 10)
    QTest.qWait(600)
    report.update(passed=True, stage="complete")
    print("PASS closing GUI retains service; explicit cleanup completed", flush=True)
finally:
    Path(sys.argv[3]).write_text(
        json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf8"
    )
    # Only processes started with this exact private temporary profile are touched.
    for proc in Path("/proc").iterdir():
        if not proc.name.isdigit():
            continue
        try:
            args = (proc / "cmdline").read_bytes().split(b"\0")
            if (
                str(config_path()).encode() in args
                and b"--config" in args
                and b"drive-native-fsd" in args[0]
            ):
                os.kill(int(proc.name), signal.SIGTERM)
        except (OSError, ProcessLookupError):
            pass
    QTest.qWait(600)
    password = ""
    window.close()
    scratch.cleanup()
