use super::*;
use drive_core::{NodeMetadata, Status, sha256};
use std::os::unix::net::UnixStream;

fn node(version: &str) -> Node {
    Node {
        id: "2".into(),
        parent: Some("1".into()),
        name: "A.doc".into(),
        version: version.into(),
        directory: false,
        metadata: Some(NodeMetadata {
            size: 3,
            mtime: 0,
            mode: 0o644,
        }),
    }
}
fn fixture(socket: UnixStream) -> (tempfile::TempDir, Arc<Mutex<Store>>, Bridge) {
    let dir = tempfile::tempdir().unwrap();
    let store = Arc::new(Mutex::new(Store::open(dir.path()).unwrap()));
    let bridge = Bridge::connect(
        store.clone(),
        Arc::new(Rpc::new(socket).unwrap()),
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
            &[
                Node {
                    id: "1".into(),
                    parent: None,
                    name: "lab".into(),
                    version: "root".into(),
                    directory: true,
                    metadata: None,
                },
                node("v1"),
            ],
            "1",
            "fixture",
        )
        .unwrap();
        s.publish_download(
            &DownloadTicket {
                id: "2".into(),
                version: "v1".into(),
            },
            b"old",
            &sha256(b"old"),
        )
        .unwrap();
        s.save_local("2", b"new").unwrap();
    }
    (dir, store, bridge)
}
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
#[test]
fn uncertain_commit_uses_read_only_reconciliation_and_never_replays_upload() {
    use std::io::Read;
    let (a, mut b) = UnixStream::pair().unwrap();
    let server = std::thread::spawn(move || {
        handshake(&mut b);
        for reconcile in [false, true] {
            let (req, fd) = crate::transport::receive::<Envelope<Command>>(&mut b).unwrap();
            match req.body {
                Command::Upload {
                    path,
                    reconcile: got,
                    sha256: hash,
                    ..
                } => {
                    assert_eq!(got, reconcile);
                    assert_eq!(path, "/lab/A.doc");
                    assert_eq!(hash, sha256(b"new"));
                }
                _ => panic!("expected upload intent"),
            }
            let mut body = Vec::new();
            std::fs::File::from(fd.unwrap())
                .read_to_end(&mut body)
                .unwrap();
            assert_eq!(body, b"new");
            crate::transport::send(
                &mut b,
                &Envelope {
                    version: VERSION,
                    id: req.id,
                    body: if reconcile {
                        Response::Committed { node: node("v2") }
                    } else {
                        Response::Uncertain
                    },
                },
                None,
            )
            .unwrap();
        }
    });
    let (_dir, store, bridge) = fixture(a);
    assert_eq!(bridge.upload_pending(false).unwrap(), 0);
    assert_eq!(
        store
            .lock()
            .unwrap()
            .upload_operations("uncertain")
            .unwrap()
            .len(),
        1
    );
    assert_eq!(bridge.upload_pending(false).unwrap(), 0);
    assert_eq!(bridge.upload_pending(true).unwrap(), 1);
    assert_eq!(
        store.lock().unwrap().status("2").unwrap(),
        Status::LocalComplete
    );
    server.join().unwrap();
}
#[test]
fn stopped_worker_leaves_saved_operation_pending_without_marking_it_submitted() {
    let (a, mut b) = UnixStream::pair().unwrap();
    let server = std::thread::spawn(move || {
        handshake(&mut b);
        b
    });
    let (_dir, store, bridge) = fixture(a);
    let _peer = server.join().unwrap();
    bridge.connection().close();
    assert!(bridge.upload_pending(false).is_err());
    let s = store.lock().unwrap();
    assert_eq!(s.upload_operations("pending").unwrap().len(), 1);
    assert!(s.upload_operations("uncertain").unwrap().is_empty());
    assert_eq!(s.status("2").unwrap(), Status::PendingUpload);
}

