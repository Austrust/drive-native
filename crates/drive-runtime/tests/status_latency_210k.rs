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
//!      LAT_OVERVIEW_ITERS (40), LAT_MIX_SECONDS (15), LAT_LOCK_MS (2000),
//!      LAT_TIMEOUT_MS (15000), LAT_KEEP (keep the fixture directory),
//!      LAT_FIXTURE_DIR (reuse a persistent fixture), LAT_FUSE=0 (skip the FUSE
//!      mount scenario).
//!
//! Issue #21 (GetOverview without the Store lock): this harness also prints
//!      `#8 OVERVIEW` with the query / serialize / lock-hold split and the
//!      `overview_total_p95_ms`, `overview_lock_hold_p95_ms` and
//!      `overview_responsive_under_lock` SLO lines. `overview_scale_curve`
//!      (ignored, separate run) reports the 210k/294k/500k `#8 CURVE` points;
//!      LAT_CURVE_DIRS selects the directory counts, LAT_CURVE_WARM_ITERS (5)
//!      the steady-state calls per point.
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
//!
//! Issue #94 gate: the run is now an explicit mode with a machine-readable
//! verdict. `LAT_MODE=cold|warm|degraded|mixed` selects
//! cold/startup, service+projection-warm, forced-degraded (the authoritative
//! fallback window right after `Service::start`; there is no product hook to
//! force the fallback) or steady mixed; without LAT_MODE the legacy
//! LAT_SERVICE/LAT_WARM pair maps as documented in `choose_mode`. Warm/mixed
//! modes require `LAT_SERVICE=start` and a published committed projection;
//! missing warm evidence is INCONCLUSIVE with a non-zero exit, never a silent
//! fallback. SLO thresholds are gating steady-state claims in warm/mixed mode;
//! cold/degraded keep the same measured values but report them as disclosed
//! observations (errors/timeouts stay gating), so a cold fallback number is
//! never presented as steady-state product performance.
//! Every interface reports p50/p95/p99/max/errors/timeouts/completions
//! (`#94 PER_INTERFACE`), the per-interface checks are gating and the aggregate
//! mixed p95 is informational only, the classic mixed scenario is an
//! independent gate, and `#94 VERDICT` is the final machine-readable verdict
//! (cargo test exits non-zero on FAIL/INCONCLUSIVE). `LAT_GATE_INJECT=
//! classic_timeout|interface_overrun|states_overrun` injects a synthetic
//! failure to prove the gate trips; the same logic is covered by the fast
//! `gate_selftest_*` unit tests. Staged memory samples (`#94 MEM`) replace
//! single-HWM leak claims; `workload_hooks_94` runs the other workload axes
//! (confirmed history, draft scale, load-more, GC pressure, large-directory
//! kernel-FUSE continuation) with an explicit scope per hook.
//!
//! Env (issue #94 additions): LAT_MODE, LAT_FORCE_DEGRADED, LAT_HEAD, LAT_ROUND,
//!      LAT_GATE_INJECT, LAT_HISTORY_ROWS/LAT_QUARANTINE (reported scale axes),
//!      LAT_HOOKS/LAT_HOOK_SCALE/LAT_HOOK_HISTORY (workload_hooks_94).

