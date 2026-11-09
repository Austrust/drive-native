//! Exhausted permission-uncertain uploads reach an explicit paused terminal
//! state instead of blocking pending work forever (issue #14).
use drive_core::sync_history::HistoryQuery;
use drive_core::*;

fn node(id: &str, parent: Option<&str>, name: &str, version: &str, directory: bool) -> Node {
    Node {
        id: id.into(),
        parent: parent.map(str::to_owned),
        name: name.into(),
        version: version.into(),
        directory,
        metadata: Some(NodeMetadata {
            size: 3,
            mtime: 1,
            mode: 0o644,
        }),
    }
}
fn root() -> Node {
    node("1", None, "root", "r1", true)
}
fn file(version: &str) -> Node {
    node("2", Some("1"), "A.doc", version, false)
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
/// Unbound local creation whose first upload ended in a permission-uncertain
/// commit, exactly the state produced by the server's 0x9004 reply.
fn uncertain_creation(s: &mut Store) -> (Node, String) {
    let n = s.create_local_file("1", "new", 0o600).unwrap();
    let op = s.save_local(&n.id, b"new creation").unwrap();
    s.creation_saved(&n.id).unwrap();
    s.mark_upload_started(&op).unwrap();
    s.mark_upload_uncertain(&op).unwrap();
    (n, op)
}
fn mark_uncertain(s: &mut Store, op: &str) {
    s.mark_upload_started(op).unwrap();
    s.mark_upload_uncertain(op).unwrap();
}
/// Spend the whole read-only reconcile budget. The final attempt is the one
/// that must move the operation into the paused terminal state.
fn exhaust_budget(s: &mut Store, op: &str) -> (u32, i64) {
    let mut now = 1_700_000_000_i64;
    let mut attempts = 0;
    while attempts < MAX_UPLOAD_RECONCILE_ATTEMPTS {
        now += 1;
        attempts = s.note_upload_reconcile_attempt(op, now).unwrap();
    }
    (attempts, now)
}
fn history_row<'a>(
    page: &'a drive_core::sync_history::HistoryPage,
    node: &str,
) -> &'a drive_core::sync_history::SyncHistoryRecord {
    page.records
        .iter()
        .find(|r| r.node.as_deref() == Some(node) && r.action == "upload")
        .unwrap()
}

#[test]
fn exhausted_reconcile_abandons_upload_without_blocking_pending_work() {
    let (_d, mut s) = fixture();
    let (n, op) = uncertain_creation(&mut s);
    assert_eq!(s.pending_operations().unwrap(), 1);
    assert!(s.upload_reconcile_due(1_700_000_000).unwrap());
    let (attempts, now) = exhaust_budget(&mut s, &op);
    assert_eq!(attempts, MAX_UPLOAD_RECONCILE_ATTEMPTS);
    // Terminal, explainable state: no longer counted as unfinished work.
    assert_eq!(s.upload_job(&op).unwrap().state, "abandoned");
    assert_eq!(s.pending_operations().unwrap(), 0);
    assert_eq!(s.pending_namespace_operations().unwrap(), 0);
    assert_eq!(s.upload_operations("abandoned").unwrap(), vec![op.clone()]);
    assert!(s.upload_operations("uncertain").unwrap().is_empty());
    assert!(!s.upload_reconcile_due(now + 86_400).unwrap());
    // The local creation is preserved; only the upload is paused.
    assert!(s.lookup("1", "new").is_ok());
    assert_eq!(
        s.open_cached(&n.id).unwrap().size().unwrap(),
        b"new creation".len() as u64
    );
    // Storage garbage collection and cache eviction must keep the only copy of
    // a paused upload: the server result is unknown, so the blob stays pinned.
    s.collect_garbage().unwrap();
    assert_eq!(
        s.open_cached(&n.id).unwrap().size().unwrap(),
        b"new creation".len() as u64
    );
    let page = s.sync_history(&HistoryQuery::default()).unwrap();
    let row = history_row(&page, &n.id);
    assert_eq!(row.result, "failed");
    assert_eq!(
        row.error_kind.as_deref(),
        Some("permission_commit_uncertain")
    );
    let detail = row.detail.as_ref().unwrap();
    assert_eq!(
        detail["abandoned"].as_str(),
        Some("reconcile_exhausted")
    );
    assert_eq!(
        detail["reconcile_attempts"].as_u64(),
        Some(u64::from(MAX_UPLOAD_RECONCILE_ATTEMPTS))
    );
    let summary = row.error_summary.as_deref().unwrap();
    assert!(summary.contains("自动核对已暂停"), "{summary}");
    assert!(summary.contains("本地内容保留"), "{summary}");
}

