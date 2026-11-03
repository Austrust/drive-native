//! Linux platform adapter. Current development slice: metadata and immutable
//! cached reads, injected native hydration, durable writes and file creation.
//! Rename, replacement, unlink and mkdir use durable local journals.
//! No callback waits for the NAS, and unsupported writes are never acknowledged.
#[cfg(test)]
mod cached_close_tests;
pub mod diagnostics;
mod directory;
#[cfg(test)]
mod inode_backfill_tests;
#[cfg(test)]
mod inline_metadata_tests;
#[cfg(test)]
mod isolation_tests;
mod namespace;
mod preview;
pub mod publication;
pub use publication::NotifyHub;
mod read_requests;
#[cfg(test)]
mod shutdown_tests;
mod writable;
use drive_core::filesystem::FileRecord;
use drive_core::{Error, Node, ReadHandle, Store};
use fuser::{
    FileAttr, FileType, Filesystem, ReplyAttr, ReplyData, ReplyDirectory, ReplyDirectoryPlus,
    ReplyEmpty, ReplyEntry, ReplyOpen, Request,
};
pub use read_requests::ReadCancellation;
use read_requests::{ReadReply, ReadRequests};
use std::{
    collections::HashMap,
    ffi::OsStr,
    sync::{
        Arc, Mutex,
        mpsc::{SyncSender, sync_channel},
    },
    time::{Duration, UNIX_EPOCH},
};

const TTL: Duration = Duration::ZERO;
type Job = Box<dyn FnOnce() + Send + 'static>;
/// Bounded pools keep work off the FUSE dispatcher. Metadata/file I/O use the
/// metadata pool; held-file reads and native hydration each use separate pools.
struct LocalQueue {
    sender: Option<SyncSender<Job>>,
    threads: Vec<std::thread::JoinHandle<()>>,
}
impl LocalQueue {
    fn new() -> Self {
        let (sender, receiver) = sync_channel::<Job>(256);
        let receiver = Arc::new(Mutex::new(receiver));
        let threads = (0..4)
            .map(|_| {
                let receiver = receiver.clone();
                std::thread::spawn(move || {
                    loop {
                        let job = receiver.lock().unwrap().recv();
                        match job {
                            Ok(job) => job(),
                            Err(_) => break,
                        }
                    }
                })
            })
            .collect();
        Self {
            sender: Some(sender),
            threads,
        }
    }
    fn submit(&self, job: Job) {
        // fuser replies EIO when an unsent Reply is dropped on overload. No
        // successful operation or empty data is fabricated to satisfy a read.
        let _ = self.sender.as_ref().unwrap().try_send(job);
    }
}
impl LocalQueue {
    fn drain(&mut self) {
        self.sender.take();
        for t in self.threads.drain(..) {
            let _ = t.join();
        }
    }
}
impl Drop for LocalQueue {
    fn drop(&mut self) {
        self.drain();
    }
}
#[derive(Clone)]
enum OpenContent {
    Cached(Arc<ReadHandle>),
    Missing(Node),
    Draft(Arc<drive_core::drafts::DraftHandle>),
}
struct Handle {
    ino: u64,
    node: String,
    flags: i32,
    content: OpenContent,
}
struct Handles {
    next: u64,
    entries: HashMap<u64, Handle>,
}
/// Shared only with the local state owner; the network process receives IPC,
/// never a SQLite connection. The runtime injects network integration here.
pub trait Hydrator: Send + Sync {
    fn local_saved(&self) {}
    fn hydrate(&self, node: &Node) -> Result<Arc<ReadHandle>, i32>;
    fn hydrate_cancellable(
        &self,
        node: &Node,
        cancel: &ReadCancellation,
    ) -> Result<Arc<ReadHandle>, i32> {
        if cancel.is_cancelled() {
            return Err(libc::EINTR);
        }
        self.hydrate(node)
    }
}
pub struct CacheFilesystem {
    storage: Arc<std::fs::File>,
    writable: bool,
    store: Arc<drive_core::SharedStore>,
    handles: Arc<Mutex<Handles>>,
    // Serializes registration/promotion/namespace barriers, never held by reads.
    // Inside this gate, Store and the handle table are acquired separately.
    handle_updates: Arc<Mutex<()>>,
    directories: Arc<directory::Views>,
    queue: LocalQueue,
    read_queue: LocalQueue,
    reads: ReadRequests,
    network_queue: Arc<LocalQueue>,
    hydrator: Option<Arc<dyn Hydrator>>,
    // Metadata publication gate + kernel invalidation delivery (see
    // publication.rs). Inert until a session notifier is installed.
    notify: NotifyHub,
    // Request READDIRPLUS at FUSE_INIT. FUSE_INIT necessarily happens before
    // the session (and therefore the notifier) exists, so "notifier installed"
    // cannot be observed here; `with_notify_hub` is the mount owner's
    // declaration that it will install one. Without that declaration the mount
    // keeps the plain READDIR profile (e.g. kernel tests that never install a
    // notifier).
    readdirplus: bool,
    uid: u32,
    gid: u32,
}
impl CacheFilesystem {
    pub fn with_writes_enabled(mut self) -> Self {
        self.writable = true;
        self
    }

