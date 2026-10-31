//! Stable directory cookies. Only opendir projects SQLite; continuation and
//! release use local immutable views and never wait for Store or network work.
use super::*;
use drive_core::filesystem::DirectorySnapshot;
use std::sync::{
    Weak,
    atomic::{AtomicUsize, Ordering},
};

const MAX_BYTES: usize = 64 * 1024 * 1024;
const MAX_HANDLES: usize = 4096;

pub(super) struct View {
    ino: u64,
    snapshot: Option<DirectorySnapshot>,
    reclaimer: Arc<Reclaimer>,
}
impl View {
    fn snapshot(&self) -> &DirectorySnapshot {
        self.snapshot.as_ref().unwrap()
    }
}
impl Drop for View {
    fn drop(&mut self) {
        self.reclaimer.release(self.snapshot.take().unwrap());
    }
}
struct Reclaimer {
    sender: Option<SyncSender<DirectorySnapshot>>,
    thread: Option<std::thread::JoinHandle<()>>,
    charged: Arc<AtomicUsize>,
    allocated: Arc<AtomicUsize>,
}
fn reclaim(snapshot: DirectorySnapshot, charged: &AtomicUsize, allocated: &AtomicUsize) {
    let bytes = snapshot.bytes;
    drop(snapshot);
    // Return capacity only after the allocations are actually released.
    charged.fetch_sub(bytes, Ordering::AcqRel);
    allocated.fetch_sub(1, Ordering::AcqRel);
}
impl Reclaimer {
    fn new(charged: Arc<AtomicUsize>, allocated: Arc<AtomicUsize>) -> Self {
        let (sender, receiver) = sync_channel(MAX_HANDLES);
        let bytes = charged.clone();
        let views = allocated.clone();
        let thread = std::thread::spawn(move || {
            while let Ok(snapshot) = receiver.recv() {
                reclaim(snapshot, &bytes, &views);
            }
        });
        Self {
            sender: Some(sender),
            thread: Some(thread),
            charged,
            allocated,
        }
    }
    fn release(&self, snapshot: DirectorySnapshot) {
        // Each allocated view sends exactly once. Allocated (including queued)
        // views are capped at channel capacity, so a normal send cannot be full.
        // On exceptional receiver failure still release memory, never leak it.
        if let Err(error) = self.sender.as_ref().unwrap().try_send(snapshot) {
            let snapshot = match error {
                std::sync::mpsc::TrySendError::Full(snapshot)
                | std::sync::mpsc::TrySendError::Disconnected(snapshot) => snapshot,
            };
            reclaim(snapshot, &self.charged, &self.allocated);
        }
    }
}
impl Drop for Reclaimer {
    fn drop(&mut self) {
        self.sender.take();
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}
struct Table {
    next: u64,
    handles: HashMap<u64, Arc<View>>,
}
pub(super) struct Views {
    table: Mutex<Table>,
    // Conservative key: local_revision can invalidate more than one directory,
    // but never exposes a new open to a known old namespace. Held views survive.
    cached: Mutex<HashMap<u64, (u64, Weak<View>)>>,
    builder: Mutex<()>,
    charged: Arc<AtomicUsize>,
    allocated: Arc<AtomicUsize>,
    reclaimer: Arc<Reclaimer>,
}
impl Default for Views {
    fn default() -> Self {
        let charged = Arc::new(AtomicUsize::new(0));
        let allocated = Arc::new(AtomicUsize::new(0));
        let reclaimer = Arc::new(Reclaimer::new(charged.clone(), allocated.clone()));
        Self {
            table: Mutex::new(Table {
                next: 1,
                handles: HashMap::new(),
            }),
            cached: Mutex::new(HashMap::new()),
            builder: Mutex::new(()),
            charged,
            allocated,
            reclaimer,
        }
    }
}
impl Views {
    fn open(&self, store: &drive_core::SharedStore, ino: u64) -> Result<u64, i32> {
        let _building = self.builder.lock().unwrap();
        {
            let table = self.table.lock().unwrap();
            if table.handles.len() >= MAX_HANDLES || table.next == u64::MAX {
                return Err(libc::EMFILE);
            }
        }
        let revision = store.lock().unwrap().local_revision();
        let cached = self
            .cached
            .lock()
            .unwrap()
            .get(&ino)
            .and_then(|(rev, view)| (*rev == revision).then(|| view.upgrade()).flatten());
        let view = match cached {
            Some(view) => view,
            None => {
                // The builder lock reserves the remaining slot/bytes across
                // projection: no other builder can consume them. Failed builds
                // roll back without publishing either a view or its charge.
                if self.allocated.load(Ordering::Acquire) >= MAX_HANDLES {
                    return Err(libc::ENOMEM);
                }
                let remaining = MAX_BYTES.saturating_sub(self.charged.load(Ordering::Acquire));
                let (snapshot, revision) = {
                    let store = store.lock().unwrap();
                    let snapshot = store
                        .directory_snapshot(ino, remaining)
                        .map_err(|e| errno(&e))?;
                    (snapshot, store.local_revision())
                };
                self.charged.fetch_add(snapshot.bytes, Ordering::AcqRel);
                self.allocated.fetch_add(1, Ordering::AcqRel);
                let view = Arc::new(View {
                    ino,
                    snapshot: Some(snapshot),
                    reclaimer: self.reclaimer.clone(),
                });
                let mut cache = self.cached.lock().unwrap();
                cache.retain(|_, (_, view)| view.strong_count() != 0);
                cache.insert(ino, (revision, Arc::downgrade(&view)));
                view
            }
        };
        let mut table = self.table.lock().unwrap();
        let fh = table.next;
        table.next += 1;
        table.handles.insert(fh, view);
        Ok(fh)
    }
    fn get(&self, ino: u64, fh: u64) -> Result<Arc<View>, i32> {
        self.table
            .lock()
            .unwrap()
            .handles
            .get(&fh)
            .filter(|v| v.ino == ino)
            .cloned()
            .ok_or(libc::EBADF)
    }
    fn release(&self, ino: u64, fh: u64) -> Result<(), i32> {
        let removed = {
            let mut table = self.table.lock().unwrap();
            if !table.handles.get(&fh).is_some_and(|v| v.ino == ino) {
                return Err(libc::EBADF);
            }
            table.handles.remove(&fh)
        };
        drop(removed);
        Ok(())
    }
}

pub(super) fn open(fs: &mut CacheFilesystem, ino: u64, flags: i32, reply: ReplyOpen) {
    if flags & libc::O_ACCMODE != libc::O_RDONLY {
        reply.error(libc::EISDIR);
        return;
    }
    let views = fs.directories.clone();
    let store = fs.store.clone();
    fs.queue
        .submit(Box::new(move || match views.open(&store, ino) {
            Ok(fh) => reply.opened(fh, 0),
            Err(e) => reply.error(e),
        }));
}
pub(super) fn read(
    fs: &mut CacheFilesystem,
    ino: u64,
    fh: u64,
    offset: i64,
    mut reply: ReplyDirectory,
) {
    let Ok(offset) = usize::try_from(offset) else {
        reply.error(libc::EINVAL);
        return;
    };
    let views = fs.directories.clone();
    fs.read_queue
        .submit(Box::new(move || match views.get(ino, fh) {
            Ok(view) => {
                if offset > view.snapshot().entries.len() {
                    reply.error(libc::EINVAL);
                    return;
                }
                for (index, entry) in view.snapshot().entries.iter().enumerate().skip(offset) {
                    let kind = if entry.directory {
                        FileType::Directory
                    } else {
                        FileType::RegularFile
                    };
                    if reply.add(entry.ino, (index + 1) as i64, kind, &entry.name) {
                        break;
                    }
                }
                reply.ok();
            }
            Err(e) => reply.error(e),
        }));
}
pub(super) fn release(fs: &mut CacheFilesystem, ino: u64, fh: u64, reply: ReplyEmpty) {
    // Kernel close is not retried if a bounded work queue rejects its job.
    // Remove the handle directly so overload cannot permanently leak a view.
    // The final reference transfers the snapshot to its capacity-reserved
    // reclaimer; large deallocation does not run on the FUSE dispatcher.
    match fs.directories.release(ino, fh) {
        Ok(()) => reply.ok(),
        Err(e) => reply.error(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn shared_views_keep_old_namespace_and_release_their_budget() {
        let temp = tempfile::tempdir().unwrap();
        let mut store = Store::open(temp.path()).unwrap();
        let node = |id: &str| Node {
            id: id.into(),
            name: id.into(),
            parent: (id != "root").then(|| "root".into()),
            directory: id == "root",
            version: "1".into(),
            metadata: None,
        };
        store
            .install_remote_snapshot(&[node("root"), node("a")], "1", "s")
            .unwrap();
        store.ensure_fs_root("root").unwrap();
        let store = drive_core::SharedStore::new(store);
        let views = Views::default();
        let first = views.open(&store, 1).unwrap();
        let second = views.open(&store, 1).unwrap();
        let a = views.get(1, first).unwrap();
        let b = views.get(1, second).unwrap();
        assert!(Arc::ptr_eq(&a, &b));
        assert_eq!(views.charged.load(Ordering::Acquire), a.snapshot().bytes);
        store
            .lock()
            .unwrap()
            .create_local_directory("root", "added", 0o755)
            .unwrap();
        let next = views.open(&store, 1).unwrap();
        let c = views.get(1, next).unwrap();
        assert_eq!(a.snapshot().entries.len(), 3);
        assert_eq!(c.snapshot().entries.len(), 4);
        assert!(!Arc::ptr_eq(&a, &c));
        assert!(matches!(views.get(999, first), Err(libc::EBADF)));
        assert_eq!(views.release(999, first), Err(libc::EBADF));
        views.release(1, first).unwrap();
        views.release(1, second).unwrap();
        views.release(1, next).unwrap();
        assert!(views.charged.load(Ordering::Acquire) > 0); // in-flight readers still own views
        drop((a, b, c));
        let deadline = std::time::Instant::now() + Duration::from_secs(2);
        while views.allocated.load(Ordering::Acquire) != 0 && std::time::Instant::now() < deadline {
            std::thread::yield_now();
        }
        assert_eq!(views.charged.load(Ordering::Acquire), 0);
        assert_eq!(views.allocated.load(Ordering::Acquire), 0);
        assert!(matches!(views.get(1, first), Err(libc::EBADF)));
    }

    #[test]
    fn exhausted_budgets_reject_new_views_without_losing_existing_handles() {
        let temp = tempfile::tempdir().unwrap();
        let mut store = Store::open(temp.path()).unwrap();
        store
            .install_remote_snapshot(
                &[Node {
                    id: "root".into(),
                    name: "root".into(),
                    parent: None,
                    directory: true,
                    version: "1".into(),
                    metadata: None,
                }],
                "1",
                "s",
            )
            .unwrap();
        store.ensure_fs_root("root").unwrap();
        let store = drive_core::SharedStore::new(store);
        let views = Views::default();
        views.charged.store(MAX_BYTES, Ordering::Release);
        let revision = store.lock().unwrap().local_revision();
        assert_eq!(views.open(&store, 1), Err(libc::ENOMEM));
        assert_eq!(store.lock().unwrap().local_revision(), revision);
        assert!(views.table.lock().unwrap().handles.is_empty());
        views.charged.store(0, Ordering::Release);
        views.allocated.store(MAX_HANDLES, Ordering::Release);
        assert_eq!(views.open(&store, 1), Err(libc::ENOMEM));
        views.allocated.store(0, Ordering::Release);
        let fh = views.open(&store, 1).unwrap();
        let view = views.get(1, fh).unwrap();
        {
            let mut table = views.table.lock().unwrap();
            for handle in 2..=MAX_HANDLES as u64 {
                table.handles.insert(handle, view.clone());
            }
            table.next = MAX_HANDLES as u64 + 1;
        }
        assert_eq!(views.open(&store, 1), Err(libc::EMFILE));
        assert_eq!(views.get(1, fh).unwrap().snapshot().entries.len(), 2);
        views.release(1, 2).unwrap();
        assert!(views.open(&store, 1).is_ok());
        assert_eq!(views.charged.load(Ordering::Acquire), view.snapshot().bytes);
    }

    #[test]
    fn reclaim_channel_holds_every_reserved_view_without_returning_capacity_early() {
        let (sender, receiver) = sync_channel(MAX_HANDLES);
        let charged = Arc::new(AtomicUsize::new(0));
        let allocated = Arc::new(AtomicUsize::new(0));
        // Deliberately leave the receiver idle to model a stalled reclaimer.
        let reclaimer = Arc::new(Reclaimer {
            sender: Some(sender),
            thread: None,
            charged: charged.clone(),
            allocated: allocated.clone(),
        });
        let bytes = std::mem::size_of::<drive_core::filesystem::DirectoryEntry>() + 1;
        for _ in 0..MAX_HANDLES {
            let snapshot = DirectorySnapshot {
                entries: vec![drive_core::filesystem::DirectoryEntry {
                    ino: 1,
                    directory: true,
                    name: ".".into(),
                }],
                bytes,
            };
            charged.fetch_add(bytes, Ordering::AcqRel);
            allocated.fetch_add(1, Ordering::AcqRel);
            drop(View {
                ino: 1,
                snapshot: Some(snapshot),
                reclaimer: reclaimer.clone(),
            });
        }
        assert_eq!(charged.load(Ordering::Acquire), MAX_HANDLES * bytes);
        assert_eq!(allocated.load(Ordering::Acquire), MAX_HANDLES);
        for _ in 0..MAX_HANDLES {
            reclaim(receiver.try_recv().unwrap(), &charged, &allocated);
        }
        assert!(receiver.try_recv().is_err());
        assert_eq!(charged.load(Ordering::Acquire), 0);
        assert_eq!(allocated.load(Ordering::Acquire), 0);
    }

    #[test]
    fn last_reader_can_outlive_the_handle_table_and_drain_reclaimer() {
        let temp = tempfile::tempdir().unwrap();
        let mut store = Store::open(temp.path()).unwrap();
        store
            .install_remote_snapshot(
                &[Node {
                    id: "root".into(),
                    name: "root".into(),
                    parent: None,
                    directory: true,
                    version: "1".into(),
                    metadata: None,
                }],
                "1",
                "s",
            )
            .unwrap();
        store.ensure_fs_root("root").unwrap();
        let views = Views::default();
        let fh = views.open(&drive_core::SharedStore::new(store), 1).unwrap();
        let reader = views.get(1, fh).unwrap();
        let charged = views.charged.clone();
        let allocated = views.allocated.clone();
        drop(views);
        assert_eq!(reader.snapshot().entries.len(), 2);
        assert!(charged.load(Ordering::Acquire) > 0);
        drop(reader); // last reclaimer owner closes channel and joins its drain
        assert_eq!(charged.load(Ordering::Acquire), 0);
        assert_eq!(allocated.load(Ordering::Acquire), 0);
    }
}
