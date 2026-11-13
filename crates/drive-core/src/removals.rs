//! Local unlink is durable before the path disappears. Native deletion is a
//! separate, ordered operation and requires an identity-specific tombstone.
use crate::{Error, Node, Result, Store};
use rusqlite::{OptionalExtension, params};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RemoveJob {
    pub operation: String,
    pub node: Node,
    pub path: String,
    pub cursor: String,
}
#[derive(Clone, Debug)]
pub struct QueuedRemove {
    pub job: RemoveJob,
    pub state: String,
}
pub(crate) fn is_unlinked(db: &rusqlite::Connection, id: &str) -> Result<bool> {
    Ok(db.query_row(
        "SELECT EXISTS(SELECT 1 FROM local_unlinks WHERE node=?1)",
        [id],
        |r| r.get(0),
    )?)
}
/// A pending remove intent that an upload must wait for because it owns the
/// upload target: the same identity, an ancestor identity, or an overlapping
/// path. Unrelated removes (for example freedesktop trash entries) never
/// appear here.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UploadRemoveBarrier {
    pub operation: String,
    pub path: String,
}
#[cfg(test)]
fn path_related(a: &str, b: &str) -> bool {
    a == b
        || a.strip_prefix(b).is_some_and(|rest| rest.starts_with('/'))
        || b.strip_prefix(a).is_some_and(|rest| rest.starts_with('/'))
}
/// Batch path resolution for every queued remove intent. The old barrier walked
/// each remove's ancestors with one query per level; resolving a whole backlog
/// shares one query per depth instead, so the first barrier after a bulk delete
/// stays bounded too.
pub(crate) fn live_paths(db: &rusqlite::Connection, ids: &[String]) -> Result<Vec<Option<String>>> {
    let mut nodes: HashMap<String, (Option<String>, String)> = HashMap::new();
    let mut level: Vec<String> = ids.to_vec();
    let mut depth = 0usize;
    while !level.is_empty() {
        depth += 1;
        if depth > 256 {
            return Err(Error::State("remove path cycle".into()));
        }
        let mut next = std::collections::HashSet::new();
        for chunk in level.chunks(500) {
            let placeholders = vec!["?"; chunk.len()].join(",");
            let sql = format!("SELECT id,parent,name FROM nodes WHERE id IN ({placeholders})");
            let mut query = db.prepare(&sql)?;
            let mut rows = query.query(rusqlite::params_from_iter(chunk.iter()))?;
            while let Some(row) = rows.next()? {
                let id: String = row.get(0)?;
                let parent: Option<String> = row.get(1)?;
                let name: String = row.get(2)?;
                if let Some(parent) = &parent {
                    next.insert(parent.clone());
                }
                nodes.insert(id, (parent, name));
            }
        }
        level = next
            .into_iter()
            .filter(|id| !nodes.contains_key(id))
            .collect();
        level.sort();
    }
    let mut paths = Vec::with_capacity(ids.len());
    for id in ids {
        let mut parts = Vec::new();
        let mut current = id.as_str();
        let mut seen = std::collections::HashSet::new();
        let mut resolved = true;
        loop {
            if !seen.insert(current.to_owned()) {
                return Err(Error::State("remove path cycle".into()));
            }
            match nodes.get(current) {
                None => {
                    resolved = false;
                    break;
                }
                Some((parent, name)) => match parent {
                    // The task root's own name is not part of relative paths.
                    Some(parent) => {
                        parts.push(name.clone());
                        current = parent;
                    }
                    None => break,
                },
            }
        }
        if resolved {
            parts.reverse();
            paths.push(Some(parts.join("/")));
        } else {
            paths.push(None);
        }
    }
    Ok(paths)
}
/// Tracks the unfinished remove intents incrementally. Rebuilding is driven by
/// TEMP bookkeeping installed on the writer connection: `ops` changes whenever
/// the queued membership or its state changes, `paths` whenever a node's parent
/// or name changes (or a node row disappears), because paths must then be
/// rebound. Batch local deletes only flip `nodes.deleted`, so draining a large
/// backlog never invalidates the path index.
#[derive(Default)]
pub(crate) struct RemoveBarrierCache {
    ops_version: i64,
    paths_version: i64,
    initialized: bool,
    rows: Vec<CachedRemove>,
    by_node: HashMap<String, usize>,
    by_path: BTreeMap<String, usize>,
    calls: u64,
    rebuilds: u64,
    last_rows: usize,
    last_us: u64,
    /// Structured `key=value` rebuild lines. Production writes them to stderr
    /// (journald); tests capture the same lines in-process through the accessor.
    #[cfg(test)]
    diagnostics: Vec<String>,
}
struct CachedRemove {
    seq: i64,
    operation: String,
    node: String,
    captured_path: Option<String>,
    live_path: Option<String>,
}
impl CachedRemove {
    fn path(&self) -> &str {
        self.live_path
            .as_deref()
            .or(self.captured_path.as_deref())
            .unwrap_or("")
    }
}
/// Install the TEMP versions used by [`RemoveBarrierCache`]. Only the writer
/// connection needs them; derived state never touches the durable schema.
pub(crate) fn install_barrier_versions(db: &rusqlite::Connection) -> Result<()> {
    db.execute_batch(
        "CREATE TEMP TABLE IF NOT EXISTS dn_barrier_version(ops INTEGER NOT NULL, paths INTEGER NOT NULL);
         INSERT INTO dn_barrier_version(ops,paths) SELECT 0,0 WHERE NOT EXISTS(SELECT 1 FROM dn_barrier_version);
         CREATE TEMP TRIGGER IF NOT EXISTS dn_barrier_ops_insert AFTER INSERT ON main.namespace_ops BEGIN UPDATE dn_barrier_version SET ops=ops+1; END;
         CREATE TEMP TRIGGER IF NOT EXISTS dn_barrier_ops_update AFTER UPDATE OF kind, state, node ON main.namespace_ops BEGIN UPDATE dn_barrier_version SET ops=ops+1; END;
         CREATE TEMP TRIGGER IF NOT EXISTS dn_barrier_ops_delete AFTER DELETE ON main.namespace_ops BEGIN UPDATE dn_barrier_version SET ops=ops+1; END;
         CREATE TEMP TRIGGER IF NOT EXISTS dn_barrier_paths_update AFTER UPDATE OF parent,name ON main.nodes WHEN OLD.parent IS NOT NEW.parent OR OLD.name IS NOT NEW.name BEGIN UPDATE dn_barrier_version SET paths=paths+1; END;
         CREATE TEMP TRIGGER IF NOT EXISTS dn_barrier_paths_delete AFTER DELETE ON main.nodes BEGIN UPDATE dn_barrier_version SET paths=paths+1; END;",
    )?;
    Ok(())
}
impl RemoveBarrierCache {
    fn refresh(&mut self, db: &rusqlite::Connection) -> Result<()> {
        let (ops, paths): (i64, i64) =
            db.query_row("SELECT ops,paths FROM dn_barrier_version", [], |r| {
                Ok((r.get(0)?, r.get(1)?))
            })?;
        if self.initialized && ops == self.ops_version && paths == self.paths_version {
            return Ok(());
        }
        let paths_changed = paths != self.paths_version;
        let mut added = Vec::new();
        if !self.initialized || ops != self.ops_version {
            let fresh: Vec<(i64, String, String)> = {
                let mut q = db.prepare("SELECT seq,id,node FROM namespace_ops WHERE kind='remove' AND state NOT IN ('committed','superseded') ORDER BY seq")?;
                q.query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?
                    .collect::<std::result::Result<Vec<_>, _>>()?
            };
            let mut cached: HashMap<String, CachedRemove> = self
                .rows
                .drain(..)
                .map(|row| (row.operation.clone(), row))
                .collect();
            let mut rows = Vec::with_capacity(fresh.len());
            for (seq, operation, node) in fresh {
                match cached.remove(&operation) {
                    Some(mut row) => {
                        row.seq = seq;
                        rows.push(row);
                    }
                    None => {
                        added.push(rows.len());
                        rows.push(CachedRemove {
                            seq,
                            operation,
                            node,
                            captured_path: None,
                            live_path: None,
                        });
                    }
                }
            }
            self.rows = rows;
        }
        let targets: Vec<usize> = if paths_changed {
            (0..self.rows.len()).collect()
        } else {
            added
        };
        if !targets.is_empty() {
            let ids: Vec<String> = targets.iter().map(|&i| self.rows[i].node.clone()).collect();
            let resolved =
                live_paths(db, &ids).map_err(|_| Error::State("remove path unreadable".into()))?;
            for (&index, path) in targets.iter().zip(resolved) {
                self.rows[index].live_path = path;
            }
        }
        // Captured paths are only needed when a node row disappeared; loading
        // their bodies lazily keeps the common bulk-delete rebuild cheap.
        for index in 0..self.rows.len() {
            if self.rows[index].live_path.is_none() && self.rows[index].captured_path.is_none() {
                let operation = self.rows[index].operation.clone();
                let body: String = db.query_row(
                    "SELECT body FROM namespace_ops WHERE id=?1",
                    [&operation],
                    |r| r.get(0),
                )?;
                let job: RemoveJob = serde_json::from_str(&body)
                    .map_err(|_| Error::State("unreadable remove intent".into()))?;
                if job.operation != operation {
                    return Err(Error::State("remove intent identity mismatch".into()));
                }
                self.rows[index].captured_path = Some(job.path);
            }
        }
        self.by_node.clear();
        self.by_path.clear();
        for (index, row) in self.rows.iter().enumerate() {
            self.by_node.entry(row.node.clone()).or_insert(index);
            self.by_path.entry(row.path().to_owned()).or_insert(index);
        }
        self.ops_version = ops;
        self.paths_version = paths;
        self.initialized = true;
        self.rebuilds += 1;
        Ok(())
    }
    fn barrier(
        &self,
        db: &rusqlite::Connection,
        node: &str,
        path: &str,
    ) -> Result<Option<UploadRemoveBarrier>> {
        if self.rows.is_empty() {
            return Ok(None);
        }
        let mut candidates = Vec::new();
        let mut ancestors = std::collections::HashSet::new();
        let mut current = Some(node.to_owned());
        while let Some(id) = current {
            if ancestors.len() >= 256 || !ancestors.insert(id.clone()) {
                return Err(Error::State("upload dependency ancestry invalid".into()));
            }
            if let Some(&index) = self.by_node.get(&id) {
                candidates.push(index);
            }
            current = db
                .query_row("SELECT parent FROM nodes WHERE id=?1", [&id], |r| {
                    r.get::<_, Option<String>>(0)
                })
                .optional()?
                .flatten();
        }
        let mut prefix = String::new();
        for part in path.split('/') {
            if !prefix.is_empty() {
                prefix.push('/');
            }
            prefix.push_str(part);
            if let Some(&index) = self.by_path.get(&prefix) {
                candidates.push(index);
            }
        }
        let descendants = format!("{path}/");
        for (_, &index) in self.by_path.range(descendants.clone()..) {
            if !self.rows[index].path().starts_with(&descendants) {
                break;
            }
            candidates.push(index);
        }
        if candidates.is_empty() {
            return Ok(None);
        }
        candidates.sort_unstable();
        candidates.dedup();
        // Blocked operations are derived through a recursive view; permission
        // blocks are rare, so only pay for them when denied roots exist.
        let denied: bool = db.query_row("SELECT EXISTS(SELECT 1 FROM access_denied)", [], |r| {
            r.get(0)
        })?;
        let blocked = if denied {
            let mut q = db.prepare("SELECT operation FROM access_blocked_operations")?;
            q.query_map([], |r| r.get::<_, String>(0))?
                .collect::<std::result::Result<std::collections::HashSet<_>, _>>()?
        } else {
            std::collections::HashSet::new()
        };
        for index in candidates {
            let row = &self.rows[index];
            if blocked.contains(&row.operation) {
                continue;
            }
            return Ok(Some(UploadRemoveBarrier {
                operation: row.operation.clone(),
                path: row.path().to_owned(),
            }));
        }
        Ok(None)
    }
}
impl Store {
    /// Scope the remove barrier to dependencies of one upload target. It runs on
    /// the writer connection, so the TEMP versions above let it answer from an
    /// incremental index instead of materializing every queued intent.
    pub(crate) fn upload_remove_barrier(
        &mut self,
        node: &str,
        path: &str,
    ) -> Result<Option<UploadRemoveBarrier>> {
        let started = std::time::Instant::now();
        let rebuilds_before = self.barrier.rebuilds;
        self.barrier.refresh(&self.db)?;
        let blocked = self.barrier.barrier(&self.db, node, path)?;
        let cache = &mut self.barrier;
        cache.calls += 1;
        cache.last_rows = cache.rows.len();
        cache.last_us = started.elapsed().as_micros() as u64;
        if cache.rebuilds != rebuilds_before {
            // #28 residual: one structured line per index rebuild, consumed by
            // the service journal (stderr, key=value); `remove_barrier_stats`
            // and the test accessor expose the same counters in-process.
            let line = format!(
                "level=info event=remove_barrier_rebuild calls={} rebuilds={} rows={} last_us={}",
                cache.calls, cache.rebuilds, cache.last_rows, cache.last_us
            );
            eprintln!("{line}");
            #[cfg(test)]
            cache.diagnostics.push(line);
        }
        Ok(blocked)
    }
    /// Barrier bookkeeping for tests and latency diagnostics: (calls, rebuilds,
    /// last lookup microseconds, indexed rows).
    pub fn remove_barrier_stats(&self) -> (u64, u64, u64, usize) {
        (
            self.barrier.calls,
            self.barrier.rebuilds,
            self.barrier.last_us,
            self.barrier.last_rows,
        )
    }
    /// Structured rebuild diagnostics captured in-process for tests. Production
    /// emits the same lines to stderr (systemd journal) instead of buffering them.
    #[cfg(test)]
    pub(crate) fn remove_barrier_diagnostics(&self) -> Vec<String> {
        self.barrier.diagnostics.clone()
    }
}
fn os(code: i32) -> Error {
    Error::Io(std::io::Error::from_raw_os_error(code))
}
/// A local creation that has never acquired a server identity. Deleting it
/// needs no native remove: no server object can be acknowledged for it.
pub(crate) fn creation_unbound(db: &rusqlite::Connection, node: &str) -> Result<bool> {
    Ok(db.query_row(
        "SELECT EXISTS(SELECT 1 FROM local_creations c JOIN nodes n ON n.id=c.node WHERE c.node=?1 AND n.directory=0) AND NOT EXISTS(SELECT 1 FROM remote_aliases WHERE node=?1)",
        [node],
        |r| r.get(0),
    )?)
}
pub(crate) fn has_uncertain_upload(db: &rusqlite::Connection, node: &str) -> Result<bool> {
    Ok(db.query_row(
        "SELECT EXISTS(SELECT 1 FROM operations WHERE kind='upload' AND node=?1 AND state IN ('uncertain','abandoned'))",
        [node],
        |r| r.get(0),
    )?)
}
/// Retire an unconfirmed creation. The caller owns the transaction and has
/// already established that the path was deleted locally and that the creation
/// never acquired a server identity. The uncertain upload becomes superseded
/// (never replayed) and history keeps the permission evidence plus the reason
/// automatic reconciliation stopped. Content/blob cleanup follows the existing
/// `commit_kept_both` retirement template. Contract §1.2 asks for
/// `remote_version=NULL, dirty_generation=NULL`, but both columns are NOT NULL;
/// they keep their current values instead and are semantically equivalent here
/// (the row is already a tombstone and no upload can ever read it again).
pub(crate) fn retire_creation_tx(
    tx: &rusqlite::Connection,
    operation: &str,
    node: &str,
    reason: &str,
) -> Result<()> {
    let attempts: i64 = tx.query_row(
        "SELECT reconcile_attempts FROM operations WHERE id=?1 AND kind='upload'",
        [operation],
        |r| r.get(0),
    )?;
    // A local edit after the permission-uncertain commit can queue a second,
    // still-pending upload for the same node. It must not outlive the tombstone:
    // otherwise it keeps `pending_operations` at 1 and the upload gate reports a
    // permanent block with no issue entry left to act on. `inflight` is left to
    // the bridge, which resolves it from the in-flight response and then fails
    // against the deleted node instead of leaving a dangling row.
    tx.execute(
        "UPDATE operations SET state='superseded' WHERE node=?1 AND kind='upload' AND id<>?2 AND state IN ('pending','uncertain')",
        [node, operation],
    )?;
    tx.execute(
        "UPDATE operations SET state='superseded' WHERE id=?1 AND state IN ('uncertain','abandoned')",
        [operation],
    )?;
    // #9b TB2: the retired operation and any residual sibling upload reach a
    // queue-level terminal outcome here. They are never replayed, but the
    // server-side creation may still exist, so they are confirmed/local_abandoned
    // rather than an unknown retry. `inflight` siblings are resolved by the
    // bridge from their response and keep their own diagnosis.
    tx.execute(
        "UPDATE operations SET outcome='confirmed', outcome_category='local_abandoned', last_error='local creation was retired before this upload was confirmed', next_retry_at=0, updated_at=unixepoch() WHERE node=?1 AND kind='upload' AND id<>?2 AND state='superseded' AND outcome IS NULL",
        [node, operation],
    )?;
    tx.execute(
        "UPDATE operations SET outcome='confirmed', outcome_category='local_abandoned', last_error=?2, next_retry_at=0, updated_at=unixepoch() WHERE id=?1",
        rusqlite::params![
            operation,
            format!("local creation was retired ({reason}); server creation outcome unconfirmed")
        ],
    )?;
    // The operation trigger above writes a cancelled history row; recording the
    // abandoned outcome afterwards replaces it with the explanatory failure.
    crate::sync_history::record_abandoned_upload(
        tx,
        &format!("upload:{operation}"),
        node,
        u32::try_from(attempts).unwrap_or(u32::MAX),
        reason,
    )?;
    // Issue #50 phase 4 (R5): every upload operation superseded above may still
    // have staging rows in `upload_paths`/`upload_origins`. A superseded upload
    // is never replayed, so keep no orphaned staging metadata behind; this also
    // covers the pending upload that used to survive the tombstone.
    tx.execute(
        "DELETE FROM upload_paths WHERE operation IN (SELECT id FROM operations WHERE node=?1 AND kind='upload' AND state='superseded')",
        [node],
    )?;
    tx.execute(
        "DELETE FROM upload_origins WHERE operation IN (SELECT id FROM operations WHERE node=?1 AND kind='upload' AND state='superseded')",
        [node],
    )?;
    tx.execute("DELETE FROM local_creations WHERE node=?1", [node])?;
    tx.execute("UPDATE namespace_ops SET state='superseded' WHERE node=?1 AND kind='remove' AND state IN ('pending','cleanup_pending','uncertain','cleanup_uncertain')", [node])?;
    tx.execute("DELETE FROM namespace_overlay WHERE node=?1", [node])?;
    tx.execute(
        "INSERT OR REPLACE INTO local_unlinks(node,confirmed) VALUES(?1,1)",
        [node],
    )?;
    tx.execute(
        "UPDATE nodes SET deleted=1,blob=NULL,local_version=NULL,base_version=NULL,dirty_blob=NULL,conflict=0 WHERE id=?1",
        [node],
    )?;
    tx.execute("DELETE FROM drafts WHERE node=?1", [node])?;
    tx.execute("DELETE FROM transfers WHERE node=?1", [node])?;
    tx.execute("DELETE FROM upload_paths WHERE operation=?1", [operation])?;
    tx.execute("DELETE FROM upload_origins WHERE operation=?1", [operation])?;
    Ok(())
}
impl Store {
    pub fn locally_unlinked(&self, id: &str) -> Result<bool> {
        is_unlinked(&self.db, id)
    }

