//! Issue #50 phase 4: purge/restore tests for rule-excluded subtrees.
//!
//! Purging must hide the subtree from the local namespace, drop its cached and
//! dirty content plus any local scheduling state, and never queue a remote
//! delete. Removing the rules restores the local view.
use crate::Store;
use crate::exclude::ExcludeRules;
use rusqlite::OptionalExtension;

fn open() -> (tempfile::TempDir, Store) {
    let dir = tempfile::tempdir().expect("tempdir");
    let store = Store::open(dir.path()).expect("open store");
    (dir, store)
}

fn insert_node(store: &Store, id: &str, parent: Option<&str>, name: &str, directory: bool) {
    store
        .db
        .execute(
            "INSERT INTO nodes(id,parent,name,remote_version,directory) VALUES(?1,?2,?3,'v1',?4)",
            rusqlite::params![id, parent, name, directory],
        )
        .expect("insert node");
}

fn allocate_inodes(store: &Store) {
    store
        .db
        .execute("DELETE FROM metadata WHERE key='fs_inodes_complete'", [])
        .expect("reset inode marker");
    store.ensure_fs_root("100").expect("root inode");
    store.ensure_filesystem_inodes("100").expect("namespace inodes");
}

fn scalar(store: &Store, sql: &str) -> i64 {
    store
        .db
        .query_row(sql, [], |row| row.get(0))
        .expect("scalar query")
}

fn marker(store: &Store, key: &str) -> Option<String> {
    store
        .db
        .query_row("SELECT value FROM metadata WHERE key=?1", [key], |row| {
            row.get(0)
        })
        .optional()
        .expect("metadata query")
}

fn blob_path(store: &Store, blob: &str) -> std::path::PathBuf {
    store.root.join("blobs").join(blob)
}