    pub fn with_hydrator(mut self, hydrator: Arc<dyn Hydrator>) -> Self {
        self.hydrator = Some(hydrator);
        self
    }

    /// Install the mount's publication/invalidation hub. Mount owners must call
    /// [`NotifyHub::install`] with `BackgroundSession::notifier()` after the
    /// session exists; until then replies carry zero TTL.
    pub fn with_notify_hub(mut self, hub: NotifyHub) -> Self {
        self.notify = hub;
        // The owner promises to install the session notifier right after the
        // session exists, which is the earliest point the API allows.
        self.readdirplus = true;
        self
    }

    /// Override the metadata TTL (0 disables caching). The default comes from
    /// `DRIVE_NATIVE_FS_TTL_MS` (2000 ms).
    pub fn with_metadata_ttl_ms(mut self, ttl_ms: u64) -> Self {
        self.notify = NotifyHub::with_ttl_ms(ttl_ms);
        self
    }

    pub fn notify_hub(&self) -> NotifyHub {
        self.notify.clone()
    }

    pub fn new(
        store: Arc<drive_core::SharedStore>,
        root: &str,
        uid: u32,
        gid: u32,
    ) -> drive_core::Result<Self> {
        {
            let s = store.lock().unwrap();
            match s.node(root) {
                Ok(_) => {
                    s.ensure_fs_root(root)?;
                    // Upgrade path: databases installed before batch inode
                    // allocation get their `fs_inodes` mapping backfilled in one
                    // transaction here, while mount initialization still
                    // serializes Store access. Idempotent and writes nothing
                    // (no WAL frame, no fsync) when the mapping is complete.
                    s.ensure_filesystem_inodes(root)?;
                }
                Err(e) if e.is_not_found() => {
                    if s.preview_mount_root(root)?.is_none() {
                        return Err(e);
                    }
                }
                Err(e) => return Err(e),
            }
        }
        let storage = Arc::new(std::fs::File::open(
            store.lock().unwrap().storage_directory(),
        )?);
        Ok(Self {
            storage,
            writable: false,
            store,
            handles: Arc::new(Mutex::new(Handles {
                next: 1,
                entries: HashMap::new(),
            })),
            directories: Arc::new(directory::Views::default()),
            queue: LocalQueue::new(),
            read_queue: LocalQueue::new(),
            handle_updates: Arc::new(Mutex::new(())),
            reads: ReadRequests::default(),
            network_queue: Arc::new(LocalQueue::new()),
            hydrator: None,
            notify: NotifyHub::from_env(),
            readdirplus: false,
            uid,
            gid,
        })
    }
}
fn errno(error: &Error) -> i32 {
    if error.is_not_found() {
        return libc::ENOENT;
    }
    match error {
        Error::Offline => libc::ENETDOWN,
        Error::Io(e) => e.raw_os_error().unwrap_or(libc::EIO),
        _ => libc::EIO,
    }
}
fn filetype(record: &FileRecord) -> FileType {
    if record.node.directory {
        FileType::Directory
    } else {
        FileType::RegularFile
    }
}
fn attr(record: &FileRecord, uid: u32, gid: u32) -> FileAttr {
    let time = UNIX_EPOCH
        .checked_add(Duration::from_secs(record.mtime))
        .unwrap_or(UNIX_EPOCH);
    FileAttr {
        ino: record.ino,
        size: record.size,
        blocks: if matches!(
            record.status,
            drive_core::Status::OnlineOnly | drive_core::Status::Pinning
        ) {
            0
        } else {
            record.size.div_ceil(512)
        },
        atime: UNIX_EPOCH
            .checked_add(Duration::new(record.atime, record.atime_nsec))
            .unwrap_or(UNIX_EPOCH),
        mtime: time,
        ctime: time,
        crtime: UNIX_EPOCH,
        kind: filetype(record),
        perm: record.mode as u16,
        nlink: if record.unlinked {
            0
        } else if record.node.directory {
            2
        } else {
            1
        },
        uid,
        gid,
        rdev: 0,
        blksize: 4096,
        flags: 0,
    }
}
impl Filesystem for CacheFilesystem {
    fn init(
        &mut self,
        _req: &Request<'_>,
        config: &mut fuser::KernelConfig,
    ) -> Result<(), libc::c_int> {
        config
            .set_time_granularity(Duration::from_secs(1))
            .map_err(|_| libc::EINVAL)?;
        // READDIRPLUS pays off only with a bounded TTL and only when the mount
        // owner declared that it installs an invalidation notifier (see the
        // `readdirplus` field); otherwise the mount keeps the safe-stage profile.
        if self.readdirplus && self.notify.configured_ttl_ms() > 0 {
            let _ = config.enable_readdirplus();
        }
        Ok(())
    }
    fn destroy(&mut self) {
        // No new callbacks are dispatched after destroy. Cancel reads only;
        // accepted writes, namespace edits and fsync jobs must finish normally.
        self.reads.cancel_all();
        self.queue.drain();
        self.read_queue.drain();
        // Only read_queue jobs clone this pool; all those jobs are now joined.
        if let Some(network) = Arc::get_mut(&mut self.network_queue) {
            network.drain();
        }
    }