#[test]
fn abandoned_upload_survives_restart_and_stays_in_the_issue_list() {
    let (d, mut s) = fixture();
    let (n, op) = uncertain_creation(&mut s);
    exhaust_budget(&mut s, &op);
    drop(s);
    let s = Store::open(d.path()).unwrap();
    assert_eq!(s.upload_job(&op).unwrap().state, "abandoned");
    assert_eq!(s.pending_operations().unwrap(), 0);
    let issues = s.sync_issues("1", "").unwrap();
    assert_eq!(issues.entries.len(), 1);
    let issue = &issues.entries[0];
    assert_eq!(issue.id, n.id);
    assert_eq!(issue.reason, "commit_uncertain");
    assert!(issue.reconcile_exhausted);
    assert_eq!(issue.reconcile_attempts, MAX_UPLOAD_RECONCILE_ATTEMPTS);
    let json = serde_json::to_value(issue).unwrap();
    assert_eq!(json["can_abandon_upload"], true);
    assert_eq!(s.sync_issue_count("1").unwrap(), 1);
}

#[test]
fn bound_upload_abandoned_after_budget_keeps_local_bytes_without_abandon_action() {
    let (_d, mut s) = fixture();
    let op = s.save_local("2", b"local edit").unwrap();
    mark_uncertain(&mut s, &op);
    exhaust_budget(&mut s, &op);
    assert_eq!(s.upload_job(&op).unwrap().state, "abandoned");
    assert_eq!(s.pending_operations().unwrap(), 0);
    // Existing remote files keep the local edit and only offer recheck.
    assert_eq!(
        s.open_cached("2").unwrap().size().unwrap(),
        b"local edit".len() as u64
    );
    let issue = s.sync_issue("1", "2").unwrap().unwrap();
    assert_eq!(issue.reason, "commit_uncertain");
    let json = serde_json::to_value(&issue).unwrap();
    assert_eq!(json["can_abandon_upload"], false);
}

#[test]
fn recheck_rearms_an_abandoned_upload_without_a_new_operation() {
    let (_d, mut s) = fixture();
    let (_n, op) = uncertain_creation(&mut s);
    let (_attempts, now) = exhaust_budget(&mut s, &op);
    assert!(!s.upload_reconcile_due(now + 86_400).unwrap());
    assert_eq!(s.reset_upload_reconcile_budget().unwrap(), 1);
    assert_eq!(s.upload_job(&op).unwrap().state, "uncertain");
    assert!(s.upload_reconcile_due(now + 86_400).unwrap());
    assert_eq!(s.pending_operations().unwrap(), 1);
    let page = s.sync_history(&HistoryQuery::default()).unwrap();
    let row = history_row(&page, &uncertain_creation_node(&s));
    assert_eq!(row.result, "pending");
    let detail = row.detail.as_ref().unwrap();
    assert!(detail.get("abandoned").is_none(), "{detail}");
    assert_eq!(detail["reconcile_attempts"].as_u64(), Some(0));
}

fn uncertain_creation_node(s: &Store) -> String {
    s.lookup("1", "new").unwrap().node.id
}

#[test]
fn a_new_local_save_supersedes_the_abandoned_operation() {
    let (_d, mut s) = fixture();
    let (n, op) = uncertain_creation(&mut s);
    exhaust_budget(&mut s, &op);
    let next = s.save_local(&n.id, b"edited after pause").unwrap();
    assert_eq!(s.upload_job(&op).unwrap().state, "superseded");
    assert!(s.upload_operations("abandoned").unwrap().is_empty());
    assert_eq!(s.upload_operations("pending").unwrap(), vec![next.clone()]);
    assert_eq!(s.pending_operations().unwrap(), 1);
    s.mark_upload_started(&next).unwrap();
    assert_eq!(s.upload_job(&next).unwrap().state, "inflight");
}

#[test]
fn deleting_the_local_file_still_retires_an_abandoned_creation() {
    let (_d, mut s) = fixture();
    let (n, op) = uncertain_creation(&mut s);
    exhaust_budget(&mut s, &op);
    s.unlink_local("1", "new", &n.id, false).unwrap();
    assert!(s.next_remove().unwrap().is_none());
    assert_eq!(s.pending_operations().unwrap(), 0);
    assert_eq!(s.pending_namespace_operations().unwrap(), 0);
    assert!(s.upload_operations("abandoned").unwrap().is_empty());
    assert_eq!(s.upload_job(&op).unwrap().state, "superseded");
    let page = s.sync_history(&HistoryQuery::default()).unwrap();
    let row = history_row(&page, &n.id);
    assert_eq!(row.result, "failed");
    let detail = row.detail.as_ref().unwrap();
    assert_eq!(detail["abandoned"].as_str(), Some("local_deleted"));
    assert_eq!(
        detail["reconcile_attempts"].as_u64(),
        Some(u64::from(MAX_UPLOAD_RECONCILE_ATTEMPTS))
    );
    assert_eq!(s.status(&n.id).unwrap(), Status::Deleted);
}

