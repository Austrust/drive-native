//! Issue #8 acceptance harness: 210k-node local latency measurements.
//!
//! Scope and limits: this runs against a synthetic SQLite state plus local blob
//! files. There is no NAS, no D-Bus client, no FUSE mount and no real GUI, so
//! the numbers here are local query/latency evidence only, not end-to-end
//! product throughput.
//!
//! The fixture follows `crates/drive-core/examples/performance_fixture.rs`
//! (210,000 nodes = root + 105 directories + 209,894 files, one 2,000-item
//! directory `d000`, one 1 MiB cached file `d000/f00000`) and adds a realistic
//! amount of cached content, pinned nodes and dirty drafts before measuring.
//!
//! Scenarios and issue #8 acceptance thresholds (`docs/P1_ARCHITECTURE.md`):
//! 1. `states_root`    GetStates of the mount root (210k nodes): p95 <= 500 ms.
//! 2. `dir2000`        full enumeration of the 2,000-item directory: p95 <= 300 ms.
//! 3. `mixed48`        48-way mixed states/overview/issues/list/read: zero timeouts.
//! 4. `lock_hold`      GetStates while another thread holds the global Store
//!                     lock for ~2 s: the status query must still be answered.
//!
//! Directory enumeration path: FUSE `readdir` (`crates/drive-fs/src/lib.rs`) ->
//! `directory::read` -> view build in `directory::open` ->
//! `Store::directory_snapshot_measured(ino, MAX_BYTES)`. The namespace-index
//! lease path only short-circuits directories that were already indexed, so a
//! cold authoritative listing is the `directory_snapshot_measured` SQL build.
//! This harness times exactly that call (from the shared writer Store, like the
//! FUSE read queue does); it does not mount a filesystem.
//!
//! Run (release, remote workstation):
//!   cargo test -p drive-runtime --release --all-features \
//!     --test status_latency_210k -- --ignored --nocapture
//!
//! Env: LAT_ITERS (default 120), LAT_BATCH_ITERS (24), LAT_DIR_ITERS (120),
//!      LAT_MIX_SECONDS (15), LAT_LOCK_MS (2000), LAT_TIMEOUT_MS (15000),
//!      LAT_KEEP (keep the fixture directory), LAT_FIXTURE_DIR (reuse a
//!      persistent fixture), LAT_FUSE=0 (skip the FUSE mount scenario).
//!
//! #8 status-cache runs (merged tree with the committed status projection):
//!      LAT_SERVICE=start starts the runtime tasks (required for the projection
//!      worker), LAT_WARM=1 polls GetStates until three consecutive calls are
//!      below LAT_WARM_MS (default 400 ms: the warm 210k render is ~307 ms,
//!      so a 50 ms threshold can never be met) or LAT_WARM_TIMEOUT_MS (60 s), and
//!      reports `#8 WARM` with the cold first-call latency and time-to-warm.
//!      The projection accessor is `pub(crate)`, so warm state is inferred from
//!      latency (an authoritative 210k-node scan cannot answer below ~400 ms).
//!      Without LAT_SERVICE=start the test measures the pure scoped-query path,
//!      which is the before column for the pre-cache trees.
//!
//! Both `before` (b991858/4a0f0a9, no status cache) and `after` (67bf943 with
//! the cache) columns use the same fixture and the same command; only the
//! LAT_SERVICE/LAT_WARM pair differs. `analysis/#8-acceptance.md` records the
//! exact invocations and the loadavg of every run.

use drive_core::{sha256, DownloadTicket, Node, NodeMetadata, Store};
use drive_runtime::{
    desktop,
    service::{Profile, Service},
};
use std::{
    collections::BTreeMap,
    env,
    hint::black_box,
    io::Read,
    path::PathBuf,
    sync::{mpsc, Arc},
    thread,
    time::{Duration, Instant},
};

const NODES: usize = 210_000;
const DIRS: usize = 105;
const FILES_PER_DIR: usize = 2_000;
const LAST_DIR_FILES: usize = 1_894;
const CACHED_BYTES: usize = 1_048_576;
const EXTRA_CACHED: usize = 2_048;
const EXTRA_CACHED_BYTES: usize = 4_096;
const EXTRA_CACHED_STRIDE: usize = 103;
const DRAFTS: usize = 5;
const PINS: usize = 3;
const DIR_BUDGET: usize = 4 * 1024 * 1024;
const ROOT_BUDGET: usize = 64 * 1024 * 1024;

const SLO_STATES_P95_MS: f64 = 500.0;
const SLO_DIR_P95_MS: f64 = 300.0;
const SLO_TIMEOUTS: u64 = 0;
const MIX_THREADS: usize = 48;

fn env_usize(key: &str, default: usize) -> usize {
    env::var(key)
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(default)
}

fn env_u64(key: &str, default: u64) -> u64 {
    env::var(key)
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(default)
}

fn env_flag(key: &str) -> bool {
    matches!(env::var(key).as_deref(), Ok("1") | Ok("true") | Ok("yes"))
}

fn percentile(sorted: &[u64], fraction: f64) -> u64 {
    if sorted.is_empty() {
        return 0;
    }
    let index = ((sorted.len() as f64 * fraction).ceil() as usize)
        .saturating_sub(1)
        .min(sorted.len() - 1);
    sorted[index]
}

#[derive(Clone, Default)]
struct Samples {
    us: Vec<u64>,
    errors: u64,
    timeouts: u64,
}

impl Samples {
    fn record(&mut self, us: u64, failed: bool, timeout_ms: u64) {
        self.us.push(us);
        if failed {
            self.errors += 1;
        }
        if us > timeout_ms.saturating_mul(1_000) {
            self.timeouts += 1;
        }
    }

