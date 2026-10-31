//! Private candidate reconciliation. Only complete direct file listings can be
//! replaced locally; directory/ACL ambiguity retains the original full retry.
//! No intermediate candidate or advanced cursor is exposed to the state owner.
use super::*;
use sproto::actions::native::NativeEvent;
use std::collections::BTreeSet;
use std::time::Duration;
use tokio::time::{Instant, timeout_at};

const PAGE_LIMIT: usize = 4096;
const LIST_LIMIT: usize = 4096;
const EVENT_LIMIT: usize = 2_000_000;
const TIME_LIMIT: Duration = Duration::from_secs(900);

struct Candidate {
    snapshot: RemoteSnapshot,
    nodes: HashMap<String, Node>,
    children: HashMap<String, HashSet<String>>,
    inaccessible: HashSet<String>,
}

impl Candidate {
    fn validate_token(snapshot: &RemoteSnapshot, node: &Node) -> Result<()> {
        crate::state::valid_node(node)?;
        let token: VersionToken = serde_json::from_str(&node.version)?;
        if token.schema != 1
            || token.restore != snapshot.restore
            || token.view != snapshot.scope.view_id
            || token.root != snapshot.scope.file_id
            || token.file != node.id
            || token.directory != node.directory
        {
            return Err(Error::State("snapshot candidate identity mismatch".into()));
        }
        Ok(())
    }

    fn new(mut snapshot: RemoteSnapshot) -> Result<Self> {
        snapshot.scope.validate()?;
        if snapshot.nodes.len() > SNAPSHOT_NODE_LIMIT {
            return Err(Error::State("snapshot node budget exceeded".into()));
        }
        let mut nodes = HashMap::new();
        let mut children: HashMap<String, HashSet<String>> = HashMap::new();
        let mut names = HashSet::new();
        for node in std::mem::take(&mut snapshot.nodes) {
            Self::validate_token(&snapshot, &node)?;
            if !names.insert((node.parent.clone(), node.name.clone())) {
                return Err(Error::State("snapshot candidate identity mismatch".into()));
            }
            if let Some(parent) = &node.parent {
                children
                    .entry(parent.clone())
                    .or_default()
                    .insert(node.id.clone());
            }
            if nodes.insert(node.id.clone(), node).is_some() {
                return Err(Error::State("snapshot candidate duplicate identity".into()));
            }
        }
        let candidate = Self {
            inaccessible: snapshot.inaccessible.iter().cloned().collect(),
            snapshot,
            nodes,
            children,
        };
        let root = candidate
            .nodes
            .get(&candidate.snapshot.scope.file_id)
            .ok_or_else(|| Error::State("snapshot candidate root missing".into()))?;
        if !root.directory || root.parent.is_some() {
            return Err(Error::State("snapshot candidate root invalid".into()));
        }
        // One O(N) traversal validates the graph. Do not allocate every node's
        // full path: a stationary fence must not add O(N * depth) path work.
        let mut pending = VecDeque::from([(root.id.as_str(), 0usize)]);
        let mut visited = HashSet::new();
        while let Some((id, depth)) = pending.pop_front() {
            let node = &candidate.nodes[id];
            if !visited.insert(id) || depth > if node.directory { 128 } else { 129 } {
                return Err(Error::State(
                    "snapshot candidate cycle or depth exceeded".into(),
                ));
            }
            if let Some(children) = candidate.children.get(id) {
                if !node.directory {
                    return Err(Error::State(
                        "snapshot candidate parent not directory".into(),
                    ));
                }
                pending.extend(children.iter().map(|id| (id.as_str(), depth + 1)));
            }
        }
        if visited.len() != candidate.nodes.len() {
            return Err(Error::State("snapshot candidate unreachable nodes".into()));
        }
        Ok(candidate)
    }

    fn path(&self, id: &str) -> Result<String> {
        let mut names: Vec<&str> = Vec::new();
        let mut current = id;
        // Existing enumeration permits a file below the deepest (128) directory.
        for _ in 0..=129 {
            let node = self
                .nodes
                .get(current)
                .ok_or_else(|| Error::State("snapshot candidate parent absent".into()))?;
            if current == self.snapshot.scope.file_id {
                let mut path = self.snapshot.scope.path.trim_end_matches('/').to_owned();
                for name in names.iter().rev() {
                    path.push('/');
                    path.push_str(name);
                }
                return Ok(if path.is_empty() { "/".into() } else { path });
            }
            names.push(node.name.as_str());
            current = node
                .parent
                .as_deref()
                .ok_or_else(|| Error::State("snapshot candidate extra root".into()))?;
            if !self.nodes.get(current).is_some_and(|node| node.directory) {
                return Err(Error::State(
                    "snapshot candidate parent not directory".into(),
                ));
            }
        }
        Err(Error::State(
            "snapshot candidate cycle or depth exceeded".into(),
        ))
    }

    fn accessible_parent(&self, id: &str) -> bool {
        self.nodes.get(id).is_some_and(|node| node.directory) && !self.inaccessible.contains(id)
    }

