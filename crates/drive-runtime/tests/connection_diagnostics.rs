use drive_core::{DownloadTicket, Node, NodeMetadata, Store};
use drive_runtime::{
    desktop,
    service::{ConnectionState, Profile, Service},
};
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    sync::Arc,
    time::{Duration, Instant},
};

struct Running {
    service: Arc<Service>,
    tasks: Option<drive_runtime::service::Tasks>,
}
impl Running {
    fn start(service: &Arc<Service>) -> Self {
        Self {
            service: service.clone(),
            tasks: Some(service.start()),
        }
    }
}
impl Drop for Running {
    fn drop(&mut self) {
        self.service.shutdown();
        self.tasks.take().unwrap().join();
    }
}

fn fixture(mode: &str) -> (tempfile::TempDir, Arc<Service>) {
    let root = tempfile::tempdir().unwrap();
    let peer = root.path().join("peer");
    fs::create_dir(&peer).unwrap();
    let worker = peer.join("worker");
    fs::write(&worker, include_str!("fixtures/diagnostic_worker.py")).unwrap();
    fs::set_permissions(&worker, fs::Permissions::from_mode(0o700)).unwrap();
    fs::write(peer.join("mode"), mode).unwrap();
    let state = root.path().join("state");
    let mut store = Store::open(&state).unwrap();
    let root_node = Node {
        id: "root".into(),
        parent: None,
        name: "root".into(),
        version: "1".into(),
        directory: true,
        metadata: None,
    };
    let file = Node {
        id: "cached".into(),
        parent: Some("root".into()),
        name: "A.doc".into(),
        version: "1".into(),
        directory: false,
        metadata: Some(NodeMetadata {
            size: 3,
            mtime: 1,
            mode: 0o644,
        }),
    };
    store
        .install_remote_snapshot(&[root_node, file], "1", "server")
        .unwrap();
    store.ensure_fs_root("root").unwrap();
    store
        .publish_download(
            &DownloadTicket {
                id: "cached".into(),
                version: "1".into(),
            },
            b"old",
            &drive_core::sha256(b"old"),
        )
        .unwrap();
    let profile = Profile {
        schema: 1,
        task_id: uuid::Uuid::new_v4().to_string(),
        host: "example.invalid".into(),
        tls_name: "example.invalid".into(),
        trusted_ca_pem: vec![],
        username: "synthetic".into(),
        device_id: uuid::Uuid::new_v4().to_string(),
        scope: drive_core::native::Scope {
            view_id: 1,
            node_id: 1,
            file_id: "root".into(),
            path: "/synthetic".into(),
        },
        mount: root.path().join("mount"),
        writable: false,
    };
    (root, Service::new(profile, store, worker).unwrap())
}

fn wait_for(mut check: impl FnMut() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(8);
    while !check() {
        assert!(
            Instant::now() < deadline,
            "synthetic supervisor did not reach expected boundary"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[test]
fn actual_supervisor_persists_stage_and_pre_cleanup_state_without_losing_cache() {
    for (mode, stage, cause, recovery) in [
        (
            "auth",
            "authenticate",
            "authentication_or_protocol",
            "needs_attention",
        ),
        (
            "pull",
            "initial_index",
            "native_response_capacity",
            "needs_attention",
        ),
        (
            "idle",
            "wait_notification",
            "native_receive_idle",
            "reconnect",
        ),
        (
            "disconnect",
            "wait_notification",
            "worker_disconnected",
            "reconnect",
        ),
    ] {
        let (root, service) = fixture(mode);
        let running = Running::start(&service);
        service.unlock("synthetic-private-input".into()).unwrap();
        let file = root.path().join("state/connection-diagnostics.json");
        wait_for(|| file.exists());
        // Stop retries before examining the immutable saved first transition.
        service.pause().unwrap();
        let raw = fs::read_to_string(&file).unwrap();
        assert!(!raw.contains("synthetic-private-input"));
        let saved: serde_json::Value = serde_json::from_str(&raw).unwrap();
        let failure = &saved["last_failure"];
        assert_eq!(failure["stage"], stage, "{mode}");
        assert_eq!(failure["cause"]["kind"], cause, "{mode}");
        assert_eq!(failure["recovery"], recovery, "{mode}");
        assert_eq!(failure["before_cleanup"]["state"], "running", "{mode}");
        let pid = failure["worker_pid"].as_u64().unwrap();
        wait_for(|| !std::path::Path::new(&format!("/proc/{pid}")).exists());
        let handle = service
            .bridge
            .store
            .lock()
            .unwrap()
            .open_cached("cached")
            .unwrap();
        let mut bytes = [0; 3];
        assert_eq!(handle.read_at(&mut bytes, 0).unwrap(), 3);
        assert_eq!(&bytes, b"old");
        let overview: serde_json::Value =
            serde_json::from_str(&desktop::overview(&service).unwrap()).unwrap();
        assert_eq!(
            overview["connection_diagnostics"]["last_failure"]["cause"]["kind"],
            cause
        );
        drop(running);
    }
}

#[test]
fn pause_interrupts_wait_without_inventing_a_failure_or_resetting_the_cursor() {
    let (root, service) = fixture("hold");
    let running = Running::start(&service);
    service.unlock("synthetic-private-input".into()).unwrap();
    let commands = root.path().join("peer/commands.jsonl");
    wait_for(|| fs::read_to_string(&commands).is_ok_and(|s| s.contains("Wait")));
    assert_eq!(
        service.control.lock().unwrap().state,
        ConnectionState::Online
    );
    service.pause().unwrap();
    let file = root.path().join("state/connection-diagnostics.json");
    wait_for(|| file.exists());
    let snapshot: serde_json::Value = serde_json::from_slice(&fs::read(&file).unwrap()).unwrap();
    assert_eq!(snapshot["last_transition"]["stage"], "wait_notification");
    assert_eq!(
        snapshot["last_transition"]["recovery"],
        "requested_interruption"
    );
    assert!(snapshot["last_failure"].is_null());
    assert_eq!(service.bridge.store.lock().unwrap().cursor().unwrap(), "1");
    assert!(
        !fs::read_to_string(commands)
            .unwrap()
            .contains("RecoverySnapshot")
    );
    drop(running);
}