/// Insert every kind of local scheduling/cache row that can reference a node.
fn queue_local_work(store: &Store, node: &str) {
    store
        .db
        .execute(
            "INSERT INTO operations(id,node,kind,base,generation,state) VALUES(?1,?2,'upload','v1',1,'pending')",
            rusqlite::params![format!("op-{node}"), node],
        )
        .unwrap();
    store
        .db
        .execute(
            "INSERT INTO transfers(id,node,version,direction,state) VALUES(?1,?2,'v1','download','pending')",
            rusqlite::params![format!("tr-{node}"), node],
        )
        .unwrap();
    store
        .db
        .execute(
            "INSERT INTO drafts(node,token,base,generation,checkpoint,dirty) VALUES(?1,?2,'v1',1,1,1)",
            rusqlite::params![node, format!("draft-{node}")],
        )
        .unwrap();
    store
        .db
        .execute(
            "INSERT INTO local_creations(node,ready) VALUES(?1,1)",
            [node],
        )
        .unwrap();
    store
        .db
        .execute(
            "INSERT INTO namespace_ops(id,node,body,state,kind) VALUES(?1,?2,?3,'pending','remove')",
            rusqlite::params![format!("ns-{node}"), node, format!(r#"{{"id":"{node}"}}"#)],
        )
        .unwrap();
    store
        .db
        .execute(
            "INSERT INTO local_unlinks(node,confirmed) VALUES(?1,0)",
            [node],
        )
        .unwrap();
    store
        .db
        .execute("INSERT INTO cache_access(node) VALUES(?1)", [node])
        .unwrap();
    store
        .db
        .execute("INSERT INTO cache_published(node,at) VALUES(?1,1)", [node])
        .unwrap();
    store
        .db
        .execute(
            "INSERT INTO local_atimes(node,seconds,nanoseconds) VALUES(?1,1,0)",
            [node],
        )
        .unwrap();
}

#[test]
fn purge_hides_the_subtree_and_never_queues_a_remote_delete() {
    let (_dir, mut store) = open();
    insert_node(&store, "100", None, "root", true);
    insert_node(&store, "101", Some("100"), "A.txt", false);
    insert_node(&store, "200", Some("100"), "node_modules", true);
    insert_node(&store, "201", Some("200"), "index.js", false);
    allocate_inodes(&store);

    let blob = store.write_blob(b"payload").unwrap();
    store
        .db
        .execute(
            "UPDATE nodes SET blob=?1,local_version='v1' WHERE id='201'",
            [&blob],
        )
        .unwrap();
    assert!(blob_path(&store, &blob).exists(), "fixture must cache the blob");
    queue_local_work(&store, "200");
    queue_local_work(&store, "201");
    // Unrelated local work on a visible node must survive the purge.
    store
        .db
        .execute(
            "INSERT INTO operations(id,node,kind,base,generation,state) VALUES('op-unrelated','101','upload','v1',1,'pending')",
            [],
        )
        .unwrap();

    let purged = store
        .purge_excluded_nodes(&["200".to_owned(), "201".to_owned()])
        .unwrap();
    assert_eq!(purged, 2);

    assert!(store.node("200").is_err(), "excluded directory is hidden");
    assert!(store.node("201").is_err(), "excluded child is hidden");
    assert!(store.node("101").is_ok(), "sibling stays visible");

    for node in ["200", "201"] {
        for (table, column) in [
            ("operations", "node"),
            ("transfers", "node"),
            ("drafts", "node"),
            ("local_creations", "node"),
            ("local_unlinks", "node"),
            ("cache_access", "node"),
            ("cache_published", "node"),
            ("local_atimes", "node"),
            ("fs_inodes", "node"),
        ] {
            let sql = format!("SELECT count(*) FROM {table} WHERE {column}='{node}'");
            assert_eq!(scalar(&store, &sql), 0, "stale row in {table} for {node}");
        }
        assert_eq!(
            scalar(
                &store,
                &format!("SELECT count(*) FROM namespace_ops WHERE node='{node}'"),
            ),
            0,
            "stale namespace operation for {node}"
        );
    }
    assert_eq!(
        scalar(
            &store,
            "SELECT count(*) FROM operations WHERE node='101'",
        ),
        1,
        "unrelated local work survives"
    );
    // No remote delete was queued: the remove queue is empty and no `remove`
    // namespace_ops row was created for the purged nodes.
    assert!(
        store.next_remove().unwrap().is_none(),
        "purge must not queue a remote remove"
    );
    assert_eq!(
        scalar(
            &store,
            "SELECT count(*) FROM namespace_ops WHERE kind='remove'",
        ),
        0
    );
    assert!(
        marker(&store, "excluded_purged").is_some(),
        "purged ids are recorded for a later restore"
    );
    assert!(
        marker(&store, "fs_inodes_complete").is_none(),
        "derived inodes are rebuilt on the next mount"
    );

    store.collect_garbage().unwrap();
    assert!(
        !blob_path(&store, &blob).exists(),
        "cached blob is reclaimed once unreferenced"
    );
}

#[test]
fn restore_brings_back_rows_hidden_by_an_earlier_rule_set() {
    let (_dir, mut store) = open();
    insert_node(&store, "100", None, "root", true);
    insert_node(&store, "101", Some("100"), "A.txt", false);
    insert_node(&store, "200", Some("100"), "node_modules", true);
    allocate_inodes(&store);

    store
        .purge_excluded_nodes(&["200".to_owned()])
        .unwrap();
    store.collect_garbage().unwrap();
    assert!(store.node("200").is_err());

    let restored = store.restore_excluded_nodes().unwrap();
    assert_eq!(restored, 1);
    assert!(store.node("200").is_ok(), "restored node is visible again");
    assert!(
        marker(&store, "excluded_purged").is_none(),
        "restore clears the purge marker"
    );
    store
        .ensure_filesystem_inodes("100")
        .expect("inode map is rebuilt after restore");
    assert!(store.lookup("100", "node_modules").is_ok());
}

#[test]
fn conflicted_nodes_are_never_purged() {
    let (_dir, mut store) = open();
    insert_node(&store, "100", None, "root", true);
    insert_node(&store, "200", Some("100"), "node_modules", true);
    insert_node(&store, "201", Some("200"), "index.js", false);
    allocate_inodes(&store);
    store
        .db
        .execute("UPDATE nodes SET conflict=1 WHERE id='200'", [])
        .unwrap();

    let purged = store
        .purge_excluded_nodes(&["200".to_owned(), "201".to_owned()])
        .unwrap();
    assert_eq!(purged, 1, "conflicted row is skipped");
    assert!(store.node("200").is_ok(), "conflicted node keeps its local copy");
    assert_eq!(
        scalar(&store, "SELECT conflict FROM nodes WHERE id='200'"),
        1
    );
    assert!(store.node("201").is_err());
}

#[test]
fn dirty_content_is_dropped_and_reclaimed() {
    let (_dir, mut store) = open();
    insert_node(&store, "100", None, "root", true);
    insert_node(&store, "200", Some("100"), "node_modules", true);
    allocate_inodes(&store);
    let blob = store.write_blob(b"dirty").unwrap();
    store
        .db
        .execute(
            "UPDATE nodes SET dirty_blob=?1 WHERE id='200'",
            [&blob],
        )
        .unwrap();
    queue_local_work(&store, "200");

    store
        .purge_excluded_nodes(&["200".to_owned()])
        .unwrap();
    let dirty: Option<String> = store
        .db
        .query_row("SELECT dirty_blob FROM nodes WHERE id='200'", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert!(dirty.is_none(), "pending dirty content is dropped");
    store.collect_garbage().unwrap();
    assert!(!blob_path(&store, &blob).exists());
}

#[test]
fn purge_with_no_ids_is_a_no_op() {
    let (_dir, mut store) = open();
    insert_node(&store, "100", None, "root", true);
    allocate_inodes(&store);
    assert_eq!(store.purge_excluded_nodes(&[]).unwrap(), 0);
    assert!(store.node("100").is_ok());
    assert!(marker(&store, "excluded_purged").is_none());
}

#[test]
fn restore_without_a_purge_marker_is_a_no_op() {
    let (_dir, mut store) = open();
    insert_node(&store, "100", None, "root", true);
    allocate_inodes(&store);
    assert_eq!(store.restore_excluded_nodes().unwrap(), 0);
    assert!(store.node("100").is_ok());
}

#[test]
fn recovery_snapshot_does_not_conflict_nodes_hidden_by_the_rules() {
    let (_dir, mut store) = open();
    insert_node(&store, "100", None, "root", true);
    insert_node(&store, "101", Some("100"), "A.txt", false);
    insert_node(&store, "200", Some("100"), "node_modules", true);
    insert_node(&store, "201", Some("200"), "index.js", false);
    allocate_inodes(&store);
    let blob = store.write_blob(b"dirty").unwrap();
    store
        .db
        .execute(
            "UPDATE nodes SET blob=?1,local_version='v1',dirty_blob=?1 WHERE id='201'",
            [&blob],
        )
        .unwrap();
    queue_local_work(&store, "200");
    queue_local_work(&store, "201");

    // What the mount does when the rules are active: hide the subtree before a
    // recovery snapshot can observe it as "absent".
    let purged = store
        .purge_excluded_nodes(&["200".to_owned(), "201".to_owned()])
        .unwrap();
    assert_eq!(purged, 2);
    store.collect_garbage().unwrap();

    // The filtered recovery snapshot omits the excluded subtree entirely.
    store
        .install_remote_snapshot(
            &[
                node("100", None, "root", true),
                node("101", Some("100"), "A.txt", false),
            ],
            "v2",
            "server",
        )
        .unwrap();

    assert!(store.node("200").is_err(), "hidden directory stays hidden");
    assert!(store.node("201").is_err(), "hidden child stays hidden");
    assert!(store.node("101").is_ok(), "sibling is still visible");
    assert_eq!(
        scalar(
            &store,
            "SELECT count(*) FROM nodes WHERE id IN ('200','201') AND conflict=1",
        ),
        0,
        "a hidden node must not become a recovery conflict"
    );
    assert_eq!(
        scalar(
            &store,
            "SELECT count(*) FROM nodes WHERE id IN ('200','201') AND dirty_blob IS NOT NULL",
        ),
        0,
        "hidden dirty content stays dropped"
    );
    assert_eq!(
        scalar(&store, "SELECT count(*) FROM operations"),
        0,
        "no local scheduling row survives the purge"
    );
    assert_eq!(
        scalar(
            &store,
            "SELECT count(*) FROM namespace_ops WHERE kind='remove'",
        ),
        0
    );
    assert!(
        store.next_remove().unwrap().is_none(),
        "recovery must not queue a remote delete"
    );
    assert_eq!(
        scalar(&store, "SELECT count(*) FROM namespace_overlay"),
        0,
        "overlay rows cannot resurrect a hidden node as conflicted"
    );
}

fn node(id: &str, parent: Option<&str>, name: &str, directory: bool) -> crate::Node {
    crate::Node {
        id: id.to_owned(),
        parent: parent.map(str::to_owned),
        name: name.to_owned(),
        version: "v1".into(),
        directory,
        metadata: None,
    }
}

/// Issue #56: the restore predicate is the caller's policy decision, so the
/// hidden-exclude mode keeps the rows matched by the remaining rules and
/// restores the rest. A future local-only mode can pass its own predicate
/// without changing the store; the marker keeps exactly the still-hidden ids.
#[test]
fn restore_unexcluded_nodes_keeps_rows_matching_the_current_rules() {
    let (_dir, mut store) = open();
    insert_node(&store, "100", None, "root", true);
    insert_node(&store, "101", Some("100"), "A.txt", false);
    insert_node(&store, "200", Some("100"), ".git", true);
    insert_node(&store, "201", Some("200"), "config", false);
    insert_node(&store, "300", Some("100"), "cache", true);
    insert_node(&store, "301", Some("300"), "objects", false);
    allocate_inodes(&store);
    store
        .purge_excluded_nodes(&[
            "200".to_owned(),
            "201".to_owned(),
            "300".to_owned(),
            "301".to_owned(),
        ])
        .unwrap();
    let rules = ExcludeRules::compile(&[r"(^|/)cache(/|$)"]).unwrap();
    let restored = store
        .restore_unexcluded_nodes(|path, is_dir| rules.matches(path, is_dir))
        .unwrap();
    assert_eq!(restored, 2, "the .git subtree is restored");
    assert!(store.node("200").is_ok());
    assert!(store.node("201").is_ok());
    assert!(store.node("300").is_err(), "the cache subtree stays hidden");
    assert!(store.node("301").is_err());
    assert_eq!(
        marker(&store, "excluded_purged").as_deref(),
        Some(r#"["300","301"]"#),
        "only the still-hidden ids stay marked"
    );
    // A later full restore brings back exactly the remaining cache subtree.
    assert_eq!(store.restore_excluded_nodes().unwrap(), 2);
    assert!(marker(&store, "excluded_purged").is_none());
}
