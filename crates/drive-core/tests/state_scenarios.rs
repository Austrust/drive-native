use drive_core::*;
use std::io::Read;

fn node(id: &str, parent: Option<&str>, version: &str, directory: bool) -> Node {
    Node {
        id: id.into(),
        parent: parent.map(str::to_owned),
        name: format!("{id}.doc"),
        version: version.into(),
        directory,
        metadata: None,
    }
}
fn batch(from: &str, to: &str, changes: Vec<Change>) -> Batch {
    Batch {
        from: from.into(),
        to: to.into(),
        events: changes
            .into_iter()
            .enumerate()
            .map(|(i, change)| Event {
                id: format!("{to}-{i}"),
                change,
            })
            .collect(),
    }
}
fn update(s: &mut Store, from: &str, to: &str, n: Node) {
    s.receive(&batch(from, to, vec![Change::Upsert(n)]))
        .unwrap();
    s.drain().unwrap();
}
fn cache(s: &mut Store, id: &str, version: &str, b: &[u8]) {
    s.publish_download(
        &DownloadTicket {
            id: id.into(),
            version: version.into(),
        },
        b,
        &sha256(b),
    )
    .unwrap();
}
fn read(s: &Store, id: &str) -> Vec<u8> {
    let mut v = vec![];
    s.open_cached(id).unwrap().read_to_end(&mut v).unwrap();
    v
}