#[test]
fn uncertain_rename_is_reconciled_without_resending_and_rebases_waiting_upload() {
    let (a, mut b) = UnixStream::pair().unwrap();
    let server = std::thread::spawn(move || {
        handshake(&mut b);
        for reconcile in [false, true] {
            let (req, fd) = crate::transport::receive::<Envelope<Command>>(&mut b).unwrap();
            assert!(fd.is_none());
            let Command::Rename {
                job,
                reconcile: got,
            } = req.body
            else {
                panic!("expected rename intent")
            };
            assert_eq!(got, reconcile);
            assert_eq!(job.source, "A.doc");
            assert_eq!(job.destination, "B.doc");
            let mut renamed = node("v2");
            renamed.name = "B.doc".into();
            crate::transport::send(
                &mut b,
                &Envelope {
                    version: VERSION,
                    id: req.id,
                    body: if reconcile {
                        Response::Renamed {
                            node: renamed,
                            same_content: true,
                        }
                    } else {
                        Response::Uncertain
                    },
                },
                None,
            )
            .unwrap();
        }
    });
    let (_dir, store, bridge) = fixture(a);
    store.lock().unwrap().ensure_fs_root("1").unwrap();
    store
        .lock()
        .unwrap()
        .rename_local("1", "A.doc", "1", "B.doc", false)
        .unwrap();
    assert_eq!(bridge.rename_pending().unwrap(), 0);
    assert_eq!(
        store.lock().unwrap().next_rename().unwrap().unwrap().state,
        "uncertain"
    );
    assert_eq!(bridge.rename_pending().unwrap(), 1);
    assert!(store.lock().unwrap().next_rename().unwrap().is_none());
    let s = store.lock().unwrap();
    let upload = s.upload_operations("pending").unwrap().pop().unwrap();
    let job = s.upload_job(&upload).unwrap();
    assert_eq!(job.relative_path, "B.doc");
    assert_eq!(job.node.version, "v2");
    drop(s);
    server.join().unwrap();
}

#[test]
fn uncertain_remove_reconciles_identity_without_resending_and_unblocks_recreation() {
    let (a, mut b) = UnixStream::pair().unwrap();
    let server = std::thread::spawn(move || {
        handshake(&mut b);
        for reconcile in [false, true] {
            let (req, fd) = crate::transport::receive::<Envelope<Command>>(&mut b).unwrap();
            assert!(fd.is_none());
            let Command::Remove {
                job,
                reconcile: got,
            } = req.body
            else {
                panic!("expected remove")
            };
            assert_eq!(got, reconcile);
            assert_eq!(job.path, "A.doc");
            assert_eq!(job.node.id, "2");
            crate::transport::send(
                &mut b,
                &Envelope {
                    version: VERSION,
                    id: req.id,
                    body: if reconcile {
                        Response::Removed {
                            identity: job.node.id,
                        }
                    } else {
                        Response::Uncertain
                    },
                },
                None,
            )
            .unwrap();
        }
    });
    let (_dir, store, bridge) = fixture(a);
    {
        let mut s = store.lock().unwrap();
        s.ensure_fs_root("1").unwrap();
        s.unlink_local("1", "A.doc", "2", false).unwrap();
        let n = s.create_local_file("1", "A.doc", 0o644).unwrap();
        s.creation_saved(&n.id).unwrap();
    }
    assert_eq!(bridge.namespace_pending().unwrap(), 0);
    assert_eq!(
        store.lock().unwrap().next_remove().unwrap().unwrap().state,
        "uncertain"
    );
    assert_eq!(bridge.namespace_pending().unwrap(), 1);
    let mut s = store.lock().unwrap();
    assert!(s.next_namespace_kind().unwrap().is_none());
    let op = s.upload_operations("pending").unwrap().remove(0);
    s.mark_upload_started(&op).unwrap();
    drop(s);
    server.join().unwrap();
}

