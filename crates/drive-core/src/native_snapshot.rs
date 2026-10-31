//! Native snapshot enumeration fenced by the same incremental event stream.
//! Used only for initial/recovery indexing, never as a periodic change detector.
use super::*;
use crate::snapshot_progress::{Phase, Progress, emit};
use sproto::actions::list::{FileType, NodeInfo};
use std::collections::{HashMap, HashSet, VecDeque};

pub const SNAPSHOT_NODE_LIMIT: usize = 500_000;
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RemoteSnapshot {
    pub scope: Scope,
    pub restore: String,
    pub cursor: String,
    pub nodes: Vec<Node>,
    #[serde(default)]
    pub inaccessible: Vec<String>,
    #[serde(default)]
    pub skipped_symlinks: u64,
}
impl NativeReadSession {
    /// Collect a candidate, not yet authorized for installation. An independent
    /// fence check is mandatory even when every directory list succeeded.
    pub async fn snapshot_candidate(&self) -> Result<RemoteSnapshot> {
        self.snapshot_candidate_with_observer(|_| async { Ok(()) }, 1)
            .await
    }
    /// Explicit research barrier after a directory has been enumerated. The
    /// production path uses a no-op; no environment or network hook is exposed.
    #[cfg(feature = "native-research")]
    pub async fn research_snapshot_candidate<F, Fut>(&self, observer: F) -> Result<RemoteSnapshot>
    where
        F: FnMut(String) -> Fut,
        Fut: std::future::Future<Output = Result<()>>,
    {
        self.snapshot_candidate_with_observer(observer, 1).await
    }
    async fn snapshot_candidate_with_observer<F, Fut>(
        &self,
        mut observer: F,
        attempt: u8,
    ) -> Result<RemoteSnapshot>
    where
        F: FnMut(String) -> Fut,
        Fut: std::future::Future<Output = Result<()>>,
    {
        let mut watch = self.watch("")?;
        let mut root = None;
        let mut event_count = 0usize;
        let mut boundary = false;
        // Scoped history can have empty pages with advancing global cursors.
        // Only a stationary, empty response is the verified boundary.
        let started = std::time::Instant::now();
        emit(Progress {
            attempt,
            phase: Phase::Seed,
            units: 0,
            items: 0,
        });
        for page in 0..4096 {
            if started.elapsed() > std::time::Duration::from_secs(900) {
                return Err(Error::State("snapshot seed time budget exceeded".into()));
            }
            let batch = self.scoped_events(&watch).await?;
            if batch.recovery_required {
                return Err(Error::Unsupported("unverified native merge/rescan mode"));
            }
            event_count = event_count
                .checked_add(batch.events.len())
                .ok_or_else(|| Error::State("snapshot seed event overflow".into()))?;
            if event_count > 2_000_000 {
                return Err(Error::State("snapshot seed budget exceeded".into()));
            }
            emit(Progress {
                attempt,
                phase: Phase::Seed,
                units: page + 1,
                items: event_count as u64,
            });
            // Translate only the selected root here. Full directory contents come
            // from list replies; a historical event replay is not a full snapshot.
            for event in batch
                .events
                .iter()
                .filter(|e| e.file_id == self.scope.file_id)
            {
                let root_batch = EventBatch {
                    events: vec![event.clone()],
                    next: batch.next,
                    recovery_required: false,
                };
                let changes =
                    translate(&self.scope, self.client.restore_identity(), "", root_batch)?
                        .ok_or_else(|| Error::State("root event missing".into()))?;
                for e in changes.events {
                    match e.change {
                        Change::Upsert(n) if n.directory && n.parent.is_none() => root = Some(n),
                        _ => return Err(Error::State("selected root removed or replaced".into())),
                    }
                }
            }
            if batch.next == watch.cursor {
                if !batch.events.is_empty() {
                    return Err(Error::State(
                        "stationary seed cursor contains events".into(),
                    ));
                }
                boundary = true;
                break;
            }
            watch.cursor = batch.next;
        }
        if !boundary {
            return Err(Error::State("snapshot seed did not reach boundary".into()));
        }
        let root = root
            .ok_or_else(|| Error::State("selected root identity absent from native seed".into()))?;
        let mut nodes = vec![root];
        let mut inaccessible = Vec::new();
        let mut skipped_symlinks = 0u64;
        let mut identities = HashSet::from([self.scope.file_id.clone()]);
        let mut pending =
            VecDeque::from([(self.scope.file_id.clone(), self.scope.path.clone(), 0usize)]);
        let mut directories = 0;
        emit(Progress {
            attempt,
            phase: Phase::Enumerate,
            units: 0,
            items: 1,
        });
        // Only a locally observed namespace collision is eligible for a race
        // check. A transport/permission/format failure is not made transient by
        // unrelated activity on the server's global cursor.
        let mut conflict = None;
        let enumeration: Result<()> = async {
            // Bounded parallel metadata requests; no unbounded per-directory tasks.
            // Dropping JoinSet cancels siblings on failure/cancellation.
            let mut requests = tokio::task::JoinSet::new();
            while !pending.is_empty() || !requests.is_empty() {
                while requests.len() < 4 {
                    let Some((parent, path, depth)) = pending.pop_front() else {
                        break;
                    };
                    if depth > 128 {
                        return Err(Error::State("snapshot depth exceeded".into()));
                    }
                    let client = self.client.clone();
                    let view = self.scope.view_id;
                    requests.spawn(async move {
                        let result = client.list_dir(view, &path).await;
                        (parent, path, depth, result)
                    });
                }
                let (parent, path, depth, result) = requests
                    .join_next()
                    .await
                    .ok_or_else(|| Error::State("snapshot enumeration task absent".into()))?
                    .map_err(|_| Error::State("snapshot enumeration task failed".into()))?;
                let children = match result {
                    Ok(children) => children,
                    Err(sproto::Error::Server { code: 12290, .. })
                        if parent != self.scope.file_id =>
                    {
                        inaccessible.push(parent);
                        directories += 1;
                        emit(Progress {
                            attempt,
                            phase: Phase::Enumerate,
                            units: directories,
                            items: nodes.len() as u64,
                        });
                        continue;
                    }
                    Err(error) => return Err(error.into()),
                };
                let mut names = HashMap::new();
                for child in children {
                    if child.is_removed {
                        continue;
                    }
                    if let Some(collision) =
                        remember_child(&mut identities, &mut names, &child.file_id, &child.name)
                    {
                        conflict = Some(collision);
                        return Err(Error::State(
                            "snapshot identity or name changed during enumeration".into(),
                        ));
                    }
                    // Never follow remote symlinks or expose them as empty regular files.
                    if child.file_type == FileType::Symlink {
                        skipped_symlinks += 1;
                        continue;
                    }
                    let node = self.snapshot_node(&parent, &child)?;
                    if node.directory {
                        pending.push_back((
                            node.id.clone(),
                            format!("{}/{}", path.trim_end_matches('/'), node.name),
                            depth + 1,
                        ));
                    }
                    nodes.push(node);
                    if nodes.len() > SNAPSHOT_NODE_LIMIT {
                        return Err(Error::State("snapshot node budget exceeded".into()));
                    }
                }
                directories += 1;
                emit(Progress {
                    attempt,
                    phase: Phase::Enumerate,
                    units: directories,
                    items: nodes.len() as u64,
                });
                observer(path).await?;
            }
            Ok(())
        }
        .await;
        if let Err(error) = enumeration {
            // Advancement alone (including an empty scoped page) does not
            // explain the collision. Require an event for an involved identity.
            return Err(resolve_enumeration_failure(
                error,
                conflict,
                &self.scope,
                watch.cursor,
                |cursor| {
                    let mut probe = watch.clone();
                    probe.cursor = cursor;
                    async move { self.scoped_events(&probe).await }
                },
            )
            .await);
        }
        Ok(RemoteSnapshot {
            scope: self.scope.clone(),
            restore: self.client.restore_identity().into(),
            cursor: format!("native-v1:{}", watch.cursor.0),
            nodes,
            inaccessible,
            skipped_symlinks,
        })
    }
    fn snapshot_node(&self, parent: &str, n: &NodeInfo) -> Result<Node> {
        if !native_file_id(&n.file_id)
            || n.sync_id == 0
            || n.max_id == 0
            || n.file_type == FileType::Symlink
        {
            return Err(Error::Unsupported(
                "snapshot identity/version/type not supported",
            ));
        }
        let version = VersionToken {
            schema: 1,
            restore: self.client.restore_identity().into(),
            view: self.scope.view_id,
            root: self.scope.file_id.clone(),
            file: n.file_id.clone(),
            sync: n.sync_id,
            max: n.max_id,
            size: n.file_size,
            hash: n.hash.clone(),
            directory: n.file_type == FileType::Dir,
        };
        let node = Node {
            id: n.file_id.clone(),
            parent: Some(parent.into()),
            name: n.name.clone(),
            version: serde_json::to_string(&version)?,
            directory: version.directory,
            metadata: Some(crate::NodeMetadata {
                size: n.file_size,
                mtime: n.mtime,
                mode: if version.directory { 0o755 } else { 0o644 },
            }),
        };
        crate::state::valid_node(&node)?;
        if node.name.len() > 255 {
            return Err(Error::State("snapshot leaf name too long".into()));
        }
        Ok(node)
    }
    /// A scoped empty batch confirms no relevant changes through that batch.
    /// A global cursor may advance for unrelated paths. Keep the older snapshot
    /// cursor: subsequent pulls still replay everything after it, so no event is lost.
    pub async fn snapshot_unchanged(&self, snapshot: &RemoteSnapshot) -> Result<bool> {
        self.snapshot_unchanged_observed(snapshot, 1).await
    }
    async fn snapshot_unchanged_observed(
        &self,
        snapshot: &RemoteSnapshot,
        attempt: u8,
    ) -> Result<bool> {
        if snapshot.scope != self.scope || snapshot.restore != self.client.restore_identity() {
            return Err(Error::State("snapshot task or server changed".into()));
        }
        let mut watch = self.watch(&snapshot.cursor)?;
        let started = std::time::Instant::now();
        let mut events = 0;
        emit(Progress {
            attempt,
            phase: Phase::Fence,
            units: 0,
            items: 0,
        });
        for page in 0..4096 {
            if started.elapsed() > std::time::Duration::from_secs(900) {
                return Err(Error::State("snapshot seed time budget exceeded".into()));
            }
            let batch = self.scoped_events(&watch).await?;
            if batch.recovery_required {
                return Err(Error::Unsupported("unverified native merge/rescan mode"));
            }
            events += batch.events.len() as u64;
            emit(Progress {
                attempt,
                phase: Phase::Fence,
                units: page + 1,
                items: events,
            });
            match snapshot_fence_step(watch.cursor, batch.next, batch.events.len()) {
                FenceStep::Changed => return Ok(false),
                FenceStep::Complete => return Ok(true),
                FenceStep::Continue => watch.cursor = batch.next,
            }
        }
        Err(Error::State("snapshot seed did not reach boundary".into()))
    }

