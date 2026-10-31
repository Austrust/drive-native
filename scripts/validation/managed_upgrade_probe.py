"""Synthetic installed-file + managed-user-service upgrade probe.

The caller installs packages in an isolated installation root. This script never
installs packages, accesses a NAS/keyring, or changes global D-Bus environment.
Run as the same non-root test user on the host systemd user bus. The GUI modules
on PYTHONPATH must correspond to the package under test.
"""
import argparse
from contextlib import closing
import hashlib
import json
import os
from pathlib import Path
import re
import sqlite3
import stat
import subprocess
import tempfile
import time
import uuid

BODY = b"managed upgrade pending edit\n"


def require(condition, message):
    if not condition:
        raise RuntimeError(message)


def private_json(path):
    fd = os.open(path, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK)
    with os.fdopen(fd, "rb") as stream:
        info = os.fstat(stream.fileno())
        require(stat.S_ISREG(info.st_mode) and info.st_uid == os.getuid()
                and not info.st_mode & 0o077 and info.st_size <= 65536,
                "coordination file is not a private owned regular file")
        return json.loads(stream.read(65537))


def publish(path, value):
    with tempfile.NamedTemporaryFile(dir=path.parent, delete=False) as stream:
        temporary = Path(stream.name)
        try:
            stream.write(json.dumps(value, indent=2).encode())
            stream.flush()
            os.fsync(stream.fileno())
            temporary.replace(path)
            fd = os.open(path.parent, os.O_RDONLY | os.O_DIRECTORY)
            try:
                os.fsync(fd)
            finally:
                os.close(fd)
        finally:
            temporary.unlink(missing_ok=True)


def absolute_path(value):
    path = Path(value)
    require(path.is_absolute() and path == path.resolve(), "paths must be absolute without symlink components")
    require(not any(c in str(path) for c in '$%\r\n\0'), "unsupported systemd path characters")
    return path


def word(value):
    return '"' + str(value).replace('\\', '\\\\').replace('"', '\\"') + '"'


def identity(path):
    value = os.stat(path)
    return [value.st_dev, value.st_ino]


def package_version(installation):
    return subprocess.check_output(
        ["dpkg-query", "--admindir=" + str(installation / "var/lib/dpkg"),
         "-W", "-f=${Version}", "drive-native"], text=True, timeout=10).strip()


def verify_package(installation):
    result = subprocess.run(["dpkg", "--root=" + str(installation), "--verify", "drive-native"],
                            capture_output=True, timeout=30)
    require(result.returncode == 0 and not result.stdout.strip(), "installed package payload verification failed")


def database_snapshot(path):
    with closing(sqlite3.connect(path.as_uri() + "?mode=ro", uri=True)) as db:
        db.execute("BEGIN")
        return {
            "cursor": db.execute("SELECT value FROM metadata WHERE key='committed_cursor'").fetchone()[0],
            "pending": db.execute("SELECT id,node,kind,base,blob,generation,state FROM operations WHERE state='pending' ORDER BY id").fetchall(),
            "dirty": db.execute("SELECT node,token,base,generation,checkpoint,dirty FROM drafts WHERE dirty=1 ORDER BY node").fetchall(),
        }


