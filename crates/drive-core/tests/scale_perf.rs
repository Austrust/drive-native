//! Synthetic scale benchmark for the 294k-node library reported in issue #15/#16.
//!
//! Builds a SQLite state store that mimics a large Synology Drive mount:
//! ~294,000 nodes (a few hundred cached files, a small pinned subtree, a
//! handful of dirty drafts, uncertain operations, namespace operations,
//! transfers, and ~370 conflicted-or-pending nodes), then measures wall clock
//! and `/proc/self/io` `rchar` deltas for the read paths that were reported to
//! block the desktop status / GetIssues flows.
//!
//! Fixture rows are written through a second `rusqlite` connection so building
//! the fixture is not part of any measured window. Metrics are Linux-only; on
//! other platforms `rchar` is reported as 0.
//!
//! ```text
//! CARGO_TARGET_DIR=... cargo test -p drive-core --release --all-features \
//!     --test scale_perf -- --ignored --nocapture
//! ```
//!
//! Environment knobs:
//! * `SCALE_RUNS` - measured repetitions per scenario (default 3, warmup extra)
//! * `SCALE_CACHE` - number of cached files (default 400)
//! * `SCALE_KEEP=1` - keep the fixture directory for manual inspection
//! * `SCALE_DIR` - build the fixture in this directory instead of a tempdir
//!
//! Output lines (`SCALE_*`) are stable enough to diff a baseline run against a
//! fixed run: one `SCALE run ...` line per measured repetition and one
//! `SCALE_MEDIAN ...` line per scenario.

use std::collections::HashSet;
use std::fs::{self, File};
use std::hint::black_box;
#[cfg(target_os = "linux")]
use std::io::Read as _;
use std::path::{Path, PathBuf};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use drive_core::Store;
use rusqlite::{params, Connection};

// ---------------------------------------------------------------------------
// Synthetic tree shape
// ---------------------------------------------------------------------------

/// Total node count; matches the library size from issue #15/#16.
const NODES_TARGET: usize = 294_033;
const TOP_DIRS: usize = 100;
const SUB_DIRS_PER_TOP: usize = 45;
const SUB_DIRS: usize = TOP_DIRS * SUB_DIRS_PER_TOP;
const PINNED_SUBS: usize = 10;
const PINNED_FILES: usize = 180;

const ROOT: usize = 0;
const TOP_START: usize = ROOT + 1;
const SUB_START: usize = TOP_START + TOP_DIRS;
const PINNED_ROOT: usize = SUB_START + SUB_DIRS;
const PINNED_SUB_START: usize = PINNED_ROOT + 1;
const DIR_COUNT: usize = PINNED_SUB_START + PINNED_SUBS;
const FILE_START: usize = DIR_COUNT;
const REGULAR_START: usize = FILE_START + PINNED_FILES;
const REGULAR_COUNT: usize = NODES_TARGET - REGULAR_START;
const FILES_TOTAL: usize = PINNED_FILES + REGULAR_COUNT;

// Per-regular-file fixture category, stored per ordinal in `0..REGULAR_COUNT`.
const CAT_PLAIN: u8 = 0;
const CAT_CACHED: u8 = 1;
const CAT_STALE: u8 = 2;
const CAT_CONFLICT: u8 = 3;
const CAT_DRAFT_CONFLICT: u8 = 4;
const CAT_DRAFT_DIRTY: u8 = 5;
const CAT_OP_UNCERTAIN: u8 = 6;
const CAT_NS_ABNORMAL: u8 = 7;
const CAT_DOWNLOAD_INTENT: u8 = 8;
const CAT_CACHED_PINNED: u8 = 9;
const CAT_CACHED_DRAFT: u8 = 10;
const CAT_CACHED_OP_BLOB: u8 = 11;
const CAT_CACHED_OVERLAY: u8 = 12;
const CAT_CACHED_REPLACE: u8 = 13;
const CAT_CACHED_RECENT: u8 = 14;
const CAT_COUNT: usize = 15;

