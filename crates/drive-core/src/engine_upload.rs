//! Official-engine upload provenance. Matching bytes/path alone are not a receipt.
//! Staging binds an immutable local operation to one private-workspace inode;
//! admission binds that inode to a specific official worker attempt before I/O.
use crate::{Error, Result, Store};
use rusqlite::{OptionalExtension, params};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::io::Read;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EngineUploadClaim {
    pub session: u64,
    pub device: u64,
    pub inode: u64,
    pub size: u64,
    pub mtime_sec: u64,
    pub mtime_nsec: u64,
    pub path: String,
    pub remote_id: String,
    pub base_version: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EngineUploadSource {
    pub claim: EngineUploadClaim,
    pub sha256: String,
    pub md4: String,
}
#[derive(Clone, Debug)]
pub struct EngineUploadStage {
    pub operation: String,
    pub source: EngineUploadSource,
    pub state: String,
    pub attempt: Option<String>,
    pub receipt: Option<String>,
    pub error: Option<String>,
}
fn invalid(message: &str) -> Error { Error::State(message.into()) }
fn hex(value: &str, length: usize) -> bool {
    value.len()==length && value.bytes().all(|b|b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
impl EngineUploadClaim {
    fn key(&self) -> Result<String> {
        if self.session==0 || self.inode==0 || self.size>i64::MAX as u64
            || self.mtime_nsec>=1_000_000_000 || self.path.len()>16384
            || !self.path.starts_with('/') || self.path.contains('\0')
            || self.path[1..].split('/').any(|s|s.is_empty() || s=="." || s=="..")
            || self.remote_id.len()>1024 || self.base_version.is_empty() || self.base_version.len()>4096 {
            return Err(invalid("invalid official upload source"));
        }
        Ok(format!("{:x}",Sha256::digest(serde_json::to_vec(self)?)))
    }
}
impl Store {
    pub fn engine_upload_stage(&self, operation: &str) -> Result<Option<EngineUploadStage>> {
        let row:Option<(String,String,Option<String>,Option<String>,Option<String>)>=self.db.query_row(
            "SELECT source,state,attempt,receipt,error FROM engine_upload_stages WHERE operation=?1",
            [operation],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?))).optional()?;
        row.map(|(source,state,attempt,receipt,error)|Ok(EngineUploadStage {
            operation:operation.into(),source:serde_json::from_str(&source)?,state,attempt,receipt,error
        })).transpose()
    }
    /// Call after mark_upload_started and before the staged inode becomes visible
    /// to the official watcher. The caller owns safe staging/rename/fsync.
    pub fn stage_engine_upload(&mut self, operation: &str, source: &EngineUploadSource) -> Result<()> {
        let key=source.claim.key()?;
        if !hex(&source.sha256,64) || !hex(&source.md4,32) { return Err(Error::Integrity); }
        let mut job=self.upload_job(operation)?;
        let is_creation=job.node.version==crate::creations::CREATE_VERSION;
        if job.state!="inflight" || job.node.directory
            || source.claim.base_version!=job.node.version
            || source.claim.path!=format!("/{}",job.relative_path)
            || source.claim.size!=job.file.metadata()?.len()
            || (!is_creation && self.local_identity(&source.claim.remote_id)?!=job.node.id)
            || (is_creation && !source.claim.remote_id.is_empty()) {
            return Err(invalid("official staging does not match immutable operation"));
        }
        if let Some(old)=self.engine_upload_stage(operation)? {
            return if old.source==*source {Ok(())} else {Err(invalid("official stage already bound"))};
        }
        // Verify against the immutable operation, not whatever happens to be
        // present at its current user-visible pathname.
        let mut sha=Sha256::new();let mut md4=md4::Md4::new();
        let mut buffer=[0u8;65536];let mut size=0u64;
        loop {
            let n=job.file.read(&mut buffer)?;
            if n==0 {break;}
            size=size.checked_add(n as u64).ok_or(Error::Integrity)?;
            sha.update(&buffer[..n]);md4.update(&buffer[..n]);
        }
        if size!=source.claim.size || format!("{:x}",sha.finalize())!=source.sha256
            || format!("{:x}",md4.finalize())!=source.md4 {return Err(Error::Integrity);}
        self.db.execute("INSERT INTO engine_upload_stages(operation,source_key,source,state) VALUES(?1,?2,?3,'staged')",
            params![operation,key,serde_json::to_string(source)?])?;
        Ok(())
    }
    /// Repeated IPC delivery of the same attempt is idempotent. Another attempt
    /// cannot silently retry a claimed operation after an uncertain submission.
    pub fn claim_engine_upload(&mut self, attempt: &str, claim: &EngineUploadClaim) -> Result<()> {
        if !hex(attempt,32) { return Err(invalid("invalid official upload attempt")); }
        let key=claim.key()?;
        let operation:String=self.db.query_row("SELECT operation FROM engine_upload_stages WHERE source_key=?1",[key],|r|r.get(0))?;
        let stage=self.engine_upload_stage(&operation)?.ok_or_else(||invalid("upload stage missing"))?;
        if stage.source.claim!=*claim { return Err(Error::Integrity); }
        if stage.attempt.as_deref()==Some(attempt) && stage.state=="claimed" { return Ok(()); }
        let tx=self.db.transaction()?;
        let n=tx.execute("UPDATE engine_upload_stages SET state='claimed',attempt=?2 WHERE operation=?1 AND state='staged' AND attempt IS NULL AND EXISTS(SELECT 1 FROM operations o WHERE o.id=?1 AND o.state='inflight')",params![operation,attempt])?;
        if n!=1 { return Err(invalid("upload already claimed or uncertain")); }
        tx.commit()?;
        Ok(())
    }
    pub fn engine_upload_for_attempt(&self, attempt: &str) -> Result<EngineUploadStage> {
        let operation:String=self.db.query_row("SELECT operation FROM engine_upload_stages WHERE attempt=?1",[attempt],|r|r.get(0))?;
        self.engine_upload_stage(&operation)?.ok_or_else(||invalid("upload stage missing"))
    }
    /// Persist the exact callback before acknowledging it. Application to the
    /// local operation is separate and restartable; this does not mark in-sync.
    pub fn record_engine_upload_receipt(&mut self, attempt: &str, receipt: &str) -> Result<()> {
        if receipt.is_empty() || receipt.len()>65536 { return Err(invalid("invalid upload receipt")); }
        let stage=self.engine_upload_for_attempt(attempt)?;
        if let Some(old)=stage.receipt {
            return if old==receipt {Ok(())} else {Err(Error::Integrity)};
        }
        if !matches!(stage.state.as_str(),"claimed"|"uncertain") { return Err(invalid("upload not admitted")); }
        self.db.execute("UPDATE engine_upload_stages SET state='receipted',receipt=?2,error=NULL WHERE operation=?1",
            params![stage.operation,receipt])?;
        Ok(())
    }
    pub fn uncertain_engine_upload(&mut self, attempt: &str) -> Result<()> {
        let stage=self.engine_upload_for_attempt(attempt)?;
        if stage.receipt.is_some() || stage.state=="applied" {return Ok(());}
        let tx=self.db.transaction()?;
        tx.execute("UPDATE operations SET state='uncertain' WHERE id=?1 AND state='inflight'",[&stage.operation])?;
        tx.execute("UPDATE engine_upload_stages SET state='uncertain',error='commit_not_observed' WHERE operation=?1",[&stage.operation])?;
        tx.commit()?;
        Ok(())
    }
    pub fn engine_upload_receipts(&self) -> Result<Vec<EngineUploadStage>> {
        let mut q=self.db.prepare("SELECT operation FROM engine_upload_stages WHERE state='receipted' ORDER BY rowid LIMIT 64")?;
        let ids=q.query_map([],|r|r.get::<_,String>(0))?.collect::<std::result::Result<Vec<_>,_>>()?;
        ids.into_iter().map(|id|self.engine_upload_stage(&id)?.ok_or_else(||invalid("upload stage missing"))).collect()
    }
    pub fn finish_engine_upload_receipt(&mut self, operation: &str) -> Result<()> {
        let n=self.db.execute("UPDATE engine_upload_stages SET state='applied',error=NULL WHERE operation=?1 AND state='receipted' AND EXISTS(SELECT 1 FROM operations o WHERE o.id=?1 AND o.state='committed')",[operation])?;
        if n!=1 {return Err(invalid("upload receipt not committed"));}
        Ok(())
    }
    pub fn fail_engine_upload_receipt(&mut self, operation: &str) -> Result<()> {
        // Preserve receipt and source for explicit conflict/uncertainty recovery.
        self.db.execute("UPDATE engine_upload_stages SET state='uncertain',error='receipt_reconciliation_failed' WHERE operation=?1 AND state='receipted'",[operation])?;
        Ok(())
    }
}