    fn ms(us: u64) -> f64 {
        us as f64 / 1_000.0
    }

    fn report(&self, scenario: &str, timeout_ms: u64) {
        let mut sorted = self.us.clone();
        sorted.sort_unstable();
        println!(
            "#8 STAT {}",
            serde_json::json!({
                "scenario": scenario,
                "n": sorted.len(),
                "errors": self.errors,
                "timeouts": self.timeouts,
                "timeout_ms": timeout_ms,
                "p50_ms": Self::ms(percentile(&sorted, 0.50)),
                "p95_ms": Self::ms(percentile(&sorted, 0.95)),
                "p99_ms": Self::ms(percentile(&sorted, 0.99)),
                "max_ms": Self::ms(*sorted.last().unwrap_or(&0)),
            })
        );
    }

    fn p50_ms(&self) -> f64 {
        let mut sorted = self.us.clone();
        sorted.sort_unstable();
        Self::ms(percentile(&sorted, 0.50))
    }

    fn p95_ms(&self) -> f64 {
        let mut sorted = self.us.clone();
        sorted.sort_unstable();
        Self::ms(percentile(&sorted, 0.95))
    }
}

struct FixtureFacts {
    nodes: usize,
    cached: usize,
    pins: usize,
    drafts: usize,
    db_bytes: u64,
    build_ms: u128,
    dir_inos: Vec<u64>,
    dir_entries: Vec<usize>,
}

struct Fixture {
    dir: PathBuf,
    /// Keeps the temporary fixture directory alive; `None` when the caller
    /// supplied LAT_FIXTURE_DIR and the fixture is intentionally persistent.
    temp: Option<tempfile::TempDir>,
    store: Store,
    facts: FixtureFacts,
}

fn node(
    id: String,
    parent: Option<String>,
    name: String,
    directory: bool,
    size: u64,
) -> Node {
    Node {
        id,
        parent,
        name,
        version: "fixture-v1".into(),
        directory,
        metadata: Some(NodeMetadata {
            size,
            mtime: 1_700_000_000,
            mode: if directory { 0o755 } else { 0o644 },
        }),
    }
}

fn build_fixture() -> Result<Fixture, Box<dyn std::error::Error>> {
    let started = Instant::now();
    let reuse = env::var("LAT_FIXTURE_DIR")
        .ok()
        .filter(|path| !path.trim().is_empty());
    let (dir, temp) = match reuse {
        Some(path) => (PathBuf::from(path), None),
        None => {
            let temp = tempfile::tempdir()?;
            (temp.path().to_path_buf(), Some(temp))
        }
    };
    let facts_path = dir.join("fixture-facts.json");
    let existed = dir.join("state.sqlite").is_file();
    let mut dir_entries = Vec::with_capacity(DIRS);
    for d in 0..DIRS {
        let count = if d == DIRS - 1 {
            LAST_DIR_FILES
        } else {
            FILES_PER_DIR
        };
        dir_entries.push(count + 2); // "." and ".."
    }

    // LAT_FIXTURE_DIR lets repeated acceptance runs build the 210k-node
    // database once (the snapshot import is the slow part) and reuse it.
    let (store, file_index, cached, build_ms) = if existed {
        let persisted: serde_json::Value = serde_json::from_slice(&std::fs::read(&facts_path)?)?;
        let file_index = persisted["files"].as_u64().unwrap_or(0) as usize;
        let cached = persisted["cached"].as_u64().unwrap_or(0) as usize;
        assert!(
            file_index > 0 && cached > 0,
            "reused fixture is missing fixture-facts.json"
        );
        (
            Store::open(&dir)?,
            file_index,
            cached,
            started.elapsed().as_millis(),
        )
    } else {
        let mut nodes = Vec::with_capacity(NODES);
        nodes.push(node("root".into(), None, "fixture".into(), true, 0));
        let mut file_index = 0usize;
        for d in 0..DIRS {
            let id = format!("d{d:03}");
            nodes.push(node(id.clone(), Some("root".into()), id.clone(), true, 0));
            let count = if d == DIRS - 1 {
                LAST_DIR_FILES
            } else {
                FILES_PER_DIR
            };
            for c in 0..count {
                let size = if file_index == 0 {
                    CACHED_BYTES as u64
                } else {
                    0
                };
                nodes.push(node(
                    format!("file-{file_index:06}"),
                    Some(id.clone()),
                    format!("f{c:05}"),
                    false,
                    size,
                ));
                file_index += 1;
            }
        }
        assert_eq!(nodes.len(), NODES, "fixture node count");
        assert_eq!(file_index, NODES - DIRS - 1, "fixture file count");

        let mut store = Store::open(&dir)?;
        store.install_remote_snapshot(&nodes, "fixture-cursor-1", "fixture-server-1")?;
        store.ensure_fs_root("root")?;

        // The one cached file from the recorded P0 fixture (d000/f00000).
        let body: Vec<u8> = (0..CACHED_BYTES).map(|i| (i % 251) as u8).collect();
        let content_hash = sha256(&body);
        store.publish_download(
            &DownloadTicket {
                id: "file-000000".into(),
                version: "fixture-v1".into(),
            },
            &body,
            &content_hash,
        )?;
        let mut cached = 1usize;

        // A few thousand cached files spread over the tree, so seed queries hit
        // the detailed-status path instead of only the trivial uncached case.
        let mut index = EXTRA_CACHED_STRIDE;
        while cached - 1 < EXTRA_CACHED && index < file_index {
            let extra: Vec<u8> = (0..EXTRA_CACHED_BYTES)
                .map(|i| ((i + index) % 251) as u8)
                .collect();
            store.publish_download(
                &DownloadTicket {
                    id: format!("file-{index:06}"),
                    version: "fixture-v1".into(),
                },
                &extra,
                &sha256(&extra),
            )?;
            cached += 1;
            index += EXTRA_CACHED_STRIDE;
        }

        // Pinned subtree + pinned files (perf-relevant protected content).
        store.pin("d077", true)?;
        store.pin("file-002000", true)?;
        store.pin("file-003000", true)?;

        // A handful of dirty drafts so status seeds exercise the draft predicates.
        for k in 0..DRAFTS {
            let id = format!("file-{:06}", 10_000 + k * 17);
            let draft = store.begin_draft(&id, "fixture-v1", None)?;
            store.draft_write(&draft, 0, b"issue-8 draft", false)?;
        }

        let facts = serde_json::json!({
            "nodes": NODES,
            "files": file_index,
            "cached": cached,
            "pins": PINS,
            "drafts": DRAFTS,
        });
        std::fs::write(&facts_path, serde_json::to_vec_pretty(&facts)?)?;
        (store, file_index, cached, started.elapsed().as_millis())
    };

    // Root listing: one inode per top-level directory, in creation order.
    let root_view = store.directory_snapshot(1, ROOT_BUDGET)?;
    let dir_inos: Vec<u64> = root_view.entries.iter().skip(2).map(|e| e.ino).collect();
    assert_eq!(dir_inos.len(), DIRS, "top-level directory count");

    let db_bytes = std::fs::metadata(dir.join("state.sqlite"))
        .map(|m| m.len())
        .unwrap_or(0);
    println!(
        "#8 FIXTURE {}",
        serde_json::json!({
            "nodes": NODES,
            "dirs": DIRS,
            "files": file_index,
            "cached": cached,
            "pins": PINS,
            "drafts": DRAFTS,
            "db_bytes": db_bytes,
            "build_ms": build_ms,
            "reused": existed,
        })
    );
    Ok(Fixture {
        dir,
        temp,
        store,
        facts: FixtureFacts {
            nodes: NODES,
            cached,
            pins: PINS,
            drafts: DRAFTS,
            db_bytes,
            build_ms,
            dir_inos,
            dir_entries,
        },
    })
}