const N_STALE: usize = 150;
const N_CONFLICT: usize = 300;
const N_DRAFT_CONFLICT: usize = 20;
const N_DRAFT_DIRTY: usize = 20;
const N_OP_UNCERTAIN: usize = 30;
const N_NS_ABNORMAL: usize = 20;
const N_DOWNLOAD_INTENT: usize = 30;
const N_CACHED_PINNED: usize = 5;
const N_CACHED_DRAFT: usize = 5;
const N_CACHED_OP_BLOB: usize = 5;
const N_CACHED_OVERLAY: usize = 2;
const N_CACHED_REPLACE: usize = 2;
const N_CACHED_RECENT: usize = 3;
const N_CACHED_SPECIAL: usize = N_CACHED_PINNED
    + N_CACHED_DRAFT
    + N_CACHED_OP_BLOB
    + N_CACHED_OVERLAY
    + N_CACHED_REPLACE
    + N_CACHED_RECENT;

// `sync_issues` sees one entry per conflicted-or-pending node.
const EXPECTED_ISSUES: usize = N_CONFLICT + N_DRAFT_CONFLICT + N_OP_UNCERTAIN + N_NS_ABNORMAL;
const EXPECTED_PAGE1: usize = 256;
const EXPECTED_PAGE2: usize = EXPECTED_ISSUES - EXPECTED_PAGE1;

// `downloads_needed`: pinned uncached files + stale cached files minus cache
// releases + download intents.
const EXPECTED_DOWNLOADS: usize = PINNED_FILES + N_STALE - 10 + N_DOWNLOAD_INTENT;

// ---------------------------------------------------------------------------
// Deterministic ids
// ---------------------------------------------------------------------------

