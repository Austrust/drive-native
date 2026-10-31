use drive_core::*;
use std::io::{Cursor, Read};

#[test]
fn directory_projection_is_bounded_and_does_not_inspect_body_or_status() {
    let dir = tempfile::tempdir().unwrap();
    let mut store = Store::open(dir.path()).unwrap();
    let mut nodes = vec![node("root", "1", 0)];
    nodes.extend((0..2000).map(|i| node(&format!("child-{i:04}"), "1", 3)));
    store.install_remote_snapshot(&nodes, "1", "s").unwrap();
    store.ensure_fs_root("root").unwrap();
    let db = rusqlite::Connection::open(dir.path().join("state.sqlite")).unwrap();
    let inodes = || {
        db.query_row("SELECT count(*) FROM fs_inodes", [], |r| r.get::<_, i64>(0))
            .unwrap()
    };
    assert_eq!(inodes(), 1);
    assert!(store.directory_snapshot(1, 64).is_err());
    assert_eq!(inodes(), 1);
    store
        .publish_download(
            &DownloadTicket {
                id: "child-0000".into(),
                version: "1".into(),
            },
            b"abc",
            &sha256(b"abc"),
        )
        .unwrap();
    for entry in std::fs::read_dir(dir.path().join("blobs")).unwrap() {
        std::fs::remove_file(entry.unwrap().path()).unwrap();
    }
    assert!(store.record("child-0000").is_err()); // old FileRecord enumeration would fail
    let snapshot = store.directory_snapshot(1, 1024 * 1024).unwrap();
    assert_eq!(snapshot.entries.len(), 2002);
    assert_eq!(inodes(), 2001);
    assert_eq!(snapshot.entries[2].name, "child-0000");
    let revision = store.local_revision();
    let repeated = store.directory_snapshot(1, 1024 * 1024).unwrap();
    assert_eq!(snapshot.entries, repeated.entries);
    assert_eq!(store.local_revision(), revision);
    assert!(snapshot.bytes < 1024 * 1024);
    assert_eq!(snapshot.bytes, repeated.bytes);
    assert!(
        store
            .directory_snapshot(snapshot.entries[2].ino, 1024 * 1024)
            .is_err()
    );
}

#[test]
fn desktop_pages_preserve_order_boundaries_and_do_not_allocate_inodes() {
    let dir = tempfile::tempdir().unwrap();
    let mut store = Store::open(dir.path()).unwrap();
    let mut nodes = vec![node("root", "1", 0)];
    for i in (0..513).rev() {
        nodes.push(node(&format!("item-{i:04}"), "1", 3));
    }
    store
        .install_remote_snapshot(&nodes, "1", "server")
        .unwrap();
    store.ensure_fs_root("root").unwrap();
    let db = rusqlite::Connection::open(dir.path().join("state.sqlite")).unwrap();
    let inode_count = || {
        db.query_row("SELECT count(*) FROM fs_inodes", [], |r| r.get::<_, i64>(0))
            .unwrap()
    };
    let before = inode_count();
    let revision = store.local_revision();
    let mut actual = Vec::new();
    for offset in [0, 256, 512, 768] {
        actual.extend(
            store
                .child_nodes_page("root", offset, 256)
                .unwrap()
                .into_iter()
                .map(|n| n.name),
        );
    }
    assert_eq!(
        actual,
        (0..513).map(|i| format!("item-{i:04}")).collect::<Vec<_>>()
    );
    assert_eq!(store.child_node_count("root").unwrap(), 513);
    assert_eq!(store.child_nodes("root").unwrap().len(), 513);
    assert!(store.child_nodes_page("root", 0, 0).unwrap().is_empty());
    assert!(
        store
            .child_nodes_page("root", usize::MAX, 256)
            .unwrap()
            .is_empty()
    );
    assert!(store.child_nodes_page("missing", 0, 1).is_err());
    assert!(store.child_node_count("missing").is_err());
    assert!(store.child_nodes_page("item-0000", 0, 1).is_err());
    assert!(store.child_node_count("item-0000").is_err());
    assert_eq!(store.local_revision(), revision);
    assert_eq!(inode_count(), before);
    assert!(store.downloads_needed().unwrap().is_empty());
    assert_eq!(
        std::fs::read_dir(dir.path().join("blobs")).unwrap().count(),
        0
    );
}

