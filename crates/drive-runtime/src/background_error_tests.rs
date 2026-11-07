use super::*;
use drive_core::{Batch, Change, Event, NodeMetadata};
use std::os::unix::net::UnixStream;

fn handshake(socket: &mut UnixStream) {
    let (hello, fd) = crate::transport::receive::<Envelope<Command>>(socket).unwrap();
    assert!(fd.is_none());
    assert!(matches!(hello.body, Command::Connect(_)));
    crate::transport::send(
        socket,
        &Envelope {
            version: VERSION,
            id: hello.id,
            body: Response::Connected {
                restore: "fixture".into(),
            },
        },
        None,
    )
    .unwrap();
}

fn connected_bridge() -> (
    tempfile::TempDir,
    Arc<drive_core::SharedStore>,
    Bridge,
    std::thread::JoinHandle<()>,
) {
    let (a, mut b) = UnixStream::pair().unwrap();
    b.set_read_timeout(Some(Duration::from_secs(10))).unwrap();
    let server = std::thread::spawn(move || {
        handshake(&mut b);
        assert!(crate::transport::receive::<Envelope<Command>>(&mut b).is_err());
    });
    let dir = tempfile::tempdir().unwrap();
    let store = Arc::new(drive_core::SharedStore::new(
        Store::open(dir.path()).unwrap(),
    ));
    let bridge = Bridge::connect(
        store.clone(),
        Arc::new(Rpc::new(a).unwrap()),
        Login {
            host: "fixture.invalid".into(),
            tls_name: "fixture.invalid".into(),
            trusted_ca_pem: Vec::new(),
            username: "fixture".into(),
            password: String::new(),
            device_id: "fixture".into(),
            scope: drive_core::native::Scope {
                view_id: 1,
                node_id: 1,
                file_id: "1".into(),
                path: "/lab".into(),
            },
        },
    )
    .unwrap();
    {
        let mut s = store.lock().unwrap();
        s.install_remote_snapshot(
            &[Node {
                id: "1".into(),
                parent: None,
                name: "lab".into(),
                version: "root".into(),
                directory: true,
                metadata: None,
            }],
            "1",
            "fixture",
        )
        .unwrap();
    }
    (dir, store, bridge, server)
}

#[test]
fn successful_cycle_clears_transient_failures_and_keeps_persistent_ones() {
    let (_dir, _store, bridge, server) = connected_bridge();
    bridge.note_failure(BackgroundSource::UploadQueueStopped);
    assert!(bridge.background_failure().is_some());
    bridge.run_upload_cycle();
    assert!(bridge.background_failure().is_none());
    bridge.note_failure(BackgroundSource::PermissionRecheck);
    bridge.run_upload_cycle();
    // A queue pass cannot prove a permission recheck recovered; only the
    // recheck itself (or a reconnect) clears this source.
    assert_eq!(
        bridge.background_failure().unwrap().source,
        BackgroundSource::PermissionRecheck
    );
    bridge.note_failure(BackgroundSource::DownloadPublication);
    bridge.run_upload_cycle();
    let failure = bridge.background_failure().unwrap();
    assert_eq!(failure.source, BackgroundSource::DownloadPublication);
    assert_eq!(
        failure.message,
        "cannot persist download failure; local state needs inspection"
    );
    assert!(failure.since_ms > 0);
    bridge.disconnect();
    server.join().unwrap();
}

#[test]
fn recheck_clears_the_permission_failure_when_reset_succeeds() {
    let (_dir, store, bridge, server) = connected_bridge();
    bridge.note_failure(BackgroundSource::PermissionRecheck);
    {
        let mut s = store.lock().unwrap();
        bridge.handle_recheck_request(&mut s);
    }
    assert!(bridge.background_failure().is_none());
    bridge.disconnect();
    server.join().unwrap();
}

#[test]
fn gated_cycle_neither_clears_nor_sets_a_failure() {
    let (_dir, _store, bridge, server) = connected_bridge();
    bridge.note_failure(BackgroundSource::UploadQueueStopped);
    bridge.session_pending.store(true, Ordering::Release);
    bridge.run_upload_cycle();
    assert_eq!(
        bridge.background_failure().unwrap().source,
        BackgroundSource::UploadQueueStopped
    );
    bridge.session_pending.store(false, Ordering::Release);
    bridge.run_upload_cycle();
    assert!(bridge.background_failure().is_none());
    bridge.disconnect();
    server.join().unwrap();
}

#[test]
fn disconnected_cycle_neither_clears_nor_sets_a_failure() {
    let (_dir, _store, bridge, server) = connected_bridge();
    bridge.note_failure(BackgroundSource::UploadQueueStopped);
    bridge.connection().close();
    bridge.run_upload_cycle();
    assert_eq!(
        bridge.background_failure().unwrap().source,
        BackgroundSource::UploadQueueStopped
    );
    bridge.note_failure(BackgroundSource::DownloadPublication);
    bridge.run_upload_cycle();
    assert_eq!(
        bridge.background_failure().unwrap().source,
        BackgroundSource::DownloadPublication
    );
    bridge.disconnect();
    server.join().unwrap();
}

