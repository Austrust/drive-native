//! Linux platform adapter. Current development slice: metadata and immutable
//! cached reads, injected native hydration, durable writes and file creation.
//! Rename, replacement, unlink and mkdir use durable local journals.
//! No callback waits for the NAS, and unsupported writes are never acknowledged.
mod namespace;
mod read_requests;
mod writable;
use drive_core::filesystem::FileRecord;
use drive_core::{Error, Node, ReadHandle, Store};
use fuser::{
    FileAttr, FileType, Filesystem, ReplyAttr, ReplyData, ReplyDirectory, ReplyEmpty, ReplyEntry,
    ReplyOpen, Request,
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
/// local pool; waits for native hydration use a separate pool and worker.
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
impl Drop for LocalQueue {
    fn drop(&mut self) {
        self.sender.take();
        for t in self.threads.drain(..) {
            let _ = t.join();
        }
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
    store: Arc<Mutex<Store>>,
    handles: Arc<Mutex<Handles>>,
    queue: LocalQueue,
    reads: ReadRequests,
    network_queue: Arc<LocalQueue>,
    hydrator: Option<Arc<dyn Hydrator>>,
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

    pub fn new(
        store: Arc<Mutex<Store>>,
        root: &str,
        uid: u32,
        gid: u32,
    ) -> drive_core::Result<Self> {
        store.lock().unwrap().ensure_fs_root(root)?;
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
            queue: LocalQueue::new(),
            reads: ReadRequests::default(),
            network_queue: Arc::new(LocalQueue::new()),
            hydrator: None,
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
        atime: time,
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
        let (uid, gid) = (self.uid, self.gid);
        self.queue.submit(Box::new(move || {
            let result = (|| -> drive_core::Result<_> {
                let mut s = store.lock().unwrap();
                let p = s.record_by_inode(parent)?;
                let node = s.create_local_directory(&p.node.id, &name, mode & !umask)?;
                s.record(&node.id)
            })();
            match result {
                Ok(record) => {
                    if let Some(h) = hydrator {
                        h.local_saved();
                    }
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
        let store = self.store.clone();
        let (uid, gid) = (self.uid, self.gid);
        self.queue.submit(Box::new(move || {
            let store = store.lock().unwrap();
            let result = store.record_by_inode(parent).and_then(|p| {
                if !p.node.directory {
                    return Err(Error::Io(std::io::Error::from_raw_os_error(libc::ENOTDIR)));
                }
                store.lookup(&p.node.id, &name)
            });
            match result {
                Ok(record) => reply.entry(&TTL, &attr(&record, uid, gid), 0),
                Err(e) => reply.error(errno(&e)),
            }
        }));
    }
    fn getattr(&mut self, _req: &Request<'_>, ino: u64, fh: Option<u64>, reply: ReplyAttr) {
        let store = self.store.clone();
        let handles = self.handles.clone();
        let (uid, gid) = (self.uid, self.gid);
        self.queue.submit(Box::new(move || {
            let result = store.lock().unwrap().record_by_inode(ino);
            match result {
                Ok(record) => {
                    let mut a = attr(&record, uid, gid);
                    if let Some(fh) = fh {
                        let h = handles.lock().unwrap();
                        match h.entries.get(&fh) {
                            Some(h) if h.ino == ino => {
                                if let OpenContent::Draft(d) = &h.content {
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

                                if let OpenContent::Cached(content) = &h.content {
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
                    reply.attr(&TTL, &a)
                }
                Err(e) => reply.error(errno(&e)),
            }
        }));
    }
    fn readdir(
        &mut self,
        _req: &Request<'_>,
        ino: u64,
        _fh: u64,
        offset: i64,
        mut reply: ReplyDirectory,
    ) {
        if offset < 0 {
            reply.error(libc::EINVAL);
            return;
        }
        let store = self.store.clone();
        self.queue.submit(Box::new(move || {
            let store = store.lock().unwrap();
            let result = (|| -> drive_core::Result<Vec<(u64, FileType, String)>> {
                let p = store.record_by_inode(ino)?;
                let parent = match p.node.parent.as_deref() {
                    Some(id) => store.record(id)?.ino,
                    None => 1,
                };
                let mut entries = vec![
                    (ino, FileType::Directory, ".".into()),
                    (parent, FileType::Directory, "..".into()),
                ];
                for c in store.children(&p.node.id)? {
                    entries.push((c.ino, filetype(&c), c.node.name));
                }
                Ok(entries)
            })();
            match result {
                Ok(entries) => {
                    for (i, (n, t, name)) in entries.iter().enumerate().skip(offset as usize) {
                        if reply.add(*n, (i + 1) as i64, *t, name) {
                            break;
                        }
                    }
                    reply.ok()
                }
                Err(e) => reply.error(errno(&e)),
            }
        }));
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
        self.queue.submit(Box::new(move || {
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
