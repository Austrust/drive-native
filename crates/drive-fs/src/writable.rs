use super::*;
use drive_core::drafts::DraftHandle;
use fuser::{ReplyWrite, TimeOrNow};
use std::time::SystemTime;

fn content(store: &Store, ino: u64) -> drive_core::Result<(String, OpenContent)> {
    let record = store.record_by_inode(ino)?;
    if record.node.directory {
        return Err(Error::Io(std::io::Error::from_raw_os_error(libc::EISDIR)));
    }
    if let Some(d) = store.open_dirty_draft(&record.node.id)? {
        return Ok((record.node.id, OpenContent::Draft(Arc::new(d))));
    }
    let body = match store.open_cached(&record.node.id) {
        Ok(h) => OpenContent::Cached(Arc::new(h)),
        Err(Error::Offline) => OpenContent::Missing(record.node.clone()),
        Err(e) => return Err(e),
    };
    Ok((record.node.id, body))
}
fn draft(
    store: &Arc<Mutex<Store>>,
    handles: &Arc<Mutex<Handles>>,
    updates: &Arc<Mutex<()>>,
    hydrator: &Option<Arc<dyn Hydrator>>,
    id: &str,
    body: OpenContent,
    empty: bool,
) -> Result<Arc<DraftHandle>, i32> {
    if let OpenContent::Draft(d) = body {
        return Ok(d);
    }
    let (base, source) = match body {
        OpenContent::Cached(c) => (c.version().to_owned(), Some(c)),
        OpenContent::Missing(n) if empty => (n.version, None),
        OpenContent::Missing(n) => {
            let c = hydrator.as_ref().ok_or(libc::ENETDOWN)?.hydrate(&n)?;
            (c.version().to_owned(), Some(c))
        }
        OpenContent::Draft(_) => unreachable!(),
    };
    let _update = updates.lock().unwrap();
    let d = Arc::new(
        store
            .lock()
            .unwrap()
            .begin_draft(id, &base, if empty { None } else { source.as_deref() })
            .map_err(|e| errno(&e))?,
    );
    let candidates: Vec<_> = {
        let h = handles.lock().unwrap();
        h.entries
            .iter()
            .filter(|(_, h)| h.node == id)
            .map(|(fh, h)| (*fh, h.content.clone()))
            .collect()
    };
    let matching: Vec<_> = {
        let s = store.lock().unwrap();
        candidates
            .iter()
            .filter_map(|(fh, content)| {
                let matches = match content {
                    OpenContent::Cached(c) => s
                        .equivalent_content_version(id, c.version())
                        .is_ok_and(|v| v == d.version),
                    OpenContent::Missing(n) => s
                        .equivalent_content_version(id, &n.version)
                        .is_ok_and(|v| v == d.version),
                    OpenContent::Draft(old) => old.token == d.token,
                };
                matches.then_some(*fh)
            })
            .collect()
    };
    let mut h = handles.lock().unwrap();
    for fh in matching {
        if let Some(entry) = h.entries.get_mut(&fh) {
            entry.content = OpenContent::Draft(d.clone());
        }
    }
    Ok(d)
}
pub(super) fn open(fs: &mut CacheFilesystem, ino: u64, flags: i32, reply: ReplyOpen) {
    let writable = flags & libc::O_ACCMODE != libc::O_RDONLY;
    if (writable || flags & libc::O_TRUNC != 0) && !fs.writable {
        reply.error(libc::EROFS);
        return;
    }
    if flags & libc::O_TRUNC != 0 && !writable {
        reply.error(libc::EACCES);
        return;
    }
    let store = fs.store.clone();
    let handles = fs.handles.clone();
    let updates = fs.handle_updates.clone();
    let hydrator = fs.hydrator.clone();
    fs.queue.submit(Box::new(move || {
        let result = content(&store.lock().unwrap(), ino);
        let (id, mut body) = match result {
            Ok(r) => r,
            Err(e) => {
                reply.error(errno(&e));
                return;
            }
        };
        if flags & libc::O_TRUNC != 0 {
            match draft(&store, &handles, &updates, &hydrator, &id, body, true) {
                Ok(d) => {
                    if let Err(e) = store.lock().unwrap().draft_truncate(&d, 0) {
                        reply.error(errno(&e));
                        return;
                    }
                    body = OpenContent::Draft(d)
                }
                Err(e) => {
                    reply.error(e);
                    return;
                }
            }
        }
        let _update = updates.lock().unwrap();
        let refreshed = (|| -> drive_core::Result<()> {
            let s = store.lock().unwrap();
            s.node(&id)?;
            let base = match &body {
                OpenContent::Cached(c) => Some(c.version()),
                OpenContent::Missing(n) => Some(n.version.as_str()),
                OpenContent::Draft(_) => None,
            };
            if let Some(base) = base {
                if let Some(d) = s.open_draft_for_version(&id, base)? {
                    body = OpenContent::Draft(Arc::new(d));
                }
            }
            let _ = s.note_cache_use(&id);
            Ok(())
        })();
        if let Err(e) = refreshed {
            reply.error(errno(&e));
            return;
        }
        let mut h = handles.lock().unwrap();
        if h.entries.len() >= 65536 || h.next == u64::MAX {
            reply.error(libc::EMFILE);
            return;
        }
        let fh = h.next;
        h.next += 1;
        h.entries.insert(
            fh,
            Handle {
                ino,
                node: id,
                flags,
                content: body,
            },
        );
        reply.opened(fh, fuser::consts::FOPEN_DIRECT_IO);
    }));
}
pub(super) fn write(
    fs: &mut CacheFilesystem,
    ino: u64,
    fh: u64,
    offset: i64,
    bytes: &[u8],
    reply: ReplyWrite,
) {
    if !fs.writable {
        reply.error(libc::EROFS);
        return;
    }
    if offset < 0 || bytes.len() > 1024 * 1024 {
        reply.error(libc::EINVAL);
        return;
    }
    let store = fs.store.clone();
    let handles = fs.handles.clone();
    let updates = fs.handle_updates.clone();
    let hydrator = fs.hydrator.clone();
    let network = fs.network_queue.clone();
    let data = bytes.to_vec();
    fs.queue.submit(Box::new(move || {
        let (id, body, flags) = {
            let h = handles.lock().unwrap();
            match h.entries.get(&fh) {
                Some(h) if h.ino == ino => (h.node.clone(), h.content.clone(), h.flags),
                _ => {
                    reply.error(libc::EBADF);
                    return;
                }
            }
        };
        if flags & libc::O_ACCMODE == libc::O_RDONLY {
            reply.error(libc::EBADF);
            return;
        }
        let missing = matches!(body, OpenContent::Missing(_));
        let job: Job = Box::new(move || {
            let result =
                draft(&store, &handles, &updates, &hydrator, &id, body, false).and_then(|d| {
                    store
                        .lock()
                        .unwrap()
                        .draft_write(&d, offset as u64, &data, flags & libc::O_APPEND != 0)
                        .map_err(|e| errno(&e))
                });
            match result {
                Ok(n) => reply.written(n as u32),
                Err(e) => reply.error(e),
            }
        });
        if missing { network.submit(job) } else { job() }
    }));
}
enum SyncRoute {
    // A removed handle is owned by the queued release, including queue rejection.
    Immutable(Option<Handle>),
    Metadata,
    Invalid,
}
fn sync_route(handles: &mut Handles, ino: u64, fh: u64, release: bool) -> SyncRoute {
    match handles.entries.get(&fh) {
        Some(handle) if handle.ino == ino => {
            if handle.flags & libc::O_ACCMODE == libc::O_RDONLY
                && matches!(handle.content, OpenContent::Cached(_))
            {
                // Classification and ownership transfer share the handle-table
                // lock used by promotion. Never re-read it on the read queue.
                SyncRoute::Immutable(if release {
                    handles.entries.remove(&fh)
                } else {
                    None
                })
            } else {
                SyncRoute::Metadata
            }
        }
        _ => SyncRoute::Invalid,
    }
}

