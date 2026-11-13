//! T1 storage-events perf probe (test-only, `#[ignore]`; issue #67 W2).
//!
//! Not part of the default suite: run explicitly through the serialized remote
//! runner so timing never overlaps another build. `T1_CHUNK=0` measures the
//! legacy one-transaction-per-Pull apply; `T1_CHUNK=64` measures the chunked
//! product path (Bridge::pull). Keep the same harness on base with T1_CHUNK
//! unset and compare like-for-like modes.
//!
//! Measures the cost of applying *remote* events through the real Store write
//! path (`receive` -> inbox tx, `drain` -> apply tx with sync_history triggers),
//! i.e. the work `Bridge::pull` does per Pull batch after the RPC returns.
//!
//! Modes (T1_MODE):
//!   batch       apply N events with batch sizes T1_BATCHES (fsync/commit amortisation)
//!   cumulative  apply N events in batches of T1_BATCH; per-batch time series (retention-cap crossing)
//!   history     pre-seed H sync_history rows at the retention cap; per-event cost vs history depth
//!   depth       chain depth D; per-event cost vs path length (ancestor walk)
//!   lookup      per-event runtime-side store lookups (Bridge invalidation pattern)
//!   fsync       single run intended to be wrapped in `strace -c -f` for syscall counts
//!
//! Output: `T1R ...` / `T1C ...` / `T1S ...` lines on stdout.
//!
//! Example (from the t1 tree, through the serialized runner):
//!   T1_MODE=batch T1_N=1000 T1_REPEAT=4 \
//!     ~/work/dn-perf-run.sh t1 heavy -- cargo test -p drive-core --release \
//!       --all-features --locked --test perf_t1_apply_cost t1_probe -- --ignored --nocapture
use drive_core::{Batch, Change, Event, Node, NodeMetadata, Store};
use rusqlite::{Connection, OpenFlags};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

const DIRS: usize = 100;

fn env_usize(key: &str, default: usize) -> usize {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

fn env_str(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

/// Issue #67 W2: 0 keeps the legacy "one transaction per Pull receipt" path; a
/// positive value applies the same receipt through `Store::apply_chunk`, which is
/// what the Bridge does in production (64 events per committed chunk).
fn chunk_events() -> usize {
    env_usize("T1_CHUNK", 0)
}

fn apply_receipt(store: &mut Store, chunk: usize) {
    if chunk == 0 {
        store.drain().unwrap();
        return;
    }
    loop {
        if matches!(
            store.apply_chunk(chunk).unwrap(),
            drive_core::ApplyChunk::Idle
        ) {
            break;
        }
    }
}

fn node(id: &str, parent: Option<&str>, name: &str, directory: bool) -> Node {
    Node {
        id: id.into(),
        parent: parent.map(str::to_owned),
        name: name.into(),
        version: "v1".into(),
        directory,
        metadata: Some(NodeMetadata {
            size: if directory { 0 } else { 4096 },
            mtime: 1_700_000_000,
            mode: if directory { 0o755 } else { 0o644 },
        }),
    }
}

fn file_node(id: &str, parent: &str) -> Node {
    Node {
        id: id.into(),
        parent: Some(parent.into()),
        name: format!("{id}.bin"),
        version: "v1".into(),
        directory: false,
        metadata: Some(NodeMetadata {
            size: 4096,
            mtime: 1_700_000_000,
            mode: 0o644,
        }),
    }
}

fn upsert(event_id: &str, n: Node) -> Event {
    Event {
        id: event_id.into(),
        change: Change::Upsert(n),
    }
}

fn make_batch(from: &str, to: &str, events: Vec<Event>) -> Batch {
    Batch {
        from: from.into(),
        to: to.into(),
        events,
    }
}

// ---------------------------------------------------------------------------
// metrics
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Default)]
struct Io {
    rchar: u64,
    wchar: u64,
    read_bytes: u64,
    write_bytes: u64,
    syscr: u64,
    syscw: u64,
}

