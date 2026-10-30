//! Resolve an unbound local mkdir tree without adopting the colliding remote tree.
use crate::{Error, Node, Result, Store, creations::CREATE_VERSION, directories::DirectoryJob};
use rusqlite::params;
use serde::Serialize;
use std::collections::{HashMap, HashSet};

#[derive(Serialize)]
pub(crate) struct DirectoryConflictPlan {
    nodes: Vec<Node>,
    jobs: Vec<(i64, DirectoryJob, String)>,
    operations: Vec<String>,
    drafts: Vec<String>,
}
#[derive(Serialize)]
pub struct KeptDirectory {
    pub node: Node,
    pub path: String,
    pub old_name: String,
}
fn references(value: &serde_json::Value, ids: &HashSet<String>, path: &str) -> bool {
    match value {
        serde_json::Value::String(s) => {
            ids.contains(s) || s == path || s.starts_with(&format!("{path}/"))
        }
        serde_json::Value::Array(v) => v.iter().any(|v| references(v, ids, path)),
        serde_json::Value::Object(v) => v.values().any(|v| references(v, ids, path)),
        _ => false,
    }
}
impl Store {
    /// Local metadata only. This gate deliberately excludes mixed remote/local
    /// trees and other unresolved namespace operations; it never guesses ownership.
    pub(crate) fn directory_conflict_plan(
        &self,
        id: &str,
    ) -> Result<Option<DirectoryConflictPlan>> {
        let root = self.node(id)?;
        if !root.directory || root.version != CREATE_VERSION {
            return Ok(None);
        }
        let Some(parent) = root.parent.as_deref() else {
            return Ok(None);
        };
        let parent = self.node(parent)?;
        if !parent.directory
            || parent.version == CREATE_VERSION
            || crate::namespace::blocks_upload(&self.db, &parent.id)?
        {
            return Ok(None);
        }
        let ids: Vec<String> = {
            let mut q=self.db.prepare("WITH RECURSIVE tree(id) AS (SELECT ?1 UNION SELECT n.id FROM nodes n JOIN tree t ON n.parent=t.id) SELECT id FROM tree ORDER BY id LIMIT 10001")?;
            q.query_map([id], |r| r.get(0))?
                .collect::<std::result::Result<_, _>>()?
        };
        if ids.len() > 10000 {
            return Ok(None);
        }
        let set: HashSet<_> = ids.iter().cloned().collect();
        let mut nodes = Vec::new();
        let mut operations = Vec::new();
        let mut drafts = Vec::new();
        for member in &ids {
            let eligible:bool=self.db.query_row("SELECT remote_version=?2 AND deleted=0 AND (conflict=0 OR id=?3) AND EXISTS(SELECT 1 FROM local_creations WHERE node=n.id) AND NOT EXISTS(SELECT 1 FROM remote_aliases WHERE node=n.id) AND NOT EXISTS(SELECT 1 FROM local_unlinks WHERE node=n.id) FROM nodes n WHERE id=?1",params![member,CREATE_VERSION,id],|r|r.get(0))?;
            if !eligible || !crate::creations::is_local_identity(member) {
                return Ok(None);
            }
            nodes.push(self.node(member)?);
            let unsafe_upload:bool=self.db.query_row("SELECT EXISTS(SELECT 1 FROM operations o WHERE node=?1 AND state NOT IN ('pending','superseded')) OR EXISTS(SELECT 1 FROM upload_origins x JOIN operations o ON o.id=x.operation WHERE o.node=?1 AND o.state!='superseded') OR EXISTS(SELECT 1 FROM upload_paths x JOIN operations o ON o.id=x.operation WHERE o.node=?1 AND o.state!='superseded')",[member],|r|r.get(0))?;
            if unsafe_upload {
                return Ok(None);
            }
            let mut q=self.db.prepare("SELECT json_array(id,node,base,blob,generation,state) FROM operations WHERE node=?1 AND state!='superseded' ORDER BY id")?;
            operations.extend(
                q.query_map([member], |r| r.get::<_, String>(0))?
                    .collect::<std::result::Result<Vec<_>, _>>()?,
            );
            let mut q=self.db.prepare("SELECT json_array(node,token,base,generation,checkpoint,dirty) FROM drafts WHERE node=?1")?;
            drafts.extend(
                q.query_map([member], |r| r.get::<_, String>(0))?
                    .collect::<std::result::Result<Vec<_>, _>>()?,
            );
        }
        let path = crate::namespace::node_path(&self.db, id)?;
        let rows: Vec<(i64, String, String, String, String, String)> = {
            let mut q=self.db.prepare("SELECT seq,id,node,body,state,kind FROM namespace_ops WHERE state NOT IN ('committed','superseded') ORDER BY seq LIMIT 10001")?;
            q.query_map([], |r| {
                Ok((
                    r.get(0)?,
                    r.get(1)?,
                    r.get(2)?,
                    r.get(3)?,
                    r.get(4)?,
                    r.get(5)?,
                ))
            })?
            .collect::<std::result::Result<_, _>>()?
        };
        if rows.len() > 10000 {
            return Ok(None);
        }
        let mut jobs = Vec::new();
        let mut visited = HashSet::new();
        for (seq, operation, member, body, state, kind) in rows {
            if !set.contains(&member) {
                if references(&serde_json::from_str(&body)?, &set, &path) {
                    return Ok(None);
                }
                continue;
            }
            if kind != "mkdir"
                || state != if member == id { "conflict" } else { "pending" }
                || !visited.insert(member.clone())
            {
                return Ok(None);
            }
            let job: DirectoryJob = serde_json::from_str(&body)?;
            let node = self.node(&member)?;
            if !node.directory
                || job.operation != operation
                || job.node.id != member
                || job.node.name != node.name
                || job.node.parent != node.parent
                || job.node.version != CREATE_VERSION
                || job.parent.id != node.parent.as_deref().unwrap_or("")
                || job.path != crate::namespace::node_path(&self.db, &member)?
                || job.creation_contract != 1
                || (member != id && job.origin.is_some())
            {
                return Ok(None);
            }
            jobs.push((seq, job, state));
        }
        if !visited.contains(id)
            || nodes
                .iter()
                .any(|n| n.directory && !visited.contains(&n.id))
        {
            return Ok(None);
        }
        // Creation ordering is part of the persisted contract, not just UI state.
        let seqs: HashMap<_, _> = jobs
            .iter()
            .map(|(seq, j, _)| (j.node.id.as_str(), *seq))
            .collect();
        if jobs
            .iter()
            .any(|(seq, j, _)| seqs.get(j.parent.id.as_str()).is_some_and(|p| p >= seq))
        {
            return Ok(None);
        }
        Ok(Some(DirectoryConflictPlan {
            nodes,
            jobs,
            operations,
            drafts,
        }))
    }

