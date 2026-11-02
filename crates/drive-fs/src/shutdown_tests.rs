//! Explicit kernel shutdown regressions; synthetic local Store, no NAS.
//! Failure keeps the fixture on disk. Descriptor release happens in a bounded
//! cleanup observer, never implicitly on the assertion/unwind thread.
use super::cached_close_tests::{fixture, mount};
use super::*;
use std::{
    fs::{File, OpenOptions},
    io::{Read, Write},
    mem::ManuallyDrop,
    sync::mpsc,
    time::Instant,
};
const LIMIT: Duration = Duration::from_secs(5);
fn cleanup(temp: ManuallyDrop<tempfile::TempDir>) {
    let mount = temp.path().join("mount");
    let escaped = mount
        .to_string_lossy()
        .replace('\\', "\\134")
        .replace(' ', "\\040")
        .replace('\t', "\\011")
        .replace('\n', "\\012");
    let mounts = std::fs::read_to_string("/proc/self/mountinfo")
        .expect("fixture kept: mount state unavailable");
    assert!(
        !mounts
            .lines()
            .any(|line| line.split(' ').nth(4) == Some(escaped.as_str())),
        "fixture kept: mount is still attached"
    );
    drop(ManuallyDrop::into_inner(temp));
}

struct Held {
    close: mpsc::Sender<()>,
    closed: mpsc::Receiver<()>,
}
impl Held {
    fn new(files: Vec<File>, notifier: Option<fuser::Notifier>) -> Self {
        let (close, command) = mpsc::channel();
        let (done, closed) = mpsc::channel();
        std::thread::spawn(move || {
            let _ = command.recv();
            // Closing a kernel FD may send FLUSH. Do not keep an otherwise-last
            // /dev/fuse notifier alive while waiting for that FLUSH to abort.
            drop(notifier);
            drop(files);
            let _ = done.send(());
        });
        Self { close, closed }
    }
    fn release(&self) -> bool {
        let _ = self.close.send(());
        self.closed.recv_timeout(LIMIT).is_ok()
    }
}
impl Drop for Held {
    fn drop(&mut self) {
        let _ = self.close.send(());
    }
}
fn stop(session: fuser::BackgroundSession) -> mpsc::Receiver<std::io::Result<()>> {
    let (done, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let result = session.join_result();
        let _ = done.send(result);
    });
    rx
}
fn finish(rx: mpsc::Receiver<std::io::Result<()>>, held: &Held) -> bool {
    let first = rx.recv_timeout(LIMIT);
    let on_time = matches!(&first, Ok(Ok(())));
    let released = held.release();
    // A baseline without a wake FD is allowed to finish naturally once its
    // own held descriptors close. This is cleanup, not a successful result.
    let ended = if first.is_err() {
        matches!(rx.recv_timeout(LIMIT), Ok(Ok(())))
    } else {
        on_time
    };
    assert!(
        released && ended,
        "shutdown fixture retained: normal descriptor cleanup incomplete"
    );
    on_time
}