impl Io {
    fn delta(self, newer: Io) -> Io {
        Io {
            rchar: newer.rchar.saturating_sub(self.rchar),
            wchar: newer.wchar.saturating_sub(self.wchar),
            read_bytes: newer.read_bytes.saturating_sub(self.read_bytes),
            write_bytes: newer.write_bytes.saturating_sub(self.write_bytes),
            syscr: newer.syscr.saturating_sub(self.syscr),
            syscw: newer.syscw.saturating_sub(self.syscw),
        }
    }
}

fn io_stats() -> Io {
    let mut io = Io::default();
    let Ok(text) = fs::read_to_string("/proc/self/io") else {
        return io;
    };
    for line in text.lines() {
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        let Ok(v) = value.trim().parse::<u64>() else {
            continue;
        };
        match key {
            "rchar" => io.rchar = v,
            "wchar" => io.wchar = v,
            "read_bytes" => io.read_bytes = v,
            "write_bytes" => io.write_bytes = v,
            "syscr" => io.syscr = v,
            "syscw" => io.syscw = v,
            _ => {}
        }
    }
    io
}

fn cpu_ms() -> f64 {
    // SAFETY: getrusage fills the provided, fully-owned struct.
    unsafe {
        let mut ru: libc::rusage = std::mem::zeroed();
        if libc::getrusage(libc::RUSAGE_SELF, &mut ru) != 0 {
            return 0.0;
        }
        (ru.ru_utime.tv_sec as f64 + ru.ru_utime.tv_usec as f64 / 1e6) * 1e3
            + (ru.ru_stime.tv_sec as f64 + ru.ru_stime.tv_usec as f64 / 1e6) * 1e3
    }
}

fn peak_rss_kb() -> u64 {
    let Ok(text) = fs::read_to_string("/proc/self/status") else {
        return 0;
    };
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("VmHWM:") {
            return rest
                .trim()
                .trim_end_matches(" kB")
                .trim()
                .parse()
                .unwrap_or(0);
        }
    }
    0
}

fn file_len(path: &Path) -> u64 {
    fs::metadata(path).map(|m| m.len()).unwrap_or(0)
}

fn wal_bytes(root: &Path) -> u64 {
    file_len(&root.join("state.sqlite-wal"))
}

fn db_bytes(root: &Path) -> u64 {
    file_len(&root.join("state.sqlite"))
}

fn emit(fields: &[(&str, String)]) {
    let body: Vec<String> = fields.iter().map(|(k, v)| format!("{k}={v}")).collect();
    println!("T1R {}", body.join(" "));
}

fn num(x: f64, decimals: usize) -> String {
    format!("{:.*}", decimals, x)
}

#[derive(Clone, Copy, Default)]
struct DbStats {
    history_rows: i64,
    history_seq: i64,
    nodes: i64,
    seen: i64,
    metadata_rows: i64,
}

fn db_stats(root: &Path) -> DbStats {
    let mut stats = DbStats::default();
    let Ok(db) = Connection::open_with_flags(
        root.join("state.sqlite"),
        OpenFlags::SQLITE_OPEN_READ_ONLY,
    ) else {
        return stats;
    };
    let one = |sql: &str| -> i64 { db.query_row(sql, [], |r| r.get(0)).unwrap_or(-1) };
    stats.history_rows = one("SELECT COUNT(*) FROM sync_history");
    stats.history_seq = one(
        "SELECT CAST(value AS INTEGER) FROM metadata WHERE key='sync_history_sequence'",
    );
    stats.metadata_rows = one(
        "SELECT CAST(value AS INTEGER) FROM metadata WHERE key='sync_history_rows'",
    );
    stats.nodes = one("SELECT COUNT(*) FROM nodes");
    stats.seen = one("SELECT COUNT(*) FROM seen");
    stats
}

