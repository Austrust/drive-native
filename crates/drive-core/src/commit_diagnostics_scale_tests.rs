//! Issue #88: the overview diagnostics snapshot must keep the exact
//! `open_unknown`/`needs_attention`/`list(20)` semantics while reading a bounded
//! amount of rows once the successful `confirmed` history grows.
//!
//! The unit tests live here (not in `tests/`) because the query-plan assertions
//! inspect the production SQL constants and the raw bundled-SQLite connection.
//! The scale test is `#[ignore]` and runs explicitly on the remote builder.
use super::*;
use crate::Store;
use rusqlite::Connection;

/// The pre-fix counts, kept only as the reference implementation.
const LEGACY_COUNTS_SQL: &str = "SELECT
   (SELECT COUNT(*) FROM commit_diagnostics WHERE outcome='unknown_retryable'),
   (SELECT COUNT(*) FROM commit_diagnostics
     WHERE outcome='conflict' OR (outcome='unknown_fatal' AND state<>'abandoned'))";

/// The pre-fix list SQL, kept only as the reference ordering.
const LEGACY_LIST_SQL: &str = "SELECT scope, operation, node, kind, state, outcome, outcome_category,
        last_error, attempts, next_retry_at, updated_at
   FROM commit_diagnostics
  ORDER BY CASE outcome WHEN 'conflict' THEN 0 WHEN 'unknown_retryable' THEN 1 ELSE 2 END,
           next_retry_at, scope, operation
  LIMIT ?1";

/// Insert `committed` successful history rows plus one fixed anomaly set that
/// covers every scope, every counted class and the terminal abandoned state.
fn seed(store: &Store, committed: usize) {
    if committed > 0 {
        store
            .db
            .execute_batch(&format!(
                "INSERT INTO operations(id,node,kind,base,generation,state,outcome,
                                        outcome_category,next_retry_at,updated_at,reconcile_attempts)
                 WITH RECURSIVE gen(n) AS (
                     SELECT 0 UNION ALL SELECT n+1 FROM gen WHERE n < {}
                 )
                 SELECT 'op'||n, 'n'||n, 'upload', 'v1', 1, 'committed',
                        'confirmed', 'committed', 0, 1, 0
                   FROM gen;",
                committed - 1
            ))
            .unwrap();
    }
    store
        .db
        .execute_batch(
            "INSERT INTO operations(id,node,kind,base,generation,state,outcome,
                                    outcome_category,next_retry_at,updated_at,reconcile_attempts)
             VALUES
               ('op-c1','n-c1','upload','v1',1,'conflict','conflict','server_rejected',0,10,1),
               ('op-r1','n-r1','upload','v1',1,'uncertain','unknown_retryable','transport',5,11,2),
               ('op-r2','n-r2','upload','v1',1,'uncertain','unknown_retryable','timeout',5,12,3),
               ('op-f1','n-f1','upload','v1',1,'uncertain','unknown_fatal','protocol',7,13,5),
               ('op-ab1','n-ab1','upload','v1',1,'abandoned','unknown_fatal',
                'reconcile_exhausted',0,14,5),
               ('op-pending','n-pending','upload','v1',1,'pending',NULL,NULL,0,0,0);
             INSERT INTO namespace_ops(id,node,body,state,kind,outcome,outcome_category,
                                       next_retry_at,updated_at,attempts)
             VALUES
               ('ns-c1','n-ns-c1','{}','conflict','rename','conflict','server_rejected',0,10,1),
               ('ns-r1','n-ns-r1','{}','uncertain','rename','unknown_retryable','transport',5,11,2),
               ('ns-ab1','n-ns-ab1','{}','abandoned','rename','unknown_fatal',
                'reconcile_exhausted',0,12,5);
             INSERT INTO transfers(id,node,version,direction,state,outcome,outcome_category,
                                   next_retry_at)
             VALUES
               ('tr-c1','n-tr-c1','v1','download','failed','conflict','server_rejected',0),
               ('tr-r1','n-tr-r1','v1','download','retry','unknown_retryable','transport',5);",
        )
        .unwrap();
}

fn fixture(committed: usize) -> (tempfile::TempDir, Store) {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(dir.path()).unwrap();
    seed(&store, committed);
    (dir, store)
}

