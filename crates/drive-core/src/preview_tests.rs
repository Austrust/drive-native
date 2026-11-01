use super::*;
use crate::NodeMetadata;
use crate::initialization_events::InitializationDirectoryEntry;
use crate::initialization_store::{InitializationBinding, InitializationBudget, InitializationRun};

fn scope() -> crate::native::Scope {
    crate::native::Scope {
        view_id: 1,
        node_id: 2,
        file_id: "100".into(),
        path: "/Lab".into(),
    }
}
fn node(id: &str, parent: Option<&str>, name: &str, directory: bool) -> Node {
    Node{id:id.into(),parent:parent.map(str::to_owned),name:name.into(),directory,
        version:serde_json::json!({"schema":1,"restore":"server","view":1,"root":"100","file":id,"sync":7,"max":7,"size":3,"hash":"synthetic","directory":directory}).to_string(),
        metadata:Some(NodeMetadata{size:3,mtime:7,mode:if directory{0o755}else{0o644}})}
}
fn root() -> Node {
    node("100", None, "Lab", true)
}
fn dir() -> Node {
    node("200", Some("100"), "Dir", true)
}
fn file() -> Node {
    node("101", Some("100"), "A.doc", false)
}
fn entry(node: Node, native_node_id: u64) -> InitializationDirectoryEntry {
    InitializationDirectoryEntry {
        node,
        native_node_id,
    }
}
fn setup() -> (
    tempfile::TempDir,
    Store,
    InitializationBinding,
    InitializationRun,
) {
    let temp = tempfile::tempdir().unwrap();
    let mut store = Store::open(temp.path()).unwrap();
    store
        .bind_native_scope("server", &serde_json::to_string(&scope()).unwrap())
        .unwrap();
    let binding = InitializationBinding {
        task_id: "preview-task".into(),
        scope: serde_json::to_value(scope()).unwrap(),
        restore: "server".into(),
        start_cursor: "native-v1:100".into(),
        expected_cursor: "".into(),
    };
    let run = store
        .begin_initialization(&binding, &root(), InitializationBudget::default())
        .unwrap();
    (temp, store, binding, run)
}
fn list(
    store: &mut Store,
    run: &InitializationRun,
    id: &str,
    entries: &[InitializationDirectoryEntry],
) {
    let ticket = store.begin_initialization_directory(run, id).unwrap();
    store
        .store_initialization_native_directory(&ticket, 1, entries)
        .unwrap();
}
fn ready_root(store: &mut Store, run: &InitializationRun) {
    list(store, run, "100", &[entry(file(), 51), entry(dir(), 71)]);
}
fn errno<T: std::fmt::Debug>(result: Result<T>, expected: i32) {
    assert!(
        matches!(result,Err(Error::Io(ref e)) if e.raw_os_error()==Some(expected)),
        "{result:?}"
    );
}
fn finish(store: &mut Store, run: &InitializationRun) {
    let ticket = store.begin_initialization_fence(run).unwrap();
    store
        .confirm_initialization_fence(&ticket, "native-v1:100")
        .unwrap();
}