// ---------------------------------------------------------------------------
// fixture
// ---------------------------------------------------------------------------

/// A store with a root directory and `DIRS` subdirectories, installed as a
/// recovery snapshot (no sync_history rows, no event journal writes).
fn open_tree() -> (tempfile::TempDir, Store) {
    let dir = tempfile::tempdir().unwrap();
    let mut store = Store::open(dir.path()).unwrap();
    let mut nodes = vec![node("root", None, "Drive", true)];
    for i in 0..DIRS {
        let id = format!("d{i:03}");
        nodes.push(node(&id, Some("root"), &id, true));
    }
    store.install_remote_snapshot(&nodes, "0", "server").unwrap();
    store.ensure_fs_root("root").unwrap();
    (dir, store)
}

/// A store whose namespace is a single directory chain `root -> c00 -> c01 ...`.
fn open_chain(depth: usize) -> (tempfile::TempDir, Store, String) {
    let dir = tempfile::tempdir().unwrap();
    let mut store = Store::open(dir.path()).unwrap();
    let mut nodes = vec![node("root", None, "Drive", true)];
    let mut parent = "root".to_string();
    for i in 0..depth {
        let id = format!("c{i:02}");
        nodes.push(node(&id, Some(&parent), &id, true));
        parent = id;
    }
    store.install_remote_snapshot(&nodes, "0", "server").unwrap();
    store.ensure_fs_root("root").unwrap();
    (dir, store, parent)
}

fn seed_history(db: &Path, rows: usize) {
    let conn = Connection::open(db).unwrap();
    conn.execute_batch("BEGIN IMMEDIATE;").unwrap();
    conn.execute(
        &format!(
            "WITH RECURSIVE n(x) AS (SELECT 1 UNION ALL SELECT x+1 FROM n WHERE x<{rows})
             INSERT INTO sync_history(seq,mount,node,name,path_at_event,action,result,
               attempts,completed_at,operation_key)
             SELECT x,'server',NULL,'seed-'||x,'seed/path/'||x,'remote_change','ok',1,
               unixepoch(),'seed:'||x FROM n"
        ),
        [],
    )
    .unwrap();
    conn.execute(
        "UPDATE metadata SET value=?1 WHERE key='sync_history_sequence'",
        [rows as i64],
    )
    .unwrap();
    conn.execute_batch("COMMIT;").unwrap();
}

// ---------------------------------------------------------------------------
// apply stream
// ---------------------------------------------------------------------------

struct BatchRun {
    recv_ms: f64,
    apply_ms: f64,
    wal_after: u64,
}

/// `receive` + `drain` `total` events in batches of `batch`, exactly once per
/// batch like Bridge::pull. `obs(batch_idx, events_so_far, apply_ms, wal)` runs
/// after each committed batch.
#[allow(clippy::type_complexity)]
fn run_stream(
    store: &mut Store,
    root: &Path,
    total: usize,
    batch: usize,
    prefix: &str,
    parent_for: &dyn Fn(usize) -> String,
    mut obs: impl FnMut(usize, usize, f64, u64),
) -> Vec<BatchRun> {
    let mut runs = Vec::new();
    let mut cursor = store.cursor().unwrap();
    let mut done = 0usize;
    let mut idx = 0usize;
    while done < total {
        let take = batch.min(total - done);
        let mut events = Vec::with_capacity(take);
        for k in 0..take {
            let i = done + k;
            let id = format!("{prefix}{i}");
            events.push(upsert(&format!("ev-{prefix}{i}"), file_node(&id, &parent_for(i))));
        }
        let next = format!("{prefix}c{idx}");
        let b = make_batch(&cursor, &next, events);
        let t0 = Instant::now();
        store.receive(&b).unwrap();
        let recv_ms = t0.elapsed().as_secs_f64() * 1e3;
        let t1 = Instant::now();
        apply_receipt(store, chunk_events());
        let apply_ms = t1.elapsed().as_secs_f64() * 1e3;
        cursor = next;
        let wal = wal_bytes(root);
        runs.push(BatchRun {
            recv_ms,
            apply_ms,
            wal_after: wal,
        });
        obs(idx, done + take, apply_ms, wal);
        done += take;
        idx += 1;
    }
    runs
}