/// One reference row in the historical view shape.
#[derive(Debug, PartialEq, Eq)]
struct LegacyRow {
    scope: String,
    operation: String,
    node: Option<String>,
    kind: String,
    state: String,
    outcome: String,
    category: String,
    last_error: Option<String>,
    attempts: u32,
    next_retry_at: i64,
    updated_at: i64,
}

fn legacy_rows(conn: &Connection, limit: usize) -> Vec<LegacyRow> {
    let mut statement = conn.prepare(LEGACY_LIST_SQL).unwrap();
    let rows = statement
        .query_map([i64::try_from(limit).unwrap()], |row| {
            Ok(LegacyRow {
                scope: row.get(0)?,
                operation: row.get(1)?,
                node: row.get(2)?,
                kind: row.get(3)?,
                state: row.get(4)?,
                outcome: row.get(5)?,
                category: row.get(6)?,
                last_error: row.get(7)?,
                attempts: u32::try_from(row.get::<_, i64>(8)?).unwrap_or(u32::MAX),
                next_retry_at: row.get(9)?,
                updated_at: row.get(10)?,
            })
        })
        .unwrap();
    rows.map(|row| row.unwrap()).collect()
}

fn rows_of(records: &[CommitDiagnosticRecord]) -> Vec<LegacyRow> {
    records
        .iter()
        .map(|record| LegacyRow {
            scope: record.scope.clone(),
            operation: record.operation.clone(),
            node: record.node.clone(),
            kind: record.kind.clone(),
            state: record.state.clone(),
            outcome: record.outcome.as_str().to_owned(),
            category: record.category.as_str().to_owned(),
            last_error: record.last_error.clone(),
            attempts: record.attempts,
            next_retry_at: record.next_retry_at,
            updated_at: record.updated_at,
        })
        .collect()
}

fn query_plan(conn: &Connection, sql: &str) -> Vec<String> {
    let mut statement = conn.prepare(&format!("EXPLAIN QUERY PLAN {sql}")).unwrap();
    let parameters = std::iter::repeat(rusqlite::types::Value::Null)
        .take(statement.parameter_count());
    statement
        .query_map(rusqlite::params_from_iter(parameters), |row| {
            row.get::<_, String>(3)
        })
        .unwrap()
        .map(|row| row.unwrap())
        .collect()
}

/// The snapshot must reproduce the historical counts and list byte-for-byte
/// (stable ordering, abandoned visible but not counted as attention).
#[test]
fn snapshot_matches_the_legacy_counts_and_ordering() {
    let (_dir, store) = fixture(3);
    let snapshot = snapshot(&store).unwrap();

    assert_eq!(
        snapshot.unknown_retryable,
        open_unknown(&store).unwrap(),
        "snapshot and standalone open_unknown must agree"
    );
    assert_eq!(snapshot.unknown_retryable, 4);
    assert_eq!(
        snapshot.unknown_fatal,
        needs_attention(&store).unwrap(),
        "snapshot and standalone needs_attention must agree"
    );
    assert_eq!(snapshot.unknown_fatal, 4);
    assert!(snapshot.consistent);
    assert_eq!(snapshot.open_unknown(), snapshot.unknown_retryable);
    assert_eq!(snapshot.needs_attention(), snapshot.unknown_fatal);

    let legacy = legacy_rows(&store.db, 20);
    assert_eq!(
        rows_of(&snapshot.records),
        legacy,
        "snapshot list must keep the historical ordering"
    );
    // The terminal abandoned row stays visible for the user, but it is not an
    // open unknown and does not force attention.
    assert!(snapshot.records.iter().any(|record| record.operation == "op-ab1"));
    assert!(snapshot.records.iter().any(|record| record.operation == "ns-ab1"));
    assert_eq!(snapshot.records.iter().filter(|r| r.state == "abandoned").count(), 2);
}

