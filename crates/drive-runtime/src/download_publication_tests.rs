use super::*;
use std::{io::Read, sync::mpsc};

#[test]
fn native_read_refusal_learns_only_the_requested_identity_and_recheck_releases_retry() {
    use drive_core::write_permissions::{self as permissions, Action, Decision};
    for (code, invalidate_inflight) in [(0x3002u32, false), (0x3008, false),
        (0x3004, false), (0x8004, false), (0x9004, false), (0xffff, false), (0x3002, true)] {
        let dir = tempfile::tempdir().unwrap();
        let mut store = Store::open(dir.path()).unwrap();
        let nodes: Vec<Node> = [("root", None, true), ("file", Some("root"), false),
            ("sibling", Some("root"), false)].into_iter().map(|(id, parent, directory)| Node {
                id: id.into(), parent: parent.map(str::to_owned), name: id.into(),
                version: "v1".into(), directory, metadata: None,
            }).collect();
        store.install_remote_snapshot(&nodes, "1", "server").unwrap();
        store.ensure_fs_root("root").unwrap();
        let (socket, mut peer) = UnixStream::pair().unwrap();
        peer.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
        let bridge = Bridge::from_connection(Arc::new(drive_core::SharedStore::new(store)),
            Arc::new(Rpc::new(socket).unwrap())).unwrap();
        let response_store = bridge.store.clone();
        let server = std::thread::spawn(move || {
            let (request, _) = crate::transport::receive::<Envelope<Command>>(&mut peer).unwrap();
            let Command::Download { node } = request.body else { panic!("expected download") };
            if invalidate_inflight {
                permissions::invalidate_remote_permissions(&response_store.lock().unwrap(), "root").unwrap();
            }
            // Receive-boundary classification -> production worker mapper ->
            // IPC. The protocol's raw reply mapping has a separate test.
            let error = drive_core::Error::Native(if matches!(code, 0x3002 | 0x3008) {
                sproto::Error::DownloadPermissionDenied { code }
            } else { sproto::Error::Server {
                code, reason: "permission denied; synthetic secret must not leave worker".into(),
            }});
            let body = match crate::worker::download_error_response(&node.id, error) {
                Ok((body, fd)) => { assert!(fd.is_none()); body },
                Err(_) => Response::Failed { category: format!("native_server_code:{code}") },
            };
            crate::transport::send(&mut peer, &Envelope {
                version: VERSION, id: request.id, body,
            }, None).unwrap();
        });
        let flight = Flight {
            demand: Mutex::new(Demand { keep: true, ..Demand::default() }),
            done: Mutex::new(None), changed: Condvar::new(),
        };
        let refused = matches!(code, 0x3002 | 0x3008);
        assert_eq!(bridge.transfer(&nodes[1], &flight), Err(if refused { libc::EACCES } else { libc::EIO }));
        server.join().unwrap();
        let store = bridge.store.lock().unwrap();
        assert_eq!(permissions::decide(&store, "root", "file", Action::Read).unwrap(),
            if refused && !invalidate_inflight { Decision::Deny } else { Decision::Unknown });
        for (node, action) in [("root", Action::Read), ("sibling", Action::Read), ("file", Action::WriteContent)] {
            assert_eq!(permissions::decide(&store, "root", node, action).unwrap(), Decision::Unknown);
        }
        assert!(store.open_cached("file").is_err());
        assert_eq!(store.download_failure("file").unwrap().is_some(), !invalidate_inflight);
        if invalidate_inflight {
            assert!(store.download_ready(&DownloadTicket { id: "file".into(), version: "v1".into() }, unix_seconds()).unwrap());
        }
        permissions::invalidate_remote_permissions(&store, "root").unwrap();
        assert_eq!(permissions::decide(&store, "root", "file", Action::Read).unwrap(), Decision::Unknown);
        assert_eq!(store.download_failure("file").unwrap().is_none(), refused);
    }
}

#[test]
fn local_io_and_transport_errors_are_not_native_permission_evidence() {
    for error in [
        drive_core::Error::Io(io::Error::from_raw_os_error(libc::EACCES)),
        drive_core::Error::Native(sproto::Error::Timeout("synthetic download")),
        drive_core::Error::Native(sproto::Error::SessionInvalid),
        drive_core::Error::Native(sproto::Error::Server { code: 0x3002, reason: "preflight is not a download reply".into() }),
    ] {
        assert!(crate::worker::download_error_response("file", error).is_err());
    }
}

