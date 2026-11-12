//! T2 experiment harness: polling amplification of the desktop RPC paths.
//!
//! Answers, on a large synthetic store (default 147 top dirs x 2000 files =
//! 294,148 nodes, the issue #15/#16 scale):
//!
//! * H1: is a namespace-root `GetStates(".")` O(nodes) even after the committed
//!   status projection is warm (CPU + rchar per call at 294k)?
//! * H2: how do CPU and rchar scale with the poll frequency (1/4/10 Hz) across
//!   the desktop poll set (GetStates root / GetOverview / GetTransfers /
//!   GetIssues page 1 / GetCacheSettings), and at which frequency does the
//!   schedule start to slip (cliff)?
//! * H3: which poll path still does large per-call IO (rchar spike) or is
//!   otherwise not O(1) per call at 294k?
//!
//! It drives the same `desktop::*` entry points the D-Bus bridge calls
//! (`GetStates` -> `desktop::states`, `GetOverview` -> `desktop::overview_measured`,
//! `GetIssues` -> `desktop::issues`, `GetCacheSettings` -> `desktop::cache_settings`,
//! `GetTransfers` -> `desktop::transfers`), in-process, after
//! `Service::start()` has published the status projection.
//!
//! Output is `T2POLL <KIND> <json>` on stdout (`--nocapture`), one line per
//! fixture/warm/burst/phase plus a SUMMARY. CPU comes from `/proc/thread-self/stat`
//! (polling thread only) and `/proc/self/stat` (process); IO from
//! `/proc/self/io`; loadavg from `/proc/loadavg`. Linux only.
//!
//! Run (release, remote workstation):
//!   cargo test -p drive-runtime --release --all-features --locked \
//!     --test perf_t2_poll -- --ignored --nocapture
//!
//! Env:
//!   POLL_FIXTURE_DIR  persistent fixture dir (reused when state.sqlite exists)
//!   POLL_DIRS (147), POLL_FILES_PER_DIR (2000), POLL_CACHED (400),
//!   POLL_ISSUES (300 conflicted files), POLL_DRAFTS (5)
//!   POLL_RATES ("1,4,10"), POLL_SECONDS (10 per phase), POLL_REPS (3)
//!   POLL_BURST_ITERS (8 per interface)
//!   POLL_WARM_TIMEOUT_MS (90000), POLL_WARM_RCHAR_MAX (20 MiB per call)
//!   POLL_KEEP=1 (keep a temp fixture; a POLL_FIXTURE_DIR is always kept)
//!
//! Fixture build uses the same shape as `status_latency_210k` / `scale_perf`:
//! snapshot import, ~400 published cached files (4 KiB), a pinned subtree, a
//! few dirty drafts, then `sqlite3` injects N conflicted files so `GetIssues`
//! has a full first page (256 candidates) like the issue #15/#16 report.

#![recursion_limit = "256"]

use drive_core::{sha256, DownloadTicket, Node, NodeMetadata, Store};
use drive_runtime::{
    desktop,
    service::{Profile, Service},
};
use std::{
    env, fs,
    io::{self, Read},
    path::{Path, PathBuf},
    sync::Arc,
    thread,
    time::{Duration, Instant},
};

// ---------------------------------------------------------------------------
// env helpers
// ---------------------------------------------------------------------------

fn env_usize(key: &str, default: usize) -> usize {
    env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

fn env_u64(key: &str, default: u64) -> u64 {
    env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

fn env_flag(key: &str) -> bool {
    matches!(env::var(key).as_deref(), Ok("1") | Ok("true") | Ok("yes"))
}

fn percentile(sorted: &[u64], fraction: f64) -> u64 {
    if sorted.is_empty() {
        return 0;
    }
    let index = ((sorted.len() as f64 * fraction).ceil() as usize).saturating_sub(1);
    sorted[index.min(sorted.len() - 1)]
}

fn ms(us: u64) -> f64 {
    us as f64 / 1_000.0
}

#[derive(Default)]
struct Samples {
    us: Vec<u64>,
    /// Per-call `/proc/self/io` rchar delta (process-wide; the poller is the
    /// only active caller, so this attributes read amplification to the call).
    rchar: Vec<u64>,
    errors: u64,
}

impl Samples {
    fn record(&mut self, us: u64, rchar: u64, failed: bool) {
        self.us.push(us);
        self.rchar.push(rchar);
        if failed {
            self.errors += 1;
        }
    }

    fn ms(&self, fraction: f64) -> f64 {
        let mut sorted = self.us.clone();
        sorted.sort_unstable();
        ms(percentile(&sorted, fraction))
    }

    fn rchar_ms(&self, fraction: f64) -> u64 {
        let mut sorted = self.rchar.clone();
        sorted.sort_unstable();
        percentile(&sorted, fraction)
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "n": self.us.len(),
            "errors": self.errors,
            "p50_ms": self.ms(0.50),
            "p95_ms": self.ms(0.95),
            "p99_ms": self.ms(0.99),
            "max_ms": self.ms(1.0),
            "rchar_p50_bytes": self.rchar_ms(0.50),
            "rchar_p95_bytes": self.rchar_ms(0.95),
            "rchar_total_bytes": self.rchar.iter().sum::<u64>(),
        })
    }
}

