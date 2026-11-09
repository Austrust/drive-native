//! Schema 29 migration contract (#9 TB1).
//!
//! The migration is deliberately behaviour-free: it adds the commit-diagnostic
//! columns (#9), the recovery journal (#10) and the platform state push queue
//! (#11), backfills diagnoses for already-submitted legacy rows, and leaves
//! every existing journal state machine untouched.

use drive_core::*;
use rusqlite::Connection;

fn columns(db: &Connection, table: &str) -> Vec<String> {
    let mut query = db
        .prepare(&format!("PRAGMA table_info({table})"))
        .expect("table_info");
    query
        .query_map([], |row| row.get::<_, String>(1))
        .expect("columns")
        .map(|column| column.expect("column name"))
        .collect()
}

fn has_table(db: &Connection, name: &str) -> bool {
    db.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE name=?1 AND type IN ('table','view')",
        [name],
        |row| row.get::<_, i64>(0),
    )
    .expect("sqlite_master")
        == 1
}

fn user_version(db: &Connection) -> i64 {
    db.query_row("PRAGMA user_version", [], |row| row.get::<_, i64>(0))
        .expect("user_version")
}

fn state_epoch() -> i64 {
    1_800_000_000
}

/// Build a real v28 `state.sqlite` by hand. We cannot downgrade a v29 store by
/// dropping columns (SQLite refuses to drop a column referenced by its own
/// CHECK constraint), so the fixture writes the v28 column sets directly and
/// lets `Store::open` perform the upgrade exactly like a shipped v28 install.
fn write_v28_fixture(root: &std::path::Path) {
    let db = Connection::open(root.join("state.sqlite")).expect("open fixture");
    db.execute_batch(
        "CREATE TABLE nodes(
            id TEXT PRIMARY KEY, parent TEXT, name TEXT NOT NULL,
            remote_version TEXT NOT NULL, directory INTEGER NOT NULL,
            local_version TEXT, blob TEXT, dirty_blob TEXT, base_version TEXT,
            dirty_generation INTEGER NOT NULL DEFAULT 0,
            pinned INTEGER NOT NULL DEFAULT 0, deleted INTEGER NOT NULL DEFAULT 0,
            conflict INTEGER NOT NULL DEFAULT 0);
         CREATE TABLE operations(
            id TEXT PRIMARY KEY, node TEXT, kind TEXT NOT NULL, base TEXT, blob TEXT,
            generation INTEGER NOT NULL, state TEXT NOT NULL DEFAULT 'pending',
            reconcile_attempts INTEGER NOT NULL DEFAULT 0,
            reconcile_at INTEGER NOT NULL DEFAULT 0);
         CREATE TABLE transfers(
            id TEXT PRIMARY KEY, node TEXT NOT NULL, version TEXT NOT NULL,
            direction TEXT NOT NULL, state TEXT NOT NULL, error TEXT,
            UNIQUE(node,version,direction));
         CREATE TABLE namespace_ops(
            seq INTEGER PRIMARY KEY AUTOINCREMENT, id TEXT NOT NULL UNIQUE,
            node TEXT, body TEXT NOT NULL, state TEXT NOT NULL,
            kind TEXT NOT NULL DEFAULT 'rename');
         CREATE TABLE metadata(key TEXT PRIMARY KEY, value TEXT NOT NULL);
         -- A real v28 store has already run its one-off storage cleanup; the
         -- durable markers keep the upgrade from pruning the seeded history
         -- rows below (startup prune only runs once per store version).
         INSERT INTO metadata(key,value) VALUES
            ('storage_v28_pruned','28'),
            ('storage_v28_committed_deduped','28');
         INSERT INTO nodes(id,parent,name,remote_version,directory)
            VALUES('root',NULL,'root','1',1),('node1','root','A.txt','v1',0);
         INSERT INTO operations(id,node,kind,base,blob,generation,state,reconcile_attempts,reconcile_at) VALUES
            ('op-pending','node1','upload',NULL,NULL,0,'pending',0,0),
            ('op-uncertain-upload','node1','upload','v1',NULL,1,'uncertain',0,0),
            ('op-uncertain-exhausted','node1','upload','v1',NULL,1,'uncertain',5,999),
            ('op-inflight','node1','remove',NULL,NULL,1,'inflight',0,0),
            ('op-committed','node1','upload','v1',NULL,1,'committed',0,0),
            ('op-conflict','node1','upload','v1',NULL,1,'conflict',0,0),
            ('op-superseded','node1','upload','v1',NULL,1,'superseded',0,0),
            -- A #14-era terminal state can already be present in a v29 store.
            -- TB1 has no abandoned terminal and leaves its mapping to TB2.
            ('op-abandoned','node1','upload','v1',NULL,1,'abandoned',0,0);
         INSERT INTO transfers(id,node,version,direction,state,error) VALUES
            ('tr-pending','node1','v1','download','pending',NULL),
            ('tr-inflight','node1','v2','download','inflight',NULL),
            ('tr-retry','node1','v3','download','retry',
             '{\"schema\":1,\"kind\":\"transport\",\"attempts\":2,\"retry_at\":1234}'),
            ('tr-failed','node1','v4','download','failed',
             '{\"schema\":1,\"kind\":\"server_rejected\",\"attempts\":5,\"retry_at\":0}');
         INSERT INTO namespace_ops(id,node,body,state,kind) VALUES
            ('ns-pending','node1','{}','pending','rename'),
            ('ns-inflight','node1','{}','inflight','rename'),
            ('ns-uncertain','node1','{}','uncertain','rename'),
            ('ns-cleanup-uncertain','node1','{}','cleanup_uncertain','replace'),
            ('ns-cleanup-inflight','node1','{}','cleanup_inflight','replace'),
            ('ns-committed','node1','{}','committed','mkdir'),
            ('ns-conflict','node1','{}','conflict','remove'),
            ('ns-superseded','node1','{}','superseded','rename');
         PRAGMA user_version=28;",
    )
    .expect("write v28 fixture");
}