struct ReleaseCleanup {
    handles: Arc<Mutex<Handles>>,
    ino: u64,
    fh: u64,
}
impl Drop for ReleaseCleanup {
    fn drop(&mut self) {
        let removed = {
            let mut handles = self.handles.lock().unwrap();
            if handles
                .entries
                .get(&self.fh)
                .is_some_and(|h| h.ino == self.ino)
            {
                handles.entries.remove(&self.fh)
            } else {
                None
            }
        };
        // Drop local content references outside the table lock; never take Store.
        drop(removed);
    }
}

pub(super) fn sync(fs: &mut CacheFilesystem, ino: u64, fh: u64, release: bool, reply: ReplyEmpty) {
    let route = sync_route(&mut fs.handles.lock().unwrap(), ino, fh, release);
    match route {
        SyncRoute::Immutable(None) => {
            // This read-only handle has no local writes to checkpoint. A later
            // promotion cannot retroactively change the captured flush/fsync.
            reply.ok();
            return;
        }
        SyncRoute::Immutable(Some(handle)) => {
            fs.read_queue.submit(Box::new(move || {
                drop(handle);
                reply.ok();
            }));
            return;
        }
        SyncRoute::Invalid => {
            reply.error(libc::EBADF);
            return;
        }
        SyncRoute::Metadata => {}
    }
    let store = fs.store.clone();
    let handles = fs.handles.clone();
    let hydrator = fs.hydrator.clone();
    // The slow path must observe any promotion when it actually executes.
    // Its cleanup also runs if the metadata queue rejects the release job.
    let cleanup = release.then(|| ReleaseCleanup {
        handles: handles.clone(),
        ino,
        fh,
    });
    fs.queue.submit(Box::new(move || {
        let snapshot = {
            let h = handles.lock().unwrap();
            match h.entries.get(&fh) {
                Some(h) if h.ino == ino => Some((h.flags, h.content.clone())),
                _ => None,
            }
        };
        let result = match snapshot {
            Some((flags, OpenContent::Draft(d))) if flags & libc::O_ACCMODE != libc::O_RDONLY => {
                let mut s = store.lock().unwrap();
                s.checkpoint_draft(&d)
                    .and_then(|_| s.creation_saved(&d.node))
                    .map_err(|e| errno(&e))
            }
            Some((_, OpenContent::Draft(d))) => store
                .lock()
                .unwrap()
                .creation_saved(&d.node)
                .map_err(|e| errno(&e)),
            Some(_) => Ok(()),
            None => Err(libc::EBADF),
        };
        drop(cleanup);
        match result {
            Ok(()) => {
                if let Some(h) = hydrator {
                    h.local_saved()
                }
                reply.ok()
            }
            Err(e) => reply.error(e),
        }
    }));
}

