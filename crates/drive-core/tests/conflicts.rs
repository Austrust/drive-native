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

/// A committed directory with a remote identity has no namespace action left.
/// A stale conflict flag used to surface it as an unrecoverable
/// `namespace_conflict`, hiding which operation actually blocked descendants.
#[test]
fn committed_directory_with_remote_alias_is_not_a_namespace_conflict_issue() {
    let (d, mut s) = fixture();
    let db = rusqlite::Connection::open(d.path().join("state.sqlite")).unwrap();
    db.execute(
        "INSERT INTO nodes(id,parent,name,remote_version,directory) VALUES('folder','root','folder','v-folder',1)",
        [],
    )
    .unwrap();
    db.execute(
        "INSERT INTO remote_aliases(remote,node) VALUES('900','folder')",
        [],
    )
    .unwrap();
    db.execute("UPDATE nodes SET conflict=1 WHERE id='folder'", [])
        .unwrap();
    drop(db);
    assert!(
        s.sync_issue("root", "folder").unwrap().is_none(),
        "bound committed directory must not need namespace recovery"
    );
}

fn named(id: &str, parent: Option<&str>, name: &str, version: &str, directory: bool) -> Node {
    Node {
        id: id.into(),
        parent: parent.map(str::to_owned),
        name: name.into(),
        version: version.into(),
        directory,
        metadata: Some(NodeMetadata {
            size: if directory { 5 } else { 3 },
            mtime: 1,
            mode: if directory { 0o755 } else { 0o644 },
        }),
    }
}

fn bound_folder_snapshot() -> Vec<Node> {
    vec![
        node("root", None, "r"),
        node("a", Some("root"), "v1"),
        named("900", Some("root"), "folder", "v-folder", true),
    ]
}

/// #6 follow-up: a snapshot installed while a mkdir is in flight marks the
/// overlay member as conflicted. Once the mkdir commits, the directory is bound
/// to its remote identity and has no namespace work left, so the flag is stale:
/// it must not keep `status()` at Conflict and `release()` refusing forever.
#[test]
fn mkdir_commit_heals_stale_directory_conflict() {
    let (_d, mut s) = fixture();
    let directory = s.create_local_directory("root", "folder", 0o755).unwrap();
    let queued = s.next_directory().unwrap().unwrap();
    let job = s.mark_directory_started(&queued.job.operation).unwrap();
    // The install below does not observe the in-flight directory and flags it
    // until the pending operation is confirmed.
    s.install_remote_snapshot(
        &[node("root", None, "r"), node("a", Some("root"), "v1")],
        "2",
        "server",
    )
    .unwrap();
    assert_eq!(s.status(&directory.id).unwrap(), Status::Conflict);
    let mut confirmed = job.node;
    confirmed.version = "v-folder".into();
    s.confirm_directory(&queued.job.operation, &confirmed, "900")
        .unwrap();
    assert_eq!(
        s.status(&directory.id).unwrap(),
        Status::OnlineOnly,
        "committed directory must not keep the stale conflict flag"
    );
    assert!(
        s.release(&directory.id).is_ok(),
        "healed directory must be releasable"
    );
    assert!(s.sync_issue("root", &directory.id).unwrap().is_none());
}

/// Directory renames are confirmed without content comparison; a successful
/// rename acknowledgement leaves the directory bound and conflict-free even if
/// a snapshot install flagged it while the rename was in flight.
#[test]
fn rename_commit_heals_stale_directory_conflict() {
    let (d, mut s) = fixture();
    {
        let db = rusqlite::Connection::open(d.path().join("state.sqlite")).unwrap();
        db.execute("INSERT INTO nodes(id,parent,name,remote_version,directory) VALUES('folder','root','folder','v-folder',1)", []).unwrap();
        db.execute(
            "INSERT INTO remote_aliases(remote,node) VALUES('900','folder')",
            [],
        )
        .unwrap();
        db.execute("UPDATE nodes SET conflict=1 WHERE id='folder'", [])
            .unwrap();
    }
    s.rename_local("root", "folder", "root", "folder2", false)
        .unwrap();
    let queued = s.next_rename().unwrap().unwrap();
    s.mark_rename_started(&queued.job.operation).unwrap();
    let confirmed = named("folder", Some("root"), "folder2", "v-folder2", true);
    s.confirm_rename(&queued.job.operation, &confirmed, false)
        .unwrap();
    assert_eq!(
        s.status("folder").unwrap(),
        Status::OnlineOnly,
        "committed directory rename must not keep the stale conflict flag"
    );
    assert!(s.release("folder").is_ok());
}

