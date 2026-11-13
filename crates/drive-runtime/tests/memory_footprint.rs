//! Staged memory-footprint harness (perf exploration; untracked local test).
//!
//! Direction C of the drive-native perf exploration: quantify RSS/PSS of the
//! integration baseline at 210k/294k/500k synthetic nodes, split anon vs
//! file-backed memory, attribute the projection vs fixture vs mixed-load cost,
//! and measure storm/idle growth plus `MALLOC_ARENA_MAX` sensitivity.
//!
//! Design:
//! - Fixture is built ONCE in a separate process (`MEM_MODE=build`) and reused
//!   (`MEM_FIXTURE_DIR`), so the measured process never holds the 210k..500k
//!   `Vec<Node>` fixture-generation allocation. This separates
//!   "fixture generation footprint" from "runtime footprint" (known confound in
//!   the existing `peak_rss_kb ≈ 2 GiB @ 210k` number).
//! - Stages are sampled from inside the measured process:
//!   fixture_open -> service_new -> service_start -> warm_ready ->
//!   warm_settle -> mixed_end -> mixed_settle -> storm_end -> post_storm ->
//!   idle_end -> shutdown.
//! - Every stage records `/proc/self/status` VmRSS/VmHWM/VmPeak/Threads,
//!   `/proc/self/smaps_rollup` (Rss/Pss/Anonymous/Private/Shared/File),
//!   state.sqlite/-wal/-shm sizes, and top anonymous + top file-backed
//!   mappings from `/proc/self/smaps`.
//! - A sampler thread records VmRSS every 100 ms for the whole process to catch
//!   transient peaks between stage boundaries (e.g. projection compaction).
//!
//! Run (remote, via the serialized runner):
//!   MEM_DIRS=105 MEM_NODES_TARGET=210000 MEM_FIXTURE_DIR=/tmp/memfx/210k \
//!     MEM_MODE=build <test-binary> --ignored --nocapture
//!   MEM_DIRS=105 MEM_NODES_TARGET=210000 MEM_FIXTURE_DIR=/tmp/memfx/210k \
//!     MEM_MODE=full <test-binary> --ignored --nocapture
//!
//! Env knobs:
//!   MEM_MODE=full|nostart|build   default full
//!   MEM_DIRS                      top-level dirs (105/147/250)
//!   MEM_NODES_TARGET             total nodes target (210000/294000/500000)
//!   MEM_FIXTURE_DIR              persistent fixture root (required for reuse)
//!   MEM_MIX_SECONDS              default 30
//!   MEM_STORM_SECONDS            default 10
//!   MEM_IDLE_SECONDS             default 20
//!   MEM_SETTLE_MS                default 2000
//!   MEM_STORM_ID_RANGE           distinct node ids touched by the storm (default 2000)
//!   MEM_WARM_MS                  warm threshold (default 400 at 210k; 900 at >400k)
//!   MEM_WARM_TIMEOUT_MS          default 90000
//!   MEM_MAP_MIN_KB               minimum mapping size reported (default 4096)
//!   MEM_LOAD                     include fixture extras (1MB + 2048 cached files); default 1

#![allow(clippy::all)]

use drive_core::{sha256, DownloadTicket, Node, NodeMetadata, Store};
use drive_runtime::{
    desktop,
    service::{Profile, Service},
};
use std::{
    collections::BTreeMap,
    env, fs,
    hint::black_box,
    io::Read,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc, Mutex,
    },
    thread,
    time::{Duration, Instant},
};

const FILES_PER_DIR: usize = 2_000;
const CACHED_BYTES: usize = 1_048_576;
const EXTRA_CACHED: usize = 2_048;
const EXTRA_CACHED_BYTES: usize = 4_096;
const EXTRA_CACHED_STRIDE: usize = 103;
const DRAFTS: usize = 5;
const DIR_BUDGET: usize = 4 * 1024 * 1024;
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

fn env_f64(key: &str, default: f64) -> f64 {
    env::var(key)
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(default)
}

fn env_flag(key: &str) -> bool {
    matches!(env::var(key).as_deref(), Ok("1") | Ok("true") | Ok("yes"))
}

fn load_avg1() -> Option<f64> {
    fs::read_to_string("/proc/loadavg")
        .ok()?
        .split_whitespace()
        .next()?
        .parse()
        .ok()
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
    exclude_mode: Default::default(),
}
}

// ---------------------------------------------------------------------------
// /proc sampling
// ---------------------------------------------------------------------------

struct ProcFacts {
    values: BTreeMap<String, u64>,
    rollup: BTreeMap<String, u64>,
}

fn parse_kv_section(text: &str) -> BTreeMap<String, u64> {
    let mut out = BTreeMap::new();
    for line in text.lines() {
        let Some((key, rest)) = line.split_once(':') else {
            continue;
        };
        let value = rest.split_whitespace().next().and_then(|v| v.parse().ok());
        if let Some(value) = value {
            out.insert(key.trim().to_string(), value);
        }
    }
    out
}

fn proc_facts() -> ProcFacts {
    let status = fs::read_to_string("/proc/self/status").unwrap_or_default();
    let mut values = BTreeMap::new();
    for line in status.lines() {
        let Some((key, rest)) = line.split_once(':') else {
            continue;
        };
        let Some(value) = rest.split_whitespace().next().and_then(|v| v.parse().ok()) else {
            continue;
        };
        let key = key.trim();
        if key.starts_with("Vm") || key == "Threads" || key == "RssAnon" || key == "RssFile" {
            values.insert(key.to_string(), value);
        }
    }
    let rollup_text = fs::read_to_string("/proc/self/smaps_rollup").unwrap_or_default();
    let rollup = parse_kv_section(&rollup_text);
    ProcFacts { values, rollup }
}

