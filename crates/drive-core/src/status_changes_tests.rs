use super::*;
use crate::{Batch, Change, DownloadTicket, Event, Node, NodeMetadata, Status, sha256};

fn node(id: &str, parent: Option<&str>, directory: bool) -> Node {
    Node {
        id: id.into(),
        parent: parent.map(str::to_owned),
        name: id.into(),
        directory,
        version: "v1".into(),
        metadata: Some(NodeMetadata {
            size: if directory { 0 } else { 3 },
            mtime: 1,
            mode: 0o644,
        }),
    }
}
fn fixture() -> (tempfile::TempDir, Store) {
    let dir = tempfile::tempdir().unwrap();
    let mut store = Store::open(dir.path()).unwrap();
    store
        .install_remote_snapshot(
            &[
                node("root", None, true),
                node("p", Some("root"), true),
                node("q", Some("root"), true),
                node("a", Some("p"), false),
                node("b", Some("q"), false),
            ],
            "1",
            "server",
        )
        .unwrap();
    store.ensure_fs_root("root").unwrap();
    let watermark = store.status_change_watermark().unwrap();
    store.prune_status_changes(&watermark.cursor).unwrap();
    (dir, store)
}
fn rows(store: &Store, after: &StatusCursor) -> Vec<StatusChange> {
    match store.read_status_changes(after, MAX_ENTRIES).unwrap() {
        ChangePage::Ready {
            changes,
            has_more: false,
            ..
        } => changes,
        other => panic!("expected complete changes: {other:?}"),
    }
}
fn consistent(store: &Store) {
    let m = store.status_change_watermark().unwrap();
    let (entries, bytes): (usize, usize) = store
        .db
        .query_row(
            "SELECT count(*),coalesce(sum(bytes),0) FROM dn_status_changes",
            [],
            |r| Ok((read_integer(r, 0)?, read_integer(r, 1)?)),
        )
        .unwrap();
    assert_eq!((m.entries, m.bytes), (entries, bytes));
    assert!(entries <= m.budget.entries && bytes <= m.budget.bytes);
}

#[test]
fn uncommitted_rows_are_unreadable_and_savepoint_rollback_restores_sequence_and_budget() {
    let (_dir, s) = fixture();
    let initial = s.status_change_watermark().unwrap();
    s.db.execute_batch("BEGIN; UPDATE nodes SET pinned=1 WHERE id='p'; SAVEPOINT edit; UPDATE nodes SET name='changed' WHERE id='a'; ROLLBACK TO edit; RELEASE edit;").unwrap();
    assert!(s.status_change_watermark().is_err());
    assert!(s.read_status_changes(&initial.cursor, 10).is_err());
    s.db.execute_batch("COMMIT;").unwrap();
    let changes = rows(&s, &initial.cursor);
    assert_eq!(changes.len(), 1);
    assert_eq!(changes[0].new_node.as_deref(), Some("p"));
    let committed = s.status_change_watermark().unwrap();
    s.db.execute_batch("BEGIN; UPDATE nodes SET name='other' WHERE id='a'; ROLLBACK;")
        .unwrap();
    assert_eq!(s.status_change_watermark().unwrap(), committed);
    assert!(rows(&s, &committed.cursor).is_empty());
    consistent(&s);
}

#[test]
fn committed_write_survives_a_later_outer_error_and_unchanged_updates_are_ignored() {
    let (_dir, mut s) = fixture();
    let start = s.status_change_watermark().unwrap().cursor;
    let partial = (|| -> Result<()> {
        s.pin("a", true)?;
        Err(Error::State("later diagnostic failed".into()))
    })();
    assert!(partial.is_err());
    assert_eq!(rows(&s, &start).len(), 1);
    let committed = s.status_change_watermark().unwrap();
    s.pin("a", true).unwrap();
    assert_eq!(s.status_change_watermark().unwrap(), committed);
}

#[test]
fn directory_content_and_conflict_updates_do_not_invalidate_descendants() {
    for update in [
        "remote_version='v2'",
        "local_version='v1'",
        "blob='content-token'",
        "dirty_blob='dirty-token'",
        "base_version='v1'",
        "dirty_generation=dirty_generation+1",
        "conflict=1",
    ] {
        let (_dir, s) = fixture();
        let start = s.status_change_watermark().unwrap().cursor;
        s.db.execute(&format!("UPDATE nodes SET {update} WHERE id='p'"), [])
            .unwrap();
        let changes = rows(&s, &start);
        assert_eq!(changes.len(), 1, "{update}");
        assert_eq!(changes[0].scope, ChangeScope::NodeAndAncestors, "{update}");
        assert_eq!(changes[0].new_node.as_deref(), Some("p"));
    }
}