#[test]
fn desktop_pages_match_local_creation_shadow_and_deleted_visibility() {
    let dir = tempfile::tempdir().unwrap();
    let mut store = Store::open(dir.path()).unwrap();
    setup(&mut store);
    let local = store.create_local_file("root", "duplicate", 0o644).unwrap();
    let mut remote = node("server-new", "2", 3);
    remote.name = "duplicate".into();
    event(&mut store, "1", "2", remote);
    let children = store.child_nodes_page("root", 0, 256).unwrap();
    assert_eq!(
        children.iter().map(|n| n.id.as_str()).collect::<Vec<_>>(),
        vec!["a", &local.id]
    );
    assert_eq!(store.child_node_count("root").unwrap(), 2);
    assert_eq!(
        store.child_nodes_page("root", 1, 1).unwrap()[0].id,
        local.id
    );
    let dir_entries = store.directory_snapshot(1, 1024 * 1024).unwrap();
    assert_eq!(
        dir_entries
            .entries
            .iter()
            .map(|e| e.name.as_str())
            .collect::<Vec<_>>(),
        vec![".", "..", "a", "duplicate"]
    );
    store
        .receive(&Batch {
            from: "2".into(),
            to: "3".into(),
            events: vec![Event {
                id: "deleted-a".into(),
                change: Change::Delete { id: "a".into() },
            }],
        })
        .unwrap();
    store.drain().unwrap();
    assert_eq!(store.child_node_count("root").unwrap(), 1);
    assert_eq!(
        store.child_nodes_page("root", 0, 256).unwrap()[0].id,
        local.id
    );
}

fn node(id: &str, version: &str, size: u64) -> Node {
    Node {
        id: id.into(),
        parent: if id == "root" {
            None
        } else {
            Some("root".into())
        },
        name: id.into(),
        version: version.into(),
        directory: id == "root",
        metadata: Some(NodeMetadata {
            size,
            mtime: 1700000000,
            mode: if id == "root" { 0o755 } else { 0o644 },
        }),
    }
}
fn setup(s: &mut Store) {
    s.install_remote_snapshot(&[node("root", "1", 0), node("a", "1", 3)], "1", "server")
        .unwrap();
    s.ensure_fs_root("root").unwrap();
}
fn event(s: &mut Store, from: &str, to: &str, n: Node) {
    s.receive(&Batch {
        from: from.into(),
        to: to.into(),
        events: vec![Event {
            id: to.into(),
            change: Change::Upsert(n),
        }],
    })
    .unwrap();
    s.drain().unwrap();
}
#[test]
fn lookup_is_metadata_only_and_inode_survives_move_restart_and_replacement() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = Store::open(dir.path()).unwrap();
    setup(&mut s);
    let first = s.lookup("root", "a").unwrap();
    assert_eq!(first.size, 3);
    assert_eq!(first.status, Status::OnlineOnly);
    assert!(s.open_cached("a").is_err());
    assert!(s.downloads_needed().unwrap().is_empty());
    assert_eq!(s.node("a").unwrap().metadata.unwrap().size, 3);
    let mut moved = node("a", "2", 500);
    moved.name = "renamed.doc".into();
    event(&mut s, "1", "2", moved);
    assert!(s.lookup("root", "a").is_err());
    assert_eq!(s.lookup("root", "renamed.doc").unwrap().ino, first.ino);
    let mut replacement = node("b", "3", 7);
    replacement.name = "a".into();
    event(&mut s, "2", "3", replacement);
    let new = s.lookup("root", "a").unwrap();
    assert_ne!(new.ino, first.ino);
    drop(s);
    let s = Store::open(dir.path()).unwrap();
    s.ensure_fs_root("root").unwrap();
    assert_eq!(
        s.record_by_inode(first.ino).unwrap().node.name,
        "renamed.doc"
    );
    assert_eq!(s.children("root").unwrap().len(), 2);
    assert_eq!(s.record("root").unwrap().ino, 1);
}
#[test]
fn failed_refresh_reports_actual_old_content_size_and_preserves_open_version() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = Store::open(dir.path()).unwrap();
    setup(&mut s);
    s.publish_download(
        &DownloadTicket {
            id: "a".into(),
            version: "1".into(),
        },
        b"old",
        &sha256(b"old"),
    )
    .unwrap();
    let mut old = s.open_cached("a").unwrap();
    event(&mut s, "1", "2", node("a", "2", 15));
    assert_eq!(s.record("a").unwrap().size, 3);
    assert_eq!(s.record("a").unwrap().status, Status::Updating);
    s.publish_download(
        &DownloadTicket {
            id: "a".into(),
            version: "2".into(),
        },
        b"new content now",
        &sha256(b"new content now"),
    )
    .unwrap();
    assert_eq!(s.record("a").unwrap().size, 15);
    let mut bytes = Vec::new();
    old.read_to_end(&mut bytes).unwrap();
    assert_eq!(bytes, b"old");
}
#[test]
fn streamed_local_save_failure_preserves_previous_durable_edit() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = Store::open(dir.path()).unwrap();
    setup(&mut s);
    s.save_local_reader_from_version("a", "1", &mut Cursor::new(b"durable edit"))
        .unwrap();
    struct Broken(bool);
    impl Read for Broken {
        fn read(&mut self, out: &mut [u8]) -> std::io::Result<usize> {
            if self.0 {
                return Err(std::io::Error::other("injected disk/read fault"));
            }
            self.0 = true;
            out[0] = 42;
            Ok(1)
        }
    }
    assert!(
        s.save_local_reader_from_version("a", "1", &mut Broken(false))
            .is_err()
    );
    assert_eq!(s.record("a").unwrap().size, 12);
    assert_eq!(s.record("a").unwrap().status, Status::PendingUpload);
    drop(s);
    let s = Store::open(dir.path()).unwrap();
    let mut bytes = Vec::new();
    s.open_cached("a").unwrap().read_to_end(&mut bytes).unwrap();
    assert_eq!(bytes, b"durable edit");
}
#[test]
fn v2_migration_requires_missing_metadata_recovery_without_faking_zero_length() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = Store::open(dir.path()).unwrap();
    setup(&mut s);
    drop(s);
    let db = rusqlite::Connection::open(dir.path().join("state.sqlite")).unwrap();
    db.execute_batch("DROP TABLE fs_inodes; DROP TABLE node_metadata; PRAGMA user_version=2;")
        .unwrap();
    drop(db);
    let mut s = Store::open(dir.path()).unwrap();
    s.ensure_fs_root("root").unwrap();
    assert!(s.record("a").is_err());
    assert!(std::fs::read_dir(dir.path()).unwrap().any(|e| {
        e.unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with(&format!(
                "state-before-v{}-",
                drive_core::STATE_SCHEMA_VERSION
            ))
    }));
    event(&mut s, "1", "2", node("a", "2", 3));
    assert_eq!(s.record("a").unwrap().size, 3);
}
#[test]
fn cancelling_pin_inherits_parent_rule_and_does_not_evict_content() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = Store::open(dir.path()).unwrap();
    setup(&mut s);
    s.publish_download(
        &DownloadTicket {
            id: "a".into(),
            version: "1".into(),
        },
        b"old",
        &sha256(b"old"),
    )
    .unwrap();
    s.pin("root", true).unwrap();
    s.pin("a", true).unwrap();
    s.pin("a", false).unwrap();
    assert!(s.is_pinned("a").unwrap());
    assert!(s.release("a").is_err());
    s.pin("root", false).unwrap();
    assert!(!s.is_pinned("a").unwrap());
    assert!(s.open_cached("a").is_ok());
    s.release("a").unwrap();
    assert_eq!(s.status("a").unwrap(), Status::OnlineOnly);
    assert_eq!(s.pending_operations().unwrap(), 0);
}

