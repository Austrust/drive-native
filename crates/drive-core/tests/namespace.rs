use drive_core::*;
fn n(id: &str, parent: Option<&str>, name: &str, version: &str, dir: bool) -> Node {
    Node {
        id: id.into(),
        parent: parent.map(str::to_owned),
        name: name.into(),
        version: version.into(),
        directory: dir,
        metadata: Some(NodeMetadata {
            size: if dir { 0 } else { 3 },
            mtime: 1,
            mode: 0o644,
        }),
    }
}
fn root() -> Node {
    n("1", None, "root", "root-v1", true)
}
fn file(version: &str, name: &str) -> Node {
    n("2", Some("1"), name, version, false)
}
fn fixture() -> (tempfile::TempDir, Store) {
    let d = tempfile::tempdir().unwrap();
    let mut s = Store::open(d.path()).unwrap();
    s.install_remote_snapshot(&[root(), file("v1", "A.doc")], "1", "server")
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
fn event(s: &mut Store, node: Node, to: &str) {
    s.receive(&Batch {
        from: s.cursor().unwrap(),
        to: to.into(),
        events: vec![Event {
            id: format!("event-{to}"),
            change: Change::Upsert(node),
        }],
    })
    .unwrap();
    s.drain().unwrap();
}
#[test]
fn chained_offline_moves_keep_inode_and_late_echo_cannot_undo_local_path() {
    let (d, mut s) = fixture();
    let inode = s.record("2").unwrap().ino;
    let mut open = s.open_cached("2").unwrap();
    s.rename_local("1", "A.doc", "1", "B.doc", false).unwrap();
    s.rename_local("1", "B.doc", "1", "C.doc", false).unwrap();
    assert!(s.lookup("1", "A.doc").is_err());
    assert_eq!(s.lookup("1", "C.doc").unwrap().ino, inode);
    let first = s.next_rename().unwrap().unwrap();
    s.mark_rename_started(&first.job.operation).unwrap();
    event(&mut s, file("v2", "B.doc"), "2");
    assert_eq!(s.node("2").unwrap().name, "C.doc");
    drop(s);
    let mut s = Store::open(d.path()).unwrap();
    assert_eq!(s.next_rename().unwrap().unwrap().state, "uncertain");
    s.confirm_rename(&first.job.operation, &file("v2", "B.doc"), true)
        .unwrap();
    let second = s.next_rename().unwrap().unwrap();
    assert_eq!(second.job.node.version, "v2");
    assert_eq!(second.job.source, "B.doc");
    s.mark_rename_started(&second.job.operation).unwrap();
    s.confirm_rename(&second.job.operation, &file("v3", "C.doc"), true)
        .unwrap();
    assert!(s.next_rename().unwrap().is_none());
    assert_eq!(s.record("2").unwrap().ino, inode);
    assert_eq!(s.status("2").unwrap(), Status::LocalComplete);
    assert_eq!(s.open_cached("2").unwrap().version(), "v3");
    let mut data = Vec::new();
    std::io::Read::read_to_end(&mut open, &mut data).unwrap();
    assert_eq!(data, b"old");
}
#[test]
fn pending_write_waits_for_move_then_uses_confirmed_base() {
    let (_d, mut s) = fixture();
    let upload = s.save_local("2", b"edited").unwrap();
    s.rename_local("1", "A.doc", "1", "B.doc", false).unwrap();
    assert!(s.mark_upload_started(&upload).is_err());
    let q = s.next_rename().unwrap().unwrap();
    s.mark_rename_started(&q.job.operation).unwrap();
    event(&mut s, file("v2", "B.doc"), "2");
    s.confirm_rename(&q.job.operation, &file("v2", "B.doc"), true)
        .unwrap();
    let upload_job = s.upload_job(&upload).unwrap();
    assert_eq!(upload_job.relative_path, "B.doc");
    assert_eq!(upload_job.node.version, "v2");
    s.mark_upload_started(&upload).unwrap();
}
#[test]
fn already_started_upload_finishes_before_move_and_rebases_it() {
    let (_d, mut s) = fixture();
    let upload = s.save_local("2", b"edited").unwrap();
    s.mark_upload_started(&upload).unwrap();
    s.rename_local("1", "A.doc", "1", "B.doc", false).unwrap();
    let q = s.next_rename().unwrap().unwrap();
    assert!(s.mark_rename_started(&q.job.operation).is_err());
    s.confirm_upload(&upload, "upload-v2").unwrap();
    let q = s.next_rename().unwrap().unwrap();
    assert_eq!(q.job.node.version, "upload-v2");
    s.mark_rename_started(&q.job.operation).unwrap();
    s.confirm_rename(&q.job.operation, &file("move-v3", "B.doc"), true)
        .unwrap();
    assert_eq!(s.node("2").unwrap().name, "B.doc");
}
#[test]
fn remote_change_after_ack_is_not_regressed_or_uploaded_over() {
    let (_d, mut s) = fixture();
    let upload = s.save_local("2", b"edit").unwrap();
    s.rename_local("1", "A.doc", "1", "B.doc", false).unwrap();
    let q = s.next_rename().unwrap().unwrap();
    s.mark_rename_started(&q.job.operation).unwrap();
    event(&mut s, file("remote-v3", "C.doc"), "2");
    s.confirm_rename(&q.job.operation, &file("v2", "B.doc"), true)
        .unwrap();
    assert_eq!(s.node("2").unwrap().version, "remote-v3");
    assert_eq!(s.node("2").unwrap().name, "C.doc");
    assert_eq!(s.status("2").unwrap(), Status::Conflict);
    assert!(s.mark_upload_started(&upload).is_err());
}
#[test]
fn recovery_keeps_local_move_and_remote_deletion_remains_conflict() {
    let (d, mut s) = fixture();
    s.rename_local("1", "A.doc", "1", "B.doc", false).unwrap();
    s.install_remote_snapshot(&[root(), file("v1", "A.doc")], "3", "server")
        .unwrap();
    assert_eq!(s.node("2").unwrap().name, "B.doc");
    s.install_remote_snapshot(&[root()], "4", "server").unwrap();
    assert_eq!(s.status("2").unwrap(), Status::Conflict);
    drop(s);
    let s = Store::open(d.path()).unwrap();
    assert_eq!(s.node("2").unwrap().name, "B.doc");
    assert!(s.open_cached("2").is_ok());
}
#[test]
fn never_submitted_creation_moves_without_creating_a_remote_rename() {
    let (_d, mut s) = fixture();
    let local = s.create_local_file("1", "temp", 0o644).unwrap();
    s.rename_local("1", "temp", "1", "new.doc", false).unwrap();
    assert!(s.next_rename().unwrap().is_none());
    let op = s.upload_operations("pending").unwrap().pop().unwrap();
    assert_eq!(s.upload_job(&op).unwrap().relative_path, "new.doc");
    assert_eq!(s.lookup("1", "new.doc").unwrap().node.id, local.id);
}
#[test]
fn invalid_rename_never_changes_existing_namespace() {
    let (_d, mut s) = fixture();
    assert!(s.rename_local("1", "A.doc", "1", "A.doc", true).is_err());
    s.rename_local("1", "A.doc", "1", "A.doc", false).unwrap();

    s.create_local_file("1", "occupied", 0o644).unwrap();
    assert!(s.rename_local("1", "A.doc", "1", "occupied", true).is_err());
    assert!(
        s.rename_local("1", "A.doc", "1", "occupied", false)
            .is_err()
    );
    assert!(s.rename_local("1", "A.doc", "1", "..", false).is_err());
    assert!(s.next_rename().unwrap().is_none());
    assert_eq!(s.lookup("1", "A.doc").unwrap().node.id, "2");
}

#[test]
fn old_open_content_version_can_edit_after_pure_move_but_not_after_new_content() {
    let (_d, mut s) = fixture();
    let open = s.open_cached("2").unwrap();
    s.rename_local("1", "A.doc", "1", "B.doc", false).unwrap();
    let q = s.next_rename().unwrap().unwrap();
    s.mark_rename_started(&q.job.operation).unwrap();
    s.confirm_rename(&q.job.operation, &file("v2", "B.doc"), true)
        .unwrap();
    let draft = s.begin_draft("2", open.version(), Some(&open)).unwrap();
    s.draft_write(&draft, 0, b"new", false).unwrap();
    let op = s.checkpoint_draft(&draft).unwrap().unwrap();
    assert_eq!(s.upload_job(&op).unwrap().node.version, "v2");
    s.mark_upload_started(&op).unwrap();
    s.confirm_upload(&op, "v3").unwrap();
    let op = s
        .save_local_from_version("2", open.version(), b"stale")
        .unwrap();
    assert_eq!(s.status("2").unwrap(), Status::Conflict);
    assert!(s.mark_upload_started(&op).is_err());
}
#[test]
fn uncertain_upload_retains_submitted_path_after_local_move_and_restart() {
    let (d, mut s) = fixture();
    let op = s.save_local("2", b"new").unwrap();
    s.mark_upload_started(&op).unwrap();
    s.rename_local("1", "A.doc", "1", "B.doc", false).unwrap();
    drop(s);
    let mut s = Store::open(d.path()).unwrap();
    assert_eq!(s.upload_job(&op).unwrap().relative_path, "A.doc");
    assert_eq!(s.upload_job(&op).unwrap().state, "uncertain");
    let q = s.next_rename().unwrap().unwrap();
    assert!(s.mark_rename_started(&q.job.operation).is_err());
    s.confirm_upload(&op, "v2").unwrap();
    s.mark_rename_started(&q.job.operation).unwrap();
}
#[test]
fn remote_ancestor_deletion_does_not_silently_accept_a_pending_move_out() {
    let (_d, mut s) = fixture();
    let dir = n("3", Some("1"), "folder", "dir-v1", true);
    let child = n("4", Some("3"), "child", "child-v1", false);
    s.install_remote_snapshot(&[root(), file("v1", "A.doc"), dir, child], "2", "server")
        .unwrap();
    s.rename_local("3", "child", "1", "moved", false).unwrap();
    s.receive(&Batch {
        from: "2".into(),
        to: "3".into(),
        events: vec![Event {
            id: "delete-parent".into(),
            change: Change::Delete { id: "3".into() },
        }],
    })
    .unwrap();
    s.drain().unwrap();
    assert_eq!(s.node("4").unwrap().name, "moved");
    assert_eq!(s.status("4").unwrap(), Status::Conflict);
    assert_eq!(s.next_rename().unwrap().unwrap().state, "conflict");
}

#[test]
fn late_pre_ack_move_event_cannot_revert_a_confirmed_chain() {
    let (_d, mut s) = fixture();
    s.rename_local("1", "A.doc", "1", "B.doc", false).unwrap();
    let first = s.next_rename().unwrap().unwrap();
    s.mark_rename_started(&first.job.operation).unwrap();
    s.confirm_rename(&first.job.operation, &file("v2", "B.doc"), true)
        .unwrap();
    s.rename_local("1", "B.doc", "1", "C.doc", false).unwrap();
    let second = s.next_rename().unwrap().unwrap();
    s.mark_rename_started(&second.job.operation).unwrap();
    s.confirm_rename(&second.job.operation, &file("v3", "C.doc"), true)
        .unwrap();
    event(&mut s, file("v2", "B.doc"), "2");
    assert_eq!(s.node("2").unwrap().name, "C.doc");
    assert_eq!(s.node("2").unwrap().version, "v3");
    assert_eq!(s.cursor().unwrap(), "2");
}
#[test]
#[cfg(feature = "native")]
fn native_old_echo_after_new_content_commit_does_not_regress_path_or_version() {
    fn token(sync: u64, max: u64) -> String {
        serde_json::json!({"schema":1,"restore":"server","view":54,"root":"1","file":"2","sync":sync,"max":max,"size":3,"hash":"hash","directory":false}).to_string()
    }
    let (_d, mut s) = fixture();
    let latest = token(20, 20);
    s.install_remote_snapshot(&[root(), file(&latest, "latest.doc")], "2", "server")
        .unwrap();
    event(&mut s, file(&token(18, 18), "old.doc"), "3");
    assert_eq!(s.node("2").unwrap().name, "latest.doc");
    assert_eq!(s.node("2").unwrap().version, latest);
}
