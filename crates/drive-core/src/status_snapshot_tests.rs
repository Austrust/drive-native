use super::*;
use crate::{
    Node, Store,
    status_changes::{ChangePage, GapReason, JournalBudget},
    status_projection::{Capture, ProjectionBudget},
};

fn node(id: &str, parent: Option<&str>, directory: bool) -> Node {
    Node {
        id: id.into(),
        parent: parent.map(str::to_owned),
        name: id.into(),
        version: "v1".into(),
        directory,
        metadata: None,
    }
}
fn fixture() -> (tempfile::TempDir, SharedStore) {
    let directory = tempfile::tempdir().unwrap();
    let mut store = Store::open(directory.path()).unwrap();
    store
        .install_remote_snapshot(
            &[
                node("root", None, true),
                node("a", Some("root"), false),
                node("b", Some("root"), false),
                node("c", Some("root"), false),
            ],
            "1",
            "scope",
        )
        .unwrap();
    (directory, SharedStore::new(store))
}
fn budget() -> SnapshotBudget {
    SnapshotBudget {
        page_nodes: 1,
        ..SnapshotBudget::default()
    }
}
fn finish(mut reader: StatusSnapshotReader) -> CompletedSnapshot {
    while !reader.next_page().unwrap().complete {}
    reader.finish().unwrap()
}

#[test]
fn pages_use_one_wal_view_while_writer_commits_and_delta_catches_up() {
    let (_dir, shared) = fixture();
    let expected = match shared
        .lock()
        .unwrap()
        .snapshot_status(ProjectionBudget::default())
        .unwrap()
    {
        Capture::Ready(snapshot) => snapshot,
        _ => panic!("fixture snapshot"),
    };
    let mut reader = shared.try_begin_status_snapshot(budget()).unwrap();
    assert!(!reader.next_page().unwrap().complete);
    // The reader holds no Store guard, and writer commits under WAL do not wait
    // for completion of the old read transaction.
    let store = shared.try_lock().unwrap();
    store
        .db
        .execute(
            "UPDATE nodes SET name='changed',remote_version='v2' WHERE id='b'",
            [],
        )
        .unwrap();
    store
        .db
        .execute("UPDATE nodes SET deleted=1 WHERE id='c'", [])
        .unwrap();
    drop(store);
    let completed = finish(reader);
    assert_eq!(completed.snapshot, expected);
    assert_eq!(completed.source.journal_epoch, expected.through.epoch);
    assert!(completed.source.commit.verified);
    assert!(shared.change_state().revision > completed.source.commit.revision);
    let delta = shared
        .lock()
        .unwrap()
        .capture_status_delta(&completed.snapshot.through, ProjectionBudget::default())
        .unwrap();
    let Capture::Ready(delta) = delta else {
        panic!("retained changes")
    };
    assert!(delta.through.sequence > completed.snapshot.through.sequence);
    assert_eq!(
        delta
            .nodes
            .iter()
            .find(|n| n.id == "b")
            .unwrap()
            .atom
            .as_ref()
            .unwrap()
            .name,
        "changed"
    );
    assert!(
        delta
            .nodes
            .iter()
            .find(|n| n.id == "c")
            .unwrap()
            .atom
            .is_none()
    );
}

#[test]
fn empty_main_table_is_pinned_before_first_page_not_just_begin() {
    let dir = tempfile::tempdir().unwrap();
    let shared = SharedStore::new(Store::open(dir.path()).unwrap());
    let reader = shared.try_begin_status_snapshot(budget()).unwrap();
    let cursor = reader.cursor().clone();
    shared.lock().unwrap().db.execute(
        "INSERT INTO nodes(id,parent,name,remote_version,directory) VALUES('root',NULL,'root','v1',1)",
        [],
    ).unwrap();
    let completed = finish(reader);
    assert!(completed.snapshot.atoms.is_empty());
    assert_eq!(completed.snapshot.through, cursor);
    assert!(
        matches!(shared.lock().unwrap().read_status_changes(&cursor, 10).unwrap(),
        ChangePage::Ready { changes, .. } if !changes.is_empty())
    );
}

#[test]
fn rollback_during_pages_changes_neither_snapshot_nor_committed_cursor() {
    let (_dir, shared) = fixture();
    let reader = shared.try_begin_status_snapshot(budget()).unwrap();
    let before = reader.cursor().clone();
    let store = shared.lock().unwrap();
    store
        .db
        .execute_batch("BEGIN; UPDATE nodes SET name='not-committed' WHERE id='b'")
        .unwrap();
    let completed = finish(reader);
    assert_eq!(
        completed
            .snapshot
            .atoms
            .iter()
            .find(|n| n.id == "b")
            .unwrap()
            .name,
        "b"
    );
    store.db.execute_batch("ROLLBACK").unwrap();
    drop(store);
    assert_eq!(
        shared
            .lock()
            .unwrap()
            .status_change_watermark()
            .unwrap()
            .cursor,
        before
    );
}