// The `#8 METRICS`/`#8 CURVE` payloads are one large `json!` literal; the
// default macro recursion limit is not enough for that many fields.
#![recursion_limit = "256"]

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
/// Issue #21 acceptance: GetOverview must never serialize behind the writer
/// (lock hold p95 < 200 ms) and must stay well under a desktop poll budget.
const SLO_OVERVIEW_TOTAL_P95_MS: f64 = 500.0;
const SLO_OVERVIEW_LOCK_HOLD_P95_MS: f64 = 200.0;
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

    fn p99_ms(&self) -> f64 {
        let mut sorted = self.us.clone();
        sorted.sort_unstable();
        Self::ms(percentile(&sorted, 0.99))
    }

    fn max_ms(&self) -> f64 {
        Self::ms(*self.us.iter().max().unwrap_or(&0))
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
        exclude_patterns: Vec::new(),
        exclude_mode: drive_core::exclude::ExcludeMode::default(),
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
            // Issue #21: the overview lanes report the store timing split, so a
            // regression that reintroduces a writer-lock stretch shows up even
            // when the coarse latency stays under the timeout.
            let mut mix_total_us = 0u64;
            let mut mix_query_us = 0u64;
            let mut mix_serialize_us = 0u64;
            let mut mix_lock_hold_us = 0u64;
            let mut mix_read_only = 0usize;
            let mut mix_cached = 0usize;
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
                    "overview" => match desktop::overview_measured(&service) {
                        Ok((body, timings)) => {
                            mix_total_us += timings.total_us;
                            mix_query_us += timings.query_us;
                            mix_serialize_us += timings.serialize_us;
                            mix_lock_hold_us += timings.lock_hold_us;
                            mix_read_only += usize::from(timings.read_only);
                            mix_cached += usize::from(timings.derived_cached);
                            serde_json::from_str::<serde_json::Value>(&body)
                                .map(|_| ())
                                .map_err(|error| error.to_string())
                                .err()
                        }
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
            if kind == "overview" {
                println!(
                    "#8 OVERVIEW_MIX ops={ops} total_us={mix_total_us} query_us={mix_query_us} serialize_us={mix_serialize_us} lock_hold_us={mix_lock_hold_us} read_only_calls={mix_read_only} cached_calls={mix_cached}"
                );
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
/// the service whether a snapshot is published. Evidence, in order of strength:
///  1. product projection counters, when the integrated tree exposes them
///     additively in the GetStates document (`projection:{ready,hits}`);
///  2. otherwise, three consecutive GetStates calls at or below the warm
///     threshold after a slower first post-start call (the authoritative
///     210k-node fallback cannot answer that fast).
struct WarmResult {
    cold_ms: f64,
    ready: bool,
    ready_ms: Option<f64>,
    calls: usize,
    warm_p50_ms: f64,
    warm_p95_ms: f64,
    counter_ready: Option<bool>,
    counter_hits: Option<u64>,
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
    let mut counter_ready = None;
    let mut counter_hits = None;
    loop {
        let started = Instant::now();
        let result = desktop::states(service, vec![".".to_string()]);
        let us = started.elapsed().as_micros() as u64;
        let ms = Samples::ms(us);
        calls += 1;
        if cold_ms.is_none() {
            cold_ms = Some(ms);
        }
        let ok = result.is_ok();
        if let Ok(body) = &result {
            if let Some((ready, hits)) = projection_counter_evidence(body) {
                counter_ready = Some(counter_ready.unwrap_or(false) || ready);
                if let Some(hits) = hits {
                    counter_hits = Some(counter_hits.unwrap_or(0) + hits);
                }
            }
        }
        if ok && ms <= threshold_ms {
            warm.us.push(us);
            consecutive += 1;
        } else {
            consecutive = 0;
        }
        let counter_ok =
            counter_ready.unwrap_or(false) && counter_hits.map(|hits| hits > 0).unwrap_or(false);
        let ready = if counter_ready.is_some() {
            counter_ok
        } else {
            consecutive >= 3
        };
        if ready {
            ready_at = Some(first.elapsed().as_secs_f64() * 1_000.0);
            break;
        }
        if first.elapsed() >= Duration::from_millis(timeout_ms) {
            break;
        }
        thread::sleep(Duration::from_millis(50));
    }
    WarmResult {
        cold_ms: cold_ms.unwrap_or(0.0),
        ready: ready_at.is_some(),
        ready_ms: ready_at,
        calls,
        warm_p50_ms: warm.p50_ms(),
        warm_p95_ms: warm.p95_ms(),
        counter_ready,
        counter_hits,
    }
}

/// Issue #21 acceptance segments for one GetOverview call, measured on the
/// steady-state path a desktop poll actually takes.
struct OverviewBench {
    timings: Vec<desktop::OverviewTimings>,
    total: Samples,
}

impl OverviewBench {
    fn p95(&self, field: fn(&desktop::OverviewTimings) -> u64) -> f64 {
        let mut values: Vec<u64> = self.timings.iter().map(field).collect();
        values.sort_unstable();
        Samples::ms(percentile(&values, 0.95))
    }

    fn read_only_calls(&self) -> usize {
        self.timings.iter().filter(|timings| timings.read_only).count()
    }

    fn cached_calls(&self) -> usize {
        self.timings
            .iter()
            .filter(|timings| timings.derived_cached)
            .count()
    }
}

/// Quiet single-call benchmark. The first call after a commit recomputes the
/// availability scan and the issue count, so the reported segments cover both
/// the warm (cached) and the cold (generation-miss) shape; `#8 OVERVIEW` prints
/// the whole distribution instead of hiding the miss in an average.
fn bench_overview(service: &Arc<Service>, iters: usize, timeout_ms: u64) -> OverviewBench {
    let mut total = Samples::default();
    let mut timings = Vec::with_capacity(iters);
    for _ in 0..iters {
        let started = Instant::now();
        let result = desktop::overview_measured(service);
        let us = started.elapsed().as_micros() as u64;
        match result {
            Ok((body, timings_for_call)) => {
                let failed = serde_json::from_str::<serde_json::Value>(&body).is_err();
                total.record(us, failed, timeout_ms);
                timings.push(timings_for_call);
            }
            Err(_) => total.record(us, true, timeout_ms),
        }
    }
    total.report("overview", timeout_ms);
    OverviewBench { timings, total }
}

// ===========================================================================
// Issue #94: explicit measurement modes, per-interface gates, classic gate and
// a machine-readable verdict.
// ===========================================================================
//
// The historical harness printed `#8 SLO` lines and nothing else: a run that
// never started `Service` could still read like "9/9 SLO PASS", the recorded
// `classic_*` samples were not gated, the aggregate `mix_all` p95 could hide a
// slow interface and the process exit code was always 0. This section keeps the
// measured scenarios but makes the claim explicit:
//
//  - the mode is named (`#94 MODE`) and selected deliberately: cold/startup,
//    service+projection-warm, forced-degraded (authoritative fallback window
//    right after `Service::start`, no product hook can force the fallback) and
//    steady mixed;
//  - warm/mixed modes must start the runtime tasks and prove the committed
//    projection is published before their gated scenarios run; a missing warm
//    evidence path is INCONCLUSIVE (non-zero exit), never a quiet fallback;
//  - every interface reports p50/p95/p99/max, errors, timeouts, completions and
//    throughput (`#94 PER_INTERFACE`);
//  - per-interface checks are gating and the aggregate is reported only; the
//    classic mixed scenario is an independent gate, so a `classic` timeout
//    cannot be hidden by a clean `mix_all` p95;
//  - the final line is a machine-readable `#94 VERDICT` JSON object whose
//    `verdict`/`exit_code` fields and the cargo test exit status both reflect
//    FAIL/INCONCLUSIVE;
//  - staged memory samples (`#94 MEM`) are reported per phase; a single
//    whole-process HWM is never used to assert a leak;
//  - `LAT_GATE_INJECT=classic_timeout|interface_overrun|states_overrun` injects
//    a synthetic failure into the measured samples to prove the gate trips; the
//    same failure paths are covered by the fast `gate_selftest_*` unit tests,
//    which do not build a fixture.

/// #8 GetStates p95 budget applied per interface under steady mixed load.
const SLO_STATES_MIXED_P95_MS: f64 = 500.0;
/// The classic mixed scenario is an independent gate: its GetStates tail must
/// meet the same user-facing GetStates budget (#8) and it must not time out.
const SLO_CLASSIC_STATES_P95_MS: f64 = 500.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LatMode {
    /// No runtime tasks: pure cold/startup scoped-query path.
    Cold,
    /// Runtime tasks started and the committed status projection is published.
    Warm,
    /// Runtime tasks started, but the gated scenarios run in the authoritative
    /// fallback window immediately after `Service::start` (projection not yet
    /// published). There is no product switch to force a degraded fallback, so
    /// the window is validated from the observed first-call latency and from
    /// the later projection publication.
    Degraded,
    /// Runtime tasks + published projection + all steady mixed workloads.
    Mixed,
}

impl LatMode {
    fn short(self) -> &'static str {
        match self {
            LatMode::Cold => "cold",
            LatMode::Warm => "warm",
            LatMode::Degraded => "degraded",
            LatMode::Mixed => "mixed",
        }
    }

    fn name(self) -> &'static str {
        match self {
            LatMode::Cold => "cold/startup",
            LatMode::Warm => "service+projection-warm",
            LatMode::Degraded => "forced-degraded",
            LatMode::Mixed => "steady-mixed",
        }
    }

    fn requires_service(self) -> bool {
        !matches!(self, LatMode::Cold)
    }

    fn requires_warm(self) -> bool {
        matches!(self, LatMode::Warm | LatMode::Mixed)
    }

    fn runs_mixed(self) -> bool {
        matches!(self, LatMode::Mixed)
    }

    fn measures_degraded_window(self) -> bool {
        matches!(self, LatMode::Degraded)
    }
}

#[derive(Debug)]
struct ModeChoice {
    mode: LatMode,
    explicit: bool,
    /// Invalid/contradictory mode configuration. A problem is INCONCLUSIVE and
    /// exits non-zero instead of silently measuring a different mode.
    problem: Option<String>,
}

/// Resolve the measurement mode. Pure so the mapping is unit-tested.
fn choose_mode(
    requested: Option<&str>,
    service_started: bool,
    warm_requested: bool,
    force_degraded: bool,
) -> ModeChoice {
    let explicit = requested.is_some();
    let mut problem = None;
    let mode = match requested {
        Some(value) => match value.trim().to_ascii_lowercase().as_str() {
            "cold" | "startup" | "cold/startup" => LatMode::Cold,
            "warm" | "projection-warm" | "service+warm" => LatMode::Warm,
            "degraded" | "forced-degraded" => LatMode::Degraded,
            "mixed" | "steady" | "steady-mixed" => LatMode::Mixed,
            other => {
                problem = Some(format!(
                    "unknown LAT_MODE={other:?}; expected cold|warm|degraded|mixed"
                ));
                LatMode::Cold
            }
        },
        None => {
            if force_degraded {
                LatMode::Degraded
            } else if service_started && warm_requested {
                LatMode::Mixed
            } else if service_started {
                // Service started but no warm evidence was requested: measure
                // the startup/degraded window explicitly rather than pretending
                // the run is steady state.
                LatMode::Degraded
            } else if warm_requested {
                problem = Some(
                    "LAT_WARM=1 was requested without LAT_SERVICE=start: the runtime tasks (and \
                     therefore the committed status projection) never start, so no warm evidence \
                     can exist. Refusing to report a warm result (issue #94 acceptance)."
                        .to_string(),
                );
                LatMode::Warm
            } else {
                LatMode::Cold
            }
        }
    };
    if problem.is_none() {
        if mode.requires_service() && !service_started {
            problem = Some(format!(
                "mode {} requires LAT_SERVICE=start (service_started=false)",
                mode.short()
            ));
        }
        if mode == LatMode::Cold && service_started {
            problem = Some(
                "mode cold/startup must not start the runtime tasks (LAT_SERVICE is set)".to_string(),
            );
        }
        if force_degraded && mode != LatMode::Degraded {
            problem = Some(format!(
                "LAT_FORCE_DEGRADED conflicts with LAT_MODE={}",
                mode.short()
            ));
        }
        if matches!(mode, LatMode::Warm | LatMode::Mixed) && !warm_requested && !explicit {
            // Unreachable with the mapping above; keep the invariant explicit.
            problem = Some("warm mode without LAT_WARM=1".to_string());
        }
    }
    ModeChoice {
        mode,
        explicit,
        problem,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Verdict {
    Pass,
    Fail,
    Inconclusive,
}

impl Verdict {
    fn as_str(self) -> &'static str {
        match self {
            Verdict::Pass => "PASS",
            Verdict::Fail => "FAIL",
            Verdict::Inconclusive => "INCONCLUSIVE",
        }
    }

    /// Intended process exit code for CI. The cargo test runner exits 101 when
    /// the gate panics, so any non-zero value is a failure either way.
    fn exit_code(self) -> i32 {
        match self {
            Verdict::Pass => 0,
            Verdict::Fail => 1,
            Verdict::Inconclusive => 2,
        }
    }
}

#[derive(Clone, Debug)]
struct Check {
    scenario: &'static str,
    id: String,
    observed: f64,
    limit: f64,
    unit: &'static str,
    gating: bool,
    note: String,
}

impl Check {
    fn passed(&self) -> bool {
        self.observed <= self.limit
    }
}

struct GateReport {
    mode: LatMode,
    mode_explicit: bool,
    head: String,
    round: u64,
    checks: Vec<Check>,
    inconclusive: Vec<String>,
    disclosures: Vec<String>,
    extra: serde_json::Map<String, serde_json::Value>,
}

impl GateReport {
    fn new(mode: LatMode, mode_explicit: bool, head: String, round: u64) -> Self {
        GateReport {
            mode,
            mode_explicit,
            head,
            round,
            checks: Vec::new(),
            inconclusive: Vec::new(),
            disclosures: Vec::new(),
            extra: serde_json::Map::new(),
        }
    }

    fn add(
        &mut self,
        scenario: &'static str,
        id: impl Into<String>,
        observed: f64,
        limit: f64,
        unit: &'static str,
        note: impl Into<String>,
    ) {
        self.checks.push(Check {
            scenario,
            id: id.into(),
            observed,
            limit,
            unit,
            gating: true,
            note: note.into(),
        });
    }

    fn add_non_gating(
        &mut self,
        scenario: &'static str,
        id: impl Into<String>,
        observed: f64,
        limit: f64,
        unit: &'static str,
        note: impl Into<String>,
    ) {
        self.checks.push(Check {
            scenario,
            id: id.into(),
            observed,
            limit,
            unit,
            gating: false,
            note: note.into(),
        });
    }

    fn add_flag(&mut self, scenario: &'static str, id: impl Into<String>, ok: bool, note: impl Into<String>) {
        self.add(scenario, id, if ok { 0.0 } else { 1.0 }, 0.0, "flag", note);
    }

    /// Errors and timeouts are gating for every interface; the optional p95
    /// budget is the published per-interface tail target. `prefix` keeps the
    /// check id unique per interface (e.g. `mix_states_p95_ms`) so a slow lane
    /// cannot be mistaken for another lane in the failed-check list.
    fn add_stats(
        &mut self,
        scenario: &'static str,
        prefix: &str,
        samples: &Samples,
        p95_limit_ms: Option<f64>,
    ) {
        self.add_stats_mode(scenario, prefix, samples, p95_limit_ms, true);
    }

    /// Errors and timeouts are always gating. `slo_gating` controls whether the
    /// p95 threshold is a steady-state claim (warm/mixed) or a disclosed cold/
    /// degraded-path observation (#94: cold-path results are not steady-state
    /// product performance and must not be presented as one).
    fn add_stats_mode(
        &mut self,
        scenario: &'static str,
        prefix: &str,
        samples: &Samples,
        p95_limit_ms: Option<f64>,
        slo_gating: bool,
    ) {
        let n = samples.us.len();
        self.add(
            scenario,
            format!("{prefix}_errors"),
            samples.errors as f64,
            0.0,
            "count",
            format!("{prefix}: per-interface errors must be zero (n={n})"),
        );
        self.add(
            scenario,
            format!("{prefix}_timeouts"),
            samples.timeouts as f64,
            0.0,
            "count",
            format!("{prefix}: per-interface timeouts must be zero (n={n})"),
        );
        if let Some(limit) = p95_limit_ms {
            if slo_gating {
                self.add(
                    scenario,
                    format!("{prefix}_p95_ms"),
                    samples.p95_ms(),
                    limit,
                    "ms",
                    format!("{prefix}: per-interface p95 budget"),
                );
            } else {
                self.add_non_gating(
                    scenario,
                    format!("{prefix}_p95_ms"),
                    samples.p95_ms(),
                    limit,
                    "ms",
                    format!(
                        "{prefix}: p95 reported only; this mode is not a steady-state claim \
                         (issue #94), the SLO is gating in warm/mixed mode"
                    ),
                );
            }
        }
    }

    fn inconclusive(&mut self, reason: impl Into<String>) {
        self.inconclusive.push(reason.into());
    }

    fn disclose(&mut self, reason: impl Into<String>) {
        self.disclosures.push(reason.into());
    }

    fn set(&mut self, key: &'static str, value: serde_json::Value) {
        self.extra.insert(key.to_string(), value);
    }

    fn verdict(&self) -> Verdict {
        if !self.inconclusive.is_empty() {
            Verdict::Inconclusive
        } else if self.checks.iter().any(|check| check.gating && !check.passed()) {
            Verdict::Fail
        } else {
            Verdict::Pass
        }
    }

    fn all_scenarios_pass(&self) -> bool {
        self.failed().is_empty()
            && self
                .checks
                .iter()
                .all(|check| check.passed())
            && self.inconclusive.is_empty()
            && self.disclosures.is_empty()
    }

    fn failed(&self) -> Vec<String> {
        self.checks
            .iter()
            .filter(|check| check.gating && !check.passed())
            .map(|check| check.id.clone())
            .collect()
    }

    /// Print the per-check lines and the single machine-readable `#94 VERDICT`
    /// line, then panic for FAIL/INCONCLUSIVE so the cargo test exit status is
    /// non-zero. `cargo test` turns the panic into exit code 101.
    fn finish(mut self, loadavg_start: Option<f64>) {
        self.extra
            .insert("loadavg_end".to_string(), serde_json::json!(load_avg1()));
        if let Some(start) = loadavg_start {
            self.extra
                .insert("loadavg_start".to_string(), serde_json::json!(start));
        }
        self.extra
            .insert("peak_rss_kb".to_string(), serde_json::json!(peak_rss_kb()));
        for check in &self.checks {
            println!(
                "#94 CHECK {}",
                serde_json::json!({
                    "scenario": check.scenario,
                    "check": check.id,
                    "observed": check.observed,
                    "limit": check.limit,
                    "unit": check.unit,
                    "gating": check.gating,
                    "verdict": if check.passed() { "PASS" } else { "FAIL" },
                    "note": check.note,
                })
            );
        }
        let verdict = self.verdict();
        let failed = self.failed();
        let non_gating_failed: Vec<String> = self
            .checks
            .iter()
            .filter(|check| !check.gating && !check.passed())
            .map(|check| check.id.clone())
            .collect();
        let all_scenarios_pass = self.all_scenarios_pass();
        println!(
            "#94 VERDICT {}",
            serde_json::json!({
                "schema": 1,
                "issue": 94,
                "head": self.head,
                "round": self.round,
                "mode": self.mode.short(),
                "mode_name": self.mode.name(),
                "mode_explicit": self.mode_explicit,
                "verdict": verdict.as_str(),
                "exit_code": verdict.exit_code(),
                "all_scenarios_pass": all_scenarios_pass,
                "checks": self.checks.len(),
                "checks_failed": failed,
                "checks_non_gating_failed": non_gating_failed,
                "inconclusive": self.inconclusive,
                "disclosures": self.disclosures,
                "context": self.extra,
            })
        );
        match verdict {
            Verdict::Pass => {}
            Verdict::Fail => panic!(
                "issue #94 performance gate FAILED: mode={} failed=[{}]; see the #94 VERDICT line",
                self.mode.short(),
                failed.join(", ")
            ),
            Verdict::Inconclusive => panic!(
                "issue #94 performance gate INCONCLUSIVE: mode={} reasons=[{}]; see the #94 VERDICT line",
                self.mode.short(),
                self.inconclusive.join(" | ")
            ),
        }
    }
}

struct DegradedWindow {
    samples: Samples,
    first_ms: f64,
    #[allow(dead_code)]
    probe: WarmResult,
}

struct SingleScenario<'a> {
    scenario: &'static str,
    samples: &'a Samples,
    p95_limit_ms: Option<f64>,
}

fn gate_single_scenarios(
    report: &mut GateReport,
    scenarios: &[SingleScenario<'_>],
    slo_gating: bool,
) {
    for scenario in scenarios {
        report.add_stats_mode(
            scenario.scenario,
            scenario.scenario,
            scenario.samples,
            scenario.p95_limit_ms,
            slo_gating,
        );
    }
}

/// Gate the steady mixed and classic mixed lanes. The aggregate is reported but
/// never used as the per-interface gate (issue #94): a `mix_all` p95 of 20 ms
/// must not hide a 900 ms `mix_states` p95 or a `classic_all` timeout.
fn gate_mixed_lanes(
    report: &mut GateReport,
    mixed: &BTreeMap<String, Samples>,
    classic: &BTreeMap<String, Samples>,
) {
    for lane in ["states", "dir", "issues", "read", "overview"] {
        let key = format!("mix_{lane}");
        match mixed.get(&key) {
            Some(samples) => {
                // Issue #94 requires the slow GetStates interface to stay
                // visible under mixed load and to meet the user-facing
                // GetStates budget. The single-call #8/#21 budgets for the
                // directory and overview scenarios are gated on their own
                // quiet-path scenarios; there is no published user target for
                // those interfaces *under 48-way mixed load*, so their mixed
                // p95 is reported but not gated (no invented or silently
                // relaxed threshold; the quiet-path SLOs remain gating).
                let limit = match lane {
                    "states" => Some(SLO_STATES_MIXED_P95_MS),
                    _ => None,
                };
                report.add_stats("mixed", &key, samples, limit);
                if matches!(lane, "dir" | "overview") {
                    report.add_non_gating(
                        "mixed",
                        format!("{key}_p95_ms"),
                        samples.p95_ms(),
                        match lane {
                            "dir" => SLO_DIR_P95_MS,
                            _ => SLO_OVERVIEW_TOTAL_P95_MS,
                        },
                        "ms",
                        format!(
                            "{key}: no published user target under 48-way mixed load; the \
                             single-call SLO is gated separately, this value is reported only"
                        ),
                    );
                }
            }
            None => report.inconclusive(format!("mixed lane {key} missing from the report")),
        }
    }
    match mixed.get("mix_all") {
        Some(all) => {
            report.add(
                "mixed",
                "mix_all_errors",
                all.errors as f64,
                0.0,
                "count",
                "aggregate errors must be zero (#8 mixed)",
            );
            report.add(
                "mixed",
                "mix_all_timeouts",
                all.timeouts as f64,
                0.0,
                "count",
                "aggregate timeouts must be zero (#8 mixed SLO)",
            );
            report.add_non_gating(
                "mixed",
                "mix_all_p95_ms",
                all.p95_ms(),
                0.0,
                "ms",
                "aggregate p95 is reported for comparability only; the per-interface checks above \
                 are the gate (issue #94)",
            );
        }
        None => report.inconclusive("mixed_all aggregate missing from the report"),
    }

    // Classic mixed scenario: an independent gate (issue #94). Its samples were
    // recorded historically but never checked, so a classic timeout used to be
    // invisible while the `#8 SLO` block read PASS.
    for lane in ["states", "dir", "read"] {
        let key = format!("classic_{lane}");
        match classic.get(&key) {
            Some(samples) => {
                let limit = if lane == "states" {
                    Some(SLO_CLASSIC_STATES_P95_MS)
                } else {
                    None
                };
                report.add_stats("classic", &key, samples, limit);
            }
            None => report.inconclusive(format!("classic lane {key} missing from the report")),
        }
    }
    match classic.get("classic_all") {
        Some(all) => {
            report.add(
                "classic",
                "classic_all_timeouts",
                all.timeouts as f64,
                0.0,
                "count",
                "independent classic gate: the classic mixed scenario must not time out \
                 (issue #94); not replaceable by mix_all p95",
            );
            report.add(
                "classic",
                "classic_all_errors",
                all.errors as f64,
                0.0,
                "count",
                "classic mixed errors must be zero",
            );
            report.add_non_gating(
                "classic",
                "classic_all_p95_ms",
                all.p95_ms(),
                0.0,
                "ms",
                "classic aggregate p95 informational; per-interface classic_* checks are the gate",
            );
        }
        None => report.inconclusive("classic_all aggregate missing from the report"),
    }
}

/// Warm evidence: the mode must start the runtime tasks and prove the committed
/// projection is published before the gated warm measurements. The projection
/// accessor is `pub(crate)`, so an integration test infers publication from
/// latency (three consecutive sub-threshold GetStates calls after a slower
/// first call) unless the product exposes projection counters in the GetStates
/// document, in which case those counters are the primary evidence.
fn gate_warm_evidence(report: &mut GateReport, warm: &WarmResult, threshold_ms: f64) {
    report.set(
        "projection",
        serde_json::json!({
            "ready": warm.ready,
            "ready_ms": warm.ready_ms,
            "calls": warm.calls,
            "cold_probe_ms": warm.cold_ms,
            "warm_p50_ms": warm.warm_p50_ms,
            "warm_p95_ms": warm.warm_p95_ms,
            "threshold_ms": threshold_ms,
            "counter_ready": warm.counter_ready,
            "counter_hits": warm.counter_hits,
            "evidence": if warm.counter_ready.is_some() { "product_counters" } else { "latency_inference" },
        }),
    );
    if !warm.ready {
        report.inconclusive(format!(
            "projection never published warm evidence: no run of three consecutive GetStates calls \
             at or below {threshold_ms}ms and no product projection counters (calls={}); a warm \
             result must not be inferred from a fallback run",
            warm.calls
        ));
        return;
    }
    report.add_flag(
        "warm",
        "projection_ready",
        true,
        "committed status projection published before the gated warm scenarios",
    );
    report.add(
        "warm",
        "projection_warm_p95_ms",
        warm.warm_p95_ms,
        threshold_ms,
        "ms",
        "warm projection tail must stay below the publication threshold",
    );
    if warm.counter_ready.is_none() {
        report.disclose(
            "warm readiness is inferred from latency only (Service::status_projection is \
             crate-private and the GetStates document exposes no projection counters); \
             the first post-start GetStates probe documents the authoritative fallback cost",
        );
        if warm.cold_ms <= threshold_ms {
            report.inconclusive(format!(
                "warm evidence is ambiguous: the first GetStates probe after start was already \
                 {:.1}ms (<= {threshold_ms}ms), so a fast authoritative fallback cannot be excluded \
                 and no projection counter is available to confirm the provenance",
                warm.cold_ms
            ));
        }
    }
}

/// Product-side projection counters, if the integrated tree exposes them
/// additively in the GetStates document (issue #30 adds ready/degraded/hit
/// counters to the projection). Older trees simply return `None` and the
/// latency inference path above is used.
fn projection_counter_evidence(body: &str) -> Option<(bool, Option<u64>)> {
    let value: serde_json::Value = serde_json::from_str(body).ok()?;
    for key in ["projection", "status_projection", "statusProjection"] {
        let Some(object) = value.get(key) else {
            continue;
        };
        let ready = object.get("ready").and_then(|value| value.as_bool());
        let hits = object
            .get("hits")
            .or_else(|| object.get("hit"))
            .and_then(|value| value.as_u64().or_else(|| value.as_bool().map(u64::from)));
        if ready.is_some() || hits.is_some() {
            return Some((ready.unwrap_or(false), hits));
        }
    }
    None
}

fn proc_status_kb(field: &str) -> Option<u64> {
    let status = std::fs::read_to_string("/proc/self/status").ok()?;
    for line in status.lines() {
        if let Some(value) = line.strip_prefix(field) {
            return value.split_whitespace().next()?.parse().ok();
        }
    }
    None
}

/// Staged memory sample: fixture build / Store open / projection warm /
/// concurrent peak / thread exit / idle. A single whole-process HWM is never a
/// leak assertion (issue #94 #6); the gate only reports the phases.
fn print_mem_phase(mode: LatMode, phase: &str, context: serde_json::Value) {
    println!(
        "#94 MEM {}",
        serde_json::json!({
            "mode": mode.short(),
            "phase": phase,
            "rss_kb": proc_status_kb("VmRSS:"),
            "hwm_kb": proc_status_kb("VmHWM:"),
            "context": context,
        })
    );
}

fn emit_per_interface(
    mode: LatMode,
    interface: &str,
    scenario: &str,
    samples: &Samples,
    dur_s: f64,
    concurrency: usize,
    timeout_ms: u64,
) {
    let n = samples.us.len();
    println!(
        "#94 PER_INTERFACE {}",
        serde_json::json!({
            "mode": mode.short(),
            "interface": interface,
            "scenario": scenario,
            "n": n,
            "errors": samples.errors,
            "timeouts": samples.timeouts,
            "p50_ms": samples.p50_ms(),
            "p95_ms": samples.p95_ms(),
            "p99_ms": samples.p99_ms(),
            "max_ms": samples.max_ms(),
            "dur_s": dur_s,
            "ops_per_sec": if dur_s > 0.0 { n as f64 / dur_s } else { 0.0 },
            "concurrency": concurrency,
            "timeout_ms": timeout_ms,
        })
    );
}

/// Synthetic gate failure injection for the running harness (acceptance: an
/// injected classic timeout / single-interface overrun must make the final
/// report stop claiming "all scenarios pass"). Only the sample containers are
/// touched before adjudication, so the normal gating code path is exercised.
fn inject_gate_failure(
    label: &str,
    states_root: &mut Samples,
    mixed: &mut BTreeMap<String, Samples>,
    classic: &mut BTreeMap<String, Samples>,
) {
    match label {
        "classic_timeout" => {
            if let Some(samples) = classic.get_mut("classic_states") {
                samples.record(50_000_000, false, 15_000);
            }
            if let Some(samples) = classic.get_mut("classic_all") {
                samples.record(50_000_000, false, 15_000);
            }
        }
        "interface_overrun" => {
            if let Some(samples) = mixed.get_mut("mix_states") {
                let n = samples.us.len().max(40);
                for _ in 0..n {
                    samples.record(900_000, false, 15_000);
                }
            }
        }
        "states_overrun" => {
            let n = states_root.us.len().max(40);
            for _ in 0..n {
                states_root.record(900_000, false, 15_000);
            }
        }
        other => println!("#94 NOTE unknown LAT_GATE_INJECT={other:?} (expected classic_timeout|interface_overrun|states_overrun)"),
    }
}

// ---------------------------------------------------------------------------
// Fast gate self-tests (no fixture, no timing): the acceptance criteria of
// issue #94 as executable assertions. These run with the default `cargo test`
// filter, so a plain `--test status_latency_210k gate_selftest_` checks the
// gate logic in seconds.
// ---------------------------------------------------------------------------

fn synthetic_samples(p95_us: u64, n: usize) -> Samples {
    let mut samples = Samples::default();
    for _ in 0..n {
        samples.record(p95_us, false, 15_000);
    }
    samples
}

fn synthetic_mixed_maps(
    mixed_states_p95_us: u64,
    classic_timeouts: u64,
) -> (BTreeMap<String, Samples>, BTreeMap<String, Samples>) {
    let mut mixed = BTreeMap::new();
    for lane in ["states", "dir", "issues", "read", "overview"] {
        let us = if lane == "states" {
            mixed_states_p95_us
        } else {
            5_000
        };
        mixed.insert(format!("mix_{lane}"), synthetic_samples(us, 100));
    }
    let mut all = synthetic_samples(5_000, 500);
    all.errors = 0;
    all.timeouts = 0;
    mixed.insert("mix_all".to_string(), all);

    let mut classic = BTreeMap::new();
    for lane in ["states", "dir", "read"] {
        let us = if lane == "states" { 10_000 } else { 5_000 };
        classic.insert(format!("classic_{lane}"), synthetic_samples(us, 100));
    }
    let mut classic_all = synthetic_samples(5_000, 300);
    classic_all.timeouts = classic_timeouts;
    classic.insert("classic_all".to_string(), classic_all);
    (mixed, classic)
}

#[test]
fn gate_selftest_injected_classic_timeout_cannot_hide_behind_clean_mixed_aggregate() {
    let (mixed, classic) = synthetic_mixed_maps(10_000, 1);
    let mut report = GateReport::new(LatMode::Mixed, true, "selftest".to_string(), 0);
    gate_mixed_lanes(&mut report, &mixed, &classic);
    assert_eq!(
        report.verdict(),
        Verdict::Fail,
        "an injected classic timeout must fail the gate even when mix_all is clean"
    );
    let failed = report.failed();
    assert!(
        failed.iter().any(|id| id == "classic_all_timeouts"),
        "classic_all_timeouts must be an independent gating check, failed={failed:?}"
    );
}

#[test]
fn gate_selftest_injected_interface_overrun_is_not_hidden_by_fast_aggregate() {
    // mix_states p95 = 900 ms > 500 ms budget while the aggregate (all lanes)
    // averages 5 ms, which is exactly the historical masking shape.
    let (mixed, classic) = synthetic_mixed_maps(900_000, 0);
    let mut report = GateReport::new(LatMode::Mixed, true, "selftest".to_string(), 0);
    gate_mixed_lanes(&mut report, &mixed, &classic);
    assert_eq!(report.verdict(), Verdict::Fail);
    let failed = report.failed();
    assert!(
        failed.iter().any(|id| id == "mix_states_p95_ms"),
        "the slow GetStates interface must be gated on its own, failed={failed:?}"
    );
}

#[test]
fn gate_selftest_interface_timeout_is_not_hidden_by_aggregate() {
    let (mut mixed, classic) = synthetic_mixed_maps(10_000, 0);
    if let Some(samples) = mixed.get_mut("mix_read") {
        samples.record(50_000_000, false, 15_000);
    }
    let mut report = GateReport::new(LatMode::Mixed, true, "selftest".to_string(), 0);
    gate_mixed_lanes(&mut report, &mixed, &classic);
    assert_eq!(report.verdict(), Verdict::Fail);
    assert!(report
        .failed()
        .iter()
        .any(|id| id == "mix_read_timeouts"));
}

#[test]
fn gate_selftest_clean_mixed_report_passes() {
    let (mixed, classic) = synthetic_mixed_maps(10_000, 0);
    let mut report = GateReport::new(LatMode::Mixed, true, "selftest".to_string(), 0);
    gate_mixed_lanes(&mut report, &mixed, &classic);
    assert_eq!(report.verdict(), Verdict::Pass);
    assert!(report.failed().is_empty());
}

#[test]
fn gate_selftest_cold_mode_discloses_slo_without_steady_state_claim() {
    let slow = synthetic_samples(900_000, 100);
    let mut report = GateReport::new(LatMode::Cold, true, "selftest".to_string(), 0);
    gate_single_scenarios(
        &mut report,
        &[SingleScenario {
            scenario: "states_root",
            samples: &slow,
            p95_limit_ms: Some(SLO_STATES_P95_MS),
        }],
        false,
    );
    // Cold/degraded errors and timeouts are still gating, so a slow but correct
    // cold run does not fail the process...
    assert_eq!(report.verdict(), Verdict::Pass);
    // ...but it must never claim "all scenarios passed": the cold SLO miss is
    // disclosed (issue #94: cold-path results are not steady-state product
    // performance).
    assert!(!report.all_scenarios_pass());
    let disclosed: Vec<String> = report
        .checks
        .iter()
        .filter(|check| !check.gating && !check.passed())
        .map(|check| check.id.clone())
        .collect();
    assert!(
        disclosed.iter().any(|id| id == "states_root_p95_ms"),
        "cold SLO miss must be disclosed, disclosed={disclosed:?}"
    );
    // The same measurement in warm/mixed mode stays a gating steady-state claim.
    let mut warm_report = GateReport::new(LatMode::Warm, true, "selftest".to_string(), 0);
    gate_single_scenarios(
        &mut warm_report,
        &[SingleScenario {
            scenario: "states_root",
            samples: &slow,
            p95_limit_ms: Some(SLO_STATES_P95_MS),
        }],
        true,
    );
    assert_eq!(warm_report.verdict(), Verdict::Fail);
}

#[test]
fn gate_selftest_missing_warm_evidence_is_never_pass() {
    let choice = choose_mode(None, false, true, false);
    assert!(
        choice.problem.is_some(),
        "LAT_WARM=1 without LAT_SERVICE=start must be rejected"
    );
    let mut report = GateReport::new(choice.mode, choice.explicit, "selftest".to_string(), 0);
    report.inconclusive(choice.problem.unwrap());
    assert_eq!(report.verdict(), Verdict::Inconclusive);
    assert_ne!(report.verdict(), Verdict::Pass);
    assert_ne!(report.verdict().exit_code(), 0);
}

#[test]
fn gate_selftest_mode_mapping() {
    assert_eq!(choose_mode(None, false, false, false).mode, LatMode::Cold);
    assert_eq!(choose_mode(None, true, true, false).mode, LatMode::Mixed);
    assert_eq!(
        choose_mode(Some("warm"), true, false, false).mode,
        LatMode::Warm
    );
    assert_eq!(
        choose_mode(Some("degraded"), true, false, false).mode,
        LatMode::Degraded
    );
    assert_eq!(
        choose_mode(Some("cold"), false, false, false).mode,
        LatMode::Cold
    );
    // Contradictory configurations are incomplete configurations, not silent
    // mode switches.
    assert!(choose_mode(Some("warm"), false, true, false).problem.is_some());
    assert!(choose_mode(Some("cold"), true, false, false).problem.is_some());
    assert!(choose_mode(Some("bogus"), false, false, false).problem.is_some());
    assert!(choose_mode(Some("mixed"), true, false, false).problem.is_none());
    assert!(choose_mode(Some("degraded"), true, false, true).problem.is_none());
}

#[test]
fn gate_selftest_verdict_exit_codes_are_nonzero_on_failure() {
    let mut report = GateReport::new(LatMode::Mixed, true, "selftest".to_string(), 0);
    report.add("mixed", "engineered_fail", 2.0, 1.0, "count", "synthetic");
    assert_eq!(report.verdict(), Verdict::Fail);
    assert_eq!(report.verdict().exit_code(), 1);
    let report = GateReport::new(LatMode::Warm, true, "selftest".to_string(), 0);
    let mut report = report;
    report.inconclusive("missing warm evidence");
    assert_eq!(report.verdict(), Verdict::Inconclusive);
    assert_eq!(report.verdict().exit_code(), 2);
}

#[test]
#[ignore]
fn status_latency_210k() {
    let iters = env_usize("LAT_ITERS", 120);
    let batch_iters = env_usize("LAT_BATCH_ITERS", 24);
    let dir_iters = env_usize("LAT_DIR_ITERS", 120);
    let overview_iters = env_usize("LAT_OVERVIEW_ITERS", 40);
    let mix_seconds = env_u64("LAT_MIX_SECONDS", 15);
    let lock_ms = env_u64("LAT_LOCK_MS", 2_000);
    let timeout_ms = env_u64("LAT_TIMEOUT_MS", 15_000);
    let service_started = matches!(
        env::var("LAT_SERVICE").as_deref(),
        Ok("start") | Ok("1") | Ok("true")
    );
    let warm_requested = env_flag("LAT_WARM");
    let warm_threshold_ms = env::var("LAT_WARM_MS")
        .ok()
        .and_then(|value| value.parse().ok())
        // A warm 210k-node GetStates render is ~307 ms on the 32-core host, so a
        // 50 ms threshold can never mark publication. The default is the point
        // where latency has settled at the warm-path level, not a 50 ms SLO.
        .unwrap_or(400.0);
    let warm_timeout_ms = env_u64("LAT_WARM_TIMEOUT_MS", 60_000);
    let head = env::var("LAT_HEAD").unwrap_or_else(|_| "unset".to_string());
    let round = env_u64("LAT_ROUND", 0);
    let choice = choose_mode(
        env::var("LAT_MODE").ok().as_deref(),
        service_started,
        warm_requested,
        env_flag("LAT_FORCE_DEGRADED"),
    );
    let mode = choice.mode;
    let loadavg_start = load_avg1();
    let mut report = GateReport::new(mode, choice.explicit, head, round);
    report.set("service_started", serde_json::json!(service_started));
    report.set("warm_requested", serde_json::json!(warm_requested));
    report.set("warm_threshold_ms", serde_json::json!(warm_threshold_ms));
    report.set("warm_timeout_ms", serde_json::json!(warm_timeout_ms));
    report.set("timeout_ms", serde_json::json!(timeout_ms));
    report.set(
        "build",
        serde_json::json!({
            "profile": "release",
            "features": "all-features",
            "locked": true,
            "crate_version": env!("CARGO_PKG_VERSION"),
        }),
    );
    report.set(
        "concurrency",
        serde_json::json!({
            "single_call": 1,
            "mix_threads": MIX_THREADS,
            "states_concurrent": 16,
        }),
    );
    println!(
        "#94 MODE {}",
        serde_json::json!({
            "mode": mode.short(),
            "mode_name": mode.name(),
            "mode_explicit": choice.explicit,
            "service_started": service_started,
            "warm_requested": warm_requested,
            "force_degraded": env_flag("LAT_FORCE_DEGRADED"),
            "head": report.head.clone(),
            "round": round,
        })
    );
    println!(
        "#8 ENV {}",
        serde_json::json!({
            "iters": iters,
            "batch_iters": batch_iters,
            "dir_iters": dir_iters,
            "overview_iters": overview_iters,
            "mix_seconds": mix_seconds,
            "lock_ms": lock_ms,
            "timeout_ms": timeout_ms,
            "threads": MIX_THREADS,
            "service": if service_started { "start" } else { "new" },
            "warm": warm_requested,
            "warm_threshold_ms": warm_threshold_ms,
            "warm_timeout_ms": warm_timeout_ms,
            "mode": mode.short(),
            "loadavg_start": loadavg_start,
        })
    );
    print_mem_phase(
        mode,
        "process_start",
        serde_json::json!({"loadavg_start": loadavg_start}),
    );
    if let Some(problem) = choice.problem.clone() {
        report.inconclusive(problem);
        report.finish(loadavg_start);
        return;
    }

    let fixture = build_fixture().expect("build fixture");
    let Fixture {
        dir,
        temp,
        store,
        facts,
    } = fixture;
    print_mem_phase(
        mode,
        "fixture_built",
        serde_json::json!({
            "db_bytes": facts.db_bytes,
            "build_ms": facts.build_ms,
            "nodes": facts.nodes,
            "cached": facts.cached,
            "drafts": facts.drafts,
            "confirmed_history_rows": env_usize("LAT_HISTORY_ROWS", 0),
            "quarantine_records": env_usize("LAT_QUARANTINE", 0),
        }),
    );
    let service = Service::new(
        profile(PathBuf::from("/tmp/drive-native-issue8-mount")),
        store,
        PathBuf::from("/nonexistent-worker"),
    )
    .expect("service");
    print_mem_phase(mode, "store_open_service_new", serde_json::json!({}));
    let tasks = if mode.requires_service() {
        let tasks = service.start();
        println!(
            "#8 NOTE service_started=1 (runtime background tasks running; on a #8 cache tree this starts the committed status projection worker)"
        );
        print_mem_phase(mode, "service_started", serde_json::json!({}));
        Some(tasks)
    } else {
        None
    };

    // Warm/mixed: the committed projection must be published before the gated
    // warm scenarios run. A missing/failed warm evidence path is INCONCLUSIVE
    // and exits non-zero; the run never silently measures the fallback path and
    // reports it as warm.
    let mut warm: Option<WarmResult> = None;
    if mode.requires_warm() {
        let warm_result = warm_status_projection(&service, warm_timeout_ms, warm_threshold_ms);
        println!(
            "#8 WARM {}",
            serde_json::json!({
                "cold_ms": warm_result.cold_ms,
                "ready": warm_result.ready,
                "ready_ms": warm_result.ready_ms,
                "calls": warm_result.calls,
                "warm_p50_ms": warm_result.warm_p50_ms,
                "warm_p95_ms": warm_result.warm_p95_ms,
                "threshold_ms": warm_threshold_ms,
                "timeout_ms": warm_timeout_ms,
                "counter_ready": warm_result.counter_ready,
                "counter_hits": warm_result.counter_hits,
            })
        );
        gate_warm_evidence(&mut report, &warm_result, warm_threshold_ms);
        print_mem_phase(
            mode,
            "projection_warm",
            serde_json::json!({"ready": warm_result.ready, "calls": warm_result.calls}),
        );
        warm = Some(warm_result);
    }

    // Degraded: measure the authoritative fallback window immediately after
    // `Service::start` (before the projection worker publishes). There is no
    // product switch to force a fallback, so the window is validated by the
    // observed first-call latency and by the later publication.
    let mut degraded_window: Option<DegradedWindow> = None;
    if mode.measures_degraded_window() {
        let window_started = Instant::now();
        let mut samples = Samples::default();
        for _ in 0..3 {
            let started = Instant::now();
            let result = desktop::states(&service, vec![".".to_string()]);
            let us = started.elapsed().as_micros() as u64;
            samples.record(us, result.is_err(), timeout_ms);
        }
        let window_measure_ms = window_started.elapsed().as_secs_f64() * 1_000.0;
        let first_ms = Samples::ms(*samples.us.first().unwrap_or(&0));
        // Poll after the degraded samples so publication timing is known but
        // cannot retroactively turn the samples into warm samples.
        let probe = warm_status_projection(&service, warm_timeout_ms, warm_threshold_ms);
        let valid = first_ms > warm_threshold_ms && probe.ready;
        println!(
            "#8 WARM {}",
            serde_json::json!({
                "cold_ms": first_ms,
                "probe_cold_ms": probe.cold_ms,
                "ready": probe.ready,
                "ready_ms": probe.ready_ms,
                "calls": probe.calls,
                "warm_p50_ms": probe.warm_p50_ms,
                "warm_p95_ms": probe.warm_p95_ms,
                "threshold_ms": warm_threshold_ms,
                "timeout_ms": warm_timeout_ms,
                "degraded_window_valid": valid,
            })
        );
        if !valid {
            report.inconclusive(format!(
                "degraded window could not be established: first degraded GetStates was {first_ms:.1}ms \
                 (warm threshold {warm_threshold_ms}ms, so the authoritative fallback cannot be excluded \
                 by latency) and/or the projection never published afterwards (ready={})",
                probe.ready
            ));
        }
        report.set(
            "degraded_window",
            serde_json::json!({
                "calls": samples.us.len(),
                "first_ms": first_ms,
                "window_measure_ms": window_measure_ms,
                "ready_after_ms": probe.ready_ms,
                "projection_ready_later": probe.ready,
                "valid": valid,
                "note": "authoritative fallback window immediately after Service::start; no product hook forces a degraded fallback",
            }),
        );
        print_mem_phase(
            mode,
            "degraded_window",
            serde_json::json!({"first_ms": first_ms, "ready_after": probe.ready}),
        );
        degraded_window = Some(DegradedWindow {
            samples,
            first_ms,
            probe,
        });
    }

    // ---------------- single-call scenarios ----------------
    let mut states_root = match &degraded_window {
        Some(window) => window.samples.clone(),
        None => bench_states(
            &service,
            "states_root",
            vec![".".to_string()],
            iters,
            timeout_ms,
        ),
    };
    let started = Instant::now();
    let states_dir = bench_states(
        &service,
        "states_dir2000",
        vec!["d000".to_string()],
        iters,
        timeout_ms,
    );
    let states_dir_dur = started.elapsed().as_secs_f64();
    let batch_paths: Vec<String> = (0..45).map(|i| format!("d{i:03}")).collect();
    let started = Instant::now();
    let states_batch = bench_states(
        &service,
        "states_batch45",
        batch_paths,
        batch_iters,
        timeout_ms,
    );
    let states_batch_dur = started.elapsed().as_secs_f64();
    let started = Instant::now();
    let dir2000 = bench_directories(&service, &facts, dir_iters, timeout_ms);
    let dir2000_dur = started.elapsed().as_secs_f64();
    let started = Instant::now();
    let overview = bench_overview(&service, overview_iters, timeout_ms);
    let overview_dur = started.elapsed().as_secs_f64();
    let first_overview = overview.timings.first().copied();
    println!(
        "#8 OVERVIEW {}",
        serde_json::json!({
            "n": overview.total.us.len(),
            "errors": overview.total.errors,
            "read_only_calls": overview.read_only_calls(),
            "cached_calls": overview.cached_calls(),
            "first_total_ms": first_overview.map(|timings| Samples::ms(timings.total_us)),
            "first_cached": first_overview.map(|timings| timings.derived_cached),
            "total_p50_ms": overview.total.p50_ms(),
            "total_p95_ms": overview.total.p95_ms(),
            "total_max_ms": overview.total.max_ms(),
            "query_p95_ms": overview.p95(|timings| timings.query_us),
            "serialize_p95_ms": overview.p95(|timings| timings.serialize_us),
            "lock_wait_p95_ms": overview.p95(|timings| timings.lock_wait_us),
            "lock_hold_p95_ms": overview.p95(|timings| timings.lock_hold_us),
            "lock_hold_max_ms": Samples::ms(
                overview.timings.iter().map(|timings| timings.lock_hold_us).max().unwrap_or(0)
            ),
        })
    );
    print_mem_phase(mode, "single_scenarios", serde_json::json!({}));

    let fuse_skipped_explicitly = env::var("LAT_FUSE").map(|value| value == "0").unwrap_or(false);
    let fuse = if fuse_skipped_explicitly {
        None
    } else {
        fuse_mount(&service).and_then(|(mount, session)| {
            let bench = bench_fuse(mount.path(), dir_iters.min(60), timeout_ms);
            drop(session);
            drop(mount);
            bench
        })
    };
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

    // ---------------- steady mixed / classic workloads ----------------
    let mut states_conc: Option<Samples> = None;
    let mut mixed_maps: Option<(BTreeMap<String, Samples>, BTreeMap<String, Samples>)> = None;
    if mode.runs_mixed() {
        let conc_started = Instant::now();
        let conc = bench_states_concurrent(&service, 16, mix_seconds, timeout_ms);
        let conc_dur = conc_started.elapsed().as_secs_f64();
        emit_per_interface(
            mode,
            "states_conc16",
            "states_conc16",
            &conc,
            conc_dur,
            16,
            timeout_ms,
        );
        let mixed = mixed_load(&service, &facts, mix_seconds, timeout_ms, "mix", false);
        let classic = mixed_load(&service, &facts, mix_seconds, timeout_ms, "classic", true);
        states_conc = Some(conc);
        mixed_maps = Some((mixed, classic));
        print_mem_phase(mode, "mixed_peak", serde_json::json!({}));
    }

    // Injected failure for the acceptance self-test of the running harness:
    // the sample containers are mutated before adjudication, so the regular
    // gating code path produces the verdict.
    if let Some(label) = env::var("LAT_GATE_INJECT").ok() {
        match mixed_maps.as_mut() {
            Some((mixed, classic)) => {
                inject_gate_failure(&label, &mut states_root, mixed, classic);
                println!("#94 INJECT {}", serde_json::json!({"label": label}));
            }
            None => println!(
                "#94 NOTE LAT_GATE_INJECT={label:?} ignored: mode {} runs no mixed/classic workloads",
                mode.short()
            ),
        }
    }

    // ---------------- per-interface metrics ----------------
    // `bench_states` does not return its wall clock, so the per-interface
    // throughput for the sequential scenarios is reported as completions with
    // dur_s=0; the mixed lanes report their measured throughput.
    let states_root_dur = 0.0;
    emit_per_interface(
        mode,
        "states_root",
        "states_root",
        &states_root,
        states_root_dur,
        1,
        timeout_ms,
    );
    emit_per_interface(
        mode,
        "states_dir2000",
        "states_dir2000",
        &states_dir,
        states_dir_dur,
        1,
        timeout_ms,
    );
    emit_per_interface(
        mode,
        "states_batch45",
        "states_batch45",
        &states_batch,
        states_batch_dur,
        1,
        timeout_ms,
    );
    emit_per_interface(
        mode,
        "dir2000",
        "dir2000",
        &dir2000,
        dir2000_dur,
        1,
        timeout_ms,
    );
    emit_per_interface(
        mode,
        "overview",
        "overview",
        &overview.total,
        overview_dur,
        1,
        timeout_ms,
    );
    if let Some(fuse) = &fuse {
        emit_per_interface(mode, "fuse_dir2000", "fuse_dir2000", &fuse.warm, 0.0, 1, timeout_ms);
        emit_per_interface(mode, "fuse_read1m", "fuse_read1m", &fuse.read_warm, 0.0, 1, timeout_ms);
    }
    emit_per_interface(
        mode,
        "lock_hold_post",
        "lock_hold_post",
        &lock_post,
        0.0,
        1,
        timeout_ms,
    );
    if let Some((mixed, classic)) = &mixed_maps {
        for lane in ["states", "dir", "issues", "read", "overview"] {
            if let Some(samples) = mixed.get(&format!("mix_{lane}")) {
                emit_per_interface(
                    mode,
                    &format!("mixed:{lane}"),
                    &format!("mix_{lane}"),
                    samples,
                    mix_seconds as f64,
                    MIX_THREADS,
                    timeout_ms,
                );
            }
        }
        if let Some(samples) = mixed.get("mix_all") {
            emit_per_interface(
                mode,
                "mixed:all",
                "mix_all",
                samples,
                mix_seconds as f64,
                MIX_THREADS,
                timeout_ms,
            );
        }
        for lane in ["states", "dir", "read"] {
            if let Some(samples) = classic.get(&format!("classic_{lane}")) {
                emit_per_interface(
                    mode,
                    &format!("classic:{lane}"),
                    &format!("classic_{lane}"),
                    samples,
                    mix_seconds as f64,
                    MIX_THREADS,
                    timeout_ms,
                );
            }
        }
        if let Some(samples) = classic.get("classic_all") {
            emit_per_interface(
                mode,
                "classic:all",
                "classic_all",
                samples,
                mix_seconds as f64,
                MIX_THREADS,
                timeout_ms,
            );
        }
    }

    // ---------------- gates ----------------
    let mut single_scenarios = vec![
        SingleScenario {
            scenario: "states_root",
            samples: &states_root,
            p95_limit_ms: Some(SLO_STATES_P95_MS),
        },
        SingleScenario {
            scenario: "states_dir2000",
            samples: &states_dir,
            p95_limit_ms: None,
        },
        SingleScenario {
            scenario: "states_batch45",
            samples: &states_batch,
            p95_limit_ms: None,
        },
        SingleScenario {
            scenario: "dir2000",
            samples: &dir2000,
            p95_limit_ms: Some(SLO_DIR_P95_MS),
        },
        SingleScenario {
            scenario: "lock_hold_post",
            samples: &lock_post,
            p95_limit_ms: None,
        },
    ];
    if let Some(fuse) = &fuse {
        single_scenarios.push(SingleScenario {
            scenario: "fuse_dir2000",
            samples: &fuse.warm,
            p95_limit_ms: Some(SLO_DIR_P95_MS),
        });
        single_scenarios.push(SingleScenario {
            scenario: "fuse_read1m",
            samples: &fuse.read_warm,
            p95_limit_ms: None,
        });
    }
    // Issue #94: warm/mixed are the steady-state claims; cold/degraded keep the
    // same measured SLO values but report them as disclosed observations rather
    // than gating claims (the cold fallback path is not steady-state product
    // performance). Errors/timeouts stay gating in every mode.
    let slo_gating = matches!(mode, LatMode::Warm | LatMode::Mixed);
    gate_single_scenarios(&mut report, &single_scenarios, slo_gating);
    let overview_total_limit = SLO_OVERVIEW_TOTAL_P95_MS;
    let overview_lock_limit = SLO_OVERVIEW_LOCK_HOLD_P95_MS;
    if slo_gating {
        report.add(
            "overview",
            "overview_total_p95_ms",
            overview.total.p95_ms(),
            overview_total_limit,
            "ms",
            "GetOverview total p95 budget (#21)",
        );
        report.add(
            "overview",
            "overview_lock_hold_p95_ms",
            overview.p95(|timings| timings.lock_hold_us),
            overview_lock_limit,
            "ms",
            "GetOverview must not serialize behind the writer (#21)",
        );
    } else {
        report.add_non_gating(
            "overview",
            "overview_total_p95_ms",
            overview.total.p95_ms(),
            overview_total_limit,
            "ms",
            "GetOverview total p95 reported only: this mode is not a steady-state claim (issue #94)",
        );
        report.add_non_gating(
            "overview",
            "overview_lock_hold_p95_ms",
            overview.p95(|timings| timings.lock_hold_us),
            overview_lock_limit,
            "ms",
            "GetOverview lock-hold p95 reported only: this mode is not a steady-state claim (issue #94)",
        );
    }
    report.set(
        "slo_claim",
        serde_json::json!({
            "steady_state": slo_gating,
            "note": if slo_gating {
                "warm/mixed: the SLO thresholds are gating steady-state claims"
            } else {
                "cold/degraded: SLO values are disclosed observations, not steady-state claims (issue #94); errors/timeouts remain gating"
            },
        }),
    );
    for probe in &probes {
        report.add_flag(
            "lock_hold",
            format!("lock_hold_{}_responsive", probe.name),
            probe.completed_while_locked,
            format!(
                "Get{} must answer while the writer lock is held for {}ms",
                probe.name, lock_ms
            ),
        );
    }
    match &fuse {
        Some(_) => {}
        None if fuse_skipped_explicitly => report.disclose(
            "kernel FUSE SLO not measured: LAT_FUSE=0 explicitly skips the mount (not a pass)",
        ),
        None => report.inconclusive(
            "kernel FUSE mount unavailable (no /dev/fuse?); fuse_dir2000 SLO cannot be verified",
        ),
    }

    let mut measured = vec![
        "states_root",
        "states_dir2000",
        "states_batch45",
        "dir2000",
        "overview",
        "fuse_dir2000",
        "fuse_read1m",
        "lock_hold",
    ];
    let mut not_measured = Vec::new();
    if mode.requires_warm() {
        measured.push("projection_warm_evidence");
    } else {
        not_measured.push("projection_warm_evidence");
    }
    if mode.requires_service() {
        measured.push("degraded_fallback_window");
    }
    if let (Some((mixed, classic)), Some(conc)) = (&mixed_maps, &states_conc) {
        report.add_stats(
            "states_conc16",
            "states_conc16",
            conc,
            Some(SLO_STATES_MIXED_P95_MS),
        );
        gate_mixed_lanes(&mut report, mixed, classic);
        measured.push("states_conc16");
        measured.push("mixed48");
        measured.push("classic48");
    } else {
        not_measured.push("states_conc16");
        not_measured.push("mixed48");
        not_measured.push("classic48");
    }
    report.set(
        "scope",
        serde_json::json!({"measured": measured, "not_measured": not_measured}),
    );
    report.set(
        "scales",
        serde_json::json!({
            "nodes": facts.nodes,
            "cached": facts.cached,
            "pins": facts.pins,
            "drafts": facts.drafts,
            "confirmed_history_rows": env_usize("LAT_HISTORY_ROWS", 0),
            "quarantine_records": env_usize("LAT_QUARANTINE", 0),
            "dir_entries": facts.dir_entries.iter().max().copied().unwrap_or(0),
            "db_bytes": facts.db_bytes,
        }),
    );
    report.set(
        "fixture",
        serde_json::json!({
            "nodes": facts.nodes,
            "cached": facts.cached,
            "pins": facts.pins,
            "drafts": facts.drafts,
            "db_bytes": facts.db_bytes,
            "build_ms": facts.build_ms,
        }),
    );
    report.set("round", serde_json::json!(round));
    if let Some(warm) = &warm {
        report.set("warm_result", serde_json::json!({
            "ready": warm.ready,
            "ready_ms": warm.ready_ms,
            "calls": warm.calls,
            "cold_ms": warm.cold_ms,
            "warm_p50_ms": warm.warm_p50_ms,
            "warm_p95_ms": warm.warm_p95_ms,
        }));
    }

    println!(
        "#8 METRICS {}",
        serde_json::json!({
            "mode": mode.short(),
            "fixture": {
                "nodes": facts.nodes,
                "cached": facts.cached,
                "pins": facts.pins,
                "drafts": facts.drafts,
                "db_bytes": facts.db_bytes,
                "build_ms": facts.build_ms,
            },
            "states_root_p95_ms": states_root.p95_ms(),
            "states_root_p99_ms": states_root.p99_ms(),
            "states_dir2000_p95_ms": states_dir.p95_ms(),
            "states_batch45_p95_ms": states_batch.p95_ms(),
            "dir2000_p95_ms": dir2000.p95_ms(),
            "dir2000_p99_ms": dir2000.p99_ms(),
            "overview_total_p95_ms": overview.total.p95_ms(),
            "overview_total_max_ms": overview.total.max_ms(),
            "overview_query_p95_ms": overview.p95(|timings| timings.query_us),
            "overview_serialize_p95_ms": overview.p95(|timings| timings.serialize_us),
            "overview_lock_hold_p95_ms": overview.p95(|timings| timings.lock_hold_us),
            "overview_lock_wait_p95_ms": overview.p95(|timings| timings.lock_wait_us),
            "overview_read_only_calls": overview.read_only_calls(),
            "overview_cached_calls": overview.cached_calls(),
            "overview_calls": overview.total.us.len(),
            "overview_errors": overview.total.errors,
            "fuse_dir_entries": fuse.as_ref().map(|fuse| fuse.entries),
            "fuse_dir_cold_ms": fuse.as_ref().map(|fuse| fuse.cold_ms),
            "fuse_dir2000_p95_ms": fuse.as_ref().map(|fuse| fuse.warm.p95_ms()),
            "fuse_read1m_cold_ms": fuse.as_ref().map(|fuse| fuse.read_cold_ms),
            "fuse_read1m_p95_ms": fuse.as_ref().map(|fuse| fuse.read_warm.p95_ms()),
            "states_conc16_p95_ms": states_conc.as_ref().map(|samples| samples.p95_ms()),
            "states_conc16_ops": states_conc.as_ref().map(|samples| samples.us.len()),
            "mixed_timeouts": mixed_maps.as_ref().and_then(|(mixed, _)| mixed.get("mix_all")).map(|samples| samples.timeouts),
            "mixed_ops": mixed_maps.as_ref().and_then(|(mixed, _)| mixed.get("mix_all")).map(|samples| samples.us.len()),
            "mixed_p95_ms": mixed_maps.as_ref().and_then(|(mixed, _)| mixed.get("mix_all")).map(|samples| samples.p95_ms()),
            "mix_states_p95_ms": mixed_maps.as_ref().and_then(|(mixed, _)| mixed.get("mix_states")).map(|samples| samples.p95_ms()),
            "classic_timeouts": mixed_maps.as_ref().and_then(|(_, classic)| classic.get("classic_all")).map(|samples| samples.timeouts),
            "classic_states_p95_ms": mixed_maps.as_ref().and_then(|(_, classic)| classic.get("classic_states")).map(|samples| samples.p95_ms()),
            "peak_rss_kb": peak_rss_kb(),
            "loadavg_end": load_avg1(),
            "lock_ms": lock_ms,
            "lock_post_p95_ms": lock_post.p95_ms(),
            "service_started": service_started,
            "cold_states_ms": warm.as_ref().map(|warm| warm.cold_ms),
            "projection_ready": warm.as_ref().map(|warm| warm.ready),
            "projection_ready_ms": warm.as_ref().and_then(|warm| warm.ready_ms),
            "projection_warm_p95_ms": warm.as_ref().map(|warm| warm.warm_p95_ms),
            "projection_warm_calls": warm.as_ref().map(|warm| warm.calls),
            "projection_warm_threshold_ms": if mode.requires_warm() { Some(warm_threshold_ms) } else { None },
        })
    );

    if let Some(tasks) = tasks {
        // Background tasks must be stopped before the fixture/store is dropped;
        // joining a stuck service thread would hang the test, so the stop flag is
        // set and the handles are detached after a short grace.
        service.shutdown();
        thread::sleep(Duration::from_millis(250));
        drop(tasks);
        println!("#8 NOTE service_shutdown=1 (stop flag set; handles detached)");
    }
    print_mem_phase(mode, "threads_exit", serde_json::json!({}));
    thread::sleep(Duration::from_millis(250));
    print_mem_phase(mode, "idle", serde_json::json!({}));
    if env_flag("LAT_KEEP") {
        println!("#8 KEEP fixture_dir={}", dir.display());
        if let Some(temp) = temp {
            std::mem::forget(temp);
        }
    }
    report.finish(loadavg_start);
    println!("#8 DONE");
}


/// Issue #21 acceptance #3: the GetOverview cost curve across synthetic library
/// sizes. Each point builds `dirs` directories with `per_dir` files (the last
/// directory holds `last` files), installs the snapshot, then reports both the
/// generation-miss (cold) call and the steady-state (cached) calls.
///
/// Run explicitly:
/// `cargo test -p drive-runtime --release --all-features --locked \
///    --test status_latency_210k overview_scale_curve -- --ignored --nocapture`
///
/// `LAT_CURVE_DIRS` selects the directory counts (default `105,147,250`, i.e.
/// the 210k / 294k / 500k shapes used by the acceptance report).
#[test]
#[ignore]
fn overview_scale_curve() {
    let counts: Vec<usize> = env::var("LAT_CURVE_DIRS")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .map(|value| {
            value
                .split(',')
                .filter_map(|part| part.trim().parse().ok())
                .collect()
        })
        .unwrap_or_else(|| vec![105, 147, 250]);
    for dirs in counts {
        curve_point(dirs, FILES_PER_DIR, if dirs == DIRS { LAST_DIR_FILES } else { FILES_PER_DIR });
    }
}

fn curve_point(dirs: usize, per_dir: usize, last_dir_files: usize) {
    let started = Instant::now();
    let temp = tempfile::tempdir().expect("curve fixture dir");
    let mut nodes = Vec::with_capacity(dirs * per_dir + dirs + 1);
    nodes.push(node("root".into(), None, "fixture".into(), true, 0));
    let mut files = 0usize;
    for d in 0..dirs {
        let id = format!("d{d:03}");
        nodes.push(node(id.clone(), Some("root".into()), id.clone(), true, 0));
        let count = if d + 1 == dirs { last_dir_files } else { per_dir };
        for c in 0..count {
            nodes.push(node(
                format!("file-{files:06}"),
                Some(id.clone()),
                format!("f{c:05}"),
                false,
                0,
            ));
            files += 1;
        }
    }
    let node_count = nodes.len();
    let mut store = Store::open(temp.path()).expect("curve store");
    store
        .install_remote_snapshot(&nodes, "curve-cursor-1", "curve-server-1")
        .expect("curve snapshot");
    store.ensure_fs_root("root").expect("curve fs root");
    // Keep the availability predicate non-trivial: cache a slice of files.
    let cached = files.min(256).min(2_048);
    for index in 0..cached {
        let body: Vec<u8> = (0..4_096).map(|i| ((i + index) % 251) as u8).collect();
        store
            .publish_download(
                &DownloadTicket {
                    id: format!("file-{index:06}"),
                    version: "fixture-v1".into(),
                },
                &body,
                &sha256(&body),
            )
            .expect("curve cached publish");
    }
    let build_ms = started.elapsed().as_millis();
    let service = Service::new(
        profile(PathBuf::from("/tmp/drive-native-issue21-curve")),
        store,
        PathBuf::from("/nonexistent-worker"),
    )
    .expect("curve service");
    // Isolated counter timings on the same private read-only path.
    let counter_store = service.bridge.store.read_only().expect("curve read-only");
    let started = Instant::now();
    let availability = counter_store
        .cache_availability()
        .expect("curve availability");
    let availability_ms = started.elapsed().as_secs_f64() * 1_000.0;
    let started = Instant::now();
    let issue_count = counter_store
        .sync_issue_count("root")
        .expect("curve issue count");
    let issue_count_ms = started.elapsed().as_secs_f64() * 1_000.0;
    let (cold_body, cold_timings) = desktop::overview_measured(&service).expect("cold overview");
    let cold: serde_json::Value = serde_json::from_str(&cold_body).expect("cold overview json");
    let warm_iters = env_usize("LAT_CURVE_WARM_ITERS", 5);
    let mut warm_total = Samples::default();
    let mut warm_timings = Vec::with_capacity(warm_iters);
    for _ in 0..warm_iters {
        let (_, timings) = desktop::overview_measured(&service).expect("warm overview");
        warm_total.record(timings.total_us, false, 60_000);
        warm_timings.push(timings);
    }
    let mut warm_queries: Vec<u64> = warm_timings.iter().map(|timings| timings.query_us).collect();
    warm_queries.sort_unstable();
    println!(
        "#8 CURVE {}",
        serde_json::json!({
            "nodes": node_count,
            "dirs": dirs,
            "files": files,
            "cached": cached,
            "build_ms": build_ms,
            "availability_state": availability.state,
            "availability_total": availability.total,
            "availability_offline": availability.offline,
            "availability_pinned": availability.pinned,
            "availability_ms": availability_ms,
            "issue_count": issue_count,
            "issue_count_ms": issue_count_ms,
            "cold_read_only": cold_timings.read_only,
            "cold_total_ms": Samples::ms(cold_timings.total_us),
            "cold_query_ms": Samples::ms(cold_timings.query_us),
            "cold_serialize_ms": Samples::ms(cold_timings.serialize_us),
            "cold_lock_wait_ms": Samples::ms(cold_timings.lock_wait_us),
            "cold_lock_hold_ms": Samples::ms(cold_timings.lock_hold_us),
            "cold_cache_availability": cold["cache_availability"].clone(),
            "cold_issue_count": cold["issue_count"].clone(),
            "warm_calls": warm_timings.len(),
            "warm_cached_calls": warm_timings.iter().filter(|timings| timings.derived_cached).count(),
            "warm_total_p95_ms": warm_total.p95_ms(),
            "warm_query_p95_ms": Samples::ms(percentile(&warm_queries, 0.95)),
            "warm_lock_hold_max_ms": Samples::ms(
                warm_timings.iter().map(|timings| timings.lock_hold_us).max().unwrap_or(0)
            ),
        })
    );
}
// ===========================================================================
// Issue #94 workload hooks: the missing workload-size axes.
// ===========================================================================
//
// `status_latency_210k` only scales the node count. Issue #94 asks for the
// other workload axes to be runnable and to state their measurement scope:
// long confirmed commit history, draft/quarantine scale, large-directory ACL
// continuation, multi-mount slow replies, load-more paging and GC pressure.
//
// Each hook prints one `#94 HOOK` JSON line with `status=ran|skipped`, the
// scale it ran at, the measured distribution and the口径 (`scope`) it really
// covers. Hooks owned by another member are reported `skipped` with the
// delegated evidence path instead of turning into a pass.
//
// Run:
//   cargo test -p drive-runtime --release --all-features --locked \
//     --test status_latency_210k workload_hooks_94 -- --ignored --nocapture
// Env: LAT_HOOKS (default all), LAT_HOOK_SCALE (drafts/gc default 5000),
//      LAT_HOOK_HISTORY (default 10000), LAT_HOOK_DIR (default the 2,560-entry
//      synthetic directory).

const HOOK_DIR_ENTRIES: usize = 2_560;

fn hook_service(store: Store) -> Arc<Service> {
    Service::new(
        profile(PathBuf::from("/tmp/drive-native-hook-mount")),
        store,
        PathBuf::from("/nonexistent-worker"),
    )
    .expect("hook service")
}

fn build_hook_fixture(
    dirs: usize,
    per_dir: usize,
    cached: usize,
) -> Result<(tempfile::TempDir, Store), String> {
    let temp = tempfile::tempdir().map_err(|error| error.to_string())?;
    let mut nodes = Vec::with_capacity(dirs.saturating_mul(per_dir).saturating_add(dirs + 1));
    nodes.push(node("root".into(), None, "fixture".into(), true, 0));
    let mut files = 0usize;
    for d in 0..dirs {
        let id = format!("d{d:03}");
        nodes.push(node(id.clone(), Some("root".into()), id.clone(), true, 0));
        for c in 0..per_dir {
            nodes.push(node(
                format!("file-{files:06}"),
                Some(id.clone()),
                format!("f{c:05}"),
                false,
                0,
            ));
            files += 1;
        }
    }
    let mut store = Store::open(temp.path()).map_err(|error| error.to_string())?;
    store
        .install_remote_snapshot(&nodes, "hook-cursor-1", "hook-server-1")
        .map_err(|error| error.to_string())?;
    store
        .ensure_fs_root("root")
        .map_err(|error| error.to_string())?;
    for index in 0..cached.min(files) {
        let body: Vec<u8> = (0..4_096).map(|i| ((i + index) % 251) as u8).collect();
        store
            .publish_download(
                &DownloadTicket {
                    id: format!("file-{index:06}"),
                    version: "fixture-v1".into(),
                },
                &body,
                &sha256(&body),
            )
            .map_err(|error| error.to_string())?;
    }
    Ok((temp, store))
}

fn measure_hook(
    iters: usize,
    mut operation: impl FnMut() -> Result<(), String>,
) -> Result<Samples, String> {
    let mut samples = Samples::default();
    for _ in 0..iters {
        let started = Instant::now();
        let result = operation();
        let us = started.elapsed().as_micros() as u64;
        samples.record(us, result.is_err(), 60_000);
        if let Err(error) = result {
            return Err(error);
        }
    }
    Ok(samples)
}

fn print_hook(
    name: &str,
    status: &str,
    scale: &str,
    scope: &str,
    samples: Option<&Samples>,
    context: serde_json::Value,
) {
    let stats = samples.map(|samples| {
        serde_json::json!({
            "n": samples.us.len(),
            "errors": samples.errors,
            "timeouts": samples.timeouts,
            "p50_ms": samples.p50_ms(),
            "p95_ms": samples.p95_ms(),
            "p99_ms": samples.p99_ms(),
            "max_ms": samples.max_ms(),
        })
    });
    println!(
        "#94 HOOK {}",
        serde_json::json!({
            "hook": name,
            "status": status,
            "scale": scale,
            "scope": scope,
            "stats": stats,
            "context": context,
            "loadavg": load_avg1(),
        })
    );
}

/// Long confirmed commit history: seed N confirmed upload operations and
/// measure the GetOverview / GetIssues cost that reads the diagnostics views.
/// Scope: store-level synthetic history on the fixture DB (not a real NAS
/// commit log, no background sync).
fn hook_confirmed_history(scale: usize) -> Result<(), String> {
    let scale = scale.max(2);
    for rows in [0usize, scale] {
        let (temp, mut store) = build_hook_fixture(1, scale.max(16), 0)?;
        for index in 0..rows {
            let id = format!("file-{index:06}");
            let operation = store
                .save_local(&id, b"hook-history")
                .map_err(|error| error.to_string())?;
            store
                .mark_upload_started(&operation)
                .map_err(|error| error.to_string())?;
            store
                .confirm_upload(&operation, "hook-v1")
                .map_err(|error| error.to_string())?;
        }
        let service = hook_service(store);
        let overview = measure_hook(10, || {
            desktop::overview(&service)
                .map(|_| ())
                .map_err(|error| error.to_string())
        })?;
        let issues = measure_hook(10, || {
            desktop::issues(&service, "")
                .map(|_| ())
                .map_err(|error| error.to_string())
        })?;
        print_hook(
            "confirmed_history",
            "ran",
            &format!("rows={rows}"),
            "store-level synthetic confirmed upload history; GetOverview/GetIssues cost (not a real NAS commit log)",
            Some(&overview),
            serde_json::json!({
                "rows": rows,
                "issues_p95_ms": issues.p95_ms(),
                "note": "projection/derived cache state is whatever the tree under test provides; compare baseline vs fixed with the same hook",
            }),
        );
        drop(service);
        drop(temp);
    }
    Ok(())
}

/// Draft scale: N clean drafts (dirty=0) measured through storage_summary and
/// GetOverview. Scope: store-level synthetic drafts; quarantine creation needs
/// orphan tombstones and has no public fixture API (delegated to #89 owner
/// tests).
fn hook_drafts(scale: usize) -> Result<(), String> {
    for drafts_count in [0usize, scale] {
        let (temp, mut store) = build_hook_fixture(1, 64, 0)?;
        for index in 0..drafts_count {
            let id = format!("file-{:06}", index % 64);
            let draft = store
                .begin_draft(&id, "fixture-v1", None)
                .map_err(|error| error.to_string())?;
            store
                .draft_write(&draft, 0, b"hook-draft", false)
                .map_err(|error| error.to_string())?;
        }
        let service = hook_service(store);
        let summary = measure_hook(10, || {
            let guard = service.bridge.store.lock().unwrap();
            guard
                .storage_summary()
                .map(|_| ())
                .map_err(|error| error.to_string())
        })?;
        let overview = measure_hook(5, || {
            desktop::overview(&service)
                .map(|_| ())
                .map_err(|error| error.to_string())
        })?;
        print_hook(
            "draft_scale",
            "ran",
            &format!("drafts={drafts_count}"),
            "store-level synthetic clean drafts; storage_summary/GetOverview cost (quarantine records not covered: no public orphan-tombstone fixture API)",
            Some(&summary),
            serde_json::json!({
                "drafts": drafts_count,
                "overview_p95_ms": overview.p95_ms(),
                "storage_summary_p95_ms": summary.p95_ms(),
            }),
        );
        drop(service);
        drop(temp);
    }
    Ok(())
}

/// Load-more paging: request the backend GetDirectory offsets 0,256,... and
/// verify the page count is O(K) (the historical GUI bug issued
/// K(K+1)/2 requests; the GUI pattern itself is covered by desktop-gui #93).
fn hook_load_more() -> Result<(), String> {
    let (_temp, store) = build_hook_fixture(1, HOOK_DIR_ENTRIES, 0)?;
    let service = hook_service(store);
    let mut per_page: Vec<u64> = Vec::new();
    let mut offset = 0u32;
    let mut seen = 0usize;
    let mut pages = 0usize;
    let mut reported_total = None;
    loop {
        let started = Instant::now();
        let body = desktop::directory(&service, "d000", offset).map_err(|error| error.to_string())?;
        let us = started.elapsed().as_micros() as u64;
        let value: serde_json::Value =
            serde_json::from_str(&body).map_err(|error| error.to_string())?;
        let entries = value["entries"].as_array().map(Vec::len).unwrap_or(0);
        if reported_total.is_none() {
            reported_total = value["total"].as_u64();
        }
        if entries == 0 {
            break;
        }
        seen += entries;
        per_page.push(us);
        pages += 1;
        offset = u32::try_from(seen).map_err(|error| error.to_string())?;
        if pages > 32 {
            return Err(format!(
                "load-more paging did not finish within 32 pages (seen={seen})"
            ));
        }
    }
    if seen != HOOK_DIR_ENTRIES {
        return Err(format!(
            "load-more offset paging returned {seen} entries, expected {HOOK_DIR_ENTRIES}"
        ));
    }
    if pages > 16 {
        return Err(format!(
            "load-more pagination is not O(K): {pages} requests for {seen} entries"
        ));
    }
    let mut samples = Samples::default();
    for us in &per_page {
        samples.record(*us, false, 60_000);
    }
    print_hook(
        "load_more",
        "ran",
        &format!("entries={HOOK_DIR_ENTRIES}"),
        "backend GetDirectory offset pages; the desktop O(K^2) request pattern is delegated to the desktop-gui #93 fake-bus test",
        Some(&samples),
        serde_json::json!({
            "pages": pages,
            "entries_seen": seen,
            "server_total": reported_total,
            "request_offsets": "0,256,... (page size controlled by the backend)",
        }),
    );
    Ok(())
}

/// GC pressure: create N clean drafts (dirty=0) and time one store-level
/// collect_garbage call. Scope: store-level single-threaded GC cost; the
/// runtime `maintain_cache` batching/lock ownership is the gc-storage owner's
/// #90 test.
fn hook_gc_pressure(scale: usize) -> Result<(), String> {
    for drafts_count in [0usize, scale] {
        let (temp, mut store) = build_hook_fixture(1, 64, 0)?;
        for index in 0..drafts_count {
            let id = format!("file-{:06}", index % 64);
            let draft = store
                .begin_draft(&id, "fixture-v1", None)
                .map_err(|error| error.to_string())?;
            store
                .draft_write(&draft, 0, b"gc", false)
                .map_err(|error| error.to_string())?;
        }
        let started = Instant::now();
        let freed = store
            .collect_garbage()
            .map_err(|error| error.to_string())?;
        let gc_ms = started.elapsed().as_secs_f64() * 1_000.0;
        print_hook(
            "gc_pressure",
            "ran",
            &format!("clean_drafts={drafts_count}"),
            "store-level collect_garbage call on synthetic clean drafts; runtime maintain_cache batching/lock contention is the gc-storage #90 test",
            None,
            serde_json::json!({
                "clean_drafts": drafts_count,
                "freed": freed,
                "gc_ms": gc_ms,
                "note": "single-threaded store-level cost; no runtime lock contention",
            }),
        );
        drop(temp);
    }
    Ok(())
}

/// Large-directory continuation through the mounted kernel FUSE path: readdir
/// returns the 2,560 entries across several getdents replies; one pass also
/// stats every entry (attribute/ACL work). Scope: kernel FUSE mounted
/// continuation; the internal drive-fs READDIRPLUS/ACL projection is delegated
/// to the filesystem owner's test.
fn hook_large_dir_continuation() -> Result<(), String> {
    if env::var("LAT_FUSE").map(|value| value == "0").unwrap_or(false) {
        print_hook(
            "large_dir_continuation",
            "skipped",
            &format!("entries={HOOK_DIR_ENTRIES}"),
            "kernel FUSE readdir continuation",
            None,
            serde_json::json!({"reason": "LAT_FUSE=0 explicitly skips the mount"}),
        );
        return Ok(());
    }
    let (_temp, store) = build_hook_fixture(1, HOOK_DIR_ENTRIES, 0)?;
    let service = hook_service(store);
    let Some((mount, session)) = fuse_mount(&service) else {
        print_hook(
            "large_dir_continuation",
            "skipped",
            &format!("entries={HOOK_DIR_ENTRIES}"),
            "kernel FUSE readdir continuation",
            None,
            serde_json::json!({"reason": "kernel FUSE mount unavailable (no /dev/fuse?)"}),
        );
        return Ok(());
    };
    let directory = mount.path().join("d000");
    let mut samples = Samples::default();
    let mut counts = Vec::new();
    for _ in 0..5 {
        let started = Instant::now();
        let count = std::fs::read_dir(&directory)
            .map_err(|error| error.to_string())?
            .collect::<std::io::Result<Vec<_>>>()
            .map_err(|error| error.to_string())?
            .len();
        let us = started.elapsed().as_micros() as u64;
        samples.record(us, count != HOOK_DIR_ENTRIES, 60_000);
        counts.push(count);
    }
    if counts.iter().any(|count| *count != HOOK_DIR_ENTRIES) {
        return Err(format!(
            "kernel FUSE readdir continuation returned {counts:?}, expected {HOOK_DIR_ENTRIES} per pass"
        ));
    }
    let started = Instant::now();
    let mut statted = 0usize;
    for entry in std::fs::read_dir(&directory).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        if entry.metadata().is_ok() {
            statted += 1;
        }
    }
    let stat_ms = started.elapsed().as_secs_f64() * 1_000.0;
    drop(session);
    drop(mount);
    drop(service);
    drop(_temp);
    print_hook(
        "large_dir_continuation",
        "ran",
        &format!("entries={HOOK_DIR_ENTRIES}"),
        "kernel FUSE readdir continuation (multiple getdents replies) + one stat pass; drive-fs READDIRPLUS/ACL projection delegated to the filesystem owner's test",
        Some(&samples),
        serde_json::json!({
            "entries": HOOK_DIR_ENTRIES,
            "statted_entries": statted,
            "stat_pass_ms": stat_ms,
        }),
    );
    Ok(())
}

#[test]
#[ignore]
fn workload_hooks_94() {
    let hooks: Vec<String> = env::var("LAT_HOOKS")
        .unwrap_or_else(|_| {
            "confirmed_history,drafts,load_more,gc,fuse-readdir,multi-mount,gui-load-more".to_string()
        })
        .split(',')
        .map(str::trim)
        .filter(|hook| !hook.is_empty())
        .map(str::to_string)
        .collect();
    let scale = env_usize("LAT_HOOK_SCALE", 5_000);
    let history_scale = env_usize("LAT_HOOK_HISTORY", 10_000);
    println!(
        "#94 HOOKS_START {}",
        serde_json::json!({
            "hooks": hooks,
            "draft_gc_scale": scale,
            "history_scale": history_scale,
            "head": env::var("LAT_HEAD").unwrap_or_else(|_| "unset".to_string()),
            "loadavg": load_avg1(),
        })
    );
    let mut ran = 0usize;
    let mut skipped = 0usize;
    let mut failed = 0usize;
    for hook in &hooks {
        let result = match hook.as_str() {
            "confirmed_history" | "history" => hook_confirmed_history(history_scale),
            "drafts" => hook_drafts(scale),
            "quarantine" => {
                print_hook(
                    "quarantine_scale",
                    "skipped",
                    &format!("records={}", env_usize("LAT_QUARANTINE", 0)),
                    "quarantine record scale",
                    None,
                    serde_json::json!({
                        "reason": "no public API can synthesize orphan tombstones; #89 owner tests cover quarantine record scale and storage_summary stale/error semantics",
                    }),
                );
                skipped += 1;
                continue;
            }
            "load_more" | "loadmore" => hook_load_more(),
            "gc" => hook_gc_pressure(scale),
            "fuse-readdir" | "acl" => hook_large_dir_continuation(),
            "multi-mount" => {
                print_hook(
                    "multi_mount_slow_replies",
                    "skipped",
                    "mounts=2 (delegated)",
                    "multi-mount slow-reply refresh self-excitation",
                    None,
                    serde_json::json!({
                        "reason": "GUI fake-bus workload; desktop-gui #92 test is the authority (validation re-runs it and records request counts)",
                    }),
                );
                skipped += 1;
                continue;
            }
            "gui-load-more" => {
                print_hook(
                    "gui_load_more",
                    "skipped",
                    "entries=2560 (delegated)",
                    "GUI load-more request pattern",
                    None,
                    serde_json::json!({
                        "reason": "GUI fake-bus workload; desktop-gui #93 test is the authority (validation re-runs it and records the offset sequence)",
                    }),
                );
                skipped += 1;
                continue;
            }
            other => {
                print_hook(
                    other,
                    "skipped",
                    "n/a",
                    "unknown hook",
                    None,
                    serde_json::json!({"reason": "unknown LAT_HOOKS entry"}),
                );
                skipped += 1;
                continue;
            }
        };
        match result {
            Ok(()) => ran += 1,
            Err(error) => {
                failed += 1;
                print_hook(
                    hook,
                    "failed",
                    "n/a",
                    "hook invariant failed",
                    None,
                    serde_json::json!({"error": error}),
                );
            }
        }
    }
    println!(
        "#94 HOOKS_DONE {}",
        serde_json::json!({
            "ran": ran,
            "skipped": skipped,
            "failed": failed,
            "loadavg": load_avg1(),
        })
    );
    assert_eq!(failed, 0, "issue #94 workload hooks failed; see the #94 HOOK lines");
}
