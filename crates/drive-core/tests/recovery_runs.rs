//! Recovery journal contract (#10 TB1).
//!
//! The journal is single-flight, records bounded attempts, and keeps the
//! terminal outcome (`recovered` / `needs_attention`) queryable for the
//! additive desktop `recovery` object.

use drive_core::recovery_runs::*;
use drive_core::Store;

fn store() -> (tempfile::TempDir, Store) {
    let directory = tempfile::tempdir().expect("tempdir");
    let store = Store::open(directory.path()).expect("open store");
    (directory, store)
}

#[test]
fn begin_update_and_latest_roundtrip() {
    let (_directory, store) = store();
    let run = store
        .begin_recovery_run(CAUSE_CURSOR_INVALIDATED)
        .expect("begin");
    assert_eq!(run.cause, CAUSE_CURSOR_INVALIDATED);
    assert_eq!(run.status, STATUS_RECOVERING);
    assert_eq!(run.attempts, 0);
    assert!(run.last_error.is_none());

    store
        .update_recovery_run(run.id, STATUS_RETRYING, 1, Some("native_cursor_reset"), 1_234)
        .expect("update");
    let latest = store.latest_recovery_run().expect("latest").expect("row");
    assert_eq!(latest.id, run.id);
    assert_eq!(latest.status, STATUS_RETRYING);
    assert_eq!(latest.attempts, 1);
    assert_eq!(latest.next_retry_at, 1_234);
    assert_eq!(latest.last_error.as_deref(), Some("native_cursor_reset"));
    assert!(latest.updated_at >= latest.started_at);
}

#[test]
fn a_single_open_run_is_reused_across_begin_calls() {
    let (_directory, store) = store();
    let first = store.begin_recovery_run(CAUSE_RESCAN_LATER).expect("begin");
    let again = store.begin_recovery_run(CAUSE_RESCAN_LATER).expect("reuse");
    assert_eq!(first.id, again.id);
    // A second signal joins the in-flight run instead of stacking a new row.
    let joined = store.begin_recovery_run(CAUSE_MERGE_MODE).expect("join");
    assert_eq!(joined.id, first.id);
    assert_eq!(store.open_recovery_runs().expect("open").len(), 1);
}

#[test]
fn exhausted_run_becomes_needs_attention_and_leaves_the_open_set() {
    let (_directory, store) = store();
    let run = store.begin_recovery_run(CAUSE_RESCAN_LATER).expect("begin");
    for attempt in 1..=3 {
        store
            .update_recovery_run(run.id, STATUS_RETRYING, attempt, Some("transport"), 0)
            .expect("retry");
    }
    store
        .finish_recovery_run(run.id, STATUS_NEEDS_ATTENTION)
        .expect("finish");
    let latest = store.latest_recovery_run().expect("latest").expect("row");
    assert_eq!(latest.status, STATUS_NEEDS_ATTENTION);
    assert_eq!(latest.attempts, 3);
    assert!(store.open_recovery_runs().expect("open").is_empty());
    assert!(store.open_recovery_run().expect("open one").is_none());
}

#[test]
fn prune_keeps_finished_rows_and_preserves_open_runs() {
    let (_directory, store) = store();
    for _ in 0..5 {
        let run = store.begin_recovery_run(CAUSE_SNAPSHOT_CHANGED).expect("begin");
        store
            .finish_recovery_run(run.id, STATUS_RECOVERED)
            .expect("finish");
    }
    let open = store.begin_recovery_run(CAUSE_RESCAN_LATER).expect("open");
    assert_eq!(store.prune_recovery_runs(2).expect("prune"), 3);
    assert_eq!(store.open_recovery_runs().expect("open").len(), 1);
    let latest = store.latest_recovery_run().expect("latest").expect("row");
    assert_eq!(latest.id, open.id);
}

#[test]
fn invalid_cause_or_status_is_rejected() {
    let (_directory, store) = store();
    assert!(store.begin_recovery_run("not_a_cause").is_err());
    let run = store.begin_recovery_run(CAUSE_RESCAN_LATER).expect("begin");
    assert!(store
        .update_recovery_run(run.id, "not_a_status", 0, None, 0)
        .is_err());
    assert!(store.finish_recovery_run(run.id, "done").is_err());
}