use drive_core::conflicts::KeptBoth;
use drive_core::*;
use std::io::Read;

fn root() -> Node {
    Node {
        id: "root".into(),
        parent: None,
        name: "root".into(),
        version: "r1".into(),
        directory: true,
        metadata: None,
    }
}
fn peer(directory: bool) -> Node {
    Node {
        id: "999".into(),
        parent: Some("root".into()),
        name: "A.doc".into(),
        version: "remote-v1".into(),
        directory,
        metadata: Some(NodeMetadata {
            size: 6,
            mtime: 1,
            mode: if directory { 0o755 } else { 0o644 },
        }),
    }
}
fn fixture() -> (tempfile::TempDir, Store, String) {
    let d = tempfile::tempdir().unwrap();
    let mut s = Store::open(d.path()).unwrap();
    s.install_remote_snapshot(&[root()], "1", "server").unwrap();
    s.ensure_fs_root("root").unwrap();
    let n = s.create_local_file("root", "A.doc", 0o640).unwrap();
    let op = s.save_local(&n.id, b"my local file").unwrap();
    s.creation_saved(&n.id).unwrap();
    s.conflict_upload(&op).unwrap();
    (d, s, n.id)
}
fn remote(s: &mut Store, n: Node) {
    s.receive(&Batch {
        from: s.cursor().unwrap(),
        to: "2".into(),
        events: vec![Event {
            id: "remote-event".into(),
            change: Change::Upsert(n),
        }],
    })
    .unwrap();
    s.drain().unwrap();
}
fn bytes(s: &Store, id: &str) -> Vec<u8> {
    let mut v = Vec::new();
    s.open_cached(id).unwrap().read_to_end(&mut v).unwrap();
    v
}
fn resolve(s: &mut Store, id: &str) -> KeptBoth {
    let issue = s.sync_issue("root", id).unwrap().unwrap();
    assert_eq!(issue.reason, "creation_conflict");
    assert!(issue.can_keep_both);
    s.keep_both("root", id, &issue.token, "copy.doc").unwrap()
}
#[test]
fn creation_keep_both_retires_only_local_identity_and_survives_restart() {
    let (d, mut s, id) = fixture();
    s.pin(&id, true).unwrap();
    s.set_local_mode(&id, 0o600).unwrap();
    remote(&mut s, peer(false));
    s.publish_download(
        &DownloadTicket {
            id: "999".into(),
            version: "remote-v1".into(),
        },
        b"theirs",
        &sha256(b"theirs"),
    )
    .unwrap();
    assert_eq!(s.lookup("root", "A.doc").unwrap().node.id, id);
    let old_ino = s.record(&id).unwrap().ino;
    let kept = resolve(&mut s, &id);
    assert!(s.node(&id).is_err());
    assert!(s.locally_unlinked(&id).unwrap());
    assert_ne!(s.lookup("root", "A.doc").unwrap().ino, old_ino);
    assert_eq!(s.lookup("root", "A.doc").unwrap().node.id, "999");
    assert_eq!(s.local_identity("999").unwrap(), "999");
    assert_eq!(s.node("999").unwrap().version, "remote-v1");
    assert_eq!(bytes(&s, "999"), b"theirs");
    assert_eq!(bytes(&s, &kept.copy.id), b"my local file");
    assert_eq!(s.record(&kept.copy.id).unwrap().mode, 0o600);
    assert!(s.sync_issues("root", "").unwrap().entries.is_empty());
    assert_eq!(s.pending_namespace_operations().unwrap(), 0);
    let ops = s.upload_operations("pending").unwrap();
    assert_eq!(ops.len(), 1);
    assert_eq!(s.upload_job(&ops[0]).unwrap().node.id, kept.copy.id);
    let db = rusqlite::Connection::open(d.path().join("state.sqlite")).unwrap();
    let pinned: bool = db
        .query_row(
            "SELECT pinned FROM nodes WHERE id=?1",
            [&kept.copy.id],
            |r| r.get(0),
        )
        .unwrap();
    assert!(pinned);
    drop(db);
    drop(s);
    let s = Store::open(d.path()).unwrap();
    assert_eq!(s.lookup("root", "A.doc").unwrap().node.id, "999");
    assert_eq!(bytes(&s, &kept.copy.id), b"my local file");
    assert!(s.sync_issues("root", "").unwrap().entries.is_empty());
}
#[test]
fn missing_peer_index_stays_absent_until_real_event_and_never_adopts_peer() {
    let (_d, mut s, id) = fixture();
    let kept = resolve(&mut s, &id);
    assert!(s.lookup("root", "A.doc").is_err());
    assert_eq!(bytes(&s, &kept.copy.id), b"my local file");
    remote(&mut s, peer(false));
    assert_eq!(s.lookup("root", "A.doc").unwrap().node.id, "999");
    assert_eq!(s.local_identity("999").unwrap(), "999");
    assert!(s.open_cached("999").is_err());
}
#[test]
fn same_named_remote_directory_and_children_remain_intact() {
    let (_d, mut s, id) = fixture();
    remote(&mut s, peer(true));
    let child = Node {
        id: "1000".into(),
        parent: Some("999".into()),
        name: "child".into(),
        ..peer(false)
    };
    s.receive(&Batch {
        from: "2".into(),
        to: "3".into(),
        events: vec![Event {
            id: "child-event".into(),
            change: Change::Upsert(child),
        }],
    })
    .unwrap();
    s.drain().unwrap();
    let kept = resolve(&mut s, &id);
    assert!(s.lookup("root", "A.doc").unwrap().node.directory);
    assert_eq!(s.lookup("999", "child").unwrap().node.id, "1000");
    assert_eq!(bytes(&s, &kept.copy.id), b"my local file");
    assert_eq!(s.pending_namespace_operations().unwrap(), 0);
}
#[test]
fn creation_resolution_rejects_open_uncertain_and_changed_decisions() {
    let (_d, mut s, id) = fixture();
    let original = s.sync_issue("root", &id).unwrap().unwrap();
    let h = s.open_cached(&id).unwrap();
    assert_eq!(
        s.sync_issue("root", &id).unwrap().unwrap().reason,
        "file_in_use"
    );
    assert!(
        s.keep_both("root", &id, &original.token, "copy.doc")
            .is_err()
    );
    drop(h);
    s.save_local(&id, b"new local bytes").unwrap();
    assert!(
        s.keep_both("root", &id, &original.token, "copy.doc")
            .is_err()
    );
    assert_eq!(bytes(&s, &id), b"new local bytes");
}
#[test]
fn unconfirmed_creation_commit_cannot_be_resolved() {
    let (_d, mut s, _) = fixture();
    let id = s
        .create_local_file("root", "uncertain.doc", 0o644)
        .unwrap()
        .id;
    let op = s.save_local(&id, b"new local bytes").unwrap();
    s.creation_saved(&id).unwrap();
    s.mark_upload_started(&op).unwrap();
    s.mark_upload_uncertain(&op).unwrap();
    let issue = s.sync_issue("root", &id).unwrap().unwrap();
    assert_eq!(issue.reason, "commit_uncertain");
    assert!(!issue.can_keep_both);
    assert!(s.keep_both("root", &id, &issue.token, "copy.doc").is_err());
    assert_eq!(bytes(&s, &id), b"new local bytes");
}
#[test]
fn retirement_failure_rolls_back_copy_queue_and_path_claim() {
    let (d, mut s, id) = fixture();
    remote(&mut s, peer(false));
    let issue = s.sync_issue("root", &id).unwrap().unwrap();
    let db = rusqlite::Connection::open(d.path().join("state.sqlite")).unwrap();
    let pending_before = s.upload_operations("pending").unwrap();
    db.execute_batch("CREATE TRIGGER fail_retire BEFORE DELETE ON local_creations BEGIN SELECT RAISE(ABORT,'injected retirement failure'); END;").unwrap();
    assert!(s.keep_both("root", &id, &issue.token, "copy.doc").is_err());
    assert!(s.lookup("root", "copy.doc").is_err());
    assert_eq!(s.lookup("root", "A.doc").unwrap().node.id, id);
    assert_eq!(bytes(&s, &id), b"my local file");
    assert!(!s.locally_unlinked(&id).unwrap());
    assert_eq!(s.status(&id).unwrap(), Status::Conflict);
    assert_eq!(s.upload_operations("pending").unwrap(), pending_before);
    assert_eq!(s.node("999").unwrap().version, "remote-v1");
}

#[test]
fn latest_closed_draft_is_copied_without_reverting_to_last_snapshot() {
    let (_d, mut s, id) = fixture();
    let source = s.open_cached(&id).unwrap();
    let draft = s
        .begin_draft(&id, creations::CREATE_VERSION, Some(&source))
        .unwrap();
    drop(source);
    s.draft_write(&draft, 0, b"latest uncheckpointed local draft", false)
        .unwrap();
    assert_eq!(
        s.sync_issue("root", &id).unwrap().unwrap().reason,
        "file_in_use"
    );
    drop(draft);
    let kept = resolve(&mut s, &id);
    assert_eq!(
        bytes(&s, &kept.copy.id),
        b"latest uncheckpointed local draft"
    );
    assert!(s.node(&id).is_err());
}
