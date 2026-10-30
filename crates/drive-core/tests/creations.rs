use drive_core::*;
fn root() -> Node {
    Node {
        id: "root".into(),
        parent: None,
        name: "root".into(),
        version: "root-v1".into(),
        directory: true,
        metadata: None,
    }
}
fn setup() -> (tempfile::TempDir, Store) {
    let dir = tempfile::tempdir().unwrap();
    let mut s = Store::open(dir.path()).unwrap();
    s.install_remote_snapshot(&[root()], "1", "server").unwrap();
    s.ensure_fs_root("root").unwrap();
    (dir, s)
}
fn echo(id: &str, version: &str, name: &str) -> Node {
    Node {
        id: id.into(),
        parent: Some("root".into()),
        name: name.into(),
        version: version.into(),
        directory: false,
        metadata: Some(NodeMetadata {
            size: 3,
            mtime: 1,
            mode: 0o644,
        }),
    }
}
#[test]
fn creation_is_durable_but_not_submitted_before_save() {
    let (dir, mut s) = setup();
    let n = s.create_local_file("root", "new.doc", 0o644).unwrap();
    assert!(s.create_local_file("root", "new.doc", 0o644).is_err());
    let op = s.upload_operations("pending").unwrap().pop().unwrap();
    assert!(s.mark_upload_started(&op).is_err());
    let inode = s.record(&n.id).unwrap().ino;
    drop(s);
    let mut s = Store::open(dir.path()).unwrap();
    assert_eq!(s.lookup("root", "new.doc").unwrap().ino, inode);
    s.creation_saved(&n.id).unwrap();
    s.mark_upload_started(&op).unwrap();
}
#[test]
fn early_server_echo_folds_into_local_identity_and_duplicates_remain_idempotent() {
    let (_dir, mut s) = setup();
    let local = s.create_local_file("root", "new.doc", 0o644).unwrap();
    let ino = s.record(&local.id).unwrap().ino;
    let op = s.save_local(&local.id, b"new").unwrap();
    s.creation_saved(&local.id).unwrap();
    s.mark_upload_started(&op).unwrap();
    let event = Event {
        id: "event1".into(),
        change: Change::Upsert(echo("99", "remote-v1", "new.doc")),
    };
    s.receive(&Batch {
        from: "1".into(),
        to: "2".into(),
        events: vec![event.clone()],
    })
    .unwrap();
    s.drain().unwrap();
    assert_eq!(s.children("root").unwrap().len(), 1);
    assert_eq!(s.lookup("root", "new.doc").unwrap().ino, ino);
    s.confirm_upload_identity(&op, "remote-v1", Some("99"))
        .unwrap();
    assert_eq!(s.local_identity("99").unwrap(), local.id);
    assert_eq!(s.lookup("root", "new.doc").unwrap().ino, ino);
    assert_eq!(s.status(&local.id).unwrap(), Status::LocalComplete);
    s.receive(&Batch {
        from: "2".into(),
        to: "3".into(),
        events: vec![event],
    })
    .unwrap();
    s.drain().unwrap();
    s.receive(&Batch {
        from: "3".into(),
        to: "4".into(),
        events: vec![Event {
            id: "event2".into(),
            change: Change::Upsert(echo("99", "remote-v2", "renamed.doc")),
        }],
    })
    .unwrap();
    s.drain().unwrap();
    assert_eq!(s.lookup("root", "renamed.doc").unwrap().ino, ino);
    assert!(s.lookup("root", "new.doc").is_err());
    assert_eq!(s.downloads_needed().unwrap()[0].id, local.id);
}
#[test]
fn snapshot_does_not_delete_unsubmitted_creation_and_maps_bound_identity() {
    let (_dir, mut s) = setup();
    let local = s.create_local_file("root", "new.doc", 0o644).unwrap();
    s.install_remote_snapshot(&[root()], "2", "server").unwrap();
    assert_eq!(s.status(&local.id).unwrap(), Status::PendingUpload);
    let op = s.upload_operations("pending").unwrap().pop().unwrap();
    s.creation_saved(&local.id).unwrap();
    s.mark_upload_started(&op).unwrap();
    s.confirm_upload_identity(&op, "remote-v1", Some("99"))
        .unwrap();
    s.install_remote_snapshot(&[root(), echo("99", "remote-v1", "new.doc")], "3", "server")
        .unwrap();
    assert_eq!(s.lookup("root", "new.doc").unwrap().node.id, local.id);
}
#[test]
fn newer_remote_echo_cannot_be_erased_by_creation_receipt() {
    let (_dir, mut s) = setup();
    let local = s.create_local_file("root", "new.doc", 0o644).unwrap();
    let op = s.save_local(&local.id, b"new").unwrap();
    s.creation_saved(&local.id).unwrap();
    s.mark_upload_started(&op).unwrap();
    s.receive(&Batch {
        from: "1".into(),
        to: "2".into(),
        events: vec![Event {
            id: "later".into(),
            change: Change::Upsert(echo("99", "remote-v2", "new.doc")),
        }],
    })
    .unwrap();
    s.drain().unwrap();
    assert!(
        s.confirm_upload_identity(&op, "remote-v1", Some("99"))
            .is_err()
    );
    assert_eq!(s.node("99").unwrap().version, "remote-v2");
    assert_eq!(s.local_identity("99").unwrap(), "99");
    assert_eq!(s.status(&local.id).unwrap(), Status::Conflict);
}

#[test]
fn v4_upgrade_preserves_cached_and_unsubmitted_content_and_saves_consistent_backup() {
    use std::io::Read;
    let (dir, mut s) = setup();
    let remote = echo("99", "remote-v1", "old.doc");
    s.install_remote_snapshot(&[root(), remote], "2", "server")
        .unwrap();
    s.publish_download(
        &DownloadTicket {
            id: "99".into(),
            version: "remote-v1".into(),
        },
        b"old",
        &sha256(b"old"),
    )
    .unwrap();
    let op = s.save_local("99", b"unsubmitted edit").unwrap();
    drop(s);
    let db = rusqlite::Connection::open(dir.path().join("state.sqlite")).unwrap();
    db.execute_batch("DROP TABLE remote_aliases;DROP TABLE local_creations;PRAGMA user_version=4;")
        .unwrap();
    drop(db);
    let s = Store::open(dir.path()).unwrap();
    let mut bytes = Vec::new();
    s.open_cached("99")
        .unwrap()
        .read_to_end(&mut bytes)
        .unwrap();
    assert_eq!(bytes, b"unsubmitted edit");
    assert_eq!(s.upload_operations("pending").unwrap(), vec![op]);
    let backups: Vec<_> = std::fs::read_dir(dir.path())
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| {
            p.file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with(&format!(
                    "state-before-v{}-",
                    drive_core::STATE_SCHEMA_VERSION
                ))
        })
        .collect();
    assert_eq!(backups.len(), 1);
    let backup = rusqlite::Connection::open(&backups[0]).unwrap();
    let version: i64 = backup
        .pragma_query_value(None, "user_version", |r| r.get(0))
        .unwrap();
    assert_eq!(version, 4);
    let rows: i64 = backup
        .query_row(
            "SELECT count(*) FROM operations WHERE state='pending'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(rows, 1);
}