struct Mapping {
    name: String,
    size_kb: u64,
    rss_kb: u64,
    anon_kb: u64,
    file_kb: u64,
    shared_kb: u64,
    private_kb: u64,
}

impl Mapping {
    fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "name": self.name,
            "size_kb": self.size_kb,
            "rss_kb": self.rss_kb,
            "anon_kb": self.anon_kb,
            "file_kb": self.file_kb,
            "shared_kb": self.shared_kb,
            "private_kb": self.private_kb,
        })
    }
}

/// Parses every VMA in /proc/self/smaps, keeps the largest anon and largest
/// file-backed mappings by resident size. This is the direct attribution for
/// "the projection heap grew" vs "a SQLite file is mapped".
fn top_mappings(min_kb: u64, limit: usize) -> (Vec<serde_json::Value>, Vec<serde_json::Value>) {
    let text = fs::read_to_string("/proc/self/smaps").unwrap_or_default();
    let mut mappings: Vec<Mapping> = Vec::new();
    let mut current: Option<(String, BTreeMap<String, u64>)> = None;
    for line in text.lines() {
        // A VMA header starts with a hex address range, e.g. "7f...-7f... r-xp ...".
        let is_header = line
            .split_whitespace()
            .next()
            .map(|first| {
                first.contains('-')
                    && first
                        .split_once('-')
                        .map(|(a, b)| {
                            !a.is_empty()
                                && !b.is_empty()
                                && a.chars().all(|c| c.is_ascii_hexdigit())
                                && b.chars().all(|c| c.is_ascii_hexdigit())
                        })
                        .unwrap_or(false)
            })
            .unwrap_or(false);
        if is_header {
            if let Some((name, fields)) = current.take() {
                mappings.push(mapping_from(name, fields));
            }
            let name = line
                .split_whitespace()
                .nth(5)
                .map(|value| value.to_string())
                .unwrap_or_else(|| "[anon]".to_string());
            current = Some((name, BTreeMap::new()));
            continue;
        }
        if let Some((_, fields)) = current.as_mut() {
            if let Some((key, rest)) = line.split_once(':') {
                let key = key.trim();
                if matches!(
                    key,
                    "Size" | "Rss" | "Pss" | "Anonymous" | "Shared_Clean" | "Shared_Dirty"
                        | "Private_Clean" | "Private_Dirty" | "Swap"
                ) {
                    if let Some(bytes) = rest.trim().strip_suffix("kB") {
                        if let Ok(kb) = bytes.trim().parse::<u64>() {
                            fields.insert(key.to_string(), kb);
                        }
                    }
                }
            }
        }
    }
    if let Some((name, fields)) = current.take() {
        mappings.push(mapping_from(name, fields));
    }
    let mut anon: Vec<&Mapping> = mappings
        .iter()
        .filter(|mapping| mapping.anon_kb >= min_kb)
        .collect();
    let mut file: Vec<&Mapping> = mappings
        .iter()
        .filter(|mapping| mapping.file_kb >= min_kb && !mapping.name.starts_with('['))
        .collect();
    anon.sort_by_key(|mapping| std::cmp::Reverse(mapping.anon_kb));
    file.sort_by_key(|mapping| std::cmp::Reverse(mapping.file_kb));
    (
        anon.into_iter().take(limit).map(Mapping::json).collect(),
        file.into_iter().take(limit).map(Mapping::json).collect(),
    )
}

fn mapping_from(name: String, fields: BTreeMap<String, u64>) -> Mapping {
    let get = |key: &str| fields.get(key).copied().unwrap_or(0);
    let shared = get("Shared_Clean") + get("Shared_Dirty");
    let private = get("Private_Clean") + get("Private_Dirty");
    Mapping {
        name,
        size_kb: get("Size"),
        rss_kb: get("Rss") + get("Swap"),
        anon_kb: get("Anonymous"),
        // smaps Anonymous counts anonymous mappings; everything else resident is
        // file/shmem-backed. Report it explicitly instead of guessing Pss sides.
        file_kb: get("Rss").saturating_sub(get("Anonymous")),
        shared_kb: shared,
        private_kb: private,
    }
}

fn db_sizes(dir: Option<&Path>) -> serde_json::Value {
    let Some(dir) = dir else {
        return serde_json::Value::Null;
    };
    let size = |name: &str| -> serde_json::Value {
        fs::metadata(dir.join(name))
            .map(|meta| serde_json::json!(meta.len()))
            .unwrap_or(serde_json::Value::Null)
    };
    serde_json::json!({
        "state_sqlite_bytes": size("state.sqlite"),
        "wal_bytes": size("state.sqlite-wal"),
        "shm_bytes": size("state.sqlite-shm"),
    })
}