fn profile(mount: PathBuf) -> Profile {
    Profile {
        schema: 1,
        task_id: uuid::Uuid::new_v4().to_string(),
        host: "issue8.example.invalid".into(),
        tls_name: "issue8.example.invalid".into(),
        trusted_ca_pem: Vec::new(),
        username: "issue8".into(),
        device_id: uuid::Uuid::new_v4().to_string(),
        scope: drive_core::native::Scope {
            view_id: 1,
            node_id: 1,
            file_id: "root".into(),
            path: "/".into(),
        },
        mount,
        writable: false,
        local_policy: None,
        writable_source: None,
    }
}

fn check_states_body(body: &str, expected_entries: usize) -> Result<(), String> {
    let value: serde_json::Value =
        serde_json::from_str(body).map_err(|error| error.to_string())?;
    let entries = value["entries"]
        .as_array()
        .ok_or_else(|| "states response without entries".to_string())?;
    if entries.len() != expected_entries {
        return Err(format!(
            "states entries {} != {}",
            entries.len(),
            expected_entries
        ));
    }
    Ok(())
}

fn bench_states(
    service: &Service,
    scenario: &str,
    paths: Vec<String>,
    iters: usize,
    timeout_ms: u64,
) -> Samples {
    for _ in 0..2 {
        let _ = black_box(desktop::states(service, paths.clone()));
    }
    let mut samples = Samples::default();
    for i in 0..iters {
        let started = Instant::now();
        let result = desktop::states(service, paths.clone());
        let us = started.elapsed().as_micros() as u64;
        let error = match result {
            Ok(body) => check_states_body(&body, paths.len()).err(),
            Err(error) => Some(error.to_string()),
        };
        samples.record(us, error.is_some(), timeout_ms);
        println!(
            "#8 OP scenario={scenario} i={i} us={us} err={}",
            error.as_deref().unwrap_or("-")
        );
    }
    samples.report(scenario, timeout_ms);
    samples
}

fn bench_directories(
    service: &Arc<Service>,
    facts: &FixtureFacts,
    iters: usize,
    timeout_ms: u64,
) -> Samples {
    for i in 0..3 {
        let ino = facts.dir_inos[i % facts.dir_inos.len()];
        let guard = service.bridge.store.lock().unwrap();
        let _ = black_box(guard.directory_snapshot_measured(ino, DIR_BUDGET));
    }
    let mut samples = Samples::default();
    for i in 0..iters {
        let slot = i % facts.dir_inos.len();
        let ino = facts.dir_inos[slot];
        let expected = facts.dir_entries[slot];
        let started = Instant::now();
        let result = {
            let guard = service.bridge.store.lock().unwrap();
            guard.directory_snapshot_measured(ino, DIR_BUDGET)
        };
        let us = started.elapsed().as_micros() as u64;
        let error = match result {
            Ok((view, _timings)) => {
                if view.entries.len() == expected {
                    None
                } else {
                    Some(format!("entries {} != {}", view.entries.len(), expected))
                }
            }
            Err(error) => Some(error.to_string()),
        };
        samples.record(us, error.is_some(), timeout_ms);
        println!(
            "#8 OP scenario=dir2000 i={i} ino={ino} us={us} err={}",
            error.as_deref().unwrap_or("-")
        );
    }
    samples.report("dir2000", timeout_ms);
    samples
}

