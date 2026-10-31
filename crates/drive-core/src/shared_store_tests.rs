use super::*;
use crate::{
    Error, Node,
    status_changes::{ChangePage, JournalBudget},
};

fn fixture() -> (tempfile::TempDir, SharedStore, CommitNotifications) {
    let dir = tempfile::tempdir().unwrap();
    let mut store = Store::open(dir.path()).unwrap();
    store
        .install_remote_snapshot(
            &[
                Node {
                    id: "root".into(),
                    parent: None,
                    name: "root".into(),
                    version: "1".into(),
                    directory: true,
                    metadata: None,
                },
                Node {
                    id: "a".into(),
                    parent: Some("root".into()),
                    name: "a".into(),
                    version: "1".into(),
                    directory: false,
                    metadata: None,
                },
            ],
            "1",
            "server",
        )
        .unwrap();
    let watermark = store.status_change_watermark().unwrap();
    store.prune_status_changes(&watermark.cursor).unwrap();
    let shared = SharedStore::new(store);
    let notifications = shared.take_notifications().unwrap();
    (dir, shared, notifications)
}
fn quiet(n: &CommitNotifications) {
    assert_eq!(n.try_recv(), Err(TryRecvError::Empty));
}

#[test]
fn outer_error_after_commit_wakes_once_for_the_entire_guard() {
    let (_dir, shared, notifications) = fixture();
    let result = (|| -> crate::Result<()> {
        let mut guard = shared.lock().unwrap();
        guard.pin("a", true)?;
        guard.pin("root", true)?;
        assert_eq!(shared.change_state().revision, 0);
        Err(Error::State("later operation failed".into()))
    })();
    assert!(result.is_err());
    notifications.try_recv().unwrap();
    assert_eq!(
        notifications.state(),
        CommitState {
            revision: 1,
            verified: true
        }
    );
    quiet(&notifications);
    assert!(shared.lock().unwrap().is_pinned("a").unwrap());
}

#[test]
fn committed_snapshot_gap_wakes_even_when_later_diagnostic_fails() {
    let (_dir, shared, n) = fixture();
    let before = shared.lock().unwrap().status_change_watermark().unwrap();
    let result = (|| -> crate::Result<()> {
        let mut guard = shared.lock().unwrap();
        guard.install_remote_snapshot(&[Node {
            id: "root".into(), parent: None, name: "root".into(),
            version: "2".into(), directory: true, metadata: None,
        }], "2", "server")?;
        Err(Error::State("later diagnostic failed".into()))
    })();
    assert!(result.is_err());
    n.try_recv().unwrap();
    assert_eq!(n.state(), CommitState { revision: 1, verified: true });
    quiet(&n);
    let after = shared.lock().unwrap().status_change_watermark().unwrap();
    assert_eq!(after.snapshot_resets, before.snapshot_resets + 1);
    assert_eq!(after.cursor.sequence, before.cursor.sequence + 1);
    assert_eq!((after.entries, after.bytes), (0, 0));
}

#[test]
fn readonly_noop_rollback_and_savepoint_rollback_do_not_wake() {
    let (_dir, shared, n) = fixture();
    for _ in 0..3 {
        assert!(shared.lock().unwrap().node("a").is_ok());
    }
    shared.lock().unwrap().pin("a", false).unwrap();
    shared
        .lock()
        .unwrap()
        .db
        .execute_batch("BEGIN; UPDATE nodes SET pinned=1 WHERE id='a'; ROLLBACK;")
        .unwrap();
    shared
        .lock()
        .unwrap()
        .db
        .execute_batch(
            "SAVEPOINT s; UPDATE nodes SET pinned=1 WHERE id='a'; ROLLBACK TO s; RELEASE s;",
        )
        .unwrap();
    assert_eq!(
        n.state(),
        CommitState {
            revision: 0,
            verified: true
        }
    );
    quiet(&n);
}

