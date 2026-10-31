"""Observe a single existing FSD over local D-Bus; never unlock or control it.

Progress has its own bounded request lane. An overview waiting on Store cannot
stall stage observations. This tool does not read configuration, credentials,
databases, mounted paths or file bodies, and issues no NAS requests.
"""
import argparse
import ast
import asyncio
import json
import os
import pathlib
import re
import signal
import time

IFACE = 'org.drivenative.Client1'
PATH = '/org/drivenative/Client1'
PHASES = {'starting', 'authenticating', 'configuring', 'binding', 'catching_up',
          'recovering', 'seeding', 'enumerating', 'fencing', 'importing', 'waiting'}
RECOVERY = {'initial_index', 'permission_recheck', 'cursor_invalidated', 'filtered_type_changed'}
COUNTERS = ('attempt', 'started_unix_ms', 'elapsed_ms', 'phase_started_unix_ms',
            'phase_elapsed_ms', 'units', 'items', 'snapshot_attempt', 'cursor_advances',
            'events_applied', 'changed_without_advance', 'consecutive_changed_without_advance')
ACTIVE_PHASES = {'seeding', 'enumerating', 'fencing', 'importing', 'catching_up'}
PROGRESS_SIGNAL = re.compile(r'org\.drivenative\.Client1\.ProgressChanged\s+\((?:uint64\s+)?(\d+),?\)')
OWNER = re.compile(r'is owned by (:\d+\.\d+)\s*$')


def integer(value):
    if type(value) is not int or not 0 <= value <= (1 << 64) - 1:
        raise ValueError('invalid counter')
    return value


def sanitize_progress(document):
    """Allowlist structured fields; arbitrary server/error/extra text is dropped."""
    if not isinstance(document, dict) or document.get('schema') != 1:
        raise ValueError('invalid progress schema')
    value = document.get('initialization')
    if not isinstance(value, dict) or type(value.get('active')) is not bool:
        raise ValueError('invalid initialization')
    if value.get('phase') not in PHASES or value.get('recovery_reason') not in RECOVERY | {None}:
        raise ValueError('invalid stage')
    result = {key: integer(value.get(key)) for key in COUNTERS}
    result.update(active=value['active'], phase=value['phase'], recovery_reason=value.get('recovery_reason'))
    since = value.get('since_progress_ms')
    result['since_progress_ms'] = None if since is None else integer(since)
    spans = value.get('recent_stages')
    totals = value.get('phase_totals_ms')
    if not isinstance(spans, list) or len(spans) > 16 or not isinstance(totals, list) or len(totals) > 12:
        raise ValueError('invalid bounded stage history')
    result['recent_stages'] = []
    for span in spans:
        if not isinstance(span, dict) or span.get('phase') not in PHASES:
            raise ValueError('invalid history phase')
        result['recent_stages'].append({'phase': span['phase'], **{
            key: integer(span.get(key)) for key in ('started_unix_ms', 'elapsed_ms', 'units', 'items')}})
    result['phase_totals_ms'] = []
    for pair in totals:
        if not isinstance(pair, list) or len(pair) != 2 or pair[0] not in PHASES:
            raise ValueError('invalid phase total')
        result['phase_totals_ms'].append([pair[0], integer(pair[1])])
    return result


class ProgressTracker:
    """Diagnoses a confirmed unchanged local counter window, not a deadlock."""
    def __init__(self):
        self.reset()

    def reset(self):
        self.key = None
        self.since = None

    def observe(self, state, now):
        if not state['active'] or state['phase'] not in ACTIVE_PHASES:
            self.reset()
            return 0.0
        key = tuple(state[k] for k in ('attempt', 'snapshot_attempt', 'phase', 'units', 'items',
                                      'cursor_advances', 'events_applied'))
        if key != self.key:
            self.key, self.since = key, now
        return max(0.0, now - self.since)


