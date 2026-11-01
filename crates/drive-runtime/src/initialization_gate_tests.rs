//! Real persistent gate tests. No GUI state, credentials, NAS or external worker.
use super::*;
use crate::transport;
use drive_core::{
    NodeMetadata,
    native::{RemoteSnapshot, Scope},
};

fn snapshot() -> RemoteSnapshot {
    let scope = Scope {
        view_id: 1,
        node_id: 1,
        file_id: "100".into(),
        path: "/Lab".into(),
    };
    let nodes = [("100", None, "Lab", true), ("101", Some("100"), "A.txt", false),
        ("102", Some("100"), "B.txt", false)].into_iter().map(|(id, parent, name, directory)| Node {
        id: id.into(), parent: parent.map(str::to_owned), name: name.into(), directory,
        version: serde_json::json!({"schema":1,"restore":"server","view":1,"root":"100",
            "file":id,"sync":7,"max":7,"size":if directory {0} else {3},"hash":"","directory":directory}).to_string(),
        metadata: Some(NodeMetadata { size: if directory { 0 } else { 3 }, mtime: 0,
            mode: if directory { 0o755 } else { 0o644 } }),
    }).collect();
    RemoteSnapshot {
        scope,
        restore: "server".into(),
        cursor: "native-v1:7".into(),
        nodes,
        inaccessible: vec![],
        skipped_symlinks: 0,
    }
}

#[derive(Clone, Copy)]
enum PendingState {
    Pending,
    UploadUncertain,
    RenameUncertain,
}

fn prepared_store(path: &std::path::Path, state: PendingState) -> Store {
    let mut store = Store::open(path).unwrap();
    let snapshot = snapshot();
    store
        .bind_native_scope("server", &serde_json::to_string(&snapshot.scope).unwrap())
        .unwrap();
    store.install_native_snapshot(&snapshot, "").unwrap();
    store.ensure_fs_root("100").unwrap();
    for node in snapshot.nodes.iter().filter(|n| !n.directory) {
        store
            .publish_download(
                &DownloadTicket {
                    id: node.id.clone(),
                    version: node.version.clone(),
                },
                b"old",
                &drive_core::sha256(b"old"),
            )
            .unwrap();
    }
    let node = store.node("101").unwrap();
    let source = store.open_cached("101").unwrap();
    let draft = store
        .begin_draft("101", &node.version, Some(&source))
        .unwrap();
    store.draft_write(&draft, 0, b"new", false).unwrap();
    let operation = store.checkpoint_draft(&draft).unwrap().unwrap();
    drop(draft);
    drop(source);
    if matches!(state, PendingState::UploadUncertain) {
        store.mark_upload_started(&operation).unwrap();
        store.mark_upload_uncertain(&operation).unwrap();
    }
    store
        .rename_local("100", "B.txt", "100", "C.txt", false)
        .unwrap();
    if matches!(state, PendingState::RenameUncertain) {
        let operation = store.next_rename().unwrap().unwrap().job.operation;
        store.mark_rename_started(&operation).unwrap();
        store.mark_rename_uncertain(&operation).unwrap();
    }
    store
}

fn preserved_state(store: &Store) -> serde_json::Value {
    let draft = store.open_dirty_draft("101").unwrap().unwrap();
    let mut body = [0; 3];
    assert_eq!(draft.read_at(&mut body, 0).unwrap(), 3);
    assert_eq!(&body, b"new");
    let cached = store.open_cached("102").unwrap();
    let mut old = [0; 3];
    assert_eq!(cached.read_at(&mut old, 0).unwrap(), 3);
    assert_eq!(&old, b"old");
    let rename = store.next_rename().unwrap().unwrap();
    serde_json::json!({
        "cursor":store.cursor().unwrap(), "draft_token":draft.token, "draft_version":draft.version,
        "pending":store.upload_operations("pending").unwrap(),
        "uncertain":store.upload_operations("uncertain").unwrap(),
        "rename_operation":rename.job.operation, "rename_state":rename.state,
        "rename_path":store.lookup_node("100", "C.txt").unwrap().id,
    })
}