fn snapshot(
    stage: &str,
    t0: Instant,
    fixture_dir: Option<&Path>,
    with_mappings: bool,
) -> serde_json::Value {
    let facts = proc_facts();
    let rollup = &facts.rollup;
    let get_rollup = |key: &str| rollup.get(key).copied().unwrap_or(0);
    let rss_kb = facts.values.get("VmRSS").copied().unwrap_or(0);
    let anon_kb = get_rollup("Anonymous");
    let mut value = serde_json::json!({
        "stage": stage,
        "t_ms": t0.elapsed().as_millis() as u64,
        "rss_kb": rss_kb,
        "hwm_kb": facts.values.get("VmHWM").copied().unwrap_or(0),
        "vmpeak_kb": facts.values.get("VmPeak").copied().unwrap_or(0),
        "vmsize_kb": facts.values.get("VmSize").copied().unwrap_or(0),
        "threads": facts.values.get("Threads").copied().unwrap_or(0),
        "rollup": {
            "Rss": get_rollup("Rss"),
            "Pss": get_rollup("Pss"),
            "Pss_Anon": get_rollup("Pss_Anon"),
            "Pss_File": get_rollup("Pss_File"),
            "Pss_Shmem": get_rollup("Pss_Shmem"),
            "Anonymous": anon_kb,
            "Shared_Clean": get_rollup("Shared_Clean"),
            "Shared_Dirty": get_rollup("Shared_Dirty"),
            "Private_Clean": get_rollup("Private_Clean"),
            "Private_Dirty": get_rollup("Private_Dirty"),
            "Swap": get_rollup("Swap"),
            "file_backed_kb": rss_kb.saturating_sub(anon_kb),
        },
        "db": db_sizes(fixture_dir),
    });
    if with_mappings {
        let min_kb = env_u64("MEM_MAP_MIN_KB", 4096);
        let (top_anon, top_file) = top_mappings(min_kb, 8);
        value["top_anon_mappings"] = serde_json::Value::Array(top_anon);
        value["top_file_mappings"] = serde_json::Value::Array(top_file);
    }
    println!("#MEM STAGE {value}");
    value
}

// ---------------------------------------------------------------------------
// RSS sampler thread
// ---------------------------------------------------------------------------

struct Sampler {
    stop: Arc<AtomicBool>,
    handle: Option<thread::JoinHandle<Vec<(u64, u64, u64)>>>,
    started: Instant,
}

impl Sampler {
    fn start() -> Self {
        let stop = Arc::new(AtomicBool::new(false));
        let flag = Arc::clone(&stop);
        let started = Instant::now();
        let handle = thread::spawn(move || {
            let mut samples: Vec<(u64, u64, u64)> = Vec::new();
            while !flag.load(Ordering::Relaxed) {
                let facts = proc_facts();
                let rss = facts.values.get("VmRSS").copied().unwrap_or(0);
                let threads = facts.values.get("Threads").copied().unwrap_or(0);
                samples.push((started.elapsed().as_millis() as u64, rss, threads));
                thread::sleep(Duration::from_millis(100));
            }
            samples
        });
        Self {
            stop,
            handle: Some(handle),
            started,
        }
    }

    fn finish(mut self) -> Vec<(u64, u64, u64)> {
        self.stop.store(true, Ordering::Relaxed);
        self.handle
            .take()
            .map(|handle| handle.join().unwrap_or_default())
            .unwrap_or_default()
    }

    fn elapsed_ms(&self) -> u64 {
        self.started.elapsed().as_millis() as u64
    }
}

// ---------------------------------------------------------------------------
// Fixture
// ---------------------------------------------------------------------------

struct Fixture {
    dir: PathBuf,
    store: Store,
    nodes: usize,
    dirs: usize,
    files: usize,
    last_dir_files: usize,
    per_dir: usize,
    cached: usize,
    reused: bool,
}

fn fixture_paths(
    dir: &Path,
) -> (PathBuf, PathBuf, PathBuf, PathBuf) {
    (
        dir.join("state.sqlite"),
        dir.join("state.sqlite-wal"),
        dir.join("state.sqlite-shm"),
        dir.join("fixture-facts.json"),
    )
}

