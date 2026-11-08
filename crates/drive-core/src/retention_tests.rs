use super::*;

fn fixture() -> (tempfile::TempDir, Store) {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(dir.path()).unwrap();
    (dir, store)
}

fn operation(store: &Store, id: &str, node: &str, state: &str) {
    store
        .db
        .execute(
            "INSERT INTO operations(id,node,kind,base,blob,generation,state)
             VALUES(?1,?2,'upload','v1',NULL,1,?3)",
            params![id, node, state],
        )
        .unwrap();
}

fn namespace_operation(store: &Store, id: &str, state: &str) {
    store
        .db
        .execute(
            "INSERT INTO namespace_ops(id,node,body,state,kind)
             VALUES(?1,'node','{}',?2,'rename')",
            params![id, state],
        )
        .unwrap();
}

fn count(store: &Store, table: &str) -> i64 {
    store
        .db
        .query_row(&format!("SELECT count(*) FROM {table}"), [], |row| row.get(0))
        .unwrap()
}

/// Simulate a store written before schema 28: the one-time legacy dedupe has
/// not run yet.
fn mark_legacy(store: &Store) {
    store
        .db
        .execute(
            "DELETE FROM metadata WHERE key='storage_v28_committed_deduped'",
            [],
        )
        .unwrap();
}

#[test]
fn prune_removes_only_unobservable_rows_and_never_moves_the_journal() {
    let (_dir, mut store) = fixture();
    mark_legacy(&store);
    operation(&store, "a-old-1", "a", "superseded");
    operation(&store, "a-old-2", "a", "superseded");
    operation(&store, "a-1", "a", "committed");
    operation(&store, "a-2", "a", "committed");
    operation(&store, "a-3", "a", "committed");
    operation(&store, "b-1", "b", "committed");
    operation(&store, "b-2", "b", "committed");
    operation(&store, "p-1", "a", "pending");
    namespace_operation(&store, "ns-1", "committed");
    namespace_operation(&store, "ns-2", "committed");
    namespace_operation(&store, "ns-3", "superseded");
    namespace_operation(&store, "ns-4", "pending");
    store
        .db
        .execute_batch(
            "INSERT INTO inbox(seq,body,done) VALUES(1,'{\"a\":1}',1);
             INSERT INTO inbox(seq,body,done) VALUES(2,'{\"a\":2}',1);
             INSERT INTO inbox(seq,body,done) VALUES(3,'{\"a\":3}',0);",
        )
        .unwrap();
    let journal = store.status_change_watermark().unwrap();
    let namespace = store.namespace_watermark().unwrap();
    assert_eq!(store.pending_operations().unwrap(), 1);

    let stats = store.prune_storage().unwrap();
    assert_eq!(
        stats,
        RetentionStats {
            namespace_ops: 0,
            operations_superseded: 2,
            operations_committed: 3,
            inbox: 2,
            upload_stages: 0,
            upload_origins: 0,
            upload_paths: 0,
        }
    );
    // The semantic journal and the namespace projection token are untouched:
    // retention deletes bookkeeping, not namespace facts.
    assert_eq!(store.status_change_watermark().unwrap(), journal);
    assert_eq!(store.namespace_watermark().unwrap(), namespace);
    // Superseded rows are gone, each node keeps exactly one committed row and
    // every pending row survives.
    assert_eq!(count(&store, "operations"), 3);
    assert_eq!(
        store
            .db
            .query_row(
                "SELECT count(*) FROM operations WHERE node='a' AND state='committed'",
                [],
                |row| row.get::<_, i64>(0)
            )
            .unwrap(),
        1
    );
    assert_eq!(
        store
            .db
            .query_row(
                "SELECT id FROM operations WHERE node='a' AND state='committed'",
                [],
                |row| row.get::<_, String>(0)
            )
            .unwrap(),
        "a-3"
    );
    assert_eq!(store.pending_operations().unwrap(), 1);
    // Terminal namespace history below the retention window is kept, the
    // pending row is kept and applied inbox receipts are gone.
    assert_eq!(count(&store, "namespace_ops"), 4);
    assert_eq!(count(&store, "inbox"), 1);

    // A second prune has nothing left to do and still does not wake the journal.
    let again = store.prune_storage().unwrap();
    assert_eq!(again, RetentionStats::default());
    assert_eq!(store.status_change_watermark().unwrap(), journal);
    assert_eq!(store.namespace_watermark().unwrap(), namespace);
}

