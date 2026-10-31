#!/usr/bin/python3
"""Synthetic local IPC peer. No network connections and no request logging."""
import json
import pathlib
import socket
import struct
import time

root = pathlib.Path(__file__).resolve().parent
mode = (root / "mode").read_text().strip()
peer = socket.socket(fileno=0)


def exact(count):
    data = bytearray()
    while len(data) < count:
        block = peer.recv(count - len(data))
        if not block:
            raise EOFError
        data.extend(block)
    return data


def response(request, body):
    data = json.dumps({"version": request["version"], "id": request["id"], "body": body}).encode()
    peer.sendall(struct.pack("!I", len(data)) + data)


def remain_alive():
    while peer.recv(4096):
        pass
    raise EOFError


try:
    while True:
        size = struct.unpack("!I", exact(4))[0]
        assert 0 < size < 65536
        request = json.loads(exact(size))
        body = request["body"]
        command = body if isinstance(body, str) else next(iter(body))
        with (root / "commands.jsonl").open("a") as log:
            log.write(json.dumps({"command": command}) + "\n")
        if command == "Connect":
            if mode == "auth":
                response(request, {"Failed": {"category": "authentication_or_protocol"}})
                remain_alive()
            response(request, {"Connected": {"restore": "server"}})
        elif command == "ConfigureTransfers":
            response(request, {"TransfersConfigured": body[command]})
        elif command == "Pull":
            if mode == "pull":
                response(request, {"Failed": {"category": "native_response_capacity"}})
                remain_alive()
            response(request, {"Events": None})
        elif command == "Wait":
            if mode == "idle":
                response(request, {"Failed": {"category": "native_receive_idle"}})
            elif mode == "disconnect":
                peer.close()
                time.sleep(30)  # IPC failure with a live child, not a process crash.
                break
            elif mode != "hold":
                raise AssertionError("unknown synthetic mode")
            remain_alive()
        else:
            raise AssertionError("unexpected synthetic command")
except EOFError:
    pass
