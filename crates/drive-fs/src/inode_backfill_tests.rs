//! Mount initialization derives missing `fs_inodes` mappings for state
//! directories installed by builds that allocated inodes lazily.
use super::*;
use drive_core::{Node, NodeMetadata, SharedStore};

fn node(id: &str, parent: Option<&str>, directory: bool) -> Node {
    Node {
        id: id.into(),
        parent: parent.map(str::to_owned),
        name: id.into(),
        version: "v1".into(),
        directory,
        metadata: Some(NodeMetadata {
            size: if directory { 0 } else { 3 },
            mtime: 1,
            mode: if directory { 0o755 } else { 0o644 },
        }),
    }
}
fn store_with_nodes(temp: &tempfile::TempDir) -> Store {
    let mut store = Store::open(temp.path()).unwrap();
    store
        .install_remote_snapshot(
            &[
                node("root", None, true),
                node("a", Some("root"), false),
                node("d", Some("root"), true),
                node("b", Some("d"), false),
            ],
            "1",
            "server",
        )
        .unwrap();
    store.ensure_fs_root("root").unwrap();
    store
}

#[test]
fn mount_initialization_backfills_a_state_directory_installed_without_mappings() {
    let temp = tempfile::tempdir().unwrap();
    let store = store_with_nodes(&temp);
    drop(store);
    // Model a database installed by a build without batch allocation: the root
    // was mapped at mount, everything else only on first access.
    let db = rusqlite::Connection::open(temp.path().join("state.sqlite")).unwrap();
    db.execute("DELETE FROM fs_inodes WHERE ino!=1", [])
        .unwrap();
    drop(db);
    let shared = Arc::new(SharedStore::new(Store::open(temp.path()).unwrap()));
    let filesystem = CacheFilesystem::new(shared.clone(), "root", 1000, 1000).unwrap();
    drop(filesystem);
    {
        let guard = shared.lock().unwrap();
        // The mount already backfilled the whole namespace in one transaction.
        assert_eq!(guard.ensure_filesystem_inodes("root").unwrap(), 0);
        assert_eq!(guard.posix_record_by_inode(1).unwrap().id, "root");
        assert_eq!(guard.posix_lookup("d", "b").unwrap().id, "b");
    }
    // A second mount on the repaired directory writes nothing.
    let revision = shared.lock().unwrap().local_revision();
    let filesystem = CacheFilesystem::new(shared.clone(), "root", 1000, 1000).unwrap();
    drop(filesystem);
    assert_eq!(shared.lock().unwrap().local_revision(), revision);
}

#[test]
fn mount_initialization_keeps_an_already_complete_mapping_untouched() {
    let temp = tempfile::tempdir().unwrap();
    let shared = Arc::new(SharedStore::new(store_with_nodes(&temp)));
    let revision = shared.lock().unwrap().local_revision();
    let filesystem = CacheFilesystem::new(shared.clone(), "root", 1000, 1000).unwrap();
    drop(filesystem);
    assert_eq!(shared.lock().unwrap().local_revision(), revision);
    assert_eq!(
        shared
            .lock()
            .unwrap()
            .ensure_filesystem_inodes("root")
            .unwrap(),
        0
    );
}