#[test]
fn namespace_failure_is_cleared_only_when_the_namespace_queue_drains() {
    let (_dir, store, bridge, server) = connected_bridge();
    bridge.note_failure(BackgroundSource::NamespaceUnconfirmed);
    bridge.clear_namespace_failure_if_drained().unwrap();
    assert!(bridge.background_failure().is_none());
    bridge.note_failure(BackgroundSource::NamespaceUnconfirmed);
    {
        let mut s = store.lock().unwrap();
        s.ensure_fs_root("1").unwrap();
        s.receive(&Batch {
            from: "1".into(),
            to: "2".into(),
            events: vec![Event {
                id: "peer".into(),
                change: Change::Upsert(Node {
                    id: "3".into(),
                    parent: Some("1".into()),
                    name: "B.doc".into(),
                    version: "v-3".into(),
                    directory: false,
                    metadata: Some(NodeMetadata {
                        size: 3,
                        mtime: 1,
                        mode: 0o644,
                    }),
                }),
            }],
        })
        .unwrap();
        s.drain().unwrap();
        s.unlink_local("1", "B.doc", "3", false).unwrap();
        assert_eq!(s.pending_namespace_operations().unwrap(), 1);
    }
    bridge.clear_namespace_failure_if_drained().unwrap();
    assert_eq!(
        bridge.background_failure().unwrap().source,
        BackgroundSource::NamespaceUnconfirmed
    );
    bridge.disconnect();
    server.join().unwrap();
}

#[test]
fn repeated_failure_keeps_the_first_timestamp_and_switches_source() {
    let (_dir, _store, bridge, server) = connected_bridge();
    bridge.note_failure(BackgroundSource::UploadQueueStopped);
    let first = bridge.background_failure().unwrap().since_ms;
    std::thread::sleep(Duration::from_millis(5));
    bridge.note_failure(BackgroundSource::UploadQueueStopped);
    let again = bridge.background_failure().unwrap();
    assert_eq!(again.since_ms, first);
    assert_eq!(again.source, BackgroundSource::UploadQueueStopped);
    bridge.note_failure(BackgroundSource::NamespaceUnconfirmed);
    assert_eq!(
        bridge.background_failure().unwrap().source,
        BackgroundSource::NamespaceUnconfirmed
    );
    bridge.disconnect();
    server.join().unwrap();
}

#[test]
fn overview_reports_the_failure_source_and_stays_schema_compatible() {
    let dir = tempfile::tempdir().unwrap();
    let mut store = Store::open(dir.path()).unwrap();
    store
        .install_remote_snapshot(
            &[Node {
                id: "1".into(),
                parent: None,
                name: "lab".into(),
                version: "root".into(),
                directory: true,
                metadata: None,
            }],
            "1",
            "fixture",
        )
        .unwrap();
    store.ensure_fs_root("1").unwrap();
    let service = crate::service::Service::new(
        crate::service::Profile {
            schema: 1,
            task_id: uuid::Uuid::new_v4().to_string(),
            host: "fixture.invalid".into(),
            tls_name: "fixture.invalid".into(),
            trusted_ca_pem: Vec::new(),
            username: "fixture".into(),
            device_id: uuid::Uuid::new_v4().to_string(),
            scope: drive_core::native::Scope {
                view_id: 1,
                node_id: 1,
                file_id: "1".into(),
                path: "/lab".into(),
            },
            mount: std::path::PathBuf::from("/tmp/drive-native-background-test"),
            writable: false,
            local_policy: None,
            writable_source: None,
        },
        store,
        std::path::PathBuf::from("/nonexistent-worker"),
    )
    .unwrap();
    let healthy: serde_json::Value =
        serde_json::from_str(&crate::desktop::overview(&service).unwrap()).unwrap();
    assert!(healthy["background_error"].is_null());
    assert!(healthy["background_error_kind"].is_null());
    assert!(healthy["background_error_since_ms"].is_null());
    assert!(healthy["upload_block"].is_null());
    assert_eq!(healthy["schema"].as_u64().unwrap(), 1);
    assert!(healthy["pending_operations"].is_number());
    service
        .bridge
        .note_failure(BackgroundSource::NamespaceUnconfirmed);
    *service.bridge.upload_block.lock().unwrap() = Some("permission_denied".into());
    let failing: serde_json::Value =
        serde_json::from_str(&crate::desktop::overview(&service).unwrap()).unwrap();
    assert_eq!(
        failing["background_error"],
        "namespace commit result not confirmed; local intent retained"
    );
    assert_eq!(failing["background_error_kind"], "namespace_unconfirmed");
    assert!(failing["background_error_since_ms"].as_i64().unwrap() > 0);
    assert_eq!(failing["upload_block"], "permission_denied");
    service.bridge.clear_all_failures();
    let cleared: serde_json::Value =
        serde_json::from_str(&crate::desktop::overview(&service).unwrap()).unwrap();
    assert!(cleared["background_error"].is_null());
    assert!(cleared["background_error_kind"].is_null());
    assert!(cleared["background_error_since_ms"].is_null());
}