    pub fn keep_directory_both(
        &mut self,
        root: &str,
        id: &str,
        token: &str,
        name: &str,
    ) -> Result<KeptDirectory> {
        let issue = self
            .sync_issue(root, id)?
            .ok_or_else(|| Error::State("directory conflict absent".into()))?;
        if !issue.can_keep_directory || issue.token != token {
            return Err(Error::State(
                "directory conflict changed or requires reconciliation".into(),
            ));
        }
        let plan = self
            .directory_conflict_plan(id)?
            .ok_or_else(|| Error::State("directory conflict requires reconciliation".into()))?;
        let mut desired = self.node(id)?;
        let old_name = desired.name.clone();
        desired.name = name.into();
        crate::state::valid_node(&desired)?;
        let parent = desired
            .parent
            .as_deref()
            .ok_or_else(|| Error::State("cannot rename task root".into()))?;
        let occupied: bool = self.db.query_row(
            "SELECT EXISTS(SELECT 1 FROM nodes WHERE parent=?1 AND name=?2 AND deleted=0)",
            params![parent, name],
            |r| r.get(0),
        )?;
        if name.len() > 255 || occupied || name == old_name {
            return Err(Error::State(
                "directory copy name invalid or occupied".into(),
            ));
        }
        let cursor = self.cursor()?;
        let tx = self.db.transaction()?;
        tx.execute(
            "UPDATE nodes SET name=?2,conflict=0 WHERE id=?1",
            params![id, name],
        )?;
        for (seq, mut job, _) in plan.jobs {
            let original = job.clone();
            if job.node.id == id {
                job.node = desired.clone();
                job.operation = uuid::Uuid::new_v4().to_string();
            }
            if job.parent.id == id {
                job.parent = desired.clone();
            }
            job.path = crate::namespace::node_path(&tx, &job.node.id)?;
            job.cursor = cursor.clone();
            job.origin = None;
            // Keep the queue slot: child mkdirs cannot precede their parent.
            tx.execute(
                "UPDATE namespace_ops SET id=?2,body=?3,state='pending' WHERE seq=?1",
                params![seq, job.operation, serde_json::to_string(&job)?],
            )?;
            tx.execute(
                "UPDATE namespace_overlay SET remote=?2 WHERE node=?1",
                params![job.node.id, serde_json::to_string(&Some(&job.node))?],
            )?;
            if job.node.id == id {
                // Preserve the rejected attempt, including its original provenance.
                tx.execute("INSERT INTO namespace_ops(id,node,body,state,kind) VALUES(?1,?2,?3,'superseded','mkdir')",params![original.operation,id,serde_json::to_string(&original)?])?;
            }
        }
        let path = crate::namespace::node_path(&tx, id)?;
        tx.commit()?;
        Ok(KeptDirectory {
            node: desired,
            path,
            old_name,
        })
    }
}
