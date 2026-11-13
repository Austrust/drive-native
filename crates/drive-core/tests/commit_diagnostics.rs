use drive_core::commit_diagnostics::CommitDiagnosticRecord;
use drive_core::download_jobs::DownloadFailureKind;
use drive_core::*;

fn n(id: &str, parent: Option<&str>, name: &str, version: &str, dir: bool) -> Node {
    Node {
        id: id.into(),
        parent: parent.map(str::to_owned),
        name: name.into(),
        version: version.into(),
        directory: dir,
        metadata: Some(NodeMetadata {
            size: if dir { 0 } else { 3 },
            mtime: 1,
            mode: 0o644,
        }),
    }
}

fn root() -> Node {
    n("1", None, "root", "root-v1", true)
}

fn file(version: &str) -> Node {
    n("2", Some("1"), "A.doc", version, false)
}

fn fixture() -> (tempfile::TempDir, Store) {
    let d = tempfile::tempdir().unwrap();
    let mut s = Store::open(d.path()).unwrap();
    s.install_remote_snapshot(&[root(), file("v1")], "1", "server")
        .unwrap();
    s.ensure_fs_root("1").unwrap();
    s.publish_download(
        &DownloadTicket {
            id: "2".into(),
            version: "v1".into(),
        },
        b"old",
        &sha256(b"old"),
    )
    .unwrap();
    (d, s)
}

fn diagnostics(s: &Store) -> Vec<CommitDiagnosticRecord> {
    commit_diagnostics::list(s, 200).unwrap()
}

fn operation_diagnostic(s: &Store) -> CommitDiagnosticRecord {
    diagnostics(s)
        .into_iter()
        .find(|d| d.scope == "operation")
        .expect("operation diagnostic")
}

fn started_upload(s: &mut Store, bytes: &[u8]) -> String {
    let op = s.save_local("2", bytes).unwrap();
    s.mark_upload_started(&op).unwrap();
    op
}

#[test]
fn unknown_upload_is_visible_with_category_error_and_retry_time() {
    let (_d, mut s) = fixture();
    let op = started_upload(&mut s, b"local edits");
    s.mark_upload_uncertain_diagnosed(
        &op,
        CommitCategory::Transport,
        Some("connection lost before reply"),
    )
    .unwrap();

    let d = operation_diagnostic(&s);
    assert_eq!(d.scope, "operation");
    assert_eq!(d.operation, op);
    assert_eq!(d.kind, "upload");
    assert_eq!(d.state, "uncertain");
    assert_eq!(d.outcome, CommitOutcome::UnknownRetryable);
    assert_eq!(d.category, CommitCategory::Transport);
    assert_eq!(
        d.last_error.as_deref(),
        Some("connection lost before reply")
    );
    assert_eq!(d.attempts, 0);
    assert_eq!(d.next_retry_at, 0);
    assert!(d.outcome.is_open());
    assert!(!d.outcome.needs_attention());

    assert_eq!(commit_diagnostics::open_unknown(&s).unwrap(), 1);
    assert_eq!(commit_diagnostics::needs_attention(&s).unwrap(), 0);
}

