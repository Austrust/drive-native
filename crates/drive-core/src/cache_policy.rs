//! Local soft content budget. Never creates server mutations or deletes dirty/open drafts.
use crate::{Error, Result, Store};
use rusqlite::OptionalExtension;
use serde::Serialize;
use std::collections::HashSet;
pub const RECENT_DOWNLOAD_SECONDS: u64 = 60;
pub const MAX_CACHE_LIMIT: u64 = 1024 * 1024 * 1024 * 1024 * 1024;
/// Candidate discovery with every structural protection evaluated once as a
/// set instead of once per cached row. The previous correlated `NOT EXISTS`
/// forms rescanned `nodes` for each candidate (GB-scale page reads); the
/// protection sets here are scanned once and reused. Only handle, recency and
/// filesystem checks stay in Rust because they are not stored in SQL.
const CACHE_CANDIDATES_SQL: &str = concat!(
    "WITH RECURSIVE ",
    // Pinned nodes protect their whole subtree (is_pinned walks ancestors).
    "pinned(id) AS (SELECT id FROM nodes WHERE pinned=1 UNION SELECT n.id FROM pinned p CROSS JOIN nodes n ON n.parent=p.id), ",
    // Namespace overlay blocks upload for its whole subtree (blocks_upload).
    "upload_blocked(id) AS (SELECT node FROM namespace_overlay UNION SELECT n.id FROM upload_blocked b CROSS JOIN nodes n ON n.parent=b.id), ",
    // Unapplied engine namespace intents resolve through remote aliases.
    "engine_pending(node) AS (SELECT COALESCE(a.node,i.node) FROM engine_bridge_intents i LEFT JOIN remote_aliases a ON a.remote=i.node WHERE i.phase!='applied'), ",
    "ops_node(node) AS (SELECT DISTINCT node FROM operations WHERE node IS NOT NULL AND state IN ('pending','inflight','uncertain','conflict')), ",
    "local_node(node) AS (SELECT node FROM local_creations), ",
    // Operations and pending namespace replacements protect a shared blob.
    "protected_blobs(blob) AS (SELECT DISTINCT blob FROM operations WHERE blob IS NOT NULL AND state IN ('pending','inflight','uncertain','conflict') UNION SELECT json_extract(body,'$.blob') FROM namespace_ops WHERE kind='replace' AND state NOT IN ('committed','superseded') AND json_extract(body,'$.blob') IS NOT NULL), ",
    // A blob referenced by any other node (or by any dirty blob) is shared.
    "shared_blobs(blob) AS (SELECT blob FROM nodes WHERE blob IS NOT NULL GROUP BY blob HAVING COUNT(*)>1 UNION SELECT dirty_blob FROM nodes WHERE dirty_blob IS NOT NULL) ",
    "SELECT n.id,n.blob,COALESCE(p.at,0) FROM nodes n ",
    "LEFT JOIN cache_access a ON a.node=n.id ",
    "LEFT JOIN cache_published p ON p.node=n.id ",
    "WHERE n.directory=0 AND n.deleted=0 AND n.blob IS NOT NULL AND n.dirty_blob IS NULL ",
    "AND n.conflict=0 AND n.local_version=n.remote_version ",
    "AND NOT EXISTS(SELECT 1 FROM drafts d WHERE d.node=n.id AND d.dirty=1) ",
    "AND n.id NOT IN (SELECT id FROM access_blocked) ",
    "AND n.id NOT IN (SELECT id FROM pinned) ",
    "AND n.id NOT IN (SELECT id FROM upload_blocked) ",
    "AND n.id NOT IN (SELECT node FROM engine_pending WHERE node IS NOT NULL) ",
    "AND n.id NOT IN (SELECT node FROM ops_node) ",
    "AND n.id NOT IN (SELECT node FROM local_node) ",
    "AND n.blob NOT IN (SELECT blob FROM protected_blobs) ",
    "AND n.blob NOT IN (SELECT blob FROM shared_blobs) ",
    "ORDER BY COALESCE(a.seq,0),n.id",
);
#[derive(Clone, Debug, Serialize)]
pub struct CacheUsage {
    pub limit_bytes: u64,
    pub content_bytes: u64,
    pub reclaimable_bytes: u64,
    pub protected_bytes: u64,
    pub over_target_bytes: u64,
}
#[derive(Debug, Serialize)]
pub struct CacheTrim {
    pub revision: u64,
    pub usage: CacheUsage,
    pub released: Vec<String>,
    pub reclaimed_bytes: u64,
}
impl Store {
    pub fn downloads_paused(&self) -> Result<bool> {
        let value: Option<String> = self.db.query_row(
            "SELECT value FROM metadata WHERE key='official_downloads_paused'", [], |r| r.get(0)
        ).optional()?;
        match value.as_deref() {
            None | Some("0") => Ok(false), Some("1") => Ok(true),
            _ => Err(Error::State("invalid download pause setting".into())),
        }
    }
    pub fn set_downloads_paused(&mut self, value: bool) -> Result<()> {
        self.db.execute("INSERT INTO metadata(key,value) VALUES('official_downloads_paused',?1) ON CONFLICT(key) DO UPDATE SET value=excluded.value", [if value {"1"} else {"0"}])?;
        Ok(())
    }
    /// Zero disables automatic eviction. Existing profiles retain all contents
    /// until the user explicitly enables a limit in settings.
    pub fn cache_limit(&self) -> Result<u64> {
        let value: Option<String> = self
            .db
            .query_row(
                "SELECT value FROM metadata WHERE key='cache_limit_bytes'",
                [],
                |r| r.get(0),
            )
            .optional()?;
        let limit = value
            .map(|v| {
                v.parse::<u64>()
                    .map_err(|_| Error::State("invalid cache policy".into()))
            })
            .transpose()?
            .unwrap_or(0);
        if limit > MAX_CACHE_LIMIT {
            return Err(Error::State("cache limit out of range".into()));
        }
        Ok(limit)
    }
    pub fn set_cache_limit(&mut self, bytes: u64) -> Result<()> {
        if bytes > MAX_CACHE_LIMIT {
            return Err(Error::State("cache limit out of range".into()));
        }
        self.db.execute("INSERT INTO metadata(key,value) VALUES('cache_limit_bytes',?1) ON CONFLICT(key) DO UPDATE SET value=excluded.value",[bytes.to_string()])?;
        Ok(())
    }
    /// Actual file opens only; icon/status and directory queries must not call it.
    pub fn note_cache_use(&self, id: &str) -> Result<()> {
        self.db.execute(
            "INSERT OR REPLACE INTO cache_access(node) SELECT id FROM nodes WHERE id=?1",
            [id],
        )?;
        Ok(())
    }
    /// Protection shared by explicit release and automatic eviction. Open draft
    /// handles, unresolved namespace work and uncertain commits all retain data.
    pub(crate) fn cache_is_protected(&self, id: &str) -> Result<bool> {
        if self.dirty_draft(id)?
            || self.draft_conflicts(id)?
            || self.is_pinned(id)?
            || crate::namespace::blocks_upload(&self.db, id)?
            || self.engine_namespace_pending(id)?
        {
            return Ok(true);
        }
        if self
            .handles
            .lock()
            .unwrap()
            .values()
            .any(|(node, _)| node == id)
        {
            return Ok(true);
        }
        Ok(self.db.query_row("SELECT EXISTS(SELECT 1 FROM operations WHERE node=?1 AND state IN ('pending','inflight','uncertain','conflict')) OR EXISTS(SELECT 1 FROM local_creations WHERE node=?1)",[id],|r|r.get(0))?)
    }
    fn cache_candidates(&self, now: u64, active: &HashSet<String>) -> Result<Vec<(String, u64)>> {
        let mut q = self.db.prepare(CACHE_CANDIDATES_SQL)?;
        let rows = q
            .query_map([], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, i64>(2)?,
                ))
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        let mut result = Vec::new();
        for (id, blob, published) in rows {
            if active.contains(&id) {
                continue;
            }
            {
                // Answer for open handles; every other `cache_is_protected`
                // clause is already enforced by the SQL above.
                let handles = self.handles.lock().unwrap();
                if handles.contains_key(&blob) || handles.values().any(|(node, _)| node == &id) {
                    continue;
                }
            }
            if published > 0 && now.saturating_sub(published as u64) < RECENT_DOWNLOAD_SECONDS {
                continue;
            }
            let path = self.root.join("blobs").join(&blob);
            if !path.is_file() {
                return Err(Error::State("referenced content missing on disk".into()));
            }
            let metadata = std::fs::symlink_metadata(&path)?;
            if !metadata.is_file() {
                return Err(Error::State("invalid cache content file".into()));
            }
            result.push((id, metadata.len()));
        }
        Ok(result)
    }
    /// Pre-rewrite per-candidate implementation, kept only so tests can prove
    /// the set-based query selects exactly the same candidates.
    #[cfg(test)]
    fn cache_candidates_legacy(
        &self,
        now: u64,
        active: &HashSet<String>,
    ) -> Result<Vec<(String, u64)>> {
        let mut q=self.db.prepare("SELECT n.id,n.blob,COALESCE(p.at,0) FROM nodes n LEFT JOIN cache_access a ON a.node=n.id LEFT JOIN cache_published p ON p.node=n.id WHERE n.id NOT IN (SELECT id FROM access_blocked) AND n.directory=0 AND n.deleted=0 AND n.blob IS NOT NULL AND n.dirty_blob IS NULL AND n.conflict=0 AND n.local_version=n.remote_version AND NOT EXISTS(SELECT 1 FROM nodes other WHERE other.id!=n.id AND (other.blob=n.blob OR other.dirty_blob=n.blob)) AND NOT EXISTS(SELECT 1 FROM operations o WHERE o.blob=n.blob AND o.state IN ('pending','inflight','uncertain','conflict')) AND NOT EXISTS(SELECT 1 FROM namespace_ops o WHERE o.kind='replace' AND o.state NOT IN ('committed','superseded') AND json_extract(o.body,'$.blob')=n.blob) ORDER BY COALESCE(a.seq,0),n.id")?;
        let rows = q
            .query_map([], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, i64>(2)?,
                ))
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        let mut result = Vec::new();
        for (id, blob, published) in rows {
            if active.contains(&id)
                || self.handles.lock().unwrap().contains_key(&blob)
                || self.cache_is_protected(&id)?
                || published > 0 && now.saturating_sub(published as u64) < RECENT_DOWNLOAD_SECONDS
            {
                continue;
            }
            if self.status(&id)? != crate::Status::LocalComplete {
                continue;
            }
            let metadata = std::fs::symlink_metadata(self.root.join("blobs").join(&blob))?;
            if !metadata.is_file() {
                return Err(Error::State("invalid cache content file".into()));
            }
            result.push((id, metadata.len()));
        }
        Ok(result)
    }
    /// Logical content bytes, not filesystem allocation or NAS quota. Drafts,
    /// older versions and temporary content count toward usage but remain protected.
    fn content_bytes(&self) -> Result<u64> {
        let mut bytes = 0u64;
        for name in ["blobs", "drafts"] {
            let entries = match std::fs::read_dir(self.root.join(name)) {
                Ok(v) => v,
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
                Err(e) => return Err(e.into()),
            };
            for entry in entries {
                let entry = entry?;
                if entry.file_type()?.is_file() {
                    bytes = bytes
                        .checked_add(entry.metadata()?.len())
                        .ok_or_else(|| Error::State("content size overflow".into()))?;
                }
            }
        }
        Ok(bytes)
    }
    pub fn cache_usage(&self, now: u64, active: &HashSet<String>) -> Result<CacheUsage> {
        let limit_bytes = self.cache_limit()?;
        let content_bytes = self.content_bytes()?;
        let reclaimable_bytes = self
            .cache_candidates(now, active)?
            .iter()
            .map(|(_, size)| *size)
            .sum::<u64>()
            .min(content_bytes);
        Ok(CacheUsage {
            limit_bytes,
            content_bytes,
            reclaimable_bytes,
            protected_bytes: content_bytes - reclaimable_bytes,
            over_target_bytes: if limit_bytes == 0 {
                0
            } else {
                content_bytes.saturating_sub(limit_bytes)
            },
        })
    }
    /// Official-engine mode must release both its private copy and our blob.
    /// Queue durable two-phase releases; never call the local-only trim path.
    /// Pending requests count toward the target but not toward reclaimed bytes.
    pub fn queue_cache_budget(&mut self, now: u64, active: &HashSet<String>) -> Result<Vec<String>> {
        let limit = self.cache_limit()?;
        if limit == 0 { return Ok(Vec::new()); }
        let mut projected = self.content_bytes()?;
        let mut queued = Vec::new();
        for (id, size) in self.cache_candidates(now, active)? {
            if projected <= limit { break; }
            let node = self.node(&id)?;
            if self.cache_release(&id)?.is_some_and(|r| r.version == node.version && r.state == "pending" && r.attempts < 5) {
                projected = projected.saturating_sub(size);
                continue;
            }
            // A retained/failed release is visible to the user. Automatic policy
            // must not reset its retry count on each scheduler wakeup.
            if self.cache_release(&id)?.is_some_and(|r| r.version == node.version && r.attempts >= 5) { continue; }
            self.begin_cache_release(&id)?;
            queued.push(id);
            projected = projected.saturating_sub(size);
            if queued.len() == 256 { break; }
        }
        Ok(queued)
    }
    pub fn trim_cache(&mut self, now: u64, active: &HashSet<String>) -> Result<CacheTrim> {
        let limit = self.cache_limit()?;
        if limit == 0 {
            return Ok(CacheTrim {
                revision: self.local_revision(),
                usage: self.cache_usage(now, active)?,
                released: Vec::new(),
                reclaimed_bytes: 0,
            });
        }
        let mut reclaimed = self.collect_garbage()?;
        let mut bytes = self.content_bytes()?;
        let mut released = Vec::new();
        if bytes > limit {
            // Bound metadata changes per pass; physical GC runs once, not once
            // per file. Shared/orphan references may require another pass.
            for (id, size) in self.cache_candidates(now, active)?.into_iter().take(256) {
                if bytes <= limit {
                    break;
                }
                self.release(&id)?;
                released.push(id);
                bytes = bytes.saturating_sub(size);
            }
            reclaimed += self.collect_garbage()?;
        }
        Ok(CacheTrim {
            revision: self.local_revision(),
            usage: self.cache_usage(now, active)?,
            released,
            reclaimed_bytes: reclaimed,
        })
    }
}

#[cfg(test)]
#[path = "cache_policy_tests.rs"]
mod tests;