async def command(args, timeout, limit=262144):
    process = await asyncio.create_subprocess_exec(
        *args, stdout=asyncio.subprocess.PIPE, stderr=asyncio.subprocess.DEVNULL,
        limit=limit)
    try:
        async def collect():
            # Bound allocation even if a malformed local peer returns huge JSON.
            output = bytearray()
            while chunk := await process.stdout.read(min(16384, limit + 1 - len(output))):
                output.extend(chunk)
                if len(output) > limit:
                    raise ValueError('response_limit')
            if await process.wait():
                raise RuntimeError('call_failed')
            return output.decode('utf-8')
        return await asyncio.wait_for(collect(), timeout)
    finally:
        if process.returncode is None:
            process.kill()  # This is our gdbus reader, never the FSD or worker.
            # A paused stdout transport can keep wait() pending after exit.
            # The killed reader can no longer grow output; drain its pipe.
            await process.communicate()


class Probe:
    def __init__(self, options):
        self.options = options
        self.stop = asyncio.Event()
        self.wake = asyncio.Event()
        self.owner = None
        self.owner_epoch = 0
        self.signals = 0
        self.tracker = ProgressTracker()
        self.rows = self.failures = 0
        self.root = options.output
        self.root.mkdir(mode=0o700, parents=False, exist_ok=False)
        self.log = (self.root / 'observations.jsonl').open('x', encoding='utf-8')

    def emit(self, kind, **fields):
        self.rows += 1
        row = {'time': time.time(), 'monotonic': time.monotonic(), 'kind': kind,
               'owner_epoch': self.owner_epoch, **fields}
        self.log.write(json.dumps(row, ensure_ascii=True) + '\n')
        self.log.flush()

    def set_owner(self, owner):
        if owner == self.owner:
            return
        self.owner = owner
        self.owner_epoch += 1
        self.tracker.reset()
        self.emit('owner', owned=owner is not None)
        self.wake.set()

    async def call(self, owner, method):
        raw = await command(['gdbus', 'call', '--session', '--dest', owner,
                             '--object-path', PATH, '--method', IFACE + '.' + method],
                            self.options.timeout)
        result = ast.literal_eval(raw)
        if not isinstance(result, tuple) or len(result) != 1 or not isinstance(result[0], str):
            raise ValueError('invalid_envelope')
        return json.loads(result[0])

    async def monitor(self):
        while not self.stop.is_set():
            process = await asyncio.create_subprocess_exec(
                'gdbus', 'monitor', '--session', '--dest', self.options.bus,
                '--object-path', PATH, stdout=asyncio.subprocess.PIPE,
                stderr=asyncio.subprocess.DEVNULL, limit=16384)
            try:
                while line := await process.stdout.readline():
                    text = line.decode('utf-8', errors='replace')
                    if match := OWNER.search(text):
                        self.set_owner(match[1])
                    elif 'does not have an owner' in text:
                        self.set_owner(None)
                    elif PROGRESS_SIGNAL.search(text):
                        self.signals += 1
                        self.wake.set()
                await process.wait()
                self.failures += 1
                self.emit('monitor_gap', returncode=process.returncode)
            except (ValueError, UnicodeError):
                self.failures += 1
                self.emit('monitor_gap', error='invalid_monitor_output')
            finally:
                self.set_owner(None)
                if process.returncode is None:
                    process.terminate()
                    try:
                        await asyncio.wait_for(process.wait(), 2)
                    except asyncio.TimeoutError:
                        process.kill()
                        await process.communicate()
            await asyncio.sleep(1)

    async def progress(self):
        previous_signals = 0
        while not self.stop.is_set():
            try:
                await asyncio.wait_for(self.wake.wait(), self.options.interval)
                await asyncio.sleep(.1)
            except asyncio.TimeoutError:
                pass
            self.wake.clear()
            owner, epoch = self.owner, self.owner_epoch
            if owner is None:
                continue
            start = time.monotonic()
            signal_count = self.signals - previous_signals
            previous_signals = self.signals
            try:
                state = sanitize_progress(await self.call(owner, 'GetInitialization'))
                if self.owner_epoch != epoch:
                    self.emit('stale_reply', lane='progress')
                    continue
                unchanged = self.tracker.observe(state, time.monotonic())
                self.emit('progress', latency_ms=(time.monotonic() - start) * 1000,
                          signals=signal_count, progress=state,
                          unchanged_counter_seconds=unchanged,
                          no_counter_progress=unchanged >= self.options.stall_seconds)
            except (RuntimeError, ValueError, TypeError, SyntaxError, asyncio.TimeoutError, UnicodeError):
                self.failures += 1
                self.tracker.reset()  # A gap cannot prove continuous non-progress.
                self.emit('query_failed', lane='progress', error='timeout' if time.monotonic() - start >= self.options.timeout else 'invalid_or_failed_response')

    async def overview(self):
        while not self.stop.is_set():
            owner, epoch = self.owner, self.owner_epoch
            if owner is not None:
                start = time.monotonic()
                try:
                    state = await self.call(owner, 'GetOverview')
                    if self.owner_epoch == epoch:
                        connection = state.get('connection')
                        if connection not in {'locked', 'offline', 'online', 'connecting', 'paused', 'error', 'stopped'}:
                            connection = 'unknown'
                        self.emit('overview', latency_ms=(time.monotonic() - start) * 1000,
                                  connection=connection, mounted=state.get('mounted') is True,
                                  has_error=state.get('error') is not None,
                                  pending_operations=integer(state.get('pending_operations', 0)))
                except (RuntimeError, ValueError, SyntaxError, asyncio.TimeoutError, UnicodeError, AttributeError):
                    self.failures += 1
                    self.emit('query_failed', lane='overview', error='timeout' if time.monotonic() - start >= self.options.timeout else 'invalid_or_failed_response')
            await asyncio.sleep(self.options.overview_interval)

    async def run(self):
        for sig in (signal.SIGINT, signal.SIGTERM):
            asyncio.get_running_loop().add_signal_handler(sig, self.stop.set)
        self.emit('start', schema=1, duration_seconds=self.options.seconds,
                  interval_seconds=self.options.interval, timeout_seconds=self.options.timeout,
                  controls_backend=False, reads_nas=False)
        tasks = [asyncio.create_task(f()) for f in (self.monitor, self.progress, self.overview)]
        ended_by = 'window_complete'
        def lane_done(task):
            nonlocal ended_by
            if not task.cancelled() and task.exception() is not None:
                self.failures += 1
                ended_by = 'observer_failure'
                self.emit('observer_failed', error='local_observer_task_failed')
                self.stop.set()
        for task in tasks:
            task.add_done_callback(lane_done)
        try:
            await asyncio.wait_for(self.stop.wait(), self.options.seconds)
            if ended_by != 'observer_failure':
                ended_by = 'signal'
        except asyncio.TimeoutError:
            pass
        finally:
            self.stop.set()
            for task in tasks:
                task.cancel()
            await asyncio.gather(*tasks, return_exceptions=True)
            self.emit('end', reason=ended_by, query_failures=self.failures)
            self.log.close()
            (self.root / 'completed.json').write_text(json.dumps({
                'schema': 1, 'reason': ended_by, 'rows': self.rows, 'query_failures': self.failures,
                'nas_validation': False, 'backend_controlled': False}) + '\n')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--bus', required=True)
    parser.add_argument('--output', type=pathlib.Path, required=True)
    parser.add_argument('--seconds', type=float, default=3600)
    parser.add_argument('--interval', type=float, default=2)
    parser.add_argument('--overview-interval', type=float, default=10)
    parser.add_argument('--timeout', type=float, default=3)
    parser.add_argument('--stall-seconds', type=float, default=300)
    options = parser.parse_args()
    if not re.fullmatch(r'org\.drivenative\.[A-Za-z0-9_.]+', options.bus):
        parser.error('use a Drive Native well-known bus name')
    for name, low, high in [('seconds', 1, 86400), ('interval', .1, 60),
                            ('overview_interval', .1, 120), ('timeout', .1, 30),
                            ('stall_seconds', .1, 3600)]:
        if not low <= getattr(options, name) <= high:
            parser.error('timing parameter out of range')
    os.umask(0o077)
    asyncio.run(Probe(options).run())


if __name__ == '__main__':
    main()