fn totals(runs: &[BatchRun]) -> (f64, f64) {
    (
        runs.iter().map(|r| r.recv_ms).sum(),
        runs.iter().map(|r| r.apply_ms).sum(),
    )
}

fn percentile(values: &[f64], p: f64) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let mut v = values.to_vec();
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let rank = ((v.len() - 1) as f64 * p).round() as usize;
    v[rank]
}

// ---------------------------------------------------------------------------
// modes
// ---------------------------------------------------------------------------

fn mode_batch(repeats: usize, warmup: usize) {
    let total = env_usize("T1_N", 1000);
    let sizes: Vec<usize> = std::env::var("T1_BATCHES")
        .unwrap_or_else(|_| "1,10,100,1000".to_string())
        .split(',')
        .filter_map(|s| s.trim().parse().ok())
        .collect();
    for size in sizes {
        let mut all_apply = Vec::new();
        let mut all_apply_per_event = Vec::new();
        for rep in 0..repeats + warmup {
            let (dir, mut store) = open_tree();
            let root = dir.path().to_path_buf();
            let io0 = io_stats();
            let cpu0 = cpu_ms();
            let runs = run_stream(
                &mut store,
                &root,
                total,
                size,
                &format!("b{size}r{rep}-"),
                &|i: usize| format!("d{:03}", i % DIRS),
                |_, _, _, _| {},
            );
            let cpu = cpu_ms() - cpu0;
            let dio = io0.delta(io_stats());
            let (recv_ms, apply_ms) = totals(&runs);
            let total_ms = recv_ms + apply_ms;
            let stats = db_stats(&root);
            let per_event_us = apply_ms * 1e3 / total as f64;
            if rep >= warmup {
                all_apply.push(apply_ms);
                all_apply_per_event.push(per_event_us);
            }
            emit(&[
                ("mode", "batch".into()),
                ("rep", rep.to_string()),
                ("batch", size.to_string()),
                ("events", total.to_string()),
                ("recv_ms", num(recv_ms, 2)),
                ("apply_ms", num(apply_ms, 2)),
                ("total_ms", num(total_ms, 2)),
                ("apply_us_per_event", num(per_event_us, 2)),
                ("cpu_ms", num(cpu, 2)),
                ("rchar", dio.rchar.to_string()),
                ("wchar", dio.wchar.to_string()),
                ("write_bytes", dio.write_bytes.to_string()),
                ("syscw", dio.syscw.to_string()),
                ("wal_after", runs.last().map(|r| r.wal_after).unwrap_or(0).to_string()),
                ("wal_max", runs.iter().map(|r| r.wal_after).max().unwrap_or(0).to_string()),
                ("db_bytes", db_bytes(&root).to_string()),
                ("history_rows", stats.history_rows.to_string()),
                ("history_seq", stats.history_seq.to_string()),
                ("nodes", stats.nodes.to_string()),
                ("seen", stats.seen.to_string()),
                ("peak_rss_kb", peak_rss_kb().to_string()),
            ]);
            drop(store);
        }
        let median_apply = percentile(&all_apply, 0.5);
        let median_per_event = percentile(&all_apply_per_event, 0.5);
        let p95_per_event = percentile(&all_apply_per_event, 0.95);
        emit(&[
            ("mode", "batch_median".into()),
            ("batch", size.to_string()),
            ("events", total.to_string()),
            ("n", all_apply.len().to_string()),
            ("apply_ms_median", num(median_apply, 2)),
            ("apply_us_per_event_median", num(median_per_event, 2)),
            ("apply_us_per_event_p95", num(p95_per_event, 2)),
        ]);
    }
}