#[test]
fn cancel_and_drop_release_the_read_transaction() {
    for cancel in [true, false] {
        let (_dir, shared) = fixture();
        let reader = shared.try_begin_status_snapshot(budget()).unwrap();
        {
            let store = shared.lock().unwrap();
            store.db.busy_timeout(Duration::ZERO).unwrap();
            store
                .db
                .execute("UPDATE nodes SET name='written' WHERE id='b'", [])
                .unwrap();
            let busy: i64 = store
                .db
                .query_row("PRAGMA wal_checkpoint(TRUNCATE)", [], |r| r.get(0))
                .unwrap();
            assert_eq!(busy, 1);
        }
        if cancel {
            reader.cancel();
        } else {
            drop(reader);
        }
        let store = shared.lock().unwrap();
        let busy: i64 = store
            .db
            .query_row("PRAGMA wal_checkpoint(TRUNCATE)", [], |r| r.get(0))
            .unwrap();
        assert_eq!(busy, 0);
    }
}

#[test]
fn main_reader_is_readonly_and_cannot_read_writer_temp_journal() {
    let (_dir, shared) = fixture();
    let reader = shared.try_begin_status_snapshot(budget()).unwrap();
    let db = reader.db.as_ref().unwrap();
    assert!(db.execute("UPDATE nodes SET name='forbidden'", []).is_err());
    assert!(db.prepare("SELECT * FROM temp.dn_status_changes").is_err());
}

#[test]
fn journal_gap_after_snapshot_is_an_explicit_rebuild() {
    let (_dir, shared) = fixture();
    let reader = shared.try_begin_status_snapshot(budget()).unwrap();
    let mut store = shared.lock().unwrap();
    store
        .set_status_journal_budget(JournalBudget {
            entries: 1,
            bytes: 4096,
        })
        .unwrap();
    store
        .db
        .execute("UPDATE nodes SET name='v2' WHERE id='a'", [])
        .unwrap();
    store
        .db
        .execute("UPDATE nodes SET name='v3' WHERE id='a'", [])
        .unwrap();
    drop(store);
    let completed = finish(reader);
    assert!(matches!(
        shared
            .lock()
            .unwrap()
            .capture_status_delta(&completed.snapshot.through, ProjectionBudget::default())
            .unwrap(),
        Capture::Rebuild {
            reason: RebuildReason::Journal(GapReason::HistoryDiscarded),
            ..
        }
    ));
}

#[test]
fn store_reopen_has_a_different_source_epoch_even_at_same_path() {
    let (dir, shared) = fixture();
    let completed = finish(shared.try_begin_status_snapshot(budget()).unwrap());
    drop(shared);
    let reopened = SharedStore::new(Store::open(dir.path()).unwrap());
    let reader = reopened.try_begin_status_snapshot(budget()).unwrap();
    assert_eq!(reader.source.database, completed.source.database);
    assert_ne!(reader.source.journal_epoch, completed.source.journal_epoch);
    assert!(matches!(
        reopened
            .lock()
            .unwrap()
            .capture_status_delta(&completed.snapshot.through, ProjectionBudget::default())
            .unwrap(),
        Capture::Rebuild {
            reason: RebuildReason::Journal(GapReason::EpochChanged),
            ..
        }
    ));
}

#[test]
fn busy_or_uncommitted_store_never_yields_a_snapshot() {
    let (_dir, shared) = fixture();
    let store = shared.lock().unwrap();
    assert!(matches!(
        shared.try_begin_status_snapshot(budget()),
        Err(SnapshotError::Busy)
    ));
    store
        .db
        .execute_batch("BEGIN; UPDATE nodes SET name='uncommitted' WHERE id='a'")
        .unwrap();
    drop(store);
    assert!(matches!(
        shared.try_begin_status_snapshot(budget()),
        Err(SnapshotError::Unverified)
    ));
    shared.lock().unwrap().db.execute_batch("ROLLBACK").unwrap();
    assert!(shared.try_begin_status_snapshot(budget()).is_ok());
}

#[test]
fn incomplete_and_failed_reads_never_publish_prefixes() {
    let (_dir, shared) = fixture();
    let mut reader = shared.try_begin_status_snapshot(budget()).unwrap();
    assert_eq!(reader.next_page().unwrap().visible_nodes, 1);
    assert!(matches!(reader.finish(), Err(SnapshotError::Incomplete)));
    let mut reader = shared
        .try_begin_status_snapshot(SnapshotBudget {
            max_nodes: 2,
            ..budget()
        })
        .unwrap();
    reader.next_page().unwrap();
    reader.next_page().unwrap();
    assert!(matches!(reader.next_page(), Err(SnapshotError::Budget)));
    assert!(reader.atoms.is_empty());
    assert!(reader.db.is_none());
    assert!(matches!(reader.finish(), Err(SnapshotError::Closed)));
}