#[test]
fn inherited_namespace_and_visibility_updates_still_invalidate_subtrees() {
    for update in [
        "id='renamed-id'",
        "parent='q'",
        "name='new-name'",
        "directory=0",
        "pinned=1",
        "deleted=1",
        "remote_version='v2', pinned=1",
    ] {
        let (_dir, s) = fixture();
        let start = s.status_change_watermark().unwrap().cursor;
        s.db.execute(&format!("UPDATE nodes SET {update} WHERE id='p'"), [])
            .unwrap();
        let changes = rows(&s, &start);
        assert_eq!(changes.len(), 1, "{update}");
        assert_eq!(changes[0].scope, ChangeScope::SubtreeAndAncestors, "{update}");
    }
    let (_dir, s) = fixture();
    let start = s.status_change_watermark().unwrap().cursor;
    s.db.execute("DELETE FROM nodes WHERE id='p'", []).unwrap();
    let changes = rows(&s, &start);
    assert_eq!(changes.len(), 1);
    assert_eq!(changes[0].scope, ChangeScope::SubtreeAndAncestors);
    assert_eq!(changes[0].old_parent.as_deref(), Some("root"));
}

#[test]
fn rename_and_atomic_replace_keep_both_identities_and_old_new_locations() {
    let (_dir, mut s) = fixture();
    let start = s.status_change_watermark().unwrap().cursor;
    s.rename_local("p", "a", "q", "renamed", true).unwrap();
    let changes = rows(&s, &start);
    assert!(changes.iter().any(|c| c.source == "nodes"
        && c.old_node.as_deref() == Some("a")
        && c.old_parent.as_deref() == Some("p")
        && c.new_parent.as_deref() == Some("q")
        && c.old_name.as_deref() == Some("a")
        && c.new_name.as_deref() == Some("renamed")));
    assert!(changes.iter().any(|c| c.source=="namespace_overlay" && c.scope==ChangeScope::SubtreeAndAncestors));
    // A separate clean fixture avoids replacing a source with a preceding move.
    let (_dir, mut s) = fixture();
    for id in ["a", "b"] {
        s.publish_download(
            &DownloadTicket {
                id: id.into(),
                version: "v1".into(),
            },
            b"abc",
            &sha256(b"abc"),
        )
        .unwrap();
    }
    let start = s.status_change_watermark().unwrap().cursor;
    s.replace_local("p", "a", "q", "b", "a", "b").unwrap();
    let changes = rows(&s, &start);
    for id in ["a", "b"] {
        assert!(
            changes
                .iter()
                .any(|c| c.old_node.as_deref() == Some(id) || c.new_node.as_deref() == Some(id))
        );
    }
    assert!(changes.iter().any(|c| c.old_parent.as_deref() == Some("p")));
    assert!(changes.iter().any(|c| c.new_parent.as_deref() == Some("q")));
    consistent(&s);
}

#[test]
fn creation_shadow_and_early_remote_identity_fold_are_observable() {
    let (_dir, mut s) = fixture();
    let local = s.create_local_file("p", "new", 0o644).unwrap();
    let op = s.save_local(&local.id, b"abc").unwrap();
    s.creation_saved(&local.id).unwrap();
    s.mark_upload_started(&op).unwrap();
    let start = s.status_change_watermark().unwrap().cursor;
    let mut remote = node("99", Some("p"), false);
    remote.name = "new".into();
    remote.version = "remote-v2".into();
    s.receive(&Batch {
        from: "1".into(),
        to: "2".into(),
        events: vec![Event {
            id: "echo".into(),
            change: Change::Upsert(remote),
        }],
    })
    .unwrap();
    s.drain().unwrap();
    assert_eq!(s.lookup_node("p", "new").unwrap().id, local.id);
    s.confirm_upload_identity(&op, "remote-v2", Some("99"))
        .unwrap();
    let changes = rows(&s, &start);
    assert!(changes.iter().any(|c| c.source == "nodes"
        && c.operation == "DELETE"
        && c.old_node.as_deref() == Some("99")
        && c.old_parent.as_deref() == Some("p")
        && c.old_name.as_deref() == Some("new")));
    assert!(changes.iter().any(|c| c.source == "local_creations"
        && c.operation == "DELETE"
        && c.old_node.as_deref() == Some(&local.id)));
    assert!(
        changes
            .iter()
            .any(|c| c.source == "remote_aliases" && c.new_node.as_deref() == Some(&local.id))
    );
    assert_eq!(s.status(&local.id).unwrap(), Status::LocalComplete);
}

