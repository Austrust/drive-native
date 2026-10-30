use super::*;
use crate::transport;
#[test]
fn event_subscription_is_not_blocked_by_pending_hydration() {
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
        .install_remote_snapshot(&[root, file], "1", "server")
        .unwrap();
    store.ensure_fs_root("root").unwrap();
    store.pin("file", true).unwrap();
    assert_eq!(store.downloads_needed().unwrap().len(), 1);
    let (socket, mut peer) = UnixStream::pair().unwrap();
    peer.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
    let bridge = Bridge::from_connection(
        Arc::new(Mutex::new(store)),
        Arc::new(Rpc::new(socket).unwrap()),
    )
    .unwrap();
    let server = std::thread::spawn(move || {
        let (pull, _) = transport::receive::<Envelope<Command>>(&mut peer).unwrap();
        assert!(matches!(pull.body, Command::Pull { .. }));
        transport::send(
            &mut peer,
            &Envelope {
                version: VERSION,
                id: pull.id,
                body: Response::Events(None),
            },
            None,
        )
        .unwrap();
        let (next, _) = transport::receive::<Envelope<Command>>(&mut peer).unwrap();
        assert!(
            matches!(next.body, Command::Wait { .. }),
            "event thread attempted body transfer before subscription"
        );
    });
    assert!(bridge.event_loop().is_err());
    server.join().unwrap();
    assert_eq!(
        bridge
            .store
            .lock()
            .unwrap()
            .downloads_needed()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(bridge.downloads.load(Ordering::Relaxed), 0);
    assert!(*bridge.upload_signal.0.lock().unwrap());
}
