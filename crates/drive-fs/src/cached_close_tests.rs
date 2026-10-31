//! Kernel dependency regressions with synthetic content; no network/backend.
use super::*;
use drive_core::{DownloadTicket, NodeMetadata, sha256};
use std::{
    io::Write,
    os::unix::fs::FileExt,
    sync::{
        atomic::{AtomicUsize, Ordering},
        mpsc,
    },
    time::Instant,
};

struct Saved(AtomicUsize);
impl Hydrator for Saved {
    fn local_saved(&self) {
        self.0.fetch_add(1, Ordering::Relaxed);
    }
    fn hydrate(&self, _: &Node) -> Result<Arc<ReadHandle>, i32> {
        Err(libc::ENETDOWN)
    }
}
fn fixture() -> (tempfile::TempDir, Arc<drive_core::SharedStore>, CacheFilesystem) {
    let uid = unsafe { libc::getuid() };
    assert_ne!(uid, 0);
    let temp = tempfile::tempdir().unwrap();
    let mut store = Store::open(temp.path().join("state")).unwrap();
    let nodes: Vec<_> = ["root", "cached", "working"]
        .into_iter()
        .map(|id| Node {
            id: id.into(),
            parent: (id != "root").then(|| "root".into()),
            name: id.into(),
            directory: id == "root",
            version: "1".into(),
            metadata: Some(NodeMetadata {
                size: if id == "root" { 0 } else { 1024 * 1024 },
                mtime: 1,
                mode: 0o755,
            }),
        })
        .collect();
    store
        .install_remote_snapshot(&nodes, "1", "synthetic")
        .unwrap();
    let body = vec![19; 1024 * 1024];
    for id in ["cached", "working"] {
        store
            .publish_download(
                &DownloadTicket {
                    id: id.into(),
                    version: "1".into(),
                },
                &body,
                &sha256(&body),
            )
            .unwrap();
    }
    let store = Arc::new(drive_core::SharedStore::new(store));
    let fs = CacheFilesystem::new(store.clone(), "root", uid, unsafe { libc::getgid() })
        .unwrap()
        .with_writes_enabled();
    (temp, store, fs)
}
fn mount(fs: CacheFilesystem, path: &std::path::Path) -> fuser::BackgroundSession {
    std::fs::create_dir(path).unwrap();
    fuser::spawn_mount2(
        fs,
        path,
        &[
            fuser::MountOption::DefaultPermissions,
            fuser::MountOption::NoDev,
            fuser::MountOption::NoSuid,
            fuser::MountOption::FSName("drive-native-cached-close-test".into()),
        ],
    )
    .unwrap()
}
fn block_workers(jobs: &SyncSender<Job>, store: &Arc<drive_core::SharedStore>) -> bool {
    let (started, ready) = mpsc::channel();
    for _ in 0..4 {
        let store = store.clone();
        let started = started.clone();
        jobs.send(Box::new(move || {
            let _ = started.send(());
            drop(store.lock().unwrap());
        }))
        .unwrap();
    }
    (0..4).all(|_| ready.recv_timeout(Duration::from_secs(3)).is_ok())
}

