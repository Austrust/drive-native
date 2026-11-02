use super::*;

/// The first mkdir has a durable origin but no acknowledgement. An unrelated
/// rename may finish; its dependent child cannot be sent, even after retry.
#[test]
fn lost_mkdir_ack_does_not_head_of_line_block_independent_rename_or_replay_create() {
    let (a, mut b) = UnixStream::pair().unwrap();
    b.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
    let (tx, rx) = std::sync::mpsc::channel();
    let server = std::thread::spawn(move || {
        handshake(&mut b);
        for phase in 0..4 {
            let (req, fd) = crate::transport::receive::<Envelope<Command>>(&mut b).unwrap();
            assert!(fd.is_none());
            let response = match req.body {
                Command::Mkdir { job, reconcile } => {
                    assert_eq!(job.path, ".Trash-1000");
                    assert!(job.origin.is_some());
                    assert_eq!(
                        reconcile,
                        phase != 0,
                        "an unknown commit must never be resent"
                    );
                    assert!(matches!(phase, 0 | 2 | 3));
                    Response::Uncertain
                }
                Command::Rename { mut job, reconcile } => {
                    assert_eq!(phase, 1);
                    assert!(!reconcile);
                    assert_eq!(job.node.id, "2");
                    job.node.name = "renamed".into();
                    job.node.version = "v2".into();
                    Response::Renamed {
                        node: job.node,
                        same_content: true,
                    }
                }
                _ => panic!("dependent mkdir/upload must not run"),
            };
            crate::transport::send(
                &mut b,
                &Envelope {
                    version: VERSION,
                    id: req.id,
                    body: response,
                },
                None,
            )
            .unwrap();
        }
        tx.send(()).unwrap();
        // Keep the connection alive until the client explicitly finishes. No
        // reply-close race or unbounded fixture wait masks request ordering.
        assert!(crate::transport::receive::<Envelope<Command>>(&mut b).is_err());
    });
    let (d, store, bridge) = fixture(a);
    let (op, child_file) = {
        let mut s = store.lock().unwrap();
        s.ensure_fs_root("1").unwrap();
        let dir = s.create_local_directory("1", ".Trash-1000", 0o700).unwrap();
        let op = s.next_directory().unwrap().unwrap().job.operation;
        s.mark_directory_started(&op).unwrap();
        s.prepare_directory(
            &op,
            drive_core::directories::DirectoryOrigin {
                cursor: "native-v1:12".into(),
                committer: "a".repeat(64),
                restore: "fixture".into(),
            },
        )
        .unwrap();
        let child = s.create_local_directory(&dir.id, "files", 0o700).unwrap();
        let f = s.create_local_file(&child.id, "draft", 0o600).unwrap();
        s.save_local(&f.id, b"protected").unwrap();
        s.creation_saved(&f.id).unwrap();
        s.rename_local("1", "A.doc", "1", "renamed", false).unwrap();
        (op, f.id)
    };
    assert_eq!(bridge.namespace_pending().unwrap(), 1);
    {
        let s = store.lock().unwrap();
        let h = s.queue_health().unwrap();
        assert_eq!(h.namespace_uncertain, 1);
        assert_eq!(h.namespace_unfinished, 2);
        assert_eq!(h.upload_unfinished, 2);
        assert!(h.next_recovery_ms.is_some());
        assert!(!s.namespace_recovery_due().unwrap());
        assert_eq!(s.node("2").unwrap().name, "renamed");
    }
    // A UI refresh/upload wake does not cause another network observation.
    assert_eq!(bridge.namespace_pending().unwrap(), 0);
    for _ in 0..2 {
        store.lock().unwrap().clear_namespace_observation(&op);
        assert_eq!(bridge.namespace_pending().unwrap(), 0);
    }
    rx.recv_timeout(Duration::from_secs(5)).unwrap();
    bridge.connection().close();
    server.join().unwrap();
    let mut content = Vec::new();
    std::io::Read::read_to_end(
        &mut store.lock().unwrap().open_cached(&child_file).unwrap(),
        &mut content,
    )
    .unwrap();
    assert_eq!(content, b"protected");
    assert!(d.path().join("state.sqlite").is_file());
    assert_eq!(
        store
            .lock()
            .unwrap()
            .queue_health()
            .unwrap()
            .namespace_uncertain,
        1
    );
}
