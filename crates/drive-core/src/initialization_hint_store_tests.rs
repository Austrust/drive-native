use super::*;

fn page(changes: Vec<Change>) -> InitializationEventPage {
    let batch = batch(100, 101, changes);
    let hints = batch
        .events
        .iter()
        .enumerate()
        .map(|(ordinal, event)| {
            let (id, path, parent, removed) = match &event.change {
                Change::Upsert(node) => (
                    node.id.clone(),
                    format!("/Lab/{}", node.name),
                    node.parent.clone(),
                    false,
                ),
                Change::Delete { id } => (id.clone(), "/Lab/removed".into(), None, true),
            };
            InitializationEventHint {
                ordinal: ordinal as u64,
                event_id: event.id.clone(),
                event_sync_id: 101,
                file_id: id,
                kind: if removed {
                    InitializationEventKind::Remove
                } else {
                    InitializationEventKind::Modify
                },
                event_path: path,
                destination_path: None,
                metadata_parent_id: parent,
                destination_parent_id: None,
                metadata_removed: removed,
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
fn ordinary() -> InitializationEventPage {
    page(vec![Change::Upsert(file())])
}
fn count(store: &Store, table: &str) -> i64 {
    store
        .db
        .query_row(&format!("SELECT count(*) FROM {table}"), [], |r| r.get(0))
        .unwrap()
}
fn native_list(
    store: &mut Store,
    run: &InitializationRun,
    id: &str,
    entries: &[InitializationDirectoryEntry],
) {
    let ticket = store.begin_initialization_directory(run, id).unwrap();
    assert!(
        store
            .store_initialization_native_directory(&ticket, 1, entries)
            .unwrap()
    );
}
fn entry(node: Node, native_node_id: u64) -> InitializationDirectoryEntry {
    InitializationDirectoryEntry {
        node,
        native_node_id,
    }
}

#[test]
fn native_receipt_keeps_original_batch_and_compact_hint_evidence_across_reopen() {
    let (temp, mut store, binding) = setup();
    let run = start(&mut store, &binding);
    let page = ordinary();
    let stamp = store.status_change_watermark().unwrap();
    assert!(
        store
            .record_initialization_event_page(&run, 1, &page)
            .unwrap()
    );
    let (body, digest): (String, String) = store
        .db
        .query_row("SELECT body,digest FROM initialization_events", [], |r| {
            Ok((r.get(0)?, r.get(1)?))
        })
        .unwrap();
    assert_eq!(body, serde_json::to_string(&page.batch).unwrap());
    assert_eq!(digest, crate::sha256(body.as_bytes()));
    let hints: String = store
        .db
        .query_row("SELECT body FROM initialization_event_hints", [], |r| {
            r.get(0)
        })
        .unwrap();
    assert!(
        serde_json::from_str::<serde_json::Value>(&hints)
            .unwrap()
            .get("batch")
            .is_none()
    );
    let charged: i64 = store
        .db
        .query_row("SELECT event_bytes FROM initialization_run", [], |r| {
            r.get(0)
        })
        .unwrap();
    assert_eq!(charged, (body.len() + hints.len()) as i64);
    assert_eq!(store.status_change_watermark().unwrap(), stamp);
    assert_eq!(
        store.initialization_event_page(&run, 1).unwrap(),
        Some(page.clone())
    );
    drop(store);
    let store = Store::open(temp.path()).unwrap();
    let resumed = store.initialization_status().unwrap().unwrap().run;
    assert!(resumed.epoch > run.epoch);
    assert!(store.initialization_event_page(&run, 1).is_err());
    assert_eq!(
        store.initialization_event_page(&resumed, 1).unwrap(),
        Some(page)
    );
    assert_eq!(store.cursor().unwrap(), "native-v1:7");
}

#[test]
fn duplicate_native_page_is_immutable_and_legacy_replay_cannot_remove_evidence() {
    let (_temp, mut store, binding) = setup();
    let run = start(&mut store, &binding);
    let page = ordinary();
    store
        .record_initialization_event_page(&run, 1, &page)
        .unwrap();
    let before = store.db.total_changes();
    assert!(
        !store
            .record_initialization_event_page(&run, 1, &page)
            .unwrap()
    );
    assert!(
        !store
            .record_initialization_events(&run, 1, &page.batch)
            .unwrap()
    );
    assert_eq!(store.db.total_changes(), before);
    let mut changed = page.clone();
    changed.hints[0].event_sync_id -= 1;
    assert!(
        store
            .record_initialization_event_page(&run, 1, &changed)
            .is_err()
    );
    assert_eq!(
        store.initialization_event_page(&run, 1).unwrap(),
        Some(page)
    );
}

#[test]
fn legacy_receipt_never_gains_retrospective_hints() {
    let (_temp, mut store, binding) = setup();
    let run = start(&mut store, &binding);
    let page = ordinary();
    store
        .record_initialization_events(&run, 1, &page.batch)
        .unwrap();
    let before = store.db.total_changes();
    let status = store.initialization_status().unwrap().unwrap();
    let bytes: i64 = store
        .db
        .query_row("SELECT event_bytes FROM initialization_run", [], |r| {
            r.get(0)
        })
        .unwrap();
    assert!(
        !store
            .record_initialization_event_page(&run, 1, &page)
            .unwrap()
    );
    assert_eq!(store.db.total_changes(), before);
    assert_eq!(
        store
            .initialization_status()
            .unwrap()
            .unwrap()
            .received_cursor,
        status.received_cursor
    );
    assert_eq!(
        store
            .db
            .query_row("SELECT event_bytes FROM initialization_run", [], |r| r
                .get::<_, i64>(0))
            .unwrap(),
        bytes
    );
    let mut conflicting = page.clone();
    conflicting.batch.to = "native-v1:102".into();
    assert!(
        store
            .record_initialization_event_page(&run, 1, &conflicting)
            .is_err()
    );
    assert_eq!(count(&store, "initialization_event_hints"), 0);
    assert_eq!(store.initialization_event_page(&run, 1).unwrap(), None);
    assert_eq!(
        store.apply_initialization_events(&run, 1).unwrap(),
        InitializationApplyResult::Applied
    );
}

#[test]
fn malformed_scope_restore_ordinals_identity_and_paths_leave_no_receipt() {
    let mut cases = Vec::new();
    let base = ordinary();
    let mut p = base.clone();
    p.restore = "other".into();
    cases.push(p);
    let mut p = base.clone();
    p.scope["node_id"] = 999.into();
    cases.push(p);
    let mut p = base.clone();
    p.hints[0].ordinal = 1;
    cases.push(p);
    let mut p = base.clone();
    p.hints[0].event_id = "other".into();
    cases.push(p);
    let mut p = base.clone();
    p.hints[0].file_id = "999".into();
    cases.push(p);
    for path in ["/Lab2/A.doc", "/Lab/../A.doc", "/Lab//A.doc", "Lab/A.doc"] {
        let mut p = base.clone();
        p.hints[0].event_path = path.into();
        cases.push(p);
    }
    let mut p = base.clone();
    p.hints[0].metadata_parent_id = Some("999".into());
    cases.push(p);
    let mut p = base.clone();
    p.hints.clear();
    cases.push(p);
    let mut p = page(vec![
        Change::Upsert(file()),
        Change::Upsert(node("102", Some("100"), "B.doc", false, 7, "server")),
    ]);
    p.hints.reverse();
    cases.push(p);
    for page in cases {
        let (_temp, mut store, binding) = setup();
        let run = start(&mut store, &binding);
        assert!(
            store
                .record_initialization_event_page(&run, 1, &page)
                .is_err()
        );
        assert_eq!(count(&store, "initialization_events"), 0);
        assert_eq!(count(&store, "initialization_event_hints"), 0);
        assert_eq!(
            store
                .initialization_status()
                .unwrap()
                .unwrap()
                .received_sequence,
            0
        );
    }
}

#[test]
fn move_in_move_out_and_current_tombstone_metadata_preserve_existing_semantics() {
    let mut incoming = ordinary();
    incoming.hints[0].kind = InitializationEventKind::Move;
    incoming.hints[0].event_path = "/Outside/A.doc".into();
    incoming.hints[0].destination_path = Some("/Lab/A.doc".into());
    incoming.hints[0].metadata_parent_id = Some("999".into());
    incoming.hints[0].destination_parent_id = Some("100".into());
    let mut outgoing = page(vec![Change::Delete { id: "101".into() }]);
    outgoing.hints[0].kind = InitializationEventKind::Move;
    outgoing.hints[0].event_path = "/Lab/A.doc".into();
    outgoing.hints[0].destination_path = Some("/Outside/A.doc".into());
    outgoing.hints[0].metadata_parent_id = Some("777".into());
    outgoing.hints[0].destination_parent_id = Some("999".into());
    outgoing.hints[0].metadata_removed = false;
    let mut tombstone = page(vec![Change::Delete { id: "101".into() }]);
    tombstone.hints[0].kind = InitializationEventKind::Modify;
    for page in [incoming, outgoing, tombstone] {
        let (_temp, mut store, binding) = setup();
        let run = start(&mut store, &binding);
        assert!(
            store
                .record_initialization_event_page(&run, 1, &page)
                .unwrap()
        );
        // The event sync ID is bounded by the page cursor. Current tombstone
        // metadata may nevertheless turn a historical Modify into Delete.
        assert_eq!(
            store.initialization_event_page(&run, 1).unwrap(),
            Some(page)
        );
    }
}

#[test]
fn raw_event_upper_bound_rejects_without_writes_but_zero_and_older_events_remain_valid() {
    for sync_id in [0, 1, 100, 101, 102] {
        let (_temp, mut store, binding) = setup();
        let run = start(&mut store, &binding);
        let mut page = ordinary();
        page.hints[0].event_sync_id = sync_id;
        if sync_id <= 101 {
            assert!(
                store
                    .record_initialization_event_page(&run, 1, &page)
                    .unwrap()
            );
        } else {
            let before = store.db.total_changes();
            assert!(
                store
                    .record_initialization_event_page(&run, 1, &page)
                    .is_err()
            );
            assert_eq!(store.db.total_changes(), before);
            assert_eq!(count(&store, "initialization_events"), 0);
            assert_eq!(count(&store, "initialization_event_hints"), 0);
            assert_eq!(
                store
                    .initialization_status()
                    .unwrap()
                    .unwrap()
                    .received_cursor,
                binding.start_cursor
            );
            assert_eq!(
                store
                    .db
                    .query_row("SELECT event_bytes FROM initialization_run", [], |r| r
                        .get::<_, i64>(0))
                    .unwrap(),
                0
            );
        }
    }
}

#[test]
fn current_file_maximum_can_exceed_the_event_page_cursor() {
    let (_temp, mut store, binding) = setup();
    let run = start(&mut store, &binding);
    let mut page = ordinary();
    let Change::Upsert(node) = &mut page.batch.events[0].change else {
        panic!()
    };
    let mut token: serde_json::Value = serde_json::from_str(&node.version).unwrap();
    token["sync"] = 101.into();
    token["max"] = 500.into();
    node.version = serde_json::to_string(&token).unwrap();
    assert!(
        store
            .record_initialization_event_page(&run, 1, &page)
            .unwrap()
    );
    assert_eq!(
        store.initialization_event_page(&run, 1).unwrap(),
        Some(page)
    );
}

#[test]
fn corruption_of_batch_or_sidecar_fails_before_read_and_application() {
    for table in ["initialization_events", "initialization_event_hints"] {
        let (_temp, mut store, binding) = setup();
        let run = start(&mut store, &binding);
        let page = ordinary();
        store
            .record_initialization_event_page(&run, 1, &page)
            .unwrap();
        store
            .db
            .execute(&format!("UPDATE {table} SET body='{{}}'"), [])
            .unwrap();
        assert!(store.initialization_event_page(&run, 1).is_err());
        assert!(store.pending_initialization_events(&run, 1).is_err());
        assert!(store.apply_initialization_events(&run, 1).is_err());
        assert_eq!(
            store
                .initialization_status()
                .unwrap()
                .unwrap()
                .applied_sequence,
            0
        );
        assert_eq!(store.cursor().unwrap(), "native-v1:7");
    }
}

#[test]
fn recomputed_digest_cannot_hide_wrong_run_sequence_or_batch_binding() {
    for field in ["run_id", "sequence", "batch_digest", "restore"] {
        let (_temp, mut store, binding) = setup();
        let run = start(&mut store, &binding);
        let page = ordinary();
        store
            .record_initialization_event_page(&run, 1, &page)
            .unwrap();
        let raw: String = store
            .db
            .query_row("SELECT body FROM initialization_event_hints", [], |r| {
                r.get(0)
            })
            .unwrap();
        let mut value: serde_json::Value = serde_json::from_str(&raw).unwrap();
        value[field] = if field == "sequence" {
            2.into()
        } else {
            "different".into()
        };
        let body = serde_json::to_string(&value).unwrap();
        store
            .db
            .execute(
                "UPDATE initialization_event_hints SET body=?1,digest=?2,bytes=?3",
                params![body, crate::sha256(body.as_bytes()), body.len() as i64],
            )
            .unwrap();
        assert!(store.initialization_event_page(&run, 1).is_err());
        assert!(store.apply_initialization_events(&run, 1).is_err());
    }
}

#[test]
fn page_byte_budget_and_sidecar_insert_failure_roll_back_batch_and_cursor() {
    for inject in [false, true] {
        let (_temp, mut store, binding) = setup();
        let page = ordinary();
        let mut budget = InitializationBudget::default();
        if !inject {
            budget.event_bytes = serde_json::to_vec(&page.batch).unwrap().len() as u64;
        }
        let run = store
            .begin_initialization(&binding, &root(), budget)
            .unwrap();
        if inject {
            store.db.execute_batch("CREATE TEMP TRIGGER fail_hint BEFORE INSERT ON initialization_event_hints BEGIN SELECT RAISE(ABORT,'synthetic'); END;").unwrap();
        }
        assert!(
            store
                .record_initialization_event_page(&run, 1, &page)
                .is_err()
        );
        assert_eq!(count(&store, "initialization_events"), 0);
        assert_eq!(count(&store, "initialization_event_hints"), 0);
        assert_eq!(
            store
                .initialization_status()
                .unwrap()
                .unwrap()
                .received_cursor,
            binding.start_cursor
        );
        let bytes: i64 = store
            .db
            .query_row("SELECT event_bytes FROM initialization_run", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(bytes, 0);
        if inject {
            store.db.execute_batch("DROP TRIGGER fail_hint;").unwrap();
            assert!(
                store
                    .record_initialization_event_page(&run, 1, &page)
                    .unwrap()
            );
        }
    }
}

#[test]
fn event_page_gap_and_stale_epoch_cannot_store_sidecars() {
    let (temp, mut store, binding) = setup();
    let run = start(&mut store, &binding);
    let page = ordinary();
    assert!(
        store
            .record_initialization_event_page(&run, 2, &page)
            .is_err()
    );
    assert_eq!(count(&store, "initialization_event_hints"), 0);
    drop(store);
    let mut store = Store::open(temp.path()).unwrap();
    assert!(
        store
            .record_initialization_event_page(&run, 1, &page)
            .is_err()
    );
    assert_eq!(count(&store, "initialization_events"), 0);
    let current = store.initialization_status().unwrap().unwrap().run;
    assert!(
        store
            .record_initialization_event_page(&current, 1, &page)
            .unwrap()
    );
}

#[test]
fn native_identity_budget_failure_rolls_back_listing_and_can_retry_same_ticket() {
    let (_temp, mut store, binding) = setup();
    let mut budget = InitializationBudget::default();
    // Raw Node rows fit exactly; adding actual native identity evidence does not.
    budget.node_bytes = (serde_json::to_vec(&root()).unwrap().len()
        + serde_json::to_vec(&directory()).unwrap().len()) as u64;
    let run = store
        .begin_initialization(&binding, &root(), budget)
        .unwrap();
    let ticket = store.begin_initialization_directory(&run, "100").unwrap();
    assert!(
        store
            .store_initialization_native_directory(&ticket, 1, &[entry(directory(), 77)])
            .is_err()
    );
    assert_eq!(count(&store, "initialization_native_nodes"), 0);
    assert_eq!(
        store.initialization_status().unwrap().unwrap().node_count,
        1
    );
    assert!(
        store
            .initialization_directory_ticket_current(&ticket)
            .unwrap()
    );
    assert!(
        store
            .store_initialization_native_directory(&ticket, 1, &[])
            .unwrap()
    );
    assert_eq!(store.cursor().unwrap(), "native-v1:7");
}

#[test]
fn native_identity_survives_epoch_but_is_bound_to_exact_candidate_node() {
    let (temp, mut store, binding) = setup();
    let run = start(&mut store, &binding);
    native_list(&mut store, &run, "100", &[entry(directory(), u64::MAX)]);
    drop(store);
    let store = Store::open(temp.path()).unwrap();
    let current = store.initialization_status().unwrap().unwrap().run;
    assert!(store.initialization_native_node_id(&run, "200").is_err());
    assert_eq!(
        store
            .initialization_native_node_id(&current, "200")
            .unwrap(),
        Some(u64::MAX)
    );
    let changed = node("200", Some("100"), "Renamed", true, 8, "server");
    store
        .db
        .execute(
            "UPDATE initialization_entries SET body=?1 WHERE id='200'",
            [serde_json::to_string(&changed).unwrap()],
        )
        .unwrap();
    assert!(
        store
            .initialization_native_node_id(&current, "200")
            .is_err()
    );
}

#[test]
fn native_directory_identity_is_exact_u64_and_changed_identity_invalidates_descendants() {
    let (_temp, mut store, binding) = setup();
    let run = start(&mut store, &binding);
    native_list(&mut store, &run, "100", &[entry(directory(), u64::MAX)]);
    assert_eq!(
        store.initialization_native_node_id(&run, "100").unwrap(),
        Some(2)
    );
    assert_eq!(
        store.initialization_native_node_id(&run, "200").unwrap(),
        Some(u64::MAX)
    );
    let child = node("201", Some("200"), "child", false, 7, "server");
    native_list(&mut store, &run, "200", &[entry(child, 700)]);
    native_list(&mut store, &run, "100", &[entry(directory(), u64::MAX)]);
    assert!(store.initialization_node(&run, "201").is_ok());
    let stale = store.begin_initialization_directory(&run, "200").unwrap();
    native_list(&mut store, &run, "100", &[entry(directory(), 99)]);
    assert!(
        !store
            .initialization_directory_ticket_current(&stale)
            .unwrap()
    );
    assert!(store.initialization_node(&run, "201").is_err());
    assert_eq!(
        store.initialization_native_node_id(&run, "200").unwrap(),
        Some(99)
    );
    assert_eq!(
        store
            .initialization_coverage(&run)
            .unwrap()
            .iter()
            .find(|d| d.directory == "200")
            .unwrap()
            .state,
        "pending"
    );
    let ticket = store.begin_initialization_directory(&run, "100").unwrap();
    assert!(
        store
            .store_initialization_native_directory(&ticket, 1, &[entry(directory(), 0)])
            .is_err()
    );
}

#[test]
fn legacy_and_native_directory_receipts_do_not_retroactively_assert_identity() {
    let (_temp, mut store, binding) = setup();
    let run = start(&mut store, &binding);
    list(&mut store, &run, "100", &[directory()]);
    list(
        &mut store,
        &run,
        "200",
        &[node("201", Some("200"), "child", false, 7, "server")],
    );
    assert_eq!(
        store.initialization_native_node_id(&run, "200").unwrap(),
        None
    );
    native_list(&mut store, &run, "100", &[entry(directory(), 77)]);
    assert!(store.initialization_node(&run, "201").is_err());
    let ticket = store.begin_initialization_directory(&run, "100").unwrap();
    let entries = [entry(directory(), 77)];
    assert!(
        store
            .store_initialization_native_directory(&ticket, 1, &entries)
            .unwrap()
    );
    assert!(
        !store
            .store_initialization_native_directory(&ticket, 1, &entries)
            .unwrap()
    );
    assert!(
        store
            .store_initialization_native_directory(&ticket, 1, &[entry(directory(), 78)])
            .is_err()
    );
    assert!(
        store
            .store_initialization_directory(&ticket, 1, &[directory()])
            .is_err()
    );
    list(&mut store, &run, "100", &[directory()]);
    assert_eq!(
        store.initialization_native_node_id(&run, "200").unwrap(),
        None
    );
}

#[test]
fn native_identity_bytes_and_rows_follow_subtree_cascade_and_mapping_digest() {
    let (_temp, mut store, binding) = setup();
    let run = start(&mut store, &binding);
    native_list(&mut store, &run, "100", &[entry(directory(), 77)]);
    let total: i64 = store
        .db
        .query_row("SELECT sum(bytes) FROM initialization_entries", [], |r| {
            r.get(0)
        })
        .unwrap();
    assert_eq!(
        store.initialization_status().unwrap().unwrap().node_bytes,
        total as u64
    );
    assert!(count(&store, "initialization_native_nodes") > 0);
    store
        .db
        .execute("UPDATE initialization_native_nodes SET digest='bad'", [])
        .unwrap();
    assert!(store.initialization_native_node_id(&run, "200").is_err());
    // Removing the whole child does not need to reuse its corrupt identity.
    list(&mut store, &run, "100", &[]);
    assert_eq!(count(&store, "initialization_native_nodes"), 0);
    assert_eq!(
        store.initialization_status().unwrap().unwrap().node_bytes,
        serde_json::to_vec(&root()).unwrap().len() as u64
    );
}

#[test]
fn actual_schema20_legacy_receipts_migrate_without_hints_and_preserve_dirty_queue() {
    let (temp, mut store, binding) = setup();
    let draft = store.begin_draft("101", &file().version, None).unwrap();
    store.draft_write(&draft, 0, b"dirty", false).unwrap();
    store.checkpoint_draft(&draft).unwrap();
    drop(draft);
    let pending = store.pending_operations().unwrap();
    let run = start(&mut store, &binding);
    let page = ordinary();
    store
        .record_initialization_events(&run, 1, &page.batch)
        .unwrap();
    let receipt: String = store
        .db
        .query_row("SELECT body FROM initialization_events", [], |r| r.get(0))
        .unwrap();
    store.db.execute_batch("DROP TABLE initialization_event_hints; DROP TABLE initialization_native_nodes; PRAGMA user_version=20;").unwrap();
    drop(store);
    let mut store = Store::open(temp.path()).unwrap();
    let resumed = store.initialization_status().unwrap().unwrap().run;
    assert!(resumed.epoch > run.epoch);
    assert!(store.initialization_writeback_blocked().unwrap());
    assert_eq!(store.pending_operations().unwrap(), pending);
    assert!(store.open_dirty_draft("101").unwrap().is_some());
    assert_eq!(store.initialization_event_page(&resumed, 1).unwrap(), None);
    assert!(
        !store
            .record_initialization_event_page(&resumed, 1, &page)
            .unwrap()
    );
    assert_eq!(
        store
            .db
            .query_row("SELECT body FROM initialization_events", [], |r| r
                .get::<_, String>(0))
            .unwrap(),
        receipt
    );
    assert_eq!(
        store
            .db
            .pragma_query_value(None, "user_version", |r| r.get::<_, i64>(0))
            .unwrap(),
        21
    );
    let backup = std::fs::read_dir(temp.path())
        .unwrap()
        .map(|e| e.unwrap().path())
        .find(|p| {
            p.file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("state-before-v21-")
        })
        .unwrap();
    let old = Connection::open(backup).unwrap();
    assert_eq!(
        old.pragma_query_value(None, "user_version", |r| r.get::<_, i64>(0))
            .unwrap(),
        20
    );
    assert_eq!(
        old.query_row(
            "SELECT count(*) FROM sqlite_master WHERE name='initialization_event_hints'",
            [],
            |r| r.get::<_, i64>(0)
        )
        .unwrap(),
        0
    );
    // Executing the old schema20 ELF remains an installation-package gate.
}
