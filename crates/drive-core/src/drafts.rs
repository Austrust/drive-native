//! Durable mutable working copies. Immutable upload snapshots remain separate.
use crate::{Error, ReadHandle, Result, Store};
use rusqlite::{OptionalExtension, params};
use serde::{Deserialize, Serialize};
use std::{
    fs::{File, OpenOptions},
    io::{Seek, SeekFrom},
    os::unix::fs::FileExt,
};
type Handles = std::sync::Arc<std::sync::Mutex<std::collections::HashMap<String, (String, usize)>>>;

// ---- issue #60: drafts quota + orphan quarantine ---------------------------

/// Default drafts quota: `min(5 GiB, 2 % of the filesystem total)`. The quota
/// is a soft admission/diagnostic bound; it never hard-deletes user state.
pub const DEFAULT_DRAFT_QUOTA_CAP_BYTES: u64 = 5 * 1024 * 1024 * 1024;
pub const DEFAULT_DRAFT_QUOTA_PERCENT: u64 = 2;
/// A working copy with no queue/namespace parent, a deleted node and no open
/// handle is an orphan. It is only touched after this age window.
pub const DRAFT_ORPHAN_AGE_SECONDS: u64 = 30 * 24 * 60 * 60;
/// Recoverable holding area: orphan bodies are moved here, never deleted.
pub const DRAFT_QUARANTINE_DIR: &str = "drafts-quarantine";
/// Metadata key holding the quarantine index (token/node/bytes/age). A
/// recoverable quarantine needs no schema change.
const DRAFT_QUARANTINE_KEY: &str = "drafts_quarantined";
/// Metadata key for an explicit drafts quota override. 0 disables the quota
/// report (the orphan GC itself still uses the age window).
const DRAFT_QUOTA_KEY: &str = "draft_quota_bytes";

pub fn default_draft_quota_bytes(total_disk_bytes: u64) -> u64 {
    if total_disk_bytes == 0 {
        return DEFAULT_DRAFT_QUOTA_CAP_BYTES;
    }
    let percent = u128::from(total_disk_bytes) * u128::from(DEFAULT_DRAFT_QUOTA_PERCENT) / 100;
    u64::try_from(percent.min(u128::from(DEFAULT_DRAFT_QUOTA_CAP_BYTES)))
        .unwrap_or(DEFAULT_DRAFT_QUOTA_CAP_BYTES)
}

/// Per-category drafts accounting for `GetCacheSettings`/overview (#60).
#[derive(Clone, Debug, Default, Serialize)]
pub struct DraftUsage {
    pub total_bytes: u64,
    pub count: u64,
    /// Working copies with no `operations` and no `namespace_ops` row at all:
    /// invisible to every queue/health surface before this field existed.
    pub unreferenced_bytes: u64,
    pub unreferenced_count: u64,
    pub quota_bytes: u64,
    pub over_quota_bytes: u64,
    pub quarantine_bytes: u64,
    pub quarantine_count: u64,
    /// Orphan bodies already past the age window (eligible for quarantine now)
    /// and the rest (still protected by age/open-handle/queue references).
    pub reclaimable_bytes: u64,
    pub protected_bytes: u64,
}

