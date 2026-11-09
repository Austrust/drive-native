//! Local conflict decisions are atomic. Uncertain server commits remain gated.
use crate::{Error, Node, NodeMetadata, Result, Status, Store, creations::CREATE_VERSION};
use rusqlite::{OptionalExtension, params};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct SyncIssue {
    pub id: String,
    pub path: String,
    pub directory: bool,
    pub reason: &'static str,
    pub can_keep_both: bool,
    pub can_keep_directory: bool,
    pub token: String,
    pub reconcile_attempts: u32,
    pub reconcile_exhausted: bool,
    /// True only for an unconfirmed local creation that can be discarded
    /// locally without touching a server object (no bound remote identity).
    pub can_abandon_upload: bool,
    /// True when a bound directory's remote identity disappeared while local
    /// descendants survive; the tree can be requeued as a local creation.
    pub can_keep_local: bool,
    /// True when accepting the remote deletion of that bound directory would
    /// not lose any surviving local-only content.
    pub can_keep_remote: bool,
}
#[derive(Debug, Serialize)]
pub struct IssuePage {
    pub entries: Vec<SyncIssue>,
    pub next: Option<String>,
}
#[derive(Debug, Serialize)]
pub struct KeptBoth {
    pub original: String,
    pub copy: Node,
    pub path: String,
}

/// Candidate identity source shared by [`Store::sync_issues`] and
/// [`Store::sync_issue_count`].
///
/// The previous count shape evaluated four correlated `EXISTS` subqueries for
/// every node row: that forced a full `nodes` scan and, without a `node` index
/// on `operations`/`namespace_ops`, a full scan of both tables per candidate row
/// (issue #16), and cost ~52ms per GetOverview at 210k nodes (issue #21).
/// Expressing each issue source as its own `UNION` arm lets the planner start
/// from the small partial indexes (conflict/pinned flags, unfinished operation
/// states) and walk only the combined candidate set.
///
/// The unfinished operation arms use `INDEXED BY` so the plan does not depend
/// on statistics the store deliberately does not maintain: the partial indexes
/// hold only in-flight rows, while the plain `node` indexes would scan every
/// finished operation. The redundant `state NOT IN (...)` terms make the
/// partial predicates visible to the planner.
const ISSUE_CANDIDATE_SOURCES: &str = "
    SELECT id FROM (
        SELECT id FROM nodes WHERE conflict=1
        UNION
        SELECT n.id FROM nodes n JOIN drafts d ON d.node=n.id
         WHERE d.dirty=1 AND (d.base!=n.remote_version OR n.deleted=1)
           AND NOT EXISTS(SELECT 1 FROM local_unlinks WHERE node=n.id)
        UNION
        SELECT o.node FROM operations o INDEXED BY operations_unfinished
         WHERE o.state IN ('uncertain','abandoned') AND o.state NOT IN ('committed','superseded')
           AND EXISTS(SELECT 1 FROM nodes n WHERE n.id=o.node)
        UNION
        SELECT o.node FROM namespace_ops o INDEXED BY namespace_unfinished
         WHERE o.state IN ('uncertain','cleanup_uncertain','conflict')
           AND o.state NOT IN ('committed','superseded')
           AND EXISTS(SELECT 1 FROM nodes n WHERE n.id=o.node)
    )";
fn issue_candidates(tail: &str) -> String {
    format!("{ISSUE_CANDIDATE_SOURCES} {tail}")
}
impl Store {
    // Conflict history includes tombstones hidden from ordinary filesystem lookup.
    // This accessor is local to the issue service and never makes deleted paths live.
    fn issue_node(&self, id: &str) -> Result<Node> {
        let mut node = self.db.query_row(
            "SELECT id,parent,name,remote_version,directory FROM nodes WHERE id=?1",
            [id],
            |r| {
                Ok(Node {
                    id: r.get(0)?,
                    parent: r.get(1)?,
                    name: r.get(2)?,
                    version: r.get(3)?,
                    directory: r.get(4)?,
                    metadata: None,
                })
            },
        )?;
        node.metadata = self.metadata(id)?;
        Ok(node)
    }

