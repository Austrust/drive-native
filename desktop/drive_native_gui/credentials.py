"""Secret Service access runs off the GUI thread; there is no plaintext fallback."""

from PySide6.QtCore import QObject, QRunnable, Signal, QThreadPool


class Result(QObject):
    completed = Signal(object, object)


class SecretJob(QRunnable):
    def __init__(self, task_id, password=None, *, allow_unlock=True):
        super().__init__()
        self.task_id, self.password = task_id, password
        self.allow_unlock = allow_unlock
        self.result = Result()

    def run(self):
        value, error = None, None
        try:
            import secretstorage

            connection = secretstorage.dbus_init()
            try:
                collection = (
                    secretstorage.get_default_collection(connection)
                    if self.password is not None
                    else secretstorage.Collection(connection)
                )
                if collection.is_locked():
                    if not self.allow_unlock:
                        raise RuntimeError("background keyring locked")
                    if collection.unlock(timeout=120):
                        raise RuntimeError("unlock cancelled")
                attributes = {"application": "drive-native", "task": self.task_id}
                if self.password is not None:
                    collection.create_item(
                        "Drive Native 登录凭据",
                        attributes,
                        self.password.encode("utf-8"),
                        replace=True,
                    )
                    value = True
                else:
                    items = list(collection.search_items(attributes))
                    if len(items) > 1:
                        raise RuntimeError("ambiguous keyring identity")
                    value = items[0].get_secret().decode("utf-8") if items else None
            finally:
                connection.close()
        except Exception:
            error = "系统钥匙环不可用或尚未解锁，请手动输入密码。"
        self.password = None
        self.result.completed.emit(value, error)


def secret_job(task_id, callback, password=None, *, allow_unlock=True):
    job = SecretJob(task_id, password, allow_unlock=allow_unlock)
    job.result.completed.connect(callback)
    QThreadPool.globalInstance().start(job)
    return job