    fn event_parent(&self, id: &str, path: &str, parents: &mut BTreeSet<String>) -> Result<bool> {
        if !self.snapshot.scope.contains(path) {
            return Ok(true);
        }
        let Some((parent_path, _)) = path.rsplit_once('/') else {
            return Ok(false);
        };
        if !self.accessible_parent(id) {
            return Ok(false);
        }
        let expected = if parent_path.is_empty() {
            "/"
        } else {
            parent_path
        };
        if self.path(id)? != expected {
            return Ok(false);
        }
        parents.insert(id.into());
        Ok(true)
    }

    fn affected(&self, events: &[NativeEvent]) -> Result<Option<BTreeSet<String>>> {
        let mut parents = BTreeSet::new();
        for event in events {
            root_guard::root_event(&self.snapshot.scope, event)?;
            // Directory metadata can encode visibility changes. Do not treat
            // Modify as proof that descendants or access are unchanged.
            if event.file_type != FileType::File
                || event.file_id == self.snapshot.scope.file_id
                || self
                    .nodes
                    .get(&event.file_id)
                    .is_some_and(|node| node.directory)
            {
                return Ok(None);
            }
            if !self.event_parent(&event.parent_id, &event.path, &mut parents)? {
                return Ok(None);
            }
            if let EventKind::Move {
                destination,
                parent_id,
            } = &event.kind
            {
                if !self.event_parent(parent_id, destination, &mut parents)? {
                    return Ok(None);
                }
            }
            if let Some(node) = self.nodes.get(&event.file_id) {
                let Some(parent) = &node.parent else {
                    return Ok(None);
                };
                if !self.accessible_parent(parent) {
                    return Ok(None);
                }
                parents.insert(parent.clone());
            }
            // An out-of-scope event must not authorize cursor advancement even
            // when a buggy fetcher accidentally supplies it.
            let destination = match &event.kind {
                EventKind::Move { destination, .. } => destination,
                _ => &event.path,
            };
            if !self.snapshot.scope.contains(&event.path)
                && !self.snapshot.scope.contains(destination)
            {
                return Err(Error::State("event outside selected native subtree".into()));
            }
        }
        Ok(Some(parents))
    }

    /// The map is private and disposable. All replies must agree on the same
    /// directory identities/metadata before changing any direct file membership.
    fn replace_files(&mut self, listings: Vec<(String, Vec<Node>)>) -> Result<bool> {
        let parents: HashSet<_> = listings.iter().map(|(id, _)| id.clone()).collect();
        let mut fresh_count = 0;
        let mut identities = HashSet::new();
        for (parent, listing) in &listings {
            let mut names = HashSet::new();
            let mut directories = HashSet::new();
            for node in listing {
                Self::validate_token(&self.snapshot, node)?;
                if node.parent.as_ref() != Some(parent)
                    || !names.insert(&node.name)
                    || !identities.insert(&node.id)
                {
                    return Ok(false);
                }
                if node.directory {
                    if self.nodes.get(&node.id) != Some(node) {
                        return Ok(false);
                    }
                    directories.insert(node.id.clone());
                } else {
                    if self.nodes.get(&node.id).is_some_and(|old| {
                        old.directory || old.parent.as_ref().is_none_or(|p| !parents.contains(p))
                    }) {
                        return Ok(false);
                    }
                    fresh_count += 1;
                }
            }
            let old_directories: HashSet<_> = self
                .children
                .get(parent)
                .into_iter()
                .flatten()
                .filter(|id| self.nodes[*id].directory)
                .cloned()
                .collect();
            if directories != old_directories {
                return Ok(false);
            }
        }
        let mut remove = Vec::new();
        for parent in &parents {
            remove.extend(
                self.children
                    .get(parent)
                    .into_iter()
                    .flatten()
                    .filter(|id| !self.nodes[*id].directory)
                    .cloned(),
            );
        }
        let final_count = self.nodes.len() - remove.len() + fresh_count;
        if final_count > SNAPSHOT_NODE_LIMIT {
            return Err(Error::State("snapshot node budget exceeded".into()));
        }
        for id in remove {
            let node = self.nodes.remove(&id).unwrap();
            self.children
                .get_mut(node.parent.as_ref().unwrap())
                .unwrap()
                .remove(&id);
        }
        for node in listings
            .into_iter()
            .flat_map(|(_, nodes)| nodes)
            .filter(|node| !node.directory)
        {
            self.children
                .entry(node.parent.clone().unwrap())
                .or_default()
                .insert(node.id.clone());
            self.nodes.insert(node.id.clone(), node);
        }
        Ok(true)
    }

    fn finish(mut self, cursor: EventCursor) -> RemoteSnapshot {
        self.snapshot.cursor = format!("native-v1:{}", cursor.0);
        self.snapshot.nodes = self.nodes.into_values().collect();
        // Installation uses identities, not order (parallel enumeration already
        // has nondeterministic ordering). Avoid an unnecessary O(N log N) sort.
        self.snapshot
    }
}