class Probe:
    def __init__(self, root, manifest):
        self.root, self.manifest = root, manifest
        require(manifest.get("schema") == 1 and re.fullmatch(r"[0-9a-f]{32}", manifest["run_id"]), "invalid run identity")
        require(str(uuid.UUID(manifest["task_id"])) == manifest["task_id"], "invalid synthetic task identity")
        self.instance = manifest["instance"]
        require(re.fullmatch(r"[0-9a-f]{32}", self.instance), "invalid isolated instance")
        self.unit = "drive-native-upgrade-test-" + self.instance + ".service"
        require(manifest["unit"] == self.unit, "unit identity mismatch")
        self.definition = root / self.unit
        self.link = Path(manifest["unit_link"])
        host_config = absolute_path(os.environ.get("XDG_CONFIG_HOME", str(Path.home() / ".config")))
        require(self.link == host_config / "systemd/user" / self.unit, "user unit link is outside this invocation's unit directory")
        self.host_env = dict(os.environ)
        self.host_env["DBUS_SESSION_BUS_ADDRESS"] = "unix:path=/run/user/" + str(os.getuid()) + "/bus"
        self.host_env["XDG_RUNTIME_DIR"] = "/run/user/" + str(os.getuid())
        require(stat.S_ISSOCK(os.stat(Path(self.host_env["XDG_RUNTIME_DIR"]) / "bus").st_mode), "host user bus is unavailable")
        self.installation = absolute_path(manifest["installation_root"])
        require(self.installation != Path("/"), "the host installation cannot be tested")
        require(not root.is_relative_to(self.installation) and not self.installation.is_relative_to(root), "installation and fixture roots must be separate")
        self.binaries = self.installation / "usr/lib/drive-native"
        self.fixture = root / "fixture"
        self.profile_path = self.fixture / "config/drive-native/profiles" / (self.instance + ".json")
        self.profile = private_json(self.profile_path)
        require(self.profile["task_id"] == manifest["task_id"] and self.profile["username"] == "fixture"
                and self.profile["host"] == "127.0.0.1", "only the isolated synthetic task is supported")
        self.mount = absolute_path(self.profile["mount"])
        require(self.mount.is_relative_to(self.fixture), "mount escaped synthetic fixture")
        self.state = self.fixture / "data/drive-native" / self.profile["task_id"]
        self.load_log = Path(manifest["load_log"]) if manifest.get("load_log") else None
        # These assignments affect only this process and its controller subprocesses.
        os.environ.update(DBUS_SESSION_BUS_ADDRESS=self.host_env["DBUS_SESSION_BUS_ADDRESS"],
                          XDG_RUNTIME_DIR=self.host_env["XDG_RUNTIME_DIR"],
                          XDG_CONFIG_HOME=str(self.fixture / "config"), XDG_DATA_HOME=str(self.fixture / "data"),
                          DRIVE_NATIVE_INSTANCE=self.instance, DRIVE_NATIVE_BIN_DIR=str(self.binaries))
        from PySide6.QtCore import QCoreApplication
        from drive_native_gui.bus import ClientBus
        from drive_native_gui.lifecycle import ServiceLauncher
        from drive_native_gui.upgrade import BackendUpgrade
        self.app = QCoreApplication.instance() or QCoreApplication([])
        self.bus = ClientBus(instance=self.instance)
        self.launcher = ServiceLauncher(managed=True, unit=self.unit)
        self.upgrade = BackendUpgrade(self.bus, self.launcher, save_pause=None)

    def load(self, kind):
        if self.load_log:
            with self.load_log.open("a") as stream:
                stream.write(json.dumps(dict(time=time.time(), kind=kind, scope=self.root.name,
                                             activity="isolated installed-file managed upgrade probe", baseline_changed=False)) + "\n")

    def systemctl(self, *args):
        return subprocess.check_output(["systemctl", "--user", *args], env=self.host_env,
                                       text=True, stderr=subprocess.DEVNULL, timeout=10).strip()

    def info(self):
        result = self.systemctl("show", self.unit, "-p", "ActiveState", "-p", "MainPID", "-p", "FragmentPath")
        return dict(line.split("=", 1) for line in result.splitlines() if "=" in line)

    def wait(self, predicate, seconds=30):
        deadline = time.monotonic() + seconds
        while time.monotonic() < deadline:
            self.app.processEvents()
            if predicate():
                return
            time.sleep(.05)
        raise TimeoutError("bounded isolated condition did not complete; preserve the fixture")

    def call(self, method, allow_disappearance=False):
        result = []
        self.bus.call(method, callback=lambda data, error: result.append((data, error)))
        self.wait(lambda: result, 20)
        data, error = result[0]
        require(not error or allow_disappearance and error == "service_changed", "local desktop operation was not confirmed")
        return data

    def overview(self):
        value = self.call("GetOverview")
        require(value["task_id"] == self.manifest["task_id"], "running task identity mismatch")
        return value

    def install_unit(self):
        require(not self.bus.owner, "isolated bus name is already owned")
        require(not self.link.exists() and not self.link.is_symlink(), "unit link already exists")
        runtime = self.root / "runtime"
        runtime.mkdir(mode=0o700)
        lines = ["[Unit]", "Description=Isolated Drive Native managed upgrade fixture", "[Service]",
                 "Type=dbus", "BusName=" + self.bus.service,
                 "ExecStart=" + word(self.binaries / "drive-native-fsd") + " --config " + word(self.profile_path),
                 "Restart=no", "TimeoutStopSec=infinity", "SendSIGKILL=no", "UMask=0077", "LimitCORE=0",
                 "StandardOutput=null", "StandardError=journal"]
        for name, value in {"XDG_CONFIG_HOME": self.fixture / "config", "XDG_DATA_HOME": self.fixture / "data",
                            "XDG_RUNTIME_DIR": runtime, "DBUS_SESSION_BUS_ADDRESS": self.host_env["DBUS_SESSION_BUS_ADDRESS"],
                            "DRIVE_NATIVE_INSTANCE": self.instance}.items():
            lines.append("Environment=" + word(name + "=" + str(value)))
        self.definition.write_text("\n".join(lines) + "\n")
        self.link.parent.mkdir(parents=True, exist_ok=True)
        self.link.symlink_to(self.definition)
        self.systemctl("daemon-reload")
        require(Path(self.info()["FragmentPath"]).resolve() == self.definition, "systemd loaded a different unit")
        results = []
        def started(error):
            results.append(error)
        self.launcher.completed.connect(started)
        self.launcher.start()
        self.wait(lambda: results)
        self.launcher.completed.disconnect(started)
        require(results == [None], "initial managed service did not start")
        self.wait(lambda: self.bus.owner)
        self.wait(lambda: self.overview().get("mounted"))

    def cleanup(self):
        require(self.link.is_symlink() and self.link.resolve() == self.definition, "refuse to remove a different unit")
        require(Path(self.info()["FragmentPath"]).resolve() == self.definition, "running unit definition changed")
        if self.bus.owner:
            self.overview()  # Identity check before sending a task-scoped stop.
            self.call("StopMount", allow_disappearance=True)
        self.wait(lambda: not self.bus.owner and self.info().get("MainPID") == "0"
                  and self.info().get("ActiveState") in ("inactive", "failed"), 60)
        require(not self.mount.is_mount(), "owned mount is still active")
        require(self.link.is_symlink() and self.link.resolve() == self.definition, "unit link changed during cleanup")
        self.link.unlink()
        self.systemctl("daemon-reload")
        # Keep the fixture, database, drafts, package and report for inspection.

    def run(self, continue_timeout):
        self.load("isolated_development")
        self.install_unit()
        require(self.overview()["connection"] == "locked", "fixture unexpectedly authenticated")
        with (self.mount / "A.txt").open("r+b") as stream:
            stream.write(BODY)
            stream.truncate()
            stream.flush()
            os.fsync(stream.fileno())
        self.call("Pause")
        require(self.overview()["connection"] == "paused", "old task did not pause")
        before = database_snapshot(self.state / "state.sqlite")
        require(len(before["pending"]) == 1 and len(before["dirty"]) == 1, "fixture must contain one pending dirty edit")
        old_owner = self.bus.owner
        old_pid = int(self.info()["MainPID"])
        old_inode = identity(Path("/proc") / str(old_pid) / "exe")
        ready = dict(schema=1, run_id=self.manifest["run_id"], unit=self.unit,
                     old_version=package_version(self.installation), old_pid=old_pid,
                     instance=self.instance, task_id=self.manifest["task_id"], old_inode=old_inode)
        publish(self.root / "ready-for-upgrade.json", ready)
        print(json.dumps({"phase": "ready-for-upgrade", "root": str(self.root), "run_id": ready["run_id"]}), flush=True)
        self.wait(lambda: (self.root / "continue-upgrade.json").exists(), continue_timeout)
        instruction = private_json(self.root / "continue-upgrade.json")
        require(instruction.get("schema") == 1 and instruction.get("run_id") == ready["run_id"], "continue marker belongs to another run")
        new_version = package_version(self.installation)
        require(new_version == instruction.get("expected_version") and new_version != ready["old_version"], "new package version was not confirmed")
        verify_package(self.installation)
        require(self.bus.owner == old_owner and self.overview()["connection"] == "paused", "old paused backend changed during installation")
        require(identity(Path("/proc") / str(old_pid) / "exe") == old_inode
                and identity(self.binaries / "drive-native-fsd") != old_inode, "installed file was not replaced while retaining the old backend")
        self.wait(lambda: (self.upgrade.check() or self.upgrade.available))
        outcomes = []
        self.upgrade.finished.connect(outcomes.append)
        require(self.upgrade.begin(self.profile), "upgrade controller did not begin")
        self.wait(lambda: outcomes, 150)
        require(outcomes == [None], "managed upgrade was not confirmed")
        overview = self.overview()
        info = self.info()
        new_pid = int(info["MainPID"])
        require(self.bus.owner != old_owner and new_pid != old_pid and info["ActiveState"] == "active", "managed backend identity did not change")
        require(overview["connection"] == "paused" and overview["mounted"], "new backend is not a paused mount")
        require(identity(Path("/proc") / str(new_pid) / "exe") == identity(self.binaries / "drive-native-fsd"), "new backend does not run the installed image")
        require((self.mount / "A.txt").read_bytes() == BODY, "pending file content changed")
        require(database_snapshot(self.state / "state.sqlite") == before, "pending journal or cursor changed")
        self.cleanup()
        publish(self.root / "result.json", dict(schema=1, passed=True, run_id=ready["run_id"],
                old_version=ready["old_version"], new_version=new_version, managed=True, paused=True,
                pending_edits=1, content_sha256=hashlib.sha256(BODY).hexdigest(), normal_cleanup=True,
                nas_accessed=False, host_packages_changed=False))
        self.load("isolated_development_completed")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="action", required=True)
    run = sub.add_parser("run")
    run.add_argument("--root", required=True)
    run.add_argument("--fixture-builder", required=True)
    run.add_argument("--installation-root", required=True)
    run.add_argument("--expected-old-version", default="0.1.0~dev20261031.3")
    run.add_argument("--continue-timeout", type=int, default=1800)
    run.add_argument("--load-log")
    advance = sub.add_parser("continue")
    advance.add_argument("--root", required=True)
    advance.add_argument("--expected-version", required=True)
    cleanup = sub.add_parser("cleanup")
    cleanup.add_argument("--root", required=True)
    args = parser.parse_args()
    require(os.getuid() != 0, "run as the unprivileged test user")
    root = absolute_path(args.root)
    os.umask(0o077)
    if args.action == "run":
        require(not root.exists(), "a fresh isolated root is required")
        installation = absolute_path(args.installation_root)
        require(installation != Path("/"), "refuse the host installation")
        require(not root.is_relative_to(installation) and not installation.is_relative_to(root), "installation and fixture roots must be separate")
        require(0 < args.continue_timeout <= 86400, "continue timeout must be between 1 and 86400 seconds")
        require(package_version(installation) == args.expected_old_version, "unexpected initial package version")
        verify_package(installation)
        root.mkdir(mode=0o700)
        builder = absolute_path(args.fixture_builder)
        subprocess.run([str(builder), str(root / "fixture")], check=True, timeout=30, stdout=subprocess.DEVNULL)
        original = root / "fixture/config/drive-native/profile.json"
        profile = private_json(original)
        require(profile["username"] == "fixture" and profile["host"] == "example.invalid", "builder is not the public synthetic fixture")
        profile.update(host="127.0.0.1", tls_name="localhost", writable=True)
        instance = uuid.uuid4().hex
        profiles = original.parent / "profiles"
        profiles.mkdir(mode=0o700, exist_ok=True)
        publish(profiles / (instance + ".json"), profile)
        unit = "drive-native-upgrade-test-" + instance + ".service"
        host_config = absolute_path(os.environ.get("XDG_CONFIG_HOME", str(Path.home() / ".config")))
        manifest = dict(schema=1, run_id=uuid.uuid4().hex, instance=instance, task_id=profile["task_id"], unit=unit,
                        unit_link=str(host_config / "systemd/user" / unit), installation_root=str(installation),
                        load_log=args.load_log)
        publish(root / "run.json", manifest)
    else:
        info = root.stat()
        require(stat.S_ISDIR(info.st_mode) and info.st_uid == os.getuid() and not info.st_mode & 0o077, "fixture root must be owned and private")
        manifest = private_json(root / "run.json")
    if args.action == "continue":
        require(args.expected_version and not any(ord(c) < 32 for c in args.expected_version), "invalid expected version")
        ready = private_json(root / "ready-for-upgrade.json")
        require(ready["run_id"] == manifest["run_id"], "ready marker mismatch")
        publish(root / "continue-upgrade.json", dict(schema=1, run_id=manifest["run_id"], expected_version=args.expected_version))
        return
    probe = None
    try:
        probe = Probe(root, manifest)
        if args.action == "cleanup":
            probe.cleanup()
            publish(root / "cleanup.json", dict(schema=1, normal_cleanup=True, run_id=manifest["run_id"]))
        else:
            probe.run(args.continue_timeout)
    except Exception as error:
        publish(root / "failure.json", dict(schema=1, passed=False, run_id=manifest["run_id"],
                error_type=type(error).__name__, reason=str(error), preserve_fixture=True))
        if probe:
            probe.load("isolated_development_failed_preserved")
        raise


if __name__ == "__main__":
    main()