/// End-to-end enumeration through a real FUSE mount (`CacheFilesystem::readdir`),
/// which is what a client (GUI/Explorer/Finder) triggers. The Store-level
/// `dir2000` scenario above measures `Store::directory_snapshot_measured`, the
/// authoritative call this path falls back to on a namespace-index miss; the
/// recorded p0 baseline (1.04 s p95 for 2000 entries) was measured through the
/// mounted client, so this scenario is the directly comparable one.
struct FuseBench {
    entries: usize,
    cold_ms: f64,
    warm: Samples,
    read_cold_ms: f64,
    read_warm: Samples,
}

fn fuse_mount(service: &Arc<Service>) -> Option<(tempfile::TempDir, fuser::BackgroundSession)> {
    if env::var("LAT_FUSE").map(|value| value == "0").unwrap_or(false) {
        return None;
    }
    let mount = match tempfile::tempdir() {
        Ok(mount) => mount,
        Err(error) => {
            println!("#8 NOTE fuse_mount_failed error={error}");
            return None;
        }
    };
    let filesystem = match drive_fs::CacheFilesystem::new(
        service.bridge.store.clone(),
        "root",
        unsafe { libc::geteuid() },
        unsafe { libc::getegid() },
    ) {
        Ok(filesystem) => filesystem,
        Err(error) => {
            println!("#8 NOTE fuse_mount_failed error={error}");
            return None;
        }
    };
    let session = match fuser::spawn_mount2(
        filesystem,
        mount.path(),
        &[fuser::MountOption::FSName("drive-native-issue8".into())],
    ) {
        Ok(session) => session,
        Err(error) => {
            println!("#8 NOTE fuse_spawn_failed error={error}");
            return None;
        }
    };
    Some((mount, session))
}

fn bench_fuse(mount: &std::path::Path, iters: usize, timeout_ms: u64) -> Option<FuseBench> {
    let directory = mount.join("d000");
    let started = Instant::now();
    let entries = match std::fs::read_dir(&directory) {
        Ok(entries) => entries.collect::<std::io::Result<Vec<_>>>().ok()?.len(),
        Err(error) => {
            println!("#8 NOTE fuse_readdir_failed error={error}");
            return None;
        }
    };
    let cold_ms = started.elapsed().as_secs_f64() * 1000.0;
    let mut warm = Samples::default();
    for i in 0..iters {
        let started = Instant::now();
        let count = std::fs::read_dir(&directory)
            .ok()
            .and_then(|entries| entries.collect::<std::io::Result<Vec<_>>>().ok())
            .map(|entries| entries.len());
        let us = started.elapsed().as_micros() as u64;
        let error = match count {
            Some(count) if count == entries => None,
            Some(count) => Some(format!("entries {count} != {entries}")),
            None => Some("readdir failed".to_string()),
        };
        warm.record(us, error.is_some(), timeout_ms);
        println!(
            "#8 OP scenario=fuse_dir2000 i={i} us={us} err={}",
            error.as_deref().unwrap_or("-")
        );
    }
    warm.report("fuse_dir2000", timeout_ms);

    let file = mount.join("d000").join("f00000");
    let started = Instant::now();
    let bytes = std::fs::read(&file).ok()?;
    let read_cold_ms = started.elapsed().as_secs_f64() * 1000.0;
    if bytes.len() != CACHED_BYTES {
        println!(
            "#8 NOTE fuse_read1m_size actual={} expected={}",
            bytes.len(),
            CACHED_BYTES
        );
    }
    let mut read_warm = Samples::default();
    for i in 0..iters.min(64) {
        let started = Instant::now();
        let ok = std::fs::read(&file)
            .map(|bytes| bytes.len() == CACHED_BYTES)
            .unwrap_or(false);
        let us = started.elapsed().as_micros() as u64;
        read_warm.record(us, !ok, timeout_ms);
        println!(
            "#8 OP scenario=fuse_read1m i={i} us={us} err={}",
            if ok { "-" } else { "size" }
        );
    }
    read_warm.report("fuse_read1m", timeout_ms);
    println!(
        "#8 FUSE {}",
        serde_json::json!({
            "entries": entries,
            "cold_ms": cold_ms,
            "read_cold_ms": read_cold_ms,
            "read_bytes": CACHED_BYTES,
        })
    );
    Some(FuseBench {
        entries,
        cold_ms,
        warm,
        read_cold_ms,
        read_warm,
    })
}

fn read_cached(service: &Service) -> Option<String> {
    let mut handle = match {
        let guard = service.bridge.store.lock().unwrap();
        guard.open_cached("file-000000")
    } {
        Ok(handle) => handle,
        Err(error) => return Some(error.to_string()),
    };
    let mut buffer = Vec::with_capacity(CACHED_BYTES);
    match handle.read_to_end(&mut buffer) {
        Ok(read) => {
            if read == CACHED_BYTES {
                None
            } else {
                Some(format!("read {read} != {CACHED_BYTES}"))
            }
        }
        Err(error) => Some(error.to_string()),
    }
}

/// Pure GetStates concurrency probe: `threads` callers hammer the same root
/// status request for `seconds`. This isolates the "16 concurrent GetStates"
/// component of the classic mixed load from the directory/read callers, so the
/// classic p95 can be attributed to either the query cost itself (memory
/// bandwidth) or to serialized fallbacks on the Store mutex.
fn bench_states_concurrent(
    service: &Arc<Service>,
    threads: usize,
    seconds: u64,
    timeout_ms: u64,
) -> Samples {
    let deadline = Instant::now() + Duration::from_secs(seconds);
    let (tx, rx) = mpsc::channel::<(u64, bool)>();
    let mut handles = Vec::new();
    for _ in 0..threads {
        let service = service.clone();
        let tx = tx.clone();
        handles.push(thread::spawn(move || {
            while Instant::now() < deadline {
                let started = Instant::now();
                let result = desktop::states(&service, vec![".".to_string()]);
                let us = started.elapsed().as_micros() as u64;
                if tx.send((us, result.is_err())).is_err() {
                    break;
                }
            }
        }));
    }
    drop(tx);
    let mut samples = Samples::default();
    for (us, failed) in rx {
        samples.record(us, failed, timeout_ms);
    }
    for handle in handles {
        handle.join().ok();
    }
    samples.report("states_conc16", timeout_ms);
    samples
}