#[test]
fn replacement_upload_and_cleanup_have_independent_uncertainty_barriers() {
    let (a, mut b) = UnixStream::pair().unwrap();
    let server = std::thread::spawn(move || {
        handshake(&mut b);
        for reconcile in [false, true] {
            let (req, fd) = crate::transport::receive::<Envelope<Command>>(&mut b).unwrap();
            assert!(fd.is_some());
            let Command::Replace {
                job,
                reconcile: got,
                ..
            } = req.body
            else {
                panic!("expected replacement")
            };
            assert_eq!(got, reconcile);
            assert_eq!(job.destination, "A.doc");
            assert_eq!(job.source_remote.as_deref(), Some("3"));
            crate::transport::send(
                &mut b,
                &Envelope {
                    version: VERSION,
                    id: req.id,
                    body: if reconcile {
                        Response::Committed { node: node("v2") }
                    } else {
                        Response::Uncertain
                    },
                },
                None,
            )
            .unwrap();
        }
        for reconcile in [false, true] {
            let (req, fd) = crate::transport::receive::<Envelope<Command>>(&mut b).unwrap();
            assert!(fd.is_none());
            let Command::Remove {
                job,
                reconcile: got,
            } = req.body
            else {
                panic!("expected source cleanup")
            };
            assert_eq!(got, reconcile);
            assert_eq!(job.path, "temporary");
            assert_eq!(job.node.version, "s1");
            assert_ne!(job.node.id, "3");
            crate::transport::send(
                &mut b,
                &Envelope {
                    version: VERSION,
                    id: req.id,
                    body: if reconcile {
                        Response::Removed {
                            identity: job.node.id,
                        }
                    } else {
                        Response::Uncertain
                    },
                },
                None,
            )
            .unwrap();
        }
    });
    let (_dir, store, bridge) = fixture(a);
    {
        let mut s = store.lock().unwrap();
        s.ensure_fs_root("1").unwrap();
        let mut source = node("s1");
        source.id = "3".into();
        source.name = "temporary".into();
        let from = s.cursor().unwrap();
        s.receive(&drive_core::Batch {
            from,
            to: "source-event".into(),
            events: vec![drive_core::Event {
                id: "source".into(),
                change: drive_core::Change::Upsert(source),
            }],
        })
        .unwrap();
        s.drain().unwrap();
        s.publish_download(
            &drive_core::DownloadTicket {
                id: "3".into(),
                version: "s1".into(),
            },
            b"new",
            &drive_core::sha256(b"new"),
        )
        .unwrap();
        s.replace_local("1", "temporary", "1", "A.doc", "3", "2")
            .unwrap();
    }
    assert_eq!(bridge.namespace_pending().unwrap(), 0);
    assert_eq!(
        store.lock().unwrap().next_replace().unwrap().unwrap().state,
        "uncertain"
    );
    assert_eq!(bridge.namespace_pending().unwrap(), 1);
    assert_eq!(
        store.lock().unwrap().next_replace().unwrap().unwrap().state,
        "cleanup_uncertain"
    );
    assert_eq!(bridge.namespace_pending().unwrap(), 1);
    assert!(
        store
            .lock()
            .unwrap()
            .next_namespace_kind()
            .unwrap()
            .is_none()
    );
    assert_eq!(store.lock().unwrap().local_identity("2").unwrap(), "3");
    server.join().unwrap();
}

#[test]
fn unknown_directory_receipt_never_replays_creation_or_releases_child_uploads() {
    let (a, mut b) = UnixStream::pair().unwrap();
    let server = std::thread::spawn(move || {
        handshake(&mut b);
        for expected in [false, true] {
            let (req, fd) = crate::transport::receive::<Envelope<Command>>(&mut b).unwrap();
            assert!(fd.is_none());
            match req.body {
                Command::Mkdir { job, reconcile } => {
                    assert_eq!(reconcile, expected);
                    assert_eq!(job.path, "newdir");
                }
                _ => panic!("unexpected mutation"),
            }
            crate::transport::send(
                &mut b,
                &Envelope {
                    version: VERSION,
                    id: req.id,
                    body: Response::Uncertain,
                },
                None,
            )
            .unwrap();
        }
    });
    let (_d, store, bridge) = fixture(a);
    let (dir, op) = {
        let mut s = store.lock().unwrap();
        let dir = s.create_local_directory("1", "newdir", 0o755).unwrap();
        let child = s.create_local_file(&dir.id, "file", 0o644).unwrap();
        let op = s.save_local(&child.id, b"local child").unwrap();
        s.creation_saved(&child.id).unwrap();
        (dir, op)
    };
    assert_eq!(bridge.namespace_pending().unwrap(), 0);
    assert_eq!(bridge.namespace_pending().unwrap(), 0);
    let mut s = store.lock().unwrap();
    assert_eq!(s.next_directory().unwrap().unwrap().state, "uncertain");
    assert!(s.mark_upload_started(&op).is_err());
    assert_eq!(s.status(&dir.id).unwrap(), Status::PendingUpload);
    drop(s);
    server.join().unwrap();
}

