"""Explicit local FUSE/D-Bus benchmark; config must identify a synthetic fixture.

Workers start before measured operations. Real directories and credentials are
not accepted by this driver: it requires performance_fixture's fixed manifest.
"""
import argparse
import concurrent.futures
import ctypes
import hashlib
import json
import math
import mmap
import os
import pathlib
import select
import subprocess
import sys
import threading
import time


def residency(path, evict=False):
    with open(path, 'rb') as f:
        if evict:
            os.posix_fadvise(f.fileno(), 0, 0, os.POSIX_FADV_DONTNEED)
        size = os.fstat(f.fileno()).st_size
        with mmap.mmap(f.fileno(), size, flags=mmap.MAP_PRIVATE,
                       prot=mmap.PROT_READ | mmap.PROT_WRITE) as area:
            pages = math.ceil(size / mmap.PAGESIZE)
            vector = (ctypes.c_ubyte * pages)()
            address = ctypes.addressof(ctypes.c_char.from_buffer(area))
            libc = ctypes.CDLL(None, use_errno=True)
            if libc.mincore(ctypes.c_void_p(address), ctypes.c_size_t(size), vector):
                raise OSError(ctypes.get_errno(), 'mincore failed')
            return sum(bool(v & 1) for v in vector), pages


def worker(config, kind):
    from gi.repository import Gio, GLib
    started = time.monotonic()
    cfg = json.loads(pathlib.Path(config).read_text())
    conn = Gio.bus_get_sync(Gio.BusType.SESSION, None) if kind == 'status' else None
    held = open(cfg['file'], 'rb') if kind == 'held_read' else None
    ready = {'ready': True, 'setup_seconds': time.monotonic() - started}
    if kind in ('warm_read', 'disk_read'):
        if kind == 'warm_read':
            with open(cfg['blob'], 'rb') as f:
                while f.read(1024 * 1024):
                    pass
        resident, total = residency(cfg['blob'], evict=kind == 'disk_read')
        ready.update(resident_pages=resident, total_pages=total,
                     cache_condition_met=resident == (total if kind == 'warm_read' else 0))
    print(json.dumps(ready), flush=True)
    if sys.stdin.readline().strip() != 'run':
        return
    start = time.monotonic()
    result = {'start': time.time()}
    try:
        if kind == 'directory':
            names = os.listdir(cfg['directory'])
            result['ok'] = len(names) == 2000 and set(names) == {f'f{i:05}' for i in range(2000)}
        elif kind == 'status':
            response = conn.call_sync(cfg['bus'], '/org/drivenative/Client1',
                'org.drivenative.Client1', 'GetStates', GLib.Variant('(as)', ([cfg['mount']],)),
                GLib.VariantType.new('(s)'), Gio.DBusCallFlags.NONE, 15000, None)
            data = json.loads(response.unpack()[0])
            result['ok'] = len(data['entries']) == 1 and data['entries'][0].get('error') is None
        else:
            f = held if held is not None else open(cfg['file'], 'rb')
            result['open_s'] = time.monotonic() - start
            h = hashlib.sha256()
            count = 0
            try:
                b = f.read(65536)
                result['first_byte_s'] = time.monotonic() - start
                while b:
                    count += len(b)
                    h.update(b)
                    b = f.read(65536)
                result['content_complete_s'] = time.monotonic() - start
            finally:
                f.close()
            result['ok'] = count == cfg['fixture']['cached_bytes'] and h.hexdigest() == cfg['fixture']['content_sha256']
        if ready.get('cache_condition_met') is False:
            result.update(ok=False, error='cache_condition_not_established')
    except Exception as exc:
        result.update(ok=False, error=type(exc).__name__)
    result['seconds'] = time.monotonic() - start
    print(json.dumps(result), flush=True)


def percentile_summary(rows):
    values = sorted(r['seconds'] for r in rows)
    return {'n': len(rows), 'errors': sum(not r['ok'] for r in rows),
            'timeouts': sum(r.get('error') == 'timeout' for r in rows),
            **{f'p{q}_s': values[math.ceil(len(values)*q/100)-1] for q in (50,95,99)},
            'max_s': max(values)}


