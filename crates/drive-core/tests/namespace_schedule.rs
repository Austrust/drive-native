use drive_core::{namespace_schedule::NamespaceRecoveryObservation as Observation, *};
use std::io::Read;

fn n(id: &str, parent: Option<&str>, name: &str, directory: bool) -> Node {
    Node {
        id: id.into(),
        parent: parent.map(str::to_owned),
        name: name.into(),
        version: format!("v-{id}"),
        directory,
        metadata: Some(NodeMetadata {
            size: if directory { 0 } else { 3 },
            mtime: 1,
            mode: if directory { 0o755 } else { 0o644 },
        }),
    }
}
fn fixture() -> (tempfile::TempDir, Store) {
    let d = tempfile::tempdir().unwrap();
    let mut s = Store::open(d.path()).unwrap();
    s.install_remote_snapshot(
        &[
            n("1", None, "root", true),
            n("2", Some("1"), "A", false),
            n("3", Some("1"), "B", false),
        ],
        "1",
        "server",
    )
    .unwrap();
    s.ensure_fs_root("1").unwrap();
    s.publish_download(
        &DownloadTicket {
            id: "2".into(),
            version: "v-2".into(),
        },
        b"old",
        &sha256(b"old"),
    )
    .unwrap();
    (d, s)
}
fn uncertain(s: &mut Store) -> (Node, String) {
    let a = s.create_local_directory("1", ".Trash-1000", 0o700).unwrap();
    let op = s.next_directory().unwrap().unwrap().job.operation;
    s.mark_directory_started(&op).unwrap();
    s.prepare_directory(
        &op,
        drive_core::directories::DirectoryOrigin {
            cursor: "native-v1:12".into(),
            committer: "a".repeat(64),
            restore: "server".into(),
        },
    )
    .unwrap();
    s.mark_directory_started(&op).unwrap();
    s.mark_directory_uncertain(&op).unwrap();
    (a, op)
}
fn defer(s: &mut Store, op: &str) {
    assert!(
        s.observe_namespace_unresolved(op, Observation::CommitUnconfirmed)
            .unwrap()
    );
}

#[test]
fn unknown_mkdir_keeps_children_and_drafts_but_independent_shared_parent_rename_runs() {
    let (d, mut s) = fixture();
    let (dir, op) = uncertain(&mut s);
    let child = s.create_local_directory(&dir.id, "files", 0o700).unwrap();
    let file = s.create_local_file(&child.id, "draft", 0o600).unwrap();
    s.save_local(&file.id, b"unuploaded draft").unwrap();
    s.creation_saved(&file.id).unwrap();
    s.rename_local("1", "A", "1", "renamed", false).unwrap();
    assert_eq!(s.next_namespace_operation().unwrap().unwrap().operation, op);
    defer(&mut s, &op);
    let rename = s.next_rename().unwrap().unwrap();
    assert_eq!(rename.job.node.id, "2");
    s.mark_rename_started(&rename.job.operation).unwrap();
    assert_eq!(
        s.next_namespace_operation().unwrap().unwrap().operation,
        rename.job.operation
    );
    let mut confirmed = rename.job.node;
    confirmed.name = "renamed".into();
    confirmed.version = "v-2-moved".into();
    s.confirm_rename(&rename.job.operation, &confirmed, true)
        .unwrap();
    assert!(s.next_namespace_operation().unwrap().is_none());
    let h = s.queue_health().unwrap();
    assert_eq!(h.namespace_uncertain, 1);
    assert_eq!(h.namespace_unfinished, 2);
    assert_eq!(h.upload_unfinished, 1);
    assert!(h.schedule.unwrap().dependency_blocked >= 1);
    let mut bytes = Vec::new();
    s.open_cached(&file.id)
        .unwrap()
        .read_to_end(&mut bytes)
        .unwrap();
    assert_eq!(bytes, b"unuploaded draft");
    drop(s);
    let mut s = Store::open(d.path()).unwrap();
    let recovered = s.next_directory().unwrap().unwrap();
    assert_eq!(recovered.state, "uncertain");
    assert!(recovered.job.origin.is_some());
    assert!(s.mark_directory_started(&op).is_err());
    assert_eq!(s.queue_health().unwrap().recovery_attempts_observed, 0);
    assert_eq!(s.record(&file.id).unwrap().size, 16);
    defer(&mut s, &op);
    assert!(s.next_namespace_operation().unwrap().is_none());
}