fn mix64(mut z: u64) -> u64 {
    z = z.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// Stable 40-char id with UUID-ish entropy, so the `nodes` table has a
/// realistic on-disk footprint instead of trivially compressible short keys.
fn node_id(idx: usize) -> String {
    if idx == ROOT {
        return "root".to_string();
    }
    format!("{:016x}{:024x}", mix64(idx as u64), idx)
}

fn blob_of(idx: usize) -> String {
    format!("blob{:036x}", idx)
}

fn dirty_blob_of(idx: usize) -> String {
    format!("dirty{:035x}", idx)
}

fn content_size(idx: usize) -> u64 {
    65_536 + ((idx as u64).wrapping_mul(7_919) % 4_000_000)
}

fn node_loc(idx: usize) -> (Option<String>, String, bool) {
    if idx == ROOT {
        return (None, "Drive".to_string(), true);
    }
    if idx < SUB_START {
        return (Some(node_id(ROOT)), format!("d{:03}", idx - TOP_START), true);
    }
    if idx < PINNED_ROOT {
        let k = idx - SUB_START;
        return (
            Some(node_id(TOP_START + k / SUB_DIRS_PER_TOP)),
            format!("s{:02}", k % SUB_DIRS_PER_TOP),
            true,
        );
    }
    if idx == PINNED_ROOT {
        return (Some(node_id(ROOT)), "pinned".to_string(), true);
    }
    if idx < DIR_COUNT {
        return (
            Some(node_id(PINNED_ROOT)),
            format!("p{:02}", idx - PINNED_SUB_START),
            true,
        );
    }
    if idx < REGULAR_START {
        let k = idx - FILE_START;
        return (
            Some(node_id(PINNED_SUB_START + k % PINNED_SUBS)),
            format!("p{:04}.dat", k),
            false,
        );
    }
    let ord = idx - REGULAR_START;
    (
        Some(node_id(SUB_START + ord % SUB_DIRS)),
        format!("f{:06}.dat", ord),
        false,
    )
}

fn pick_ords(
    regular: usize,
    count: usize,
    used: &mut HashSet<usize>,
    offset: usize,
) -> Vec<usize> {
    if count == 0 {
        return Vec::new();
    }
    let stride = (regular / count).max(1);
    let mut out = Vec::with_capacity(count);
    let mut k = 0usize;
    while out.len() < count {
        let ord = (offset + k * stride) % regular;
        k += 1;
        if used.insert(ord) {
            out.push(ord);
        }
        assert!(k <= regular * 2, "pick_ords could not find disjoint slots");
    }
    out
}

fn classify(regular: usize, cached: usize) -> Vec<u8> {
    let mut cat = vec![CAT_PLAIN; regular];
    let mut used: HashSet<usize> = HashSet::new();

    let cached_ords = pick_ords(regular, cached, &mut used, 0);
    for (k, &ord) in cached_ords.iter().enumerate() {
        cat[ord] = match k {
            i if i < N_CACHED_PINNED => CAT_CACHED_PINNED,
            i if i < N_CACHED_PINNED + N_CACHED_DRAFT => CAT_CACHED_DRAFT,
            i if i < N_CACHED_PINNED + N_CACHED_DRAFT + N_CACHED_OP_BLOB => CAT_CACHED_OP_BLOB,
            i if i < N_CACHED_PINNED + N_CACHED_DRAFT + N_CACHED_OP_BLOB + N_CACHED_OVERLAY => {
                CAT_CACHED_OVERLAY
            }
            i if i
                < N_CACHED_PINNED
                    + N_CACHED_DRAFT
                    + N_CACHED_OP_BLOB
                    + N_CACHED_OVERLAY
                    + N_CACHED_REPLACE =>
            {
                CAT_CACHED_REPLACE
            }
            i if i
                < N_CACHED_PINNED
                    + N_CACHED_DRAFT
                    + N_CACHED_OP_BLOB
                    + N_CACHED_OVERLAY
                    + N_CACHED_REPLACE
                    + N_CACHED_RECENT =>
            {
                CAT_CACHED_RECENT
            }
            _ => CAT_CACHED,
        };
    }

    for (kind, count, offset) in [
        (CAT_STALE, N_STALE, 104_729usize),
        (CAT_CONFLICT, N_CONFLICT, 211_111),
        (CAT_DRAFT_CONFLICT, N_DRAFT_CONFLICT, 311_111),
        (CAT_DRAFT_DIRTY, N_DRAFT_DIRTY, 411_111),
        (CAT_OP_UNCERTAIN, N_OP_UNCERTAIN, 511_111),
        (CAT_NS_ABNORMAL, N_NS_ABNORMAL, 611_111),
        (CAT_DOWNLOAD_INTENT, N_DOWNLOAD_INTENT, 711_111),
    ] {
        for ord in pick_ords(regular, count, &mut used, offset) {
            cat[ord] = kind;
        }
    }
    assert_eq!(
        cat.iter().filter(|c| **c != CAT_PLAIN).count(),
        cached + EXPECTED_ISSUES + N_DRAFT_DIRTY + N_STALE + N_DOWNLOAD_INTENT
    );
    cat
}

// ---------------------------------------------------------------------------
// Fixture construction
// ---------------------------------------------------------------------------

struct FixtureStats {
    nodes: usize,
    dirs: usize,
    files: usize,
    cached: usize,
    db_bytes: u64,
    build_ms: f64,
}

fn build_fixture(
    dir: &Path,
    cached: usize,
) -> Result<(Store, FixtureStats), Box<dyn std::error::Error>> {
    let started = Instant::now();
    let store = Store::open(dir)?;
    fs::create_dir_all(dir.join("drafts"))?;

    let regular = classify(REGULAR_COUNT, cached);
    let mut groups: Vec<Vec<usize>> = (0..CAT_COUNT).map(|_| Vec::new()).collect();
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

    let db = Connection::open(dir.join("state.sqlite"))?;
    db.execute_batch(
        "PRAGMA synchronous=OFF;
         PRAGMA journal_mode=WAL;
         PRAGMA cache_size=-65536;
         PRAGMA temp_store=MEMORY;
         BEGIN IMMEDIATE",
    )?;

    {
        let mut st_nodes = db.prepare(
            "INSERT INTO nodes(id,parent,name,remote_version,directory,local_version,blob,\
             dirty_blob,base_version,pinned,deleted,conflict) \
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,0,?11)",
        )?;
        let mut st_meta =
            db.prepare("INSERT INTO node_metadata(node,size,mtime,mode) VALUES (?1,?2,?3,?4)")?;

        for idx in 0..NODES_TARGET {
            let (parent, name, directory) = node_loc(idx);
            let mut remote = "1".to_string();
            let mut local: Option<String> = None;
            let mut blob: Option<String> = None;
            let mut dirty_blob: Option<String> = None;
            let mut base: Option<String> = None;
            let mut pinned = false;
            let mut conflict = false;

            if !directory {
                let cat = if idx < REGULAR_START {
                    CAT_PLAIN
                } else {
                    regular[idx - REGULAR_START]
                };
                groups[cat as usize].push(idx);
                match cat {
                    CAT_CACHED | CAT_CACHED_RECENT => {
                        blob = Some(blob_of(idx));
                        local = Some("1".to_string());
                    }
                    CAT_CACHED_PINNED => {
                        blob = Some(blob_of(idx));
                        local = Some("1".to_string());
                        pinned = true;
                    }
                    CAT_CACHED_DRAFT
                    | CAT_CACHED_OP_BLOB
                    | CAT_CACHED_OVERLAY
                    | CAT_CACHED_REPLACE => {
                        blob = Some(blob_of(idx));
                        local = Some("1".to_string());
                    }
                    CAT_STALE => {
                        blob = Some(blob_of(idx));
                        local = Some("1".to_string());
                        remote = "2".to_string();
                    }
                    CAT_CONFLICT => {
                        blob = Some(blob_of(idx));
                        dirty_blob = Some(dirty_blob_of(idx));
                        local = Some("1".to_string());
                        remote = "2".to_string();
                        base = Some("1".to_string());
                        conflict = true;
                    }
                    CAT_DRAFT_CONFLICT => {
                        remote = "2".to_string();
                        base = Some("1".to_string());
                    }
                    CAT_DRAFT_DIRTY => {
                        base = Some("1".to_string());
                    }
                    _ => {}
                }
            } else if idx == PINNED_ROOT || (PINNED_SUB_START..DIR_COUNT).contains(&idx) {
                pinned = true;
            } else if idx >= FILE_START && idx < REGULAR_START {
                pinned = true;
            }

            st_nodes.execute(params![
                node_id(idx),
                parent,
                name,
                remote,
                directory,
                local,
                blob,
                dirty_blob,
                base,
                pinned,
                conflict,
            ])?;
            st_meta.execute(params![
                node_id(idx),
                content_size(idx) as i64,
                1_700_000_000i64 + (idx as i64 % 1_000_000),
                if directory { 0o40_755i64 } else { 0o100_644i64 },
            ])?;
        }
    }

    // ---- side tables -------------------------------------------------------
    {
        let mut st = db.prepare(
            "INSERT INTO drafts(node,token,base,generation,checkpoint,dirty) \
             VALUES (?1,?2,?3,?4,?5,?6)",
        )?;
        for (n, &idx) in groups[CAT_CACHED_DRAFT as usize].iter().enumerate() {
            st.execute(params![node_id(idx), format!("tok{:06}", n), "1", 1i64, 0i64, 1i64])?;
        }
        for (n, &idx) in groups[CAT_DRAFT_CONFLICT as usize].iter().enumerate() {
            st.execute(params![node_id(idx), format!("tdc{:06}", n), "1", 1i64, 0i64, 1i64])?;
        }
        for (n, &idx) in groups[CAT_DRAFT_DIRTY as usize].iter().enumerate() {
            st.execute(params![node_id(idx), format!("tdd{:06}", n), "1", 1i64, 0i64, 1i64])?;
        }
    }
    {
        let mut st = db.prepare(
            "INSERT INTO operations(id,node,kind,base,blob,generation,state,\
             reconcile_attempts,reconcile_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)",
        )?;
        for n in 0..2900usize {
            let idx = REGULAR_START + (n * 1009) % REGULAR_COUNT;
            st.execute(params![
                format!("op{:06}", n),
                node_id(idx),
                "upload",
                "1",
                Option::<String>::None,
                1i64,
                "committed",
                0i64,
                0i64,
            ])?;
        }
        for (n, &idx) in groups[CAT_OP_UNCERTAIN as usize].iter().enumerate() {
            st.execute(params![
                format!("unc{:06}", n),
                node_id(idx),
                "upload",
                "1",
                Option::<String>::None,
                1i64,
                "uncertain",
                3i64,
                (now - 60) as i64,
            ])?;
        }
        for (n, &idx) in groups[CAT_CACHED_OP_BLOB as usize].iter().enumerate() {
            st.execute(params![
                format!("opb{:06}", n),
                node_id(idx),
                "upload",
                "1",
                Some(blob_of(idx)),
                1i64,
                "pending",
                0i64,
                0i64,
            ])?;
        }
        for n in 0..20usize {
            let idx = REGULAR_START + (n * 7919) % REGULAR_COUNT;
            st.execute(params![
                format!("opp{:06}", n),
                node_id(idx),
                "upload",
                "1",
                Some(blob_of(idx)),
                1i64,
                "pending",
                0i64,
                0i64,
            ])?;
        }
    }
    {
        let mut st = db.prepare(
            "INSERT INTO transfers(id,node,version,direction,state,error) \
             VALUES (?1,?2,?3,?4,?5,NULL)",
        )?;
        for n in 0..150usize {
            let idx = REGULAR_START + (n * 1543) % REGULAR_COUNT;
            st.execute(params![
                format!("tup{:06}", n),
                node_id(idx),
                "1",
                "upload",
                "committed",
            ])?;
        }
        let states = ["pending", "inflight", "retry", "failed"];
        for (n, &idx) in groups[CAT_DOWNLOAD_INTENT as usize].iter().enumerate() {
            st.execute(params![
                format!("tdl{:06}", n),
                node_id(idx),
                "1",
                "download",
                states[n % states.len()],
            ])?;
        }
    }
    {
        let mut st = db.prepare(
            "INSERT INTO namespace_ops(id,node,body,state,kind) VALUES (?1,?2,?3,?4,?5)",
        )?;
        for n in 0..500usize {
            let idx = REGULAR_START + (n * 2749) % REGULAR_COUNT;
            st.execute(params![format!("ns{:06}", n), node_id(idx), "{}", "committed", "rename"])?;
        }
        for (n, &idx) in groups[CAT_NS_ABNORMAL as usize].iter().enumerate() {
            let state = if n < N_NS_ABNORMAL / 2 {
                "uncertain"
            } else {
                "cleanup_uncertain"
            };
            st.execute(params![format!("nsw{:06}", n), node_id(idx), "{}", state, "rename"])?;
        }
        for (n, &idx) in groups[CAT_CACHED_REPLACE as usize].iter().enumerate() {
            let body = format!("{{\"blob\":\"{}\"}}", blob_of(idx));
            st.execute(params![format!("nsr{:06}", n), node_id(idx), body, "pending", "replace"])?;
        }
    }
    {
        let mut st = db.prepare("INSERT INTO namespace_overlay(node,remote) VALUES (?1,'{}')")?;
        for &idx in &groups[CAT_CACHED_OVERLAY as usize] {
            st.execute(params![node_id(idx)])?;
        }
        for sub in [1234usize, 3456] {
            st.execute(params![node_id(SUB_START + sub % SUB_DIRS)])?;
        }
    }
    {
        let mut st = db.prepare("INSERT INTO cache_access(node) VALUES (?1)")?;
        for &idx in groups[CAT_CACHED as usize].iter().take(50) {
            st.execute(params![node_id(idx)])?;
        }
        let mut st = db.prepare("INSERT INTO cache_published(node,at) VALUES (?1,?2)")?;
        for &idx in &groups[CAT_CACHED_RECENT as usize] {
            st.execute(params![node_id(idx), (now - 1) as i64])?;
        }
        let mut st = db.prepare(
            "INSERT INTO cache_releases(node,version,state,attempts,retry_at) \
             VALUES (?1,'2','pending',0,0)",
        )?;
        for &idx in groups[CAT_STALE as usize].iter().take(10) {
            st.execute(params![node_id(idx)])?;
        }
    }

    db.execute_batch("COMMIT")?;
    let _ = db.query_row("PRAGMA wal_checkpoint(TRUNCATE)", [], |_| Ok(()));

    // On-disk content: sparse blob files plus tiny draft checkpoints, so
    // `cache_usage`'s directory walk sees realistic sizes.
    let blobs = dir.join("blobs");
    fs::create_dir_all(&blobs)?;
    for cat in [
        CAT_CACHED,
        CAT_CACHED_PINNED,
        CAT_CACHED_DRAFT,
        CAT_CACHED_OP_BLOB,
        CAT_CACHED_OVERLAY,
        CAT_CACHED_REPLACE,
        CAT_CACHED_RECENT,
        CAT_STALE,
        CAT_CONFLICT,
    ] {
        for &idx in &groups[cat as usize] {
            let f = File::create(blobs.join(blob_of(idx)))?;
            f.set_len(content_size(idx))?;
        }
    }
    let drafts_dir = dir.join("drafts");
    for n in 0..45usize {
        let f = File::create(drafts_dir.join(format!("draft{:04}", n)))?;
        f.set_len(1_024 + (n as u64) * 131)?;
    }

    let db_bytes = fs::metadata(dir.join("state.sqlite"))?.len()
        + fs::metadata(dir.join("state.sqlite-wal")).map_or(0, |m| m.len());
    let stats = FixtureStats {
        nodes: NODES_TARGET,
        dirs: DIR_COUNT,
        files: FILES_TOTAL,
        cached,
        db_bytes,
        build_ms: started.elapsed().as_secs_f64() * 1000.0,
    };
    Ok((store, stats))
}

// ---------------------------------------------------------------------------
// Measurement
// ---------------------------------------------------------------------------

#[cfg(target_os = "linux")]
fn io_rchar() -> Option<u64> {
    let mut text = String::new();
    File::open("/proc/self/io").ok()?.read_to_string(&mut text).ok()?;
    text.lines()
        .find_map(|line| line.strip_prefix("rchar:")?.trim().parse().ok())
}

#[cfg(not(target_os = "linux"))]
fn io_rchar() -> Option<u64> {
    None
}

#[derive(Clone, Copy)]
struct Sample {
    ms: f64,
    rchar: u64,
}

fn median(mut values: Vec<f64>) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let mid = values.len() / 2;
    if values.len() % 2 == 1 {
        values[mid]
    } else {
        (values[mid - 1] + values[mid]) / 2.0
    }
}

