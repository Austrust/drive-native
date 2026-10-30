//! Identity-addressed uploads. A native receipt alone never publishes sync state.
use super::*;
impl NativeReadSession {
    /// Legacy existing-file entrypoint. New creations need durable provenance.
    pub async fn upload_existing(
        &self,
        node: &Node,
        path: &str,
        source: &Path,
        sha256: &str,
        _staging: &Path,
    ) -> Result<NativeWriteOutcome> {
        self.upload_prepared(node, path, source, sha256, None).await
    }
    async fn upload_parent_matches(&self, path: &str, parent_file: &str) -> Result<bool> {
        if !native_file_id(parent_file) {
            return Err(Error::State("invalid upload parent identity".into()));
        }
        let (parent, _) = path
            .rsplit_once('/')
            .ok_or_else(|| Error::State("upload parent absent".into()))?;
        let parent = if parent.is_empty() { "/" } else { parent };
        if parent == self.scope.path {
            return Ok(parent_file == self.scope.file_id);
        }
        let (grandparent, name) = parent
            .rsplit_once('/')
            .ok_or_else(|| Error::State("upload parent outside scope".into()))?;
        let entries = self
            .client
            .list_dir(
                self.scope.view_id,
                if grandparent.is_empty() {
                    "/"
                } else {
                    grandparent
                },
            )
            .await?;
        let mut found = entries.iter().filter(|n| n.name == name && !n.is_removed);
        Ok(
            matches!((found.next(),found.next()),(Some(n),None) if n.file_id==parent_file && n.file_type==FileType::Dir),
        )
    }
    pub(super) async fn upload_prepared(
        &self,
        node: &Node,
        path: &str,
        source: &Path,
        sha256: &str,
        origin: Option<&crate::upload_origin::UploadOrigin>,
    ) -> Result<NativeWriteOutcome> {
        let intent = self.write_intent(node, path, source, sha256).await?;
        let (parent, name) = path
            .rsplit_once('/')
            .ok_or_else(|| Error::State("write path invalid".into()))?;
        let parent = if parent.is_empty() { "/" } else { parent };
        let creating = intent.original_file_id.is_none();
        let address = if let Some(original) = intent.original_file_id.as_deref() {
            (original, None)
        } else {
            let Some(origin) = origin else {
                return Ok(NativeWriteOutcome::Uncertain);
            };
            origin.validate(node, path, sha256)?;
            let id = origin.parent_file.as_str();
            if !self.upload_parent_matches(path, id).await? {
                return Ok(NativeWriteOutcome::Conflict);
            }
            (id, Some(name))
        };
        let nodes = self.client.list_dir(intent.view_id, parent).await?;
        let mut found = nodes.iter().filter(|n| n.name == name && !n.is_removed);
        if let Some(original) = intent.original_file_id.as_deref() {
            let Some(current) = found.next() else {
                return Ok(NativeWriteOutcome::Conflict);
            };
            if found.next().is_some()
                || current.file_id != original
                || current.sync_id != intent.base.sync_id
                || current.max_id != intent.base.max_id
            {
                return Ok(NativeWriteOutcome::Conflict);
            }
        } else if found.next().is_some() {
            return Ok(NativeWriteOutcome::Conflict);
        }
        self.verify_root().await?;
        let receipt = match self
            .client
            .upload_snapshot_at_identity(address.0, address.1, intent.base, source)
            .await
        {
            Ok(receipt)
                if intent
                    .original_file_id
                    .as_ref()
                    .is_none_or(|id| id == &receipt.file_id) =>
            {
                receipt
            }
            Err(sproto::Error::Server { code: 24579, .. }) => {
                return Ok(NativeWriteOutcome::Conflict);
            }
            _ => return Ok(NativeWriteOutcome::Uncertain),
        };
        // A parent can be moved/deleted after preflight. Identity paths avoid
        // writing its replacement, but deleted-parent uploads may still be
        // accepted by the NAS. Never equate that receipt with a live synced file.
        self.verify_root().await?;
        if creating {
            let origin = origin.unwrap();
            let (_, events) = self.directory_events(&origin.boundary.cursor).await?;
            if upload_origin::parent_removed(origin, &events)? {
                return Ok(NativeWriteOutcome::Conflict);
            }
        }
        if creating && !self.upload_parent_matches(path, address.0).await? {
            return Ok(NativeWriteOutcome::Uncertain);
        }
        let after = self.client.list_dir(intent.view_id, parent).await?;
        let mut found = after.iter().filter(|n| n.name == name && !n.is_removed);
        match (found.next(), found.next()) {
            (Some(n), None)
                if n.file_id == receipt.file_id
                    && n.sync_id == receipt.version.sync_id
                    && n.max_id == receipt.version.max_id
                    && n.file_size == receipt.size
                    && n.hash == receipt.hash =>
            {
                self.confirmed_node(node, receipt)
            }
            (Some(n), None)
                if n.file_id != receipt.file_id
                    || n.sync_id != receipt.version.sync_id
                    || n.max_id != receipt.version.max_id =>
            {
                Ok(NativeWriteOutcome::Conflict)
            }
            _ => Ok(NativeWriteOutcome::Uncertain),
        }
    }
}