#[test]
fn same_identity_ancestor_and_destination_name_depend_on_prior_intent() {
    for mode in 0..3 {
        let (_d, mut s) = fixture();
        let (dir, op) = uncertain(&mut s);
        match mode {
            0 => {
                s.rename_local("1", ".Trash-1000", "1", "trash-renamed", false)
                    .unwrap();
            }
            1 => {
                s.rename_local("1", "A", &dir.id, "inside", false).unwrap();
            }
            _ => {
                s.rename_local("1", ".Trash-1000", "1", "elsewhere", false)
                    .unwrap();
                s.rename_local("1", "A", "1", ".Trash-1000", false).unwrap();
            }
        }
        defer(&mut s, &op);
        assert!(
            s.next_namespace_operation().unwrap().is_none(),
            "case {mode}"
        );
        assert!(
            s.queue_health()
                .unwrap()
                .schedule
                .unwrap()
                .dependency_blocked
                > 0
        );
    }
}

#[test]
fn conflict_is_a_dependency_barrier_but_not_a_global_head_of_line_block() {
    let (_d, mut s) = fixture();
    let (dir, op) = uncertain(&mut s);
    s.conflict_directory(&op).unwrap();
    s.create_local_directory(&dir.id, "blocked", 0o700).unwrap();
    s.rename_local("1", "A", "1", "C", false).unwrap();
    assert_eq!(
        s.next_namespace_operation().unwrap().unwrap().kind,
        "rename"
    );
    assert_eq!(s.queue_health().unwrap().namespace_uncertain, 0);
}

#[test]
fn permission_gated_parent_keeps_its_subtree_blocked_but_not_other_root_children() {
    let (d, mut s) = fixture();
    let (dir, _op) = uncertain(&mut s);
    s.create_local_directory(&dir.id, "blocked", 0o700).unwrap();
    s.rename_local("1", "A", "1", "C", false).unwrap();
    let db = rusqlite::Connection::open(d.path().join("state.sqlite")).unwrap();
    db.execute("INSERT INTO access_denied(node) VALUES(?1)", [&dir.id])
        .unwrap();
    assert_eq!(
        s.next_namespace_operation().unwrap().unwrap().kind,
        "rename"
    );
    let h = s.queue_health().unwrap();
    assert_eq!(h.namespace_uncertain, 1);
    assert_eq!(h.schedule.unwrap().permission_blocked, 2);
}

#[test]
fn malformed_and_unrecognized_intents_fail_closed_without_hiding_exact_counts() {
    for mode in 0..3 {
        let (d, mut s) = fixture();
        let (_dir, op) = uncertain(&mut s);
        s.rename_local("1", "A", "1", "C", false).unwrap();
        let db = rusqlite::Connection::open(d.path().join("state.sqlite")).unwrap();
        match mode {
            0 => {
                db.execute("UPDATE namespace_ops SET body='{}' WHERE id=?1", [&op])
                    .unwrap();
            }
            1 => {
                db.execute(
                    "UPDATE namespace_ops SET state='unexpected' WHERE id=?1",
                    [&op],
                )
                .unwrap();
            }
            _ => {
                db.execute(
                    "UPDATE namespace_ops SET body=?2 WHERE id=?1",
                    rusqlite::params![op, "x".repeat(1_048_577)],
                )
                .unwrap();
            }
        }
        assert!(s.next_namespace_operation().unwrap().is_none());
        let h = s.queue_health().unwrap();
        assert_eq!(h.namespace_unfinished, 2);
        assert!(h.schedule.unwrap().unresolved_barrier);
    }
}

#[test]
fn inflight_outside_budget_owns_the_serial_lane_and_truncation_is_explicit() {
    let (d, mut s) = fixture();
    let (_dir, op) = uncertain(&mut s);
    let db = rusqlite::Connection::open(d.path().join("state.sqlite")).unwrap();
    let body: String = db
        .query_row("SELECT body FROM namespace_ops WHERE id=?1", [&op], |r| {
            r.get(0)
        })
        .unwrap();
    for i in 0..257 {
        let id = format!("fixture-{i}");
        let mut value: serde_json::Value = serde_json::from_str(&body).unwrap();
        value["operation"] = id.clone().into();
        db.execute("INSERT INTO namespace_ops(id,node,body,state,kind) VALUES(?1,?2,?3,'conflict','mkdir')",rusqlite::params![id,value["node"]["id"].as_str().unwrap(),value.to_string()]).unwrap();
    }
    defer(&mut s, &op);
    assert!(s.next_namespace_operation().unwrap().is_none());
    assert!(s.queue_health().unwrap().schedule.unwrap().truncated);
    db.execute(
        "UPDATE namespace_ops SET state='inflight' WHERE id='fixture-256'",
        [],
    )
    .unwrap();
    assert_eq!(
        s.next_namespace_operation().unwrap().unwrap().operation,
        "fixture-256"
    );
    assert!(s.mark_directory_started(&op).is_err());
}
