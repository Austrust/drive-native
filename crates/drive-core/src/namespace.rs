//! Local path changes and native namespace intents share one durable transaction.
//! Nodes expose the user's path; namespace_overlay retains the last server path.
use crate::{Change, Error, Node, Result, Store};
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RenameJob {
    pub operation: String,
    pub node: Node,
    pub source: String,
    pub destination: String,
    pub parent: String,
    pub parent_node: Node,
    pub name: String,
    pub cursor: String,
}
#[derive(Clone, Debug)]
pub struct QueuedRename {
    pub job: RenameJob,
    pub state: String,
}
fn os(code: i32) -> Error {
    Error::Io(std::io::Error::from_raw_os_error(code))
}
pub(crate) fn node_path(db: &Connection, id: &str) -> Result<String> {
    let mut parts = Vec::new();
    let mut id = id.to_owned();
    let mut seen = std::collections::HashSet::new();
    loop {
        if !seen.insert(id.clone()) {
            return Err(Error::State("namespace path cycle".into()));
        }
        let (parent, name): (Option<String>, String) = db.query_row(
            "SELECT parent,name FROM nodes WHERE id=?1 AND deleted=0",
            [&id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )?;
        match parent {
            Some(parent) => {
                parts.push(name);
                id = parent
            }
            None => break,
        }
    }
    parts.reverse();
    Ok(parts.join("/"))
}
pub(crate) fn normalized_version(db: &Connection, id: &str, version: &str) -> Result<String> {
    let mut value = version.to_owned();
    let mut seen = std::collections::HashSet::new();
    loop {
        if !seen.insert(value.clone()) {
            return Err(Error::State("content version alias cycle".into()));
        }
        let next: Option<String> = db
            .query_row(
                "SELECT new FROM content_version_aliases WHERE node=?1 AND old=?2",
                params![id, value],
                |r| r.get(0),
            )
            .optional()?;
        match next {
            Some(next) => value = next,
            None => return Ok(value),
        }
    }
}
fn protect_remote_deletion(db: &Connection, removed: &str) -> Result<()> {
    let overlays = {
        let mut q = db.prepare("SELECT node FROM namespace_overlay")?;
        q.query_map([], |r| r.get::<_, String>(0))?
            .collect::<std::result::Result<Vec<_>, _>>()?
    };
    for original in overlays {
        let mut current = Some(original.clone());
        let mut seen = std::collections::HashSet::new();
        while let Some(id) = current {
            if id == removed {
                db.execute(
                    "UPDATE namespace_overlay SET remote='null' WHERE node=?1",
                    [&original],
                )?;
                if crate::removals::is_unlinked(db, &original)? {
                    break;
                }
                db.execute("UPDATE nodes SET conflict=1 WHERE id=?1", [&original])?;
                db.execute(
                    "UPDATE namespace_ops SET state='conflict' WHERE node=?1 AND state='pending'",
                    [&original],
                )?;
                break;
            }
            if !seen.insert(id.clone()) {
                return Err(Error::State("remote ancestry cycle".into()));
            }
            let remote: Option<String> = db
                .query_row(
                    "SELECT remote FROM namespace_overlay WHERE node=?1",
                    [&id],
                    |r| r.get(0),
                )
                .optional()?;
            current = match remote {
                Some(body) => serde_json::from_str::<Option<Node>>(&body)?.and_then(|n| n.parent),
                None => db
                    .query_row("SELECT parent FROM nodes WHERE id=?1", [&id], |r| {
                        r.get::<_, Option<String>>(0)
                    })
                    .optional()?
                    .flatten(),
            };
        }
    }
    Ok(())
}
pub(crate) fn blocks_upload(db: &Connection, id: &str) -> Result<bool> {
    Ok(db.query_row("WITH RECURSIVE parents(id) AS (SELECT ?1 UNION SELECT n.parent FROM nodes n JOIN parents p ON p.id=n.id WHERE n.parent IS NOT NULL) SELECT EXISTS(SELECT 1 FROM namespace_overlay o JOIN parents p ON o.node=p.id)",[id],|r|r.get(0))?)
}
pub(crate) fn rebase_jobs(db: &Connection, id: &str, base: &str, version: &str) -> Result<()> {
    let rows = {
        let mut q = db.prepare(
            "SELECT body,kind FROM namespace_ops WHERE (node=?1 OR kind IN ('replace','rename')) AND state='pending' ORDER BY seq",
        )?;
        q.query_map([id], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?
    };
    for (body, kind) in rows {
        if kind == "replace" {
            crate::replacements::rebase_job(db, &body, id, base, version)?;
            continue;
        }
        if kind == "remove" {
            let mut job: crate::removals::RemoveJob = serde_json::from_str(&body)?;
            if job.node.version == base {
                job.node.version = version.into();
                db.execute(
                    "UPDATE namespace_ops SET body=?2 WHERE id=?1",
                    params![job.operation, serde_json::to_string(&job)?],
                )?;
            }
            continue;
        }
        if kind != "rename" {
            return Err(Error::State("unknown namespace operation".into()));
        }
        let mut job: RenameJob = serde_json::from_str(&body)?;
        let changed_source = job.node.id == id && job.node.version == base;
        let changed_parent = job.parent_node.id == id && job.parent_node.version == base;
        if changed_source {
            job.node.version = version.into();
        }
        if changed_parent {
            job.parent_node.version = version.into();
        }
        if changed_source || changed_parent {
            db.execute(
                "UPDATE namespace_ops SET body=?2 WHERE id=?1",
                params![job.operation, serde_json::to_string(&job)?],
            )?;
        }
    }
    Ok(())
}
/// Keep server evidence even when its path is hidden by a local pending move.
/// Returning None preserves the local object when a remote delete conflicts.
pub(crate) fn overlay_change(db: &Connection, change: &Change) -> Result<Option<Change>> {
    if let Change::Upsert(n) = change {
        let current: Option<String> = db
            .query_row(
                "SELECT remote_version FROM nodes WHERE id=?1",
                [&n.id],
                |r| r.get(0),
            )
            .optional()?;
        if let Some(current) = current {
            if n.version != current && normalized_version(db, &n.id, &n.version)? == current {
                return Ok(None);
            }
            #[cfg(feature = "native")]
            if crate::native::older_version(&n.version, &current) {
                return Ok(None);
            }
        }
    }

    if let Change::Delete { id } = change {
        protect_remote_deletion(db, id)?;
    }
    let id = match change {
        Change::Upsert(n) => &n.id,
        Change::Delete { id } => id,
    };
    let overlay: bool = db.query_row(
        "SELECT EXISTS(SELECT 1 FROM namespace_overlay WHERE node=?1)",
        [id],
        |r| r.get(0),
    )?;
    if !overlay {
        return Ok(Some(change.clone()));
    }
    match change {
        Change::Upsert(n) => {
            db.execute(
                "UPDATE namespace_overlay SET remote=?2 WHERE node=?1",
                params![id, serde_json::to_string(&Some(n))?],
            )?;
            let (parent, name): (Option<String>, String) =
                db.query_row("SELECT parent,name FROM nodes WHERE id=?1", [id], |r| {
                    Ok((r.get(0)?, r.get(1)?))
                })?;
            let mut visible = n.clone();
            visible.parent = parent;
            visible.name = name;
            Ok(Some(Change::Upsert(visible)))
        }
        Change::Delete { .. } => {
            db.execute(
                "UPDATE namespace_overlay SET remote='null' WHERE node=?1",
                [id],
            )?;
            if crate::removals::is_unlinked(db, id)? {
                return Ok(None);
            }
            db.execute("UPDATE nodes SET conflict=1 WHERE id=?1", [id])?;
            db.execute(
                "UPDATE namespace_ops SET state='conflict' WHERE node=?1 AND state='pending'",
                [id],
            )?;
            Ok(None)
        }
    }
}
impl Store {
    pub fn equivalent_content_version(&self, id: &str, version: &str) -> Result<String> {
        normalized_version(&self.db, id, version)
    }

    pub fn rename_local(
        &mut self,
        parent: &str,
        name: &str,
        new_parent: &str,
        new_name: &str,
        no_replace: bool,
    ) -> Result<()> {
        let source = self.lookup(parent, name)?.node;
        if source.parent.is_none() {
            return Err(os(libc::EBUSY));
        }
        if !self.node(new_parent)?.directory {
            return Err(os(libc::ENOTDIR));
        }
        let mut desired = source.clone();
        desired.parent = Some(new_parent.into());
        desired.name = new_name.into();
        crate::state::valid_node(&desired)?;
        if parent == new_parent && name == new_name {
            return if no_replace {
                Err(os(libc::EEXIST))
            } else {
                Ok(())
            };
        }
        match self.lookup(new_parent, new_name) {
            Ok(_) => {
                return Err(os(if no_replace {
                    libc::EEXIST
                } else {
                    libc::EOPNOTSUPP
                }));
            }
            Err(e) if e.is_not_found() => {}
            Err(e) => return Err(e),
        }
        let cycle:bool=self.db.query_row("WITH RECURSIVE parents(id) AS (SELECT ?1 UNION SELECT n.parent FROM nodes n JOIN parents p ON n.id=p.id WHERE n.parent IS NOT NULL) SELECT EXISTS(SELECT 1 FROM parents WHERE id=?2)",params![new_parent,source.id],|r|r.get(0))?;
        if cycle {
            return Err(os(libc::EINVAL));
        }
        let source_path = node_path(&self.db, &source.id)?;
        let parent_path = node_path(&self.db, new_parent)?;
        let destination = if parent_path.is_empty() {
            new_name.to_owned()
        } else {
            format!("{parent_path}/{new_name}")
        };
        // A never-submitted new file has no remote path to rename. Its existing
        // durable upload intent will derive the new path when it starts.
        let unsubmitted:bool=self.db.query_row("SELECT EXISTS(SELECT 1 FROM local_creations c JOIN nodes n ON n.id=c.node WHERE c.node=?1 AND n.directory=0) AND NOT EXISTS(SELECT 1 FROM operations WHERE node=?1 AND state IN ('inflight','uncertain','committed','conflict'))",[&source.id],|r|r.get(0))?;
        let job = RenameJob {
            operation: uuid::Uuid::new_v4().to_string(),
            node: source.clone(),
            source: source_path,
            destination,
            parent: new_parent.into(),
            parent_node: self.node(new_parent)?,
            name: new_name.into(),
            cursor: self.cursor()?,
        };
        let tx = self.db.transaction()?;
        if !unsubmitted {
            tx.execute(
                "INSERT OR IGNORE INTO namespace_overlay(node,remote) VALUES(?1,?2)",
                params![source.id, serde_json::to_string(&Some(&source))?],
            )?;
            tx.execute(
                "INSERT INTO namespace_ops(id,node,body,state) VALUES(?1,?2,?3,'pending')",
                params![job.operation, source.id, serde_json::to_string(&job)?],
            )?;
        }
        tx.execute(
            "UPDATE nodes SET parent=?2,name=?3 WHERE id=?1",
            params![source.id, new_parent, new_name],
        )?;
        tx.commit()?;
        Ok(())
    }
    pub fn next_rename(&self) -> Result<Option<QueuedRename>> {
        if self.next_namespace_kind()?.as_deref() != Some("rename") {
            return Ok(None);
        }
        let row:Option<(String,String)>=self.db.query_row("SELECT body,state FROM namespace_ops WHERE id NOT IN (SELECT operation FROM access_blocked_operations) AND state NOT IN ('committed','superseded') ORDER BY seq LIMIT 1",[],|r|Ok((r.get(0)?,r.get(1)?))).optional()?;
        row.map(|(body, state)| {
            Ok(QueuedRename {
                job: serde_json::from_str(&body)?,
                state,
            })
        })
        .transpose()
    }
    pub fn mark_rename_started(&mut self, id: &str) -> Result<()> {
        let head = self
            .next_rename()?
            .ok_or_else(|| Error::State("namespace queue empty".into()))?;
        if head.job.operation != id || head.state != "pending" {
            return Err(Error::State(
                "namespace order/uncertainty requires reconciliation".into(),
            ));
        }
        let busy: bool = self.db.query_row(
            "SELECT EXISTS(SELECT 1 FROM operations WHERE node NOT IN (SELECT id FROM access_blocked) AND state IN ('inflight','uncertain'))",
            [],
            |r| r.get(0),
        )?;
        if busy {
            return Err(os(libc::EBUSY));
        }
        if head.job.node.version == crate::creations::CREATE_VERSION {
            return Err(os(libc::EBUSY));
        }
        let current = self.node_for_handle(&head.job.node.id)?.version;
        if head.job.node.directory
            && current != head.job.node.version
            && directory_advance(&head.job.node.version, &current)
        {
            rebase_directory_moves(
                &self.db,
                &head.job.node.id,
                &head.job.node.version,
                &current,
            )?;
        } else if current != head.job.node.version {
            self.conflict_rename(id)?;
            return Err(Error::State(
                "remote changed before namespace submission".into(),
            ));
        }
        self.db.execute(
            "UPDATE namespace_ops SET state='inflight' WHERE id=?1 AND state='pending'",
            [id],
        )?;
        Ok(())
    }
    pub fn mark_rename_uncertain(&mut self, id: &str) -> Result<()> {
        if self.db.execute(
            "UPDATE namespace_ops SET state='uncertain' WHERE id=?1 AND state='inflight'",
            [id],
        )? != 1
        {
            return Err(Error::State("rename not inflight".into()));
        }
        Ok(())
    }
    pub fn conflict_rename(&mut self, id: &str) -> Result<()> {
        let tx = self.db.transaction()?;
        tx.execute(
            "UPDATE nodes SET conflict=1 WHERE id=(SELECT node FROM namespace_ops WHERE id=?1)",
            [id],
        )?;
        tx.execute(
            "UPDATE namespace_ops SET state='conflict' WHERE id=?1 AND state NOT IN ('committed','superseded')",
            [id],
        )?;
        tx.commit()?;
        Ok(())
    }
    pub fn confirm_rename(&mut self, id: &str, confirmed: &Node, same_content: bool) -> Result<()> {
        let tx = self.db.transaction()?;
        let (body, state): (String, String) = tx.query_row(
            "SELECT body,state FROM namespace_ops WHERE id=?1",
            [id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )?;
        let job: RenameJob = serde_json::from_str(&body)?;
        if state == "committed" {
            return Ok(());
        }
        if !matches!(state.as_str(), "inflight" | "uncertain")
            || confirmed.id != job.node.id
            || confirmed.parent.as_deref() != Some(job.parent.as_str())
            || confirmed.name != job.name
            || confirmed.directory != job.node.directory
            || confirmed.version.is_empty()
        {
            return Err(Error::State(
                "rename acknowledgement identity/path mismatch".into(),
            ));
        }
        let observed: String = tx.query_row(
            "SELECT remote FROM namespace_overlay WHERE node=?1",
            [&confirmed.id],
            |r| r.get(0),
        )?;
        let observed: Option<Node> = serde_json::from_str(&observed)?;
        let Some(observed) = observed else {
            tx.execute(
                "UPDATE namespace_ops SET state='conflict' WHERE id=?1",
                [id],
            )?;
            tx.commit()?;
            return Err(Error::State(
                "rename object deleted during confirmation".into(),
            ));
        };
        let remote: String = tx.query_row(
            "SELECT remote_version FROM nodes WHERE id=?1",
            [&confirmed.id],
            |r| r.get(0),
        )?;
        if remote == job.node.version || remote == confirmed.version {
            tx.execute("UPDATE nodes SET remote_version=?2,local_version=CASE WHEN ?4 AND local_version=?3 THEN ?2 ELSE local_version END,
                base_version=CASE WHEN ?4 AND base_version=?3 THEN ?2 ELSE base_version END,
                conflict=CASE WHEN ?4 AND (base_version IS NULL OR base_version=?3) THEN 0 ELSE conflict END WHERE id=?1",
                params![confirmed.id,confirmed.version,job.node.version,same_content])?;
            if same_content {
                if job.node.version != confirmed.version {
                    tx.execute("INSERT OR REPLACE INTO content_version_aliases(node,old,new) VALUES(?1,?2,?3)",params![confirmed.id,job.node.version,confirmed.version])?;
                }
                tx.execute(
                    "UPDATE operations SET base=?3 WHERE node=?1 AND base=?2 AND state='pending'",
                    params![confirmed.id, job.node.version, confirmed.version],
                )?;
                tx.execute(
                    "UPDATE drafts SET base=?3 WHERE node=?1 AND base=?2",
                    params![confirmed.id, job.node.version, confirmed.version],
                )?;
            } else {
                tx.execute("UPDATE nodes SET conflict=CASE WHEN dirty_blob IS NOT NULL OR EXISTS(SELECT 1 FROM drafts WHERE node=?1 AND dirty=1) THEN 1 ELSE conflict END WHERE id=?1",[&confirmed.id])?;
            }
            rebase_jobs(&tx, &confirmed.id, &job.node.version, &confirmed.version)?;
            crate::filesystem::write_metadata(&tx, confirmed)?;
        }
        if observed.version == job.node.version {
            tx.execute(
                "UPDATE namespace_overlay SET remote=?2 WHERE node=?1",
                params![confirmed.id, serde_json::to_string(&Some(confirmed))?],
            )?;
        }
        tx.execute(
            "UPDATE namespace_ops SET state='committed' WHERE id=?1",
            [id],
        )?;
        let remaining: bool = tx.query_row(
            "SELECT EXISTS(SELECT 1 FROM namespace_ops WHERE node=?1 AND state NOT IN ('committed','superseded'))",
            [&confirmed.id],
            |r| r.get(0),
        )?;
        if !remaining {
            let saved: String = tx.query_row(
                "SELECT remote FROM namespace_overlay WHERE node=?1",
                [&confirmed.id],
                |r| r.get(0),
            )?;
            let actual: Node = serde_json::from_str::<Option<Node>>(&saved)?
                .ok_or_else(|| Error::State("missing remote rename state".into()))?;
            tx.execute(
                "UPDATE nodes SET parent=?2,name=?3 WHERE id=?1",
                params![actual.id, actual.parent, actual.name],
            )?;
            tx.execute(
                "DELETE FROM namespace_overlay WHERE node=?1",
                [&confirmed.id],
            )?;
        }
        tx.commit()?;
        Ok(())
    }
}

/// A directory subtree revision can advance without changing the directory's
/// own identity or location. Only moves may refresh it; never weaken deletion.
fn directory_advance(old: &str, new: &str) -> bool {
    #[cfg(feature = "native")]
    {
        crate::native::directory_subtree_advance(old, new)
    }
    #[cfg(not(feature = "native"))]
    {
        let _ = (old, new);
        false
    }
}
fn rebase_directory_moves(db: &Connection, id: &str, base: &str, version: &str) -> Result<()> {
    let rows = {
        let mut q = db.prepare(
            "SELECT body FROM namespace_ops WHERE node=?1 AND kind='rename' AND state='pending'",
        )?;
        q.query_map([id], |r| r.get::<_, String>(0))?
            .collect::<std::result::Result<Vec<_>, _>>()?
    };
    for body in rows {
        let mut job: RenameJob = serde_json::from_str(&body)?;
        if job.node.version == base {
            job.node.version = version.into();
            db.execute(
                "UPDATE namespace_ops SET body=?2 WHERE id=?1",
                params![job.operation, serde_json::to_string(&job)?],
            )?;
        }
    }
    Ok(())
}
impl Store {
    /// The worker guarantees this response precedes sending any rename. An IPC
    /// loss instead stays uncertain; this method must never recover a timeout.
    pub fn refresh_directory_rename(&mut self, op: &str, observed: &Node) -> Result<()> {
        let tx = self.db.transaction()?;
        let (body, state): (String, String) = tx.query_row(
            "SELECT body,state FROM namespace_ops WHERE id=?1 AND kind='rename'",
            [op],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )?;
        let mut job: RenameJob = serde_json::from_str(&body)?;
        if state != "inflight"
            || !job.node.directory
            || !observed.directory
            || observed.id != job.node.id
            || observed.parent != job.node.parent
            || observed.name != job.node.name
            || !directory_advance(&job.node.version, &observed.version)
        {
            return Err(Error::State(
                "invalid pre-mutation directory observation".into(),
            ));
        }
        let current: String = tx.query_row(
            "SELECT remote_version FROM nodes WHERE id=?1",
            [&observed.id],
            |r| r.get(0),
        )?;
        let version =
            if current == job.node.version || directory_advance(&current, &observed.version) {
                tx.execute(
                    "UPDATE nodes SET remote_version=?2 WHERE id=?1",
                    params![observed.id, observed.version],
                )?;
                tx.execute(
                    "UPDATE namespace_overlay SET remote=?2 WHERE node=?1",
                    params![observed.id, serde_json::to_string(&Some(observed))?],
                )?;
                observed.version.clone()
            } else if directory_advance(&observed.version, &current) {
                current
            } else {
                return Err(Error::State(
                    "directory identity changed during preparation".into(),
                ));
            };
        let old = job.node.version.clone();
        job.node.version = version.clone();
        rebase_directory_moves(&tx, &job.node.id, &old, &version)?;
        tx.execute(
            "UPDATE namespace_ops SET state='pending',body=?2 WHERE id=?1",
            params![op, serde_json::to_string(&job)?],
        )?;
        tx.commit()?;
        Ok(())
    }
}
