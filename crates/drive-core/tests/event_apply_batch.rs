//! Event-apply batching and chunked catch-up contracts (issue #67 W2).
//!
//! These tests pin the parts of the event application path that batching must
//! not change: `path_at_event` is captured per event, the inbox receipt and
//! `committed_cursor` only move in the receipt's final transaction, crash replay
//! is idempotent through `seen`, a failing chunk rolls back without advancing the
//! cursor, and the retention cap/counters stay exact when history rows are
//! written in batches.
use drive_core::sync_history::HistoryQuery;
use drive_core::{
    ApplyChunk, Batch, Change, Event, Node, NodeMetadata, SharedStore, Store,
};
use rusqlite::Connection;
use std::time::{Duration, Instant};

fn node(id: &str, parent: Option<&str>, name: &str, version: &str, directory: bool) -> Node {
    Node {
        id: id.into(),
        parent: parent.map(str::to_owned),
        name: name.into(),
        version: version.into(),
        directory,
        metadata: Some(NodeMetadata {
            size: if directory { 0 } else { 3 },
            mtime: 1_700_000_000,
            mode: if directory { 0o755 } else { 0o644 },
        }),
    }
}

/// A store with root `root`, directories `A` and `B`, installed as a snapshot
/// (no event journal writes yet).
fn fixture() -> (tempfile::TempDir, Store) {
    let dir = tempfile::tempdir().unwrap();
    let mut store = Store::open(dir.path()).unwrap();
    let nodes = [
        node("root", None, "root", "v1", true),
        node("A", Some("root"), "A", "v1", true),
        node("B", Some("root"), "B", "v1", true),
    ];
    store.install_remote_snapshot(&nodes, "0", "server").unwrap();
    store.ensure_fs_root("root").unwrap();
    (dir, store)
}

fn upsert(event_id: &str, n: Node) -> Event {
    Event {
        id: event_id.into(),
        change: Change::Upsert(n),
    }
}

fn create_files(count: usize) -> Vec<Event> {
    (0..count)
        .map(|i| {
            upsert(
                &format!("ev-create-{i}"),
                node(
                    &format!("create-{i}"),
                    Some("root"),
                    &format!("{i:04}.bin"),
                    "v1",
                    false,
                ),
            )
        })
        .collect()
}

fn batch(from: &str, to: &str, events: Vec<Event>) -> Batch {
    Batch {
        from: from.into(),
        to: to.into(),
        events,
    }
}

fn history(store: &Store) -> Vec<(i64, String, String)> {
    // `HistoryQuery::default()` pages at 100 rows; these tests assert exact
    // counts, so request the API maximum.
    let query = HistoryQuery {
        limit: Some(500),
        ..Default::default()
    };
    let mut rows: Vec<(i64, String, String)> = store
        .sync_history(&query)
        .unwrap()
        .records
        .into_iter()
        .map(|r| (r.seq, r.action, r.path_at_event))
        .collect();
    rows.sort();
    rows
}

/// Raw durable state, for assertions the public API intentionally hides.
struct Durable {
    conn: Connection,
}
impl Durable {
    fn new(dir: &std::path::Path) -> Self {
        Self {
            conn: Connection::open(dir.join("state.sqlite")).unwrap(),
        }
    }
    fn count(&self, table: &str) -> i64 {
        self.conn
            .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |r| r.get(0))
            .unwrap()
    }
    fn metadata(&self, key: &str) -> Option<i64> {
        self.conn
            .query_row(
                "SELECT CAST(value AS INTEGER) FROM metadata WHERE key=?1",
                [key],
                |r| r.get(0),
            )
            .ok()
    }
}

#[test]
fn history_keeps_path_at_each_event_inside_one_batch() {
    // R4: the row of the first event must carry the path the namespace had when
    // that event applied, not the path after the batch's later move.
    let (_dir, mut store) = fixture();
    store
        .receive(&batch(
            "0",
            "c1",
            vec![
                upsert("event-create", node("x", Some("A"), "x", "v1", false)),
                upsert("event-move", node("x", Some("B"), "x", "v2", false)),
            ],
        ))
        .unwrap();
    store.drain().unwrap();
    let rows = history(&store);
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].1, "remote_change");
    assert_eq!(rows[0].2, "A/x");
    assert_eq!(rows[1].2, "B/x");
    assert!(rows[0].0 < rows[1].0, "sequence must follow event order");
    assert_eq!(store.node("x").unwrap().parent.as_deref(), Some("B"));
}

