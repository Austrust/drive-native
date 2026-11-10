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

// ---- issue #60: orphan draft lifecycle ------------------------------------

/// The 763,913,234 B aborted-zip leftover has zero `operations` and zero
/// `namespace_ops` rows and its node is deleted, so no code path can ever
/// consume it. Past the orphan age window its body must move to a recoverable
/// quarantine instead of staying in `drafts/` forever (and must not be
/// hard-deleted).
#[test]
fn aged_unreferenced_draft_is_quarantined_and_recoverable() {
    let (dir, mut s) = setup();
    let mut source = s.open_cached("a").unwrap();
    let working = s.begin_draft("a", "1", Some(&mut source)).unwrap();
    drop(source);
    s.draft_write(&working, 0, b"partial", false).unwrap();
    let token = working.token.clone();
    drop(working);
    // The server drops the object: the node becomes a tombstone but no local
    // operation or namespace intent references the working copy.
    s.install_remote_snapshot(&[node("root", "2", true)], "2", "server")
        .unwrap();
    let body = dir.path().join("drafts").join(&token);
    assert!(body.is_file(), "draft body must exist before GC");
    // Fresh bodies stay: the default orphan age window is 30 days.
    s.collect_garbage().unwrap();
    assert!(
        !dir.path().join("drafts-quarantine").join(&token).exists(),
        "a fresh draft must not be quarantined"
    );
    // Backdate the body past the orphan age window.
    std::fs::OpenOptions::new()
        .write(true)
        .open(&body)
        .unwrap()
        .set_modified(
            std::time::SystemTime::now() - std::time::Duration::from_secs(31 * 24 * 3600),
        )
        .unwrap();
    s.collect_garbage().unwrap();
    let quarantined = dir.path().join("drafts-quarantine").join(&token);
    assert!(
        quarantined.is_file(),
        "orphan draft body must be quarantined, not deleted"
    );
    assert!(!body.exists(), "draft body must leave drafts/");
    let restored = std::fs::read(quarantined).unwrap();
    assert_eq!(restored, b"partial");
}

/// The 805,129,740 B archive: upload committed at 21:07:16, deleted locally,
/// remote `remove` stuck. The working copy must become reclaimable once the
/// remove reaches a terminal state.
#[test]
fn committed_upload_deleted_locally_releases_draft_after_remove_commits() {
    let (dir, mut s) = setup();
    let mut source = s.open_cached("a").unwrap();
    let working = s.begin_draft("a", "1", Some(&mut source)).unwrap();
    drop(source);
    s.draft_write(&working, 0, b"X", false).unwrap();
    let op = s.checkpoint_draft(&working).unwrap().unwrap();
    s.mark_upload_started(&op).unwrap();
    s.confirm_upload(&op, "2").unwrap();
    // A later local edit keeps the working copy dirty after the commit.
    s.draft_write(&working, 1, b"Y", false).unwrap();
    let token = working.token.clone();
    drop(working);
    let body = dir.path().join("drafts").join(&token);
    assert!(body.is_file());
    s.unlink_local("root", "a", "a", false).unwrap();
    let remove = s.next_remove().unwrap().expect("remove intent");
    s.mark_remove_started(&remove.job.operation).unwrap();
    s.confirm_remove(&remove.job.operation, "a").unwrap();
    s.collect_garbage().unwrap();
    assert!(
        !body.exists(),
        "a confirmed remove must not retain the deleted node's working copy"
    );
}


/// Recovery path for the quarantine: a restored body returns to `drafts/` and
/// stays openable, so quarantine is a recoverable holding area, not a delete.
#[test]
fn quarantined_orphan_draft_can_be_restored() {
    let (dir, mut s) = setup();
    let mut source = s.open_cached("a").unwrap();
    let working = s.begin_draft("a", "1", Some(&mut source)).unwrap();
    drop(source);
    s.draft_write(&working, 0, b"partial", false).unwrap();
    let token = working.token.clone();
    drop(working);
    s.install_remote_snapshot(&[node("root", "2", true)], "2", "server")
        .unwrap();
    let body = dir.path().join("drafts").join(&token);
    std::fs::OpenOptions::new()
        .write(true)
        .open(&body)
        .unwrap()
        .set_modified(
            std::time::SystemTime::now() - std::time::Duration::from_secs(31 * 24 * 3600),
        )
        .unwrap();
    s.collect_garbage().unwrap();
    assert!(dir
        .path()
        .join("drafts-quarantine")
        .join(&token)
        .is_file());
    assert_eq!(s.restore_quarantined_drafts().unwrap(), 1);
    assert!(body.is_file());
    assert_eq!(std::fs::read(&body).unwrap(), b"partial");
    assert!(s.open_dirty_draft("a").unwrap().is_some());
}

/// A remove that stays unresolved is terminal for automatic reconciliation, so
/// the user needs an explicit abandon action. It must retire the local
/// tombstone without ever replaying a remote delete.
#[test]
fn unresolved_remove_can_be_abandoned_locally_without_remote_delete() {
    let (dir, mut s) = setup();
    let mut source = s.open_cached("a").unwrap();
    let working = s.begin_draft("a", "1", Some(&mut source)).unwrap();
    drop(source);
    s.draft_write(&working, 0, b"leftover", false).unwrap();
    let token = working.token.clone();
    drop(working);
    s.unlink_local("root", "a", "a", false).unwrap();
    assert_eq!(s.pending_namespace_operations().unwrap(), 1);
    let remove = s.next_remove().unwrap().expect("remove intent");
    let remove_id = remove.job.operation.clone();
    assert!(s.abandon_unresolved_remove("a").unwrap());
    assert_eq!(s.pending_namespace_operations().unwrap(), 0);
    assert!(s.locally_unlinked("a").unwrap());
    assert!(s.next_remove().unwrap().is_none(), "no remote delete re-armed");
    s.collect_garbage().unwrap();
    assert!(
        !dir.path().join("drafts").join(&token).exists(),
        "abandoning the local tombstone must release the draft body"
    );
    // The abandoned intent is terminal: it can never be resubmitted.
    assert!(s.mark_remove_started(&remove_id).is_err());
}

/// Issue #60: a working copy created for a node that is already a tombstone
/// (delete an open file, then keep writing) has no live namespace object. It
/// must be detectable at creation time, not only as unexplained draft bytes.
#[test]
fn draft_creation_on_tombstone_node_is_reported() {
    let (_dir, mut s) = setup();
    s.unlink_local("root", "a", "a", false).unwrap();
    let before = s.draft_write_stats().unreferenced_draft_begins;
    let working = s.begin_draft("a", "1", None).unwrap();
    assert_eq!(
        s.draft_write_stats().unreferenced_draft_begins,
        before + 1,
        "a draft created on a tombstone must be counted at creation"
    );
    drop(working);
}
