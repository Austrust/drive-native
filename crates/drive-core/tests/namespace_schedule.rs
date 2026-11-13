use drive_core::{namespace_schedule::NamespaceRecoveryObservation as Observation, *};
use std::collections::HashMap;
use std::io::Read;
use std::time::{Duration, Instant};

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

// ---- #57 scale regression: frozen incident shape (3,114 pending removes) ---
//
// This is a scale/timing regression, deliberately `#[ignore]`d so it does not
// run with the default suite. Run it explicitly on the remote build host:
//
//   cargo test -p drive-core --release --all-features --locked \
//       --test namespace_schedule -- --ignored --nocapture frozen_incident_shape
//
// Thresholds are calibrated from the release measurement recorded in
// notes/work/issue-57.md (workstation202611, target i57, 2026-11-11) and kept
// loose enough to catch gross dispatch regressions rather than machine load
// jitter:
//   - frozen-shape construction (fixture + one INSERT transaction): < 2,000 ms
//     (measured 143 ms)
//   - per-dispatch-round p95, including mark_remove_started + confirm_remove,
//     which is what the single upload/commit lane pays per attempt: < 60,000 us
//     (measured 20.1 ms; p50 19.1 ms, max 24.1 ms)
//   - selector-only scan p95 at the 3,173-row shape: < 30,000 us
//     (measured 5.9 ms; the scan stops after the 4 conflict + 55 uncertain
//     barriers and the first pending candidate)
// Every run prints the measured values with the `#57 SCALE` prefix.

const INCIDENT_CONFLICTS: usize = 4;
const INCIDENT_UNCERTAIN: usize = 55;
const INCIDENT_PENDING: usize = 3_114;
const INCIDENT_SCAN_SAMPLES: usize = 50;
const INCIDENT_WARMUP_ROUNDS: usize = 10;
const INCIDENT_MEASURE_ROUNDS: usize = 200;
/// A dispatch round must stay far below the 120 s single-RPC budget. The
/// release measurement is ~20 ms per round (dominated by two selector scans of
/// the 59 unresolved barriers); 60 ms is ~3x headroom so an O(queue)-per-round
/// regression is still caught without tripping on a loaded build host.
const INCIDENT_ROUND_P95_BUDGET_US: u128 = 60_000;
/// The selector-only scan at the frozen shape measures ~5.9 ms; 30 ms gives
/// ~5x headroom while still failing loudly if the scan becomes linear in the
/// 3,114-row pending tail.
const INCIDENT_SCAN_P95_BUDGET_US: u128 = 30_000;

/// Bulk variant of [`insert_removes`] for the frozen shape: all rows go in one
/// transaction. The incident has 3,114+ journal rows; one autocommit per row
/// would dominate fixture construction instead of measuring dispatch.
fn insert_removes_bulk(
    d: &tempfile::TempDir,
    first: usize,
    count: usize,
    state: &str,
    prefix: &str,
) -> Vec<(String, String)> {
    let mut db = rusqlite::Connection::open(d.path().join("state.sqlite")).unwrap();
    let tx = db.transaction().unwrap();
    let mut ids = Vec::with_capacity(count);
    {
        let mut insert = tx
            .prepare(
                "INSERT INTO namespace_ops(id,node,body,state,kind) VALUES(?1,?2,?3,?4,'remove')",
            )
            .unwrap();
        let mut unlink = tx
            .prepare("INSERT OR REPLACE INTO local_unlinks(node,confirmed) VALUES(?1,0)")
            .unwrap();
        for i in first..first + count {
            let id = format!("{prefix}-{i}");
            let node = n(&format!("f{i}"), Some(SYNTH_ROOT), &format!("f{i}"), false);
            let job = drive_core::removals::RemoveJob {
                operation: id.clone(),
                node: node.clone(),
                path: format!("f{i}"),
                cursor: "synthetic".into(),
            };
            insert
                .execute(rusqlite::params![
                    id,
                    node.id,
                    serde_json::to_string(&job).unwrap(),
                    state
                ])
                .unwrap();
            unlink.execute([&node.id]).unwrap();
            ids.push((id, node.id));
        }
    }
    tx.commit().unwrap();
    ids
}

fn percentile(samples: &[Duration], percentile: f64) -> Duration {
    assert!(!samples.is_empty());
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = ((sorted.len() - 1) as f64 * percentile / 100.0).round() as usize;
    sorted[rank]
}

