//! Persisted file-creation provenance; it is never inferred from matching bytes.
use crate::{Error, Node, Result, Store, commit_origin::CommitOrigin, creations::CREATE_VERSION};
use rusqlite::{OptionalExtension, params};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct UploadOrigin {
    pub schema: u8,
    pub node: String,
    pub parent_file: String,
    pub path: String,
    pub sha256: String,
    pub boundary: CommitOrigin,
}
impl UploadOrigin {
    pub fn validate(&self, node: &Node, path: &str, sha256: &str) -> Result<()> {
        self.boundary.validate()?;
        if self.schema != 1
            || node.directory
            || node.version != CREATE_VERSION
            || !crate::creations::is_local_identity(&node.id)
            || self.node != node.id
            || self.path != path
            || self.sha256 != sha256
            || self.sha256.len() != 64
            || !self.sha256.bytes().all(|c| c.is_ascii_hexdigit())
            || self.parent_file.is_empty()
            || self.parent_file == "0"
        {
            return Err(Error::State("file creation provenance mismatch".into()));
        }
        Ok(())
    }
}
impl Store {
    pub fn upload_origin(&self, op: &str) -> Result<Option<UploadOrigin>> {
        let raw: Option<String> = self
            .db
            .query_row(
                "SELECT body FROM upload_origins WHERE operation=?1",
                [op],
                |r| r.get(0),
            )
            .optional()?;
        raw.map(|v| serde_json::from_str(&v).map_err(Error::from))
            .transpose()
    }
    /// Preparation is read-only remotely. Only a still-pending immutable operation
    /// can receive provenance; never retrofit it to a legacy uncertain write.
    pub fn prepare_upload_origin(
        &mut self,
        op: &str,
        origin: &UploadOrigin,
        sha256: &str,
    ) -> Result<()> {
        let job = self.upload_job(op)?;
        if job.state != "pending" {
            return Err(Error::State("creation already submitted".into()));
        }
        let path = format!(
            "{}/{}",
            self.scope_path()?.trim_end_matches('/'),
            job.relative_path
        );
        origin.validate(&job.node, &path, sha256)?;
        self.db.execute("INSERT INTO upload_origins(operation,body) VALUES(?1,?2) ON CONFLICT(operation) DO UPDATE SET body=excluded.body", params![op,serde_json::to_string(origin)?])?;
        Ok(())
    }
}
