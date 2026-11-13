//! #9 TB3 runtime tests: every namespace/upload commit branch that can end in
//! an unknown outcome must leave a durable `commit_diagnostics` row, and the
//! scheduler must not overwrite the concrete reconcile reason with its generic
//! observation text. Also pins the defensive journal-state branches: they may
//! no longer silently return success.
use super::*;
use drive_core::{NodeMetadata, sha256};
use std::os::unix::net::UnixStream;
use std::sync::Arc;

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

fn fixture(socket: UnixStream) -> (tempfile::TempDir, Arc<drive_core::SharedStore>, Bridge) {
    let dir = tempfile::tempdir().unwrap();
    let store = Arc::new(drive_core::SharedStore::new(
        Store::open(dir.path()).unwrap(),
    ));
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

/// These runtime fixtures deliberately advance reconciliation without wall-clock
/// waits; the scheduler-specific backoff is covered in the core tests.
fn namespace_without_backoff(
    bridge: &Bridge,
    store: &Arc<drive_core::SharedStore>,
) -> io::Result<usize> {
    let operation = store
        .lock()
        .unwrap()
        .next_namespace_operation()
        .unwrap()
        .map(|q| q.operation);
    let result = bridge.namespace_pending();
    if let Some(operation) = operation {
        store
            .lock()
            .unwrap()
            .clear_namespace_observation(&operation);
    }
    result
}

fn namespace_diagnostic(
    store: &Arc<drive_core::SharedStore>,
    operation: &str,
) -> Option<drive_core::commit_diagnostics::CommitDiagnosticRecord> {
    drive_core::commit_diagnostics::list(&store.lock().unwrap(), 100)
        .unwrap()
        .into_iter()
        .find(|diagnostic| diagnostic.scope == "namespace" && diagnostic.operation == operation)
}

fn upload_diagnostic(
    store: &Arc<drive_core::SharedStore>,
    operation: &str,
) -> Option<drive_core::commit_diagnostics::CommitDiagnosticRecord> {
    drive_core::commit_diagnostics::list(&store.lock().unwrap(), 100)
        .unwrap()
        .into_iter()
        .find(|diagnostic| diagnostic.scope == "operation" && diagnostic.operation == operation)
}

fn committed_node(version: &str) -> Node {
    node(version)
}

// ---------------------------------------------------------------------------
// T1: replace PrepareUpload unexpected reply must surface as a failure, not a
// silent successful pass. It must not fabricate a commit diagnosis either.
// ---------------------------------------------------------------------------
#[test]
fn replace_prepare_upload_unexpected_reply_surfaces_background_failure() {
    let (a, mut b) = UnixStream::pair().unwrap();
    let server = std::thread::spawn(move || {
        handshake(&mut b);
        let (req, fd) = crate::transport::receive::<Envelope<Command>>(&mut b).unwrap();
        assert!(fd.is_none());
        let Command::PrepareUpload { node: target, .. } = req.body else {
            panic!("expected replacement preparation")
        };
        assert_eq!(target.version, drive_core::creations::CREATE_VERSION);
        // Contract violation: a prepared-upload request can only be answered by
        // UploadPrepared or Conflict.
        crate::transport::send(
            &mut b,
            &Envelope {
                version: VERSION,
                id: req.id,
                body: Response::Renamed {
                    node: committed_node("v2"),
                    same_content: true,
                },
            },
            None,
        )
        .unwrap();
    });
    let (_dir, store, bridge) = fixture(a);
    let replace_op = {
        let mut s = store.lock().unwrap();
        s.ensure_fs_root("1").unwrap();
        let target = s.create_local_file("1", "New.txt", 0o644).unwrap();
        s.creation_saved(&target.id).unwrap();
        s.replace_local("1", "A.doc", "1", "New.txt", "2", &target.id)
            .unwrap();
        s.next_replace().unwrap().unwrap().job.operation
    };
    assert_eq!(
        store.lock().unwrap().next_replace().unwrap().unwrap().state,
        "pending"
    );
    bridge.run_upload_cycle();
    assert!(
        bridge.background_failure().is_some(),
        "an unexpected prepare reply must be reported instead of silently skipped"
    );
    let s = store.lock().unwrap();
    // Queue retained, no Replace sent, and no fabricated commit diagnosis.
    assert_eq!(
        s.next_replace().unwrap().unwrap().state,
        "pending",
        "replacement must stay queued for a later retry"
    );
    assert!(
        drive_core::commit_diagnostics::list(&s, 100)
            .unwrap()
            .iter()
            .all(|diagnostic| diagnostic.operation != replace_op),
        "a pre-commit preparation failure is not an unknown commit outcome"
    );
    server.join().unwrap();
}

// ---------------------------------------------------------------------------
// T4/upload: reconcile branches persist the observed category and text.
// ---------------------------------------------------------------------------
#[test]
fn upload_reconcile_unexpected_reply_records_protocol_diagnostic() {
    let (a, mut b) = UnixStream::pair().unwrap();
    let server = std::thread::spawn(move || {
        handshake(&mut b);
        for reconcile in [false, true] {
            let (req, fd) = crate::transport::receive::<Envelope<Command>>(&mut b).unwrap();
            assert!(fd.is_some());
            let Command::Upload {
                reconcile: got, ..
            } = req.body
            else {
                panic!("expected upload")
            };
            assert_eq!(got, reconcile);
            crate::transport::send(
                &mut b,
                &Envelope {
                    version: VERSION,
                    id: req.id,
                    body: if reconcile {
                        Response::Renamed {
                            node: committed_node("v2"),
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
    let operation = store
        .lock()
        .unwrap()
        .upload_operations("pending")
        .unwrap()
        .pop()
        .unwrap();
    assert_eq!(bridge.upload_pending(false).unwrap(), 0);
    assert_eq!(bridge.upload_pending(true).unwrap(), 0);
    let diagnostic = upload_diagnostic(&store, &operation).expect("durable upload diagnostic");
    assert_eq!(
        diagnostic.outcome,
        drive_core::commit_outcome::CommitOutcome::UnknownRetryable
    );
    assert_eq!(
        diagnostic.category,
        drive_core::commit_outcome::CommitCategory::Protocol
    );
    assert_eq!(
        diagnostic.last_error.as_deref(),
        Some("server returned an unexpected reply during reconcile")
    );
    assert_eq!(diagnostic.state, "uncertain");
    assert_eq!(diagnostic.attempts, 1);
    assert!(diagnostic.next_retry_at > 0);
    server.join().unwrap();
}

#[test]
fn upload_reconcile_transport_error_records_transport_diagnostic_without_spending_budget() {
    let (a, mut b) = UnixStream::pair().unwrap();
    let server = std::thread::spawn(move || {
        handshake(&mut b);
        let (req, fd) = crate::transport::receive::<Envelope<Command>>(&mut b).unwrap();
        assert!(fd.is_some());
        let Command::Upload {
            reconcile: got, ..
        } = req.body
        else {
            panic!("expected upload")
        };
        assert!(!got);
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
        // The worker disappears during the read-only reconciliation.
        let (_req, _fd) = crate::transport::receive::<Envelope<Command>>(&mut b).unwrap();
    });
    let (_dir, store, bridge) = fixture(a);
    let operation = store
        .lock()
        .unwrap()
        .upload_operations("pending")
        .unwrap()
        .pop()
        .unwrap();
    assert_eq!(bridge.upload_pending(false).unwrap(), 0);
    assert!(bridge.upload_pending(true).is_err());
    let diagnostic = upload_diagnostic(&store, &operation).expect("durable upload diagnostic");
    assert_eq!(
        diagnostic.category,
        drive_core::commit_outcome::CommitCategory::Transport
    );
    assert_eq!(
        diagnostic.last_error.as_deref(),
        Some("transport lost during reconcile; waiting for reconnect")
    );
    assert_eq!(diagnostic.state, "uncertain");
    assert_eq!(diagnostic.attempts, 0);
    server.join().unwrap();
}

// ---------------------------------------------------------------------------
// T4/rename: the reconcile branch writes a concrete reason, and the scheduler
// observation must preserve it (T3).
// ---------------------------------------------------------------------------
#[test]
fn rename_reconcile_unexpected_reply_keeps_concrete_diagnostic() {
    let (a, mut b) = UnixStream::pair().unwrap();
    let server = std::thread::spawn(move || {
        handshake(&mut b);
        for reconcile in [false, true] {
            let (req, fd) = crate::transport::receive::<Envelope<Command>>(&mut b).unwrap();
            assert!(fd.is_none());
            let Command::Rename {
                reconcile: got, ..
            } = req.body
            else {
                panic!("expected rename")
            };
            assert_eq!(got, reconcile);
            crate::transport::send(
                &mut b,
                &Envelope {
                    version: VERSION,
                    id: req.id,
                    body: if reconcile {
                        Response::Created {
                            node: committed_node("v2"),
                            remote: "remote-2".into(),
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
    let operation = {
        let mut s = store.lock().unwrap();
        s.ensure_fs_root("1").unwrap();
        s.rename_local("1", "A.doc", "1", "B.doc", false).unwrap();
        s.next_rename().unwrap().unwrap().job.operation
    };
    assert_eq!(bridge.rename_pending().unwrap(), 0);
    assert_eq!(namespace_without_backoff(&bridge, &store).unwrap(), 0);
    let diagnostic = namespace_diagnostic(&store, &operation).expect("durable rename diagnostic");
    assert_eq!(
        diagnostic.outcome,
        drive_core::commit_outcome::CommitOutcome::UnknownRetryable
    );
    assert_eq!(
        diagnostic.category,
        drive_core::commit_outcome::CommitCategory::Protocol
    );
    assert_eq!(diagnostic.state, "uncertain");
    assert_eq!(
        diagnostic.last_error.as_deref(),
        Some("server returned an unexpected reply during rename reconcile"),
        "scheduler observation must not replace the concrete reconcile reason"
    );
    assert!(diagnostic.attempts > 0);
    assert!(diagnostic.next_retry_at > 0);
    server.join().unwrap();
}

// ---------------------------------------------------------------------------
// T4/remove: same contract for the destructive path.
// ---------------------------------------------------------------------------
#[test]
fn remove_reconcile_unexpected_reply_keeps_concrete_diagnostic() {
    let (a, mut b) = UnixStream::pair().unwrap();
    let server = std::thread::spawn(move || {
        handshake(&mut b);
        for reconcile in [false, true] {
            let (req, fd) = crate::transport::receive::<Envelope<Command>>(&mut b).unwrap();
            assert!(fd.is_none());
            let Command::Remove {
                reconcile: got, ..
            } = req.body
            else {
                panic!("expected remove")
            };
            assert_eq!(got, reconcile);
            crate::transport::send(
                &mut b,
                &Envelope {
                    version: VERSION,
                    id: req.id,
                    body: if reconcile {
                        Response::Created {
                            node: committed_node("v2"),
                            remote: "remote-2".into(),
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
    let operation = {
        let mut s = store.lock().unwrap();
        s.ensure_fs_root("1").unwrap();
        s.unlink_local("1", "A.doc", "2", false).unwrap();
        let recreated = s.create_local_file("1", "A.doc", 0o644).unwrap();
        s.creation_saved(&recreated.id).unwrap();
        s.next_remove().unwrap().unwrap().job.operation
    };
    assert_eq!(namespace_without_backoff(&bridge, &store).unwrap(), 0);
    assert_eq!(namespace_without_backoff(&bridge, &store).unwrap(), 0);
    let diagnostic = namespace_diagnostic(&store, &operation).expect("durable remove diagnostic");
    assert_eq!(diagnostic.state, "uncertain");
    assert_eq!(
        diagnostic.category,
        drive_core::commit_outcome::CommitCategory::Protocol
    );
    assert_eq!(
        diagnostic.last_error.as_deref(),
        Some("server returned an unexpected reply during remove reconcile")
    );
    assert!(diagnostic.attempts > 0);
    assert!(diagnostic.next_retry_at > 0);
    server.join().unwrap();
}

// ---------------------------------------------------------------------------
// T4/replace: the replace reconcile branch also keeps its concrete reason.
// ---------------------------------------------------------------------------
#[test]
fn replace_reconcile_unexpected_reply_keeps_concrete_diagnostic() {
    let (a, mut b) = UnixStream::pair().unwrap();
    let server = std::thread::spawn(move || {
        handshake(&mut b);
        for reconcile in [false, true] {
            let (req, fd) = crate::transport::receive::<Envelope<Command>>(&mut b).unwrap();
            assert!(fd.is_some());
            let Command::Replace {
                reconcile: got, ..
            } = req.body
            else {
                panic!("expected replace")
            };
            assert_eq!(got, reconcile);
            crate::transport::send(
                &mut b,
                &Envelope {
                    version: VERSION,
                    id: req.id,
                    body: if reconcile {
                        Response::Removed {
                            identity: "7".into(),
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
    let operation = {
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
        s.next_replace().unwrap().unwrap().job.operation
    };
    assert_eq!(namespace_without_backoff(&bridge, &store).unwrap(), 0);
    assert_eq!(namespace_without_backoff(&bridge, &store).unwrap(), 0);
    let diagnostic =
        namespace_diagnostic(&store, &operation).expect("durable replacement diagnostic");
    assert_eq!(diagnostic.state, "uncertain");
    assert_eq!(
        diagnostic.category,
        drive_core::commit_outcome::CommitCategory::Protocol
    );
    assert_eq!(
        diagnostic.last_error.as_deref(),
        Some("server returned an unexpected replacement reply")
    );
    assert!(diagnostic.attempts > 0);
    assert!(diagnostic.next_retry_at > 0);
    server.join().unwrap();
}

// ---------------------------------------------------------------------------
// T4/mkdir: the directory reconcile branch keeps its concrete reason.
// ---------------------------------------------------------------------------
#[test]
fn mkdir_reconcile_unexpected_reply_keeps_concrete_diagnostic() {
    let (a, mut b) = UnixStream::pair().unwrap();
    let server = std::thread::spawn(move || {
        handshake(&mut b);
        for reconcile in [false, true] {
            let (req, fd) = crate::transport::receive::<Envelope<Command>>(&mut b).unwrap();
            assert!(fd.is_none());
            let Command::Mkdir {
                reconcile: got, ..
            } = req.body
            else {
                panic!("expected mkdir")
            };
            assert_eq!(got, reconcile);
            crate::transport::send(
                &mut b,
                &Envelope {
                    version: VERSION,
                    id: req.id,
                    body: if reconcile {
                        Response::Renamed {
                            node: committed_node("v2"),
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
    let operation = {
        let mut s = store.lock().unwrap();
        s.ensure_fs_root("1").unwrap();
        s.create_local_directory("1", "newdir", 0o700).unwrap();
        s.next_directory().unwrap().unwrap().job.operation
    };
    assert_eq!(namespace_without_backoff(&bridge, &store).unwrap(), 0);
    assert_eq!(namespace_without_backoff(&bridge, &store).unwrap(), 0);
    let diagnostic = namespace_diagnostic(&store, &operation).expect("durable mkdir diagnostic");
    assert_eq!(diagnostic.state, "uncertain");
    assert_eq!(
        diagnostic.category,
        drive_core::commit_outcome::CommitCategory::Protocol
    );
    assert_eq!(
        diagnostic.last_error.as_deref(),
        Some("server returned an unexpected directory reply")
    );
    assert!(diagnostic.attempts > 0);
    assert!(diagnostic.next_retry_at > 0);
    server.join().unwrap();
}

// ---------------------------------------------------------------------------
// T2 / F1: the selector's known handoff states are yielded; only a durable
// state outside its seven-state filter is a defensive invariant failure.
// ---------------------------------------------------------------------------
fn idle_server(mut b: UnixStream) {
    handshake(&mut b);
    while crate::transport::receive::<Envelope<Command>>(&mut b).is_ok() {}
}

// F1: `namespace_selection` legitimately hands an `inflight` row to the owning
// handler (serial-lane handoff between mark_started and send). The handlers must
// yield that pass - no RPC, no diagnostic, no panic - instead of treating the
// known handoff state as an invariant violation.
#[test]
fn rename_inflight_handoff_is_yielded_without_diagnostic() {
    let (a, b) = UnixStream::pair().unwrap();
    let server = std::thread::spawn(move || idle_server(b));
    let (_dir, store, bridge) = fixture(a);
    let operation = {
        let mut s = store.lock().unwrap();
        s.ensure_fs_root("1").unwrap();
        s.rename_local("1", "A.doc", "1", "B.doc", false).unwrap();
        let operation = s.next_rename().unwrap().unwrap().job.operation;
        s.mark_rename_started(&operation).unwrap();
        operation
    };
    assert_eq!(bridge.rename_pending().unwrap(), 0);
    let s = store.lock().unwrap();
    assert_eq!(
        s.next_rename().unwrap().unwrap().state,
        "inflight",
        "handoff row is left for its owner"
    );
    assert!(
        drive_core::commit_diagnostics::list(&s, 100)
            .unwrap()
            .iter()
            .all(|diagnostic| diagnostic.operation != operation),
        "yielding an inflight handoff must not write a diagnosis for that operation"
    );
    drop(s);
    bridge.disconnect();
    drop(server);
}

#[test]
fn remove_inflight_handoff_is_yielded_without_diagnostic() {
    let (a, b) = UnixStream::pair().unwrap();
    let server = std::thread::spawn(move || idle_server(b));
    let (_dir, store, bridge) = fixture(a);
    let operation = {
        let mut s = store.lock().unwrap();
        s.ensure_fs_root("1").unwrap();
        s.unlink_local("1", "A.doc", "2", false).unwrap();
        let recreated = s.create_local_file("1", "A.doc", 0o644).unwrap();
        s.creation_saved(&recreated.id).unwrap();
        let operation = s.next_remove().unwrap().unwrap().job.operation;
        s.mark_remove_started(&operation).unwrap();
        operation
    };
    assert_eq!(bridge.namespace_pending().unwrap(), 0);
    let s = store.lock().unwrap();
    assert_eq!(s.next_remove().unwrap().unwrap().state, "inflight");
    assert!(
        drive_core::commit_diagnostics::list(&s, 100)
            .unwrap()
            .iter()
            .all(|diagnostic| diagnostic.operation != operation),
        "yielding an inflight handoff must not write a diagnosis for that operation"
    );
    drop(s);
    bridge.disconnect();
    drop(server);
}

#[test]
fn mkdir_inflight_handoff_is_yielded_without_diagnostic() {
    let (a, b) = UnixStream::pair().unwrap();
    let server = std::thread::spawn(move || idle_server(b));
    let (_dir, store, bridge) = fixture(a);
    let operation = {
        let mut s = store.lock().unwrap();
        s.ensure_fs_root("1").unwrap();
        s.create_local_directory("1", "newdir", 0o700).unwrap();
        let operation = s.next_directory().unwrap().unwrap().job.operation;
        s.mark_directory_started(&operation).unwrap();
        operation
    };
    assert_eq!(bridge.namespace_pending().unwrap(), 0);
    let s = store.lock().unwrap();
    assert_eq!(s.next_directory().unwrap().unwrap().state, "inflight");
    assert!(
        drive_core::commit_diagnostics::list(&s, 100)
            .unwrap()
            .iter()
            .all(|diagnostic| diagnostic.operation != operation),
        "yielding an inflight handoff must not write a diagnosis for that operation"
    );
    drop(s);
    bridge.disconnect();
    drop(server);
}
