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
    // #57 pending-first: the independent rename is selected even though the
    // mkdir reconcile is due; the dependent child still cannot pass it.
    let rename = s.scheduled_rename().unwrap().unwrap();
    assert_eq!(
        s.next_namespace_operation().unwrap().unwrap().operation,
        rename.job.operation
    );
    assert_ne!(rename.job.operation, op);
    defer(&mut s, &op);
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
    // Inspection retains the legacy conflict head even when independent work
    // is executable. Reading that record is not permission to send it.
    assert_eq!(s.next_namespace_kind().unwrap().as_deref(), Some("mkdir"));
    assert_eq!(s.next_directory().unwrap().unwrap().state, "conflict");
    assert!(s.next_rename().unwrap().is_none());
    assert!(s.mark_directory_started(&op).is_err());
    assert!(s.scheduled_directory().unwrap().is_none());
    let rename = s.scheduled_rename().unwrap().unwrap();
    assert_eq!(
        s.next_namespace_operation().unwrap().unwrap().kind,
        "rename"
    );
    assert_eq!(s.queue_health().unwrap().namespace_uncertain, 0);
    s.mark_rename_started(&rename.job.operation).unwrap();
    assert_eq!(s.scheduled_rename().unwrap().unwrap().state, "inflight");
}

#[test]
fn deferred_inspection_cannot_bypass_executable_mutation_gate() {
    let (_d, mut s) = fixture();
    let (_dir, op) = uncertain(&mut s);
    defer(&mut s, &op);
    assert_eq!(s.next_directory().unwrap().unwrap().job.operation, op);
    assert_eq!(s.next_directory().unwrap().unwrap().state, "uncertain");
    assert!(s.scheduled_directory().unwrap().is_none());
    assert!(s.next_namespace_operation().unwrap().is_none());
    assert!(s.mark_directory_started(&op).is_err());
    assert!(
        s.prepare_directory(
            &op,
            drive_core::directories::DirectoryOrigin {
                cursor: "native-v1:12".into(),
                committer: "b".repeat(64),
                restore: "server".into()
            }
        )
        .is_err()
    );
    assert_eq!(
        s.next_directory()
            .unwrap()
            .unwrap()
            .job
            .origin
            .unwrap()
            .committer,
        "a".repeat(64)
    );
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
                    rusqlite::params![op, serde_json::to_string(&"x".repeat(1_048_577)).unwrap()],
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

// ---- #57: namespace dispatch starvation (P0) -------------------------------

const SYNTH_ROOT: &str = "1";

/// Build a store whose remote snapshot has `count` independent root files, then
/// inject synthetic remove journal rows (the field shape: a low-seq `uncertain`
/// block ahead of a large high-seq `pending` tail).
fn synthetic_remove_store(count: usize) -> (tempfile::TempDir, Store) {
    let d = tempfile::tempdir().unwrap();
    let mut s = Store::open(d.path()).unwrap();
    let mut nodes = vec![n("1", None, "root", true)];
    for i in 0..count {
        nodes.push(n(
            &format!("f{i}"),
            Some(SYNTH_ROOT),
            &format!("f{i}"),
            false,
        ));
    }
    s.install_remote_snapshot(&nodes, "1", "server").unwrap();
    s.ensure_fs_root(SYNTH_ROOT).unwrap();
    (d, s)
}

fn insert_removes(
    d: &tempfile::TempDir,
    first: usize,
    count: usize,
    state: &str,
    prefix: &str,
) -> Vec<String> {
    let db = rusqlite::Connection::open(d.path().join("state.sqlite")).unwrap();
    let mut ids = Vec::new();
    for i in first..first + count {
        let id = format!("{prefix}-{i}");
        let node = n(&format!("f{i}"), Some(SYNTH_ROOT), &format!("f{i}"), false);
        let job = drive_core::removals::RemoveJob {
            operation: id.clone(),
            node: node.clone(),
            path: format!("f{i}"),
            cursor: "synthetic".into(),
        };
        db.execute(
            "INSERT INTO namespace_ops(id,node,body,state,kind) VALUES(?1,?2,?3,?4,'remove')",
            rusqlite::params![id, node.id, serde_json::to_string(&job).unwrap(), state],
        )
        .unwrap();
        db.execute(
            "INSERT OR REPLACE INTO local_unlinks(node,confirmed) VALUES(?1,0)",
            [&node.id],
        )
        .unwrap();
        ids.push(id);
    }
    ids
}

fn journal_attempts(d: &tempfile::TempDir, id: &str) -> (i64, i64) {
    let db = rusqlite::Connection::open(d.path().join("state.sqlite")).unwrap();
    db.query_row(
        "SELECT attempts,updated_at FROM namespace_ops WHERE id=?1",
        [id],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )
    .unwrap()
}

fn journal_commit(d: &tempfile::TempDir, id: &str) {
    let db = rusqlite::Connection::open(d.path().join("state.sqlite")).unwrap();
    db.execute(
        "UPDATE namespace_ops SET state='committed' WHERE id=?1",
        [id],
    )
    .unwrap();
}

/// Acceptance criterion: 55 due `uncertain` removes whose seq precedes a 100-row
/// `pending` tail must not monopolize the single dispatch lane. Before #57 each
/// dispatch round selects the low-seq reconcile head, so every assertion below
/// observes `uncertain` instead of a pending attempt.
#[test]
fn due_uncertain_block_does_not_starve_the_pending_remove_tail() {
    let (d, mut s) = synthetic_remove_store(160);
    let _uncertain = insert_removes(&d, 0, 55, "uncertain", "u");
    let pending = insert_removes(&d, 55, 100, "pending", "p");
    let mut attempted = Vec::new();
    for round in 0..10 {
        let selected = s
            .next_namespace_operation()
            .unwrap()
            .unwrap_or_else(|| panic!("round {round}: no namespace candidate"));
        assert_eq!(
            selected.state, "pending",
            "round {round}: reconcile candidate {} starved the pending tail",
            selected.operation
        );
        assert!(pending.contains(&selected.operation));
        // The runtime always marks a selected submission started; the journal
        // must show evidence of the attempt instead of attempts=0/updated_at=0.
        s.mark_remove_started(&selected.operation).unwrap();
        let (_attempts, updated_at) = journal_attempts(&d, &selected.operation);
        assert!(
            updated_at > 0,
            "round {round}: dispatch never stamped updated_at"
        );
        journal_commit(&d, &selected.operation);
        attempted.push(selected.operation);
    }
    attempted.sort();
    attempted.dedup();
    assert_eq!(
        attempted.len(),
        10,
        "fewer than 10 distinct pending attempts"
    );
    let h = s.queue_health().unwrap();
    assert_eq!(h.namespace_uncertain, 55);
    assert_eq!(h.namespace_unfinished, 55 + 100 - 10);
}

/// Acceptance criterion: replaying the frozen field shape (conflicts + a mid-seq
/// uncertain block + a large pending tail) after a restart must advance the tail.
#[test]
fn frozen_field_shape_advances_the_pending_tail_after_restart() {
    let (d, s) = synthetic_remove_store(160);
    insert_removes(&d, 0, 4, "conflict", "c");
    insert_removes(&d, 4, 55, "uncertain", "u");
    insert_removes(&d, 59, 100, "pending", "p");
    drop(s);
    let mut s = Store::open(d.path()).unwrap();
    for round in 0..10 {
        let selected = s
            .next_namespace_operation()
            .unwrap()
            .unwrap_or_else(|| panic!("round {round}: no namespace candidate"));
        assert_eq!(
            selected.state, "pending",
            "round {round}: post-restart dispatch selected {} instead of the pending tail",
            selected.state
        );
        s.mark_remove_started(&selected.operation).unwrap();
        journal_commit(&d, &selected.operation);
    }
    let h = s.queue_health().unwrap();
    let schedule = h.schedule.unwrap();
    assert!(schedule.selected);
    assert!(!schedule.unresolved_barrier);
    assert_eq!(schedule.permission_blocked, 0);
    assert_eq!(h.namespace_uncertain, 55);
}

/// Acceptance criterion: a slow (up to 120 s) read-only reconcile must not
/// head-of-line block later candidates. At the selector level the reconcile is
/// represented by the armed 2 s observation: the next dispatch cycle must pick
/// the pending candidate, not re-own the lane with the deferred reconcile.
#[test]
fn slow_read_only_reconcile_does_not_head_of_line_block_pending_work() {
    let (d, mut s) = synthetic_remove_store(8);
    insert_removes(&d, 0, 1, "uncertain", "u");
    let first = s.next_namespace_operation().unwrap().unwrap();
    assert_eq!(first.state, "uncertain");
    // The 120 s reconcile returned "cannot prove": arm the backoff.
    assert!(
        s.observe_namespace_unresolved(&first.operation, Observation::CommitUnconfirmed)
            .unwrap()
    );
    // Pending remove appears while the reconcile lane is still blocked.
    insert_removes(&d, 1, 1, "pending", "p");
    let second = s.next_namespace_operation().unwrap().unwrap();
    assert_eq!(
        second.state, "pending",
        "the deferred reconcile re-owned the dispatch lane"
    );
    assert!(second.operation.starts_with("p-"));
    s.mark_remove_started(&second.operation).unwrap();
}

/// Acceptance criterion: `namespace.pending_starved` is emitted after 3
/// consecutive dispatch rounds without a pending attempt and surfaced
/// additively through `queue_health`/`GetOverview`.
#[test]
fn pending_starvation_is_observable_after_three_rounds_without_a_pending_attempt() {
    let (_d, mut s) = fixture();
    let (dir, op) = uncertain(&mut s);
    // The only pending mutation depends on the unresolved intent: ordering is
    // correct, but the scheduler must still report that pending never ran.
    s.create_local_directory(&dir.id, "blocked", 0o700).unwrap();
    let first = s.dispatch_namespace_operation().unwrap().unwrap();
    assert_eq!(first.operation, op);
    assert_eq!(first.state, "uncertain");
    defer(&mut s, &op);
    assert!(s.dispatch_namespace_operation().unwrap().is_none());
    s.request_namespace_recheck();
    assert!(s.dispatch_namespace_operation().unwrap().is_some());
    let h = s.queue_health().unwrap();
    assert_eq!(h.consecutive_pending_starved_rounds, 3);
    assert!(h.pending_starved);
    assert_eq!(h.pending_starved_events, 1);
    assert_eq!(
        h.last_starvation_event.as_deref(),
        Some("namespace.pending_starved")
    );
    assert!(h.pending_last_attempt_age_ms.is_none());
    let snapshot = h.schedule.unwrap();
    assert_eq!(snapshot.selected_state.as_deref(), Some("uncertain"));
    assert_eq!(snapshot.eligible_pending, 0);
}

/// Acceptance criterion: `pending_last_attempt_age_ms` is exposed by the
/// dispatch accounting (the field the issue asks to add additively).
#[test]
fn dispatch_rounds_report_the_last_pending_attempt_age() {
    let (d, mut s) = synthetic_remove_store(8);
    insert_removes(&d, 0, 1, "uncertain", "u");
    insert_removes(&d, 1, 3, "pending", "p");
    assert!(
        s.queue_health()
            .unwrap()
            .pending_last_attempt_age_ms
            .is_none()
    );
    let selected = s.dispatch_namespace_operation().unwrap().unwrap();
    assert_eq!(selected.state, "pending");
    let h = s.queue_health().unwrap();
    let age = h.pending_last_attempt_age_ms.expect("pending attempt age");
    assert!(age < 300_000, "pending_last_attempt_age_ms={age}");
    assert!(!h.pending_starved);
    assert_eq!(h.consecutive_pending_starved_rounds, 0);
    assert_eq!(
        h.schedule.unwrap().selected_state.as_deref(),
        Some("pending")
    );
}