    fn issue_path(&self, root: &str, id: &str) -> Result<String> {
        let mut current = id.to_owned();
        let mut parts = Vec::new();
        let mut seen = std::collections::HashSet::new();
        while current != root {
            if parts.len() >= 128 || !seen.insert(current.clone()) {
                return Err(Error::State("issue ancestry invalid".into()));
            }
            let node = self.issue_node(&current)?;
            parts.push(node.name);
            current = node
                .parent
                .ok_or_else(|| Error::State("issue outside task".into()))?;
        }
        parts.reverse();
        Ok(parts.join("/"))
    }
    /// Count and content signature of surviving descendants of a tombstoned
    /// directory. The signature participates in the issue token so a stale
    /// keep-local decision can never requeue a different tree.
    fn surviving_descendants(&self, id: &str) -> Result<(u64, String)> {
        let mut q = self.db.prepare(
            "WITH RECURSIVE tree(id) AS (
                 SELECT id FROM nodes WHERE parent=?1 AND deleted=0
                 UNION
                 SELECT n.id FROM nodes n JOIN tree t ON n.parent=t.id WHERE n.deleted=0
             )
             SELECT id FROM tree ORDER BY id",
        )?;
        let ids = q
            .query_map([id], |r| r.get::<_, String>(0))?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        let count = u64::try_from(ids.len()).unwrap_or(u64::MAX);
        Ok((count, crate::sha256(ids.join("\n").as_bytes())))
    }

    /// True when at least one surviving descendant of the tombstoned directory
    /// is a local-only creation: accepting the remote deletion would drop those
    /// bytes, so keep-remote must not be offered.
    fn has_local_only_surviving_content(&self, id: &str) -> Result<bool> {
        let local_only: bool = self.db.query_row(
            "WITH RECURSIVE tree(id) AS (
                 SELECT id FROM nodes WHERE parent=?1 AND deleted=0
                 UNION
                 SELECT n.id FROM nodes n JOIN tree t ON n.parent=t.id WHERE n.deleted=0
             )
             SELECT EXISTS(
                 SELECT 1 FROM tree t
                  WHERE EXISTS(SELECT 1 FROM local_creations c WHERE c.node=t.id)
                    AND NOT EXISTS(SELECT 1 FROM remote_aliases a WHERE a.node=t.id)
             )",
            [id],
            |r| r.get(0),
        )?;
        Ok(local_only)
    }
    pub fn sync_issue(&self, root: &str, id: &str) -> Result<Option<SyncIssue>> {
        let node = self.issue_node(id)?;
        let path = self.issue_path(root, id)?;
        let (dirty, blob, base, generation, deleted): (
            Option<String>,
            Option<String>,
            Option<String>,
            i64,
            bool,
        ) = self.db.query_row(
            "SELECT dirty_blob,blob,base_version,dirty_generation,deleted FROM nodes WHERE id=?1",
            [id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
        )?;
        let draft: Option<(String, String, i64, i64, bool)> = self
            .db
            .query_row(
                "SELECT token,base,generation,checkpoint,dirty FROM drafts WHERE node=?1",
                [id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
            )
            .optional()?;
        let uncertain: bool = self.db.query_row("SELECT EXISTS(SELECT 1 FROM operations WHERE node=?1 AND state IN ('inflight','uncertain','abandoned')) OR EXISTS(SELECT 1 FROM namespace_ops WHERE node=?1 AND state IN ('uncertain','cleanup_uncertain','inflight','cleanup_inflight'))",[id],|r|r.get(0))?;
        let conflict = self.status(id)? == Status::Conflict;
        if !uncertain && !conflict {
            return Ok(None);
        }
        // The durable upload reconciliation budget is surfaced so the GUI can
        // explain that automatic retries paused instead of implying a live wait.
        let reconcile_attempts: u32 = self
            .db
            .query_row(
                "SELECT COALESCE(MAX(reconcile_attempts),0) FROM operations WHERE kind='upload' AND node=?1 AND state IN ('inflight','uncertain','abandoned')",
                [id],
                |r| r.get::<_, i64>(0),
            )
            .map(|attempts| u32::try_from(attempts).unwrap_or(u32::MAX))?;
        let reconcile_exhausted = reconcile_attempts >= crate::MAX_UPLOAD_RECONCILE_ATTEMPTS;
        let namespace = crate::namespace::blocks_upload(&self.db, id)?;
        let local_creation: bool = self.db.query_row(
            "SELECT EXISTS(SELECT 1 FROM local_creations WHERE node=?1)",
            [id],
            |r| r.get(0),
        )?;
        let bound_identity: bool = self.db.query_row(
            "SELECT EXISTS(SELECT 1 FROM remote_aliases WHERE node=?1)",
            [id],
            |r| r.get(0),
        )?;
        let parent_live = node
            .parent
            .as_deref()
            .and_then(|p| self.node(p).ok())
            .is_some_and(|p| p.directory);
        let busy = self
            .handles
            .lock()
            .unwrap()
            .iter()
            .any(|(key, (owner, count))| {
                *count > 0
                    && (owner == id || dirty.as_ref() == Some(key) || blob.as_ref() == Some(key))
            });
        let has_local = dirty.is_some() || draft.as_ref().is_some_and(|d| d.4);
        // Bound-directory recovery (issue #54): a directory whose remote
        // identity disappeared is only actionable when local descendants still
        // exist. Otherwise the stale-directory guard below keeps it hidden.
        let (survivor_count, survivor_sig, local_only) =
            if node.directory && bound_identity && deleted && !local_creation {
                let (count, signature) = self.surviving_descendants(id)?;
                let local_only = if count > 0 {
                    self.has_local_only_surviving_content(id)?
                } else {
                    false
                };
                (count, signature, local_only)
            } else {
                (0, String::new(), false)
            };
        let alias_remote: Option<String> = self
            .db
            .query_row(
                "SELECT remote FROM remote_aliases WHERE node=?1",
                [id],
                |r| r.get(0),
            )
            .optional()?;
        // Discarding is limited to an unconfirmed local creation: the local
        // bytes were never acknowledged by the server and no remote identity is
        // bound, so deleting them cannot leave a dangling server object.
        let unconfirmed_upload: bool = self.db.query_row(
            "SELECT EXISTS(SELECT 1 FROM operations WHERE kind='upload' AND node=?1 AND state IN ('uncertain','abandoned'))",
            [id],
            |r| r.get(0),
        )?;
        let can_abandon_upload = !node.directory
            && local_creation
            && !bound_identity
            && !deleted
            && parent_live
            && unconfirmed_upload
            && crate::removals::creation_unbound(&self.db, id)?;
        let directory_plan =
            if node.directory && local_creation && !uncertain && !deleted && parent_live {
                self.directory_conflict_plan(id)?
            } else {
                None
            };
        let reason = if uncertain {
            "commit_uncertain"
        } else if directory_plan.is_some() {
            "directory_creation_conflict"
        } else if namespace || (node.directory && !bound_identity) || self.locally_unlinked(id)? {
            "namespace_conflict"
        } else if !parent_live {
            "parent_unavailable"
        } else if node.directory {
            // A directory bound to a remote identity with no ancestor namespace
            // pressure, deletion, or unresolved commit has no recoverable
            // action unless local descendants survive the remote tombstone.
            if survivor_count > 0 {
                "bound_directory_conflict"
            } else {
                return Ok(None);
            }
        } else if !has_local {
            "local_content_unavailable"
        } else if busy {
            "file_in_use"
        } else if local_creation {
            if node.version == CREATE_VERSION
                && crate::creations::is_local_identity(id)
                && !bound_identity
                && !deleted
            {
                "creation_conflict"
            } else {
                "creation_identity_mismatch"
            }
        } else if deleted {
            "remote_deleted"
        } else {
            "content_conflict"
        };
        let can_keep_local = reason == "bound_directory_conflict";
        let can_keep_remote = can_keep_local && !local_only;
        // serde tuples stop at 16 elements, so the token input is hashed as two
        // concatenated JSON fragments. Every decision-relevant field must be
        // included: a stale token must fail after any visible state change.
        let mut token_input = serde_json::to_vec(&(
            &node,
            &dirty,
            &base,
            generation,
            deleted,
            &draft,
            uncertain,
            namespace,
            conflict,
            local_creation,
            bound_identity,
            &directory_plan,
            reconcile_attempts,
            reconcile_exhausted,
            can_abandon_upload,
        ))?;
        token_input.extend(serde_json::to_vec(&(
            &alias_remote,
            survivor_count,
            &survivor_sig,
            can_keep_local,
            can_keep_remote,
        ))?);
        let token = crate::sha256(&token_input);
        Ok(Some(SyncIssue {
            id: id.into(),
            path,
            directory: node.directory,
            reason,
            can_keep_both: matches!(
                reason,
                "content_conflict" | "remote_deleted" | "creation_conflict"
            ),
            can_keep_directory: directory_plan.is_some(),
            reconcile_attempts,
            reconcile_exhausted,
            can_abandon_upload,
            can_keep_local,
            can_keep_remote,
            token,
        }))
    }
    /// Keyset pagination, local metadata only. No file open or NAS request.
    pub fn sync_issues(&self, root: &str, after: &str) -> Result<IssuePage> {
        self.node(root)?;
        if after.len() > 8192 {
            return Err(Error::State("invalid issue cursor".into()));
        }
        let query = issue_candidates("WHERE id>?1 ORDER BY id LIMIT 257");
        let ids = {
            let mut q = self.db.prepare(&query)?;
            q.query_map([after], |r| r.get::<_, String>(0))?
                .collect::<std::result::Result<Vec<_>, _>>()?
        };
        let next = (ids.len() > 256).then(|| ids[255].clone());
        let mut entries = Vec::new();
        for id in ids.into_iter().take(256) {
            if let Some(issue) = self.sync_issue(root, &id)? {
                entries.push(issue)
            }
        }
        Ok(IssuePage { entries, next })
    }
    /// Issue count for status summaries. Local metadata only; no paths, tokens,
    /// or NAS requests. The candidate set is exactly the one [`Self::sync_issues`]
    /// paginates; entries that resolve to no actionable issue are not counted.
    pub fn sync_issue_count(&self, root: &str) -> Result<u64> {
        self.node(root)?;
        let query = issue_candidates("ORDER BY id");
        let ids = {
            let mut q = self.db.prepare(&query)?;
            q.query_map([], |r| r.get::<_, String>(0))?
                .collect::<std::result::Result<Vec<_>, _>>()?
        };
        let mut count = 0_u64;
        for id in ids {
            if self.sync_issue(root, &id)?.is_some() {
                count += 1;
            }
        }
        Ok(count)
    }
    /// Recover a bound directory whose remote identity disappeared by detaching
    /// the old identity and requeueing the same node as a local creation.
    ///
    /// The node id is preserved so the inode, path and mounted descendants stay
    /// stable; only the remote alias is dropped. Descendants that are still
    /// bound keep their own identities (mixed subtrees: only the recovered root
    /// is requeued, see docs/STALE_CONFLICTS.md).
    pub fn keep_directory_local(
        &mut self,
        root: &str,
        id: &str,
        expected: &str,
    ) -> Result<Node> {
        let issue = self
            .sync_issue(root, id)?
            .ok_or_else(|| Error::State("recovery entry is stale".into()))?;
        if issue.token != expected || !issue.can_keep_local || !issue.directory {
            return Err(Error::State("recovery entry is stale".into()));
        }
        let mut node = self.issue_node(id)?;
        let deleted: bool = self.db.query_row(
            "SELECT deleted FROM nodes WHERE id=?1",
            [id],
            |r| r.get(0),
        )?;
        let bound: bool = self.db.query_row(
            "SELECT EXISTS(SELECT 1 FROM remote_aliases WHERE node=?1)",
            [id],
            |r| r.get(0),
        )?;
        if !node.directory || !deleted || !bound {
            return Err(Error::State("directory is not recoverable".into()));
        }
        let parent_id = node
            .parent
            .clone()
            .ok_or_else(|| Error::State("directory outside task".into()))?;
        let parent = self.node(&parent_id)?;
        if !parent.directory {
            return Err(Error::Io(std::io::Error::from_raw_os_error(libc::ENOTDIR)));
        }
        if parent.version == CREATE_VERSION {
            return Err(Error::State("parent directory not yet committed".into()));
        }
        let parent_queued: bool = self.db.query_row(
            "SELECT EXISTS(SELECT 1 FROM namespace_overlay WHERE node=?1)",
            [&parent_id],
            |r| r.get(0),
        )?;
        if parent_queued {
            return Err(Error::State("parent directory not yet committed".into()));
        }
        let exists: bool = self.db.query_row(
            "SELECT EXISTS(SELECT 1 FROM nodes WHERE parent=?1 AND name=?2 AND deleted=0 AND id<>?3)",
            params![parent_id, node.name, id],
            |r| r.get(0),
        )?;
        if exists {
            return Err(Error::Io(std::io::Error::from_raw_os_error(libc::EEXIST)));
        }
        // Requeueing a tree with an unresolved child commit would fight the
        // reconciliation worker for the same server object.
        let unsettled: bool = self.db.query_row(
            "WITH RECURSIVE tree(id) AS (
                 SELECT id FROM nodes WHERE id=?1
                 UNION
                 SELECT n.id FROM nodes n JOIN tree t ON n.parent=t.id WHERE n.deleted=0
             )
             SELECT EXISTS(
                 SELECT 1 FROM operations o JOIN tree t ON t.id=o.node
                  WHERE o.kind='upload' AND o.state IN ('inflight','uncertain')
             )",
            [id],
            |r| r.get(0),
        )?;
        if unsettled {
            return Err(Error::State("a child upload is still unresolved".into()));
        }
        let base = crate::namespace::node_path(&self.db, &parent_id)?;
        let path = if base.is_empty() {
            node.name.clone()
        } else {
            format!("{base}/{}", node.name)
        };
        node.version = CREATE_VERSION.into();
        let job = crate::directories::DirectoryJob {
            creation_contract: 1,
            cursor: self.cursor()?,
            origin: None,
            operation: uuid::Uuid::new_v4().to_string(),
            node: node.clone(),
            parent,
            path,
        };
        let tx = self.db.transaction()?;
        tx.execute("DELETE FROM remote_aliases WHERE node=?1", [id])?;
        tx.execute(
            "UPDATE nodes SET remote_version=?2, deleted=0, conflict=0 WHERE id=?1",
            params![id, CREATE_VERSION],
        )?;
        crate::filesystem::mark_inode_map_incomplete(&tx)?;
        crate::filesystem::write_metadata(&tx, &node)?;
        tx.execute(
            "INSERT INTO local_creations(node,ready) VALUES(?1,1)",
            [id],
        )?;
        tx.execute(
            "INSERT INTO namespace_overlay(node,remote) VALUES(?1,?2)",
            params![id, serde_json::to_string(&Some(&node))?],
        )?;
        tx.execute(
            "INSERT INTO namespace_ops(id,node,body,state,kind) VALUES(?1,?2,?3,'pending','mkdir')",
            params![job.operation, id, serde_json::to_string(&job)?],
        )?;
        tx.commit()?;
        Ok(node)
    }

    /// Accept the server deletion of a bound directory once no surviving
    /// local-only content would be lost. The tombstone and remote alias stay so
    /// nothing is recreated locally.
    pub fn keep_directory_remote(&mut self, root: &str, id: &str, expected: &str) -> Result<()> {
        let issue = self
            .sync_issue(root, id)?
            .ok_or_else(|| Error::State("recovery entry is stale".into()))?;
        if issue.token != expected || !issue.can_keep_remote || !issue.directory {
            return Err(Error::State("recovery entry is stale".into()));
        }
        let tx = self.db.transaction()?;
        let changed = tx.execute(
            "UPDATE nodes SET conflict=0 WHERE id=?1 AND deleted=1",
            [id],
        )?;
        if changed != 1 {
            return Err(Error::State("directory is not recoverable".into()));
        }
        tx.execute("DELETE FROM local_creations WHERE node=?1", [id])?;
        tx.execute("DELETE FROM namespace_overlay WHERE node=?1", [id])?;
        tx.execute(
            "UPDATE namespace_ops SET state='superseded' WHERE node=?1 AND state NOT IN ('committed','superseded')",
            [id],
        )?;
        tx.commit()?;
        Ok(())
    }

    /// Preserve local bytes under a new name and resume the original remote identity.
    /// No server overwrite is generated. Copy upload uses the normal create contract.
    pub fn keep_both(
        &mut self,
        root: &str,
        id: &str,
        expected: &str,
        name: &str,
    ) -> Result<KeptBoth> {
        let issue = self
            .sync_issue(root, id)?
            .ok_or_else(|| Error::State("conflict no longer present".into()))?;
        if issue.token != expected || !issue.can_keep_both {
            return Err(Error::State(
                "conflict changed or is not safe to resolve".into(),
            ));
        }
        let original = self.issue_node(id)?;
        let parent = original
            .parent
            .as_deref()
            .ok_or_else(|| Error::State("cannot resolve root".into()))?;
        let mut copy = Node {
            id: format!("local:{}", uuid::Uuid::new_v4()),
            parent: Some(parent.into()),
            name: name.into(),
            version: CREATE_VERSION.into(),
            directory: false,
            metadata: None,
        };
        crate::state::valid_node(&copy)?;
        if name.len() > 255 {
            return Err(Error::State("conflict copy name too long".into()));
        }
        let exists: bool = self.db.query_row(
            "SELECT EXISTS(SELECT 1 FROM nodes WHERE parent=?1 AND name=?2 AND deleted=0)",
            params![parent, name],
            |r| r.get(0),
        )?;
        if exists || name == original.name {
            return Err(Error::State("conflict copy name already in use".into()));
        }
        // Snapshot the latest closed draft without changing the original journal.
        // This also works when the remote file has been deleted.
        let blob = match self.snapshot_dirty_draft(id)? {
            Some(blob) => blob,
            None => self
                .db
                .query_row("SELECT dirty_blob FROM nodes WHERE id=?1", [id], |r| {
                    r.get::<_, String>(0)
                })?,
        };
        let metadata = std::fs::metadata(self.root.join("blobs").join(&blob))?;
        if !metadata.is_file() {
            return Err(Error::State("conflict content unavailable".into()));
        }
        copy.metadata = Some(NodeMetadata {
            size: metadata.len(),
            mtime: u64::from(crate::timestamps::native_mtime(metadata.modified()?)?),
            mode: self.metadata(id)?.map_or(0o644, |m| m.mode),
        });
        self.commit_kept_both(root, id, copy, blob, issue.reason == "creation_conflict")
    }
    fn commit_kept_both(
        &mut self,
        root: &str,
        id: &str,
        copy: Node,
        blob: String,
        retire_creation: bool,
    ) -> Result<KeptBoth> {
        let mtime = u32::try_from(
            copy.metadata
                .as_ref()
                .ok_or_else(|| Error::State("conflict copy metadata missing".into()))?
                .mtime,
        )
        .map_err(|_| Error::Io(std::io::Error::from_raw_os_error(libc::EOVERFLOW)))?;
        let op = uuid::Uuid::new_v4().to_string();
        let tx = self.db.transaction()?;
        tx.execute("INSERT INTO nodes(id,parent,name,remote_version,directory,local_version,blob,dirty_blob,base_version,dirty_generation) VALUES(?1,?2,?3,?4,0,?4,?5,?5,?4,1)",params![copy.id,copy.parent,copy.name,CREATE_VERSION,blob])?;
        crate::filesystem::mark_inode_map_incomplete(&tx)?;
        crate::filesystem::write_metadata(&tx, &copy)?;
        tx.execute(
            "INSERT INTO local_creations(node,ready) VALUES(?1,1)",
            [&copy.id],
        )?;
        tx.execute("INSERT INTO operations(id,node,kind,base,blob,generation) VALUES(?1,?2,'upload',?3,?4,1)",params![op,copy.id,CREATE_VERSION,blob])?;
        tx.execute(
            "INSERT INTO upload_times(operation,mtime) VALUES(?1,?2)",
            params![op, mtime],
        )?;
        // #9b TB2: keeping the remote copy resolves the queued local upload at
        // the queue level. The local content is not replayed; the server copy is
        // adopted instead, so the outcome is confirmed/local_abandoned.
        tx.execute("UPDATE operations SET state='superseded', outcome='confirmed', outcome_category='local_abandoned', last_error='conflict resolution superseded the local upload', next_retry_at=0, updated_at=unixepoch() WHERE node=?1 AND state IN ('pending','conflict')",[id])?;
        tx.execute("DELETE FROM drafts WHERE node=?1", [id])?;
        tx.execute(
            "UPDATE nodes SET dirty_blob=NULL,base_version=NULL,conflict=0 WHERE id=?1",
            [id],
        )?;
        if retire_creation {
            // The local object has never acquired a server identity. Retire its
            // path claim; do not adopt, rename, update or delete the remote peer.
            tx.execute(
                "UPDATE nodes SET pinned=(SELECT pinned FROM nodes WHERE id=?1) WHERE id=?2",
                params![id, copy.id],
            )?;
            tx.execute(
                "INSERT INTO local_modes(node,mode) SELECT ?2,mode FROM local_modes WHERE node=?1",
                params![id, copy.id],
            )?;
            tx.execute("DELETE FROM local_creations WHERE node=?1", [id])?;
            tx.execute(
                "INSERT INTO local_unlinks(node,confirmed) VALUES(?1,1)",
                [id],
            )?;
            tx.execute(
                "UPDATE nodes SET deleted=1,blob=NULL,local_version=NULL,pinned=0 WHERE id=?1",
                [id],
            )?;
            tx.execute("DELETE FROM transfers WHERE node=?1", [id])?;
        }
        tx.commit()?;
        let path = self.issue_path(root, &copy.id)?;
        Ok(KeptBoth {
            original: id.into(),
            copy,
            path,
        })
    }
}