struct SavedShutdown(std::sync::atomic::AtomicUsize);
impl Hydrator for SavedShutdown {
    fn local_saved(&self) {
        self.0.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    }
    fn hydrate(&self, _: &Node) -> Result<Arc<ReadHandle>, i32> {
        Err(libc::ENETDOWN)
    }
}
#[test]
#[ignore = "requires an ordinary Linux user and /dev/fuse"]
fn held_file_and_directory_do_not_hold_session_join_and_dirty_bytes_survive() {
    let (temp, store, fs) = fixture();
    let temp = ManuallyDrop::new(temp);
    let saved = Arc::new(SavedShutdown(std::sync::atomic::AtomicUsize::new(0)));
    let path = temp.path().join("mount");
    let session = mount(fs.with_hydrator(saved.clone()), &path);
    let notifier = session.notifier();
    let directory = File::open(&path).unwrap();
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(path.join("working"))
        .unwrap();
    let body = b"accepted write before stop; application still holds fd";
    file.write_all(body).unwrap();
    let before_pending = store.lock().unwrap().pending_operations().unwrap();
    let held = Held::new(vec![file, directory], Some(notifier));
    let stopped = stop(session);
    let first = stopped.recv_timeout(LIMIT);
    let on_time = matches!(&first, Ok(Ok(())));
    // Observe/save while application descriptors are STILL HELD, matching FSD.
    // The kernel may itself emit fsync during normal unmount; then the final
    // local barrier legitimately has no additional checkpoint to create.
    let evidence = if on_time {
        let saved_before = saved.0.load(std::sync::atomic::Ordering::SeqCst);
        let mut s = store.lock().unwrap();
        let checkpoints = s.checkpoint_recovered_drafts().unwrap();
        Some((saved_before, checkpoints, s.pending_operations().unwrap()))
    } else {
        None
    };
    let released = held.release();
    let ended = if first.is_err() {
        matches!(stopped.recv_timeout(LIMIT), Ok(Ok(())))
    } else {
        on_time
    };
    assert!(
        released && ended,
        "fixture retained: normal descriptor cleanup incomplete"
    );
    assert!(
        on_time,
        "session depended on descriptor closure; fixture retained"
    );
    assert_eq!(
        before_pending, 0,
        "write fixture was already checkpointed before stop"
    );
    let (saved_before, checkpoints, pending) = evidence.unwrap();
    eprintln!(
        "shutdown durability: kernel_saved_callbacks={saved_before} final_checkpoints={checkpoints} pending={pending}"
    );
    assert!(
        checkpoints == 1 || (checkpoints == 0 && saved_before > 0),
        "no local checkpoint was observed"
    );
    assert_eq!(pending, 1);
    drop(store);
    let reopened = Store::open(temp.path().join("state")).unwrap();
    let draft = reopened.open_dirty_draft("working").unwrap().unwrap();
    let mut data = vec![0; body.len()];
    draft.read_at(&mut data, 0).unwrap();
    assert_eq!(data, body);
    assert_eq!(reopened.pending_operations().unwrap(), 1);
    drop(draft);
    drop(reopened);
    cleanup(temp);
}

#[test]
#[ignore = "requires an ordinary Linux user and /dev/fuse"]
fn stop_drains_accepted_local_jobs_and_preserves_successful_fsync() {
    let (temp, store, fs) = fixture();
    let temp = ManuallyDrop::new(temp);
    let jobs = fs.queue.sender.as_ref().unwrap().clone();
    let path = temp.path().join("mount");
    let session = mount(fs, &path);
    let directory = File::open(&path).unwrap();
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(path.join("working"))
        .unwrap();
    file.write_all(b"durable before stop").unwrap();
    file.sync_all().unwrap();
    let held = Held::new(vec![file, directory], None);
    let (accepted, ready) = mpsc::channel();
    let (release, wait) = mpsc::channel();
    jobs.send(Box::new(move || {
        let _ = accepted.send(());
        let _ = wait.recv_timeout(Duration::from_secs(10));
    }))
    .unwrap();
    ready.recv_timeout(LIMIT).unwrap();
    drop(jobs);
    let stopped = stop(session);
    let premature = stopped.recv_timeout(Duration::from_millis(200));
    let _ = release.send(());
    if premature.is_ok() {
        let _ = held.release();
        panic!("stop bypassed accepted local work; fixture retained");
    }
    let on_time = finish(stopped, &held);
    assert_eq!(
        store.lock().unwrap().checkpoint_recovered_drafts().unwrap(),
        0
    );
    drop(store);
    let reopened = Store::open(temp.path().join("state")).unwrap();
    assert_eq!(reopened.pending_operations().unwrap(), 1);
    let draft = reopened.open_dirty_draft("working").unwrap().unwrap();
    let mut data = [0; b"durable before stop".len()];
    draft.read_at(&mut data, 0).unwrap();
    assert_eq!(&data, b"durable before stop");
    drop(draft);
    drop(reopened);
    assert!(
        on_time,
        "session required descriptor release; fixture retained"
    );
    cleanup(temp);
}