    fn interrupt(&mut self, _req: &Request<'_>, unique: u64, reply: ReplyEmpty) {
        if self.reads.interrupt(unique) {
            reply.ignore_interrupt();
        } else {
            // INTERRUPT can overtake its original request. Retry after a short
            // bounded queue delay; never wait on the dispatcher or disable
            // interrupts for the whole mount because this request is unknown.
            self.queue.submit(Box::new(move || {
                std::thread::sleep(Duration::from_millis(10));
                reply.error(libc::EAGAIN);
            }));
        }
    }

    fn mkdir(
        &mut self,
        _req: &Request<'_>,
        parent: u64,
        name: &OsStr,
        mode: u32,
        umask: u32,
        reply: ReplyEntry,
    ) {
        if !self.writable {
            reply.error(libc::EROFS);
            return;
        }
        let Some(name) = name.to_str().map(str::to_owned) else {
            reply.error(libc::EINVAL);
            return;
        };
        let store = self.store.clone();
        let hydrator = self.hydrator.clone();
        let notify = self.notify.clone();
        let (uid, gid) = (self.uid, self.gid);
        self.queue.submit(Box::new(move || {
            let result = (|| -> drive_core::Result<_> {
                let mut s = store.lock().unwrap();
                let p = preview::authoritative(&s, parent, libc::EROFS)?;
                let node = s.create_local_directory(&p.node.id, &name, mode & !umask)?;
                s.record(&node.id)
            })();
            match result {
                Ok(record) => {
                    if let Some(h) = hydrator {
                        h.local_saved();
                    }
                    // Queue after the commit: the worker takes the publication
                    // gate before writing anything to the kernel.
                    notify.invalidate_entry(parent, &name);
                    notify.invalidate_inode(record.ino);
                    reply.entry(&TTL, &attr(&record, uid, gid), 0)
                }
                Err(e) => reply.error(errno(&e)),
            }
        }));
    }