fn single_text(db: &Connection, sql: &str, id: &str) -> Option<String> {
    db.query_row(sql, [id], |row| row.get::<_, Option<String>>(0))
        .expect("query single value")
}

#[test]
fn fresh_database_carries_v29_diagnostics_and_recovery_tables() {
    let directory = tempfile::tempdir().expect("tempdir");
    let store = Store::open(directory.path()).expect("open fresh store");
    drop(store);

    let db = Connection::open(directory.path().join("state.sqlite")).expect("open");
    assert_eq!(user_version(&db), 29);

    for table in ["operations", "namespace_ops", "transfers"] {
        let found = columns(&db, table);
        for column in ["outcome", "outcome_category", "last_error", "next_retry_at"] {
            assert!(
                found.contains(&column.to_string()),
                "{table} is missing {column}"
            );
        }
    }
    assert!(columns(&db, "operations").contains(&"updated_at".to_string()));
    assert!(columns(&db, "namespace_ops").contains(&"attempts".to_string()));
    assert!(columns(&db, "nodes").contains(&"in_sync".to_string()));

    for object in [
        "schema_migrations",
        "recovery_runs",
        "platform_state_changes",
        "commit_diagnostics",
    ] {
        assert!(has_table(&db, object), "missing {object}");
    }
    let applied: Vec<i64> = db
        .prepare("SELECT version FROM schema_migrations ORDER BY version")
        .expect("prepare")
        .query_map([], |row| row.get(0))
        .expect("rows")
        .map(|row| row.expect("version"))
        .collect();
    assert_eq!(applied, vec![28, 29]);
    let diagnostics: i64 = db
        .query_row("SELECT COUNT(*) FROM commit_diagnostics", [], |row| {
            row.get(0)
        })
        .expect("view");
    assert_eq!(diagnostics, 0);
    // Re-opening a current store neither re-migrates nor writes another backup.
    drop(db);
    let store = Store::open(directory.path()).expect("reopen fresh store");
    drop(store);
    assert_eq!(
        std::fs::read_dir(directory.path())
            .expect("read dir")
            .filter(|entry| entry
                .as_ref()
                .expect("entry")
                .file_name()
                .to_string_lossy()
                .starts_with("state-before-v"))
            .count(),
        0
    );
}

