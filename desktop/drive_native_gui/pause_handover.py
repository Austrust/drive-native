"""A durable startup intent, never a second writer of the live state database."""
import json
import os
import stat
import uuid
from pathlib import Path


def preserve_pause(profile):
    task = profile["task_id"]
    if str(uuid.UUID(task)) != task:
        raise ValueError("invalid task identity")
    base = Path(os.environ.get("XDG_DATA_HOME", str(Path.home() / ".local/share")))
    if not base.is_absolute():
        raise ValueError("data path must be absolute")
    directory = os.open(base / "drive-native" / task, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW)
    temp = ".startup-pause-" + uuid.uuid4().hex
    try:
        info = os.fstat(directory)
        if not stat.S_ISDIR(info.st_mode) or info.st_uid != os.getuid() or info.st_mode & 0o077:
            raise ValueError("task state must be private")
        try:
            existing = os.open("startup-pause-v1.json", os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK, dir_fd=directory)
        except FileNotFoundError:
            existing = None
        if existing is not None:
            with os.fdopen(existing, "rb") as stream:
                info = os.fstat(stream.fileno())
                if not stat.S_ISREG(info.st_mode) or info.st_uid != os.getuid() or info.st_mode & 0o077 or info.st_size > 4096:
                    raise ValueError("invalid existing startup intent")
                if json.loads(stream.read(4097)) != {"schema": 1, "task_id": task}:
                    raise ValueError("startup intent task mismatch")
        fd = os.open(temp, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, 0o600, dir_fd=directory)
        with os.fdopen(fd, "wb") as stream:
            stream.write(json.dumps({"schema": 1, "task_id": task}).encode())
            stream.flush()
            os.fsync(stream.fileno())
        os.rename(temp, "startup-pause-v1.json", src_dir_fd=directory, dst_dir_fd=directory)
        os.fsync(directory)
    finally:
        try:
            os.unlink(temp, dir_fd=directory)
        except FileNotFoundError:
            pass
        os.close(directory)