#[test]
fn reconcile_ladder_persists_next_retry_and_exhausts_to_fatal() {
    let (_d, mut s) = fixture();
    let op = started_upload(&mut s, b"local edits");
    s.mark_upload_uncertain(&op).unwrap();

    let now = 1_700_000_000;
    let delays = [10, 30, 120, 300];
    for (i, delay) in delays.iter().enumerate() {
        let attempts = s
            .note_upload_reconcile_attempt_diagnosed(
                &op,
                now,
                CommitCategory::PermissionUncertain,
                Some("server did not confirm the commit"),
            )
            .unwrap();
        assert_eq!(attempts as usize, i + 1);
        let d = operation_diagnostic(&s);
        assert_eq!(d.state, "uncertain");
        assert_eq!(d.outcome, CommitOutcome::UnknownRetryable);
        assert_eq!(d.category, CommitCategory::PermissionUncertain);
        assert_eq!(d.attempts as usize, i + 1);
        assert_eq!(d.next_retry_at, now + delay);
        assert_eq!(
            d.last_error.as_deref(),
            Some("server did not confirm the commit")
        );
    }

    let attempts = s
        .note_upload_reconcile_attempt_diagnosed(
            &op,
            now,
            CommitCategory::PermissionUncertain,
            Some("server did not confirm the commit"),
        )
        .unwrap();
    assert_eq!(attempts, 5);
    let d = operation_diagnostic(&s);
    assert_eq!(d.state, "abandoned");
    assert_eq!(d.outcome, CommitOutcome::UnknownFatal);
    assert_eq!(d.category, CommitCategory::ReconcileExhausted);
    assert_eq!(d.attempts, 5);
    assert_eq!(d.next_retry_at, 0);
    assert!(!d.outcome.is_open());

    assert_eq!(s.paused_upload_operations().unwrap(), 1);
    assert_eq!(commit_diagnostics::open_unknown(&s).unwrap(), 0);
    assert_eq!(commit_diagnostics::needs_attention(&s).unwrap(), 0);
}

#[test]
fn restart_maps_uncertain_and_legacy_abandoned_without_open_unknown_attention() {
    let (dir, mut s) = fixture();
    let op = started_upload(&mut s, b"local edits");
    s.mark_upload_uncertain(&op).unwrap();
    drop(s);

    // Simulate a pre-TB2 v29 row: the state is durable but the diagnosis columns
    // were never written. The startup backfill must rebuild them idempotently.
    let db = rusqlite::Connection::open(dir.path().join("state.sqlite")).unwrap();
    db.execute(
        "UPDATE operations SET outcome=NULL, outcome_category=NULL, last_error=NULL,
                next_retry_at=0 WHERE id=?1",
        [&op],
    )
    .unwrap();
    drop(db);

    let s = Store::open(dir.path()).unwrap();
    let diag = operation_diagnostic(&s);
    assert_eq!(diag.state, "uncertain");
    assert_eq!(diag.outcome, CommitOutcome::UnknownRetryable);
    assert_eq!(diag.category, CommitCategory::Interrupted);
    assert_eq!(diag.attempts, 0);
    assert!(diag.next_retry_at > 0);
    assert!(diag.last_error.as_deref().unwrap().contains("restart"));
    drop(s);

    let db = rusqlite::Connection::open(dir.path().join("state.sqlite")).unwrap();
    db.execute(
        "UPDATE operations SET state='abandoned', outcome=NULL, outcome_category=NULL,
                last_error=NULL, next_retry_at=0, reconcile_attempts=5 WHERE id=?1",
        [&op],
    )
    .unwrap();
    drop(db);

    let s = Store::open(dir.path()).unwrap();
    let diag = operation_diagnostic(&s);
    assert_eq!(diag.state, "abandoned");
    assert_eq!(diag.outcome, CommitOutcome::Confirmed);
    assert_eq!(diag.category, CommitCategory::LocalAbandoned);
    assert_eq!(diag.attempts, 5);
    assert_eq!(diag.next_retry_at, 0);
    assert_eq!(commit_diagnostics::open_unknown(&s).unwrap(), 0);
    assert_eq!(commit_diagnostics::needs_attention(&s).unwrap(), 0);
}