/// One real dispatch round for the frozen shape. A selected `pending` remove is
/// attempted and committed through the same `Store` API the worker uses
/// (`mark_remove_started` + `confirm_remove`, which also stamps the durable
/// attempts/updated_at evidence); a due `uncertain` reconcile is observed as
/// unprovable, exactly like a lost response.
fn dispatch_incident_round(
    s: &mut Store,
    pending_nodes: &HashMap<String, String>,
    attempted: &mut usize,
) -> drive_core::namespace_schedule::NamespaceOperation {
    let selected = s
        .dispatch_namespace_operation()
        .unwrap()
        .expect("frozen shape must always have a namespace candidate");
    match selected.state.as_str() {
        "pending" => {
            let node_id = pending_nodes
                .get(&selected.operation)
                .expect("pending remove must reference the synthetic node")
                .clone();
            s.mark_remove_started(&selected.operation).unwrap();
            s.confirm_remove(&selected.operation, &node_id).unwrap();
            *attempted += 1;
        }
        "uncertain" => {
            assert!(
                s.observe_namespace_unresolved(&selected.operation, Observation::CommitUnconfirmed)
                    .unwrap()
            );
        }
        state => panic!("unexpected selected state {state} in the frozen shape"),
    }
    selected
}

/// Frozen incident shape: 4 `conflict`, 55 low-seq due `uncertain` removes and
/// 3,114 high-seq `pending` removes (`attempts=0`, `updated_at=0`).
///
/// Before #57 every round selected the low-seq reconcile head, so none of the
/// 3,114 pending removes was ever attempted. After #57 the pending tail must be
/// dispatched at a stable per-round cost; this test replays the shape at real
/// scale and prints the timings needed for baseline/integration comparisons.
#[test]
#[ignore = "scale/timing regression for the frozen 3,114-row incident shape; run with --ignored --nocapture"]
fn frozen_incident_shape_dispatches_pending_tail_at_scale() {
    // --- construction: one remote snapshot transaction + one journal tx ---
    let construction_started = Instant::now();
    let (d, mut s) =
        synthetic_remove_store(INCIDENT_CONFLICTS + INCIDENT_UNCERTAIN + INCIDENT_PENDING);
    insert_removes_bulk(&d, 0, INCIDENT_CONFLICTS, "conflict", "c");
    let uncertain =
        insert_removes_bulk(&d, INCIDENT_CONFLICTS, INCIDENT_UNCERTAIN, "uncertain", "u");
    let pending = insert_removes_bulk(
        &d,
        INCIDENT_CONFLICTS + INCIDENT_UNCERTAIN,
        INCIDENT_PENDING,
        "pending",
        "p",
    );
    let construction = construction_started.elapsed();
    // Field evidence: every pending row is still attempts=0/updated_at=0 before
    // the first dispatch. Sample the head of the tail (all 3,114 stay untouched
    // until dispatched by the rounds below).
    for (operation, _) in pending.iter().take(16) {
        assert_eq!(
            journal_attempts(&d, operation),
            (0, 0),
            "pending {operation} must start un-attempted"
        );
    }

    // The uncertain block was retried on a fixed cadence with an unprovable
    // outcome. Arm the process-local observation, then make it due again so
    // every measured round faces the same reconcile pressure as the field.
    let arm_started = Instant::now();
    for (operation, _) in &uncertain {
        assert!(
            s.observe_namespace_unresolved(operation, Observation::CommitUnconfirmed)
                .unwrap()
        );
    }
    s.request_namespace_recheck();
    let arm = arm_started.elapsed();
    let node_count = INCIDENT_CONFLICTS + INCIDENT_UNCERTAIN + INCIDENT_PENDING + 1;
    println!(
        "#57 SCALE construction_ms={} arm_ms={} nodes={node_count} conflict={INCIDENT_CONFLICTS} uncertain={INCIDENT_UNCERTAIN} pending={INCIDENT_PENDING}",
        construction.as_millis(),
        arm.as_millis(),
    );
    assert!(
        construction.as_millis() < 2_000,
        "frozen shape construction took {} ms (target < 2000 ms)",
        construction.as_millis()
    );

    // --- selector-only scan cost at full incident size (no state changes) ---
    let mut scans = Vec::with_capacity(INCIDENT_SCAN_SAMPLES);
    for _ in 0..INCIDENT_SCAN_SAMPLES {
        let started = Instant::now();
        let selected = s
            .next_namespace_operation()
            .unwrap()
            .expect("pending candidate");
        scans.push(started.elapsed());
        assert_eq!(selected.state, "pending");
    }
    let scan_p50 = percentile(&scans, 50.0);
    let scan_p95 = percentile(&scans, 95.0);
    let scan_max = *scans.iter().max().unwrap();
    println!(
        "#57 SCALE selector_scan_p50_us={} p95_us={} max_us={} rows_scanned={}",
        scan_p50.as_micros(),
        scan_p95.as_micros(),
        scan_max.as_micros(),
        INCIDENT_CONFLICTS + INCIDENT_UNCERTAIN + 1,
    );

    // --- warmup, then timed dispatch rounds through the real Store API ---
    let pending_nodes: HashMap<String, String> = pending.iter().cloned().collect();
    let mut attempted = 0usize;
    for _ in 0..INCIDENT_WARMUP_ROUNDS {
        dispatch_incident_round(&mut s, &pending_nodes, &mut attempted);
    }
    assert_eq!(attempted, INCIDENT_WARMUP_ROUNDS);

    let measure_started = Instant::now();
    let mut rounds = Vec::with_capacity(INCIDENT_MEASURE_ROUNDS);
    let mut first_ten = Vec::new();
    let mut distinct_first_ten = std::collections::HashSet::new();
    for round in 0..INCIDENT_MEASURE_ROUNDS {
        let started = Instant::now();
        let selected = dispatch_incident_round(&mut s, &pending_nodes, &mut attempted);
        rounds.push(started.elapsed());
        assert_eq!(
            selected.state, "pending",
            "round {round}: reconcile candidate {} starved the pending tail",
            selected.operation
        );
        if round < 10 {
            distinct_first_ten.insert(selected.operation.clone());
            first_ten.push(selected.operation);
        }
    }
    let measure = measure_started.elapsed();
    let round_p50 = percentile(&rounds, 50.0);
    let round_p95 = percentile(&rounds, 95.0);
    let round_max = *rounds.iter().max().unwrap();

    // --- behavior assertions: attempts are real and starvation-free ---
    assert_eq!(
        distinct_first_ten.len(),
        10,
        "fewer than 10 distinct pending removes attempted in the first 10 rounds"
    );
    for operation in &first_ten {
        let (attempts, updated_at) = journal_attempts(&d, operation);
        assert!(
            attempts > 0,
            "pending {operation} was dispatched but attempts={attempts}"
        );
        assert!(
            updated_at > 0,
            "pending {operation} was dispatched but updated_at={updated_at}"
        );
    }
    let health = s.queue_health().unwrap();
    assert!(!health.pending_starved);
    assert_eq!(health.consecutive_pending_starved_rounds, 0);
    let age_ms = health
        .pending_last_attempt_age_ms
        .expect("pending attempts must be accounted");
    assert!(
        age_ms < 5_000,
        "pending_last_attempt_age_ms={age_ms} exceeds the 5 s bound"
    );
    assert_eq!(
        health.namespace_unfinished,
        (INCIDENT_CONFLICTS + INCIDENT_UNCERTAIN + INCIDENT_PENDING - attempted) as u64
    );

    let attempts_per_second = attempted as f64 / measure.as_secs_f64();
    println!(
        "#57 SCALE dispatch_rounds={} attempted_total={} measured_rounds={} elapsed_ms={:.1} round_p50_us={} round_p95_us={} round_max_us={} pending_attempts_per_sec={attempts_per_second:.0} pending_last_attempt_age_ms={age_ms}",
        INCIDENT_MEASURE_ROUNDS,
        attempted,
        INCIDENT_MEASURE_ROUNDS,
        measure.as_secs_f64() * 1_000.0,
        round_p50.as_micros(),
        round_p95.as_micros(),
        round_max.as_micros(),
    );
    println!(
        "#57 SCALE thresholds construction_ms<2000 round_p95_us<{INCIDENT_ROUND_P95_BUDGET_US} selector_scan_p95_us<{INCIDENT_SCAN_P95_BUDGET_US}"
    );
    assert!(
        round_p95.as_micros() < INCIDENT_ROUND_P95_BUDGET_US,
        "dispatch round p95 {} us exceeds {} us budget",
        round_p95.as_micros(),
        INCIDENT_ROUND_P95_BUDGET_US
    );
    assert!(
        scan_p95.as_micros() < INCIDENT_SCAN_P95_BUDGET_US,
        "selector scan p95 {} us exceeds {} us budget",
        scan_p95.as_micros(),
        INCIDENT_SCAN_P95_BUDGET_US
    );
}

