//! Explicit metadata-only candidate routing. Never turn a candidate into a Node
//! accepted by the hydration or mutation paths.
//!
//! Authority metadata is cheap enough to answer on the FUSE dispatcher thread:
//! the single-statement projections below replace a submit + channel wakeup +
//! worker projection + reply round trip. The inline path only runs while the
//! Store lock is immediately available; it releases the lock before writing the
//! reply and never touches the candidate namespace (preview mounts and preview
//! fallbacks stay on the offloaded path, which owns the reply-while-locked
//! publication contract).
use super::*;
use drive_core::preview::PreviewRecord;

pub(super) enum Metadata {
    Authority(drive_core::filesystem::PosixRecord),
    Preview(PreviewRecord),
}
impl Metadata {
    pub(super) fn attr(&self, uid: u32, gid: u32) -> FileAttr {
        match self {
            Self::Authority(r) => posix_attr(r, uid, gid),
            Self::Preview(r) => preview_attr(r, uid, gid),
        }
    }
    /// Only authoritative, mutable-body-independent metadata may be cached in
    /// the kernel; candidate previews stay zero-TTL.
    pub(super) fn cacheable(&self) -> bool {
        match self {
            Self::Authority(r) => r.cacheable,
            Self::Preview(_) => false,
        }
    }
}

/// (entry TTL, attr TTL) for one positive entry.
///
/// `cache_entry` is false for candidate-preview answers: they must stay
/// zero-TTL. Positive names always revalidate: an old kernel dentry must not
/// redirect a new path-based open after a remote move. Only clean authority
/// attributes may use the independently configured TTL.
pub(super) fn reply_ttls(
    hub: &NotifyHub,
    cacheable: bool,
    cache_entry: bool,
) -> (Duration, Duration) {
    let ttl = hub.ttl();
    if ttl.is_zero() || !cache_entry {
        return (Duration::ZERO, Duration::ZERO);
    }
    (Duration::ZERO, if cacheable { ttl } else { Duration::ZERO })
}
pub(super) fn metadata(s: &Store, ino: u64) -> drive_core::Result<Metadata> {
    match s.posix_record_by_inode(ino) {
        Ok(mut r) => {permissions::project(s,&mut r)?;Ok(Metadata::Authority(r))},
        Err(e) if e.is_not_found() => s.preview_record_by_inode(ino).map(Metadata::Preview),
        Err(e) => Err(e),
    }
}
pub(super) fn lookup(s: &Store, parent: u64, name: &str) -> drive_core::Result<Metadata> {
    // Check the parent even when the requested child does not exist. A denied
    // traversal must not leak the child's existence through ENOENT.
    permissions::inode(s, parent, permissions::Action::Traverse)?;
    // One statement resolves the authority parent, ENOTDIR and the child
    // projection. Only a missing authority answer can be a cold preview parent,
    // and that fallback keeps the original candidate-namespace errors.
    match s.posix_lookup_by_inode(parent, name) {
        Ok(mut r) => {permissions::project(s,&mut r)?;Ok(Metadata::Authority(r))},
        Err(e) if e.is_not_found() => match s.preview_record_by_inode(parent) {
            Ok(_) => s.preview_lookup(parent, name).map(Metadata::Preview),
            // Keep the candidate-namespace error code (ESTALE/EACCES/...) that
            // the old metadata(parent) routing propagated instead of flattening
            // a stale preview parent into ENOENT.
            Err(pe) => Err(pe),
        },
        Err(e) => Err(e),
    }
}