fn mode_cumulative(repeats: usize, warmup: usize) {
    let total = env_usize("T1_N", 12_000);
    let batch = env_usize("T1_BATCH", 100);
    for rep in 0..repeats + warmup {
        let (dir, mut store) = open_tree();
        let root = dir.path().to_path_buf();
        let io0 = io_stats();
        let cpu0 = cpu_ms();
        let report_every = 20usize;
        let mut last_stats = DbStats::default();
        let mut stats_at = Vec::new();
        let runs = run_stream(
            &mut store,
            &root,
            total,
            batch,
            &format!("cum{rep}-"),
            &|i: usize| format!("d{:03}", i % DIRS),
            |bi, events, apply_ms, wal| {
                println!(
                    "T1C rep={rep} bi={bi} events={events} apply_ms={:.2} us_per_event={:.2} wal={wal}",
                    apply_ms,
                    apply_ms * 1e3 / batch as f64
                );
                if (bi + 1) % report_every == 0 {
                    last_stats = db_stats(&root);
                    stats_at.push((
                        events,
                        last_stats.history_rows,
                        last_stats.history_seq,
                        last_stats.metadata_rows,
                    ));
                    println!(
                        "T1S rep={rep} events={events} history_rows={} history_seq={} metadata_rows={}",
                        last_stats.history_rows, last_stats.history_seq, last_stats.metadata_rows
                    );
                }
            },
        );
        let cpu = cpu_ms() - cpu0;
        let dio = io0.delta(io_stats());
        let (recv_ms, apply_ms) = totals(&runs);
        let stats = db_stats(&root);
        let (_lo, _l1, _l2, _l3) = stats_at.first().copied().unwrap_or_default();
        let (hi_events, hi_rows, _hi_seq, _hi_meta) = stats_at.last().copied().unwrap_or_default();
        emit(&[
            ("mode", "cumulative".into()),
            ("rep", rep.to_string()),
            ("events", total.to_string()),
            ("batch", batch.to_string()),
            ("recv_ms", num(recv_ms, 2)),
            ("apply_ms", num(apply_ms, 2)),
            ("apply_us_per_event", num(apply_ms * 1e3 / total as f64, 2)),
            ("cpu_ms", num(cpu, 2)),
            ("rchar", dio.rchar.to_string()),
            ("wchar", dio.wchar.to_string()),
            ("write_bytes", dio.write_bytes.to_string()),
            ("wal_after", runs.last().map(|r| r.wal_after).unwrap_or(0).to_string()),
            ("wal_max", runs.iter().map(|r| r.wal_after).max().unwrap_or(0).to_string()),
            ("db_bytes", db_bytes(&root).to_string()),
            ("history_rows", stats.history_rows.to_string()),
            ("history_seq", stats.history_seq.to_string()),
            ("metadata_rows", stats.metadata_rows.to_string()),
            ("last_report_events", hi_events.to_string()),
            ("last_report_history_rows", hi_rows.to_string()),
            ("nodes", stats.nodes.to_string()),
            ("seen", stats.seen.to_string()),
            ("peak_rss_kb", peak_rss_kb().to_string()),
        ]);
        drop(store);
    }
}

fn prepare_seeded_history(rows: usize) -> (tempfile::TempDir, PathBuf) {
    let (dir, mut store) = open_tree();
    let cap = rows.max(10_000) as u32;
    store.set_sync_history_retention(30, cap).unwrap();
    drop(store);
    let path = dir.path().to_path_buf();
    if rows > 0 {
        seed_history(&path.join("state.sqlite"), rows);
    }
    (dir, path)
}