#[test]
fn root_can_mount_before_list_without_authoritative_node_or_false_empty() {
    let (_temp, store, _binding, _run) = setup();
    let status = store.status_change_watermark().unwrap();
    assert!(store.preview_available("100").unwrap());
    assert_eq!(
        store
            .db
            .query_row("SELECT count(*) FROM fs_inodes", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        0
    );
    assert!(!store.preview_available("other").unwrap());
    let root = store.preview_mount_root("100").unwrap().unwrap();
    assert_eq!(root.ino, 1);
    assert_eq!(root.mode, 0o555);
    assert!(store.node("100").is_err());
    assert!(store.record_by_inode(1).is_err());
    errno(store.preview_directory_snapshot(1, 1024), libc::EAGAIN);
    errno(store.preview_lookup(1, "missing"), libc::EAGAIN);
    errno(store.preview_children_page(1, 0, 10), libc::EAGAIN);
    errno(store.preview_record_by_inode(99), libc::ENOENT);
    assert_eq!(store.cursor().unwrap(), "");
    assert_eq!(store.status_change_watermark().unwrap(), status);
}

#[test]
fn received_parent_exposes_metadata_but_pending_child_is_not_empty() {
    let (_temp, mut store, _binding, run) = setup();
    store.preview_mount_root("100").unwrap();
    ready_root(&mut store, &run);
    let snap = store.preview_directory_snapshot(1, 64 * 1024).unwrap();
    assert_eq!(
        snap.snapshot
            .entries
            .iter()
            .map(|e| e.name.as_str())
            .collect::<Vec<_>>(),
        [".", "..", "A.doc", "Dir"]
    );
    store.preview_validate_view(&snap.stamp).unwrap();
    let d = store.preview_lookup(1, "Dir").unwrap();
    let f = store.preview_lookup(1, "A.doc").unwrap();
    assert_eq!((f.size, f.mode), (3, 0o444));
    assert!(store.node(&f.id).is_err());
    errno(store.preview_directory_snapshot(d.ino, 1024), libc::EAGAIN);
    errno(store.preview_lookup(1, "missing"), libc::ENOENT);
    list(&mut store, &run, "200", &[]);
    let empty = store.preview_directory_snapshot(d.ino, 1024).unwrap();
    assert_eq!(empty.snapshot.entries.len(), 2);
    // Finishing an unrelated child's list does not revoke the held root view.
    store.preview_validate_view(&snap.stamp).unwrap();
}

#[test]
fn ancestry_invalidation_and_denial_revoke_held_views_and_hide_children() {
    let (_temp, mut store, _binding, run) = setup();
    store.preview_mount_root("100").unwrap();
    ready_root(&mut store, &run);
    list(
        &mut store,
        &run,
        "200",
        &[entry(node("201", Some("200"), "Child", true), 81)],
    );
    let d = store.preview_lookup(1, "Dir").unwrap();
    let child = store.preview_lookup(d.ino, "Child").unwrap();
    let root_view = store.preview_directory_snapshot(1, 4096).unwrap();
    let snap = store.preview_directory_snapshot(d.ino, 1024).unwrap();
    let ticket = store.begin_initialization_directory(&run, "200").unwrap();
    errno(store.preview_validate_view(&snap.stamp), libc::ESTALE);
    errno(store.preview_record_by_inode(child.ino), libc::EAGAIN);
    store.store_initialization_denied(&ticket, 1).unwrap();
    errno(store.preview_validate_view(&root_view.stamp), libc::ESTALE);
    errno(store.preview_record_by_inode(d.ino), libc::EACCES);
    errno(store.preview_lookup(1, "Dir"), libc::EACCES);
    errno(store.preview_validate_view(&snap.stamp), libc::EACCES);
    assert_eq!(store.preview_children_page(1, 0, 256).unwrap().total, 1);
    assert!(store.node("100").is_err());
}

#[test]
fn rename_keeps_inode_but_changed_numeric_identity_and_name_replacement_do_not() {
    let (_temp, mut store, _binding, run) = setup();
    store.preview_mount_root("100").unwrap();
    ready_root(&mut store, &run);
    let old = store.preview_lookup(1, "Dir").unwrap();
    let mut renamed = dir();
    renamed.name = "Renamed".into();
    list(
        &mut store,
        &run,
        "100",
        &[entry(file(), 51), entry(renamed.clone(), 71)],
    );
    assert_eq!(store.preview_lookup(1, "Renamed").unwrap().ino, old.ino);
    list(
        &mut store,
        &run,
        "100",
        &[entry(file(), 51), entry(renamed, 72)],
    );
    let newer = store.preview_lookup(1, "Renamed").unwrap();
    assert_ne!(newer.ino, old.ino);
    errno(store.preview_record_by_inode(old.ino), libc::ESTALE);
    list(
        &mut store,
        &run,
        "100",
        &[
            entry(file(), 51),
            entry(node("300", Some("100"), "Renamed", true), 73),
        ],
    );
    assert_ne!(store.preview_lookup(1, "Renamed").unwrap().ino, newer.ino);
    errno(store.preview_record_by_inode(newer.ino), libc::ESTALE);
}

#[test]
fn page_generation_and_budget_do_not_publish_partial_views_or_allocate_on_failure() {
    let (_temp, mut store, _binding, run) = setup();
    store.preview_mount_root("100").unwrap();
    ready_root(&mut store, &run);
    let before = store
        .db
        .query_row("SELECT count(*) FROM fs_inodes", [], |r| r.get::<_, i64>(0))
        .unwrap();
    errno(store.preview_directory_snapshot(1, 1), libc::ENOMEM);
    assert_eq!(
        store
            .db
            .query_row("SELECT count(*) FROM fs_inodes", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        before
    );
    errno(store.preview_children_page(1, 0, 257), libc::EINVAL);
    let first = store.preview_children_page(1, 0, 1).unwrap();
    let second = store.preview_children_page(1, 1, 1).unwrap();
    assert_eq!(first.total, 2);
    assert_eq!(first.entries.len(), 1);
    assert_eq!(first.stamp.fingerprint(), second.stamp.fingerprint());
    ready_root(&mut store, &run);
    let changed = store.preview_children_page(1, 1, 1).unwrap();
    assert_ne!(first.stamp.fingerprint(), changed.stamp.fingerprint());
    errno(store.preview_validate_view(&first.stamp), libc::ESTALE);
}

#[test]
fn reopen_keeps_mappings_but_revokes_old_epoch_and_revalidates_binding() {
    let (temp, mut store, binding, run) = setup();
    store.preview_mount_root("100").unwrap();
    ready_root(&mut store, &run);
    let old = store.preview_directory_snapshot(1, 4096).unwrap();
    let inode = store.preview_lookup(1, "A.doc").unwrap().ino;
    drop(store);
    let mut store = Store::open(temp.path()).unwrap();
    let next = store.initialization_status().unwrap().unwrap().run;
    errno(store.preview_validate_view(&old.stamp), libc::ESTALE);
    assert_eq!(store.preview_lookup(1, "A.doc").unwrap().ino, inode);
    let resumed = store.resume_initialization(&next, &binding).unwrap();
    assert!(resumed.epoch > next.epoch);
    let mut wrong = binding.clone();
    wrong.task_id = "different".into();
    store
        .db
        .execute(
            "UPDATE initialization_run SET binding=?1",
            [serde_json::to_string(&wrong).unwrap()],
        )
        .unwrap();
    assert!(store.preview_available("100").is_err());
}

#[test]
fn complete_promotes_exact_inodes_in_same_authoritative_commit() {
    let (_temp, mut store, _binding, run) = setup();
    store.preview_mount_root("100").unwrap();
    ready_root(&mut store, &run);
    list(&mut store, &run, "200", &[]);
    let old = store.preview_directory_snapshot(1, 4096).unwrap();
    let file = store.preview_lookup(1, "A.doc").unwrap();
    let d = store.preview_lookup(1, "Dir").unwrap();
    finish(&mut store, &run);
    store.complete_initialization(&run).unwrap();
    assert_eq!(store.record_by_inode(1).unwrap().node.id, "100");
    assert_eq!(store.record_by_inode(file.ino).unwrap().node.id, "101");
    assert_eq!(store.record_by_inode(d.ino).unwrap().node.id, "200");
    assert_eq!(store.cursor().unwrap(), "native-v1:100");
    assert!(!store.initialization_writeback_blocked().unwrap());
    assert!(!store.preview_available("100").unwrap());
    errno(store.preview_validate_view(&old.stamp), libc::ESTALE);
}

#[test]
fn final_ready_failure_rolls_back_namespace_cursor_and_inode_promotion_together() {
    let (_temp, mut store, _binding, run) = setup();
    store.preview_mount_root("100").unwrap();
    ready_root(&mut store, &run);
    list(&mut store, &run, "200", &[]);
    let inode = store.preview_lookup(1, "A.doc").unwrap().ino;
    finish(&mut store, &run);
    store.db.execute_batch("CREATE TEMP TRIGGER fail_preview_ready BEFORE UPDATE OF state ON initialization_run WHEN new.state='ready' BEGIN SELECT RAISE(ABORT,'synthetic promotion failure'); END;").unwrap();
    assert!(store.complete_initialization(&run).is_err());
    assert!(store.node("100").is_err());
    assert_eq!(store.cursor().unwrap(), "");
    assert!(store.initialization_writeback_blocked().unwrap());
    assert_eq!(store.preview_lookup(1, "A.doc").unwrap().ino, inode);
    assert!(store.record_by_inode(inode).is_err());
    store
        .db
        .execute_batch("DROP TRIGGER fail_preview_ready")
        .unwrap();
    store.complete_initialization(&run).unwrap();
    assert_eq!(store.record_by_inode(inode).unwrap().node.id, "101");
}

#[test]
fn restarted_run_rebinds_root_without_gui_and_can_commit_without_any_new_reader() {
    for read_root in [false, true] {
        let (_temp, mut store, binding, run) = setup();
        store.preview_mount_root("100").unwrap();
        ready_root(&mut store, &run);
        let old = store.preview_directory_snapshot(1, 4096).unwrap();
        let old_file = store.preview_lookup(1, "A.doc").unwrap();
        let next = store
            .restart_initialization(&run, &binding, &root(), InitializationBudget::default())
            .unwrap();
        errno(store.preview_validate_view(&old.stamp), libc::ESTALE);
        errno(store.preview_record_by_inode(old_file.ino), libc::ESTALE);
        if read_root {
            let root = store.preview_record_by_inode(1).unwrap();
            assert_eq!(root.origin.run_id, next.run_id);
            errno(store.preview_directory_snapshot(1, 4096), libc::EAGAIN);
        }
        ready_root(&mut store, &next);
        list(&mut store, &next, "200", &[]);
        finish(&mut store, &next);
        store.complete_initialization(&next).unwrap();
        assert_eq!(store.record_by_inode(1).unwrap().node.id, "100");
        assert!(store.record_by_inode(old_file.ino).is_err());
        assert_eq!(store.cursor().unwrap(), "native-v1:100");
    }
}

#[test]
fn reservation_limit_and_visibility_overflow_fail_atomically() {
    let (_temp, mut store, _binding, run) = setup();
    store.preview_mount_root("100").unwrap();
    ready_root(&mut store, &run);
    let before = store
        .db
        .query_row("SELECT count(*) FROM fs_inodes", [], |r| r.get::<_, i64>(0))
        .unwrap();
    store
        .db
        .execute("UPDATE preview_mount SET allocated=500000", [])
        .unwrap();
    assert!(store.preview_lookup(1, "A.doc").is_err());
    assert_eq!(
        store
            .db
            .query_row("SELECT count(*) FROM fs_inodes", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        before
    );
    store
        .db
        .execute("UPDATE preview_mount SET allocated=?1", [before])
        .unwrap();
    let root_view = store.preview_directory_snapshot(1, 4096).unwrap();
    store
        .db
        .execute(
            "INSERT INTO preview_visibility(run,id,revision) VALUES(?1,'100',9223372036854775807)",
            [&run.run_id],
        )
        .unwrap();
    let ticket = store.begin_initialization_directory(&run, "200").unwrap();
    assert!(store.store_initialization_denied(&ticket, 1).is_err());
    assert!(
        store
            .initialization_directory_ticket_current(&ticket)
            .unwrap()
    );
    assert_eq!(
        store
            .initialization_status()
            .unwrap()
            .unwrap()
            .applied_sequence,
        0
    );
    // Artificial overflow state differs from the captured stamp, never reuses it.
    errno(store.preview_validate_view(&root_view.stamp), libc::ESTALE);
}

#[test]
fn populated_or_damaged_authoritative_state_is_never_cold_preview() {
    for sql in [
        "UPDATE metadata SET value='native-v1:7' WHERE key='committed_cursor'",
        "INSERT INTO fs_inodes(ino,node) VALUES(1,'missing-authoritative-root')",
        "INSERT INTO drafts VALUES('missing','token','base',1,0,1)",
        "INSERT INTO operations VALUES('op','missing','upload','base',NULL,1,'uncertain')",
    ] {
        let (_temp, store, _binding, _run) = setup();
        store.db.execute_batch(sql).unwrap();
        assert!(!store.preview_available("100").unwrap());
        assert!(store.preview_mount_root("100").unwrap().is_none());
    }
}

#[test]
fn schema21_migration_backs_up_private_candidates_and_future_reader_rejects() {
    let (temp, store, _binding, _run) = setup();
    store.db.execute_batch("DROP TRIGGER preview_denied_capture; DROP TRIGGER preview_denied_clear; DROP TABLE preview_denied; DROP TABLE preview_visibility; DROP TABLE preview_inodes; DROP TABLE preview_mount; PRAGMA user_version=21;").unwrap();
    drop(store);
    let store = Store::open(temp.path()).unwrap();
    assert!(store.preview_available("100").unwrap());
    let backup = std::fs::read_dir(temp.path())
        .unwrap()
        .map(|e| e.unwrap().path())
        .find(|p| {
            p.file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("state-before-v22-")
        })
        .unwrap();
    let old = Connection::open(backup).unwrap();
    assert_eq!(
        old.pragma_query_value(None, "user_version", |r| r.get::<_, i64>(0))
            .unwrap(),
        21
    );
    assert_eq!(
        old.query_row(
            "SELECT count(*) FROM sqlite_master WHERE name='preview_mount'",
            [],
            |r| r.get::<_, i64>(0)
        )
        .unwrap(),
        0
    );
    store
        .db
        .pragma_update(None, "user_version", crate::STATE_SCHEMA_VERSION + 1)
        .unwrap();
    drop(store);
    assert!(
        matches!(Store::open(temp.path()),Err(Error::State(m)) if m=="database is newer than this build")
    );
}

#[test]
fn warm_schema21_migration_preserves_dirty_content_and_never_enables_preview() {
    let temp = tempfile::tempdir().unwrap();
    let mut store = Store::open(temp.path()).unwrap();
    store
        .bind_native_scope("server", &serde_json::to_string(&scope()).unwrap())
        .unwrap();
    store
        .install_native_snapshot(
            &crate::native::RemoteSnapshot {
                scope: scope(),
                restore: "server".into(),
                cursor: "native-v1:7".into(),
                nodes: vec![root(), file()],
                inaccessible: vec![],
                skipped_symlinks: 0,
            },
            "",
        )
        .unwrap();
    store.ensure_fs_root("100").unwrap();
    let ino = store.record("101").unwrap().ino;
    let draft = store.begin_draft("101", &file().version, None).unwrap();
    store.draft_write(&draft, 0, b"dirty", false).unwrap();
    store.checkpoint_draft(&draft).unwrap();
    drop(draft);
    let pending = store.pending_operations().unwrap();
    store.db.execute_batch("DROP TRIGGER preview_denied_capture; DROP TRIGGER preview_denied_clear; DROP TABLE preview_denied; DROP TABLE preview_visibility; DROP TABLE preview_inodes; DROP TABLE preview_mount; PRAGMA user_version=21;").unwrap();
    drop(store);
    let store = Store::open(temp.path()).unwrap();
    assert!(!store.preview_available("100").unwrap());
    assert!(store.preview_mount_root("100").unwrap().is_none());
    assert_eq!(store.record_by_inode(ino).unwrap().node.id, "101");
    assert_eq!(store.pending_operations().unwrap(), pending);
    let draft = store.open_dirty_draft("101").unwrap().unwrap();
    let mut bytes = [0; 5];
    draft.read_at(&mut bytes, 0).unwrap();
    assert_eq!(&bytes, b"dirty");
}

#[test]
fn denied_directory_stays_hidden_through_retry_until_new_successful_list() {
    let (_temp, mut store, _binding, run) = setup();
    store.preview_mount_root("100").unwrap();
    ready_root(&mut store, &run);
    let d = store.preview_lookup(1, "Dir").unwrap();
    let ticket = store.begin_initialization_directory(&run, "200").unwrap();
    store.store_initialization_denied(&ticket, 1).unwrap();
    let hidden = store.preview_directory_snapshot(1, 4096).unwrap();
    assert_eq!(hidden.snapshot.entries.len(), 3);
    let retry = store.begin_initialization_directory(&run, "200").unwrap();
    errno(store.preview_record_by_inode(d.ino), libc::EACCES);
    errno(store.preview_lookup(1, "Dir"), libc::EACCES);
    assert_eq!(store.preview_children_page(1, 0, 256).unwrap().total, 1);
    store.preview_validate_view(&hidden.stamp).unwrap();
    store
        .store_initialization_native_directory(&retry, 1, &[])
        .unwrap();
    errno(store.preview_validate_view(&hidden.stamp), libc::ESTALE);
    assert_eq!(store.preview_lookup(1, "Dir").unwrap().ino, d.ino);
}

#[test]
fn confirmed_root_denial_survives_failure_resume_and_reopen_until_verified_reply() {
    let (temp, mut store, binding, run) = setup();
    store.preview_mount_root("100").unwrap();
    ready_root(&mut store, &run);
    let old = store.preview_directory_snapshot(1, 4096).unwrap();
    store.revoke_initialization_preview_root(&run).unwrap();
    store.fail_initialization(&run, "root_unavailable").unwrap();
    errno(store.preview_record_by_inode(1), libc::EACCES);
    errno(store.preview_validate_view(&old.stamp), libc::EACCES);
    drop(store);
    let mut store = Store::open(temp.path()).unwrap();
    let recovered = store.initialization_status().unwrap().unwrap().run;
    let resumed = store.resume_initialization(&recovered, &binding).unwrap();
    errno(store.preview_lookup(1, "A.doc"), libc::EACCES);
    let verify = store.begin_initialization_root_refresh(&resumed).unwrap();
    errno(store.preview_record_by_inode(1), libc::EACCES);
    store.refresh_initialization_root(&verify, &root()).unwrap();
    assert_eq!(
        store.preview_record_by_inode(1).unwrap().origin.epoch,
        resumed.epoch
    );
    assert!(store.preview_lookup(1, "A.doc").is_ok());
}

#[test]
fn root_listing_cannot_clear_confirmed_root_denial_without_verified_root_reply() {
    let (_temp, mut store, _binding, run) = setup();
    store.preview_mount_root("100").unwrap();
    let old = store.begin_initialization_directory(&run, "100").unwrap();
    store.revoke_initialization_preview_root(&run).unwrap();
    store
        .store_initialization_native_directory(&old, 1, &[entry(file(), 51)])
        .unwrap();
    errno(store.preview_record_by_inode(1), libc::EACCES);
    assert!(
        store
            .initialization_status()
            .unwrap()
            .unwrap()
            .root_needs_refresh
    );
    assert!(store.complete_initialization(&run).is_err());
}

#[test]
fn orphan_draft_or_symlink_blocks_cold_preview_even_after_sealing_without_touching_contents() {
    for sealed in [false, true] {
        let (temp, store, _binding, _run) = setup();
        if sealed {
            store.preview_mount_root("100").unwrap();
        }
        std::fs::create_dir(temp.path().join("drafts")).unwrap();
        let orphan = temp.path().join("drafts/orphan");
        std::fs::write(&orphan, b"unsent").unwrap();
        assert!(!store.preview_available("100").unwrap());
        assert!(store.preview_mount_root("100").unwrap().is_none());
        assert_eq!(std::fs::read(&orphan).unwrap(), b"unsent");
        drop(store);
        let store = Store::open(temp.path()).unwrap();
        assert!(!store.preview_available("100").unwrap());
        assert_eq!(std::fs::read(&orphan).unwrap(), b"unsent");
    }
    let (temp, store, _binding, _run) = setup();
    let other = tempfile::tempdir().unwrap();
    std::os::unix::fs::symlink(other.path(), temp.path().join("drafts")).unwrap();
    assert!(!store.preview_available("100").unwrap());
}

#[test]
fn ordinary_root_probe_failure_preserves_preview_but_not_final_fence_across_reopen() {
    let (temp, mut store, binding, run) = setup();
    store.preview_mount_root("100").unwrap();
    ready_root(&mut store, &run);
    list(&mut store, &run, "200", &[]);
    let held = store.preview_directory_snapshot(1, 4096).unwrap();
    finish(&mut store, &run);
    let _probe = store.begin_initialization_root_refresh(&run).unwrap();
    assert!(
        store
            .initialization_status()
            .unwrap()
            .unwrap()
            .root_needs_refresh
    );
    store.preview_validate_view(&held.stamp).unwrap();
    assert!(store.preview_lookup(1, "A.doc").is_ok());
    assert!(store.complete_initialization(&run).is_err());
    store
        .fail_initialization(&run, "initialization_rejected")
        .unwrap();
    assert!(store.preview_directory_snapshot(1, 4096).is_ok());
    drop(store);
    let mut store = Store::open(temp.path()).unwrap();
    assert!(store.preview_directory_snapshot(1, 4096).is_ok());
    let current = store.initialization_status().unwrap().unwrap().run;
    let resumed = store.resume_initialization(&current, &binding).unwrap();
    let fence = store.begin_initialization_fence(&resumed).unwrap();
    assert!(
        store
            .confirm_initialization_fence(&fence, "native-v1:100")
            .is_err()
    );
    let probe = store.begin_initialization_root_refresh(&resumed).unwrap();
    store.refresh_initialization_root(&probe, &root()).unwrap();
    finish(&mut store, &resumed);
    store.complete_initialization(&resumed).unwrap();
    assert_eq!(store.cursor().unwrap(), "native-v1:100");
}

#[test]
fn explicit_root_event_revokes_preview_and_old_probe_cannot_restore_it() {
    let (temp, mut store, binding, run) = setup();
    store.preview_mount_root("100").unwrap();
    ready_root(&mut store, &run);
    let held = store.preview_directory_snapshot(1, 4096).unwrap();
    let old = store.begin_initialization_root_refresh(&run).unwrap();
    store
        .record_initialization_events(
            &run,
            1,
            &crate::Batch {
                from: "native-v1:100".into(),
                to: "native-v1:101".into(),
                events: vec![crate::Event {
                    id: "root-change".into(),
                    change: crate::Change::Upsert(root()),
                }],
            },
        )
        .unwrap();
    store.apply_initialization_events(&run, 1).unwrap();
    errno(store.preview_record_by_inode(1), libc::ESTALE);
    errno(store.preview_validate_view(&held.stamp), libc::ESTALE);
    assert!(store.refresh_initialization_root(&old, &root()).is_err());
    drop(store);
    let mut store = Store::open(temp.path()).unwrap();
    let current = store.initialization_status().unwrap().unwrap().run;
    let resumed = store.resume_initialization(&current, &binding).unwrap();
    assert!(!store.preview_available("100").unwrap());
    let probe = store.begin_initialization_root_refresh(&resumed).unwrap();
    store.refresh_initialization_root(&probe, &root()).unwrap();
    assert!(store.preview_record_by_inode(1).is_ok());
    errno(store.preview_lookup(1, "A.doc"), libc::EAGAIN);
    ready_root(&mut store, &resumed);
    assert!(store.preview_lookup(1, "A.doc").is_ok());
}

#[test]
fn rejected_durable_root_event_commits_only_revocation_not_coverage_or_cursor() {
    for deleted in [false, true] {
        let (_temp, mut store, _binding, run) = setup();
        store.preview_mount_root("100").unwrap();
        ready_root(&mut store, &run);
        let original = store.initialization_coverage(&run).unwrap();
        let change = if deleted {
            crate::Change::Delete { id: "100".into() }
        } else {
            let mut wrong = root();
            wrong.name = "Other".into();
            crate::Change::Upsert(wrong)
        };
        store
            .record_initialization_events(
                &run,
                1,
                &crate::Batch {
                    from: "native-v1:100".into(),
                    to: "native-v1:101".into(),
                    events: vec![crate::Event {
                        id: "rejected-root".into(),
                        change,
                    }],
                },
            )
            .unwrap();
        assert!(store.apply_initialization_events(&run, 1).is_err());
        errno(store.preview_record_by_inode(1), libc::EACCES);
        assert_eq!(store.initialization_coverage(&run).unwrap(), original);
        let status = store.initialization_status().unwrap().unwrap();
        assert_eq!(status.applied_sequence, 0);
        assert_eq!(status.received_sequence, 1);
        assert_eq!(status.applied_cursor, "native-v1:100");
        assert!(status.root_needs_refresh);
        assert_eq!(store.cursor().unwrap(), "");
        assert!(store.node("100").is_err());
        assert!(store.initialization_writeback_blocked().unwrap());
    }
}
