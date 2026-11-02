use super::*;

fn entries(nodes: Vec<Node>) -> Vec<InitializationDirectoryEntry> {
    nodes
        .into_iter()
        .map(|node| InitializationDirectoryEntry {
            native_node_id: 100_000 + node.id.parse::<u64>().unwrap(),
            node,
        })
        .collect()
}
fn native_list(store: &mut Store, run: &InitializationRun, id: &str, nodes: Vec<Node>) {
    let ticket = store.begin_initialization_directory(run, id).unwrap();
    store
        .store_initialization_native_directory(&ticket, 1, &entries(nodes))
        .unwrap();
}
fn page(count: usize) -> InitializationEventPage {
    let changes = (0..count)
        .map(|i| {
            Change::Upsert(node(
                &(1500 + i).to_string(),
                Some("1100"),
                &format!("Later{i}.txt"),
                false,
                101,
                "server",
            ))
        })
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
                event_path: format!("/Lab/Target/Nested/Last/{}", node.name),
                destination_path: None,
                metadata_parent_id: node.parent.clone(),
                destination_parent_id: None,
                metadata_removed: false,
            }
        })
        .collect();
    InitializationEventPage {
        batch,
        scope: serde_json::to_value(scope()).unwrap(),
        restore: "server".into(),
        hints,
    }
}
fn fixture() -> (tempfile::TempDir, Store, InitializationRun) {
    let (temp, mut store, binding) = setup();
    store.save_local("101", b"offline").unwrap();
    let run = start(&mut store, &binding);
    let mut children = (200..220)
        .map(|id| {
            node(
                &id.to_string(),
                Some("100"),
                &format!("Normal{id}"),
                true,
                7,
                "server",
            )
        })
        .collect::<Vec<_>>();
    children.push(node("900", Some("100"), "Target", true, 7, "server"));
    native_list(&mut store, &run, "100", children);
    (temp, store, run)
}
fn prepared(store: &mut Store, run: &InitializationRun) -> InitializationPriorityCache {
    store
        .record_initialization_event_page(run, 1, &page(1))
        .unwrap();
    assert_eq!(
        store.apply_initialization_events(run, 1).unwrap(),
        InitializationApplyResult::AwaitingCoverage
    );
    let mut cache = InitializationPriorityCache::default();
    assert_eq!(
        store
            .prepare_initialization_priority(run, &mut cache)
            .unwrap(),
        InitializationPriorityPreparation::Prepared
    );
    cache
}

#[test]
fn priority_claim_walks_current_known_ancestors_and_reuses_one_validated_head() {
    let (_temp, mut store, run) = fixture();
    let before = store.pending_operations().unwrap();
    let mut cache = prepared(&mut store, &run);
    let claim = store
        .claim_initialization_priority(&run, &cache)
        .unwrap()
        .unwrap();
    assert_eq!(
        (claim.node.id.as_str(), claim.path.as_str()),
        ("900", "/Lab/Target")
    );
    assert!(
        store
            .initialization_directory_ticket_current(&claim.ticket)
            .unwrap()
    );
    let ordinary = store
        .claim_initialization_directories(&run, 3, None)
        .unwrap();
    assert_eq!(ordinary.jobs.len(), 3);
    assert!(ordinary.jobs.iter().all(|job| job.node.id != "900"));
    store
        .store_initialization_native_directory(
            &claim.ticket,
            1,
            &entries(vec![node("1000", Some("900"), "Nested", true, 7, "server")]),
        )
        .unwrap();
    assert_eq!(
        store
            .prepare_initialization_priority(&run, &mut cache)
            .unwrap(),
        InitializationPriorityPreparation::Reused
    );
    let next = store
        .claim_initialization_priority(&run, &cache)
        .unwrap()
        .unwrap();
    assert_eq!(
        (next.node.id.as_str(), next.path.as_str()),
        ("1000", "/Lab/Target/Nested")
    );
    store
        .store_initialization_native_directory(
            &next.ticket,
            1,
            &entries(vec![node("1100", Some("1000"), "Last", true, 7, "server")]),
        )
        .unwrap();
    assert!(
        store
            .claim_initialization_priority(&run, &cache)
            .unwrap()
            .is_none(),
        "known parent must return to event apply"
    );
    let status = store.initialization_status().unwrap().unwrap();
    assert_eq!((status.received_sequence, status.applied_sequence), (1, 0));
    assert_eq!(store.cursor().unwrap(), "native-v1:7");
    assert_eq!(store.pending_operations().unwrap(), before);
    assert!(store.test_fence(&run, "native-v1:101").is_err());
    assert_eq!(
        store.apply_initialization_events(&run, 1).unwrap(),
        InitializationApplyResult::Applied
    );
    assert!(
        store
            .claim_initialization_priority(&run, &cache)
            .unwrap()
            .is_none()
    );
}