#[test]
fn abandoning_an_unconfirmed_creation_deletes_only_the_local_file() {
    let (_d, mut s) = fixture();
    let (n, op) = uncertain_creation(&mut s);
    let (_, _) = exhaust_budget(&mut s, &op);
    assert!(s.abandon_unconfirmed_creation(&n.id).unwrap());
    // Only the local file disappears; no server remove is queued.
    assert!(s.lookup("1", "new").is_err());
    assert!(s.next_remove().unwrap().is_none());
    assert_eq!(s.pending_operations().unwrap(), 0);
    assert_eq!(s.pending_namespace_operations().unwrap(), 0);
    assert!(s.upload_operations("abandoned").unwrap().is_empty());
    assert_eq!(s.upload_job(&op).unwrap().state, "superseded");
    let page = s.sync_history(&HistoryQuery::default()).unwrap();
    let row = history_row(&page, &n.id);
    assert_eq!(row.result, "failed");
    assert_eq!(
        row.error_kind.as_deref(),
        Some("permission_commit_uncertain")
    );
    let detail = row.detail.as_ref().unwrap();
    assert_eq!(detail["abandoned"].as_str(), Some("user_discarded"));
    assert_eq!(
        detail["reconcile_attempts"].as_u64(),
        Some(u64::from(MAX_UPLOAD_RECONCILE_ATTEMPTS))
    );
    assert!(row
        .error_summary
        .as_deref()
        .unwrap()
        .contains("已放弃本次本地待上传"));
    // The action is idempotent-safe: a second request changes nothing.
    assert!(!s.abandon_unconfirmed_creation(&n.id).unwrap());
    assert!(!s.retire_unconfirmed_creation(&op).unwrap());
}

#[test]
fn abandon_action_refuses_bound_or_still_uploading_entries() {
    let (_d, mut s) = fixture();
    // A bound remote file only offers recheck; abandoning it would hide an
    // unknown server commit.
    let op = s.save_local("2", b"local edit").unwrap();
    mark_uncertain(&mut s, &op);
    exhaust_budget(&mut s, &op);
    assert_eq!(s.upload_job(&op).unwrap().state, "abandoned");
    assert!(!s.abandon_unconfirmed_creation("2").unwrap());
    assert_eq!(s.upload_job(&op).unwrap().state, "abandoned");
    assert_eq!(
        s.open_cached("2").unwrap().size().unwrap(),
        b"local edit".len() as u64
    );
    // A local creation whose upload is still pending must not be guessed away.
    let n = s.create_local_file("1", "later", 0o600).unwrap();
    let pending = s.save_local(&n.id, b"not sent yet").unwrap();
    assert_eq!(s.upload_job(&pending).unwrap().state, "pending");
    assert!(!s.abandon_unconfirmed_creation(&n.id).unwrap());
    assert_eq!(s.upload_job(&pending).unwrap().state, "pending");
    assert!(s.lookup("1", "later").is_ok());
    assert_eq!(
        s.open_cached(&n.id).unwrap().size().unwrap(),
        b"not sent yet".len() as u64
    );
}

/// Regression for issue #14 release blocker: after a permission-uncertain
/// upload, editing again used to queue a second pending upload for the same
/// node. Abandoning the exhausted (older) operation left that pending row
/// behind on a tombstoned node, so `pending_operations` stayed at 1 forever
/// and the upload gate reported a permanent block with no issue entry to act
/// on. Abandoning must supersede the node's residual non-terminal upload rows
/// in the same transaction.
#[test]
fn abandoning_after_resave_supersedes_the_residual_pending_upload() {
    let (_d, mut s) = fixture();
    let (n, old) = uncertain_creation(&mut s);
    let new_op = s.save_local(&n.id, b"edited after uncertainty").unwrap();
    assert_ne!(new_op, old);
    assert_eq!(s.upload_operations("pending").unwrap(), vec![new_op.clone()]);
    let (_, _) = exhaust_budget(&mut s, &old);
    assert_eq!(s.upload_job(&old).unwrap().state, "abandoned");
    assert_eq!(s.pending_operations().unwrap(), 1);
    assert!(s.abandon_unconfirmed_creation(&n.id).unwrap());
    assert_eq!(s.pending_operations().unwrap(), 0);
    assert!(s.upload_operations("pending").unwrap().is_empty());
    assert!(s.upload_operations("uncertain").unwrap().is_empty());
    assert!(s.upload_operations("abandoned").unwrap().is_empty());
    assert_eq!(s.upload_job(&new_op).unwrap().state, "superseded");
    assert!(s.next_remove().unwrap().is_none());
    assert!(s.lookup("1", "new").is_err());
}

