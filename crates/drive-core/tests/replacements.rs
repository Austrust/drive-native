use drive_core::*;
fn node(id: &str, name: &str, version: &str) -> Node {
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
fn root() -> Node {
    Node {
        id: "root".into(),
        parent: None,
        name: "root".into(),
        version: "root1".into(),
        directory: true,
        metadata: Some(NodeMetadata {
            size: 0,
            mtime: 1,
            mode: 0o755,
        }),
    }
}
fn fixture() -> (tempfile::TempDir, Store) {
    let d = tempfile::tempdir().unwrap();
    let mut s = Store::open(d.path()).unwrap();
    s.install_remote_snapshot(
        &[root(), node("10", "tmp", "s1"), node("20", "A.doc", "t1")],
        "1",
        "server",
    )
    .unwrap();
    s.ensure_fs_root("root").unwrap();
    for (id, version, bytes) in [("10", "s1", b"new"), ("20", "t1", b"old")] {
        s.publish_download(
            &DownloadTicket {
                id: id.into(),
                version: version.into(),
            },
            bytes,
            &sha256(bytes),
        )
        .unwrap();
    }
    (d, s)
}
fn replace(s: &mut Store) {
    s.replace_local("root", "tmp", "root", "A.doc", "10", "20")
        .unwrap();
}
fn uploaded(s: &mut Store) -> String {
    let q = s.next_replace().unwrap().unwrap();
    s.mark_replace_started(&q.job.operation).unwrap();
    s.confirm_replace_upload(&q.job.operation, &node("20", "A.doc", "t2"), "20")
        .unwrap();
    q.job.operation
}
fn event(s: &mut Store, node: Node, to: &str) {
    s.receive(&Batch {
        from: s.cursor().unwrap(),
        to: to.into(),
        events: vec![Event {
            id: to.into(),
            change: Change::Upsert(node),
        }],
    })
    .unwrap();
    s.drain().unwrap();
}
#[test]
fn local_replace_is_one_namespace_change_and_preserves_both_open_versions() {
    let (_d, mut s) = fixture();
    let src = s.open_cached("10").unwrap();
    let dst = s.open_cached("20").unwrap();
    let source_ino = s.record("10").unwrap().ino;
    let target_ino = s.record("20").unwrap().ino;
    replace(&mut s);
    assert!(s.lookup("root", "tmp").is_err());
    assert_eq!(s.lookup("root", "A.doc").unwrap().ino, source_ino);
    assert!(s.record_by_inode(target_ino).unwrap().unlinked);
    let mut b = [0; 3];
    src.read_at(&mut b, 0).unwrap();
    assert_eq!(&b, b"new");
    dst.read_at(&mut b, 0).unwrap();
    assert_eq!(&b, b"old");
    assert!(s.next_remove().unwrap().is_none());
    let draft = s.begin_draft("20", "t1", Some(&dst)).unwrap();
    s.draft_write(&draft, 0, b"orphan", false).unwrap();
    assert!(s.checkpoint_draft(&draft).unwrap().is_none());
    let op = uploaded(&mut s);
    let q = s.next_replace().unwrap().unwrap();
    assert_eq!(q.state, "cleanup_pending");
    assert_eq!(s.local_identity("20").unwrap(), "10");
    let shadow = s.local_identity("10").unwrap();
    assert_ne!(shadow, "10");
    event(&mut s, node("10", "tmp", "s1"), "2");
    assert!(s.lookup("root", "tmp").is_err());
    assert_eq!(s.node("10").unwrap().version, "t2");
    let cleanup = s.replacement_cleanup(&q.job).unwrap();
    assert_eq!(cleanup.node.id, shadow);
    s.mark_replace_cleanup_started(&op).unwrap();
    s.confirm_replace_cleanup(&op, &shadow).unwrap();
    assert!(s.next_replace().unwrap().is_none());
    assert_eq!(s.status("10").unwrap(), Status::LocalComplete);
    assert_eq!(
        s.equivalent_content_version("10", src.version()).unwrap(),
        "t2"
    );
}
#[test]
fn upload_and_cleanup_unknown_states_survive_restart_separately() {
    let (d, mut s) = fixture();
    replace(&mut s);
    let q = s.next_replace().unwrap().unwrap();
    s.mark_replace_started(&q.job.operation).unwrap();
    drop(s);
    let mut s = Store::open(d.path()).unwrap();
    assert_eq!(s.next_replace().unwrap().unwrap().state, "uncertain");
    assert!(s.mark_replace_started(&q.job.operation).is_err());
    s.confirm_replace_upload(&q.job.operation, &node("20", "A.doc", "t2"), "20")
        .unwrap();
    s.mark_replace_cleanup_started(&q.job.operation).unwrap();
    drop(s);
    let mut s = Store::open(d.path()).unwrap();
    let q = s.next_replace().unwrap().unwrap();
    assert_eq!(q.state, "cleanup_uncertain");
    assert!(s.mark_replace_cleanup_started(&q.job.operation).is_err());
    s.confirm_replace_cleanup(&q.job.operation, q.job.cleanup_node.as_deref().unwrap())
        .unwrap();
}
#[test]
fn unsent_source_never_creates_remote_temp_and_target_is_not_deleted() {
    let (_d, mut s) = fixture();
    let tmp = s.create_local_file("root", "newtmp", 0o600).unwrap();
    s.save_local(&tmp.id, b"replacement").unwrap();
    s.replace_local("root", "newtmp", "root", "A.doc", &tmp.id, "20")
        .unwrap();
    let q = s.next_replace().unwrap().unwrap();
    assert!(q.job.source_remote.is_none());
    s.mark_replace_started(&q.job.operation).unwrap();
    s.confirm_replace_upload(&q.job.operation, &node("20", "A.doc", "t2"), "20")
        .unwrap();
    assert!(s.next_namespace_kind().unwrap().is_none());
    assert_eq!(s.local_identity("20").unwrap(), tmp.id);
    assert!(s.next_remove().unwrap().is_none());
}
#[test]
fn two_unsent_files_become_one_server_creation_with_stable_source_inode() {
    let (_d, mut s) = fixture();
    let src = s.create_local_file("root", "x", 0o644).unwrap();
    let target = s.create_local_file("root", "y", 0o644).unwrap();
    s.save_local(&src.id, b"body").unwrap();
    let ino = s.record(&src.id).unwrap().ino;
    s.replace_local("root", "x", "root", "y", &src.id, &target.id)
        .unwrap();
    let q = s.next_replace().unwrap().unwrap();
    assert!(q.job.target_remote.is_none());
    s.mark_replace_started(&q.job.operation).unwrap();
    let mut confirmed = target.clone();
    confirmed.version = "created-v1".into();
    s.confirm_replace_upload(&q.job.operation, &confirmed, "999")
        .unwrap();
    assert_eq!(s.local_identity("999").unwrap(), src.id);
    assert_eq!(s.lookup("root", "y").unwrap().ino, ino);
}
#[test]
fn inflight_source_and_target_uploads_rebase_replacement_before_it_starts() {
    let (_d, mut s) = fixture();
    let src = s.save_local("10", b"newer source").unwrap();
    s.mark_upload_started(&src).unwrap();
    let dst = s.save_local("20", b"previous save").unwrap();
    s.mark_upload_started(&dst).unwrap();
    replace(&mut s);
    let q = s.next_replace().unwrap().unwrap();
    assert!(s.mark_replace_started(&q.job.operation).is_err());
    s.confirm_upload(&src, "s2").unwrap();
    s.confirm_upload(&dst, "t2").unwrap();
    let q = s.next_replace().unwrap().unwrap();
    assert_eq!(q.job.source.version, "s2");
    assert_eq!(q.job.target.version, "t2");
    s.mark_replace_started(&q.job.operation).unwrap();
}
#[test]
fn later_edits_and_moves_use_new_binding_and_old_source_cleanup_cannot_remove_destination() {
    let (_d, mut s) = fixture();
    let held = s.open_cached("10").unwrap();
    replace(&mut s);
    let op = s.save_local("10", b"after replace").unwrap();
    s.rename_local("root", "A.doc", "root", "B.doc", false)
        .unwrap();
    let id = uploaded(&mut s);
    let q = s.next_replace().unwrap().unwrap();
    let cleanup = s.replacement_cleanup(&q.job).unwrap();
    assert_eq!(cleanup.path, "tmp");
    assert_eq!(cleanup.node.version, "s1");
    s.mark_replace_cleanup_started(&id).unwrap();
    s.confirm_replace_cleanup(&id, &cleanup.node.id).unwrap();
    let q = s.next_rename().unwrap().unwrap();
    assert_eq!(q.job.node.version, "t2");
    assert_eq!(q.job.source, "A.doc");
    assert_eq!(s.upload_job(&op).unwrap().node.version, "t2");
    assert_eq!(
        s.equivalent_content_version("10", held.version()).unwrap(),
        "t2"
    );
}
#[test]
fn later_save_and_gc_preserve_snapshot_of_replacement_operation() {
    let (_d, mut s) = fixture();
    s.save_local("10", b"replacement snapshot").unwrap();
    replace(&mut s);
    let q = s.next_replace().unwrap().unwrap();
    s.save_local("10", b"next save").unwrap();
    s.collect_garbage().unwrap();
    let mut bytes = Vec::new();
    std::io::Read::read_to_end(&mut s.replacement_file(&q.job).unwrap(), &mut bytes).unwrap();
    assert_eq!(bytes, b"replacement snapshot");
}
#[test]
fn newer_target_event_prevents_confirmation_and_keeps_original_bindings() {
    let (_d, mut s) = fixture();
    replace(&mut s);
    let q = s.next_replace().unwrap().unwrap();
    s.mark_replace_started(&q.job.operation).unwrap();
    event(&mut s, node("20", "A.doc", "t3"), "3");
    assert!(
        s.confirm_replace_upload(&q.job.operation, &node("20", "A.doc", "t2"), "20")
            .is_err()
    );
    assert_eq!(s.local_identity("20").unwrap(), "20");
    assert_eq!(s.local_identity("10").unwrap(), "10");
    assert_eq!(s.lookup("root", "A.doc").unwrap().node.id, "10");
}
#[test]
fn chained_offline_replacements_keep_sequence_and_rebase_second_destination() {
    let (_d, mut s) = fixture();
    replace(&mut s);
    let tmp = s.create_local_file("root", "second", 0o644).unwrap();
    s.save_local(&tmp.id, b"second save").unwrap();
    s.replace_local("root", "second", "root", "A.doc", &tmp.id, "10")
        .unwrap();
    let id = uploaded(&mut s);
    let q = s.next_replace().unwrap().unwrap();
    s.mark_replace_cleanup_started(&id).unwrap();
    s.confirm_replace_cleanup(&id, q.job.cleanup_node.as_deref().unwrap())
        .unwrap();
    let second = s.next_replace().unwrap().unwrap();
    assert_eq!(second.job.target.version, "t2");
    assert_eq!(second.job.target_remote.as_deref(), Some("20"));
    assert_eq!(s.lookup("root", "A.doc").unwrap().node.id, tmp.id);
    s.mark_replace_started(&second.job.operation).unwrap();
    let mut receipt = second.job.target.clone();
    receipt.version = "t3".into();
    s.confirm_replace_upload(&second.job.operation, &receipt, "20")
        .unwrap();
    assert!(s.next_replace().unwrap().is_none());
    assert_eq!(s.local_identity("20").unwrap(), tmp.id);
    assert!(s.record_for_handle("20").unwrap().unlinked); // old target orphan must not be folded as a creation echo
}

#[test]
fn empty_temp_creation_ack_does_not_make_preexisting_empty_handle_stale() {
    let (_d, mut s) = fixture();
    let tmp = s.create_local_file("root", "writer.tmp", 0o644).unwrap();
    let held = s.open_cached(&tmp.id).unwrap();
    let clean = s.begin_draft(&tmp.id, &tmp.version, Some(&held)).unwrap();
    drop(clean);
    s.creation_saved(&tmp.id).unwrap();
    let op = s
        .upload_operations("pending")
        .unwrap()
        .into_iter()
        .find(|op| s.upload_job(op).unwrap().node.id == tmp.id)
        .unwrap();
    s.mark_upload_started(&op).unwrap();
    s.confirm_upload_identity(&op, "created-empty", Some("998"))
        .unwrap();
    // Writer closes an initially empty temp file and opens its old cache handle
    // before the upload receipt arrives. Its first write happens after the ACK.
    let draft = s.begin_draft(&tmp.id, held.version(), Some(&held)).unwrap();
    s.draft_write(&draft, 0, b"Writer final document", false)
        .unwrap();
    s.checkpoint_draft(&draft).unwrap();
    assert_eq!(s.status(&tmp.id).unwrap(), Status::PendingUpload);
    s.replace_local("root", "writer.tmp", "root", "A.doc", &tmp.id, "20")
        .unwrap();
    let q = s.next_replace().unwrap().unwrap();
    assert_eq!(q.state, "pending");
    assert_eq!(q.job.source.version, "created-empty");
    assert_eq!(q.job.source_remote.as_deref(), Some("998"));
    s.mark_replace_started(&q.job.operation).unwrap();
}

#[test]
fn acknowledged_changed_content_does_not_rebase_an_old_cached_reader() {
    let (_d, mut s) = fixture();
    let held = s.open_cached("10").unwrap();
    let op = s.save_local("10", b"other local save").unwrap();
    s.mark_upload_started(&op).unwrap();
    s.confirm_upload(&op, "s2").unwrap();
    let draft = s.begin_draft("10", held.version(), Some(&held)).unwrap();
    s.draft_write(&draft, 0, b"stale writer", false).unwrap();
    s.checkpoint_draft(&draft).unwrap();
    assert_eq!(s.status("10").unwrap(), Status::Conflict);
}