/// Runs `op` once as warmup, then `runs` times measuring wall clock and the
/// process-wide `rchar` delta. Returns the last output plus the samples.
fn bench<T>(
    query: &str,
    scenario: &str,
    runs: usize,
    mut op: impl FnMut() -> T,
) -> (T, Vec<Sample>) {
    let warm = op();
    black_box(&warm);
    drop(warm);

    let mut samples = Vec::with_capacity(runs);
    let mut last = None;
    for i in 0..runs {
        let before = io_rchar();
        let started = Instant::now();
        let out = op();
        let ms = started.elapsed().as_secs_f64() * 1000.0;
        let rchar = match (before, io_rchar()) {
            (Some(a), Some(b)) => b.saturating_sub(a),
            _ => 0,
        };
        black_box(&out);
        println!("SCALE run query={query} scenario={scenario} i={i} ms={ms:.3} rchar={rchar}");
        samples.push(Sample { ms, rchar });
        last = Some(out);
    }
    println!(
        "SCALE_MEDIAN query={query} scenario={scenario} runs={runs} ms_median={:.3} \
         ms_min={:.3} ms_max={:.3} rchar_median={} rchar_min={} rchar_max={}",
        median(samples.iter().map(|s| s.ms).collect()),
        samples.iter().map(|s| s.ms).fold(f64::INFINITY, f64::min),
        samples.iter().map(|s| s.ms).fold(f64::NEG_INFINITY, f64::max),
        {
            let mut v: Vec<u64> = samples.iter().map(|s| s.rchar).collect();
            v.sort_unstable();
            v[v.len() / 2]
        },
        samples.iter().map(|s| s.rchar).min().unwrap_or(0),
        samples.iter().map(|s| s.rchar).max().unwrap_or(0),
    );
    (last.expect("at least one run"), samples)
}