fn mixed_load(
    service: &Arc<Service>,
    facts: &FixtureFacts,
    seconds: u64,
    timeout_ms: u64,
    tag: &'static str,
    classic: bool,
) -> BTreeMap<String, Samples> {
    let deadline = Instant::now() + Duration::from_secs(seconds);
    let (tx, rx) = mpsc::channel::<(String, u64, bool)>();
    let mut handles = Vec::new();
    for slot in 0..MIX_THREADS {
        // `classic` mirrors docs/validation/p0-mixed-48-001 (16 directory
        // listings + 16 root GetStates + 16 cached reads) so the states tail
        // is directly comparable with the recorded 9.3 s baseline. The default
        // layout follows the acceptance brief: states + directories + issues +
        // cached reads + overview per group of 12 threads.
        let kind = if classic {
            match slot % 3 {
                0 => "states",
                1 => "dir",
                _ => "read",
            }
        } else {
            match slot % 12 {
                0..=2 => "states",
                3..=5 => "dir",
                6..=7 => "issues",
                8..=9 => "read",
                _ => "overview",
            }
        };
        let service = Arc::clone(service);
        let sender = tx.clone();
        let dir_inos = facts.dir_inos.clone();
        let dir_entries = facts.dir_entries.clone();
        handles.push(thread::spawn(move || {
            let mut ops = 0usize;
            while Instant::now() < deadline {
                let started = Instant::now();
                let error: Option<String> = match kind {
                    "states" => match desktop::states(&service, vec![".".to_string()]) {
                        Ok(body) => check_states_body(&body, 1).err(),
                        Err(error) => Some(error.to_string()),
                    },
                    "dir" => {
                        let slot = ops % dir_inos.len();
                        let result = {
                            let guard = service.bridge.store.lock().unwrap();
                            guard.directory_snapshot_measured(dir_inos[slot], DIR_BUDGET)
                        };
                        match result {
                            Ok((view, _timings)) => {
                                if view.entries.len() == dir_entries[slot] {
                                    None
                                } else {
                                    Some(format!(
                                        "entries {} != {}",
                                        view.entries.len(),
                                        dir_entries[slot]
                                    ))
                                }
                            }
                            Err(error) => Some(error.to_string()),
                        }
                    }
                    "issues" => match desktop::issues(&service, "") {
                        Ok(body) => serde_json::from_str::<serde_json::Value>(&body)
                            .map(|_| ())
                            .map_err(|error| error.to_string())
                            .err(),
                        Err(error) => Some(error.to_string()),
                    },
                    "read" => read_cached(&service),
                    "overview" => match desktop::overview(&service) {
                        Ok(body) => serde_json::from_str::<serde_json::Value>(&body)
                            .map(|_| ())
                            .map_err(|error| error.to_string())
                            .err(),
                        Err(error) => Some(error.to_string()),
                    },
                    _ => unreachable!(),
                };
                let us = started.elapsed().as_micros() as u64;
                if sender
                    .send((format!("{tag}_{kind}"), us, error.is_some()))
                    .is_err()
                {
                    break;
                }
                ops += 1;
            }
            println!("#8 MIX_THREAD kind={kind} ops={ops}");
        }));
    }
    drop(tx);
    let mut samples: BTreeMap<String, Samples> = BTreeMap::new();
    for (kind, us, failed) in rx {
        samples
            .entry(kind)
            .or_default()
            .record(us, failed, timeout_ms);
    }
    for handle in handles {
        handle.join().expect("mixed thread panicked");
    }
    for kind in ["states", "dir", "issues", "read", "overview"] {
        let key = format!("{tag}_{kind}");
        if let Some(kind_samples) = samples.get(&key) {
            kind_samples.report(&key, timeout_ms);
        }
    }
    let mut all = Samples::default();
    for kind_samples in samples.values() {
        all.us.extend_from_slice(&kind_samples.us);
        all.errors += kind_samples.errors;
        all.timeouts += kind_samples.timeouts;
    }
    all.report(&format!("{tag}_all"), timeout_ms);
    // Keep the aggregate in the returned map: the METRICS line and the SLO
    // verdicts read `{tag}_all` from it, so a missing entry would silently
    // report zero timeouts (and a zero p95) instead of the real values.
    samples.insert(format!("{tag}_all"), all);
    samples
}

struct Probe {
    name: &'static str,
    ms: u64,
    ok: bool,
    completed_while_locked: bool,
}

fn spawn_probe(
    name: &'static str,
    started: Instant,
    lock_ms: u64,
    wait: Duration,
    operation: impl FnOnce() -> Result<(), String> + Send + 'static,
) -> Probe {
    let (tx, rx) = mpsc::channel::<Result<(), String>>();
    thread::spawn(move || {
        let result = operation();
        let _ = tx.send(result);
    });
    match rx.recv_timeout(wait) {
        Ok(result) => {
            let elapsed = started.elapsed().as_millis() as u64;
            Probe {
                name,
                ms: elapsed,
                ok: result.is_ok(),
                completed_while_locked: elapsed < lock_ms,
            }
        }
        Err(_) => Probe {
            name,
            ms: wait.as_millis() as u64,
            ok: false,
            completed_while_locked: false,
        },
    }
}

