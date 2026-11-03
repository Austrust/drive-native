use super::*;
use crate::SharedStore;

fn node(id: &str, parent: Option<&str>, directory: bool) -> Node {
    Node {
        id: id.into(),
        parent: parent.map(str::to_owned),
        name: id.into(),
        version: "v1".into(),
        directory,
        metadata: Some(NodeMetadata {
            size: if directory { 0 } else { 3 },
            mtime: 123,
            mode: if directory { 0o755 } else { 0o644 },
        }),
    }
}
fn snapshot() -> Vec<Node> {
    vec![
        node("root", None, true),
        node("a", Some("root"), false),
        node("d", Some("root"), true),
        node("b", Some("d"), false),
    ]
}
fn inode_count(store: &Store) -> i64 {
    store
        .db
        .query_row("SELECT count(*) FROM fs_inodes", [], |r| r.get(0))
        .unwrap()
}
fn ino_of(store: &Store, id: &str) -> Option<u64> {
    let raw: Option<i64> = store
        .db
        .query_row("SELECT ino FROM fs_inodes WHERE node=?1", [id], |r| {
            r.get(0)
        })
        .optional()
        .unwrap();
    raw.map(|ino| u64::try_from(ino).unwrap())
}
fn sequence(store: &Store) -> i64 {
    store
        .db
        .query_row(
            "SELECT COALESCE((SELECT seq FROM sqlite_sequence WHERE name='fs_inodes'),0)",
            [],
            |r| r.get(0),
        )
        .unwrap()
}

#[test]
fn install_allocates_every_live_node_and_pins_the_root() {
    let temp = tempfile::tempdir().unwrap();
    let mut store = Store::open(temp.path()).unwrap();
    store
        .install_remote_snapshot(&snapshot(), "1", "server")
        .unwrap();
    // The install transaction allocated identities for the whole namespace; a
    // mount root is `ino=1` before `ensure_fs_root` ever runs.
    assert_eq!(inode_count(&store), 4);
    assert_eq!(ino_of(&store, "root"), Some(1));
    for id in ["a", "d", "b"] {
        let ino = ino_of(&store, id).unwrap();
        assert!(ino > 1, "{id} got reserved root inode");
    }
    let distinct: i64 = store
        .db
        .query_row("SELECT count(DISTINCT ino) FROM fs_inodes", [], |r| {
            r.get(0)
        })
        .unwrap();
    assert_eq!(distinct, 4);
    store.ensure_fs_root("root").unwrap();
    // The root marker is already the pinned mapping: no extra row.
    assert_eq!(inode_count(&store), 4);
}

#[test]
fn repeated_install_is_idempotent_and_keeps_identities() {
    let temp = tempfile::tempdir().unwrap();
    let mut store = Store::open(temp.path()).unwrap();
    store
        .install_remote_snapshot(&snapshot(), "1", "server")
        .unwrap();
    store.ensure_fs_root("root").unwrap();
    let before: Vec<Option<u64>> = ["root", "a", "d", "b"]
        .iter()
        .map(|id| ino_of(&store, id))
        .collect();
    let seq_before = sequence(&store);
    store
        .install_remote_snapshot(&snapshot(), "1", "server")
        .unwrap();
    // The upsert re-writes the snapshot rows, but no derived inode identity is
    // reallocated: same numbers, same AUTOINCREMENT sequence.
    assert_eq!(
        ["root", "a", "d", "b"]
            .iter()
            .map(|id| ino_of(&store, id))
            .collect::<Vec<_>>(),
        before
    );
    assert_eq!(sequence(&store), seq_before);
    assert_eq!(inode_count(&store), 4);
}

#[test]
fn opendir_after_install_commits_no_write() {
    let temp = tempfile::tempdir().unwrap();
    let mut store = Store::open(temp.path()).unwrap();
    store
        .install_remote_snapshot(&snapshot(), "1", "server")
        .unwrap();
    store.ensure_fs_root("root").unwrap();
    let revision = store.local_revision();
    let seq_before = sequence(&store);
    let snapshot = store.directory_snapshot(1, 1 << 20).unwrap();
    assert_eq!(
        snapshot
            .entries
            .iter()
            .map(|e| e.name.as_str())
            .collect::<Vec<_>>(),
        vec![".", "..", "a", "d"]
    );
    // The projection inserted no inode row, so the transaction commit cannot
    // have appended (or fsynced) a WAL frame.
    assert_eq!(store.local_revision(), revision);
    assert_eq!(sequence(&store), seq_before);
    let repeated = store.directory_snapshot(1, 1 << 20).unwrap();
    assert_eq!(repeated.entries, snapshot.entries);
    assert_eq!(store.local_revision(), revision);
}

