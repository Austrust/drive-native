//! Direction D (#59 FUSE data plane) perf probe: mixed slow data-plane reads
//! vs metadata (stat/readdir/cache-hit read) latency on a real kernel FUSE
//! mount. Synthetic fixture only; no NAS, no user directories.
//!
//! Not part of the product suite (`#[ignore]`). Run explicitly, serially:
//!   DRIVE_NATIVE_FS_DIAGNOSTICS=1 DRIVE_NATIVE_FUSE_DEADLINE_METADATA_MS=300 \
//!     cargo test -p drive-fs --release --all-features --locked \
//!     --test perf_isolation_probe -- --ignored --nocapture --test-threads=1
//!
//! Cases (one process, one mount, same fixture):
//!   control    : no slow reads (baseline)
//!   slow1      : 1 slow cache-miss read (H2: data lane internal isolation)
//!   slow16     : 16 concurrent slow cache-miss reads (the #59 AC target)
//!   slow64     : 64 concurrent slow cache-miss reads (>= max_background)
//!   store_hold : shared Store held 4 s while metadata probes run (H3/EAGAIN)
//!
//! The "slow download" is a test-only Hydrator that sleeps and then fails with
//! EIO: the kernel READ request stays in flight for the sleep duration (same
//! queue/slot occupancy as a real slow network download) without polluting the
//! local content cache, so the pressure can be sustained indefinitely.

use drive_core::{
    sha256, DownloadTicket, Node, NodeMetadata, ReadHandle, SharedStore, Store,
};
use drive_fs::{CacheFilesystem, Hydrator, ReadCancellation};
use std::collections::BTreeSet;
use std::fs::File;
use std::io::Read;
use std::os::fd::AsRawFd;
use std::os::unix::fs::FileExt;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc;
use std::time::{Duration, Instant};

const ROOT: &str = "root";
const DIR: &str = "d000";
const CACHED: usize = 64;
const SLOW: usize = 96;
const CACHED_BYTES: usize = 256 * 1024;
const READ_SIZE: u32 = 64 * 1024;
const SLOW_MS: u64 = 1500;
const METADATA_BUDGET_MS: u64 = 300;
const STAT_INTERVAL: Duration = Duration::from_millis(2);
const READDIR_INTERVAL: Duration = Duration::from_millis(5);
const READ_INTERVAL: Duration = Duration::from_millis(2);
const CASE_SECS: u64 = 10;
const STORE_HOLD_MS: u64 = 4000;

fn mknode(id: &str, parent: Option<&str>, name: &str, directory: bool, size: u64) -> Node {
    Node {
        id: id.into(),
        parent: parent.map(Into::into),
        name: name.into(),
        version: "1".into(),
        directory,
        metadata: Some(NodeMetadata {
            size,
            mtime: 1_700_000_000,
            mode: if directory { 0o755 } else { 0o644 },
        }),
    }
}

struct SlowHydrator {
    wait: Duration,
    started: AtomicU64,
    finished: AtomicU64,
}
impl SlowHydrator {
    fn new(wait: Duration) -> Self {
        Self {
            wait,
            started: AtomicU64::new(0),
            finished: AtomicU64::new(0),
        }
    }
    fn run(&self) -> Result<Arc<ReadHandle>, i32> {
        self.started.fetch_add(1, Ordering::Relaxed);
        std::thread::sleep(self.wait);
        self.finished.fetch_add(1, Ordering::Relaxed);
        // The READ request above stays unanswered for `wait`; the content is
        // deliberately absent so the same handle keeps going to hydration.
        Err(libc::EIO)
    }
}
impl Hydrator for SlowHydrator {
    fn hydrate(&self, _node: &Node) -> Result<Arc<ReadHandle>, i32> {
        self.run()
    }
    fn hydrate_cancellable(
        &self,
        _node: &Node,
        cancel: &ReadCancellation,
    ) -> Result<Arc<ReadHandle>, i32> {
        if cancel.is_cancelled() {
            return Err(libc::EINTR);
        }
        self.run()
    }
}