fn lock_hold_scenario(
    service: &Arc<Service>,
    lock_ms: u64,
    iters: usize,
    timeout_ms: u64,
) -> (Vec<Probe>, Samples) {
    let _ = desktop::states(service, vec![".".to_string()]);
    let hold = {
        let shared = service.bridge.store.clone();
        thread::spawn(move || {
            let guard = shared.lock().unwrap();
            thread::sleep(Duration::from_millis(lock_ms));
            drop(guard);
        })
    };
    thread::sleep(Duration::from_millis(100));

    let started = Instant::now();
    let states_probe = {
        let service = Arc::clone(service);
        spawn_probe("states", started, lock_ms, Duration::from_millis(lock_ms + 30_000), move || {
            desktop::states(&service, vec![".".to_string()])
                .map(|_| ())
                .map_err(|error| error.to_string())
        })
    };
    let issues_probe = {
        let service = Arc::clone(service);
        spawn_probe("issues", started, lock_ms, Duration::from_millis(lock_ms + 30_000), move || {
            desktop::issues(&service, "")
                .map(|_| ())
                .map_err(|error| error.to_string())
        })
    };
    let overview_probe = {
        let service = Arc::clone(service);
        spawn_probe("overview", started, lock_ms, Duration::from_millis(lock_ms + 30_000), move || {
            desktop::overview(&service)
                .map(|_| ())
                .map_err(|error| error.to_string())
        })
    };
    hold.join().expect("lock holder panicked");

    let mut post = Samples::default();
    for _ in 0..iters {
        let started = Instant::now();
        let result = desktop::states(service, vec![".".to_string()]);
        let us = started.elapsed().as_micros() as u64;
        let failed = match result {
            Ok(body) => check_states_body(&body, 1).is_err(),
            Err(_) => true,
        };
        post.record(us, failed, timeout_ms);
    }
    post.report("lock_hold_post", timeout_ms);
    (vec![states_probe, issues_probe, overview_probe], post)
}

fn peak_rss_kb() -> Option<u64> {
    let status = std::fs::read_to_string("/proc/self/status").ok()?;
    for line in status.lines() {
        if let Some(value) = line.strip_prefix("VmHWM:") {
            return value.split_whitespace().next()?.parse().ok();
        }
    }
    None
}

/// 1-minute load average of the host. The acceptance box is shared with other
/// build/test jobs, so the raw numbers are only comparable across runs with a
/// similar load (recorded in `#8 ENV` and `#8 METRICS`).
fn load_avg1() -> Option<f64> {
    std::fs::read_to_string("/proc/loadavg")
        .ok()?
        .split_whitespace()
        .next()?
        .parse()
        .ok()
}

/// Observation of the committed-status-projection warm-up on the #8 cache tree.
///
/// The projection accessor is `pub(crate)`, so an integration test cannot ask
/// the service whether a snapshot is published. Instead this times GetStates
/// calls: three consecutive calls below `threshold_ms` cannot be served by the
/// authoritative 210k-node scoped query, so they mean "projection published".
struct WarmResult {
    cold_ms: f64,
    ready: bool,
    ready_ms: Option<f64>,
    calls: usize,
    warm_p50_ms: f64,
    warm_p95_ms: f64,
}

fn warm_status_projection(
    service: &Arc<Service>,
    timeout_ms: u64,
    threshold_ms: f64,
) -> WarmResult {
    let first = Instant::now();
    let mut cold_ms = None;
    let mut warm = Samples::default();
    let mut calls = 0usize;
    let mut consecutive = 0usize;
    let mut ready_at = None;
    loop {
        let started = Instant::now();
        let result = desktop::states(service, vec![".".to_string()]);
        let us = started.elapsed().as_micros() as u64;
        let ms = Samples::ms(us);
        calls += 1;
        if cold_ms.is_none() {
            cold_ms = Some(ms);
        }
        if result.is_ok() && ms <= threshold_ms {
            warm.us.push(us);
            consecutive += 1;
        } else {
            consecutive = 0;
        }
        if consecutive >= 3 {
            ready_at = Some(first.elapsed().as_secs_f64() * 1_000.0);
            break;
        }
        if first.elapsed() >= Duration::from_millis(timeout_ms) {
            break;
        }
        thread::sleep(Duration::from_millis(100));
    }
    WarmResult {
        cold_ms: cold_ms.unwrap_or(0.0),
        ready: ready_at.is_some(),
        ready_ms: ready_at,
        calls,
        warm_p50_ms: warm.p50_ms(),
        warm_p95_ms: warm.p95_ms(),
    }
}

fn verdict(passed: bool) -> &'static str {
    if passed {
        "PASS"
    } else {
        "FAIL"
    }
}