pub(super) async fn converge<F, Fut, L, LFut>(
    snapshot: RemoteSnapshot,
    attempt: u8,
    fetch: F,
    list: L,
) -> Result<Option<RemoteSnapshot>>
where
    F: FnMut(EventCursor) -> Fut,
    Fut: std::future::Future<Output = Result<EventBatch>>,
    L: FnMut(String, String) -> LFut,
    LFut: std::future::Future<Output = Result<Vec<Node>>>,
{
    converge_until(snapshot, attempt, fetch, list, Instant::now() + TIME_LIMIT).await
}

async fn converge_until<F, Fut, L, LFut>(
    snapshot: RemoteSnapshot,
    attempt: u8,
    mut fetch: F,
    mut list: L,
    deadline: Instant,
) -> Result<Option<RemoteSnapshot>>
where
    F: FnMut(EventCursor) -> Fut,
    Fut: std::future::Future<Output = Result<EventBatch>>,
    L: FnMut(String, String) -> LFut,
    LFut: std::future::Future<Output = Result<Vec<Node>>>,
{
    let mut cursor = EventCursor(
        snapshot
            .cursor
            .strip_prefix("native-v1:")
            .and_then(|s| s.parse().ok())
            .ok_or_else(|| Error::State("invalid snapshot cursor".into()))?,
    );
    let mut candidate = Candidate::new(snapshot)?;
    let mut events = 0usize;
    let mut lists = 0usize;
    emit(Progress {
        attempt,
        phase: Phase::Fence,
        units: 0,
        items: 0,
    });
    for page in 0..PAGE_LIMIT {
        if Instant::now() >= deadline {
            return Err(Error::State(
                "snapshot convergence time budget exceeded".into(),
            ));
        }
        let batch = timeout_at(deadline, fetch(cursor))
            .await
            .map_err(|_| Error::State("snapshot convergence time budget exceeded".into()))??;
        if Instant::now() >= deadline {
            return Err(Error::State(
                "snapshot convergence time budget exceeded".into(),
            ));
        }
        if batch.recovery_required {
            return Err(Error::Unsupported("unverified native merge/rescan mode"));
        }
        if batch.next.0 < cursor.0 || (batch.next == cursor && !batch.events.is_empty()) {
            return Err(Error::State(
                "snapshot convergence cursor did not advance".into(),
            ));
        }
        events = events
            .checked_add(batch.events.len())
            .ok_or_else(|| Error::State("snapshot convergence event overflow".into()))?;
        if events > EVENT_LIMIT {
            return Err(Error::State(
                "snapshot convergence event budget exceeded".into(),
            ));
        }
        emit(Progress {
            attempt,
            phase: Phase::Fence,
            units: page as u64 + 1,
            items: events as u64,
        });
        if batch.next == cursor {
            return Ok(Some(candidate.finish(cursor)));
        }
        if !batch.events.is_empty() {
            let Some(parents) = candidate.affected(&batch.events)? else {
                return Ok(None);
            };
            let mut replies = Vec::new();
            let mut reply_nodes = 0usize;
            for parent in parents {
                lists += 1;
                if lists > LIST_LIMIT || Instant::now() >= deadline {
                    return Err(Error::State(
                        "snapshot convergence local listing budget exceeded".into(),
                    ));
                }
                let path = candidate.path(&parent)?;
                emit(Progress {
                    attempt,
                    phase: Phase::Reconcile,
                    units: lists as u64 - 1,
                    items: events as u64,
                });
                let nodes = match timeout_at(deadline, list(parent.clone(), path))
                    .await
                    .map_err(|_| Error::State("snapshot convergence time budget exceeded".into()))?
                {
                    Ok(nodes) => nodes,
                    Err(Error::SnapshotChanged)
                    | Err(Error::Native(sproto::Error::Server { code: 12290, .. })) => {
                        return Ok(None);
                    }
                    Err(error) => return Err(error),
                };
                if Instant::now() >= deadline {
                    return Err(Error::State(
                        "snapshot convergence time budget exceeded".into(),
                    ));
                }
                reply_nodes = reply_nodes
                    .checked_add(nodes.len())
                    .ok_or_else(|| Error::State("snapshot convergence listing overflow".into()))?;
                if reply_nodes > SNAPSHOT_NODE_LIMIT {
                    return Err(Error::State(
                        "snapshot convergence listing budget exceeded".into(),
                    ));
                }
                emit(Progress {
                    attempt,
                    phase: Phase::Reconcile,
                    units: lists as u64,
                    items: events as u64,
                });
                replies.push((parent, nodes));
            }
            if !candidate.replace_files(replies)? {
                return Ok(None);
            }
        }
        // These are received positions only. Nothing becomes installable until
        // a subsequent stationary empty fence covers all local rechecks.
        cursor = batch.next;
    }
    Err(Error::State(
        "snapshot convergence did not reach boundary".into(),
    ))
}

#[cfg(test)]
#[path = "native_snapshot_reconcile_tests.rs"]
mod tests;