/// The same residual-pending shape resolved by deleting the local file: the
/// unlink retirement path must clear both the uncertain/abandoned evidence and
/// the newer pending upload, leaving no server remove behind.
#[test]
fn deleting_after_resave_supersedes_the_residual_pending_upload() {
    let (_d, mut s) = fixture();
    let (n, old) = uncertain_creation(&mut s);
    let new_op = s.save_local(&n.id, b"edited after uncertainty").unwrap();
    exhaust_budget(&mut s, &old);
    assert_eq!(s.pending_operations().unwrap(), 1);
    s.unlink_local("1", "new", &n.id, false).unwrap();
    assert_eq!(s.pending_operations().unwrap(), 0);
    assert_eq!(s.pending_namespace_operations().unwrap(), 0);
    assert!(s.upload_operations("pending").unwrap().is_empty());
    assert!(s.upload_operations("uncertain").unwrap().is_empty());
    assert_eq!(s.upload_job(&new_op).unwrap().state, "superseded");
    assert!(s.next_remove().unwrap().is_none());
    assert_eq!(s.status(&n.id).unwrap(), Status::Deleted);
}

/// The abandoned/resaved cleanup is durable: reopening the store must not
/// resurrect pending work or a paused upload row.
#[test]
fn abandoned_resave_work_remains_clean_after_restart() {
    let (d, mut s) = fixture();
    let (n, old) = uncertain_creation(&mut s);
    let new_op = s.save_local(&n.id, b"edited after uncertainty").unwrap();
    exhaust_budget(&mut s, &old);
    assert!(s.abandon_unconfirmed_creation(&n.id).unwrap());
    assert_eq!(s.upload_job(&new_op).unwrap().state, "superseded");
    drop(s);
    let s = Store::open(d.path()).unwrap();
    assert_eq!(s.pending_operations().unwrap(), 0);
    assert_eq!(s.pending_namespace_operations().unwrap(), 0);
    assert!(s.upload_operations("pending").unwrap().is_empty());
    assert!(s.upload_operations("uncertain").unwrap().is_empty());
    assert!(s.upload_operations("abandoned").unwrap().is_empty());
    assert!(s.lookup("1", "new").is_err());
}

/// The residual sibling upload retired by #14b must also carry a queue-level
/// diagnosis, otherwise the status API exposes a `superseded` row with no
/// outcome at all (#9b TB2).
#[test]
fn abandoning_after_resave_records_local_abandoned_for_residual_upload() {
    let (_d, mut s) = fixture();
    let (n, old) = uncertain_creation(&mut s);
    let new_op = s.save_local(&n.id, b"edited after uncertainty").unwrap();
    exhaust_budget(&mut s, &old);
    assert!(s.abandon_unconfirmed_creation(&n.id).unwrap());

    let diagnostics = drive_core::commit_diagnostics::list(&s, 200).unwrap();
    let sibling = diagnostics
        .iter()
        .find(|d| d.operation == new_op)
        .expect("residual sibling diagnostic");
    assert_eq!(sibling.scope, "operation");
    assert_eq!(sibling.state, "superseded");
    assert_eq!(sibling.outcome, CommitOutcome::Confirmed);
    assert_eq!(sibling.category, CommitCategory::LocalAbandoned);
    assert_eq!(sibling.next_retry_at, 0);
    assert!(sibling
        .last_error
        .as_deref()
        .unwrap()
        .contains("retired before"));

    let retired = diagnostics
        .iter()
        .find(|d| d.operation == old)
        .expect("retired upload diagnostic");
    assert_eq!(retired.outcome, CommitOutcome::Confirmed);
    assert_eq!(retired.category, CommitCategory::LocalAbandoned);
    assert!(
        retired
            .last_error
            .as_deref()
            .unwrap_or("")
            .contains("user discarded"),
        "retired diagnostic = {retired:?}"
    );
    assert_eq!(
        drive_core::commit_diagnostics::needs_attention(&s).unwrap(),
        0
    );
}