#[test]
fn directory_move_refresh_is_persisted_before_a_second_conditional_request() {
    fn version(max: u64) -> String {
        serde_json::json!({"schema":1,"restore":"fixture","view":1,"root":"1","file":"30","sync":2,"max":max,"size":0,"hash":"","directory":true}).to_string()
    }
    let (a, mut b) = UnixStream::pair().unwrap();
    let server = std::thread::spawn(move || {
        handshake(&mut b);
        for max in [2, 3] {
            let (req, fd) = crate::transport::receive::<Envelope<Command>>(&mut b).unwrap();
            assert!(fd.is_none());
            let Command::Rename { job, reconcile } = req.body else {
                panic!("expected rename")
            };
            assert!(!reconcile);
            assert_eq!(job.node.version, version(max));
            let mut node = job.node;
            node.version = version(3);
            let body = if max == 2 {
                Response::DirectoryRenameRefresh { node }
            } else {
                node.name = "B".into();
                Response::Renamed {
                    node,
                    same_content: true,
                }
            };
            crate::transport::send(
                &mut b,
                &Envelope {
                    version: VERSION,
                    id: req.id,
                    body,
                },
                None,
            )
            .unwrap();
        }
    });
    let (_d, store, bridge) = fixture(a);
    {
        let mut s = store.lock().unwrap();
        let mut dir = node(&version(2));
        dir.id = "30".into();
        dir.name = "A".into();
        dir.directory = true;
        dir.metadata.as_mut().unwrap().size = 0;
        s.receive(&drive_core::Batch {
            from: "1".into(),
            to: "2".into(),
            events: vec![drive_core::Event {
                id: "dir".into(),
                change: drive_core::Change::Upsert(dir),
            }],
        })
        .unwrap();
        s.drain().unwrap();
        s.ensure_fs_root("1").unwrap();
        s.rename_local("1", "A", "1", "B", false).unwrap();
    }
    assert_eq!(bridge.namespace_pending().unwrap(), 0);
    assert_eq!(
        store.lock().unwrap().next_rename().unwrap().unwrap().state,
        "pending"
    );
    assert_eq!(bridge.namespace_pending().unwrap(), 1);
    server.join().unwrap();
}

#[test]
fn directory_preparation_is_persisted_before_send_and_survives_uncertain_reply() {
    let (a, mut b) = UnixStream::pair().unwrap();
    let server = std::thread::spawn(move || {
        handshake(&mut b);
        for phase in 0..3 {
            let (req, fd) = crate::transport::receive::<Envelope<Command>>(&mut b).unwrap();
            assert!(fd.is_none());
            let Command::Mkdir { job, reconcile } = req.body else {
                panic!("unexpected command")
            };
            assert_eq!(reconcile, phase == 2);
            assert_eq!(job.creation_contract, 1);
            let body = if phase == 0 {
                assert!(job.origin.is_none());
                Response::DirectoryPrepared {
                    origin: drive_core::directories::DirectoryOrigin {
                        cursor: "native-v1:12".into(),
                        committer: "a".repeat(64),
                        restore: "r".into(),
                    },
                }
            } else {
                let origin = job.origin.unwrap();
                assert_eq!(origin.cursor, "native-v1:12");
                assert_eq!(origin.committer, "a".repeat(64));
                if phase == 1 {
                    Response::Uncertain
                } else {
                    let mut node = job.node;
                    node.version = "confirmed".into();
                    Response::Created {
                        node,
                        remote: "10".into(),
                    }
                }
            };
            crate::transport::send(
                &mut b,
                &Envelope {
                    version: VERSION,
                    id: req.id,
                    body,
                },
                None,
            )
            .unwrap();
        }
    });
    let (_d, store, bridge) = fixture(a);
    let (dir, op) = {
        let mut s = store.lock().unwrap();
        let d = s.create_local_directory("1", "prepared", 0o755).unwrap();
        let f = s.create_local_file(&d.id, "file", 0o644).unwrap();
        let op = s.save_local(&f.id, b"local").unwrap();
        s.creation_saved(&f.id).unwrap();
        (d, op)
    };
    assert_eq!(bridge.namespace_pending().unwrap(), 1);
    {
        let mut s = store.lock().unwrap();
        assert_eq!(s.next_directory().unwrap().unwrap().state, "uncertain");
        assert!(s.mark_upload_started(&op).is_err());
    }
    assert_eq!(bridge.namespace_pending().unwrap(), 1);
    let mut s = store.lock().unwrap();
    assert_eq!(s.local_identity("10").unwrap(), dir.id);
    s.mark_upload_started(&op).unwrap();
    drop(s);
    server.join().unwrap();
}