    pub async fn recovery_snapshot(&self) -> Result<RemoteSnapshot> {
        self.recovery_snapshot_with_observer(|_| async { Ok(()) })
            .await
    }
    #[cfg(feature = "native-research")]
    pub async fn research_recovery_snapshot<F, Fut>(&self, observer: F) -> Result<RemoteSnapshot>
    where
        F: FnMut(String) -> Fut,
        Fut: std::future::Future<Output = Result<()>>,
    {
        self.recovery_snapshot_with_observer(observer).await
    }
    async fn recovery_snapshot_with_observer<F, Fut>(
        &self,
        mut observer: F,
    ) -> Result<RemoteSnapshot>
    where
        F: FnMut(String) -> Fut,
        Fut: std::future::Future<Output = Result<()>>,
    {
        for attempt in 1..=3 {
            let snapshot = match self
                .snapshot_candidate_with_observer(&mut observer, attempt)
                .await
            {
                Ok(snapshot) => snapshot,
                Err(Error::SnapshotChanged) => continue,
                Err(error) => return Err(error),
            };
            if self.snapshot_unchanged_observed(&snapshot, attempt).await? {
                return Ok(snapshot);
            }
        }
        Err(sproto::Error::Timeout("snapshot repeatedly changed; retry recovery later").into())
    }
}

