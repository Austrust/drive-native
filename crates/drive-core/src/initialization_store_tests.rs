#![cfg(feature = "native")]
#[path = "initialization_event_coverage_tests.rs"]
mod event_coverage;
#[path = "initialization_hint_store_tests.rs"]
mod hint_store;
#[path = "initialization_local_recheck_tests.rs"]
mod local_recheck;
#[path = "initialization_priority_tests.rs"]
mod priority_tests;
#[path = "initialization_scheduler_tests.rs"]
mod scheduler_tests;
#[path = "initialization_exclude_tests.rs"]
mod exclude_tests;
use super::*;
use crate::{
    Event, NodeMetadata,
    native::{RemoteSnapshot, Scope},
};

fn scope() -> Scope {
    Scope {
        view_id: 1,
        node_id: 2,
        file_id: "100".into(),
        path: "/Lab".into(),
    }
}
fn node(
    id: &str,
    parent: Option<&str>,
    name: &str,
    directory: bool,
    version: u64,
    restore: &str,
) -> Node {
    Node{id:id.into(),parent:parent.map(str::to_owned),name:name.into(),directory,
        version:serde_json::json!({"schema":1,"restore":restore,"view":1,"root":"100","file":id,"sync":version,"max":version,"size":3,"hash":"synthetic","directory":directory}).to_string(),
        metadata:Some(NodeMetadata{size:3,mtime:version,mode:if directory{0o755}else{0o644}})}
}
fn root() -> Node {
    node("100", None, "Lab", true, 7, "server")
}
fn file() -> Node {
    node("101", Some("100"), "A.doc", false, 7, "server")
}
fn directory() -> Node {
    node("200", Some("100"), "Dir", true, 7, "server")
}
fn snapshot() -> RemoteSnapshot {
    RemoteSnapshot {
        scope: scope(),
        restore: "server".into(),
        cursor: "native-v1:7".into(),
        nodes: vec![root(), file()],
        inaccessible: vec![],
        skipped_symlinks: 0,
    }
}
fn setup() -> (tempfile::TempDir, Store, InitializationBinding) {
    let temp = tempfile::tempdir().unwrap();
    let mut store = Store::open(temp.path()).unwrap();
    store
        .bind_native_scope("server", &serde_json::to_string(&scope()).unwrap())
        .unwrap();
    store.install_native_snapshot(&snapshot(), "").unwrap();
    let binding = InitializationBinding {
        task_id: "fixture-task".into(),
        scope: serde_json::to_value(scope()).unwrap(),
        restore: "server".into(),
        start_cursor: "native-v1:100".into(),
        expected_cursor: "native-v1:7".into(),
rules_digest: String::new(),
    };
    (temp, store, binding)
}
fn start(store: &mut Store, binding: &InitializationBinding) -> InitializationRun {
    store
        .begin_initialization(binding, &root(), InitializationBudget::default())
        .unwrap()
}
fn list(store: &mut Store, run: &InitializationRun, id: &str, children: &[Node]) {
    let ticket = store.begin_initialization_directory(run, id).unwrap();
    assert!(
        store
            .store_initialization_directory(&ticket, 1, children)
            .unwrap()
    );
}
fn batch(from: u64, to: u64, changes: Vec<Change>) -> Batch {
    Batch {
        from: format!("native-v1:{from}"),
        to: format!("native-v1:{to}"),
        events: changes
            .into_iter()
            .enumerate()
            .map(|(i, change)| Event {
                id: format!("event-{to}-{i}"),
                change,
            })
            .collect(),
    }
}

#[test]
fn candidate_is_separate_from_authoritative_tree_cursor_and_dirty_data() {
    let (_temp, mut store, binding) = setup();
    store.save_local("101", b"offline").unwrap();
    let pending = store.pending_operations().unwrap();
    let run = start(&mut store, &binding);
    list(&mut store, &run, "100", &[directory()]);
    assert!(store.initialization_writeback_blocked().unwrap());
    assert_eq!(store.cursor().unwrap(), "native-v1:7");
    assert_eq!(store.node("101").unwrap(), file());
    assert!(store.node("200").is_err());
    assert_eq!(store.pending_operations().unwrap(), pending);
    assert!(store.test_fence(&run, "native-v1:100").is_err());
    assert!(store.complete_initialization(&run).is_err());
    assert!(
        store
            .install_native_snapshot(&snapshot(), "native-v1:7")
            .is_err()
    );
    assert_eq!(store.cursor().unwrap(), "native-v1:7");
}

#[test]
fn restart_preserves_complete_lists_but_reissues_inflight_work_and_rejects_late_epochs() {
    let (temp, mut store, binding) = setup();
    let run = start(&mut store, &binding);
    list(&mut store, &run, "100", &[directory(), file()]);
    let stale = store.begin_initialization_directory(&run, "200").unwrap();
    drop(store);
    let mut store = Store::open(temp.path()).unwrap();
    assert!(store.initialization_writeback_blocked().unwrap());
    assert_eq!(store.cursor().unwrap(), "native-v1:7");
    let current = store.initialization_status().unwrap().unwrap().run;
    assert!(current.epoch > run.epoch);
    assert!(store.resume_initialization(&run, &binding).is_err());
    assert!(
        store
            .restart_initialization(&run, &binding, &root(), InitializationBudget::default())
            .is_err()
    );
    let resumed = store.resume_initialization(&current, &binding).unwrap();
    assert!(resumed.epoch > run.epoch);
    let coverage = store.initialization_coverage(&resumed).unwrap();
    assert_eq!(
        coverage
            .iter()
            .find(|c| c.directory == "100")
            .unwrap()
            .state,
        "received"
    );
    assert_eq!(
        coverage
            .iter()
            .find(|c| c.directory == "200")
            .unwrap()
            .state,
        "pending"
    );
    assert!(
        store
            .store_initialization_directory(&stale, 1, &[])
            .is_err()
    );
    assert!(store.begin_initialization_directory(&run, "100").is_err());
    list(&mut store, &resumed, "200", &[]);
    store.test_fence(&resumed, "native-v1:100").unwrap();
    store.complete_initialization(&resumed).unwrap();
    assert!(!store.initialization_writeback_blocked().unwrap());
    assert_eq!(store.cursor().unwrap(), "native-v1:100");
    drop(store);
    let store = Store::open(temp.path()).unwrap();
    assert!(!store.initialization_writeback_blocked().unwrap());
    assert_eq!(store.initialization_status().unwrap().unwrap().run, resumed);
    assert_eq!(store.node("200").unwrap().name, "Dir");
}

#[test]
fn duplicate_directory_receipt_is_idempotent_but_reused_sequence_and_generation_are_rejected() {
    let (_temp, mut store, binding) = setup();
    let run = start(&mut store, &binding);
    let ticket = store.begin_initialization_directory(&run, "100").unwrap();
    assert!(
        store
            .store_initialization_directory(&ticket, 1, &[file()])
            .unwrap()
    );
    assert!(
        !store
            .store_initialization_directory(&ticket, 1, &[file()])
            .unwrap()
    );
    assert!(
        store
            .store_initialization_directory(&ticket, 1, &[])
            .is_err()
    );
    assert!(
        store
            .store_initialization_directory(&ticket, 2, &[file()])
            .is_err()
    );
    let next = store.begin_initialization_directory(&run, "100").unwrap();
    assert!(
        store
            .store_initialization_directory(&ticket, 1, &[file()])
            .is_err()
    );
    assert!(store.store_initialization_directory(&next, 1, &[]).unwrap());
    assert!(store.node("101").is_ok());
}

