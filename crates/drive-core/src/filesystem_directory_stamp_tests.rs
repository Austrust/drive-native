use super::*;
use crate::status_changes::JournalBudget;
use crate::{Batch, Change, Event};

fn node(id: &str, parent: Option<&str>, name: &str, directory: bool) -> Node {
    Node {
        id: id.into(),
        parent: parent.map(str::to_owned),
        name: name.into(),
        directory,
        version: "v1".into(),
        metadata: Some(NodeMetadata {
            size: if directory { 0 } else { 3 },
            mtime: 1,
            mode: if directory { 0o755 } else { 0o644 },
        }),
    }
}
fn fixture() -> (tempfile::TempDir, Store) {
    let temp = tempfile::tempdir().unwrap();
    let mut store = Store::open(temp.path()).unwrap();
    store
        .install_remote_snapshot(
            &[
                node("root", None, "root", true),
                node("a", Some("root"), "a", false),
                node("d", Some("root"), "d", true),
                node("b", Some("d"), "b", false),
            ],
            "1",
            "server",
        )
        .unwrap();
    (temp, store)
}
fn names(store: &Store, ino: u64) -> Vec<String> {
    store
        .directory_snapshot(ino, 1 << 20)
        .unwrap()
        .entries
        .into_iter()
        .map(|e| e.name)
        .collect()
}

#[test]
fn inode_allocations_and_private_metadata_do_not_change_directory_stamp() {
    let (_temp, mut store) = fixture();
    let stamp = store.directory_view_stamp().unwrap();
    let old_total = store.local_revision();
    store.ensure_fs_root("root").unwrap();
    names(&store, 1);
    // Instance install already allocated every identity; this only projects.
    store.record("b").unwrap();
    store
        .claim_extension_marker("test:unrelated", "value")
        .unwrap();
    assert!(store.local_revision() > old_total);
    assert_eq!(store.directory_view_stamp().unwrap(), stamp);
    assert!(store.ensure_fs_root("d").is_err());
    assert_eq!(store.directory_view_stamp().unwrap(), stamp);
}

#[test]
fn rollback_keeps_stamp_but_open_transaction_or_unconfirmed_bulk_cannot_be_cached() {
    let (_temp, store) = fixture();
    let stamp = store.directory_view_stamp().unwrap();
    store
        .db
        .execute_batch("BEGIN; UPDATE nodes SET name='temporary' WHERE id='a';")
        .unwrap();
    assert!(store.directory_view_stamp().is_err());
    store.db.execute_batch("ROLLBACK;").unwrap();
    assert_eq!(store.directory_view_stamp().unwrap(), stamp);
    store
        .db
        .execute_batch("UPDATE dn_status_meta SET bulk=1;")
        .unwrap();
    assert!(store.directory_view_stamp().is_err());
    store
        .db
        .execute_batch("UPDATE dn_status_meta SET bulk=0;")
        .unwrap();
    assert_eq!(store.directory_view_stamp().unwrap(), stamp);
    store
        .db
        .execute_batch("UPDATE dn_status_meta SET head=-1;")
        .unwrap();
    assert!(store.directory_view_stamp().is_err());
}

#[test]
fn pruning_history_is_not_a_namespace_change_but_overflow_reset_and_reopen_are() {
    let (temp, mut store) = fixture();
    store.ensure_fs_root("root").unwrap();
    store
        .create_local_directory("root", "local", 0o755)
        .unwrap();
    let before = store.directory_view_stamp().unwrap();
    let cursor = store.status_change_watermark().unwrap().cursor;
    store.prune_status_changes(&cursor).unwrap();
    assert_eq!(store.directory_view_stamp().unwrap(), before);
    store
        .set_status_journal_budget(JournalBudget {
            entries: 1,
            bytes: 512,
        })
        .unwrap();
    store
        .create_local_directory("root", "other", 0o755)
        .unwrap();
    let overflow = store.directory_view_stamp().unwrap();
    assert!(overflow.overflows > before.overflows);
    assert_ne!(overflow, before);
    let nodes = [
        node("root", None, "root", true),
        node("a", Some("root"), "changed", false),
    ];
    store
        .install_remote_snapshot(&nodes, "2", "server")
        .unwrap();
    let reset = store.directory_view_stamp().unwrap();
    assert!(reset.snapshot_resets > overflow.snapshot_resets);
    assert_ne!(reset, overflow);
    drop(store);
    let store = Store::open(temp.path()).unwrap();
    assert_ne!(
        store.directory_view_stamp().unwrap().cursor.epoch,
        reset.cursor.epoch
    );
}