#[derive(Default)]
struct Latencies {
    samples: Vec<u64>,
    eagain: u64,
    other_errors: u64,
}
impl Latencies {
    fn record(&mut self, elapsed: Duration, result: Result<(), i32>) {
        self.samples.push(elapsed.as_micros() as u64);
        match result {
            Ok(()) => {}
            Err(libc::EAGAIN) => self.eagain += 1,
            Err(_) => self.other_errors += 1,
        }
    }
    fn merge(&mut self, other: Latencies) {
        self.samples.extend(other.samples);
        self.eagain += other.eagain;
        self.other_errors += other.other_errors;
    }
    fn quantile(sorted: &[u64], pct: u64) -> u64 {
        if sorted.is_empty() {
            return 0;
        }
        let rank = ((sorted.len() as u64 * pct).div_ceil(100)).max(1) as usize - 1;
        sorted[rank.min(sorted.len() - 1)]
    }
    fn report(&self, case: &str, probe: &str) {
        let mut sorted = self.samples.clone();
        sorted.sort_unstable();
        println!(
            "PERF case={case} probe={probe} n={} p50_us={} p95_us={} p99_us={} max_us={} eagain={} err={}",
            sorted.len(),
            Self::quantile(&sorted, 50),
            Self::quantile(&sorted, 95),
            Self::quantile(&sorted, 99),
            sorted.last().copied().unwrap_or(0),
            self.eagain,
            self.other_errors
        );
    }
}

fn errno(error: std::io::Error) -> i32 {
    error.raw_os_error().unwrap_or(-1)
}

fn rate_limit(next: &mut Instant, interval: Duration) {
    let now = Instant::now();
    if *next > now {
        std::thread::sleep(*next - now);
    }
    *next = Instant::now() + interval;
}

fn probe_stat(mount: PathBuf, stop: Arc<AtomicBool>) -> Latencies {
    let mut lat = Latencies::default();
    let mut next = Instant::now();
    let mut i = 0usize;
    while !stop.load(Ordering::Relaxed) {
        let path = mount.join(DIR).join(format!("c{:03}", i % CACHED));
        let started = Instant::now();
        let result = std::fs::metadata(&path).map(|_| ());
        lat.record(started.elapsed(), result.map_err(errno));
        i += 1;
        rate_limit(&mut next, STAT_INTERVAL);
    }
    lat
}

fn probe_readdir(mount: PathBuf, stop: Arc<AtomicBool>, expected: usize) -> Latencies {
    let dir = mount.join(DIR);
    let mut lat = Latencies::default();
    let mut next = Instant::now();
    while !stop.load(Ordering::Relaxed) {
        let started = Instant::now();
        let result = std::fs::read_dir(&dir)
            .and_then(|entries| entries.collect::<std::io::Result<Vec<_>>>())
            .map(|entries| entries.len() == expected);
        let outcome = match result {
            Ok(true) => Ok(()),
            Ok(false) => Err(-2),
            Err(error) => Err(errno(error)),
        };
        lat.record(started.elapsed(), outcome);
        rate_limit(&mut next, READDIR_INTERVAL);
    }
    lat
}

/// Rotating 4 KiB pread on one cached file. With `fadvise` the just-read page is
/// evicted, so the next read of that offset must issue FUSE READ again.
fn probe_small_read(file: File, fadvise: bool, stop: Arc<AtomicBool>) -> Latencies {
    let mut lat = Latencies::default();
    let mut next = Instant::now();
    let mut buf = [0u8; 4096];
    let mut i = 0u64;
    while !stop.load(Ordering::Relaxed) {
        let offset = (i % (CACHED_BYTES as u64 / 4096)) * 4096;
        let started = Instant::now();
        let result = file.read_at(&mut buf, offset).map(|_| ());
        lat.record(started.elapsed(), result.map_err(errno));
        if fadvise {
            unsafe {
                libc::posix_fadvise(
                    file.as_raw_fd(),
                    offset as libc::off_t,
                    4096,
                    libc::POSIX_FADV_DONTNEED,
                );
            }
        }
        i += 1;
        rate_limit(&mut next, READ_INTERVAL);
    }
    lat
}