#[test]
fn unpinned_doc_is_refreshed_without_reopening_and_available_after_restart() {
    let d = tempfile::tempdir().unwrap();
    let mut s = Store::open(d.path()).unwrap();
    update(&mut s, "", "c1", node("A", None, "v1", false));
    cache(&mut s, "A", "v1", b"old");
    assert!(!s.is_pinned("A").unwrap());
    update(&mut s, "c1", "c2", node("A", None, "v2", false));
    assert_eq!(s.status("A").unwrap(), Status::Updating);
    let jobs = s.downloads_needed().unwrap();
    assert_eq!(jobs.len(), 1);
    s.publish_download(&jobs[0], b"new", &sha256(b"new"))
        .unwrap();
    drop(s);
    let s = Store::open(d.path()).unwrap();
    assert_eq!(read(&s, "A"), b"new");
    assert_eq!(s.status("A").unwrap(), Status::LocalComplete);
}
#[test]
fn received_events_survive_restart_and_duplicates_are_idempotent() {
    let d = tempfile::tempdir().unwrap();
    let mut s = Store::open(d.path()).unwrap();
    let b = batch("", "c1", vec![Change::Upsert(node("A", None, "v1", false))]);
    s.receive(&b).unwrap();
    assert_eq!(s.cursor().unwrap(), "");
    drop(s);
    let mut s = Store::open(d.path()).unwrap();
    s.receive(&b).unwrap();
    s.drain().unwrap();
    assert_eq!(s.cursor().unwrap(), "c1");
    s.receive(&b).unwrap();
    assert!(!s.apply_next().unwrap());
}
#[test]
fn event_gaps_and_reused_identity_fail_without_advancing_cursor() {
    let d = tempfile::tempdir().unwrap();
    let mut s = Store::open(d.path()).unwrap();
    let b = batch("", "c1", vec![Change::Upsert(node("A", None, "v1", false))]);
    s.receive(&b).unwrap();
    s.drain().unwrap();
    assert!(s.receive(&batch("missing", "c3", vec![])).is_err());
    let mut reused = batch("c1", "c2", vec![Change::Delete { id: "A".into() }]);
    reused.events[0].id = b.events[0].id.clone();
    s.receive(&reused).unwrap();
    assert!(s.drain().is_err());
    assert_eq!(s.cursor().unwrap(), "c1");
    assert_eq!(s.status("A").unwrap(), Status::OnlineOnly);
}
#[test]
fn failed_hash_or_stale_download_never_replaces_complete_version() {
    let d = tempfile::tempdir().unwrap();
    let mut s = Store::open(d.path()).unwrap();
    update(&mut s, "", "c1", node("A", None, "v1", false));
    cache(&mut s, "A", "v1", b"old");
    update(&mut s, "c1", "c2", node("A", None, "v2", false));
    let ticket = s.downloads_needed().unwrap().remove(0);
    assert!(
        s.publish_download(&ticket, b"bad", &sha256(b"new"))
            .is_err()
    );
    assert_eq!(read(&s, "A"), b"old");
    update(&mut s, "c2", "c3", node("A", None, "v3", false));
    assert!(
        s.publish_download(&ticket, b"new", &sha256(b"new"))
            .is_err()
    );
    assert_eq!(read(&s, "A"), b"old");
}
#[test]
fn pin_inherits_new_children_and_unpin_does_not_evict() {
    let d = tempfile::tempdir().unwrap();
    let mut s = Store::open(d.path()).unwrap();
    update(&mut s, "", "c1", node("folder", None, "v1", true));
    s.pin("folder", true).unwrap();
    update(&mut s, "c1", "c2", node("A", Some("folder"), "v1", false));
    assert_eq!(s.downloads_needed().unwrap().len(), 1);
    assert_eq!(s.status("A").unwrap(), Status::Pinning);
    cache(&mut s, "A", "v1", b"cached");
    assert!(s.release("A").is_err());
    s.pin("folder", false).unwrap();
    assert_eq!(read(&s, "A"), b"cached");
    s.release("A").unwrap();
    assert_eq!(s.status("A").unwrap(), Status::OnlineOnly);
    assert_eq!(s.pending_operations().unwrap(), 0);
}
#[test]
fn open_handle_keeps_old_version_and_prevents_eviction() {
    let d = tempfile::tempdir().unwrap();
    let mut s = Store::open(d.path()).unwrap();
    update(&mut s, "", "c1", node("A", None, "v1", false));
    cache(&mut s, "A", "v1", b"old");
    let mut handle = s.open_cached("A").unwrap();
    update(&mut s, "c1", "c2", node("A", None, "v2", false));
    cache(&mut s, "A", "v2", b"new");
    assert!(s.release("A").is_err());
    let mut old = vec![];
    handle.read_to_end(&mut old).unwrap();
    assert_eq!(old, b"old");
    assert_eq!(read(&s, "A"), b"new");
    drop(handle);
    s.release("A").unwrap();
}
#[test]
fn dirty_data_survives_remote_conflict_and_restart() {
    let d = tempfile::tempdir().unwrap();
    let mut s = Store::open(d.path()).unwrap();
    update(&mut s, "", "c1", node("A", None, "v1", false));
    cache(&mut s, "A", "v1", b"base");
    let op = s.save_local("A", b"local edits").unwrap();
    update(&mut s, "c1", "c2", node("A", None, "v2", false));
    assert_eq!(s.status("A").unwrap(), Status::Conflict);
    assert!(s.release("A").is_err());
    assert!(s.mark_upload_started(&op).is_err());
    drop(s);
    let s = Store::open(d.path()).unwrap();
    assert_eq!(read(&s, "A"), b"local edits");
}
#[test]
fn uncertain_upload_cannot_be_retried_or_overtaken() {
    let d = tempfile::tempdir().unwrap();
    let mut s = Store::open(d.path()).unwrap();
    update(&mut s, "", "c1", node("A", None, "v1", false));
    let first = s.save_local("A", b"one").unwrap();
    s.mark_upload_started(&first).unwrap();
    s.mark_upload_uncertain(&first).unwrap();
    assert!(s.mark_upload_started(&first).is_err());
    let second = s.save_local("A", b"two").unwrap();
    assert!(s.mark_upload_started(&second).is_err());
    s.confirm_upload(&first, "v2").unwrap();
    assert_eq!(read(&s, "A"), b"two");
    assert_eq!(s.status("A").unwrap(), Status::PendingUpload);
    s.mark_upload_started(&second).unwrap();
    s.confirm_upload(&second, "v3").unwrap();
    assert_eq!(s.status("A").unwrap(), Status::LocalComplete);
    assert_eq!(s.pending_operations().unwrap(), 0);
}
#[test]
fn rename_does_not_change_identity_and_delayed_download_cannot_target_replacement() {
    let d = tempfile::tempdir().unwrap();
    let mut s = Store::open(d.path()).unwrap();
    update(&mut s, "", "c1", node("A", None, "v1", false));
    cache(&mut s, "A", "v1", b"old");
    let mut moved = node("A", None, "v1", false);
    moved.name = "renamed.doc".into();
    update(&mut s, "c1", "c2", moved);
    assert_eq!(read(&s, "A"), b"old");
    s.receive(&batch(
        "c2",
        "c3",
        vec![
            Change::Delete { id: "A".into() },
            Change::Upsert(node("replacement", None, "v1", false)),
        ],
    ))
    .unwrap();
    s.drain().unwrap();
    assert!(
        s.publish_download(
            &DownloadTicket {
                id: "A".into(),
                version: "v1".into()
            },
            b"late",
            &sha256(b"late")
        )
        .is_err()
    );
    assert!(matches!(s.open_cached("replacement"), Err(Error::Offline)));
}
#[test]
fn directory_delete_preserves_dirty_descendants_as_conflicts() {
    let d = tempfile::tempdir().unwrap();
    let mut s = Store::open(d.path()).unwrap();
    update(&mut s, "", "c1", node("folder", None, "v1", true));
    update(&mut s, "c1", "c2", node("A", Some("folder"), "v1", false));
    s.save_local("A", b"draft").unwrap();
    s.receive(&batch(
        "c2",
        "c3",
        vec![Change::Delete {
            id: "folder".into(),
        }],
    ))
    .unwrap();
    s.drain().unwrap();
    assert_eq!(s.status("A").unwrap(), Status::Conflict);
    assert_eq!(read(&s, "A"), b"draft");
}
#[test]
fn invalid_path_rolls_back_entire_batch_and_cycle_is_rejected() {
    let d = tempfile::tempdir().unwrap();
    let mut s = Store::open(d.path()).unwrap();
    let mut invalid = node("B", None, "v1", false);
    invalid.name = "../escape".into();
    s.receive(&batch(
        "",
        "c1",
        vec![
            Change::Upsert(node("A", None, "v1", false)),
            Change::Upsert(invalid),
        ],
    ))
    .unwrap();
    assert!(s.drain().is_err());
    assert_eq!(s.cursor().unwrap(), "");
    assert!(s.status("A").is_err());
}
#[test]
fn stale_server_confirmation_cannot_clear_a_new_remote_conflict() {
    let d = tempfile::tempdir().unwrap();
    let mut s = Store::open(d.path()).unwrap();
    update(&mut s, "", "c1", node("A", None, "v1", false));
    let op = s.save_local("A", b"local").unwrap();
    s.mark_upload_started(&op).unwrap();
    update(&mut s, "c1", "c2", node("A", None, "external", false));
    assert!(s.confirm_upload(&op, "uploaded").is_err());
    assert_eq!(read(&s, "A"), b"local");
    assert_eq!(s.status("A").unwrap(), Status::Conflict);
}
#[test]
fn absent_cache_returns_offline_error_and_never_empty_content() {
    let d = tempfile::tempdir().unwrap();
    let mut s = Store::open(d.path()).unwrap();
    update(&mut s, "", "c1", node("A", None, "v1", false));
    assert!(matches!(s.open_cached("A"), Err(Error::Offline)));
    cache(&mut s, "A", "v1", b"");
    assert_eq!(read(&s, "A"), b"");
}