#[test]
fn overlay_acl_revocation_changes_stamp_and_hides_moved_descendant() {
    let (_temp, mut store) = fixture();
    store.ensure_fs_root("root").unwrap();
    store
        .rename_local("d", "b", "root", "moved", false)
        .unwrap();
    assert!(names(&store, 1).contains(&"moved".into()));
    let before = store.directory_view_stamp().unwrap();
    // Exercise the actual access_blocked dependency, without a snapshot reset
    // obscuring whether access_denied's own transactional trigger is covered.
    store
        .db
        .execute("INSERT INTO access_denied(node) VALUES('d')", [])
        .unwrap();
    assert_ne!(store.directory_view_stamp().unwrap(), before);
    assert!(!names(&store, 1).contains(&"moved".into()));
    let denied = store.directory_view_stamp().unwrap();
    store
        .db
        .execute("DELETE FROM access_denied WHERE node='d'", [])
        .unwrap();
    assert_ne!(store.directory_view_stamp().unwrap(), denied);
    assert!(names(&store, 1).contains(&"moved".into()));
}

#[test]
fn creation_echo_inode_deletion_is_covered_by_same_transaction_semantic_writes() {
    let (_temp, mut store) = fixture();
    store.ensure_fs_root("root").unwrap();
    let local = store.create_local_file("root", "created", 0o644).unwrap();
    let ino = store.record(&local.id).unwrap().ino;
    let op = store.save_local(&local.id, b"abc").unwrap();
    store.creation_saved(&local.id).unwrap();
    store.mark_upload_started(&op).unwrap();
    store
        .receive(&Batch {
            from: "1".into(),
            to: "2".into(),
            events: vec![Event {
                id: "echo".into(),
                change: Change::Upsert(node("99", Some("root"), "created", false)),
            }],
        })
        .unwrap();
    store.drain().unwrap();
    let echo_ino = store.record("99").unwrap().ino;
    assert_ne!(echo_ino, ino);
    let before = store.directory_view_stamp().unwrap();
    assert_eq!(
        names(&store, 1)
            .iter()
            .filter(|n| n.as_str() == "created")
            .count(),
        1
    );
    store
        .confirm_upload_identity(&op, "v1", Some("99"))
        .unwrap();
    assert_ne!(store.directory_view_stamp().unwrap(), before);
    assert!(store.record_by_inode(echo_ino).is_err());
    assert_eq!(store.lookup("root", "created").unwrap().ino, ino);
    assert_eq!(
        names(&store, 1)
            .iter()
            .filter(|n| n.as_str() == "created")
            .count(),
        1
    );
}

#[test]
fn unlinking_local_shadow_reveals_remote_entry_with_a_new_stamp() {
    let (_temp, mut store) = fixture();
    store.ensure_fs_root("root").unwrap();
    let local = store
        .create_local_file("root", "shared-name", 0o644)
        .unwrap();
    let local_ino = store.record(&local.id).unwrap().ino;
    store
        .receive(&Batch {
            from: "1".into(),
            to: "2".into(),
            events: vec![Event {
                id: "echo".into(),
                change: Change::Upsert(node("99", Some("root"), "shared-name", false)),
            }],
        })
        .unwrap();
    store.drain().unwrap();
    let before = store.directory_view_stamp().unwrap();
    assert_eq!(store.lookup("root", "shared-name").unwrap().ino, local_ino);
    store
        .unlink_local("root", "shared-name", &local.id, false)
        .unwrap();
    assert_ne!(store.directory_view_stamp().unwrap(), before);
    assert_eq!(store.lookup("root", "shared-name").unwrap().node.id, "99");
}
