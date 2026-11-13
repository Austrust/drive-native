//! Issue #30 acceptance harness: cached-directory `GetStates` batches.
//!
//! Scope and limits: this runs against a synthetic SQLite state plus local blob
//! files. There is no NAS, no D-Bus client and no FUSE mount, so the numbers are
//! local query/latency evidence for the desktop status read path only.
//!
//! The fixture repeats the issue-#8 210,000-node shape (root + 105 directories +
//! 209,894 files, one 2,000-item directory `d000`) and then caches every file of
//! `d000` (2,000 cached children) plus 600 files of `d001`. That is the
//! "all-downloaded folder" shape the issue reports: every child carries local
//! content state, so the status read path must handle 2,000 / 600 detailed nodes
//! per request instead of a single cached subtree.
//!
//! Scenarios and issue #30 acceptance thresholds:
//! 1. `states_cached600_dir`   `GetStates(["d001"])` with 600 cached children:
//!                             p95 < 100 ms (issue #30 acceptance).
//! 2. `states_cached2000_dir`  `GetStates(["d000"])` with 2,000 cached children:
//!                             p95 <= 300 ms (the #8 `dir2000` view budget).
//! 3. `directory_cached2000`   `GetDirectory("d000")` first page (256 entries):
//!                             p95 <= 300 ms (regression guard; the point of the
//!                             scenario is that the page no longer re-resolves
//!                             each child, asserted by the unit test in
//!                             `desktop.rs`, not by this latency alone).
//!
//! Run (release, remote workstation):
//!   cargo test -p drive-runtime --release --all-features \
//!     --test status_cached_batch -- --ignored --nocapture
//!
//! Env: FIX30_FIXTURE_DIR (persistent fixture directory; the 210k snapshot
//!      import is the slow part, so repeated before/after runs should reuse it),
//!      FIX30_ITER_600 (default 30), FIX30_ITER_2000 (10), FIX30_ITER_DIR (10),
//!      FIX30_TIMEOUT_MS (15000).

use drive_core::{sha256, DownloadTicket, Node, NodeMetadata, Store};
use drive_runtime::{
    desktop,
    service::{Profile, Service},
};
use std::{
    env,
    hint::black_box,
    path::PathBuf,
    time::Instant,
};

const NODES: usize = 210_000;
const DIRS: usize = 105;
const FILES_PER_DIR: usize = 2_000;
const LAST_DIR_FILES: usize = 1_894;
const CACHED_DIR: &str = "d000";
const CACHED_DIR_FILES: usize = FILES_PER_DIR;
const CACHED_600_DIR: &str = "d001";
const CACHED_600_FILES: usize = 600;
const CACHED_BODY_BYTES: usize = 4_096;

const SLO_CACHED600_P95_MS: f64 = 100.0;
const SLO_CACHED2000_P95_MS: f64 = 300.0;
const SLO_DIRECTORY_P95_MS: f64 = 300.0;
/// Marginal per-cached-child cost budget that flags per-node point queries.
/// Per-node detail work measures ~50 us/child on this fixture; a single batch
/// query plus a blob stat stays well below 10 us/child.
const SLO_CACHED_CHILD_US: f64 = 20.0;

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
}

impl Samples {
    fn record(&mut self, us: u64, failed: bool) {
        self.us.push(us);
        if failed {
            self.errors += 1;
        }
    }

    fn p50_ms(&self) -> f64 {
        let mut sorted = self.us.clone();
        sorted.sort_unstable();
        percentile(&sorted, 0.50) as f64 / 1_000.0
    }

    fn p95_ms(&self) -> f64 {
        let mut sorted = self.us.clone();
        sorted.sort_unstable();
        percentile(&sorted, 0.95) as f64 / 1_000.0
    }

