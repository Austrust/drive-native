//! Directory creation is a durable namespace operation, preceding child writes.
use crate::{Error, Node, NodeMetadata, Result, Store, creations::CREATE_VERSION};
use rusqlite::params;
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DirectoryJob {
    /// Zero denotes a legacy job that may have been sent without provenance.
    #[serde(default)]
    pub creation_contract: u8,
    #[serde(default)]
    pub cursor: String,
    #[serde(default)]
    pub origin: Option<DirectoryOrigin>,
    pub operation: String,
    pub node: Node,
    pub parent: Node,
    pub path: String,
}
/// Compatibility name for existing directory journal records.
pub use crate::commit_origin::CommitOrigin as DirectoryOrigin;
#[derive(Clone, Debug)]
pub struct QueuedDirectory {
    pub job: DirectoryJob,
    pub state: String,
}
impl Store {
    pub fn create_local_directory(&mut self, parent: &str, name: &str, mode: u32) -> Result<Node> {
        let p = self.node(parent)?;
        if !p.directory {
            return Err(Error::Io(std::io::Error::from_raw_os_error(libc::ENOTDIR)));
        }
        let node = Node {
            id: format!("local:{}", uuid::Uuid::new_v4()),
            parent: Some(parent.into()),
            name: name.into(),
            version: CREATE_VERSION.into(),
            directory: true,
            metadata: Some(NodeMetadata {
                size: 0,
                mtime: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                mode: mode & 0o777,
            }),
        };
        crate::state::valid_node(&node)?;
        let exists: bool = self.db.query_row(
            "SELECT EXISTS(SELECT 1 FROM nodes WHERE parent=?1 AND name=?2 AND deleted=0)",
            params![parent, name],
            |r| r.get(0),
        )?;
        if exists {
            return Err(Error::Io(std::io::Error::from_raw_os_error(libc::EEXIST)));
        }
        let base = crate::namespace::node_path(&self.db, parent)?;
        let job = DirectoryJob {
            creation_contract: 1,
            cursor: self.cursor()?,
            origin: None,
            operation: uuid::Uuid::new_v4().to_string(),
            node: node.clone(),
            parent: p,
            path: if base.is_empty() {
                name.into()
            } else {
                format!("{base}/{name}")
            },
        };
        let tx = self.db.transaction()?;
        tx.execute(
            "INSERT INTO nodes(id,parent,name,remote_version,directory) VALUES(?1,?2,?3,?4,1)",
            params![node.id, parent, name, CREATE_VERSION],
        )?;
        crate::filesystem::write_metadata(&tx, &node)?;
        tx.execute(
            "INSERT INTO local_creations(node,ready) VALUES(?1,1)",
            [&node.id],
        )?;
        tx.execute(
            "INSERT INTO namespace_overlay(node,remote) VALUES(?1,?2)",
            params![node.id, serde_json::to_string(&Some(&node))?],
        )?;
        tx.execute(
            "INSERT INTO namespace_ops(id,node,body,state,kind) VALUES(?1,?2,?3,'pending','mkdir')",
            params![job.operation, node.id, serde_json::to_string(&job)?],
        )?;
        tx.commit()?;
        Ok(node)
    }
    pub fn next_directory(&self) -> Result<Option<QueuedDirectory>> {
        if self.next_namespace_kind()?.as_deref() != Some("mkdir") {
            return Ok(None);
        }
        let (body,state):(String,String)=self.db.query_row("SELECT body,state FROM namespace_ops WHERE id NOT IN (SELECT operation FROM access_blocked_operations) AND state NOT IN ('committed','superseded') ORDER BY seq LIMIT 1",[],|r|Ok((r.get(0)?,r.get(1)?)))?;
        Ok(Some(QueuedDirectory {
            job: serde_json::from_str(&body)?,
            state,
        }))
    }
    pub fn mark_directory_started(&mut self, op: &str) -> Result<DirectoryJob> {
        let mut q = self
            .next_directory()?
            .ok_or_else(|| Error::State("directory not queue head".into()))?;
        if q.job.operation != op || q.state != "pending" {
            return Err(Error::State("directory requires reconciliation".into()));
        }
        if q.job.origin.is_none() {
            q.job.creation_contract = 1;
            q.job.cursor = self.cursor()?;
            q.job.parent = self.node_for_handle(&q.job.parent.id)?;
        }
        if q.job.parent.version == CREATE_VERSION {
            return Err(Error::State("parent directory not yet committed".into()));
        }
        self.db.execute(
            "UPDATE namespace_ops SET state='inflight',body=?2 WHERE id=?1 AND state='pending'",
            params![op, serde_json::to_string(&q.job)?],
        )?;
        Ok(q.job)
    }
    /// Worker has not sent a mutation. Persist provenance before allowing a second request.
    pub fn prepare_directory(&mut self, op: &str, origin: DirectoryOrigin) -> Result<()> {
        origin.validate()?;
        let mut q = self
            .next_directory()?
            .ok_or_else(|| Error::State("directory not queue head".into()))?;
        if q.job.operation != op
            || q.job.creation_contract != 1
            || !(q.state == "inflight" || (q.state == "uncertain" && q.job.origin.is_none()))
        {
            return Err(Error::State(
                "cannot replace possibly submitted directory provenance".into(),
            ));
        }
        q.job.origin = Some(origin);
        self.db.execute(
            "UPDATE namespace_ops SET state='pending',body=?2 WHERE id=?1",
            params![op, serde_json::to_string(&q.job)?],
        )?;
        Ok(())
    }
    pub fn mark_directory_uncertain(&mut self, op: &str) -> Result<()> {
        if self.db.execute("UPDATE namespace_ops SET state='uncertain' WHERE id=?1 AND kind='mkdir' AND state='inflight'",[op])?!=1{return Err(Error::State("directory not in flight".into()));}
        Ok(())
    }
    pub fn conflict_directory(&mut self, op: &str) -> Result<()> {
        let tx = self.db.transaction()?;
        if tx.execute("UPDATE namespace_ops SET state='conflict' WHERE id=?1 AND kind='mkdir' AND state IN ('inflight','uncertain','pending')",[op])?!=1{return Err(Error::State("directory operation absent".into()));}
        tx.execute(
            "UPDATE nodes SET conflict=1 WHERE id=(SELECT node FROM namespace_ops WHERE id=?1)",
            [op],
        )?;
        tx.commit()?;
        Ok(())
    }
    pub fn confirm_directory(&mut self, op: &str, confirmed: &Node, remote: &str) -> Result<()> {
        let tx = self.db.transaction()?;
        let (body, state): (String, String) = tx.query_row(
            "SELECT body,state FROM namespace_ops WHERE id=?1 AND kind='mkdir'",
            [op],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )?;
        let job: DirectoryJob = serde_json::from_str(&body)?;
        if !matches!(state.as_str(), "inflight" | "uncertain")
            || !confirmed.directory
            || confirmed.id != job.node.id
            || confirmed.name != job.node.name
            || confirmed.parent != job.node.parent
            || confirmed.version == CREATE_VERSION
        {
            return Err(Error::State("directory confirmation mismatch".into()));
        }
        crate::creations::bind_created_identity(&tx, &confirmed.id, remote, &confirmed.version)?;
        // Early events for children may have used the not-yet-bound parent ID.
        tx.execute(
            "UPDATE nodes SET parent=?2 WHERE parent=?1",
            params![remote, confirmed.id],
        )?;
        tx.execute(
            "UPDATE nodes SET remote_version=?2 WHERE id=?1",
            params![confirmed.id, confirmed.version],
        )?;
        crate::filesystem::write_metadata(&tx, confirmed)?;
        crate::namespace::rebase_jobs(&tx, &confirmed.id, CREATE_VERSION, &confirmed.version)?;
        tx.execute(
            "UPDATE namespace_ops SET state='committed' WHERE id=?1",
            [op],
        )?;
        let remaining:bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM namespace_ops WHERE node=?1 AND state NOT IN ('committed','superseded'))",[&confirmed.id],|r|r.get(0))?;
        if remaining {
            tx.execute(
                "UPDATE namespace_overlay SET remote=?2 WHERE node=?1",
                params![confirmed.id, serde_json::to_string(&Some(confirmed))?],
            )?;
        } else {
            tx.execute(
                "DELETE FROM namespace_overlay WHERE node=?1",
                [&confirmed.id],
            )?;
        }
        tx.commit()?;
        Ok(())
    }
}