/// Run one RPC, returning `(result, elapsed_us, rchar_delta_bytes)`.
fn probe<T>(f: impl FnOnce() -> T) -> (T, u64, u64) {
    let before = rchar_bytes().unwrap_or(0);
    let began = Instant::now();
    let out = f();
    let us = began.elapsed().as_micros() as u64;
    let read = rchar_bytes().unwrap_or(0).saturating_sub(before);
    (out, us, read)
}

#[derive(Default)]
struct OpSet {
    states_root: Samples,
    /// GetStates for all top-level directory rows of the root page (147 paths at
    /// the issue #15/#16 shape). The GUI sends exactly these when a status change
    /// touches the visible root-folder rows.
    states_page_root: Samples,
    /// GetStates for a small visible-row patch (2 dirs + 22 files).
    states_patch: Samples,
    overview: Samples,
    transfers: Samples,
    issues: Samples,
    cache_settings: Samples,
    overview_read_only: u64,
    overview_derived_cached: u64,
}

// ---------------------------------------------------------------------------
// /proc probes
// ---------------------------------------------------------------------------

fn read_text(path: &str) -> Option<String> {
    let mut text = String::new();
    fs::File::open(path).ok()?.read_to_string(&mut text).ok()?;
    Some(text)
}

/// utime+stime in microseconds from one `/proc/*/stat` line.
fn stat_cpu_us(stat: &str) -> Option<u64> {
    let close = stat.rfind(')')?;
    let fields: Vec<&str> = stat[close + 1..].split_whitespace().collect();
    // fields[0] is state (field 3); utime is field 14 -> index 11.
    let ticks: u64 = fields.get(11)?.parse::<u64>().ok()? + fields.get(12)?.parse::<u64>().ok()?;
    let hz = unsafe { libc::sysconf(libc::_SC_CLK_TCK) };
    if hz <= 0 {
        return None;
    }
    Some(ticks * 1_000_000 / hz as u64)
}

/// CPU time consumed by the calling thread only (polling thread).
fn thread_cpu_us() -> Option<u64> {
    stat_cpu_us(&read_text("/proc/thread-self/stat")?)
}

/// CPU time consumed by the whole test process.
fn process_cpu_us() -> Option<u64> {
    stat_cpu_us(&read_text("/proc/self/stat")?)
}

fn rchar_bytes() -> Option<u64> {
    read_text("/proc/self/io")?
        .lines()
        .find_map(|line| line.strip_prefix("rchar:")?.trim().parse::<u64>().ok())
}

fn loadavg1() -> Option<f64> {
    read_text("/proc/loadavg")?
        .split_whitespace()
        .next()?
        .parse::<f64>()
        .ok()
}

fn peak_rss_kb() -> Option<u64> {
    read_text("/proc/self/status")?
        .lines()
        .find_map(|line| {
            line.strip_prefix("VmHWM:")?
                .split_whitespace()
                .next()?
                .parse::<u64>()
                .ok()
        })
}

/// Process-wide counters over one measured window.
#[derive(Clone, Copy, Default)]
struct Window {
    thread_cpu_us: u64,
    process_cpu_us: u64,
    rchar: u64,
}

impl Window {
    fn snapshot() -> Self {
        Self {
            thread_cpu_us: thread_cpu_us().unwrap_or(0),
            process_cpu_us: process_cpu_us().unwrap_or(0),
            rchar: rchar_bytes().unwrap_or(0),
        }
    }

    fn delta(self, start: Self) -> WindowDelta {
        WindowDelta {
            thread_cpu_ms: (self.thread_cpu_us.saturating_sub(start.thread_cpu_us)) as f64 / 1_000.0,
            process_cpu_ms: (self.process_cpu_us.saturating_sub(start.process_cpu_us)) as f64
                / 1_000.0,
            rchar_bytes: self.rchar.saturating_sub(start.rchar),
        }
    }
}

