use super::*;
use crate::namespace::RenameJob;
use sproto::actions::{
    list::NodeInfo,
    namespace::{NamespaceTarget, RenameReconciliation},
};
pub enum NativeRenameOutcome {
    Refresh { node: Node },
    Confirmed { node: Node, same_content: bool },
    Conflict,
    Uncertain,
}
impl NativeReadSession {
    pub(super) fn namespace_path(&self, relative: &str) -> Result<String> {
        if relative.is_empty()
            || relative.contains('\0')
            || relative.split('/').any(|s| matches!(s, "" | "." | ".."))
        {
            return Err(Error::State("invalid relative namespace path".into()));
        }
        Ok(format!(
            "{}/{}",
            self.scope.path.trim_end_matches('/'),
            relative
        ))
    }
    fn namespace_target(&self, job: &RenameJob) -> Result<(NamespaceTarget, String, VersionToken)> {
        let token: VersionToken = serde_json::from_str(&job.node.version)?;
        if token.schema != 1
            || token.restore != self.client.restore_identity()
            || token.view != self.scope.view_id
            || token.root != self.scope.file_id
            || token.directory != job.node.directory
            || !native_file_id(&token.file)
        {
            return Err(Error::State("rename identity/scope mismatch".into()));
        }
        if job.source.rsplit('/').next() != Some(job.node.name.as_str())
            || job.destination.rsplit('/').next() != Some(job.name.as_str())
        {
            return Err(Error::State(
                "namespace job leaf does not match path".into(),
            ));
        }
        let source = self.namespace_path(&job.source)?;
        let destination = self.namespace_path(&job.destination)?;
        Ok((
            NamespaceTarget {
                view_id: token.view,
                path: source,
                file_id: token.file.clone(),
                base: NativeVersion {
                    sync_id: token.sync,
                    max_id: token.max,
                },
                file_type: if token.directory {
                    FileType::Dir
                } else {
                    FileType::File
                },
            },
            destination,
            token,
        ))
    }
    fn renamed_node(
        &self,
        job: &RenameJob,
        mut token: VersionToken,
        receipt: NodeInfo,
    ) -> Result<NativeRenameOutcome> {
        let same_content = token.hash == receipt.hash && token.size == receipt.file_size;
        token.sync = receipt.sync_id;
        token.max = receipt.max_id;
        token.size = receipt.file_size;
        token.hash = receipt.hash;
        let mut node = job.node.clone();
        node.parent = Some(job.parent.clone());
        node.name = job.name.clone();
        node.version = serde_json::to_string(&token)?;
        node.metadata = Some(crate::NodeMetadata {
            size: receipt.file_size,
            mtime: receipt.mtime,
            mode: node
                .metadata
                .as_ref()
                .map_or(if node.directory { 0o755 } else { 0o644 }, |m| m.mode),
        });
        Ok(NativeRenameOutcome::Confirmed { node, same_content })
    }
    pub async fn rename(&self, job: &RenameJob, reconcile: bool) -> Result<NativeRenameOutcome> {
        let (target, destination, token) = self.namespace_target(job)?;
        let parent_token: VersionToken = serde_json::from_str(&job.parent_node.version)?;
        if parent_token.schema != 1
            || (parent_token.file != job.parent
                && !crate::creations::is_local_identity(&job.parent))
            || parent_token.restore != self.client.restore_identity()
            || parent_token.view != self.scope.view_id
            || parent_token.root != self.scope.file_id
            || !parent_token.directory
            || job.parent_node.id != job.parent
        {
            return Err(Error::State("rename parent identity/scope mismatch".into()));
        }
        let parent_path = destination.rsplit_once('/').unwrap().0;
        if parent_path == self.scope.path.trim_end_matches('/') {
            if parent_token.file != self.scope.file_id {
                return Ok(NativeRenameOutcome::Conflict);
            }
        } else {
            let (grandparent, name) = parent_path
                .rsplit_once('/')
                .ok_or_else(|| Error::State("invalid destination parent path".into()))?;
            let parents = self
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
            if !parents.iter().any(|n| {
                n.name == name
                    && n.file_id == parent_token.file
                    && n.file_type == FileType::Dir
                    && !n.is_removed
            }) {
                return Ok(NativeRenameOutcome::Conflict);
            }
        }

        if reconcile {
            return match self.client.reconcile_rename(&target, &destination).await? {
                RenameReconciliation::DesiredStateObserved(node) => {
                    self.renamed_node(job, token, node)
                }
                RenameReconciliation::Diverged {
                    source,
                    destination,
                } if destination.is_some()
                    || source.as_ref().is_some_and(|n| {
                        n.file_id != target.file_id
                            || n.sync_id != target.base.sync_id
                            || n.max_id != target.base.max_id
                    }) =>
                {
                    Ok(NativeRenameOutcome::Conflict)
                }
                _ => Ok(NativeRenameOutcome::Uncertain),
            };
        }
        let (parent, name) = target
            .path
            .rsplit_once('/')
            .ok_or_else(|| Error::State("invalid rename source".into()))?;
        let listed = self
            .client
            .list_dir(target.view_id, if parent.is_empty() { "/" } else { parent })
            .await?;
        let mut found = listed.iter().filter(|n| n.name == name && !n.is_removed);
        let (Some(observed), None) = (found.next(), found.next()) else {
            return Ok(NativeRenameOutcome::Conflict);
        };
        if observed.file_id != target.file_id || observed.sync_id != target.base.sync_id {
            return Ok(NativeRenameOutcome::Conflict);
        }
        if observed.max_id != target.base.max_id {
            if job.node.directory
                && observed.file_type == FileType::Dir
                && observed.max_id > target.base.max_id
            {
                let mut refreshed = job.node.clone();
                let mut token: VersionToken = serde_json::from_str(&job.node.version)?;
                token.max = observed.max_id;
                refreshed.version = serde_json::to_string(&token)?;
                // No mutation has been sent. fsd must durably accept this
                // observation before a new conditional request is allowed.
                return Ok(NativeRenameOutcome::Refresh { node: refreshed });
            }
            return Ok(NativeRenameOutcome::Conflict);
        }
        self.verify_root().await?;
        match self
            .client
            .rename_native_checked_parent(&target, &destination, &parent_token.file)
            .await
        {
            Ok(node) => self.renamed_node(job, token, node),
            Err(sproto::Error::Server {
                code: 40962 | 36872,
                ..
            }) => Ok(NativeRenameOutcome::Conflict),
            _ => Ok(NativeRenameOutcome::Uncertain),
        }
    }
}