// ---- #57-A: scan-window starvation (residual NV-1) -------------------------
//
// The fixed 256-row selector window used to stop scanning at the head of the
// journal: a prefix of >=256 permanently unfinished intents hid every later
// independent `pending` intent forever. These tests pin the bounded-pagination
// contract: within a declared finite number of rounds the suffix must receive a
// real attempt, and a genuinely dependent suffix must still never be skipped.

/// Prefix sizes around the historical window boundary plus the 1,024-row shape
/// named by the issue. 255 is the control: the suffix is inside the window and
/// already passed before the fix.
const WINDOW_PREFIXES: [usize; 4] = [255, 256, 257, 1024];
/// Declared bound for "the suffix is eventually attempted": two full passes over
/// a 1,024-row prefix at one bounded page per round plus headroom.
const WINDOW_ROUND_BUDGET: usize = 12;

fn window_starvation_store(
    prefix: usize,
) -> (
    tempfile::TempDir,
    Store,
    Vec<(String, String)>,
    Vec<(String, String)>,
) {
    let (d, mut s) = synthetic_remove_store(prefix + 100);
    let half = prefix / 2;
    let conflicts = insert_removes_bulk(&d, 0, half, "conflict", "c");
    let uncertain = insert_removes_bulk(&d, half, prefix - half, "uncertain", "u");
    let pending = insert_removes_bulk(&d, prefix, 100, "pending", "p");
    // A due read-only reconcile is the worst case for progress: it is always
    // executable, so a selector that cannot see past the window keeps picking
    // it and never even observes the independent pending suffix.
    for (operation, _) in &uncertain {
        assert!(s
            .observe_namespace_unresolved(operation, Observation::CommitUnconfirmed)
            .unwrap());
    }
    s.request_namespace_recheck();
    (d, s, conflicts, pending)
}

