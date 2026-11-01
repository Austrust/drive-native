//! Known-directory local scheduling; real Store transactions, no server calls.
use super::*;
use std::collections::BTreeMap;

fn dir(id: &str, parent: &str, name: &str) -> Node {
    node(id, Some(parent), name, true, 7, "server")
}
fn p() -> Node {
    dir("200", "100", "P")
}
fn q() -> Node {
    dir("300", "100", "Q")
}
fn unrelated() -> Node {
    dir("400", "100", "Unrelated")
}
fn d() -> Node {
    dir("210", "200", "D")
}
fn child() -> Node {
    dir("211", "210", "Child")
}
fn sibling() -> Node {
    dir("220", "200", "Sibling")
}
fn entry(node: Node) -> InitializationDirectoryEntry {
    // Distinct synthetic native identities, never parsed from file IDs.
    let native_node_id = match node.id.as_str() {
        "101" => 51,
        "200" => 71,
        "300" => 72,
        "400" => 73,
        "210" => 81,
        "211" => 91,
        "220" => 82,
        "999" => 101,
        _ => panic!("unregistered fixture native identity"),
    };
    InitializationDirectoryEntry {
        node,
        native_node_id,
    }
}
fn native_list(store: &mut Store, run: &InitializationRun, id: &str, nodes: Vec<Node>) {
    let ticket = store.begin_initialization_directory(run, id).unwrap();
    let entries = nodes.into_iter().map(entry).collect::<Vec<_>>();
    store
        .store_initialization_native_directory(&ticket, 1, &entries)
        .unwrap();
}
fn prepared() -> (
    tempfile::TempDir,
    Store,
    InitializationBinding,
    InitializationRun,
) {
    let (temp, mut store, binding) = setup();
    store
        .publish_download(
            &crate::DownloadTicket {
                id: "101".into(),
                version: file().version,
            },
            b"old",
            &crate::sha256(b"old"),
        )
        .unwrap();
    let draft = store.begin_draft("101", &file().version, None).unwrap();
    store.draft_write(&draft, 0, b"dirty", false).unwrap();
    store.checkpoint_draft(&draft).unwrap();
    drop(draft);
    let run = start(&mut store, &binding);
    native_list(&mut store, &run, "100", vec![file(), p(), q(), unrelated()]);
    native_list(&mut store, &run, "200", vec![d(), sibling()]);
    native_list(&mut store, &run, "210", vec![child()]);
    for id in ["211", "220", "300", "400"] {
        native_list(&mut store, &run, id, vec![]);
    }
    (temp, store, binding, run)
}
fn states(store: &Store, run: &InitializationRun) -> BTreeMap<String, String> {
    store
        .initialization_coverage(run)
        .unwrap()
        .into_iter()
        .map(|c| (c.directory, c.state))
        .collect()
}
fn preserved(store: &Store) -> (String, Vec<u8>, Vec<u8>, i64) {
    let draft = store.open_dirty_draft("101").unwrap().unwrap();
    let mut dirty = vec![0; 5];
    assert_eq!(draft.read_at(&mut dirty, 0).unwrap(), 5);
    let cache = store.open_cached("101").unwrap();
    let mut body = vec![0; 3];
    assert_eq!(cache.read_at(&mut body, 0).unwrap(), 3);
    (
        store.cursor().unwrap(),
        dirty,
        body,
        store.pending_operations().unwrap(),
    )
}
fn page(
    change: Change,
    kind: InitializationEventKind,
    source: &str,
    dest: Option<(&str, &str)>,
) -> InitializationEventPage {
    let (id, parent, removed) = match &change {
        Change::Upsert(node) => (node.id.clone(), node.parent.clone(), false),
        Change::Delete { id } => (id.clone(), None, true),
    };
    let batch = batch(100, 101, vec![change]);
    InitializationEventPage {
        hints: vec![InitializationEventHint {
            ordinal: 0,
            event_id: batch.events[0].id.clone(),
            event_sync_id: 101,
            file_id: id,
            kind,
            event_path: source.into(),
            destination_path: dest.map(|(path, _)| path.into()),
            metadata_parent_id: parent,
            destination_parent_id: dest.map(|(_, parent)| parent.into()),
            metadata_removed: removed,
        }],
        batch,
        scope: serde_json::to_value(scope()).unwrap(),
        restore: "server".into(),
    }
}
fn modify() -> InitializationEventPage {
    page(
        Change::Upsert(d()),
        InitializationEventKind::Modify,
        "/Lab/P/D",
        None,
    )
}
fn moving() -> InitializationEventPage {
    let mut moved = d();
    moved.parent = Some("300".into());
    moved.name = "Moved".into();
    let mut page = page(
        Change::Upsert(moved),
        InitializationEventKind::Move,
        "/Lab/P/D",
        Some(("/Lab/Q/Moved", "300")),
    );
    // Current metadata may describe a third parent; it is not source evidence.
    page.hints[0].metadata_parent_id = Some("400".into());
    page
}
fn apply(store: &mut Store, run: &InitializationRun, page: &InitializationEventPage) {
    store
        .record_initialization_event_page(run, 1, page)
        .unwrap();
    assert_eq!(
        store.apply_initialization_events(run, 1).unwrap(),
        InitializationApplyResult::Applied
    );
}