def one(config, kind, index, barrier=None):
    launch = time.monotonic()
    row = {'kind':kind, 'index':index}
    proc = subprocess.Popen([sys.executable, __file__, '--worker', kind, '--config', str(config)],
                            stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, text=True)
    began = None
    try:
        if not select.select([proc.stdout], [], [], 20)[0]:
            raise TimeoutError('setup')
        ready = json.loads(proc.stdout.readline())
        if not ready.get('ready'):
            raise ValueError('worker not ready')
        row.update(ready)
        row['process_ready_s'] = time.monotonic() - launch
        if barrier is not None:
            barrier.wait(timeout=25)
        row['parent_start'] = time.time()
        began = time.monotonic()
        stdout, _ = proc.communicate('run\n', timeout=15)
        row['parent_seconds'] = time.monotonic() - began
        row.update(json.loads(stdout))
    except subprocess.TimeoutExpired:
        row.update(ok=False, error='timeout', censored=True, seconds=time.monotonic()-began)
    except Exception as exc:
        if barrier is not None:
            barrier.abort()
        row.update(ok=False, error='setup_'+type(exc).__name__, seconds=time.monotonic()-launch)
    finally:
        if proc.poll() is None:
            proc.kill()
        try:
            proc.communicate(timeout=3)
        except subprocess.TimeoutExpired:
            row['cleanup_pending_pid'] = proc.pid
    return row


def benchmark(config, output, phase):
    cfg = json.loads(config.read_text())
    assert cfg['fixture']['nodes'] == 210000 and cfg['fixture']['directory_children'] == 2000
    assert cfg['fixture']['purpose'].startswith('local fixed workload;')
    assert cfg['native_network'].startswith('disabled:')
    output.mkdir(parents=True, exist_ok=False)
    mixed = phase in ('mixed_open', 'mixed_held')
    kinds = ['directory', 'status', 'held_read' if phase == 'mixed_held' else 'cached_read']*16 if mixed else [phase]
    manifest = {'registered_at':time.time(), 'phase':phase, 'samples':240 if mixed else 30,
        'lanes':48 if mixed else 1,'rounds':5 if mixed else 30,'warmup_per_kind':2,'timeout_s':15,
        'fixture':cfg['fixture'],'binary_sha256':cfg['binary_sha256'],
        'runtime_source_commit':cfg['runtime_source_commit'], 'interference':cfg['interference'],
        'native_network':cfg['native_network'], 'timing':'worker operation only; process start and parent wall latency separate; errors/timeouts retained',
        'cache':'warm/disk groups verify backing-blob residency with mincore; FUSE DIRECT_IO; lower block/hypervisor storage cache uncontrolled',
        'percentiles':'nearest-rank; timeout duration is a right-censored lower bound',
        'runner_lf_sha256':hashlib.sha256(pathlib.Path(__file__).read_bytes().replace(b'\r\n',b'\n')).hexdigest()}
    (output/'manifest.json').write_text(json.dumps(manifest, indent=2))
    warmup = [one(config, kind, i) for kind in sorted(set(kinds)) for i in range(2)]
    (output/'warmup.json').write_text(json.dumps(warmup, indent=2))
    rows = []
    with concurrent.futures.ThreadPoolExecutor(max_workers=len(kinds)) as pool:
        for rnd in range(manifest['rounds']):
            barrier = threading.Barrier(len(kinds)) if mixed else None
            futures = [pool.submit(one, config, kind, rnd*len(kinds)+i, barrier) for i,kind in enumerate(kinds)]
            batch = [f.result() for f in futures]
            rows.extend(batch)
            with (output/'samples.jsonl').open('a') as f:
                for row in batch:
                    f.write(json.dumps(row)+'\n')
            if mixed or (rnd+1)%10 == 0:
                print(json.dumps({'phase':phase,'round':rnd+1,'errors':sum(not r['ok'] for r in rows)}), flush=True)
    summaries = {kind:percentile_summary([r for r in rows if r['kind']==kind]) for kind in sorted(set(kinds))}
    summaries['all'] = percentile_summary(rows)
    (output/'summary.json').write_text(json.dumps(summaries, indent=2))
    print(json.dumps(summaries), flush=True)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--config', required=True, type=pathlib.Path)
    parser.add_argument('--output', type=pathlib.Path)
    parser.add_argument('--worker', choices=['directory','status','cached_read','held_read','warm_read','disk_read'])
    parser.add_argument('--phase', choices=['directory','status','warm_read','disk_read','held_read','mixed_open','mixed_held'])
    args = parser.parse_args()
    if args.worker:
        worker(args.config, args.worker)
    else:
        if args.phase is None or args.output is None:
            parser.error('benchmark needs --phase and --output')
        benchmark(args.config, args.output, args.phase)