#[test]
fn cleanup_does_not_feedback_but_a_lagging_cursor_still_detects_prune() {
    let (_dir, shared, n) = fixture();
    let old = shared
        .lock()
        .unwrap()
        .status_change_watermark()
        .unwrap()
        .cursor;
    shared.lock().unwrap().pin("a", true).unwrap();
    n.try_recv().unwrap();
    let revision = n.state();
    let watermark = shared.lock().unwrap().status_change_watermark().unwrap();
    {
        let mut guard = shared.lock().unwrap();
        guard.prune_status_changes(&watermark.cursor).unwrap();
        guard.prune_status_changes(&watermark.cursor).unwrap();
        guard.set_status_journal_budget(watermark.budget).unwrap();
        guard
            .set_status_journal_budget(JournalBudget {
                entries: 4096,
                ..watermark.budget
            })
            .unwrap();
    }
    assert_eq!(n.state(), revision);
    quiet(&n);
    assert!(matches!(
        shared
            .lock()
            .unwrap()
            .read_status_changes(&old, 10)
            .unwrap(),
        ChangePage::Rebuild { .. }
    ));
}

#[test]
fn budget_shrink_that_discards_history_is_an_invalidation() {
    let (_dir, shared, n) = fixture();
    {
        let mut guard = shared.lock().unwrap();
        guard.pin("a", true).unwrap();
        guard.pin("root", true).unwrap();
    }
    n.try_recv().unwrap();
    shared
        .lock()
        .unwrap()
        .set_status_journal_budget(JournalBudget {
            entries: 1,
            bytes: 4096,
        })
        .unwrap();
    n.try_recv().unwrap();
    assert_eq!(n.state().revision, 2);
}

#[test]
fn uncommitted_boundary_invalidates_without_publishing_internal_sequence() {
    let (_dir, shared, n) = fixture();
    let cursor = shared
        .lock()
        .unwrap()
        .status_change_watermark()
        .unwrap()
        .cursor;
    shared
        .lock()
        .unwrap()
        .db
        .execute_batch("BEGIN; UPDATE nodes SET pinned=1 WHERE id='a';")
        .unwrap();
    n.try_recv().unwrap();
    assert_eq!(
        n.state(),
        CommitState {
            revision: 1,
            verified: false
        }
    );
    assert!(shared.lock().unwrap().status_change_watermark().is_err());
    quiet(&n); // repeated observation of the same failed boundary is not progress
    shared
        .lock()
        .unwrap()
        .db
        .execute_batch("ROLLBACK;")
        .unwrap();
    n.try_recv().unwrap();
    assert_eq!(
        n.state(),
        CommitState {
            revision: 2,
            verified: true
        }
    );
    assert_eq!(
        shared
            .lock()
            .unwrap()
            .status_change_watermark()
            .unwrap()
            .cursor,
        cursor
    );
    // Even a transaction with no row writes must not escape as a ready boundary.
    shared.lock().unwrap().db.execute_batch("BEGIN;").unwrap();
    n.try_recv().unwrap();
    assert!(!n.state().verified);
    shared
        .lock()
        .unwrap()
        .db
        .execute_batch("ROLLBACK;")
        .unwrap();
}

#[test]
fn failed_initial_watermark_is_invalid_not_ready_zero() {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(dir.path()).unwrap();
    store.db.execute_batch("BEGIN;").unwrap();
    let shared = SharedStore::new(store);
    let n = shared.take_notifications().unwrap();
    assert_eq!(
        n.state(),
        CommitState {
            revision: 1,
            verified: false
        }
    );
    n.try_recv().unwrap();
    shared
        .lock()
        .unwrap()
        .db
        .execute_batch("ROLLBACK;")
        .unwrap();
    n.try_recv().unwrap();
    assert!(n.state().verified);
}