/// The overview serves diagnostics from a pooled read-only connection, so the
/// snapshot must work there and expose the data_version invalidation signal.
#[test]
fn snapshot_works_on_a_read_only_connection_and_tracks_commits() {
    let (dir, store) = fixture(3);
    let reader = Store::open_read_only(dir.path()).unwrap();

    let first = snapshot(&reader).unwrap();
    assert!(first.consistent);
    assert_eq!(first.unknown_retryable, 4);
    assert_eq!(first.unknown_fatal, 4);
    assert_eq!(first.data_version, reader.data_version().unwrap());

    // A writer commit is visible to the read-only connection only through
    // PRAGMA data_version; the next snapshot must both see it and report the new
    // version, which is what the desktop cache keys on.
    store
        .db
        .execute(
            "UPDATE operations SET outcome='unknown_fatal', outcome_category='transport',
                    state='uncertain' WHERE id='op-r1'",
            [],
        )
        .unwrap();
    let moved = reader.data_version().unwrap();
    assert_ne!(moved, first.data_version, "writer commit must change data_version");
    let second = snapshot(&reader).unwrap();
    assert!(second.consistent);
    assert_eq!(second.data_version, moved);
    assert_eq!(second.unknown_retryable, 3);
    assert_eq!(second.unknown_fatal, 5);

    // Restart the writer handle: the snapshot is computed from durable rows, so
    // it must not depend on process-local state.
    drop(store);
    let reopened = Store::open(dir.path()).unwrap();
    let after_restart = snapshot(&reopened).unwrap();
    assert_eq!(after_restart.unknown_retryable, 3);
    assert_eq!(after_restart.unknown_fatal, 5);
    assert_eq!(
        rows_of(&after_restart.records),
        legacy_rows(&reopened.db, 20)
    );
}

/// Red target: with only the shared view indexes, the count/list statements walk
/// the confirmed history; after the fix each branch is served by a dedicated
/// partial diagnostics index and the successful history is not scanned.
#[test]
fn diagnostics_queries_use_dedicated_indexes() {
    let (_dir, store) = fixture(5_000);
    let conn = &store.db;

    let counts = query_plan(conn, DIAGNOSTICS_COUNTS_SQL);
    assert!(
        counts.iter().any(|line| line.contains("operations_diagnostics")),
        "#88 count query must use the operations diagnostics index: {counts:?}"
    );
    assert!(
        !counts.iter().any(|line| line.contains("SCAN operations")),
        "#88 count query must not scan the operations history: {counts:?}"
    );
    assert!(
        !counts.iter().any(|line| line.contains("SCAN namespace_ops")),
        "#88 count query must not scan the namespace history: {counts:?}"
    );

    let list = query_plan(conn, diagnostics_list_sql());
    assert!(
        list.iter().any(|line| line.contains("operations_diagnostics")),
        "#88 list query must use the operations diagnostics index: {list:?}"
    );
    assert!(
        !list.iter().any(|line| line.contains("SCAN operations")),
        "#88 list query must not scan the operations history: {list:?}"
    );
}

/// Issue #88 scale evidence: keep the anomaly set fixed, grow the retained
/// confirmed history 1k -> 100k, and compare the fixed snapshot against the
/// historical view queries. Run with `--ignored --nocapture`.
#[test]
#[ignore = "manual scaling evidence; run on the remote builder"]
fn snapshot_scale_is_history_independent() {
    for committed in [1_000usize, 10_000, 100_000] {
        let (_dir, store) = fixture(committed);
        let legacy_started = std::time::Instant::now();
        let legacy_counts: (i64, i64) = store
            .db
            .query_row(LEGACY_COUNTS_SQL, [], |row| Ok((row.get(0)?, row.get(1)?)))
            .unwrap();
        let legacy_records = legacy_rows(&store.db, 20);
        let legacy_us = legacy_started.elapsed().as_micros();

        let snapshot_started = std::time::Instant::now();
        let snapshot = snapshot(&store).unwrap();
        let snapshot_us = snapshot_started.elapsed().as_micros();

        assert_eq!(
            (snapshot.unknown_retryable, snapshot.unknown_fatal),
            legacy_counts
        );
        assert_eq!(rows_of(&snapshot.records), legacy_records);
        let counts_plan = query_plan(&store.db, DIAGNOSTICS_COUNTS_SQL);
        let list_plan = query_plan(&store.db, diagnostics_list_sql());
        eprintln!(
            "DIAGNOSTICS_CURVE committed={committed} legacy_us={legacy_us} snapshot_us={snapshot_us} \
             open_unknown={} needs_attention={} records={} \
             counts_plan_uses_index={} list_plan_uses_index={}",
            snapshot.unknown_retryable,
            snapshot.unknown_fatal,
            snapshot.records.len(),
            counts_plan.iter().any(|line| line.contains("_diagnostics")),
            list_plan.iter().any(|line| line.contains("_diagnostics")),
        );
        eprintln!("DIAGNOSTICS_CURVE counts_plan={counts_plan:?}");
        eprintln!("DIAGNOSTICS_CURVE list_plan={list_plan:?}");
    }
}