fn mode_history(repeats: usize, warmup: usize) {
    let total = env_usize("T1_N", 1000);
    let batch = env_usize("T1_BATCH", 100);
    let depth: Vec<usize> = std::env::var("T1_H")
        .unwrap_or_else(|_| "0,10000,100000".to_string())
        .split(',')
        .filter_map(|s| s.trim().parse().ok())
        .collect();
    for h in depth {
        let (seed_dir, seed_path) = prepare_seeded_history(h);
        let mut per_event_values = Vec::new();
        for rep in 0..repeats + warmup {
            let rep_dir = tempfile::tempdir().unwrap();
            fs::copy(
                seed_path.join("state.sqlite"),
                rep_dir.path().join("state.sqlite"),
            )
            .unwrap();
            let mut store = Store::open(rep_dir.path()).unwrap();
            let root = rep_dir.path().to_path_buf();
            let before = db_stats(&root);
            let io0 = io_stats();
            let cpu0 = cpu_ms();
            let runs = run_stream(
                &mut store,
                &root,
                total,
                batch,
                &format!("h{h}r{rep}-"),
                &|i: usize| format!("d{:03}", i % DIRS),
                |_, _, _, _| {},
            );
            let cpu = cpu_ms() - cpu0;
            let dio = io0.delta(io_stats());
            let (recv_ms, apply_ms) = totals(&runs);
            let after = db_stats(&root);
            let per_event_us = apply_ms * 1e3 / total as f64;
            if rep >= warmup {
                per_event_values.push(per_event_us);
            }
            emit(&[
                ("mode", "history".into()),
                ("rep", rep.to_string()),
                ("seeded_history", h.to_string()),
                ("events", total.to_string()),
                ("batch", batch.to_string()),
                ("history_before", before.history_rows.to_string()),
                ("history_after", after.history_rows.to_string()),
                ("recv_ms", num(recv_ms, 2)),
                ("apply_ms", num(apply_ms, 2)),
                ("apply_us_per_event", num(per_event_us, 2)),
                ("cpu_ms", num(cpu, 2)),
                ("rchar", dio.rchar.to_string()),
                ("wchar", dio.wchar.to_string()),
                ("write_bytes", dio.write_bytes.to_string()),
                ("syscw", dio.syscw.to_string()),
                ("wal_after", runs.last().map(|r| r.wal_after).unwrap_or(0).to_string()),
                ("wal_max", runs.iter().map(|r| r.wal_after).max().unwrap_or(0).to_string()),
                ("db_bytes", db_bytes(&root).to_string()),
                ("peak_rss_kb", peak_rss_kb().to_string()),
            ]);
            drop(store);
        }
        emit(&[
            ("mode", "history_median".into()),
            ("seeded_history", h.to_string()),
            ("n", per_event_values.len().to_string()),
            ("apply_us_per_event_median", num(percentile(&per_event_values, 0.5), 2)),
            ("apply_us_per_event_p95", num(percentile(&per_event_values, 0.95), 2)),
        ]);
        drop(seed_dir);
    }
}

fn mode_depth(repeats: usize, warmup: usize) {
    let total = env_usize("T1_N", 500);
    let batch = env_usize("T1_BATCH", 100);
    let depths: Vec<usize> = std::env::var("T1_DEPTHS")
        .unwrap_or_else(|_| "3,10,30,60".to_string())
        .split(',')
        .filter_map(|s| s.trim().parse().ok())
        .collect();
    for d in depths {
        let mut values = Vec::new();
        for rep in 0..repeats + warmup {
            let (dir, mut store, deepest) = open_chain(d);
            let root = dir.path().to_path_buf();
            let io0 = io_stats();
            let cpu0 = cpu_ms();
            let runs = run_stream(
                &mut store,
                &root,
                total,
                batch,
                &format!("dep{d}r{rep}-"),
                &|_: usize| deepest.clone(),
                |_, _, _, _| {},
            );
            let cpu = cpu_ms() - cpu0;
            let dio = io0.delta(io_stats());
            let (recv_ms, apply_ms) = totals(&runs);
            let per_event_us = apply_ms * 1e3 / total as f64;
            if rep >= warmup {
                values.push(per_event_us);
            }
            emit(&[
                ("mode", "depth".into()),
                ("rep", rep.to_string()),
                ("chain_depth", d.to_string()),
                ("events", total.to_string()),
                ("recv_ms", num(recv_ms, 2)),
                ("apply_ms", num(apply_ms, 2)),
                ("apply_us_per_event", num(per_event_us, 2)),
                ("cpu_ms", num(cpu, 2)),
                ("wchar", dio.wchar.to_string()),
                ("wal_after", runs.last().map(|r| r.wal_after).unwrap_or(0).to_string()),
            ]);
            drop(store);
        }
        emit(&[
            ("mode", "depth_median".into()),
            ("chain_depth", d.to_string()),
            ("n", values.len().to_string()),
            ("apply_us_per_event_median", num(percentile(&values, 0.5), 2)),
        ]);
    }
}