#[test]
fn known_directory_modify_schedules_only_subtree_and_direct_parent_and_clears_fence() {
    let (_temp, mut store, _binding, run) = prepared();
    let before = preserved(&store);
    let fence = store.begin_initialization_fence(&run).unwrap();
    store
        .confirm_initialization_fence(&fence, "native-v1:100")
        .unwrap();
    apply(&mut store, &run, &modify());
    let state = states(&store, &run);
    for id in ["200", "210", "211"] {
        assert_eq!(state[id], "pending");
    }
    for id in ["100", "220", "300", "400"] {
        assert_eq!(state[id], "received");
    }
    assert!(
        store
            .confirm_initialization_fence(&fence, "native-v1:100")
            .is_err()
    );
    let early = store.begin_initialization_fence(&run).unwrap();
    assert!(
        store
            .confirm_initialization_fence(&early, "native-v1:101")
            .is_err()
    );
    assert!(store.complete_initialization(&run).is_err());
    assert_eq!(preserved(&store), before);
    assert!(store.initialization_writeback_blocked().unwrap());
    let status = store.initialization_status().unwrap().unwrap();
    assert_eq!(
        (status.applied_sequence, status.applied_cursor.as_str()),
        (1, "native-v1:101")
    );
}

#[test]
fn local_coverage_survives_reopen_and_does_not_restore_old_tickets() {
    let (temp, mut store, binding, run) = prepared();
    let before = preserved(&store);
    let old = store.begin_initialization_directory(&run, "211").unwrap();
    apply(&mut store, &run, &modify());
    let expected = states(&store, &run);
    drop(store);
    let mut store = Store::open(temp.path()).unwrap();
    let resumed = store.initialization_status().unwrap().unwrap().run;
    assert!(resumed.epoch > run.epoch);
    assert_eq!(states(&store, &resumed), expected);
    assert!(
        store
            .store_initialization_native_directory(&old, 1, &[])
            .is_err()
    );
    store.resume_initialization(&resumed, &binding).unwrap();
    assert_eq!(preserved(&store), before);
    assert!(store.initialization_writeback_blocked().unwrap());
}

#[test]
fn late_parent_and_child_tickets_cannot_publish_after_local_event() {
    let (_temp, mut store, _binding, run) = prepared();
    let parent = store.begin_initialization_directory(&run, "200").unwrap();
    let child_ticket = store.begin_initialization_directory(&run, "210").unwrap();
    apply(&mut store, &run, &modify());
    assert!(
        !store
            .initialization_directory_ticket_current(&parent)
            .unwrap()
    );
    assert!(
        !store
            .initialization_directory_ticket_current(&child_ticket)
            .unwrap()
    );
    assert!(
        store
            .store_initialization_native_directory(&parent, 1, &[entry(d()), entry(sibling())])
            .is_err()
    );
    assert!(
        store
            .store_initialization_native_directory(&child_ticket, 1, &[entry(child())])
            .is_err()
    );
    // SQL orders by IDs, but frontier explicitly promotes a pending ancestor.
    let frontier = store.initialization_pending_directories(&run, 4).unwrap();
    assert_eq!(
        frontier.iter().map(|n| n.id.as_str()).collect::<Vec<_>>(),
        vec!["200"]
    );
}

