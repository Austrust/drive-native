//! Issue #87: the durable `excluded_purged` merge must be linear, not the
//! former `Vec::contains` merge that compared K unique ids against each other.
//!
//! The unit tests measure the merge's membership probes directly (test-only
//! profile): the former algorithm performs K(K-1)/2 string comparisons with an
//! empty previous list, the hash-set merge performs O(K+P) probes. The heavy
//! test also runs a real purge over 1k/10k/50k nodes and records the merge,
//! transaction and Store-lock hold cost; it is `#[ignore]` so the normal suite
//! stays fast and it runs explicitly on the remote builder.
use crate::state::{
    ExcludedPurgedMerge, excluded_purge_profile, reset_excluded_purge_profile,
};
use crate::Store;
use rusqlite::params;

/// The pre-fix reference merge: exactly the former `purge_excluded_nodes` loop.
fn legacy_merge(previous: Vec<String>, persisted: Vec<String>) -> Vec<String> {
    let mut merged = previous;
    for id in persisted {
        if !merged.contains(&id) {
            merged.push(id);
        }
    }
    merged
}

fn ids(prefix: &str, count: usize) -> Vec<String> {
    (0..count).map(|i| format!("{prefix}{i:07}")).collect()
}

/// Run one prepared merge and return (merged, probes).
fn measured_merge(previous: Vec<String>, persisted: Vec<String>) -> (Vec<String>, u64) {
    let raw = if previous.is_empty() {
        None
    } else {
        Some(serde_json::to_string(&previous).unwrap())
    };
    reset_excluded_purge_profile();
    let merged = ExcludedPurgedMerge::prepare(raw).absorb(persisted);
    (merged, excluded_purge_profile().probes)
}

/// The stable serialization order and duplicate handling must match the former
/// `Vec::contains` merge for every shape, including duplicate ids.
#[test]
fn merge_preserves_the_legacy_serialization_order() {
    let shapes: Vec<(Vec<String>, Vec<String>)> = vec![
        (Vec::new(), ids("a", 10)),
        (ids("a", 5), ids("a", 10)),
        (ids("a", 10), ids("a", 5)),
        (ids("a", 5), ids("a", 5)),
        (Vec::new(), vec!["x".into(), "x".into(), "y".into(), "x".into()]),
        (vec!["x".into(), "x".into()], vec!["x".into(), "y".into()]),
        (ids("b", 3), vec!["b0000001".into(), "c".into(), "b0000001".into()]),
    ];
    for (previous, persisted) in shapes {
        let expected = legacy_merge(previous.clone(), persisted.clone());
        let (actual, _) = measured_merge(previous, persisted);
        assert_eq!(actual, expected);
    }
}

/// Work bound AC: K unique ids with a previous list of size P do O(K+P)
/// membership probes, for empty, half-overlapping and fully-duplicated shapes.
#[test]
fn membership_work_is_linear_in_ids_and_previous() {
    const K: usize = 1_000;
    let new_ids = ids("n", K);
    let shapes = [
        ("empty", Vec::new()),
        ("half", ids("n", K / 2)),
        ("full", new_ids.clone()),
    ];
    for (shape, previous) in shapes {
        let p = previous.len();
        let (_, probes) = measured_merge(previous, new_ids.clone());
        let bound = 4 * u64::try_from(K + p).unwrap() + 64;
        assert!(
            probes <= bound,
            "#87 shape={shape} K={K} P={p}: merge performed {probes} membership probes, \
             expected at most {bound} (linear in K+P); a Vec::contains merge needs \
             about {} comparisons",
            u64::try_from(K).unwrap() * u64::try_from(K + p).unwrap() / 2
        );
    }
}