/// One working copy moved to the recoverable quarantine. Only size/age are
/// exposed; the token is already a random local identity, not a file name.
#[derive(Clone, Debug, Serialize)]
pub struct DraftQuarantineEntry {
    pub token: String,
    pub node: String,
    pub bytes: u64,
    pub quarantined_at: u64,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct DraftQuarantine {
    pub quarantined: Vec<DraftQuarantineEntry>,
    pub quarantined_bytes: u64,
    pub skipped_open: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct QuarantinedDraftRecord {
    token: String,
    node: String,
    bytes: u64,
    quarantined_at: u64,
}

#[derive(Clone, Debug)]
struct DraftRowInfo {
    node: String,
    dirty: bool,
    has_operations: bool,
    has_namespace_ops: bool,
    has_local_creation: bool,
    node_deleted: bool,
}
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
        let modified = file.metadata()?.modified()?;
        let blob = self.write_blob_reader(&mut file, None)?;
        let snapshot = File::open(self.root.join("blobs").join(&blob))?;
        snapshot.set_modified(modified)?;
        snapshot.sync_all()?;
        Ok(Some(blob))
    }
    pub fn open_dirty_draft(&self, id: &str) -> Result<Option<DraftHandle>> {
        self.draft_handle(id, true)
    }
    /// Register a newly opened handle with an already promoted working copy,
    /// including the interval before its first write. Older opened versions
    /// must not be rebound to a draft based on newer remote content.
    pub fn open_draft_for_version(&self, id: &str, base: &str) -> Result<Option<DraftHandle>> {
        let base = crate::namespace::normalized_version(&self.db, id, base)?;
        Ok(self.draft_handle(id, false)?.filter(|d| d.version == base))
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
        self.draft_stats.draft_file_fsyncs += 2;
        self.db.execute("INSERT INTO drafts(node,token,base,generation,checkpoint,dirty) VALUES(?1,?2,?3,0,0,0) ON CONFLICT(node) DO UPDATE SET token=excluded.token,base=excluded.base,generation=0,checkpoint=0,dirty=0",params![id,token,base])?;
        // Issue #60: a draft created for an already-deleted node with no queued
        // operation can never be uploaded. Count it at creation time so this
        // class of orphan is visible instead of silently accumulating.
        if self.draft_created_for_tombstone(id)? {
            self.draft_stats.unreferenced_draft_begins += 1;
        }
        Ok(self.retain_draft(id, token, base.into(), file))
    }
    /// Resolve the working file for an edit and durably mark the edit epoch.
    ///
    /// The durable statement is issued only on the first edit after a
    /// checkpoint: subsequent writes in the same dirty epoch do not commit
    /// anything until one of them crosses a durability boundary (fsync/flush,
    /// close, checkpoint). Before #24 every 4 KiB write autocommitted a
    /// generation bump, which forced one WAL fsync per syscall (100 MiB
    /// sequential write: 800 commits / ~817 fsyncs). The first dirty mark is
    /// still committed before the writer returns, so an unclean reopen can
    /// recover the draft and the upload gate still refuses an older snapshot.
    fn editing_file(&mut self, draft: &DraftHandle) -> Result<File> {
        let row: Option<(i64, i64, bool)> = self
            .db
            .query_row(
                "SELECT d.generation,d.checkpoint,d.dirty FROM drafts d JOIN nodes n ON n.id=d.node \
                 WHERE d.node=?1 AND d.token=?2 \
                 AND (n.deleted=0 OR EXISTS(SELECT 1 FROM local_unlinks WHERE node=n.id))",
                params![draft.node, draft.token],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .optional()?;
        let Some((generation, checkpoint, dirty)) = row else {
            return Err(Error::Io(std::io::Error::from_raw_os_error(libc::ESTALE)));
        };
        if !dirty || generation == checkpoint {
            let changed = self.db.execute(
                "UPDATE drafts SET generation=generation+1,dirty=1 WHERE node=?1 AND token=?2",
                params![draft.node, draft.token],
            )?;
            if changed != 1 {
                return Err(Error::Io(std::io::Error::from_raw_os_error(libc::ESTALE)));
            }
            self.draft_stats.metadata_commits += 1;
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
        self.draft_stats.writes += 1;
        Ok(f.write_at(data, offset)?)
    }
    pub fn draft_truncate(&mut self, draft: &DraftHandle, size: u64) -> Result<()> {
        self.editing_file(draft)?.set_len(size)?;
        Ok(())
    }
    /// Existing handles retain the same working inode; the generation prevents
    /// an older upload acknowledgement from clearing this timestamp edit.
    /// No explicit fsync here: a timestamp edit is only observable through a
    /// checkpoint, and that checkpoint fsyncs the same inode afterwards (#24).
    pub fn draft_set_mtime(&mut self, draft: &DraftHandle, seconds: u32) -> Result<()> {
        let file = self.editing_file(draft)?;
        file.set_modified(
            std::time::UNIX_EPOCH + std::time::Duration::from_secs(u64::from(seconds)),
        )?;
        Ok(())
    }
    fn checkpoint_needed(&self, draft: &DraftHandle) -> Result<bool> {
        let needed: i64 = self.db.query_row(
            "SELECT EXISTS(SELECT 1 FROM drafts WHERE node=?1 AND token=?2 AND dirty=1 AND generation!=checkpoint)",
            params![draft.node, draft.token],
            |r| r.get(0),
        )?;
        Ok(needed != 0)
    }
    /// Complete a checkpoint whose working-file fsync already happened.
    ///
    /// The `checkpoint=generation` advance is folded into the snapshot
    /// transaction (`save_draft_file_from_version`), so the snapshot and the
    /// draft row become durable together.
    fn checkpoint_after_sync(&mut self, draft: &DraftHandle) -> Result<Option<String>> {
        let row: Option<(String, i64, i64, bool)> = self
            .db
            .query_row(
                "SELECT base,generation,checkpoint,dirty FROM drafts WHERE node=?1 AND token=?2",
                params![draft.node, draft.token],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
            )
            .optional()?;
        let Some((base, generation, checkpoint, dirty)) = row else {
            return Ok(None);
        };
        if !dirty || generation == checkpoint {
            return Ok(None);
        }
        if self.locally_unlinked(&draft.node)? {
            self.db.execute(
                "UPDATE drafts SET checkpoint=?3 WHERE node=?1 AND token=?2",
                params![draft.node, draft.token, generation],
            )?;
            self.draft_stats.metadata_commits += 1;
            return Ok(None);
        }
        let mut f = File::open(self.draft_path(&draft.token)?)?;
        f.seek(SeekFrom::Start(0))?;
        let op = self.save_draft_file_from_version(
            &draft.node,
            &base,
            &mut f,
            Some((&draft.token, generation)),
        )?;
        // The checkpoint advance is one statement folded into the snapshot
        // transaction, so it still costs one durable metadata commit.
        self.draft_stats.metadata_commits += 1;
        self.draft_stats.checkpoints += 1;
        Ok(Some(op))
    }
    /// Local durability barrier. A failed snapshot leaves the durable draft intact
    /// and prevents an older snapshot being uploaded as the latest saved content.
    pub fn checkpoint_draft(&mut self, draft: &DraftHandle) -> Result<Option<String>> {
        if !self.checkpoint_needed(draft)? {
            return Ok(None);
        }
        File::open(self.draft_path(&draft.token)?)?.sync_all()?;
        self.draft_stats.draft_file_fsyncs += 1;
        self.checkpoint_after_sync(draft)
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
    // ---- issue #60: drafts quota, orphan quarantine and recovery ----------

    /// Effective drafts quota in bytes: an explicit `draft_quota_bytes`
    /// metadata override wins, otherwise `min(5 GiB, 2 % disk)`. `0` means the
    /// quota is reported as disabled; the orphan GC keeps using its age window.
    pub fn draft_quota(&self) -> Result<u64> {
        let value: Option<String> = self
            .db
            .query_row(
                "SELECT value FROM metadata WHERE key=?1",
                [DRAFT_QUOTA_KEY],
                |r| r.get(0),
            )
            .optional()?;
        match value {
            None => Ok(self.default_draft_quota),
            Some(value) => value
                .parse::<u64>()
                .map_err(|_| Error::State("invalid drafts quota".into())),
        }
    }
    pub fn set_draft_quota(&mut self, bytes: u64) -> Result<()> {
        if bytes > crate::cache_policy::MAX_CACHE_LIMIT {
            return Err(Error::State("drafts quota out of range".into()));
        }
        self.db.execute(
            "INSERT INTO metadata(key,value) VALUES(?1,?2) ON CONFLICT(key) DO UPDATE SET value=excluded.value",
            params![DRAFT_QUOTA_KEY, bytes.to_string()],
        )?;
        Ok(())
    }
    fn draft_dir_entries(&self, dir: &str) -> Result<Vec<(String, u64, u64)>> {
        let mut files = Vec::new();
        let entries = match std::fs::read_dir(self.root.join(dir)) {
            Ok(entries) => entries,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(files),
            Err(error) => return Err(error.into()),
        };
        for entry in entries {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().into_owned();
            if uuid::Uuid::parse_str(&name).is_err() || !entry.file_type()?.is_file() {
                continue;
            }
            let metadata = entry.metadata()?;
            let modified = metadata
                .modified()
                .ok()
                .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|elapsed| elapsed.as_secs())
                .unwrap_or(0);
            files.push((name, metadata.len(), modified));
        }
        Ok(files)
    }
    fn draft_rows(&self) -> Result<std::collections::HashMap<String, DraftRowInfo>> {
        let mut query = self.db.prepare(
            "SELECT d.token,d.node,d.dirty, \
                    EXISTS(SELECT 1 FROM operations o WHERE o.node=d.node), \
                    EXISTS(SELECT 1 FROM namespace_ops n WHERE n.node=d.node), \
                    EXISTS(SELECT 1 FROM local_creations c WHERE c.node=d.node), \
                    COALESCE(n.deleted,1) \
               FROM drafts d LEFT JOIN nodes n ON n.id=d.node",
        )?;
        let rows = query
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    DraftRowInfo {
                        node: row.get(1)?,
                        dirty: row.get(2)?,
                        has_operations: row.get(3)?,
                        has_namespace_ops: row.get(4)?,
                        has_local_creation: row.get(5)?,
                        node_deleted: row.get(6)?,
                    },
                ))
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(rows.into_iter().collect())
    }
    fn open_draft_tokens(&self) -> std::collections::HashSet<String> {
        self.handles
            .lock()
            .unwrap()
            .keys()
            .filter_map(|key| key.strip_prefix("draft:").map(str::to_owned))
            .collect()
    }
    /// An orphan is a dirty working copy whose node is a tombstone, that no
    /// `operations`/`namespace_ops`/`local_creations` row references, with no
    /// open handle, past the age window. Unuploaded edits for live nodes are
    /// protected and never touched by this rule.
    fn draft_is_orphan(
        &self,
        info: &DraftRowInfo,
        open: &std::collections::HashSet<String>,
        token: &str,
        modified: u64,
        now: u64,
    ) -> bool {
        info.dirty
            && info.node_deleted
            && !info.has_operations
            && !info.has_namespace_ops
            && !info.has_local_creation
            && !open.contains(token)
            && now.saturating_sub(modified) >= DRAFT_ORPHAN_AGE_SECONDS
    }
    fn quarantine_records(&self) -> Result<Vec<QuarantinedDraftRecord>> {
        let value: Option<String> = self
            .db
            .query_row(
                "SELECT value FROM metadata WHERE key=?1",
                [DRAFT_QUARANTINE_KEY],
                |r| r.get(0),
            )
            .optional()?;
        Ok(value
            .and_then(|value| serde_json::from_str(&value).ok())
            .unwrap_or_default())
    }
    fn set_quarantine_records(&self, records: &[QuarantinedDraftRecord]) -> Result<()> {
        self.db.execute(
            "INSERT INTO metadata(key,value) VALUES(?1,?2) ON CONFLICT(key) DO UPDATE SET value=excluded.value",
            params![DRAFT_QUARANTINE_KEY, serde_json::to_string(records)?],
        )?;
        Ok(())
    }
    /// Per-category drafts accounting: totals, queue-unreferenced bytes (the
    /// 1.571 GB invisible class), quota pressure and the recoverable quarantine.
    pub fn draft_usage(&self, now: u64) -> Result<DraftUsage> {
        let mut usage = DraftUsage {
            quota_bytes: self.draft_quota()?,
            ..DraftUsage::default()
        };
        let rows = self.draft_rows()?;
        let open = self.open_draft_tokens();
        for (token, bytes, modified) in self.draft_dir_entries("drafts")? {
            usage.total_bytes = usage.total_bytes.saturating_add(bytes);
            usage.count += 1;
            let info = rows.get(&token);
            let unreferenced = info
                .map(|info| !info.has_operations && !info.has_namespace_ops)
                .unwrap_or(true);
            if unreferenced {
                usage.unreferenced_bytes = usage.unreferenced_bytes.saturating_add(bytes);
                usage.unreferenced_count += 1;
            }
            if info.is_some_and(|info| self.draft_is_orphan(info, &open, &token, modified, now)) {
                usage.reclaimable_bytes = usage.reclaimable_bytes.saturating_add(bytes);
            }
        }
        usage.protected_bytes = usage.total_bytes.saturating_sub(usage.reclaimable_bytes);
        usage.over_quota_bytes = if usage.quota_bytes == 0 {
            0
        } else {
            usage.total_bytes.saturating_sub(usage.quota_bytes)
        };
        for record in self.quarantine_records()? {
            let path = self.root.join(DRAFT_QUARANTINE_DIR).join(&record.token);
            if let Ok(metadata) = std::fs::metadata(&path) {
                if metadata.is_file() {
                    usage.quarantine_bytes = usage.quarantine_bytes.saturating_add(metadata.len());
                    usage.quarantine_count += 1;
                }
            }
        }
        Ok(usage)
    }
    /// Move orphan bodies into the recoverable quarantine directory. Nothing is
    /// deleted: `restore_quarantined_drafts` puts a body back into `drafts/`.
    /// Only bodies past [`DRAFT_ORPHAN_AGE_SECONDS`] with no queue reference move.
    pub fn collect_orphan_drafts(&mut self, now: u64) -> Result<DraftQuarantine> {
        let rows = self.draft_rows()?;
        let open = self.open_draft_tokens();
        let mut records = self.quarantine_records()?;
        let mut result = DraftQuarantine::default();
        let quarantine_dir = self.root.join(DRAFT_QUARANTINE_DIR);
        for (token, bytes, modified) in self.draft_dir_entries("drafts")? {
            let Some(info) = rows.get(&token) else {
                continue;
            };
            if open.contains(&token) {
                result.skipped_open += 1;
                continue;
            }
            if !self.draft_is_orphan(info, &open, &token, modified, now) {
                continue;
            }
            std::fs::create_dir_all(&quarantine_dir)?;
            let destination = quarantine_dir.join(&token);
            if destination.exists() {
                continue;
            }
            std::fs::rename(self.root.join("drafts").join(&token), &destination)?;
            self.draft_stats.orphan_drafts_quarantined += 1;
            records.push(QuarantinedDraftRecord {
                token: token.clone(),
                node: info.node.clone(),
                bytes,
                quarantined_at: now,
            });
            result.quarantined.push(DraftQuarantineEntry {
                token,
                node: info.node.clone(),
                bytes,
                quarantined_at: now,
            });
            result.quarantined_bytes = result.quarantined_bytes.saturating_add(bytes);
        }
        if !result.quarantined.is_empty() {
            self.set_quarantine_records(&records)?;
            #[cfg(unix)]
            {
                File::open(self.root.join("drafts"))?.sync_all()?;
                File::open(&quarantine_dir)?.sync_all()?;
            }
        }
        Ok(result)
    }
    /// Undo [`Self::collect_orphan_drafts`]: move every indexed body back into
    /// `drafts/` without overwriting a newer working copy. Returns how many
    /// bodies were restored.
    pub fn restore_quarantined_drafts(&mut self) -> Result<u64> {
        let records = self.quarantine_records()?;
        if records.is_empty() {
            return Ok(0);
        }
        let quarantine_dir = self.root.join(DRAFT_QUARANTINE_DIR);
        let drafts_dir = self.root.join("drafts");
        std::fs::create_dir_all(&drafts_dir)?;
        let mut restored = 0u64;
        let mut remaining = Vec::new();
        for record in records {
            let source = quarantine_dir.join(&record.token);
            let destination = drafts_dir.join(&record.token);
            if !source.is_file() || destination.exists() {
                remaining.push(record);
                continue;
            }
            std::fs::rename(&source, &destination)?;
            self.draft_stats.quarantined_drafts_restored += 1;
            restored += 1;
        }
        self.set_quarantine_records(&remaining)?;
        Ok(restored)
    }
    /// Creation-time detection (#60): a working copy created for a node that is
    /// already a tombstone has no live namespace object to attach to. Typical
    /// path: the user deletes a file that still has an open write handle, so a
    /// later write creates a draft on the tombstone. Counting it here makes the
    /// class visible instead of only appearing as unexplained draft bytes.
    fn draft_created_for_tombstone(&self, id: &str) -> Result<bool> {
        Ok(self.db.query_row(
            "SELECT NOT EXISTS(SELECT 1 FROM nodes WHERE id=?1 AND deleted=0)",
            [id],
            |r| r.get::<_, bool>(0),
        )?)
    }
    /// Issue #24 diagnostics: monotonic counters for the draft write path.
    pub fn draft_write_stats(&self) -> crate::DraftWriteStats {
        self.draft_stats
    }
}