#[test]
#[ignore]
fn status_latency_210k() {
    let iters = env_usize("LAT_ITERS", 120);
    let batch_iters = env_usize("LAT_BATCH_ITERS", 24);
    let dir_iters = env_usize("LAT_DIR_ITERS", 120);
    let mix_seconds = env_u64("LAT_MIX_SECONDS", 15);
    let lock_ms = env_u64("LAT_LOCK_MS", 2_000);
    let timeout_ms = env_u64("LAT_TIMEOUT_MS", 15_000);
    let service_started = matches!(
        env::var("LAT_SERVICE").as_deref(),
        Ok("start") | Ok("1") | Ok("true")
    );
    let warm_threshold_ms = env::var("LAT_WARM_MS")
        .ok()
        .and_then(|value| value.parse().ok())
        // A warm 210k-node GetStates render is ~307 ms on this 32-core host,
        // so a 50 ms threshold can never be met. The default marks the point
        // where latency has settled at the warm-path level, not a 50 ms SLO.
        .unwrap_or(400.0);
    let warm_timeout_ms = env_u64("LAT_WARM_TIMEOUT_MS", 60_000);
    println!(
        "#8 ENV {}",
        serde_json::json!({
            "iters": iters,
            "batch_iters": batch_iters,
            "dir_iters": dir_iters,
            "mix_seconds": mix_seconds,
            "lock_ms": lock_ms,
            "timeout_ms": timeout_ms,
            "threads": MIX_THREADS,
            "service": if service_started { "start" } else { "new" },
            "warm": env_flag("LAT_WARM"),
            "warm_threshold_ms": warm_threshold_ms,
            "warm_timeout_ms": warm_timeout_ms,
            "loadavg_start": load_avg1(),
        })
    );

    let fixture = build_fixture().expect("build fixture");
    let Fixture {
        dir,
        temp,
        store,
        facts,
    } = fixture;
    let service = Service::new(
        profile(PathBuf::from("/tmp/drive-native-issue8-mount")),
        store,
        PathBuf::from("/nonexistent-worker"),
    )
    .expect("service");
    let tasks = if service_started {
        let tasks = service.start();
        println!(
            "#8 NOTE service_started=1 (runtime background tasks running; on a #8 cache tree this starts the committed status projection worker)"
        );
        Some(tasks)
    } else {
        None
    };
    let cold_states_ms = if service_started {
        let started = Instant::now();
        let _ = black_box(desktop::states(&service, vec![".".to_string()]));
        Some(started.elapsed().as_secs_f64() * 1_000.0)
    } else {
        None
    };
    let cold_lock = if service_started {
        // #8 wants both a warm-cache and a cold-cache answer for "responsive
        // while the writer lock is held"; this probe runs right after
        // `Service::start`, before the projection worker publishes its first
        // snapshot (the cold state for a cache tree).
        let (probes, _) = lock_hold_scenario(&service, lock_ms, 0, timeout_ms);
        Some(probes)
    } else {
        None
    };
    let warm = if service_started && env_flag("LAT_WARM") {
        let warm = warm_status_projection(&service, warm_timeout_ms, warm_threshold_ms);
        println!(
            "#8 WARM {}",
            serde_json::json!({
                "cold_ms": cold_states_ms,
                "probe_cold_ms": warm.cold_ms,
                "ready": warm.ready,
                "ready_ms": warm.ready_ms,
                "calls": warm.calls,
                "warm_p50_ms": warm.warm_p50_ms,
                "warm_p95_ms": warm.warm_p95_ms,
                "threshold_ms": warm_threshold_ms,
                "timeout_ms": warm_timeout_ms,
            })
        );
        Some(warm)
    } else {
        if service_started {
            thread::sleep(Duration::from_millis(env_u64("LAT_SETTLE_MS", 250)));
        }
        None
    };
    if let Some(probes) = &cold_lock {
        println!(
            "#8 NOTE lock_hold_cold states={}ms issues={}ms overview={}ms (immediately after Service::start, before the first projection publication)",
            probes[0].ms, probes[1].ms, probes[2].ms
        );
    }

    let states_root = bench_states(
        &service,
        "states_root",
        vec![".".to_string()],
        iters,
        timeout_ms,
    );
    let states_dir = bench_states(
        &service,
        "states_dir2000",
        vec!["d000".to_string()],
        iters,
        timeout_ms,
    );
    let batch_paths: Vec<String> = (0..45).map(|i| format!("d{i:03}")).collect();
    let states_batch = bench_states(&service, "states_batch45", batch_paths, batch_iters, timeout_ms);
    let dir2000 = bench_directories(&service, &facts, dir_iters, timeout_ms);
    let fuse = fuse_mount(&service).and_then(|(mount, session)| {
        let bench = bench_fuse(mount.path(), dir_iters.min(60), timeout_ms);
        drop(session);
        drop(mount);
        bench
    });
    let states_conc = bench_states_concurrent(&service, 16, mix_seconds, timeout_ms);
    let mixed = mixed_load(&service, &facts, mix_seconds, timeout_ms, "mix", false);
    let classic = mixed_load(&service, &facts, mix_seconds, timeout_ms, "classic", true);
    let (probes, lock_post) = lock_hold_scenario(&service, lock_ms, 10, timeout_ms);

    for probe in &probes {
        println!(
            "#8 PROBE {}",
            serde_json::json!({
                "probe": probe.name,
                "ms": probe.ms,
                "ok": probe.ok,
                "lock_ms": lock_ms,
                "completed_while_locked": probe.completed_while_locked,
            })
        );
    }
    let mixed_all = mixed.get("mix_all").cloned().unwrap_or_default();
    let mix_states = mixed.get("mix_states").cloned().unwrap_or_default();
    let classic_all = classic.get("classic_all").cloned().unwrap_or_default();
    let classic_states = classic.get("classic_states").cloned().unwrap_or_default();
    println!(
        "#8 METRICS {}",
        serde_json::json!({
            "fixture": {
                "nodes": facts.nodes,
                "cached": facts.cached,
                "pins": facts.pins,
                "drafts": facts.drafts,
                "db_bytes": facts.db_bytes,
                "build_ms": facts.build_ms,
            },
            "states_root_p95_ms": states_root.p95_ms(),
            "states_dir2000_p95_ms": states_dir.p95_ms(),
            "states_batch45_p95_ms": states_batch.p95_ms(),
            "dir2000_p95_ms": dir2000.p95_ms(),
            "fuse_dir_entries": fuse.as_ref().map(|fuse| fuse.entries),
            "fuse_dir_cold_ms": fuse.as_ref().map(|fuse| fuse.cold_ms),
            "fuse_dir2000_p95_ms": fuse.as_ref().map(|fuse| fuse.warm.p95_ms()),
            "fuse_read1m_cold_ms": fuse.as_ref().map(|fuse| fuse.read_cold_ms),
            "fuse_read1m_p95_ms": fuse.as_ref().map(|fuse| fuse.read_warm.p95_ms()),
            "states_conc16_p95_ms": states_conc.p95_ms(),
            "states_conc16_ops": states_conc.us.len(),
            "mixed_timeouts": mixed_all.timeouts,
            "mixed_ops": mixed_all.us.len(),
            "mixed_p95_ms": mixed_all.p95_ms(),
            "mix_states_p95_ms": mix_states.p95_ms(),
            "classic_timeouts": classic_all.timeouts,
            "classic_states_p95_ms": classic_states.p95_ms(),
            "peak_rss_kb": peak_rss_kb(),
            "loadavg_end": load_avg1(),
            "lock_ms": lock_ms,
            "lock_post_p95_ms": lock_post.p95_ms(),
            "service_started": service_started,
            "cold_states_ms": cold_states_ms,
            "lock_cold_states_ms": cold_lock.as_ref().map(|probes| probes[0].ms),
            "lock_cold_states_completed": cold_lock
                .as_ref()
                .map(|probes| probes[0].completed_while_locked),
            "lock_cold_issues_ms": cold_lock.as_ref().map(|probes| probes[1].ms),
            "projection_ready": warm.as_ref().map(|warm| warm.ready),
            "projection_ready_ms": warm.as_ref().and_then(|warm| warm.ready_ms),
            "projection_warm_p95_ms": warm.as_ref().map(|warm| warm.warm_p95_ms),
            "projection_warm_calls": warm.as_ref().map(|warm| warm.calls),
            "projection_warm_threshold_ms": warm.as_ref().map(|_| warm_threshold_ms),
        })
    );

    println!(
        "#8 SLO name=states_root_p95_ms observed={:.3} limit={:.0} verdict={}",
        states_root.p95_ms(),
        SLO_STATES_P95_MS,
        verdict(states_root.p95_ms() <= SLO_STATES_P95_MS)
    );
    println!(
        "#8 SLO name=dir2000_p95_ms observed={:.3} limit={:.0} verdict={}",
        dir2000.p95_ms(),
        SLO_DIR_P95_MS,
        verdict(dir2000.p95_ms() <= SLO_DIR_P95_MS)
    );
    match &fuse {
        Some(fuse) => println!(
            "#8 SLO name=dir2000_fuse_p95_ms observed={:.3} limit={:.0} verdict={}",
            fuse.warm.p95_ms(),
            SLO_DIR_P95_MS,
            verdict(fuse.warm.p95_ms() <= SLO_DIR_P95_MS)
        ),
        None => println!("#8 NOTE fuse_dir2000 skipped (mount unavailable; set LAT_FUSE=0 to skip explicitly)"),
    }
    println!(
        "#8 NOTE dir2000_definition store-level scenario calls Store::directory_snapshot_measured under SharedStore::lock (the authoritative path drive-fs readdir falls back to on a namespace-index miss); fuse_dir2000 iterates a real CacheFilesystem mount, matching the p0 issue-#8 baseline measurement (1.04 s p95)"
    );
    println!(
        "#8 SLO name=mixed_timeouts observed={} limit={} verdict={}",
        mixed_all.timeouts,
        SLO_TIMEOUTS,
        verdict(mixed_all.timeouts == SLO_TIMEOUTS)
    );
    println!(
        "#8 SLO name=lock_hold_states_responsive observed={}ms limit=<{}ms verdict={}",
        probes[0].ms,
        lock_ms,
        verdict(probes[0].completed_while_locked)
    );
    println!(
        "#8 SLO name=lock_hold_issues_responsive observed={}ms limit=<{}ms verdict={}",
        probes[1].ms,
        lock_ms,
        verdict(probes[1].completed_while_locked)
    );
    println!(
        "#8 NOTE overview_during_lock observed={}ms completed_while_locked={} (overview still takes the global Store mutex on both trees)",
        probes[2].ms,
        probes[2].completed_while_locked
    );
    println!(
        "#8 NOTE classic_mixed_states_p95_ms observed={:.3} baseline_p95_ms=9302.908 (classic mix = 16 dir + 16 root GetStates + 16 cached reads, mirrors docs/validation/p0-mixed-48-001); pure 16-way GetStates p95 (no dir/read callers) = {:.3}ms",
        classic_states.p95_ms(),
        states_conc.p95_ms()
    );
    println!(
        "#8 NOTE mix_states_p95_ms observed={:.3} (acceptance mix also runs overview, which still takes the global Store mutex on both trees)",
        mix_states.p95_ms()
    );
    if let Some(tasks) = tasks {
        // Background tasks must be stopped before the fixture/store is dropped;
        // joining a stuck service thread would hang the capacity test, so the
        // stop flag is set and the handles are detached after a short grace.
        service.shutdown();
        thread::sleep(Duration::from_millis(250));
        drop(tasks);
        println!("#8 NOTE service_shutdown=1 (stop flag set; handles detached)");
    }
    if env_flag("LAT_KEEP") {
        println!("#8 KEEP fixture_dir={}", dir.display());
        if let Some(temp) = temp {
            std::mem::forget(temp);
        }
    }
    println!("#8 DONE");
}