#[test]
fn v28_database_upgrades_with_backfill_backup_and_queries() {
    let directory = tempfile::tempdir().expect("tempdir");
    write_v28_fixture(directory.path());

    let store = Store::open(directory.path()).expect("upgrade v28 store");
    drop(store);

    // Backup happens before the migration and is a coherent v28 snapshot.
    let backup_path = std::fs::read_dir(directory.path())
        .expect("read dir")
        .map(|entry| entry.expect("entry").path())
        .find(|path| {
            path.file_name()
                .expect("name")
                .to_string_lossy()
                .starts_with("state-before-v29-")
        })
        .expect("v29 backup");
    let backup = Connection::open(&backup_path).expect("open backup");
    assert_eq!(user_version(&backup), 28);
    assert!(!columns(&backup, "operations").contains(&"outcome".to_string()));
    assert_eq!(
        backup
            .query_row("PRAGMA integrity_check", [], |row| row.get::<_, String>(0))
            .expect("integrity"),
        "ok"
    );
    drop(backup);

    let db = Connection::open(directory.path().join("state.sqlite")).expect("open upgraded");
    assert_eq!(user_version(&db), 29);
    for table in ["operations", "namespace_ops", "transfers"] {
        let found = columns(&db, table);
        assert!(
            found.contains(&"outcome".to_string()),
            "{table} missing outcome"
        );
        assert!(
            found.contains(&"next_retry_at".to_string()),
            "{table} missing next_retry_at"
        );
    }
    assert!(columns(&db, "nodes").contains(&"in_sync".to_string()));

    // #9: every already-submitted legacy row gains a diagnosis. Exhausted
    // uploads stay retryable/interrupted on purpose: TB2 remaps them to
    // unknown_fatal/reconcile_exhausted after #14 adds its abandoned terminal.
    let cases = [
        ("op-uncertain-upload", "unknown_retryable", "interrupted"),
        ("op-uncertain-exhausted", "unknown_retryable", "interrupted"),
        ("op-inflight", "unknown_retryable", "interrupted"),
        ("op-committed", "confirmed", "committed"),
        ("op-conflict", "conflict", "server_rejected"),
        ("op-superseded", "confirmed", "local_abandoned"),
        ("ns-uncertain", "unknown_retryable", "interrupted"),
        ("ns-cleanup-uncertain", "unknown_retryable", "interrupted"),
        ("ns-cleanup-inflight", "unknown_retryable", "interrupted"),
        ("ns-committed", "confirmed", "committed"),
        ("ns-conflict", "conflict", "server_rejected"),
        ("ns-superseded", "confirmed", "local_abandoned"),
    ];
    for (id, outcome, category) in cases {
        let table = if id.starts_with("op-") {
            "operations"
        } else {
            "namespace_ops"
        };
        let row: (Option<String>, Option<String>) = db
            .query_row(
                &format!("SELECT outcome,outcome_category FROM {table} WHERE id=?1"),
                [id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .expect("row");
        assert_eq!(row.0.as_deref(), Some(outcome), "{id} outcome");
        assert_eq!(row.1.as_deref(), Some(category), "{id} category");
    }
    // Rows that were never submitted keep no commit outcome.
    assert_eq!(
        single_text(
            &db,
            "SELECT outcome FROM operations WHERE id=?1",
            "op-pending"
        ),
        None
    );
    // A pre-existing #14-style `abandoned` row is preserved verbatim by the
    // v29 migration: no outcome is invented here, TB2 owns the mapping.
    let abandoned: (String, Option<String>) = db
        .query_row(
            "SELECT state,outcome FROM operations WHERE id='op-abandoned'",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .expect("abandoned row");
    assert_eq!(abandoned.0, "abandoned");
    assert_eq!(abandoned.1, None);
    assert_eq!(
        single_text(
            &db,
            "SELECT outcome FROM namespace_ops WHERE id=?1",
            "ns-pending"
        ),
        None
    );

    // Download failures map the old JSON diagnosis onto the same vocabulary.
    let retry: (Option<String>, Option<String>, i64, Option<String>) = db
        .query_row(
            "SELECT outcome,outcome_category,next_retry_at,last_error
               FROM transfers WHERE id='tr-retry'",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .expect("retry row");
    assert_eq!(retry.0.as_deref(), Some("unknown_retryable"));
    assert_eq!(retry.1.as_deref(), Some("transport"));
    assert_eq!(retry.2, 1234);
    assert_eq!(retry.3.as_deref(), Some("transport"));
    let failed: (Option<String>, Option<String>) = db
        .query_row(
            "SELECT outcome,outcome_category FROM transfers WHERE id='tr-failed'",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .expect("failed row");
    assert_eq!(failed.0.as_deref(), Some("unknown_retryable"));
    assert_eq!(failed.1.as_deref(), Some("server_rejected"));
    assert_eq!(
        single_text(
            &db,
            "SELECT outcome FROM transfers WHERE id=?1",
            "tr-pending"
        ),
        None
    );
    // Startup recovery still runs after the migration: submitted-but-interrupted
    // rows stay visible as uncertain/pending, now with a diagnosis attached.
    let inflight: (String, Option<String>) = db
        .query_row(
            "SELECT state,outcome FROM transfers WHERE id='tr-inflight'",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .expect("inflight row");
    assert_eq!(inflight.0, "pending");
    assert_eq!(inflight.1.as_deref(), Some("unknown_retryable"));
    let operation_inflight: (String, Option<String>) = db
        .query_row(
            "SELECT state,outcome FROM operations WHERE id='op-inflight'",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .expect("inflight operation");
    assert_eq!(operation_inflight.0, "uncertain");
    assert_eq!(operation_inflight.1.as_deref(), Some("unknown_retryable"));

    // The read-only status surface sees the same rows without joins per call.
    let diagnostics: i64 = db
        .query_row("SELECT COUNT(*) FROM commit_diagnostics", [], |row| {
            row.get(0)
        })
        .expect("view");
    assert_eq!(diagnostics, 16);
    let backfilled: i64 = db
        .query_row(
            "SELECT COUNT(*) FROM commit_diagnostics
              WHERE scope='operation' AND outcome_category='interrupted'",
            [],
            |row| row.get(0),
        )
        .expect("view filter");
    assert_eq!(backfilled, 3);

    // Reconcile-class queries keep working against the new columns and only
    // select the in-budget upload row.
    assert_eq!(store_pending(&db), 4);
    drop(db);

    let store = Store::open(directory.path()).expect("reopen upgraded");
    let now = state_epoch();
    let due = store
        .upload_reconcile_operations(now)
        .expect("reconcile query");
    assert_eq!(due, vec!["op-uncertain-upload".to_string()]);
    assert_eq!(store.pending_operations().expect("pending"), 4);
    drop(store);

    // Second open after a completed migration is idempotent: no new backup, no
    // duplicate ledger row, no rewritten diagnosis.
    let backups = std::fs::read_dir(directory.path())
        .expect("read dir")
        .filter(|entry| {
            entry
                .as_ref()
                .expect("entry")
                .file_name()
                .to_string_lossy()
                .starts_with("state-before-v29-")
        })
        .count();
    assert_eq!(backups, 1);
    let db = Connection::open(directory.path().join("state.sqlite")).expect("open");
    let ledger: i64 = db
        .query_row(
            "SELECT COUNT(*) FROM schema_migrations WHERE version=29",
            [],
            |row| row.get(0),
        )
        .expect("ledger");
    assert_eq!(ledger, 1);
    assert_eq!(user_version(&db), 29);
    assert_eq!(
        single_text(
            &db,
            "SELECT outcome FROM operations WHERE id=?1",
            "op-uncertain-upload"
        )
        .as_deref(),
        Some("unknown_retryable")
    );
    assert_eq!(
        single_text(
            &db,
            "SELECT outcome FROM operations WHERE id=?1",
            "op-pending"
        ),
        None
    );
}

/// `pending_operations` is not part of the TB1 surface change; this helper just
/// mirrors it for a raw connection so the test can assert the startup recovery
/// shape before the store handle is reopened.
fn store_pending(db: &Connection) -> i64 {
    db.query_row(
        "SELECT COUNT(*) FROM operations WHERE state IN ('pending','inflight','uncertain')",
        [],
        |row| row.get(0),
    )
    .expect("pending count")
}