#[cfg(test)]
#[path = "sync_routing_tests.rs"]
mod sync_routing_tests;
#[allow(clippy::too_many_arguments)]
pub(super) fn setattr(
    fs: &mut CacheFilesystem,
    ino: u64,
    mode: Option<u32>,
    uid: Option<u32>,
    gid: Option<u32>,
    size: Option<u64>,
    atime: Option<TimeOrNow>,
    mtime: Option<TimeOrNow>,
    _ctime: Option<SystemTime>,
    fh: Option<u64>,
    _crtime: Option<SystemTime>,
    _chgtime: Option<SystemTime>,
    _bkuptime: Option<SystemTime>,
    flags: Option<u32>,
    reply: ReplyAttr,
) {
    if !fs.writable {
        reply.error(libc::EROFS);
        return;
    }
    // This mount belongs to one ordinary user. Restoring the same owner is
    // idempotent; never acknowledge a request to transfer ownership.
    if uid.is_some_and(|v| v != fs.uid) || gid.is_some_and(|v| v != fs.gid) {
        reply.error(libc::EPERM);
        return;
    }
    if atime.is_some()
        || mtime.is_some()
        || flags.is_some()
        || mode.is_some_and(|v| v & !(0o777 | libc::S_IFMT) != 0)
    {
        reply.error(libc::EOPNOTSUPP);
        return;
    }
    let Some(size) = size else {
        let store = fs.store.clone();
        let handles = fs.handles.clone();
        let (uid, gid) = (fs.uid, fs.gid);
        fs.queue.submit(Box::new(move || {
            if let Some(fh) = fh {
                if !handles
                    .lock()
                    .unwrap()
                    .entries
                    .get(&fh)
                    .is_some_and(|h| h.ino == ino)
                {
                    reply.error(libc::EBADF);
                    return;
                }
            }
            let result = (|| -> drive_core::Result<_> {
                let mut s = store.lock().unwrap();
                let record = s.record_by_inode(ino)?;
                if let Some(mode) = mode {
                    s.set_local_mode(&record.node.id, mode & 0o777)?;
                }
                s.record_by_inode(ino)
            })();
            match result {
                Ok(record) => reply.attr(&TTL, &attr(&record, uid, gid)),
                Err(e) => reply.error(errno(&e)),
            }
        }));
        return;
    };
    // Combined chmod + truncate is not emitted by the supported Linux calls;
    // reject before touching contents instead of partially applying attributes.
    if mode.is_some() {
        reply.error(libc::EOPNOTSUPP);
        return;
    }
    let store = fs.store.clone();
    let handles = fs.handles.clone();
    let hydrator = fs.hydrator.clone();
    let network = fs.network_queue.clone();
    let (uid, gid) = (fs.uid, fs.gid);
    let updates = fs.handle_updates.clone();
    fs.queue.submit(Box::new(move || {
        let resolved = if let Some(fh) = fh {
            let h = handles.lock().unwrap();
            match h.entries.get(&fh) {
                Some(h) if h.ino == ino && h.flags & libc::O_ACCMODE != libc::O_RDONLY => {
                    Ok((h.node.clone(), h.content.clone()))
                }
                _ => Err(Error::Io(std::io::Error::from_raw_os_error(libc::EBADF))),
            }
        } else {
            content(&store.lock().unwrap(), ino)
        };
        let (id, body) = match resolved {
            Ok(v) => v,
            Err(e) => {
                reply.error(errno(&e));
                return;
            }
        };
        let missing = matches!(body, OpenContent::Missing(_)) && size != 0;
        let job: Job = Box::new(move || {
            let result = draft(&store, &handles, &updates, &hydrator, &id, body, size == 0)
                .and_then(|d| {
                    let mut s = store.lock().unwrap();
                    s.draft_truncate(&d, size).map_err(|e| errno(&e))?;
                    // Path-based truncate has no later file-handle fsync/release callback.
                    if fh.is_none() {
                        s.checkpoint_draft(&d).map_err(|e| errno(&e))?;
                        s.creation_saved(&d.node).map_err(|e| errno(&e))?;
                    }
                    s.record_for_handle(&id).map_err(|e| errno(&e))
                });
            match result {
                Ok(record) => {
                    if let Some(h) = hydrator {
                        h.local_saved()
                    }
                    reply.attr(&TTL, &attr(&record, uid, gid))
                }
                Err(e) => reply.error(e),
            }
        });
        if missing { network.submit(job) } else { job() }
    }));
}