/// A real purge must route through the prepared merge: 200 hidden nodes must
/// not cost 200*199/2 comparisons. This is the smallest end-to-end red sample.
#[test]
fn purge_path_uses_the_linear_merge() {
    let dir = tempfile::tempdir().unwrap();
    let mut store = Store::open(dir.path()).unwrap();
    let wanted = ids("p", 200);
    insert_nodes(&store, &wanted);
    reset_excluded_purge_profile();
    let purged = store.purge_excluded_nodes(&wanted).unwrap();
    assert_eq!(purged, 200);
    let profile = excluded_purge_profile();
    assert!(
        profile.probes <= 4 * 200 + 64,
        "#87 real purge performed {} membership probes for 200 ids (expected linear)",
        profile.probes
    );
    let marker: String = store
        .db
        .query_row(
            "SELECT value FROM metadata WHERE key='excluded_purged'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    let recorded: Vec<String> = serde_json::from_str(&marker).unwrap();
    assert_eq!(recorded, wanted);
}

fn insert_nodes(store: &Store, ids: &[String]) {
    let tx = store.db.unchecked_transaction().unwrap();
    {
        let mut insert = tx
            .prepare(
                "INSERT INTO nodes(id,parent,name,remote_version,directory)
                 VALUES(?1,NULL,?2,'v1',0)",
            )
            .unwrap();
        for id in ids {
            insert.execute(params![id, format!("name-{id}")]).unwrap();
        }
    }
    tx.commit().unwrap();
}

/// Issue #87 AC: 1k -> 100k curve. The merge work is measured separately from
/// the SQL/transaction work so the report can attribute both. Run explicitly on
/// the remote builder with `--ignored --nocapture`.
#[test]
#[ignore = "manual scaling evidence; run on the remote builder"]
fn dedupe_scale_curve_1k_to_100k() {
    let mut total_probes = 0u64;
    for k in [1_000usize, 10_000, 50_000, 100_000] {
        let new_ids = ids("s", k);
        let (merged, probes) = measured_merge(Vec::new(), new_ids.clone());
        assert_eq!(merged.len(), k);
        assert!(
            probes <= 4 * u64::try_from(k).unwrap() + 64,
            "K={k}: {probes} membership probes"
        );
        total_probes += probes;
        eprintln!(
            "DEDUPE_CURVE shape=unique previous=0 K={k} probes={probes} merged={}",
            merged.len()
        );
        // Half-overlap shape: P = K/2.
        let previous = ids("s", k / 2);
        let (merged, probes) = measured_merge(previous, new_ids);
        assert_eq!(merged.len(), k);
        eprintln!("DEDUPE_CURVE shape=half K={k} probes={probes} merged={}", merged.len());
        total_probes += probes;
    }
    eprintln!("DEDUPE_CURVE total_probes={total_probes}");
}

/// One real purge per size, with the dedupe, transaction and whole-call
/// durations split, plus metadata-query latency observed by a concurrent
/// reader while the serialized Store is held by the purge.
#[test]
#[ignore = "manual scaling evidence; run on the remote builder"]
fn purge_scale_curve_and_concurrent_metadata_latency() {
    use std::sync::{Arc, Mutex};
    use std::time::{Duration, Instant};

    for k in [1_000usize, 10_000, 50_000] {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path()).unwrap();
        let wanted = ids("q", k);
        insert_nodes(&store, &wanted);
        let shared = Arc::new(Mutex::new(store));

        // Concurrent metadata reader: it observes how long the serialized Store
        // is unavailable while the purge runs. The lock wait is the cost the
        // mount's metadata path pays, recorded as p50/p95/p99/max.
        let stop = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let reader = {
            let shared = shared.clone();
            let stop = stop.clone();
            std::thread::spawn(move || {
                let mut waits = Vec::new();
                while !stop.load(std::sync::atomic::Ordering::Relaxed) {
                    let started = Instant::now();
                    let guard = shared.lock().unwrap();
                    waits.push(started.elapsed());
                    let _ = guard.pending_operations();
                    drop(guard);
                    std::thread::sleep(Duration::from_millis(1));
                }
                waits
            })
        };
        std::thread::sleep(Duration::from_millis(20));
        reset_excluded_purge_profile();
        let started = Instant::now();
        let purged = shared
            .lock()
            .unwrap()
            .purge_excluded_nodes(&wanted)
            .unwrap();
        let elapsed = started.elapsed();
        stop.store(true, std::sync::atomic::Ordering::Relaxed);
        let mut waits = reader.join().unwrap();
        waits.sort_unstable();
        let pct = |p: f64| {
            if waits.is_empty() {
                0.0
            } else {
                waits[((waits.len() - 1) as f64 * p) as usize].as_secs_f64() * 1e3
            }
        };
        let profile = excluded_purge_profile();
        eprintln!(
            "PURGE_CURVE K={k} purged={purged} total_us={} probes={} prepare_us={} absorb_us={} tx_us={} reader_n={} reader_p50_ms={:.3} reader_p95_ms={:.3} reader_p99_ms={:.3} reader_max_ms={:.3}",
            elapsed.as_micros(),
            profile.probes,
            profile.prepare_ns / 1_000,
            profile.absorb_ns / 1_000,
            profile.transaction_ns / 1_000,
            waits.len(),
            pct(0.50),
            pct(0.95),
            pct(0.99),
            waits.last().map(|w| w.as_secs_f64() * 1e3).unwrap_or(0.0),
        );
        assert!(
            profile.probes <= 4 * u64::try_from(k).unwrap() + 64,
            "K={k}: the prepared merge performs linear membership work, got {} probes",
            profile.probes
        );
    }
}
