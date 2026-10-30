use super::*;

fn fail(peer: &mut UnixStream, request: u64, category: &str) {
    transport::send(
        peer,
        &Envelope {
            version: VERSION,
            id: request,
            body: Response::Failed {
                category: category.into(),
            },
        },
        None,
    )
    .unwrap();
}
fn until(mut predicate: impl FnMut() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(2);
    while !predicate() {
        assert!(Instant::now() < deadline, "state transition timed out");
        std::thread::sleep(Duration::from_millis(5));
    }
}
fn backoff(bridge: &Bridge) {
    until(|| {
        bridge
            .store
            .lock()
            .unwrap()
            .download_failure("file")
            .unwrap()
            .is_some()
    });
}
fn readers(bridge: &Bridge, count: usize) {
    until(|| {
        bridge
            .flights
            .lock()
            .unwrap()
            .values()
            .next()
            .is_some_and(|f| f.demand.lock().unwrap().readers.len() == count)
    });
}
fn start(
    bridge: &Arc<Bridge>,
    node: &Node,
    token: &drive_fs::ReadCancellation,
) -> std::thread::JoinHandle<Result<(), i32>> {
    let (bridge, node, token) = (bridge.clone(), node.clone(), token.clone());
    std::thread::spawn(move || bridge.ensure_cancellable(&node, Some(&token)))
}

#[test]
fn cancelling_in_backoff_removes_only_read_intent_and_survives_restart() {
    for category in ["native_server_code:32772", "transport"] {
        let (dir, bridge, file, mut peer) = fixture(false);
        let token = drive_fs::ReadCancellation::default();
        let owner = start(&bridge, &file, &token);
        let first = request(&mut peer);
        fail(&mut peer, first.id, category);
        backoff(&bridge);
        // This previously saw no flight during backoff. It must now remain
        // protected against both queue takeover and cache release races.
        assert_eq!(bridge.active_downloads(), vec!["file"]);
        token.cancel();
        let started = Instant::now();
        assert_eq!(owner.join().unwrap(), Err(libc::EINTR));
        assert!(started.elapsed() < Duration::from_millis(500));
        bridge.refresh_cached();
        assert_eq!(bridge.downloads.load(Ordering::Relaxed), 1);
        assert_eq!(bridge.download_failures_seen.load(Ordering::Relaxed), 1);
        assert!(bridge.active_downloads().is_empty());
        bridge.disconnect();
        drop(bridge);
        drop(peer);
        let s = Store::open(dir.path()).unwrap();
        assert!(s.downloads_needed().unwrap().is_empty());
        assert_eq!(s.next_download_retry().unwrap(), None);
        assert_eq!(s.download_failure_count().unwrap(), 0);
        assert!(s.open_cached("file").is_err());
        assert_eq!(s.pending_operations().unwrap(), 0);
    }
}

#[test]
fn a_retry_attempt_is_still_cancellable_and_not_promoted_to_background() {
    let (_dir, bridge, file, mut peer) = fixture(false);
    let token = drive_fs::ReadCancellation::default();
    let owner = start(&bridge, &file, &token);
    let first = request(&mut peer);
    fail(&mut peer, first.id, "native_server_code:32772");
    backoff(&bridge);
    let second = request(&mut peer);
    assert!(matches!(second.body, Command::Download { .. }));
    bridge.refresh_cached();
    token.cancel();
    let cancelled = request(&mut peer);
    assert!(matches!(cancelled.body, Command::Cancel { request } if request == second.id));
    assert_eq!(owner.join().unwrap(), Err(libc::EINTR));
    assert!(
        bridge
            .store
            .lock()
            .unwrap()
            .downloads_needed()
            .unwrap()
            .is_empty()
    );
    // A late terminal message cannot attach the cancelled attempt to the next
    // read, even when identity and version are identical.
    content(&mut peer, second.id);
    let new_token = drive_fs::ReadCancellation::default();
    let fresh = start(&bridge, &file, &new_token);
    let new_request = request(&mut peer);
    assert_ne!(new_request.id, second.id);
    content(&mut peer, new_request.id);
    fresh.join().unwrap().unwrap();
    assert_eq!(bridge.downloads.load(Ordering::Relaxed), 3);
}

