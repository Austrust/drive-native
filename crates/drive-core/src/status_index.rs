//! Pure, single-writer derived tree. No Store, filesystem, network, threads, or
//! desktop fallback. The caller owns publication synchronization and must mask
//! queries while a committed journal/transient change is pending.
use crate::{
    Status,
    status_aggregate::{
        self as aggregate, AggregateBudget, AggregateError, ChildState, DirectoryAggregate,
        LeafFacts, PreparedChanges, Summary,
    },
    status_changes::StatusCursor,
    status_projection::{NameBucket, ProjectedNode, ProjectionBatch, StatusAtom, StatusSnapshot},
    status_transient::{
        ContentKey, ContentResult, NodeTransient, PreparedTransient, TransientBatch,
        TransientBudget, TransientCursor, TransientError, TransientSnapshot, TransientView,
    },
};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug)]
pub struct IndexBudget {
    pub nodes: usize,
    pub bytes: usize,
    pub batch_nodes: usize,
    pub batch_bytes: usize,
    pub depth: usize,
    pub prepare_time: Duration,
    pub transient: TransientBudget,
}
impl Default for IndexBudget {
    fn default() -> Self {
        Self {
            nodes: 500_000,
            bytes: 1024 * 1024 * 1024,
            batch_nodes: 100_000,
            batch_bytes: 128 * 1024 * 1024,
            depth: 128,
            prepare_time: Duration::from_secs(5),
            transient: TransientBudget::default(),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IndexError {
    Epoch,
    Discontinuous,
    Budget,
    InvalidGraph,
    AmbiguousName,
    RootUnavailable,
    UnknownContent,
    InvalidPath,
    StaleContent,
    Transient(TransientError),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Stamp {
    pub database: StatusCursor,
    pub transient: Option<TransientCursor>,
    pub generation: u64,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ApplyStats {
    pub changed: bool,
    pub touched_nodes: usize,
    pub generation: u64,
}
struct Record {
    atom: StatusAtom,
    uid: u64,
    summary: ChildState,
    directory: Option<DirectoryAggregate>,
    aggregate_charge: usize,
}
struct Receipt {
    from: StatusCursor,
    through: StatusCursor,
    digest: [u8; 32],
}
pub struct IndexOwner {
    root: String,
    stamp: Stamp,
    budget: IndexBudget,
    nodes: BTreeMap<String, Record>,
    names: BTreeMap<NameBucket, String>,
    next_uid: u64,
    bytes: usize,
    receipts: VecDeque<Receipt>,
    transients: TransientView,
}
fn bucket(atom: &StatusAtom) -> NameBucket {
    NameBucket {
        parent: atom.parent.clone(),
        name: atom.name.clone(),
    }
}
fn atom_charge(a: &StatusAtom) -> usize {
    let c = &a.content;
    let strings = a.id.len()
        + a.parent.as_ref().map_or(0, String::len)
        + a.name.len()
        + c.remote_version.len()
        + c.local_version.as_ref().map_or(0, String::len)
        + c.blob.as_ref().map_or(0, String::len)
        + c.dirty_blob.as_ref().map_or(0, String::len)
        + c.draft.as_ref().map_or(0, |d| d.token.len() + d.base.len());
    // Includes node/namespace keys, ancestor work identifiers and BTree slots.
    strings.saturating_mul(8).saturating_add(1024)
}
fn known(result: Result<Summary, AggregateError>) -> Result<ChildState, IndexError> {
    match result {
        Ok(s) => Ok(ChildState::Known(s)),
        Err(AggregateError::UnverifiedContent | AggregateError::UnverifiedChildren(_)) => {
            Ok(ChildState::Unverified)
        }
        Err(AggregateError::Capacity) => Err(IndexError::Budget),
        Err(_) => Err(IndexError::InvalidGraph),
    }
}
fn leaf(atom: &StatusAtom, active_downloads: u32) -> Result<ChildState, IndexError> {
    known(aggregate::leaf(&LeafFacts {
        own: atom.own.clone(),
        inherited_pin: atom.inherited_pin,
        availability: atom.availability,
        active_downloads,
        failure: atom.failure.clone(),
    }))
}
fn digest(batch: &ProjectionBatch) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(format!("{:?}{:?}", batch.from, batch.through));
    for node in &batch.nodes {
        h.update(format!("{node:?}"));
    }
    for b in &batch.affected_buckets {
        h.update(format!("{b:?}"));
    }
    h.finalize().into()
}
impl IndexOwner {
    pub fn build(
        root: String,
        snapshot: StatusSnapshot,
        budget: IndexBudget,
    ) -> Result<Self, IndexError> {
        budget.transient.validate().map_err(IndexError::Transient)?;
        if budget.depth == 0
            || budget.depth > 128
            || budget.nodes == 0
            || budget.batch_nodes == 0
            || budget.prepare_time.is_zero()
        {
            return Err(IndexError::Budget);
        }
        if root.is_empty()
            || root.len() > 8192
            || snapshot.through.epoch.is_empty()
            || snapshot.through.epoch.len() > 128
        {
            return Err(IndexError::InvalidGraph);
        }
        if snapshot.atoms.len() > budget.nodes {
            return Err(IndexError::Budget);
        }
        if snapshot
            .atoms
            .iter()
            .fold(0usize, |n, a| n.saturating_add(atom_charge(a)))
            > budget.bytes
        {
            return Err(IndexError::Budget);
        }
        let mut owner = Self {
            root,
            stamp: Stamp {
                database: StatusCursor {
                    epoch: snapshot.through.epoch.clone(),
                    sequence: 0,
                },
                transient: None,
                generation: 0,
            },
            budget,
            nodes: BTreeMap::new(),
            names: BTreeMap::new(),
            next_uid: 1,
            // Fixed receipt history, root/stamp and empty owner containers.
            bytes: 32 * 1024,
            receipts: VecDeque::new(),
            transients: TransientView::new(budget.transient),
        };
        // Initialization is the one whole-tree operation. Its separate limits
        // permit a large snapshot without enlarging everyday batch limits.
        let original = owner.budget;
        owner.budget.batch_nodes = budget.nodes;
        owner.budget.batch_bytes = budget.bytes;
        let batch = ProjectionBatch {
            from: owner.stamp.database.clone(),
            through: snapshot.through,
            nodes: snapshot
                .atoms
                .into_iter()
                .map(|a| crate::status_projection::ProjectedNode {
                    id: a.id.clone(),
                    atom: Some(a),
                })
                .collect(),
            affected_buckets: Vec::new(),
        };
        owner.apply_internal(batch, true)?;
        owner.budget = original;
        owner.receipts.clear();
        Ok(owner)
    }
    pub fn stamp(&self) -> &Stamp {
        &self.stamp
    }
    pub fn accounted_bytes(&self) -> usize {
        self.bytes.saturating_add(self.transients.bytes())
    }
    pub fn len(&self) -> usize {
        self.nodes.len()
    }
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
    fn directory(&self) -> DirectoryAggregate {
        DirectoryAggregate::new(AggregateBudget {
            children: self.budget.nodes,
            bytes: self.budget.bytes,
        })
    }
    pub fn apply(&mut self, batch: ProjectionBatch) -> Result<ApplyStats, IndexError> {
        self.apply_internal(batch, false)
    }
    /// A second, independent stream. Database cursors never stand in for flight
    /// or availability changes. First use and gaps require an explicit snapshot.
    pub fn apply_transient(&mut self, batch: TransientBatch) -> Result<ApplyStats, IndexError> {
        let plan = self
            .transients
            .batch(batch)
            .map_err(IndexError::Transient)?;
        self.apply_transient_plan(plan)
    }
    /// The caller must keep pending queries masked while capturing DB and
    /// transient snapshots consistently. Cursor equality checks association,
    /// not whether the caller really held the required capture locks.
    pub fn install_transient_snapshot(
        &mut self,
        expected_database: &StatusCursor,
        snapshot: TransientSnapshot,
    ) -> Result<ApplyStats, IndexError> {
        if expected_database != &self.stamp.database {
            return Err(IndexError::Discontinuous);
        }
        let plan = self
            .transients
            .snapshot(snapshot)
            .map_err(IndexError::Transient)?;
        self.apply_transient_plan(plan)
    }
    fn apply_transient_plan(&mut self, plan: PreparedTransient) -> Result<ApplyStats, IndexError> {
        if plan.duplicate {
            return Ok(ApplyStats {
                changed: false,
                touched_nodes: 0,
                generation: self.stamp.generation,
            });
        }
        let mut ids: BTreeSet<_> = plan.ids().cloned().collect();
        // Snapshot replacement may invalidate previously verified facts even
        // if the new source no longer has an entry for that body.
        if plan.reset {
            ids.extend(
                self.nodes
                    .iter()
                    .filter(|(_, r)| {
                        !r.atom.directory
                            && r.atom.availability == aggregate::Availability::Complete
                    })
                    .map(|(id, _)| id.clone()),
            );
        }
        let mut work_bytes = plan.work_bytes;
        let mut nodes = Vec::new();
        for id in ids {
            if let Some(record) = self.nodes.get(&id) {
                work_bytes = work_bytes.saturating_add(atom_charge(&record.atom));
                if work_bytes > self.budget.bytes {
                    return Err(IndexError::Budget);
                }
                nodes.push(ProjectedNode {
                    id,
                    atom: Some(record.atom.clone()),
                });
            }
        }
        let wide = plan.reset;
        self.apply_tree(nodes, work_bytes, None, Some(plan), wide)
    }
    fn transient_state<'a>(
        &'a self,
        plan: Option<&'a PreparedTransient>,
        id: &str,
    ) -> Option<&'a NodeTransient> {
        match plan {
            Some(plan) => plan.state(&self.transients, id),
            None => self.transients.state(id),
        }
    }
    fn materialize(&self, atom: &mut StatusAtom, plan: Option<&PreparedTransient>) {
        if atom.directory {
            return;
        }
        let key = ContentKey::from_content(&atom.content);
        let state = self.transient_state(plan, &atom.id);
        if let Some(state) = state.filter(|s| s.content == key && key.is_some()) {
            atom.availability = match state.availability {
                Some(ContentResult::Complete) => aggregate::Availability::Complete,
                Some(ContentResult::Missing) | None => aggregate::Availability::Unverified,
            };
            return;
        }
        // Explicit unbind revokes its proof even if a later different binding
        // shares this folded batch. A mismatched old-body result, by contrast,
        // cannot revoke this body. A fresh matching proof above supersedes it.
        if key
            .as_ref()
            .is_some_and(|key| plan.is_some_and(|p| p.revoked(&atom.id, key)))
        {
            atom.availability = aggregate::Availability::Unverified;
            return;
        }
        if plan.is_some_and(|p| p.reset) && key.is_some() {
            atom.availability = aggregate::Availability::Unverified;
            return;
        }
        if atom.availability == aggregate::Availability::Unverified {
            if let Some(old) = self
                .nodes
                .get(&atom.id)
                .filter(|old| ContentKey::from_content(&old.atom.content) == key)
            {
                atom.availability = old.atom.availability;
            }
        }
    }
    fn leaf(
        &self,
        atom: &StatusAtom,
        plan: Option<&PreparedTransient>,
    ) -> Result<ChildState, IndexError> {
        let active = self
            .transient_state(plan, &atom.id)
            .map_or(0, |n| n.flights.len());
        leaf(atom, u32::try_from(active).map_err(|_| IndexError::Budget)?)
    }
    fn final_atom<'a>(
        &'a self,
        changes: &'a BTreeMap<String, Option<StatusAtom>>,
        id: &str,
    ) -> Option<&'a StatusAtom> {
        match changes.get(id) {
            Some(a) => a.as_ref(),
            None => self.nodes.get(id).map(|r| &r.atom),
        }
    }
    fn ancestry(
        &self,
        changes: &BTreeMap<String, Option<StatusAtom>>,
        id: &str,
    ) -> Result<Vec<String>, IndexError> {
        let mut result = Vec::new();
        let mut at = Some(id);
        while let Some(id) = at {
            if result.len() >= self.budget.depth || result.iter().any(|s| s == id) {
                return Err(IndexError::InvalidGraph);
            }
            let a = self
                .final_atom(changes, id)
                .ok_or(IndexError::InvalidGraph)?;
            if !result.is_empty() && !a.directory {
                return Err(IndexError::InvalidGraph);
            }
            result.push(id.to_owned());
            at = a.parent.as_deref();
        }
        Ok(result)
    }
    fn apply_internal(
        &mut self,
        batch: ProjectionBatch,
        building: bool,
    ) -> Result<ApplyStats, IndexError> {
        if batch.from.epoch != self.stamp.database.epoch
            || batch.through.epoch != self.stamp.database.epoch
        {
            return Err(IndexError::Epoch);
        }
        if batch.nodes.len() > self.budget.batch_nodes
            || batch.affected_buckets.len() > self.budget.batch_nodes.saturating_mul(2)
        {
            return Err(IndexError::Budget);
        }
        let work_bytes = batch
            .nodes
            .iter()
            .fold(0usize, |sum, n| {
                sum.saturating_add(n.id.len().saturating_mul(8))
                    .saturating_add(
                        n.atom
                            .as_ref()
                            .map_or(n.id.len().saturating_mul(8) + 128, atom_charge),
                    )
            })
            .saturating_add(
                batch
                    .affected_buckets
                    .iter()
                    .map(|b| b.name.len() + b.parent.as_ref().map_or(0, String::len) + 128)
                    .sum::<usize>(),
            );
        if work_bytes > self.budget.batch_bytes {
            return Err(IndexError::Budget);
        }
        let fingerprint = digest(&batch);
        if !building
            && self.receipts.iter().any(|r| {
                r.from == batch.from && r.through == batch.through && r.digest == fingerprint
            })
        {
            return Ok(ApplyStats {
                changed: false,
                touched_nodes: 0,
                generation: self.stamp.generation,
            });
        }
        if batch.from != self.stamp.database
            || batch.through.sequence < batch.from.sequence
            || (!building
                && batch.through.sequence == batch.from.sequence
                && !batch.nodes.is_empty())
        {
            return Err(IndexError::Discontinuous);
        }
        let receipt = Receipt {
            from: batch.from,
            through: batch.through,
            digest: fingerprint,
        };
        self.apply_tree(batch.nodes, work_bytes, Some(receipt), None, false)
    }
    fn apply_tree(
        &mut self,
        nodes: Vec<ProjectedNode>,
        mut work_bytes: usize,
        receipt: Option<Receipt>,
        transient: Option<PreparedTransient>,
        wide: bool,
    ) -> Result<ApplyStats, IndexError> {
        let started = Instant::now();
        let work_limit = if wide {
            self.budget.bytes
        } else {
            self.budget.batch_bytes
        };
        if nodes.len()
            > if wide {
                self.budget.nodes
            } else {
                self.budget.batch_nodes
            }
            || work_bytes > work_limit
        {
            return Err(IndexError::Budget);
        }
        let transient_bytes = transient
            .as_ref()
            .map_or(self.transients.bytes(), |p| p.bytes);
        let mut changes = BTreeMap::new();
        for mut n in nodes {
            if started.elapsed() > self.budget.prepare_time {
                return Err(IndexError::Budget);
            }
            if n.id.is_empty() || n.id.len() > 8192 || changes.contains_key(&n.id) {
                return Err(IndexError::InvalidGraph);
            }
            if let Some(a) = &n.atom {
                if a.id != n.id
                    || a.name.contains('/')
                    || a.name.contains('\0')
                    || matches!(a.name.as_str(), "." | "..")
                    || a.name.len() > 8192
                    || a.own == Status::Deleted
                {
                    return Err(IndexError::InvalidGraph);
                }
            }
            if let Some(atom) = &mut n.atom {
                self.materialize(atom, transient.as_ref());
            }
            changes.insert(n.id, n.atom);
        }
        // No complete tree copy. Only input rows and their old/new ancestor
        // closures are staged; untouched directory contribution tables stay put.
        let empty = BTreeMap::new();
        let mut affected = BTreeSet::new();
        let mut depths = BTreeMap::new();
        let mut added_names = BTreeMap::new();
        let mut count = self.nodes.len();
        let mut bytes = self.bytes;
        let mut uids = BTreeMap::new();
        let mut next_uid = self.next_uid;
        let mut add_affected = |id: String, set: &mut BTreeSet<String>| -> Result<(), IndexError> {
            if !set.contains(&id) {
                work_bytes =
                    work_bytes.saturating_add(id.len().saturating_mul(12).saturating_add(1024));
                if work_bytes > work_limit {
                    return Err(IndexError::Budget);
                }
                set.insert(id);
            }
            Ok(())
        };
        for (id, new) in &changes {
            if started.elapsed() > self.budget.prepare_time {
                return Err(IndexError::Budget);
            }
            if let Some(old) = self.nodes.get(id) {
                for a in self.ancestry(&empty, id)? {
                    add_affected(a, &mut affected)?;
                }
                bytes -= atom_charge(&old.atom);
            } else if new.is_some() {
                count += 1;
            }
            if new.is_none() && self.nodes.contains_key(id) {
                count -= 1;
            }
            if let Some(a) = new {
                if a.name.is_empty() && a.parent.is_some() {
                    return Err(IndexError::InvalidGraph);
                }
                bytes = bytes
                    .checked_add(atom_charge(a))
                    .ok_or(IndexError::Budget)?;
                let chain = self.ancestry(&changes, id)?;
                for parent in chain {
                    add_affected(parent, &mut affected)?;
                }
                if a.parent
                    .as_deref()
                    .and_then(|p| self.final_atom(&changes, p))
                    .is_some_and(|p| p.inherited_pin)
                    && !a.inherited_pin
                {
                    return Err(IndexError::InvalidGraph);
                }
                let key = bucket(a);
                if added_names.insert(key.clone(), id.clone()).is_some() {
                    return Err(IndexError::AmbiguousName);
                }
                if let Some(existing) = self.names.get(&key) {
                    if existing != id
                        && self
                            .final_atom(&changes, existing)
                            .is_some_and(|n| bucket(n) == key)
                    {
                        return Err(IndexError::AmbiguousName);
                    }
                }
                let uid = if let Some(r) = self.nodes.get(id) {
                    r.uid
                } else {
                    let u = next_uid;
                    next_uid = next_uid.checked_add(1).ok_or(IndexError::Budget)?;
                    u
                };
                uids.insert(id.clone(), uid);
            }
            add_affected(id.clone(), &mut affected)?;
        }
        drop(add_affected);
        // A changed inherited pin requires an explicit projection of existing
        // descendants. The atom intentionally lacks a local-pin bit, so the
        // owner must not guess which old inherited values remain valid.
        for (id, new) in &changes {
            if self
                .nodes
                .get(id)
                .zip(new.as_ref())
                .is_some_and(|(old, new)| {
                    old.atom.directory
                        && new.directory
                        && old.atom.inherited_pin != new.inherited_pin
                })
            {
                let mut pending = vec![id.clone()];
                let mut visited = 0usize;
                while let Some(parent) = pending.pop() {
                    if started.elapsed() > self.budget.prepare_time {
                        return Err(IndexError::Budget);
                    }
                    let start = NameBucket {
                        parent: Some(parent.clone()),
                        name: String::new(),
                    };
                    for (key, child) in self.names.range(start..) {
                        if key.parent.as_deref() != Some(parent.as_str()) {
                            break;
                        }
                        visited += 1;
                        if visited > self.budget.batch_nodes || !changes.contains_key(child) {
                            return Err(IndexError::InvalidGraph);
                        }
                        if self.nodes[child].atom.directory {
                            pending.push(child.clone());
                        }
                    }
                }
            }
        }
        if count > self.budget.nodes
            || bytes
                .max(self.bytes)
                .saturating_add(transient_bytes.max(self.transients.bytes()))
                .checked_add(work_bytes)
                .is_none_or(|n| n > self.budget.bytes)
        {
            return Err(IndexError::Budget);
        }
        if self
            .final_atom(&changes, &self.root)
            .is_none_or(|a| !a.directory)
        {
            return Err(IndexError::RootUnavailable);
        }
        // Validate final ancestors, and compute depth once per affected row.
        for id in &affected {
            if started.elapsed() > self.budget.prepare_time {
                return Err(IndexError::Budget);
            }
            if self.final_atom(&changes, id).is_some() {
                depths.insert(id.clone(), self.ancestry(&changes, id)?.len());
            }
        }
        let mut child_changes: BTreeMap<String, BTreeMap<u64, Option<String>>> = BTreeMap::new();
        for id in &affected {
            if let Some(old) = self.nodes.get(id) {
                if let Some(p) = &old.atom.parent {
                    child_changes
                        .entry(p.clone())
                        .or_default()
                        .insert(old.uid, None);
                }
            }
            if let Some(a) = self.final_atom(&changes, id) {
                if let Some(p) = &a.parent {
                    let uid = uids
                        .get(id)
                        .copied()
                        .or_else(|| self.nodes.get(id).map(|r| r.uid))
                        .ok_or(IndexError::InvalidGraph)?;
                    child_changes
                        .entry(p.clone())
                        .or_default()
                        .insert(uid, Some(id.clone()));
                }
            }
        }
        let mut order: Vec<_> = affected.iter().cloned().collect();
        order.sort_by_key(|id| std::cmp::Reverse(depths.get(id).copied().unwrap_or(0)));
        let mut summaries: BTreeMap<String, ChildState> = BTreeMap::new();
        let mut patches: BTreeMap<String, PreparedChanges> = BTreeMap::new();
        let mut new_directories = BTreeMap::new();
        let mut charges = BTreeMap::new();
        for id in &order {
            if started.elapsed() > self.budget.prepare_time {
                return Err(IndexError::Budget);
            }
            let atom = self.final_atom(&changes, id);
            let old = self.nodes.get(id);
            let mut edits = Vec::new();
            for (uid, child) in child_changes.remove(id).unwrap_or_default() {
                let state = match child {
                    Some(c) => Some(
                        summaries
                            .get(&c)
                            .or_else(|| self.nodes.get(&c).map(|r| &r.summary))
                            .ok_or(IndexError::InvalidGraph)?
                            .clone(),
                    ),
                    None => None,
                };
                edits.push((uid, state));
            }
            if atom.is_some_and(|a| a.directory) || old.is_some_and(|r| r.directory.is_some()) {
                let agg = if let Some(agg) = old.and_then(|r| r.directory.as_ref()) {
                    agg
                } else {
                    new_directories
                        .entry(id.clone())
                        .or_insert_with(|| self.directory())
                };
                let own = atom.map_or(&Status::Deleted, |a| &a.own);
                let patch = agg
                    .prepare(edits, own, atom.is_some_and(|a| a.inherited_pin))
                    .map_err(|_| IndexError::InvalidGraph)?;
                if !atom.is_some_and(|a| a.directory) && patch.count() > 0 {
                    return Err(IndexError::InvalidGraph);
                }
                if let Some(a) = atom {
                    if a.directory {
                        summaries.insert(id.clone(), known(patch.summary().clone())?);
                    } else {
                        summaries.insert(id.clone(), self.leaf(a, transient.as_ref())?);
                    }
                }
                bytes -= old.map_or(0, |r| r.aggregate_charge);
                let charge = if atom.is_some_and(|a| a.directory) {
                    patch.retained()
                } else {
                    0
                };
                bytes = bytes.checked_add(charge).ok_or(IndexError::Budget)?;
                charges.insert(id.clone(), charge);
                patches.insert(id.clone(), patch);
            } else if let Some(a) = atom {
                if !edits.is_empty() {
                    return Err(IndexError::InvalidGraph);
                }
                summaries.insert(id.clone(), self.leaf(a, transient.as_ref())?);
            }
        }
        if bytes
            .max(self.bytes)
            .saturating_add(transient_bytes.max(self.transients.bytes()))
            .checked_add(work_bytes)
            .is_none_or(|n| n > self.budget.bytes)
        {
            return Err(IndexError::Budget);
        }
        let changed = transient.as_ref().is_some_and(|p| p.reset)
            || changes
                .iter()
                .any(|(id, next)| self.nodes.get(id).map(|r| &r.atom) != next.as_ref())
            || summaries
                .iter()
                .any(|(id, next)| self.nodes.get(id).map(|r| &r.summary) != Some(next));
        let generation = if changed {
            self.stamp
                .generation
                .checked_add(1)
                .ok_or(IndexError::Budget)?
        } else {
            self.stamp.generation
        };
        if started.elapsed() > self.budget.prepare_time {
            return Err(IndexError::Budget);
        }
        // All graph, semantic and logical-budget failures precede this point.
        // Exclusivity (&mut self) ensures no reader sees a partial generation.
        for (id, _) in &changes {
            if let Some(r) = self.nodes.get(id) {
                self.names.remove(&bucket(&r.atom));
            }
        }
        for (id, patch) in patches {
            if let Some(agg) = self.nodes.get_mut(&id).and_then(|r| r.directory.as_mut()) {
                agg.commit(patch).expect("exclusive prepared aggregate");
            } else if let Some(agg) = new_directories.get_mut(&id) {
                agg.commit(patch).expect("exclusive new aggregate");
            }
        }
        for (id, atom) in changes {
            match atom {
                None => {
                    self.nodes.remove(&id);
                }
                Some(atom) => {
                    self.names.insert(bucket(&atom), id.clone());
                    if let Some(record) = self.nodes.get_mut(&id) {
                        if !atom.directory {
                            record.directory = None
                        } else if record.directory.is_none() {
                            record.directory = new_directories.remove(&id)
                        }
                        record.atom = atom;
                    } else {
                        self.nodes.insert(
                            id.clone(),
                            Record {
                                atom,
                                uid: uids[&id],
                                summary: summaries[&id].clone(),
                                directory: new_directories.remove(&id),
                                aggregate_charge: 0,
                            },
                        );
                    }
                }
            }
        }
        for (id, summary) in summaries {
            if let Some(r) = self.nodes.get_mut(&id) {
                r.summary = summary;
            }
        }
        for (id, charge) in charges {
            if let Some(r) = self.nodes.get_mut(&id) {
                r.aggregate_charge = charge;
            }
        }
        self.bytes = bytes;
        self.next_uid = next_uid;
        self.stamp.generation = generation;
        if let Some(receipt) = receipt {
            self.stamp.database = receipt.through.clone();
            self.receipts.push_back(receipt);
            if self.receipts.len() > 32 {
                self.receipts.pop_front();
            }
        }
        if let Some(plan) = transient {
            self.stamp.transient = Some(plan.through.clone());
            self.transients.commit(plan);
        }
        Ok(ApplyStats {
            changed,
            touched_nodes: affected.len(),
            generation: self.stamp.generation,
        })
    }
    /// Relative paths only. No SQL, file probing or temporary tree construction.
    pub fn query(&self, path: &str) -> Result<&Summary, IndexError> {
        if path.len() > 8192 || path.starts_with('/') || path.contains('\0') {
            return Err(IndexError::InvalidPath);
        }
        let mut id = self.root.as_str();
        if !path.is_empty() {
            for (depth, name) in path.split('/').enumerate() {
                if depth >= self.budget.depth || matches!(name, "" | "." | "..") {
                    return Err(IndexError::InvalidPath);
                }
                if !self.nodes.get(id).is_some_and(|r| r.atom.directory) {
                    return Err(IndexError::InvalidPath);
                }
                id = self
                    .names
                    .get(&NameBucket {
                        parent: Some(id.to_owned()),
                        name: name.to_owned(),
                    })
                    .ok_or(IndexError::InvalidPath)?;
            }
        }
        match &self
            .nodes
            .get(id)
            .ok_or(IndexError::RootUnavailable)?
            .summary
        {
            ChildState::Known(s) => Ok(s),
            ChildState::Unverified => Err(IndexError::UnknownContent),
        }
    }
    pub fn query_many(
        &self,
        paths: &[String],
    ) -> Result<(Stamp, Vec<Result<Summary, IndexError>>), IndexError> {
        if paths.len() > 256 {
            return Err(IndexError::Budget);
        }
        Ok((
            self.stamp.clone(),
            paths.iter().map(|p| self.query(p).cloned()).collect(),
        ))
    }
}

#[cfg(test)]
#[path = "status_index_tests.rs"]
mod tests;