#[test]
fn try_lock_and_poison_recovery_keep_the_boundary_wrapper() {
    let (_dir, shared, n) = fixture();
    let guard = shared.lock().unwrap();
    assert!(matches!(shared.try_lock(), Err(TryLockError::WouldBlock)));
    drop(guard);
    shared.try_lock().unwrap().pin("a", true).unwrap();
    n.try_recv().unwrap();
    let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let mut guard = shared.lock().unwrap();
        guard.pin("a", false).unwrap();
        panic!("synthetic panic after a committed mutation");
    }));
    assert!(panic.is_err());
    assert!(shared.is_poisoned());
    n.try_recv().unwrap();
    assert_eq!(
        n.state(),
        CommitState {
            revision: 2,
            verified: false
        }
    );
    let mut guard = match shared.try_lock() {
        Err(TryLockError::Poisoned(error)) => error.into_inner(),
        _ => panic!("try_lock lost poison"),
    };
    guard.pin("a", true).unwrap();
    drop(guard);
    n.try_recv().unwrap();
    assert_eq!(
        n.state(),
        CommitState {
            revision: 3,
            verified: false
        }
    );
    drop(shared.lock().unwrap_err().into_inner());
    quiet(&n);
}

#[test]
fn full_wake_queue_coalesces_without_losing_latest_revision_or_holding_store() {
    let (_dir, shared, n) = fixture();
    for index in 0..300 {
        shared.lock().unwrap().pin("a", index % 2 == 0).unwrap();
    }
    assert_eq!(
        n.state(),
        CommitState {
            revision: 300,
            verified: true
        }
    );
    n.try_recv().unwrap();
    // Receipt is not a callback under the writer's lock.
    assert!(shared.try_lock().is_ok());
    quiet(&n);
    assert!(matches!(
        shared.take_notifications(),
        Err(AlreadySubscribed)
    ));
    drop(n);
    assert!(matches!(
        shared.take_notifications(),
        Err(AlreadySubscribed)
    ));
    shared.lock().unwrap().pin("a", true).unwrap(); // disconnected receiver is nonblocking too
}

#[test]
fn receiver_disconnects_when_store_is_gone() {
    let (_dir, shared, n) = fixture();
    drop(shared);
    assert_eq!(
        n.state(),
        CommitState {
            revision: 1,
            verified: false
        }
    );
    n.try_recv().unwrap();
    assert_eq!(n.try_recv(), Err(TryRecvError::Disconnected));
}

#[test]
fn extracting_unique_store_invalidates_the_old_domain() {
    let (_dir, shared, n) = fixture();
    let store = shared.into_inner().unwrap();
    n.try_recv().unwrap();
    assert!(!n.state().verified);
    assert_eq!(n.try_recv(), Err(TryRecvError::Disconnected));
    assert!(store.node("a").is_ok());
}

#[test]
fn journal_read_failure_on_drop_invalidates_instead_of_panicking() {
    let (_dir, shared, n) = fixture();
    {
        let mut guard = shared.lock().unwrap();
        let store = &mut *guard;
        store
            .db
            .execute_batch("DROP TABLE dn_status_meta;")
            .unwrap();
    }
    n.try_recv().unwrap();
    assert_eq!(
        n.state(),
        CommitState {
            revision: 1,
            verified: false
        }
    );
    assert!(!shared.is_poisoned());
    drop(shared.lock().unwrap());
    quiet(&n);
}

#[test]
fn panic_with_readonly_guard_invalidates_even_without_sql_changes() {
    let (_dir, shared, n) = fixture();
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _guard = shared.lock().unwrap();
        panic!("synthetic readonly panic");
    }));
    n.try_recv().unwrap();
    assert_eq!(
        n.state(),
        CommitState {
            revision: 1,
            verified: false
        }
    );
    assert!(shared.is_poisoned());
}

#[test]
fn real_draft_io_failure_after_dirty_commit_still_wakes() {
    let (dir, shared, n) = fixture();
    let draft = shared.lock().unwrap().begin_draft("a", "1", None).unwrap();
    n.try_recv().unwrap();
    let before = n.state().revision;
    // Test-owned draft pathname is removed; the open descriptor remains alive.
    // editing_file commits dirty/generation before attempting to reopen it.
    std::fs::remove_file(dir.path().join("drafts").join(&draft.token)).unwrap();
    assert!(
        shared
            .lock()
            .unwrap()
            .draft_write(&draft, 0, b"edit", false)
            .is_err()
    );
    n.try_recv().unwrap();
    assert_eq!(n.state().revision, before + 1);
    assert!(shared.lock().unwrap().dirty_draft("a").unwrap());
}