struct UntilCancelled(mpsc::Sender<()>);
impl Hydrator for UntilCancelled {
    fn hydrate(&self, _: &Node) -> Result<Arc<ReadHandle>, i32> {
        unreachable!()
    }
    fn hydrate_cancellable(
        &self,
        _: &Node,
        cancel: &ReadCancellation,
    ) -> Result<Arc<ReadHandle>, i32> {
        let _ = self.0.send(());
        let deadline = Instant::now() + Duration::from_secs(10);
        while !cancel.is_cancelled() && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(5));
        }
        if cancel.is_cancelled() {
            Err(libc::EINTR)
        } else {
            Err(libc::ETIMEDOUT)
        }
    }
}
#[test]
#[ignore = "requires an ordinary Linux user and /dev/fuse"]
fn stop_cancels_unfinished_network_read_without_waiting_for_download_timeout() {
    let (temp, store, fs) = fixture();
    let temp = ManuallyDrop::new(temp);
    store.lock().unwrap().release("working").unwrap();
    let (began, rx) = mpsc::channel();
    let path = temp.path().join("mount");
    let session = mount(fs.with_hydrator(Arc::new(UntilCancelled(began))), &path);
    let mut file = File::open(path.join("working")).unwrap();
    let (read_done, read_result) = mpsc::channel();
    std::thread::spawn(move || {
        let mut bytes = [0; 4];
        let result = file.read(&mut bytes);
        drop(file);
        let _ = read_done.send(result);
    });
    rx.recv_timeout(LIMIT).unwrap();
    let stopped = stop(session);
    let first = stopped.recv_timeout(LIMIT);
    let on_time = matches!(&first, Ok(Ok(())));
    // If cancellation regresses, the synthetic downloader's own bounded
    // deadline releases its read. Await cleanup, but still fail the timing gate.
    assert!(
        matches!(
            read_result.recv_timeout(Duration::from_secs(12)),
            Ok(Err(_))
        ),
        "read/FD cleanup did not complete; fixture retained"
    );
    if first.is_err() {
        assert!(
            matches!(stopped.recv_timeout(LIMIT), Ok(Ok(()))),
            "session cleanup incomplete; fixture retained"
        );
    }
    assert_eq!(store.lock().unwrap().pending_operations().unwrap(), 0);
    drop(store);
    assert!(
        on_time,
        "stop waited for download timeout; fixture retained"
    );
    cleanup(temp);
}

#[test]
fn private_wake_stops_idle_session_even_with_live_sender() {
    use std::{
        os::unix::net::UnixStream,
        sync::atomic::{AtomicUsize, Ordering},
    };
    struct Destroyed(Arc<AtomicUsize>);
    impl fuser::Filesystem for Destroyed {
        fn destroy(&mut self) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }
    }
    let (device, peer) = UnixStream::pair().unwrap();
    let destroyed = Arc::new(AtomicUsize::new(0));
    let session = fuser::Session::from_fd(
        Destroyed(destroyed.clone()),
        device.into(),
        fuser::SessionACL::Owner,
    )
    .spawn()
    .unwrap();
    let notifier = session.notifier();
    let stopped = stop(session);
    let first = stopped.recv_timeout(LIMIT);
    let on_time = matches!(&first, Ok(Ok(())));
    drop(peer);
    drop(notifier); // EOF is the controlled legacy wake, not a signal.
    if first.is_err() {
        assert!(matches!(stopped.recv_timeout(LIMIT), Ok(Ok(()))));
    }
    assert_eq!(destroyed.load(Ordering::SeqCst), 1);
    assert!(on_time, "idle session needed peer closure");
}