#[test]
fn stale_priority_cannot_cross_reopen_epoch() {
    let (temp, mut store, run) = fixture();
    let cache = prepared(&mut store, &run);
    drop(store);
    let mut store = Store::open(temp.path()).unwrap();
    let current = store.initialization_status().unwrap().unwrap().run;
    assert!(
        store
            .claim_initialization_priority(&current, &cache)
            .unwrap()
            .is_none()
    );
    assert!(store.claim_initialization_priority(&run, &cache).is_err());
    let mut refreshed = cache;
    assert_eq!(
        store
            .prepare_initialization_priority(&current, &mut refreshed)
            .unwrap(),
        InitializationPriorityPreparation::Prepared
    );
    assert!(
        store
            .claim_initialization_priority(&current, &refreshed)
            .unwrap()
            .is_some()
    );
}

#[test]
fn old_plan_cannot_claim_after_applied_head_advances_within_same_epoch() {
    let (_temp, mut store, run) = fixture();
    let mut cache = prepared(&mut store, &run);
    native_list(
        &mut store,
        &run,
        "900",
        vec![node("1000", Some("900"), "Nested", true, 7, "server")],
    );
    native_list(
        &mut store,
        &run,
        "1000",
        vec![node("1100", Some("1000"), "Last", true, 7, "server")],
    );
    let mut second = page(1);
    second.batch.from = "native-v1:101".into();
    second.batch.to = "native-v1:102".into();
    second.batch.events[0].id = "event-102-0".into();
    let Change::Upsert(node) = &mut second.batch.events[0].change else {
        unreachable!()
    };
    node.parent = Some("2100".into());
    second.hints[0].event_id = "event-102-0".into();
    second.hints[0].event_sync_id = 102;
    second.hints[0].metadata_parent_id = Some("2100".into());
    second.hints[0].event_path = "/Lab/Normal200/Missing/Later0.txt".into();
    store
        .record_initialization_event_page(&run, 2, &second)
        .unwrap();
    assert_eq!(
        store.apply_initialization_events(&run, 1).unwrap(),
        InitializationApplyResult::Applied
    );
    let before = store.initialization_coverage(&run).unwrap();
    assert!(
        store
            .claim_initialization_priority(&run, &cache)
            .unwrap()
            .is_none()
    );
    assert_eq!(store.initialization_coverage(&run).unwrap(), before);
    assert_eq!(
        store
            .prepare_initialization_priority(&run, &mut cache)
            .unwrap(),
        InitializationPriorityPreparation::Prepared
    );
    assert_eq!(
        store
            .claim_initialization_priority(&run, &cache)
            .unwrap()
            .unwrap()
            .node
            .id,
        "200"
    );
}

