//! Scoped desktop-status seeds: read-volume comparison against the full read.
//!
//! Complements `scale_perf.rs`: that test freezes the pre-fix baseline on a
//! 294k-node fixture, while this one runs only on trees that already provide
//! `Store::desktop_status_seeds_for` (contract C, db-status branch). It measures
//! the full namespace read and the scoped reads for a directory with 45
//! subdirectories + ~2.9k files (top level), a single leaf subdirectory, and a
//! batch of 50 sibling subdirectories, on one shared fixture, so the ratio is a
//! like-for-like comparison instead of a cross-fixture guess.
//!
//! Run on the merged tree:
//!
//! ```text
//! CARGO_TARGET_DIR=$HOME/dn-1516/merge/target CARGO_BUILD_JOBS=8 SCALE_RUNS=3 \
//!   cargo test -p drive-core --release --all-features --test scale_perf_scoped \
//!   -- --ignored --nocapture
//! ```
//!
//! Env: `SCALE_RUNS` (default 3), `SCALE_DIR` (fixture dir), `SCALE_KEEP=1`
//! (keep fixture for manual sqlite3 inspection). Linux-only read accounting.

use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
use std::hint::black_box;
#[cfg(target_os = "linux")]
use std::io::Read as _;
use std::path::{Path, PathBuf};
use std::time::Instant;

use drive_core::Store;
use rusqlite::{params, Connection};

// ---------------------------------------------------------------------------
// Fixture shape: same 294,033-node namespace as scale_perf.rs, but only the
// rows that desktop-status seeds read (nodes + two overlay rows + one denied
// file). Side tables (drafts/operations/transfers/cache) do not influence seed
// counts; omitting them keeps this comparison focused and fast.
// ---------------------------------------------------------------------------

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

/// The access_denied row: a pinned-subtree file, so it changes the full read
/// but never the scoped reads measured here.
const DENIED_FILE: usize = FILE_START;

/// Number of sibling leaf subdirectories in the batch scenario (a GUI status
/// batch listing 50 children of one visible directory). Must stay below
/// `SUB_DIRS`.
const BATCH_ROOTS: usize = 50;

// ---------------------------------------------------------------------------
// Deterministic ids (identical scheme to scale_perf.rs)
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

// ---------------------------------------------------------------------------
// Fixture construction
// ---------------------------------------------------------------------------

struct FixtureStats {
    nodes: usize,
    dirs: usize,
    files: usize,
    db_bytes: u64,
    build_ms: f64,
}

fn build_fixture(dir: &Path) -> Result<(Store, FixtureStats), Box<dyn std::error::Error>> {
    let started = Instant::now();
    let store = Store::open(dir)?;

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
        for idx in 0..NODES_TARGET {
            let (parent, name, directory) = node_loc(idx);
            let pinned = (PINNED_ROOT..DIR_COUNT).contains(&idx)
                || (FILE_START..REGULAR_START).contains(&idx);
            st_nodes.execute(params![
                node_id(idx),
                parent,
                name,
                "1",
                directory,
                Option::<String>::None,
                Option::<String>::None,
                Option::<String>::None,
                Option::<String>::None,
                pinned,
                false,
            ])?;
        }
        // Overlay rows on two subdirectories: seed `overlay` must flip for them,
        // and ancestor walks must see them too.
        for sub in [SUB_START, SUB_START + 1] {
            db.execute(
                "INSERT INTO namespace_overlay(node,remote) VALUES (?1,?2)",
                params![
                    node_id(sub),
                    format!("{{\"parent\":\"{}\",\"name\":\"renamed\"}}", node_id(SUB_START))
                ],
            )?;
        }
        // One denied leaf: excluded from the full read, not from the scoped ones.
        db.execute(
            "INSERT INTO access_denied(node) VALUES (?1)",
            params![node_id(DENIED_FILE)],
        )?;
        db.execute_batch("COMMIT")?;
    }
    let _ = db.query_row("PRAGMA wal_checkpoint(TRUNCATE)", [], |_| Ok(()));
    drop(db);

    let db_bytes = fs::metadata(dir.join("state.sqlite"))?.len();
    Ok((
        store,
        FixtureStats {
            nodes: NODES_TARGET,
            dirs: DIR_COUNT,
            files: FILES_TOTAL,
            db_bytes,
            build_ms: started.elapsed().as_secs_f64() * 1000.0,
        },
    ))
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

