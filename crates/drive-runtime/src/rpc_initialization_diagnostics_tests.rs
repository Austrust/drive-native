use super::*;
use drive_core::native::InitializationRequest;
use sproto::diagnostics::{RequestOutcome, Snapshot};

fn diagnostic() -> Snapshot {
    Snapshot {
        list_calls: 1,
        list_failures: 1,
        last_list_outcome: Some(RequestOutcome::ResponseTimeout),
        ..Snapshot::default()
    }
}
fn pair() -> (Arc<Rpc>, UnixStream) {
    let (a, peer) = UnixStream::pair().unwrap();
    peer.set_read_timeout(Some(Duration::from_secs(3))).unwrap();
    peer.set_write_timeout(Some(Duration::from_secs(3)))
        .unwrap();
    (Arc::new(Rpc::new(a).unwrap()), peer)
}
fn send(peer: &mut UnixStream, id: u64, body: Response) {
    transport::send(
        peer,
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
fn diagnostics_are_nonfinal_delivered_before_failure_and_callback_has_no_pending_lock() {
    let (rpc, mut peer) = pair();
    let caller = rpc.clone();
    let weak = Arc::downgrade(&rpc);
    let (observed_tx, observed) = mpsc::channel();
    let (done_tx, done) = mpsc::channel();
    let task = std::thread::spawn(move || {
        let result = caller.call_initialization(
            InitializationRequest::Root,
            Duration::from_secs(2),
            Arc::new(move |snapshot| {
                assert!(weak.upgrade().unwrap().pending.try_lock().is_ok());
                observed_tx.send(snapshot).unwrap();
            }),
        );
        done_tx.send(result).unwrap();
    });
    let (request, _) = transport::receive::<Envelope<Command>>(&mut peer).unwrap();
    send(
        &mut peer,
        request.id,
        Response::InitializationDiagnostics(diagnostic()),
    );
    assert_eq!(
        observed.recv_timeout(Duration::from_secs(1)).unwrap(),
        diagnostic()
    );
    assert!(matches!(
        done.recv_timeout(Duration::from_millis(40)),
        Err(mpsc::RecvTimeoutError::Timeout)
    ));
    send(
        &mut peer,
        request.id,
        Response::Failed {
            category: "synthetic-native-response-timeout".into(),
        },
    );
    let reply = done.recv_timeout(Duration::from_secs(1)).unwrap().unwrap();
    assert!(
        matches!(reply.body,Response::Failed{category} if category=="synthetic-native-response-timeout")
    );
    rpc.close();
    task.join().unwrap();
}

#[test]
fn diagnostics_do_not_renew_the_original_deadline() {
    let (rpc, mut peer) = pair();
    let count = Arc::new(AtomicU64::new(0));
    let observed = count.clone();
    let server = std::thread::spawn(move || {
        let (request, _) = transport::receive::<Envelope<Command>>(&mut peer).unwrap();
        std::thread::sleep(Duration::from_millis(100));
        send(
            &mut peer,
            request.id,
            Response::InitializationDiagnostics(diagnostic()),
        );
        std::thread::sleep(Duration::from_millis(200));
        // A renewed 250 ms budget would incorrectly accept this final response.
        send(
            &mut peer,
            request.id,
            Response::Failed {
                category: "synthetic-final".into(),
            },
        );
        let _ = transport::receive::<Envelope<Command>>(&mut peer);
    });
    let result = rpc.call_initialization(
        InitializationRequest::Root,
        Duration::from_millis(250),
        Arc::new(move |_| {
            observed.fetch_add(1, Ordering::Relaxed);
        }),
    );
    assert_eq!(result.err().unwrap().kind(), io::ErrorKind::TimedOut);
    assert_eq!(count.load(Ordering::Relaxed), 1);
    server.join().unwrap();
    rpc.close();
}

#[test]
fn duplicate_current_diagnostics_disconnect_after_only_one_callback() {
    let (rpc, mut peer) = pair();
    let count = Arc::new(AtomicU64::new(0));
    let observed = count.clone();
    let server = std::thread::spawn(move || {
        let (request, _) = transport::receive::<Envelope<Command>>(&mut peer).unwrap();
        send(
            &mut peer,
            request.id,
            Response::InitializationDiagnostics(diagnostic()),
        );
        send(
            &mut peer,
            request.id,
            Response::InitializationDiagnostics(diagnostic()),
        );
        let _ = transport::receive::<Envelope<Command>>(&mut peer);
    });
    let result = rpc.call_initialization(
        InitializationRequest::Root,
        Duration::from_secs(2),
        Arc::new(move |_| {
            observed.fetch_add(1, Ordering::Relaxed);
        }),
    );
    assert_eq!(result.err().unwrap().kind(), io::ErrorKind::NotConnected);
    assert_eq!(count.load(Ordering::Relaxed), 1);
    assert!(!rpc.is_connected());
    server.join().unwrap();
}

#[test]
fn diagnostics_for_another_command_kind_disconnect() {
    let (rpc, mut peer) = pair();
    let server = std::thread::spawn(move || {
        let (request, _) = transport::receive::<Envelope<Command>>(&mut peer).unwrap();
        send(
            &mut peer,
            request.id,
            Response::InitializationDiagnostics(diagnostic()),
        );
        let _ = transport::receive::<Envelope<Command>>(&mut peer);
    });
    let result = rpc.call(
        Command::Pull {
            cursor: "synthetic".into(),
        },
        Duration::from_secs(2),
    );
    assert_eq!(result.err().unwrap().kind(), io::ErrorKind::NotConnected);
    server.join().unwrap();
}

#[test]
fn diagnostics_reject_unexpected_descriptor_or_invalid_bounded_shape() {
    for with_descriptor in [true, false] {
        let (rpc, mut peer) = pair();
        let count = Arc::new(AtomicU64::new(0));
        let observed = count.clone();
        let server = std::thread::spawn(move || {
            let (request, _) = transport::receive::<Envelope<Command>>(&mut peer).unwrap();
            let mut snapshot = diagnostic();
            if !with_descriptor {
                snapshot.list_calls = 0;
                assert!(!snapshot.valid());
            }
            let fd: OwnedFd = tempfile::tempfile().unwrap().into();
            transport::send(
                &mut peer,
                &Envelope {
                    version: VERSION,
                    id: request.id,
                    body: Response::InitializationDiagnostics(snapshot),
                },
                with_descriptor.then_some(&fd),
            )
            .unwrap();
            let _ = transport::receive::<Envelope<Command>>(&mut peer);
        });
        let result = rpc.call_initialization(
            InitializationRequest::Root,
            Duration::from_secs(2),
            Arc::new(move |_| {
                observed.fetch_add(1, Ordering::Relaxed);
            }),
        );
        assert_eq!(result.err().unwrap().kind(), io::ErrorKind::NotConnected);
        assert_eq!(count.load(Ordering::Relaxed), 0);
        server.join().unwrap();
    }
}

#[test]
fn late_diagnostics_after_timeout_do_not_revive_or_poison_the_next_command() {
    let (rpc, mut peer) = pair();
    let count = Arc::new(AtomicU64::new(0));
    let observed = count.clone();
    let server = std::thread::spawn(move || {
        let (old, _) = transport::receive::<Envelope<Command>>(&mut peer).unwrap();
        let (cancel, _) = transport::receive::<Envelope<Command>>(&mut peer).unwrap();
        assert!(matches!(cancel.body,Command::Cancel{request} if request==old.id));
        let (next, _) = transport::receive::<Envelope<Command>>(&mut peer).unwrap();
        send(
            &mut peer,
            old.id,
            Response::InitializationDiagnostics(diagnostic()),
        );
        send(
            &mut peer,
            old.id,
            Response::InitializationDiagnostics(diagnostic()),
        );
        send(&mut peer, next.id, Response::Events(None));
        let _ = transport::receive::<Envelope<Command>>(&mut peer);
    });
    let result = rpc.call_initialization(
        InitializationRequest::Root,
        Duration::from_millis(30),
        Arc::new(move |_| {
            observed.fetch_add(1, Ordering::Relaxed);
        }),
    );
    assert_eq!(result.err().unwrap().kind(), io::ErrorKind::TimedOut);
    let reply = rpc
        .call(
            Command::Pull {
                cursor: "synthetic".into(),
            },
            Duration::from_secs(2),
        )
        .unwrap();
    assert!(matches!(reply.body, Response::Events(None)));
    assert_eq!(count.load(Ordering::Relaxed), 0);
    assert!(rpc.is_connected());
    rpc.close();
    server.join().unwrap();
}
