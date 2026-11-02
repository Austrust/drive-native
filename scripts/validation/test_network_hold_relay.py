"""Synthetic local socket tests only; no NAS or credentials."""
import asyncio
import unittest
from unittest.mock import patch
from network_hold_relay import Relay


class RelayTests(unittest.IsolatedAsyncioTestCase):
    async def asyncSetUp(self):
        async def echo(reader, writer):
            try:
                while body := await reader.read(4096):
                    writer.write(body)
                    await writer.drain()
            except OSError:
                pass
            finally:
                writer.close()
        self.upstream = await asyncio.start_server(echo, '127.0.0.1', 0)
        self.relay = Relay('127.0.0.1', self.upstream.sockets[0].getsockname()[1])
        self.front = await asyncio.start_server(self.relay.connection, '127.0.0.1', 0)
        self.port = self.front.sockets[0].getsockname()[1]
        self.writers = []

    async def asyncTearDown(self):
        self.relay.command('hold')
        for writer in self.writers:
            writer.close()
        for server in (self.front, self.upstream):
            server.close()
            await server.wait_closed()
        await asyncio.sleep(0)

    async def connect(self):
        reader, writer = await asyncio.open_connection('127.0.0.1', self.port)
        self.writers.append(writer)
        return reader, writer

    async def closed(self, reader):
        try:
            self.assertEqual(await asyncio.wait_for(reader.read(1), 3), b'')
        except ConnectionResetError:
            pass

    async def test_hold_closes_existing_and_rejects_new_until_release(self):
        reader, writer = await self.connect()
        writer.write(b'opaque-test')
        await writer.drain()
        self.assertEqual(await reader.readexactly(11), b'opaque-test')
        self.assertEqual(self.relay.command('hold')['active_streams'], 0)
        await self.closed(reader)
        blocked, _ = await self.connect()
        await self.closed(blocked)
        self.assertGreaterEqual(self.relay.rejected, 1)
        self.relay.command('release')
        reader, writer = await self.connect()
        writer.write(b'again')
        await writer.drain()
        self.assertEqual(await reader.readexactly(5), b'again')

    async def test_inflight_connect_cannot_escape_hold_release_generation(self):
        real = asyncio.open_connection
        connected, proceed = asyncio.Event(), asyncio.Event()
        async def delayed(host, port):
            connection = await real(host, port)
            connected.set()
            await proceed.wait()
            return connection
        with patch('network_hold_relay.asyncio.open_connection', delayed):
            reader, writer = await real('127.0.0.1', self.port)
            self.writers.append(writer)
            await asyncio.wait_for(connected.wait(), 3)
            self.relay.command('hold')
            self.relay.command('release')
            proceed.set()
            await self.closed(reader)
        self.assertEqual(self.relay.forwarded, 0)

    async def test_unknown_control_does_not_release_hold(self):
        self.relay.command('hold')
        with self.assertRaises(ValueError):
            self.relay.command('drop_everything')
        self.assertTrue(self.relay.command('status')['held'])


if __name__ == '__main__':
    unittest.main()