/// Reply to every unexpected request so a regression fails assertions rather
/// than waiting for the production 120-second mutation deadline.
fn peer(socket: UnixStream) -> std::thread::JoinHandle<usize> {
    std::thread::spawn(move || {
        let mut socket = socket;
        let mut requests = 0;
        while let Ok((request, _)) = transport::receive::<Envelope<Command>>(&mut socket) {
            requests += 1;
            if transport::send(
                &mut socket,
                &Envelope {
                    version: VERSION,
                    id: request.id,
                    body: Response::Failed {
                        category: "synthetic_unexpected_mutation".into(),
                    },
                },
                None,
            )
            .is_err()
            {
                break;
            }
        }
        requests
    })
}

fn begin(store: &mut Store) -> drive_core::initialization_store::InitializationRun {
    use drive_core::initialization_store::{InitializationBinding, InitializationBudget};
    let snapshot = snapshot();
    store
        .begin_initialization(
            &InitializationBinding {
                task_id: "6dc79fbe-afd0-4115-aefe-e35ea297a3be".into(),
                scope: serde_json::to_value(&snapshot.scope).unwrap(),
                restore: snapshot.restore,
                start_cursor: "native-v1:7".into(),
                expected_cursor: store.cursor().unwrap(),
            },
            &snapshot.nodes[0],
            InitializationBudget::default(),
        )
        .unwrap()
}

#[test]
fn reopened_active_and_failed_candidates_block_pending_and_uncertain_rpc_without_losing_drafts() {
    for failed in [false, true] {
        // Upload and namespace uncertainty are separately reachable states;
        // the real queue deliberately forbids starting a rename while an upload
        // remains uncertain. Do not bypass that guard to build the fixture.
        for state in [
            PendingState::Pending,
            PendingState::UploadUncertain,
            PendingState::RenameUncertain,
        ] {
            let dir = tempfile::tempdir().unwrap();
            let mut store = prepared_store(dir.path(), state);
            let before = preserved_state(&store);
            let run = begin(&mut store);
            if failed {
                store
                    .fail_initialization(&run, "synthetic_interrupted")
                    .unwrap();
            }
            assert!(store.initialization_writeback_blocked().unwrap());
            drop(store); // No in-memory recovery lock or GUI survives this point.
            let store = Store::open(dir.path()).unwrap();
            assert!(store.initialization_writeback_blocked().unwrap());
            let recovered = store.initialization_status().unwrap().unwrap();
            assert_eq!(recovered.run.run_id, run.run_id);
            assert!(recovered.run.epoch > run.epoch);
            assert_eq!(recovered.state, if failed { "failed" } else { "active" });
            let bridge = Bridge::offline(Arc::new(drive_core::SharedStore::new(store))).unwrap();
            let (socket, remote) = UnixStream::pair().unwrap();
            let server = peer(remote);
            let rpc = Arc::new(Rpc::new(socket).unwrap());
            bridge
                .attach(
                    rpc.clone(),
                    "server",
                    &serde_json::to_string(&snapshot().scope).unwrap(),
                )
                .unwrap();
            assert!(Arc::ptr_eq(&rpc, &bridge.connection())); // Reads/recovery may use the connection.
            assert!(!*bridge.upload_signal.0.lock().unwrap());
            bridge.signal_upload(); // A GUI/local-save wake cannot bypass the durable gate.
            let outcomes = [
                bridge.upload_pending(false),
                bridge.upload_pending(true),
                bridge.namespace_pending(),
                bridge.rename_pending(),
            ];
            bridge.disconnect();
            assert_eq!(server.join().unwrap(), 0);
            for result in outcomes {
                assert_eq!(result.unwrap(), 0);
            }
            assert_eq!(preserved_state(&bridge.store.lock().unwrap()), before);
        }
    }
}

