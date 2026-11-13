use drive_core::{STATE_SCHEMA_VERSION, Store};
use rusqlite::Connection;
use std::path::Path;

// The minimal indexes issue #15/#16 relies on: candidate scans for conflicts,
// cached/stale rows, pinned nodes and per-node verification of unfinished
// operations. `nodes_cached` carries the version columns so the stale-download
// branch is answered without touching the wide nodes table.
//
// Issue #32 adds the storage-residual indexes: the pending-operation counter
// must only read active rows, per-node operation probes must target active
// states, blob protection must find operations by blob, uncertain upload
// reconciliation must be a range scan, the inbox must skip applied history and
// sync history filters must stay ordered by sequence.
const CANDIDATE_INDEXES: [(&str, &str); 12] = [
    ("nodes_conflict", "WHERE conflict=1"),
    ("nodes_pinned", "WHERE pinned=1"),
    (
        "nodes_cached",
        "ON nodes(id,local_version,remote_version) WHERE blob IS NOT NULL",
    ),
    ("operations_node", "ON operations(node)"),
    ("namespace_ops_node", "ON namespace_ops(node)"),
    (
        "operations_countable",
        "ON operations(id) WHERE state IN ('pending','inflight','uncertain')",
    ),
    (
        "operations_node_active",
        "ON operations(node) WHERE state IN ('pending','inflight','uncertain','conflict','abandoned')",
    ),
    (
        "operations_blob_active",
        "ON operations(blob) WHERE blob IS NOT NULL AND state IN ('pending','inflight','uncertain','conflict','abandoned')",
    ),
    (
        "operations_uncertain_reconcile",
        "ON operations(reconcile_at) WHERE kind='upload' AND state='uncertain'",
    ),
    ("inbox_pending", "ON inbox(seq) WHERE done=0"),
    (
        "sync_history_mount_seq",
        "ON sync_history(mount,seq DESC)",
    ),
    (
        "sync_history_result_seq",
        "ON sync_history(result,seq DESC)",
    ),
];

/// Index definitions in build order for the upgrade measurement.
const INDEX_DEFINITIONS: [(&str, &str); 12] = [
    ("nodes_conflict", "ON nodes(id) WHERE conflict=1"),
    ("nodes_pinned", "ON nodes(id) WHERE pinned=1"),
    (
        "nodes_cached",
        "ON nodes(id,local_version,remote_version) WHERE blob IS NOT NULL",
    ),
    ("operations_node", "ON operations(node)"),
    ("namespace_ops_node", "ON namespace_ops(node)"),
    (
        "operations_countable",
        "ON operations(id) WHERE state IN ('pending','inflight','uncertain')",
    ),
    (
        "operations_node_active",
        "ON operations(node) WHERE state IN ('pending','inflight','uncertain','conflict','abandoned')",
    ),
    (
        "operations_blob_active",
        "ON operations(blob) WHERE blob IS NOT NULL AND state IN ('pending','inflight','uncertain','conflict','abandoned')",
    ),
    (
        "operations_uncertain_reconcile",
        "ON operations(reconcile_at) WHERE kind='upload' AND state='uncertain'",
    ),
    ("inbox_pending", "ON inbox(seq) WHERE done=0"),
    (
        "sync_history_mount_seq",
        "ON sync_history(mount,seq DESC)",
    ),
    (
        "sync_history_result_seq",
        "ON sync_history(result,seq DESC)",
    ),
];

fn database(root: &Path) -> std::path::PathBuf {
    root.join("state.sqlite")
}

fn user_version(db: &Connection) -> i64 {
    db.query_row("PRAGMA user_version", [], |r| r.get(0))
        .unwrap()
}

fn index_sql(db: &Connection, name: &str) -> Option<String> {
    db.query_row(
        "SELECT sql FROM sqlite_master WHERE type='index' AND name=?1",
        [name],
        |r| r.get(0),
    )
    .ok()
}

fn drop_new_indexes(db: &Connection) {
    db.execute_batch(
        "DROP INDEX IF EXISTS nodes_conflict;
         DROP INDEX IF EXISTS nodes_pinned;
         DROP INDEX IF EXISTS nodes_cached;
         DROP INDEX IF EXISTS operations_node;
         DROP INDEX IF EXISTS namespace_ops_node;
         DROP INDEX IF EXISTS operations_countable;
         DROP INDEX IF EXISTS operations_node_active;
         DROP INDEX IF EXISTS operations_blob_active;
         DROP INDEX IF EXISTS operations_uncertain_reconcile;
         DROP INDEX IF EXISTS inbox_pending;
         DROP INDEX IF EXISTS sync_history_mount_seq;
         DROP INDEX IF EXISTS sync_history_result_seq;",
    )
    .unwrap();
}

