use super::*;
use crate::transport;
use std::{
    io::{Seek, SeekFrom, Write},
    sync::mpsc,
    time::Instant,
};
fn fixture(pin: bool) -> (tempfile::TempDir, Arc<Bridge>, Node, UnixStream) {
    let dir = tempfile::tempdir().unwrap();
    let mut store = Store::open(dir.path()).unwrap();
    let root = Node {
        id: "root".into(),
        parent: None,
        name: "root".into(),
        version: "v1".into(),
        directory: true,
        metadata: None,
    };
    let file = Node {
        id: "file".into(),
        parent: Some("root".into()),
        name: "A.doc".into(),
        version: "v1".into(),
        directory: false,
        metadata: None,
    };
    store
        .install_remote_snapshot(&[root, file.clone()], "1", "server")
        .unwrap();
    store.ensure_fs_root("root").unwrap();
    if pin {
        store.pin("file", true).unwrap();
    }
    let (a, b) = UnixStream::pair().unwrap();
    b.set_read_timeout(Some(Duration::from_secs(4))).unwrap();
    let bridge = Arc::new(
        Bridge::from_connection(Arc::new(Mutex::new(store)), Arc::new(Rpc::new(a).unwrap()))
            .unwrap(),
    );
    (dir, bridge, file, b)
}
fn request(b: &mut UnixStream) -> Envelope<Command> {
    transport::receive(b).unwrap().0
}
fn content(b: &mut UnixStream, id: u64) {
    let mut body = tempfile::tempfile().unwrap();
    body.write_all(b"new").unwrap();
    body.seek(SeekFrom::Start(0)).unwrap();
    transport::send(
        b,
        &Envelope {
            version: VERSION,
            id,
            body: Response::Content {
                identity: "file".into(),
                version: "v1".into(),
                size: 3,
                sha256: drive_core::sha256(b"new"),
            },
        },
        Some(&OwnedFd::from(body)),
    )
    .unwrap();
}
#[test]
fn abandoned_read_cancels_rpc_without_failure_or_retry_and_later_read_succeeds() {
    let (_dir, bridge, file, mut peer) = fixture(false);
    let token = drive_fs::ReadCancellation::default();
    let c = token.clone();
    let b = bridge.clone();
    let n = file.clone();
    let first = std::thread::spawn(move || b.ensure_cancellable(&n, Some(&c)));
    let old = request(&mut peer);
    assert!(matches!(old.body, Command::Download { .. }));
    // A local background queue pass may see this inflight record, but must
    // not turn it into an independent uninterruptible download demand.
    let scan = Instant::now();
    bridge.refresh_cached();
    assert!(scan.elapsed() < Duration::from_millis(200));
    token.cancel();
    let started = Instant::now();
    let cancel = request(&mut peer);
    assert!(matches!(cancel.body,Command::Cancel{request} if request==old.id));
    assert_eq!(first.join().unwrap(), Err(libc::EINTR));
    assert!(started.elapsed() < Duration::from_secs(1));
    {
        let s = bridge.store.lock().unwrap();
        assert_eq!(s.download_failure_count().unwrap(), 0);
        assert!(s.downloads_needed().unwrap().is_empty());
        assert!(s.open_cached("file").is_err());
    }
    assert_eq!(bridge.download_failures_seen.load(Ordering::Relaxed), 0);
    // An already in-flight terminal descriptor may arrive after cancellation.
    content(&mut peer, old.id);
    let b = bridge.clone();
    let second = std::thread::spawn(move || b.ensure(&file));
    let new = request(&mut peer);
    assert!(matches!(new.body, Command::Download { .. }));
    assert_ne!(new.id, old.id);
    content(&mut peer, new.id);
    second.join().unwrap().unwrap();
    assert_eq!(bridge.downloads.load(Ordering::Relaxed), 2);
}
#[test]
fn cancelling_owner_does_not_cancel_another_reader_of_same_version() {
    let (_dir, bridge, file, mut peer) = fixture(false);
    let first = drive_fs::ReadCancellation::default();
    let c = first.clone();
    let b = bridge.clone();
    let n = file.clone();
    let owner = std::thread::spawn(move || b.ensure_cancellable(&n, Some(&c)));
    let active = request(&mut peer);
    let other = drive_fs::ReadCancellation::default();
    let b = bridge.clone();
    let n = file.clone();
    let waiter = std::thread::spawn(move || b.ensure_cancellable(&n, Some(&other)));
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        let count = bridge
            .flights
            .lock()
            .unwrap()
            .values()
            .next()
            .unwrap()
            .demand
            .lock()
            .unwrap()
            .readers
            .len();
        if count == 2 {
            break;
        }
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(5));
    }
    first.cancel();
    std::thread::sleep(Duration::from_millis(80));
    content(&mut peer, active.id);
    assert_eq!(owner.join().unwrap(), Err(libc::EINTR));
    waiter.join().unwrap().unwrap();
    assert_eq!(bridge.downloads.load(Ordering::Relaxed), 1);
    assert!(bridge.store.lock().unwrap().open_cached("file").is_ok());
    peer.set_read_timeout(Some(Duration::from_millis(100)))
        .unwrap();
    assert!(transport::receive::<Envelope<Command>>(&mut peer).is_err());
}
#[test]
fn pinned_body_finishes_after_its_only_foreground_reader_cancels() {
    for pinned_before_open in [true, false] {
        let (_dir, bridge, file, mut peer) = fixture(pinned_before_open);
        let token = drive_fs::ReadCancellation::default();
        let c = token.clone();
        let b = bridge.clone();
        let reader = std::thread::spawn(move || b.ensure_cancellable(&file, Some(&c)));
        let active = request(&mut peer);
        if !pinned_before_open {
            bridge.store.lock().unwrap().pin("file", true).unwrap();
        }
        token.cancel();
        std::thread::sleep(Duration::from_millis(80));
        content(&mut peer, active.id);
        assert_eq!(reader.join().unwrap(), Err(libc::EINTR));
        assert!(bridge.store.lock().unwrap().open_cached("file").is_ok());
        assert_eq!(bridge.download_failures_seen.load(Ordering::Relaxed), 0);
    }
}

#[test]
fn cancelled_waiter_detaches_promptly_while_original_reader_keeps_downloading() {
    let (_dir, bridge, file, mut peer) = fixture(false);
    let b = bridge.clone();
    let n = file.clone();
    let owner = std::thread::spawn(move || b.ensure(&n));
    let active = request(&mut peer);
    let cancel = drive_fs::ReadCancellation::default();
    let c = cancel.clone();
    let b = bridge.clone();
    let (tx, rx) = mpsc::channel();
    let waiter =
        std::thread::spawn(move || tx.send(b.ensure_cancellable(&file, Some(&c))).unwrap());
    cancel.cancel();
    assert_eq!(
        rx.recv_timeout(Duration::from_millis(500)).unwrap(),
        Err(libc::EINTR)
    );
    content(&mut peer, active.id);
    owner.join().unwrap().unwrap();
    waiter.join().unwrap();
    assert_eq!(bridge.downloads.load(Ordering::Relaxed), 1);
}

#[path = "cancel_backoff_tests.rs"]
mod backoff;