#[test]
#[ignore = "requires ordinary Linux user and /dev/fuse; run explicitly"]
fn cached_readonly_sync_and_close_ignore_metadata_backlog() {
    let (temp, store, fs) = fixture();
    let jobs = fs.queue.sender.as_ref().unwrap().clone();
    let handles = fs.handles.clone();
    let saved = Arc::new(Saved(AtomicUsize::new(0)));
    let path = temp.path().join("mount");
    let session = mount(fs.with_hydrator(saved.clone()), &path);
    let file = std::fs::File::open(path.join("cached")).unwrap();
    let duplicate = file.try_clone().unwrap();
    let guard = store.lock().unwrap();
    let blocked = block_workers(&jobs, &store);
    let (tx, rx) = mpsc::channel();
    let reader = std::thread::spawn(move || {
        let begin = Instant::now();
        let mut bytes = vec![0; 1024 * 1024];
        let read = file.read_at(&mut bytes, 0).is_ok_and(|n| n == bytes.len())
            && bytes.iter().all(|b| *b == 19);
        let synced = file.sync_all().is_ok();
        // dup close flushes the same kernel handle before its final release.
        drop(duplicate);
        let reread =
            file.read_at(&mut bytes[..4], 0).is_ok_and(|n| n == 4) && bytes[..4] == [19; 4];
        drop(file);
        let _ = tx.send((read && synced && reread, begin.elapsed()));
    });
    let before_unlock = rx.recv_timeout(Duration::from_secs(2));
    let deadline = Instant::now() + Duration::from_secs(2);
    while before_unlock.is_ok()
        && !handles.lock().unwrap().entries.is_empty()
        && Instant::now() < deadline
    {
        std::thread::sleep(Duration::from_millis(5));
    }
    let released = handles.lock().unwrap().entries.is_empty();
    let wakes = saved.0.load(Ordering::Relaxed);
    // Always lift the barrier before joining/unmounting, including regressions.
    drop(guard);
    reader.join().unwrap();
    drop(jobs);
    session.join();
    assert!(
        blocked,
        "not all metadata workers reached the Store barrier"
    );
    let (correct, elapsed) =
        before_unlock.expect("read-only cached read/fsync/close waited for metadata");
    assert!(correct);
    assert!(released, "release retained a closed handle");
    assert_eq!(wakes, 0, "pure read-only synchronization woke uploads");
    eprintln!("cached read + fsync + duplicate/final close under Store barrier: {elapsed:?}");
}

#[test]
#[ignore = "requires ordinary Linux user and /dev/fuse; run explicitly"]
fn draft_and_writable_fsync_still_wait_for_metadata_checkpoint() {
    let (temp, store, fs) = fixture();
    let jobs = fs.queue.sender.as_ref().unwrap().clone();
    let path = temp.path().join("mount");
    let session = mount(fs, &path);
    let joined = std::fs::File::open(path.join("working")).unwrap();
    let mut writer = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(path.join("working"))
        .unwrap();
    writer.write_all(b"new").unwrap(); // promotes the pre-existing read-only handle too
    let writable_cached = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(path.join("cached"))
        .unwrap();
    assert_eq!(store.lock().unwrap().pending_operations().unwrap(), 0);
    let guard = store.lock().unwrap();
    let blocked = block_workers(&jobs, &store);
    let (started, ready) = mpsc::channel();
    let (tx, rx) = mpsc::channel();
    let threads: Vec<_> = [writer, joined, writable_cached]
        .into_iter()
        .map(|file| {
            let started = started.clone();
            let tx = tx.clone();
            std::thread::spawn(move || {
                let _ = started.send(());
                let ok = file.sync_all().is_ok();
                let _ = tx.send(ok);
                drop(file);
            })
        })
        .collect();
    let issued = (0..3).all(|_| ready.recv_timeout(Duration::from_secs(2)).is_ok());
    let premature = rx.recv_timeout(Duration::from_millis(150)).ok();
    drop(guard);
    let mut completed = usize::from(premature.is_some());
    let mut correct = premature.unwrap_or(true);
    for _ in completed..3 {
        match rx.recv_timeout(Duration::from_secs(3)) {
            Ok(ok) => {
                completed += 1;
                correct &= ok;
            }
            Err(_) => break,
        }
    }
    for thread in threads {
        thread.join().unwrap();
    }
    drop(jobs);
    session.join();
    assert!(blocked && issued);
    assert!(
        premature.is_none(),
        "writable or promoted draft fsync bypassed checkpoint queue"
    );
    assert_eq!(completed, 3);
    assert!(correct);
    assert!(store.lock().unwrap().pending_operations().unwrap() > 0);
    // A fresh Store must see the checkpointed bytes, not only an open draft.
    drop(store);
    let store = Store::open(temp.path().join("state")).unwrap();
    let mut bytes = [0; 3];
    store
        .open_cached("working")
        .unwrap()
        .read_at(&mut bytes, 0)
        .unwrap();
    assert_eq!(&bytes, b"new");
}