fn mode_lookup(repeats: usize, warmup: usize) {
    let total = env_usize("T1_N", 20_000);
    let (dir, mut store) = open_tree();
    let root = dir.path().to_path_buf();
    let _ = run_stream(
        &mut store,
        &root,
        1000,
        100,
        "lk-",
        &|i: usize| format!("d{:03}", i % DIRS),
        |_, _, _, _| {},
    );
    let ids: Vec<String> = (0..1000).map(|i| format!("lk-{i}")).collect();
    for rep in 0..repeats + warmup {
        let t = Instant::now();
        for k in 0..total {
            let id = &ids[k % ids.len()];
            let local = store.local_identity(id).unwrap();
            let rec = store.record(&local).unwrap();
            if let Some(p) = rec.node.parent.clone() {
                let lp = store.local_identity(&p).unwrap();
                let _ = store.record(&lp).unwrap();
            }
        }
        let us = t.elapsed().as_secs_f64() * 1e6 / total as f64;
        emit(&[
            ("mode", "lookup".into()),
            ("rep", rep.to_string()),
            ("it", total.to_string()),
            ("bridge_lookup_us_per_event", num(us, 3)),
        ]);
    }
    drop(store);
}

fn mode_fsync(repeats: usize, warmup: usize) {
    let total = env_usize("T1_N", 0);
    let batch = env_usize("T1_BATCH", 1);
    for rep in 0..repeats + warmup {
        let (dir, mut store) = open_tree();
        let root = dir.path().to_path_buf();
        let runs = if total > 0 {
            run_stream(
                &mut store,
                &root,
                total,
                batch,
                &format!("fs{rep}-"),
                &|i: usize| format!("d{:03}", i % DIRS),
                |_, _, _, _| {},
            )
        } else {
            Vec::new()
        };
        let (recv_ms, apply_ms) = totals(&runs);
        let stats = db_stats(&root);
        emit(&[
            ("mode", "fsync".into()),
            ("rep", rep.to_string()),
            ("events", total.to_string()),
            ("batch", batch.to_string()),
            ("recv_ms", num(recv_ms, 2)),
            ("apply_ms", num(apply_ms, 2)),
            ("history_rows", stats.history_rows.to_string()),
            ("wal_after", runs.last().map(|r| r.wal_after).unwrap_or(0).to_string()),
        ]);
        drop(store);
    }
}

#[test]
#[ignore]
fn t1_probe() {
    let mode = env_str("T1_MODE", "");
    let repeats = env_usize("T1_REPEAT", 3);
    let warmup = env_usize("T1_WARMUP", 1);
    match mode.as_str() {
        "batch" => mode_batch(repeats, warmup),
        "cumulative" => mode_cumulative(repeats, warmup),
        "history" => mode_history(repeats, warmup),
        "depth" => mode_depth(repeats, warmup),
        "lookup" => mode_lookup(repeats, warmup),
        "fsync" => mode_fsync(repeats, warmup),
        other => panic!("T1_MODE must be one of batch|cumulative|history|depth|lookup|fsync, got {other:?}"),
    }
}
