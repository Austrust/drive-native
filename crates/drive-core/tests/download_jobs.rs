use drive_core::{download_jobs::*, *};
fn node(version: &str) -> Node {
    Node {
        id: "file".into(),
        parent: Some("root".into()),
        name: "A.doc".into(),
        version: version.into(),
        directory: false,
        metadata: None,
    }
}
fn root() -> Node {
    Node {
        id: "root".into(),
        parent: None,
        name: "root".into(),
        version: "v1".into(),
        directory: true,
        metadata: None,
    }
}
fn ticket(version: &str) -> DownloadTicket {
    DownloadTicket {
        id: "file".into(),
        version: version.into(),
    }
}
fn fixture() -> (tempfile::TempDir, Store) {
    let d = tempfile::tempdir().unwrap();
    let mut s = Store::open(d.path()).unwrap();
    s.install_remote_snapshot(&[root(), node("v1")], "c1", "server")
        .unwrap();
    (d, s)
}
#[test]
fn failure_backoff_survives_restart_and_stops_until_explicit_retry() {
    let (d, mut s) = fixture();
    let t = ticket("v1");
    for attempt in 1..=5 {
        s.begin_download(&t).unwrap();
        s.fail_download(&t, DownloadFailureKind::ServerRejected, Some(32772), 100)
            .unwrap();
        let f = s.download_failure("file").unwrap().unwrap();
        assert_eq!(f.attempts, attempt);
        assert_eq!(f.server_code, Some(32772));
        assert!(!s.download_ready(&t, 100).unwrap());
        assert_eq!(
            f.retry_at,
            if attempt < 5 {
                Some(100 + (2 << (attempt - 1)))
            } else {
                None
            }
        );
        drop(s);
        s = Store::open(d.path()).unwrap();
        assert_eq!(s.download_failure("file").unwrap(), Some(f));
    }
    assert_eq!(s.next_download_retry().unwrap(), None);
    assert!(!s.download_ready(&t, u64::MAX).unwrap());
    assert_eq!(s.download_failure_count().unwrap(), 1);
    assert_eq!(s.downloads_needed().unwrap().len(), 1);
    assert!(s.retry_download("file").unwrap());
    assert!(s.download_ready(&t, 0).unwrap());
    assert_eq!(s.download_failure_count().unwrap(), 0);
}
#[test]
fn failure_is_bound_to_version_and_preserves_complete_old_content() {
    let (_d, mut s) = fixture();
    s.publish_download(&ticket("v1"), b"old", &sha256(b"old"))
        .unwrap();
    s.install_remote_snapshot(&[root(), node("v2")], "c2", "server")
        .unwrap();
    s.fail_download(&ticket("v2"), DownloadFailureKind::Transport, None, 10)
        .unwrap();
    assert_eq!(s.open_cached("file").unwrap().version(), "v1");
    assert_eq!(s.next_download_retry().unwrap(), Some(12));
    s.install_remote_snapshot(&[root(), node("v3")], "c3", "server")
        .unwrap();
    assert!(s.download_failure("file").unwrap().is_none());
    assert!(s.download_ready(&ticket("v3"), 0).unwrap());
    s.fail_download(
        &ticket("v2"),
        DownloadFailureKind::InvalidContent,
        None,
        100,
    )
    .unwrap();
    assert!(s.download_failure("file").unwrap().is_none());
    s.begin_download(&ticket("v3")).unwrap();
    s.publish_download(&ticket("v3"), b"new", &sha256(b"new"))
        .unwrap();
    s.finish_download(&ticket("v3")).unwrap();
    assert_eq!(s.download_failure_count().unwrap(), 0);
    assert_eq!(s.open_cached("file").unwrap().version(), "v3");
}
#[test]
fn release_cancels_failed_read_intent_without_remote_operation() {
    let (_d, mut s) = fixture();
    s.fail_download(&ticket("v1"), DownloadFailureKind::Transport, None, 100)
        .unwrap();
    assert_eq!(s.downloads_needed().unwrap().len(), 1);
    s.release("file").unwrap();
    assert!(s.downloads_needed().unwrap().is_empty());
    assert_eq!(s.next_download_retry().unwrap(), None);
    assert_eq!(s.pending_operations().unwrap(), 0);
    assert_eq!(s.pending_namespace_operations().unwrap(), 0);
}
#[test]
fn in_flight_read_survives_restart_and_local_edits_block_background_retry() {
    let (d, mut s) = fixture();
    let t = ticket("v1");
    s.begin_download(&t).unwrap();
    drop(s);
    let mut s = Store::open(d.path()).unwrap();
    assert_eq!(s.downloads_needed().unwrap().len(), 1);
    s.fail_download(&t, DownloadFailureKind::Transport, None, 100)
        .unwrap();
    s.save_local("file", b"local").unwrap();
    assert!(s.downloads_needed().unwrap().is_empty());
    assert_eq!(s.next_download_retry().unwrap(), None);
}

#[test]
fn v9_upgrade_backs_up_and_preserves_cache_cursor_and_download_intent() {
    let (d, mut s) = fixture();
    s.publish_download(&ticket("v1"), b"old", &sha256(b"old"))
        .unwrap();
    s.queue_download(&ticket("v1")).unwrap();
    drop(s);
    let db = rusqlite::Connection::open(d.path().join("state.sqlite")).unwrap();
    db.pragma_update(None, "user_version", 9).unwrap();
    drop(db);
    let s = Store::open(d.path()).unwrap();
    assert_eq!(s.cursor().unwrap(), "c1");
    assert_eq!(s.open_cached("file").unwrap().version(), "v1");
    assert!(std::fs::read_dir(d.path()).unwrap().any(|p| {
        p.unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with(&format!(
                "state-before-v{}-",
                drive_core::STATE_SCHEMA_VERSION
            ))
    }));
    let db = rusqlite::Connection::open(d.path().join("state.sqlite")).unwrap();
    assert_eq!(
        db.pragma_query_value(None, "user_version", |r| r.get::<_, u32>(0))
            .unwrap(),
        drive_core::STATE_SCHEMA_VERSION as u32
    );
    assert_eq!(
        db.query_row("SELECT count(*) FROM transfers", [], |r| r.get::<_, u32>(0))
            .unwrap(),
        1
    );
}