fn spawn_downloader(path: PathBuf, stop: Arc<AtomicBool>) -> std::thread::JoinHandle<u64> {
    std::thread::spawn(move || {
        let mut buf = vec![0u8; READ_SIZE as usize];
        let mut iterations = 0u64;
        let mut file: Option<File> = None;
        while !stop.load(Ordering::Relaxed) {
            if file.is_none() {
                match File::open(&path) {
                    Ok(f) => file = Some(f),
                    Err(_) => {
                        std::thread::sleep(Duration::from_millis(10));
                        continue;
                    }
                }
            }
            let f = file.as_mut().unwrap();
            // Blocks until the slow hydrator replies (EIO after SLOW_MS).
            let _ = f.read(&mut buf);
            iterations += 1;
        }
        iterations
    })
}

fn fuse_connections() -> BTreeSet<u32> {
    std::fs::read_dir("/sys/fs/fuse/connections")
        .map(|entries| {
            entries
                .filter_map(|entry| entry.ok())
                .filter_map(|entry| entry.file_name().to_str().and_then(|n| n.parse().ok()))
                .collect()
        })
        .unwrap_or_default()
}

fn wait_for_new_connection(before: &BTreeSet<u32>) -> Option<u32> {
    let started = Instant::now();
    while started.elapsed() < Duration::from_secs(2) {
        let now = fuse_connections();
        let new: Vec<u32> = now.difference(before).copied().collect();
        match new.as_slice() {
            [single] => return Some(*single),
            [] => std::thread::sleep(Duration::from_millis(10)),
            _ => return None,
        }
    }
    None
}

fn sysfs_u16(connection: u32, name: &str) -> u16 {
    std::fs::read_to_string(format!("/sys/fs/fuse/connections/{connection}/{name}"))
        .ok()
        .and_then(|text| text.trim().parse().ok())
        .unwrap_or(0)
}

fn proc_usage() -> (u64, u64) {
    let mut usage: libc::rusage = unsafe { std::mem::zeroed() };
    unsafe { libc::getrusage(libc::RUSAGE_SELF, &mut usage) };
    let to_us = |tv: libc::timeval| tv.tv_sec as u64 * 1_000_000 + tv.tv_usec as u64;
    (to_us(usage.ru_utime), to_us(usage.ru_stime))
}

fn proc_io() -> (u64, u64) {
    let text = std::fs::read_to_string("/proc/self/io").unwrap_or_default();
    let field = |name: &str| {
        text.lines()
            .find_map(|line| line.strip_prefix(name))
            .and_then(|rest| rest.trim().parse::<u64>().ok())
            .unwrap_or(0)
    };
    (field("rchar:"), field("wchar:"))
}

fn thread_count() -> u64 {
    std::fs::read_to_string("/proc/self/status")
        .unwrap_or_default()
        .lines()
        .find_map(|line| line.strip_prefix("Threads:"))
        .and_then(|rest| rest.trim().parse().ok())
        .unwrap_or(0)
}

struct Counters {
    user_us: u64,
    sys_us: u64,
    rchar: u64,
    wchar: u64,
    threads: u64,
}
fn counters_now() -> Counters {
    let (user_us, sys_us) = proc_usage();
    let (rchar, wchar) = proc_io();
    Counters {
        user_us,
        sys_us,
        rchar,
        wchar,
        threads: thread_count(),
    }
}

fn loadavg() -> String {
    std::fs::read_to_string("/proc/loadavg")
        .unwrap_or_default()
        .split_whitespace()
        .next()
        .unwrap_or("?")
        .to_owned()
}