pub enum NativeRemoveOutcome {
    Confirmed { identity: String },
    Conflict,
    Uncertain,
}
impl NativeReadSession {
    fn remove_target(&self, job: &crate::removals::RemoveJob) -> Result<NamespaceTarget> {
        let token: VersionToken = serde_json::from_str(&job.node.version)?;
        if token.schema != 1
            || token.restore != self.client.restore_identity()
            || token.view != self.scope.view_id
            || token.root != self.scope.file_id
            || token.file == self.scope.file_id
            || token.directory != job.node.directory
            || !native_file_id(&token.file)
            || job.path.rsplit('/').next() != Some(job.node.name.as_str())
        {
            return Err(Error::State("remove identity/scope/path mismatch".into()));
        }
        Ok(NamespaceTarget {
            view_id: token.view,
            path: self.namespace_path(&job.path)?,
            file_id: token.file,
            base: NativeVersion {
                sync_id: token.sync,
                max_id: token.max,
            },
            file_type: if token.directory {
                FileType::Dir
            } else {
                FileType::File
            },
        })
    }
    async fn reconcile_remove_job(
        &self,
        job: &crate::removals::RemoveJob,
        target: &NamespaceTarget,
    ) -> Result<NativeRemoveOutcome> {
        use sproto::actions::namespace::RemoveReconciliation;
        Ok(
            match self
                .client
                .reconcile_remove(target, &self.watch(&job.cursor)?)
                .await?
            {
                RemoveReconciliation::TombstoneObserved { .. } => NativeRemoveOutcome::Confirmed {
                    identity: job.node.id.clone(),
                },
                RemoveReconciliation::StillPresent(n)
                    if n.file_id != target.file_id
                        || n.sync_id != target.base.sync_id
                        || (!job.node.directory && n.max_id != target.base.max_id) =>
                {
                    NativeRemoveOutcome::Conflict
                }
                _ => NativeRemoveOutcome::Uncertain,
            },
        )
    }
    pub async fn remove(
        &self,
        job: &crate::removals::RemoveJob,
        reconcile: bool,
    ) -> Result<NativeRemoveOutcome> {
        let mut target = self.remove_target(job)?;
        // Validate the saved watch before any mutation is allowed onto the wire.
        self.watch(&job.cursor)?;
        if reconcile {
            return self.reconcile_remove_job(job, &target).await;
        }
        let (parent, name) = target
            .path
            .rsplit_once('/')
            .ok_or_else(|| Error::State("remove parent absent".into()))?;
        let entries = self
            .client
            .list_dir(target.view_id, if parent.is_empty() { "/" } else { parent })
            .await?;
        let mut found = entries.iter().filter(|n| n.name == name && !n.is_removed);
        let current = match (found.next(), found.next()) {
            (Some(n), None) if n.file_id == target.file_id && n.file_type == target.file_type => n,
            (None, None) => return self.reconcile_remove_job(job, &target).await,
            _ => return Ok(NativeRemoveOutcome::Conflict),
        };
        if job.node.directory {
            // Child operations legitimately advance max_id. Capture fresh base
            // BEFORE the empty-directory check; a racing addition invalidates it.
            if current.sync_id != target.base.sync_id {
                return Ok(NativeRemoveOutcome::Conflict);
            }
            target.base.max_id = current.max_id;
            if self
                .client
                .list_dir(target.view_id, &target.path)
                .await?
                .iter()
                .any(|n| !n.is_removed)
            {
                return Ok(NativeRemoveOutcome::Conflict);
            }
        } else if current.sync_id != target.base.sync_id || current.max_id != target.base.max_id {
            return Ok(NativeRemoveOutcome::Conflict);
        }
        self.verify_root().await?;
        match self.client.remove_at_identity(&target).await {
            Ok(_) => self.reconcile_remove_job(job, &target).await,
            Err(sproto::Error::Server { code: 40962, .. }) => Ok(NativeRemoveOutcome::Conflict),
            _ => Ok(NativeRemoveOutcome::Uncertain),
        }
    }
}

