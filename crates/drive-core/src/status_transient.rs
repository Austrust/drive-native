//! FSD-lifetime transient facts, independent of the database and RPC session.
//! No I/O or threads. Runtime callers serialize lifecycle changes, capture
//! snapshots with the DB publication boundary, and deliver notifications after
//! releasing their locks. Journal overflow is an explicit snapshot requirement.
use crate::status_projection::ContentIdentity;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ContentKey {
    Blob {
        blob_id: String,
        version: Option<String>,
    },
    DirtyBlob {
        blob_id: String,
    },
    Draft {
        token: String,
        base: String,
        generation: u64,
    },
}
impl ContentKey {
    /// The version belongs to the actual local body. A remote expected version
    /// advancing never invalidates a still-present immutable cached blob.
    pub fn from_content(c: &ContentIdentity) -> Option<Self> {
        if let Some(d) = c.draft.as_ref().filter(|d| d.dirty) {
            Some(Self::Draft {
                token: d.token.clone(),
                base: d.base.clone(),
                generation: d.generation,
            })
        } else if let Some(blob_id) = &c.dirty_blob {
            Some(Self::DirtyBlob { blob_id: blob_id.clone() })
        } else {
            c.blob.as_ref().map(|blob_id| Self::Blob {
                blob_id: blob_id.clone(),
                version: c.local_version.clone(),
            })
        }
    }
    fn bytes(&self) -> usize {
        match self {
            Self::Blob { blob_id, version } => blob_id.len() + version.as_ref().map_or(0, String::len),
            Self::DirtyBlob { blob_id } => blob_id.len(),
            Self::Draft { token, base, .. } => token.len() + base.len(),
        }
    }
    fn valid(&self) -> bool {
        match self {
            Self::Blob { blob_id, version } => {
                valid(blob_id) && version.as_ref().is_none_or(|v| valid(v))
            }
            Self::DirtyBlob { blob_id } => valid(blob_id),
            Self::Draft { token, base, .. } => valid(token) && valid(base),
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContentResult {
    Complete,
    Missing,
}
/// Issued only by the current FSD source. A ticket is single-use, and a later
/// probe or content rebind invalidates it even when the body key is identical.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VerificationTicket {
    epoch: String,
    token: u64,
    id: String,
    content: ContentKey,
}
impl VerificationTicket {
    pub fn id(&self) -> &str {
        &self.id
    }
    pub fn content(&self) -> &ContentKey {
        &self.content
    }
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Flight {
    pub version: String,
    pub token: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NodeTransient {
    pub id: String,
    /// Sorted unique (version, lifecycle-token) pairs. Readers sharing a flight
    /// do not create extra entries; cancelling one reader must not finish it.
    pub flights: Vec<Flight>,
    pub content: Option<ContentKey>,
    pub availability: Option<ContentResult>,
}
impl NodeTransient {
    fn empty(id: String) -> Self {
        Self {
            id,
            flights: Vec::new(),
            content: None,
            availability: None,
        }
    }
    pub fn is_empty(&self) -> bool {
        self.flights.is_empty() && self.content.is_none()
    }
    pub(crate) fn bytes(&self) -> usize {
        512usize
            .saturating_add(self.id.len().saturating_mul(4))
            .saturating_add(
                self.content
                    .as_ref()
                    .map_or(0, |c| c.bytes().saturating_mul(4)),
            )
            .saturating_add(
                self.flights
                    .iter()
                    .map(|f| (f.version.len() + f.token.len()).saturating_mul(4) + 128)
                    .sum::<usize>(),
            )
    }
    pub(crate) fn validate(&self, budget: TransientBudget) -> Result<(), TransientError> {
        if !valid(&self.id)
            || self.flights.len() > budget.flights_per_node
            || self.content.as_ref().is_some_and(|c| !c.valid())
            || self.content.is_none() && self.availability.is_some()
        {
            return Err(TransientError::Invalid);
        }
        let mut previous = None;
        for f in &self.flights {
            if !valid(&f.version) || !valid(&f.token) || previous.is_some_and(|p| p >= f) {
                return Err(TransientError::Invalid);
            }
            previous = Some(f);
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TransientCursor {
    pub epoch: String,
    pub sequence: u64,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TransientChange {
    pub sequence: u64,
    pub state: NodeTransient,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TransientBatch {
    pub from: TransientCursor,
    pub through: TransientCursor,
    pub changes: Vec<TransientChange>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TransientSnapshot {
    pub through: TransientCursor,
    pub nodes: Vec<NodeTransient>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TransientRead {
    Batch(TransientBatch),
    SnapshotRequired { through: TransientCursor },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransientError {
    Invalid,
    Budget,
    SequenceExhausted,
    Epoch,
    Discontinuous,
    SnapshotRequired,
}
#[derive(Clone, Copy, Debug)]
pub struct TransientBudget {
    pub nodes: usize,
    pub flights: usize,
    pub flights_per_node: usize,
    pub bytes: usize,
    pub journal_entries: usize,
    pub journal_bytes: usize,
    pub batch_entries: usize,
}
impl Default for TransientBudget {
    fn default() -> Self {
        Self {
            nodes: 500_000,
            flights: 4096,
            flights_per_node: 16,
            bytes: 256 * 1024 * 1024,
            journal_entries: 8192,
            journal_bytes: 8 * 1024 * 1024,
            batch_entries: 8192,
        }
    }
}
impl TransientBudget {
    pub(crate) fn validate(self) -> Result<(), TransientError> {
        if self.nodes == 0
            || self.flights == 0
            || self.flights_per_node == 0
            || self.flights_per_node > u32::MAX as usize
            || self.bytes == 0
            || self.journal_entries == 0
            || self.journal_bytes == 0
            || self.batch_entries == 0
        {
            Err(TransientError::Invalid)
        } else {
            Ok(())
        }
    }
}
fn valid(value: &str) -> bool {
    !value.is_empty() && value.len() <= 8192 && !value.contains('\0')
}

/// Authoritative transient state owned by the FSD, not by an individual worker.
/// Each semantic state change is one indivisible journal transition. Probe
/// issuance/cancellation and acknowledgements do not advance display sequence.
/// Duplicates and stale verification/completion callbacks do not bump seq.
pub struct TransientSource {
    cursor: TransientCursor,
    budget: TransientBudget,
    nodes: BTreeMap<String, NodeTransient>,
    bytes: usize,
    flights: usize,
    journal: VecDeque<TransientChange>,
    journal_bytes: usize,
    floor: u64,
    probes: BTreeMap<String, u64>,
    probe_bytes: usize,
    next_probe: u64,
}
impl TransientSource {
    pub fn new(epoch: String, budget: TransientBudget) -> Result<Self, TransientError> {
        budget.validate()?;
        if !valid(&epoch) || epoch.len() > 128 {
            return Err(TransientError::Invalid);
        }
        Ok(Self {
            cursor: TransientCursor { epoch, sequence: 0 },
            budget,
            nodes: BTreeMap::new(),
            bytes: 0,
            flights: 0,
            journal: VecDeque::new(),
            journal_bytes: 0,
            floor: 0,
            probes: BTreeMap::new(),
            probe_bytes: 0,
            next_probe: 0,
        })
    }
    pub fn cursor(&self) -> &TransientCursor {
        &self.cursor
    }
    pub fn accounted_bytes(&self) -> usize {
        self.bytes
            + self.probe_bytes
            + self.journal_bytes
            + self
                .journal
                .capacity()
                .saturating_mul(std::mem::size_of::<TransientChange>())
    }
    pub fn snapshot(&self) -> TransientSnapshot {
        TransientSnapshot {
            through: self.cursor.clone(),
            nodes: self.nodes.values().cloned().collect(),
        }
    }
    fn change(&mut self, next: NodeTransient) -> Result<bool, TransientError> {
        next.validate(self.budget)?;
        let old = self.nodes.get(&next.id);
        if old == Some(&next) || old.is_none() && next.is_empty() {
            return Ok(false);
        }
        let count = self.nodes.len() - usize::from(old.is_some()) + usize::from(!next.is_empty());
        let flights = self.flights - old.map_or(0, |n| n.flights.len()) + next.flights.len();
        let bytes = self.bytes - old.map_or(0, NodeTransient::bytes)
            + if next.is_empty() { 0 } else { next.bytes() };
        if count > self.budget.nodes
            || flights > self.budget.flights
            || bytes.saturating_add(self.probe_bytes) > self.budget.bytes
        {
            return Err(TransientError::Budget);
        }
        let sequence = self
            .cursor
            .sequence
            .checked_add(1)
            .ok_or(TransientError::SequenceExhausted)?;
        let event = TransientChange {
            sequence,
            state: next.clone(),
        };
        let event_bytes = next.bytes();
        let ring_allowance = self
            .journal
            .capacity()
            .max(
                self.journal
                    .len()
                    .saturating_add(1)
                    .saturating_mul(2)
                    .max(4),
            )
            .saturating_mul(std::mem::size_of::<TransientChange>());
        // Reserve the input/event clone as part of the logical live+queue cap.
        if bytes
            .saturating_add(event_bytes)
            .saturating_add(self.probe_bytes)
            > self.budget.bytes
        {
            return Err(TransientError::Budget);
        }
        if next.is_empty() {
            self.nodes.remove(&next.id);
        } else {
            self.nodes.insert(next.id.clone(), next);
        }
        self.bytes = bytes;
        self.flights = flights;
        self.cursor.sequence = sequence;
        if self.journal.len() >= self.budget.journal_entries
            || self.journal_bytes.saturating_add(event_bytes) > self.budget.journal_bytes
            || bytes
                .saturating_add(self.probe_bytes)
                .saturating_add(self.journal_bytes)
                .saturating_add(event_bytes)
                .saturating_add(ring_allowance)
                > self.budget.bytes
        {
            // Release the old ring as well as its rows. Retaining its capacity
            // after reporting zero journal bytes would evade the logical cap.
            self.journal = VecDeque::new();
            self.journal_bytes = 0;
            self.floor = sequence;
        } else {
            self.journal.push_back(event);
            self.journal_bytes += event_bytes;
        }
        Ok(true)
    }
    pub fn start(&mut self, id: &str, version: &str, token: &str) -> Result<bool, TransientError> {
        if !valid(id) || !valid(version) || !valid(token) {
            return Err(TransientError::Invalid);
        }
        let mut next = self
            .nodes
            .get(id)
            .cloned()
            .unwrap_or_else(|| NodeTransient::empty(id.into()));
        let flight = Flight {
            version: version.into(),
            token: token.into(),
        };
        match next.flights.binary_search(&flight) {
            Ok(_) => Ok(false),
            Err(at) => {
                next.flights.insert(at, flight);
                self.change(next)
            }
        }
    }
    pub fn finish(&mut self, id: &str, version: &str, token: &str) -> Result<bool, TransientError> {
        if !valid(id) || !valid(version) || !valid(token) {
            return Err(TransientError::Invalid);
        }
        let Some(mut next) = self.nodes.get(id).cloned() else {
            return Ok(false);
        };
        let flight = Flight {
            version: version.into(),
            token: token.into(),
        };
        match next.flights.binary_search(&flight) {
            Err(_) => Ok(false),
            Ok(at) => {
                next.flights.remove(at);
                self.change(next)
            }
        }
    }
    pub fn bind_content(
        &mut self,
        id: &str,
        key: Option<ContentKey>,
    ) -> Result<bool, TransientError> {
        if !valid(id) || key.as_ref().is_some_and(|k| !k.valid()) {
            return Err(TransientError::Invalid);
        }
        let mut next = self
            .nodes
            .get(id)
            .cloned()
            .unwrap_or_else(|| NodeTransient::empty(id.into()));
        if next.content == key {
            return Ok(false);
        }
        next.content = key;
        next.availability = None;
        let changed = self.change(next)?;
        self.remove_probe(id);
        Ok(changed)
    }
    fn remove_probe(&mut self, id: &str) {
        if self.probes.remove(id).is_some() {
            self.probe_bytes -= Self::probe_charge(id);
        }
    }
    fn probe_charge(id: &str) -> usize {
        128usize.saturating_add(id.len().saturating_mul(4))
    }
    /// Call before starting each body inspection, while serializing content
    /// lifecycle changes. A new probe supersedes any older probe for this node.
    /// Starting a probe alone does not revoke an already verified body; report
    /// Missing (or unbind it) when deletion/corruption is actually observed.
    pub fn begin_verification(
        &mut self,
        id: &str,
        expected: &ContentKey,
    ) -> Result<Option<VerificationTicket>, TransientError> {
        if !valid(id) || !expected.valid() {
            return Err(TransientError::Invalid);
        }
        if self.nodes.get(id).and_then(|n| n.content.as_ref()) != Some(expected) {
            return Ok(None);
        }
        let token = self
            .next_probe
            .checked_add(1)
            .ok_or(TransientError::SequenceExhausted)?;
        let extra = if self.probes.contains_key(id) {
            0
        } else {
            Self::probe_charge(id)
        };
        if self.accounted_bytes().saturating_add(extra) > self.budget.bytes {
            return Err(TransientError::Budget);
        }
        let ticket = VerificationTicket {
            epoch: self.cursor.epoch.clone(),
            token,
            id: id.into(),
            content: expected.clone(),
        };
        self.probes.insert(id.into(), token);
        self.probe_bytes += extra;
        self.next_probe = token;
        Ok(Some(ticket))
    }
    /// A valid result consumes its ticket even when the observed state equals
    /// the preceding state. Failed publication keeps it retryable; stale, used,
    /// cancelled or foreign-epoch tickets cannot change state or sequence.
    pub fn record_availability(
        &mut self,
        ticket: &VerificationTicket,
        outcome: ContentResult,
    ) -> Result<bool, TransientError> {
        if ticket.epoch != self.cursor.epoch || self.probes.get(&ticket.id) != Some(&ticket.token) {
            return Ok(false);
        }
        let Some(mut next) = self.nodes.get(&ticket.id).cloned() else {
            return Ok(false);
        };
        if next.content.as_ref() != Some(&ticket.content) {
            return Ok(false);
        }
        next.availability = Some(outcome);
        let changed = self.change(next)?;
        self.remove_probe(&ticket.id);
        Ok(changed)
    }
    pub fn cancel_verification(&mut self, ticket: &VerificationTicket) -> bool {
        if ticket.epoch != self.cursor.epoch || self.probes.get(&ticket.id) != Some(&ticket.token) {
            return false;
        }
        self.remove_probe(&ticket.id);
        true
    }
    pub fn read(
        &self,
        after: &TransientCursor,
        limit: usize,
    ) -> Result<TransientRead, TransientError> {
        if limit == 0 || limit > self.budget.batch_entries {
            return Err(TransientError::Invalid);
        }
        if after.epoch != self.cursor.epoch
            || after.sequence < self.floor
            || after.sequence > self.cursor.sequence
        {
            return Ok(TransientRead::SnapshotRequired {
                through: self.cursor.clone(),
            });
        }
        let changes: Vec<_> = self
            .journal
            .iter()
            .filter(|c| c.sequence > after.sequence)
            .take(limit)
            .cloned()
            .collect();
        let through = TransientCursor {
            epoch: self.cursor.epoch.clone(),
            sequence: changes.last().map_or(after.sequence, |c| c.sequence),
        };
        Ok(TransientRead::Batch(TransientBatch {
            from: after.clone(),
            through,
            changes,
        }))
    }
    /// Call only after the consumer has atomically published through this
    /// cursor. Acknowledgement is not a new fact and never advances sequence.
    pub fn acknowledge(&mut self, through: &TransientCursor) -> Result<(), TransientError> {
        if through.epoch != self.cursor.epoch {
            return Err(TransientError::Epoch);
        }
        if through.sequence > self.cursor.sequence {
            return Err(TransientError::Discontinuous);
        }
        if through.sequence <= self.floor {
            return Ok(());
        }
        while self
            .journal
            .front()
            .is_some_and(|c| c.sequence <= through.sequence)
        {
            let event = self.journal.pop_front().expect("checked journal prefix");
            self.journal_bytes -= event.state.bytes();
        }
        self.floor = through.sequence;
        if self.journal.is_empty() {
            self.journal = VecDeque::new();
        }
        Ok(())
    }
}

struct Receipt {
    from: TransientCursor,
    through: TransientCursor,
    digest: [u8; 32],
}
/// Consumer-side state. It is committed only after the tree preflight succeeds.
pub(crate) struct TransientView {
    cursor: Option<TransientCursor>,
    nodes: BTreeMap<String, NodeTransient>,
    bytes: usize,
    flights: usize,
    receipts: VecDeque<Receipt>,
    budget: TransientBudget,
}
pub(crate) struct PreparedTransient {
    pub through: TransientCursor,
    pub reset: bool,
    pub duplicate: bool,
    pub bytes: usize,
    pub work_bytes: usize,
    flights: usize,
    updates: BTreeMap<String, Option<NodeTransient>>,
    revoked: BTreeMap<String, BTreeSet<ContentKey>>,
    receipt: Option<Receipt>,
}
impl PreparedTransient {
    pub fn revoked(&self, id: &str, key: &ContentKey) -> bool {
        self.revoked.get(id).is_some_and(|keys| keys.contains(key))
    }
    pub fn ids(&self) -> impl Iterator<Item = &String> {
        self.updates.keys()
    }
    pub fn state<'a>(&'a self, view: &'a TransientView, id: &str) -> Option<&'a NodeTransient> {
        match self.updates.get(id) {
            Some(n) => n.as_ref(),
            None if self.reset => None,
            None => view.nodes.get(id),
        }
    }
}
impl TransientView {
    pub fn new(budget: TransientBudget) -> Self {
        Self {
            cursor: None,
            nodes: BTreeMap::new(),
            bytes: 0,
            flights: 0,
            receipts: VecDeque::new(),
            budget,
        }
    }
    pub fn bytes(&self) -> usize {
        self.bytes
    }
    pub fn state(&self, id: &str) -> Option<&NodeTransient> {
        self.nodes.get(id)
    }
    fn prepare(
        &self,
        updates: BTreeMap<String, Option<NodeTransient>>,
        through: TransientCursor,
        reset: bool,
        receipt: Option<Receipt>,
        work_bytes: usize,
        revoked: BTreeMap<String, BTreeSet<ContentKey>>,
    ) -> Result<PreparedTransient, TransientError> {
        let (mut count, mut bytes, mut flights) = if reset {
            (0, 0, 0)
        } else {
            (self.nodes.len(), self.bytes, self.flights)
        };
        for (id, next) in &updates {
            if !reset {
                if let Some(old) = self.nodes.get(id) {
                    count -= 1;
                    bytes -= old.bytes();
                    flights -= old.flights.len();
                }
            }
            if let Some(next) = next {
                count += 1;
                bytes += next.bytes();
                flights += next.flights.len();
            }
        }
        // Retained facts and one incoming work batch have separate bounded
        // allowances. IndexOwner additionally charges their combined peak.
        if count > self.budget.nodes
            || bytes > self.budget.bytes
            || work_bytes > self.budget.bytes
            || flights > self.budget.flights
        {
            return Err(TransientError::Budget);
        }
        Ok(PreparedTransient {
            through,
            reset,
            duplicate: false,
            bytes,
            work_bytes,
            flights,
            updates,
            revoked,
            receipt,
        })
    }
    pub fn batch(&self, batch: TransientBatch) -> Result<PreparedTransient, TransientError> {
        let current = self
            .cursor
            .as_ref()
            .ok_or(TransientError::SnapshotRequired)?;
        if batch.from.epoch != current.epoch || batch.through.epoch != current.epoch {
            return Err(TransientError::Epoch);
        }
        if batch.changes.len() > self.budget.batch_entries {
            return Err(TransientError::Budget);
        }
        let mut work_bytes = batch
            .changes
            .iter()
            .fold(0usize, |n, c| n.saturating_add(c.state.bytes()));
        if work_bytes > self.budget.bytes {
            return Err(TransientError::Budget);
        }
        let mut hash = Sha256::new();
        hash.update(format!("{:?}{:?}", batch.from, batch.through));
        for change in &batch.changes {
            hash.update(format!("{change:?}"));
        }
        let digest: [u8; 32] = hash.finalize().into();
        if self
            .receipts
            .iter()
            .any(|r| r.from == batch.from && r.through == batch.through && r.digest == digest)
        {
            return Ok(PreparedTransient {
                through: current.clone(),
                reset: false,
                duplicate: true,
                bytes: self.bytes,
                work_bytes: 0,
                flights: self.flights,
                updates: BTreeMap::new(),
                revoked: BTreeMap::new(),
                receipt: None,
            });
        }
        if &batch.from != current || batch.through.sequence < batch.from.sequence {
            return Err(TransientError::Discontinuous);
        }
        let mut sequence = current.sequence;
        let mut updates: BTreeMap<String, Option<NodeTransient>> = BTreeMap::new();
        let mut revoked: BTreeMap<String, BTreeSet<ContentKey>> = BTreeMap::new();
        for change in batch.changes {
            sequence = sequence
                .checked_add(1)
                .ok_or(TransientError::SequenceExhausted)?;
            if change.sequence != sequence {
                return Err(TransientError::Discontinuous);
            }
            change.state.validate(self.budget)?;
            let state = change.state;
            // Folding full states must not erase an intervening explicit
            // unbind. A later mismatched binding cannot restore its proof.
            let previous = match updates.get(&state.id) {
                Some(previous) => previous.as_ref(),
                None => self.nodes.get(&state.id),
            };
            if state.content.is_none() {
                if let Some(key) = previous.and_then(|n| n.content.as_ref()) {
                    work_bytes = work_bytes.saturating_add(
                        256 + state.id.len().saturating_mul(4) + key.bytes().saturating_mul(4),
                    );
                    if work_bytes > self.budget.bytes {
                        return Err(TransientError::Budget);
                    }
                    revoked
                        .entry(state.id.clone())
                        .or_default()
                        .insert(key.clone());
                }
            }
            updates.insert(state.id.clone(), (!state.is_empty()).then_some(state));
        }
        if sequence != batch.through.sequence {
            return Err(TransientError::Discontinuous);
        }
        let receipt = Receipt {
            from: batch.from,
            through: batch.through.clone(),
            digest,
        };
        self.prepare(
            updates,
            batch.through,
            false,
            Some(receipt),
            work_bytes,
            revoked,
        )
    }
    pub fn snapshot(
        &self,
        snapshot: TransientSnapshot,
    ) -> Result<PreparedTransient, TransientError> {
        if !valid(&snapshot.through.epoch) || snapshot.through.epoch.len() > 128 {
            return Err(TransientError::Invalid);
        }
        if snapshot.nodes.len() > self.budget.nodes {
            return Err(TransientError::Budget);
        }
        if self.cursor.as_ref().is_some_and(|c| {
            c.epoch == snapshot.through.epoch && snapshot.through.sequence < c.sequence
        }) {
            return Err(TransientError::Discontinuous);
        }
        let mut updates = BTreeMap::new();
        let mut work_bytes = 0usize;
        for state in snapshot.nodes {
            state.validate(self.budget)?;
            work_bytes = work_bytes.saturating_add(state.bytes());
            if state.is_empty() || work_bytes > self.budget.bytes {
                return Err(TransientError::Budget);
            }
            if updates.insert(state.id.clone(), Some(state)).is_some() {
                return Err(TransientError::Invalid);
            }
        }
        if self.cursor.as_ref() == Some(&snapshot.through) {
            if updates.len() != self.nodes.len()
                || updates
                    .iter()
                    .any(|(id, n)| n.as_ref() != self.nodes.get(id))
            {
                return Err(TransientError::Discontinuous);
            }
            return Ok(PreparedTransient {
                through: snapshot.through,
                reset: false,
                duplicate: true,
                bytes: self.bytes,
                work_bytes: 0,
                flights: self.flights,
                updates: BTreeMap::new(),
                revoked: BTreeMap::new(),
                receipt: None,
            });
        }
        for id in self.nodes.keys() {
            if !updates.contains_key(id) {
                work_bytes = work_bytes.saturating_add(128 + id.len().saturating_mul(4));
                if work_bytes > self.budget.bytes {
                    return Err(TransientError::Budget);
                }
                updates.insert(id.clone(), None);
            }
        }
        self.prepare(
            updates,
            snapshot.through,
            true,
            None,
            work_bytes,
            BTreeMap::new(),
        )
    }
    pub fn commit(&mut self, plan: PreparedTransient) {
        if plan.duplicate {
            return;
        }
        if plan.reset {
            self.nodes.clear();
            self.receipts.clear();
        }
        for (id, state) in plan.updates {
            match state {
                Some(state) => {
                    self.nodes.insert(id, state);
                }
                None => {
                    self.nodes.remove(&id);
                }
            }
        }
        self.cursor = Some(plan.through);
        self.bytes = plan.bytes;
        self.flights = plan.flights;
        if let Some(r) = plan.receipt {
            self.receipts.push_back(r);
            if self.receipts.len() > 32 {
                self.receipts.pop_front();
            }
        }
    }
}

#[cfg(test)]
#[path = "status_transient_tests.rs"]
mod tests;
