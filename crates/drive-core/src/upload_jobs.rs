use crate::{Error, Node, Result, Store};
use std::fs::File;
pub struct UploadJob {
    pub operation: String,
    pub node: Node,
    pub relative_path: String,
    pub file: File,
    pub state: String,
}
impl Store {
    pub fn upload_operations(&self, state: &str) -> Result<Vec<String>> {
        if !matches!(state, "pending" | "uncertain") {
            return Err(Error::State("invalid operation selection".into()));
        }
        let mut q = self
            .db
            .prepare("SELECT id FROM operations WHERE kind='upload' AND node NOT IN (SELECT id FROM access_blocked) AND state=?1 ORDER BY rowid")?;
        Ok(q.query_map([state], |r| r.get::<_, String>(0))?
            .collect::<std::result::Result<Vec<_>, _>>()?)
    }
    pub fn upload_job(&self, op: &str) -> Result<UploadJob> {
        let (id, base, blob, state): (String, String, String, String) = self.db.query_row(
            "SELECT node,base,blob,state FROM operations WHERE id=?1 AND kind='upload'",
            [op],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )?;
        let mut node = self.node_for_handle(&id)?;
        node.version = base;
        let mut parts = Vec::new();
        let mut current = self.node_for_handle(&id)?;
        let mut seen = std::collections::HashSet::new();
        while let Some(parent) = current.parent {
            if !seen.insert(current.id) {
                return Err(Error::State("path cycle".into()));
            }
            parts.push(current.name);
            current = self.node_for_handle(&parent)?;
        }
        parts.reverse();
        if parts.is_empty() {
            return Err(Error::State("cannot upload root".into()));
        }
        Ok(UploadJob {
            operation: op.into(),
            node,
            relative_path: if matches!(state.as_str(), "inflight" | "uncertain") {
                self.db.query_row(
                    "SELECT path FROM upload_paths WHERE operation=?1",
                    [op],
                    |r| r.get(0),
                )?
            } else {
                parts.join("/")
            },
            file: File::open(self.root.join("blobs").join(blob))?,
            state,
        })
    }
    pub fn conflict_upload(&mut self, op: &str) -> Result<()> {
        let tx = self.db.transaction()?;
        tx.execute(
            "UPDATE nodes SET conflict=1 WHERE id=(SELECT node FROM operations WHERE id=?1)",
            [op],
        )?;
        tx.execute("UPDATE operations SET state='conflict' WHERE id=?1 AND state IN ('inflight','uncertain')",[op])?;
        tx.commit()?;
        Ok(())
    }
    pub fn scope_path(&self) -> Result<String> {
        let scope: String = self.db.query_row(
            "SELECT value FROM metadata WHERE key='native_scope'",
            [],
            |r| r.get(0),
        )?;
        let value: serde_json::Value = serde_json::from_str(&scope)?;
        Ok(value
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::State("native scope path absent".into()))?
            .to_string())
    }
}