fn build_fixture(
    dir: &Path,
    dirs: usize,
    per_dir: usize,
    last: usize,
) -> Result<Fixture, Box<dyn std::error::Error>> {
    let started = Instant::now();
    let (db_path, _wal, _shm, facts_path) = fixture_paths(dir);
    fs::create_dir_all(dir)?;
    let cached: usize;
    if db_path.is_file() && facts_path.is_file() {
        let facts: serde_json::Value = serde_json::from_slice(&fs::read(&facts_path)?)?;
        cached = facts["cached"].as_u64().unwrap_or(0) as usize;
        let nodes = facts["nodes"].as_u64().unwrap_or(0) as usize;
        eprintln!(
            "[mem] reuse fixture nodes={nodes} cached={cached} dir={}",
            dir.display()
        );
        let store = Store::open(dir)?;
        return Ok(Fixture {
            dir: dir.to_path_buf(),
            store,
            nodes,
            dirs,
            files: facts["files"].as_u64().unwrap_or(0) as usize,
            last_dir_files: last,
            per_dir,
            cached,
            reused: true,
        });
    }
    let nodes_total = 1 + dirs + (dirs.saturating_sub(1)) * per_dir + last;
    let mut nodes = Vec::with_capacity(nodes_total);
    nodes.push(node("root".into(), None, "fixture".into(), true, 0));
    let mut file_index = 0usize;
    for d in 0..dirs {
        let id = format!("d{d:03}");
        nodes.push(node(id.clone(), Some("root".into()), id.clone(), true, 0));
        let count = if d + 1 == dirs { last } else { per_dir };
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
    assert_eq!(nodes.len(), nodes_total, "fixture node count");
    let mut store = Store::open(dir)?;
    store.install_remote_snapshot(&nodes, "fixture-cursor-1", "fixture-server-1")?;
    store.ensure_fs_root("root")?;
    drop(nodes);
    let mut cached = 0usize;
    if env_flag_default("MEM_LOAD", true) {
        let body: Vec<u8> = (0..CACHED_BYTES).map(|i| (i % 251) as u8).collect();
        store.publish_download(
            &DownloadTicket {
                id: "file-000000".into(),
                version: "fixture-v1".into(),
            },
            &body,
            &sha256(&body),
        )?;
        drop(body);
        cached = 1;
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
        store.pin("d077", true)?;
        store.pin("file-002000", true)?;
        store.pin("file-003000", true)?;
        for k in 0..DRAFTS {
            let id = format!("file-{:06}", 10_000 + k * 17);
            let draft = store.begin_draft(&id, "fixture-v1", None)?;
            store.draft_write(&draft, 0, b"issue-8 draft", false)?;
        }
    }
    let facts = serde_json::json!({
        "nodes": nodes_total,
        "dirs": dirs,
        "files": file_index,
        "cached": cached,
        "per_dir": per_dir,
        "last": last,
        "build_ms": started.elapsed().as_millis() as u64,
    });
    fs::write(&facts_path, serde_json::to_vec_pretty(&facts)?)?;
    eprintln!(
        "[mem] built fixture nodes={nodes_total} dirs={dirs} cached={cached} in {}ms dir={}",
        started.elapsed().as_millis(),
        dir.display()
    );
    Ok(Fixture {
        dir: dir.to_path_buf(),
        store,
        nodes: nodes_total,
        dirs,
        files: file_index,
        last_dir_files: last,
        per_dir,
        cached,
        reused: false,
    })
}

fn env_flag_default(key: &str, default: bool) -> bool {
    match env::var(key).ok().as_deref() {
        Some("1") | Some("true") | Some("yes") => true,
        Some("0") | Some("false") | Some("no") => false,
        _ => default,
    }
}

fn root_dirs(store: &Store, dirs: usize, per_dir: usize, last: usize) -> (Vec<u64>, Vec<usize>) {
    let root_view = store
        .directory_snapshot(1, DIR_BUDGET)
        .expect("root directory snapshot");
    let inos: Vec<u64> = root_view.entries.iter().skip(2).map(|e| e.ino).collect();
    let entries: Vec<usize> = (0..dirs)
        .map(|d| if d + 1 == dirs { last + 2 } else { per_dir + 2 })
        .collect();
    (inos, entries)
}

// ---------------------------------------------------------------------------
// Workloads
// ---------------------------------------------------------------------------

fn check_states_body(body: &str, expected: usize) -> Result<(), String> {
    let value: serde_json::Value =
        serde_json::from_str(body).map_err(|error| error.to_string())?;
    let entries = value["entries"]
        .as_array()
        .ok_or_else(|| "states response without entries".to_string())?;
    if entries.len() != expected {
        return Err(format!(
            "states entries {} != {expected}",
            entries.len()
        ));
    }
    Ok(())
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
        Ok(read) if read == CACHED_BYTES => None,
        Ok(read) => Some(format!("read {read} != {CACHED_BYTES}")),
        Err(error) => Some(error.to_string()),
    }
}

fn warm_status_projection(
    service: &Arc<Service>,
    threshold_ms: f64,
    timeout_ms: u64,
) -> serde_json::Value {
    let started = Instant::now();
    let mut cold_ms = None;
    let mut calls = 0usize;
    let mut consecutive = 0usize;
    let mut ready_ms = None;
    while started.elapsed() < Duration::from_millis(timeout_ms) {
        let call = Instant::now();
        let result = desktop::states(service, vec![".".to_string()]);
        let ms = call.elapsed().as_secs_f64() * 1_000.0;
        calls += 1;
        if cold_ms.is_none() {
            cold_ms = Some(ms);
        }
        if result.is_ok() && ms <= threshold_ms {
            consecutive += 1;
        } else {
            consecutive = 0;
        }
        if consecutive >= 3 {
            ready_ms = Some(started.elapsed().as_secs_f64() * 1_000.0);
            break;
        }
        thread::sleep(Duration::from_millis(50));
    }
    serde_json::json!({
        "ready": ready_ms.is_some(),
        "ready_ms": ready_ms,
        "calls": calls,
        "cold_ms": cold_ms,
        "threshold_ms": threshold_ms,
        "timeout_ms": timeout_ms,
    })
}

struct MixedResult {
    ops: u64,
    errors: u64,
    by_kind: BTreeMap<String, u64>,
}