/// Checkpoint a draft with the fsync barrier outside the store lock.
///
/// Phase one briefly locks the store to decide whether work is pending; phase
/// two runs `sync_all` on the working file without the store lock, so flushing
/// a large dirty draft does not stall unrelated readers and writers; phase
/// three re-locks, re-validates the row (another actor may have checkpointed
/// in between) and copies the synced file into an immutable snapshot.
///
/// Safety: every draft write runs while holding the store lock, so bytes
/// written between the fsync and the snapshot copy are also copied; the copy
/// itself cannot race a writer. A crash after the fsync but before the
/// snapshot transaction commits leaves `generation != checkpoint`, which
/// startup recovery treats as a dirty draft and re-checkpoints.
pub fn checkpoint_draft_shared(
    shared: &crate::SharedStore,
    draft: &DraftHandle,
) -> Result<Option<String>> {
    let needed = {
        let guard = shared
            .lock()
            .map_err(|_| Error::State("store lock poisoned".into()))?;
        guard.checkpoint_needed(draft)?
    };
    if !needed {
        return Ok(None);
    }
    // Same inode as every write through this handle: this is the barrier that
    // used to run while the store lock was held.
    draft.file.sync_all()?;
    let mut guard = shared
        .lock()
        .map_err(|_| Error::State("store lock poisoned".into()))?;
    guard.draft_stats.draft_file_fsyncs += 1;
    guard.checkpoint_after_sync(draft)
}

#[cfg(test)]
#[path = "draft_fsync_tests.rs"]
mod draft_fsync_tests;
