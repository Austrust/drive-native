//! Dependency scheduling tests use real durable candidate transactions.
use super::*;

fn missing_parent_event() -> Node {
    node("301", Some("300"), "Later.txt", false, 101, "server")
}
fn coverage(store: &Store, run: &InitializationRun, id: &str) -> String {
    store
        .initialization_coverage(run)
        .unwrap()
        .into_iter()
        .find(|entry| entry.directory == id)
        .unwrap()
        .state
}
fn receive_missing(store: &mut Store, run: &InitializationRun) {
    store
        .record_initialization_events(
            run,
            1,
            &batch(100, 101, vec![Change::Upsert(missing_parent_event())]),
        )
        .unwrap();
}

#[test]
fn unfinished_parent_dependency_preserves_completed_branches_and_newer_content() {
    let (_temp, mut store, binding) = setup();
    store.save_local("101", b"offline").unwrap();
    let pending = store.pending_operations().unwrap();
    let run = start(&mut store, &binding);
    let done = node("400", Some("100"), "Complete", true, 7, "server");
    list(&mut store, &run, "100", &[directory(), done, file()]);
    list(&mut store, &run, "400", &[]);
    receive_missing(&mut store, &run);
    let before = store.initialization_coverage(&run).unwrap();
    for _ in 0..2 {
        assert_eq!(
            store.apply_initialization_events(&run, 1).unwrap(),
            InitializationApplyResult::AwaitingCoverage
        );
        assert_eq!(store.initialization_coverage(&run).unwrap(), before);
    }
    let state = store.initialization_status().unwrap().unwrap();
    assert_eq!((state.received_sequence, state.applied_sequence), (1, 0));
    assert_eq!(state.applied_cursor, "native-v1:100");
    assert_eq!(store.cursor().unwrap(), "native-v1:7");
    assert_eq!(store.pending_operations().unwrap(), pending);
    assert!(store.test_fence(&run, "native-v1:101").is_err());
    assert!(store.complete_initialization(&run).is_err());
    let parent = node("300", Some("200"), "Later", true, 7, "server");
    list(&mut store, &run, "200", &[parent]);
    assert_eq!(
        store.apply_initialization_events(&run, 1).unwrap(),
        InitializationApplyResult::Applied
    );
    assert_eq!(coverage(&store, &run, "100"), "received");
    assert_eq!(coverage(&store, &run, "400"), "received");
    // An old event never writes its version over a current directory response.
    let newer = node("301", Some("300"), "Later.txt", false, 150, "server");
    list(&mut store, &run, "300", &[newer.clone()]);
    store.test_fence(&run, "native-v1:101").unwrap();
    store.complete_initialization(&run).unwrap();
    assert_eq!(store.node("301").unwrap(), newer);
}

#[test]
fn deferred_receipt_survives_reopen_without_product_checkpoint_or_writeback() {
    let (temp, mut store, binding) = setup();
    let run = start(&mut store, &binding);
    list(&mut store, &run, "100", &[directory()]);
    receive_missing(&mut store, &run);
    assert_eq!(
        store.apply_initialization_events(&run, 1).unwrap(),
        InitializationApplyResult::AwaitingCoverage
    );
    drop(store);
    let mut store = Store::open(temp.path()).unwrap();
    let run = store.initialization_status().unwrap().unwrap().run;
    let run = store.resume_initialization(&run, &binding).unwrap();
    assert!(store.initialization_writeback_blocked().unwrap());
    assert_eq!(store.cursor().unwrap(), "native-v1:7");
    assert_eq!(
        store.pending_initialization_events(&run, 2).unwrap().len(),
        1
    );
    assert_eq!(
        store.apply_initialization_events(&run, 1).unwrap(),
        InitializationApplyResult::AwaitingCoverage
    );
    assert_eq!(coverage(&store, &run, "100"), "received");
}

#[test]
fn complete_coverage_with_unresolved_parent_uses_original_conservative_fallback() {
    let (_temp, mut store, binding) = setup();
    let run = start(&mut store, &binding);
    list(&mut store, &run, "100", &[directory()]);
    receive_missing(&mut store, &run);
    assert_eq!(
        store.apply_initialization_events(&run, 1).unwrap(),
        InitializationApplyResult::AwaitingCoverage
    );
    list(&mut store, &run, "200", &[]);
    assert_eq!(
        store.apply_initialization_events(&run, 1).unwrap(),
        InitializationApplyResult::Applied
    );
    assert!(
        store
            .initialization_coverage(&run)
            .unwrap()
            .iter()
            .all(|row| row.state == "pending")
    );
    assert!(store.initialization_node(&run, "301").is_err());
    assert!(store.test_fence(&run, "native-v1:101").is_err());
}