/// Existing databases carry the stale flag on directories that are already
/// bound and live. A snapshot install that observes such a directory has enough
/// evidence to heal it: alias present, no overlay/creation/unlink, no pending
/// operation or draft, and no dirty content.
#[test]
fn install_heals_stale_conflict_on_bound_directory() {
    let (d, mut s) = fixture();
    {
        let db = rusqlite::Connection::open(d.path().join("state.sqlite")).unwrap();
        db.execute("INSERT INTO nodes(id,parent,name,remote_version,directory) VALUES('folder','root','folder','v-folder',1)", []).unwrap();
        db.execute(
            "INSERT INTO remote_aliases(remote,node) VALUES('900','folder')",
            [],
        )
        .unwrap();
        db.execute("UPDATE nodes SET conflict=1 WHERE id='folder'", [])
            .unwrap();
    }
    assert_eq!(s.status("folder").unwrap(), Status::Conflict);
    assert!(s.release("folder").is_err());
    s.install_remote_snapshot(&bound_folder_snapshot(), "2", "server")
        .unwrap();
    assert_eq!(s.status("folder").unwrap(), Status::OnlineOnly);
    assert!(s.release("folder").is_ok());
    assert!(s.sync_issue("root", "folder").unwrap().is_none());
    // Installing the same namespace again must keep the flag cleared.
    s.install_remote_snapshot(&bound_folder_snapshot(), "3", "server")
        .unwrap();
    assert_eq!(s.status("folder").unwrap(), Status::OnlineOnly);
}

/// The heal is not conflict resolution: a bound directory with an unresolved
/// namespace operation keeps its flag, and dirty file conflicts are untouched.
#[test]
fn stale_heal_does_not_clear_real_conflicts() {
    let (d, mut s) = fixture();
    {
        let db = rusqlite::Connection::open(d.path().join("state.sqlite")).unwrap();
        db.execute("INSERT INTO nodes(id,parent,name,remote_version,directory) VALUES('folder','root','folder','v-folder',1)", []).unwrap();
        db.execute(
            "INSERT INTO remote_aliases(remote,node) VALUES('900','folder')",
            [],
        )
        .unwrap();
        db.execute("UPDATE nodes SET conflict=1 WHERE id='folder'", [])
            .unwrap();
        db.execute(
            "INSERT INTO namespace_ops(id,node,body,state) VALUES('op-pending','folder','{}','pending')",
            [],
        )
        .unwrap();
    }
    s.install_remote_snapshot(&bound_folder_snapshot(), "2", "server")
        .unwrap();
    assert_eq!(
        s.status("folder").unwrap(),
        Status::Conflict,
        "an unresolved namespace operation must keep the flag"
    );

    let (_d2, mut files) = fixture();
    conflict(&mut files);
    assert_eq!(files.status("a").unwrap(), Status::Conflict);
    files
        .install_remote_snapshot(
            &[node("root", None, "r"), node("a", Some("root"), "v2")],
            "3",
            "server",
        )
        .unwrap();
    assert_eq!(
        files.status("a").unwrap(),
        Status::Conflict,
        "dirty file conflicts must not be healed"
    );
}

/// issue #54: a bound directory tombstoned by the server while local-only
/// children survive must be visible in the issue center with keep-local /
/// keep-remote flags instead of disappearing behind the stale-directory guard.
fn bound_directory_with_local_child() -> (tempfile::TempDir, Store, String, String) {
    let d = tempfile::tempdir().unwrap();
    let mut s = Store::open(d.path()).unwrap();
    s.install_remote_snapshot(&[node("root", None, "r")], "1", "server")
        .unwrap();
    s.ensure_fs_root("root").unwrap();
    // A locally created directory becomes bound to its confirmed remote identity.
    let directory = s.create_local_directory("root", "folder", 0o755).unwrap();
    let queued = s.next_directory().unwrap().unwrap();
    let job = s.mark_directory_started(&queued.job.operation).unwrap();
    let mut confirmed = job.node.clone();
    confirmed.version = "v-folder".into();
    s.confirm_directory(&queued.job.operation, &confirmed, "900")
        .unwrap();
    let child = s.create_local_file(&directory.id, "new", 0o600).unwrap();
    s.save_local(&child.id, b"local creation").unwrap();
    s.creation_saved(&child.id).unwrap();
    // The server stops reporting the bound folder; the local-only child keeps
    // the tombstone reachable only through the issue service.
    s.install_remote_snapshot(&[node("root", None, "r")], "2", "server")
        .unwrap();
    (d, s, directory.id, child.id)
}

