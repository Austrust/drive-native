"""Test-only derived-source diagnostic; never replaces immutable candidate results."""
import datetime, difflib, hashlib, json, os, pathlib, shutil, subprocess, threading, time

run = pathlib.Path('/home/drive-native-test/.local/state/drive-native-ultra-validation/g4-009')
out = run / 'clock-diagnostic-001'
assert not out.exists()
out.mkdir()
source = out / 'source'
target = out / 'target'
original = run / 'source'
seed = run.parent / 'g4-007/target'
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
record = {'candidate_commit': '4e6aaf7166fd370076adf43a0dad3e6ecdca36d6',
          'scope': 'diagnostic test-only derived source; original assertions retained; no pass counts certify the candidate',
          'original_source': str(original), 'target': str(target), 'commands': [], 'stage': 'source-copy', 'complete': False}
def save():
    (out / 'summary.json').write_text(json.dumps(record, indent=2) + '\n')
save()
shutil.copytree(original, source, ignore=shutil.ignore_patterns('target', '__pycache__', '*.pyc'))
bridge = source / 'crates/drive-runtime/src/bridge.rs'
before = bridge.read_bytes()
needle = b'            let first = Instant::now();\n            let failed_at = unix_seconds();'
assert before.count(needle) == 1
after = before.replace(needle, b'''            let first = Instant::now();
            let diagnostic_wall_start = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
            let failed_at = unix_seconds();''')
needle = b'''            let (request, _) = transport::receive::<Envelope<Command>>(&mut b).unwrap();
            assert!(first.elapsed() >= Duration::from_millis(900));'''
assert after.count(needle) == 1
after = after.replace(needle, b'''            let (request, _) = transport::receive::<Envelope<Command>>(&mut b).unwrap();
            let diagnostic_elapsed = first.elapsed();
            let diagnostic_wall_end = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
            let diagnostic_unix_seconds = unix_seconds();
            let diagnostic_packet = serde_json::to_value(&request.body).unwrap();
            let diagnostic_kind = diagnostic_packet.as_object()
                .map(|object| object.keys().cloned().collect::<Vec<_>>());
            let diagnostic_failure = {
                let store = peer_store.lock().unwrap();
                store.download_failure("file").unwrap()
                    .map(|failure| serde_json::json!({
                        "attempts": failure.attempts, "retry_at": failure.retry_at,
                    }))
            };
            eprintln!("RETRY_CLOCK_DIAGNOSTIC {}", serde_json::json!({
                "foreground": foreground, "failure_category": failure_category,
                "queued_observer": queued_observer, "packet_top_level_keys": diagnostic_kind,
                "elapsed_before_diagnostic_ns": diagnostic_elapsed.as_nanos(),
                "wall_start_ns": diagnostic_wall_start, "wall_end_ns": diagnostic_wall_end,
                "wall_elapsed_ns": diagnostic_wall_end - diagnostic_wall_start,
                "failed_at": failed_at, "unix_seconds_after_receive": diagnostic_unix_seconds,
                "failure": diagnostic_failure,
            }));
            assert!(first.elapsed() >= Duration::from_millis(900));''')
bridge.write_bytes(after)
(out / 'test-only-instrumentation.patch').write_text(''.join(difflib.unified_diff(before.decode().splitlines(True), after.decode().splitlines(True),
    fromfile='immutable/crates/drive-runtime/src/bridge.rs', tofile='diagnostic/crates/drive-runtime/src/bridge.rs')))
record['instrumentation'] = {'path': 'crates/drive-runtime/src/bridge.rs', 'before_sha256': hashlib.sha256(before).hexdigest(),
    'after_sha256': sha(bridge), 'notice': 'Elapsed captured at original assertion point. Logging and a Store read occur before the original assertion; diagnostic passes are not final-candidate acceptance.'}
changes = []
for p in source.rglob('*'):
    if p.is_file():
        rel = p.relative_to(source)
        if not (original / rel).exists() or sha(p) != sha(original / rel): changes.append(str(rel))
assert changes == ['crates/drive-runtime/src/bridge.rs'], changes
record['changed_source_files'] = changes
record['stage'] = 'seed-registry-artifacts'
save()
products = ('drive_core', 'drive_fs', 'drive_runtime', 'sproto', 'fuser', 'drive-core', 'drive-fs', 'drive-runtime')
copied_bytes = copied_files = 0
for directory in ('debug/deps', 'debug/.fingerprint', 'debug/build'):
    dest = target / directory
    dest.mkdir(parents=True)
    for item in (seed / directory).iterdir():
        if item.name.removeprefix('lib').startswith(products): continue
        if item.is_dir():
            # Fingerprint/build folders for registry dependencies only; no shared links.
            shutil.copytree(item, dest / item.name)
            for p in (dest / item.name).rglob('*'):
                if p.is_file(): copied_files += 1; copied_bytes += p.stat().st_size
        elif item.suffix in ('.rlib', '.rmeta', '.so', '.d'):
            shutil.copy2(item, dest / item.name)
            copied_files += 1; copied_bytes += item.stat().st_size