impl Store {
    /// Caller serializes remote mutation completions and event application while
    /// collecting/installing this snapshot. Local FUSE edits may continue.
    pub fn install_native_snapshot(
        &mut self,
        snapshot: &RemoteSnapshot,
        expected_cursor: &str,
    ) -> Result<()> {
        if self.cursor()? != expected_cursor || snapshot.nodes.len() > SNAPSHOT_NODE_LIMIT {
            return Err(Error::State(
                "snapshot base changed or exceeds budget".into(),
            ));
        }
        let saved: String = self.db.query_row(
            "SELECT value FROM metadata WHERE key='native_scope'",
            [],
            |r| r.get(0),
        )?;
        let scope: Scope = serde_json::from_str(&saved)?;
        let restore: String = self.db.query_row(
            "SELECT value FROM metadata WHERE key='server_restore_id'",
            [],
            |r| r.get(0),
        )?;
        if scope != snapshot.scope
            || restore != snapshot.restore
            || snapshot
                .cursor
                .strip_prefix("native-v1:")
                .and_then(|s| s.parse::<u64>().ok())
                .is_none()
        {
            return Err(Error::State(
                "snapshot scope, server or cursor mismatch".into(),
            ));
        }
        for n in &snapshot.nodes {
            let token: VersionToken = serde_json::from_str(&n.version)?;
            if token.schema != 1
                || token.restore != restore
                || token.view != scope.view_id
                || token.root != scope.file_id
                || token.file != n.id
                || token.directory != n.directory
            {
                return Err(Error::State("snapshot node token mismatch".into()));
            }
        }
        self.install_remote_snapshot_with_access(&snapshot.nodes, &snapshot.cursor, &snapshot.restore, &snapshot.inaccessible)?;
        // Informational only; content/cursor commit remains transactional above.
        self.db.execute("INSERT OR REPLACE INTO metadata(key,value) VALUES('skipped_symlinks',?1)", [snapshot.skipped_symlinks.to_string()])?;
        Ok(())
    }
    /// Existing inode and parent-name pairs whose kernel caches may need eviction.
    pub fn snapshot_invalidations(&self) -> Result<Vec<(Option<u64>, Option<u64>, String)>> {
        let mut q=self.db.prepare("SELECT i.ino,p.ino,n.name FROM nodes n LEFT JOIN fs_inodes i ON i.node=n.id LEFT JOIN fs_inodes p ON p.node=n.parent WHERE i.ino IS NOT NULL OR p.ino IS NOT NULL")?;
        Ok(q.query_map([], |r| {
            Ok((
                r.get::<_, Option<i64>>(0)?.map(|n| n as u64),
                r.get::<_, Option<i64>>(1)?.map(|n| n as u64),
                r.get(2)?,
            ))
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?)
    }
}

/// Private evidence created only by duplicate identity/name detection, never by
/// matching an error's text. At most two identities, not a copy of the candidate.
struct EnumerationConflict {
    first: String,
    second: Option<String>,
}
fn remember_child(
    identities: &mut HashSet<String>,
    names: &mut HashMap<String, String>,
    id: &str,
    name: &str,
) -> Option<EnumerationConflict> {
    if !identities.insert(id.to_owned()) {
        return Some(EnumerationConflict {
            first: id.to_owned(),
            second: None,
        });
    }
    names
        .insert(name.to_owned(), id.to_owned())
        .map(|first| EnumerationConflict {
            first,
            second: Some(id.to_owned()),
        })
}
fn classify_enumeration_error(
    error: Error,
    conflict: &EnumerationConflict,
    scope: &Scope,
    cursor: EventCursor,
    observed: &EventBatch,
) -> Error {
    let related = observed.events.iter().any(|event| {
        let identity = event.file_id == conflict.first
            || conflict
                .second
                .as_ref()
                .is_some_and(|id| id == &event.file_id);
        let destination = match &event.kind {
            EventKind::Move { destination, .. } => destination.as_str(),
            _ => event.path.as_str(),
        };
        identity && (scope.contains(&event.path) || scope.contains(destination))
    });
    if !observed.recovery_required && observed.next.0 > cursor.0 && related {
        Error::SnapshotChanged
    } else {
        error
    }
}
async fn resolve_enumeration_failure<F, Fut>(
    mut error: Error,
    conflict: Option<EnumerationConflict>,
    scope: &Scope,
    mut cursor: EventCursor,
    mut observe: F,
) -> Error
where
    F: FnMut(EventCursor) -> Fut,
    Fut: std::future::Future<Output = Result<EventBatch>>,
{
    let Some(conflict) = conflict else {
        return error;
    };
    // Related events may follow empty advancing pages. Use the existing fence
    // budgets, but never treat an incomplete probe as evidence of a race.
    let started = std::time::Instant::now();
    for _ in 0..4096 {
        if started.elapsed() > std::time::Duration::from_secs(900) {
            break;
        }
        let observed = match observe(cursor).await {
            Ok(batch) => batch,
            // Verification failure keeps its transport/root/protocol category.
            Err(error) => return error,
        };
        error = classify_enumeration_error(error, &conflict, scope, cursor, &observed);
        if matches!(error, Error::SnapshotChanged)
            || observed.recovery_required
            || observed.next.0 <= cursor.0
        {
            return error;
        }
        cursor = observed.next;
    }
    error
}
#[cfg(test)]
#[path = "native_snapshot_error_tests.rs"]
mod race_tests;

#[derive(Debug,PartialEq)]enum FenceStep { Changed, Continue, Complete }
fn snapshot_fence_step(cursor: EventCursor, next: EventCursor, count: usize) -> FenceStep {
    if count != 0 || next.0 < cursor.0 { FenceStep::Changed }
    else if next == cursor { FenceStep::Complete }
    else { FenceStep::Continue }
}
#[cfg(test)]mod scoped_fence_tests {
    use super::*;
    #[test]fn empty_advancing_page_is_not_a_complete_fence(){
        assert_eq!(snapshot_fence_step(EventCursor(2000),EventCursor(4000),0),FenceStep::Continue);
        assert_eq!(snapshot_fence_step(EventCursor(4000),EventCursor(6000),31),FenceStep::Changed);
        assert_eq!(snapshot_fence_step(EventCursor(6000),EventCursor(6000),0),FenceStep::Complete);
        assert_eq!(snapshot_fence_step(EventCursor(6000),EventCursor(4000),0),FenceStep::Changed);
    }
}