fn rchar_median(samples: &[Sample]) -> u64 {
    let mut v: Vec<u64> = samples.iter().map(|s| s.rchar).collect();
    v.sort_unstable();
    v[v.len() / 2]
}

/// Warmup + `runs` measured calls, printing machine-greppable lines.
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
        println!("SCALE_SCOPED run query={query} scenario={scenario} i={i} ms={ms:.3} rchar={rchar}");
        samples.push(Sample { ms, rchar });
        last = Some(out);
    }
    println!(
        "SCALE_SCOPED_MEDIAN query={query} scenario={scenario} runs={runs} ms_median={:.3} \
         ms_min={:.3} ms_max={:.3} rchar_median={} rchar_min={} rchar_max={}",
        median(samples.iter().map(|s| s.ms).collect()),
        samples.iter().map(|s| s.ms).fold(f64::INFINITY, f64::min),
        samples.iter().map(|s| s.ms).fold(f64::NEG_INFINITY, f64::max),
        rchar_median(&samples),
        samples.iter().map(|s| s.rchar).min().unwrap_or(0),
        samples.iter().map(|s| s.rchar).max().unwrap_or(0),
    );
    (last.expect("at least one run"), samples)
}

// ---------------------------------------------------------------------------
// Expected scope sets
// ---------------------------------------------------------------------------

fn file_subdir(ord: usize) -> usize {
    SUB_START + ord % SUB_DIRS
}

/// `roots` subtree + ancestor chain, mirroring the documented scope contract.
fn expected_scope(root_idx: usize) -> HashSet<String> {
    let mut ids = HashSet::new();
    ids.insert(node_id(ROOT));
    ids.insert(node_id(root_idx));
    let (lo, hi) = if root_idx < SUB_START {
        let lo = SUB_START + (root_idx - TOP_START) * SUB_DIRS_PER_TOP;
        (lo, lo + SUB_DIRS_PER_TOP)
    } else {
        let top = TOP_START + (root_idx - SUB_START) / SUB_DIRS_PER_TOP;
        ids.insert(node_id(top));
        (root_idx, root_idx + 1)
    };
    for sub in lo..hi {
        ids.insert(node_id(sub));
    }
    for ord in 0..REGULAR_COUNT {
        if (lo..hi).contains(&file_subdir(ord)) {
            ids.insert(node_id(REGULAR_START + ord));
        }
    }
    ids
}

fn denials() -> HashSet<String> {
    let mut set = HashSet::new();
    set.insert(node_id(DENIED_FILE));
    set
}

// ---------------------------------------------------------------------------
// Test entry point
// ---------------------------------------------------------------------------

