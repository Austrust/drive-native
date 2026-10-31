//! Bounded committed metadata projections for a future status index.
//! No body I/O, inode/LRU writes, journal pruning, or desktop publication.
use crate::{
    Error, Result, Status, Store,
    download_jobs::DownloadFailure,
    status_aggregate::Availability,
    status_changes::{ChangePage, ChangeScope, GapReason, JournalWatermark, StatusCursor},
};
use rusqlite::params;
use std::{
    collections::{BTreeSet, HashMap, HashSet, VecDeque},
    time::{Duration, Instant},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DraftIdentity {
    pub token: String,
    pub base: String,
    pub generation: u64,
    pub checkpoint: u64,
    pub dirty: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContentIdentity {
    pub remote_version: String,
    pub local_version: Option<String>,
    pub blob: Option<String>,
    pub dirty_blob: Option<String>,
    pub draft: Option<DraftIdentity>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StatusAtom {
    pub id: String,
    pub parent: Option<String>,
    pub name: String,
    pub directory: bool,
    pub own: Status,
    pub inherited_pin: bool,
    pub availability: Availability,
    pub failure: Option<DownloadFailure>,
    pub content: ContentIdentity,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProjectedNode {
    pub id: String,
    pub atom: Option<StatusAtom>,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NameBucket {
    pub parent: Option<String>,
    pub name: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProjectionBatch {
    pub from: StatusCursor,
    pub through: StatusCursor,
    pub nodes: Vec<ProjectedNode>,
    pub affected_buckets: Vec<NameBucket>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StatusSnapshot {
    pub through: StatusCursor,
    pub atoms: Vec<StatusAtom>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProjectionBudget {
    pub max_changes: usize,
    pub max_nodes: usize,
    pub max_buckets: usize,
    /// Logical string/payload allowance, not SQLite/allocator RSS.
    pub max_bytes: usize,
    pub max_work: usize,
    pub max_elapsed: Duration,
}
impl Default for ProjectionBudget {
    fn default() -> Self {
        Self {
            max_changes: 8192,
            max_nodes: 100_000,
            max_buckets: 100_000,
            max_bytes: 64 * 1024 * 1024,
            max_work: 2_000_000,
            max_elapsed: Duration::from_secs(2),
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RebuildReason {
    Journal(GapReason),
    Budget,
    InvalidGraph,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Capture<T> {
    Ready(T),
    Rebuild {
        reason: RebuildReason,
        watermark: JournalWatermark,
    },
}

enum Stop {
    Rebuild(RebuildReason),
    Error(Error),
}
type ProjectResult<T> = std::result::Result<T, Stop>;
impl From<rusqlite::Error> for Stop {
    fn from(e: rusqlite::Error) -> Self {
        Self::Error(e.into())
    }
}
impl From<Error> for Stop {
    fn from(e: Error) -> Self {
        Self::Error(e)
    }
}
struct Meter {
    budget: ProjectionBudget,
    bytes: usize,
    work: usize,
    start: Instant,
}
impl Meter {
    fn spend(&mut self, bytes: usize) -> ProjectResult<()> {
        self.work = self.work.saturating_add(1);
        self.bytes = self.bytes.saturating_add(bytes);
        if self.bytes > self.budget.max_bytes
            || self.work > self.budget.max_work
            || self.start.elapsed() > self.budget.max_elapsed
        {
            Err(Stop::Rebuild(RebuildReason::Budget))
        } else {
            Ok(())
        }
    }
}
#[derive(Clone)]
struct Raw {
    id: String,
    parent: Option<String>,
    name: String,
    directory: bool,
    pinned: bool,
    deleted: bool,
    conflict: bool,
    denied: bool,
    shadowed: bool,
    overlay: bool,
    remote_parent: Option<String>,
    unlinked: bool,
    content: ContentIdentity,
    failure: Option<String>,
}
fn unsigned(row: &rusqlite::Row<'_>, index: usize) -> rusqlite::Result<u64> {
    let value: i64 = row.get(index)?;
    value
        .try_into()
        .map_err(|_| rusqlite::Error::IntegralValueOutOfRange(index, value))
}
fn length(row: &rusqlite::Row<'_>, index: usize) -> rusqlite::Result<usize> {
    let value: i64 = row.get(index)?;
    value
        .try_into()
        .map_err(|_| rusqlite::Error::IntegralValueOutOfRange(index, value))
}
struct Projector<'a> {
    store: &'a Store,
    meter: Meter,
    raw: HashMap<String, Option<Raw>>,
    overlay_children: Option<HashMap<String, Vec<String>>>,
}
impl<'a> Projector<'a> {
    fn new(store: &'a Store, budget: ProjectionBudget) -> Result<Self> {
        if budget.max_changes == 0
            || budget.max_changes > 65_536
            || budget.max_nodes == 0
            || budget.max_buckets == 0
            || budget.max_bytes == 0
            || budget.max_work == 0
            || budget.max_elapsed.is_zero()
        {
            return Err(Error::State("invalid status projection budget".into()));
        }
        Ok(Self {
            store,
            meter: Meter {
                budget,
                bytes: 0,
                work: 0,
                start: Instant::now(),
            },
            raw: HashMap::new(),
            overlay_children: None,
        })
    }
    fn raw(&mut self, id: &str) -> ProjectResult<Option<Raw>> {
        self.meter.spend(0)?;
        if let Some(value) = self.raw.get(id) {
            return Ok(value.clone());
        }
        if self.raw.len() >= self.meter.budget.max_nodes {
            return Err(Stop::Rebuild(RebuildReason::Budget));
        }
        // Check encoded lengths before allocating strings from SQLite. Each
        // unique source row is charged with room for output and worklist copies.
        let mut q=self.store.db.prepare_cached("SELECT n.parent,n.name,n.directory,n.pinned,n.deleted,n.conflict,
            EXISTS(SELECT 1 FROM access_denied a WHERE a.node=n.id),
            EXISTS(SELECT 1 FROM nodes l JOIN local_creations c ON c.node=l.id WHERE l.parent=n.parent AND l.name=n.name AND l.id!=n.id AND l.deleted=0),
            o.node IS NOT NULL,json_extract(o.remote,'$.parent'),EXISTS(SELECT 1 FROM local_unlinks u WHERE u.node=n.id),
            n.remote_version,n.local_version,n.blob,n.dirty_blob,d.token,d.base,d.generation,d.checkpoint,d.dirty,t.error,
            length(CAST(n.id AS BLOB))+coalesce(length(CAST(n.parent AS BLOB)),0)+length(CAST(n.name AS BLOB))+length(CAST(n.remote_version AS BLOB))
            +coalesce(length(CAST(n.local_version AS BLOB)),0)+coalesce(length(CAST(n.blob AS BLOB)),0)+coalesce(length(CAST(n.dirty_blob AS BLOB)),0)
            +coalesce(length(CAST(d.token AS BLOB)),0)+coalesce(length(CAST(d.base AS BLOB)),0)+coalesce(length(CAST(t.error AS BLOB)),0)
            +coalesce(length(CAST(json_extract(o.remote,'$.parent') AS BLOB)),0)
            FROM nodes n LEFT JOIN drafts d ON d.node=n.id LEFT JOIN namespace_overlay o ON o.node=n.id
            LEFT JOIN transfers t ON t.node=n.id AND t.direction='download' AND t.version=n.remote_version AND (n.blob IS NULL OR n.local_version!=n.remote_version)
            WHERE n.id=?1")?;
        let mut rows = q.query([id])?;
        let value = if let Some(r) = rows.next()? {
            let bytes = length(r, 21)?;
            self.meter
                .spend(bytes.saturating_mul(8).saturating_add(1024))?;
            let token: Option<String> = r.get(15)?;
            let draft = match token {
                Some(token) => Some(DraftIdentity {
                    token,
                    base: r.get(16)?,
                    generation: unsigned(r, 17)?,
                    checkpoint: unsigned(r, 18)?,
                    dirty: r.get(19)?,
                }),
                None => None,
            };
            let failure = r.get(20)?;
            Some(Raw {
                id: id.into(),
                parent: r.get(0)?,
                name: r.get(1)?,
                directory: r.get(2)?,
                pinned: r.get(3)?,
                deleted: r.get(4)?,
                conflict: r.get(5)?,
                denied: r.get(6)?,
                shadowed: r.get(7)?,
                overlay: r.get(8)?,
                remote_parent: r.get(9)?,
                unlinked: r.get(10)?,
                content: ContentIdentity {
                    remote_version: r.get(11)?,
                    local_version: r.get(12)?,
                    blob: r.get(13)?,
                    dirty_blob: r.get(14)?,
                    draft,
                },
                failure,
            })
        } else {
            self.meter
                .spend(id.len().saturating_mul(8).saturating_add(128))?;
            None
        };
        self.raw.insert(id.into(), value.clone());
        Ok(value)
    }
    fn atom(&mut self, id: &str) -> ProjectResult<Option<StatusAtom>> {
        let Some(node) = self.raw(id)? else {
            return Ok(None);
        };
        let mut current = Some(node.id.clone());
        let mut seen = HashSet::new();
        let mut pin = false;
        let mut overlay = false;
        let mut visible = true;
        while let Some(id) = current {
            if seen.len() >= 128 || !seen.insert(id.clone()) {
                return Err(Stop::Rebuild(RebuildReason::InvalidGraph));
            }
            let Some(n) = self.raw(&id)? else {
                return Err(Stop::Rebuild(RebuildReason::InvalidGraph));
            };
            pin |= n.pinned;
            overlay |= n.overlay;
            visible &= !n.deleted && !n.shadowed;
            current = n.parent;
        }
        // access_blocked propagates along BOTH local and retained remote edges.
        // Walk ancestors by primary key, not the global recursive SQL view.
        let mut pending = vec![(id.to_owned(), false, 0usize)];
        let mut colors = HashMap::new();
        while let Some((id, exit, depth)) = pending.pop() {
            if exit {
                colors.insert(id, 2u8);
                continue;
            }
            if depth >= 128 {
                return Err(Stop::Rebuild(RebuildReason::InvalidGraph));
            }
            match colors.get(&id) {
                Some(1) => return Err(Stop::Rebuild(RebuildReason::InvalidGraph)),
                Some(2) => continue,
                _ => {}
            }
            colors.insert(id.clone(), 1u8);
            let Some(n) = self.raw(&id)? else {
                return Err(Stop::Rebuild(RebuildReason::InvalidGraph));
            };
            visible &= !n.denied;
            pending.push((id, true, depth));
            if let Some(parent) = n.parent {
                pending.push((parent, false, depth + 1))
            }
            if let Some(parent) = n.remote_parent {
                pending.push((parent, false, depth + 1))
            }
        }
        if !visible {
            return Ok(None);
        }
        let dirty_draft = node.content.draft.as_ref().is_some_and(|d| d.dirty);
        let draft_conflict = dirty_draft
            && !node.unlinked
            && node
                .content
                .draft
                .as_ref()
                .is_some_and(|d| d.base != node.content.remote_version || node.deleted);
        let own = if node.conflict || draft_conflict {
            Status::Conflict
        } else if node.content.dirty_blob.is_some() || dirty_draft || overlay {
            Status::PendingUpload
        } else if node.content.blob.is_none() {
            if pin {
                Status::Pinning
            } else {
                Status::OnlineOnly
            }
        } else if node.content.local_version.as_ref() != Some(&node.content.remote_version) {
            Status::Updating
        } else if pin {
            Status::PinnedComplete
        } else {
            Status::LocalComplete
        };
        let availability =
            if node.content.blob.is_some() || node.content.dirty_blob.is_some() || dirty_draft {
                Availability::Unverified
            } else {
                Availability::Absent
            };
        let failure = if node.directory || matches!(own, Status::Conflict | Status::PendingUpload) {
            None
        } else {
            node.failure
                .map(|raw| -> ProjectResult<DownloadFailure> {
                    let failure: DownloadFailure =
                        serde_json::from_str(&raw).map_err(Error::from)?;
                    if failure.schema != 1 || failure.attempts == 0 {
                        return Err(Stop::Error(Error::State("invalid download failure".into())));
                    }
                    Ok(failure)
                })
                .transpose()?
        };
        Ok(Some(StatusAtom {
            id: node.id,
            parent: node.parent,
            name: node.name,
            directory: node.directory,
            own,
            inherited_pin: pin,
            availability,
            failure,
            content: node.content,
        }))
    }
    fn ids(
        &mut self,
        sql: &str,
        parent: Option<&str>,
        name: Option<&str>,
    ) -> ProjectResult<Vec<String>> {
        let mut q = self.store.db.prepare_cached(sql)?;
        let mut rows = q.query(params![parent, name])?;
        let mut out = Vec::new();
        while let Some(r) = rows.next()? {
            if out.len() >= self.meter.budget.max_nodes {
                return Err(Stop::Rebuild(RebuildReason::Budget));
            }
            self.meter
                .spend(length(r, 1)?.saturating_mul(4).saturating_add(128))?;
            out.push(r.get(0)?);
        }
        Ok(out)
    }
    fn descendants(&mut self, id: &str) -> ProjectResult<Vec<String>> {
        let mut out = self.ids(
            "SELECT id,length(CAST(id AS BLOB)) FROM nodes WHERE parent=?1 AND ?2 IS NULL",
            Some(id),
            None,
        )?;
        if self.overlay_children.is_none() {
            let mut edges: HashMap<String, Vec<String>> = HashMap::new();
            let mut count = 0;
            let mut q=self.store.db.prepare("SELECT node,json_extract(remote,'$.parent'),length(CAST(node AS BLOB))+coalesce(length(CAST(json_extract(remote,'$.parent') AS BLOB)),0) FROM namespace_overlay WHERE json_extract(remote,'$.parent') IS NOT NULL")?;
            let mut rows = q.query([])?;
            while let Some(r) = rows.next()? {
                count += 1;
                if count > self.meter.budget.max_nodes {
                    return Err(Stop::Rebuild(RebuildReason::Budget));
                }
                self.meter
                    .spend(length(r, 2)?.saturating_mul(4).saturating_add(128))?;
                edges.entry(r.get(1)?).or_default().push(r.get(0)?);
            }
            self.overlay_children = Some(edges);
        }
        if let Some(children) = self.overlay_children.as_ref().unwrap().get(id) {
            out.extend(children.iter().cloned())
        }
        Ok(out)
    }
}

fn complete<T>(result: ProjectResult<T>, watermark: JournalWatermark) -> Result<Capture<T>> {
    match result {
        Ok(value) => Ok(Capture::Ready(value)),
        Err(Stop::Rebuild(reason)) => Ok(Capture::Rebuild { reason, watermark }),
        Err(Stop::Error(e)) => Err(e),
    }
}
impl Store {
    /// Capture the WHOLE committed Store's visible tree under its existing
    /// exclusive caller boundary. Bounded, non-paged initial/rebuild foundation.
    pub fn snapshot_status(&self, budget: ProjectionBudget) -> Result<Capture<StatusSnapshot>> {
        let watermark = self.status_change_watermark()?;
        let mut p = Projector::new(self, budget)?;
        let result = (|| -> ProjectResult<_> {
            let ids=p.ids("SELECT id,length(CAST(id AS BLOB)) FROM nodes WHERE ?1 IS NULL AND ?2 IS NULL ORDER BY id",None,None)?;
            let mut atoms = Vec::new();
            for id in ids {
                if let Some(atom) = p.atom(&id)? {
                    atoms.push(atom)
                }
            }
            Ok(StatusSnapshot {
                through: watermark.cursor.clone(),
                atoms,
            })
        })();
        complete(result, watermark)
    }
    /// Read ALL retained changes through one committed watermark and project
    /// their current facts before returning. No journal acknowledgement occurs.
    pub fn capture_status_delta(
        &self,
        after: &StatusCursor,
        budget: ProjectionBudget,
    ) -> Result<Capture<ProjectionBatch>> {
        let mut p = Projector::new(self, budget)?;
        let watermark = self.status_change_watermark()?;
        if after.epoch == watermark.cursor.epoch
            && after.sequence >= watermark.retained_after
            && after.sequence <= watermark.cursor.sequence
        {
            let changes = watermark.cursor.sequence - after.sequence;
            let bytes = self.db.query_row(
                "SELECT coalesce(sum(bytes),0) FROM dn_status_changes WHERE seq>?1",
                [after.sequence as i64],
                |r| length(r, 0),
            )?;
            if changes > budget.max_changes as u64 || bytes > budget.max_bytes {
                return Ok(Capture::Rebuild {
                    reason: RebuildReason::Budget,
                    watermark,
                });
            }
        }
        let (changes, watermark) = match self.read_status_changes(after, budget.max_changes)? {
            ChangePage::Rebuild { reason, watermark } => {
                return Ok(Capture::Rebuild {
                    reason: RebuildReason::Journal(reason),
                    watermark,
                });
            }
            ChangePage::Ready {
                changes,
                watermark,
                has_more,
                ..
            } => {
                if has_more {
                    return Ok(Capture::Rebuild {
                        reason: RebuildReason::Budget,
                        watermark,
                    });
                }
                (changes, watermark)
            }
        };
        let result = (|| -> ProjectResult<_> {
            let mut todo = VecDeque::new();
            let mut buckets = std::collections::BTreeMap::new();
            let mut bucket_queue = VecDeque::new();
            for change in changes {
                p.meter.spend(change.bytes)?;
                let subtree = change.scope == ChangeScope::SubtreeAndAncestors;
                for id in [change.old_node, change.new_node].into_iter().flatten() {
                    todo.push_back((id, subtree))
                }
                for (parent, name) in [
                    (change.old_parent, change.old_name),
                    (change.new_parent, change.new_name),
                ] {
                    if let Some(name) = name {
                        bucket_queue.push_back((NameBucket { parent, name }, subtree))
                    }
                }
            }
            let mut affected = BTreeSet::new();
            let mut expanded = HashSet::new();
            while !todo.is_empty() || !bucket_queue.is_empty() {
                if let Some((bucket, subtree)) = bucket_queue.pop_front() {
                    if buckets.get(&bucket).is_some_and(|prior| *prior || !subtree) {
                        continue;
                    }
                    if !buckets.contains_key(&bucket) && buckets.len() >= budget.max_buckets {
                        return Err(Stop::Rebuild(RebuildReason::Budget));
                    }
                    p.meter.spend(
                        bucket.name.len() + bucket.parent.as_ref().map_or(0, String::len) + 128,
                    )?;
                    let ids=p.ids("SELECT id,length(CAST(id AS BLOB)) FROM nodes WHERE parent IS ?1 AND name=?2",bucket.parent.as_deref(),Some(&bucket.name))?;
                    buckets.insert(bucket, subtree);
                    for id in ids {
                        todo.push_back((id, subtree))
                    }
                    continue;
                }
                let (id, subtree) = todo.pop_front().unwrap();
                let first = affected.insert(id.clone());
                if affected.len() > budget.max_nodes {
                    return Err(Stop::Rebuild(RebuildReason::Budget));
                }
                let raw = p.raw(&id)?;
                if first {
                    if let Some(n) = &raw {
                        bucket_queue.push_back((
                            NameBucket {
                                parent: n.parent.clone(),
                                name: n.name.clone(),
                            },
                            subtree,
                        ))
                    }
                }
                if subtree
                    && expanded.insert(id.clone())
                    && raw.as_ref().is_none_or(|n| n.directory)
                {
                    for child in p.descendants(&id)? {
                        todo.push_back((child, true))
                    }
                }
            }
            let mut nodes = Vec::new();
            for id in affected {
                let atom = p.atom(&id)?;
                nodes.push(ProjectedNode { id, atom })
            }
            Ok(ProjectionBatch {
                from: after.clone(),
                through: watermark.cursor.clone(),
                nodes,
                affected_buckets: buckets.into_keys().collect(),
            })
        })();
        complete(result, watermark)
    }
}

#[cfg(test)]
#[path = "status_projection_tests.rs"]
mod tests;