record['registry_seed'] = {'path': str(seed), 'copied_files': copied_files, 'copied_bytes': copied_bytes,
    'excluded_product_prefixes': products, 'excluded': 'test ELFs, incremental, workspace/vendored product fingerprint/build/deps artifacts'}
for p in source.rglob('*'):
    if p.is_file(): os.utime(p, None)
env = os.environ.copy()
for key in list(env):
    if key.startswith('DRIVE_NATIVE_') or key in {'DBUS_SESSION_BUS_ADDRESS', 'DISPLAY', 'WAYLAND_DISPLAY'}: env.pop(key, None)
env.update(CARGO_TARGET_DIR=str(target), CARGO_BUILD_JOBS='2', RUST_BACKTRACE='1', CARGO_NET_OFFLINE='true')
(out / 'tmp').mkdir()
env['TMPDIR'] = str(out / 'tmp')
def command(name, argv, timeout=1200):
    row = {'name': name, 'argv': argv, 'log': name + '.log', 'utc_start': datetime.datetime.now(datetime.timezone.utc).isoformat()}
    record['commands'].append(row); record['stage'] = name; save()
    started = time.monotonic()
    with (out / row['log']).open('w') as stream:
        proc = subprocess.run(argv, cwd=source, env=env, stdout=stream, stderr=subprocess.STDOUT, timeout=timeout)
    row.update(exit_code=proc.returncode, elapsed_seconds=time.monotonic()-started, log_sha256=sha(out / row['log']))
    save()
    return proc.returncode
assert command('clean-products', ['cargo', 'clean', '-p', 'drive-core', '-p', 'drive-fs', '-p', 'drive-runtime', '-p', 'sproto']) == 0
assert command('build-diagnostic', ['cargo', 'test', '--offline', '--locked', '-p', 'drive-runtime', '--lib', '--all-features', '--no-run', '--message-format=json']) == 0
artifacts = []
for line in (out / 'build-diagnostic.log').read_text().splitlines():
    try: message = json.loads(line)
    except json.JSONDecodeError: continue
    if message.get('reason') == 'compiler-artifact' and message.get('target', {}).get('name') == 'drive_runtime' and message.get('executable'):
        artifacts.append(message)
assert len(artifacts) == 1, len(artifacts)
binary = pathlib.Path(artifacts[0]['executable'])
record['binary'] = {'path': str(binary), 'sha256': sha(binary)}
samples = []
stop = threading.Event()
def sampler():
    while not stop.is_set():
        before = time.monotonic_ns(); wall = time.time_ns(); after = time.monotonic_ns()
        samples.append({'monotonic_before_ns': before, 'realtime_ns': wall, 'monotonic_after_ns': after})
        stop.wait(.01)
thread = threading.Thread(target=sampler); thread.start()
try:
    command('two-original-cases-concurrent', [str(binary), 'concurrent_foreground_reads_wait_for_', '--nocapture', '--test-threads=2'], 60)
    command('runtime-full-original-parallelism', [str(binary), '--nocapture'], 180)
finally:
    stop.set(); thread.join()
    (out / 'clock-samples.json').write_text(json.dumps(samples, indent=2) + '\n')
    differences = []
    for left, right in zip(samples, samples[1:]):
        mono = (right['monotonic_before_ns'] + right['monotonic_after_ns'] - left['monotonic_before_ns'] - left['monotonic_after_ns'])/2
        wall = right['realtime_ns'] - left['realtime_ns']
        differences.append({'wall_delta_ns': wall, 'monotonic_delta_ns': mono, 'difference_ns': wall - mono})
    record['clock'] = {'samples': len(samples), 'read_order': 'monotonic_before -> realtime -> monotonic_after', 'requested_interval_seconds': .01,
        'largest_differences': sorted(differences, key=lambda row: abs(row['difference_ns']), reverse=True)[:10]}
    record['diagnostics'] = []
    for row in record['commands']:
        for line in (out / row['log']).read_text(errors='replace').splitlines():
            if 'RETRY_CLOCK_DIAGNOSTIC ' in line:
                record['diagnostics'].append({'command': row['name'], 'value': json.loads(line.split('RETRY_CLOCK_DIAGNOSTIC ', 1)[1])})
    record['complete'] = True; record['stage'] = 'complete'; save()
    print(json.dumps(record, indent=2))
