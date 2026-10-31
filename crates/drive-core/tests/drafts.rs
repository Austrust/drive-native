use drive_core::*;
use std::io::Read;
fn setup() -> (tempfile::TempDir, Store) {
    let d = tempfile::tempdir().unwrap();
    let mut s = Store::open(d.path()).unwrap();
    s.install_remote_snapshot(
        &[node("root", "1", true), node("a", "1", false)],
        "1",
        "server",
    )
    .unwrap();
    s.publish_download(
        &DownloadTicket {
            id: "a".into(),
            version: "1".into(),
        },
        b"abcdef",
        &sha256(b"abcdef"),
    )
    .unwrap();
    (d, s)
}
fn node(id: &str, v: &str, dir: bool) -> Node {
    Node {
        id: id.into(),
        parent: if dir { None } else { Some("root".into()) },
        name: id.into(),
        version: v.into(),
        directory: dir,
        metadata: Some(NodeMetadata {
            size: if dir { 0 } else { 6 },
            mtime: 0,
            mode: 0o644,
        }),
    }
}
fn draft(s: &mut Store) -> drafts::DraftHandle {
    let mut read = s.open_cached("a").unwrap();
    s.begin_draft("a", "1", Some(&mut read)).unwrap()
}
#[test]
fn a_new_handle_joins_only_the_matching_clean_or_dirty_draft() {
    let (_dir, mut s) = setup();
    assert!(s.open_draft_for_version("a", "1").unwrap().is_none());
    let working = draft(&mut s);
    assert!(!s.dirty_draft("a").unwrap());
    let reader = s.open_draft_for_version("a", "1").unwrap().unwrap();
    assert_eq!(reader.token, working.token);
    assert!(
        s.open_draft_for_version("a", "different")
            .unwrap()
            .is_none()
    );
    s.draft_write(&working, 0, b"new", false).unwrap();
    let mut bytes = [0; 6];
    assert_eq!(reader.read_at(&mut bytes, 0).unwrap(), 6);
    assert_eq!(&bytes, b"newdef");
    assert_eq!(
        s.open_draft_for_version("a", "1").unwrap().unwrap().token,
        working.token
    );
}
#[test]
fn random_writes_and_truncate_survive_reopen_and_checkpoint() {
    let (d, mut s) = setup();
    let h = draft(&mut s);
    s.draft_write(&h, 2, b"XY", false).unwrap();
    s.draft_truncate(&h, 5).unwrap();
    assert_eq!(s.status("a").unwrap(), Status::PendingUpload);
    assert!(s.release("a").is_err());
    drop(h);
    drop(s);
    let mut s = Store::open(d.path()).unwrap();
    let h = s.open_dirty_draft("a").unwrap().unwrap();
    let mut b = [0; 9];
    assert_eq!(h.read_at(&mut b, 0).unwrap(), 5);
    assert_eq!(&b[..5], b"abXYe");
    assert_eq!(s.checkpoint_recovered_drafts().unwrap(), 1);
    let mut bytes = Vec::new();
    s.open_cached("a").unwrap().read_to_end(&mut bytes).unwrap();
    assert_eq!(bytes, b"abXYe");
}
#[test]
fn unsnapshotted_edit_prevents_uploading_older_snapshot() {
    let (_dir, mut s) = setup();
    let h = draft(&mut s);
    s.draft_write(&h, 0, b"X", false).unwrap();
    let op = s.checkpoint_draft(&h).unwrap().unwrap();
    s.draft_write(&h, 1, b"Y", false).unwrap();
    assert!(s.mark_upload_started(&op).is_err());
    let next = s.checkpoint_draft(&h).unwrap().unwrap();
    s.mark_upload_started(&next).unwrap();
}
#[test]
fn remote_update_during_unsaved_draft_is_conflict_and_cannot_replace_it() {
    let (_dir, mut s) = setup();
    let h = draft(&mut s);
    s.draft_write(&h, 0, b"local", false).unwrap();
    s.install_remote_snapshot(
        &[node("root", "1", true), node("a", "2", false)],
        "2",
        "server",
    )
    .unwrap();
    assert_eq!(s.status("a").unwrap(), Status::Conflict);
    assert!(s.downloads_needed().unwrap().is_empty());
    assert!(
        s.publish_download(
            &DownloadTicket {
                id: "a".into(),
                version: "2".into()
            },
            b"remote",
            &sha256(b"remote")
        )
        .is_err()
    );
    let mut b = [0; 6];
    h.read_at(&mut b, 0).unwrap();
    assert_eq!(&b, b"localf");
}
#[test]
fn confirming_snapshot_preserves_later_draft_and_advances_its_base() {
    let (_dir, mut s) = setup();
    let h = draft(&mut s);
    s.draft_write(&h, 0, b"X", false).unwrap();
    let op = s.checkpoint_draft(&h).unwrap().unwrap();
    s.mark_upload_started(&op).unwrap();
    s.draft_write(&h, 1, b"Y", false).unwrap();
    s.confirm_upload(&op, "2").unwrap();
    assert_eq!(s.status("a").unwrap(), Status::PendingUpload);
    assert!(!s.draft_conflicts("a").unwrap());
    let next = s.checkpoint_draft(&h).unwrap().unwrap();
    s.mark_upload_started(&next).unwrap();
    s.confirm_upload(&next, "3").unwrap();
    assert_eq!(s.status("a").unwrap(), Status::LocalComplete);
}
#[test]
fn stale_draft_handle_cannot_write_into_newer_working_copy() {
    let (_dir, mut s) = setup();
    let old = draft(&mut s);
    s.install_remote_snapshot(
        &[node("root", "1", true), node("a", "2", false)],
        "2",
        "server",
    )
    .unwrap();
    s.publish_download(
        &DownloadTicket {
            id: "a".into(),
            version: "2".into(),
        },
        b"remote",
        &sha256(b"remote"),
    )
    .unwrap();
    let mut source = s.open_cached("a").unwrap();
    let new = s.begin_draft("a", "2", Some(&mut source)).unwrap();
    assert!(s.draft_write(&old, 0, b"bad", false).is_err());
    let mut b = [0; 6];
    new.read_at(&mut b, 0).unwrap();
    assert_eq!(&b, b"remote");
}