fn query_plan(db: &Connection, sql: &str) -> Vec<String> {
    let mut statement = db.prepare(&format!("EXPLAIN QUERY PLAN {sql}")).unwrap();
    // The production statements carry positional parameters. SQLite refuses to
    // prepare an unbound parameterised query, so bind NULL for every slot:
    // parameter values never change the chosen plan, only the affinity does.
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

fn assert_plan_uses(db: &Connection, sql: &str, index: &str) {
    let plan = query_plan(db, sql);
    assert!(
        plan.iter().any(|line| line.contains(index)
            || line.contains(&format!("sqlite_autoindex_{index}"))),
        "{index} not used by {sql}: {plan:?}"
    );
}

/// Simulates the pre-fix database at `nodes` rows and times the v27 upgrade.
/// This is a manual measurement (run with `--ignored --nocapture`), not a
/// performance gate: it only prints numbers and asserts the upgrade succeeds.
#[test]
#[ignore = "manual measurement over a 294k-node database"]
fn v27_upgrade_cost_on_a_294k_node_database() {
    const NODES: usize = 294_000;
    let d = tempfile::tempdir().unwrap();
    drop(Store::open(d.path()).unwrap());
    let db = Connection::open(database(d.path())).unwrap();
    drop_new_indexes(&db);
    db.execute_batch("BEGIN IMMEDIATE;").unwrap();
    {
        // Every tenth node has cached bytes with a stale local version so the
        // partial `nodes_cached` index is not empty in this measurement.
        let mut insert = db
            .prepare(
                "INSERT INTO nodes(id,parent,name,remote_version,directory,local_version,blob,conflict,pinned)
                 VALUES(?1,?2,?3,'v1',0,?4,?5,?6,?7)",
            )
            .unwrap();
        for i in 0..NODES {
            let parent = if i == 0 {
                None
            } else {
                Some(format!("n{:07}", (i - 1) / 16))
            };
            let cached = i % 10 == 0;
            insert
                .execute(rusqlite::params![
                    format!("n{i:07}"),
                    parent,
                    format!("file-{i}"),
                    if cached { Some("v0") } else { None },
                    if cached {
                        Some(format!("blob{i:07}"))
                    } else {
                        None
                    },
                    i64::from(i % 1000 == 1),
                    i64::from(i % 500 == 0),
                ])
                .unwrap();
        }
        let mut operation = db
            .prepare(
                "INSERT INTO operations(id,node,kind,base,generation,state)
                 VALUES(?1,?2,'upload','v1',1,?3)",
            )
            .unwrap();
        for i in 0..NODES {
            operation
                .execute(rusqlite::params![
                    format!("op{i:07}"),
                    format!("n{i:07}"),
                    if i % 10_000 == 0 { "uncertain" } else { "committed" },
                ])
                .unwrap();
        }
        let mut namespace = db
            .prepare(
                "INSERT INTO namespace_ops(id,node,body,state,kind)
                 VALUES(?1,?2,'{}',?3,'rename')",
            )
            .unwrap();
        for i in 0..NODES {
            namespace
                .execute(rusqlite::params![
                    format!("ns{i:07}"),
                    format!("n{i:07}"),
                    if i % 20_000 == 0 {
                        "uncertain"
                    } else {
                        "committed"
                    },
                ])
                .unwrap();
        }
    }
    db.pragma_update(None, "user_version", STATE_SCHEMA_VERSION - 1)
        .unwrap();
    db.execute_batch("COMMIT; PRAGMA wal_checkpoint(TRUNCATE);")
        .unwrap();
    drop(db);

    // Component timings on a copy: backup copy plus each new index build.
    let copy = tempfile::tempdir().unwrap();
    std::fs::copy(database(d.path()), database(copy.path())).unwrap();
    let db = Connection::open(database(copy.path())).unwrap();
    let backup_path = copy.path().join("backup.sqlite");
    let started = std::time::Instant::now();
    db.execute("VACUUM INTO ?1", [backup_path.to_string_lossy().as_ref()])
        .unwrap();
    eprintln!("v27 backup VACUUM INTO: {:?}", started.elapsed());
    for (name, statement) in INDEX_DEFINITIONS {
        let started = std::time::Instant::now();
        db.execute_batch(&format!("CREATE INDEX {name} {statement};"))
            .unwrap();
        eprintln!("v27 index {name}: {:?}", started.elapsed());
    }
    drop(db);

    // End-to-end upgrade through Store::open (backup + migrations + indexes).
    let started = std::time::Instant::now();
    let store = Store::open(d.path()).unwrap();
    eprintln!("v27 Store::open upgrade total: {:?}", started.elapsed());
    let db = Connection::open(database(d.path())).unwrap();
    assert_eq!(user_version(&db), STATE_SCHEMA_VERSION);
    for (name, _) in CANDIDATE_INDEXES {
        assert!(index_sql(&db, name).is_some(), "missing index {name}");
    }
    drop(db);
    drop(store);
}

#[test]
fn fresh_database_carries_the_candidate_indexes() {
    let d = tempfile::tempdir().unwrap();
    let store = Store::open(d.path()).unwrap();
    let db = Connection::open(database(d.path())).unwrap();
    assert_eq!(user_version(&db), STATE_SCHEMA_VERSION);
    for (name, definition) in CANDIDATE_INDEXES {
        let sql = index_sql(&db, name).unwrap_or_else(|| panic!("missing index {name}"));
        assert!(sql.contains(definition), "{name}: {sql}");
    }
    drop(db);
    drop(store);
}

#[test]
fn queries_select_the_v28_indexes() {
    let d = tempfile::tempdir().unwrap();
    let store = Store::open(d.path()).unwrap();
    let db = Connection::open(database(d.path())).unwrap();

    // The planner needs realistic cardinalities to prefer a partial index over
    // a table scan, so seed a few thousand terminal rows plus a small active
    // tail first. The rows are small and the test database is discarded.
    db.execute_batch(
        "INSERT INTO operations(id,node,kind,base,generation,state,reconcile_at)
         WITH RECURSIVE generate(n) AS (SELECT 0 UNION ALL SELECT n+1 FROM generate WHERE n < 4999)
         SELECT 'op'||n, 'n'||n, 'upload', 'v1', 1,
                CASE WHEN n % 1000 = 0 THEN 'uncertain' ELSE 'committed' END,
                n
         FROM generate;
         INSERT INTO inbox(seq,body,done)
         WITH RECURSIVE generate(n) AS (SELECT 1 UNION ALL SELECT n+1 FROM generate WHERE n < 200)
         SELECT n, '{\"inbox\":'||n||'}', CASE WHEN n < 196 THEN 1 ELSE 0 END FROM generate;
         INSERT INTO sync_history(seq,mount,name,path_at_event,action,result,completed_at,operation_key)
         WITH RECURSIVE generate(n) AS (SELECT 1 UNION ALL SELECT n+1 FROM generate WHERE n < 200)
         SELECT n, 'm', 'name-'||n, '/path-'||n, 'upload',
                CASE WHEN n % 2 = 0 THEN 'ok' ELSE 'failed' END,
                n, 'key-'||n
         FROM generate;",
    )
    .unwrap();

    // `pending_operations` must read only active rows, not the terminal history.
    assert_plan_uses(
        &db,
        "SELECT count(*) FROM operations WHERE state IN ('pending','inflight','uncertain')",
        "operations_countable",
    );
    // Per-node operation probes (conflicts, blob protection) target active rows.
    // Issue #91: the protection set is five-valued, so the production predicates
    // mirror the abandoned state as well.
    assert_plan_uses(
        &db,
        "SELECT EXISTS(SELECT 1 FROM operations WHERE node=?1 AND state IN ('pending','inflight','uncertain','conflict','abandoned'))",
        "operations_node_active",
    );
    assert_plan_uses(
        &db,
        "SELECT EXISTS(SELECT 1 FROM operations WHERE blob=?1 AND state IN ('pending','inflight','uncertain','conflict','abandoned'))",
        "operations_blob_active",
    );
    // Uncertain upload reconciliation is a range scan over the retry clock.
    // The statement intentionally mirrors `upload_reconcile_operations`: the
    // `ORDER BY reconcile_at` (not `rowid`) is what lets the partial index serve
    // the range without a table scan or a temporary sort.
    assert_plan_uses(
        &db,
        "SELECT id FROM operations WHERE kind='upload' AND node NOT IN (SELECT id FROM access_blocked) AND state='uncertain' AND reconcile_attempts < ?1 AND reconcile_at <= ?2 ORDER BY reconcile_at",
        "operations_uncertain_reconcile",
    );
    // apply_next must skip the applied inbox prefix.
    assert_plan_uses(
        &db,
        "SELECT seq,body FROM inbox WHERE done=0 ORDER BY seq LIMIT 1",
        "inbox_pending",
    );
    // Sync history filters stay ordered by sequence instead of sorting.
    assert_plan_uses(
        &db,
        "SELECT id FROM sync_history WHERE mount=?1 ORDER BY seq DESC, id DESC LIMIT ?2",
        "sync_history_mount_seq",
    );
    assert_plan_uses(
        &db,
        "SELECT id FROM sync_history WHERE result=?1 ORDER BY seq DESC, id DESC LIMIT ?2",
        "sync_history_result_seq",
    );
    drop(db);
    drop(store);
}

#[test]
fn previous_schema_upgrades_to_the_current_version_with_backup_and_indexes() {
    let d = tempfile::tempdir().unwrap();
    drop(Store::open(d.path()).unwrap());
    // Simulate the schema before this fix: the candidate indexes did not exist.
    let db = Connection::open(database(d.path())).unwrap();
    drop_new_indexes(&db);
    db.pragma_update(None, "user_version", STATE_SCHEMA_VERSION - 1)
        .unwrap();
    drop(db);

    let store = Store::open(d.path()).unwrap();
    let db = Connection::open(database(d.path())).unwrap();
    assert_eq!(user_version(&db), STATE_SCHEMA_VERSION);
    for (name, definition) in CANDIDATE_INDEXES {
        let sql = index_sql(&db, name).unwrap_or_else(|| panic!("missing index {name}"));
        assert!(sql.contains(definition), "{name}: {sql}");
    }
    drop(db);

    // Opening again is idempotent: the version and indexes stay in place.
    drop(store);
    let store = Store::open(d.path()).unwrap();
    let db = Connection::open(database(d.path())).unwrap();
    assert_eq!(user_version(&db), STATE_SCHEMA_VERSION);
    for (name, _) in CANDIDATE_INDEXES {
        assert!(index_sql(&db, name).is_some(), "missing index after reopen {name}");
    }
    drop(db);

    // The pre-upgrade copy keeps the old version and stays integrity-checkable.
    let prefix = format!("state-before-v{}-", STATE_SCHEMA_VERSION);
    let backup = std::fs::read_dir(d.path())
        .unwrap()
        .map(|e| e.unwrap().path())
        .find(|p| p.file_name().unwrap().to_string_lossy().starts_with(&prefix))
        .unwrap();
    let db = Connection::open(backup).unwrap();
    assert_eq!(user_version(&db), STATE_SCHEMA_VERSION - 1);
    assert_eq!(
        db.query_row("PRAGMA integrity_check", [], |r| r.get::<_, String>(0))
            .unwrap(),
        "ok"
    );
    drop(db);
    drop(store);
}

/// Issue #67 W2 follow-up (`notes/W2_INDEX_ANALYSIS.md`): three `sync_history`
/// indexes are maintained by every append/eviction but no runtime query can use
/// them — the paging query's `? IS NULL OR column=?` form defeats filtered
/// index use, the name filter is an `instr()` substring, and nothing filters
/// mount/result by completion time. They must not be installed on a fresh store.
#[test]
fn unused_sync_history_indexes_are_not_installed() {
    let d = tempfile::tempdir().unwrap();
    let store = Store::open(d.path()).unwrap();
    let db = Connection::open(database(d.path())).unwrap();
    for name in [
        "sync_history_name",
        "sync_history_mount_time",
        "sync_history_result",
    ] {
        assert!(
            index_sql(&db, name).is_none(),
            "{name} is not used by any runtime query and must not be installed"
        );
    }
    // The age-eviction index and the two candidate indexes the #32 plan tests
    // rely on stay installed.
    for name in [
        "sync_history_time",
        "sync_history_mount_seq",
        "sync_history_result_seq",
    ] {
        assert!(
            index_sql(&db, name).is_some(),
            "missing required index {name}"
        );
    }
    drop(db);
    drop(store);
}

/// An existing store may still carry the three unused indexes from an older
/// binary. Opening it with this build drops them; index removal is
/// data-neutral, so the retained history rows must be untouched.
#[test]
fn store_upgrade_drops_legacy_unused_sync_history_indexes_and_keeps_rows() {
    let d = tempfile::tempdir().unwrap();
    drop(Store::open(d.path()).unwrap());
    let db = Connection::open(database(d.path())).unwrap();
    db.execute_batch(
        "INSERT INTO sync_history(seq,mount,name,path_at_event,action,result,completed_at,operation_key)
         VALUES (1,'server','a','p/a','remote_change','ok',unixepoch(),'remote:e1'),
                (2,'server','b','p/b','remote_change','ok',unixepoch(),'remote:e2');
         CREATE INDEX IF NOT EXISTS sync_history_mount_time ON sync_history(mount,completed_at DESC);
         CREATE INDEX IF NOT EXISTS sync_history_name ON sync_history(name COLLATE NOCASE);
         CREATE INDEX IF NOT EXISTS sync_history_result ON sync_history(result,completed_at DESC);",
    )
    .unwrap();
    drop(db);

    let store = Store::open(d.path()).unwrap();
    let db = Connection::open(database(d.path())).unwrap();
    for name in [
        "sync_history_name",
        "sync_history_mount_time",
        "sync_history_result",
    ] {
        assert!(
            index_sql(&db, name).is_none(),
            "legacy index {name} must be dropped when the store is opened"
        );
    }
    let rows: i64 = db
        .query_row("SELECT COUNT(*) FROM sync_history", [], |r| r.get(0))
        .unwrap();
    assert_eq!(rows, 2, "dropping indexes must not lose retained history rows");
    drop(db);
    drop(store);
}

// ---- issue #91: abandoned is part of the active protection set --------------

/// The cache-protection state set is five-valued: `cache_policy` protects
/// `pending`,`inflight`,`uncertain`,`conflict` **and** `abandoned` (the upload
/// result is unknown, so the blob may be the only local copy). A four-state
/// partial index cannot answer that predicate completely, so the planner falls
/// back to the full-history `operations_node` index or a table scan.
fn assert_active_indexes_cover_abandoned(db: &Connection) {
    for name in ["operations_node_active", "operations_blob_active"] {
        let sql = index_sql(db, name).unwrap_or_else(|| panic!("missing index {name}"));
        assert!(
            sql.contains("'abandoned'"),
            "#91: {name} must cover the five-state protection set: {sql}"
        );
    }
}

#[test]
fn fresh_database_active_indexes_cover_abandoned() {
    let d = tempfile::tempdir().unwrap();
    let store = Store::open(d.path()).unwrap();
    let db = Connection::open(database(d.path())).unwrap();
    assert_active_indexes_cover_abandoned(&db);
    drop(db);
    drop(store);
}

/// An existing profile that already reached the current schema version still
/// carries the old four-state indexes. `CREATE INDEX IF NOT EXISTS` cannot
/// repair them, so `Store::open` must reconcile the definition explicitly.
/// Reopening must be idempotent and must not lose rows.
#[test]
fn legacy_four_state_indexes_are_migrated_on_open() {
    let d = tempfile::tempdir().unwrap();
    drop(Store::open(d.path()).unwrap());
    let db = Connection::open(database(d.path())).unwrap();
    db.execute_batch(
        "DROP INDEX IF EXISTS operations_node_active;
         DROP INDEX IF EXISTS operations_blob_active;
         CREATE INDEX operations_node_active ON operations(node)
             WHERE state IN ('pending','inflight','uncertain','conflict');
         CREATE INDEX operations_blob_active ON operations(blob)
             WHERE blob IS NOT NULL AND state IN ('pending','inflight','uncertain','conflict');
         INSERT INTO operations(id,node,kind,base,generation,state)
             VALUES('legacy-op-1','legacy-node','upload','v1',1,'abandoned');",
    )
    .unwrap();
    assert!(!index_sql(&db, "operations_node_active").unwrap().contains("'abandoned'"));
    drop(db);

    let store = Store::open(d.path()).unwrap();
    let db = Connection::open(database(d.path())).unwrap();
    assert_active_indexes_cover_abandoned(&db);
    let rows: i64 = db
        .query_row("SELECT COUNT(*) FROM operations", [], |r| r.get(0))
        .unwrap();
    assert_eq!(rows, 1, "index migration must not drop operations rows");
    drop(db);
    drop(store);

    // Idempotent: a second open sees the reconciled definition and keeps it.
    let store = Store::open(d.path()).unwrap();
    let db = Connection::open(database(d.path())).unwrap();
    assert_active_indexes_cover_abandoned(&db);
    drop(db);
    drop(store);
}

/// The production five-state predicates (operations node and blob protection,
/// and the cache candidate guard) must be answered by the partial indexes, not
/// by scanning committed operation history.
#[test]
fn five_state_protection_queries_use_the_active_indexes() {
    let d = tempfile::tempdir().unwrap();
    let store = Store::open(d.path()).unwrap();
    let db = Connection::open(database(d.path())).unwrap();
    db.execute_batch(
        "INSERT INTO operations(id,node,kind,base,generation,state,blob)
         WITH RECURSIVE generate(n) AS (SELECT 0 UNION ALL SELECT n+1 FROM generate WHERE n < 4999)
         SELECT 'op'||n, 'n'||n, 'upload', 'v1', 1,
                CASE WHEN n % 1000 = 0 THEN 'uncertain' ELSE 'committed' END,
                NULL
         FROM generate;
         INSERT INTO operations(id,node,kind,base,generation,state,blob)
         VALUES('abandoned-op','abandoned-node','upload','v1',1,'abandoned','abandoned-blob');",
    )
    .unwrap();
    assert_plan_uses(
        &db,
        "SELECT EXISTS(SELECT 1 FROM operations WHERE node=?1 AND state IN ('pending','inflight','uncertain','conflict','abandoned'))",
        "operations_node_active",
    );
    assert_plan_uses(
        &db,
        "SELECT EXISTS(SELECT 1 FROM operations WHERE blob=?1 AND state IN ('pending','inflight','uncertain','conflict','abandoned'))",
        "operations_blob_active",
    );
    // `CACHE_CANDIDATES_SQL`'s ops_node CTE: DISTINCT over the five-state set.
    assert_plan_uses(
        &db,
        "SELECT DISTINCT node FROM operations WHERE node IS NOT NULL AND state IN ('pending','inflight','uncertain','conflict','abandoned')",
        "operations_node_active",
    );
    assert_plan_uses(
        &db,
        "SELECT DISTINCT blob FROM operations WHERE blob IS NOT NULL AND state IN ('pending','inflight','uncertain','conflict','abandoned')",
        "operations_blob_active",
    );
    drop(db);
    drop(store);
}

/// Issue #91 scale evidence: fix the five-state protection-set size and grow the
/// committed history to 100k. The protection branches must stay index-bound
/// instead of walking the history. Run explicitly via `--ignored --nocapture`.
#[test]
#[ignore = "manual query-plan/scale evidence; run on the remote builder"]
fn five_state_protection_plan_is_history_independent_at_100k() {
    let d = tempfile::tempdir().unwrap();
    let store = Store::open(d.path()).unwrap();
    let db = Connection::open(database(d.path())).unwrap();
    db.execute_batch(
        "BEGIN;
         INSERT INTO operations(id,node,kind,base,generation,state,blob)
         WITH RECURSIVE generate(n) AS (SELECT 0 UNION ALL SELECT n+1 FROM generate WHERE n < 99999)
         SELECT 'op'||n, 'n'||n, 'upload', 'v1', 1,
                CASE WHEN n % 250 = 0 THEN 'abandoned' ELSE 'committed' END,
                CASE WHEN n % 250 = 0 THEN 'blob'||n ELSE NULL END
         FROM generate;
         INSERT INTO operations(id,node,kind,base,generation,state,blob)
         SELECT 'live'||n,'n'||n,'upload','v1',1,'pending','live-blob'||n
         FROM generate WHERE n < 40;
         COMMIT;",
    )
    .unwrap();
    for (label, sql) in [
        (
            "ops_node",
            "SELECT DISTINCT node FROM operations WHERE node IS NOT NULL AND state IN ('pending','inflight','uncertain','conflict','abandoned')",
        ),
        (
            "protected_blobs",
            "SELECT DISTINCT blob FROM operations WHERE blob IS NOT NULL AND state IN ('pending','inflight','uncertain','conflict','abandoned')",
        ),
    ] {
        let plan = query_plan(&db, sql);
        eprintln!("PROTECTION_PLAN {label}: {plan:?}");
        assert!(
            plan.iter().any(|line| line.contains("_active")),
            "{label}: five-state branch must use the active partial index: {plan:?}"
        );
        assert!(
            !plan.iter().any(|line| line.contains("SCAN operations")),
            "{label}: five-state branch must not scan history: {plan:?}"
        );
    }
    drop(db);
    drop(store);
}
