"""Test-only PAM-style unlock of the isolated GNOME login keyring.

Wire reference: GNOME/gnome-keyring tag 46.1, daemon/control/
gkd-control-client.c and gkd-control-codes.h. Never imported by product code.
"""

import os
import socket
import stat
import struct
from pathlib import Path


def unlock_test_keyring(directory, password):
    directory = Path(directory)
    info = directory.lstat()
    assert stat.S_ISDIR(info.st_mode) and info.st_uid == os.getuid()
    assert info.st_mode & 0o077 == 0
    path = directory / "control"
    info = path.lstat()
    assert stat.S_ISSOCK(info.st_mode) and info.st_uid == os.getuid()
    encoded = password.encode("utf-8")
    assert 0 < len(encoded) < 8192 and b"\0" not in encoded
    with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as channel:
        channel.settimeout(5)
        channel.connect(str(path))
        _, peer_uid, _ = struct.unpack(
            "3i", channel.getsockopt(socket.SOL_SOCKET, socket.SO_PEERCRED, 12)
        )
        assert peer_uid == os.getuid()
        channel.sendall(b"\0" + struct.pack("!III", len(encoded) + 12, 1, len(encoded)) + encoded)
        response = bytearray()
        while len(response) < 8:
            part = channel.recv(8 - len(response))
            assert part, "test keyring control disconnected"
            response.extend(part)
        assert struct.unpack("!II", response) == (8, 0), "test keyring rejected unlock"
