//! Page-local nested dependencies never stand in for real listed identities.
use super::*;

fn nested_dir() -> Node {
    dir("994", "999", "Nested")
}
fn nested_file() -> Node {
    node("993", Some("994"), "Leaf", false, 7, "server")
}
fn nested_page(order: [usize; 3]) -> InitializationEventPage {
    let nodes = [new_dir(), nested_dir(), nested_file()];
    let paths = ["/Lab/Q/New", "/Lab/Q/New/Nested", "/Lab/Q/New/Nested/Leaf"];
    let batch = batch(
        100,
        101,
        order
            .iter()
            .map(|&i| Change::Upsert(nodes[i].clone()))
            .collect(),
    );
    let hints = order
        .iter()
        .enumerate()
        .map(|(ordinal, &i)| InitializationEventHint {
            ordinal: ordinal as u64,
            event_id: batch.events[ordinal].id.clone(),
            event_sync_id: 101,
            file_id: nodes[i].id.clone(),
            kind: InitializationEventKind::Modify,
            event_path: paths[i].into(),
            destination_path: None,
            metadata_parent_id: nodes[i].parent.clone(),
            destination_parent_id: None,
            metadata_removed: false,
        })
        .collect();
    InitializationEventPage {
        batch,
        hints,
        scope: serde_json::to_value(scope()).unwrap(),
        restore: "server".into(),
    }
}

fn apply_diagnosed(
    store: &mut Store,
    run: &InitializationRun,
    page: &InitializationEventPage,
) -> InitializationApplyOutcome {
    store
        .record_initialization_event_page(run, 1, page)
        .unwrap();
    store
        .apply_initialization_events_diagnosed(run, 1, false)
        .unwrap()
}

#[test]
fn nested_plan_is_order_independent_and_only_the_real_anchor_becomes_pending() {
    for order in [
        [0, 1, 2],
        [0, 2, 1],
        [1, 0, 2],
        [1, 2, 0],
        [2, 0, 1],
        [2, 1, 0],
    ] {
        let (temp, mut store, _binding, run) = prepared();
        let protected = preserved(&store);
        let outcome = apply_diagnosed(&mut store, &run, &nested_page(order));
        assert_eq!(
            outcome.summary,
            InitializationApplySummary {
                local_parent_plans: 1,
                unknown_directory_dependency_plans: 2,
                ..InitializationApplySummary::default()
            }
        );
        let coverage = states(&store, &run);
        assert_eq!(coverage["300"], "pending");
        for id in ["100", "200", "210", "211", "220", "400"] {
            assert_eq!(coverage[id], "received");
        }
        for id in ["999", "994", "993"] {
            assert!(store.initialization_node(&run, id).is_err());
        }
        assert_eq!(
            store
                .initialization_status()
                .unwrap()
                .unwrap()
                .applied_sequence,
            1
        );
        assert!(store.initialization_writeback_blocked().unwrap());
        assert_eq!(preserved(&store), protected);
        drop(store);
        let store = Store::open(temp.path()).unwrap();
        let reopened = store.initialization_status().unwrap().unwrap().run;
        assert_eq!(states(&store, &reopened)["300"], "pending");
        assert_eq!(states(&store, &reopened)["400"], "received");
        assert_eq!(preserved(&store), protected);
    }
}

#[test]
fn nested_nodes_only_appear_after_separate_native_lists_and_fence_waits() {
    let (_temp, mut store, _binding, run) = prepared();
    apply_diagnosed(&mut store, &run, &nested_page([2, 1, 0]));
    native_list(&mut store, &run, "300", vec![new_dir()]);
    assert!(store.initialization_node(&run, "994").is_err());
    let premature = store.begin_initialization_fence(&run).unwrap();
    assert!(
        store
            .confirm_initialization_fence(&premature, "native-v1:101")
            .is_err()
    );
    let first = store.begin_initialization_directory(&run, "999").unwrap();
    store
        .store_initialization_native_directory(
            &first,
            1,
            &[InitializationDirectoryEntry {
                node: nested_dir(),
                native_node_id: 204,
            }],
        )
        .unwrap();
    assert_eq!(
        store.initialization_native_node_id(&run, "994").unwrap(),
        Some(204)
    );
    assert_eq!(states(&store, &run)["994"], "pending");
    assert!(store.initialization_node(&run, "993").is_err());
    let last = store.begin_initialization_directory(&run, "994").unwrap();
    store
        .store_initialization_native_directory(
            &last,
            1,
            &[InitializationDirectoryEntry {
                node: nested_file(),
                native_node_id: 205,
            }],
        )
        .unwrap();
    let fence = store.begin_initialization_fence(&run).unwrap();
    store
        .confirm_initialization_fence(&fence, "native-v1:101")
        .unwrap();
}

