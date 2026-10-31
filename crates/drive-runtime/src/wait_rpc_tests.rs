use super::*;
use crate::wait_activity::monotonic_ns;

fn wait_command() -> Command {
    Command::Wait {
        cursor: "synthetic".into(),
        digest: String::new(),
    }
}
fn send(socket: &mut UnixStream, id: u64, body: Response) {
    transport::send(
        socket,
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
fn actual_progress_outlives_initial_ipc_budget_and_completes() {
    let (a, mut b) = UnixStream::pair().unwrap();
    let rpc = Rpc::new(a).unwrap();
    let server = std::thread::spawn(move || {
        let (request, _) = transport::receive::<Envelope<Command>>(&mut b).unwrap();
        for bytes in 1..=5 {
            std::thread::sleep(Duration::from_millis(200));
            send(
                &mut b,
                request.id,
                Response::WaitProgress {
                    bytes,
                    received_at_ns: monotonic_ns().unwrap(),
                },
            );
        }
        send(
            &mut b,
            request.id,
            Response::Notification {
                changed: false,
                digest: "done".into(),
            },
        );
    });
    let response = rpc
        .call(wait_command(), Duration::from_millis(750))
        .unwrap();
    assert!(matches!(response.body, Response::Notification { digest, .. } if digest == "done"));
    assert!(rpc.pending.lock().unwrap().is_empty());
    server.join().unwrap();
}

#[test]
fn silence_cancels_and_late_progress_cannot_extend_another_request() {
    let (a, mut b) = UnixStream::pair().unwrap();
    let rpc = Rpc::new(a).unwrap();
    let server = std::thread::spawn(move || {
        let (old, _) = transport::receive::<Envelope<Command>>(&mut b).unwrap();
        let (cancel, _) = transport::receive::<Envelope<Command>>(&mut b).unwrap();
        assert!(matches!(cancel.body, Command::Cancel { request } if request == old.id));
        let (next, _) = transport::receive::<Envelope<Command>>(&mut b).unwrap();
        send(
            &mut b,
            old.id,
            Response::WaitProgress {
                bytes: 1,
                received_at_ns: monotonic_ns().unwrap(),
            },
        );
        send(
            &mut b,
            old.id,
            Response::Notification {
                changed: false,
                digest: "old".into(),
            },
        );
        send(
            &mut b,
            next.id,
            Response::Notification {
                changed: false,
                digest: "next".into(),
            },
        );
    });
    let error = rpc
        .call(wait_command(), Duration::from_millis(100))
        .err()
        .unwrap();
    assert_eq!(error.kind(), io::ErrorKind::TimedOut);
    let response = rpc.call(wait_command(), Duration::from_secs(2)).unwrap();
    assert!(matches!(response.body, Response::Notification { digest, .. } if digest == "next"));
    server.join().unwrap();
}

#[test]
fn cancellation_still_interrupts_a_wait_with_active_progress() {
    let (a, mut b) = UnixStream::pair().unwrap();
    let rpc = Rpc::new(a).unwrap();
    let stop = Arc::new(AtomicBool::new(false));
    let server_stop = stop.clone();
    let server = std::thread::spawn(move || {
        let (request, _) = transport::receive::<Envelope<Command>>(&mut b).unwrap();
        send(
            &mut b,
            request.id,
            Response::WaitProgress {
                bytes: 1,
                received_at_ns: monotonic_ns().unwrap(),
            },
        );
        server_stop.store(true, Ordering::Release);
        let (cancel, _) = transport::receive::<Envelope<Command>>(&mut b).unwrap();
        assert!(matches!(cancel.body, Command::Cancel { request: id } if id == request.id));
    });
    let result = rpc.call_cancellable(wait_command(), Duration::from_secs(3), &|| {
        stop.load(Ordering::Acquire)
    });
    assert_eq!(result.err().unwrap().kind(), io::ErrorKind::Interrupted);
    assert!(rpc.pending.lock().unwrap().is_empty());
    server.join().unwrap();
}

#[test]
fn progress_on_non_wait_request_and_future_timestamps_fail_closed() {
    for body in [
        Command::Pull {
            cursor: String::new(),
        },
        wait_command(),
    ] {
        let (a, mut b) = UnixStream::pair().unwrap();
        let rpc = Rpc::new(a).unwrap();
        let server = std::thread::spawn(move || {
            let (request, _) = transport::receive::<Envelope<Command>>(&mut b).unwrap();
            send(
                &mut b,
                request.id,
                Response::WaitProgress {
                    bytes: 1,
                    received_at_ns: u64::MAX,
                },
            );
        });
        let result = rpc.call(body, Duration::from_secs(2));
        assert_eq!(result.err().unwrap().kind(), io::ErrorKind::NotConnected);
        server.join().unwrap();
    }
}