#[test]
fn bound_deleted_directory_with_local_content_is_a_recoverable_issue() {
    let (_d, s, folder, child) = bound_directory_with_local_child();
    assert_eq!(s.status(&folder).unwrap(), Status::Conflict);
    let issue = s
        .sync_issue("root", &folder)
        .unwrap()
        .expect("bound deleted directory with surviving local content must be an issue");
    assert_eq!(issue.reason, "bound_directory_conflict");
    assert!(issue.can_keep_local);
    assert!(!issue.can_keep_remote);
    assert!(!issue.can_keep_both);
    assert!(!issue.can_keep_directory);
    assert_eq!(issue.path, "folder");
    assert!(issue.directory);
    let json = serde_json::to_value(&issue).unwrap();
    assert_eq!(json["can_keep_local"], serde_json::json!(true));
    assert_eq!(json["can_keep_remote"], serde_json::json!(false));
    let page = s.sync_issues("root", "").unwrap();
    assert_eq!(page.entries.len(), 1);
    assert_eq!(page.entries[0].id, folder);
    assert_eq!(s.sync_issue_count("root").unwrap(), 1);
    assert!(s.node(&child).is_ok());
}

#[test]
fn keep_directory_local_rebinds_the_tree_and_survives_restart() {
    let (d, mut s, folder, child) = bound_directory_with_local_child();
    let issue = s.sync_issue("root", &folder).unwrap().unwrap();
    let revived = s
        .keep_directory_local("root", &folder, &issue.token)
        .unwrap();
    assert_eq!(revived.id, folder);
    assert_eq!(revived.version, creations::CREATE_VERSION);
    assert_eq!(
        s.node(&folder).unwrap().version,
        creations::CREATE_VERSION,
        "keep-local must detach the old remote identity and requeue a local creation"
    );
    assert!(s.sync_issue("root", &folder).unwrap().is_none());
    let kept_child = s.node(&child).unwrap();
    assert_eq!(kept_child.parent.as_deref(), Some(folder.as_str()));
    let queued = s.next_directory().unwrap().unwrap();
    assert_eq!(queued.state, "pending");
    assert_eq!(queued.job.node.id, folder);
    assert_eq!(queued.job.parent.id, "root");
    drop(queued);
    drop(s);

    let mut s = Store::open(d.path()).unwrap();
    assert_eq!(
        s.node(&folder).unwrap().version,
        creations::CREATE_VERSION,
        "restart must keep the requeued local directory identity"
    );
    assert!(s.node(&child).is_ok());
    let queued = s.next_directory().unwrap().unwrap();
    let job = s.mark_directory_started(&queued.job.operation).unwrap();
    let mut confirmed = job.node.clone();
    confirmed.version = "v-folder2".into();
    s.confirm_directory(&queued.job.operation, &confirmed, "901")
        .unwrap();
    assert_eq!(s.node(&folder).unwrap().version, "v-folder2");
    assert!(s.sync_issue("root", &folder).unwrap().is_none());
}

#[test]
fn keep_directory_remote_clears_the_flag_without_resurrecting_local_content() {
    let (d, mut s, folder, child) = bound_directory_with_local_child();
    // Simulate a surviving child that already carries a remote identity instead
    // of being a local-only creation, so accepting the remote deletion loses no
    // local-only content.
    {
        let db = rusqlite::Connection::open(d.path().join("state.sqlite")).unwrap();
        db.execute("DELETE FROM local_creations WHERE node=?1", [&child])
            .unwrap();
        db.execute(
            "INSERT INTO remote_aliases(remote,node) VALUES('901',?1)",
            [&child],
        )
        .unwrap();
    }
    let issue = s
        .sync_issue("root", &folder)
        .unwrap()
        .expect("bound deleted directory must be recoverable even when content is remote-backed");
    assert_eq!(issue.reason, "bound_directory_conflict");
    assert!(issue.can_keep_local);
    assert!(issue.can_keep_remote);
    s.keep_directory_remote("root", &folder, &issue.token)
        .unwrap();
    assert!(s.sync_issue("root", &folder).unwrap().is_none());
    assert_eq!(s.status(&folder).unwrap(), Status::Deleted);
    assert!(s.node(&child).is_ok());
}

