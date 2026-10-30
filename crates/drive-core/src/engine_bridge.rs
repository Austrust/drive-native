//! Durable boundary between an external engine's commit and our namespace.
//! No vendor database writes, network operations or filesystem deletions here.
use crate::{Change, Error, Node, Result, Store};
use rusqlite::{OptionalExtension, params};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EngineIntent {
    pub key: String,
    pub node: String,
    pub body: String,
    pub baseline: Option<Node>,
    pub phase: String,
    pub attempts: u32,
    pub retry_at: i64,
    pub error: Option<String>,
}

impl Store {
    pub fn engine_intent(&self, key: &str) -> Result<Option<EngineIntent>> {
        let raw: Option<(String,String,String,Option<String>,String,u32,i64,Option<String>)> = self.db.query_row(
            "SELECT key,node,body,baseline,phase,attempts,retry_at,error FROM engine_bridge_intents WHERE key=?1",
            [key], |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?,r.get(6)?,r.get(7)?))
        ).optional()?;
        raw.map(|(key,node,body,baseline,phase,attempts,retry_at,error)| Ok(EngineIntent {
            key,node,body,baseline:baseline.map(|b|serde_json::from_str(&b)).transpose()?,phase,attempts,retry_at,error
        })).transpose()
    }
    /// Must succeed before the adapter allows the selected namespace operation.
    /// Repeated requests keep the original baseline for crash recovery.
    pub fn prepare_engine_intent(&mut self, key: &str, node: &str, body: &str) -> Result<()> {
        if key.is_empty() || key.len()>128 || node.is_empty() || node.len()>1024 || body.len()>65536 {
            return Err(Error::State("invalid engine intent".into()));
        }
        if let Some(old) = self.engine_intent(key)? {
            if old.node != node || old.body != body { return Err(Error::Integrity); }
            if old.phase != "applied" {
                self.db.execute("UPDATE engine_bridge_intents SET retry_at=0,error=NULL WHERE key=?1",[key])?;
            }
            return Ok(());
        }
        let baseline = match self.node(&self.local_identity(node)?) {
            Ok(n) => Some(serde_json::to_string(&n)?),
            Err(e) if e.is_not_found() => None,
            Err(e) => return Err(e),
        };
        self.db.execute("INSERT INTO engine_bridge_intents(key,node,body,baseline,phase) VALUES(?1,?2,?3,?4,'prepared')",
            params![key,node,body,baseline])?;
        Ok(())
    }
    /// Handler success is not yet an index observation, and never a NAS receipt.
    pub fn engine_handler_finished(&mut self, key: &str, succeeded: bool) -> Result<()> {
        if self.engine_intent(key)?.is_none() { return Err(Error::State("unknown engine intent".into())); }
        self.db.execute("UPDATE engine_bridge_intents SET phase=?2,retry_at=0 WHERE key=?1 AND phase!='applied'",
            params![key,if succeeded {"succeeded"} else {"failed"}])?;
        Ok(())
    }
    pub fn engine_intents_due(&self, now: u64) -> Result<Vec<EngineIntent>> {
        let mut query=self.db.prepare("SELECT key FROM engine_bridge_intents WHERE phase!='applied' AND retry_at<=?1 ORDER BY retry_at,key LIMIT 64")?;
        let keys=query.query_map([i64::try_from(now).unwrap_or(i64::MAX)],|r|r.get::<_,String>(0))?
            .collect::<std::result::Result<Vec<_>,_>>()?;
        keys.into_iter().map(|key|self.engine_intent(&key)?.ok_or_else(||Error::State("intent disappeared".into()))).collect()
    }
    pub fn retry_engine_intent(&mut self, key: &str, now: u64, reason: &str) -> Result<()> {
        // Only caller-defined categories, never vendor error payloads or paths.
        if !matches!(reason,"index_pending"|"index_unavailable"|"namespace_changed"|"local_state_error") {
            return Err(Error::State("invalid bridge error category".into()));
        }
        let attempt=self.engine_intent(key)?.ok_or_else(||Error::State("intent missing".into()))?.attempts;
        let delay=(1u64<<attempt.min(6)).min(60);
        let retry=i64::try_from(now.saturating_add(delay)).unwrap_or(i64::MAX);
        self.db.execute("UPDATE engine_bridge_intents SET attempts=MIN(attempts+1,1000000),retry_at=?2,error=?3 WHERE key=?1 AND phase!='applied'",params![key,retry,reason])?;
        Ok(())
    }
    pub fn next_engine_intent_retry(&self) -> Result<Option<u64>> {
        let value:Option<i64>=self.db.query_row("SELECT MIN(retry_at) FROM engine_bridge_intents WHERE phase!='applied'",[],|r|r.get(0))?;
        Ok(value.map(|v|v.max(0) as u64))
    }
    pub fn engine_intent_counts(&self) -> Result<(u64,u64)> {
        let (pending,failed):(i64,i64)=self.db.query_row("SELECT COUNT(*),COALESCE(SUM(error IS NOT NULL),0) FROM engine_bridge_intents WHERE phase!='applied'",[],|r|Ok((r.get(0)?,r.get(1)?)))?;
        Ok((u64::try_from(pending).map_err(|_|Error::Integrity)?,u64::try_from(failed).map_err(|_|Error::Integrity)?))
    }
    pub fn engine_namespace_pending(&self,id:&str)->Result<bool> {
        Ok(self.db.query_row("SELECT EXISTS(SELECT 1 FROM engine_bridge_intents i LEFT JOIN remote_aliases a ON a.remote=i.node WHERE COALESCE(a.node,i.node)=?1 AND i.phase!='applied')",[id],|r|r.get(0))?)
    }
    pub fn engine_delete_cutoff(&self, id: &str) -> Result<Option<u64>> {
        let id=self.local_identity(id)?;
        let value:Option<String>=self.db.query_row("SELECT cutoff FROM engine_bridge_tombstones WHERE node=?1",[id],|r|r.get(0)).optional()?;
        value.map(|v|v.parse().map_err(|_|Error::State("invalid engine tombstone".into()))).transpose()
    }
    /// Apply one index-confirmed observation, preserving local overlays/drafts.
    /// Namespace, cache-version alias, tombstone and journal completion commit
    /// together. A changed baseline is retained for reconciliation, not overwritten.
    pub fn finish_engine_observation(&mut self, key: &str, observed: Option<&Node>,
                                      same_content: bool, delete_cutoff: u64) -> Result<bool> {
        let intent=self.engine_intent(key)?.ok_or_else(||Error::State("intent missing".into()))?;
        if intent.phase=="applied" { return Ok(true); }
        if let Some(n)=observed {
            crate::state::valid_node(n)?;
            if n.id!=intent.node || n.parent.is_none() {
                return Err(Error::State("invalid engine observation".into()));
            }
        }
        let local_id=self.local_identity(&intent.node)?;
        let current:Option<(String,bool)>=self.db.query_row("SELECT remote_version,deleted FROM nodes WHERE id=?1",[&local_id],|r|Ok((r.get(0)?,r.get(1)?))).optional()?;
        if let Some((version,deleted))=&current {
            let base_matches=intent.baseline.as_ref().is_some_and(|n|n.version==*version);
            let already_matches=observed.is_some_and(|n|n.version==*version);
            if !base_matches && !already_matches && !(*deleted && observed.is_none()) { return Ok(false); }
        } else if intent.baseline.is_some() { return Ok(false); }
        let previous_cutoff=self.engine_delete_cutoff(&intent.node)?.unwrap_or(0);
        let tx=self.db.transaction()?;
        let change=observed.map(|n|Change::Upsert(n.clone())).unwrap_or_else(||Change::Delete{id:intent.node.clone()});
        let change=crate::creations::map_change(&tx,&change)?;
        if let Some(change)=crate::namespace::overlay_change(&tx,&change)? {
            match change {
                Change::Upsert(n) => {
                    let cycle:bool=tx.query_row("WITH RECURSIVE parents(id) AS (SELECT ?1 UNION SELECT n.parent FROM nodes n JOIN parents p ON n.id=p.id WHERE n.parent IS NOT NULL) SELECT EXISTS(SELECT 1 FROM parents WHERE id=?2)",params![n.parent,n.id],|r|r.get(0))?;
                    if cycle { return Err(Error::State("directory cycle".into())); }
                    tx.execute("INSERT INTO nodes(id,parent,name,remote_version,directory) VALUES(?1,?2,?3,?4,?5) ON CONFLICT(id) DO UPDATE SET parent=excluded.parent,name=excluded.name,remote_version=excluded.remote_version,directory=excluded.directory,deleted=CASE WHEN EXISTS(SELECT 1 FROM local_unlinks WHERE node=nodes.id) THEN 1 ELSE 0 END,conflict=CASE WHEN nodes.dirty_blob IS NOT NULL AND nodes.base_version!=excluded.remote_version AND NOT EXISTS(SELECT 1 FROM local_unlinks WHERE node=nodes.id) THEN 1 ELSE nodes.conflict END",params![n.id,n.parent,n.name,n.version,n.directory])?;
                    crate::filesystem::write_metadata(&tx,&n)?;
                    if same_content && !n.directory {
                        if let Some(base)=&intent.baseline {
                            if base.version!=n.version {
                                tx.execute("UPDATE nodes SET local_version=?3 WHERE id=?1 AND remote_version=?3 AND local_version=?2 AND dirty_blob IS NULL AND blob IS NOT NULL AND conflict=0",params![n.id,base.version,n.version])?;
                                tx.execute("INSERT OR REPLACE INTO content_version_aliases(node,old,new) VALUES(?1,?2,?3)",params![n.id,base.version,n.version])?;
                            }
                        }
                    }
                }
                Change::Delete{id} => {
                    // Retain immutable blobs and open handles. GC separately owns reclamation.
                    tx.execute("WITH RECURSIVE subtree(id) AS (SELECT ?1 UNION SELECT n.id FROM nodes n JOIN subtree s ON n.parent=s.id) UPDATE nodes SET deleted=CASE WHEN id IN (SELECT node FROM local_unlinks) THEN 1 WHEN id IN (SELECT node FROM namespace_overlay) THEN 0 ELSE 1 END,conflict=CASE WHEN id IN (SELECT node FROM local_unlinks) THEN conflict WHEN dirty_blob IS NOT NULL OR id IN (SELECT node FROM namespace_overlay) THEN 1 ELSE conflict END WHERE id IN subtree",[id])?;
                }
            }
        }
        if observed.is_none() {
            // Descendants also need a replay fence when a directory is removed.
            // Decimal text preserves the full unsigned native counter range.
            tx.execute("WITH RECURSIVE subtree(id) AS (SELECT ?1 UNION SELECT n.id FROM nodes n JOIN subtree s ON n.parent=s.id) INSERT INTO engine_bridge_tombstones(node,cutoff) SELECT id,?2 FROM subtree WHERE 1 ON CONFLICT(node) DO UPDATE SET cutoff=CASE WHEN length(excluded.cutoff)>length(engine_bridge_tombstones.cutoff) OR (length(excluded.cutoff)=length(engine_bridge_tombstones.cutoff) AND excluded.cutoff>engine_bridge_tombstones.cutoff) THEN excluded.cutoff ELSE engine_bridge_tombstones.cutoff END",params![local_id,previous_cutoff.max(delete_cutoff).to_string()])?;
        }
        tx.execute("UPDATE engine_bridge_intents SET phase='applied',error=NULL,retry_at=0 WHERE key=?1",[key])?;
        tx.commit()?;
        Ok(true)
    }
}
