//! Atomic local replacement, followed by conditional target upload and only
//! then identity-checked cleanup of an existing remote source. Never delete the
//! target as a prerequisite for saving its replacement.
use crate::{Error, Node, Result, Store};
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReplaceJob {
    /// Present only when the destination had no remote identity at submission.
    #[serde(default)]
    pub creation_origin: Option<crate::upload_origin::UploadOrigin>,
    pub operation: String,
    pub source: Node,
    pub target: Node,
    pub source_remote: Option<String>,
    pub target_remote: Option<String>,
    pub source_path: String,
    pub destination: String,
    pub cursor: String,
    pub blob: String,
    pub generation: i64,
    pub cleanup_node: Option<String>,
}
#[derive(Clone, Debug)]
pub struct QueuedReplace {
    pub job: ReplaceJob,
    pub state: String,
}
pub(crate) fn remote_id(db: &Connection, id: &str) -> Result<Option<String>> {
    let creating: bool = db.query_row(
        "SELECT remote_version=?2 FROM nodes WHERE id=?1",
        params![id, crate::creations::CREATE_VERSION],
        |r| r.get(0),
    )?;
    if creating {
        return Ok(None);
    }
    let alias: Option<String> = db
        .query_row(
            "SELECT remote FROM remote_aliases WHERE node=?1",
            [id],
            |r| r.get(0),
        )
        .optional()?;
    Ok(Some(alias.unwrap_or_else(|| id.to_owned())))
}
pub(crate) fn rebase_job(
    db: &Connection,
    body: &str,
    id: &str,
    base: &str,
    version: &str,
) -> Result<()> {
    let mut job: ReplaceJob = serde_json::from_str(body)?;
    if job.source.id == id && job.source.version == base {
        job.source.version = version.into();
        job.source_remote = remote_id(db, id)?;
    }
    if job.target.id == id && job.target.version == base {
        job.target.version = version.into();
        job.target_remote = remote_id(db, id)?;
    }
    db.execute(
        "UPDATE namespace_ops SET body=?2 WHERE id=?1",
        params![job.operation, serde_json::to_string(&job)?],
    )?;
    Ok(())
}
fn os(code: i32) -> Error {
    Error::Io(std::io::Error::from_raw_os_error(code))
}
impl Store {
    /// Requires source contents to have been hydrated. Existing open target
    /// handles must be protected by the filesystem before calling this method.
    pub fn replace_local(
        &mut self,
        parent: &str,
        name: &str,
        new_parent: &str,
        new_name: &str,
        expected_source: &str,
        expected_target: &str,
    ) -> Result<()> {
        let source_record = self.lookup(parent, name)?;
        let mut source = source_record.node;
        source.metadata = Some(crate::NodeMetadata {
            size: source_record.size,
            mtime: source_record.mtime,
            mode: source_record.mode,
        });
        let mut target = self.lookup(new_parent, new_name)?.node;
        if source.id != expected_source || target.id != expected_target {
            return Err(os(libc::EAGAIN));
        }
        if source.id == target.id {
            return Ok(());
        }
        if source.directory || target.directory {
            return Err(os(if source.directory && !target.directory {
                libc::ENOTDIR
            } else if !source.directory && target.directory {
                libc::EISDIR
            } else {
                libc::EOPNOTSUPP
            }));
        }
        if let Some(draft) = self.open_dirty_draft(&source.id)? {
            self.checkpoint_draft(&draft)?;
        }
        let (blob,generation,content_base,conflict):(Option<String>,i64,Option<String>,bool)=self.db.query_row("SELECT COALESCE(dirty_blob,blob),dirty_generation,COALESCE(base_version,local_version),conflict FROM nodes WHERE id=?1",[&source.id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?)))?;
        let blob = blob.ok_or(Error::Offline)?;
        let content_base = content_base.ok_or(Error::Offline)?;
        // Preserve the baseline of the destination's cached/edited version.
        target.version = self.db.query_row(
            "SELECT COALESCE(base_version,local_version,remote_version) FROM nodes WHERE id=?1",
            [&target.id],
            |r| r.get(0),
        )?;
        let source_remote = remote_id(&self.db, &source.id)?;
        let target_remote = remote_id(&self.db, &target.id)?;
        let job = ReplaceJob {
            creation_origin: None,
            operation: uuid::Uuid::new_v4().to_string(),
            source: source.clone(),
            target: target.clone(),
            source_remote,
            target_remote,
            source_path: crate::namespace::node_path(&self.db, &source.id)?,
            destination: crate::namespace::node_path(&self.db, &target.id)?,
            cursor: self.cursor()?,
            blob: blob.clone(),
            generation,
            cleanup_node: None,
        };
        let conflict = conflict || content_base != source.version;
        let tx = self.db.transaction()?;
        tx.execute(
            "INSERT INTO local_unlinks(node,confirmed) VALUES(?1,0)",
            [&target.id],
        )?;
        tx.execute("UPDATE nodes SET deleted=1 WHERE id=?1", [&target.id])?;
        tx.execute(
            "UPDATE operations SET state='superseded' WHERE node IN (?1,?2) AND state='pending'",
            params![source.id, target.id],
        )?;
        tx.execute(
            "INSERT OR IGNORE INTO namespace_overlay(node,remote) VALUES(?1,?2)",
            params![source.id, serde_json::to_string(&Some(&source))?],
        )?;
        tx.execute(
            "INSERT INTO namespace_ops(id,node,body,state,kind) VALUES(?1,?2,?3,?4,'replace')",
            params![
                job.operation,
                source.id,
                serde_json::to_string(&job)?,
                if conflict { "conflict" } else { "pending" }
            ],
        )?;
        tx.execute("UPDATE nodes SET parent=?2,name=?3,dirty_blob=?4,base_version=?5,conflict=?6 WHERE id=?1",params![source.id,new_parent,new_name,blob,content_base,conflict])?;
        tx.commit()?;
        Ok(())
    }
    pub fn next_replace(&self) -> Result<Option<QueuedReplace>> {
        if self.next_namespace_kind()?.as_deref() != Some("replace") {
            return Ok(None);
        }
        let (body,state):(String,String)=self.db.query_row("SELECT body,state FROM namespace_ops WHERE id NOT IN (SELECT operation FROM access_blocked_operations) AND state NOT IN ('committed','superseded') ORDER BY seq LIMIT 1",[],|r|Ok((r.get(0)?,r.get(1)?)))?;
        Ok(Some(QueuedReplace {
            job: serde_json::from_str(&body)?,
            state,
        }))
    }
    pub fn replacement_file(&self, job: &ReplaceJob) -> Result<std::fs::File> {
        if uuid::Uuid::parse_str(&job.blob).is_err() {
            return Err(Error::State("invalid replacement snapshot".into()));
        }
        Ok(std::fs::File::open(
            self.root.join("blobs").join(&job.blob),
        )?)
    }
    /// Persist read-only preparation before the target creation is sent. Legacy
    /// uncertain operations can never gain historical provenance retroactively.
    pub fn prepare_replace_origin(
        &mut self,
        id: &str,
        origin: crate::upload_origin::UploadOrigin,
        sha256: &str,
    ) -> Result<()> {
        let mut head = self
            .next_replace()?
            .ok_or_else(|| Error::State("replacement not queue head".into()))?;
        if head.job.operation != id || head.state != "pending" || head.job.target_remote.is_some() {
            return Err(Error::State(
                "replacement cannot acquire creation provenance".into(),
            ));
        }
        let path = format!(
            "{}/{}",
            self.scope_path()?.trim_end_matches('/'),
            head.job.destination
        );
        origin.validate(&head.job.target, &path, sha256)?;
        head.job.creation_origin = Some(origin);
        self.db.execute(
            "UPDATE namespace_ops SET body=?2 WHERE id=?1 AND kind='replace' AND state='pending'",
            params![id, serde_json::to_string(&head.job)?],
        )?;
        Ok(())
    }
    pub fn mark_replace_started(&mut self, id: &str) -> Result<()> {
        let head = self
            .next_replace()?
            .ok_or_else(|| Error::State("replacement not queue head".into()))?;
        if head.job.operation != id || head.state != "pending" {
            return Err(Error::State("replacement requires reconciliation".into()));
        }
        let busy: bool = self.db.query_row(
            "SELECT EXISTS(SELECT 1 FROM operations WHERE node NOT IN (SELECT id FROM access_blocked) AND state IN ('inflight','uncertain'))",
            [],
            |r| r.get(0),
        )?;
        if busy {
            return Err(os(libc::EBUSY));
        }
        let remote: String = self.db.query_row(
            "SELECT remote_version FROM nodes WHERE id=?1",
            [&head.job.source.id],
            |r| r.get(0),
        )?;
        if remote != head.job.source.version {
            self.conflict_replace(id)?;
            return Err(Error::State("replacement source changed".into()));
        }
        self.db.execute("UPDATE namespace_ops SET state='inflight' WHERE id=?1 AND kind='replace' AND state='pending'",[id])?;
        Ok(())
    }
    pub fn mark_replace_uncertain(&mut self, id: &str) -> Result<()> {
        if self.db.execute("UPDATE namespace_ops SET state=CASE state WHEN 'inflight' THEN 'uncertain' ELSE 'cleanup_uncertain' END WHERE id=?1 AND kind='replace' AND state IN ('inflight','cleanup_inflight')",[id])?!=1{return Err(Error::State("replacement not in flight".into()));}
        Ok(())
    }
    pub fn conflict_replace(&mut self, id: &str) -> Result<()> {
        let tx = self.db.transaction()?;
        if tx.execute("UPDATE namespace_ops SET state='conflict' WHERE id=?1 AND kind='replace' AND state!='committed'",[id])?!=1{return Err(Error::State("replacement not active".into()));}
        tx.execute(
            "UPDATE nodes SET conflict=1 WHERE id=(SELECT node FROM namespace_ops WHERE id=?1)",
            [id],
        )?;
        tx.commit()?;
        Ok(())
    }
    /// Publish a verified target version and remap remote identities in the same
    /// transaction. Local source inode survives; old target handles stay orphaned.
    pub fn confirm_replace_upload(
        &mut self,
        id: &str,
        confirmed: &Node,
        remote: &str,
    ) -> Result<()> {
        if remote.is_empty() {
            return Err(Error::State("replacement remote identity missing".into()));
        }
        let tx = self.db.transaction()?;
        let (body, state): (String, String) = tx.query_row(
            "SELECT body,state FROM namespace_ops WHERE id=?1 AND kind='replace'",
            [id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )?;
        let mut job: ReplaceJob = serde_json::from_str(&body)?;
        if !matches!(state.as_str(), "inflight" | "uncertain")
            || confirmed.id != job.target.id
            || confirmed.directory
            || confirmed.parent != job.target.parent
            || confirmed.name != job.target.name
            || confirmed.version.is_empty()
            || job.target_remote.as_deref().is_some_and(|v| v != remote)
        {
            return Err(Error::State("replacement acknowledgement mismatch".into()));
        }
        // A target event may precede its receipt. Newer evidence must win.
        let observed: String = tx.query_row(
            "SELECT remote_version FROM nodes WHERE id=?1",
            [&job.target.id],
            |r| r.get(0),
        )?;
        if observed != job.target.version && observed != confirmed.version {
            return Err(Error::State(
                "target changed during replacement confirmation".into(),
            ));
        }
        // Retain the old source identity under a hidden object until its own
        // conditional deletion is confirmed; late source events cannot mutate
        // the newly bound destination object.
        tx.execute(
            "DELETE FROM remote_aliases WHERE node IN (?1,?2)",
            params![job.source.id, job.target.id],
        )?;
        if let Some(old) = &job.source_remote {
            if old == remote {
                return Err(Error::State(
                    "replacement source/target identity collision".into(),
                ));
            }
            let shadow = format!("local:{}", uuid::Uuid::new_v4());
            tx.execute("INSERT INTO nodes(id,parent,name,remote_version,directory,deleted) VALUES(?1,?2,?3,?4,0,1)",params![shadow,job.source.parent,job.source.name,job.source.version])?;
            let mut n = job.source.clone();
            n.id = shadow.clone();
            crate::filesystem::write_metadata(&tx, &n)?;
            tx.execute(
                "INSERT INTO local_unlinks(node,confirmed) VALUES(?1,0)",
                [&shadow],
            )?;
            tx.execute(
                "INSERT INTO remote_aliases(remote,node) VALUES(?1,?2)",
                params![old, shadow],
            )?;
            job.cleanup_node = Some(shadow);
        }
        // Creation echoes can be separate read-only rows until the receipt is
        // bound. Do not silently fold a different or locally edited version.
        if job.target_remote.is_none() && remote != job.source.id && remote != job.target.id {
            let echo: Option<(String, bool)> = tx
                .query_row(
                    "SELECT remote_version,dirty_blob IS NOT NULL FROM nodes WHERE id=?1",
                    [remote],
                    |r| Ok((r.get(0)?, r.get(1)?)),
                )
                .optional()?;
            if let Some((v, dirty)) = echo {
                if v != confirmed.version || dirty {
                    return Err(Error::State("replacement creation echo changed".into()));
                }
                for (table, column) in [
                    ("nodes", "id"),
                    ("node_metadata", "node"),
                    ("fs_inodes", "node"),
                ] {
                    tx.execute(&format!("DELETE FROM {table} WHERE {column}=?1"), [remote])?;
                }
            }
        }
        tx.execute(
            "INSERT INTO remote_aliases(remote,node) VALUES(?1,?2)",
            params![remote, job.source.id],
        )?;
        tx.execute(
            "DELETE FROM local_creations WHERE node IN (?1,?2)",
            params![job.source.id, job.target.id],
        )?;
        tx.execute(
            "UPDATE local_unlinks SET confirmed=1 WHERE node=?1",
            [&job.target.id],
        )?;
        tx.execute("UPDATE nodes SET remote_version=?2,local_version=?2,blob=?3,dirty_blob=CASE WHEN dirty_generation=?4 THEN NULL ELSE dirty_blob END,base_version=CASE WHEN dirty_generation=?4 THEN NULL ELSE ?2 END,conflict=0 WHERE id=?1",params![job.source.id,confirmed.version,job.blob,job.generation])?;
        tx.execute("UPDATE drafts SET base=?2,dirty=CASE WHEN EXISTS(SELECT 1 FROM local_unlinks WHERE node=?1) THEN dirty WHEN generation=checkpoint AND ?3=(SELECT dirty_generation FROM nodes WHERE id=?1) THEN 0 ELSE dirty END WHERE node=?1",params![job.source.id,confirmed.version,job.generation])?;
        tx.execute(
            "UPDATE operations SET base=?3 WHERE node=?1 AND state='pending' AND base=?2",
            params![job.source.id, job.source.version, confirmed.version],
        )?;
        if job.source.version != confirmed.version {
            tx.execute(
                "INSERT OR REPLACE INTO content_version_aliases(node,old,new) VALUES(?1,?2,?3)",
                params![job.source.id, job.source.version, confirmed.version],
            )?;
        }
        let mut new = confirmed.clone();
        new.id = job.source.id.clone();
        new.metadata = job.source.metadata.clone();
        crate::filesystem::write_metadata(&tx, &new)?;
        tx.execute(
            "UPDATE namespace_overlay SET remote=?2 WHERE node=?1",
            params![new.id, serde_json::to_string(&Some(&new))?],
        )?;
        crate::namespace::rebase_jobs(
            &tx,
            &job.source.id,
            &job.source.version,
            &confirmed.version,
        )?;
        let next = if job.cleanup_node.is_some() {
            "cleanup_pending"
        } else {
            "committed"
        };
        tx.execute(
            "UPDATE namespace_ops SET state=?2,body=?3 WHERE id=?1",
            params![id, next, serde_json::to_string(&job)?],
        )?;
        if next == "committed" {
            finish_overlay(&tx, &new.id)?;
        }
        tx.commit()?;
        Ok(())
    }
    pub fn replacement_cleanup(&self, job: &ReplaceJob) -> Result<crate::removals::RemoveJob> {
        let mut node = job.source.clone();
        node.id = job
            .cleanup_node
            .clone()
            .ok_or_else(|| Error::State("replacement cleanup absent".into()))?;
        Ok(crate::removals::RemoveJob {
            operation: job.operation.clone(),
            node,
            path: job.source_path.clone(),
            cursor: job.cursor.clone(),
        })
    }
    pub fn mark_replace_cleanup_started(&mut self, id: &str) -> Result<()> {
        let q = self
            .next_replace()?
            .ok_or_else(|| Error::State("replacement not head".into()))?;
        if q.job.operation != id || q.state != "cleanup_pending" {
            return Err(Error::State("cleanup must reconcile".into()));
        }
        self.db.execute(
            "UPDATE namespace_ops SET state='cleanup_inflight' WHERE id=?1",
            [id],
        )?;
        Ok(())
    }
    pub fn confirm_replace_cleanup(&mut self, id: &str, identity: &str) -> Result<()> {
        let tx = self.db.transaction()?;
        let (body, state): (String, String) = tx.query_row(
            "SELECT body,state FROM namespace_ops WHERE id=?1 AND kind='replace'",
            [id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )?;
        let job: ReplaceJob = serde_json::from_str(&body)?;
        if !matches!(state.as_str(), "cleanup_inflight" | "cleanup_uncertain")
            || job.cleanup_node.as_deref() != Some(identity)
        {
            return Err(Error::State("cleanup receipt mismatch".into()));
        }
        tx.execute(
            "UPDATE local_unlinks SET confirmed=1 WHERE node=?1",
            [identity],
        )?;
        tx.execute(
            "UPDATE namespace_ops SET state='committed' WHERE id=?1",
            [id],
        )?;
        finish_overlay(&tx, &job.source.id)?;
        tx.commit()?;
        Ok(())
    }
}
fn finish_overlay(db: &Connection, id: &str) -> Result<()> {
    let active:bool=db.query_row("SELECT EXISTS(SELECT 1 FROM namespace_ops WHERE node=?1 AND state NOT IN ('committed','superseded'))",[id],|r|r.get(0))?;
    if !active {
        db.execute("DELETE FROM namespace_overlay WHERE node=?1", [id])?;
    }
    Ok(())
}
