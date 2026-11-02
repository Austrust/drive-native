use drive_core::*;
use std::{
    io::Read,
    time::{Duration, UNIX_EPOCH},
};
fn node(id: &str, dir: bool) -> Node {
    Node {
        id: id.into(),
        parent: (!dir).then(|| "root".into()),
        name: id.into(),
        version: "1".into(),
        directory: dir,
        metadata: Some(NodeMetadata {
            size: if dir { 0 } else { 3 },
            mtime: 7,
            mode: if dir { 0o755 } else { 0o644 },
        }),
    }
}
fn fixture() -> (tempfile::TempDir, Store) {
    let d = tempfile::tempdir().unwrap();
    let mut s = Store::open(d.path()).unwrap();
    s.install_remote_snapshot(&[node("root", true), node("a", false)], "1", "server")
        .unwrap();
    s.ensure_fs_root("root").unwrap();
    s.publish_download(
        &DownloadTicket {
            id: "a".into(),
            version: "1".into(),
        },
        b"old",
        &sha256(b"old"),
    )
    .unwrap();
    (d, s)
}
fn draft(s: &mut Store) -> drafts::DraftHandle {
    let c = s.open_cached("a").unwrap();
    s.begin_draft("a", "1", Some(&c)).unwrap()
}
#[test]
fn native_seconds_bounds_are_explicit_and_fractional_input_is_normalized() {
    assert_eq!(
        timestamps::native_mtime(UNIX_EPOCH + Duration::new(u32::MAX as u64, 999_999_999)).unwrap(),
        u32::MAX
    );
    assert_eq!(
        timestamps::local_time(UNIX_EPOCH + Duration::new(123, 456)).unwrap(),
        (123, 456)
    );
    assert!(timestamps::native_mtime(UNIX_EPOCH - Duration::from_nanos(1)).is_err());
    assert!(
        timestamps::native_mtime(UNIX_EPOCH + Duration::from_secs(u32::MAX as u64 + 1)).is_err()
    );
}
#[test]
fn unchanged_body_timestamp_is_durable_queued_and_copied_to_snapshot() {
    let (d, mut s) = fixture();
    let h = draft(&mut s);
    s.draft_set_mtime(&h, 123456).unwrap();
    let op = s.checkpoint_draft(&h).unwrap().unwrap();
    assert_eq!(s.record("a").unwrap().mtime, 123456);
    assert_eq!(
        s.posix_record_by_inode(s.record("a").unwrap().ino)
            .unwrap()
            .mtime,
        123456
    );
    let mut j = s.upload_job(&op).unwrap();
    assert_eq!(j.mtime, Some(123456));
    assert_eq!(
        j.file.metadata().unwrap().modified().unwrap(),
        UNIX_EPOCH + Duration::from_secs(123456)
    );
    let mut b = Vec::new();
    j.file.read_to_end(&mut b).unwrap();
    assert_eq!(b, b"old");
    drop(j);
    drop(h);
    drop(s);
    let s = Store::open(d.path()).unwrap();
    assert_eq!(s.upload_mtime(&op).unwrap(), Some(123456));
    assert_eq!(s.record("a").unwrap().mtime, 123456);
    assert_eq!(s.status("a").unwrap(), Status::PendingUpload);
}
#[test]
fn atime_only_is_persistent_and_never_queues_or_hydrates() {
    let (d, mut s) = fixture();
    s.release("a").unwrap();
    s.set_local_atime("a", 123, 456).unwrap();
    assert_eq!(s.status("a").unwrap(), Status::OnlineOnly);
    assert!(s.upload_operations("pending").unwrap().is_empty());
    assert!(s.downloads_needed().unwrap().is_empty());
    drop(s);
    let s = Store::open(d.path()).unwrap();
    let r = s.record("a").unwrap();
    assert_eq!((r.atime, r.atime_nsec, r.mtime), (123, 456, 7));
    let p = s.posix_record_by_inode(r.ino).unwrap();
    assert_eq!((p.atime, p.atime_nsec), (123, 456));
}
#[test]
fn older_timestamp_ack_cannot_clear_newer_dirty_draft() {
    let (_d, mut s) = fixture();
    let h = draft(&mut s);
    s.draft_set_mtime(&h, 100).unwrap();
    let old = s.checkpoint_draft(&h).unwrap().unwrap();
    s.mark_upload_started(&old).unwrap();
    s.draft_set_mtime(&h, 200).unwrap();
    s.confirm_upload(&old, "2").unwrap();
    assert_eq!(s.record("a").unwrap().mtime, 200);
    assert!(s.dirty_draft("a").unwrap());
    let new = s.checkpoint_draft(&h).unwrap().unwrap();
    assert_eq!(s.upload_mtime(&new).unwrap(), Some(200));
    assert_eq!(s.upload_job(&new).unwrap().node.version, "2");
}
#[test]
fn failed_intent_insert_rolls_back_operation_and_node_generation_but_retains_draft() {
    let (d, mut s) = fixture();
    let h = draft(&mut s);
    s.draft_set_mtime(&h, 100).unwrap();
    let db = rusqlite::Connection::open(d.path().join("state.sqlite")).unwrap();
    db.execute_batch("CREATE TRIGGER reject_time BEFORE INSERT ON upload_times BEGIN SELECT RAISE(ABORT,'fixture'); END;").unwrap();
    assert!(s.checkpoint_draft(&h).is_err());
    assert!(s.upload_operations("pending").unwrap().is_empty());
    assert_eq!(
        db.query_row("SELECT dirty_generation FROM nodes WHERE id='a'", [], |r| r
            .get::<_, i64>(0))
            .unwrap(),
        0
    );
    assert!(s.dirty_draft("a").unwrap());
    assert_eq!(s.record("a").unwrap().mtime, 100);
    db.execute_batch("DROP TRIGGER reject_time").unwrap();
    let op = s.checkpoint_draft(&h).unwrap().unwrap();
    assert_eq!(s.upload_mtime(&op).unwrap(), Some(100));
}
#[test]
fn schema22_upgrade_does_not_invent_mtime_for_uncertain_requests() {
    let (d, mut s) = fixture();
    let op = s.save_local("a", b"new").unwrap();
    s.mark_upload_started(&op).unwrap();
    s.mark_upload_uncertain(&op).unwrap();
    drop(s);
    let db = rusqlite::Connection::open(d.path().join("state.sqlite")).unwrap();
    db.execute_batch("DROP TABLE upload_times;DROP TABLE local_atimes;PRAGMA user_version=22;")
        .unwrap();
    drop(db);
    let mut s = Store::open(d.path()).unwrap();
    assert_eq!(s.upload_mtime(&op).unwrap(), None);
    assert_eq!(s.upload_job(&op).unwrap().state, "uncertain");
    s.validate_upload_mtime(&op, &node("a", false)).unwrap();
    s.confirm_upload(&op, "2").unwrap();
    let backup = std::fs::read_dir(d.path())
        .unwrap()
        .map(|v| v.unwrap().path())
        .find(|p| {
            p.file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("state-before-v23-")
        })
        .unwrap();
    let db = rusqlite::Connection::open(backup).unwrap();
    assert_eq!(
        db.query_row("PRAGMA user_version", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        22
    );
    assert_eq!(
        db.query_row("PRAGMA integrity_check", [], |r| r.get::<_, String>(0))
            .unwrap(),
        "ok"
    );
}
#[test]
fn mismatching_remote_time_never_confirms_new_request() {
    let (_d, mut s) = fixture();
    let h = draft(&mut s);
    s.draft_set_mtime(&h, 100).unwrap();
    let op = s.checkpoint_draft(&h).unwrap().unwrap();
    s.mark_upload_started(&op).unwrap();
    assert!(s.validate_upload_mtime(&op, &node("a", false)).is_err());
    let mut confirmed = node("a", false);
    confirmed.metadata.as_mut().unwrap().mtime = 100;
    s.validate_upload_mtime(&op, &confirmed).unwrap();
    s.confirm_upload(&op, "2").unwrap();
    assert_eq!(s.record("a").unwrap().mtime, 100);
    assert_eq!(s.status("a").unwrap(), Status::LocalComplete);
}
#[test]
fn unlinked_open_file_times_survive_without_uploading_deleted_path() {
    let (_d, mut s) = fixture();
    let h = draft(&mut s);
    s.unlink_local("root", "a", "a", false).unwrap();
    s.draft_set_mtime(&h, 111).unwrap();
    s.set_local_atime("a", 222, 333).unwrap();
    assert!(s.checkpoint_draft(&h).unwrap().is_none());
    assert!(s.upload_operations("pending").unwrap().is_empty());
    let r = s.record_for_handle("a").unwrap();
    assert!(r.unlinked);
    assert_eq!((r.mtime, r.atime, r.atime_nsec), (111, 222, 333));
}
#[test]
fn local_atime_tracks_stable_created_identity_and_not_folded_echo() {
    let (_d, mut s) = fixture();
    let n = s.create_local_file("root", "new", 0o644).unwrap();
    s.set_local_atime(&n.id, 11, 22).unwrap();
    s.creation_saved(&n.id).unwrap();
    let op = s.upload_operations("pending").unwrap().pop().unwrap();
    s.mark_upload_started(&op).unwrap();
    s.confirm_upload_identity(&op, "2", Some("99")).unwrap();
    assert_eq!(s.local_identity("99").unwrap(), n.id);
    assert_eq!(s.local_atime(&n.id, 0).unwrap(), (11, 22));
}
#[test]
fn replacement_retains_source_time_and_atime_while_old_target_handles_keep_their_own() {
    let (_d, mut s) = fixture();
    let n = s.create_local_file("root", "tmp", 0o644).unwrap();
    s.save_local(&n.id, b"new").unwrap();
    s.set_local_atime(&n.id, 11, 22).unwrap();
    s.set_local_atime("a", 33, 44).unwrap();
    let cached = s.open_cached(&n.id).unwrap();
    let h = s.begin_draft(&n.id, &n.version, Some(&cached)).unwrap();
    s.draft_set_mtime(&h, 123).unwrap();
    s.checkpoint_draft(&h).unwrap();
    s.replace_local("root", "tmp", "root", "a", &n.id, "a")
        .unwrap();
    let job = s.next_replace().unwrap().unwrap().job;
    assert_eq!(job.mtime, Some(123));
    s.mark_replace_started(&job.operation).unwrap();
    let mut confirmed = job.target.clone();
    confirmed.version = "2".into();
    assert!(
        s.confirm_replace_upload(&job.operation, &confirmed, "a")
            .is_err()
    );
    confirmed.metadata.as_mut().unwrap().mtime = 123;
    s.confirm_replace_upload(&job.operation, &confirmed, "a")
        .unwrap();
    let r = s.lookup("root", "a").unwrap();
    assert_eq!((r.mtime, r.atime, r.atime_nsec), (123, 11, 22));
    let old = s.record_for_handle("a").unwrap();
    assert!(old.unlinked);
    assert_eq!((old.atime, old.atime_nsec), (33, 44));
}
#[test]
fn legacy_uncertain_replacement_does_not_gain_a_time_intent_on_restart() {
    let (d, mut s) = fixture();
    let n = s.create_local_file("root", "tmp", 0o644).unwrap();
    s.save_local(&n.id, b"new").unwrap();
    s.replace_local("root", "tmp", "root", "a", &n.id, "a")
        .unwrap();
    let job = s.next_replace().unwrap().unwrap().job;
    s.mark_replace_started(&job.operation).unwrap();
    drop(s);
    let db = rusqlite::Connection::open(d.path().join("state.sqlite")).unwrap();
    db.execute(
        "UPDATE namespace_ops SET body=json_remove(body,'$.mtime') WHERE id=?1",
        [&job.operation],
    )
    .unwrap();
    drop(db);
    let s = Store::open(d.path()).unwrap();
    let queued = s.next_replace().unwrap().unwrap();
    assert_eq!(queued.state, "uncertain");
    assert_eq!(queued.job.mtime, None);
}
