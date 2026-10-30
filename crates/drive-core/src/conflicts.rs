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
        let uncertain: bool = self.db.query_row("SELECT EXISTS(SELECT 1 FROM operations WHERE node=?1 AND state IN ('inflight','uncertain')) OR EXISTS(SELECT 1 FROM namespace_ops WHERE node=?1 AND state IN ('uncertain','cleanup_uncertain','inflight','cleanup_inflight'))",[id],|r|r.get(0))?;
        let conflict = self.status(id)? == Status::Conflict;
        if !uncertain && !conflict {
            return Ok(None);
        }
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
        } else if namespace || node.directory || self.locally_unlinked(id)? {
            "namespace_conflict"
        } else if !parent_live {
            "parent_unavailable"
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
        let token = crate::sha256(&serde_json::to_vec(&(
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
        ))?);
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
            token,
        }))
    }
    /// Keyset pagination, local metadata only. No file open or NAS request.
    pub fn sync_issues(&self, root: &str, after: &str) -> Result<IssuePage> {
        self.node(root)?;
        if after.len() > 8192 {
            return Err(Error::State("invalid issue cursor".into()));
        }
        let ids = {
            let mut q = self.db.prepare("SELECT id FROM nodes n WHERE id>?1 AND (conflict=1 OR EXISTS(SELECT 1 FROM drafts d WHERE d.node=n.id AND d.dirty=1 AND (d.base!=n.remote_version OR n.deleted=1) AND NOT EXISTS(SELECT 1 FROM local_unlinks WHERE node=n.id)) OR EXISTS(SELECT 1 FROM operations WHERE node=n.id AND state='uncertain') OR EXISTS(SELECT 1 FROM namespace_ops WHERE node=n.id AND state IN ('uncertain','cleanup_uncertain','conflict'))) ORDER BY id LIMIT 257")?;
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
            mtime: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
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
        let op = uuid::Uuid::new_v4().to_string();
        let tx = self.db.transaction()?;
        tx.execute("INSERT INTO nodes(id,parent,name,remote_version,directory,local_version,blob,dirty_blob,base_version,dirty_generation) VALUES(?1,?2,?3,?4,0,?4,?5,?5,?4,1)",params![copy.id,copy.parent,copy.name,CREATE_VERSION,blob])?;
        crate::filesystem::write_metadata(&tx, &copy)?;
        tx.execute(
            "INSERT INTO local_creations(node,ready) VALUES(?1,1)",
            [&copy.id],
        )?;
        tx.execute("INSERT INTO operations(id,node,kind,base,blob,generation) VALUES(?1,?2,'upload',?3,?4,1)",params![op,copy.id,CREATE_VERSION,blob])?;
        tx.execute("UPDATE operations SET state='superseded' WHERE node=?1 AND state IN ('pending','conflict')",[id])?;
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
