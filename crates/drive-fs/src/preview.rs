//! Explicit metadata-only candidate routing. Never turn a candidate into a Node
//! accepted by the hydration or mutation paths.
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
}
pub(super) fn metadata(s: &Store, ino: u64) -> drive_core::Result<Metadata> {
    match s.posix_record_by_inode(ino) {
        Ok(r) => Ok(Metadata::Authority(r)),
        Err(e) if e.is_not_found() => s.preview_record_by_inode(ino).map(Metadata::Preview),
        Err(e) => Err(e),
    }
}
pub(super) fn lookup(s: &Store, parent: u64, name: &str) -> drive_core::Result<Metadata> {
    // One statement resolves the authority parent, ENOTDIR and the child
    // projection. Only a missing authority answer can be a cold preview parent,
    // and that fallback keeps the original candidate-namespace errors.
    match s.posix_lookup_by_inode(parent, name) {
        Ok(r) => Ok(Metadata::Authority(r)),
        Err(e) if e.is_not_found() => match s.preview_record_by_inode(parent) {
            Ok(_) => s.preview_lookup(parent, name).map(Metadata::Preview),
            Err(_) => Err(e),
        },
        Err(e) => Err(e),
    }
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
