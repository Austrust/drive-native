//! Mutation reconciliation uses current authoritative state, never blind replay.
//! Matching a desired state is deliberately distinct from proving which request
//! committed: another writer may have produced identical bytes.
use crate::{
    Client, Error, Result,
    actions::{
        list::{FileType, NodeInfo},
        native::{NativeVersion, validate_path},
        transfer::ContentVersion,
    },
};
use std::path::Path;

#[derive(Clone, Debug)]
pub struct UploadIntent {
    pub view_id: u64,
    pub path: String,
    /// Some for update, None only when creating a new local file.
    pub original_file_id: Option<String>,
    pub base: NativeVersion,
    pub size: u64,
    pub md4: String,
    pub sha256: String,
}
#[derive(Debug)]
pub enum UploadReconciliation {
    /// Exact desired bytes exist at this path and the original identity survived.
    DesiredStatePresent(ContentVersion),
    /// Current state differs. Retain the local journal and expose a conflict.
    Diverged(NodeInfo),
    /// Absence alone cannot prove an old in-flight request will not commit later.
    Missing,
    /// The object changed while verifying. Await events/reconcile again; no write.
    ChangedDuringVerification,
}
fn same_revision(a: &NodeInfo, b: &NodeInfo) -> bool {
    a.file_id == b.file_id
        && a.sync_id == b.sync_id
        && a.max_id == b.max_id
        && a.file_size == b.file_size
        && a.hash == b.hash
        && a.file_type == b.file_type
        && a.is_removed == b.is_removed
}
impl UploadIntent {
    fn validate(&self) -> Result<()> {
        validate_path(&self.path)?;
        if self.path == "/"
            || self.md4.len() != 32
            || self.sha256.len() != 64
            || !self.md4.bytes().all(|x| x.is_ascii_hexdigit())
            || !self.sha256.bytes().all(|x| x.is_ascii_hexdigit())
        {
            return Err(Error::InvalidConfig(
                "invalid journaled upload intent".into(),
            ));
        }
        if self.original_file_id.is_some() && self.base.sync_id == 0 {
            return Err(Error::InvalidConfig(
                "existing file needs a real base version".into(),
            ));
        }
        Ok(())
    }
}
impl Client {
    pub(crate) async fn current_path(&self, view: u64, path: &str) -> Result<Option<NodeInfo>> {
        validate_path(path)?;
        let (parent, name) = path
            .rsplit_once('/')
            .ok_or_else(|| Error::InvalidConfig("missing parent".into()))?;
        if name.is_empty() {
            return Err(Error::InvalidConfig("expected file leaf".into()));
        }
        let parent = if parent.is_empty() { "/" } else { parent };
        let mut found = self
            .list_dir(view, parent)
            .await?
            .into_iter()
            .filter(|n| n.name == name && !n.is_removed);
        let node = found.next();
        if found.next().is_some() {
            return Err(Error::Decode("ambiguous native path".into()));
        }
        Ok(node)
    }
    /// Read-only reconciliation of an upload whose reply was lost. Uses a bounded
    /// temporary file and a second metadata read to detect verification races.
    /// Missing/mismatch never causes an automatic replay of the original request.
    pub async fn reconcile_upload(
        &self,
        intent: &UploadIntent,
        staging: &Path,
    ) -> Result<UploadReconciliation> {
        intent.validate()?;
        let Some(before) = self.current_path(intent.view_id, &intent.path).await? else {
            return Ok(UploadReconciliation::Missing);
        };
        if before.file_type != FileType::File
            || before.file_size != intent.size
            || before.hash != intent.md4
            || intent
                .original_file_id
                .as_ref()
                .is_some_and(|id| id != &before.file_id)
        {
            return Ok(UploadReconciliation::Diverged(before));
        }
        let expected = ContentVersion {
            file_id: before.file_id.clone(),
            version: NativeVersion {
                sync_id: before.sync_id,
                max_id: before.max_id,
            },
            size: before.file_size,
            hash: before.hash.clone(),
        };
        let temp = tempfile::tempdir_in(staging)?;
        let verified = self
            .download_checked(&expected, &temp.path().join("verification"))
            .await?;
        if verified.sha256 != intent.sha256 {
            return Ok(UploadReconciliation::Diverged(before));
        }
        let after = self.current_path(intent.view_id, &intent.path).await?;
        if !after.as_ref().is_some_and(|n| same_revision(&before, n)) {
            return Ok(UploadReconciliation::ChangedDuringVerification);
        }
        Ok(UploadReconciliation::DesiredStatePresent(verified.remote))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn node() -> NodeInfo {
        NodeInfo {
            node_id: 2,
            sync_id: 7,
            max_id: 7,
            name: "a".into(),
            file_size: 3,
            mtime: 0,
            hash: "hash".into(),
            file_id: "19".into(),
            file_type: FileType::File,
            is_removed: false,
        }
    }
    #[test]
    fn content_equality_does_not_hide_identity_or_version_race() {
        let n = node();
        let mut changed = n.clone();
        changed.file_id = "20".into();
        assert!(!same_revision(&n, &changed));
        changed = n.clone();
        changed.max_id += 1;
        assert!(!same_revision(&n, &changed));
        changed = n.clone();
        changed.sync_id += 1;
        assert!(!same_revision(&n, &changed));
        changed = n.clone();
        changed.is_removed = true;
        assert!(!same_revision(&n, &changed));
    }
    #[test]
    fn malformed_intents_are_rejected_before_network() {
        let mut i = UploadIntent {
            view_id: 54,
            path: "/lab/a".into(),
            original_file_id: None,
            base: NativeVersion {
                sync_id: 0,
                max_id: 0,
            },
            size: 0,
            md4: "a".repeat(32),
            sha256: "b".repeat(64),
        };
        i.validate().unwrap();
        i.path = "/lab/../a".into();
        assert!(i.validate().is_err());
        i.path = "/lab/a".into();
        i.original_file_id = Some("1".into());
        assert!(i.validate().is_err());
    }
}
