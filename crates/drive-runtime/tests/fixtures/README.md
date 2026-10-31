# Synthetic worker fixtures

- [diagnostic_worker_loader.c](diagnostic_worker_loader.c) is a test-only ELF entry point, compiled with the existing C toolchain into each private fixture. It goes through the real supervisor's pinned executable path and then invokes the Python peer by its ordinary sibling path. It is not a product binary and never changes the pinned descriptor's `CLOEXEC` flag.
- [diagnostic_worker.py](diagnostic_worker.py) exchanges local IPC messages without opening network connections. It records command names, never request payloads, and rejects an inherited worker-image descriptor.

[connection_diagnostics.rs](../connection_diagnostics.rs) checks diagnostics, requested interruption and the case where the installed executable pathname is replaced after the supervisor has pinned its original image. The Python peer remains a stable test dependency; these fixtures do not establish support for running interpreter scripts as production workers.