#[test]
fn new_file_provenance_is_durable_before_upload_and_preserved_for_reconciliation() {
    use drive_core::{directories::DirectoryOrigin, upload_origin::UploadOrigin};
    let (a, mut b) = UnixStream::pair().unwrap();
    b.set_read_timeout(Some(Duration::from_secs(10))).unwrap();
    let (tx, rx) = std::sync::mpsc::channel::<(Arc<Mutex<Store>>, String)>();
    let server = std::thread::spawn(move || {
        handshake(&mut b);
        let (store, op) = rx.recv().unwrap();
        let (req, fd) = crate::transport::receive::<Envelope<Command>>(&mut b).unwrap();
        assert!(fd.is_none());
        let Command::PrepareUpload {
            node,
            parent,
            path,
            sha256,
            cursor: _,
        } = req.body
        else {
            panic!("expected read-only preparation")
        };
        assert_eq!(parent.id, "1");
        assert_eq!(path, "/lab/New.txt");
        assert_eq!(
            store.lock().unwrap().upload_job(&op).unwrap().state,
            "pending"
        );
        let origin = UploadOrigin {
            schema: 1,
            node: node.id.clone(),
            parent_file: "1".into(),
            path,
            sha256,
            boundary: DirectoryOrigin {
                cursor: "native-v1:10".into(),
                committer: "a".repeat(64),
                restore: "fixture".into(),
            },
        };
        crate::transport::send(
            &mut b,
            &Envelope {
                version: VERSION,
                id: req.id,
                body: Response::UploadPrepared {
                    origin: origin.clone(),
                },
            },
            None,
        )
        .unwrap();
        for expected_reconcile in [false, true] {
            let (req, fd) = crate::transport::receive::<Envelope<Command>>(&mut b).unwrap();
            assert!(fd.is_some());
            let Command::Upload {
                origin: received,
                reconcile,
                ..
            } = req.body
            else {
                panic!("expected creation upload")
            };
            assert_eq!(reconcile, expected_reconcile);
            assert_eq!(received, Some(origin.clone()));
            assert_eq!(
                store.lock().unwrap().upload_origin(&op).unwrap(),
                Some(origin.clone())
            );
            let mut confirmed = node.clone();
            confirmed.version = "created-v1".into();
            crate::transport::send(
                &mut b,
                &Envelope {
                    version: VERSION,
                    id: req.id,
                    body: if reconcile {
                        Response::Created {
                            node: confirmed,
                            remote: "90".into(),
                        }
                    } else {
                        Response::Uncertain
                    },
                },
                None,
            )
            .unwrap();
        }
    });
    let (_dir, store, bridge) = fixture(a);
    let op = {
        let mut s = store.lock().unwrap();
        for old in s.upload_operations("pending").unwrap() {
            s.mark_upload_started(&old).unwrap();
            s.confirm_upload(&old, "v2").unwrap();
        }
        let n = s.create_local_file("1", "New.txt", 0o644).unwrap();
        let op = s.save_local(&n.id, b"new creation").unwrap();
        s.creation_saved(&n.id).unwrap();
        op
    };
    tx.send((store.clone(), op.clone())).unwrap();
    assert_eq!(bridge.upload_pending(false).unwrap(), 0);
    assert_eq!(
        store.lock().unwrap().upload_job(&op).unwrap().state,
        "uncertain"
    );
    assert_eq!(bridge.upload_pending(true).unwrap(), 1);
    assert_eq!(
        store.lock().unwrap().upload_job(&op).unwrap().state,
        "committed"
    );
    server.join().unwrap();
}

