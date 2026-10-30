use drive_core::*;
use std::io::Read;
fn node(id: &str, parent: Option<&str>, name: &str, directory: bool) -> Node {
    Node {
        id: id.into(),
        parent: parent.map(str::to_owned),
        name: name.into(),
        version: "v1".into(),
        directory,
        metadata: Some(NodeMetadata {
            size: 5,
            mtime: 1,
            mode: if directory { 0o755 } else { 0o644 },
        }),
    }
}
fn fixture() -> (tempfile::TempDir, Store, String, String, String) {
    let d = tempfile::tempdir().unwrap();
    let mut s = Store::open(d.path()).unwrap();
    s.install_remote_snapshot(&[node("root", None, "root", true)], "1", "server")
        .unwrap();
    s.ensure_fs_root("root").unwrap();
    let a = s.create_local_directory("root", "A", 0o750).unwrap();
    let b = s.create_local_directory(&a.id, "B", 0o700).unwrap();
    let f = s.create_local_file(&b.id, "note", 0o600).unwrap();
    s.save_local(&f.id, b"offline note").unwrap();
    s.creation_saved(&f.id).unwrap();
    let q = s.next_directory().unwrap().unwrap();
    s.conflict_directory(&q.job.operation).unwrap();
    (d, s, a.id, b.id, f.id)
}
fn issue(s: &Store, id: &str) -> conflicts::SyncIssue {
    let i = s.sync_issue("root", id).unwrap().unwrap();
    assert!(i.can_keep_directory);
    assert!(!i.can_keep_both);
    assert_eq!(i.reason, "directory_creation_conflict");
    i
}
fn confirm(s: &mut Store, remote: &str) {
    let q = s.next_directory().unwrap().unwrap();
    let j = s.mark_directory_started(&q.job.operation).unwrap();
    let mut n = j.node;
    n.version = format!("version-{remote}");
    s.confirm_directory(&q.job.operation, &n, remote).unwrap();
}
fn bytes(s: &Store, id: &str) -> Vec<u8> {
    let mut v = Vec::new();
    s.open_cached(id).unwrap().read_to_end(&mut v).unwrap();
    v
}
#[test]
fn nested_directory_conflict_keeps_inode_content_and_journal_order_across_restart() {
    let (d, mut s, a, b, f) = fixture();
    s.pin(&a, true).unwrap();
    s.set_local_mode(&a, 0o700).unwrap();
    let inos = [
        s.record(&a).unwrap().ino,
        s.record(&b).unwrap().ino,
        s.record(&f).unwrap().ino,
    ];
    let mut handle = s.open_cached(&f).unwrap();
    s.receive(&Batch {
        from: "1".into(),
        to: "2".into(),
        events: vec![
            Event {
                id: "remote-directory".into(),
                change: Change::Upsert(node("99", Some("root"), "A", true)),
            },
            Event {
                id: "remote-child".into(),
                change: Change::Upsert(node("100", Some("99"), "theirs", false)),
            },
        ],
    })
    .unwrap();
    s.drain().unwrap();
    let before = s.next_directory().unwrap().unwrap();
    let i = issue(&s, &a);
    let kept = s
        .keep_directory_both("root", &a, &i.token, "Local A")
        .unwrap();
    assert_eq!(kept.node.id, a);
    assert_eq!(kept.path, "Local A");
    assert_eq!(kept.old_name, "A");
    assert_eq!(s.lookup("root", "A").unwrap().node.id, "99");
    assert_eq!(s.lookup("99", "theirs").unwrap().node.id, "100");
    assert_eq!(s.record(&a).unwrap().ino, inos[0]);
    assert_eq!(s.record(&b).unwrap().ino, inos[1]);
    assert_eq!(s.record(&f).unwrap().ino, inos[2]);
    let mut v = Vec::new();
    handle.read_to_end(&mut v).unwrap();
    assert_eq!(v, b"offline note");
    drop(handle);
    assert_eq!(s.record(&a).unwrap().mode, 0o700);
    let q = s.next_directory().unwrap().unwrap();
    assert_ne!(q.job.operation, before.job.operation);
    assert_eq!(q.job.path, "Local A");
    assert!(q.job.origin.is_none());
    assert!(s.sync_issues("root", "").unwrap().entries.is_empty());
    drop(s);
    let mut s = Store::open(d.path()).unwrap();
    assert_eq!(bytes(&s, &f), b"offline note");
    assert_eq!(s.lookup("root", "Local A").unwrap().ino, inos[0]);
    confirm(&mut s, "101");
    let q = s.next_directory().unwrap().unwrap();
    assert_eq!(q.job.path, "Local A/B");
    assert_eq!(q.job.parent.id, a);
    assert_eq!(q.job.parent.name, "Local A");
    confirm(&mut s, "102");
    assert!(s.next_namespace_kind().unwrap().is_none());
    for op in s.upload_operations("pending").unwrap() {
        assert_eq!(s.upload_job(&op).unwrap().relative_path, "Local A/B/note");
    }
    assert_eq!(s.node("99").unwrap().version, "v1");
    assert_eq!(s.local_identity("99").unwrap(), "99");
    let db = rusqlite::Connection::open(d.path().join("state.sqlite")).unwrap();
    let pinned: bool = db
        .query_row("SELECT pinned FROM nodes WHERE id=?1", [&a], |r| r.get(0))
        .unwrap();
    assert!(pinned);
    let old: (String, String) = db
        .query_row(
            "SELECT state,body FROM namespace_ops WHERE id=?1",
            [&before.job.operation],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert_eq!(old.0, "superseded");
    assert_eq!(
        serde_json::from_str::<directories::DirectoryJob>(&old.1)
            .unwrap()
            .path,
        "A"
    );
}
#[test]
fn child_edits_and_new_children_invalidate_old_decision() {
    let (_d, mut s, a, b, f) = fixture();
    let i = issue(&s, &a);
    s.save_local(&f, b"later edit").unwrap();
    assert!(s.keep_directory_both("root", &a, &i.token, "Copy").is_err());
    let i = issue(&s, &a);
    s.create_local_file(&b, "later", 0o644).unwrap();
    assert!(s.keep_directory_both("root", &a, &i.token, "Copy").is_err());
    let i = issue(&s, &a);
    s.keep_directory_both("root", &a, &i.token, "Copy").unwrap();
    assert_eq!(bytes(&s, &f), b"later edit");
}
#[test]
fn uncertain_directory_or_child_commit_is_not_rewritten() {
    let (d, mut s, a, _b, f) = fixture();
    let db = rusqlite::Connection::open(d.path().join("state.sqlite")).unwrap();
    db.execute(
        "UPDATE namespace_ops SET state='uncertain' WHERE node=?1",
        [&a],
    )
    .unwrap();
    let i = s.sync_issue("root", &a).unwrap().unwrap();
    assert_eq!(i.reason, "commit_uncertain");
    assert!(!i.can_keep_directory);
    assert!(s.keep_directory_both("root", &a, &i.token, "Copy").is_err());
    db.execute(
        "UPDATE namespace_ops SET state='conflict' WHERE node=?1",
        [&a],
    )
    .unwrap();
    db.execute(
        "UPDATE operations SET state='uncertain' WHERE node=?1",
        [&f],
    )
    .unwrap();
    let i = s.sync_issue("root", &a).unwrap().unwrap();
    assert!(!i.can_keep_directory);
    assert!(s.keep_directory_both("root", &a, &i.token, "Copy").is_err());
}
#[test]
fn mixed_remote_identity_or_dependent_move_prevents_local_tree_rewrite() {
    let (_d, mut s, a, b, _f) = fixture();
    s.receive(&Batch {
        from: "1".into(),
        to: "2".into(),
        events: vec![Event {
            id: "e".into(),
            change: Change::Upsert(node("77", Some("root"), "remote", false)),
        }],
    })
    .unwrap();
    s.drain().unwrap();
    s.rename_local("root", "remote", &b, "remote", true)
        .unwrap();
    assert!(
        !s.sync_issue("root", &a)
            .unwrap()
            .unwrap()
            .can_keep_directory
    );
}
#[test]
fn outward_pending_move_is_not_orphaned_by_parent_resolution() {
    let (_d, mut s, a, b, _f) = fixture();
    s.rename_local(&a, "B", "root", "Moved B", true).unwrap();
    assert!(
        !s.sync_issue("root", &a)
            .unwrap()
            .unwrap()
            .can_keep_directory
    );
    assert_eq!(s.lookup("root", "Moved B").unwrap().node.id, b);
}
#[test]
fn journal_failure_rolls_back_names_children_and_old_conflict() {
    let (d, mut s, a, _b, f) = fixture();
    let i = issue(&s, &a);
    let before = s.next_directory().unwrap().unwrap();
    let db = rusqlite::Connection::open(d.path().join("state.sqlite")).unwrap();
    db.execute_batch("CREATE TRIGGER fail_archive BEFORE INSERT ON namespace_ops WHEN NEW.state='superseded' BEGIN SELECT RAISE(ABORT,'injected archive failure'); END;").unwrap();
    assert!(s.keep_directory_both("root", &a, &i.token, "Copy").is_err());
    assert_eq!(s.lookup("root", "A").unwrap().node.id, a);
    assert!(s.lookup("root", "Copy").is_err());
    assert_eq!(bytes(&s, &f), b"offline note");
    let q = s.next_directory().unwrap().unwrap();
    assert_eq!(q.state, "conflict");
    assert_eq!(q.job.operation, before.job.operation);
    assert_eq!(q.job.path, "A");
}
#[test]
fn invalid_or_occupied_names_do_not_modify_local_tree() {
    let (_d, mut s, a, _b, f) = fixture();
    s.create_local_file("root", "occupied", 0o644).unwrap();
    let i = issue(&s, &a);
    for name in ["A", "occupied", "../outside", ""] {
        assert!(s.keep_directory_both("root", &a, &i.token, name).is_err());
    }
    assert_eq!(bytes(&s, &f), b"offline note");
    assert_eq!(s.lookup("root", "A").unwrap().node.id, a);
}

#[test]
fn new_attempt_discards_old_provenance_but_archives_it_and_rejects_late_receipt() {
    let (d, mut s, a, _b, _f) = fixture();
    let mut old = s.next_directory().unwrap().unwrap().job;
    old.origin = Some(directories::DirectoryOrigin {
        cursor: "native-v1:10".into(),
        committer: "a".repeat(64),
        restore: "server".into(),
    });
    let db = rusqlite::Connection::open(d.path().join("state.sqlite")).unwrap();
    db.execute(
        "UPDATE namespace_ops SET body=?2 WHERE id=?1",
        rusqlite::params![old.operation, serde_json::to_string(&old).unwrap()],
    )
    .unwrap();
    let i = issue(&s, &a);
    s.keep_directory_both("root", &a, &i.token, "Copy").unwrap();
    let q = s.next_directory().unwrap().unwrap();
    assert!(q.job.origin.is_none());
    assert_ne!(q.job.operation, old.operation);
    let archived: String = db
        .query_row(
            "SELECT body FROM namespace_ops WHERE id=?1",
            [&old.operation],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(
        serde_json::from_str::<directories::DirectoryJob>(&archived)
            .unwrap()
            .origin
            .unwrap()
            .committer,
        "a".repeat(64)
    );
    let mut late = old.node;
    late.version = "old-created-version".into();
    assert!(s.confirm_directory(&old.operation, &late, "999").is_err());
    assert_eq!(s.local_identity("999").unwrap(), "999");
    s.conflict_directory(&q.job.operation).unwrap();
    let i = issue(&s, &a);
    s.keep_directory_both("root", &a, &i.token, "Copy again")
        .unwrap();
    assert_eq!(s.next_directory().unwrap().unwrap().job.path, "Copy again");
    confirm(&mut s, "900");
    assert_eq!(
        s.next_directory().unwrap().unwrap().job.path,
        "Copy again/B"
    );
}
