use drive_core::{STATE_SCHEMA_VERSION, Store};
use rusqlite::Connection;
use std::path::Path;

// The minimal indexes issue #15/#16 relies on: candidate scans for conflicts,
// cached/stale rows, pinned nodes and per-node verification of unfinished
// operations. `nodes_cached` carries the version columns so the stale-download
// branch is answered without touching the wide nodes table.
const CANDIDATE_INDEXES: [(&str, &str); 5] = [
    ("nodes_conflict", "WHERE conflict=1"),
    ("nodes_pinned", "WHERE pinned=1"),
    (
        "nodes_cached",
        "ON nodes(id,local_version,remote_version) WHERE blob IS NOT NULL",
    ),
    ("operations_node", "ON operations(node)"),
    ("namespace_ops_node", "ON namespace_ops(node)"),
];

/// Index definitions in build order for the upgrade measurement.
const INDEX_DEFINITIONS: [(&str, &str); 5] = [
    ("nodes_conflict", "ON nodes(id) WHERE conflict=1"),
    ("nodes_pinned", "ON nodes(id) WHERE pinned=1"),
    (
        "nodes_cached",
        "ON nodes(id,local_version,remote_version) WHERE blob IS NOT NULL",
    ),
    ("operations_node", "ON operations(node)"),
    ("namespace_ops_node", "ON namespace_ops(node)"),
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
         DROP INDEX IF EXISTS namespace_ops_node;",
    )
    .unwrap();
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