#[test]
fn replacement_prepares_durable_target_creation_before_upload_and_reuses_it_after_uncertainty() {
    replacement_origin_roundtrip(false);
}
#[test]
fn lost_read_only_replacement_preparation_leaves_operation_pending() {
    replacement_origin_roundtrip(true);
}
fn replacement_origin_roundtrip(lose_preparation: bool) {
    use drive_core::{commit_origin::CommitOrigin, upload_origin::UploadOrigin};
    let (a, mut b) = UnixStream::pair().unwrap();
    b.set_read_timeout(Some(Duration::from_secs(10))).unwrap();
    let (tx, rx) = std::sync::mpsc::channel::<Arc<Mutex<Store>>>();
    let server = std::thread::spawn(move || {
        handshake(&mut b);
        let store = rx.recv().unwrap();
        let (req, fd) = crate::transport::receive::<Envelope<Command>>(&mut b).unwrap();
        assert!(fd.is_none());
        let Command::PrepareUpload {
            node,
            parent,
            path,
            sha256,
            ..
        } = req.body
        else {
            panic!("expected read-only preparation")
        };
        assert_eq!(parent.id, "1");
        assert_eq!(path, "/lab/Destination");
        assert_eq!(
            store.lock().unwrap().next_replace().unwrap().unwrap().state,
            "pending"
        );
        if lose_preparation {
            return;
        }
        let origin = UploadOrigin {
            schema: 1,
            node: node.id,
            parent_file: "1".into(),
            path,
            sha256,
            boundary: CommitOrigin {
                cursor: "native-v1:10".into(),
                committer: "a".repeat(64),
                restore: "fixture".into(),
            },
        };
        crate::transport::send(
            &mut b,
            &Envelope {
                version: VERSION,
                id: req.id,
                body: Response::UploadPrepared {
                    origin: origin.clone(),
                },
            },
            None,
        )
        .unwrap();
        for expected in [false, true] {
            let (req, fd) = crate::transport::receive::<Envelope<Command>>(&mut b).unwrap();
            assert!(fd.is_some());
            let Command::Replace { job, reconcile, .. } = req.body else {
                panic!("expected replacement target submission")
            };
            assert_eq!(reconcile, expected);
            assert_eq!(job.creation_origin, Some(origin.clone()));
            assert_eq!(
                store
                    .lock()
                    .unwrap()
                    .next_replace()
                    .unwrap()
                    .unwrap()
                    .job
                    .creation_origin,
                Some(origin.clone())
            );
            let mut target = job.target;
            target.version = "created-destination".into();
            crate::transport::send(
                &mut b,
                &Envelope {
                    version: VERSION,
                    id: req.id,
                    body: if expected {
                        Response::Created {
                            node: target,
                            remote: "98".into(),
                        }
                    } else {
                        Response::Uncertain
                    },
                },
                None,
            )
            .unwrap();
        }
    });
    let (_dir, store, bridge) = fixture(a);
    let source = {
        let mut s = store.lock().unwrap();
        s.ensure_fs_root("1").unwrap();
        for op in s.upload_operations("pending").unwrap() {
            s.mark_upload_started(&op).unwrap();
            s.confirm_upload(&op, "v2").unwrap();
        }
        let source = s.create_local_file("1", "Temporary", 0o644).unwrap();
        s.save_local(&source.id, b"replacement").unwrap();
        s.creation_saved(&source.id).unwrap();
        let target = s.create_local_file("1", "Destination", 0o644).unwrap();
        s.save_local(&target.id, b"old").unwrap();
        s.creation_saved(&target.id).unwrap();
        s.replace_local("1", "Temporary", "1", "Destination", &source.id, &target.id)
            .unwrap();
        source
    };
    tx.send(store.clone()).unwrap();
    if lose_preparation {
        assert!(bridge.namespace_pending().is_err());
        assert_eq!(
            store.lock().unwrap().next_replace().unwrap().unwrap().state,
            "pending"
        );
    } else {
        assert_eq!(bridge.namespace_pending().unwrap(), 0);
        assert_eq!(
            store.lock().unwrap().next_replace().unwrap().unwrap().state,
            "uncertain"
        );
        assert_eq!(bridge.namespace_pending().unwrap(), 1);
        assert!(store.lock().unwrap().next_replace().unwrap().is_none());
        assert_eq!(
            store.lock().unwrap().local_identity("98").unwrap(),
            source.id
        );
    }
    server.join().unwrap();
}
