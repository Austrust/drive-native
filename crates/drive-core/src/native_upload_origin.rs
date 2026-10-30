//! Match native event author and identity before adopting an uncertain new file.
use super::*;
use crate::{commit_origin::CommitOrigin, upload_origin::UploadOrigin};
use sproto::actions::{list::NodeInfo, native::NativeEvent};

#[derive(Debug, PartialEq, Eq)]
enum Ownership {
    Own,
    Foreign,
    Unknown,
}
fn ownership(origin: &UploadOrigin, current: &NodeInfo, events: &[NativeEvent]) -> Ownership {
    let Some(cursor) = origin
        .boundary
        .cursor
        .strip_prefix("native-v1:")
        .and_then(|s| s.parse::<u64>().ok())
    else {
        return Ownership::Unknown;
    };
    if current.file_type != FileType::File || current.is_removed {
        return Ownership::Foreign;
    }
    let matching: Vec<_> = events
        .iter()
        .filter(|e| {
            e.path == origin.path
                && e.file_id == current.file_id
                && e.parent_id == origin.parent_file
                && e.version.sync_id == current.sync_id
                && e.version.max_id == current.max_id
                && e.version.sync_id > cursor
                && e.file_type == FileType::File
                && !e.removed
                && e.size == current.file_size
                && e.hash == current.hash
                && matches!(e.kind, EventKind::Modify)
        })
        .collect();
    if matching.is_empty() {
        return Ownership::Unknown;
    }
    if matching
        .iter()
        .all(|e| e.committer.as_ref() == Some(&origin.boundary.committer))
    {
        return Ownership::Own;
    }
    if matching.iter().all(|e| {
        e.committer
            .as_ref()
            .is_some_and(|a| a != &origin.boundary.committer)
    }) {
        Ownership::Foreign
    } else {
        Ownership::Unknown
    }
}
impl NativeReadSession {
    /// No server mutation. The returned boundary must be durable before upload.
    pub async fn prepare_upload_creation(
        &self,
        node: &Node,
        parent: &Node,
        path: &str,
        sha256: &str,
        cursor: &str,
    ) -> Result<Option<UploadOrigin>> {
        if node.version != crate::creations::CREATE_VERSION
            || node.directory
            || node.parent.as_deref() != Some(parent.id.as_str())
            || !self.scope.contains(path)
            || path == self.scope.path
            || path.rsplit('/').next() != Some(node.name.as_str())
        {
            return Err(Error::State("invalid creation preparation".into()));
        }
        let token: VersionToken = serde_json::from_str(&parent.version)?;
        if token.schema != 1
            || token.restore != self.client.restore_identity()
            || token.view != self.scope.view_id
            || token.root != self.scope.file_id
            || !token.directory
            || !native_file_id(&token.file)
        {
            return Err(Error::State("creation parent scope mismatch".into()));
        }
        let (parent_path, name) = path
            .rsplit_once('/')
            .ok_or_else(|| Error::State("creation path invalid".into()))?;
        let parent_path = if parent_path.is_empty() {
            "/"
        } else {
            parent_path
        };
        if parent_path == self.scope.path {
            if token.file != self.scope.file_id {
                return Ok(None);
            }
        } else {
            let (gp, leaf) = parent_path
                .rsplit_once('/')
                .ok_or_else(|| Error::State("creation parent path invalid".into()))?;
            let parents = self
                .client
                .list_dir(token.view, if gp.is_empty() { "/" } else { gp })
                .await?;
            if !parents.iter().any(|n| {
                n.name == leaf
                    && n.file_id == token.file
                    && n.file_type == FileType::Dir
                    && !n.is_removed
            }) {
                return Ok(None);
            }
        }
        let (cursor, _) = self.directory_events(cursor).await?;
        if self
            .client
            .list_dir(token.view, parent_path)
            .await?
            .iter()
            .any(|n| n.name == name && !n.is_removed)
        {
            return Ok(None);
        }
        let origin = UploadOrigin {
            schema: 1,
            node: node.id.clone(),
            parent_file: token.file,
            path: path.into(),
            sha256: sha256.into(),
            boundary: CommitOrigin {
                cursor,
                committer: self.client.commit_fingerprint(),
                restore: self.client.restore_identity().into(),
            },
        };
        origin.validate(node, path, sha256)?;
        Ok(Some(origin))
    }
    pub async fn upload_with_origin(
        &self,
        node: &Node,
        path: &str,
        source: &Path,
        sha256: &str,
        _staging: &Path,
        origin: Option<&UploadOrigin>,
    ) -> Result<NativeWriteOutcome> {
        if node.version == crate::creations::CREATE_VERSION {
            let Some(origin) = origin else {
                return Ok(NativeWriteOutcome::Uncertain);
            };
            origin.validate(node, path, sha256)?;
            if origin.boundary.restore != self.client.restore_identity()
                || origin.boundary.committer != self.client.commit_fingerprint()
            {
                return Ok(NativeWriteOutcome::Uncertain);
            }
        }
        self.upload_prepared(node, path, source, sha256, origin)
            .await
    }
    pub async fn reconcile_with_origin(
        &self,
        node: &Node,
        path: &str,
        source: &Path,
        sha256: &str,
        staging: &Path,
        origin: Option<&UploadOrigin>,
    ) -> Result<NativeWriteOutcome> {
        if node.version != crate::creations::CREATE_VERSION {
            return self
                .reconcile_existing(node, path, source, sha256, staging)
                .await;
        }
        let Some(origin) = origin else {
            return Ok(NativeWriteOutcome::Uncertain);
        };
        origin.validate(node, path, sha256)?;
        if origin.boundary.restore != self.client.restore_identity() {
            return Ok(NativeWriteOutcome::Uncertain);
        }
        let mut intent = self.write_intent(node, path, source, sha256).await?;
        let (parent, name) = path
            .rsplit_once('/')
            .ok_or_else(|| Error::State("creation path invalid".into()))?;
        let nodes = self
            .client
            .list_dir(intent.view_id, if parent.is_empty() { "/" } else { parent })
            .await?;
        let mut matches = nodes.iter().filter(|n| n.name == name && !n.is_removed);
        let Some(current) = matches.next() else {
            return Ok(NativeWriteOutcome::Uncertain);
        };
        if matches.next().is_some() {
            return Ok(NativeWriteOutcome::Uncertain);
        }
        let (_, events) = self.directory_events(&origin.boundary.cursor).await?;
        if parent_removed(origin, &events)? {
            return Ok(NativeWriteOutcome::Conflict);
        }
        match ownership(origin, current, &events) {
            Ownership::Foreign => return Ok(NativeWriteOutcome::Conflict),
            Ownership::Unknown => return Ok(NativeWriteOutcome::Uncertain),
            Ownership::Own => {}
        }
        // Keep the identity/version witnessed in the author event. Readback also
        // checks complete SHA-256 and metadata again after downloading.
        intent.original_file_id = Some(current.file_id.clone());
        intent.base = NativeVersion {
            sync_id: current.sync_id,
            max_id: current.max_id,
        };
        use sproto::actions::mutations::UploadReconciliation as R;
        match self.client.reconcile_upload(&intent, staging).await? {
            R::DesiredStatePresent(content)
                if content.file_id == current.file_id && content.version == intent.base =>
            {
                self.confirmed_node(node, content)
            }
            R::Diverged(_) => Ok(NativeWriteOutcome::Conflict),
            _ => Ok(NativeWriteOutcome::Uncertain),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn restored_parent_metadata_does_not_erase_a_post_boundary_delete() {
        let mut origin = UploadOrigin {
            schema: 1,
            node: "local:a".into(),
            parent_file: "10".into(),
            path: "/lab/P/A".into(),
            sha256: "a".repeat(64),
            boundary: CommitOrigin {
                cursor: "native-v1:20".into(),
                committer: "b".repeat(64),
                restore: "s".into(),
            },
        };
        let mut event = NativeEvent {
            committer: None,
            kind: EventKind::Remove,
            path: "/lab/P".into(),
            file_id: "10".into(),
            parent_id: "1".into(),
            version: NativeVersion {
                sync_id: 21,
                max_id: 22,
            },
            file_type: FileType::Dir,
            size: 0,
            hash: String::new(),
            mtime: 0,
            removed: false,
        };
        assert!(parent_removed(&origin, &[event.clone()]).unwrap());
        event.version.sync_id = 20;
        assert!(!parent_removed(&origin, &[event.clone()]).unwrap());
        event.version.sync_id = 21;
        event.file_id = "other".into();
        assert!(!parent_removed(&origin, &[event.clone()]).unwrap());
        origin.boundary.cursor = "unknown".into();
        assert!(parent_removed(&origin, &[event]).is_err());
    }
    #[test]
    fn equal_bytes_do_not_prove_creation_ownership() {
        let origin = UploadOrigin {
            schema: 1,
            node: "local:test".into(),
            parent_file: "1".into(),
            path: "/lab/A".into(),
            sha256: "a".repeat(64),
            boundary: CommitOrigin {
                cursor: "native-v1:10".into(),
                committer: "b".repeat(64),
                restore: "r".into(),
            },
        };
        let n = NodeInfo {
            node_id: 1,
            sync_id: 11,
            max_id: 12,
            name: "A".into(),
            file_size: 3,
            mtime: 0,
            hash: "c".repeat(32),
            file_id: "2".into(),
            file_type: FileType::File,
            is_removed: false,
        };
        let e = NativeEvent {
            committer: Some(origin.boundary.committer.clone()),
            kind: EventKind::Modify,
            path: origin.path.clone(),
            file_id: n.file_id.clone(),
            parent_id: origin.parent_file.clone(),
            version: NativeVersion {
                sync_id: 11,
                max_id: 12,
            },
            file_type: FileType::File,
            size: 3,
            hash: n.hash.clone(),
            mtime: 0,
            removed: false,
        };
        assert_eq!(ownership(&origin, &n, &[e.clone()]), Ownership::Own);
        let mut other = e.clone();
        other.committer = Some("d".repeat(64));
        assert_eq!(ownership(&origin, &n, &[other.clone()]), Ownership::Foreign);
        assert_eq!(
            ownership(&origin, &n, &[other, e.clone()]),
            Ownership::Unknown
        );
        let mut anonymous = e.clone();
        anonymous.committer = None;
        assert_eq!(ownership(&origin, &n, &[anonymous]), Ownership::Unknown);
        for mut bad in [e.clone(), e.clone(), e.clone(), e.clone(), e.clone()]
            .into_iter()
            .enumerate()
        {
            match bad.0 {
                0 => bad.1.parent_id = "3".into(),
                1 => bad.1.version.max_id = 13,
                2 => bad.1.hash = "e".repeat(32),
                3 => bad.1.file_id = "4".into(),
                _ => bad.1.removed = true,
            };
            assert_eq!(ownership(&origin, &n, &[bad.1]), Ownership::Unknown);
        }
        let mut old = origin;
        old.boundary.cursor = "native-v1:11".into();
        assert_eq!(ownership(&old, &n, &[e]), Ownership::Unknown);
    }
}

/// Path-based and identity-based uploads may restore a deleted empty parent.
/// A receipt/unchanged parent identity cannot erase the intervening delete.
pub(super) fn parent_removed(origin: &UploadOrigin, events: &[NativeEvent]) -> Result<bool> {
    let cursor = origin
        .boundary
        .cursor
        .strip_prefix("native-v1:")
        .and_then(|s| s.parse::<u64>().ok())
        .ok_or_else(|| Error::State("invalid creation event boundary".into()))?;
    Ok(events.iter().any(|e| {
        e.file_id == origin.parent_file
            && e.version.sync_id > cursor
            && matches!(e.kind, EventKind::Remove)
    }))
}
