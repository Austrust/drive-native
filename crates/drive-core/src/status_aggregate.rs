//! Pure incremental summaries; no SQLite, paths, network, clocks or body I/O.
//! The future index owner supplies validated visible children and inherited pin.
//! This module does not implement publication, ACL filtering or epoch recovery.
use crate::{Status, download_jobs::DownloadFailure};
use serde::Serialize;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::atomic::{AtomicU64, Ordering};
static NEXT_AGGREGATE: AtomicU64 = AtomicU64::new(1);
fn next_identity() -> u64 {
    let mut current = NEXT_AGGREGATE.load(Ordering::Relaxed);
    loop {
        let next = current
            .checked_add(1)
            .expect("aggregate identity space exhausted");
        match NEXT_AGGREGATE.compare_exchange_weak(
            current,
            next,
            Ordering::Relaxed,
            Ordering::Relaxed,
        ) {
            Ok(_) => return current,
            Err(value) => current = value,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
#[repr(usize)]
pub enum Label {
    OnlineOnly,
    LocalComplete,
    PinnedComplete,
    Updating,
    Pinning,
    PendingUpload,
    Conflict,
    DownloadFailed,
    UpdateFailed,
    DownloadRetry,
    UpdateRetry,
    Downloading,
    Mixed,
}
const LABELS: usize = Label::Mixed as usize + 1;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Summary {
    pub status: Label,
    pub pinned: bool,
    pub offline_available: bool,
    pub sync_error: Option<DownloadFailure>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Availability {
    Complete,
    Absent,
    Unverified,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AggregateError {
    NotVisible,
    UnverifiedContent,
    InvalidFacts,
    UnverifiedChildren(usize),
    Capacity,
}

pub struct LeafFacts {
    pub own: Status,
    pub inherited_pin: bool,
    pub availability: Availability,
    /// Number of live (identity, version) flights, not the number of RPCs.
    pub active_downloads: u32,
    pub failure: Option<DownloadFailure>,
}

pub fn leaf(facts: &LeafFacts) -> Result<Summary, AggregateError> {
    let mut status = match facts.own {
        Status::OnlineOnly => Label::OnlineOnly,
        Status::LocalComplete => Label::LocalComplete,
        Status::PinnedComplete => Label::PinnedComplete,
        Status::Updating => Label::Updating,
        Status::Pinning => Label::Pinning,
        Status::PendingUpload => Label::PendingUpload,
        Status::Conflict => Label::Conflict,
        Status::Deleted => return Err(AggregateError::NotVisible),
    };
    let offline_available = match facts.availability {
        Availability::Complete => true,
        Availability::Absent => false,
        Availability::Unverified => return Err(AggregateError::UnverifiedContent),
    };
    if !offline_available
        && matches!(
            facts.own,
            Status::LocalComplete | Status::PinnedComplete | Status::Updating
        )
    {
        return Err(AggregateError::InvalidFacts);
    }
    let protected = matches!(facts.own, Status::Conflict | Status::PendingUpload);
    let sync_error = if protected {
        None
    } else {
        facts.failure.clone()
    };
    if let Some(error) = &sync_error {
        if error.schema != 1 || error.attempts == 0 {
            return Err(AggregateError::InvalidFacts);
        }
        status = match (offline_available, error.retry_at.is_some()) {
            (true, true) => Label::UpdateRetry,
            (true, false) => Label::UpdateFailed,
            (false, true) => Label::DownloadRetry,
            (false, false) => Label::DownloadFailed,
        };
    }
    if facts.active_downloads > 0 && !protected {
        status = if offline_available {
            Label::Updating
        } else {
            Label::Downloading
        };
    }
    Ok(Summary {
        status,
        pinned: facts.inherited_pin,
        offline_available,
        sync_error,
    })
}

/// Unknown children remain counted and block a "complete" parent summary.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ChildState {
    Known(Summary),
    Unverified,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AggregateBudget {
    pub children: usize,
    /// Accounted fixed payload allowance, not a promise about allocator RSS.
    pub bytes: usize,
}

// Allowance includes a child slot and a conservative hash-table payload factor.
// The error index is charged separately. The outer index must also budget its
// node/path records, candidates and queued deltas rather than multiply this cap.
const CHILD_BYTES: usize = 2 * std::mem::size_of::<(u64, ChildState)>();
const ERROR_BYTES: usize = 2 * std::mem::size_of::<((u32, u64), DownloadFailure)>();
fn charge(state: &ChildState) -> usize {
    CHILD_BYTES
        + match state {
            ChildState::Known(Summary {
                sync_error: Some(_),
                ..
            }) => ERROR_BYTES,
            _ => 0,
        }
}

/// Maintains immediate-child contributions. A directory's summary already
/// contains descendant effects; feeding its entire subtree again double-counts.
pub struct DirectoryAggregate {
    children: HashMap<u64, ChildState>,
    counts: [usize; LABELS],
    offline: usize,
    unknown: usize,
    errors: BTreeMap<(u32, u64), DownloadFailure>,
    budget: AggregateBudget,
    bytes: usize,
    reserved_slots: usize,
    revision: u64,
    identity: u64,
}
impl DirectoryAggregate {
    pub fn new(budget: AggregateBudget) -> Self {
        Self {
            children: HashMap::new(),
            counts: [0; LABELS],
            offline: 0,
            unknown: 0,
            errors: BTreeMap::new(),
            budget,
            bytes: 0,
            reserved_slots: 0,
            revision: 0,
            identity: next_identity(),
        }
    }
    pub fn len(&self) -> usize {
        self.children.len()
    }
    pub fn is_empty(&self) -> bool {
        self.children.is_empty()
    }
    pub fn accounted_bytes(&self) -> usize {
        self.bytes
    }
    /// The tree owner must budget retained allocation too, not only live rows.
    /// This allowance is intentionally separate from measured process RSS.
    pub fn retained_allowance(&self) -> usize {
        self.reserved_slots
            .saturating_mul(CHILD_BYTES)
            .saturating_add(self.errors.len().saturating_mul(ERROR_BYTES))
    }

    /// Repeated changes are no-ops. Rejection preserves the existing summary.
    /// Only this child is inspected; queries never traverse the children map.
    pub fn set_child(&mut self, id: u64, next: Option<ChildState>) -> Result<bool, AggregateError> {
        let previous = self.children.get(&id);
        if previous == next.as_ref() {
            return Ok(false);
        }
        if let Some(ChildState::Known(summary)) = &next {
            if summary
                .sync_error
                .as_ref()
                .is_some_and(|e| e.schema != 1 || e.attempts == 0)
            {
                return Err(AggregateError::InvalidFacts);
            }
        }
        let count =
            self.children.len() - usize::from(previous.is_some()) + usize::from(next.is_some());
        let bytes = self.bytes - previous.map_or(0, charge) + next.as_ref().map_or(0, charge);
        if count > self.budget.children || bytes > self.budget.bytes {
            return Err(AggregateError::Capacity);
        }
        let next_revision = self
            .revision
            .checked_add(1)
            .ok_or(AggregateError::InvalidFacts)?;
        if let Some(previous) = self.children.remove(&id) {
            self.remove_contribution(id, &previous);
        }
        if let Some(next) = next {
            self.add_contribution(id, &next);
            self.children.insert(id, next);
        }
        if self.children.is_empty() {
            self.children = HashMap::new();
            self.reserved_slots = 0;
        } else {
            self.reserved_slots = self.reserved_slots.max(self.children.capacity());
        }
        self.bytes = bytes;
        self.revision = next_revision;
        Ok(true)
    }
    fn remove_contribution(&mut self, id: u64, state: &ChildState) {
        match state {
            ChildState::Unverified => self.unknown -= 1,
            ChildState::Known(s) => {
                self.counts[s.status as usize] -= 1;
                self.offline -= usize::from(s.offline_available);
                if let Some(e) = &s.sync_error {
                    self.errors.remove(&(e.attempts, id));
                }
            }
        }
    }
    fn add_contribution(&mut self, id: u64, state: &ChildState) {
        match state {
            ChildState::Unverified => self.unknown += 1,
            ChildState::Known(s) => {
                self.counts[s.status as usize] += 1;
                self.offline += usize::from(s.offline_available);
                if let Some(e) = &s.sync_error {
                    self.errors.insert((e.attempts, id), e.clone());
                }
            }
        }
    }
    pub fn summary(&self, own: &Status, inherited_pin: bool) -> Result<Summary, AggregateError> {
        summarize(
            &self.counts,
            self.offline,
            self.unknown,
            self.children.len(),
            self.errors.last_key_value().map(|(_, e)| e.clone()),
            own,
            inherited_pin,
        )
    }

    /// Preview a bounded set of immediate-child changes without cloning or
    /// traversing a wide directory. All removals logically precede insertions.
    pub fn prepare(
        &self,
        changes: Vec<(u64, Option<ChildState>)>,
        own: &Status,
        inherited_pin: bool,
    ) -> Result<PreparedChanges, AggregateError> {
        let mut ids = HashSet::new();
        let mut counts = self.counts;
        let (mut offline, mut unknown, mut count, mut bytes) =
            (self.offline, self.unknown, self.children.len(), self.bytes);
        let mut next_error = None;
        for (id, next) in &changes {
            if !ids.insert(*id) {
                return Err(AggregateError::InvalidFacts);
            }
            if let Some(old) = self.children.get(id) {
                count -= 1;
                bytes -= charge(old);
                match old {
                    ChildState::Unverified => unknown -= 1,
                    ChildState::Known(s) => {
                        counts[s.status as usize] -= 1;
                        offline -= usize::from(s.offline_available);
                    }
                }
            }
            if let Some(next) = next {
                count += 1;
                bytes = bytes
                    .checked_add(charge(next))
                    .ok_or(AggregateError::Capacity)?;
                match next {
                    ChildState::Unverified => unknown += 1,
                    ChildState::Known(s) => {
                        counts[s.status as usize] += 1;
                        offline += usize::from(s.offline_available);
                        if let Some(e) = &s.sync_error {
                            if e.schema != 1 || e.attempts == 0 {
                                return Err(AggregateError::InvalidFacts);
                            }
                            let candidate = ((e.attempts, *id), e.clone());
                            if next_error
                                .as_ref()
                                .is_none_or(|(key, _)| *key < candidate.0)
                            {
                                next_error = Some(candidate)
                            }
                        }
                    }
                }
            }
        }
        if count > self.budget.children || bytes > self.budget.bytes {
            return Err(AggregateError::Capacity);
        }
        // At most changes.len() old representatives are skipped, regardless of
        // the number of siblings in the directory.
        let old_error = self
            .errors
            .iter()
            .rev()
            .find(|((_, id), _)| !ids.contains(id));
        let error = match (old_error, next_error) {
            (Some((a, e)), Some((b, n))) => Some(if *a > b { e.clone() } else { n }),
            (Some((_, e)), None) => Some(e.clone()),
            (None, Some((_, e))) => Some(e),
            (None, None) => None,
        };
        let summary = summarize(&counts, offline, unknown, count, error, own, inherited_pin);
        // Conservative capacity charge, including retained old hash slots.
        // This is a logical bound, not an allocator/RSS guarantee.
        let retained = self
            .reserved_slots
            .max(count.saturating_mul(2).saturating_add(4))
            .saturating_mul(CHILD_BYTES)
            .saturating_add(count.saturating_mul(ERROR_BYTES));
        let changes: Vec<_> = changes
            .into_iter()
            .filter(|(id, next)| self.children.get(id) != next.as_ref())
            .collect();
        if !changes.is_empty() && self.revision == u64::MAX {
            return Err(AggregateError::InvalidFacts);
        }
        Ok(PreparedChanges {
            identity: self.identity,
            revision: self.revision,
            changes,
            summary,
            count,
            retained,
        })
    }

    /// Exclusive tree owner commits only after every directory was prepared.
    /// A stale patch is rejected before mutation. No fallible semantic checks
    /// remain after removal starts; allocation failure retains Rust's normal
    /// process-failure semantics rather than claiming OOM recovery.
    pub fn commit(&mut self, patch: PreparedChanges) -> Result<(), AggregateError> {
        if patch.identity != self.identity || patch.revision != self.revision {
            return Err(AggregateError::InvalidFacts);
        }
        if patch.changes.is_empty() {
            return Ok(());
        }
        let next_revision = self
            .revision
            .checked_add(1)
            .ok_or(AggregateError::InvalidFacts)?;
        for (id, _) in &patch.changes {
            if let Some(previous) = self.children.remove(id) {
                self.bytes -= charge(&previous);
                self.remove_contribution(*id, &previous);
            }
        }
        for (id, next) in patch.changes {
            if let Some(next) = next {
                self.bytes += charge(&next);
                self.add_contribution(id, &next);
                self.children.insert(id, next);
            }
        }
        if self.children.is_empty() {
            self.children = HashMap::new();
            self.reserved_slots = 0
        } else {
            self.reserved_slots = self.reserved_slots.max(self.children.capacity())
        }
        self.revision = next_revision;
        Ok(())
    }
}

pub struct PreparedChanges {
    identity: u64,
    revision: u64,
    changes: Vec<(u64, Option<ChildState>)>,
    summary: Result<Summary, AggregateError>,
    count: usize,
    retained: usize,
}
impl PreparedChanges {
    pub fn summary(&self) -> &Result<Summary, AggregateError> {
        &self.summary
    }
    pub fn count(&self) -> usize {
        self.count
    }
    pub fn retained(&self) -> usize {
        self.retained
    }
}

fn summarize(
    counts: &[usize; LABELS],
    offline: usize,
    unknown: usize,
    count: usize,
    error: Option<DownloadFailure>,
    own: &Status,
    inherited_pin: bool,
) -> Result<Summary, AggregateError> {
    if *own == Status::Deleted {
        return Err(AggregateError::NotVisible);
    }
    if unknown > 0 {
        return Err(AggregateError::UnverifiedChildren(unknown));
    }
    let offline_available = offline == count;
    let has = |label: Label| counts[label as usize] > 0;
    let status = if *own == Status::Conflict || has(Label::Conflict) {
        Label::Conflict
    } else if *own == Status::PendingUpload || has(Label::PendingUpload) {
        Label::PendingUpload
    } else if has(Label::DownloadFailed) || has(Label::UpdateFailed) {
        if offline_available {
            Label::UpdateFailed
        } else {
            Label::DownloadFailed
        }
    } else if has(Label::DownloadRetry) || has(Label::UpdateRetry) {
        if offline_available {
            Label::UpdateRetry
        } else {
            Label::DownloadRetry
        }
    } else if has(Label::Updating) {
        Label::Updating
    } else if has(Label::Downloading) {
        Label::Downloading
    } else if has(Label::Pinning) || inherited_pin && !offline_available {
        Label::Pinning
    } else if offline_available {
        if inherited_pin {
            Label::PinnedComplete
        } else {
            Label::LocalComplete
        }
    } else if offline > 0 || has(Label::Mixed) {
        Label::Mixed
    } else {
        Label::OnlineOnly
    };
    Ok(Summary {
        status,
        pinned: inherited_pin,
        offline_available,
        // Deterministic tie: greatest immediate-child identity. Clearing an
        // error promotes its successor without searching the whole subtree.
        sync_error: error,
    })
}

#[cfg(test)]
#[path = "status_aggregate_tests.rs"]
mod tests;