/// Try to answer a POSIX lookup on the dispatcher thread.
///
/// `None` means the request must be handed to the metadata pool: the Store is
/// busy, the parent may be a candidate-preview directory, or the child does not
/// exist (negative lookups also probe the candidate namespace).
pub(super) fn try_inline_lookup(
    store: &drive_core::SharedStore,
    uid: u32,
    gid: u32,
    parent: u64,
    name: &str,
) -> Option<Result<(FileAttr, bool), i32>> {
    let guard = match store.try_lock() {
        Ok(guard) => guard,
        // A held Store is exactly the case the pool exists for.
        Err(std::sync::TryLockError::WouldBlock) => return None,
        // Poisoned stores recover in the offloaded path, which reports a single
        // request result; never swallow the poison inline.
        Err(std::sync::TryLockError::Poisoned(_)) => return None,
    };
    diagnostics::count(diagnostics::Stage::LookupInline);
    if store.has_permission_denials() {
        if let Err(error) = permissions::inode(&guard, parent, permissions::Action::Traverse) {
            return Some(Err(errno(&error)));
        }
    }
    let projection = diagnostics::span(diagnostics::Stage::LookupProjection);
    let result = guard.posix_lookup_by_inode(parent, name);
    drop(projection);
    match result {
        Ok(mut record) => {
            if store.has_permission_denials() {if let Err(e)=permissions::project(&guard,&mut record){return Some(Err(errno(&e)));}}
            let attr = posix_attr(&record, uid, gid);
            let cacheable = record.cacheable;
            // The projection is fully owned; do not hold Store across the reply.
            drop(guard);
            Some(Ok((attr, cacheable)))
        }
        Err(e) if e.is_not_found() => {
            drop(guard);
            None
        }
        Err(e) => {
            drop(guard);
            Some(Err(errno(&e)))
        }
    }
}