#[test]
fn coverage_failure_and_failed_run_remain_blocked_after_reopen() {
    let (temp, mut store, binding) = setup();
    let run = start(&mut store, &binding);
    list(&mut store, &run, "100", &[directory()]);
    store.fail_initialization(&run, "cursor_gap").unwrap();
    drop(store);
    let mut store = Store::open(temp.path()).unwrap();
    assert!(store.initialization_writeback_blocked().unwrap());
    assert_eq!(
        store
            .initialization_status()
            .unwrap()
            .unwrap()
            .error
            .as_deref(),
        Some("cursor_gap")
    );
    assert!(
        store
            .begin_initialization(&binding, &root(), InitializationBudget::default())
            .is_err()
    );
    assert!(store.test_fence(&run, "native-v1:100").is_err());
    let current = store.initialization_status().unwrap().unwrap().run;
    let resumed = store.resume_initialization(&current, &binding).unwrap();
    assert!(store.initialization_writeback_blocked().unwrap());
    assert!(store.test_fence(&resumed, "native-v1:100").is_err());
}

#[test]
fn permission_denied_is_explicit_coverage_and_root_denial_is_not_empty_success() {
    let (_temp, mut store, binding) = setup();
    let run = start(&mut store, &binding);
    let root_ticket = store.begin_initialization_directory(&run, "100").unwrap();
    assert!(store.store_initialization_denied(&root_ticket, 1).is_err());
    store
        .store_initialization_directory(&root_ticket, 1, &[directory()])
        .unwrap();
    let denied = store.begin_initialization_directory(&run, "200").unwrap();
    assert!(store.store_initialization_denied(&denied, 1).unwrap());
    assert!(!store.store_initialization_denied(&denied, 1).unwrap());
    store.test_fence(&run, "native-v1:100").unwrap();
    store.complete_initialization(&run).unwrap();
    let denied: i64 = store
        .db
        .query_row(
            "SELECT count(*) FROM access_denied WHERE node='200'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(denied, 1);
}

#[test]
fn event_receipt_and_invalidation_survive_crash_without_advancing_product_cursor() {
    let (temp, mut store, binding) = setup();
    let run = start(&mut store, &binding);
    list(&mut store, &run, "100", &[file()]);
    let update = node("101", Some("100"), "A.doc", false, 8, "server");
    let event = batch(100, 101, vec![Change::Upsert(update.clone())]);
    assert!(store.record_initialization_events(&run, 1, &event).unwrap());
    assert!(!store.record_initialization_events(&run, 1, &event).unwrap());
    assert!(store.test_fence(&run, "native-v1:101").is_err());
    drop(store);
    let mut store = Store::open(temp.path()).unwrap();
    let current = store.initialization_status().unwrap().unwrap().run;
    let resumed = store.resume_initialization(&current, &binding).unwrap();
    assert_eq!(
        store.pending_initialization_events(&resumed, 10).unwrap(),
        vec![(1, event)]
    );
    assert_eq!(
        store.apply_initialization_events(&resumed, 1).unwrap(),
        InitializationApplyResult::Applied
    );
    assert_eq!(
        store.apply_initialization_events(&resumed, 1).unwrap(),
        InitializationApplyResult::AlreadyApplied
    );
    assert_eq!(store.cursor().unwrap(), "native-v1:7");
    assert_eq!(store.initialization_node(&resumed, "101").unwrap(), file());
    assert!(store.test_fence(&resumed, "native-v1:101").is_err());
    list(&mut store, &resumed, "100", &[update.clone()]);
    store.test_fence(&resumed, "native-v1:101").unwrap();
    store.complete_initialization(&resumed).unwrap();
    assert_eq!(store.node("101").unwrap(), update);
    assert_eq!(store.cursor().unwrap(), "native-v1:101");
}

#[test]
fn event_gaps_body_reuse_and_late_directory_results_never_confirm_coverage() {
    let (_temp, mut store, binding) = setup();
    let run = start(&mut store, &binding);
    let old = store.begin_initialization_directory(&run, "100").unwrap();
    let event = batch(100, 101, vec![Change::Delete { id: "101".into() }]);
    assert!(store.record_initialization_events(&run, 2, &event).is_err());
    assert!(
        store
            .record_initialization_events(&run, 1, &batch(99, 101, vec![]))
            .is_err()
    );
    store.record_initialization_events(&run, 1, &event).unwrap();
    assert!(
        store
            .record_initialization_events(&run, 1, &batch(100, 101, vec![]))
            .is_err()
    );
    store.apply_initialization_events(&run, 1).unwrap();
    assert!(
        store
            .store_initialization_directory(&old, 1, &[file()])
            .is_err()
    );
    assert_eq!(store.cursor().unwrap(), "native-v1:7");
}

#[test]
fn root_event_requires_current_root_refresh_and_rescanned_coverage() {
    let (_temp, mut store, binding) = setup();
    let run = start(&mut store, &binding);
    list(&mut store, &run, "100", &[]);
    let new_root = node("100", None, "Lab", true, 9, "server");
    store
        .record_initialization_events(
            &run,
            1,
            &batch(100, 101, vec![Change::Upsert(new_root.clone())]),
        )
        .unwrap();
    store.apply_initialization_events(&run, 1).unwrap();
    list(&mut store, &run, "100", &[]);
    assert!(store.test_fence(&run, "native-v1:101").is_err());
    let root_ticket = store.begin_initialization_root_refresh(&run).unwrap();
    store
        .refresh_initialization_root(&root_ticket, &new_root)
        .unwrap();
    store.test_fence(&run, "native-v1:101").unwrap();
    store.complete_initialization(&run).unwrap();
    assert_eq!(store.node("100").unwrap(), new_root);
}

#[test]
fn node_and_event_budgets_rollback_whole_receipts() {
    let (_temp, mut store, binding) = setup();
    let run = store
        .begin_initialization(
            &binding,
            &root(),
            InitializationBudget {
                nodes: 1,
                event_batches: 1,
                event_bytes: 128,
                ..InitializationBudget::default()
            },
        )
        .unwrap();
    let ticket = store.begin_initialization_directory(&run, "100").unwrap();
    assert!(
        store
            .store_initialization_directory(&ticket, 1, &[file()])
            .is_err()
    );
    assert_eq!(
        store.initialization_status().unwrap().unwrap().node_count,
        1
    );
    assert_eq!(
        store.initialization_coverage(&run).unwrap()[0].state,
        "loading"
    );
    assert!(
        store
            .record_initialization_events(&run, 1, &batch(100, 101, vec![Change::Upsert(file())]))
            .is_err()
    );
    assert_eq!(
        store
            .initialization_status()
            .unwrap()
            .unwrap()
            .received_cursor,
        "native-v1:100"
    );
    assert_eq!(store.cursor().unwrap(), "native-v1:7");
}

#[test]
fn atomic_final_commit_rolls_back_authority_and_gate_on_injected_failure() {
    let (temp, mut store, binding) = setup();
    let run = start(&mut store, &binding);
    list(&mut store, &run, "100", &[]);
    store.test_fence(&run, "native-v1:100").unwrap();
    store.db.execute_batch("CREATE TEMP TRIGGER fail_initialization_ready BEFORE UPDATE OF state ON initialization_run WHEN NEW.state='ready' BEGIN SELECT RAISE(ABORT,'synthetic final failure'); END;").unwrap();
    assert!(store.complete_initialization(&run).is_err());
    assert_eq!(store.cursor().unwrap(), "native-v1:7");
    assert_eq!(store.node("101").unwrap(), file());
    assert!(store.initialization_writeback_blocked().unwrap());
    drop(store);
    let mut store = Store::open(temp.path()).unwrap();
    assert!(store.initialization_writeback_blocked().unwrap());
    // Restart requires an independent new fence; the prior persisted fence must
    // not be mistaken for new-session validation by the owner.
    assert!(store.complete_initialization(&run).is_err());
    let current = store.initialization_status().unwrap().unwrap().run;
    assert!(store.complete_initialization(&current).is_err());
    let resumed = store.resume_initialization(&current, &binding).unwrap();
    assert!(store.complete_initialization(&resumed).is_err());
    store.test_fence(&resumed, "native-v1:100").unwrap();
    store.complete_initialization(&resumed).unwrap();
    assert!(!store.initialization_writeback_blocked().unwrap());
}

#[test]
fn final_install_preserves_unuploaded_draft_and_rejects_bad_native_tokens() {
    let (temp, mut store, binding) = setup();
    let base = store.node("101").unwrap().version;
    let draft = store.begin_draft("101", &base, None).unwrap();
    store
        .draft_write(&draft, 0, b"offline edit", false)
        .unwrap();
    store.checkpoint_draft(&draft).unwrap();
    drop(draft);
    let run = start(&mut store, &binding);
    let mut update = node("101", Some("100"), "A.doc", false, 9, "server");
    let good = update.clone();
    update.version = root().version;
    list(&mut store, &run, "100", &[update]);
    store.test_fence(&run, "native-v1:100").unwrap();
    assert!(store.complete_initialization(&run).is_err());
    assert_eq!(store.cursor().unwrap(), "native-v1:7");
    assert!(store.dirty_draft("101").unwrap());
    list(&mut store, &run, "100", &[good]);
    store.test_fence(&run, "native-v1:100").unwrap();
    store.complete_initialization(&run).unwrap();
    drop(store);
    let store = Store::open(temp.path()).unwrap();
    let draft = store.open_dirty_draft("101").unwrap().unwrap();
    let mut bytes = [0u8; 32];
    let n = draft.read_at(&mut bytes, 0).unwrap();
    assert_eq!(&bytes[..n], b"offline edit");
    assert!(store.draft_conflicts("101").unwrap());
    assert!(!store.initialization_writeback_blocked().unwrap());
}

#[test]
fn restarting_failed_candidate_changes_restore_only_with_same_task_scope_and_root() {
    let (temp, mut store, binding) = setup();
    store.save_local("101", b"pending").unwrap();
    let pending = store.pending_operations().unwrap();
    let run = start(&mut store, &binding);
    let ticket = store.begin_initialization_directory(&run, "100").unwrap();
    store.fail_initialization(&run, "cursor_gap").unwrap();
    let mut next_binding = binding.clone();
    next_binding.restore = "restored".into();
    next_binding.start_cursor = "native-v1:2".into();
    let new_root = node("100", None, "Lab", true, 1, "restored");
    let mut wrong = next_binding.clone();
    wrong.task_id = "another".into();
    assert!(
        store
            .restart_initialization(&run, &wrong, &new_root, InitializationBudget::default())
            .is_err()
    );
    let next = store
        .restart_initialization(
            &run,
            &next_binding,
            &new_root,
            InitializationBudget::default(),
        )
        .unwrap();
    assert_ne!(next.run_id, run.run_id);
    assert!(next.epoch > run.epoch);
    assert!(store.initialization_writeback_blocked().unwrap());
    assert!(
        store
            .store_initialization_directory(&ticket, 1, &[])
            .is_err()
    );
    assert_eq!(store.cursor().unwrap(), "native-v1:7");
    assert_eq!(store.pending_operations().unwrap(), pending);
    let saved: String = store
        .db
        .query_row(
            "SELECT value FROM metadata WHERE key='server_restore_id'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(saved, "server");
    drop(store);
    let mut store = Store::open(temp.path()).unwrap();
    assert!(store.resume_initialization(&next, &next_binding).is_err());
    let current = store.initialization_status().unwrap().unwrap().run;
    let next = store
        .resume_initialization(&current, &next_binding)
        .unwrap();
    list(
        &mut store,
        &next,
        "100",
        &[node("101", Some("100"), "A.doc", false, 1, "restored")],
    );
    store.test_fence(&next, "native-v1:2").unwrap();
    store.complete_initialization(&next).unwrap();
    let saved: String = store
        .db
        .query_row(
            "SELECT value FROM metadata WHERE key='server_restore_id'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(saved, "restored");
    assert!(!store.initialization_writeback_blocked().unwrap());
}

#[test]
fn migration_from_actual_v18_shape_backs_up_and_preserves_pending_content() {
    let (temp, mut store, _) = setup();
    store.save_local("101", b"must survive migration").unwrap();
    let pending = store.pending_operations().unwrap();
    store.db.execute_batch("DROP TRIGGER preview_denied_capture; DROP TRIGGER preview_denied_clear; DROP TABLE preview_denied; DROP TABLE preview_visibility; DROP TABLE preview_inodes; DROP TABLE preview_mount; DROP TABLE initialization_event_hints; DROP TABLE initialization_native_nodes; DROP TABLE initialization_events; DROP TABLE initialization_directories; DROP TABLE initialization_entries; DROP TABLE initialization_run; PRAGMA user_version=18;").unwrap();
    drop(store);
    let store = Store::open(temp.path()).unwrap();
    assert_eq!(store.cursor().unwrap(), "native-v1:7");
    assert_eq!(store.pending_operations().unwrap(), pending);
    assert!(!store.initialization_writeback_blocked().unwrap());
    let backups = std::fs::read_dir(temp.path())
        .unwrap()
        .map(|p| p.unwrap().path())
        .filter(|p| {
            p.file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with(&format!("state-before-v{}-", crate::STATE_SCHEMA_VERSION))
        })
        .collect::<Vec<_>>();
    assert_eq!(backups.len(), 1);
    let backup = Connection::open(&backups[0]).unwrap();
    assert_eq!(
        backup
            .pragma_query_value(None, "user_version", |r| r.get::<_, i64>(0))
            .unwrap(),
        18
    );
    let has: i64 = backup
        .query_row(
            "SELECT count(*) FROM sqlite_master WHERE name='initialization_run'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(has, 0);
    assert_eq!(
        store
            .db
            .pragma_query_value(None, "user_version", |r| r.get::<_, i64>(0))
            .unwrap(),
        crate::STATE_SCHEMA_VERSION
    );
    // Future schema rejection is the same guard used by the frozen schema18
    // client's open(): current schema > 18. The actual old-binary check is a package gate.
    store
        .db
        .pragma_update(None, "user_version", crate::STATE_SCHEMA_VERSION + 1)
        .unwrap();
    drop(store);
    assert!(
        matches!(Store::open(temp.path()),Err(Error::State(message)) if message=="database is newer than this build")
    );
}

#[test]
fn missing_coverage_row_is_not_a_completed_directory() {
    let (_temp, mut store, binding) = setup();
    let run = start(&mut store, &binding);
    list(&mut store, &run, "100", &[directory()]);
    list(&mut store, &run, "200", &[]);
    store
        .db
        .execute(
            "DELETE FROM initialization_directories WHERE run=?1 AND id='200'",
            [&run.run_id],
        )
        .unwrap();
    assert!(store.test_fence(&run, "native-v1:100").is_err());
    assert!(store.complete_initialization(&run).is_err());
    assert!(store.initialization_writeback_blocked().unwrap());
    assert_eq!(store.cursor().unwrap(), "native-v1:7");
}

#[test]
fn restore_metadata_tree_cursor_ready_and_dirty_protection_share_final_transaction() {
    let (_temp, mut store, binding) = setup();
    store.save_local("101", b"offline restore edit").unwrap();
    let old = store.node("101").unwrap();
    let pending = store.pending_operations().unwrap();
    let run = start(&mut store, &binding);
    store.fail_initialization(&run, "server_restored").unwrap();
    let mut binding = binding;
    binding.restore = "restored".into();
    binding.start_cursor = "native-v1:2".into();
    let root = node("100", None, "Lab", true, 1, "restored");
    let next = store
        .restart_initialization(&run, &binding, &root, InitializationBudget::default())
        .unwrap();
    list(
        &mut store,
        &next,
        "100",
        &[node("101", Some("100"), "A.doc", false, 1, "restored")],
    );
    store.test_fence(&next, "native-v1:2").unwrap();
    store.db.execute_batch("CREATE TEMP TRIGGER reject_restore_ready BEFORE UPDATE OF state ON initialization_run WHEN NEW.state='ready' BEGIN SELECT RAISE(ABORT,'synthetic restore final failure'); END;").unwrap();
    assert!(store.complete_initialization(&next).is_err());
    assert_eq!(store.cursor().unwrap(), "native-v1:7");
    assert_eq!(store.node("101").unwrap(), old);
    assert_eq!(store.pending_operations().unwrap(), pending);
    assert!(store.initialization_writeback_blocked().unwrap());
    let saved: String = store
        .db
        .query_row(
            "SELECT value FROM metadata WHERE key='server_restore_id'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(saved, "server");
    use std::io::Read;
    let mut content = Vec::new();
    store
        .open_cached("101")
        .unwrap()
        .read_to_end(&mut content)
        .unwrap();
    assert_eq!(content, b"offline restore edit");
    store
        .db
        .execute_batch("DROP TRIGGER reject_restore_ready;")
        .unwrap();
    store.complete_initialization(&next).unwrap();
    assert_eq!(store.cursor().unwrap(), "native-v1:2");
    assert!(!store.initialization_writeback_blocked().unwrap());
    let saved: String = store
        .db
        .query_row(
            "SELECT value FROM metadata WHERE key='server_restore_id'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(saved, "restored");
}

#[test]
fn same_directory_identity_recreated_by_parent_cannot_reuse_an_old_ticket() {
    let (_temp, mut store, binding) = setup();
    let run = start(&mut store, &binding);
    list(&mut store, &run, "100", &[directory()]);
    let stale = store.begin_initialization_directory(&run, "200").unwrap();
    let changed = node("200", Some("100"), "Dir", true, 8, "server");
    // Parent receipt deletes/recreates 200's candidate row and all descendants.
    list(&mut store, &run, "100", &[changed]);
    let fresh = store.begin_initialization_directory(&run, "200").unwrap();
    assert!(fresh.generation > stale.generation);
    let old_child = node("201", Some("200"), "stale.doc", false, 7, "server");
    assert!(
        store
            .store_initialization_directory(&stale, 1, &[old_child])
            .is_err()
    );
    assert!(store.initialization_node(&run, "201").is_err());
    assert_eq!(
        store
            .initialization_coverage(&run)
            .unwrap()
            .into_iter()
            .find(|d| d.directory == "200")
            .unwrap()
            .state,
        "loading"
    );
    assert!(
        store
            .store_initialization_directory(&fresh, 1, &[])
            .unwrap()
    );
    store.test_fence(&run, "native-v1:100").unwrap();
    store.complete_initialization(&run).unwrap();
    assert!(store.node("201").is_err());
}

#[test]
fn directory_ticket_clock_overflow_rolls_back_without_changing_coverage_or_gate() {
    let (_temp, mut store, binding) = setup();
    let run = start(&mut store, &binding);
    list(&mut store, &run, "100", &[]);
    store.test_fence(&run, "native-v1:100").unwrap();
    let coverage = store.initialization_coverage(&run).unwrap();
    store
        .db
        .execute(
            "UPDATE initialization_run SET directory_clock=?1",
            [i64::MAX],
        )
        .unwrap();
    assert!(store.begin_initialization_directory(&run, "100").is_err());
    assert_eq!(store.initialization_coverage(&run).unwrap(), coverage);
    assert_eq!(store.cursor().unwrap(), "native-v1:7");
    assert!(store.initialization_writeback_blocked().unwrap());
    let (clock, fence): (i64, Option<String>) = store
        .db
        .query_row(
            "SELECT directory_clock,fenced FROM initialization_run",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert_eq!(clock, i64::MAX);
    assert_eq!(fence.as_deref(), Some("native-v1:100"));
}

// Most fixtures model an immediate stationary reply; race fixtures below retain
// the actual ticket across asynchronous candidate mutations.
trait TestFence {
    fn test_fence(&mut self, run: &InitializationRun, at: &str) -> Result<()>;
}
impl TestFence for Store {
    fn test_fence(&mut self, run: &InitializationRun, at: &str) -> Result<()> {
        let ticket = self.begin_initialization_fence(run)?;
        self.confirm_initialization_fence(&ticket, at)
    }
}

#[test]
fn delayed_root_reply_cannot_cross_root_event_or_newer_probe() {
    let (_temp, mut store, binding) = setup();
    let run = start(&mut store, &binding);
    list(&mut store, &run, "100", &[]);
    let stale = store.begin_initialization_root_refresh(&run).unwrap();
    let newer = node("100", None, "Lab", true, 9, "server");
    store
        .record_initialization_events(
            &run,
            1,
            &batch(100, 101, vec![Change::Upsert(newer.clone())]),
        )
        .unwrap();
    store.apply_initialization_events(&run, 1).unwrap();
    list(&mut store, &run, "100", &[]);
    assert!(store.refresh_initialization_root(&stale, &root()).is_err());
    assert!(
        store
            .initialization_status()
            .unwrap()
            .unwrap()
            .root_needs_refresh
    );
    assert!(store.test_fence(&run, "native-v1:101").is_err());
    let superseded = store.begin_initialization_root_refresh(&run).unwrap();
    let current = store.begin_initialization_root_refresh(&run).unwrap();
    assert!(
        store
            .refresh_initialization_root(&superseded, &root())
            .is_err()
    );
    store.refresh_initialization_root(&current, &newer).unwrap();
    assert!(
        store
            .refresh_initialization_root(&current, &root())
            .is_err()
    );
    store.test_fence(&run, "native-v1:101").unwrap();
    store.complete_initialization(&run).unwrap();
    assert_eq!(store.node("100").unwrap(), newer);
    assert_eq!(store.cursor().unwrap(), "native-v1:101");
}

#[test]
fn fence_reply_crossing_candidate_mutation_new_probe_or_epoch_is_rejected() {
    let (temp, mut store, binding) = setup();
    let run = start(&mut store, &binding);
    list(&mut store, &run, "100", &[]);
    let before_relist = store.begin_initialization_fence(&run).unwrap();
    list(&mut store, &run, "100", &[file()]);
    assert!(
        store
            .confirm_initialization_fence(&before_relist, "native-v1:100")
            .is_err()
    );
    let before_root = store.begin_initialization_fence(&run).unwrap();
    let root_ticket = store.begin_initialization_root_refresh(&run).unwrap();
    store
        .refresh_initialization_root(&root_ticket, &root())
        .unwrap();
    assert!(
        store
            .confirm_initialization_fence(&before_root, "native-v1:100")
            .is_err()
    );
    let before_events = store.begin_initialization_fence(&run).unwrap();
    store
        .record_initialization_events(&run, 1, &batch(100, 101, vec![Change::Upsert(file())]))
        .unwrap();
    store.apply_initialization_events(&run, 1).unwrap();
    list(&mut store, &run, "100", &[file()]);
    assert!(
        store
            .confirm_initialization_fence(&before_events, "native-v1:100")
            .is_err()
    );
    assert!(
        store
            .confirm_initialization_fence(&before_events, "native-v1:101")
            .is_err()
    );
    let old = store.begin_initialization_fence(&run).unwrap();
    let current = store.begin_initialization_fence(&run).unwrap();
    assert!(
        store
            .confirm_initialization_fence(&old, "native-v1:101")
            .is_err()
    );
    assert!(
        store
            .confirm_initialization_fence(&current, "native-v1:102")
            .is_err()
    );
    store
        .confirm_initialization_fence(&current, "native-v1:101")
        .unwrap();
    assert!(
        store
            .confirm_initialization_fence(&current, "native-v1:101")
            .is_err()
    );
    let late = store.begin_initialization_fence(&run).unwrap();
    drop(store);
    let mut store = Store::open(temp.path()).unwrap();
    assert!(
        store
            .confirm_initialization_fence(&late, "native-v1:101")
            .is_err()
    );
    let reopened = store.initialization_status().unwrap().unwrap().run;
    let resumed = store.resume_initialization(&reopened, &binding).unwrap();
    store.test_fence(&resumed, "native-v1:101").unwrap();
    store.complete_initialization(&resumed).unwrap();
}

#[test]
fn active_and_failed_candidate_block_ordinary_receipt_and_preexisting_inbox() {
    let (temp, mut store, binding) = setup();
    let changed = node("101", Some("100"), "A.doc", false, 8, "server");
    let first = batch(7, 8, vec![Change::Upsert(changed)]);
    store.receive(&first).unwrap();
    let run = start(&mut store, &binding);
    let inbox: Vec<(i64, String, i64)> = store
        .db
        .prepare("SELECT seq,body,done FROM inbox ORDER BY seq")
        .unwrap()
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
        .unwrap()
        .collect::<std::result::Result<_, _>>()
        .unwrap();
    for failed in [false, true] {
        if failed {
            store.fail_initialization(&run, "fixture_failure").unwrap();
        }
        assert!(
            store
                .receive(&batch(8, 9, vec![Change::Upsert(file())]))
                .is_err()
        );
        assert!(store.receive(&first).is_err());
        assert!(store.apply_next().is_err());
        assert_eq!(store.cursor().unwrap(), "native-v1:7");
        assert_eq!(store.node("101").unwrap(), file());
        let actual: Vec<(i64, String, i64)> = store
            .db
            .prepare("SELECT seq,body,done FROM inbox ORDER BY seq")
            .unwrap()
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
            .unwrap()
            .collect::<std::result::Result<_, _>>()
            .unwrap();
        assert_eq!(actual, inbox);
    }
    drop(store);
    let mut store = Store::open(temp.path()).unwrap();
    assert!(store.apply_next().is_err());
    let reopened = store.initialization_status().unwrap().unwrap().run;
    let resumed = store.resume_initialization(&reopened, &binding).unwrap();
    list(&mut store, &resumed, "100", &[file()]);
    store.test_fence(&resumed, "native-v1:100").unwrap();
    store.complete_initialization(&resumed).unwrap();
    store
        .receive(&batch(100, 101, vec![Change::Upsert(file())]))
        .unwrap();
    assert!(store.apply_next().unwrap());
    assert_eq!(store.cursor().unwrap(), "native-v1:101");
}

#[test]
fn probe_clock_exhaustion_rolls_back_and_negative_sql_counters_are_rejected() {
    let (_temp, mut store, binding) = setup();
    let run = start(&mut store, &binding);
    list(&mut store, &run, "100", &[]);
    store.test_fence(&run, "native-v1:100").unwrap();
    store
        .db
        .execute(
            "UPDATE initialization_run SET root_clock=?1,fence_clock=?1",
            [i64::MAX],
        )
        .unwrap();
    let before = store.initialization_status().unwrap();
    assert!(store.begin_initialization_root_refresh(&run).is_err());
    assert!(store.begin_initialization_fence(&run).is_err());
    assert_eq!(store.initialization_status().unwrap(), before);
    let fence: Option<String> = store
        .db
        .query_row("SELECT fenced FROM initialization_run", [], |r| r.get(0))
        .unwrap();
    assert_eq!(fence.as_deref(), Some("native-v1:100"));
    store
        .db
        .execute("UPDATE initialization_run SET received_seq=-1", [])
        .unwrap();
    assert!(store.initialization_status().is_err());
    assert!(store.initialization_writeback_blocked().unwrap());
}

#[test]
fn same_restore_candidate_cannot_publish_before_committed_cursor() {
    let (_temp, mut store, mut binding) = setup();
    binding.start_cursor = "native-v1:2".into();
    let run = start(&mut store, &binding);
    list(&mut store, &run, "100", &[file()]);
    let fence = store.begin_initialization_fence(&run).unwrap();
    assert_eq!(fence.cursor(), "native-v1:2");
    assert!(
        store
            .confirm_initialization_fence(&fence, "native-v1:2")
            .is_err()
    );
    assert!(store.complete_initialization(&run).is_err());
    assert_eq!(store.cursor().unwrap(), "native-v1:7");
    assert!(store.initialization_writeback_blocked().unwrap());
}

#[test]
fn same_restore_candidate_may_start_behind_and_catch_up_before_publication() {
    let (_temp, mut store, mut binding) = setup();
    binding.start_cursor = "native-v1:2".into();
    let run = start(&mut store, &binding);
    list(&mut store, &run, "100", &[]);
    store
        .record_initialization_events(&run, 1, &batch(2, 7, vec![Change::Upsert(file())]))
        .unwrap();
    store.apply_initialization_events(&run, 1).unwrap();
    list(&mut store, &run, "100", &[file()]);
    let fence = store.begin_initialization_fence(&run).unwrap();
    assert_eq!(fence.cursor(), "native-v1:7");
    store
        .confirm_initialization_fence(&fence, "native-v1:7")
        .unwrap();
    store.complete_initialization(&run).unwrap();
    assert_eq!(store.cursor().unwrap(), "native-v1:7");
    assert!(!store.initialization_writeback_blocked().unwrap());
}

#[test]
fn changed_restore_candidate_may_publish_lower_cursor_in_new_domain() {
    let (_temp, mut store, mut binding) = setup();
    let old = start(&mut store, &binding);
    store.fail_initialization(&old, "restore_changed").unwrap();
    binding.restore = "restored".into();
    binding.start_cursor = "native-v1:2".into();
    let verified_root = node("100", None, "Lab", true, 1, "restored");
    let run = store
        .restart_initialization(
            &old,
            &binding,
            &verified_root,
            InitializationBudget::default(),
        )
        .unwrap();
    list(&mut store, &run, "100", &[]);
    let fence = store.begin_initialization_fence(&run).unwrap();
    assert_eq!(fence.cursor(), "native-v1:2");
    store
        .confirm_initialization_fence(&fence, "native-v1:2")
        .unwrap();
    store.complete_initialization(&run).unwrap();
    assert_eq!(store.cursor().unwrap(), "native-v1:2");
    assert_eq!(store.node("100").unwrap(), verified_root);
    assert!(!store.initialization_writeback_blocked().unwrap());
}

#[test]
fn private_candidate_mutations_do_not_publish_status_changes_final_install_resets_once() {
    use crate::status_changes::{ChangePage, GapReason};
    let (_temp, mut store, binding) = setup();
    let before = store.status_change_watermark().unwrap();
    let run = start(&mut store, &binding);
    list(&mut store, &run, "100", &[directory()]);
    let denied = store.begin_initialization_directory(&run, "200").unwrap();
    store.store_initialization_denied(&denied, 1).unwrap();
    assert_eq!(store.status_change_watermark().unwrap(), before);
    store
        .record_initialization_events(&run, 1, &batch(100, 101, vec![Change::Upsert(file())]))
        .unwrap();
    store.apply_initialization_events(&run, 1).unwrap();
    list(&mut store, &run, "100", &[directory(), file()]);
    let root_ticket = store.begin_initialization_root_refresh(&run).unwrap();
    store
        .refresh_initialization_root(&root_ticket, &root())
        .unwrap();
    store.fail_initialization(&run, "fixture_pause").unwrap();
    let resumed = store.resume_initialization(&run, &binding).unwrap();
    store.test_fence(&resumed, "native-v1:101").unwrap();
    assert_eq!(store.status_change_watermark().unwrap(), before);
    store.db.execute_batch("CREATE TEMP TRIGGER reject_candidate_status_ready BEFORE UPDATE OF state ON initialization_run WHEN NEW.state='ready' BEGIN SELECT RAISE(ABORT,'injected final publication failure'); END;").unwrap();
    assert!(store.complete_initialization(&resumed).is_err());
    assert_eq!(store.status_change_watermark().unwrap(), before);
    assert!(store.initialization_writeback_blocked().unwrap());
    store
        .db
        .execute_batch("DROP TRIGGER reject_candidate_status_ready;")
        .unwrap();
    store.complete_initialization(&resumed).unwrap();
    let after = store.status_change_watermark().unwrap();
    assert_eq!(after.cursor.sequence, before.cursor.sequence + 1);
    assert_eq!(after.snapshot_resets, before.snapshot_resets + 1);
    assert_eq!(after.overflows, before.overflows);
    assert_eq!((after.entries, after.bytes), (0, 0));
    assert!(matches!(
        store.read_status_changes(&before.cursor, 10).unwrap(),
        ChangePage::Rebuild {
            reason: GapReason::HistoryDiscarded,
            ..
        }
    ));
}

#[test]
fn directory_paths_require_connected_bounded_directory_ancestry_and_canonical_scope() {
    let (_temp, mut store, binding) = setup();
    let run = start(&mut store, &binding);
    list(&mut store, &run, "100", &[directory(), file()]);
    let nested = node("201", Some("200"), "内层", true, 7, "server");
    list(&mut store, &run, "200", &[nested]);
    assert_eq!(
        store.initialization_directory_path(&run, "100").unwrap(),
        "/Lab"
    );
    assert_eq!(
        store.initialization_directory_path(&run, "201").unwrap(),
        "/Lab/Dir/内层"
    );
    assert!(store.initialization_directory_path(&run, "101").is_err());
    assert!(
        store
            .initialization_directory_path(&run, "missing")
            .is_err()
    );
    // Simulated corrupt persistent payloads must never turn into an RPC traversal.
    let mut malformed = node("201", Some("999"), "inner", true, 7, "server");
    for (parent, name) in [
        ("999", "inner"),
        ("200", ".."),
        ("200", "a/b"),
        ("201", "inner"),
    ] {
        malformed.parent = Some(parent.into());
        malformed.name = name.into();
        store
            .db
            .execute(
                "UPDATE initialization_entries SET body=?1 WHERE id='201'",
                [serde_json::to_string(&malformed).unwrap()],
            )
            .unwrap();
        assert!(store.initialization_directory_path(&run, "201").is_err());
    }
    let mut bad_binding = binding.clone();
    bad_binding.scope["path"] = "/bad/../Lab".into();
    store
        .db
        .execute(
            "UPDATE metadata SET value=?1 WHERE key='native_scope'",
            [bad_binding.scope.to_string()],
        )
        .unwrap();
    store
        .db
        .execute(
            "UPDATE initialization_run SET binding=?1",
            [serde_json::to_string(&bad_binding).unwrap()],
        )
        .unwrap();
    assert!(store.initialization_directory_path(&run, "100").is_err());
}

#[test]
fn directory_path_rejects_depth_past_128() {
    let (_temp, mut store, binding) = setup();
    let run = start(&mut store, &binding);
    let mut parent = "100".to_owned();
    for depth in 1..=129 {
        let id = format!("deep{depth}");
        let child = node(&id, Some(&parent), "d", true, 7, "server");
        list(&mut store, &run, &parent, &[child]);
        assert_eq!(
            store.initialization_directory_path(&run, &id).is_ok(),
            depth <= 128
        );
        parent = id;
    }
}

#[test]
fn native_binding_reader_preserves_pair_and_rejects_partial_binding() {
    let temp = tempfile::tempdir().unwrap();
    let mut store = Store::open(temp.path()).unwrap();
    assert_eq!(store.native_scope_binding().unwrap(), None);
    let raw = serde_json::to_string(&scope()).unwrap();
    store.bind_native_scope("server", &raw).unwrap();
    assert_eq!(
        store.native_scope_binding().unwrap(),
        Some(("server".into(), raw.clone()))
    );
    assert!(store.bind_native_scope("other", &raw).is_err());
    store
        .db
        .execute("DELETE FROM metadata WHERE key='server_restore_id'", [])
        .unwrap();
    assert!(store.native_scope_binding().is_err());
}

#[test]
fn moved_file_or_directory_converges_in_both_parent_orders_without_old_receipts() {
    for directory_move in [false, true] {
        for new_first in [false, true] {
            let (_temp, mut store, binding) = setup();
            let run = start(&mut store, &binding);
            let other = node("300", Some("100"), "Other", true, 7, "server");
            let old = node("201", Some("200"), "item", directory_move, 7, "server");
            list(&mut store, &run, "100", &[directory(), other]);
            list(&mut store, &run, "200", &[old.clone()]);
            list(&mut store, &run, "300", &[]);
            let old_leaf = node("202", Some("201"), "old", false, 7, "server");
            let stale_desc = if directory_move {
                list(&mut store, &run, "201", &[old_leaf.clone()]);
                Some(store.begin_initialization_directory(&run, "201").unwrap())
            } else {
                None
            };
            let stale_parent = store.begin_initialization_directory(&run, "200").unwrap();
            let replacement = node("401", Some("200"), "item", false, 9, "server");
            let moved = node("201", Some("300"), "item", directory_move, 9, "server");
            if new_first {
                list(&mut store, &run, "300", &[moved.clone()]);
                assert!(
                    store
                        .store_initialization_directory(&stale_parent, 1, &[old.clone()])
                        .is_err()
                );
                list(&mut store, &run, "200", &[replacement.clone()]);
            } else {
                list(&mut store, &run, "200", &[replacement.clone()]);
                list(&mut store, &run, "300", &[moved.clone()]);
            }
            assert!(
                store
                    .store_initialization_directory(&stale_parent, 1, &[old])
                    .is_err()
            );
            if let Some(stale) = stale_desc {
                assert!(
                    store
                        .store_initialization_directory(&stale, 1, &[old_leaf])
                        .is_err()
                );
                let fresh = node("203", Some("201"), "fresh", false, 9, "server");
                list(&mut store, &run, "201", &[fresh]);
                assert_eq!(
                    store.initialization_directory_path(&run, "201").unwrap(),
                    "/Lab/Other/item"
                );
            }
            assert!(store.node("201").is_err());
            assert_eq!(store.cursor().unwrap(), "native-v1:7");
            store.test_fence(&run, "native-v1:100").unwrap();
            store.complete_initialization(&run).unwrap();
            assert_eq!(store.node("201").unwrap(), moved);
            assert_eq!(store.node("401").unwrap(), replacement);
            assert!(store.node("202").is_err());
        }
    }
}

#[test]
fn directory_relocation_into_its_subtree_rejects_whole_receipt() {
    let (_temp, mut store, binding) = setup();
    let run = start(&mut store, &binding);
    list(&mut store, &run, "100", &[directory()]);
    let child = node("201", Some("200"), "inner", true, 7, "server");
    list(&mut store, &run, "200", &[child]);
    let ticket = store.begin_initialization_directory(&run, "201").unwrap();
    let bad = node("200", Some("201"), "Dir", true, 9, "server");
    let before = store.initialization_status().unwrap();
    let coverage = store.initialization_coverage(&run).unwrap();
    assert!(
        store
            .store_initialization_directory(&ticket, 1, &[bad])
            .is_err()
    );
    assert_eq!(store.initialization_status().unwrap(), before);
    assert_eq!(store.initialization_coverage(&run).unwrap(), coverage);
    assert_eq!(
        store.initialization_directory_path(&run, "201").unwrap(),
        "/Lab/Dir/inner"
    );
}

#[test]
fn pending_queue_uses_partial_index_and_rediscovers_earlier_invalidated_directory() {
    let (_temp, mut store, binding) = setup();
    let run = start(&mut store, &binding);
    let dirs = (0..240)
        .map(|i| {
            node(
                &format!("dir{i:04}"),
                Some("100"),
                &format!("D{i}"),
                true,
                7,
                "server",
            )
        })
        .collect::<Vec<_>>();
    list(&mut store, &run, "100", &dirs);
    for d in &dirs[..238] {
        list(&mut store, &run, &d.id, &[]);
    }
    let pending = store.initialization_pending_directories(&run, 1).unwrap();
    assert_eq!(pending[0].id, "dir0238");
    let child = node("new", Some("dir0000"), "new", false, 9, "server");
    store
        .record_initialization_events(&run, 1, &batch(100, 101, vec![Change::Upsert(child)]))
        .unwrap();
    store.apply_initialization_events(&run, 1).unwrap();
    assert_eq!(
        store.initialization_pending_directories(&run, 1).unwrap()[0].id,
        "dir0000"
    );
    let plan=store.db.prepare("EXPLAIN QUERY PLAN SELECT n.body FROM initialization_directories d JOIN initialization_entries n ON n.run=d.run AND n.id=d.id WHERE d.run=?1 AND d.state IN ('pending','failed') ORDER BY d.id LIMIT ?2").unwrap().query_map(params![run.run_id,1],|r|r.get::<_,String>(3)).unwrap().collect::<std::result::Result<Vec<_>,_>>().unwrap();
    assert!(
        plan.iter()
            .any(|line| line.contains("initialization_pending")),
        "{plan:?}"
    );
    assert!(
        !plan.iter().any(|line| line.contains("TEMP B-TREE")),
        "{plan:?}"
    );
}

#[test]
fn verified_restore_begin_preserves_old_authority_until_final_commit() {
    let (_temp, mut store, mut binding) = setup();
    let old_binding = store.native_scope_binding().unwrap();
    binding.restore = "restored".into();
    binding.start_cursor = "native-v1:2".into();
    let root = node("100", None, "Lab", true, 1, "restored");
    assert!(
        store
            .begin_initialization(&binding, &root, InitializationBudget::default())
            .is_err()
    );
    let run = store
        .begin_initialization_verified_restore(&binding, &root, InitializationBudget::default())
        .unwrap();
    assert_eq!(store.native_scope_binding().unwrap(), old_binding);
    assert_eq!(store.cursor().unwrap(), "native-v1:7");
    assert!(
        store
            .authorize_initialization_cursor_reset(&run, "native-v1:7")
            .is_err()
    );
    list(&mut store, &run, "100", &[]);
    store.test_fence(&run, "native-v1:2").unwrap();
    store.complete_initialization(&run).unwrap();
    assert_eq!(store.native_scope_binding().unwrap().unwrap().0, "restored");
    assert_eq!(store.cursor().unwrap(), "native-v1:2");
}

#[test]
fn cursor_reset_requires_exact_current_epoch_authorization_and_new_fence() {
    let (temp, mut store, mut binding) = setup();
    binding.start_cursor = "native-v1:2".into();
    let run = start(&mut store, &binding);
    list(&mut store, &run, "100", &[]);
    assert!(store.test_fence(&run, "native-v1:2").is_err());
    assert!(
        store
            .authorize_initialization_cursor_reset(&run, "native-v1:6")
            .is_err()
    );
    let stale = store.begin_initialization_fence(&run).unwrap();
    store
        .authorize_initialization_cursor_reset(&run, "native-v1:7")
        .unwrap();
    assert!(
        store
            .confirm_initialization_fence(&stale, "native-v1:2")
            .is_err()
    );
    store.test_fence(&run, "native-v1:2").unwrap();
    drop(store);
    let mut store = Store::open(temp.path()).unwrap();
    let reopened = store.initialization_status().unwrap().unwrap().run;
    assert!(
        store
            .authorize_initialization_cursor_reset(&run, "native-v1:7")
            .is_err()
    );
    assert!(store.test_fence(&reopened, "native-v1:2").is_err());
    store
        .authorize_initialization_cursor_reset(&reopened, "native-v1:7")
        .unwrap();
    let resumed = store.resume_initialization(&reopened, &binding).unwrap();
    assert!(store.test_fence(&resumed, "native-v1:2").is_err());
    store
        .authorize_initialization_cursor_reset(&resumed, "native-v1:7")
        .unwrap();
    store.test_fence(&resumed, "native-v1:2").unwrap();
    store.complete_initialization(&resumed).unwrap();
    assert_eq!(store.cursor().unwrap(), "native-v1:2");
    assert!(!store.initialization_writeback_blocked().unwrap());
}

#[test]
fn schema19_candidate_migrates_with_reset_permit_closed_and_old_data_preserved() {
    let (temp, mut store, binding) = setup();
    store.save_local("101", b"dirty migration").unwrap();
    let pending = store.pending_operations().unwrap();
    let run = start(&mut store, &binding);
    list(&mut store, &run, "100", &[]);
    store.db.execute_batch("DROP TRIGGER preview_denied_capture; DROP TRIGGER preview_denied_clear; DROP TABLE preview_denied; DROP TABLE preview_visibility; DROP TABLE preview_inodes; DROP TABLE preview_mount; DROP TABLE initialization_event_hints; DROP TABLE initialization_native_nodes; ALTER TABLE initialization_run DROP COLUMN reset_epoch; ALTER TABLE initialization_run DROP COLUMN reset_expected; DROP INDEX initialization_pending; PRAGMA user_version=19;").unwrap();
    drop(store);
    let store = Store::open(temp.path()).unwrap();
    let current = store.initialization_status().unwrap().unwrap();
    assert!(current.run.epoch > run.epoch);
    assert!(store.initialization_writeback_blocked().unwrap());
    assert_eq!(store.pending_operations().unwrap(), pending);
    assert_eq!(store.cursor().unwrap(), "native-v1:7");
    assert_eq!(
        store
            .db
            .pragma_query_value(None, "user_version", |r| r.get::<_, i64>(0))
            .unwrap(),
        crate::STATE_SCHEMA_VERSION
    );
    let permit: (Option<i64>, Option<String>) = store
        .db
        .query_row(
            "SELECT reset_epoch,reset_expected FROM initialization_run",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert_eq!(permit, (None, None));
    let backup = std::fs::read_dir(temp.path())
        .unwrap()
        .map(|p| p.unwrap().path())
        .find(|p| {
            p.file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with(&format!("state-before-v{}-", crate::STATE_SCHEMA_VERSION))
        })
        .unwrap();
    let old = Connection::open(backup).unwrap();
    assert_eq!(
        old.pragma_query_value(None, "user_version", |r| r.get::<_, i64>(0))
            .unwrap(),
        19
    );
    let columns = old
        .prepare("PRAGMA table_info(initialization_run)")
        .unwrap()
        .query_map([], |r| r.get::<_, String>(1))
        .unwrap()
        .collect::<std::result::Result<Vec<_>, _>>()
        .unwrap();
    assert!(!columns.iter().any(|c| c == "reset_epoch"));
}

#[test]
fn relocating_nested_identities_is_independent_of_incoming_child_order() {
    for reversed in [false, true] {
        let (_temp, mut store, binding) = setup();
        let run = start(&mut store, &binding);
        let other = node("300", Some("100"), "Other", true, 7, "server");
        let outer = node("201", Some("200"), "Outer", true, 7, "server");
        let inner = node("202", Some("201"), "inner", false, 7, "server");
        list(&mut store, &run, "100", &[directory(), other]);
        list(&mut store, &run, "200", &[outer]);
        list(&mut store, &run, "201", &[inner]);
        let outer = node("201", Some("300"), "Outer", true, 9, "server");
        let inner = node("202", Some("300"), "inner", false, 9, "server");
        let mut children = vec![outer.clone(), inner.clone()];
        if reversed {
            children.reverse();
        }
        list(&mut store, &run, "300", &children);
        list(&mut store, &run, "200", &[]);
        list(&mut store, &run, "201", &[]);
        store.test_fence(&run, "native-v1:100").unwrap();
        store.complete_initialization(&run).unwrap();
        assert_eq!(store.node("201").unwrap(), outer);
        assert_eq!(store.node("202").unwrap(), inner);
    }
}

#[test]
fn pending_frontier_promotes_larger_id_ancestor_and_deduplicates_overlapping_paths() {
    let (_temp, mut store, binding) = setup();
    let run = start(&mut store, &binding);
    let high = node("900", Some("100"), "high", true, 7, "server");
    let middle = node("200", Some("900"), "middle", true, 7, "server");
    let a = node("010", Some("200"), "a", true, 7, "server");
    let b = node("020", Some("200"), "b", true, 7, "server");
    list(&mut store, &run, "100", &[high]);
    list(&mut store, &run, "900", &[middle]);
    list(&mut store, &run, "200", &[a, b]);
    invalidate(&store.db, &run.run_id, "900", true).unwrap();
    let pending = store.initialization_pending_directories(&run, 2).unwrap();
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].id, "900");
    store
        .db
        .execute(
            "UPDATE initialization_directories SET state='failed' WHERE id='900'",
            [],
        )
        .unwrap();
    assert_eq!(
        store.initialization_pending_directories(&run, 1).unwrap()[0].id,
        "900"
    );
}

#[test]
fn pending_frontier_skips_loading_ancestor_without_claiming_completion() {
    let (_temp, mut store, binding) = setup();
    let run = start(&mut store, &binding);
    let high = node("900", Some("100"), "high", true, 7, "server");
    let middle = node("200", Some("900"), "middle", true, 7, "server");
    let a = node("010", Some("200"), "a", true, 7, "server");
    list(&mut store, &run, "100", &[high]);
    list(&mut store, &run, "900", &[middle]);
    list(&mut store, &run, "200", &[a]);
    invalidate(&store.db, &run.run_id, "900", true).unwrap();
    let loading = store.begin_initialization_directory(&run, "200").unwrap();
    assert!(
        store
            .initialization_pending_directories(&run, 1)
            .unwrap()
            .is_empty()
    );
    assert!(store.test_fence(&run, "native-v1:100").is_err());
    assert!(
        store
            .initialization_directory_ticket_current(&loading)
            .unwrap()
    );
    store
        .store_initialization_directory(
            &loading,
            1,
            &[node("010", Some("200"), "a", true, 7, "server")],
        )
        .unwrap();
    assert_eq!(
        store.initialization_pending_directories(&run, 1).unwrap()[0].id,
        "900"
    );
}

#[test]
fn pending_frontier_without_pending_ancestors_returns_independent_leaves() {
    let (_temp, mut store, binding) = setup();
    let run = start(&mut store, &binding);
    let high = node("900", Some("100"), "high", true, 7, "server");
    let middle = node("200", Some("900"), "middle", true, 7, "server");
    let a = node("010", Some("200"), "a", true, 7, "server");
    let b = node("020", Some("200"), "b", true, 7, "server");
    list(&mut store, &run, "100", &[high]);
    list(&mut store, &run, "900", &[middle]);
    list(&mut store, &run, "200", &[a, b]);
    let ids = store
        .initialization_pending_directories(&run, 2)
        .unwrap()
        .into_iter()
        .map(|n| n.id)
        .collect::<Vec<_>>();
    assert_eq!(ids, vec!["010", "020"]);
}

#[test]
fn directory_ticket_current_rejects_deleted_recreated_or_invalidated_children() {
    let (_temp, mut store, binding) = setup();
    let run = start(&mut store, &binding);
    list(&mut store, &run, "100", &[directory()]);
    let child = node("201", Some("200"), "child", true, 7, "server");
    list(&mut store, &run, "200", &[child.clone()]);
    let deleted = store.begin_initialization_directory(&run, "201").unwrap();
    assert!(
        store
            .initialization_directory_ticket_current(&deleted)
            .unwrap()
    );
    list(&mut store, &run, "200", &[]);
    assert!(
        !store
            .initialization_directory_ticket_current(&deleted)
            .unwrap()
    );
    list(&mut store, &run, "200", &[child]);
    let current = store.begin_initialization_directory(&run, "201").unwrap();
    assert!(
        !store
            .initialization_directory_ticket_current(&deleted)
            .unwrap()
    );
    assert!(
        store
            .initialization_directory_ticket_current(&current)
            .unwrap()
    );
    invalidate(&store.db, &run.run_id, "200", true).unwrap();
    assert!(
        !store
            .initialization_directory_ticket_current(&current)
            .unwrap()
    );
    assert!(
        store
            .store_initialization_directory(&current, 1, &[])
            .is_err()
    );
    assert_eq!(store.cursor().unwrap(), "native-v1:7");
}