#[test]
fn recheck_clears_diagnosis_and_rearms_the_queue() {
    let (_d, mut s) = fixture();
    let op = started_upload(&mut s, b"local edits");
    s.mark_upload_uncertain(&op).unwrap();
    s.note_upload_reconcile_attempt_diagnosed(
        &op,
        1_700_000_000,
        CommitCategory::PermissionUncertain,
        Some("pending decision"),
    )
    .unwrap();
    assert_eq!(s.paused_upload_operations().unwrap(), 0);

    assert_eq!(s.reset_upload_reconcile_budget().unwrap(), 1);
    // A recheck rewinds the operation to a clean uncertain row: the stale
    // diagnosis must disappear rather than describe a retry that will not
    // happen, and the operation counts as pending work again.
    assert!(
        diagnostics(&s).into_iter().all(|d| d.scope != "operation"),
        "recheck must clear the stale operation diagnosis"
    );
    assert_eq!(s.pending_operations().unwrap(), 1);
    let attempts = s
        .note_upload_reconcile_attempt_diagnosed(
            &op,
            1_700_000_000,
            CommitCategory::PermissionUncertain,
            Some("server did not confirm the commit"),
        )
        .unwrap();
    assert_eq!(attempts, 1);
    let d = operation_diagnostic(&s);
    assert_eq!(d.state, "uncertain");
    assert_eq!(d.outcome, CommitOutcome::UnknownRetryable);
    assert_eq!(d.category, CommitCategory::PermissionUncertain);
    assert_eq!(d.attempts, 1);
}

#[test]
fn confirmed_upload_is_recorded_as_confirmed() {
    let (_d, mut s) = fixture();
    let op = started_upload(&mut s, b"local edits");
    s.confirm_upload(&op, "v2").unwrap();

    let d = operation_diagnostic(&s);
    assert_eq!(d.state, "committed");
    assert_eq!(d.outcome, CommitOutcome::Confirmed);
    assert_eq!(d.category, CommitCategory::Committed);
    // The view reports the journal's reconcile attempt count, not the one-shot
    // write that produced the confirmation.
    assert_eq!(d.attempts, 0);
    assert_eq!(d.next_retry_at, 0);
    assert!(!d.outcome.is_open());
    assert!(!d.outcome.needs_attention());
}

#[test]
fn namespace_uncertainty_is_recorded_with_its_own_scope() {
    let (_d, mut s) = fixture();
    s.rename_local("1", "A.doc", "1", "B.doc", false).unwrap();
    let q = s.next_rename().unwrap().unwrap();
    s.mark_rename_started(&q.job.operation).unwrap();
    s.mark_rename_uncertain_diagnosed(
        &q.job.operation,
        CommitCategory::Protocol,
        Some("unexpected rename response"),
    )
    .unwrap();

    let d = diagnostics(&s)
        .into_iter()
        .find(|d| d.scope == "namespace")
        .expect("namespace diagnostic");
    assert_eq!(d.operation, q.job.operation);
    assert_eq!(d.kind, "rename");
    assert_eq!(d.state, "uncertain");
    assert_eq!(d.outcome, CommitOutcome::UnknownRetryable);
    assert_eq!(d.category, CommitCategory::Protocol);
    assert_eq!(d.last_error.as_deref(), Some("unexpected rename response"));
}

#[test]
fn failed_download_is_recorded_with_its_retry_time() {
    let (_d, mut s) = fixture();
    let mut remote = file("v2");
    remote.name = "A.doc".into();
    s.receive(&Batch {
        from: s.cursor().unwrap(),
        to: "c2".into(),
        events: vec![Event {
            id: "event-c2".into(),
            change: Change::Upsert(remote),
        }],
    })
    .unwrap();
    s.drain().unwrap();
    let ticket = s.downloads_needed().unwrap().remove(0);
    s.fail_download(&ticket, DownloadFailureKind::Transport, None, 1_700_000_000)
        .unwrap();

    let d = diagnostics(&s)
        .into_iter()
        .find(|d| d.scope == "transfer")
        .expect("transfer diagnostic");
    assert_eq!(d.outcome, CommitOutcome::UnknownRetryable);
    assert_eq!(d.category, CommitCategory::Transport);
    // The view derives the download attempt count (0) and retry clock from the
    // transfers row; the failure JSON remains the detailed record.
    assert_eq!(d.attempts, 0);
    assert_eq!(d.next_retry_at, 1_700_000_000 + 2);
    assert_eq!(d.last_error.as_deref(), Some("transport"));
}

