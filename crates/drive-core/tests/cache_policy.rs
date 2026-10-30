use drive_core::*;
use std::collections::HashSet;
fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
fn node(id: &str) -> Node {
    Node {
        id: id.into(),
        parent: if id == "root" {
            None
        } else {
            Some("root".into())
        },
        name: id.into(),
        version: "1".into(),
        directory: id == "root",
        metadata: None,
    }
}
fn setup() -> (tempfile::TempDir, Store) {
    let dir = tempfile::tempdir().unwrap();
    let mut s = Store::open(dir.path()).unwrap();
    s.install_remote_snapshot(
        &[node("root"), node("a"), node("b"), node("c"), node("d")],
        "1",
        "server",
    )
    .unwrap();
    for id in ["a", "b", "c", "d"] {
        s.publish_download(
            &DownloadTicket {
                id: id.into(),
                version: "1".into(),
            },
            b"body",
            &sha256(b"body"),
        )
        .unwrap();
    }
    (dir, s)
}
#[test]
fn default_disabled_and_lru_survives_restart_without_icon_query_touching_it() {
    let (dir, mut s) = setup();
    assert_eq!(s.cache_limit().unwrap(), 0);
    assert!(
        s.trim_cache(now() + 120, &HashSet::new())
            .unwrap()
            .released
            .is_empty()
    );
    s.note_cache_use("c").unwrap();
    s.note_cache_use("a").unwrap();
    s.note_cache_use("b").unwrap();
    s.note_cache_use("d").unwrap();
    // Status providers open a short local read handle; that must not affect LRU.
    drop(s.open_cached("c").unwrap());
    s.set_cache_limit(8).unwrap();
    drop(s);
    let mut s = Store::open(dir.path()).unwrap();
    let result = s.trim_cache(now() + 120, &HashSet::new()).unwrap();
    assert_eq!(result.released, vec!["c", "a"]);
    assert_eq!(result.usage.content_bytes, 8);
    assert_eq!(result.reclaimed_bytes, 8);
    assert_eq!(s.pending_operations().unwrap(), 0);
    assert_eq!(s.pending_namespace_operations().unwrap(), 0);
    assert_eq!(s.status("a").unwrap(), Status::OnlineOnly);
    assert_eq!(s.status("d").unwrap(), Status::LocalComplete);
}
#[test]
fn recent_download_active_transfer_and_open_handle_are_protected() {
    let (_dir, mut s) = setup();
    s.set_cache_limit(1).unwrap();
    assert!(
        s.trim_cache(now(), &HashSet::new())
            .unwrap()
            .released
            .is_empty()
    );
    let handle = s.open_cached("a").unwrap();
    let active = HashSet::from(["b".into()]);
    let result = s.trim_cache(now() + 120, &active).unwrap();
    assert_eq!(result.released, vec!["c", "d"]);
    assert_eq!(result.usage.over_target_bytes, 7);
    assert_eq!(result.usage.protected_bytes, 8);
    drop(handle);
    assert!(
        s.trim_cache(now() + 120, &HashSet::new())
            .unwrap()
            .released
            .contains(&"a".into())
    );
}
#[test]
fn inherited_pin_and_pending_or_conflicted_writes_cannot_be_evicted() {
    let (_dir, mut s) = setup();
    s.set_cache_limit(1).unwrap();
    s.pin("root", true).unwrap();
    assert!(
        s.trim_cache(now() + 120, &HashSet::new())
            .unwrap()
            .released
            .is_empty()
    );
    s.pin("root", false).unwrap();
    s.pin("a", true).unwrap();
    let op = s.save_local("b", b"pending").unwrap();
    s.mark_upload_started(&op).unwrap();
    s.save_local("c", b"conflict").unwrap();
    let mut nodes = vec![node("root"), node("a"), node("b"), node("c"), node("d")];
    nodes[3].version = "2".into();
    s.install_remote_snapshot(&nodes, "2", "server").unwrap();
    let result = s.trim_cache(now() + 120, &HashSet::new()).unwrap();
    assert_eq!(result.released, vec!["d"]);
    assert!(s.release("b").is_err());
    assert!(s.release("c").is_err());
    assert_eq!(s.status("c").unwrap(), Status::Conflict);
    use std::io::Read;
    let mut body = Vec::new();
    s.open_cached("b").unwrap().read_to_end(&mut body).unwrap();
    assert_eq!(body, b"pending");
}
#[test]
fn stale_complete_version_remains_available_when_update_has_not_finished() {
    let (_dir, mut s) = setup();
    let mut nodes = vec![node("root"), node("a"), node("b"), node("c"), node("d")];
    nodes[1].version = "2".into();
    s.install_remote_snapshot(&nodes, "2", "server").unwrap();
    s.set_cache_limit(1).unwrap();
    let result = s.trim_cache(now() + 120, &HashSet::new()).unwrap();
    assert!(!result.released.contains(&"a".into()));
    assert_eq!(s.status("a").unwrap(), Status::Updating);
    assert!(s.open_cached("a").is_ok());
}
#[test]
fn garbage_collection_preserves_dirty_and_open_drafts_then_reclaims_closed_clean_copy() {
    let (_dir, mut s) = setup();
    let mut source = s.open_cached("a").unwrap();
    let clean = s.begin_draft("a", "1", Some(&mut source)).unwrap();
    drop(source);
    let mut source = s.open_cached("b").unwrap();
    let dirty = s.begin_draft("b", "1", Some(&mut source)).unwrap();
    drop(source);
    s.draft_write(&dirty, 0, b"edit", false).unwrap();
    drop(dirty);
    assert_eq!(s.collect_garbage().unwrap(), 0);
    drop(clean);
    assert_eq!(s.collect_garbage().unwrap(), 4);
    let dirty = s.open_dirty_draft("b").unwrap().unwrap();
    let mut buf = [0; 4];
    dirty.read_at(&mut buf, 0).unwrap();
    assert_eq!(&buf, b"edit");
    assert_eq!(
        s.cache_usage(now() + 120, &HashSet::new())
            .unwrap()
            .content_bytes,
        20
    );
}
#[test]
fn schema_11_upgrade_keeps_contents_and_creates_backup_and_policy_tables() {
    let (dir, s) = setup();
    drop(s);
    let db = rusqlite::Connection::open(dir.path().join("state.sqlite")).unwrap();
    db.execute_batch("DROP TABLE cache_access;DROP TABLE cache_published;PRAGMA user_version=11;")
        .unwrap();
    drop(db);
    let s = Store::open(dir.path()).unwrap();
    assert!(s.open_cached("a").is_ok());
    assert_eq!(s.cache_limit().unwrap(), 0);
    s.note_cache_use("a").unwrap();
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