    fn report(&self, scenario: &str) {
        let mut sorted = self.us.clone();
        sorted.sort_unstable();
        println!(
            "#30 STAT {}",
            serde_json::json!({
                "scenario": scenario,
                "n": sorted.len(),
                "errors": self.errors,
                "p50_ms": percentile(&sorted, 0.50) as f64 / 1_000.0,
                "p95_ms": percentile(&sorted, 0.95) as f64 / 1_000.0,
                "p99_ms": percentile(&sorted, 0.99) as f64 / 1_000.0,
                "max_ms": *sorted.last().unwrap_or(&0) as f64 / 1_000.0,
            })
        );
    }
}

struct Fixture {
    /// Keeps the temporary fixture directory alive; `None` when the caller
    /// supplied FIX30_FIXTURE_DIR and the fixture is intentionally persistent.
    _temp: Option<tempfile::TempDir>,
    store: Store,
    db_bytes: u64,
    build_ms: u128,
}

fn node(id: String, parent: Option<String>, name: String, directory: bool, size: u64) -> Node {
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
    let reuse = env::var("FIX30_FIXTURE_DIR")
        .ok()
        .filter(|path| !path.trim().is_empty());
    let (dir, temp) = match reuse {
        Some(path) => (PathBuf::from(path), None),
        None => {
            let temp = tempfile::tempdir()?;
            (temp.path().to_path_buf(), Some(temp))
        }
    };
    std::fs::create_dir_all(&dir)?;
    let facts_path = dir.join("fix30-facts.json");
    let existed = dir.join("state.sqlite").is_file() && facts_path.is_file();

    let store = if existed {
        let persisted: serde_json::Value = serde_json::from_slice(&std::fs::read(&facts_path)?)?;
        assert_eq!(persisted["nodes"].as_u64(), Some(NODES as u64));
        assert_eq!(persisted["cached_dir"].as_u64(), Some(CACHED_DIR_FILES as u64));
        assert_eq!(persisted["cached_600_dir"].as_u64(), Some(CACHED_600_FILES as u64));
        Store::open(&dir)?
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
                nodes.push(node(
                    format!("file-{file_index:06}"),
                    Some(id.clone()),
                    format!("f{c:05}"),
                    false,
                    0,
                ));
                file_index += 1;
            }
        }
        assert_eq!(nodes.len(), NODES, "fixture node count");
        assert_eq!(file_index, NODES - DIRS - 1, "fixture file count");

        let mut store = Store::open(&dir)?;
        store.install_remote_snapshot(&nodes, "fix30-cursor-1", "fix30-server-1")?;
        store.ensure_fs_root("root")?;

        // `d000` is fully downloaded: every child carries local content state.
        let mut body = vec![0u8; CACHED_BODY_BYTES];
        for (offset, index) in (0..CACHED_DIR_FILES).enumerate() {
            body[0] = (offset % 251) as u8;
            store.publish_download(
                &DownloadTicket {
                    id: format!("file-{index:06}"),
                    version: "fixture-v1".into(),
                },
                &body,
                &sha256(&body),
            )?;
        }
        // `d001` has 600 cached children (the issue's 600-cached batch shape).
        for (offset, index) in (0..CACHED_600_FILES).enumerate() {
            let file_index = FILES_PER_DIR + index;
            body[0] = ((offset + 17) % 251) as u8;
            store.publish_download(
                &DownloadTicket {
                    id: format!("file-{file_index:06}"),
                    version: "fixture-v1".into(),
                },
                &body,
                &sha256(&body),
            )?;
        }

        let facts = serde_json::json!({
            "nodes": NODES,
            "files": file_index,
            "cached_dir": CACHED_DIR_FILES,
            "cached_600_dir": CACHED_600_FILES,
        });
        std::fs::write(&facts_path, serde_json::to_vec_pretty(&facts)?)?;
        store
    };

    let db_bytes = std::fs::metadata(dir.join("state.sqlite"))
        .map(|m| m.len())
        .unwrap_or(0);
    println!(
        "#30 FIXTURE {}",
        serde_json::json!({
            "nodes": NODES,
            "dirs": DIRS,
            "files": NODES - DIRS - 1,
            "cached_dir": CACHED_DIR_FILES,
            "cached_600_dir": CACHED_600_FILES,
            "db_bytes": db_bytes,
            "build_ms": started.elapsed().as_millis(),
            "reused": existed,
        })
    );
    Ok(Fixture {
        _temp: temp,
        store,
        db_bytes,
        build_ms: started.elapsed().as_millis(),
    })
}