#[test]
fn reopened_candidate_resume_uses_current_handle_and_keeps_writeback_closed() {
    let dir = tempfile::tempdir().unwrap();
    let mut store = prepared_store(dir.path(), PendingState::Pending);
    let before = preserved_state(&store);
    let old = begin(&mut store);
    store
        .fail_initialization(&old, "synthetic_interrupted")
        .unwrap();
    drop(store);
    let mut store = Store::open(dir.path()).unwrap();
    let recovered = store.initialization_status().unwrap().unwrap();
    assert!(store.fail_initialization(&old, "stale_callback").is_err());
    assert!(
        store
            .resume_initialization(&old, &recovered.binding)
            .is_err()
    );
    assert_eq!(store.initialization_status().unwrap().unwrap(), recovered);
    // A real owner reauthenticates scope/restore before this call and holds the
    // runtime write gate. No pre-restart handle may authorize the new session.
    let current = store
        .resume_initialization(&recovered.run, &recovered.binding)
        .unwrap();
    assert!(current.epoch > recovered.run.epoch);
    assert!(store.initialization_writeback_blocked().unwrap());
    store
        .fail_initialization(&current, "synthetic_interrupted_again")
        .unwrap();
    assert!(store.initialization_writeback_blocked().unwrap());
    assert_eq!(preserved_state(&store), before);
}

#[test]
fn initialization_started_after_attach_is_rechecked_by_every_dispatch_entry() {
    let dir = tempfile::tempdir().unwrap();
    let store = prepared_store(dir.path(), PendingState::Pending);
    let bridge = Bridge::offline(Arc::new(drive_core::SharedStore::new(store))).unwrap();
    let (socket, remote) = UnixStream::pair().unwrap();
    let server = peer(remote);
    bridge
        .attach(
            Arc::new(Rpc::new(socket).unwrap()),
            "server",
            &serde_json::to_string(&snapshot().scope).unwrap(),
        )
        .unwrap();
    assert!(*bridge.upload_signal.0.lock().unwrap());
    {
        // Runtime begin/resume/finalize owners use this order, not Store -> gate.
        let _owner = bridge.recovery_gate.write().unwrap();
        begin(&mut bridge.store.lock().unwrap());
    }
    let outcomes = [
        bridge.upload_pending(false),
        bridge.upload_pending(true),
        bridge.namespace_pending(),
        bridge.rename_pending(),
    ];
    bridge.disconnect();
    assert_eq!(server.join().unwrap(), 0);
    for result in outcomes {
        assert_eq!(result.unwrap(), 0);
    }
}

#[test]
fn ordinary_complete_store_attach_still_wakes_writeback_without_creating_an_initialization() {
    let dir = tempfile::tempdir().unwrap();
    let store = prepared_store(dir.path(), PendingState::Pending);
    assert!(!store.initialization_writeback_blocked().unwrap());
    let before = preserved_state(&store);
    let bridge = Bridge::offline(Arc::new(drive_core::SharedStore::new(store))).unwrap();
    let (socket, remote) = UnixStream::pair().unwrap();
    let server = peer(remote);
    let rpc = Arc::new(Rpc::new(socket).unwrap());
    bridge
        .attach(
            rpc.clone(),
            "server",
            &serde_json::to_string(&snapshot().scope).unwrap(),
        )
        .unwrap();
    assert!(Arc::ptr_eq(&rpc, &bridge.connection()));
    assert!(*bridge.upload_signal.0.lock().unwrap());
    assert!(
        !bridge
            .store
            .lock()
            .unwrap()
            .initialization_writeback_blocked()
            .unwrap()
    );
    assert_eq!(preserved_state(&bridge.store.lock().unwrap()), before);
    bridge.disconnect();
    assert_eq!(server.join().unwrap(), 0);
}

#[test]
fn ordinary_complete_store_still_dispatches_pending_and_uncertain_operations() {
    for uncertain in [false, true] {
        for namespace in [false, true] {
            let dir = tempfile::tempdir().unwrap();
            let state = match (uncertain, namespace) {
                (false, _) => PendingState::Pending,
                (true, false) => PendingState::UploadUncertain,
                (true, true) => PendingState::RenameUncertain,
            };
            let store = prepared_store(dir.path(), state);
            let (socket, remote) = UnixStream::pair().unwrap();
            let server = peer(remote);
            let bridge = Bridge::from_connection(
                Arc::new(drive_core::SharedStore::new(store)),
                Arc::new(Rpc::new(socket).unwrap()),
            )
            .unwrap();
            let result = if namespace {
                bridge.namespace_pending()
            } else {
                bridge.upload_pending(uncertain)
            };
            bridge.disconnect();
            // The peer rejects the operation intentionally: this is a dispatch
            // assertion, not fabricated evidence of a native server commit.
            assert_eq!(server.join().unwrap(), 1);
            assert_eq!(result.unwrap(), 0);
        }
    }
}
