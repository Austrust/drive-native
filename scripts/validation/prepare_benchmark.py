"""Create a fresh synthetic-only benchmark; optionally start a user service.

Never loads a real account or unlocks the network worker. Existing output paths
are refused. The generated service remains available for explicit comparisons.
"""
import argparse
import hashlib
import json
import pathlib
import sqlite3
import subprocess
import uuid


def prepare(builder, fsd, output, runtime_commit, start=False):
    if not output.is_absolute() or output.parent.resolve() != output.parent:
        raise ValueError('output must have an existing canonical absolute parent')
    output.mkdir(mode=0o700, exist_ok=False)
    with (output/'setup.log').open('w') as log:
        subprocess.run([str(builder.resolve()), str(output/'fixture')], check=True, stdout=log)
    fixture = json.loads((output/'fixture/fixture.json').read_text())
    task = str(uuid.uuid4())
    instance = task.replace('-', '')
    state = output/'data/drive-native'/task
    state.parent.mkdir(parents=True)
    (output/'fixture/state').rename(state)
    state.chmod(0o700)
    mount = output/'mount'
    mount.mkdir(mode=0o700)
    with sqlite3.connect(state.joinpath('state.sqlite').as_uri()+'?mode=ro', uri=True) as db:
        if db.execute('SELECT count(*) FROM nodes').fetchone()[0] != 210000:
            raise ValueError('unexpected fixture size')
        if db.execute("SELECT count(*) FROM nodes WHERE parent='d000'").fetchone()[0] != 2000:
            raise ValueError('unexpected fixture directory size')
        blob = db.execute("SELECT blob FROM nodes WHERE id='file-000000'").fetchone()[0]
    profile = {'schema':1, 'task_id':task, 'host':'127.0.0.1:9', 'tls_name':'fixture.invalid',
        'trusted_ca_pem':[], 'username':'local-fixture', 'device_id':str(uuid.uuid4()),
        'scope':{'view_id':1, 'node_id':1, 'file_id':'root', 'path':'/'},
        'mount':str(mount), 'writable':False}
    private = output/'profile.json'
    private.write_text(json.dumps(profile))
    private.chmod(0o600)
    config = {'schema':1, 'bus':'org.drivenative.Client1.i'+instance, 'mount':str(mount),
        'directory':str(mount/fixture['directory']), 'file':str(mount/fixture['cached_file']),
        'blob':str(state/'blobs'/blob), 'state':str(state), 'fixture':fixture,
        'binary_sha256':hashlib.sha256(fsd.read_bytes()).hexdigest(),
        'builder_sha256':hashlib.sha256(builder.read_bytes()).hexdigest(),
        'runtime_source_commit':runtime_commit, 'unit':'drive-native-benchmark-'+instance+'.service',
        'native_network':'disabled: no unlock, no credentials, loopback discard address',
        'interference':'caller must record other running tasks and probes in run context'}
    (output/'config.json').write_text(json.dumps(config, indent=2))
    if start:
        subprocess.run(['systemd-run', '--user', '--unit='+config['unit'], '--property=UMask=0077',
            '--setenv=XDG_DATA_HOME='+str(output/'data'), '--setenv=DRIVE_NATIVE_INSTANCE='+instance,
            str(fsd.resolve()), '--config', str(private)], check=True)
    print(json.dumps({'config':str(output/'config.json'), 'unit':config['unit'], 'started':start}))


if __name__ == '__main__':
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--builder', required=True, type=pathlib.Path)
    p.add_argument('--fsd', required=True, type=pathlib.Path)
    p.add_argument('--output', required=True, type=pathlib.Path)
    p.add_argument('--runtime-commit', required=True)
    p.add_argument('--start', action='store_true')
    a = p.parse_args()
    prepare(a.builder, a.fsd, a.output, a.runtime_commit, a.start)
