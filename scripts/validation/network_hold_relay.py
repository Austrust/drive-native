"""Opaque loopback TCP relay with sustained disconnect, scoped to one test task.

No TLS interception, payload logs, firewall changes, or process inspection.
The client must retain the upstream certificate name and verified CA settings.
"""
import argparse
import asyncio
import ipaddress
import json
import os
from pathlib import Path


class Relay:
    def __init__(self, host, port):
        self.host, self.port = host, port
        self.held = False
        self.generation = self.accepted = self.rejected = self.forwarded = 0
        self.active = set()

    def state(self):
        return dict(held=self.held, generation=self.generation,
                    accepted=self.accepted, rejected=self.rejected,
                    forwarded=self.forwarded, active_streams=len(self.active))

    def command(self, name):
        if name == 'hold':
            self.held = True
            self.generation += 1
            for stream in tuple(self.active):
                stream.transport.abort()
            self.active.clear()
        elif name == 'release':
            self.held = False
        elif name != 'status':
            raise ValueError('unknown relay command')
        return self.state()

    async def connection(self, reader, front):
        self.accepted += 1
        back, tasks = None, []
        generation = self.generation
        if self.held:
            self.rejected += 1
            front.transport.abort()
            return
        self.active.add(front)  # Includes upstream connects pending at hold.
        try:
            remote, back = await asyncio.wait_for(
                asyncio.open_connection(self.host, self.port), 10)
            if self.held or generation != self.generation:
                self.rejected += 1
                back.transport.abort()
                front.transport.abort()
                return
            self.active.add(back)
            self.forwarded += 1

            async def copy(source, target):
                while data := await source.read(65536):
                    target.write(data)
                    await target.drain()

            tasks = [asyncio.create_task(copy(reader, back)),
                     asyncio.create_task(copy(remote, front))]
            await asyncio.wait(tasks, return_when=asyncio.FIRST_COMPLETED)
        except (OSError, TimeoutError):
            pass
        finally:
            for task in tasks:
                task.cancel()
            await asyncio.gather(*tasks, return_exceptions=True)
            for stream in (front, back):
                if stream:
                    self.active.discard(stream)
                    stream.close()
                    try:
                        await stream.wait_closed()
                    except OSError:
                        pass

    async def control(self, reader, writer):
        try:
            frame = await asyncio.wait_for(reader.readline(), 3)
            result = self.command(frame.decode('ascii').strip())
        except (ValueError, UnicodeError, TimeoutError):
            result = {'invalid_command': True}
        writer.write((json.dumps(result) + '\n').encode())
        await writer.drain()
        writer.close()


async def serve(args):
    state = args.state
    info = state.lstat()
    if (state.is_symlink() or not state.is_dir() or info.st_uid != os.geteuid()
            or info.st_mode & 0o077):
        raise ValueError('owned private state directory required')
    if (state/'relay.sock').exists() or (state/'ready.json').exists():
        raise ValueError('refusing reused relay state')
    if not ipaddress.ip_address(args.listen_host).is_loopback:
        raise ValueError('loopback listener required')
    relay = Relay(args.upstream_host, args.upstream_port)
    tcp = await asyncio.start_server(relay.connection, args.listen_host, args.listen_port)
    control = await asyncio.start_unix_server(relay.control, path=str(state/'relay.sock'), limit=128)
    os.chmod(state/'relay.sock', 0o600)
    with (state/'ready.json').open('x') as output:
        json.dump(dict(listen=tcp.sockets[0].getsockname(), tls_terminated=False,
                       payloads_logged=False), output)
    async with tcp, control:
        try:
            await asyncio.gather(tcp.serve_forever(), control.serve_forever())
        finally:
            relay.command('hold')


if __name__ == '__main__':
    os.umask(0o077)
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--state', required=True, type=Path)
    parser.add_argument('--upstream-host', required=True)
    parser.add_argument('--upstream-port', type=int, default=6690)
    parser.add_argument('--listen-host', default='127.0.0.2')
    parser.add_argument('--listen-port', type=int, default=6690)
    asyncio.run(serve(parser.parse_args()))
