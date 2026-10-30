use drive_core::*;
fn node(id: &str, parent: Option<&str>, name: &str, version: &str, directory: bool) -> Node {
    Node {
        id: id.into(),
        parent: parent.map(str::to_owned),
        name: name.into(),
        version: version.into(),
        directory,
        metadata: Some(NodeMetadata {
            size: 3,
            mtime: 1,
            mode: 0o644,
        }),
    }
}
fn root() -> Node {
    node("1", None, "root", "r1", true)
}
fn file(version: &str) -> Node {
    node("2", Some("1"), "A.doc", version, false)
}
fn fixture() -> (tempfile::TempDir, Store) {
    let d = tempfile::tempdir().unwrap();
    let mut s = Store::open(d.path()).unwrap();
    s.install_remote_snapshot(&[root(), file("v1")], "1", "server")
        .unwrap();
    s.ensure_fs_root("1").unwrap();
    s.publish_download(
        &DownloadTicket {
            id: "2".into(),
            version: "v1".into(),
        },
        b"old",
        &sha256(b"old"),
    )
    .unwrap();
    (d, s)
}
fn event(s: &mut Store, change: Change, to: &str) {
    s.receive(&Batch {
        from: s.cursor().unwrap(),
        to: to.into(),
        events: vec![Event {
            id: to.into(),
            change,
        }],
    })
    .unwrap();
    s.drain().unwrap();
}
#[test]
fn unlink_open_file_preserves_handle_and_same_name_new_identity() {
    let (_d, mut s) = fixture();
    let old = s.open_cached("2").unwrap();
    let inode = s.record("2").unwrap().ino;
    s.unlink_local("1", "A.doc", "2", false).unwrap();
    assert!(s.lookup("1", "A.doc").is_err());
    let record = s.record_by_inode(inode).unwrap();
    assert!(record.unlinked);
    assert_eq!(record.status, Status::Deleted);
    let draft = s.begin_draft("2", "v1", Some(&old)).unwrap();
    s.draft_write(&draft, 0, b"orphan", false).unwrap();
    assert!(s.checkpoint_draft(&draft).unwrap().is_none());
    assert!(s.upload_operations("pending").unwrap().is_empty());
    let new = s.create_local_file("1", "A.doc", 0o644).unwrap();
    assert_ne!(new.id, "2");
    assert_ne!(s.record(&new.id).unwrap().ino, inode);
    s.creation_saved(&new.id).unwrap();
    let upload = s.upload_operations("pending").unwrap().remove(0);
    assert!(s.mark_upload_started(&upload).is_err());
    let q = s.next_remove().unwrap().unwrap();
    s.mark_remove_started(&q.job.operation).unwrap();
    assert!(s.confirm_remove(&q.job.operation, &new.id).is_err());
    s.confirm_remove(&q.job.operation, "2").unwrap();
    s.mark_upload_started(&upload).unwrap();
    let mut buf = [0; 6];
    assert_eq!(draft.read_at(&mut buf, 0).unwrap(), 6);
    assert_eq!(&buf, b"orphan");
    let mut buf = [0; 3];
    old.read_at(&mut buf, 0).unwrap();
    assert_eq!(&buf, b"old");
}
#[test]
fn never_submitted_file_deleted_locally_makes_no_native_remove() {
    let (_d, mut s) = fixture();
    let n = s.create_local_file("1", "tmp", 0o600).unwrap();
    let old = s.open_cached(&n.id).unwrap();
    s.unlink_local("1", "tmp", &n.id, false).unwrap();
    assert!(s.next_remove().unwrap().is_none());
    assert!(s.upload_operations("pending").unwrap().is_empty());
    let d = s.begin_draft(&n.id, &n.version, Some(&old)).unwrap();
    s.draft_write(&d, 0, b"invisible", false).unwrap();
    assert!(s.checkpoint_draft(&d).unwrap().is_none());
    assert_eq!(s.status(&n.id).unwrap(), Status::Deleted);
}
#[test]
fn inflight_upload_reconciles_after_unlink_and_rebases_remove() {
    let (dir, mut s) = fixture();
    let op = s.save_local("2", b"new").unwrap();
    s.mark_upload_started(&op).unwrap();
    s.unlink_local("1", "A.doc", "2", false).unwrap();
    let q = s.next_remove().unwrap().unwrap();
    assert!(s.mark_remove_started(&q.job.operation).is_err());
    drop(s);
    let mut s = Store::open(dir.path()).unwrap();
    assert_eq!(s.upload_job(&op).unwrap().relative_path, "A.doc");
    s.confirm_upload(&op, "v2").unwrap();
    let q = s.next_remove().unwrap().unwrap();
    assert_eq!(q.job.node.version, "v2");
    s.mark_remove_started(&q.job.operation).unwrap();
    drop(s);
    let s = Store::open(dir.path()).unwrap();
    assert_eq!(s.next_remove().unwrap().unwrap().state, "uncertain");
    assert!(s.lookup("1", "A.doc").is_err());
}
#[test]
fn creation_inflight_then_unlink_binds_remote_identity_before_remove() {
    let (_d, mut s) = fixture();
    let n = s.create_local_file("1", "new", 0o600).unwrap();
    s.creation_saved(&n.id).unwrap();
    let op = s.upload_operations("pending").unwrap().remove(0);
    s.mark_upload_started(&op).unwrap();
    s.unlink_local("1", "new", &n.id, false).unwrap();
    s.confirm_upload_identity(&op, "native-v1", Some("300"))
        .unwrap();
    assert_eq!(s.local_identity("300").unwrap(), n.id);
    let q = s.next_remove().unwrap().unwrap();
    assert_eq!(q.job.node.version, "native-v1");
    s.mark_remove_started(&q.job.operation).unwrap();
    assert!(s.lookup("1", "new").is_err());
}
#[test]
fn rename_chain_then_unlink_keeps_global_order() {
    let (_d, mut s) = fixture();
    s.rename_local("1", "A.doc", "1", "B.doc", false).unwrap();
    s.rename_local("1", "B.doc", "1", "C.doc", false).unwrap();
    s.unlink_local("1", "C.doc", "2", false).unwrap();
    assert!(s.next_remove().unwrap().is_none());
    for (name, version) in [("B.doc", "v2"), ("C.doc", "v3")] {
        let q = s.next_rename().unwrap().unwrap();
        s.mark_rename_started(&q.job.operation).unwrap();
        let mut n = file(version);
        n.name = name.into();
        event(&mut s, Change::Upsert(n.clone()), version);
        s.confirm_rename(&q.job.operation, &n, true).unwrap();
        assert!(s.lookup("1", name).is_err());
    }
    assert!(s.next_rename().unwrap().is_none());
    let q = s.next_remove().unwrap().unwrap();
    assert_eq!(q.job.path, "C.doc");
    assert_eq!(q.job.node.version, "v3");
    s.mark_remove_started(&q.job.operation).unwrap();
    s.confirm_remove(&q.job.operation, "2").unwrap();
}
#[test]
fn server_echo_snapshot_and_tombstone_do_not_resurrect_or_self_conflict() {
    let (_d, mut s) = fixture();
    s.unlink_local("1", "A.doc", "2", false).unwrap();
    let q = s.next_remove().unwrap().unwrap();
    s.mark_remove_started(&q.job.operation).unwrap();
    event(&mut s, Change::Upsert(file("v1")), "2");
    assert!(s.lookup("1", "A.doc").is_err());
    s.install_remote_snapshot(&[root()], "3", "server").unwrap();
    assert_eq!(s.status("2").unwrap(), Status::Deleted);
    event(&mut s, Change::Delete { id: "2".into() }, "4");
    assert_eq!(s.status("2").unwrap(), Status::Deleted);
    assert_eq!(s.next_remove().unwrap().unwrap().state, "inflight"); // event alone is not receipt
    s.confirm_remove(&q.job.operation, "2").unwrap();
    event(&mut s, Change::Upsert(file("v1")), "5");
    assert!(s.lookup("1", "A.doc").is_err());
}
#[test]
fn unlink_checks_type_nonempty_and_captured_identity() {
    let (_d, mut s) = fixture();
    assert!(s.unlink_local("1", "A.doc", "wrong", false).is_err());
    assert!(s.unlink_local("1", "A.doc", "2", true).is_err());
    let dir = node("3", Some("1"), "dir", "d1", true);
    let child = node("4", Some("3"), "kid", "c1", false);
    event(&mut s, Change::Upsert(dir), "2");
    event(&mut s, Change::Upsert(child), "3");
    assert!(s.unlink_local("1", "dir", "3", false).is_err());
    assert!(s.unlink_local("1", "dir", "3", true).is_err());
    s.unlink_local("3", "kid", "4", false).unwrap();
    s.unlink_local("1", "dir", "3", true).unwrap();
    let q = s.next_remove().unwrap().unwrap();
    assert_eq!(q.job.node.id, "4");
    s.mark_remove_started(&q.job.operation).unwrap();
    s.confirm_remove(&q.job.operation, "4").unwrap();
    assert_eq!(s.next_remove().unwrap().unwrap().job.node.id, "3");
}
#[test]
fn orphan_checkpoint_survives_restart_without_upload_and_unrelated_state_is_clean() {
    let (d, mut s) = fixture();
    let open = s.open_cached("2").unwrap();
    let draft = s.begin_draft("2", "v1", Some(&open)).unwrap();
    s.draft_write(&draft, 0, b"edit", false).unwrap();
    s.checkpoint_draft(&draft).unwrap();
    s.unlink_local("1", "A.doc", "2", false).unwrap();
    s.draft_write(&draft, 0, b"after", false).unwrap();
    assert!(s.checkpoint_draft(&draft).unwrap().is_none());
    drop(draft);
    drop(open);
    drop(s);
    let mut s = Store::open(d.path()).unwrap();
    assert_eq!(s.checkpoint_recovered_drafts().unwrap(), 0);
    assert!(s.upload_operations("pending").unwrap().is_empty());
    let draft = s.open_dirty_draft("2").unwrap().unwrap();
    let mut buf = [0; 5];
    draft.read_at(&mut buf, 0).unwrap();
    assert_eq!(&buf, b"after");
    assert_ne!(s.status("1").unwrap(), Status::PendingUpload);
}
#[test]
fn v6_migration_preserves_queued_move_and_inflight_becomes_uncertain() {
    let (d, mut s) = fixture();
    s.rename_local("1", "A.doc", "1", "B.doc", false).unwrap();
    let q = s.next_rename().unwrap().unwrap();
    s.mark_rename_started(&q.job.operation).unwrap();
    drop(s);
    let db = rusqlite::Connection::open(d.path().join("state.sqlite")).unwrap();
    db.execute_batch("DROP TABLE local_unlinks; ALTER TABLE namespace_ops DROP COLUMN kind; PRAGMA user_version=6;").unwrap();
    drop(db);
    let s = Store::open(d.path()).unwrap();
    assert_eq!(s.next_rename().unwrap().unwrap().state, "uncertain");
    assert_eq!(s.lookup("1", "B.doc").unwrap().node.id, "2");
    assert!(std::fs::read_dir(d.path()).unwrap().any(|e| {
        e.unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with(&format!(
                "state-before-v{}-",
                drive_core::STATE_SCHEMA_VERSION
            ))
    }));
}