#[test]
fn permission_and_overlay_changes_invalidate_descendants_and_replace_captures_delete() {
    let (_dir, s) = fixture();
    let start = s.status_change_watermark().unwrap().cursor;
    s.db.execute("INSERT INTO access_denied VALUES('p')", [])
        .unwrap();
    assert!(s.node("a").is_err());
    s.db.execute("DELETE FROM access_denied WHERE node='p'", [])
        .unwrap();
    s.db.execute("INSERT INTO namespace_overlay VALUES('p','null')", [])
        .unwrap();
    s.db.execute(
        "INSERT OR REPLACE INTO namespace_overlay VALUES('p','{}')",
        [],
    )
    .unwrap();
    let changes = rows(&s, &start);
    assert_eq!(
        changes
            .iter()
            .filter(|c| c.source == "access_denied")
            .count(),
        2
    );
    assert_eq!(
        changes
            .iter()
            .filter(|c| c.source == "namespace_overlay")
            .map(|c| c.operation.as_str())
            .collect::<Vec<_>>(),
        ["INSERT", "DELETE", "INSERT"]
    );
    assert!(
        changes
            .iter()
            .all(|c| c.scope == ChangeScope::SubtreeAndAncestors)
    );
    assert!(
        changes
            .iter()
            .all(|c| c.old_node.as_deref() == Some("p") || c.new_node.as_deref() == Some("p"))
    );
    consistent(&s);
}

#[test]
fn related_status_tables_emit_and_download_error_clearing_is_not_lost() {
    let (_dir, s) = fixture();
    let start = s.status_change_watermark().unwrap().cursor;
    s.db.execute_batch(
        "INSERT INTO drafts VALUES('a','token','v1',0,0,0);
        UPDATE drafts SET dirty=1,generation=1 WHERE node='a';
        INSERT INTO transfers VALUES('t','a','v1','download','failed','{}');
        UPDATE transfers SET state='pending',error=NULL WHERE id='t';
        INSERT INTO local_unlinks VALUES('a',0);
        INSERT INTO operations VALUES('o','a','upload','v1',NULL,1,'pending');
        UPDATE operations SET state='uncertain' WHERE id='o';
        INSERT OR REPLACE INTO node_metadata VALUES('a',1,1,420);
        INSERT INTO local_modes VALUES('a',384);
        INSERT INTO cache_releases VALUES('a','v1','pending',0,0);
        INSERT INTO content_version_aliases VALUES('a','v1','v2');
        INSERT INTO engine_bridge_intents(key,node,body,phase) VALUES('e','a','{}','prepared');
        INSERT INTO engine_bridge_tombstones VALUES('a','1');",
    )
    .unwrap();
    let changes = rows(&s, &start);
    for source in [
        "drafts",
        "transfers",
        "local_unlinks",
        "operations",
        "node_metadata",
        "local_modes",
        "cache_releases",
        "content_version_aliases",
        "engine_bridge_intents",
        "engine_bridge_tombstones",
    ] {
        assert!(
            changes.iter().any(|c| c.source == source),
            "missing source {source}"
        );
    }
    assert_eq!(
        changes.iter().filter(|c| c.source == "transfers").count(),
        2
    );
    consistent(&s);
}