struct ProbeThreads {
    handles: Vec<(&'static str, std::thread::JoinHandle<Latencies>)>,
    stop: Arc<AtomicBool>,
}
impl ProbeThreads {
    fn spawn(mount: PathBuf, cached_file: PathBuf, expected: usize) -> Self {
        let stop = Arc::new(AtomicBool::new(false));
        let mut handles: Vec<(&'static str, std::thread::JoinHandle<Latencies>)> = Vec::new();
        for _ in 0..2 {
            let stop = stop.clone();
            let mount = mount.clone();
            handles.push((
                "stat",
                std::thread::spawn(move || probe_stat(mount, stop)),
            ));
        }
        {
            let stop = stop.clone();
            let mount = mount.clone();
            handles.push((
                "readdir",
                std::thread::spawn(move || probe_readdir(mount, stop, expected)),
            ));
        }
        for label in ["read_fadvise", "read_fadvise2"] {
            let file = File::open(&cached_file).unwrap();
            let stop = stop.clone();
            handles.push((
                label,
                std::thread::spawn(move || probe_small_read(file, true, stop)),
            ));
        }
        {
            let file = File::open(&cached_file).unwrap();
            let stop = stop.clone();
            handles.push((
                "read_pagecache",
                std::thread::spawn(move || probe_small_read(file, false, stop)),
            ));
        }
        Self { handles, stop }
    }
    /// Join probes, merging same-kind threads.
    fn finish(self) -> Vec<(&'static str, Latencies)> {
        self.stop.store(true, Ordering::Relaxed);
        let mut merged: Vec<(&'static str, Latencies)> = Vec::new();
        for (label, handle) in self.handles {
            let lat = handle.join().unwrap();
            match merged.iter_mut().find(|(name, _)| *name == label) {
                Some((_, slot)) => slot.merge(lat),
                None => merged.push((label, lat)),
            }
        }
        merged
    }
}

// [METRICS-BEGIN]
fn dump_request_metrics(case: &str) {
    let m = drive_fs::request::snapshot();
    println!(
        "PERF case={case} metrics request_timeouts={} retryable_replies={} lock_wait_timeouts={} degraded_events={} lock_wait_n={} lock_wait_p95_us={} configured={:?} effective={:?}",
        m.request_timeouts,
        m.retryable_replies,
        m.lock_wait_timeouts,
        m.degraded_events,
        m.lock_wait.count,
        m.lock_wait.p95_us,
        m.configured_tuning,
        m.effective_tuning
    );
    for op in m.ops.iter().filter(|op| op.latency.count > 0) {
        println!(
            "PERF case={case} op={} count={} p50_us={} p95_us={} p99_us={} max_us={} timeouts={} retryable={}",
            op.op,
            op.latency.count,
            op.latency.p50_us,
            op.latency.p95_us,
            op.latency.p99_us,
            op.latency.max_us,
            op.timeouts,
            op.retryable
        );
    }
}
// [METRICS-END]

fn report_case(
    case: &str,
    probes: Vec<(&'static str, Latencies)>,
    before: &Counters,
    after: &Counters,
    waiting: (u64, u64, u64),
    hydrator: (u64, u64),
    downloader_iterations: u64,
) {
    for (probe, lat) in probes {
        lat.report(case, probe);
    }
    println!(
        "PERF case={case} self cpu_user_ms={} cpu_sys_ms={} rchar={} wchar={} threads={}",
        (after.user_us - before.user_us) / 1000,
        (after.sys_us - before.sys_us) / 1000,
        after.rchar - before.rchar,
        after.wchar - before.wchar,
        after.threads,
    );
    println!(
        "PERF case={case} fuse waiting_samples={} waiting_max={} waiting_avg_milli={}",
        waiting.0,
        waiting.1,
        if waiting.0 == 0 {
            0
        } else {
            waiting.2 * 1000 / waiting.0
        }
    );
    println!(
        "PERF case={case} hydrator started_delta={} finished_delta={} downloader_iterations={}",
        hydrator.0, hydrator.1, downloader_iterations
    );
    dump_request_metrics(case);
}

fn run_case(
    mount: &PathBuf,
    cached_file: &PathBuf,
    connection: u32,
    hydrator: &Arc<SlowHydrator>,
    expected_dir_entries: usize,
    name: &str,
    concurrency: usize,
    duration: Duration,
    detach_downloaders: bool,
) {
    println!(
        "PERF case={name} start loadavg={} conc={concurrency} slow_ms={SLOW_MS} duration_ms={}",
        loadavg(),
        duration.as_millis()
    );
    let before = counters_now();
    let hydrator_before = (
        hydrator.started.load(Ordering::Relaxed),
        hydrator.finished.load(Ordering::Relaxed),
    );
    let stop = Arc::new(AtomicBool::new(false));
    let mut downloaders = Vec::new();
    for i in 0..concurrency {
        let path = mount.join(DIR).join(format!("s{:03}", i % SLOW));
        downloaders.push(spawn_downloader(path, stop.clone()));
    }
    if !downloaders.is_empty() {
        // Let every downloader enter its first blocked READ before probing.
        std::thread::sleep(Duration::from_millis(400));
    }
    let probes = ProbeThreads::spawn(mount.clone(), cached_file.clone(), expected_dir_entries);
    let wait_stop = Arc::new(AtomicBool::new(false));
    let sampler = {
        let wait_stop = wait_stop.clone();
        std::thread::spawn(move || {
            let path = format!("/sys/fs/fuse/connections/{connection}/waiting");
            let (mut count, mut max, mut sum) = (0u64, 0u64, 0u64);
            while !wait_stop.load(Ordering::Relaxed) {
                if let Ok(text) = std::fs::read_to_string(&path) {
                    if let Ok(value) = text.trim().parse::<u64>() {
                        count += 1;
                        max = max.max(value);
                        sum += value;
                    }
                }
                std::thread::sleep(Duration::from_millis(4));
            }
            (count, max, sum)
        })
    };
    std::thread::sleep(duration);
    let probe_results = probes.finish();
    wait_stop.store(true, Ordering::Relaxed);
    let waiting = sampler.join().unwrap();
    stop.store(true, Ordering::Relaxed);
    let mut iterations = 0u64;
    for handle in downloaders {
        if detach_downloaders {
            // Session destroy cancels the in-flight reads and the threads exit
            // on their own; waiting for the full network backlog would add up
            // to concurrency * slow_ms / 4 seconds to every repetition.
            std::mem::forget(handle);
        } else {
            iterations += handle.join().unwrap();
        }
    }
    let after = counters_now();
    let hydrator_after = (
        hydrator.started.load(Ordering::Relaxed) - hydrator_before.0,
        hydrator.finished.load(Ordering::Relaxed) - hydrator_before.1,
    );
    report_case(
        name,
        probe_results,
        &before,
        &after,
        waiting,
        hydrator_after,
        iterations,
    );
}

fn run_store_hold(
    mount: &PathBuf,
    cached_file: &PathBuf,
    store: &Arc<SharedStore>,
) {
    println!(
        "PERF case=store_hold start loadavg={} hold_ms={STORE_HOLD_MS} budget_ms={METADATA_BUDGET_MS}",
        loadavg()
    );
    let before = counters_now();
    let (tx, rx) = mpsc::channel();
    let holder_store = store.clone();
    let holder = std::thread::spawn(move || {
        let guard = holder_store.lock().unwrap();
        let _ = tx.send(());
        std::thread::sleep(Duration::from_millis(STORE_HOLD_MS));
        drop(guard);
    });
    rx.recv().unwrap();
    // Probe while the Store is held: t4 (>budget) must answer EAGAIN, base
    // blocks unbounded until release.
    let stop = Arc::new(AtomicBool::new(false));
    let mut handles = Vec::new();
    for _ in 0..2 {
        let stop = stop.clone();
        let mount = mount.clone();
        handles.push(std::thread::spawn(move || probe_stat(mount, stop)));
    }
    std::thread::sleep(Duration::from_millis(STORE_HOLD_MS - 500));
    stop.store(true, Ordering::Relaxed);
    let mut held = Latencies::default();
    for handle in handles {
        held.merge(handle.join().unwrap());
    }
    holder.join().unwrap();
    held.report("store_hold", "stat_while_store_held");

    // Recovery: same probes after the guard is dropped must succeed again.
    let recovery_before = counters_now();
    let stop = Arc::new(AtomicBool::new(false));
    let handle = {
        let stop = stop.clone();
        let mount = mount.clone();
        std::thread::spawn(move || probe_stat(mount, stop))
    };
    std::thread::sleep(Duration::from_millis(1500));
    stop.store(true, Ordering::Relaxed);
    let recovered = handle.join().unwrap();
    recovered.report("store_hold", "stat_after_release");
    let after = counters_now();
    report_case(
        "store_hold",
        Vec::new(),
        &recovery_before,
        &after,
        (0, 0, 0),
        (0, 0),
        0,
    );
    let _ = before;
    let _ = cached_file;
    dump_request_metrics("store_hold_summary");
}

#[test]
#[ignore = "requires ordinary Linux user and /dev/fuse; perf probe, run explicitly"]
fn mixed_slow_reads_vs_metadata_latency() {
    let uid = unsafe { libc::getuid() };
    let gid = unsafe { libc::getgid() };
    assert_ne!(uid, 0, "run as the unprivileged test user");
    // Issue #59 semantics: shorten the metadata budget so bounded-wait
    // degradation is observable inside one case.
    unsafe {
        std::env::set_var(
            "DRIVE_NATIVE_FUSE_DEADLINE_METADATA_MS",
            METADATA_BUDGET_MS.to_string(),
        );
    }

    let dir = tempfile::tempdir().unwrap();
    let mount = dir.path().join("mount");
    std::fs::create_dir(&mount).unwrap();
    let mut store = Store::open(dir.path().join("state")).unwrap();
    let mut nodes = vec![
        mknode(ROOT, None, "root", true, 0),
        mknode(DIR, Some(ROOT), DIR, true, 0),
    ];
    for i in 0..CACHED {
        let id = format!("c{i:03}");
        nodes.push(mknode(&id, Some(DIR), &id, false, CACHED_BYTES as u64));
    }
    for i in 0..SLOW {
        let id = format!("s{i:03}");
        nodes.push(mknode(&id, Some(DIR), &id, false, READ_SIZE as u64));
    }
    store
        .install_remote_snapshot(&nodes, "1", "perf-probe")
        .unwrap();
    for i in 0..CACHED {
        let id = format!("c{i:03}");
        let bytes = vec![(i % 251) as u8; CACHED_BYTES];
        store
            .publish_download(
                &DownloadTicket {
                    id: id.clone(),
                    version: "1".into(),
                },
                &bytes,
                &sha256(&bytes),
            )
            .unwrap();
    }
    let store = Arc::new(SharedStore::new(store));
    let hydrator = Arc::new(SlowHydrator::new(Duration::from_millis(SLOW_MS)));
    let fs = CacheFilesystem::new(store.clone(), ROOT, uid, gid)
        .unwrap()
        .with_hydrator(hydrator.clone())
        .with_metadata_ttl_ms(0);
    let before = fuse_connections();
    let session = fuser::spawn_mount2(
        fs,
        &mount,
        &[
            fuser::MountOption::RO,
            fuser::MountOption::FSName("drive-native-perf-probe".into()),
        ],
    )
    .unwrap();
    // Trigger FUSE_INIT so the connection (and its tuning) exists.
    std::fs::metadata(&mount).unwrap();
    let connection = wait_for_new_connection(&before).expect("no new FUSE connection");
    println!(
        "PERF mount connection={connection} max_background={} congestion_threshold={}",
        sysfs_u16(connection, "max_background"),
        sysfs_u16(connection, "congestion_threshold")
    );
    let cached_file = mount.join(DIR).join("c000");
    let expected_dir_entries = CACHED + SLOW;

    // Warm dentries, the directory view and the page cache.
    {
        let stop = Arc::new(AtomicBool::new(false));
        let stat = {
            let stop = stop.clone();
            let mount = mount.clone();
            std::thread::spawn(move || probe_stat(mount, stop))
        };
        let read = {
            let stop = stop.clone();
            let file = File::open(&cached_file).unwrap();
            std::thread::spawn(move || probe_small_read(file, true, stop))
        };
        std::thread::sleep(Duration::from_secs(2));
        stop.store(true, Ordering::Relaxed);
        let merged = {
            let mut m = Latencies::default();
            m.merge(stat.join().unwrap());
            m.merge(read.join().unwrap());
            m
        };
        merged.report("warmup", "stat+read");
    }

    run_case(
        &mount,
        &cached_file,
        connection,
        &hydrator,
        expected_dir_entries,
        "control",
        0,
        Duration::from_secs(CASE_SECS),
        false,
    );
    run_store_hold(&mount, &cached_file, &store);
    run_case(
        &mount,
        &cached_file,
        connection,
        &hydrator,
        expected_dir_entries,
        "slow1",
        1,
        Duration::from_secs(CASE_SECS),
        false,
    );
    run_case(
        &mount,
        &cached_file,
        connection,
        &hydrator,
        expected_dir_entries,
        "slow16",
        16,
        Duration::from_secs(CASE_SECS),
        false,
    );
    run_case(
        &mount,
        &cached_file,
        connection,
        &hydrator,
        expected_dir_entries,
        "slow64",
        64,
        Duration::from_secs(CASE_SECS + 4),
        true,
    );

    println!("PERF done loadavg={}", loadavg());
    session.join();
    drop(store);
}