#[test]
#[ignore = "capacity benchmark: builds a 294k-node fixture and measures scoped reads"]
fn scale_perf_scoped() {
    if let Err(err) = run() {
        panic!("scale_perf_scoped failed: {err}");
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let runs: usize = std::env::var("SCALE_RUNS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(3)
        .clamp(1, 50);

    let (dir, _tmp) = match std::env::var("SCALE_DIR") {
        Ok(p) => (PathBuf::from(p), None),
        Err(_) => {
            let tmp = tempfile::tempdir()?;
            (tmp.path().to_path_buf(), Some(tmp))
        }
    };

    let (store, stats) = build_fixture(&dir)?;
    println!(
        "SCALE_SCOPED_FIXTURE nodes={} dirs={} files={} db_bytes={} build_ms={:.0}",
        stats.nodes, stats.dirs, stats.files, stats.db_bytes, stats.build_ms
    );
    {
        // Report the schema the fixture was created with: v26 before the #15/#16
        // index migration, v27 after it. The Store::open above applies any
        // migration, so this is what the measured queries actually ran against.
        let conn = rusqlite::Connection::open(dir.join("state.sqlite"))?;
        let schema: i64 = conn.query_row("PRAGMA user_version", [], |row| row.get(0))?;
        println!("SCALE_SCOPED_SCHEMA user_version={schema}");
    }

    let (full, full_samples) = bench("desktop_status_seeds", "full", runs, || {
        store
            .desktop_status_seeds()
            .expect("desktop_status_seeds")
    });
    println!(
        "SCALE_SCOPED_ROWS query=desktop_status_seeds scenario=full rows={}",
        full.len()
    );

    let denied = denials();
    let expected_full = NODES_TARGET - denied.len();
    let mut projection: HashMap<String, (Option<String>, bool, bool, bool, bool)> = full
        .iter()
        .map(|s| {
            (
                s.id.clone(),
                (s.parent.clone(), s.directory, s.pinned, s.overlay, s.detailed),
            )
        })
        .collect();
    assert_eq!(
        full.len(),
        expected_full,
        "full read must return every node except the denied leaf"
    );
    assert!(
        !full.iter().any(|s| denied.contains(&s.id)),
        "denied node leaked into desktop_status_seeds"
    );

    let root_id = node_id(ROOT);
    let top_id = node_id(TOP_START);
    let sub_id = node_id(SUB_START);

    let (scoped_root, root_samples) = bench("desktop_status_seeds_for", "root", runs, || {
        store
            .desktop_status_seeds_for(std::slice::from_ref(&root_id))
            .expect("seeds_for root")
    });
    println!(
        "SCALE_SCOPED_ROWS query=desktop_status_seeds_for scenario=root rows={}",
        scoped_root.len()
    );
    assert_eq!(
        scoped_root.len(),
        full.len(),
        "scoped root read must cover the whole visible namespace"
    );
    let full_ids: HashSet<&str> = full.iter().map(|s| s.id.as_str()).collect();
    assert!(
        scoped_root.iter().all(|s| full_ids.contains(s.id.as_str())),
        "scoped root returned an id the full read does not"
    );

    let (scoped_top, top_samples) = bench("desktop_status_seeds_for", "top_dir_d000", runs, || {
        store
            .desktop_status_seeds_for(std::slice::from_ref(&top_id))
            .expect("seeds_for top")
    });
    let expected_top = expected_scope(TOP_START);
    println!(
        "SCALE_SCOPED_ROWS query=desktop_status_seeds_for scenario=top_dir_d000 rows={} expected={}",
        scoped_top.len(),
        expected_top.len()
    );
    assert_eq!(
        scoped_top.len(),
        expected_top.len(),
        "scoped top read returned an unexpected row count"
    );
    let got_top: HashSet<&str> = scoped_top.iter().map(|s| s.id.as_str()).collect();
    for id in &expected_top {
        assert!(
            got_top.contains(id.as_str()),
            "scoped top read missed in-scope id {id}"
        );
    }

    let (scoped_sub, sub_samples) = bench("desktop_status_seeds_for", "sub_dir_s00", runs, || {
        store
            .desktop_status_seeds_for(std::slice::from_ref(&sub_id))
            .expect("seeds_for sub")
    });
    let expected_sub = expected_scope(SUB_START);
    println!(
        "SCALE_SCOPED_ROWS query=desktop_status_seeds_for scenario=sub_dir_s00 rows={} expected={}",
        scoped_sub.len(),
        expected_sub.len()
    );
    assert_eq!(
        scoped_sub.len(),
        expected_sub.len(),
        "scoped sub read returned an unexpected row count"
    );
    let got_sub: HashSet<&str> = scoped_sub.iter().map(|s| s.id.as_str()).collect();
    for id in &expected_sub {
        assert!(
            got_sub.contains(id.as_str()),
            "scoped sub read missed in-scope id {id}"
        );
    }

    // Batch of 50 sibling leaf subdirectories: the shape an interactive status
    // batch asks for when it lists 50 children of one visible directory at once.
    let batch_roots: Vec<String> = (0..BATCH_ROOTS)
        .map(|i| node_id(SUB_START + i))
        .collect();
    let (scoped_batch, batch_samples) =
        bench("desktop_status_seeds_for", "batch_50_subdirs", runs, || {
            store
                .desktop_status_seeds_for(&batch_roots)
                .expect("seeds_for batch")
        });
    let mut expected_batch = HashSet::new();
    for i in 0..BATCH_ROOTS {
        expected_batch.extend(expected_scope(SUB_START + i));
    }
    println!(
        "SCALE_SCOPED_ROWS query=desktop_status_seeds_for scenario=batch_50_subdirs rows={} expected={}",
        scoped_batch.len(),
        expected_batch.len()
    );
    assert_eq!(
        scoped_batch.len(),
        expected_batch.len(),
        "scoped batch read returned an unexpected row count"
    );
    let got_batch: HashSet<&str> = scoped_batch.iter().map(|s| s.id.as_str()).collect();
    for id in &expected_batch {
        assert!(
            got_batch.contains(id.as_str()),
            "scoped batch read missed in-scope id {id}"
        );
    }

    // The GUI root page: every visible child of the namespace root in one batch
    // (the #67 "147 top-level directories" shape, at this fixture's width). The
    // scoped walk degenerates to one subtree walk per child; the batch expands
    // to the whole namespace, so it must use the linear full read.
    let mut all_root_children: Vec<String> =
        (0..TOP_DIRS).map(|i| node_id(TOP_START + i)).collect();
    all_root_children.push(node_id(PINNED_ROOT));
    let (scoped_all, all_samples) = bench(
        "desktop_status_seeds_for",
        "batch_all_root_children",
        runs,
        || {
            store
                .desktop_status_seeds_for(&all_root_children)
                .expect("seeds_for all root children")
        },
    );
    println!(
        "SCALE_SCOPED_ROWS query=desktop_status_seeds_for scenario=batch_all_root_children rows={} expected={}",
        scoped_all.len(),
        full.len()
    );
    assert_eq!(
        scoped_all.len(),
        full.len(),
        "the all-children root page batch must cover the whole visible namespace"
    );
    let got_all: HashSet<&str> = scoped_all.iter().map(|s| s.id.as_str()).collect();
    assert_eq!(
        got_all, full_ids,
        "the all-children root page batch must equal the full projection"
    );
    assert!(
        rchar_median(&all_samples) <= rchar_median(&full_samples) * 3 / 2,
        "the all-children batch must use the linear full read, rchar {} vs full {}",
        rchar_median(&all_samples),
        rchar_median(&full_samples)
    );


    // Projection equivalence: every scoped row must carry exactly the flags the
    // full read reports for the same id (inherited pin/overlay/detail rules).
    for row in scoped_root
        .iter()
        .chain(scoped_top.iter())
        .chain(scoped_sub.iter())
        .chain(scoped_batch.iter())
        .chain(scoped_all.iter())
    {
        let want = projection
            .get(&row.id)
            .unwrap_or_else(|| panic!("scoped row {} missing from full read", row.id));
        assert_eq!(
            (
                row.parent.as_deref(),
                row.directory,
                row.pinned,
                row.overlay,
                row.detailed,
            ),
            (want.0.as_deref(), want.1, want.2, want.3, want.4),
            "projection mismatch for {}",
            row.id
        );
    }
    projection.clear();

    let empty = store.desktop_status_seeds_for(&[])?;
    println!(
        "SCALE_SCOPED_ROWS query=desktop_status_seeds_for scenario=no_roots rows={}",
        empty.len()
    );
    assert!(empty.is_empty(), "empty roots must not read seeds");

    let full_ms = median(full_samples.iter().map(|s| s.ms).collect());
    let top_ms = median(top_samples.iter().map(|s| s.ms).collect());
    let sub_ms = median(sub_samples.iter().map(|s| s.ms).collect());
    let batch_ms = median(batch_samples.iter().map(|s| s.ms).collect());
    let full_rchar = rchar_median(&full_samples);
    let top_rchar = rchar_median(&top_samples);
    let sub_rchar = rchar_median(&sub_samples);
    let batch_rchar = rchar_median(&batch_samples);
    println!(
        "SCALE_SCOPED_SUMMARY full_ms={full_ms:.3} full_rchar={full_rchar} \
         root_ms={:.3} root_rchar={} top_ms={top_ms:.3} top_rchar={top_rchar} \
         sub_ms={sub_ms:.3} sub_rchar={sub_rchar} \
         batch_ms={batch_ms:.3} batch_rchar={batch_rchar} \
         top_rchar_ratio={:.1} sub_rchar_ratio={:.1} batch_rchar_ratio={:.1} \
         top_rows={} sub_rows={} batch_rows={}",
        median(root_samples.iter().map(|s| s.ms).collect()),
        rchar_median(&root_samples),
        full_rchar as f64 / top_rchar.max(1) as f64,
        full_rchar as f64 / sub_rchar.max(1) as f64,
        full_rchar as f64 / batch_rchar.max(1) as f64,
        scoped_top.len(),
        scoped_sub.len(),
        scoped_batch.len(),
    );

    drop(store);
    if std::env::var("SCALE_KEEP").as_deref() != Ok("1") {
        let _ = fs::remove_dir_all(&dir);
    }
    Ok(())
}

