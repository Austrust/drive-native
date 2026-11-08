//! Issue #22 acceptance harness: namespace semantic watermark.
//!
//! Evidence produced by this harness:
//!
//! 1. A read-only open (`note_cache_use`, the only bookkeeping the FUSE open
//!    path performs) commits no WAL frame: `PRAGMA data_version` of an observer
//!    connection and `state.sqlite-wal` size stay unchanged.
//! 2. After any number of read-only opens the namespace projection is still
//!    current: `ensure_fresh` returns `Fresh` in microseconds and rebuilds
//!    nothing. The old path bumped `total_changes` on every open and paid one
//!    full index rebuild right after it (reported here as `rebuild_ms`).
//! 3. A full rebuild of the projection succeeds inside the 1 s build budget at
//!    the node counts from the performance review (100k / 319,679 / 369,298).
//! 4. A mixed workload keeps read-only directory opens at microsecond latency:
//!    only semantic writes rebuild, and each rebuild is bounded.
//!
//! Run on the Linux builder with the release profile (the synthetic fixtures
//! are large and the production budget is a release-profile budget):
//!
//! ```text
//! CARGO_TARGET_DIR=/home/caozijian/p032/targets/t2 \
//!   cargo test --release -p drive-core --test namespace_index_budget -- --ignored --nocapture
//! ```
//!
//! Env:
//! * `NSI_FIXTURE` - reuse an existing Store directory (e.g. `fixture-210k`)
//! * `NSI_SIZES` - comma separated synthetic node counts
//!   (default `100000,319679,369298`)
//! * `NSI_RUNS` - read-only open/opendir iterations (default 200)
//! * `NSI_BUDGET_MS` - rebuild budget asserted here (default 1000)
//! * `NSI_SKIP_SYNTHETIC=1` - only measure `NSI_FIXTURE`