#[test]
fn editing_old_open_handle_preserves_its_base_and_detects_conflict() {
    let d = tempfile::tempdir().unwrap();
    let mut s = Store::open(d.path()).unwrap();
    update(&mut s, "", "c1", node("A", None, "v1", false));
    cache(&mut s, "A", "v1", b"old");
    let handle = s.open_cached("A").unwrap();
    update(&mut s, "c1", "c2", node("A", None, "v2", false));
    cache(&mut s, "A", "v2", b"remote");
    let op = s
        .save_local_from_version("A", handle.version(), b"edit of old")
        .unwrap();
    assert_eq!(s.status("A").unwrap(), Status::Conflict);
    assert!(s.mark_upload_started(&op).is_err());
    assert_eq!(read(&s, "A"), b"edit of old");
}

#[test]
fn moving_directory_into_descendant_rolls_back() {
    let d = tempfile::tempdir().unwrap();
    let mut s = Store::open(d.path()).unwrap();
    update(&mut s, "", "c1", node("parent", None, "v1", true));
    update(
        &mut s,
        "c1",
        "c2",
        node("child", Some("parent"), "v1", true),
    );
    s.receive(&batch(
        "c2",
        "c3",
        vec![Change::Upsert(node("parent", Some("child"), "v2", true))],
    ))
    .unwrap();
    assert!(s.drain().is_err());
    assert_eq!(s.cursor().unwrap(), "c2");
}