#[cfg(test)]
mod issue_count_tests {
    use super::*;
    use crate::{Batch, Change, Event};

    fn node(id: &str, parent: Option<&str>, version: &str) -> Node {
        Node {
            id: id.into(),
            parent: parent.map(str::to_owned),
            name: if parent.is_some() {
                "A.doc".into()
            } else {
                "root".into()
            },
            version: version.into(),
            directory: parent.is_none(),
            metadata: Some(NodeMetadata {
                size: 3,
                mtime: 1,
                mode: 0o644,
            }),
        }
    }

    fn fixture() -> (tempfile::TempDir, Store) {
        let d = tempfile::tempdir().unwrap();
        let mut s = Store::open(d.path()).unwrap();
        s.install_remote_snapshot(
            &[node("root", None, "r"), node("a", Some("root"), "v1")],
            "1",
            "server",
        )
        .unwrap();
        s.ensure_fs_root("root").unwrap();
        (d, s)
    }

    fn change(s: &mut Store, to: &str, change: Change) {
        let from = s.cursor().unwrap();
        s.receive(&Batch {
            from,
            to: to.into(),
            events: vec![Event {
                id: to.into(),
                change,
            }],
        })
        .unwrap();
        s.drain().unwrap();
    }

    #[test]
    fn sync_issue_count_matches_paginated_issues_and_requires_root() {
        let (_d, mut s) = fixture();
        assert_eq!(s.sync_issue_count("root").unwrap(), 0);
        s.save_local("a", b"mine").unwrap();
        change(&mut s, "2", Change::Upsert(node("a", Some("root"), "v2")));
        assert_eq!(s.sync_issue_count("root").unwrap(), 1);
        let page = s.sync_issues("root", "").unwrap();
        assert_eq!(page.entries.len() as u64, 1);
        assert_eq!(
            page.entries.len() as u64,
            s.sync_issue_count("root").unwrap()
        );
        assert!(s.sync_issue_count("missing").is_err());
    }
}