#[test]
fn chunked_apply_advances_the_cursor_only_on_the_final_chunk() {
    let (_dir, mut store) = fixture();
    store
        .receive(&batch("0", "c1", create_files(150)))
        .unwrap();
    // The receipt is durable but the committed cursor only describes applied
    // namespace state, so it must not move while a chunk is still pending. It
    // starts at the snapshot cursor and only advances on the final chunk.
    assert_eq!(store.cursor().unwrap(), "0");
    assert_eq!(
        store.apply_chunk(64).unwrap(),
        ApplyChunk::Partial { applied: 64 }
    );
    assert_eq!(store.cursor().unwrap(), "0");
    assert_eq!(history(&store).len(), 64);
    assert_eq!(
        store.apply_chunk(64).unwrap(),
        ApplyChunk::Partial { applied: 64 }
    );
    assert_eq!(store.cursor().unwrap(), "0");
    assert_eq!(
        store.apply_chunk(64).unwrap(),
        ApplyChunk::Done { applied: 22 }
    );
    assert_eq!(store.cursor().unwrap(), "c1");
    assert_eq!(store.apply_chunk(64).unwrap(), ApplyChunk::Idle);
    assert_eq!(history(&store).len(), 150);
    // The legacy entry point still applies a whole receipt in one transaction.
    store
        .receive(&batch("c1", "c2", create_files(2)))
        .unwrap();
    assert!(store.apply_next().unwrap());
    assert_eq!(store.cursor().unwrap(), "c2");
}

#[test]
fn chunked_apply_replays_after_a_crash_without_duplicate_history() {
    let (dir, mut store) = fixture();
    store
        .receive(&batch("0", "c1", create_files(150)))
        .unwrap();
    // Simulate a process death after the first chunk committed.
    assert_eq!(
        store.apply_chunk(64).unwrap(),
        ApplyChunk::Partial { applied: 64 }
    );
    drop(store);
    let mut store = Store::open(dir.path()).unwrap();
    assert_eq!(store.cursor().unwrap(), "0");
    // Replaying the retained receipt skips the committed prefix through `seen`
    // and applies the rest; no event is written twice.
    store.drain().unwrap();
    assert_eq!(store.cursor().unwrap(), "c1");
    assert_eq!(history(&store).len(), 150);
    drop(store);
    let durable = Durable::new(dir.path());
    assert_eq!(durable.count("seen"), 150);
    assert_eq!(durable.count("sync_history"), 150);
    assert_eq!(durable.count("inbox"), 0);
}

#[test]
fn failing_chunk_rolls_back_without_advancing_the_cursor() {
    let (dir, mut store) = fixture();
    let mut events = create_files(130);
    let mut invalid = node("invalid", Some("root"), "../escape", "v1", false);
    invalid.name = "../escape".into();
    events.insert(100, upsert("event-invalid", invalid));
    store.receive(&batch("0", "c1", events)).unwrap();
    // The first chunk is valid and commits; the second one fails as a unit.
    assert_eq!(
        store.apply_chunk(64).unwrap(),
        ApplyChunk::Partial { applied: 64 }
    );
    assert!(store.apply_chunk(64).is_err());
    assert_eq!(store.cursor().unwrap(), "0");
    assert_eq!(history(&store).len(), 64);
    // The same chunk is retried (not skipped) and keeps failing, so a bad event
    // can never be silently lost.
    assert!(store.apply_chunk(64).is_err());
    assert_eq!(store.cursor().unwrap(), "0");
    assert_eq!(history(&store).len(), 64);
    drop(store);
    let durable = Durable::new(dir.path());
    assert_eq!(durable.count("inbox"), 1);
    assert_eq!(durable.count("seen"), 64);
}