#[test]
fn a_reader_joining_during_backoff_preserves_retry_after_owner_cancels() {
    let (_dir, bridge, file, mut peer) = fixture(false);
    let token = drive_fs::ReadCancellation::default();
    let owner = start(&bridge, &file, &token);
    let first = request(&mut peer);
    fail(&mut peer, first.id, "transport");
    backoff(&bridge);
    let other = drive_fs::ReadCancellation::default();
    let waiter = start(&bridge, &file, &other);
    readers(&bridge, 2);
    token.cancel();
    let second = request(&mut peer);
    assert!(matches!(second.body, Command::Download { .. }));
    content(&mut peer, second.id);
    assert_eq!(owner.join().unwrap(), Err(libc::EINTR));
    waiter.join().unwrap().unwrap();
    assert_eq!(bridge.downloads.load(Ordering::Relaxed), 2);
    assert!(bridge.store.lock().unwrap().open_cached("file").is_ok());
}

#[test]
fn all_readers_can_abandon_one_shared_backoff() {
    let (_dir, bridge, file, mut peer) = fixture(false);
    let one = drive_fs::ReadCancellation::default();
    let two = drive_fs::ReadCancellation::default();
    let owner = start(&bridge, &file, &one);
    let first = request(&mut peer);
    let waiter = start(&bridge, &file, &two);
    readers(&bridge, 2);
    fail(&mut peer, first.id, "transport");
    backoff(&bridge);
    two.cancel();
    assert_eq!(waiter.join().unwrap(), Err(libc::EINTR));
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
    one.cancel();
    assert_eq!(owner.join().unwrap(), Err(libc::EINTR));
    assert!(
        bridge
            .store
            .lock()
            .unwrap()
            .downloads_needed()
            .unwrap()
            .is_empty()
    );
}

#[test]
fn pin_added_in_backoff_keeps_retry_when_reader_leaves() {
    let (_dir, bridge, file, mut peer) = fixture(false);
    let token = drive_fs::ReadCancellation::default();
    let owner = start(&bridge, &file, &token);
    let first = request(&mut peer);
    fail(&mut peer, first.id, "native_server_code:32772");
    backoff(&bridge);
    bridge.store.lock().unwrap().pin("file", true).unwrap();
    token.cancel();
    let second = request(&mut peer);
    assert!(matches!(second.body, Command::Download { .. }));
    content(&mut peer, second.id);
    assert_eq!(owner.join().unwrap(), Err(libc::EINTR));
    let s = bridge.store.lock().unwrap();
    assert!(s.is_pinned("file").unwrap());
    assert!(s.open_cached("file").is_ok());
    assert!(s.download_failure("file").unwrap().is_none());
}

#[test]
fn old_complete_cache_keeps_background_refresh_through_cancelled_backoff() {
    let (_dir, bridge, file, mut peer) = fixture(false);
    {
        let mut s = bridge.store.lock().unwrap();
        let root = s.node("root").unwrap();
        let mut old = file.clone();
        old.version = "v0".into();
        s.install_remote_snapshot(&[root.clone(), old], "old", "server")
            .unwrap();
        s.publish_download(
            &DownloadTicket {
                id: "file".into(),
                version: "v0".into(),
            },
            b"old",
            &drive_core::sha256(b"old"),
        )
        .unwrap();
        s.install_remote_snapshot(&[root, file.clone()], "new", "server")
            .unwrap();
    }
    let token = drive_fs::ReadCancellation::default();
    let owner = start(&bridge, &file, &token);
    let first = request(&mut peer);
    fail(&mut peer, first.id, "transport");
    backoff(&bridge);
    token.cancel();
    assert_eq!(
        bridge
            .store
            .lock()
            .unwrap()
            .open_cached("file")
            .unwrap()
            .version(),
        "v0"
    );
    let second = request(&mut peer);
    assert!(matches!(second.body, Command::Download { .. }));
    content(&mut peer, second.id);
    assert_eq!(owner.join().unwrap(), Err(libc::EINTR));
    assert_eq!(
        bridge
            .store
            .lock()
            .unwrap()
            .open_cached("file")
            .unwrap()
            .version(),
        "v1"
    );
}

#[test]
fn preexisting_durable_retry_is_not_owned_by_a_new_cancellable_reader() {
    let (_dir, bridge, file, mut peer) = fixture(false);
    bridge
        .store
        .lock()
        .unwrap()
        .fail_download(
            &DownloadTicket {
                id: "file".into(),
                version: "v1".into(),
            },
            DownloadFailureKind::Transport,
            None,
            unix_seconds().saturating_sub(10),
        )
        .unwrap();
    let token = drive_fs::ReadCancellation::default();
    let owner = start(&bridge, &file, &token);
    let active = request(&mut peer);
    token.cancel();
    std::thread::sleep(Duration::from_millis(80));
    content(&mut peer, active.id);
    assert_eq!(owner.join().unwrap(), Err(libc::EINTR));
    assert!(bridge.store.lock().unwrap().open_cached("file").is_ok());
    assert!(
        bridge
            .store
            .lock()
            .unwrap()
            .download_failure("file")
            .unwrap()
            .is_none()
    );
}