#[test]
fn priority_claim_failure_after_loading_rolls_back_clock_ticket_and_fence() {
    let (_temp, mut store, run) = fixture();
    let cache = prepared(&mut store, &run);
    store.begin_initialization_fence(&run).unwrap();
    let before = store.initialization_coverage(&run).unwrap();
    let read = |store: &Store| {
        store
            .db
            .query_row(
                "SELECT directory_clock,fence_ticket FROM initialization_run",
                [],
                |r| Ok((r.get::<_, i64>(0)?, r.get::<_, Option<i64>>(1)?)),
            )
            .unwrap()
    };
    let clocks = read(&store);
    store.db.execute_batch("CREATE TEMP TRIGGER reject_priority_fence BEFORE UPDATE OF fenced ON initialization_run BEGIN SELECT RAISE(ABORT,'synthetic fence storage failure'); END;").unwrap();
    assert!(store.claim_initialization_priority(&run, &cache).is_err());
    assert_eq!(store.initialization_coverage(&run).unwrap(), before);
    assert_eq!(read(&store), clocks);
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
        .execute_batch("DROP TRIGGER reject_priority_fence")
        .unwrap();
    assert!(
        store
            .claim_initialization_priority(&run, &cache)
            .unwrap()
            .is_some()
    );
}

#[test]
fn denied_or_loading_or_unproved_ancestry_never_gets_priority() {
    for kind in ["denied", "loading", "missing_native", "root_revoked"] {
        let (_temp, mut store, run) = fixture();
        let cache = prepared(&mut store, &run);
        match kind {
            "denied" => {
                let t = store.begin_initialization_directory(&run, "900").unwrap();
                store.store_initialization_denied(&t, 1).unwrap();
                invalidate(&store.db, &run.run_id, "900", false).unwrap();
            }
            "loading" => {
                store.begin_initialization_directory(&run, "900").unwrap();
            }
            "missing_native" => {
                store
                    .db
                    .execute(
                        "DELETE FROM initialization_native_nodes WHERE run=?1 AND id='900'",
                        [&run.run_id],
                    )
                    .unwrap();
            }
            "root_revoked" => {
                store.revoke_initialization_preview_root(&run).unwrap();
            }
            _ => unreachable!(),
        }
        let before = store.initialization_coverage(&run).unwrap();
        assert!(
            store
                .claim_initialization_priority(&run, &cache)
                .unwrap()
                .is_none(),
            "{kind}"
        );
        assert_eq!(store.initialization_coverage(&run).unwrap(), before);
    }
}

#[test]
fn replaced_path_is_only_a_hint_to_current_identity_not_an_old_ticket() {
    let (_temp, mut store, run) = fixture();
    let cache = prepared(&mut store, &run);
    let old = store
        .claim_initialization_priority(&run, &cache)
        .unwrap()
        .unwrap();
    native_list(
        &mut store,
        &run,
        "100",
        vec![node("901", Some("100"), "Target", true, 8, "server")],
    );
    assert!(
        !store
            .initialization_directory_ticket_current(&old.ticket)
            .unwrap()
    );
    let current = store
        .claim_initialization_priority(&run, &cache)
        .unwrap()
        .unwrap();
    assert_eq!(current.node.id, "901");
    assert_ne!(current.ticket.generation, old.ticket.generation);
    assert!(
        store
            .store_initialization_directory(&old.ticket, 1, &[])
            .is_err()
    );
    assert_eq!(
        store
            .initialization_status()
            .unwrap()
            .unwrap()
            .applied_sequence,
        0
    );
    assert!(
        store.initialization_node(&run, "1100").is_err(),
        "hint cannot invent missing parent"
    );
}

