use super::*;
use drive_core::{DownloadTicket, sha256};
use std::sync::{
    Condvar,
    atomic::{AtomicBool, Ordering},
    mpsc,
};

fn fixture(flags: i32) -> (tempfile::TempDir, Arc<Mutex<Store>>, Arc<Mutex<Handles>>) {
    let temp = tempfile::tempdir().unwrap();
    let mut store = Store::open(temp.path()).unwrap();
    let root = Node {
        id: "root".into(),
        parent: None,
        name: "root".into(),
        version: "1".into(),
        directory: true,
        metadata: None,
    };
    let file = Node {
        id: "cached".into(),
        parent: Some("root".into()),
        name: "cached".into(),
        version: "1".into(),
        directory: false,
        metadata: None,
    };
    store
        .install_remote_snapshot(&[root, file], "1", "synthetic")
        .unwrap();
    store
        .publish_download(
            &DownloadTicket {
                id: "cached".into(),
                version: "1".into(),
            },
            b"old",
            &sha256(b"old"),
        )
        .unwrap();
    let content = OpenContent::Cached(Arc::new(store.open_cached("cached").unwrap()));
    let handles = Handles {
        next: 2,
        entries: HashMap::from([(
            1,
            Handle {
                ino: 7,
                node: "cached".into(),
                flags,
                content,
            },
        )]),
    };
    (
        temp,
        Arc::new(Mutex::new(store)),
        Arc::new(Mutex::new(handles)),
    )
}

#[test]
fn readonly_route_captures_before_promotion_and_draft_routes_to_metadata() {
    let (_temp, store, handles) = fixture(libc::O_RDONLY);
    assert!(matches!(
        sync_route(&mut handles.lock().unwrap(), 7, 1, false),
        SyncRoute::Immutable(None)
    ));
    let body = handles
        .lock()
        .unwrap()
        .entries
        .get(&1)
        .unwrap()
        .content
        .clone();
    draft(
        &store,
        &handles,
        &Arc::new(Mutex::new(())),
        &None,
        "cached",
        body,
        false,
    )
    .unwrap();
    assert!(matches!(
        sync_route(&mut handles.lock().unwrap(), 7, 1, false),
        SyncRoute::Metadata
    ));
    assert!(matches!(
        sync_route(&mut handles.lock().unwrap(), 7, 1, true),
        SyncRoute::Metadata
    ));
    assert_eq!(handles.lock().unwrap().entries.len(), 1);
}

#[test]
fn writable_cached_missing_and_invalid_handles_never_take_fast_route() {
    for flags in [libc::O_WRONLY, libc::O_RDWR] {
        let (_temp, _store, handles) = fixture(flags);
        assert!(matches!(
            sync_route(&mut handles.lock().unwrap(), 7, 1, false),
            SyncRoute::Metadata
        ));
        assert!(matches!(
            sync_route(&mut handles.lock().unwrap(), 8, 1, true),
            SyncRoute::Invalid
        ));
        drop(ReleaseCleanup {
            handles: handles.clone(),
            ino: 8,
            fh: 1,
        });
        assert_eq!(handles.lock().unwrap().entries.len(), 1);
    }
    let (_temp, store, handles) = fixture(libc::O_RDONLY);
    assert!(matches!(
        sync_route(&mut handles.lock().unwrap(), 8, 1, true),
        SyncRoute::Invalid
    ));
    assert_eq!(handles.lock().unwrap().entries.len(), 1);
    handles.lock().unwrap().entries.get_mut(&1).unwrap().content =
        OpenContent::Missing(store.lock().unwrap().node("cached").unwrap());
    assert!(matches!(
        sync_route(&mut handles.lock().unwrap(), 7, 1, true),
        SyncRoute::Metadata
    ));
}

#[test]
fn removed_readonly_handle_does_not_release_an_inflight_reader() {
    let (_temp, store, handles) = fixture(libc::O_RDONLY);
    let reader = match &handles.lock().unwrap().entries[&1].content {
        OpenContent::Cached(c) => c.clone(),
        _ => panic!(),
    };
    let removed = sync_route(&mut handles.lock().unwrap(), 7, 1, true);
    assert!(handles.lock().unwrap().entries.is_empty());
    drop(removed);
    assert!(store.lock().unwrap().release("cached").is_err());
    let mut bytes = [0; 3];
    reader.read_at(&mut bytes, 0).unwrap();
    assert_eq!(&bytes, b"old");
    drop(reader);
    store.lock().unwrap().release("cached").unwrap();
}

#[test]
fn rejected_release_job_drops_removed_reference_and_slow_release_guard() {
    let (_temp, store, handles) = fixture(libc::O_RDONLY);
    let weak = match &handles.lock().unwrap().entries[&1].content {
        OpenContent::Cached(c) => Arc::downgrade(c),
        _ => panic!(),
    };
    let queue = LocalQueue::new();
    let gate = Arc::new((Mutex::new(false), Condvar::new()));
    let (tx, rx) = mpsc::channel();
    for _ in 0..4 {
        let gate = gate.clone();
        let tx = tx.clone();
        queue.submit(Box::new(move || {
            let _ = tx.send(());
            let mut open = gate.0.lock().unwrap();
            while !*open {
                open = gate.1.wait(open).unwrap();
            }
        }));
    }
    for _ in 0..4 {
        rx.recv_timeout(Duration::from_secs(3)).unwrap();
    }
    for _ in 0..256 {
        queue.submit(Box::new(|| {}));
    }
    let SyncRoute::Immutable(Some(handle)) = sync_route(&mut handles.lock().unwrap(), 7, 1, true)
    else {
        panic!()
    };
    let ran = Arc::new(AtomicBool::new(false));
    let observed = ran.clone();
    queue.submit(Box::new(move || {
        drop(handle);
        observed.store(true, Ordering::Relaxed);
    }));
    let released = weak.upgrade().is_none();
    let executed = ran.load(Ordering::Relaxed);
    let (_other, _, slow) = fixture(libc::O_RDWR);
    let cleanup = ReleaseCleanup {
        handles: slow.clone(),
        ino: 7,
        fh: 1,
    };
    queue.submit(Box::new(move || drop(cleanup)));
    let slow_released = slow.lock().unwrap().entries.is_empty();
    *gate.0.lock().unwrap() = true;
    gate.1.notify_all();
    drop(queue);
    assert!(released && slow_released);
    assert!(!executed);
    store.lock().unwrap().release("cached").unwrap();
}
