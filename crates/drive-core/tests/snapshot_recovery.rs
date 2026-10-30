use drive_core::*;
use std::io::Read;
fn node(id: &str, version: &str) -> Node {
    Node {
        id: id.into(),
        parent: if id == "root" {
            None
        } else {
            Some("root".into())
        },
        name: id.into(),
        version: version.into(),
        directory: id == "root",
        metadata: None,
    }
}
fn read(s: &Store, id: &str) -> Vec<u8> {
    let mut b = Vec::new();
    s.open_cached(id).unwrap().read_to_end(&mut b).unwrap();
    b
}
#[test]
fn snapshot_recovery_preserves_dirty_missing_file_pins_and_complete_old_content() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = Store::open(dir.path()).unwrap();
    s.install_remote_snapshot(
        &[node("root", "1"), node("a", "1"), node("b", "1")],
        "cursor1",
        "server1",
    )
    .unwrap();
    s.pin("root", true).unwrap();
    s.publish_download(
        &DownloadTicket {
            id: "a".into(),
            version: "1".into(),
        },
        b"old",
        &sha256(b"old"),
    )
    .unwrap();
    let op = s.save_local("b", b"unsent").unwrap();
    s.mark_upload_started(&op).unwrap();
    s.receive(&Batch {
        from: "cursor1".into(),
        to: "old-unapplied".into(),
        events: vec![Event {
            id: "stale".into(),
            change: Change::Delete { id: "a".into() },
        }],
    })
    .unwrap();
    s.install_remote_snapshot(
        &[node("root", "2"), node("a", "2"), node("c", "2")],
        "cursor9",
        "server2",
    )
    .unwrap();
    assert_eq!(s.cursor().unwrap(), "cursor9");
    assert!(!s.apply_next().unwrap());
    assert_eq!(read(&s, "a"), b"old");
    assert_eq!(read(&s, "b"), b"unsent");
    assert_eq!(s.status("a").unwrap(), Status::Updating);
    assert_eq!(s.status("b").unwrap(), Status::Conflict);
    assert!(s.is_pinned("c").unwrap());
    assert!(s.mark_upload_started(&op).is_err());
    s.collect_garbage().unwrap();
    drop(s);
    let s = Store::open(dir.path()).unwrap();
    assert_eq!(read(&s, "b"), b"unsent");
    assert_eq!(s.cursor().unwrap(), "cursor9");
}
#[test]
fn incomplete_cyclic_or_ambiguous_snapshot_cannot_advance_cursor() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = Store::open(dir.path()).unwrap();
    s.install_remote_snapshot(&[node("root", "1"), node("a", "1")], "c1", "server")
        .unwrap();
    let mut missing = node("a", "2");
    missing.parent = Some("missing".into());
    assert!(
        s.install_remote_snapshot(&[node("root", "2"), missing], "c2", "server")
            .is_err()
    );
    let mut a = node("a", "2");
    a.directory = true;
    a.parent = Some("b".into());
    let mut b = node("b", "2");
    b.directory = true;
    b.parent = Some("a".into());
    assert!(
        s.install_remote_snapshot(&[node("root", "2"), a, b], "c2", "server")
            .is_err()
    );
    let mut collision = node("b", "2");
    collision.name = "a".into();
    assert!(
        s.install_remote_snapshot(
            &[node("root", "2"), node("a", "2"), collision],
            "c2",
            "server"
        )
        .is_err()
    );
    assert_eq!(s.cursor().unwrap(), "c1");
    assert_eq!(s.node("a").unwrap().version, "1");
}
#[test]
fn recovery_cannot_switch_roots_and_clean_versions_are_not_needlessly_invalidated() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = Store::open(dir.path()).unwrap();
    s.bind_native_scope("server", r#"{"file_id":"root"}"#)
        .unwrap();
    s.install_remote_snapshot(&[node("root", "1"), node("a", "1")], "c1", "server")
        .unwrap();
    s.publish_download(
        &DownloadTicket {
            id: "a".into(),
            version: "1".into(),
        },
        b"body",
        &sha256(b"body"),
    )
    .unwrap();
    s.install_remote_snapshot(&[node("root", "1"), node("a", "1")], "c2", "server")
        .unwrap();
    assert_eq!(s.status("a").unwrap(), Status::LocalComplete);
    let mut other = node("root", "2");
    other.id = "other".into();
    assert!(s.install_remote_snapshot(&[other], "c3", "server").is_err());
    assert_eq!(s.cursor().unwrap(), "c2");
}