#[test]
fn complete_path_gap_does_not_invalidate_or_invent_dependencies() {
    let (_temp, mut store, run) = fixture();
    let cache = prepared(&mut store, &run);
    native_list(&mut store, &run, "900", vec![]);
    let before = store.initialization_coverage(&run).unwrap();
    assert!(
        store
            .claim_initialization_priority(&run, &cache)
            .unwrap()
            .is_none()
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
}

#[test]
fn legacy_oversized_and_mixed_dangerous_pages_have_no_priority() {
    for kind in ["legacy", "oversized", "mixed_delete"] {
        let (_temp, mut store, run) = fixture();
        let mut p = page(if kind == "oversized" { 1000 } else { 1 });
        if kind == "mixed_delete" {
            let ordinal = p.batch.events.len();
            p.batch.events.push(Event {
                id: "unknown-delete".into(),
                change: Change::Delete { id: "9999".into() },
            });
            p.hints.push(InitializationEventHint {
                ordinal: ordinal as u64,
                event_id: "unknown-delete".into(),
                event_sync_id: 101,
                file_id: "9999".into(),
                kind: InitializationEventKind::Remove,
                event_path: "/Lab/Removed".into(),
                destination_path: None,
                metadata_parent_id: None,
                destination_parent_id: None,
                metadata_removed: true,
            });
        }
        if kind == "legacy" {
            store
                .record_initialization_events(&run, 1, &p.batch)
                .unwrap();
        } else {
            store.record_initialization_event_page(&run, 1, &p).unwrap();
        }
        let before = store.initialization_coverage(&run).unwrap();
        let mut cache = InitializationPriorityCache::default();
        for _ in 0..2 {
            assert_eq!(
                store
                    .prepare_initialization_priority(&run, &mut cache)
                    .unwrap(),
                InitializationPriorityPreparation::Unavailable,
                "{kind}"
            );
            assert!(
                store
                    .claim_initialization_priority(&run, &cache)
                    .unwrap()
                    .is_none()
            );
        }
        assert_eq!(store.initialization_coverage(&run).unwrap(), before);
    }
}

#[test]
fn corrupt_receipt_or_native_proof_is_an_error_not_a_skipped_priority() {
    for table in [
        "initialization_events",
        "initialization_event_hints",
        "initialization_native_nodes",
    ] {
        let (_temp, mut store, run) = fixture();
        let mut cache = prepared(&mut store, &run);
        store
            .db
            .execute(&format!("UPDATE {table} SET digest='corrupt'"), [])
            .unwrap();
        let before = store.initialization_coverage(&run).unwrap();
        if table == "initialization_native_nodes" {
            assert!(store.claim_initialization_priority(&run, &cache).is_err());
        } else {
            assert!(
                store
                    .prepare_initialization_priority(&run, &mut cache)
                    .is_err()
            );
        }
        assert_eq!(store.initialization_coverage(&run).unwrap(), before);
    }
}

#[test]
fn at_most_eight_dependencies_are_cached_and_others_remain_normally_claimable() {
    let (_temp, mut store, run) = fixture();
    native_list(
        &mut store,
        &run,
        "100",
        (0..9)
            .map(|i| {
                node(
                    &(901 + i).to_string(),
                    Some("100"),
                    &format!("Target{i}"),
                    true,
                    7,
                    "server",
                )
            })
            .collect(),
    );
    let mut p = page(9);
    for (i, event) in p.batch.events.iter_mut().enumerate() {
        let Change::Upsert(node) = &mut event.change else {
            unreachable!()
        };
        node.parent = Some((1100 + i).to_string());
        p.hints[i].metadata_parent_id = node.parent.clone();
        p.hints[i].event_path = format!("/Lab/Target{i}/Missing/{}", node.name);
    }
    store.record_initialization_event_page(&run, 1, &p).unwrap();
    let mut cache = InitializationPriorityCache::default();
    assert_eq!(
        store
            .prepare_initialization_priority(&run, &mut cache)
            .unwrap(),
        InitializationPriorityPreparation::Prepared
    );
    for _ in 0..8 {
        assert!(
            store
                .claim_initialization_priority(&run, &cache)
                .unwrap()
                .is_some()
        );
    }
    assert!(
        store
            .claim_initialization_priority(&run, &cache)
            .unwrap()
            .is_none()
    );
    let normal = store
        .claim_initialization_directories(&run, 1, None)
        .unwrap();
    assert_eq!(normal.jobs.len(), 1);
    assert_eq!(normal.jobs[0].node.id, "909");
    assert_eq!(
        store
            .initialization_status()
            .unwrap()
            .unwrap()
            .applied_sequence,
        0
    );
}
