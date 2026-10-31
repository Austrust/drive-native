//! Prepare immutable downloaded content without retaining the Store writer.
//! Reservations use the same retention table as open immutable ReadHandles.
use crate::{DownloadTicket, Error, Result, Store};
use rusqlite::params;
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

type Retention = Arc<Mutex<HashMap<String, (String, usize)>>>;

/// Called only during Store::open after obtaining its exclusive writer lease.
/// Never recurse or follow a staging symlink, and only remove our generated
/// ordinary temporary names. Dirty drafts and pending upload blobs never use
/// this generated temporary prefix.
pub(crate) fn cleanup_staging(root: &Path) -> Result<()> {
    let staging = root.join("blobs");
    let metadata = match std::fs::symlink_metadata(&staging) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error.into()),
    };
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err(Error::State(
            "download staging must be a real directory".into(),
        ));
    }
    let mut removed = false;
    for entry in std::fs::read_dir(&staging)? {
        let entry = entry?;
        let name = entry.file_name();
        let generated = name
            .to_str()
            .and_then(|name| name.strip_prefix(".drive-download-"))
            .and_then(|name| name.strip_suffix(".part"))
            .is_some_and(|random| {
                random.len() == 12 && random.bytes().all(|b| b.is_ascii_alphanumeric())
            });
        if generated && entry.file_type()?.is_file() {
            std::fs::remove_file(entry.path())?;
            removed = true;
        }
    }
    if removed {
        File::open(staging)?.sync_all()?;
    }
    Ok(())
}

pub struct DownloadPreparation {
    root: PathBuf,
    retention: Retention,
    blob: String,
    ticket: DownloadTicket,
    previous: (Option<String>, Option<String>, i64),
}
/// Only a successful, durable preparation can construct this value.
pub struct PreparedDownload {
    preparation: DownloadPreparation,
}
impl Drop for DownloadPreparation {
    fn drop(&mut self) {
        let mut handles = self.retention.lock().unwrap_or_else(|e| e.into_inner());
        if let Some((_, count)) = handles.get_mut(&self.blob) {
            *count -= 1;
            if *count == 0 {
                handles.remove(&self.blob);
            }
        }
        // Never remove a published blob here: commit may have succeeded even
        // when its caller subsequently failed. Unreferenced bodies go to GC.
    }
}
impl DownloadPreparation {
    /// Caller holds no Store guard. The cancellation predicate must not do
    /// unbounded work; it is checked between chunks and durability boundaries.
    pub fn prepare<R: Read>(
        self,
        source: &mut R,
        size: u64,
        expected_sha256: &str,
        cancelled: impl Fn() -> bool,
    ) -> Result<PreparedDownload> {
        let check = || -> Result<()> {
            if cancelled() {
                Err(std::io::Error::from(std::io::ErrorKind::Interrupted).into())
            } else {
                Ok(())
            }
        };
        check()?;
        let parent = self.root.join("blobs");
        // GC only deletes regular UUID filenames. Our fixed non-UUID prefix
        // protects temporary data on the exact destination filesystem.
        let mut temporary = tempfile::Builder::new()
            .prefix(".drive-download-")
            .suffix(".part")
            .rand_bytes(12)
            .tempfile_in(&parent)?;
        let mut digest = Sha256::new();
        let mut total = 0u64;
        let mut buffer = [0u8; 65536];
        loop {
            check()?;
            let count = source.read(&mut buffer)?;
            if count == 0 {
                break;
            }
            total = total.checked_add(count as u64).ok_or(Error::Integrity)?;
            if total > size {
                return Err(Error::Integrity);
            }
            temporary.write_all(&buffer[..count])?;
            digest.update(&buffer[..count]);
        }
        if total != size || format!("{:x}", digest.finalize()) != expected_sha256 {
            return Err(Error::Integrity);
        }
        check()?;
        temporary.as_file().sync_all()?;
        check()?;
        // The UUID was reserved while holding Store, before any concurrent GC
        // could capture its keep set. The temporary name is not a UUID.
        temporary
            .persist_noclobber(parent.join(&self.blob))
            .map_err(|e| e.error)?;
        File::open(&parent)?.sync_all()?;
        check()?;
        Ok(PreparedDownload { preparation: self })
    }
}
impl Store {
    /// Short metadata-only capture, called under the Store serialization domain.
    pub fn prepare_download(&self, ticket: &DownloadTicket) -> Result<DownloadPreparation> {
        let node = self.node(&ticket.id)?;
        if node.directory || node.version != ticket.version || self.dirty_draft(&ticket.id)? {
            return Err(Error::State(
                "download target changed or has a working copy".into(),
            ));
        }
        let previous = self.db.query_row(
            "SELECT blob,local_version,dirty_generation FROM nodes WHERE id=?1 AND remote_version=?2 AND dirty_blob IS NULL AND deleted=0 AND directory=0",
            params![ticket.id, ticket.version], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )?;
        let blob = uuid::Uuid::new_v4().to_string();
        let mut handles = self.handles.lock().unwrap();
        if handles.contains_key(&blob) {
            return Err(Error::State("content reservation collision".into()));
        }
        handles.insert(blob.clone(), (ticket.id.clone(), 1));
        Ok(DownloadPreparation {
            root: self.root.clone(),
            retention: self.handles.clone(),
            blob,
            ticket: DownloadTicket {
                id: ticket.id.clone(),
                version: ticket.version.clone(),
            },
            previous,
        })
    }
    /// Short publication boundary. Prepared bytes and the directory entry are
    /// already durable; SQLite commit is the first point making them visible.
    pub fn publish_prepared_download(&mut self, prepared: PreparedDownload) -> Result<()> {
        let body = &prepared.preparation;
        if !Arc::ptr_eq(&body.retention, &self.handles) {
            return Err(Error::State(
                "prepared download belongs to another Store lifetime".into(),
            ));
        }
        let node = self.node(&body.ticket.id)?;
        if node.directory
            || node.version != body.ticket.version
            || self.dirty_draft(&body.ticket.id)?
        {
            return Err(Error::State(
                "download target changed or has a working copy".into(),
            ));
        }
        let tx = self.db.transaction()?;
        let changed = tx.execute(
            "UPDATE nodes SET blob=?3,local_version=?2 WHERE id=?1 AND remote_version=?2 AND dirty_blob IS NULL AND deleted=0 AND directory=0 AND blob IS ?4 AND local_version IS ?5 AND dirty_generation=?6",
            params![body.ticket.id,body.ticket.version,body.blob,body.previous.0,body.previous.1,body.previous.2],
        )?;
        if changed != 1 {
            return Err(Error::State(
                "prepared download was superseded by another local publication".into(),
            ));
        }
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            .min(i64::MAX as u64) as i64;
        tx.execute("INSERT INTO cache_published(node,at) VALUES(?1,?2) ON CONFLICT(node) DO UPDATE SET at=excluded.at", params![body.ticket.id, now])?;
        tx.commit()?;
        Ok(())
    }
}