#[test]
fn uploaded_draft_handle_still_protects_cache_from_eviction() {
    let (_dir, mut s) = setup();
    let h = draft(&mut s);
    s.draft_write(&h, 0, b"X", false).unwrap();
    let op = s.checkpoint_draft(&h).unwrap().unwrap();
    s.mark_upload_started(&op).unwrap();
    s.confirm_upload(&op, "2").unwrap();
    assert_eq!(s.status("a").unwrap(), Status::LocalComplete);
    assert!(s.release("a").is_err());
    drop(h);
    s.release("a").unwrap();
    assert_eq!(s.status("a").unwrap(), Status::OnlineOnly);
}
#[test]
fn checkpointed_edit_during_upload_advances_base_without_losing_content() {
    let (_dir, mut s) = setup();
    let h = draft(&mut s);
    s.draft_write(&h, 0, b"X", false).unwrap();
    let old = s.checkpoint_draft(&h).unwrap().unwrap();
    s.mark_upload_started(&old).unwrap();
    s.draft_write(&h, 1, b"Y", false).unwrap();
    let next = s.checkpoint_draft(&h).unwrap().unwrap();
    s.confirm_upload(&old, "2").unwrap();
    assert_eq!(s.status("a").unwrap(), Status::PendingUpload);
    assert!(!s.draft_conflicts("a").unwrap());
    assert_eq!(s.upload_job(&next).unwrap().node.version, "2");
    s.mark_upload_started(&next).unwrap();
    s.confirm_upload(&next, "3").unwrap();
    let mut bytes = Vec::new();
    s.open_cached("a").unwrap().read_to_end(&mut bytes).unwrap();
    assert_eq!(bytes, b"XYcdef");
}