#[test]
fn child_finishing_before_unchanged_parent_is_retained_only_from_new_generation() {
    let (_temp, mut store, _binding, run) = prepared();
    apply(&mut store, &run, &modify());
    // Deliberately bypass scheduler order: test Store safety independently.
    native_list(&mut store, &run, "210", vec![child()]);
    native_list(&mut store, &run, "211", vec![]);
    native_list(&mut store, &run, "200", vec![d(), sibling()]);
    let state = states(&store, &run);
    for id in ["100", "200", "210", "211", "220", "300", "400"] {
        assert_eq!(state[id], "received");
    }
    let fence = store.begin_initialization_fence(&run).unwrap();
    store
        .confirm_initialization_fence(&fence, "native-v1:101")
        .unwrap();
}

#[test]
fn parent_denial_replacement_or_numeric_change_revokes_parallel_child_receipt() {
    for mode in 0..3 {
        let (_temp, mut store, _binding, run) = prepared();
        let before = preserved(&store);
        apply(&mut store, &run, &modify());
        let parent = store.begin_initialization_directory(&run, "200").unwrap();
        let late = store.begin_initialization_directory(&run, "210").unwrap();
        match mode {
            0 => {
                store.store_initialization_denied(&parent, 1).unwrap();
            }
            1 => {
                let mut replacement = d();
                replacement.id = "999".into();
                let mut token: serde_json::Value =
                    serde_json::from_str(&replacement.version).unwrap();
                token["file"] = "999".into();
                replacement.version = token.to_string();
                store
                    .store_initialization_native_directory(
                        &parent,
                        1,
                        &[entry(replacement), entry(sibling())],
                    )
                    .unwrap();
            }
            _ => {
                let mut changed = entry(d());
                changed.native_node_id += 100;
                store
                    .store_initialization_native_directory(&parent, 1, &[changed, entry(sibling())])
                    .unwrap();
            }
        }
        assert!(
            !store
                .initialization_directory_ticket_current(&late)
                .unwrap()
        );
        assert!(
            store
                .store_initialization_native_directory(&late, 1, &[entry(child())])
                .is_err()
        );
        assert!(store.initialization_node(&run, "211").is_err());
        assert_eq!(states(&store, &run)["400"], "received");
        assert_eq!(preserved(&store), before);
    }
}

#[test]
fn cross_parent_move_is_correct_in_both_parent_commit_orders() {
    for target_first in [false, true] {
        let (_temp, mut store, _binding, run) = prepared();
        let before = preserved(&store);
        let page = moving();
        apply(&mut store, &run, &page);
        let state = states(&store, &run);
        assert_eq!(state["100"], "received");
        assert_eq!(state["400"], "received");
        assert_eq!(state["220"], "received");
        let source = store.begin_initialization_directory(&run, "200").unwrap();
        let target = store.begin_initialization_directory(&run, "300").unwrap();
        let late = store.begin_initialization_directory(&run, "210").unwrap();
        let Change::Upsert(moved) = &page.batch.events[0].change else {
            panic!()
        };
        if target_first {
            store
                .store_initialization_native_directory(&target, 1, &[entry(moved.clone())])
                .unwrap();
            assert!(
                !store
                    .initialization_directory_ticket_current(&source)
                    .unwrap()
            );
            assert!(
                store
                    .store_initialization_native_directory(
                        &source,
                        1,
                        &[entry(d()), entry(sibling())]
                    )
                    .is_err()
            );
            native_list(&mut store, &run, "200", vec![sibling()]);
        } else {
            store
                .store_initialization_native_directory(&source, 1, &[entry(sibling())])
                .unwrap();
            store
                .store_initialization_native_directory(&target, 1, &[entry(moved.clone())])
                .unwrap();
        }
        assert!(
            !store
                .initialization_directory_ticket_current(&late)
                .unwrap()
        );
        assert_eq!(
            store.initialization_directory_path(&run, "210").unwrap(),
            "/Lab/Q/Moved"
        );
        assert_eq!(states(&store, &run)["210"], "pending");
        assert_eq!(states(&store, &run)["400"], "received");
        assert_eq!(preserved(&store), before);
    }
}