struct WindowDelta {
    thread_cpu_ms: f64,
    process_cpu_ms: f64,
    rchar_bytes: u64,
}

// ---------------------------------------------------------------------------
// Fixture
// ---------------------------------------------------------------------------

struct Fixture {
    dir: PathBuf,
    /// Keeps a temp fixture alive; `None` for a caller-supplied persistent dir.
    _temp: Option<tempfile::TempDir>,
    facts: serde_json::Value,
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

fn sqlite3(db: &Path, sql: &str) -> io::Result<()> {
    let output = std::process::Command::new("sqlite3")
        .arg(db)
        .arg(sql)
        .output()?;
    if !output.status.success() {
        return Err(io::Error::other(format!(
            "sqlite3 failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }
    Ok(())
}

struct Shape {
    dirs: usize,
    per_dir: usize,
    cached: usize,
    issues: usize,
    drafts: usize,
}

impl Shape {
    fn from_env() -> Self {
        Self {
            dirs: env_usize("POLL_DIRS", 147),
            per_dir: env_usize("POLL_FILES_PER_DIR", 2_000),
            cached: env_usize("POLL_CACHED", 400),
            issues: env_usize("POLL_ISSUES", 300),
            drafts: env_usize("POLL_DRAFTS", 5),
        }
    }

    fn nodes(&self) -> usize {
        self.dirs * self.per_dir + self.dirs + 1
    }

    /// Path of every top-level directory, i.e. the visible rows of the root
    /// folder page. A `GetStates` batch of these resolves to all 147 subtrees.
    fn page_root_paths(&self) -> Vec<String> {
        (0..self.dirs).map(|d| format!("d{d:03}")).collect()
    }

    /// A mixed visible-row patch: two directory rows (subtree aggregation) plus
    /// files under one of them (point status).
    fn patch_paths(&self) -> Vec<String> {
        let mut paths = vec!["d005".to_string(), "d006".to_string()];
        let base = 5 * self.per_dir;
        for f in 0..22 {
            paths.push(format!("d005/f{f:05}"));
            let _ = base;
        }
        paths
    }

    fn cached_body(index: usize) -> Vec<u8> {
        (0..4_096).map(|i| ((i + index) % 251) as u8).collect()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "dirs": self.dirs,
            "files_per_dir": self.per_dir,
            "nodes": self.nodes(),
            "cached": self.cached,
            "issues": self.issues,
            "drafts": self.drafts,
        })
    }
}

fn build_fixture(shape: &Shape) -> Result<Fixture, Box<dyn std::error::Error>> {
    let started = Instant::now();
    let persistent = env::var("POLL_FIXTURE_DIR")
        .ok()
        .filter(|p| !p.trim().is_empty())
        .map(PathBuf::from);
    let (dir, temp) = match persistent {
        Some(path) => {
            fs::create_dir_all(&path)?;
            (path, None)
        }
        None => {
            let t = tempfile::tempdir()?;
            (t.path().to_path_buf(), Some(t))
        }
    };
    let facts_path = dir.join("poll-fixture-facts.json");
    let db_path = dir.join("state.sqlite");
    let existed = db_path.is_file();

    let build_ms = if existed {
        let persisted: serde_json::Value = serde_json::from_slice(&fs::read(&facts_path)?)?;
        let persisted_nodes = persisted["nodes"].as_u64().unwrap_or(0) as usize;
        anyhow_check(persisted_nodes == shape.nodes(), || {
            format!(
                "reused fixture has {persisted_nodes} nodes, env wants {}; delete {} or change POLL_* shape",
                shape.nodes(),
                dir.display()
            )
        })?;
        started.elapsed().as_millis()
    } else {
        let mut nodes = Vec::with_capacity(shape.nodes());
        nodes.push(node("root".into(), None, "fixture".into(), true, 0));
        let mut files = 0usize;
        for d in 0..shape.dirs {
            let id = format!("d{d:03}");
            nodes.push(node(id.clone(), Some("root".into()), id.clone(), true, 0));
            for c in 0..shape.per_dir {
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
        let mut store = Store::open(&dir)?;
        store.install_remote_snapshot(&nodes, "fixture-cursor-1", "fixture-server-1")?;
        store.ensure_fs_root("root")?;
        drop(nodes);

        for index in 0..shape.cached.min(files) {
            let body = Shape::cached_body(index);
            store.publish_download(
                &DownloadTicket {
                    id: format!("file-{index:06}"),
                    version: "fixture-v1".into(),
                },
                &body,
                &sha256(&body),
            )?;
        }
        store.set_cache_limit(1 << 30)?;
        if shape.dirs > 2 {
            let _ = store.pin("d001", true);
        }
        for k in 0..shape.drafts {
            let id = format!("file-{:06}", 5_000 + k * 17);
            let draft = store.begin_draft(&id, "fixture-v1", None)?;
            store.draft_write(&draft, 0, b"poll fixture draft", false)?;
        }
        drop(store);

        // Inject conflicted files through the sqlite3 CLI so `GetIssues` has a
        // full first page. Spread by rowid so issue paths have varying depth.
        if shape.issues > 0 {
            sqlite3(
                &db_path,
                &format!(
                    "BEGIN IMMEDIATE; \
                     UPDATE nodes SET conflict=1 WHERE id IN ( \
                       SELECT id FROM nodes WHERE directory=0 AND rowid % 977 = 0 \
                       LIMIT {}); \
                     COMMIT; PRAGMA wal_checkpoint(TRUNCATE);",
                    shape.issues
                ),
            )?;
        } else {
            let _ = sqlite3(&db_path, "PRAGMA wal_checkpoint(TRUNCATE);");
        }

        let facts = serde_json::json!({
            "nodes": shape.nodes(),
            "files": files,
            "shape": shape.json(),
        });
        fs::write(&facts_path, serde_json::to_vec_pretty(&facts)?)?;
        started.elapsed().as_millis()
    };

    let db_bytes = fs::metadata(&db_path).map(|m| m.len()).unwrap_or(0);
    let facts = serde_json::json!({
        "nodes": shape.nodes(),
        "cached": shape.cached,
        "issues": shape.issues,
        "db_bytes": db_bytes,
        "build_ms": build_ms,
        "reused": existed,
        "dir": dir.display().to_string(),
    });
    println!("T2POLL FIXTURE {}", facts);
    Ok(Fixture {
        dir,
        _temp: temp,
        facts,
    })
}

fn anyhow_check(ok: bool, message: impl FnOnce() -> String) -> Result<(), Box<dyn std::error::Error>> {
    if ok {
        Ok(())
    } else {
        Err(io::Error::other(message()).into())
    }
}

fn profile(mount: PathBuf) -> Profile {
    Profile {
        schema: 1,
        task_id: uuid::Uuid::new_v4().to_string(),
        host: "t2-poll.example.invalid".into(),
        tls_name: "t2-poll.example.invalid".into(),
        trusted_ca_pem: Vec::new(),
        username: "t2-poll".into(),
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
    }
}

// ---------------------------------------------------------------------------
// Warm-up
// ---------------------------------------------------------------------------

/// Wait until the committed status projection answers `GetStates(".")` without
/// the full-namespace seed read. Readiness is inferred from per-call rchar: a
/// cold root call re-reads every node (~1.25 GB at 294k), a warm one does not.
fn warm_projection(service: &Arc<Service>) -> serde_json::Value {
    let timeout = Duration::from_millis(env_u64("POLL_WARM_TIMEOUT_MS", 90_000));
    let rchar_max = env_u64("POLL_WARM_RCHAR_MAX", 20 * 1024 * 1024);
    let started = Instant::now();
    let mut cold_ms = 0.0f64;
    let mut calls = 0usize;
    let mut warm_us: Vec<u64> = Vec::new();
    let mut warm_rchar: Vec<u64> = Vec::new();
    let mut cold_rchar = 0u64;
    let mut consecutive = 0usize;
    loop {
        let before = rchar_bytes().unwrap_or(0);
        let began = Instant::now();
        let result = desktop::states(service, vec![".".to_string()]);
        let us = began.elapsed().as_micros() as u64;
        let read = rchar_bytes().unwrap_or(0).saturating_sub(before);
        calls += 1;
        if calls == 1 {
            cold_ms = ms(us);
            cold_rchar = read;
            println!(
                "T2POLL COLD {}",
                serde_json::json!({"ms": cold_ms, "rchar_bytes": read, "ok": result.is_ok()})
            );
        }
        if result.is_ok() && read <= rchar_max {
            warm_us.push(us);
            warm_rchar.push(read);
            consecutive += 1;
        } else {
            consecutive = 0;
        }
        if consecutive >= 2 {
            break;
        }
        if started.elapsed() >= timeout {
            break;
        }
        thread::sleep(Duration::from_millis(150));
    }
    warm_us.sort_unstable();
    warm_rchar.sort_unstable();
    serde_json::json!({
        "ready": consecutive >= 2,
        "ready_ms": ms(started.elapsed().as_micros() as u64),
        "calls": calls,
        "cold_ms": cold_ms,
        "cold_rchar_bytes": cold_rchar,
        "warm_p50_ms": ms(percentile(&warm_us, 0.50)),
        "warm_rchar_p50_bytes": warm_rchar.get(warm_rchar.len() / 2).copied().unwrap_or(0),
        "warm_rchar_max_bytes": warm_rchar.last().copied().unwrap_or(0),
    })
}

// ---------------------------------------------------------------------------
// Measurements
// ---------------------------------------------------------------------------

fn timed_states(service: &Arc<Service>, set: &mut Samples) {
    let (result, us, read) = probe(|| desktop::states(service, vec![".".to_string()]));
    set.record(us, read, result.is_err());
}

fn timed_page_root(service: &Arc<Service>, paths: &[String], set: &mut Samples) {
    let (result, us, read) = probe(|| desktop::states(service, paths.to_vec()));
    set.record(us, read, result.is_err());
}

fn timed_patch(service: &Arc<Service>, paths: &[String], set: &mut Samples) {
    let (result, us, read) = probe(|| desktop::states(service, paths.to_vec()));
    set.record(us, read, result.is_err());
}

fn timed_overview(service: &Arc<Service>, set: &mut OpSet) {
    let (result, us, read) = probe(|| desktop::overview_measured(service));
    match result {
        Ok((body, timings)) => {
            if timings.read_only {
                set.overview_read_only += 1;
            }
            if timings.derived_cached {
                set.overview_derived_cached += 1;
            }
            let failed = serde_json::from_str::<serde_json::Value>(&body).is_err();
            set.overview.record(us, read, failed);
        }
        Err(_) => set.overview.record(us, read, true),
    }
}

fn timed_transfers(service: &Arc<Service>, set: &mut Samples) {
    let (result, us, read) = probe(|| desktop::transfers(service));
    set.record(us, read, result.is_err());
}

fn timed_issues(service: &Arc<Service>, set: &mut Samples) {
    let (result, us, read) = probe(|| desktop::issues(service, ""));
    set.record(us, read, result.is_err());
}

fn timed_cache_settings(service: &Arc<Service>, set: &mut Samples) {
    let (result, us, read) = probe(|| desktop::cache_settings(service));
    set.record(us, read, result.is_err());
}

/// Per-call rchar attribution in the authoritative order the desktop uses: a
/// namespace-root `GetStates(".")` (whole-tree scoped read while no projection
/// is published), a root-page `GetStates(<147 dirs>)` (same O(nodes) walk even
/// with a warm projection), then a small visible-row patch.
fn timed_all(service: &Arc<Service>, shape: &Shape, set: &mut OpSet) {
    timed_states(service, &mut set.states_root);
    let page = shape.page_root_paths();
    timed_page_root(service, &page, &mut set.states_page_root);
    let patch = shape.patch_paths();
    timed_patch(service, &patch, &mut set.states_patch);
    timed_overview(service, set);
    timed_transfers(service, &mut set.transfers);
    timed_issues(service, &mut set.issues);
    timed_cache_settings(service, &mut set.cache_settings);
}

/// Purely authoritative (no status projection) read paths, measured before
/// `Service::start()`. This is also the per-call cost any batch pays while the
/// cache has no published snapshot (cold startup / oversized profile / a
/// projection that cannot describe the requested roots).
fn authoritative_phase(service: &Arc<Service>, shape: &Shape) -> serde_json::Value {
    let iters = env_usize("POLL_AUTH_ITERS", 2);
    let page = shape.page_root_paths();
    // One unmeasured warmup call so one-time schema/root setup is not counted.
    let _ = desktop::states(service, vec![".".to_string()]);
    let mut root = Samples::default();
    let mut page_root = Samples::default();
    let mut seeds = Samples::default();
    let mut issue_count = Samples::default();
    let mut availability = Samples::default();
    let window = Window::snapshot();
    for _ in 0..iters {
        let (result, us, read) = probe(|| desktop::states(service, vec![".".to_string()]));
        root.record(us, read, result.is_err());
        let (result, us, read) = probe(|| desktop::states(service, page.clone()));
        page_root.record(us, read, result.is_err());
        let (result, us, read) = probe(|| {
            let store = service.bridge.store.lock().unwrap();
            store.desktop_status_seeds()
        });
        seeds.record(us, read, result.is_err());
        let (result, us, read) = probe(|| {
            let store = service.bridge.store.lock().unwrap();
            store.sync_issue_count("root")
        });
        issue_count.record(us, read, result.is_err());
        let (result, us, read) = probe(|| {
            let store = service.bridge.store.lock().unwrap();
            store.cache_availability()
        });
        availability.record(us, read, result.is_err());
    }
    let delta = Window::snapshot().delta(window);
    serde_json::json!({
        "iters": iters,
        "projection": false,
        "states_root": root.json(),
        "states_page_root": page_root.json(),
        "desktop_status_seeds_full": seeds.json(),
        "sync_issue_count": issue_count.json(),
        "cache_availability": availability.json(),
        "thread_cpu_ms": delta.thread_cpu_ms,
        "process_cpu_ms": delta.process_cpu_ms,
        "rchar_bytes": delta.rchar_bytes,
    })
}

/// Back-to-back (unthrottled) per-interface cost at steady state, with per-call
/// rchar attribution. No throttling: this is the per-call cost floor.
fn burst(service: &Arc<Service>, shape: &Shape) -> serde_json::Value {
    let iters = env_usize("POLL_BURST_ITERS", 6);
    let page = shape.page_root_paths();
    let patch = shape.patch_paths();
    let mut set = OpSet::default();
    let window = Window::snapshot();
    let began = Instant::now();
    for _ in 0..iters {
        timed_states(service, &mut set.states_root);
        timed_page_root(service, &page, &mut set.states_page_root);
        timed_patch(service, &patch, &mut set.states_patch);
        timed_overview(service, &mut set);
        timed_transfers(service, &mut set.transfers);
        timed_issues(service, &mut set.issues);
        timed_cache_settings(service, &mut set.cache_settings);
    }
    let wall = began.elapsed().as_secs_f64();
    let delta = Window::snapshot().delta(window);
    serde_json::json!({
        "iters": iters,
        "wall_s": wall,
        "states_root": set.states_root.json(),
        "states_page_root": set.states_page_root.json(),
        "states_patch": set.states_patch.json(),
        "overview": set.overview.json(),
        "transfers": set.transfers.json(),
        "issues": set.issues.json(),
        "cache_settings": set.cache_settings.json(),
        "overview_read_only": set.overview_read_only,
        "overview_derived_cached": set.overview_derived_cached,
        "thread_cpu_ms": delta.thread_cpu_ms,
        "process_cpu_ms": delta.process_cpu_ms,
        "rchar_bytes": delta.rchar_bytes,
    })
}

/// Poll mode: which RPC set a phase drives.
/// * `root` / `page_root` / `patch` - only that GetStates shape, every tick;
/// * `mixed` - realistic composite: a status-signal-driven GetStates of the
///   visible root page every tick (<=4 Hz in the field), `GetOverview` 1 Hz,
///   `GetTransfers` 0.5 Hz, `GetIssues` page 1 at 0.25 Hz,
///   `GetStates(".")` 1 Hz, `GetCacheSettings` 0.125 Hz.
fn poll_phase(
    service: &Arc<Service>,
    shape: &Shape,
    mode: &str,
    hz: f64,
    seconds: f64,
    rep: usize,
) -> serde_json::Value {
    let period = Duration::from_secs_f64(1.0 / hz);
    let page = shape.page_root_paths();
    let patch = shape.patch_paths();
    let mut set = OpSet::default();
    let window = Window::snapshot();
    let load_before = loadavg1();
    let started = Instant::now();
    let deadline = started + Duration::from_secs_f64(seconds);
    let mut next = started;
    let mut ticks = 0usize;
    let mut overdue_us = 0u64;
    let mut max_overdue_us = 0u64;
    let every = |secs: f64| -> usize { ((secs * hz).round() as usize).max(1) };
    let overview_every = every(1.0);
    let root_every = every(1.0);
    let transfers_every = every(2.0);
    let issues_every = every(4.0);
    let cache_every = every(8.0);
    while Instant::now() < deadline {
        let now = Instant::now();
        if now < next {
            thread::sleep(next - now);
        }
        let late = Instant::now().saturating_duration_since(next);
        overdue_us += late.as_micros() as u64;
        max_overdue_us = max_overdue_us.max(late.as_micros() as u64);
        match mode {
            "root" => timed_states(service, &mut set.states_root),
            "page_root" => timed_page_root(service, &page, &mut set.states_page_root),
            "patch" => timed_patch(service, &patch, &mut set.states_patch),
            _ => {
                timed_page_root(service, &page, &mut set.states_page_root);
                if ticks % overview_every == 0 {
                    timed_overview(service, &mut set);
                }
                if ticks % root_every == 0 {
                    timed_states(service, &mut set.states_root);
                }
                if ticks % transfers_every == 0 {
                    timed_transfers(service, &mut set.transfers);
                }
                if ticks % issues_every == 0 {
                    timed_issues(service, &mut set.issues);
                }
                if ticks % cache_every == 0 {
                    timed_cache_settings(service, &mut set.cache_settings);
                }
            }
        }
        ticks += 1;
        next += period;
        if next < Instant::now() {
            // Fell behind schedule: restart the cadence instead of bursting.
            next = Instant::now();
        }
    }
    let wall = started.elapsed().as_secs_f64();
    let delta = Window::snapshot().delta(window);
    serde_json::json!({
        "mode": mode,
        "rep": rep,
        "hz_requested": hz,
        "seconds": seconds,
        "ticks": ticks,
        "wall_s": wall,
        "actual_hz": ticks as f64 / wall,
        "avg_overdue_ms": if ticks == 0 { 0.0 } else { overdue_us as f64 / ticks as f64 / 1_000.0 },
        "max_overdue_ms": max_overdue_us as f64 / 1_000.0,
        "states_root": set.states_root.json(),
        "states_page_root": set.states_page_root.json(),
        "states_patch": set.states_patch.json(),
        "overview": set.overview.json(),
        "transfers": set.transfers.json(),
        "issues": set.issues.json(),
        "cache_settings": set.cache_settings.json(),
        "overview_read_only": set.overview_read_only,
        "overview_derived_cached": set.overview_derived_cached,
        "thread_cpu_ms": delta.thread_cpu_ms,
        "process_cpu_ms": delta.process_cpu_ms,
        "rchar_bytes": delta.rchar_bytes,
        "load1_before": load_before,
        "load1_after": loadavg1(),
    })
}

/// Issue #27-style warm concurrency probe: N threads hammer `GetStates(".")`
/// for `seconds`, like the GUI refresh + status patches overlapping other
/// desktop reads. Reports per-call latency and read amplification per call; a
/// per-call rchar far above the single-threaded warm value means a batch fell
/// back to the authoritative scoped read.
fn concurrent_phase(service: &Arc<Service>, threads: usize, seconds: f64) -> serde_json::Value {
    let window = Window::snapshot();
    let began = Instant::now();
    let deadline = began + Duration::from_secs_f64(seconds);
    let service = service.clone();
    let mut handles = Vec::new();
    for _ in 0..threads {
        let service = service.clone();
        handles.push(thread::spawn(move || {
            let mut us = Vec::new();
            let mut errors = 0u64;
            while Instant::now() < deadline {
                let t = Instant::now();
                let result = desktop::states(&service, vec![".".to_string()]);
                us.push(t.elapsed().as_micros() as u64);
                if result.is_err() {
                    errors += 1;
                }
            }
            (us, errors)
        }));
    }
    let mut all = Vec::new();
    let mut errors = 0u64;
    for handle in handles {
        let (us, err) = handle.join().expect("concurrent probe thread");
        all.extend(us);
        errors += err;
    }
    let wall = began.elapsed().as_secs_f64();
    let delta = Window::snapshot().delta(window);
    let calls = all.len() as u64;
    let mut samples = Samples::default();
    for us in &all {
        samples.record(*us, 0, false);
    }
    serde_json::json!({
        "threads": threads,
        "seconds": seconds,
        "calls": calls,
        "wall_s": wall,
        "calls_per_s": calls as f64 / wall,
        "errors": errors,
        "latency": samples.json(),
        "phase_rchar_bytes": delta.rchar_bytes,
        "rchar_per_call_bytes": if calls == 0 { 0 } else { delta.rchar_bytes / calls },
        "thread_cpu_ms": delta.thread_cpu_ms,
        "process_cpu_ms": delta.process_cpu_ms,
    })
}

/// GetOverview while the Store generation keeps changing: every iteration
/// commits a settings write so the generation-keyed `OverviewDerived` cache
/// (`sync_issue_count` + `cache_availability`) must be recomputed. This is the
/// active-sync shape of the GUI's <=1 Hz overview poll.
fn sync_overview_phase(service: &Arc<Service>) -> serde_json::Value {
    let iters = env_usize("POLL_SYNC_ITERS", 6);
    let mut set = OpSet::default();
    let window = Window::snapshot();
    let began = Instant::now();
    for i in 0..iters {
        let limit = 1u64 << 30;
        let _ = service
            .bridge
            .store
            .lock()
            .unwrap()
            .set_cache_limit(limit + (i as u64 % 2));
        timed_overview(service, &mut set);
    }
    let wall = began.elapsed().as_secs_f64();
    let delta = Window::snapshot().delta(window);
    serde_json::json!({
        "iters": iters,
        "wall_s": wall,
        "overview": set.overview.json(),
        "overview_read_only": set.overview_read_only,
        "overview_derived_cached": set.overview_derived_cached,
        "thread_cpu_ms": delta.thread_cpu_ms,
        "process_cpu_ms": delta.process_cpu_ms,
        "rchar_bytes": delta.rchar_bytes,
        "rchar_per_call_bytes": delta.rchar_bytes / iters as u64,
    })
}

// ---------------------------------------------------------------------------
// Test entry
// ---------------------------------------------------------------------------

/// (mode, hz, seconds, reps) phase table. Overridable with `POLL_PHASES` as
/// comma-separated `mode:hz:seconds` entries; reps follow `POLL_REPS`.
fn phase_table() -> Vec<(String, f64, f64, usize)> {
    let reps = env_usize("POLL_REPS", 3).max(1);
    if let Ok(spec) = env::var("POLL_PHASES") {
        return spec
            .split(',')
            .filter_map(|entry| {
                let mut parts = entry.split(':');
                let mode = parts.next()?.to_string();
                let hz: f64 = parts.next()?.parse().ok()?;
                let seconds: f64 = parts.next()?.parse().ok()?;
                Some((mode, hz, seconds, reps))
            })
            .collect();
    }
    let seconds = env_u64("POLL_SECONDS", 6) as f64;
    let stress = env_u64("POLL_STRESS_SECONDS", 6) as f64;
    vec![
        // Warm projection per-call floor at the two real GetStates shapes.
        ("page_root".into(), 1.0, seconds, reps),
        ("page_root".into(), 4.0, seconds, reps),
        ("root".into(), 1.0, seconds, reps),
        // Realistic field composite (status storms at the 250 ms coalescing
        // ceiling) and the 10 Hz upper bound from the task statement.
        ("mixed".into(), 4.0, seconds, reps),
        ("mixed".into(), 10.0, stress, 1),
        ("root".into(), 10.0, stress, 1),
        ("patch".into(), 4.0, seconds, reps),
    ]
}

#[test]
#[ignore]
fn poll_amplification() -> Result<(), Box<dyn std::error::Error>> {
    let shape = Shape::from_env();
    let fixture = build_fixture(&shape)?;
    let mount = fixture.dir.join("mount");
    fs::create_dir_all(&mount)?;

    let store = Store::open(&fixture.dir)?;
    let service = Service::new(profile(mount), store, fixture.dir.join("worker-unavailable"))?;

    // Authoritative (no projection) cost, before the cache worker starts. This
    // is the per-call price of any batch that falls back to the scoped read.
    let auth = authoritative_phase(&service, &shape);
    println!("T2POLL AUTH {auth}");
    thread::sleep(Duration::from_millis(250));

    let tasks = service.clone().start();
    let warm = warm_projection(&service);
    println!("T2POLL WARM {warm}");
    thread::sleep(Duration::from_millis(250));

    let burst = burst(&service, &shape);
    println!("T2POLL BURST {burst}");

    let conc = concurrent_phase(
        &service,
        env_usize("POLL_CONC_THREADS", 16),
        env_u64("POLL_CONC_SECONDS", 6) as f64,
    );
    println!("T2POLL CONC {conc}");

    let fixture_facts = fixture.facts.clone();
    for (mode, hz, seconds, reps) in phase_table() {
        for rep in 0..reps {
            let phase = poll_phase(&service, &shape, &mode, hz, seconds, rep);
            println!("T2POLL PHASE {phase}");
        }
    }

    let sync_overview = sync_overview_phase(&service);
    println!("T2POLL SYNCOV {sync_overview}");

    let final_warm = warm_projection(&service);
    let summary = serde_json::json!({
        "fixture": fixture_facts,
        "shape": shape.json(),
        "final_projection_check": final_warm,
        "peak_rss_kb": peak_rss_kb(),
        "load1_end": loadavg1(),
    });
    println!("T2POLL SUMMARY {summary}");

    service.shutdown();
    tasks.join();
    if env_flag("POLL_KEEP") && fixture._temp.is_some() {
        let kept = fixture.dir.clone();
        std::mem::forget(fixture._temp);
        println!("T2POLL KEPT {}", kept.display());
    }
    Ok(())
}