#[test]
fn nested_path_id_cycle_and_same_name_ambiguities_never_bootstrap_a_plan() {
    for mode in [
        "cycle",
        "parent_path",
        "same_name",
        "missing_evidence",
        "orphan",
        "known_elsewhere",
    ] {
        let (_temp, mut store, _binding, run) = prepared();
        let mut page = nested_page([0, 1, 2]);
        match mode {
            "cycle" => {
                if let Change::Upsert(n) = &mut page.batch.events[0].change {
                    n.parent = Some("994".into());
                }
                page.hints[0].metadata_parent_id = Some("994".into());
            }
            "parent_path" => page.hints[1].event_path = "/Lab/Other/Nested".into(),
            "same_name" => {
                let mut event = page.batch.events[0].clone();
                event.id = "other-event".into();
                event.change = Change::Upsert(dir("992", "300", "New"));
                let mut hint = page.hints[0].clone();
                hint.ordinal = 3;
                hint.event_id = event.id.clone();
                hint.file_id = "992".into();
                page.batch.events.push(event);
                page.hints.push(hint);
            }
            "missing_evidence" => {
                // A live file parent absent from this page cannot use a guessed path.
                if let Change::Upsert(n) = &mut page.batch.events[2].change {
                    n.parent = Some("991".into());
                }
                page.hints[2].metadata_parent_id = Some("991".into());
            }
            "orphan" => {
                // Deliberate private DB corruption, not a server/listing operation.
                store
                    .db
                    .execute(
                        "UPDATE initialization_entries SET parent='999' WHERE run=?1 AND id='400'",
                        [&run.run_id],
                    )
                    .unwrap();
            }
            "known_elsewhere" => {
                let ticket = store.begin_initialization_directory(&run, "200").unwrap();
                store
                    .store_initialization_native_directory(
                        &ticket,
                        1,
                        &[
                            entry(d()),
                            entry(sibling()),
                            InitializationDirectoryEntry {
                                node: dir("994", "200", "Existing"),
                                native_node_id: 204,
                            },
                        ],
                    )
                    .unwrap();
            }
            _ => unreachable!(),
        }
        let outcome = apply_diagnosed(&mut store, &run, &page);
        assert!(outcome.summary.full_recheck_reason.is_some(), "{mode}");
        assert_eq!(
            outcome.summary.unknown_directory_dependency_plans, 0,
            "{mode}"
        );
        assert!(store.initialization_node(&run, "999").is_err());
    }
}

#[test]
fn known_parent_move_and_nested_dependencies_schedule_both_locations_in_either_order() {
    for move_first in [false, true] {
        let (_temp, mut store, _binding, run) = prepared();
        let mut page = nested_page([2, 1, 0]);
        let mut moved = q();
        moved.parent = Some("200".into());
        moved.name = "QMoved".into();
        let move_page = super::super::page(
            Change::Upsert(moved),
            InitializationEventKind::Move,
            "/Lab/Q",
            Some(("/Lab/P/QMoved", "200")),
        );
        let mut event = move_page.batch.events[0].clone();
        event.id = "move-parent-event".into();
        let mut hint = move_page.hints[0].clone();
        hint.event_id = event.id.clone();
        if move_first {
            page.batch.events.insert(0, event);
            page.hints.insert(0, hint);
        } else {
            page.batch.events.push(event);
            page.hints.push(hint);
        }
        for (i, h) in page.hints.iter_mut().enumerate() {
            h.ordinal = i as u64;
        }
        let outcome = apply_diagnosed(&mut store, &run, &page);
        assert_eq!(outcome.summary.full_recheck_reason, None);
        assert_eq!(outcome.summary.unknown_directory_dependency_plans, 2);
        for id in ["100", "200", "300"] {
            assert_eq!(states(&store, &run)[id], "pending");
        }
        assert_eq!(states(&store, &run)["400"], "received");
    }
}

#[test]
fn unknown_remove_retains_full_fallback_even_when_path_names_a_current_replacement() {
    let (_temp, mut store, _binding, run) = prepared();
    let event = super::super::page(
        Change::Delete {
            id: "old-missing".into(),
        },
        InitializationEventKind::Remove,
        "/Lab/P/D",
        None,
    );
    let before = store.initialization_node(&run, "211").unwrap();
    let outcome = apply_diagnosed(&mut store, &run, &event);
    assert_eq!(
        outcome.summary.full_recheck_reason,
        Some(InitializationFullRecheckReason::UnknownDelete)
    );
    assert_eq!(outcome.summary.local_parent_plans, 0);
    assert_eq!(store.initialization_node(&run, "211").unwrap(), before);
    assert_eq!(states(&store, &run)["400"], "pending");
}

#[test]
fn real_nested_listing_absence_denial_or_replacement_does_not_replay_hint_contents() {
    for outcome in ["absent", "denied", "replacement"] {
        let (_temp, mut store, _binding, run) = prepared();
        let protected = preserved(&store);
        apply_diagnosed(&mut store, &run, &nested_page([2, 0, 1]));
        native_list(&mut store, &run, "300", vec![new_dir()]);
        let outer = store.begin_initialization_directory(&run, "999").unwrap();
        match outcome {
            "denied" => {
                store.store_initialization_denied(&outer, 1).unwrap();
            }
            "replacement" => {
                store
                    .store_initialization_native_directory(
                        &outer,
                        1,
                        &[InitializationDirectoryEntry {
                            node: dir("997", "999", "Nested"),
                            native_node_id: 206,
                        }],
                    )
                    .unwrap();
                let replacement = store.begin_initialization_directory(&run, "997").unwrap();
                store
                    .store_initialization_native_directory(&replacement, 1, &[])
                    .unwrap();
            }
            _ => {
                store
                    .store_initialization_native_directory(&outer, 1, &[])
                    .unwrap();
            }
        }
        for id in ["994", "993"] {
            assert!(store.initialization_node(&run, id).is_err());
        }
        assert_eq!(states(&store, &run)["400"], "received");
        let fence = store.begin_initialization_fence(&run).unwrap();
        store
            .confirm_initialization_fence(&fence, "native-v1:101")
            .unwrap();
        assert_eq!(preserved(&store), protected);
    }
}