#[test]
fn pruning_overflow_and_oversized_rows_report_explicit_gaps_with_bounded_accounting() {
    let (_dir, mut s) = fixture();
    s.set_status_journal_budget(JournalBudget {
        entries: 2,
        bytes: 4096,
    })
    .unwrap();
    let start = s.status_change_watermark().unwrap().cursor;
    s.pin("a", true).unwrap();
    let first = s.status_change_watermark().unwrap().cursor;
    s.pin("a", false).unwrap();
    match s.read_status_changes(&start, 1).unwrap() {
        ChangePage::Ready {
            changes,
            next,
            has_more: true,
            ..
        } => {
            assert_eq!(changes.len(), 1);
            assert_eq!(next, first);
        }
        other => panic!("{other:?}"),
    }
    s.prune_status_changes(&first).unwrap();
    assert!(matches!(
        s.read_status_changes(&start, 10).unwrap(),
        ChangePage::Rebuild {
            reason: GapReason::HistoryDiscarded,
            ..
        }
    ));
    assert_eq!(rows(&s, &first).len(), 1);
    s.pin("a", true).unwrap();
    s.pin("a", false).unwrap(); // third retained row exceeds count budget
    let overflow = s.status_change_watermark().unwrap();
    assert_eq!(overflow.entries, 0);
    assert_eq!(overflow.bytes, 0);
    assert_eq!(overflow.retained_after, overflow.cursor.sequence);
    assert_eq!(overflow.overflows, 1);
    assert!(matches!(
        s.read_status_changes(&first, 10).unwrap(),
        ChangePage::Rebuild { .. }
    ));
    s.set_status_journal_budget(JournalBudget {
        entries: 2,
        bytes: 1,
    })
    .unwrap();
    s.pin("a", true).unwrap(); // one oversized row also advances the gap watermark
    assert_eq!(s.status_change_watermark().unwrap().overflows, 2);
    consistent(&s);
    s.set_status_journal_budget(JournalBudget {
        entries: 2,
        bytes: 4096,
    })
    .unwrap();
    let start = s.status_change_watermark().unwrap().cursor;
    s.pin("a", false).unwrap();
    assert_eq!(rows(&s, &start).len(), 1);
    let before = s.status_change_watermark().unwrap();
    s.db.execute_batch("BEGIN; UPDATE nodes SET pinned=1 WHERE id='a'; UPDATE nodes SET pinned=0 WHERE id='a'; ROLLBACK;").unwrap();
    assert_eq!(s.status_change_watermark().unwrap(), before); // overflow itself rolled back
}

#[test]
fn budget_shrink_invalidates_and_cursor_ahead_is_rejected() {
    let (_dir, mut s) = fixture();
    let start = s.status_change_watermark().unwrap().cursor;
    s.pin("a", true).unwrap();
    s.pin("a", false).unwrap();
    s.set_status_journal_budget(JournalBudget {
        entries: 1,
        bytes: 4096,
    })
    .unwrap();
    assert!(matches!(
        s.read_status_changes(&start, 1).unwrap(),
        ChangePage::Rebuild {
            reason: GapReason::HistoryDiscarded,
            ..
        }
    ));
    let mut ahead = s.status_change_watermark().unwrap().cursor;
    ahead.sequence += 1;
    assert!(matches!(
        s.read_status_changes(&ahead, 1).unwrap(),
        ChangePage::Rebuild {
            reason: GapReason::CursorAhead,
            ..
        }
    ));
    assert!(s.prune_status_changes(&ahead).is_err());
    assert!(
        s.set_status_journal_budget(JournalBudget {
            entries: 0,
            bytes: 1
        })
        .is_err()
    );
    consistent(&s);
}