/// Try to answer a POSIX getattr on the dispatcher thread. Missing inodes fall
/// back to the pool so the candidate-preview routing keeps its own contract.
pub(super) fn try_inline_getattr(
    store: &drive_core::SharedStore,
    handles: &Mutex<Handles>,
    uid: u32,
    gid: u32,
    ino: u64,
    fh: Option<u64>,
) -> Option<Result<(FileAttr, bool), i32>> {
    let guard = match store.try_lock() {
        Ok(guard) => guard,
        Err(std::sync::TryLockError::WouldBlock) => return None,
        Err(std::sync::TryLockError::Poisoned(_)) => return None,
    };
    diagnostics::count(diagnostics::Stage::GetattrInline);
    let projection = diagnostics::span(diagnostics::Stage::GetattrProjection);
    let result = guard.posix_record_by_inode(ino);
    drop(projection);
    let mut record = match result {
        Ok(record) => record,
        Err(e) if e.is_not_found() => {
            drop(guard);
            return None;
        }
        Err(e) => {
            drop(guard);
            return Some(Err(errno(&e)));
        }
    };
    if store.has_permission_denials() {if let Err(e)=permissions::project(&guard,&mut record){return Some(Err(errno(&e)));}}
    let mut attr = posix_attr(&record, uid, gid);
    // Same order as the offloaded handler: Store first, then the handle table,
    // and neither is held while the reply is written.
    drop(guard);
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
                            attr.size = size;
                            attr.blocks = size.div_ceil(512);
                        }
                        Err(e) => return Some(Err(e.raw_os_error().unwrap_or(libc::EIO))),
                    }
                }
                if let OpenContent::Cached(content) = &content {
                    match content.size() {
                        Ok(s) => {
                            attr.size = s;
                            attr.blocks = s.div_ceil(512)
                        }
                        Err(e) => return Some(Err(e.raw_os_error().unwrap_or(libc::EIO))),
                    }
                }
            }
            None => return Some(Err(libc::EBADF)),
        }
    }
    Some(Ok((attr, record.cacheable)))
}
/// Caller holds the Store guard through its subsequent mutation. Preview nodes
/// cannot acquire content handles; after atomic promotion new calls use authority.
pub(super) fn authoritative(
    s: &Store,
    ino: u64,
    preview_error: i32,
) -> drive_core::Result<FileRecord> {
    match s.record_by_inode(ino) {
        Ok(r) => Ok(r),
        Err(e) if e.is_not_found() => {
            s.preview_record_by_inode(ino)?;
            Err(Error::Io(std::io::Error::from_raw_os_error(preview_error)))
        }
        Err(e) => Err(e),
    }
}
pub(super) fn posix_attr(r: &drive_core::filesystem::PosixRecord, uid: u32, gid: u32) -> FileAttr {
    let time = UNIX_EPOCH
        .checked_add(Duration::from_secs(r.mtime))
        .unwrap_or(UNIX_EPOCH);
    let atime = UNIX_EPOCH
        .checked_add(Duration::new(r.atime, r.atime_nsec))
        .unwrap_or(UNIX_EPOCH);
    FileAttr {
        ino: r.ino,
        size: r.size,
        blocks: r.blocks,
        atime,
        mtime: time,
        ctime: time,
        crtime: UNIX_EPOCH,
        kind: if r.directory {
            FileType::Directory
        } else {
            FileType::RegularFile
        },
        perm: r.mode as u16,
        nlink: if r.unlinked {
            0
        } else if r.directory {
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
pub(super) fn preview_attr(r: &PreviewRecord, uid: u32, gid: u32) -> FileAttr {
    let t = UNIX_EPOCH
        .checked_add(Duration::from_secs(r.mtime))
        .unwrap_or(UNIX_EPOCH);
    FileAttr {
        ino: r.ino,
        size: r.size,
        blocks: 0,
        atime: t,
        mtime: t,
        ctime: t,
        crtime: t,
        kind: if r.directory {
            FileType::Directory
        } else {
            FileType::RegularFile
        },
        // Permission bits describe this view, not the lifetime of the RW mount.
        // TTL=0 permits the kernel to revalidate after authoritative promotion.
        perm: (r.mode & 0o555) as u16,
        nlink: if r.directory { 2 } else { 1 },
        uid,
        gid,
        rdev: 0,
        blksize: 4096,
        flags: 0,
    }
}

#[cfg(test)]
mod ttl_tests {
    use super::*;

    #[test]
    fn entry_ttl_is_always_zero_and_only_clean_authority_attributes_cache() {
        let inactive = NotifyHub::with_ttl_ms(2000);
        assert_eq!(
            reply_ttls(&inactive, true, true),
            (Duration::ZERO, Duration::ZERO)
        );
        inactive.activate_ttl_for_tests(2000);
        // Configured TTL applies only to clean attributes, never names.
        assert_eq!(
            reply_ttls(&inactive, true, true),
            (Duration::ZERO, Duration::from_millis(2000))
        );
        // Dirty/drafted/unlinked authority: both fields revalidate.
        assert_eq!(
            reply_ttls(&inactive, false, true),
            (Duration::ZERO, Duration::ZERO)
        );
        // Candidate preview never caches.
        assert_eq!(
            reply_ttls(&inactive, true, false),
            (Duration::ZERO, Duration::ZERO)
        );
        // Explicit TTL=0 disables everything (safe-stage equivalence).
        let disabled = NotifyHub::with_ttl_ms(0);
        disabled.activate_ttl_for_tests(0);
        assert_eq!(
            reply_ttls(&disabled, true, true),
            (Duration::ZERO, Duration::ZERO)
        );
    }
}

/// Serialize a candidate directory reply with ACL, generation and promotion
/// checks. A zero-nodeid dirent still discloses a name; it is not an ACL bypass.
/// The callback must send one bounded reply and must not re-enter Store.
pub(super) fn publish_view(
    store: &drive_core::SharedStore,
    stamp: &drive_core::preview::PreviewStamp,
    send: impl FnOnce(drive_core::Result<()>),
) {
    match store.lock() {
        Ok(guard) => {
            send(guard.preview_validate_view(stamp));
            drop(guard);
        }
        Err(_) => send(Err(Error::Io(std::io::Error::from_raw_os_error(libc::EIO)))),
    }
}

#[cfg(test)]
#[path = "preview_publication_tests.rs"]
mod publication_tests;
