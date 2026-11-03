use super::*;
use crate::{Batch, Change, DownloadTicket, Event};
fn fixture() -> (tempfile::TempDir, Store) {
    let temp = tempfile::tempdir().unwrap();
    let mut store = Store::open(temp.path()).unwrap();
    let node = |id: &str, parent: Option<&str>, directory: bool| Node {
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
    };
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
    (temp, store)
}
fn same(store: &Store, id: &str) -> PosixRecord {
    let full = store.record_for_handle(id).unwrap();
    let small = store.posix_record_by_inode(full.ino).unwrap();
    assert_eq!(
        (
            &small.id,
            small.ino,
            small.directory,
            small.size,
            small.mtime,
            small.mode,
            small.unlinked
        ),
        (
            &full.node.id,
            full.ino,
            full.node.directory,
            full.size,
            full.mtime,
            full.mode,
            full.unlinked
        )
    );
    small
}
#[test]
fn online_cached_dirty_and_unlinked_attributes_match_without_mutating_queues() {
    let (_temp, mut store) = fixture();
    let first = store.posix_lookup("root", "a").unwrap();
    assert_eq!(first.blocks, 0);
    // A clean, online-only authority entry is the only cacheable state.
    assert!(first.cacheable);
    assert!(store.posix_lookup("root", "d").unwrap().cacheable);
    let revision = store.local_revision();
    for _ in 0..20 {
        assert_eq!(store.posix_record_by_inode(first.ino).unwrap(), first);
    }
    assert_eq!(store.local_revision(), revision);
    store
        .publish_download(
            &DownloadTicket {
                id: "a".into(),
                version: "v1".into(),
            },
            b"abc",
            &crate::sha256(b"abc"),
        )
        .unwrap();
    let cached = same(&store, "a");
    assert_eq!(cached.blocks, 1);
    // A clean cached body is still authoritative metadata and cacheable.
    assert!(cached.cacheable);
    let draft = store.begin_draft("a", "v1", None).unwrap();
    let body = b"uncheckpointed local content";
    store.draft_write(&draft, 0, body, false).unwrap();
    let active_handles = store.handles.lock().unwrap().clone();
    let revision = store.local_revision();
    let drafted = same(&store, "a");
    assert_eq!(drafted.size, body.len() as u64);
    // An open dirty draft must never be advertised as cacheable metadata.
    assert!(!drafted.cacheable);
    assert_eq!(*store.handles.lock().unwrap(), active_handles);
    assert_eq!(store.local_revision(), revision);
    store.set_local_mode("a", 0o600).unwrap();
    assert_eq!(same(&store, "a").mode, 0o600);
    store.unlink_local("root", "a", "a", false).unwrap();
    assert!(store.posix_lookup("root", "a").is_err());
    let unlinked = same(&store, "a");
    assert!(unlinked.unlinked);
    assert!(!unlinked.cacheable);
    store
        .draft_write(&draft, body.len() as u64, b"!", false)
        .unwrap();
    assert_eq!(same(&store, "a").size, body.len() as u64 + 1);
}
#[test]
fn acl_overlay_and_local_name_shadow_use_authoritative_visibility() {
    let (_temp, mut store) = fixture();
    let b = store.posix_lookup("d", "b").unwrap();
    store
        .rename_local("d", "b", "root", "moved", false)
        .unwrap();
    store
        .db
        .execute("INSERT INTO access_denied(node) VALUES('d')", [])
        .unwrap();
    assert!(store.posix_record_by_inode(b.ino).is_err());
    assert!(store.posix_lookup("root", "moved").is_err());
    store.db.execute("DELETE FROM access_denied", []).unwrap();
    assert_eq!(store.posix_lookup("root", "moved").unwrap().ino, b.ino);
    let local = store.create_local_file("root", "new", 0o600).unwrap();
    let mut remote = store.node("a").unwrap();
    remote.id = "echo".into();
    remote.name = "new".into();
    store
        .receive(&Batch {
            from: "1".into(),
            to: "2".into(),
            events: vec![Event {
                id: "e".into(),
                change: Change::Upsert(remote),
            }],
        })
        .unwrap();
    store.drain().unwrap();
    assert_eq!(store.posix_lookup("root", "new").unwrap().id, local.id);
    store.unlink_local("root", "new", &local.id, false).unwrap();
    assert_eq!(store.posix_lookup("root", "new").unwrap().id, "echo");
}
#[test]
fn broken_body_or_missing_metadata_is_not_reported_as_successful_stat() {
    let (temp, mut store) = fixture();
    let ino = store.posix_lookup("root", "a").unwrap().ino;
    store
        .db
        .execute("DELETE FROM node_metadata WHERE node='a'", [])
        .unwrap();
    assert!(store.posix_record_by_inode(ino).is_err());
    store
        .publish_download(
            &DownloadTicket {
                id: "a".into(),
                version: "v1".into(),
            },
            b"abc",
            &crate::sha256(b"abc"),
        )
        .unwrap();
    let blob: String = store
        .db
        .query_row("SELECT blob FROM nodes WHERE id='a'", [], |r| r.get(0))
        .unwrap();
    std::fs::remove_file(temp.path().join("blobs").join(blob)).unwrap();
    assert!(store.posix_record_by_inode(ino).is_err());
}
