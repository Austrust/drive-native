use super::*;
use crate::transport;
use std::io::{Seek, Write};
fn snapshot() -> drive_core::native::RemoteSnapshot {
    let scope = drive_core::native::Scope {
        view_id: 1,
        node_id: 1,
        file_id: "100".into(),
        path: "/Lab".into(),
    };
    let version=serde_json::json!({"schema":1,"restore":"server","view":1,"root":"100","file":"100","sync":7,"max":7,"size":0,"hash":"","directory":true}).to_string();
    drive_core::native::RemoteSnapshot {
        inaccessible: vec![], skipped_symlinks: 0,
        scope,
        restore: "server".into(),
        cursor: "native-v1:7".into(),
        nodes: vec![Node {
            id: "100".into(),
            parent: None,
            name: "Lab".into(),
            version,
            directory: true,
            metadata: None,
        }],
    }
}
fn fixture() -> (tempfile::TempDir, Arc<Bridge>, UnixStream) {
    let dir = tempfile::tempdir().unwrap();
    let mut store = Store::open(dir.path()).unwrap();
    store
        .bind_native_scope("server", &serde_json::to_string(&snapshot().scope).unwrap())
        .unwrap();
    let (a, b) = UnixStream::pair().unwrap();
    b.set_read_timeout(Some(Duration::from_secs(3))).unwrap();
    let bridge = Arc::new(
        Bridge::from_connection(Arc::new(Mutex::new(store)), Arc::new(Rpc::new(a).unwrap()))
            .unwrap(),
    );
    (dir, bridge, b)
}
fn respond(socket: &mut UnixStream, id: u64, bad_hash: bool) {
    let data = serde_json::to_vec(&snapshot()).unwrap();
    let mut file = tempfile::tempfile().unwrap();
    file.write_all(&data).unwrap();
    file.rewind().unwrap();
    let fd: OwnedFd = file.into();
    transport::send(
        socket,
        &Envelope {
            version: VERSION,
            id,
            body: Response::RecoverySnapshot {
                size: data.len() as u64,
                sha256: if bad_hash {
                    "bad".into()
                } else {
                    drive_core::sha256(&data)
                },
            },
        },
        Some(&fd),
    )
    .unwrap();
}
#[test]
fn snapshot_descriptor_and_cursor_reset_recover_through_rpc() {
    for reset in [false, true] {
        let (_dir, bridge, mut socket) = fixture();
        if reset {
            let mut st = bridge.store.lock().unwrap();
            let snap = snapshot();
            st.install_remote_snapshot(&snap.nodes, "native-v1:999999", &snap.restore)
                .unwrap();
        }
        let (done, rx) = std::sync::mpsc::channel();
        let peer = std::thread::spawn(move || {
            if reset {
                let (req, _) = transport::receive::<Envelope<Command>>(&mut socket).unwrap();
                assert!(matches!(req.body, Command::Pull { .. }));
                transport::send(
                    &mut socket,
                    &Envelope {
                        version: VERSION,
                        id: req.id,
                        body: Response::Failed {
                            category: "native_cursor_reset".into(),
                        },
                    },
                    None,
                )
                .unwrap();
            }
            let (req, fd) = transport::receive::<Envelope<Command>>(&mut socket).unwrap();
            assert!(fd.is_none());
            assert!(matches!(req.body, Command::RecoverySnapshot));
            respond(&mut socket, req.id, false);
            rx.recv().unwrap();
        });
        assert!(bridge.pull().unwrap());
        assert_eq!(
            bridge.store.lock().unwrap().cursor().unwrap(),
            "native-v1:7"
        );
        done.send(()).unwrap();
        peer.join().unwrap();
    }
}
#[test]
fn corrupted_snapshot_descriptor_cannot_publish_index() {
    let (_dir, bridge, mut socket) = fixture();
    let (done, rx) = std::sync::mpsc::channel();
    let peer = std::thread::spawn(move || {
        let (req, _) = transport::receive::<Envelope<Command>>(&mut socket).unwrap();
        respond(&mut socket, req.id, true);
        rx.recv().unwrap();
    });
    assert!(bridge.pull().unwrap_err().to_string().contains("integrity"));
    assert_eq!(bridge.store.lock().unwrap().cursor().unwrap(), "");
    done.send(()).unwrap();
    peer.join().unwrap();
}
#[test]
fn recovery_waits_for_remote_mutation_and_blocks_next_commit_while_local_store_remains_available() {
    let (_dir, bridge, mut socket) = fixture();
    let mutation = bridge.recovery_gate.read().unwrap();
    let running = bridge.clone();
    let pull = std::thread::spawn(move || running.pull());
    socket
        .set_read_timeout(Some(Duration::from_millis(100)))
        .unwrap();
    let mut byte = [0];
    use std::io::Read;
    assert!(matches!(
        socket.read(&mut byte).unwrap_err().kind(),
        io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
    ));
    drop(mutation);
    socket
        .set_read_timeout(Some(Duration::from_secs(3)))
        .unwrap();
    let (req, _) = transport::receive::<Envelope<Command>>(&mut socket).unwrap();
    assert!(matches!(req.body, Command::RecoverySnapshot));
    assert!(bridge.recovery_gate.try_read().is_err());
    assert!(bridge.store.try_lock().is_ok());
    respond(&mut socket, req.id, false);
    assert!(pull.join().unwrap().unwrap());
    assert!(bridge.recovery_gate.try_read().is_ok());
}