#[test]
fn mount_backfill_repairs_an_older_install_once() {
    let temp = tempfile::tempdir().unwrap();
    let mut store = Store::open(temp.path()).unwrap();
    store
        .install_remote_snapshot(&snapshot(), "1", "server")
        .unwrap();
    store.ensure_fs_root("root").unwrap();
    // Model a database installed by a build without batch allocation: the root
    // was mapped at mount, the rest only on first access (or not at all).
    store
        .db
        .execute("DELETE FROM fs_inodes WHERE ino!=1", [])
        .unwrap();
    assert_eq!(inode_count(&store), 1);
    assert_eq!(store.ensure_filesystem_inodes("root").unwrap(), 3);
    assert_eq!(inode_count(&store), 4);
    assert_eq!(ino_of(&store, "root"), Some(1));
    let revision = store.local_revision();
    let seq_before = sequence(&store);
    assert_eq!(store.ensure_filesystem_inodes("root").unwrap(), 0);
    assert_eq!(store.local_revision(), revision);
    assert_eq!(sequence(&store), seq_before);
    // Projections and lookups now find every identity without writing.
    let b = store.posix_lookup("d", "b").unwrap();
    assert_eq!(store.posix_record_by_inode(b.ino).unwrap(), b);
    assert_eq!(store.local_revision(), revision);
}

#[test]
fn nodes_created_after_install_allocate_lazily_without_moving_the_root() {
    let temp = tempfile::tempdir().unwrap();
    let mut store = Store::open(temp.path()).unwrap();
    store
        .install_remote_snapshot(&snapshot(), "1", "server")
        .unwrap();
    store.ensure_fs_root("root").unwrap();
    let created = store.create_local_file("d", "created", 0o644).unwrap();
    assert_eq!(ino_of(&store, &created.id), None);
    let revision = store.local_revision();
    let record = store.posix_lookup("d", "created").unwrap();
    assert!(record.ino > 1);
    assert_eq!(store.local_revision(), revision + 1);
    assert_eq!(ino_of(&store, &created.id), Some(record.ino));
    assert_eq!(ino_of(&store, "root"), Some(1));
    let repeated = store.posix_lookup("d", "created").unwrap();
    assert_eq!(repeated, record);
    assert_eq!(store.local_revision(), revision + 1);
}

#[test]
fn concurrent_mount_backfills_are_serialized_and_allocate_each_missing_node_once() {
    let temp = tempfile::tempdir().unwrap();
    let mut store = Store::open(temp.path()).unwrap();
    store
        .install_remote_snapshot(&snapshot(), "1", "server")
        .unwrap();
    store.ensure_fs_root("root").unwrap();
    store
        .db
        .execute("DELETE FROM fs_inodes WHERE ino!=1", [])
        .unwrap();
    let shared = std::sync::Arc::new(SharedStore::new(store));
    let mut threads = Vec::new();
    for _ in 0..4 {
        let shared = shared.clone();
        threads.push(std::thread::spawn(move || {
            let guard = shared.lock().unwrap();
            guard.ensure_filesystem_inodes("root").unwrap()
        }));
    }
    let written: u64 = threads.into_iter().map(|t| t.join().unwrap()).sum();
    assert_eq!(written, 3);
    let guard = shared.lock().unwrap();
    assert_eq!(
        guard
            .db
            .query_row("SELECT count(*) FROM fs_inodes", [], |r| {
                r.get::<_, i64>(0)
            })
            .unwrap(),
        4
    );
    assert_eq!(
        guard
            .db
            .query_row("SELECT count(*) FROM fs_inodes WHERE ino=1", [], |r| {
                r.get::<_, i64>(0)
            })
            .unwrap(),
        1
    );
}