fn profile(mount: PathBuf) -> Profile {
    Profile {
        schema: 1,
        task_id: uuid::Uuid::new_v4().to_string(),
        host: "issue30.example.invalid".into(),
        tls_name: "issue30.example.invalid".into(),
        trusted_ca_pem: Vec::new(),
        username: "issue30".into(),
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

/// GetStates of one directory: the response has one entry (the directory
/// itself), but the aggregation walks every cached child.
fn bench_states(
    service: &Service,
    scenario: &str,
    path: &str,
    iters: usize,
) -> Samples {
    let paths = vec![path.to_string()];
    for _ in 0..2 {
        let _ = black_box(desktop::states(service, paths.clone()));
    }
    let mut samples = Samples::default();
    for i in 0..iters {
        let started = Instant::now();
        let result = desktop::states(service, paths.clone());
        let us = started.elapsed().as_micros() as u64;
        let error = match result {
            Ok(body) => check_states_body(&body, 1).err(),
            Err(error) => Some(error.to_string()),
        };
        samples.record(us, error.is_some());
        println!(
            "#30 OP scenario={scenario} i={i} us={us} err={}",
            error.as_deref().unwrap_or("-")
        );
    }
    samples.report(scenario);
    samples
}

fn bench_directory(service: &Service, scenario: &str, path: &str, iters: usize) -> Samples {
    for _ in 0..2 {
        let _ = black_box(desktop::directory(service, path, 0));
    }
    let mut samples = Samples::default();
    for i in 0..iters {
        let started = Instant::now();
        let result = desktop::directory(service, path, 0);
        let us = started.elapsed().as_micros() as u64;
        let error = match result {
            Ok(body) => check_directory_body(&body, 256, FILES_PER_DIR).err(),
            Err(error) => Some(error.to_string()),
        };
        samples.record(us, error.is_some());
        println!(
            "#30 OP scenario={scenario} i={i} us={us} err={}",
            error.as_deref().unwrap_or("-")
        );
    }
    samples.report(scenario);
    samples
}

fn check_directory_body(body: &str, expected_entries: usize, expected_total: usize) -> Result<(), String> {
    let value: serde_json::Value =
        serde_json::from_str(body).map_err(|error| error.to_string())?;
    let entries = value["entries"]
        .as_array()
        .ok_or_else(|| "directory response without entries".to_string())?;
    if entries.len() != expected_entries {
        return Err(format!(
            "directory entries {} != {}",
            entries.len(),
            expected_entries
        ));
    }
    if value["total"].as_u64() != Some(expected_total as u64) {
        return Err(format!("directory total {} != {expected_total}", value["total"]));
    }
    Ok(())
}

#[test]
#[ignore]
fn status_cached_batch() {
    let iters_600 = env_usize("FIX30_ITER_600", 30);
    let iters_2000 = env_usize("FIX30_ITER_2000", 10);
    let iters_dir = env_usize("FIX30_ITER_DIR", 10);
    let timeout_ms = env_u64("FIX30_TIMEOUT_MS", 15_000);
    let loadavg = std::fs::read_to_string("/proc/loadavg")
        .unwrap_or_default()
        .trim()
        .to_string();
    println!(
        "#30 ENV {}",
        serde_json::json!({
            "iter_600": iters_600,
            "iter_2000": iters_2000,
            "iter_dir": iters_dir,
            "timeout_ms": timeout_ms,
            "loadavg": loadavg,
        })
    );

    let fixture = build_fixture().expect("build fixture");
    println!(
        "#30 NOTE db_bytes={} build_ms={}",
        fixture.db_bytes, fixture.build_ms
    );
    let service = Service::new(
        profile(PathBuf::from("/tmp/drive-native-issue30-mount")),
        fixture.store,
        PathBuf::from("/nonexistent-worker"),
    )
    .expect("service");

    let cached_600 = bench_states(&service, "states_cached600_dir", CACHED_600_DIR, iters_600);
    let cached_2000 = bench_states(&service, "states_cached2000_dir", CACHED_DIR, iters_2000);
    let directory = bench_directory(&service, "directory_cached2000", CACHED_DIR, iters_dir);

    let p95_600 = cached_600.p95_ms();
    let p95_2000 = cached_2000.p95_ms();
    let p95_dir = directory.p95_ms();
    let passed_600 = p95_600 < SLO_CACHED600_P95_MS && cached_600.errors == 0;
    let passed_2000 = p95_2000 <= SLO_CACHED2000_P95_MS && cached_2000.errors == 0;
    let passed_dir = p95_dir <= SLO_DIRECTORY_P95_MS && directory.errors == 0;
    // Both directories have 2,000 children; only the cached-child count differs
    // (2,000 vs 600), so the marginal cost per cached child isolates the per-node
    // detail work from the fixed seed query / render cost. A point-query-per-node
    // implementation lands around 50 us/child here; a batched one below 10 us.
    let slope_us_per_cached_child = (p95_2000 - p95_600).max(0.0) * 1_000.0
        / (CACHED_DIR_FILES - CACHED_600_FILES) as f64;
    let passed_scaling = slope_us_per_cached_child < SLO_CACHED_CHILD_US;
    println!(
        "#30 SLO {}",
        serde_json::json!({
            "states_cached600_dir_p95_ms": p95_600,
            "states_cached600_dir_slo_ms": SLO_CACHED600_P95_MS,
            "states_cached600_dir_errors": cached_600.errors,
            "states_cached600_dir_pass": passed_600,
            "states_cached2000_dir_p95_ms": p95_2000,
            "states_cached2000_dir_slo_ms": SLO_CACHED2000_P95_MS,
            "states_cached2000_dir_errors": cached_2000.errors,
            "states_cached2000_dir_pass": passed_2000,
            "directory_cached2000_p95_ms": p95_dir,
            "directory_cached2000_slo_ms": SLO_DIRECTORY_P95_MS,
            "directory_cached2000_errors": directory.errors,
            "directory_cached2000_pass": passed_dir,
            "cached_child_marginal_us": slope_us_per_cached_child,
            "cached_child_marginal_slo_us": SLO_CACHED_CHILD_US,
            "cached_child_marginal_pass": passed_scaling,
            "timeout_ms": timeout_ms,
            "cached_600_p50_ms": cached_600.p50_ms(),
            "cached_2000_p50_ms": cached_2000.p50_ms(),
            "directory_p50_ms": directory.p50_ms(),
        })
    );

    assert!(
        passed_600,
        "GetStates of a 600-cached-child directory p95 {p95_600:.1} ms (SLO {SLO_CACHED600_P95_MS} ms, errors {})",
        cached_600.errors
    );
    assert!(
        passed_2000,
        "GetStates of a fully cached 2000-file directory p95 {p95_2000:.1} ms (SLO {SLO_CACHED2000_P95_MS} ms, errors {})",
        cached_2000.errors
    );
    assert!(
        passed_dir,
        "GetDirectory of a fully cached 2000-file directory p95 {p95_dir:.1} ms (SLO {SLO_DIRECTORY_P95_MS} ms, errors {})",
        directory.errors
    );
    assert!(
        passed_scaling,
        "marginal cost per cached child {slope_us_per_cached_child:.1} us (SLO {SLO_CACHED_CHILD_US} us); \
         per-node detail queries scale with the child count"
    );
}
