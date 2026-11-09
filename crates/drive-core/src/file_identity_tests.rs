//! Contract tests for `file_identity` (issue #11, step 1).
//!
//! These tests are deliberately written before the contract module exists: they
//! pin the behavior step 1 must preserve, not a new behavior. Everything here
//! runs against an in-memory-ish temporary Store and never mounts a filesystem.
use crate::file_identity::{FileIdentitySource, PlatformFileId, StoreFileIdentity};
use rusqlite::OptionalExtension;
use crate::{Node, NodeMetadata, Store};

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

fn store_with_snapshot() -> (tempfile::TempDir, Store) {
    let temp = tempfile::tempdir().unwrap();
    let mut store = Store::open(temp.path()).unwrap();
    store
        .install_remote_snapshot(&snapshot(), "1", "server")
        .unwrap();
    store.ensure_fs_root("root").unwrap();
    (temp, store)
}

fn raw_ino(store: &Store, id: &str) -> Option<u64> {
    let raw: Option<i64> = store
        .db
        .query_row("SELECT ino FROM fs_inodes WHERE node=?1", [id], |r| {
            r.get(0)
        })
        .optional()
        .unwrap();
    raw.map(|ino| u64::try_from(ino).unwrap())
}

#[test]
fn platform_file_id_keeps_the_existing_u64_inode_values() {
    // The new type is a name, not a new numbering scheme: the numbers the
    // kernel already sees must survive the conversion unchanged.
    assert_eq!(PlatformFileId::ROOT.as_u64(), 1);
    assert_eq!(PlatformFileId::new(7).as_u64(), 7);
    assert_eq!(u64::from(PlatformFileId::new(42)), 42);
    assert_eq!(PlatformFileId::from(9), PlatformFileId::new(9));
    assert_eq!(PlatformFileId::new(7).to_string(), "7");
}

#[test]
fn node_and_platform_identity_round_trip_is_idempotent() {
    let (_temp, store) = store_with_snapshot();
    let identity = StoreFileIdentity::new(&store);
    let mut seen = std::collections::HashSet::new();
    for id in ["root", "a", "d", "b"] {
        let first = identity.id_for_node(id).unwrap().unwrap();
        let second = identity.id_for_node(id).unwrap().unwrap();
        assert_eq!(first, second, "{id} identity must be stable");
        assert_eq!(first.as_u64(), raw_ino(&store, id).unwrap());
        assert_eq!(identity.node_for_id(first).unwrap().as_deref(), Some(id));
        assert!(seen.insert(first.as_u64()), "{id} reused another id");
    }
}

#[test]
fn root_identity_is_pinned_to_inode_one() {
    let (_temp, store) = store_with_snapshot();
    let identity = StoreFileIdentity::new(&store);
    assert_eq!(identity.root_id().unwrap(), Some(PlatformFileId::ROOT));
    assert_eq!(
        identity.id_for_node("root").unwrap(),
        Some(PlatformFileId::ROOT)
    );
    assert_eq!(
        identity.node_for_id(PlatformFileId::ROOT).unwrap().as_deref(),
        Some("root")
    );
    // Re-running the mount-time pin must neither move nor duplicate the root.
    store.ensure_fs_root("root").unwrap();
    assert_eq!(identity.root_id().unwrap(), Some(PlatformFileId::ROOT));
    assert_eq!(raw_ino(&store, "root"), Some(1));
}

#[test]
fn preview_reserved_inodes_keep_their_identity_round_trip() {
    let (_temp, store) = store_with_snapshot();
    // Preview candidates receive inodes reserved in `fs_inodes` and referenced
    // by `preview_inodes.ino`. The identity contract must report exactly the
    // same number and keep the reverse lookup intact.
    let reserved = "\0drive-native-preview:step1-test";
    store
        .db
        .execute("INSERT INTO fs_inodes(node) VALUES(?1)", [reserved])
        .unwrap();
    let ino = store.db.last_insert_rowid();
    let platform_id = PlatformFileId::new(u64::try_from(ino).unwrap());
    store
        .db
        .execute(
            "INSERT INTO preview_inodes(ino,reserved,run,node,native) VALUES(?1,?2,?3,?4,?5)",
            rusqlite::params![ino, reserved, "run-1", "preview-node", "1"],
        )
        .unwrap();
    let identity = StoreFileIdentity::new(&store);
    assert_eq!(
        identity.id_for_node(reserved).unwrap(),
        Some(platform_id)
    );
    assert_eq!(
        identity.node_for_id(platform_id).unwrap().as_deref(),
        Some(reserved)
    );
    let still_present: Option<String> = store
        .db
        .query_row(
            "SELECT f.node FROM preview_inodes p JOIN fs_inodes f ON f.ino=p.ino WHERE p.ino=?1",
            [ino],
            |r| r.get(0),
        )
        .optional()
        .unwrap();
    assert_eq!(still_present.as_deref(), Some(reserved));
}

#[test]
fn unknown_nodes_and_ids_report_none_without_writing() {
    let temp = tempfile::tempdir().unwrap();
    let store = Store::open(temp.path()).unwrap();
    let identity = StoreFileIdentity::new(&store);
    // Fresh store: the root identity is not pinned yet.
    assert_eq!(identity.root_id().unwrap(), None);
    assert_eq!(identity.id_for_node("missing").unwrap(), None);
    assert_eq!(
        identity.node_for_id(PlatformFileId::new(9999)).unwrap(),
        None
    );
    let rows: i64 = store
        .db
        .query_row("SELECT count(*) FROM fs_inodes", [], |r| r.get(0))
        .unwrap();
    assert_eq!(rows, 0, "identity reads must not allocate");
}

#[test]
fn namespace_backfill_forwarder_keeps_the_existing_result() {
    let (_temp, store) = store_with_snapshot();
    let identity = StoreFileIdentity::new(&store);
    // Already complete: the mount-time backfill is an O(1) no-op.
    assert_eq!(identity.ensure_namespace_identities("root").unwrap(), 0);
    assert_eq!(identity.root_id().unwrap(), Some(PlatformFileId::ROOT));
}
