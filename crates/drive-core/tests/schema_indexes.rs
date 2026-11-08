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
        "ON operations(node) WHERE state IN ('pending','inflight','uncertain','conflict')",
    ),
    (
        "operations_blob_active",
        "ON operations(blob) WHERE blob IS NOT NULL AND state IN ('pending','inflight','uncertain','conflict')",
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
        "ON operations(node) WHERE state IN ('pending','inflight','uncertain','conflict')",
    ),
    (
        "operations_blob_active",
        "ON operations(blob) WHERE blob IS NOT NULL AND state IN ('pending','inflight','uncertain','conflict')",
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
    assert_plan_uses(
        &db,
        "SELECT EXISTS(SELECT 1 FROM operations WHERE node=?1 AND state IN ('pending','inflight','uncertain','conflict'))",
        "operations_node_active",
    );
    assert_plan_uses(
        &db,
        "SELECT EXISTS(SELECT 1 FROM operations WHERE blob=?1 AND state IN ('pending','inflight','uncertain','conflict'))",
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
