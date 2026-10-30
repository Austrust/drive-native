//! Durable mutable working copies. Immutable upload snapshots remain separate.
use crate::{Error, ReadHandle, Result, Store};
use rusqlite::{OptionalExtension, params};
use std::{
    fs::{File, OpenOptions},
    io::{Seek, SeekFrom},
    os::unix::fs::FileExt,
};
type Handles = std::sync::Arc<std::sync::Mutex<std::collections::HashMap<String, (String, usize)>>>;
pub struct DraftHandle {
    pub node: String,
    pub token: String,
    pub version: String,
    file: File,
    handles: Handles,
}
impl Drop for DraftHandle {
    fn drop(&mut self) {
        let mut h = self.handles.lock().unwrap();
        let key = format!("draft:{}", self.token);
        if let Some((_, count)) = h.get_mut(&key) {
            *count -= 1;
            if *count == 0 {
                h.remove(&key);
            }
        }
    }
}
impl DraftHandle {
    pub fn read_at(&self, out: &mut [u8], offset: u64) -> std::io::Result<usize> {
        self.file.read_at(out, offset)
    }
    pub fn size(&self) -> std::io::Result<u64> {
        Ok(self.file.metadata()?.len())
    }
    pub fn mtime(&self) -> std::io::Result<u64> {
        Ok(self
            .file
            .metadata()?
            .modified()?
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs())
    }
}
impl Store {
    fn retain_draft(&self, id: &str, token: String, version: String, file: File) -> DraftHandle {
        self.handles
            .lock()
            .unwrap()
            .entry(format!("draft:{token}"))
            .or_insert_with(|| (id.into(), 0))
            .1 += 1;
        DraftHandle {
            node: id.into(),
            token,
            version,
            file,
            handles: self.handles.clone(),
        }
    }
    fn draft_path(&self, token: &str) -> Result<std::path::PathBuf> {
        if uuid::Uuid::parse_str(token).is_err() {
            return Err(Error::State("invalid draft identity".into()));
        }
        Ok(self.root.join("drafts").join(token))
    }
    pub fn dirty_draft(&self, id: &str) -> Result<bool> {
        Ok(self.db.query_row(
            "SELECT EXISTS(SELECT 1 FROM drafts WHERE node=?1 AND dirty=1)",
            [id],
            |r| r.get(0),
        )?)
    }
    pub fn draft_conflicts(&self, id: &str) -> Result<bool> {
        Ok(self.db.query_row("SELECT EXISTS(SELECT 1 FROM drafts d JOIN nodes n ON n.id=d.node WHERE d.node=?1 AND d.dirty=1 AND NOT EXISTS(SELECT 1 FROM local_unlinks WHERE node=d.node) AND (d.base!=n.remote_version OR n.deleted=1))",[id],|r|r.get(0))?)
    }
    fn draft_handle(&self, id: &str, dirty_only: bool) -> Result<Option<DraftHandle>> {
        let found: Option<(String, String)> = self
            .db
            .query_row(
                "SELECT token,base FROM drafts WHERE node=?1 AND (?2=0 OR dirty=1)",
                params![id, dirty_only],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()?;
        match found {
            Some((token, version)) => {
                let file = File::open(self.draft_path(&token)?)?;
                Ok(Some(self.retain_draft(id, token, version, file)))
            }
            None => Ok(None),
        }
    }
    /// Immutable snapshot for a closed-file conflict decision. Caller holds the
    /// Store writer and has rejected active file handles before entering here.
    pub(crate) fn snapshot_dirty_draft(&self, id: &str) -> Result<Option<String>> {
        let token: Option<String> = self
            .db
            .query_row(
                "SELECT token FROM drafts WHERE node=?1 AND dirty=1",
                [id],
                |r| r.get(0),
            )
            .optional()?;
        let Some(token) = token else { return Ok(None) };
        let mut file = File::open(self.draft_path(&token)?)?;
        file.sync_all()?;
        Ok(Some(self.write_blob_reader(&mut file, None)?))
    }
    pub fn open_dirty_draft(&self, id: &str) -> Result<Option<DraftHandle>> {
        self.draft_handle(id, true)
    }
    /// Source is an immutable handle captured when the application opened the file.
    /// None is permitted only for an explicit O_TRUNC, never a missing download.
    pub fn begin_draft(
        &mut self,
        id: &str,
        base: &str,
        source: Option<&ReadHandle>,
    ) -> Result<DraftHandle> {
        let normalized = crate::namespace::normalized_version(&self.db, id, base)?;
        let base = normalized.as_str();
        let node = self.node_for_handle(id)?;
        if node.directory {
            return Err(Error::State("cannot edit directory body".into()));
        }
        if let Some(existing) = self.draft_handle(id, false)? {
            if existing.version == base {
                return Ok(existing);
            }
            if self.dirty_draft(id)? {
                return Err(Error::Io(std::io::Error::from_raw_os_error(libc::ESTALE)));
            }
        }
        std::fs::create_dir_all(self.root.join("drafts"))?;
        let token = uuid::Uuid::new_v4().to_string();
        let path = self.root.join("drafts").join(&token);
        let mut file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .read(true)
            .open(&path)?;
        if let Some(source) = source {
            if crate::namespace::normalized_version(&self.db, id, source.version())? != base {
                return Err(Error::State("draft source/base mismatch".into()));
            }
            let mut offset = 0;
            let mut buf = [0; 65536];
            loop {
                let count = source.read_at(&mut buf, offset)?;
                if count == 0 {
                    break;
                }
                std::io::Write::write_all(&mut file, &buf[..count])?;
                offset += count as u64;
            }
        }
        file.sync_all()?;
        File::open(self.root.join("drafts"))?.sync_all()?;
        self.db.execute("INSERT INTO drafts(node,token,base,generation,checkpoint,dirty) VALUES(?1,?2,?3,0,0,0) ON CONFLICT(node) DO UPDATE SET token=excluded.token,base=excluded.base,generation=0,checkpoint=0,dirty=0",params![id,token,base])?;
        Ok(self.retain_draft(id, token, base.into(), file))
    }
    fn editing_file(&mut self, draft: &DraftHandle) -> Result<File> {
        let changed=self.db.execute("UPDATE drafts SET generation=generation+1,dirty=1 WHERE node=?1 AND token=?2 AND EXISTS(SELECT 1 FROM nodes WHERE id=?1 AND (deleted=0 OR EXISTS(SELECT 1 FROM local_unlinks WHERE node=?1)))",params![draft.node,draft.token])?;
        if changed != 1 {
            return Err(Error::Io(std::io::Error::from_raw_os_error(libc::ESTALE)));
        }
        Ok(OpenOptions::new()
            .read(true)
            .write(true)
            .open(self.draft_path(&draft.token)?)?)
    }
    pub fn draft_write(
        &mut self,
        draft: &DraftHandle,
        offset: u64,
        data: &[u8],
        append: bool,
    ) -> Result<usize> {
        let f = self.editing_file(draft)?;
        let offset = if append { f.metadata()?.len() } else { offset };
        Ok(f.write_at(data, offset)?)
    }
    pub fn draft_truncate(&mut self, draft: &DraftHandle, size: u64) -> Result<()> {
        self.editing_file(draft)?.set_len(size)?;
        Ok(())
    }
    /// Local durability barrier. A failed snapshot leaves the durable draft intact
    /// and prevents an older snapshot being uploaded as the latest saved content.
    pub fn checkpoint_draft(&mut self, draft: &DraftHandle) -> Result<Option<String>> {
        let (base, generation, checkpoint, dirty): (String, i64, i64, bool) = self.db.query_row(
            "SELECT base,generation,checkpoint,dirty FROM drafts WHERE node=?1 AND token=?2",
            params![draft.node, draft.token],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )?;
        if !dirty || generation == checkpoint {
            return Ok(None);
        }
        let mut f = File::open(self.draft_path(&draft.token)?)?;
        f.sync_all()?;
        if self.locally_unlinked(&draft.node)? {
            self.db.execute(
                "UPDATE drafts SET checkpoint=?3 WHERE node=?1 AND token=?2",
                params![draft.node, draft.token, generation],
            )?;
            return Ok(None);
        }
        f.seek(SeekFrom::Start(0))?;
        let op = self.save_local_reader_from_version(&draft.node, &base, &mut f)?;
        self.db.execute(
            "UPDATE drafts SET checkpoint=?3 WHERE node=?1 AND token=?2",
            params![draft.node, draft.token, generation],
        )?;
        Ok(Some(op))
    }
    pub fn checkpoint_recovered_drafts(&mut self) -> Result<usize> {
        let ids = {
            let mut q = self
                .db
                .prepare("SELECT node FROM drafts WHERE dirty=1 AND generation!=checkpoint")?;
            q.query_map([], |r| r.get::<_, String>(0))?
                .collect::<std::result::Result<Vec<_>, _>>()?
        };
        let mut count = 0;
        for id in ids {
            if let Some(draft) = self.open_dirty_draft(&id)? {
                if self.checkpoint_draft(&draft)?.is_some() {
                    count += 1
                }
            }
        }
        Ok(count)
    }
}