#[test]
fn publication_classification_preserves_storage_errors() {
    assert_eq!(
        download_publication_error(drive_core::Error::Io(io::Error::from_raw_os_error(
            libc::ENOSPC
        )))
        .1,
        DownloadFailureKind::LocalStorage
    );
    assert_eq!(
        download_publication_error(drive_core::Error::State("obsolete identity".into())).0,
        libc::ESTALE
    );
    let directory = tempfile::tempdir().unwrap();
    std::fs::write(
        directory.path().join("state.sqlite"),
        b"invalid sqlite fixture",
    )
    .unwrap();
    let error = match Store::open(directory.path()) {
        Ok(_) => panic!("fixture accepted"),
        Err(error) => error,
    };
    assert!(matches!(error, drive_core::Error::Db(_)));
    assert_eq!(
        download_publication_error(error).1,
        DownloadFailureKind::LocalStorage
    );
}

struct Blocked {
    reached: mpsc::Sender<()>,
    resume: mpsc::Receiver<()>,
    done: bool,
}
impl Read for Blocked {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        if self.done {
            return Ok(0);
        }
        self.reached.send(()).unwrap();
        self.resume.recv_timeout(Duration::from_secs(10)).unwrap();
        bytes[..3].copy_from_slice(b"new");
        self.done = true;
        Ok(3)
    }
}

#[test]
fn publication_preparation_releases_store_and_rejects_replaced_connection_or_abandoned_demand() {
    for replace_connection in [false, true] {
        let directory = tempfile::tempdir().unwrap();
        let mut store = Store::open(directory.path()).unwrap();
        let root = Node {
            id: "root".into(),
            parent: None,
            name: "root".into(),
            version: "v1".into(),
            directory: true,
            metadata: None,
        };
        let node = Node {
            id: "file".into(),
            parent: Some("root".into()),
            name: "file".into(),
            version: "v1".into(),
            directory: false,
            metadata: None,
        };
        store
            .install_remote_snapshot(&[root, node.clone()], "1", "server")
            .unwrap();
        store.ensure_fs_root("root").unwrap();
        if replace_connection {
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
        let (socket, _peer) = UnixStream::pair().unwrap();
        let rpc = Arc::new(Rpc::new(socket).unwrap());
        let bridge = Arc::new(
            Bridge::from_connection(Arc::new(drive_core::SharedStore::new(store)), rpc.clone())
                .unwrap(),
        );
        let flight = Arc::new(Flight {
            demand: Mutex::new(Demand {
                keep: true,
                ..Demand::default()
            }),
            done: Mutex::new(None),
            changed: Condvar::new(),
        });
        let (reached, wait) = mpsc::channel();
        let (resume, gate) = mpsc::channel();
        let worker_bridge = bridge.clone();
        let worker_flight = flight.clone();
        let work = std::thread::spawn(move || {
            worker_bridge.publish_content(
                &node,
                &worker_flight,
                &rpc,
                &mut Blocked {
                    reached,
                    resume: gate,
                    done: false,
                },
                3,
                &drive_core::sha256(b"new"),
            )
        });
        wait.recv_timeout(Duration::from_secs(10)).unwrap();
        {
            let mut s = bridge
                .store
                .try_lock()
                .expect("real Bridge publication held Store while copying");
            assert_eq!(s.node("file").unwrap().name, "file");
            assert_eq!(s.collect_garbage().unwrap(), 0);
        }
        let (next, next_peer) = UnixStream::pair().unwrap();
        if replace_connection {
            *bridge.connection.lock().unwrap() = Arc::new(Rpc::new(next).unwrap());
        } else {
            flight.demand.lock().unwrap().keep = false;
        }
        resume.send(()).unwrap();
        assert_eq!(work.join().unwrap().unwrap_err().0, libc::EINTR);
        let s = bridge.store.lock().unwrap();
        if replace_connection {
            let mut bytes = Vec::new();
            s.open_cached("file")
                .unwrap()
                .read_to_end(&mut bytes)
                .unwrap();
            assert_eq!(bytes, b"old");
        } else {
            assert!(s.open_cached("file").is_err());
        }
        drop(next_peer);
    }
}