#[test]
fn bytes_and_work_are_cumulative_and_deadline_includes_between_page_time() {
    let (_dir, shared) = fixture();
    let mut probe = shared.try_begin_status_snapshot(budget()).unwrap();
    probe.next_page().unwrap();
    let first_bytes = probe.bytes;
    let first_work = probe.work;
    probe.cancel();
    for budget in [
        SnapshotBudget {
            max_bytes: first_bytes + 1,
            ..budget()
        },
        SnapshotBudget {
            max_work: first_work + 1,
            ..budget()
        },
    ] {
        let mut reader = shared.try_begin_status_snapshot(budget).unwrap();
        reader.next_page().unwrap();
        assert!(matches!(reader.next_page(), Err(SnapshotError::Budget)));
        assert!(reader.db.is_none());
    }
    let mut reader = shared.try_begin_status_snapshot(budget()).unwrap();
    reader.next_page().unwrap();
    // Deterministic deadline test: no sleeping or competing load.
    reader.started = Instant::now() - Duration::from_secs(121);
    assert!(matches!(reader.next_page(), Err(SnapshotError::Budget)));
    assert!(reader.db.is_none());
}

#[test]
fn page_size_and_resource_caps_reject_unbounded_configuration() {
    let (_dir, shared) = fixture();
    for invalid in [
        SnapshotBudget {
            page_nodes: 0,
            ..budget()
        },
        SnapshotBudget {
            page_nodes: 4097,
            ..budget()
        },
        SnapshotBudget {
            max_nodes: 500_001,
            ..budget()
        },
        SnapshotBudget {
            max_elapsed: Duration::from_secs(301),
            ..budget()
        },
    ] {
        assert!(matches!(
            shared.try_begin_status_snapshot(invalid),
            Err(SnapshotError::InvalidBudget)
        ));
    }
}

#[test]
fn failed_reader_releases_checkpoint_without_waiting_for_caller_drop() {
    let (_dir, shared) = fixture();
    let mut reader = shared
        .try_begin_status_snapshot(SnapshotBudget {
            max_nodes: 2,
            ..budget()
        })
        .unwrap();
    reader.next_page().unwrap();
    let store = shared.lock().unwrap();
    store.db.busy_timeout(Duration::ZERO).unwrap();
    store
        .db
        .execute("UPDATE nodes SET name='new' WHERE id='b'", [])
        .unwrap();
    let checkpoint = || {
        store
            .db
            .query_row("PRAGMA wal_checkpoint(TRUNCATE)", [], |r| {
                r.get::<_, i64>(0)
            })
            .unwrap()
    };
    assert_eq!(checkpoint(), 1);
    reader.next_page().unwrap();
    assert!(matches!(reader.next_page(), Err(SnapshotError::Budget)));
    assert_eq!(checkpoint(), 0);
    // Still alive, but terminal: neither a partial output nor a retained reader.
    assert!(matches!(reader.next_page(), Err(SnapshotError::Closed)));
    assert!(matches!(reader.finish(), Err(SnapshotError::Closed)));
}

#[test]
fn non_wal_main_database_is_rejected_without_changing_its_mode() {
    let (_dir, shared) = fixture();
    shared
        .lock()
        .unwrap()
        .db
        .pragma_update(None, "journal_mode", "DELETE")
        .unwrap();
    assert!(matches!(
        shared.try_begin_status_snapshot(budget()),
        Err(SnapshotError::NotWal)
    ));
    let mode: String = shared
        .lock()
        .unwrap()
        .db
        .pragma_query_value(None, "journal_mode", |r| r.get(0))
        .unwrap();
    assert_eq!(mode, "delete");
}

#[test]
#[ignore = "explicit 210k metadata fixture; run in a coordinated functional-load window"]
fn default_budget_admits_210k_node_snapshot_without_holding_store() {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(dir.path()).unwrap();
    store.db.execute_batch("BEGIN").unwrap();
    {
        let mut insert = store.db.prepare("INSERT INTO nodes(id,parent,name,remote_version,directory) VALUES(?1,?2,?1,'v1',?3)").unwrap();
        insert
            .execute(rusqlite::params!["root", Option::<String>::None, true])
            .unwrap();
        for i in 0..210_000 {
            insert
                .execute(rusqlite::params![format!("file{i:06}"), "root", false])
                .unwrap();
        }
    }
    store.db.execute_batch("COMMIT").unwrap();
    let shared = SharedStore::new(store);
    let mut reader = shared
        .try_begin_status_snapshot(SnapshotBudget::default())
        .unwrap();
    loop {
        assert!(shared.try_lock().is_ok());
        if reader.next_page().unwrap().complete {
            break;
        }
    }
    eprintln!(
        "capacity fixture: nodes={} accounted_bytes={} work={}",
        reader.scanned, reader.bytes, reader.work
    );
    let completed = reader.finish().unwrap();
    assert_eq!(completed.snapshot.atoms.len(), 210_001);
}