#[test]
fn cancelled_follower_cannot_report_success_when_shared_body_finishes() {
    let (_dir, bridge, file, mut peer) = fixture(false);
    let owner_token = drive_fs::ReadCancellation::default();
    let owner = start(&bridge, &file, &owner_token);
    let active = request(&mut peer);
    let token = drive_fs::ReadCancellation::default();
    let waiter = start(&bridge, &file, &token);
    readers(&bridge, 2);
    token.cancel();
    content(&mut peer, active.id);
    assert_eq!(waiter.join().unwrap(), Err(libc::EINTR));
    owner.join().unwrap().unwrap();
    assert!(bridge.store.lock().unwrap().open_cached("file").is_ok());
}

#[test]
fn foreground_joining_background_flight_waits_for_retry() {
    let (_dir, bridge, file, mut peer) = fixture(true);
    let b = bridge.clone();
    let background = std::thread::spawn(move || b.refresh_cached());
    let active = request(&mut peer);
    let token = drive_fs::ReadCancellation::default();
    let waiter = start(&bridge, &file, &token);
    readers(&bridge, 1);
    fail(&mut peer, active.id, "transport");
    let second = request(&mut peer);
    assert!(matches!(second.body, Command::Download { .. }));
    content(&mut peer, second.id);
    waiter.join().unwrap().unwrap();
    background.join().unwrap();
    assert_eq!(bridge.downloads.load(Ordering::Relaxed), 2);
}

#[test]
#[ignore = "requires ordinary Linux user and /dev/fuse; local IPC fault injection only"]
fn kernel_cancel_in_backoff_clears_retry_and_later_read_gets_complete_body() {
    struct Child(std::process::Child);
    impl Drop for Child {
        fn drop(&mut self) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
    let uid = unsafe { libc::getuid() };
    assert_ne!(uid, 0);
    let (dir, bridge, mut file, mut peer) = fixture(false);
    file.metadata = Some(drive_core::NodeMetadata {
        size: 3,
        mtime: 1,
        mode: 0o644,
    });
    {
        let mut s = bridge.store.lock().unwrap();
        let root = s.node("root").unwrap();
        s.install_remote_snapshot(&[root, file], "2", "server")
            .unwrap();
    }
    let mount = dir.path().join("mount");
    std::fs::create_dir(&mount).unwrap();
    let fs = drive_fs::CacheFilesystem::new(bridge.store.clone(), "root", uid, unsafe {
        libc::getgid()
    })
    .unwrap()
    .with_hydrator(bridge.clone());
    let session = fuser::spawn_mount2(
        fs,
        &mount,
        &[
            fuser::MountOption::RO,
            fuser::MountOption::DefaultPermissions,
            fuser::MountOption::NoDev,
            fuser::MountOption::NoSuid,
        ],
    )
    .unwrap();
    let path = mount.join("A.doc");
    for round in 1..=8 {
        let mut reader = Child(
            Process::new("cat")
                .arg(&path)
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .unwrap(),
        );
        let first = request(&mut peer);
        assert!(matches!(first.body, Command::Download { .. }));
        fail(&mut peer, first.id, "native_server_code:32772");
        backoff(&bridge);
        assert_eq!(bridge.active_downloads(), vec!["file"]);
        let started = Instant::now();
        assert_eq!(
            unsafe { libc::kill(reader.0.id() as i32, libc::SIGTERM) },
            0
        );
        until(|| reader.0.try_wait().unwrap().is_some());
        until(|| bridge.active_downloads().is_empty());
        assert!(started.elapsed() < Duration::from_secs(2));
        bridge.refresh_cached();
        assert_eq!(bridge.downloads.load(Ordering::Relaxed), round);
        let s = bridge.store.lock().unwrap();
        assert!(s.downloads_needed().unwrap().is_empty());
        assert_eq!(s.next_download_retry().unwrap(), None);
        assert!(s.open_cached("file").is_err());
        drop(s);
        assert_eq!(std::fs::metadata(&path).unwrap().len(), 3);
    }
    let mut reader = Child(
        Process::new("cat")
            .arg(&path)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .unwrap(),
    );
    let fresh = request(&mut peer);
    assert!(matches!(fresh.body, Command::Download { .. }));
    content(&mut peer, fresh.id);
    let mut bytes = Vec::new();
    std::io::Read::read_to_end(&mut reader.0.stdout.take().unwrap(), &mut bytes).unwrap();
    assert!(reader.0.wait().unwrap().success());
    assert_eq!(bytes, b"new");
    assert_eq!(bridge.downloads.load(Ordering::Relaxed), 9);
    session.join();
}