#[test]
fn loading_ancestor_is_not_complete_even_with_empty_pending_frontier() {
    let (_temp, mut store, binding) = setup();
    let run = start(&mut store, &binding);
    list(&mut store, &run, "100", &[directory()]);
    let loading = store.begin_initialization_directory(&run, "200").unwrap();
    assert!(
        store
            .initialization_pending_directories(&run, 1)
            .unwrap()
            .is_empty()
    );
    receive_missing(&mut store, &run);
    assert_eq!(
        store.apply_initialization_events(&run, 1).unwrap(),
        InitializationApplyResult::AwaitingCoverage
    );
    assert!(
        store
            .initialization_directory_ticket_current(&loading)
            .unwrap()
    );
    store.store_initialization_denied(&loading, 1).unwrap();
    assert_eq!(
        store.apply_initialization_events(&run, 1).unwrap(),
        InitializationApplyResult::Applied
    );
    assert_eq!(coverage(&store, &run, "200"), "pending");
}

#[test]
fn dangerous_events_in_either_order_are_never_hidden_by_missing_file_parent() {
    let hazards = vec![
        Change::Upsert(node("100", None, "Lab", true, 101, "server")),
        Change::Upsert(node("200", Some("100"), "Renamed", true, 101, "server")),
        Change::Delete { id: "200".into() },
        Change::Delete {
            id: "unknown".into(),
        },
        // A parent known to be a regular file is not an enumeration dependency.
        Change::Upsert(node("999", Some("101"), "BadParent", false, 101, "server")),
    ];
    for hazard in hazards {
        for first in [true, false] {
            let (_temp, mut store, binding) = setup();
            let run = start(&mut store, &binding);
            list(&mut store, &run, "100", &[directory(), file()]);
            let mut changes = vec![Change::Upsert(missing_parent_event()), hazard.clone()];
            if first {
                changes.reverse();
            }
            store
                .record_initialization_events(&run, 1, &batch(100, 101, changes))
                .unwrap();
            assert_eq!(
                store.apply_initialization_events(&run, 1).unwrap(),
                InitializationApplyResult::Applied
            );
            assert_eq!(coverage(&store, &run, "100"), "pending");
            assert_eq!(store.cursor().unwrap(), "native-v1:7");
        }
    }
}

#[test]
fn root_removal_in_deferred_batch_remains_an_error_without_application() {
    let (_temp, mut store, binding) = setup();
    let run = start(&mut store, &binding);
    store
        .record_initialization_events(
            &run,
            1,
            &batch(
                100,
                101,
                vec![
                    Change::Upsert(missing_parent_event()),
                    Change::Delete { id: "100".into() },
                ],
            ),
        )
        .unwrap();
    assert!(store.apply_initialization_events(&run, 1).is_err());
    assert_eq!(
        store
            .initialization_status()
            .unwrap()
            .unwrap()
            .applied_sequence,
        0
    );
}

#[test]
fn deferred_move_rechecks_old_and_new_parents_and_preserves_same_name_replacement() {
    let (_temp, mut store, binding) = setup();
    let run = start(&mut store, &binding);
    let branch = node("400", Some("100"), "NewBranch", true, 7, "server");
    list(&mut store, &run, "100", &[directory(), branch]);
    let old = node("301", Some("200"), "Same.txt", false, 7, "server");
    list(&mut store, &run, "200", &[old]);
    let moved = node("301", Some("300"), "Same.txt", false, 101, "server");
    store
        .record_initialization_events(
            &run,
            1,
            &batch(100, 101, vec![Change::Upsert(moved.clone())]),
        )
        .unwrap();
    assert_eq!(
        store.apply_initialization_events(&run, 1).unwrap(),
        InitializationApplyResult::AwaitingCoverage
    );
    assert_eq!(coverage(&store, &run, "200"), "received");
    let parent = node("300", Some("400"), "NewParent", true, 7, "server");
    list(&mut store, &run, "400", &[parent]);
    let stale = store.begin_initialization_directory(&run, "200").unwrap();
    assert_eq!(
        store.apply_initialization_events(&run, 1).unwrap(),
        InitializationApplyResult::Applied
    );
    assert!(
        !store
            .initialization_directory_ticket_current(&stale)
            .unwrap()
    );
    assert!(
        store
            .store_initialization_directory(&stale, 1, &[])
            .is_err()
    );
    assert_eq!(coverage(&store, &run, "200"), "pending");
    assert_eq!(coverage(&store, &run, "300"), "pending");
    let replacement = node("302", Some("200"), "Same.txt", false, 102, "server");
    // Destination-first is legal even though the old identity is still present.
    list(&mut store, &run, "300", &[moved.clone()]);
    list(&mut store, &run, "200", &[replacement.clone()]);
    store.test_fence(&run, "native-v1:101").unwrap();
    store.complete_initialization(&run).unwrap();
    assert_eq!(store.node("301").unwrap(), moved);
    assert_eq!(store.node("302").unwrap(), replacement);
}