// ---------------------------------------------------------------------------
// Test entry point
// ---------------------------------------------------------------------------

#[test]
#[ignore = "capacity benchmark: builds a 294k-node fixture and takes minutes"]
fn scale_perf() {
    if let Err(err) = run() {
        panic!("scale_perf failed: {err}");
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let runs: usize = std::env::var("SCALE_RUNS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(3)
        .clamp(1, 50);
    let cached: usize = std::env::var("SCALE_CACHE")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(400)
        .max(N_CACHED_SPECIAL);

    let (dir, _tmp) = match std::env::var("SCALE_DIR") {
        Ok(p) => (PathBuf::from(p), None),
        Err(_) => {
            let tmp = tempfile::tempdir()?;
            (tmp.path().to_path_buf(), Some(tmp))
        }
    };

    let (mut store, stats) = build_fixture(&dir, cached)?;
    println!(
        "SCALE_FIXTURE nodes={} dirs={} files={} cached={} db_bytes={} build_ms={:.0}",
        stats.nodes, stats.dirs, stats.files, stats.cached, stats.db_bytes, stats.build_ms
    );

    store.set_cache_limit(1 << 30)?;
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let active: HashSet<String> = HashSet::new();

    let (seeds, _) = bench("desktop_status_seeds", "full", runs, || {
        store.desktop_status_seeds().expect("desktop_status_seeds")
    });
    println!("SCALE_SEEDS count={}", seeds.len());
    assert!(
        seeds.len() >= NODES_TARGET - 1_000,
        "desktop_status_seeds returned {} seeds, expected ~{}",
        seeds.len(),
        NODES_TARGET
    );
    drop(seeds);

    let (usage, _) = bench("cache_usage", "limit_1gib", runs, || {
        store.cache_usage(now, &active).expect("cache_usage")
    });
    println!(
        "SCALE_CACHE_USAGE limit={} content={} reclaimable={} protected={} over_target={}",
        usage.limit_bytes,
        usage.content_bytes,
        usage.reclaimable_bytes,
        usage.protected_bytes,
        usage.over_target_bytes
    );

    let (needed, _) = bench("downloads_needed", "pinned_stale_intent", runs, || {
        store.downloads_needed().expect("downloads_needed")
    });
    println!(
        "SCALE_DOWNLOADS needed={} expected={}",
        needed.len(),
        EXPECTED_DOWNLOADS
    );
    assert!(
        needed.len() >= EXPECTED_DOWNLOADS - 10 && needed.len() <= EXPECTED_DOWNLOADS,
        "downloads_needed returned {}, expected ~{}",
        needed.len(),
        EXPECTED_DOWNLOADS
    );
    drop(needed);

    let (page1, _) = bench("sync_issues", "issues_page1", runs, || {
        store.sync_issues("root", "").expect("sync_issues page1")
    });
    println!(
        "SCALE_ISSUES page1_entries={} next={:?}",
        page1.entries.len(),
        page1.next
    );
    assert_eq!(page1.entries.len(), EXPECTED_PAGE1);
    let cursor = page1.next.clone().expect("page1 cursor");

    let (page2, _) = bench("sync_issues", "issues_page2", runs, || {
        store
            .sync_issues("root", &cursor)
            .expect("sync_issues page2")
    });
    println!(
        "SCALE_ISSUES page2_entries={} next={:?} total={}",
        page2.entries.len(),
        page2.next,
        page1.entries.len() + page2.entries.len()
    );
    assert_eq!(page2.entries.len(), EXPECTED_PAGE2);
    assert!(page2.next.is_none());

    // Neutralise every issue source, then the same page must come back empty:
    // this is the "no conflicts" steady state the desktop polls most of the time.
    let db = Connection::open(dir.join("state.sqlite"))?;
    db.execute_batch(
        "UPDATE nodes SET conflict=0;
         DELETE FROM drafts WHERE node IN (
             SELECT d.node FROM drafts d JOIN nodes n ON n.id=d.node
             WHERE d.base != n.remote_version);
         UPDATE operations SET state='committed' WHERE state IN ('inflight','uncertain');
         UPDATE namespace_ops SET state='committed'
             WHERE state IN ('uncertain','cleanup_uncertain');",
    )?;
    let _ = db.query_row("PRAGMA wal_checkpoint(TRUNCATE)", [], |_| Ok(()));

    let (page0, _) = bench("sync_issues", "no_issues", runs, || {
        store.sync_issues("root", "").expect("sync_issues zero")
    });
    println!(
        "SCALE_ISSUES zero_entries={} next={:?}",
        page0.entries.len(),
        page0.next
    );
    assert!(page0.entries.is_empty());
    assert!(page0.next.is_none());

    drop(db);
    drop(store);
    if std::env::var("SCALE_KEEP").as_deref() != Ok("1") {
        let _ = fs::remove_dir_all(&dir);
    }
    Ok(())
}