#[test]
fn epoch_changes_on_reopen_and_main_schema_external_readers_remain_unchanged() {
    let (dir, mut s) = fixture();
    s.pin("a", true).unwrap();
    let before = s.status_change_watermark().unwrap().cursor;
    let external = Connection::open_with_flags(
        dir.path().join("state.sqlite"),
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    )
    .unwrap();
    let schema: i64 = external
        .pragma_query_value(None, "user_version", |r| r.get(0))
        .unwrap();
    assert_eq!(schema, crate::STATE_SCHEMA_VERSION);
    let names: i64 = external
        .query_row(
            "SELECT count(*) FROM sqlite_master WHERE name LIKE 'dn_status_%'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(names, 0);
    assert!(external.prepare("SELECT * FROM dn_status_changes").is_err());
    assert_eq!(
        external
            .query_row("SELECT pinned FROM nodes WHERE id='a'", [], |r| r
                .get::<_, i64>(0))
            .unwrap(),
        1
    );
    drop(s);
    let s = Store::open(dir.path()).unwrap();
    assert_eq!(s.status_change_watermark().unwrap().cursor.sequence, 0);
    assert_ne!(
        s.status_change_watermark().unwrap().cursor.epoch,
        before.epoch
    );
    assert!(matches!(
        s.read_status_changes(&before, 10).unwrap(),
        ChangePage::Rebuild {
            reason: GapReason::EpochChanged,
            ..
        }
    ));
    assert!(s.is_pinned("a").unwrap());
}

#[test]
fn inode_lru_settings_and_received_but_unapplied_events_do_not_change_status_watermark() {
    let (_dir, mut s) = fixture();
    let before = s.status_change_watermark().unwrap();
    s.record("a").unwrap();
    s.directory_snapshot(1, 1024 * 1024).unwrap();
    s.note_cache_use("a").unwrap();
    s.set_cache_limit(123456).unwrap();
    s.receive(&Batch {
        from: "1".into(),
        to: "2".into(),
        events: vec![Event {
            id: "change".into(),
            change: Change::Upsert(node("new", Some("p"), false)),
        }],
    })
    .unwrap();
    assert_eq!(s.status_change_watermark().unwrap(), before);
    s.drain().unwrap();
    assert!(s.status_change_watermark().unwrap().cursor.sequence > before.cursor.sequence);
}

#[test]
fn recursive_remote_delete_records_every_affected_identity() {
    let (_dir, mut s) = fixture();
    let start = s.status_change_watermark().unwrap().cursor;
    s.receive(&Batch {
        from: "1".into(),
        to: "2".into(),
        events: vec![Event {
            id: "delete-tree".into(),
            change: Change::Delete { id: "p".into() },
        }],
    })
    .unwrap();
    s.drain().unwrap();
    let changes = rows(&s, &start);
    for (id, parent) in [("p", "root"), ("a", "p")] {
        assert!(changes.iter().any(|c| c.source == "nodes"
            && c.old_node.as_deref() == Some(id)
            && c.old_parent.as_deref() == Some(parent)));
    }
    assert!(s.node("a").is_err());
    consistent(&s);
}

#[test]
fn statement_failure_rolls_back_trigger_rows_and_read_only_queries_do_not_write() {
    let (_dir, s) = fixture();
    let start = s.status_change_watermark().unwrap();
    // A late row violates NOT NULL after earlier rows may have fired triggers.
    assert!(
        s.db.execute(
            "UPDATE nodes SET name=CASE WHEN id='q' THEN NULL ELSE name||'-x' END",
            []
        )
        .is_err()
    );
    assert_eq!(s.status_change_watermark().unwrap(), start);
    let revision = s.local_revision();
    assert!(rows(&s, &start.cursor).is_empty());
    assert_eq!(s.local_revision(), revision);
    consistent(&s);
}

#[test]
fn schema_inventory_requires_an_explicit_decision_for_new_tables_and_columns() {
    use std::collections::BTreeSet;
    let (_dir, s) = fixture();
    let excluded = [
        "metadata",
        "inbox",
        "seen",
        "fs_inodes",
        "upload_origins",
        "upload_paths",
        "cache_access",
        "cache_published",
        "engine_upload_stages",
        "sqlite_sequence",
    ];
    let expected: BTreeSet<String> = SOURCES
        .iter()
        .map(|x| x.0)
        .chain(excluded)
        .map(str::to_owned)
        .collect();
    let actual =
        s.db.prepare("SELECT name FROM main.sqlite_master WHERE type='table'")
            .unwrap()
            .query_map([], |r| r.get::<_, String>(0))
            .unwrap()
            .collect::<rusqlite::Result<BTreeSet<_>>>()
            .unwrap();
    assert_eq!(
        actual, expected,
        "classify new durable tables before relying on the journal"
    );
    for &(table, _, columns, _) in SOURCES {
        let ignored: &[&str] = match table {
            "transfers" | "operations" => &["id"],
            "namespace_ops" => &["seq", "id"],
            "engine_bridge_intents" => &["key", "body", "baseline"],
            _ => &[],
        };
        let expected: BTreeSet<String> = columns
            .iter()
            .chain(ignored)
            .map(|s| (*s).to_owned())
            .collect();
        let actual =
            s.db.prepare(&format!("PRAGMA main.table_info({table})"))
                .unwrap()
                .query_map([], |r| r.get::<_, String>(1))
                .unwrap()
                .collect::<rusqlite::Result<BTreeSet<_>>>()
                .unwrap();
        assert_eq!(actual, expected, "classify new columns in {table}");
    }
}

#[test]
fn repeated_budget_and_acknowledgement_do_not_create_coarse_notifications() {
    let (_dir, mut s) = fixture();
    s.pin("a", true).unwrap();
    let watermark = s.status_change_watermark().unwrap();
    let revision = s.local_revision();
    s.set_status_journal_budget(watermark.budget).unwrap();
    assert_eq!(s.local_revision(), revision);
    assert_eq!(s.status_change_watermark().unwrap(), watermark);
    s.prune_status_changes(&watermark.cursor).unwrap();
    let revision = s.local_revision();
    s.prune_status_changes(&watermark.cursor).unwrap();
    s.set_status_journal_budget(watermark.budget).unwrap();
    assert_eq!(s.local_revision(), revision);
    // A real budget change still occurs, but is not a new semantic row change.
    let mut changed = watermark.budget;
    changed.entries -= 1;
    s.set_status_journal_budget(changed).unwrap();
    assert!(s.local_revision() > revision);
    assert_eq!(
        s.status_change_watermark().unwrap().cursor,
        watermark.cursor
    );
}