    fn statfs(&mut self, _req: &Request<'_>, _ino: u64, reply: fuser::ReplyStatfs) {
        let storage = self.storage.clone();
        self.queue.submit(Box::new(move || {
            use std::os::fd::AsRawFd;
            let mut stat = std::mem::MaybeUninit::<libc::statvfs>::uninit();
            // fstatvfs initializes the complete buffer on success. The open FD
            // pins the real durable storage directory, never the FUSE mount.
            if unsafe { libc::fstatvfs(storage.as_raw_fd(), stat.as_mut_ptr()) } != 0 {
                reply.error(
                    std::io::Error::last_os_error()
                        .raw_os_error()
                        .unwrap_or(libc::EIO),
                );
                return;
            }
            let s = unsafe { stat.assume_init() };
            reply.statfs(
                s.f_blocks,
                s.f_bfree,
                s.f_bavail,
                s.f_files,
                s.f_ffree,
                s.f_bsize.min(u32::MAX as u64) as u32,
                255,
                s.f_frsize.min(u32::MAX as u64) as u32,
            );
        }));
    }

    fn unlink(&mut self, _req: &Request<'_>, parent: u64, name: &OsStr, reply: ReplyEmpty) {
        namespace::remove(self, parent, name, false, reply)
    }
    fn rmdir(&mut self, _req: &Request<'_>, parent: u64, name: &OsStr, reply: ReplyEmpty) {
        namespace::remove(self, parent, name, true, reply)
    }
    fn rename(
        &mut self,
        _req: &Request<'_>,
        parent: u64,
        name: &OsStr,
        newparent: u64,
        newname: &OsStr,
        flags: u32,
        reply: ReplyEmpty,
    ) {
        namespace::rename(self, parent, name, newparent, newname, flags, reply)
    }

