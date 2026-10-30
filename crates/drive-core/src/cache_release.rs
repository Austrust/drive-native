//! Durable local-only two-copy reclaim. The external engine removes its clean
//! copy first; the immutable cache stays authoritative until its acknowledgement.
use crate::{DownloadTicket, Error, Result, Status, Store};
use rusqlite::{OptionalExtension, params};
use serde::Serialize;

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct CacheRelease {
    pub node: String,
    pub version: String,
    pub state: String,
    pub attempts: u32,
    pub retry_at: i64,
}
impl Store {
    pub fn cleanup_cache_releases(&mut self) -> Result<()> {
        let pending: bool = self.db.query_row(
            "SELECT EXISTS(SELECT 1 FROM cache_releases WHERE state='committed')",
            [],
            |r| r.get(0),
        )?;
        if pending {
            self.collect_garbage()?;
            self.db.execute(
                "UPDATE cache_releases SET state='reclaimed' WHERE state='committed'",
                [],
            )?;
        }
        Ok(())
    }
    pub fn cache_release(&self, id: &str) -> Result<Option<CacheRelease>> {
        Ok(self
            .db
            .query_row(
                "SELECT node,version,state,attempts,retry_at FROM cache_releases WHERE node=?1",
                [id],
                |r| {
                    Ok(CacheRelease {
                        node: r.get(0)?,
                        version: r.get(1)?,
                        state: r.get(2)?,
                        attempts: r.get(3)?,
                        retry_at: r.get(4)?,
                    })
                },
            )
            .optional()?)
    }
    pub fn cache_release_pending(&self, t: &DownloadTicket) -> Result<bool> {
        Ok(self
            .cache_release(&t.id)?
            .is_some_and(|r| r.version == t.version && r.state == "pending"))
    }
    pub fn begin_cache_release(&mut self, id: &str) -> Result<()> {
        let node = self.node(id)?;
        if node.directory
            || self.status(id)? != Status::LocalComplete
            || self.cache_is_protected(id)?
        {
            return Err(Error::State(
                "release requires a complete unprotected file".into(),
            ));
        }
        self.db.execute("INSERT INTO cache_releases(node,version,state,attempts,retry_at) VALUES(?1,?2,'pending',0,0)
            ON CONFLICT(node) DO UPDATE SET version=excluded.version,state='pending',attempts=0,retry_at=0",params![id,node.version])?;
        Ok(())
    }
    /// Called under the single Store mutex immediately before the external
    /// engine is allowed to remove anything. A later pin/open still retains the
    /// immutable body at finish, so this permit never authorizes data loss.
    pub fn authorize_cache_release(&mut self, t: &DownloadTicket) -> Result<bool> {
        if !self.cache_release_pending(t)? {
            return Ok(false);
        }
        if self.node(&t.id)?.version != t.version
            || self.status(&t.id)? != Status::LocalComplete
            || self.cache_is_protected(&t.id)?
        {
            self.db.execute(
                "UPDATE cache_releases SET state='retained' WHERE node=?1 AND version=?2",
                params![t.id, t.version],
            )?;
            return Ok(false);
        }
        Ok(true)
    }
    pub fn finish_cache_release(&mut self, t: &DownloadTicket) -> Result<bool> {
        if !self.authorize_cache_release(t)? {
            return Ok(false);
        }
        let tx = self.db.transaction()?;
        if tx.execute("UPDATE nodes SET blob=NULL,local_version=NULL WHERE id=?1 AND remote_version=?2 AND local_version=?2 AND dirty_blob IS NULL AND conflict=0 AND deleted=0",params![t.id,t.version])?!=1 {
            return Err(Error::State("release target changed".into()));
        }
        tx.execute(
            "DELETE FROM transfers WHERE node=?1 AND version=?2 AND direction='download'",
            params![t.id, t.version],
        )?;
        tx.execute(
            "UPDATE cache_releases SET state='committed' WHERE node=?1 AND version=?2",
            params![t.id, t.version],
        )?;
        tx.commit()?;
        Ok(true)
    }
    pub fn cache_releases_due(&mut self, now: u64) -> Result<Vec<DownloadTicket>> {
        // A remote version change supersedes only the old reclaim intent.
        self.db.execute("UPDATE cache_releases SET state='retained' WHERE state='pending' AND NOT EXISTS(SELECT 1 FROM nodes n WHERE n.id=cache_releases.node AND n.remote_version=cache_releases.version AND n.deleted=0)",[])?;
        let mut q=self.db.prepare("SELECT node,version FROM cache_releases WHERE state='pending' AND attempts<5 AND retry_at<=?1 ORDER BY retry_at,node")?;
        Ok(q.query_map([now.min(i64::MAX as u64) as i64], |r| {
            Ok(DownloadTicket {
                id: r.get(0)?,
                version: r.get(1)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?)
    }
    pub fn mark_cache_release_attempt(&mut self, t: &DownloadTicket, now: u64) -> Result<()> {
        self.db.execute("UPDATE cache_releases SET attempts=attempts+1,retry_at=?3 WHERE node=?1 AND version=?2 AND state='pending'",params![t.id,t.version,now.saturating_add(10).min(i64::MAX as u64) as i64])?;
        Ok(())
    }
    pub fn next_cache_release_retry(&self) -> Result<Option<u64>> {
        let value:Option<i64>=self.db.query_row("SELECT min(retry_at) FROM cache_releases WHERE (state='pending' AND attempts<5) OR state='committed'",[],|r|r.get(0))?;
        value
            .map(|v| {
                u64::try_from(v).map_err(|_| Error::State("invalid release retry time".into()))
            })
            .transpose()
    }
}
