use super::*;
fn download() -> Command {
    Command::Download {
        node: drive_core::Node {
            id: "file".into(),
            parent: None,
            name: "A.doc".into(),
            version: "v1".into(),
            directory: false,
            metadata: None,
        },
    }
}
fn progress(bytes: u64, total: u64) -> TransferProgress {
    TransferProgress {
        epoch: 1,
        direction: "download".into(),
        phase: if bytes == total {
            "verifying"
        } else {
            "transferring"
        }
        .into(),
        bytes,
        total,
    }
}
fn send(b: &mut UnixStream, id: u64, body: Response) {
    transport::send(
        b,
        &Envelope {
            version: VERSION,
            id,
            body,
        },
        None,
    )
    .unwrap();
}
#[test]
fn progress_outlives_initial_timeout_and_full_payload_waits_for_terminal_receipt() {
    let (a, mut b) = UnixStream::pair().unwrap();
    let rpc = Arc::new(Rpc::new(a).unwrap());
    let (terminal_tx, terminal_rx) = mpsc::channel();
    let (full_tx, full_rx) = mpsc::channel();
    let (done_tx, done_rx) = mpsc::channel();
    let server = std::thread::spawn(move || {
        let (request, _) = transport::receive::<Envelope<Command>>(&mut b).unwrap();
        for bytes in 0..=6 {
            send(
                &mut b,
                request.id,
                Response::TransferProgress(progress(bytes, 6)),
            );
            std::thread::sleep(Duration::from_millis(70));
        }
        full_tx.send(()).unwrap();
        terminal_rx.recv().unwrap();
        send(
            &mut b,
            request.id,
            Response::Failed {
                category: "synthetic_terminal".into(),
            },
        );
        done_rx.recv().unwrap();
    });
    let client = rpc.clone();
    let (result_tx, result_rx) = mpsc::channel();
    let caller = std::thread::spawn(move || {
        result_tx
            .send(client.call(download(), Duration::from_millis(300)))
            .unwrap()
    });
    full_rx.recv_timeout(Duration::from_secs(3)).unwrap();
    assert!(matches!(
        result_rx.try_recv(),
        Err(mpsc::TryRecvError::Empty)
    ));
    let entries = rpc.transfers.entries();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].progress.as_ref().unwrap().bytes, 6);
    assert_eq!(entries[0].progress.as_ref().unwrap().phase, "verifying");
    terminal_tx.send(()).unwrap();
    let result = result_rx
        .recv_timeout(Duration::from_secs(1))
        .unwrap()
        .unwrap();
    assert!(matches!(result.body,Response::Failed{category} if category=="synthetic_terminal"));
    assert!(rpc.transfers.entries().is_empty());
    done_tx.send(()).unwrap();
    caller.join().unwrap();
    server.join().unwrap();
}
#[test]
fn stalled_progress_is_cancelled_and_late_update_cannot_resurrect_entry() {
    let (a, mut b) = UnixStream::pair().unwrap();
    let rpc = Rpc::new(a).unwrap();
    let server = std::thread::spawn(move || {
        let (old, _) = transport::receive::<Envelope<Command>>(&mut b).unwrap();
        send(&mut b, old.id, Response::TransferProgress(progress(1, 2)));
        let (cancel, _) = transport::receive::<Envelope<Command>>(&mut b).unwrap();
        assert!(matches!(cancel.body,Command::Cancel{request} if request==old.id));
        send(&mut b, old.id, Response::TransferProgress(progress(2, 2)));
        send(
            &mut b,
            old.id,
            Response::Failed {
                category: "late".into(),
            },
        );
        let (next, _) = transport::receive::<Envelope<Command>>(&mut b).unwrap();
        send(&mut b, next.id, Response::Events(None));
    });
    assert_eq!(
        rpc.call(download(), Duration::from_millis(100))
            .err()
            .unwrap()
            .kind(),
        io::ErrorKind::TimedOut
    );
    assert!(rpc.transfers.entries().is_empty());
    assert!(matches!(
        rpc.call(Command::Pull { cursor: "x".into() }, Duration::from_secs(1))
            .unwrap()
            .body,
        Response::Events(None)
    ));
    assert!(rpc.transfers.entries().is_empty());
    server.join().unwrap();
}
#[test]
fn malformed_progress_disconnects_waiter_and_clears_registry() {
    let (a, mut b) = UnixStream::pair().unwrap();
    let rpc = Rpc::new(a).unwrap();
    let server = std::thread::spawn(move || {
        let (req, _) = transport::receive::<Envelope<Command>>(&mut b).unwrap();
        send(&mut b, req.id, Response::TransferProgress(progress(3, 2)));
    });
    assert!(rpc.call(download(), Duration::from_secs(1)).is_err());
    assert!(rpc.transfers.entries().is_empty());
    assert!(!rpc.is_connected());
    server.join().unwrap();
}
#[test]
fn registry_rejects_regressions_ignores_duplicates_and_resets_new_epoch() {
    let r = crate::transfers::TransferRegistry::default();
    r.register(1, &download());
    r.update(1, progress(2, 4)).unwrap();
    let revision = r.revision();
    r.update(1, progress(2, 4)).unwrap();
    assert_eq!(r.revision(), revision);
    assert!(r.update(1, progress(1, 4)).is_err());
    assert!(r.update(1, progress(3, 5)).is_err());
    let mut next = progress(0, 0);
    next.epoch = 2;
    next.direction = "upload".into();
    next.phase = "awaiting_confirmation".into();
    r.update(1, next).unwrap();
    r.update(1, progress(4, 4)).unwrap();
    let entries = r.entries();
    let p = entries[0].progress.as_ref().unwrap();
    assert_eq!(p.epoch, 2);
    assert_eq!(p.bytes, 0);
    r.remove(1);
    r.update(1, progress(4, 4)).unwrap();
    assert!(r.entries().is_empty());
}