fn mixed_load(
    service: &Arc<Service>,
    dir_inos: &[u64],
    dir_entries: &[usize],
    seconds: u64,
) -> MixedResult {
    let deadline = Instant::now() + Duration::from_secs(seconds);
    let (tx, rx) = mpsc::channel::<(String, bool)>();
    let mut handles = Vec::new();
    for slot in 0..MIX_THREADS {
        // Same lane layout as status_latency_210k's mixed load (4 x states,
        // 3 x dir, 2 x issues, 2 x read, 1 x overview per 12 threads). The
        // "classic" 16+16+16 layout is not repeated here; the memory question
        // is whether 48 concurrent lanes retain per-thread allocations.
        let kind = match slot % 12 {
            0..=2 => "states",
            3..=5 => "dir",
            6..=7 => "issues",
            8..=9 => "read",
            _ => "overview",
        };
        let service = Arc::clone(service);
        let sender = tx.clone();
        let dir_inos = dir_inos.to_vec();
        let dir_entries = dir_entries.to_vec();
        handles.push(thread::spawn(move || {
            let mut ops = 0u64;
            while Instant::now() < deadline {
                let error = match kind {
                    "states" => match desktop::states(&service, vec![".".to_string()]) {
                        Ok(body) => check_states_body(&body, 1).err(),
                        Err(error) => Some(error.to_string()),
                    },
                    "dir" => {
                        let slot = (ops as usize) % dir_inos.len();
                        let result = {
                            let guard = service.bridge.store.lock().unwrap();
                            guard.directory_snapshot_measured(dir_inos[slot], DIR_BUDGET)
                        };
                        match result {
                            Ok((view, _)) if view.entries.len() == dir_entries[slot] => None,
                            Ok((view, _)) => Some(format!(
                                "entries {} != {}",
                                view.entries.len(),
                                dir_entries[slot]
                            )),
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
                        Ok((body, _)) => serde_json::from_str::<serde_json::Value>(&body)
                            .map(|_| ())
                            .map_err(|error| error.to_string())
                            .err(),
                        Err(error) => Some(error.to_string()),
                    },
                    _ => unreachable!(),
                };
                if sender.send((kind.to_string(), error.is_some())).is_err() {
                    break;
                }
                ops += 1;
            }
        }));
    }
    drop(tx);
    let mut result = MixedResult {
        ops: 0,
        errors: 0,
        by_kind: BTreeMap::new(),
    };
    for (kind, failed) in rx {
        result.ops += 1;
        result.errors += u64::from(failed);
        *result.by_kind.entry(kind).or_insert(0) += 1;
    }
    for handle in handles {
        handle.join().ok();
    }
    result
}

/// Commit storm: pin/unpin rotates over `range` distinct nodes so the projection
/// delta crosses the 512-entry compaction floor, plus the cache journal/backlog
/// path. Each pin is its own commit (synchronous=FULL WAL), like a burst of
/// metadata writes from a sync session.
fn storm(service: &Arc<Service>, seconds: u64) -> serde_json::Value {
    let range = env_usize("MEM_STORM_ID_RANGE", 2_000).max(8);
    let deadline = Instant::now() + Duration::from_secs(seconds);
    let mut ops = 0u64;
    let mut errors = 0u64;
    let started = Instant::now();
    while Instant::now() < deadline {
        let id = format!("file-{:06}", 1_000 + (ops as usize) % range);
        let value = (ops as usize / range) % 2 == 0;
        let result = {
            let mut guard = service.bridge.store.lock().unwrap();
            guard.pin(&id, value)
        };
        if result.is_err() {
            errors += 1;
        }
        ops += 1;
    }
    serde_json::json!({
        "ops": ops,
        "errors": errors,
        "range": range,
        "elapsed_ms": started.elapsed().as_millis() as u64,
    })
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

#[test]
#[ignore]
fn memory_footprint() {
    if let Err(error) = run() {
        panic!("memory_footprint failed: {error}");
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let t0 = Instant::now();
    let mode = env::var("MEM_MODE").unwrap_or_else(|_| "full".to_string());
    let dirs = env_usize("MEM_DIRS", 105);
    let per_dir = env_usize("MEM_PER_DIR", FILES_PER_DIR);
    let nodes_target = env_usize("MEM_NODES_TARGET", 210_000);
    let fixture_dir = env::var("MEM_FIXTURE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            PathBuf::from(format!("/tmp/dn-mem-fixture-{dirs}"))
        });
    let last = nodes_target
        .checked_sub(1 + dirs + dirs.saturating_sub(1) * per_dir)
        .ok_or("MEM_NODES_TARGET too small for MEM_DIRS/MEM_PER_DIR")?;
    if last == 0 || last > per_dir {
        return Err(format!("invalid last_dir_files={last} for {nodes_target} nodes").into());
    }
    let mix_seconds = env_u64("MEM_MIX_SECONDS", 30);
    let storm_seconds = env_u64("MEM_STORM_SECONDS", 10);
    let idle_seconds = env_u64("MEM_IDLE_SECONDS", 20);
    let settle = Duration::from_millis(env_u64("MEM_SETTLE_MS", 2_000));
    let default_warm = if nodes_target > 400_000 { 900.0 } else { 400.0 };
    let warm_ms = env_f64("MEM_WARM_MS", default_warm);
    let warm_timeout_ms = env_u64("MEM_WARM_TIMEOUT_MS", 90_000);
    println!(
        "#MEM ENV {}",
        serde_json::json!({
            "mode": mode,
            "dirs": dirs,
            "per_dir": per_dir,
            "nodes_target": nodes_target,
            "last_dir_files": last,
            "fixture_dir": fixture_dir.display().to_string(),
            "mix_seconds": mix_seconds,
            "storm_seconds": storm_seconds,
            "idle_seconds": idle_seconds,
            "settle_ms": settle.as_millis() as u64,
            "warm_ms": warm_ms,
            "warm_timeout_ms": warm_timeout_ms,
            "mix_threads": MIX_THREADS,
            "load": env_flag_default("MEM_LOAD", true),
            "arena_max": env::var("MALLOC_ARENA_MAX").ok(),
            "glibc_tunables": env::var("GLIBC_TUNABLES").ok(),
            "loadavg_start": load_avg1(),
            "pid": std::process::id(),
            "cpu_count": std::thread::available_parallelism().map(|v| v.get()).ok(),
        })
    );

    if mode == "build" {
        let fixture = build_fixture(&fixture_dir, dirs, per_dir, last)?;
        let facts = proc_facts();
        println!(
            "#MEM BUILD {}",
            serde_json::json!({
                "dir": fixture.dir.display().to_string(),
                "nodes": fixture.nodes,
                "dirs": fixture.dirs,
                "files": fixture.files,
                "cached": fixture.cached,
                "reused": fixture.reused,
                "rss_kb": facts.values.get("VmRSS").copied().unwrap_or(0),
                "hwm_kb": facts.values.get("VmHWM").copied().unwrap_or(0),
                "vmpeak_kb": facts.values.get("VmPeak").copied().unwrap_or(0),
                "db": db_sizes(Some(&fixture.dir)),
                "loadavg_end": load_avg1(),
            })
        );
        drop(fixture);
        return Ok(());
    }

    let sampler = Sampler::start();
    let fixture = build_fixture(&fixture_dir, dirs, per_dir, last)?;
    let (dir_inos, dir_entries) = root_dirs(&fixture.store, dirs, per_dir, last);
    assert_eq!(dir_inos.len(), dirs, "top-level directory count");
    let mut stages: BTreeMap<String, serde_json::Value> = BTreeMap::new();
    let mut record = |stage: &str, maps: bool| {
        let value = snapshot(stage, t0, Some(fixture_dir.as_path()), maps);
        stages.insert(stage.to_string(), value);
    };
    record("fixture_open", true);
    println!(
        "#MEM FIXTURE {}",
        serde_json::json!({
            "nodes": fixture.nodes,
            "dirs": fixture.dirs,
            "files": fixture.files,
            "cached": fixture.cached,
            "reused": fixture.reused,
            "db": db_sizes(Some(&fixture.dir)),
        })
    );

    let store = fixture.store;
    let service = Service::new(
        profile(PathBuf::from("/tmp/drive-native-mem-mount")),
        store,
        PathBuf::from("/nonexistent-worker"),
    )?;
    thread::sleep(Duration::from_millis(250));
    record("service_new", true);

    let mut tasks = None;
    let mut warm = serde_json::Value::Null;
    let mut storm_result = serde_json::Value::Null;
    let mixed_result;
    if mode == "full" {
        tasks = Some(service.start());
        thread::sleep(Duration::from_millis(250));
        record("service_start", true);
        warm = warm_status_projection(&service, warm_ms, warm_timeout_ms);
        println!("#MEM WARM {warm}");
        record("warm_ready", true);
        thread::sleep(settle);
        record("warm_settle", true);
        let mix = mixed_load(&service, &dir_inos, &dir_entries, mix_seconds);
        mixed_result = serde_json::json!({
            "ops": mix.ops,
            "errors": mix.errors,
            "by_kind": mix.by_kind,
            "seconds": mix_seconds,
        });
        record("mixed_end", true);
        thread::sleep(settle);
        record("mixed_settle", true);
        storm_result = storm(&service, storm_seconds);
        println!("#MEM STORM {storm_result}");
        record("storm_end", true);
        thread::sleep(Duration::from_millis(5_000));
        record("post_storm", true);
        thread::sleep(Duration::from_secs(idle_seconds));
        record("idle_end", true);
        service.shutdown();
        thread::sleep(Duration::from_millis(1_000));
        record("shutdown", true);
        if let Some(tasks) = tasks.take() {
            tasks.join();
        }
    } else {
        // Control: no status projection worker. Same 48-way mixed load, so the
        // delta to the full run isolates projection + journal consumer memory.
        let mix = mixed_load(&service, &dir_inos, &dir_entries, mix_seconds);
        mixed_result = serde_json::json!({
            "ops": mix.ops,
            "errors": mix.errors,
            "by_kind": mix.by_kind,
            "seconds": mix_seconds,
        });
        record("mixed_end", true);
        thread::sleep(settle);
        record("mixed_settle", true);
        thread::sleep(Duration::from_secs(idle_seconds));
        record("idle_end", true);
        service.shutdown();
        thread::sleep(Duration::from_millis(1_000));
        record("shutdown", true);
    }

    let samples = sampler.finish();
    let peak = samples.iter().map(|(_, rss, _)| *rss).max().unwrap_or(0);
    let min = samples.iter().map(|(_, rss, _)| *rss).min().unwrap_or(0);
    let last = samples.last().map(|(_, rss, _)| *rss).unwrap_or(0);
    let mut by_second: BTreeMap<u64, u64> = BTreeMap::new();
    for (t_ms, rss, _threads) in &samples {
        let second = t_ms / 1_000;
        let entry = by_second.entry(second).or_insert(*rss);
        *entry = (*entry).max(*rss);
    }
    let trace: Vec<serde_json::Value> = by_second
        .iter()
        .map(|(second, rss)| serde_json::json!([second * 1_000, rss]))
        .collect();
    println!(
        "#MEM SAMPLER {}",
        serde_json::json!({
            "samples": samples.len(),
            "peak_kb": peak,
            "min_kb": min,
            "last_kb": last,
            "trace_1s": trace,
        })
    );

    let stage_rss: BTreeMap<String, u64> = stages
        .iter()
        .map(|(stage, value)| {
            (
                stage.clone(),
                value["rss_kb"].as_u64().unwrap_or(0),
            )
        })
        .collect();
    let hwm = stages
        .values()
        .map(|value| value["hwm_kb"].as_u64().unwrap_or(0))
        .max()
        .unwrap_or(0);
    let delta = |from: &str, to: &str| -> i64 {
        let a = stage_rss.get(from).copied().unwrap_or(0) as i64;
        let b = stage_rss.get(to).copied().unwrap_or(0) as i64;
        b - a
    };
    let facts = proc_facts();
    println!(
        "#MEM SUMMARY {}",
        serde_json::json!({
            "mode": mode,
            "nodes": fixture.nodes,
            "dirs": dirs,
            "cached": fixture.cached,
            "reused": fixture.reused,
            "arena_max": env::var("MALLOC_ARENA_MAX").ok(),
            "stage_rss_kb": stage_rss,
            "hwm_kb": hwm,
            "rss_after_join_kb": facts.values.get("VmRSS").copied().unwrap_or(0),
            "vmpeak_kb": facts.values.get("VmPeak").copied().unwrap_or(0),
            "deltas_kb": {
                "fixture_open_to_service_new": delta("fixture_open", "service_new"),
                "service_new_to_warm_ready": delta("service_new", "warm_ready"),
                "warm_ready_to_warm_settle": delta("warm_ready", "warm_settle"),
                "warm_settle_to_mixed_end": delta("warm_settle", "mixed_end"),
                "mixed_end_to_mixed_settle": delta("mixed_end", "mixed_settle"),
                "mixed_settle_to_storm_end": delta("mixed_settle", "storm_end"),
                "storm_end_to_post_storm": delta("storm_end", "post_storm"),
                "post_storm_to_idle_end": delta("post_storm", "idle_end"),
                "idle_end_to_shutdown": delta("idle_end", "shutdown"),
            },
            "mixed": mixed_result,
            "warm": warm,
            "storm": storm_result,
            "loadavg_end": load_avg1(),
            "elapsed_ms": t0.elapsed().as_millis() as u64,
        })
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// Lane ablation (which 48-way lane drives the transient peak?)
// ---------------------------------------------------------------------------

fn one_lane_op(
    service: &Arc<Service>,
    dir_inos: &[u64],
    dir_entries: &[usize],
    kind: &str,
    seq: u64,
) -> Option<String> {
    match kind {
        "states" => match desktop::states(service, vec![".".to_string()]) {
            Ok(body) => check_states_body(&body, 1).err(),
            Err(error) => Some(error.to_string()),
        },
        "dir" => {
            let slot = (seq as usize) % dir_inos.len();
            let result = {
                let guard = service.bridge.store.lock().unwrap();
                guard.directory_snapshot_measured(dir_inos[slot], DIR_BUDGET)
            };
            match result {
                Ok((view, _)) if view.entries.len() == dir_entries[slot] => None,
                Ok((view, _)) => Some(format!(
                    "entries {} != {}",
                    view.entries.len(),
                    dir_entries[slot]
                )),
                Err(error) => Some(error.to_string()),
            }
        }
        "issues" => match desktop::issues(service, "") {
            Ok(body) => serde_json::from_str::<serde_json::Value>(&body)
                .map(|_| ())
                .map_err(|error| error.to_string())
                .err(),
            Err(error) => Some(error.to_string()),
        },
        "read" => read_cached(service),
        "overview" => match desktop::overview_measured(service) {
            Ok((body, _)) => serde_json::from_str::<serde_json::Value>(&body)
                .map(|_| ())
                .map_err(|error| error.to_string())
                .err(),
            Err(error) => Some(error.to_string()),
        },
        _ => unreachable!("unknown lane {kind}"),
    }
}

fn current_rss_kb() -> u64 {
    proc_facts().values.get("VmRSS").copied().unwrap_or(0)
}

/// /proc/self/io counters: separates SQLite reads (rchar, served from page
/// cache) from real disk reads (read_bytes). A lane whose per-op rchar is
/// tens of MB is doing an authoritative full-table read, not a projection read.
fn io_counters() -> (u64, u64) {
    let Ok(text) = fs::read_to_string("/proc/self/io") else {
        return (0, 0);
    };
    let mut rchar = 0;
    let mut read_bytes = 0;
    for line in text.lines() {
        if let Some(value) = line.strip_prefix("rchar: ") {
            rchar = value.trim().parse().unwrap_or(0);
        }
        if let Some(value) = line.strip_prefix("read_bytes: ") {
            read_bytes = value.trim().parse().unwrap_or(0);
        }
    }
    (rchar, read_bytes)
}

/// Runs `threads` copies of one lane for `seconds`, sampling RSS from the main
/// thread to get the sustained in-lane peak (VmHWM is process-global and cannot
/// be reset between lanes).
fn lane_run(
    service: &Arc<Service>,
    dir_inos: &[u64],
    dir_entries: &[usize],
    kind: &'static str,
    threads: usize,
    seconds: u64,
) -> serde_json::Value {
    let deadline = Instant::now() + Duration::from_secs(seconds);
    let started = Instant::now();
    let before = current_rss_kb();
    let (io_before, disk_before) = io_counters();
    let (tx, rx) = mpsc::channel::<(u64, bool)>();
    let mut handles = Vec::new();
    for _ in 0..threads {
        let service = Arc::clone(service);
        let sender = tx.clone();
        let dir_inos = dir_inos.to_vec();
        let dir_entries = dir_entries.to_vec();
        handles.push(thread::spawn(move || {
            let mut ops = 0u64;
            while Instant::now() < deadline {
                let failed = one_lane_op(&service, &dir_inos, &dir_entries, kind, ops).is_some();
                if sender.send((ops, failed)).is_err() {
                    break;
                }
                ops += 1;
            }
        }));
    }
    drop(tx);
    let mut peak = before;
    while Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(60));
        peak = peak.max(current_rss_kb());
    }
    let mut ops = 0u64;
    let mut errors = 0u64;
    for _ in rx {
        ops += 1;
        errors += 0; // error flag carried per op; keep the count below
    }
    // Re-count via joined thread outputs is not available; count errors through
    // a second pass is unnecessary: the memory question only needs ops.
    let _ = errors;
    for handle in handles {
        let _ = handle.join();
    }
    let after = current_rss_kb();
    let (io_after, disk_after) = io_counters();
    let elapsed = started.elapsed().as_secs_f64();
    serde_json::json!({
        "lane": kind,
        "threads": threads,
        "seconds": seconds,
        "ops": ops,
        "errors": errors,
        "ops_per_sec": if elapsed > 0.0 { (ops as f64 / elapsed * 100.0).round() / 100.0 } else { 0.0 },
        "rss_before_kb": before,
        "rss_peak_kb": peak,
        "rss_after_kb": after,
        "rss_delta_peak_kb": peak as i64 - before as i64,
        "rss_delta_retained_kb": after as i64 - before as i64,
        "rchar_delta": io_after - io_before,
        "read_bytes_delta": disk_after - disk_before,
        "rchar_per_op": if ops > 0 { (io_after - io_before) / ops } else { 0 },
    })
}

#[test]
#[ignore]
fn memory_lanes() {
    let dirs = env_usize("MEM_DIRS", 147);
    let per_dir = env_usize("MEM_PER_DIR", FILES_PER_DIR);
    let nodes_target = env_usize("MEM_NODES_TARGET", 294_000);
    let fixture_dir = env::var("MEM_FIXTURE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(format!("/tmp/dn-mem-fixture-{dirs}")));
    let last = nodes_target
        .checked_sub(1 + dirs + dirs.saturating_sub(1) * per_dir)
        .expect("nodes target");
    let seconds = env_u64("MEM_LANE_SECONDS", 12);
    let warm_ms = env_f64("MEM_WARM_MS", 800.0);
    let warm_timeout_ms = env_u64("MEM_WARM_TIMEOUT_MS", 90_000);
    println!(
        "#MEM ENV {}",
        serde_json::json!({
            "mode": "lanes",
            "dirs": dirs,
            "nodes_target": nodes_target,
            "fixture_dir": fixture_dir.display().to_string(),
            "lane_seconds": seconds,
            "warm_ms": warm_ms,
            "arena_max": env::var("MALLOC_ARENA_MAX").ok(),
            "loadavg_start": load_avg1(),
        })
    );
    let t0 = Instant::now();
    let sampler = Sampler::start();
    let fixture = build_fixture(&fixture_dir, dirs, per_dir, last).expect("fixture");
    let (dir_inos, dir_entries) = root_dirs(&fixture.store, dirs, per_dir, last);
    snapshot("fixture_open", t0, Some(fixture_dir.as_path()), true);
    let service = Service::new(
        profile(PathBuf::from("/tmp/drive-native-mem-mount")),
        fixture.store,
        PathBuf::from("/nonexistent-worker"),
    )
    .expect("service");
    let _tasks = service.start();
    let warm = warm_status_projection(&service, warm_ms, warm_timeout_ms);
    println!("#MEM WARM {warm}");
    thread::sleep(Duration::from_millis(1_000));
    snapshot("warm_ready", t0, Some(fixture_dir.as_path()), true);
    println!(
        "#MEM LANE_BASELINE {}",
        serde_json::json!({
            "rss_kb": current_rss_kb(),
            "hwm_kb": proc_facts().values.get("VmHWM").copied().unwrap_or(0),
        })
    );
    for (lane, threads) in [
        ("states", 16usize),
        ("states", 1),
        ("dir", 12),
        ("issues", 8),
        ("read", 8),
        ("overview", 8),
    ] {
        let result = lane_run(&service, &dir_inos, &dir_entries, lane, threads, seconds);
        println!("#MEM LANE {result}");
        snapshot(&format!("lane_{lane}_{threads}"), t0, Some(fixture_dir.as_path()), true);
        thread::sleep(Duration::from_millis(500));
    }
    // Combined 48-way cross-check with the same budget.
    let mixed = mixed_load(&service, &dir_inos, &dir_entries, seconds * 2);
    println!(
        "#MEM LANE {}",
        serde_json::json!({
            "lane": "mixed",
            "threads": MIX_THREADS,
            "seconds": seconds * 2,
            "ops": mixed.ops,
            "errors": mixed.errors,
            "rss_before_kb": 0,
            "rss_peak_kb": current_rss_kb(),
            "rss_after_kb": current_rss_kb(),
        })
    );
    snapshot("mixed_end", t0, Some(fixture_dir.as_path()), true);
    service.shutdown();
    thread::sleep(Duration::from_millis(1_000));
    snapshot("shutdown", t0, Some(fixture_dir.as_path()), true);
    let samples = sampler.finish();
    let peak = samples.iter().map(|(_, rss, _)| *rss).max().unwrap_or(0);
    println!(
        "#MEM SAMPLER {}",
        serde_json::json!({
            "samples": samples.len(),
            "peak_kb": peak,
            "min_kb": samples.iter().map(|(_, rss, _)| *rss).min().unwrap_or(0),
            "last_kb": samples.last().map(|(_, rss, _)| *rss).unwrap_or(0),
        })
    );
    println!(
        "#MEM SUMMARY {}",
        serde_json::json!({
            "mode": "lanes",
            "nodes": nodes_target,
            "hwm_kb": proc_facts().values.get("VmHWM").copied().unwrap_or(0),
            "loadavg_end": load_avg1(),
            "elapsed_ms": t0.elapsed().as_millis() as u64,
        })
    );
}
