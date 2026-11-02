//! Unknown directory events are dependencies on real parent lists, not nodes.
use super::*;

fn new_dir() -> Node {
    dir("999", "300", "New")
}
fn new_file() -> Node {
    node("998", Some("999"), "File", false, 7, "server")
}
fn mixed_page(child_first: bool, known: bool) -> InitializationEventPage {
    let mut entries = vec![(new_dir(), "/Lab/Q/New"), (new_file(), "/Lab/Q/New/File")];
    if child_first {
        entries.reverse();
    }
    if known {
        entries.push((d(), "/Lab/P/D"));
    }
    let batch = batch(
        100,
        101,
        entries
            .iter()
            .map(|(n, _)| Change::Upsert(n.clone()))
            .collect(),
    );
    let hints = entries
        .into_iter()
        .enumerate()
        .map(|(ordinal, (n, path))| InitializationEventHint {
            ordinal: ordinal as u64,
            event_id: batch.events[ordinal].id.clone(),
            event_sync_id: 101,
            file_id: n.id,
            kind: InitializationEventKind::Modify,
            event_path: path.into(),
            destination_path: None,
            metadata_parent_id: n.parent,
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

#[test]
fn mixed_page_plans_before_children_and_atomically_preserves_unrelated_coverage() {
    for child_first in [false, true] {
        let (temp, mut store, _binding, run) = prepared();
        let protected = preserved(&store);
        let fence = store.begin_initialization_fence(&run).unwrap();
        store
            .confirm_initialization_fence(&fence, "native-v1:100")
            .unwrap();
        let stale = store.begin_initialization_directory(&run, "300").unwrap();
        store
            .record_initialization_event_page(&run, 1, &mixed_page(child_first, true))
            .unwrap();
        let outcome = store
            .apply_initialization_events_diagnosed(&run, 1, false)
            .unwrap();
        assert_eq!(outcome.result, InitializationApplyResult::Applied);
        assert_eq!(
            outcome.summary,
            InitializationApplySummary {
                local_parent_plans: 2,
                local_subtree_plans: 1,
                unknown_directory_dependency_plans: 1,
                ..InitializationApplySummary::default()
            }
        );
        let replay = store
            .apply_initialization_events_diagnosed(&run, 1, false)
            .unwrap();
        assert_eq!(replay.result, InitializationApplyResult::AlreadyApplied);
        assert_eq!(replay.summary, InitializationApplySummary::default());
        let coverage = states(&store, &run);
        for id in ["200", "210", "211", "300"] {
            assert_eq!(coverage[id], "pending");
        }
        for id in ["100", "220", "400"] {
            assert_eq!(coverage[id], "received");
        }
        assert!(store.initialization_node(&run, "999").is_err());
        assert!(store.initialization_node(&run, "998").is_err());
        assert!(
            !store
                .initialization_directory_ticket_current(&stale)
                .unwrap()
        );
        assert!(
            store
                .store_initialization_native_directory(&stale, 1, &[])
                .is_err()
        );
        assert!(
            store
                .confirm_initialization_fence(&fence, "native-v1:100")
                .is_err()
        );
        let status = store.initialization_status().unwrap().unwrap();
        assert_eq!(status.applied_sequence, 1);
        assert_eq!(status.applied_cursor, "native-v1:101");
        assert!(store.initialization_writeback_blocked().unwrap());
        assert_eq!(preserved(&store), protected);
        drop(store);
        let store = Store::open(temp.path()).unwrap();
        let resumed = store.initialization_status().unwrap().unwrap().run;
        assert_eq!(states(&store, &resumed)["300"], "pending");
        assert_eq!(states(&store, &resumed)["400"], "received");
        assert_eq!(preserved(&store), protected);
    }
}

#[test]
fn real_parent_list_creates_identity_and_child_coverage_is_still_required() {
    let (_temp, mut store, _binding, run) = prepared();
    apply(&mut store, &run, &mixed_page(true, false));
    native_list(&mut store, &run, "300", vec![new_dir()]);
    assert_eq!(
        store.initialization_native_node_id(&run, "999").unwrap(),
        Some(101)
    );
    assert_eq!(states(&store, &run)["999"], "pending");
    let premature = store.begin_initialization_fence(&run).unwrap();
    assert!(
        store
            .confirm_initialization_fence(&premature, "native-v1:101")
            .is_err()
    );
    let ticket = store.begin_initialization_directory(&run, "999").unwrap();
    store
        .store_initialization_native_directory(
            &ticket,
            1,
            &[InitializationDirectoryEntry {
                node: new_file(),
                native_node_id: 202,
            }],
        )
        .unwrap();
    assert_eq!(store.initialization_node(&run, "998").unwrap(), new_file());
    let fence = store.begin_initialization_fence(&run).unwrap();
    store
        .confirm_initialization_fence(&fence, "native-v1:101")
        .unwrap();
}

#[test]
fn parent_absence_replacement_or_denial_converges_without_inventing_event_identity() {
    for outcome in ["absent", "replacement", "denied"] {
        let (_temp, mut store, _binding, run) = prepared();
        let protected = preserved(&store);
        apply(&mut store, &run, &mixed_page(false, false));
        let ticket = store.begin_initialization_directory(&run, "300").unwrap();
        match outcome {
            "denied" => {
                store.store_initialization_denied(&ticket, 1).unwrap();
            }
            "replacement" => {
                let replacement = dir("997", "300", "New");
                store
                    .store_initialization_native_directory(
                        &ticket,
                        1,
                        &[InitializationDirectoryEntry {
                            node: replacement,
                            native_node_id: 203,
                        }],
                    )
                    .unwrap();
                let child = store.begin_initialization_directory(&run, "997").unwrap();
                store
                    .store_initialization_native_directory(&child, 1, &[])
                    .unwrap();
            }
            _ => {
                store
                    .store_initialization_native_directory(&ticket, 1, &[])
                    .unwrap();
            }
        }
        assert!(store.initialization_node(&run, "999").is_err());
        assert!(store.initialization_node(&run, "998").is_err());
        assert_eq!(states(&store, &run)["400"], "received");
        let fence = store.begin_initialization_fence(&run).unwrap();
        store
            .confirm_initialization_fence(&fence, "native-v1:101")
            .unwrap();
        assert_eq!(preserved(&store), protected);
    }
}

#[test]
fn untrusted_or_uncovered_dependencies_keep_conservative_recheck() {
    for mode in [
        "path",
        "native_missing",
        "sticky_denied",
        "file_path",
        "file_parent",
        "duplicate_directory",
        "collision",
    ] {
        let (_temp, mut store, _binding, run) = prepared();
        let mut page = mixed_page(false, false);
        match mode {
            "path" => page.hints[0].event_path = "/Lab/P/New".into(),
            "native_missing" => {
                store
                    .db
                    .execute(
                        "DELETE FROM initialization_native_nodes WHERE run=?1 AND id='300'",
                        [&run.run_id],
                    )
                    .unwrap();
            }
            "sticky_denied" => {
                let ticket = store.begin_initialization_directory(&run, "300").unwrap();
                store.store_initialization_denied(&ticket, 1).unwrap();
                store.begin_initialization_directory(&run, "300").unwrap();
            }
            "file_path" => page.hints[1].event_path = "/Lab/Other/File".into(),
            "file_parent" => {
                if let Change::Upsert(n) = &mut page.batch.events[1].change {
                    n.parent = Some("996".into());
                }
                page.hints[1].metadata_parent_id = Some("996".into());
            }
            "duplicate_directory" => {
                let mut event = page.batch.events[0].clone();
                event.id = "distinct-event".into();
                let mut hint = page.hints[0].clone();
                hint.event_id = event.id.clone();
                hint.ordinal = 2;
                page.batch.events.push(event);
                page.hints.push(hint);
            }
            "collision" => {
                let ticket = store.begin_initialization_directory(&run, "300").unwrap();
                store
                    .store_initialization_native_directory(
                        &ticket,
                        1,
                        &[InitializationDirectoryEntry {
                            node: dir("997", "300", "New"),
                            native_node_id: 203,
                        }],
                    )
                    .unwrap();
            }
            _ => unreachable!(),
        }
        apply(&mut store, &run, &page);
        assert_eq!(states(&store, &run)["400"], "pending", "{mode}");
        assert!(store.initialization_node(&run, "999").is_err());
    }
}

#[test]
fn corrupt_parent_native_binding_fails_without_advancing_or_changing_coverage() {
    let (_temp, mut store, _binding, run) = prepared();
    store
        .record_initialization_event_page(&run, 1, &mixed_page(false, false))
        .unwrap();
    let before = states(&store, &run);
    let protected = preserved(&store);
    store
        .db
        .execute(
            "UPDATE initialization_native_nodes SET digest='bad' WHERE run=?1 AND id='300'",
            [&run.run_id],
        )
        .unwrap();
    let error = store.apply_initialization_events(&run, 1).unwrap_err();
    assert!(error.to_string().contains("identity digest mismatch"));
    assert_eq!(states(&store, &run), before);
    assert_eq!(
        store
            .initialization_status()
            .unwrap()
            .unwrap()
            .applied_sequence,
        0
    );
    assert_eq!(preserved(&store), protected);
}

#[test]
fn failure_to_persist_parent_obligation_cannot_advance_event_cursor() {
    let (_temp, mut store, _binding, run) = prepared();
    let page = mixed_page(false, false);
    store
        .record_initialization_event_page(&run, 1, &page)
        .unwrap();
    let protected = preserved(&store);
    let before = states(&store, &run);
    store.db.execute_batch("CREATE TEMP TRIGGER reject_parent_pending BEFORE UPDATE OF state ON initialization_directories WHEN old.id='300' AND new.state='pending' BEGIN SELECT RAISE(ABORT,'injected obligation failure'); END;").unwrap();
    assert!(store.apply_initialization_events(&run, 1).is_err());
    assert_eq!(
        store
            .initialization_status()
            .unwrap()
            .unwrap()
            .applied_sequence,
        0
    );
    assert_eq!(states(&store, &run), before);
    assert_eq!(preserved(&store), protected);
}

#[test]
fn denied_parent_revokes_parallel_old_descendant_receipt() {
    let (_temp, mut store, _binding, run) = prepared();
    let ticket = store.begin_initialization_directory(&run, "300").unwrap();
    store
        .store_initialization_native_directory(
            &ticket,
            1,
            &[InitializationDirectoryEntry {
                node: dir("995", "300", "Existing"),
                native_node_id: 205,
            }],
        )
        .unwrap();
    let stale_child = store.begin_initialization_directory(&run, "995").unwrap();
    apply(&mut store, &run, &mixed_page(false, false));
    let parent = store.begin_initialization_directory(&run, "300").unwrap();
    store.store_initialization_denied(&parent, 1).unwrap();
    assert!(store.initialization_node(&run, "995").is_err());
    assert!(
        !store
            .initialization_directory_ticket_current(&stale_child)
            .unwrap()
    );
    assert!(
        store
            .store_initialization_native_directory(&stale_child, 1, &[])
            .is_err()
    );
    assert_eq!(states(&store, &run)["400"], "received");
}

#[test]
fn diagnosed_coverage_wait_has_no_successful_plan_counts() {
    let (_temp, mut store, _binding, run) = prepared();
    store.begin_initialization_directory(&run, "400").unwrap();
    let event = page(
        Change::Upsert(new_file()),
        InitializationEventKind::Modify,
        "/Lab/Q/New/File",
        None,
    );
    store
        .record_initialization_event_page(&run, 1, &event)
        .unwrap();
    let outcome = store
        .apply_initialization_events_diagnosed(&run, 1, false)
        .unwrap();
    assert_eq!(outcome.result, InitializationApplyResult::AwaitingCoverage);
    assert_eq!(
        outcome.summary,
        InitializationApplySummary {
            awaiting_coverage_reason: Some(InitializationCoverageWaitReason::MissingFileParent),
            ..InitializationApplySummary::default()
        }
    );
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
fn diagnosed_full_recheck_reports_priority_and_zero_local_plans() {
    for with_root in [false, true] {
        let (_temp, mut store, _binding, run) = prepared();
        let event = if with_root {
            page(
                Change::Upsert(root()),
                InitializationEventKind::Modify,
                "/Lab",
                None,
            )
        } else {
            mixed_page(false, true)
        };
        store
            .record_initialization_event_page(&run, 1, &event)
            .unwrap();
        let outcome = store
            .apply_initialization_events_diagnosed(&run, 1, true)
            .unwrap();
        assert_eq!(outcome.result, InitializationApplyResult::Applied);
        assert_eq!(
            outcome.summary,
            InitializationApplySummary {
                full_recheck_reason: Some(if with_root {
                    InitializationFullRecheckReason::Root
                } else {
                    InitializationFullRecheckReason::DirectoryRace
                }),
                ..InitializationApplySummary::default()
            }
        );
    }
}

#[test]
fn oversized_unknown_directory_plan_uses_bounded_conservative_fallback() {
    let (_temp, mut store, _binding, run) = prepared();
    let changes = (0..65)
        .map(|i| Change::Upsert(dir(&format!("n{i}"), "300", &format!("New{i}"))))
        .collect();
    let batch = batch(100, 101, changes);
    let hints = batch
        .events
        .iter()
        .enumerate()
        .map(|(ordinal, event)| {
            let Change::Upsert(node) = &event.change else {
                unreachable!()
            };
            InitializationEventHint {
                ordinal: ordinal as u64,
                event_id: event.id.clone(),
                event_sync_id: 101,
                file_id: node.id.clone(),
                kind: InitializationEventKind::Modify,
                event_path: format!("/Lab/Q/{}", node.name),
                destination_path: None,
                metadata_parent_id: node.parent.clone(),
                destination_parent_id: None,
                metadata_removed: false,
            }
        })
        .collect();
    let page = InitializationEventPage {
        batch,
        hints,
        scope: serde_json::to_value(scope()).unwrap(),
        restore: "server".into(),
    };
    store
        .record_initialization_event_page(&run, 1, &page)
        .unwrap();
    let outcome = store
        .apply_initialization_events_diagnosed(&run, 1, false)
        .unwrap();
    assert_eq!(
        outcome.summary,
        InitializationApplySummary {
            full_recheck_reason: Some(InitializationFullRecheckReason::UnsupportedDirectory),
            ..InitializationApplySummary::default()
        }
    );
    assert_eq!(states(&store, &run)["400"], "pending");
}