    fn create(
        &mut self,
        _req: &Request<'_>,
        parent: u64,
        name: &OsStr,
        mode: u32,
        umask: u32,
        flags: i32,
        reply: fuser::ReplyCreate,
    ) {
        writable::create(self, parent, name, mode, umask, flags, reply)
    }
    fn lookup(&mut self, _req: &Request<'_>, parent: u64, name: &OsStr, reply: ReplyEntry) {
        let Some(name) = name.to_str().map(str::to_owned) else {
            reply.error(libc::ENOENT);
            return;
        };
        // Observation only (never a lock): reply paths must not block on
        // invalidation delivery; the bounded TTL and the authoritative Store are
        // the correctness contract (see publication.rs).
        let inline_publication = self.notify.reply_guard();
        // Cheap authoritative metadata is answered on the dispatcher thread; the
        // single-statement projection owns its result before the reply is
        // written. Busy Store, negative and candidate-preview lookups keep the
        // original offloaded path.
        if let Some(result) =
            preview::try_inline_lookup(&self.store, self.uid, self.gid, parent, &name)
        {
            match result {
                Ok((attr, cacheable)) => {
                    let (entry_ttl, attr_ttl) =
                        preview::reply_ttls(&self.notify, cacheable, true);
                    diagnostics::ttls(Some(entry_ttl), Some(attr_ttl));
                    self.notify.apply_reply_delay();
                    reply.entry_with_timeouts(&entry_ttl, &attr_ttl, &attr, 0);
                }
                Err(e) => reply.error(e),
            }
            drop(inline_publication);
            return;
        }
        drop(inline_publication);
        let store = self.store.clone();
        let notify = self.notify.clone();
        let (uid, gid) = (self.uid, self.gid);
        let queued = diagnostics::span(diagnostics::Stage::LookupQueue);
        self.queue.submit(Box::new(move || {
            drop(queued);
            let publication = notify.reply_guard();
            let wait = diagnostics::span(diagnostics::Stage::LookupStoreWait);
            let store = store.lock().unwrap();
            drop(wait);
            let projection = diagnostics::span(diagnostics::Stage::LookupProjection);
            let result = preview::lookup(&store, parent, &name);
            drop(projection);
            // Authority results are owned; preview replies keep the Store
            // boundary through publication.
            if !matches!(&result, Ok(preview::Metadata::Preview(_))) {
                drop(store);
            }
            match result {
                Ok(record) => {
                    let cache_entry = matches!(&record, preview::Metadata::Authority(_));
                    let cacheable = record.cacheable();
                    let (entry_ttl, attr_ttl) =
                        preview::reply_ttls(&notify, cacheable, cache_entry);
                    diagnostics::ttls(Some(entry_ttl), Some(attr_ttl));
                    notify.apply_reply_delay();
                    reply.entry_with_timeouts(&entry_ttl, &attr_ttl, &record.attr(uid, gid), 0);
                }
                Err(e) => reply.error(errno(&e)),
            }
            drop(publication);
        }));
    }
    fn getattr(&mut self, _req: &Request<'_>, ino: u64, fh: Option<u64>, reply: ReplyAttr) {
        let inline_publication = self.notify.reply_guard();
        if let Some(result) = preview::try_inline_getattr(
            &self.store,
            &self.handles,
            self.uid,
            self.gid,
            ino,
            fh,
        ) {
            match result {
                Ok((attr, cacheable)) => {
                    let (_, mut attr_ttl) = preview::reply_ttls(&self.notify, cacheable, true);
                    if fh.is_some() {
                        // Attributes adjusted from an open handle must revalidate.
                        attr_ttl = Duration::ZERO;
                    }
                    diagnostics::ttls(None, Some(attr_ttl));
                    self.notify.apply_reply_delay();
                    reply.attr(&attr_ttl, &attr);
                }
                Err(e) => reply.error(e),
            }
            drop(inline_publication);
            return;
        }
        drop(inline_publication);
        let store = self.store.clone();
        let handles = self.handles.clone();
        let notify = self.notify.clone();
        let (uid, gid) = (self.uid, self.gid);
        let queued = diagnostics::span(diagnostics::Stage::GetattrQueue);
        self.queue.submit(Box::new(move || {
            drop(queued);
            let publication = notify.reply_guard();
            let wait = diagnostics::span(diagnostics::Stage::GetattrStoreWait);
            let guard = store.lock().unwrap();
            drop(wait);
            let projection = diagnostics::span(diagnostics::Stage::GetattrProjection);
            let result = preview::metadata(&guard, ino);
            drop(projection);
            // Candidate attributes are revocable metadata too; keep the same
            // validation boundary until its zero-TTL reply has been sent.
            if !matches!(&result, Ok(preview::Metadata::Preview(_))) {
                drop(guard);
            }
            match result {
                Ok(preview::Metadata::Preview(record)) => {
                    if fh.is_some() {
                        reply.error(libc::EBADF);
                    } else {
                        // Candidate previews never publish a cacheable answer.
                        diagnostics::ttls(None, Some(Duration::ZERO));
                        notify.apply_reply_delay();
                        reply.attr(&TTL, &preview::preview_attr(&record, uid, gid));
                    }
                }
                Ok(preview::Metadata::Authority(record)) => {
                    let (_, mut attr_ttl) = preview::reply_ttls(&notify, record.cacheable, true);
                    if fh.is_some() {
                        attr_ttl = Duration::ZERO;
                    }
                    let mut a = preview::posix_attr(&record, uid, gid);
                    if let Some(fh) = fh {
                        let content = {
                            let h = handles.lock().unwrap();
                            h.entries
                                .get(&fh)
                                .filter(|h| h.ino == ino)
                                .map(|h| h.content.clone())
                        };
                        match content {
                            Some(content) => {
                                if let OpenContent::Draft(d) = &content {
                                    match d.size() {
                                        Ok(size) => {
                                            a.size = size;
                                            a.blocks = size.div_ceil(512);
                                        }
                                        Err(e) => {
                                            reply.error(e.raw_os_error().unwrap_or(libc::EIO));
                                            return;
                                        }
                                    }
                                }

                                if let OpenContent::Cached(content) = &content {
                                    match content.size() {
                                        Ok(s) => {
                                            a.size = s;
                                            a.blocks = s.div_ceil(512)
                                        }
                                        Err(e) => {
                                            reply.error(e.raw_os_error().unwrap_or(libc::EIO));
                                            return;
                                        }
                                    }
                                }
                            }
                            _ => {
                                reply.error(libc::EBADF);
                                return;
                            }
                        }
                    }
                    diagnostics::ttls(None, Some(attr_ttl));
                    notify.apply_reply_delay();
                    reply.attr(&attr_ttl, &a)
                }
                Err(e) => reply.error(errno(&e)),
            }
            drop(publication);
        }));
    }
    fn opendir(&mut self, _req: &Request<'_>, ino: u64, flags: i32, reply: ReplyOpen) {
        directory::open(self, ino, flags, reply)
    }
    fn readdir(
        &mut self,
        _req: &Request<'_>,
        ino: u64,
        fh: u64,
        offset: i64,
        reply: ReplyDirectory,
    ) {
        directory::read(self, ino, fh, offset, reply)
    }
    fn readdirplus(
        &mut self,
        _req: &Request<'_>,
        ino: u64,
        fh: u64,
        offset: i64,
        reply: ReplyDirectoryPlus,
    ) {
        directory::read_plus(self, ino, fh, offset, reply)
    }
    fn releasedir(
        &mut self,
        _req: &Request<'_>,
        ino: u64,
        fh: u64,
        _flags: i32,
        reply: ReplyEmpty,
    ) {
        directory::release(self, ino, fh, reply)
    }
    fn open(&mut self, _req: &Request<'_>, ino: u64, flags: i32, reply: ReplyOpen) {
        writable::open(self, ino, flags, reply)
    }
    fn write(
        &mut self,
        _req: &Request<'_>,
        ino: u64,
        fh: u64,
        offset: i64,
        data: &[u8],
        _write_flags: u32,
        _flags: i32,
        _lock_owner: Option<u64>,
        reply: fuser::ReplyWrite,
    ) {
        writable::write(self, ino, fh, offset, data, reply)
    }
    fn fsync(&mut self, _req: &Request<'_>, ino: u64, fh: u64, _datasync: bool, reply: ReplyEmpty) {
        writable::sync(self, ino, fh, false, reply)
    }
    fn flush(
        &mut self,
        _req: &Request<'_>,
        ino: u64,
        fh: u64,
        _lock_owner: u64,
        reply: ReplyEmpty,
    ) {
        writable::sync(self, ino, fh, false, reply)
    }
    fn setattr(
        &mut self,
        _req: &Request<'_>,
        ino: u64,
        mode: Option<u32>,
        uid: Option<u32>,
        gid: Option<u32>,
        size: Option<u64>,
        atime: Option<fuser::TimeOrNow>,
        mtime: Option<fuser::TimeOrNow>,
        ctime: Option<std::time::SystemTime>,
        fh: Option<u64>,
        crtime: Option<std::time::SystemTime>,
        chgtime: Option<std::time::SystemTime>,
        bkuptime: Option<std::time::SystemTime>,
        flags: Option<u32>,
        reply: ReplyAttr,
    ) {
        writable::setattr(
            self, ino, mode, uid, gid, size, atime, mtime, ctime, fh, crtime, chgtime, bkuptime,
            flags, reply,
        )
    }
    fn read(
        &mut self,
        req: &Request<'_>,
        ino: u64,
        fh: u64,
        offset: i64,
        size: u32,
        _flags: i32,
        _lock_owner: Option<u64>,
        reply: ReplyData,
    ) {
        if offset < 0 || size > 1024 * 1024 {
            reply.error(libc::EINVAL);
            return;
        }
        let reply = self.reads.register(req.unique(), reply);
        let cancel = reply.cancellation();
        let handles = self.handles.clone();
        let network_queue = self.network_queue.clone();
        let hydrator = self.hydrator.clone();
        self.read_queue.submit(Box::new(move || {
            let (cached, missing) = {
                let table = handles.lock().unwrap();
                match table.entries.get(&fh) {
                    Some(h) if h.ino == ino => match &h.content {
                        OpenContent::Cached(c) => (Some(c.clone()), None),
                        OpenContent::Missing(n) => (None, Some(n.clone())),
                        OpenContent::Draft(d) => {
                            let d = d.clone();
                            drop(table);
                            let mut data = vec![0; size as usize];
                            match d.read_at(&mut data, offset as u64) {
                                Ok(n) => reply.data(&data[..n]),
                                Err(e) => reply.error(e.raw_os_error().unwrap_or(libc::EIO)),
                            }
                            return;
                        }
                    },
                    _ => {
                        reply.error(libc::EBADF);
                        return;
                    }
                }
            };
            if let Some(content) = cached {
                reply_read(content, offset, size, reply);
                return;
            }
            let Some(hydrator) = hydrator else {
                reply.error(libc::ENETDOWN);
                return;
            };
            network_queue.submit(Box::new(move || {
                match hydrator.hydrate_cancellable(&missing.unwrap(), &cancel) {
                    Ok(content) => {
                        let mut h = handles.lock().unwrap();
                        let Some(entry) = h.entries.get_mut(&fh) else {
                            reply.error(libc::EBADF);
                            return;
                        };
                        // A concurrent write may have promoted this handle to
                        // a mutable draft while hydration was in flight.
                        if matches!(&entry.content, OpenContent::Missing(_)) {
                            entry.content = OpenContent::Cached(content);
                        }
                        let current = entry.content.clone();
                        drop(h);
                        match current {
                            OpenContent::Cached(c) => reply_read(c, offset, size, reply),
                            OpenContent::Draft(d) => {
                                let mut data = vec![0; size as usize];
                                match d.read_at(&mut data, offset as u64) {
                                    Ok(n) => reply.data(&data[..n]),
                                    Err(e) => reply.error(e.raw_os_error().unwrap_or(libc::EIO)),
                                }
                            }
                            OpenContent::Missing(_) => reply.error(libc::EIO),
                        }
                    }
                    Err(code) => reply.error(code),
                }
            }));
        }));
    }

    fn release(
        &mut self,
        _req: &Request<'_>,
        ino: u64,
        fh: u64,
        _flags: i32,
        _lock_owner: Option<u64>,
        _flush: bool,
        reply: ReplyEmpty,
    ) {
        writable::sync(self, ino, fh, true, reply)
    }
}

fn reply_read(content: Arc<ReadHandle>, offset: i64, size: u32, reply: ReadReply) {
    let mut data = vec![0; size as usize];
    match content.read_at(&mut data, offset as u64) {
        Ok(n) => reply.data(&data[..n]),
        Err(e) => reply.error(e.raw_os_error().unwrap_or(libc::EIO)),
    }
}
