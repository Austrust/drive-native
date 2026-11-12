use super::*;
use crate::transport;

#[test]
fn profile_only_notification_invalidates_observed_acl_without_file_events() {
    use drive_core::write_permissions::{self as permissions, Action, Decision, LocalPolicy};
    for (profile_changed, changed_views) in [(false, vec![]), (true, vec![]), (true, vec![8])] {
        let dir = tempfile::tempdir().unwrap();
        let mut store = Store::open(dir.path()).unwrap();
        let root = Node {
            id: "root".into(), parent: None, name: "root".into(),
            version: "v1".into(), directory: true, metadata: None,
        };
        store.install_remote_snapshot(&[root], "1", "server").unwrap();
        store.ensure_fs_root("root").unwrap();
        permissions::record_observed(&store, "root", "root", Action::List, false).unwrap();
        permissions::set_local_policy(&store, "root", LocalPolicy::ExplicitReadOnly).unwrap();
        let revision = permissions::policy(&store, "root").unwrap().write_policy_revision;
        let (socket, mut peer) = UnixStream::pair().unwrap();
        peer.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
        let bridge = Bridge::from_connection(
            Arc::new(drive_core::SharedStore::new(store)),
            Arc::new(Rpc::new(socket).unwrap()),
        ).unwrap();
        let server = std::thread::spawn(move || {
            let (pull, _) = transport::receive::<Envelope<Command>>(&mut peer).unwrap();
            assert!(matches!(pull.body, Command::Pull { .. }));
            transport::send(&mut peer, &Envelope {
                version: VERSION, id: pull.id, body: Response::Events(None),
            }, None).unwrap();
            let (wait, _) = transport::receive::<Envelope<Command>>(&mut peer).unwrap();
            assert!(matches!(wait.body, Command::Wait { .. }));
            // Use the actual worker projection: the native bit must survive
            // even if only another view appears in changed_views.
            let body = crate::worker::notification_response(7, sproto::actions::native::Notification {
                changed_views, profile_changed, profile_digest: "next".into(),
            });
            transport::send(&mut peer, &Envelope {
                version: VERSION, id: wait.id, body,
            }, None).unwrap();
            let (pull, _) = transport::receive::<Envelope<Command>>(&mut peer).unwrap();
            assert!(matches!(pull.body, Command::Pull { .. }));
            transport::send(&mut peer, &Envelope {
                version: VERSION, id: pull.id, body: Response::Events(None),
            }, None).unwrap();
            let (next, _) = transport::receive::<Envelope<Command>>(&mut peer).unwrap();
            assert!(matches!(next.body, Command::Wait { cursor, digest } if cursor == "1" && digest == "next"));
            // Close only after notification application and resubscription.
        });
        assert!(bridge.event_loop().is_err());
        server.join().unwrap();
        let store = bridge.store.lock().unwrap();
        assert_eq!(permissions::decide(&store, "root", "root", Action::List).unwrap(),
            if profile_changed { Decision::Unknown } else { Decision::Deny });
        let policy = permissions::policy(&store, "root").unwrap();
        assert_eq!(policy.local_policy, LocalPolicy::ExplicitReadOnly);
        assert_eq!(policy.write_policy_revision, revision + u64::from(profile_changed));
        assert_eq!(bridge.downloads.load(Ordering::Relaxed), 0);
    }
}

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
        Arc::new(drive_core::SharedStore::new(store)),
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

/// Issue #67 W2: a 1000-event catch-up Pull is applied in bounded chunks under
/// the hood. The receipt/committed-cursor and namespace must still end up
/// exactly as before: every event applied once, every history row written, and
/// the cursor advanced only at the end.
#[test]
fn pull_applies_a_large_catch_up_batch_through_chunked_apply() {
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
    store
        .install_remote_snapshot(&[root], "1", "server")
        .unwrap();
    store.ensure_fs_root("root").unwrap();
    let events: Vec<drive_core::Event> = (0..1000)
        .map(|i| drive_core::Event {
            id: format!("ev-batch-{i}"),
            change: Change::Upsert(Node {
                id: format!("batch-{i}"),
                parent: Some("root".into()),
                name: format!("{i:04}.bin"),
                version: "v1".into(),
                directory: false,
                metadata: None,
            }),
        })
        .collect();
    let batch = drive_core::Batch {
        from: "1".into(),
        to: "2".into(),
        events,
    };
    let (socket, mut peer) = UnixStream::pair().unwrap();
    peer.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
    let bridge = Bridge::from_connection(
        Arc::new(drive_core::SharedStore::new(store)),
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
                body: Response::Events(Some(batch)),
            },
            None,
        )
        .unwrap();
    });
    assert!(bridge.pull().unwrap());
    server.join().unwrap();
    let store = bridge.store.lock().unwrap();
    assert_eq!(store.cursor().unwrap(), "2");
    // Events carry server ids; the namespace stores the mapped local identity.
    let first = store.local_identity("batch-0").unwrap();
    assert!(store.node(&first).is_ok());
    let last = store.local_identity("batch-999").unwrap();
    assert!(store.node(&last).is_ok());
    // The page API caps a page at 500; walk the cursor to count the batch.
    let mut total = 0usize;
    let mut cursor: Option<String> = None;
    let mut newest_first = true;
    loop {
        let page = store
            .sync_history(&drive_core::sync_history::HistoryQuery {
                limit: Some(500),
                cursor: cursor.clone(),
                ..Default::default()
            })
            .unwrap();
        newest_first &= page.records.windows(2).all(|pair| pair[0].seq > pair[1].seq);
        total += page.records.len();
        cursor = page.next_cursor.clone();
        if cursor.is_none() {
            break;
        }
    }
    assert_eq!(total, 1000);
    assert!(
        newest_first,
        "history must be ordered newest-first without duplicates"
    );
}
