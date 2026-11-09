use crate::{Error, Node, Result, Store};
use std::fs::File;
pub struct UploadJob {
    pub operation: String,
    pub node: Node,
    pub relative_path: String,
    pub file: File,
    pub state: String,
    pub mtime: Option<u32>,
}
impl Store {
    pub fn upload_operations(&self, state: &str) -> Result<Vec<String>> {
        if !matches!(state, "pending" | "uncertain" | "abandoned") {
            return Err(Error::State("invalid operation selection".into()));
        }
        let mut q = self
            .db
            .prepare("SELECT id FROM operations WHERE kind='upload' AND node NOT IN (SELECT id FROM access_blocked) AND state=?1 ORDER BY rowid")?;
        Ok(q.query_map([state], |r| r.get::<_, String>(0))?
            .collect::<std::result::Result<Vec<_>, _>>()?)
    }
    pub fn upload_job(&self, op: &str) -> Result<UploadJob> {
        // One row read for the operation (including its durable mtime) and one
        // recursive walk for the whole ancestor chain. The previous form spent
        // two point lookups per ancestor, so a depth-D job cost 2+2D queries on
        // every upload; the chain never leaves the nodes table and only needs
        // names.
        let (id, base, blob, state, mtime): (String, String, String, String, Option<i64>) =
            self.db.query_row(
                "SELECT node,base,blob,state,
                        (SELECT mtime FROM upload_times WHERE operation=operations.id)
                 FROM operations WHERE id=?1 AND kind='upload'",
                [op],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
            )?;
        let mut node = self.node_for_handle(&id)?;
        node.version = base;
        let parts: Vec<String> = {
            // The anchor is the uploaded node, so depth grows towards the root;
            // walking DESC yields root-to-leaf order. The root is the only row
            // whose parent is NULL; the depth bound turns a corrupted cycle into
            // a bounded query instead of infinite recursion.
            let mut q = self.db.prepare(
                "WITH RECURSIVE chain(id,parent,name,depth) AS (
                   SELECT id,parent,name,0 FROM nodes WHERE id=?1
                   UNION ALL
                   SELECT n.id,n.parent,n.name,c.depth+1 FROM nodes n JOIN chain c ON n.id=c.parent
                   WHERE c.depth < 65535)
                 SELECT name,parent FROM chain ORDER BY depth DESC",
            )?;
            let rows: Vec<(String, Option<String>)> = q
                .query_map([&id], |r| Ok((r.get(0)?, r.get(1)?)))?
                .collect::<std::result::Result<Vec<_>, _>>()?;
            // The deepest row in DESC order is the root; anything else means the
            // chain is cyclic or does not reach an authoritative root.
            let rooted = matches!(rows.first(), Some((_, None)));
            if !rooted {
                return Err(Error::State("path cycle".into()));
            }
            // Drop the root: names run root-child -> uploaded node.
            rows.into_iter().skip(1).map(|(name, _)| name).collect()
        };
        if parts.is_empty() {
            return Err(Error::State("cannot upload root".into()));
        }
        let mtime = mtime
            .map(u32::try_from)
            .transpose()
            .map_err(|_| Error::State("upload mtime exceeds the supported range".into()))?;
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
            mtime,
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