#[test]
fn legacy_v29_view_is_repaired_on_open() {
    let (dir, s) = fixture();
    drop(s);
    // A database that was created by TB1 already carries user_version=29 but a
    // pre-TB2 `commit_diagnostics` view (`op_id`/`op_kind`, no `updated_at`).
    // TB2 refined the DDL without bumping the schema version, so opening the
    // store must repair the view instead of skipping the whole migration.
    let db = rusqlite::Connection::open(dir.path().join("state.sqlite")).unwrap();
    db.execute_batch(
        "DROP VIEW IF EXISTS commit_diagnostics;
         CREATE VIEW commit_diagnostics AS
           SELECT 'operation' AS scope, o.id AS op_id, o.kind AS op_kind, o.outcome
           FROM operations o WHERE o.outcome IS NOT NULL;",
    )
    .unwrap();
    drop(db);

    let s = Store::open(dir.path()).unwrap();
    commit_diagnostics::list(&s, 10).unwrap();
    let db = rusqlite::Connection::open(dir.path().join("state.sqlite")).unwrap();
    let sql: String = db
        .query_row(
            "SELECT sql FROM sqlite_master WHERE type='view' AND name='commit_diagnostics'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert!(sql.contains("AS operation"), "repaired view SQL: {sql}");
    assert!(sql.contains("updated_at"), "repaired view SQL: {sql}");
    let version: i64 = db
        .query_row("PRAGMA user_version", [], |r| r.get(0))
        .unwrap();
    assert_eq!(version, 29);
}

#[test]
fn snapshot_install_marks_unresolved_inflight_upload_as_interrupted() {
    let (_d, mut s) = fixture();
    let op = started_upload(&mut s, b"local edits");
    // A namespace swap cannot observe the in-flight commit. The replacement
    // transaction moves the row to `uncertain` and must record why.
    s.install_remote_snapshot(&[root(), file("v2")], "2", "server")
        .unwrap();
    let d = operation_diagnostic(&s);
    assert_eq!(d.operation, op);
    assert_eq!(d.state, "uncertain");
    assert_eq!(d.outcome, CommitOutcome::UnknownRetryable);
    assert_eq!(d.category, CommitCategory::Interrupted);
    assert!(d
        .last_error
        .as_deref()
        .unwrap()
        .contains("snapshot replaced"));
    assert!(commit_diagnostics::open_unknown(&s).unwrap() >= 1);
}

// ---- #57: namespace reconcile attempts must be durable ---------------------

use drive_core::namespace_schedule::NamespaceRecoveryObservation as Observation;

fn namespace_attempts(s: &Store, operation: &str) -> i64 {
    let record = commit_diagnostics::list(s, 200)
        .unwrap()
        .into_iter()
        .find(|record| record.scope == "namespace" && record.operation == operation)
        .expect("namespace diagnostic");
    i64::from(record.attempts)
}

fn journal_attempts(dir: &tempfile::TempDir, operation: &str) -> i64 {
    let db = rusqlite::Connection::open(dir.path().join("state.sqlite")).unwrap();
    db.query_row(
        "SELECT attempts FROM namespace_ops WHERE id=?1",
        [operation],
        |row| row.get(0),
    )
    .unwrap()
}

/// Acceptance criterion: `namespace_ops.attempts` is written by
/// `commit_diagnostics::record()`, survives a restart and matches the
/// diagnostics view (`GetIssues`/CLI read the same projection). The in-process
/// `NamespaceRuntime.recovery` counter may reset, the durable column may not.
#[test]
fn namespace_reconcile_attempts_survive_restart_and_match_diagnostics() {
    let (dir, mut s) = fixture();
    s.rename_local("1", "A.doc", "1", "B.doc", false).unwrap();
    let queued = s.next_rename().unwrap().unwrap();
    let operation = queued.job.operation.clone();
    s.mark_rename_started(&operation).unwrap();
    s.mark_rename_uncertain_diagnosed(&operation, CommitCategory::Protocol, Some("lost reply"))
        .unwrap();

    for expected in 1..=2i64 {
        assert!(
            s.observe_namespace_unresolved(&operation, Observation::CommitUnconfirmed)
                .unwrap()
        );
        assert_eq!(
            journal_attempts(&dir, &operation),
            expected,
            "record() must persist namespace_ops.attempts"
        );
        assert_eq!(namespace_attempts(&s, &operation), expected);
    }
    // The process-local observation map is the only in-memory copy: a restart
    // must keep the durable attempts and continue the counter from the journal.
    drop(s);
    let mut s = Store::open(dir.path()).unwrap();
    assert_eq!(journal_attempts(&dir, &operation), 2);
    assert_eq!(namespace_attempts(&s, &operation), 2);
    assert_eq!(
        s.queue_health().unwrap().recovery_attempts_observed,
        0,
        "in-process counter resets on restart by design"
    );
    assert!(
        s.observe_namespace_unresolved(&operation, Observation::CommitUnconfirmed)
            .unwrap()
    );
    assert_eq!(
        journal_attempts(&dir, &operation),
        3,
        "the durable counter must continue instead of restarting at 1"
    );
    assert_eq!(namespace_attempts(&s, &operation), 3);
}

// ---- #9 TB3: scheduler observation must not erase the concrete reason ------

/// The runtime reconcile branches write a concrete diagnosis (e.g. "unexpected
/// reply during rename reconcile"). The scheduler's generic observation runs in
/// the same pass and used to overwrite that text; it may only advance the retry
/// bookkeeping for a row that already carries a concrete reason.
#[test]
fn namespace_observation_preserves_the_runtime_reconcile_reason() {
    let (_dir, mut s) = fixture();
    s.rename_local("1", "A.doc", "1", "B.doc", false).unwrap();
    let operation = s.next_rename().unwrap().unwrap().job.operation;
    s.mark_rename_started(&operation).unwrap();
    // The runtime reconcile branch writes this concrete reason once the journal
    // row is already `uncertain`.
    s.mark_rename_uncertain_diagnosed(
        &operation,
        CommitCategory::Protocol,
        Some("server returned an unexpected reply during rename reconcile"),
    )
    .unwrap();
    s.record_namespace_commit_diagnostic(
        &operation,
        CommitCategory::Protocol,
        Some("server returned an unexpected reply during rename reconcile"),
        0,
    )
    .unwrap();

    assert!(
        s.observe_namespace_unresolved(&operation, Observation::CommitUnconfirmed)
            .unwrap()
    );

    let diagnostic = diagnostics(&s)
        .into_iter()
        .find(|d| d.scope == "namespace" && d.operation == operation)
        .expect("namespace diagnostic");
    assert_eq!(diagnostic.outcome, CommitOutcome::UnknownRetryable);
    assert_eq!(diagnostic.category, CommitCategory::Protocol);
    assert_eq!(
        diagnostic.last_error.as_deref(),
        Some("server returned an unexpected reply during rename reconcile"),
        "the scheduler observation must keep the concrete reconcile reason"
    );
    assert!(diagnostic.attempts > 0);
    assert!(diagnostic.next_retry_at > 0);
}

/// A restart backfills `uncertain` rows with the placeholder
/// `interrupted` category. The first scheduler observation still owns that
/// row and must replace the placeholder with the observed category/reason.
#[test]
fn namespace_observation_refines_interrupted_backfill() {
    let (_dir, mut s) = fixture();
    s.rename_local("1", "A.doc", "1", "B.doc", false).unwrap();
    let operation = s.next_rename().unwrap().unwrap().job.operation;
    s.mark_rename_started(&operation).unwrap();
    s.mark_rename_uncertain(&operation).unwrap();

    assert!(
        s.observe_namespace_unresolved(&operation, Observation::CommitUnconfirmed)
            .unwrap()
    );

    let diagnostic = diagnostics(&s)
        .into_iter()
        .find(|d| d.scope == "namespace" && d.operation == operation)
        .expect("namespace diagnostic");
    assert_eq!(diagnostic.category, CommitCategory::Protocol);
    assert_eq!(
        diagnostic.last_error.as_deref(),
        Some("namespace commit outcome unconfirmed; reconciling without replay")
    );
}