#[test]
fn terminal_namespace_history_keeps_only_the_newest_window() {
    let (_dir, mut store) = fixture();
    {
        let tx = store.db.transaction().unwrap();
        {
            let mut insert = tx
                .prepare(
                    "INSERT INTO namespace_ops(id,node,body,state,kind)
                     VALUES(?1,'node','{}',?2,'rename')",
                )
                .unwrap();
            for index in 0..(NAMESPACE_TERMINAL_RETENTION + 7) {
                insert
                    .execute(params![format!("terminal-{index}"), "committed"])
                    .unwrap();
            }
            insert
                .execute(params!["pending-1", "pending"])
                .unwrap();
            insert
                .execute(params!["pending-2", "uncertain"])
                .unwrap();
        }
        tx.commit().unwrap();
    }
    let total = count(&store, "namespace_ops");
    let journal = store.status_change_watermark().unwrap();
    let stats = store.prune_storage().unwrap();
    assert_eq!(stats.namespace_ops, 7);
    assert_eq!(count(&store, "namespace_ops"), total - 7);
    assert_eq!(
        count(&store, "namespace_ops"),
        NAMESPACE_TERMINAL_RETENTION + 2
    );
    assert_eq!(
        store
            .db
            .query_row(
                "SELECT count(*) FROM namespace_ops
                 WHERE state IN ('committed','superseded')",
                [],
                |row| row.get::<_, i64>(0)
            )
            .unwrap(),
        NAMESPACE_TERMINAL_RETENTION
    );
    assert_eq!(store.status_change_watermark().unwrap(), journal);
    // The newest terminal row is still present; the oldest was removed.
    let oldest: bool = store
        .db
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM namespace_ops WHERE id='terminal-0')",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert!(!oldest, "the oldest terminal row must be pruned");
    assert!(store
        .db
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM namespace_ops WHERE id=?1)",
            [format!("terminal-{}", NAMESPACE_TERMINAL_RETENTION + 6)],
            |row| row.get::<_, bool>(0)
        )
        .unwrap());
    // Pending rows are never part of the terminal window.
    assert_eq!(
        store
            .db
            .query_row(
                "SELECT count(*) FROM namespace_ops WHERE state IN ('pending','uncertain')",
                [],
                |row| row.get::<_, i64>(0)
            )
            .unwrap(),
        2
    );
}

#[test]
fn committing_a_new_generation_drops_the_previous_one_without_a_journal_row() {
    let (_dir, mut store) = fixture();
    operation(&store, "a-old", "a", "committed");
    operation(&store, "a-new", "a", "committed");
    store
        .db
        .execute("INSERT INTO upload_times(operation,mtime) VALUES('a-old',7)", [])
        .unwrap();
    let journal = store.status_change_watermark().unwrap();
    let namespace = store.namespace_watermark().unwrap();
    let tx = store.db.transaction().unwrap();
    let dropped = drop_committed_generations(&tx, "a", "a-new").unwrap();
    tx.commit().unwrap();
    assert_eq!(dropped, 1);
    assert_eq!(count(&store, "operations"), 1);
    assert_eq!(
        store
            .db
            .query_row(
                "SELECT count(*) FROM upload_times WHERE operation='a-old'",
                [],
                |row| row.get::<_, i64>(0)
            )
            .unwrap(),
        0,
        "the operation FK cascade removes its upload_times row"
    );
    assert_eq!(store.status_change_watermark().unwrap(), journal);
    assert_eq!(store.namespace_watermark().unwrap(), namespace);
}

#[test]
fn prune_fails_loudly_when_the_temp_journal_is_missing() {
    let (_dir, mut store) = fixture();
    store.db.execute_batch("DROP TABLE dn_status_meta").unwrap();
    // A missing journal must abort the prune instead of deleting rows that the
    // watermark still describes. SQLite reports the dropped TEMP table before
    // our own guard can speak; both messages are loud failures.
    let error = store.prune_storage().unwrap_err();
    let message = error.to_string();
    assert!(
        message.contains("dn_status_meta") || message.contains("prune suppression"),
        "unexpected error: {error}"
    );
    assert_eq!(count(&store, "operations"), 0);
}

#[test]
fn inbox_receipts_are_removed_even_when_all_are_applied() {
    let (_dir, mut store) = fixture();
    store
        .db
        .execute_batch(
            "INSERT INTO inbox(seq,body,done) VALUES(1,'{\"a\":1}',1);
             INSERT INTO inbox(seq,body,done) VALUES(2,'{\"a\":2}',1);",
        )
        .unwrap();
    let stats = store.prune_storage().unwrap();
    assert_eq!(stats.inbox, 2);
    assert_eq!(count(&store, "inbox"), 0);
}