#[test]
fn retention_cap_and_counters_stay_exact_with_batched_history_writes() {
    let (dir, mut store) = fixture();
    store.set_sync_history_retention(30, 100).unwrap();
    let mut cursor = "0".to_string();
    for round in 0..5 {
        let next = format!("c{round}");
        let events: Vec<Event> = (0..100)
            .map(|i| {
                let n = round * 100 + i;
                upsert(
                    &format!("ev-cap-{n}"),
                    node(
                        &format!("cap-{n}"),
                        Some("root"),
                        &format!("{n:05}.bin"),
                        "v1",
                        false,
                    ),
                )
            })
            .collect();
        store.receive(&batch(&cursor, &next, events)).unwrap();
        while let ApplyChunk::Partial { .. } = store.apply_chunk(64).unwrap() {}
        cursor = next;
    }
    drop(store);
    let durable = Durable::new(dir.path());
    assert_eq!(durable.count("sync_history"), 100);
    assert_eq!(durable.metadata("sync_history_rows"), Some(100));
    assert_eq!(durable.metadata("sync_history_sequence"), Some(500));
    // The retained window is the newest 100 rows, in sequence order.
    let mut seqs: Vec<i64> = {
        let mut statement = durable
            .conn
            .prepare("SELECT seq FROM sync_history ORDER BY seq")
            .unwrap();
        statement
            .query_map([], |r| r.get(0))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap()
    };
    seqs.sort();
    assert_eq!(seqs.first().copied(), Some(401));
    assert_eq!(seqs.last().copied(), Some(500));
}

#[test]
fn chunked_apply_bounds_the_longest_store_write_lease_acquisition() {
    // Legacy behaviour: one transaction applies the whole receipt while holding
    // the single writer lease. The chunked path must cap each individual
    // acquisition (the p95 <= 100 ms contract) instead of holding it for the
    // whole batch.
    let (_dir, mut store) = fixture();
    store
        .receive(&batch("0", "c1", create_files(600)))
        .unwrap();
    let shared = SharedStore::new(store);
    let legacy = {
        let started = Instant::now();
        let mut guard = shared.lock().unwrap();
        assert!(guard.apply_next().unwrap());
        started.elapsed()
    };
    assert!(
        legacy > Duration::from_millis(10),
        "sanity: one-transaction apply of 600 events finished in only {legacy:?}"
    );

    // Mirror the Bridge's catch-up chunk size so this measures the product
    // lease bound, not just the API.
    const PRODUCT_CHUNK: usize = 256;
    let (_dir, mut store) = fixture();
    store
        .receive(&batch("0", "c1", create_files(600)))
        .unwrap();
    let shared = SharedStore::new(store);
    let mut longest = Duration::ZERO;
    let mut acquisitions = 0usize;
    loop {
        let started = Instant::now();
        let mut guard = shared.lock().unwrap();
        let chunk = guard.apply_chunk(PRODUCT_CHUNK).unwrap();
        let held = started.elapsed();
        drop(guard);
        longest = longest.max(held);
        acquisitions += 1;
        if matches!(chunk, ApplyChunk::Idle) {
            break;
        }
    }
    assert!(
        acquisitions >= 3,
        "expected one write-lease acquisition per chunk, got {acquisitions}"
    );
    // The AC number (p95 <= 100 ms per Store lease acquisition) is measured by
    // the #67 t1 hold harness under an exclusive remote timing lock, not here:
    // this test binary runs its cases in parallel and unoptimized builds run the
    // same SQL/parse path several times slower, so an absolute wall-clock bound
    // is not a stable assertion. Assert the load-independent structural bound
    // instead: a 256-event chunk is at most two thirds of the whole 600-event
    // receipt's apply, i.e. chunking really does bound the lease per acquisition.
    assert!(
        longest * 3 < legacy * 2,
        "chunked longest hold {longest:?} is not clearly below the whole-receipt \
         hold {legacy:?}"
    );
    // Release builds additionally reject a pathological lease hold (more than
    // 2.5x the AC number); debug builds are too slow to make that meaningful.
    if !cfg!(debug_assertions) {
        assert!(
            longest < Duration::from_millis(250),
            "longest Store write-lease hold was {longest:?} (unchunked: {legacy:?})"
        );
    }
}
