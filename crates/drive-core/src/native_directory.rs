//! Directory commit recovery requires a persisted author fingerprint and event boundary.
use super::*;
use crate::directories::{DirectoryJob, DirectoryOrigin};
use sproto::actions::{list::NodeInfo, native::NativeEvent};

#[derive(PartialEq, Eq)]
enum ReceiptFault {
    None,
    BeforeRead,
    AfterRead,
}

#[derive(Debug, PartialEq, Eq)]
enum Ownership {
    Own,
    Foreign,
    Unknown,
}
fn ownership(
    origin: &DirectoryOrigin,
    parent: &str,
    path: &str,
    current: &NodeInfo,
    events: &[NativeEvent],
) -> Ownership {
    let Some(cursor) = origin
        .cursor
        .strip_prefix("native-v1:")
        .and_then(|s| s.parse::<u64>().ok())
    else {
        return Ownership::Unknown;
    };
    if current.file_type != FileType::Dir || current.is_removed {
        return Ownership::Foreign;
    }
    let matches: Vec<_> = events
        .iter()
        .filter(|e| {
            e.path == path
                && e.file_id == current.file_id
                && e.parent_id == parent
                && e.version.sync_id == current.sync_id
                && e.version.sync_id > cursor
                && e.file_type == FileType::Dir
                && !e.removed
                && matches!(e.kind, EventKind::Modify)
        })
        .collect();
    if matches.is_empty() {
        return Ownership::Unknown;
    }
    let own = matches
        .iter()
        .all(|e| e.committer.as_deref() == Some(origin.committer.as_str()));
    if own {
        return Ownership::Own;
    }
    if matches
        .iter()
        .all(|e| e.committer.as_ref().is_some_and(|v| v != &origin.committer))
    {
        Ownership::Foreign
    } else {
        Ownership::Unknown
    }
}
impl NativeReadSession {
    pub(super) async fn directory_events(
        &self,
        cursor: &str,
    ) -> Result<(String, Vec<NativeEvent>)> {
        let mut watch = self.watch(cursor)?;
        let mut events = Vec::new();
        for _ in 0..32 {
            let batch = self.scoped_events(&watch).await?;
            if batch.recovery_required {
                return Err(Error::Unsupported(
                    "directory recovery needs a valid event history",
                ));
            }
            if events.len() + batch.events.len() > 65536 {
                return Err(Error::State(
                    "directory verification event budget exceeded".into(),
                ));
            }
            let next = batch.next;
            events.extend(batch.events);
            if next == watch.cursor {
                return Ok((format!("native-v1:{}", next.0), events));
            }
            watch.cursor = next;
        }
        Err(Error::State(
            "directory verification did not reach an event boundary".into(),
        ))
    }
    async fn directory_parent(
        &self,
        job: &DirectoryJob,
    ) -> Result<Option<(VersionToken, String, String)>> {
        if !job.node.directory
            || job.node.version != crate::creations::CREATE_VERSION
            || job.node.parent.as_deref() != Some(&job.parent.id)
            || job.path.rsplit('/').next() != Some(&job.node.name)
        {
            return Err(Error::State("invalid directory intent".into()));
        }
        let path = self.namespace_path(&job.path)?;
        let parent: VersionToken = serde_json::from_str(&job.parent.version)?;
        if parent.schema != 1
            || parent.restore != self.client.restore_identity()
            || parent.view != self.scope.view_id
            || parent.root != self.scope.file_id
            || !parent.directory
            || !native_file_id(&parent.file)
        {
            return Err(Error::State("directory parent scope mismatch".into()));
        }
        let (parent_path, _) = path.rsplit_once('/').unwrap();
        if parent_path == self.scope.path.trim_end_matches('/') {
            if parent.file != self.scope.file_id {
                return Ok(None);
            }
        } else {
            let (gp, name) = parent_path
                .rsplit_once('/')
                .ok_or_else(|| Error::State("invalid directory parent path".into()))?;
            let listed = self
                .client
                .list_dir(parent.view, if gp.is_empty() { "/" } else { gp })
                .await?;
            if !listed.iter().any(|n| {
                n.name == name
                    && n.file_id == parent.file
                    && n.file_type == FileType::Dir
                    && !n.is_removed
            }) {
                return Ok(None);
            }
        }
        let parent_path = if parent_path.is_empty() {
            "/".to_owned()
        } else {
            parent_path.to_owned()
        };
        Ok(Some((parent, path, parent_path)))
    }
    async fn recover_directory(
        &self,
        job: &DirectoryJob,
        parent: &VersionToken,
        path: &str,
        parent_path: &str,
    ) -> Result<NativeDirectoryOutcome> {
        let Some(origin) = &job.origin else {
            return Ok(NativeDirectoryOutcome::Uncertain);
        };
        origin.validate()?;
        if origin.restore != self.client.restore_identity() {
            return Ok(NativeDirectoryOutcome::Uncertain);
        }
        let listed = self.client.list_dir(parent.view, parent_path).await?;
        let Some(before) = listed
            .iter()
            .find(|n| n.name == job.node.name && !n.is_removed)
        else {
            return Ok(NativeDirectoryOutcome::Uncertain);
        };
        if before.file_type != FileType::Dir {
            return Ok(NativeDirectoryOutcome::Conflict);
        }
        let (_, events) = self.directory_events(&origin.cursor).await?;
        let boundary = self.watch(&origin.cursor)?.cursor.0;
        if events.iter().any(|e| {
            e.file_id == parent.file && e.kind == EventKind::Remove && e.version.sync_id > boundary
        }) {
            return Ok(NativeDirectoryOutcome::Conflict);
        }
        if self.directory_parent(job).await?.is_none() {
            return Ok(NativeDirectoryOutcome::Conflict);
        }
        let listed = self.client.list_dir(parent.view, parent_path).await?;
        let Some(current) = listed
            .iter()
            .find(|n| n.name == job.node.name && !n.is_removed)
        else {
            return Ok(NativeDirectoryOutcome::Uncertain);
        };
        if before.file_id != current.file_id
            || before.sync_id != current.sync_id
            || before.max_id != current.max_id
            || before.file_type != current.file_type
        {
            return Ok(NativeDirectoryOutcome::Uncertain);
        }
        match ownership(origin, &parent.file, path, current, &events) {
            Ownership::Foreign => return Ok(NativeDirectoryOutcome::Conflict),
            Ownership::Unknown => return Ok(NativeDirectoryOutcome::Uncertain),
            Ownership::Own => {}
        }
        let version = VersionToken {
            schema: 1,
            restore: self.client.restore_identity().into(),
            view: parent.view,
            root: self.scope.file_id.clone(),
            file: current.file_id.clone(),
            sync: current.sync_id,
            max: current.max_id,
            size: 0,
            hash: current.hash.clone(),
            directory: true,
        };
        let mut node = job.node.clone();
        node.version = serde_json::to_string(&version)?;
        node.metadata
            .as_mut()
            .ok_or_else(|| Error::State("directory attributes absent".into()))?
            .mtime = current.mtime;
        Ok(NativeDirectoryOutcome::Created {
            node,
            remote: current.file_id.clone(),
        })
    }
    pub async fn create_directory(
        &self,
        job: &DirectoryJob,
        reconcile: bool,
    ) -> Result<NativeDirectoryOutcome> {
        self.create_directory_inner(job, reconcile, ReceiptFault::None)
            .await
    }
    /// Explicit test injection: send once, poison the channel before reading any response.
    pub async fn research_create_directory_discard_reply(
        &self,
        job: &DirectoryJob,
    ) -> Result<NativeDirectoryOutcome> {
        self.create_directory_inner(job, false, ReceiptFault::BeforeRead)
            .await
    }
    /// Fault at worker/fsd boundary: a received server reply never reaches durable state.
    pub async fn research_create_directory_discard_ack(
        &self,
        job: &DirectoryJob,
    ) -> Result<NativeDirectoryOutcome> {
        self.create_directory_inner(job, false, ReceiptFault::AfterRead)
            .await
    }
    async fn create_directory_inner(
        &self,
        job: &DirectoryJob,
        reconcile: bool,
        discard: ReceiptFault,
    ) -> Result<NativeDirectoryOutcome> {
        if job.creation_contract > 1 {
            return Err(Error::State("unsupported directory contract".into()));
        }
        if reconcile && job.creation_contract == 0 {
            return Ok(NativeDirectoryOutcome::Uncertain);
        }
        let Some((parent, path, parent_path)) = self.directory_parent(job).await? else {
            return Ok(NativeDirectoryOutcome::Conflict);
        };
        if reconcile && job.origin.is_some() {
            return self
                .recover_directory(job, &parent, &path, &parent_path)
                .await;
        }
        let author = self.client.commit_fingerprint();
        if job.origin.as_ref().is_none_or(|o| o.committer != author) {
            // No mutation occurs in this branch. The fsd must commit this result before retrying.
            let (cursor, _) = self.directory_events(&job.cursor).await?;
            let listed = self.client.list_dir(parent.view, &parent_path).await?;
            if listed
                .iter()
                .any(|n| n.name == job.node.name && !n.is_removed)
            {
                return Ok(NativeDirectoryOutcome::Conflict);
            }
            return Ok(NativeDirectoryOutcome::Prepared(DirectoryOrigin {
                cursor,
                committer: author,
                restore: self.client.restore_identity().into(),
            }));
        }
        let origin = job.origin.as_ref().unwrap();
        origin.validate()?;
        if origin.restore != self.client.restore_identity() {
            return Ok(NativeDirectoryOutcome::Uncertain);
        }
        let listed = self.client.list_dir(parent.view, &parent_path).await?;
        if listed
            .iter()
            .any(|n| n.name == job.node.name && !n.is_removed)
        {
            return Ok(NativeDirectoryOutcome::Conflict);
        }
        self.verify_root().await?;
        if discard == ReceiptFault::BeforeRead {
            self.client
                .research_mkdir_identity_discard_reply(&parent.file, &job.node.name)
                .await?;
            return Ok(NativeDirectoryOutcome::Uncertain);
        }
        let receipt = self
            .client
            .mkdir_at_identity(&parent.file, &job.node.name, &path)
            .await?;
        if discard == ReceiptFault::AfterRead {
            return Err(sproto::Error::Timeout(
                "injected lost worker/coordinator mkdir acknowledgement",
            )
            .into());
        }
        let result = self
            .recover_directory(job, &parent, &path, &parent_path)
            .await?;
        if let NativeDirectoryOutcome::Created { remote, .. } = &result {
            if remote != &receipt.file_id {
                return Ok(NativeDirectoryOutcome::Uncertain);
            }
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> (DirectoryOrigin, NodeInfo, NativeEvent) {
        let origin = DirectoryOrigin {
            cursor: "native-v1:10".into(),
            committer: "a".repeat(64),
            restore: "r".into(),
        };
        let node = NodeInfo {
            node_id: 3,
            sync_id: 11,
            max_id: 14,
            name: "D".into(),
            file_size: 0,
            mtime: 1,
            hash: "".into(),
            file_id: "22".into(),
            file_type: FileType::Dir,
            is_removed: false,
        };
        let event = NativeEvent {
            committer: Some(origin.committer.clone()),
            kind: EventKind::Modify,
            path: "/lab/D".into(),
            file_id: "22".into(),
            parent_id: "1".into(),
            version: NativeVersion {
                sync_id: 11,
                max_id: 14,
            },
            file_type: FileType::Dir,
            size: 0,
            hash: "".into(),
            mtime: 1,
            removed: false,
        };
        (origin, node, event)
    }
    #[test]
    fn directory_recovery_requires_author_identity_parent_and_newer_event() {
        let (origin, node, event) = fixture();
        assert_eq!(
            ownership(&origin, "1", "/lab/D", &node, &[event.clone()]),
            Ownership::Own
        );
        let mut anonymous = event.clone();
        anonymous.committer = None;
        assert_eq!(
            ownership(&origin, "1", "/lab/D", &node, &[anonymous]),
            Ownership::Unknown
        );
        let mut old = origin.clone();
        old.cursor = "native-v1:11".into();
        assert_eq!(
            ownership(&old, "1", "/lab/D", &node, &[event.clone()]),
            Ownership::Unknown
        );
        assert_eq!(
            ownership(&origin, "2", "/lab/D", &node, &[event.clone()]),
            Ownership::Unknown
        );
        let mut replaced = node.clone();
        replaced.file_id = "23".into();
        assert_eq!(
            ownership(&origin, "1", "/lab/D", &replaced, &[event.clone()]),
            Ownership::Unknown
        );
        let mut changed = node.clone();
        changed.sync_id = 12;
        assert_eq!(
            ownership(&origin, "1", "/lab/D", &changed, &[event]),
            Ownership::Unknown
        );
    }
    #[test]
    fn foreign_or_ambiguous_creation_is_never_adopted() {
        let (origin, node, event) = fixture();
        let mut foreign = event.clone();
        foreign.committer = Some("b".repeat(64));
        assert_eq!(
            ownership(&origin, "1", "/lab/D", &node, &[foreign.clone()]),
            Ownership::Foreign
        );
        assert_eq!(
            ownership(&origin, "1", "/lab/D", &node, &[event.clone(), foreign]),
            Ownership::Unknown
        );
        let mut moved = event;
        moved.kind = EventKind::Move {
            destination: "/lab/D".into(),
            parent_id: "1".into(),
        };
        assert_eq!(
            ownership(&origin, "1", "/lab/D", &node, &[moved]),
            Ownership::Unknown
        );
    }
}
