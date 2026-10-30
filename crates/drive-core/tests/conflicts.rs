use drive_core::*;
use std::io::Read;
fn node(id: &str, parent: Option<&str>, version: &str) -> Node {
    Node {
        id: id.into(),
        parent: parent.map(str::to_owned),
        name: if parent.is_some() {
            "A.doc".into()
        } else {
            "root".into()
        },
        version: version.into(),
        directory: parent.is_none(),
        metadata: Some(NodeMetadata {
            size: 3,
            mtime: 1,
            mode: 0o644,
        }),
    }
}
fn fixture() -> (tempfile::TempDir, Store) {
    let d = tempfile::tempdir().unwrap();
    let mut s = Store::open(d.path()).unwrap();
    s.install_remote_snapshot(
        &[node("root", None, "r"), node("a", Some("root"), "v1")],
        "1",
        "server",
    )
    .unwrap();
    s.ensure_fs_root("root").unwrap();
    s.publish_download(
        &DownloadTicket {
            id: "a".into(),
            version: "v1".into(),
        },
        b"old",
        &sha256(b"old"),
    )
    .unwrap();
    (d, s)
}
fn change(s: &mut Store, to: &str, change: Change) {
    let from = s.cursor().unwrap();
    s.receive(&Batch {
        from,
        to: to.into(),
        events: vec![Event {
            id: to.into(),
            change,
        }],
    })
    .unwrap();
    s.drain().unwrap();
}
fn conflict(s: &mut Store) {
    s.save_local("a", b"mine").unwrap();
    change(s, "2", Change::Upsert(node("a", Some("root"), "v2")));
}
fn bytes(s: &Store, id: &str) -> Vec<u8> {
    let mut v = Vec::new();
    s.open_cached(id).unwrap().read_to_end(&mut v).unwrap();
    v
}
#[test]
fn keep_both_survives_restart_preserves_local_and_never_uploads_original() {
    let (d, mut s) = fixture();
    conflict(&mut s);
    let ino = s.record("a").unwrap().ino;
    let issue = s.sync_issue("root", "a").unwrap().unwrap();
    assert!(issue.can_keep_both);
    let kept = s
        .keep_both("root", "a", &issue.token, "A (local conflict).doc")
        .unwrap();
    assert_eq!(s.record("a").unwrap().ino, ino);
    assert_eq!(bytes(&s, &kept.copy.id), b"mine");
    assert_eq!(bytes(&s, "a"), b"old");
    assert!(s.sync_issues("root", "").unwrap().entries.is_empty());
    let ops = s.upload_operations("pending").unwrap();
    assert_eq!(ops.len(), 1);
    let job = s.upload_job(&ops[0]).unwrap();
    assert_eq!(job.node.id, kept.copy.id);
    assert_eq!(job.node.version, creations::CREATE_VERSION);
    drop(job);
    drop(s);
    let mut s = Store::open(d.path()).unwrap();
    assert_eq!(bytes(&s, &kept.copy.id), b"mine");
    s.publish_download(
        &DownloadTicket {
            id: "a".into(),
            version: "v2".into(),
        },
        b"theirs",
        &sha256(b"theirs"),
    )
    .unwrap();
    assert_eq!(bytes(&s, "a"), b"theirs");
    assert_eq!(bytes(&s, &kept.copy.id), b"mine");
}
#[test]
fn stale_decision_names_and_live_handles_are_rejected() {
    let (_d, mut s) = fixture();
    conflict(&mut s);
    let issue = s.sync_issue("root", "a").unwrap().unwrap();
    for name in ["A.doc", "../other", ""] {
        assert!(s.keep_both("root", "a", &issue.token, name).is_err());
    }
    let h = s.open_cached("a").unwrap();
    assert_eq!(
        s.sync_issue("root", "a").unwrap().unwrap().reason,
        "file_in_use"
    );
    assert!(s.keep_both("root", "a", &issue.token, "copy.doc").is_err());
    drop(h);
    s.save_local("a", b"newer local").unwrap();
    assert!(s.keep_both("root", "a", &issue.token, "copy.doc").is_err());
    let fresh = s.sync_issue("root", "a").unwrap().unwrap();
    change(&mut s, "3", Change::Upsert(node("a", Some("root"), "v3")));
    assert!(s.keep_both("root", "a", &fresh.token, "copy.doc").is_err());
    assert_eq!(bytes(&s, "a"), b"newer local");
}
#[test]
fn unknown_commit_and_namespace_conflicts_cannot_be_bypassed() {
    let (_d, mut s) = fixture();
    let op = s.save_local("a", b"mine").unwrap();
    s.mark_upload_started(&op).unwrap();
    s.mark_upload_uncertain(&op).unwrap();
    change(&mut s, "2", Change::Upsert(node("a", Some("root"), "v2")));
    let issue = s.sync_issue("root", "a").unwrap().unwrap();
    assert_eq!(issue.reason, "commit_uncertain");
    assert!(!issue.can_keep_both);
    assert!(s.keep_both("root", "a", &issue.token, "copy.doc").is_err());
    s.conflict_upload(&op).unwrap();
    s.rename_local("root", "A.doc", "root", "moved.doc", true)
        .unwrap();
    let issue = s.sync_issue("root", "a").unwrap().unwrap();
    assert_eq!(issue.reason, "namespace_conflict");
    assert!(!issue.can_keep_both);
}
#[test]
fn closed_uncheckpointed_draft_survives_remote_deletion() {
    let (_d, mut s) = fixture();
    let source = s.open_cached("a").unwrap();
    let draft = s.begin_draft("a", "v1", Some(&source)).unwrap();
    drop(source);
    s.draft_write(&draft, 0, b"latest unsaved draft", false)
        .unwrap();
    drop(draft);
    change(&mut s, "2", Change::Delete { id: "a".into() });
    let issue = s.sync_issue("root", "a").unwrap().unwrap();
    assert_eq!(issue.reason, "remote_deleted");
    let kept = s
        .keep_both("root", "a", &issue.token, "recovered.doc")
        .unwrap();
    assert_eq!(bytes(&s, &kept.copy.id), b"latest unsaved draft");
    assert!(s.node("a").is_err());
    assert!(s.sync_issues("root", "").unwrap().entries.is_empty());
}
#[test]
fn database_failure_rolls_back_resolution_and_keeps_original_bytes() {
    let (d, mut s) = fixture();
    conflict(&mut s);
    let issue = s.sync_issue("root", "a").unwrap().unwrap();
    let db = rusqlite::Connection::open(d.path().join("state.sqlite")).unwrap();
    db.execute_batch("CREATE TRIGGER fail_resolution BEFORE UPDATE OF conflict ON nodes WHEN NEW.conflict=0 BEGIN SELECT RAISE(ABORT,'injected transaction failure'); END;").unwrap();
    assert!(s.keep_both("root", "a", &issue.token, "copy.doc").is_err());
    assert!(s.lookup("root", "copy.doc").is_err());
    assert_eq!(bytes(&s, "a"), b"mine");
    assert_eq!(s.status("a").unwrap(), Status::Conflict);
}
#[test]
fn issue_query_is_local_and_task_scoped() {
    let (_d, mut s) = fixture();
    conflict(&mut s);
    let page = s.sync_issues("root", "").unwrap();
    assert_eq!(page.entries.len(), 1);
    assert_eq!(page.entries[0].path, "A.doc");
    assert!(s.sync_issue("foreign-root", "a").is_err());
    assert_eq!(s.upload_operations("pending").unwrap().len(), 1);
}

#[test]
fn issue_pages_are_bounded_and_do_not_skip_boundary_identity() {
    let (d, mut s) = fixture();
    let mut nodes = vec![node("root", None, "r")];
    for i in 0..260 {
        let mut n = node(&format!("file-{i:03}"), Some("root"), "v1");
        n.name = format!("{i:03}.doc");
        nodes.push(n);
    }
    s.install_remote_snapshot(&nodes, "2", "server").unwrap();
    let db = rusqlite::Connection::open(d.path().join("state.sqlite")).unwrap();
    db.execute("UPDATE nodes SET conflict=1 WHERE id LIKE 'file-%'", [])
        .unwrap();
    let first = s.sync_issues("root", "").unwrap();
    assert_eq!(first.entries.len(), 256);
    let second = s
        .sync_issues("root", first.next.as_deref().unwrap())
        .unwrap();
    assert_eq!(second.entries.len(), 4);
    assert!(second.next.is_none());
    let ids: std::collections::HashSet<_> = first
        .entries
        .iter()
        .chain(second.entries.iter())
        .map(|i| &i.id)
        .collect();
    assert_eq!(ids.len(), 260);
}