#[test]
fn pending_suffix_beyond_the_scan_window_is_eventually_attempted() {
    for prefix in WINDOW_PREFIXES {
        let (d, mut s, _conflicts, pending) = window_starvation_store(prefix);
        let suffix: std::collections::HashSet<String> =
            pending.iter().map(|(operation, _)| operation.clone()).collect();
        let mut attempted = std::collections::HashSet::new();
        for _round in 0..WINDOW_ROUND_BUDGET {
            let Some(selected) = s.next_namespace_operation().unwrap() else {
                // A selector that only sees the blocked prefix returns `None`;
                // an executable-pending fix must keep returning candidates.
                continue;
            };
            if selected.state == "pending" && suffix.contains(&selected.operation) {
                s.mark_remove_started(&selected.operation).unwrap();
                journal_commit(&d, &selected.operation);
                attempted.insert(selected.operation);
            } else if selected.state == "uncertain" {
                // Model the read-only reconcile that cannot prove the outcome.
                assert!(s
                    .observe_namespace_unresolved(
                        &selected.operation,
                        Observation::CommitUnconfirmed
                    )
                    .unwrap());
            }
        }
        assert!(
            !attempted.is_empty(),
            "prefix={prefix}: the 100-row independent pending suffix never got an attempt in \
             {WINDOW_ROUND_BUDGET} rounds (scan window starvation)"
        );
    }
}

#[test]
fn dependent_suffix_beyond_the_scan_window_is_never_skipped() {
    let (d, mut s, conflicts, _pending) = window_starvation_store(256);
    // Make every suffix intent depend on every prefix intent: same node and
    // same path as the first prefix conflict. No scheduler change may ever
    // dispatch across that barrier.
    let barrier_node = conflicts[0].1.clone();
    let db = rusqlite::Connection::open(d.path().join("state.sqlite")).unwrap();
    let mut ids = db
        .prepare("SELECT id FROM namespace_ops WHERE state='pending' ORDER BY seq")
        .unwrap()
        .query_map([], |r| r.get::<_, String>(0))
        .unwrap()
        .collect::<std::result::Result<Vec<_>, _>>()
        .unwrap();
    drop(db);
    ids.sort();
    let db = rusqlite::Connection::open(d.path().join("state.sqlite")).unwrap();
    for id in &ids {
        let job = drive_core::removals::RemoveJob {
            operation: id.clone(),
            node: n(&barrier_node, Some(SYNTH_ROOT), "f0", false),
            path: "f0".into(),
            cursor: "synthetic".into(),
        };
        db.execute(
            "UPDATE namespace_ops SET body=?2 WHERE id=?1",
            rusqlite::params![id, serde_json::to_string(&job).unwrap()],
        )
        .unwrap();
    }
    drop(db);
    let mut leaked = Vec::new();
    for _ in 0..WINDOW_ROUND_BUDGET {
        let Some(selected) = s.next_namespace_operation().unwrap() else {
            continue;
        };
        if selected.state == "pending" {
            leaked.push(selected.operation.clone());
            s.mark_remove_started(&selected.operation).unwrap();
            journal_commit(&d, &selected.operation);
        }
    }
    assert!(
        leaked.is_empty(),
        "a suffix intent dependent on an earlier blocked intent was dispatched across it: {leaked:?}"
    );
}
