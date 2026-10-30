import json
import os
import stat
from pathlib import Path
import tempfile
import uuid
import re
import fcntl
from contextlib import contextmanager


def config_path(instance=None):
    base = Path(os.environ.get("XDG_CONFIG_HOME", str(Path.home() / ".config")))
    if not base.is_absolute():
        raise ValueError("配置目录必须是绝对路径")
    root = base / "drive-native"
    instance = os.environ.get("DRIVE_NATIVE_INSTANCE", "") if instance is None else instance
    if instance and not re.fullmatch(r"[0-9a-f]{32}", instance):
        raise ValueError("挂载任务标识无效")
    return root / "profiles" / (instance + ".json") if instance else root / "profile.json"


def load_profile(instance=None):
    path = config_path(instance)
    try:
        fd = os.open(path, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK)
    except FileNotFoundError:
        return None
    with os.fdopen(fd, "r", encoding="utf-8") as stream:
        info = os.fstat(stream.fileno())
        if (
            not stat.S_ISREG(info.st_mode)
            or info.st_uid != os.getuid()
            or info.st_mode & 0o077
            or info.st_size > 65536
        ):
            raise ValueError("配置文件权限或大小不正确")
        profile = json.loads(stream.read(65537))
    if profile.get("schema") != 1 or "password" in profile:
        raise ValueError("配置格式不正确")
    return profile


def _save_profile(profile):
    if profile.get("schema") != 1 or "password" in profile:
        raise ValueError("配置格式不正确")
    path = config_path()
    path.parent.mkdir(parents=True, mode=0o700, exist_ok=True)
    if (
        path.parent.is_symlink()
        or path.parent.stat().st_uid != os.getuid()
        or path.parent.stat().st_mode & 0o077
    ):
        raise ValueError("配置目录必须由当前用户私有")
    if path.exists():
        raise ValueError("已有同步配置，请先在设置中管理现有连接")
    fd, temporary = tempfile.mkstemp(prefix=".profile-", dir=path.parent)
    try:
        with os.fdopen(fd, "w", encoding="utf-8") as stream:
            json.dump(profile, stream, ensure_ascii=False, indent=2)
            stream.flush()
            os.fsync(stream.fileno())
        # link is exclusive: another window cannot overwrite a concurrent setup.
        os.link(temporary, path)
        directory = os.open(path.parent, os.O_RDONLY | os.O_DIRECTORY)
        try:
            os.fsync(directory)
        finally:
            os.close(directory)
    finally:
        os.unlink(temporary)
    return path


def binary(name):
    base = Path(os.environ.get("DRIVE_NATIVE_BIN_DIR", "/usr/lib/drive-native"))
    if not base.is_absolute() or name not in (
        "drive-native-fsd",
        "drive-native-worker",
        "drive-native-setup",
    ):
        raise ValueError("程序安装位置无效")
    target = base / name
    if not target.is_file() or not os.access(target, os.X_OK):
        raise ValueError("后台组件未安装完整")
    return str(target)


def archive_profile(expected_task):
    """Called only after the user service has stopped; keep all task data."""
    current = load_profile()
    if not current or current.get("task_id") != expected_task:
        raise ValueError("同步配置已经变化，请重新打开客户端。")
    path = config_path()
    backup = path.with_name(f"profile-backup-{uuid.uuid4()}.json")
    os.rename(path, backup)
    directory = os.open(path.parent, os.O_RDONLY | os.O_DIRECTORY)
    try:
        os.fsync(directory)
    finally:
        os.close(directory)
    return backup


def _restore_profile(backup):
    path = config_path()
    # Exclusive restoration: never replace a newly configured task.
    os.link(backup, path)
    directory = os.open(path.parent, os.O_RDONLY | os.O_DIRECTORY)
    try:
        os.fsync(directory)
    finally:
        os.close(directory)


def service_unit(instance=None):
    selected = os.environ.get("DRIVE_NATIVE_INSTANCE", "") if instance is None else instance
    config_path(selected)  # validate before constructing a unit or bus name
    return f"drive-native@{selected}.service" if selected else "drive-native.service"


def bus_name(instance=None):
    selected = os.environ.get("DRIVE_NATIVE_INSTANCE", "") if instance is None else instance
    config_path(selected)
    return "org.drivenative.Client1" + (".i" + selected if selected else "")


def profile_records():
    records = []
    old = load_profile("")
    if old:
        records.append(("", old))
    parent = config_path("").parent / "profiles"
    if parent.is_symlink():
        raise ValueError("配置目录不能是符号链接")
    if parent.exists():
        for path in sorted(parent.glob("*.json")):
            if re.fullmatch(r"[0-9a-f]{32}", path.stem):
                profile = load_profile(path.stem)
                if profile:
                    records.append((path.stem, profile))
    return records


def validate_mount_overlap(profile):
    mount = Path(profile["mount"]).resolve()
    for _, other in profile_records():
        if other["task_id"] == profile["task_id"]:
            continue
        existing = Path(other["mount"]).resolve()
        if mount == existing or mount in existing.parents or existing in mount.parents:
            raise ValueError("挂载位置与已有任务重叠，请选择独立的空文件夹。")


@contextmanager
def profile_lock():
    root = config_path("").parent
    root.mkdir(parents=True, exist_ok=True, mode=0o700)
    st = root.lstat()
    if not stat.S_ISDIR(st.st_mode) or st.st_uid != os.getuid() or st.st_mode & 0o077:
        raise ValueError("配置目录必须由当前用户私有")
    fd = os.open(root / ".profiles.lock", os.O_RDWR | os.O_CREAT | os.O_NOFOLLOW, 0o600)
    try:
        info = os.fstat(fd)
        if not stat.S_ISREG(info.st_mode) or info.st_uid != os.getuid() or info.st_mode & 0o077:
            raise ValueError("配置锁权限不正确")
        fcntl.flock(fd, fcntl.LOCK_EX)
        yield
    finally:
        os.close(fd)


def save_profile(profile):
    with profile_lock():
        validate_mount_overlap(profile)
        return _save_profile(profile)


def restore_profile(backup):
    with profile_lock():
        with open(backup, encoding="utf-8") as stream:
            profile = json.load(stream)
        validate_mount_overlap(profile)
        _restore_profile(backup)