#[test]
fn keep_directory_remote_refuses_when_local_only_content_would_be_lost() {
    let (_d, mut s, folder, child) = bound_directory_with_local_child();
    let issue = s.sync_issue("root", &folder).unwrap().unwrap();
    assert!(!issue.can_keep_remote);
    assert!(s
        .keep_directory_remote("root", &folder, &issue.token)
        .is_err());
    assert!(s.sync_issue("root", &folder).unwrap().is_some());
    assert!(s.node(&child).is_ok());
}

#[test]
fn keep_directory_local_rejects_a_token_after_the_remote_identity_returns() {
    let (_d, mut s, folder, child) = bound_directory_with_local_child();
    let issue = s.sync_issue("root", &folder).unwrap().unwrap();
    s.install_remote_snapshot(
        &[
            node("root", None, "r"),
            named(&folder, Some("root"), "folder", "v-folder2", true),
        ],
        "3",
        "server",
    )
    .unwrap();
    assert!(s.sync_issue("root", &folder).unwrap().is_none());
    assert!(s
        .keep_directory_local("root", &folder, &issue.token)
        .is_err());
    assert_eq!(s.node(&folder).unwrap().version, "v-folder2");
    assert!(s.node(&child).is_ok());
}

#[test]
fn keep_directory_local_refuses_while_a_child_upload_is_uncertain() {
    let d = tempfile::tempdir().unwrap();
    let mut s = Store::open(d.path()).unwrap();
    s.install_remote_snapshot(&[node("root", None, "r")], "1", "server")
        .unwrap();
    s.ensure_fs_root("root").unwrap();
    let directory = s.create_local_directory("root", "folder", 0o755).unwrap();
    let queued = s.next_directory().unwrap().unwrap();
    let job = s.mark_directory_started(&queued.job.operation).unwrap();
    let mut confirmed = job.node.clone();
    confirmed.version = "v-folder".into();
    s.confirm_directory(&queued.job.operation, &confirmed, "900")
        .unwrap();
    let child = s.create_local_file(&directory.id, "new", 0o600).unwrap();
    let operation = s.save_local(&child.id, b"pending child").unwrap();
    s.creation_saved(&child.id).unwrap();
    s.mark_upload_started(&operation).unwrap();
    s.mark_upload_uncertain(&operation).unwrap();
    s.install_remote_snapshot(&[node("root", None, "r")], "2", "server")
        .unwrap();
    let issue = s
        .sync_issue("root", &directory.id)
        .unwrap()
        .expect("bound deleted directory must stay visible while a child commit is unresolved");
    assert!(issue.can_keep_local);
    assert!(
        s.keep_directory_local("root", &directory.id, &issue.token)
            .is_err(),
        "an unresolved child upload must block requeueing the directory"
    );
    assert!(s.node(&child.id).is_ok());
    assert_eq!(s.status(&directory.id).unwrap(), Status::Conflict);
}

/// The bound-directory recovery entry must not resurrect the invisible
/// tombstone problem for directories that have no surviving local content.
#[test]
fn bound_deleted_directory_without_surviving_content_stays_hidden() {
    let (_d, mut s) = fixture();
    let directory = s.create_local_directory("root", "folder", 0o755).unwrap();
    let queued = s.next_directory().unwrap().unwrap();
    let job = s.mark_directory_started(&queued.job.operation).unwrap();
    let mut confirmed = job.node.clone();
    confirmed.version = "v-folder".into();
    s.confirm_directory(&queued.job.operation, &confirmed, "900")
        .unwrap();
    s.install_remote_snapshot(&[node("root", None, "r")], "3", "server")
        .unwrap();
    assert_eq!(s.status(&directory.id).unwrap(), Status::Deleted);
    assert!(s.sync_issue("root", &directory.id).unwrap().is_none());
    assert_eq!(s.sync_issue_count("root").unwrap(), 0);
}