    /// expected guards the interval in which FUSE may hydrate an open handle.
    /// Existing queued moves run first; the delete targets their final path.
    pub fn unlink_local(
        &mut self,
        parent: &str,
        name: &str,
        expected: &str,
        directory: bool,
    ) -> Result<()> {
        let node = self.lookup(parent, name)?.node;
        if node.id != expected {
            return Err(os(libc::EAGAIN));
        }
        if node.parent.is_none() {
            return Err(os(libc::EBUSY));
        }
        if directory != node.directory {
            return Err(os(if directory {
                libc::ENOTDIR
            } else {
                libc::EISDIR
            }));
        }
        if directory && !self.children(&node.id)?.is_empty() {
            return Err(os(libc::ENOTEMPTY));
        }
        let unsubmitted: bool = self.db.query_row("SELECT EXISTS(SELECT 1 FROM local_creations c JOIN nodes n ON n.id=c.node WHERE c.node=?1 AND n.directory=0) AND NOT EXISTS(SELECT 1 FROM operations WHERE node=?1 AND state IN ('inflight','uncertain','abandoned','committed','conflict'))", [&node.id], |r| r.get(0))?;
        // An unconfirmed creation has no server identity to remove. When the
        // user deletes it locally, retire the uncertain/abandoned upload instead
        // of queueing a native remove that could never be acknowledged (and
        // would keep mark_remove_started permanently EBUSY).
        let retire = if unsubmitted
            || !creation_unbound(&self.db, &node.id)?
            || !has_uncertain_upload(&self.db, &node.id)?
        {
            None
        } else {
            self.db
                .query_row(
                    "SELECT id FROM operations WHERE kind='upload' AND node=?1 AND state IN ('uncertain','abandoned') ORDER BY rowid LIMIT 1",
                    [&node.id],
                    |r| r.get::<_, String>(0),
                )
                .optional()?
        };
        if let Some(operation) = retire {
            let tx = self.db.transaction()?;
            tx.execute(
                "INSERT INTO local_unlinks(node,confirmed) VALUES(?1,1)",
                [&node.id],
            )?;
            tx.execute(
                "UPDATE operations SET state='superseded', outcome='confirmed', outcome_category='local_abandoned', last_error='local file was deleted before this upload was sent', next_retry_at=0, updated_at=unixepoch() WHERE node=?1 AND state='pending'",
                [&node.id],
            )?;
            retire_creation_tx(&tx, &operation, &node.id, "local_deleted")?;
            tx.commit()?;
            return Ok(());
        }
        let job = RemoveJob {
            operation: uuid::Uuid::new_v4().to_string(),
            path: crate::namespace::node_path(&self.db, &node.id)?,
            cursor: self.cursor()?,
            node: node.clone(),
        };
        let tx = self.db.transaction()?;
        tx.execute(
            "INSERT INTO local_unlinks(node,confirmed) VALUES(?1,?2)",
            params![node.id, unsubmitted],
        )?;
        // An operation already sent must be reconciled before deletion. Later
        // unsent saves no longer have a name and must never reach the server.
        tx.execute(
            "UPDATE operations SET state='superseded', outcome='confirmed', outcome_category='local_abandoned', last_error='local file was deleted before this upload was sent', next_retry_at=0, updated_at=unixepoch() WHERE node=?1 AND state='pending'",
            [&node.id],
        )?;
        if unsubmitted {
            tx.execute("DELETE FROM local_creations WHERE node=?1", [&node.id])?;
        } else {
            tx.execute(
                "INSERT OR IGNORE INTO namespace_overlay(node,remote) VALUES(?1,?2)",
                params![node.id, serde_json::to_string(&Some(&node))?],
            )?;
            tx.execute("INSERT INTO namespace_ops(id,node,body,state,kind) VALUES(?1,?2,?3,'pending','remove')",params![job.operation,node.id,serde_json::to_string(&job)?])?;
        }
        tx.execute("UPDATE nodes SET deleted=1 WHERE id=?1", [&node.id])?;
        tx.commit()?;
        Ok(())
    }
    pub fn next_namespace_kind(&self) -> Result<Option<String>> {
        Ok(self.db.query_row("SELECT kind FROM namespace_ops WHERE id NOT IN (SELECT operation FROM access_blocked_operations) AND state NOT IN ('committed','superseded') ORDER BY seq LIMIT 1", [], |r|r.get(0)).optional()?)
    }
    /// Inspect the ordered journal, including conflicts; this does not authorize execution.
    pub fn next_remove(&self) -> Result<Option<QueuedRemove>> {
        self.namespace_body_for_kind("remove")?
            .map(|(body, state)| {
                Ok(QueuedRemove {
                    job: serde_json::from_str(&body)?,
                    state,
                })
            })
            .transpose()
    }
    /// Select executable work under the bounded dependency and retry gates.
    pub fn scheduled_remove(&self) -> Result<Option<QueuedRemove>> {
        self.scheduled_namespace_body_for_kind("remove")?
            .map(|(body, state)| {
                Ok(QueuedRemove {
                    job: serde_json::from_str(&body)?,
                    state,
                })
            })
            .transpose()
    }
    pub fn mark_remove_started(&mut self, id: &str) -> Result<()> {
        let head = self
            .scheduled_remove()?
            .ok_or_else(|| Error::State("remove is not queue head".into()))?;
        if head.job.operation != id || head.state != "pending" {
            return Err(Error::State(
                "remove must reconcile before resubmission".into(),
            ));
        }
        let busy: bool = self.db.query_row(
            "SELECT EXISTS(SELECT 1 FROM operations WHERE node NOT IN (SELECT id FROM access_blocked) AND state IN ('inflight','uncertain'))",
            [],
            |r| r.get(0),
        )?;
        if busy || head.job.node.version == crate::creations::CREATE_VERSION {
            return Err(os(libc::EBUSY));
        }
        if !self.locally_unlinked(&head.job.node.id)? {
            return Err(Error::State("remove lost unlink intent".into()));
        }
        self.db.execute("UPDATE namespace_ops SET state='inflight' WHERE id=?1 AND kind='remove' AND state='pending'",[id])?;
        // #9b TB2: a fresh remove submission supersedes a stale diagnosis.
        crate::commit_diagnostics::clear_tx(&self.db, "namespace_ops", id)?;
        Ok(())
    }
    pub fn mark_remove_uncertain(&mut self, id: &str) -> Result<()> {
        self.mark_remove_uncertain_diagnosed(
            id,
            crate::commit_outcome::CommitCategory::Interrupted,
            Some("remove commit outcome unknown; reconciliation required"),
        )
    }
    pub fn mark_remove_uncertain_diagnosed(
        &mut self,
        id: &str,
        category: crate::commit_outcome::CommitCategory,
        last_error: Option<&str>,
    ) -> Result<()> {
        let tx = self.db.transaction()?;
        if tx.execute("UPDATE namespace_ops SET state='uncertain' WHERE id=?1 AND kind='remove' AND state='inflight'",[id])? != 1 { return Err(Error::State("remove not inflight".into())); }
        crate::commit_diagnostics::record(
            &tx,
            "namespace_ops",
            id,
            &crate::commit_outcome::CommitDiagnostic::unknown_retryable(
                category,
                last_error.map(str::to_owned),
                0,
                0,
            ),
        )?;
        tx.commit()?;
        Ok(())
    }
    /// Issue #60: explicitly retire one unresolved local remove tombstone so
    /// the working copy becomes reclaimable, without ever replaying a remote
    /// delete. This is the user-visible escape hatch for a remove stuck as
    /// `pending` (the 805 MB incident) or one whose automatic reconciliation
    /// stopped at a terminal-unresolved diagnosis.
    ///
    /// It only touches local state: the queued intent is superseded (never
    /// resubmitted), the unlink is confirmed and the draft row is released.
    /// It refuses while the remove is `inflight`, because a remote delete may
    /// already be in progress and the local view must not diverge first.
    pub fn abandon_unresolved_remove(&mut self, node: &str) -> Result<bool> {
        let removes: Vec<(String, String)> = {
            let mut query = self.db.prepare(
                "SELECT id,state FROM namespace_ops WHERE node=?1 AND kind='remove' \
                 AND state NOT IN ('committed','superseded')",
            )?;
            query
                .query_map([node], |row| Ok((row.get(0)?, row.get(1)?)))?
                .collect::<std::result::Result<Vec<_>, _>>()?
        };
        if removes.is_empty() {
            return Ok(false);
        }
        if removes
            .iter()
            .any(|(_, state)| matches!(state.as_str(), "inflight" | "cleanup_inflight"))
        {
            return Err(os(libc::EBUSY));
        }
        let tx = self.db.transaction()?;
        tx.execute(
            "UPDATE namespace_ops SET state='superseded', outcome='confirmed', \
                outcome_category='local_abandoned', \
                last_error='local remove abandoned; no remote delete was replayed', \
                next_retry_at=0, updated_at=unixepoch() \
             WHERE node=?1 AND kind='remove' AND state NOT IN ('committed','superseded')",
            [node],
        )?;
        tx.execute(
            "INSERT OR REPLACE INTO local_unlinks(node,confirmed) VALUES(?1,1)",
            [node],
        )?;
        tx.execute("UPDATE nodes SET deleted=1,conflict=0 WHERE id=?1", [node])?;
        tx.execute("DELETE FROM namespace_overlay WHERE node=?1", [node])?;
        tx.execute("DELETE FROM drafts WHERE node=?1", [node])?;
        tx.commit()?;
        Ok(true)
    }
    pub fn conflict_remove(&mut self, id: &str) -> Result<()> {
        let tx = self.db.transaction()?;
        if tx.execute("UPDATE namespace_ops SET state='conflict' WHERE id=?1 AND kind='remove' AND state IN ('pending','inflight','uncertain')",[id])? != 1 { return Err(Error::State("remove not active".into())); }
        tx.execute(
            "UPDATE nodes SET conflict=1 WHERE id=(SELECT node FROM namespace_ops WHERE id=?1)",
            [id],
        )?;
        crate::commit_diagnostics::record(
            &tx,
            "namespace_ops",
            id,
            &crate::commit_outcome::CommitDiagnostic::conflict(Some(
                "remote changed before the remove was confirmed".into(),
            )),
        )?;
        tx.commit()?;
        Ok(())
    }
    /// Called only after backend verification of a genuine RemoveEvent for the
    /// original remote identity. A response ACK or path absence is insufficient.
    pub fn confirm_remove(&mut self, id: &str, node: &str) -> Result<()> {
        let tx = self.db.transaction()?;
        let (body, state): (String, String) = tx.query_row(
            "SELECT body,state FROM namespace_ops WHERE id=?1 AND kind='remove'",
            [id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )?;
        let job: RemoveJob = serde_json::from_str(&body)?;
        if job.node.id != node {
            return Err(Error::State("remove receipt identity mismatch".into()));
        }
        if state == "committed" {
            return Ok(());
        }
        if !matches!(state.as_str(), "inflight" | "uncertain") || !is_unlinked(&tx, node)? {
            return Err(Error::State("remove not submitted".into()));
        }
        tx.execute(
            "UPDATE namespace_ops SET state='committed' WHERE id=?1",
            [id],
        )?;
        tx.execute("UPDATE local_unlinks SET confirmed=1 WHERE node=?1", [node])?;
        tx.execute("UPDATE nodes SET deleted=1,conflict=0 WHERE id=?1", [node])?;
        tx.execute("DELETE FROM namespace_overlay WHERE node=?1", [node])?;
        // Issue #60: once the remote remove reached its terminal confirmed
        // state the working copy can never be uploaded again, so its body must
        // become reclaimable instead of pinning the drafts file forever. The
        // following `collect_garbage` removes the now-unreferenced file.
        tx.execute("DELETE FROM drafts WHERE node=?1", [node])?;
        crate::commit_diagnostics::record(
            &tx,
            "namespace_ops",
            id,
            &crate::commit_outcome::CommitDiagnostic::confirmed(),
        )?;
        // Orphan bytes survive until no open handles need them. Recovery/GC of
        // orphan working copies is deliberately separate from NAS deletion.
        tx.commit()?;
        Ok(())
    }
}

#[cfg(test)]
mod upload_remove_scope_tests {
    use super::path_related;

    #[test]
    fn only_same_or_nested_paths_are_related() {
        assert!(path_related("a/b", "a/b"));
        assert!(path_related("a/b/c", "a/b"));
        assert!(path_related("a/b", "a/b/c"));
        assert!(!path_related("a/bc", "a/b"));
        assert!(!path_related("a/b", "a/bc"));
        assert!(!path_related("x/a", "a"));
        assert!(!path_related("a", "b/a"));
    }
}

#[cfg(test)]
#[path = "removals_barrier_tests.rs"]
mod barrier_tests;

#[cfg(test)]
#[path = "removals_retire_tests.rs"]
mod retire_tests;
