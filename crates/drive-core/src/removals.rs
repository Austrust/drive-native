//! Local unlink is durable before the path disappears. Native deletion is a
//! separate, ordered operation and requires an identity-specific tombstone.
use crate::{Error, Node, Result, Store};
use rusqlite::{OptionalExtension, params};
use serde::{Deserialize, Serialize};

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
fn path_related(a: &str, b: &str) -> bool {
    a == b
        || a.strip_prefix(b).is_some_and(|rest| rest.starts_with('/'))
        || b.strip_prefix(a).is_some_and(|rest| rest.starts_with('/'))
}
/// Current path of a possibly deleted node. Live ancestor names are walked so
/// a queued ancestor rename cannot hide the dependency behind a captured path.
fn live_path(db: &rusqlite::Connection, id: &str) -> Result<Option<String>> {
    let mut parts = Vec::new();
    let mut current = id.to_owned();
    let mut seen = std::collections::HashSet::new();
    loop {
        if !seen.insert(current.clone()) {
            return Err(Error::State("remove path cycle".into()));
        }
        let row: Option<(Option<String>, String)> = db
            .query_row("SELECT parent,name FROM nodes WHERE id=?1", [&current], |r| {
                Ok((r.get(0)?, r.get(1)?))
            })
            .optional()?;
        let Some((parent, name)) = row else {
            return Ok(None);
        };
        match parent {
            // The task root's own name is not part of relative paths.
            Some(parent) => {
                parts.push(name);
                current = parent
            }
            None => break,
        }
    }
    parts.reverse();
    Ok(Some(parts.join("/")))
}
/// Scope the remove barrier to dependencies of one upload target. The previous
/// global barrier let any unfinished remove, including an uncertain trash entry,
/// stop every ordinary upload.
pub(crate) fn upload_remove_barrier(
    db: &rusqlite::Connection,
    node: &str,
    path: &str,
) -> Result<Option<UploadRemoveBarrier>> {
    let rows: Vec<(String, String, String)> = {
        let mut q = db.prepare("SELECT id,node,body FROM namespace_ops WHERE kind='remove' AND id NOT IN (SELECT operation FROM access_blocked_operations) AND state NOT IN ('committed','superseded') ORDER BY seq")?;
        q.query_map([], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?))
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?
    };
    if rows.is_empty() {
        return Ok(None);
    }
    // Identity ancestry must be checked independently of captured paths: a
    // deleted ancestor remains a dependency even if its path was rebound.
    let mut ancestors = std::collections::HashSet::new();
    let mut current = Some(node.to_owned());
    while let Some(id) = current {
        if ancestors.len() >= 256 || !ancestors.insert(id.clone()) {
            return Err(Error::State("upload dependency ancestry invalid".into()));
        }
        current = db
            .query_row("SELECT parent FROM nodes WHERE id=?1", [&id], |r| {
                r.get::<_, Option<String>>(0)
            })
            .optional()?
            .flatten();
    }
    for (operation, remove_node, body) in rows {
        let job: RemoveJob = serde_json::from_str(&body)
            .map_err(|_| Error::State("unreadable remove intent".into()))?;
        if job.operation != operation {
            return Err(Error::State("remove intent identity mismatch".into()));
        }
        let remove_path = live_path(db, &remove_node)
            .map_err(|_| Error::State("remove path unreadable".into()))?
            .unwrap_or_else(|| job.path.clone());
        if ancestors.contains(&remove_node) || path_related(path, &remove_path) {
            return Ok(Some(UploadRemoveBarrier {
                operation,
                path: remove_path,
            }));
        }
    }
    Ok(None)
}
fn os(code: i32) -> Error {
    Error::Io(std::io::Error::from_raw_os_error(code))
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
        let unsubmitted: bool = self.db.query_row("SELECT EXISTS(SELECT 1 FROM local_creations c JOIN nodes n ON n.id=c.node WHERE c.node=?1 AND n.directory=0) AND NOT EXISTS(SELECT 1 FROM operations WHERE node=?1 AND state IN ('inflight','uncertain','committed','conflict'))", [&node.id], |r| r.get(0))?;
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
            "UPDATE operations SET state='superseded' WHERE node=?1 AND state='pending'",
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
        Ok(())
    }
    pub fn mark_remove_uncertain(&mut self, id: &str) -> Result<()> {
        if self.db.execute("UPDATE namespace_ops SET state='uncertain' WHERE id=?1 AND kind='remove' AND state='inflight'",[id])? != 1 { return Err(Error::State("remove not inflight".into())); }
        Ok(())
    }
    pub fn conflict_remove(&mut self, id: &str) -> Result<()> {
        let tx = self.db.transaction()?;
        if tx.execute("UPDATE namespace_ops SET state='conflict' WHERE id=?1 AND kind='remove' AND state IN ('pending','inflight','uncertain')",[id])? != 1 { return Err(Error::State("remove not active".into())); }
        tx.execute(
            "UPDATE nodes SET conflict=1 WHERE id=(SELECT node FROM namespace_ops WHERE id=?1)",
            [id],
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
