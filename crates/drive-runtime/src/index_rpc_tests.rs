use super::*;
use drive_core::snapshot_progress::{Phase, Progress};

fn progress(units: u64) -> Progress {
    Progress {
        attempt: 1,
        phase: Phase::Seed,
        units,
        items: units * 2,
    }
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
fn index_validation_rejects_regression_and_out_of_budget_counts() {
    assert!(valid_index_progress(Some(progress(2)), progress(3)));
    assert!(!valid_index_progress(Some(progress(2)), progress(1)));
    assert!(!valid_index_progress(None, progress(4097)));
    assert!(!valid_index_progress(
        None,
        Progress {
            attempt: 4,
            ..progress(0)
        }
    ));
    assert!(valid_index_progress(
        Some(progress(4)),
        Progress {
            attempt: 2,
            ..progress(0)
        }
    ));
    assert!(!valid_index_progress(
        Some(Progress {
            phase: Phase::Fence,
            ..progress(0)
        }),
        progress(1)
    ));
}

#[test]
fn index_progress_does_not_complete_request_and_callback_can_query_pending() {
    let (a, mut peer) = UnixStream::pair().unwrap();
    let rpc = Arc::new(Rpc::new(a).unwrap());
    let server = std::thread::spawn(move || {
        let (req, _) = transport::receive::<Envelope<Command>>(&mut peer).unwrap();
        send(&mut peer, req.id, Response::IndexProgress(progress(1)));
        send(
            &mut peer,
            req.id,
            Response::Failed {
                category: "synthetic-final".into(),
            },
        );
    });
    let observed = Arc::new(AtomicU64::new(0));
    let count = observed.clone();
    let weak = Arc::downgrade(&rpc);
    let reply = rpc
        .call_index(
            Duration::from_secs(2),
            Arc::new(move |p| {
                assert_eq!(weak.upgrade().unwrap().pending.lock().unwrap().len(), 1);
                count.store(p.units, Ordering::Release);
            }),
        )
        .unwrap();
    assert_eq!(observed.load(Ordering::Acquire), 1);
    assert!(matches!(reply.body,Response::Failed{category} if category=="synthetic-final"));
    server.join().unwrap();
}

#[test]
fn late_index_progress_cannot_reanimate_timed_out_request() {
    let (a, mut peer) = UnixStream::pair().unwrap();
    let rpc = Rpc::new(a).unwrap();
    let count = Arc::new(AtomicU64::new(0));
    let observed = count.clone();
    let server = std::thread::spawn(move || {
        let (old, _) = transport::receive::<Envelope<Command>>(&mut peer).unwrap();
        let (cancel, _) = transport::receive::<Envelope<Command>>(&mut peer).unwrap();
        assert!(matches!(cancel.body,Command::Cancel{request} if request==old.id));
        let (next, _) = transport::receive::<Envelope<Command>>(&mut peer).unwrap();
        send(&mut peer, old.id, Response::IndexProgress(progress(1)));
        send(&mut peer, next.id, Response::Events(None));
    });
    assert!(
        rpc.call_index(
            Duration::from_millis(25),
            Arc::new(move |_| {
                observed.fetch_add(1, Ordering::Relaxed);
            })
        )
        .is_err()
    );
    rpc.call(
        Command::Pull {
            cursor: "synthetic".into(),
        },
        Duration::from_secs(2),
    )
    .unwrap();
    assert_eq!(count.load(Ordering::Relaxed), 0);
    server.join().unwrap();
}

#[test]
fn index_progress_on_other_command_disconnects() {
    let (a, mut peer) = UnixStream::pair().unwrap();
    let rpc = Rpc::new(a).unwrap();
    let server = std::thread::spawn(move || {
        let (req, _) = transport::receive::<Envelope<Command>>(&mut peer).unwrap();
        send(&mut peer, req.id, Response::IndexProgress(progress(1)));
    });
    assert!(
        rpc.call(
            Command::Pull {
                cursor: "synthetic".into()
            },
            Duration::from_secs(2)
        )
        .is_err()
    );
    server.join().unwrap();
}

#[test]
fn snapshot_without_ui_observer_still_drains_valid_progress() {
    let (a, mut peer) = UnixStream::pair().unwrap();
    let rpc = Rpc::new(a).unwrap();
    let server = std::thread::spawn(move || {
        let (req, _) = transport::receive::<Envelope<Command>>(&mut peer).unwrap();
        send(&mut peer, req.id, Response::IndexProgress(progress(1)));
        send(
            &mut peer,
            req.id,
            Response::Failed {
                category: "synthetic".into(),
            },
        );
    });
    assert!(
        rpc.call(Command::RecoverySnapshot, Duration::from_secs(2))
            .is_ok()
    );
    server.join().unwrap();
}

#[test]
fn index_progress_does_not_renew_the_total_request_budget() {
    let (a, mut peer) = UnixStream::pair().unwrap();
    let rpc = Rpc::new(a).unwrap();
    let observed = Arc::new(AtomicU64::new(0));
    let count = observed.clone();
    let server = std::thread::spawn(move || {
        let (request, _) = transport::receive::<Envelope<Command>>(&mut peer).unwrap();
        std::thread::sleep(Duration::from_millis(100));
        send(&mut peer, request.id, Response::IndexProgress(progress(1)));
        std::thread::sleep(Duration::from_millis(200));
        send(&mut peer, request.id, Response::Events(None));
    });
    let result = rpc.call_index(Duration::from_millis(250), Arc::new(move |_| {
        count.fetch_add(1, Ordering::Relaxed);
    }));
    assert_eq!(result.err().unwrap().kind(), io::ErrorKind::TimedOut);
    assert_eq!(observed.load(Ordering::Relaxed), 1);
    server.join().unwrap();
}