#[test]
fn known_delete_is_local_but_unknown_delete_and_bad_or_legacy_evidence_stay_conservative() {
    let (_temp, mut store, _binding, run) = prepared();
    apply(
        &mut store,
        &run,
        &page(
            Change::Delete { id: "210".into() },
            InitializationEventKind::Remove,
            "/Lab/P/D",
            None,
        ),
    );
    assert_eq!(states(&store, &run)["400"], "received");
    native_list(&mut store, &run, "200", vec![sibling()]);
    assert!(store.initialization_node(&run, "210").is_err());
    for mode in 0..5 {
        let (_temp, mut store, _binding, run) = prepared();
        let mut change = modify();
        match mode {
            0 => {
                change = page(
                    Change::Delete {
                        id: "absent".into(),
                    },
                    InitializationEventKind::Remove,
                    "/Lab/P/Absent",
                    None,
                );
            }
            1 => {
                change.hints[0].event_path = "/Lab/Q/D".into();
            }
            2 => {
                store
                    .db
                    .execute("DELETE FROM initialization_native_nodes WHERE id='210'", [])
                    .unwrap();
            }
            3 => {
                store
                    .record_initialization_events(&run, 1, &change.batch)
                    .unwrap();
            }
            _ => {
                change = moving();
                change.hints[0].destination_path = Some("/Lab/Unknown/Moved".into());
            }
        }
        if mode != 3 {
            store
                .record_initialization_event_page(&run, 1, &change)
                .unwrap();
        }
        store.apply_initialization_events(&run, 1).unwrap();
        assert!(
            states(&store, &run).values().all(|s| s == "pending"),
            "mode {mode}"
        );
    }
}

#[test]
fn mixed_root_event_prevents_partial_local_optimization_and_does_not_touch_drafts() {
    let (_temp, mut store, _binding, run) = prepared();
    let before = preserved(&store);
    let mut page = modify();
    page.batch.events.push(Event {
        id: "root-event".into(),
        change: Change::Upsert(root()),
    });
    page.hints.push(InitializationEventHint {
        ordinal: 1,
        event_id: "root-event".into(),
        event_sync_id: 101,
        file_id: "100".into(),
        kind: InitializationEventKind::Modify,
        event_path: "/Lab".into(),
        destination_path: None,
        metadata_parent_id: None,
        destination_parent_id: None,
        metadata_removed: false,
    });
    apply(&mut store, &run, &page);
    assert!(states(&store, &run).values().all(|s| s == "pending"));
    assert!(
        store
            .initialization_status()
            .unwrap()
            .unwrap()
            .root_needs_refresh
    );
    assert_eq!(preserved(&store), before);
}

#[test]
fn failed_later_root_delete_rolls_back_all_planned_local_coverage() {
    let (_temp, mut store, _binding, run) = prepared();
    let before = preserved(&store);
    let original = states(&store, &run);
    let mut page = modify();
    page.batch.events.push(Event {
        id: "removed-root".into(),
        change: Change::Delete { id: "100".into() },
    });
    page.hints.push(InitializationEventHint {
        ordinal: 1,
        event_id: "removed-root".into(),
        event_sync_id: 101,
        file_id: "100".into(),
        kind: InitializationEventKind::Remove,
        event_path: "/Lab".into(),
        destination_path: None,
        metadata_parent_id: None,
        destination_parent_id: None,
        metadata_removed: true,
    });
    store
        .record_initialization_event_page(&run, 1, &page)
        .unwrap();
    assert!(store.apply_initialization_events(&run, 1).is_err());
    assert_eq!(states(&store, &run), original);
    assert_eq!(
        store
            .initialization_status()
            .unwrap()
            .unwrap()
            .applied_sequence,
        0
    );
    assert_eq!(preserved(&store), before);
}

#[test]
fn known_directory_mixed_with_unknown_file_parent_keeps_complete_fallback() {
    let (_temp, mut store, _binding, run) = prepared();
    let mut page = modify();
    let unknown = node("999", Some("absent"), "Unknown", false, 7, "server");
    page.batch.events.push(Event {
        id: "unknown-file".into(),
        change: Change::Upsert(unknown),
    });
    page.hints.push(InitializationEventHint {
        ordinal: 1,
        event_id: "unknown-file".into(),
        event_sync_id: 101,
        file_id: "999".into(),
        kind: InitializationEventKind::Modify,
        event_path: "/Lab/Missing/Unknown".into(),
        destination_path: None,
        metadata_parent_id: Some("absent".into()),
        destination_parent_id: None,
        metadata_removed: false,
    });
    apply(&mut store, &run, &page);
    assert!(
        states(&store, &run)
            .values()
            .all(|state| state == "pending")
    );
}