#[test]
fn local_permissions_survive_remote_refresh_rename_and_restart_without_upload() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = Store::open(dir.path()).unwrap();
    setup(&mut s);
    s.set_local_mode("a", 0o640).unwrap();
    let mut updated = node("a", "2", 18);
    updated.name = "renamed.doc".into();
    event(&mut s, "1", "2", updated);
    assert_eq!(s.record("a").unwrap().mode, 0o640);
    assert_eq!(s.node("a").unwrap().metadata.unwrap().mode, 0o644);
    assert_eq!(s.pending_operations().unwrap(), 0);
    assert!(s.downloads_needed().unwrap().is_empty());
    assert!(s.set_local_mode("a", 0o4644).is_err());
    drop(s);
    let s = Store::open(dir.path()).unwrap();
    assert_eq!(s.lookup("root", "renamed.doc").unwrap().mode, 0o640);
}

#[test]
fn permissions_follow_inode_not_reused_name_or_server_identity() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = Store::open(dir.path()).unwrap();
    setup(&mut s);
    s.set_local_mode("a", 0o600).unwrap();
    let old = s.record("a").unwrap().ino;
    s.unlink_local("root", "a", "a", false).unwrap();
    let replacement = s.create_local_file("root", "a", 0o644).unwrap();
    assert_eq!(s.record_by_inode(old).unwrap().mode, 0o600);
    assert_eq!(s.record(&replacement.id).unwrap().mode, 0o644);
}

#[test]
fn v8_permissions_migration_backs_up_existing_state_and_preserves_contents() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = Store::open(dir.path()).unwrap();
    setup(&mut s);
    s.save_local("a", b"unuploaded").unwrap();
    drop(s);
    let db = rusqlite::Connection::open(dir.path().join("state.sqlite")).unwrap();
    db.execute_batch("DROP TABLE local_modes; PRAGMA user_version=8;")
        .unwrap();
    drop(db);
    let mut s = Store::open(dir.path()).unwrap();
    s.set_local_mode("a", 0o600).unwrap();
    let mut body = Vec::new();
    s.open_cached("a").unwrap().read_to_end(&mut body).unwrap();
    assert_eq!(body, b"unuploaded");
    assert!(std::fs::read_dir(dir.path()).unwrap().any(|e| {
        e.unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with(&format!(
                "state-before-v{}-",
                drive_core::STATE_SCHEMA_VERSION
            ))
    }));
}