use std::{
    env,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use drive_core::{
    namespace_index::{InvalidateHint, NamespaceIndex, RefreshOutcome},
    Node, NodeMetadata, Store,
};

fn node(id: String, parent: Option<String>, name: String, directory: bool) -> Node {
    Node {
        id,
        parent,
        name,
        version: "fixture-v1".into(),
        directory,
        metadata: Some(NodeMetadata {
            size: if directory { 0 } else { 3 },
            mtime: 1_700_000_000,
            mode: if directory { 0o755 } else { 0o644 },
        }),
    }
}

fn synthetic_fixture(dir: &Path, total: usize) -> Result<Store, Box<dyn std::error::Error>> {
    let directories = 256usize;
    let files = total.saturating_sub(directories + 1);
    let mut nodes = Vec::with_capacity(total);
    nodes.push(node("root".into(), None, "root".into(), true));
    let mut file = 0usize;
    for d in 0..directories {
        let id = format!("d{d:04}");
        nodes.push(node(id.clone(), Some("root".into()), id.clone(), true));
        let share = files / directories + usize::from(d < files % directories);
        let end = (file + share).min(files);
        while file < end {
            let name = format!("f{file:07}");
            nodes.push(node(name.clone(), Some(id.clone()), name, false));
            file += 1;
        }
    }
    assert_eq!(nodes.len(), total, "synthetic fixture node count");
    let mut store = Store::open(dir)?;
    store.install_remote_snapshot(&nodes, "fixture-cursor-1", "fixture-server-1")?;
    store.ensure_fs_root("root")?;
    Ok(store)
}

fn percentile(sorted: &[Duration], percentile: usize) -> Duration {
    if sorted.is_empty() {
        return Duration::ZERO;
    }
    let index = (sorted.len() - 1) * percentile / 100;
    sorted[index]
}

fn sqlite_connection(path: &Path) -> Result<rusqlite::Connection, Box<dyn std::error::Error>> {
    Ok(rusqlite::Connection::open_with_flags(
        path.join("state.sqlite"),
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    )?)
}

fn sqlite_count(path: &Path, sql: &str) -> Result<i64, Box<dyn std::error::Error>> {
    let db = sqlite_connection(path)?;
    Ok(db.query_row(sql, [], |row| row.get(0))?)
}

/// A node whose attributes can change without touching directory membership.
fn sample_file(path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let db = sqlite_connection(path)?;
    Ok(db.query_row(
        "SELECT id FROM nodes WHERE directory=0 AND deleted=0 AND parent IS NOT NULL LIMIT 1",
        [],
        |row| row.get(0),
    )?)
}

fn wal_len(path: &Path) -> u64 {
    std::fs::metadata(path.join("state.sqlite-wal"))
        .map(|metadata| metadata.len())
        .unwrap_or(0)
}

fn measure(
    label: &str,
    store: &mut Store,
    nodes: usize,
    sample: &str,
    runs: usize,
    budget_ms: f64,
) -> Result<(), Box<dyn std::error::Error>> {
    let root = store.root().to_path_buf();
    let observer = sqlite_connection(&root)?;
    let data_version = |db: &rusqlite::Connection| -> Result<i64, Box<dyn std::error::Error>> {
        Ok(db.query_row("PRAGMA data_version", [], |row| row.get(0))?)
    };

    let build_started = Instant::now();
    let index = NamespaceIndex::build(store)?;
    let build_ms = build_started.elapsed().as_secs_f64() * 1_000.0;
    let watermark = store.namespace_watermark()?;
    println!(
        "NSI_BUDGET label={label} nodes={nodes} build_ms={build_ms:.1} budget_ms={budget_ms:.0} degraded={} partial={} bytes={} journal={} attributes={}",
        index.is_degraded(),
        index.is_partial(),
        index.bytes(),
        watermark.journal.sequence,
        watermark.attributes,
    );
    assert!(
        !index.is_degraded(),
        "namespace index must build inside its configured budget"
    );

    // 1. A read-only open commits nothing: no WAL frame, no data_version
    //    movement. The recency buffer is memory-only until a flush is forced.
    let before_wal = wal_len(&root);
    let before_version = data_version(&observer)?;
    let open_started = Instant::now();
    for _ in 0..runs {
        store.note_cache_use(sample)?;
    }
    let open_us = open_started.elapsed().as_secs_f64() * 1_000_000.0 / runs as f64;
    let after_wal = wal_len(&root);
    let after_version = data_version(&observer)?;
    println!(
        "NSI_READONLY_OPEN label={label} opens={runs} open_us={open_us:.2} wal_bytes_delta={} data_version_delta={}",
        after_wal.saturating_sub(before_wal),
        after_version - before_version,
    );
    assert_eq!(
        after_wal, before_wal,
        "read-only open must not append a WAL frame"
    );
    assert_eq!(
        after_version, before_version,
        "read-only open must not commit a transaction"
    );

    // 2. The first directory open right after read-only opens is served by the
    //    current generation: Fresh, no rebuild. The old implementation paid one
    //    rebuild here, measured below as the forced rebuild cost.
    let mut opendir_us = Vec::with_capacity(runs);
    let mut rebuilds = 0usize;
    for _ in 0..runs {
        store.note_cache_use(sample)?;
        let started = Instant::now();
        let outcome = index.ensure_fresh(store);
        let block = index.dir_block(1);
        opendir_us.push(started.elapsed());
        match outcome {
            RefreshOutcome::Fresh => {}
            RefreshOutcome::Refreshed => rebuilds += 1,
            RefreshOutcome::Degraded => {
                panic!("read-only open must not degrade the namespace projection")
            }
        }
        assert!(block.is_some(), "root directory must stay readable from the index");
    }
    opendir_us.sort_unstable();
    println!(
        "NSI_READONLY_OPENDIR label={label} runs={runs} p50_us={:.2} p95_us={:.2} max_us={:.2} rebuilds={rebuilds}",
        percentile(&opendir_us, 50).as_secs_f64() * 1_000_000.0,
        percentile(&opendir_us, 95).as_secs_f64() * 1_000_000.0,
        opendir_us.last().copied().unwrap_or_default().as_secs_f64() * 1_000_000.0,
    );
    assert_eq!(
        rebuilds, 0,
        "read-only open must not trigger a full namespace index rebuild"
    );

    // 3. The rebuild cost the old per-open invalidation paid, on the same
    //    fixture, plus the budget contract for that rebuild.
    let forced_started = Instant::now();
    index.invalidate(InvalidateHint::All);
    let forced = index.ensure_fresh(store);
    let rebuild_ms = forced_started.elapsed().as_secs_f64() * 1_000.0;
    let budget_ok =
        forced == RefreshOutcome::Refreshed && !index.is_degraded() && rebuild_ms <= budget_ms;
    println!(
        "NSI_REBUILD label={label} nodes={nodes} outcome={forced:?} rebuild_ms={rebuild_ms:.1} budget_ms={budget_ms:.0} budget_ok={budget_ok}"
    );
    assert!(budget_ok, "full rebuild must fit the namespace index budget");

    // 4. Mixed workload: read-only opens stay fast; only semantic writes
    //    invalidate, and each repair is a single bounded rebuild.
    let mut readdir_us = Vec::with_capacity(runs);
    let mut semantic_writes = 0usize;
    let mut semantic_rebuild_ms = 0.0f64;
    for i in 0..runs {
        store.note_cache_use(sample)?;
        let started = Instant::now();
        let outcome = index.ensure_fresh(store);
        let _ = index.dir_block(1);
        readdir_us.push(started.elapsed());
        assert_eq!(outcome, RefreshOutcome::Fresh);
        if i % 50 == 49 {
            // The fixture is reused across runs, so derive the next mode from
            // the persisted one; a no-op write is correctly invisible to the
            // semantic watermark and cannot exercise invalidation.
            let current = store.record(sample)?.mode & 0o777;
            let next = if current == 0o600 { 0o601 } else { 0o600 };
            let before_write = store.namespace_watermark()?;
            store.set_local_mode(sample, next)?;
            let after_write = store.namespace_watermark()?;
            assert_ne!(before_write, after_write, "semantic write must move the watermark");
            semantic_writes += 1;
            let rebuild_started = Instant::now();
            let repair = index.ensure_fresh(store);
            println!(
                "NSI_MIXED_WRITE i={i} sample={sample} mode={next:o} attributes={} outcome={repair:?} revision={}",
                after_write.attributes,
                index.revision()
            );
            assert_eq!(repair, RefreshOutcome::Refreshed);
            semantic_rebuild_ms += rebuild_started.elapsed().as_secs_f64() * 1_000.0;
        }
    }
    readdir_us.sort_unstable();
    println!(
        "NSI_MIXED label={label} read_opens={runs} semantic_writes={semantic_writes} p50_us={:.2} p95_us={:.2} rebuild_ms_total={semantic_rebuild_ms:.1} rebuild_ms_per_write={:.1}",
        percentile(&readdir_us, 50).as_secs_f64() * 1_000_000.0,
        percentile(&readdir_us, 95).as_secs_f64() * 1_000_000.0,
        semantic_rebuild_ms / semantic_writes.max(1) as f64,
    );
    Ok(())
}

#[test]
#[ignore = "issue #22 acceptance harness: rebuilds large namespace fixtures"]
fn namespace_semantic_watermark_budget() -> Result<(), Box<dyn std::error::Error>> {
    let runs = env::var("NSI_RUNS")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(200usize);
    let budget_ms = env::var("NSI_BUDGET_MS")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(1_000.0);
    if let Some(fixture) = env::var("NSI_FIXTURE").ok().filter(|path| !path.trim().is_empty()) {
        let root = PathBuf::from(fixture);
        let nodes = sqlite_count(&root, "SELECT COUNT(*) FROM nodes WHERE deleted=0")? as usize;
        let sample = sample_file(&root)?;
        let mut store = Store::open(&root)?;
        // Fixtures built by older builds may predate the completeness hint;
        // materialize every namespace inode so the index can be fully built.
        store.ensure_filesystem_inodes("root")?;
        measure("fixture", &mut store, nodes, &sample, runs, budget_ms)?;
    }
    if env::var("NSI_SKIP_SYNTHETIC").as_deref() == Ok("1") {
        return Ok(());
    }
    let sizes = env::var("NSI_SIZES").unwrap_or_else(|_| "100000,319679,369298".into());
    for size in sizes.split(',').filter_map(|value| value.trim().parse::<usize>().ok()) {
        let temp = tempfile::tempdir()?;
        let mut store = synthetic_fixture(temp.path(), size)?;
        let sample = sample_file(temp.path())?;
        measure(&format!("synthetic-{size}"), &mut store, size, &sample, runs, budget_ms)?;
        drop(store);
    }
    Ok(())
}