#[test]
#[ignore = "requires ordinary Linux user and /dev/fuse; run explicitly"]
fn failed_draft_release_reports_error_and_survives_gc_and_reopen() {
    use std::os::{fd::IntoRawFd, unix::fs::PermissionsExt};
    for reject_queue in [true, false] {
        let (temp, store, fs) = fixture();
        let jobs = fs.queue.sender.as_ref().unwrap().clone();
        let handles = fs.handles.clone();
        let saved = Arc::new(Saved(AtomicUsize::new(0)));
        let mount_path = temp.path().join("mount");
        let session = mount(fs.with_hydrator(saved.clone()), &mount_path);
        let mut file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(mount_path.join("working"))
            .unwrap();
        let edit = b"uncheckpointed local modification";
        file.write_all(edit).unwrap();
        let mut expected = vec![19; 1024 * 1024];
        expected[..edit.len()].copy_from_slice(edit);
        let weak = {
            let handles = handles.lock().unwrap();
            match &handles.entries.values().next().unwrap().content {
                OpenContent::Draft(draft) => Arc::downgrade(draft),
                _ => panic!("write did not create a draft"),
            }
        };
        let token = store
            .lock()
            .unwrap()
            .open_dirty_draft("working")
            .unwrap()
            .unwrap()
            .token
            .clone();
        let draft_path = temp.path().join("state/drafts").join(token);
        let blobs = temp.path().join("state/blobs");
        let original_permissions = std::fs::metadata(&blobs).unwrap().permissions();
        let guard = reject_queue.then(|| store.lock().unwrap());
        let blocked = if reject_queue {
            let blocked = block_workers(&jobs, &store);
            for _ in 0..256 {
                jobs.try_send(Box::new(|| {})).unwrap();
            }
            blocked
        } else {
            // Fail immutable snapshot publication as the ordinary test user.
            // The draft and SQLite directories remain writable throughout.
            std::fs::set_permissions(&blobs, std::fs::Permissions::from_mode(0o500)).unwrap();
            true
        };
        let (tx, rx) = mpsc::channel();
        let closer = std::thread::spawn(move || {
            let fd = file.into_raw_fd();
            // Linux closes the descriptor even when flush returns an error;
            // retrying close could close a reused descriptor.
            let result = unsafe { libc::close(fd) };
            let error = (result != 0).then(|| std::io::Error::last_os_error().raw_os_error());
            let _ = tx.send(error.flatten());
        });
        let result = rx.recv_timeout(Duration::from_secs(2));
        let deadline = Instant::now() + Duration::from_secs(2);
        while (weak.upgrade().is_some() || !handles.lock().unwrap().entries.is_empty())
            && Instant::now() < deadline
        {
            std::thread::sleep(Duration::from_millis(5));
        }
        let released = weak.upgrade().is_none() && handles.lock().unwrap().entries.is_empty();
        drop(guard);
        closer.join().unwrap();
        // A failing release must finish before lifting the publication fault.
        std::fs::set_permissions(&blobs, original_permissions).unwrap();
        drop(jobs);
        session.join();
        assert!(blocked && released, "failed release retained a live handle");
        let error = result.expect("failed close blocked");
        // tempfile wraps the publication permission error without a raw errno;
        // the adapter reports EIO, just as it does for a rejected queued reply.
        assert_eq!(error, Some(libc::EIO));
        assert_eq!(
            saved.0.load(Ordering::Relaxed),
            0,
            "failed checkpoint was acknowledged as saved"
        );
        {
            let mut s = store.lock().unwrap();
            assert!(s.dirty_draft("working").unwrap());
            assert_eq!(s.pending_operations().unwrap(), 0);
            assert_eq!(
                s.status("working").unwrap(),
                drive_core::Status::PendingUpload
            );
            s.collect_garbage().unwrap();
            assert!(draft_path.is_file(), "GC deleted the uncheckpointed draft");
            let draft = s.open_dirty_draft("working").unwrap().unwrap();
            let mut actual = vec![0; expected.len()];
            assert_eq!(draft.read_at(&mut actual, 0).unwrap(), actual.len());
            assert_eq!(sha256(&actual), sha256(&expected));
        }
        drop(store);
        let mut reopened = Store::open(temp.path().join("state")).unwrap();
        assert!(reopened.dirty_draft("working").unwrap());
        assert_eq!(reopened.checkpoint_recovered_drafts().unwrap(), 1);
        let mut actual = vec![0; expected.len()];
        assert_eq!(
            reopened
                .open_cached("working")
                .unwrap()
                .read_at(&mut actual, 0)
                .unwrap(),
            actual.len()
        );
        assert_eq!(sha256(&actual), sha256(&expected));
        assert_eq!(reopened.pending_operations().unwrap(), 1);
        assert_eq!(
            reopened.status("working").unwrap(),
            drive_core::Status::PendingUpload
        );
    }
}
