//! Explicit metadata-only candidate routing. Never turn a candidate into a Node
//! accepted by the hydration or mutation paths.
use super::*;
use drive_core::preview::PreviewRecord;

pub(super) enum Metadata {
    Authority(FileRecord),
    Preview(PreviewRecord),
}
impl Metadata {
    pub(super) fn attr(&self, uid: u32, gid: u32) -> FileAttr {
        match self {
            Self::Authority(r) => attr(r, uid, gid),
            Self::Preview(r) => preview_attr(r, uid, gid),
        }
    }
}
pub(super) fn metadata(s: &Store, ino: u64) -> drive_core::Result<Metadata> {
    match s.record_by_inode(ino) {
        Ok(r) => Ok(Metadata::Authority(r)),
        Err(e) if e.is_not_found() => s.preview_record_by_inode(ino).map(Metadata::Preview),
        Err(e) => Err(e),
    }
}
pub(super) fn lookup(s: &Store, parent: u64, name: &str) -> drive_core::Result<Metadata> {
    match metadata(s, parent)? {
        Metadata::Authority(p) => {
            if !p.node.directory {
                return Err(Error::Io(std::io::Error::from_raw_os_error(libc::ENOTDIR)));
            }
            s.lookup(&p.node.id, name).map(Metadata::Authority)
        }
        Metadata::Preview(_) => s.preview_lookup(parent, name).map(Metadata::Preview),
    }
}
/// Caller holds the Store guard through its subsequent mutation. Preview nodes
/// cannot acquire content handles; after atomic promotion new calls use authority.
pub(super) fn authoritative(
    s: &Store,
    ino: u64,
    preview_error: i32,
) -> drive_core::Result<FileRecord> {
    match metadata(s, ino)? {
        Metadata::Authority(r) => Ok(r),
        Metadata::Preview(_) => Err(Error::Io(std::io::Error::from_raw_os_error(preview_error))),
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