impl NativeReadSession {
    pub async fn replace_target(
        &self,
        job: &crate::replacements::ReplaceJob,
        source: &Path,
        sha256: &str,
        staging: &Path,
        reconcile: bool,
    ) -> Result<NativeWriteOutcome> {
        if job.source.directory
            || job.target.directory
            || job.source.id == job.target.id
            || job.source_path.rsplit('/').next() != Some(job.source.name.as_str())
            || job.destination.rsplit('/').next() != Some(job.target.name.as_str())
        {
            return Err(Error::State("invalid replacement identity/path".into()));
        }
        let destination = self.namespace_path(&job.destination)?;
        match &job.target_remote {
            Some(id) => {
                let t: VersionToken = serde_json::from_str(&job.target.version)?;
                if &t.file != id {
                    return Err(Error::State("replacement target binding mismatch".into()));
                }
            }
            None if job.target.version == crate::creations::CREATE_VERSION => {}
            _ => return Err(Error::State("replacement target binding absent".into())),
        }
        if reconcile {
            return self
                .reconcile_with_origin(
                    &job.target,
                    &destination,
                    source,
                    sha256,
                    staging,
                    job.creation_origin.as_ref(),
                )
                .await;
        }
        match &job.source_remote {
            Some(id) => {
                let t: VersionToken = serde_json::from_str(&job.source.version)?;
                if t.schema != 1
                    || t.restore != self.client.restore_identity()
                    || t.view != self.scope.view_id
                    || t.root != self.scope.file_id
                    || t.directory
                    || &t.file != id
                {
                    return Err(Error::State("replacement source binding mismatch".into()));
                }
                let path = self.namespace_path(&job.source_path)?;
                let (parent, name) = path.rsplit_once('/').unwrap();
                let listing = self
                    .client
                    .list_dir(t.view, if parent.is_empty() { "/" } else { parent })
                    .await?;
                let mut found = listing.iter().filter(|n| n.name == name && !n.is_removed);
                if !matches!((found.next(),found.next()),(Some(n),None) if &n.file_id==id && n.sync_id==t.sync && n.max_id==t.max && n.file_type==FileType::File)
                {
                    return Ok(NativeWriteOutcome::Conflict);
                }
            }
            None if job.source.version == crate::creations::CREATE_VERSION => {}
            _ => return Err(Error::State("replacement source binding absent".into())),
        }
        self.upload_with_origin(
            &job.target,
            &destination,
            source,
            sha256,
            staging,
            job.creation_origin.as_ref(),
        )
        .await
    }
}
