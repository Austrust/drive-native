"""Bounded, noninteractive keyring unlock after the managed service starts."""

import resource
import sys
from PySide6.QtCore import QCoreApplication, QObject, QTimer
from .bus import ClientBus
from .config import load_profile
from .credentials import secret_job
from .lifecycle import same_task


class StartupUnlock(QObject):
    def __init__(self, app):
        super().__init__(app)
        self.app = app
        self.profile = load_profile()
        self.bus = ClientBus(self)
        self.bus.changed.connect(self.schedule)
        self.bus.availability.connect(lambda _: self.schedule())
        self.busy = False
        self.done = False
        self.job = None
        self.retry = QTimer(self)
        self.retry.setSingleShot(True)
        self.retry.setInterval(2000)
        self.retry.timeout.connect(self.check)
        self.limit = QTimer(self)
        self.limit.setSingleShot(True)
        self.limit.setInterval(120000)
        self.limit.timeout.connect(lambda: self.finish(0))
        self.limit.start()
        QTimer.singleShot(0, self.check)

    def schedule(self):
        if not self.done and not self.busy:
            self.retry.start()

    def check(self):
        if self.done or self.busy:
            return
        if not self.profile:
            self.finish(0)
            return
        self.busy = True
        self.bus.call("GetOverview", callback=self.observed)

    def observed(self, data, error):
        self.busy = False
        if error:
            self.schedule()
            return
        if not same_task(self.profile, data):
            self.finish(1)
            return
        if data.get("connection") != "locked":
            self.finish(0)
            return
        self.busy = True
        owner = self.bus.owner
        self.job = secret_job(
            self.profile["task_id"],
            lambda value, failure: self.loaded(owner, value, failure),
            allow_unlock=False,
        )

    def loaded(self, owner, password, error):
        if self.done:
            return
        if error or not password or owner != self.bus.owner:
            self.busy = False
            self.schedule()
            return

        def checked(data, failure):
            if self.done:
                return
            if failure or owner != self.bus.owner or not same_task(self.profile, data):
                self.busy = False
                self.schedule()
                return
            if data.get("connection") != "locked":
                self.finish(0)
                return
            self.bus.call("Unlock", [password], self.unlocked)

        self.bus.call("GetOverview", callback=checked)

    def unlocked(self, _data, error):
        self.busy = False
        if error:
            self.schedule()
        else:
            self.finish(0)

    def finish(self, code):
        if self.done:
            return
        self.done = True
        self.retry.stop()
        self.limit.stop()
        self.app.exit(code)


def main():
    resource.setrlimit(resource.RLIMIT_CORE, (0, 0))
    app = QCoreApplication(sys.argv)
    try:
        controller = StartupUnlock(app)
    except Exception:
        return 1
    return app.exec()


if __name__ == "__main__":
    raise SystemExit(main())