#[test]
fn sequence_only_lookup_does_not_decode_body_but_rejects_missing_receipts() {
    let (_temp, mut store, binding) = setup();
    let run = start(&mut store, &binding);
    assert_eq!(
        store.initialization_next_pending_sequence(&run).unwrap(),
        None
    );
    receive_missing(&mut store, &run);
    assert_eq!(
        store.initialization_next_pending_sequence(&run).unwrap(),
        Some(1)
    );
    store
        .record_initialization_events(&run, 2, &batch(101, 102, vec![]))
        .unwrap();
    // Fault injection proves the seq-only call does not parse the body; the
    // application boundary still rejects it, and a gap is not skipped to seq 2.
    store
        .db
        .execute(
            "UPDATE initialization_events SET body='{' WHERE run=?1 AND seq=1",
            [&run.run_id],
        )
        .unwrap();
    assert_eq!(
        store.initialization_next_pending_sequence(&run).unwrap(),
        Some(1)
    );
    assert!(store.apply_initialization_events(&run, 1).is_err());
    store
        .db
        .execute(
            "DELETE FROM initialization_events WHERE run=?1 AND seq=1",
            [&run.run_id],
        )
        .unwrap();
    assert!(store.initialization_next_pending_sequence(&run).is_err());
    assert_eq!(
        store
            .initialization_status()
            .unwrap()
            .unwrap()
            .applied_sequence,
        0
    );
}

#[test]
fn sequence_only_lookup_rejects_old_epoch_and_tracks_applied_head() {
    let (temp, mut store, binding) = setup();
    let old = start(&mut store, &binding);
    receive_missing(&mut store, &old);
    drop(store);
    let mut store = Store::open(temp.path()).unwrap();
    let run = store.initialization_status().unwrap().unwrap().run;
    assert!(store.initialization_next_pending_sequence(&old).is_err());
    assert_eq!(
        store.initialization_next_pending_sequence(&run).unwrap(),
        Some(1)
    );
    store
        .apply_initialization_events_after_directory_race(&run, 1)
        .unwrap();
    assert_eq!(
        store.initialization_next_pending_sequence(&run).unwrap(),
        None
    );
    assert_eq!(store.cursor().unwrap(), "native-v1:7");
    assert!(store.initialization_writeback_blocked().unwrap());
}

#[test]
fn directory_race_fallback_applies_receipt_and_invalidates_coverage_atomically() {
    let (_temp, mut store, binding) = setup();
    let run = start(&mut store, &binding);
    list(&mut store, &run, "100", &[directory()]);
    receive_missing(&mut store, &run);
    let before = store.initialization_coverage(&run).unwrap();
    assert_eq!(
        store.apply_initialization_events(&run, 1).unwrap(),
        InitializationApplyResult::AwaitingCoverage
    );
    store.db.execute_batch("CREATE TEMP TRIGGER fail_coverage_escape BEFORE UPDATE OF applied ON initialization_run BEGIN SELECT RAISE(ABORT,'synthetic commit failure'); END;").unwrap();
    assert!(
        store
            .apply_initialization_events_after_directory_race(&run, 1)
            .is_err()
    );
    assert_eq!(store.initialization_coverage(&run).unwrap(), before);
    assert_eq!(
        store
            .initialization_status()
            .unwrap()
            .unwrap()
            .applied_sequence,
        0
    );
    store
        .db
        .execute_batch("DROP TRIGGER fail_coverage_escape;")
        .unwrap();
    assert_eq!(
        store
            .apply_initialization_events_after_directory_race(&run, 1)
            .unwrap(),
        InitializationApplyResult::Applied
    );
    assert!(
        store
            .initialization_coverage(&run)
            .unwrap()
            .iter()
            .all(|s| s.state == "pending")
    );
    assert_eq!(
        store
            .initialization_status()
            .unwrap()
            .unwrap()
            .applied_sequence,
        1
    );
    assert!(store.test_fence(&run, "native-v1:101").is_err());
    assert!(store.complete_initialization(&run).is_err());
    assert_eq!(store.cursor().unwrap(), "native-v1:7");
}