pub(super) fn create(
    fs: &mut CacheFilesystem,
    parent: u64,
    name: &OsStr,
    mode: u32,
    umask: u32,
    flags: i32,
    reply: fuser::ReplyCreate,
) {
    if !fs.writable {
        reply.error(libc::EROFS);
        return;
    }
    let Some(name) = name.to_str().map(str::to_owned) else {
        reply.error(libc::EINVAL);
        return;
    };
    let store = fs.store.clone();
    let handles = fs.handles.clone();
    let (uid, gid) = (fs.uid, fs.gid);
    let updates = fs.handle_updates.clone();
    fs.queue.submit(Box::new(move || {
        let _update = updates.lock().unwrap();
        {
            let h = handles.lock().unwrap();
            if h.entries.len() >= 65536 || h.next == u64::MAX {
                reply.error(libc::EMFILE);
                return;
            }
        }
        let result = (|| -> drive_core::Result<_> {
            let mut s = store.lock().unwrap();
            let parent = s.record_by_inode(parent)?;
            let n = s.create_local_file(&parent.node.id, &name, mode & !umask)?;
            let source = s.open_cached(&n.id)?;
            let draft = Arc::new(s.begin_draft(&n.id, &n.version, Some(&source))?);
            let record = s.record(&n.id)?;
            Ok((record, draft))
        })();
        match result {
            Ok((record, draft)) => {
                let mut h = handles.lock().unwrap();
                let fh = h.next;
                h.next += 1;
                h.entries.insert(
                    fh,
                    Handle {
                        ino: record.ino,
                        node: record.node.id.clone(),
                        flags,
                        content: OpenContent::Draft(draft),
                    },
                );
                reply.created(
                    &TTL,
                    &attr(&record, uid, gid),
                    0,
                    fh,
                    fuser::consts::FOPEN_DIRECT_IO,
                )
            }
            Err(e) => reply.error(errno(&e)),
        }
    }));
}
