//! Bounded, serial namespace scheduling. Unknown commits keep their intents;
//! only operations proved independent of every earlier unfinished intent pass.
use crate::{Error, Result, Store};
use rusqlite::{params, OptionalExtension};
use serde::Serialize;
use std::{
    collections::{HashMap, HashSet},
    time::{Duration, Instant},
};

const LIMIT: usize = 256;
/// #57-A: bounded continuation page. The fixed front window is always rescanned
/// from the journal head so pending-first priority never waits for the cursor;
/// the continuation page walks the rest of the journal across rounds so a
/// permanently blocked prefix cannot hide an independent suffix.
const CONTINUATION_LIMIT: usize = 256;
const BODY_BUDGET: usize = 4 * 1024 * 1024;
/// #57-A: upper bound on the cross-round dependency prefix kept in memory. The
/// cache is dropped with an explicit `scan_prefix_budget` diagnostic instead of
/// growing without bound; a dropped cache only costs one extra full pass.
const SCAN_CACHE_LIMIT: usize = 8192;
/// #57-C: declared maximum time an independent pending stream may defer a due
/// read-only reconcile before the aging quota gives it the serial lane once.
const RECONCILE_AGING_BUDGET: Duration = Duration::from_secs(10);
/// #57: consecutive dispatch rounds without a pending attempt that raise the
/// structured `namespace.pending_starved` diagnostic.
const PENDING_STARVED_ROUNDS: u32 = 3;
#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NamespaceRecoveryObservation {
    CommitUnconfirmed,
    TransportUnavailable,
}
#[derive(Clone)]
struct Recovery {
    digest: String,
    attempts: u64,
    at: Instant,
    next: Instant,
    outcome: NamespaceRecoveryObservation,
}
/// #57 starvation accounting for the single serial dispatch lane. Counted by
/// [`Store::dispatch_namespace_operation`] only, so mutation gates that re-enter
/// the selector between `mark_started` and the RPC do not inflate the counter.
#[derive(Default)]
struct DispatchRuntime {
    rounds: u64,
    consecutive_without_pending: u32,
    pending_starved_events: u64,
    last_pending_attempt: Option<Instant>,
}
/// #57-A: why a bounded scan window stopped before exhausting the journal.
/// Only `Window` (and the resumable body budget) can make progress next round;
/// the other reasons name a row that cannot be crossed safely and is reported
/// as an unresolved barrier for the operator instead of being skipped.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ScanStop {
    Window,
    BodyBudget,
    OversizedBody,
    InvalidBody,
    UnresolvedState,
    PrefixBudget,
}
impl ScanStop {
    fn reason(self) -> &'static str {
        match self {
            Self::Window => "window_limit",
            Self::BodyBudget => "body_budget",
            Self::OversizedBody => "oversized_body",
            Self::InvalidBody => "invalid_body",
            Self::UnresolvedState => "unresolved_state",
            Self::PrefixBudget => "scan_prefix_budget",
        }
    }
}
/// One earlier unfinished intent's dependency footprint, cached for the
/// continuation pages. `digest` invalidates the footprint if a body rewrite
/// changes it (replace provenance/rebase paths do this in place).
#[derive(Clone)]
struct ScanIntent {
    seq: i64,
    operation: String,
    state: String,
    digest: String,
    footprint: Footprint,
}
/// Compacted union of every earlier intent before the continuation cursor.
///
/// Checking a candidate against the union is equivalent to pairwise overlap
/// against each earlier intent: the overlap relation is exactly
/// `candidate.writes ∩ (earlier.writes ∪ earlier.reads)` plus
/// `candidate.reads ∩ earlier.writes` plus path prefix intersection. Nothing is
/// skipped, so the dispatch ordering guarantees of the one-shot scan are kept.
#[derive(Default)]
struct Barrier {
    writes: HashSet<String>,
    reads: HashSet<String>,
    paths: Vec<String>,
}
impl Barrier {
    fn rebuild(intents: &[ScanIntent]) -> Self {
        let mut barrier = Self::default();
        for intent in intents {
            barrier.add(&intent.footprint);
        }
        barrier
    }
    fn add(&mut self, footprint: &Footprint) {
        self.writes.extend(footprint.writes.iter().cloned());
        self.reads.extend(footprint.reads.iter().cloned());
        self.paths.extend(footprint.paths.iter().cloned());
    }
    fn overlaps(&self, footprint: &Footprint) -> bool {
        footprint
            .writes
            .iter()
            .any(|id| self.writes.contains(id) || self.reads.contains(id))
            || footprint
                .reads
                .iter()
                .any(|id| self.writes.contains(id))
            || self.paths.iter().any(|prior| {
                footprint
                    .paths
                    .iter()
                    .any(|path| paths_overlap(prior, path))
            })
    }
}
fn paths_overlap(a: &str, b: &str) -> bool {
    a == b
        || a.strip_prefix(b).is_some_and(|rest| rest.starts_with('/'))
        || b.strip_prefix(a).is_some_and(|rest| rest.starts_with('/'))
}
/// #57-A: resumable continuation progress. `intents` holds every nonterminal
/// intent with `seq < cursor`, so a later page can prove a candidate independent
/// of all of them without ever skipping an earlier intent.
#[derive(Default)]
struct ScanProgress {
    cursor: i64,
    intents: Vec<ScanIntent>,
}
/// #57-C: aging bookkeeping for the single serial lane. Pending-first is kept,
/// but a due read-only reconcile that keeps losing the lane is given a bounded
/// service quota instead of starving (or reverting to FIFO).
#[derive(Default)]
struct ReconcileAging {
    due_since: Option<(String, Instant)>,
    last_attempt: Option<Instant>,
    attempts: u64,
    aged_overrides: u64,
    max_wait: Duration,
    budget: Option<Duration>,
}
#[derive(Default)]
pub(crate) struct NamespaceRuntime {
    recovery: HashMap<String, Recovery>,
    observed: Option<(Instant, NamespaceScheduleSnapshot)>,
    dispatch: DispatchRuntime,
    scan: Option<ScanProgress>,
    scan_cycles: u64,
    aging: ReconcileAging,
}
#[derive(Clone, Debug, Default, Serialize)]
pub struct NamespaceScheduleSnapshot {
    pub observed: u64,
    pub truncated: bool,
    pub dependency_blocked: u64,
    pub deferred_reconciliation: u64,
    pub permission_blocked: u64,
    pub unresolved_barrier: bool,
    pub selected: bool,
    /// #57 additive: journal state of the selected operation (`pending`,
    /// `uncertain`, ...), so a dispatch round is explainable from status alone.
    pub selected_state: Option<String>,
    /// #57 additive: independent `pending` candidates observed while scanning.
    /// Zero with `selected_state = "uncertain"` is the starvation signature.
    pub eligible_pending: u64,
    /// #57-A additive: journal rows examined by the front window + continuation
    /// page in this round.
    pub scanned: u64,
    /// #57-A additive: continuation resume cursor (first unscanned journal seq).
    /// Zero when the next round starts a fresh cycle from the head.
    pub scan_cursor: i64,
    /// #57-A additive: completed full scan cycles in this process.
    pub scan_cycles: u64,
    /// #57-A additive: the whole nonterminal journal was examined this round.
    pub scan_complete: bool,
    /// #57-A additive: why the scan stopped before exhausting the queue
    /// (`window_limit` / `body_budget` / `oversized_body` / `invalid_body` /
    /// `unresolved_state` / `scan_prefix_budget`). `None` means no truncation.
    pub truncation_reason: Option<String>,
    /// #57-A additive: operation whose unreadable/oversized body or unknown
    /// state is a hard barrier; it names the recovery entry point.
    pub barrier_operation: Option<String>,
    /// #57-A/#57-C additive: independent pending rows that were executable but
    /// lost this round's slot to the reconcile aging quota.
    pub executable_unscheduled: u64,
    /// #57-C additive: how long the oldest due reconcile has waited when the
    /// selection was taken (zero while it is still waiting in later rounds).
    pub reconcile_due_age_ms: Option<u64>,
    /// #57-C additive: read-only reconcile attempts accounted in-process.
    pub reconcile_attempts: u64,
    /// #57-C additive: times the aging quota gave the lane to a waiting
    /// reconcile instead of the independent pending candidate.
    pub reconcile_aged_overrides: u64,
}
#[derive(Clone, Debug, Serialize)]
pub struct QueueHealth {
    pub namespace_uncertain: u64,
    pub upload_uncertain: u64,
    pub namespace_unfinished: u64,
    pub upload_unfinished: u64,
    pub schedule: Option<NamespaceScheduleSnapshot>,
    pub schedule_age_ms: Option<u64>,
    pub recovery_attempts_observed: u64,
    pub next_recovery_ms: Option<u64>,
    pub last_recovery_outcome: Option<NamespaceRecoveryObservation>,
    /// #57 additive: age of the most recent dispatched `pending` mutation.
    pub pending_last_attempt_age_ms: Option<u64>,
    /// #57 additive: `pending` work existed but was not attempted for
    /// [`PENDING_STARVED_ROUNDS`] consecutive dispatch rounds.
    pub pending_starved: bool,
    /// #57 additive: number of completed starvation streaks.
    pub pending_starved_events: u64,
    /// #57 additive: current consecutive rounds without a pending attempt.
    pub consecutive_pending_starved_rounds: u32,
    /// #57 additive: structured diagnostic name while starved
    /// (`namespace.pending_starved`), `None` otherwise.
    pub last_starvation_event: Option<String>,
    /// #57 additive: dispatch rounds accounted in this process.
    pub dispatch_rounds: u64,
    /// #57-C additive: age of the most recent read-only reconcile attempt.
    pub reconcile_last_attempt_age_ms: Option<u64>,
    /// #57-C additive: current wait of a due reconcile that is still deferred.
    pub reconcile_due_age_ms: Option<u64>,
    /// #57-C additive: longest observed deferral of a due reconcile before the
    /// lane was granted; bounded by the aging budget plus one attempt budget.
    pub reconcile_max_wait_ms: u64,
    /// #57-C additive: read-only reconcile attempts accounted in-process.
    pub reconcile_attempts: u64,
    /// #57-C additive: aging-quota lane grants.
    pub reconcile_aged_overrides: u64,
    /// #57-C additive: declared reconcile aging budget in milliseconds.
    pub reconcile_aging_budget_ms: u64,
}
#[derive(Clone)]
pub(crate) struct NamespaceSelection {
    pub operation: String,
    pub kind: String,
    pub state: String,
    pub body: String,
}
#[derive(Clone, Debug)]
pub struct NamespaceOperation {
    pub operation: String,
    pub kind: String,
    pub state: String,
}
#[derive(Clone, Default)]
struct Footprint {
    writes: HashSet<String>,
    reads: HashSet<String>,
    paths: Vec<String>,
}
impl Footprint {
    fn overlaps(&self, other: &Self) -> bool {
        !self.writes.is_disjoint(&other.writes)
            || !self.writes.is_disjoint(&other.reads)
            || !self.reads.is_disjoint(&other.writes)
            || self.paths.iter().any(|a| {
                other.paths.iter().any(|b| {
                    a == b
                        || a.strip_prefix(b).is_some_and(|rest| rest.starts_with('/'))
                        || b.strip_prefix(a).is_some_and(|rest| rest.starts_with('/'))
                })
            })
    }
    fn path(&mut self, path: &str) -> Result<()> {
        if path.is_empty()
            || path.len() > 16384
            || path.starts_with('/')
            || path.contains('\0')
            || path.split('/').any(|part| matches!(part, "" | "." | ".."))
        {
            return Err(Error::State("invalid namespace dependency path".into()));
        }
        self.paths.push(path.into());
        Ok(())
    }
    fn write(&mut self, store: &Store, id: &str) -> Result<()> {
        if id.is_empty() {
            return Err(Error::State("empty namespace identity".into()));
        }
        self.writes.insert(id.into());
        let alias: Option<String> = store
            .db
            .query_row(
                "SELECT remote FROM remote_aliases WHERE node=?1 OR remote=?1",
                [id],
                |r| r.get(0),
            )
            .optional()?;
        if let Some(alias) = alias {
            self.writes.insert(alias);
        }
        Ok(())
    }
    fn ancestry(&mut self, store: &Store, first: Option<&str>) -> Result<()> {
        let mut next = first.map(str::to_owned);
        let mut seen = HashSet::new();
        while let Some(id) = next {
            if seen.len() >= 129 || !seen.insert(id.clone()) {
                return Err(Error::State("invalid namespace dependency ancestry".into()));
            }
            self.reads.insert(id.clone());
            let alias: Option<String> = store
                .db
                .query_row(
                    "SELECT remote FROM remote_aliases WHERE node=?1 OR remote=?1",
                    [&id],
                    |r| r.get(0),
                )
                .optional()?;
            if let Some(alias) = alias {
                self.reads.insert(alias);
            }
            next = store
                .db
                .query_row("SELECT parent FROM nodes WHERE id=?1", [&id], |r| r.get(0))?;
        }
        Ok(())
    }
}
fn footprint(store: &Store, q: &NamespaceSelection) -> Result<Footprint> {
    let mut f = Footprint::default();
    match q.kind.as_str() {
        "mkdir" => {
            let j: crate::directories::DirectoryJob = serde_json::from_str(&q.body)?;
            if j.operation != q.operation {
                return Err(Error::State("namespace operation identity mismatch".into()));
            }
            f.write(store, &j.node.id)?;
            f.ancestry(store, Some(&j.parent.id))?;
            f.path(&j.path)?;
        }
        "rename" => {
            let j: crate::namespace::RenameJob = serde_json::from_str(&q.body)?;
            if j.operation != q.operation {
                return Err(Error::State("namespace operation identity mismatch".into()));
            }
            f.write(store, &j.node.id)?;
            f.ancestry(store, j.node.parent.as_deref())?;
            f.ancestry(store, Some(&j.parent_node.id))?;
            f.path(&j.source)?;
            f.path(&j.destination)?;
        }
        "remove" => {
            let j: crate::removals::RemoveJob = serde_json::from_str(&q.body)?;
            if j.operation != q.operation {
                return Err(Error::State("namespace operation identity mismatch".into()));
            }
            f.write(store, &j.node.id)?;
            f.ancestry(store, j.node.parent.as_deref())?;
            f.path(&j.path)?;
        }
        "replace" => {
            let j: crate::replacements::ReplaceJob = serde_json::from_str(&q.body)?;
            if j.operation != q.operation {
                return Err(Error::State("namespace operation identity mismatch".into()));
            }
            f.write(store, &j.source.id)?;
            f.write(store, &j.target.id)?;
            for id in [
                j.source_remote.as_deref(),
                j.target_remote.as_deref(),
                j.cleanup_node.as_deref(),
            ]
            .into_iter()
            .flatten()
            {
                f.write(store, id)?;
            }
            f.ancestry(store, j.source.parent.as_deref())?;
            f.ancestry(store, j.target.parent.as_deref())?;
            f.path(&j.source_path)?;
            f.path(&j.destination)?;
        }
        _ => return Err(Error::State("unsupported namespace dependency kind".into())),
    }
    Ok(f)
}
fn uncertain(state: &str) -> bool {
    matches!(state, "uncertain" | "cleanup_uncertain")
}
/// #57-A: one bounded window scan's outcome.
struct WindowScan {
    pending: Option<NamespaceSelection>,
    pending_seq: Option<i64>,
    inflight: Option<NamespaceSelection>,
    reconcile: Option<NamespaceSelection>,
    /// First journal sequence not yet scanned by this window (resume cursor).
    next_seq: i64,
    stop: Option<ScanStop>,
    exhausted: bool,
    /// Operation whose body/state stopped the scan and cannot be crossed.
    barrier_operation: Option<String>,
}
fn millis(duration: Duration) -> u64 {
    duration.as_millis().min(u128::from(u64::MAX)) as u64
}
impl Store {
    /// #57-A: scan one bounded journal window and report what it found.
    ///
    /// `barrier` must already contain every earlier unfinished intent outside
    /// this window; `intents` receives the window's intents so the caller can
    /// keep a resumable prefix cache. Dependency safety is unchanged from the
    /// previous one-shot scan: a candidate is only executable when its write
    /// set, read set and paths do not overlap any earlier intent's sets.
    #[allow(clippy::too_many_arguments)]
    fn namespace_scan_window(
        &self,
        start: i64,
        limit: usize,
        body_budget: usize,
        barrier: &mut Barrier,
        intents: &mut Vec<ScanIntent>,
        observation: &mut NamespaceScheduleSnapshot,
        now: Instant,
    ) -> Result<WindowScan> {
        let mut scan = WindowScan {
            pending: None,
            pending_seq: None,
            inflight: None,
            reconcile: None,
            next_seq: start,
            stop: None,
            exhausted: false,
            barrier_operation: None,
        };
        let headers = self
            .db
            .prepare(
                "SELECT seq,id,kind,state,length(CAST(body AS BLOB)),id IN (SELECT operation FROM access_blocked_operations) FROM namespace_ops WHERE state NOT IN ('committed','superseded') AND seq>=?1 ORDER BY seq LIMIT ?2",
            )?
            .query_map(
                params![
                    start,
                    i64::try_from(limit.saturating_add(1)).unwrap_or(i64::MAX)
                ],
                |r| {
                    Ok((
                        r.get::<_, i64>(0)?,
                        r.get::<_, String>(1)?,
                        r.get::<_, String>(2)?,
                        r.get::<_, String>(3)?,
                        r.get::<_, i64>(4)?,
                        r.get::<_, bool>(5)?,
                    ))
                },
            )?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        if headers.len() > limit {
            scan.stop = Some(ScanStop::Window);
        }
        let mut total = 0usize;
        for (seq, operation, kind, state, bytes, denied_header) in headers.into_iter().take(limit) {
            let bytes = usize::try_from(bytes)
                .map_err(|_| Error::State("negative namespace body size".into()))?;
            total = total.saturating_add(bytes);
            if bytes > 1024 * 1024 {
                // A body this large cannot be parsed inside the round budget;
                // it stays a hard barrier instead of being skipped, because a
                // later intent may depend on an intent that was never read.
                scan.stop = Some(ScanStop::OversizedBody);
                scan.next_seq = seq;
                scan.barrier_operation = Some(operation);
                break;
            }
            if total > body_budget {
                // Bounded per-round body work: the unscanned row is the resume
                // cursor, so the next round continues exactly here.
                scan.stop = Some(ScanStop::BodyBudget);
                scan.next_seq = seq;
                break;
            }
            if !matches!(
                state.as_str(),
                "pending"
                    | "cleanup_pending"
                    | "uncertain"
                    | "cleanup_uncertain"
                    | "conflict"
                    | "inflight"
                    | "cleanup_inflight"
            ) {
                scan.stop = Some(ScanStop::UnresolvedState);
                scan.next_seq = seq;
                scan.barrier_operation = Some(operation);
                break;
            }
            scan.next_seq = seq.saturating_add(1);
            observation.scanned = observation.scanned.saturating_add(1);
            let body: String = self.db.query_row(
                "SELECT body FROM namespace_ops WHERE id=?1",
                [&operation],
                |r| r.get(0),
            )?;
            let q = NamespaceSelection {
                operation,
                kind,
                state,
                body,
            };
            // Keep at most one native mutation in flight, including callers
            // that re-enter a selector between mark_started and sending RPC.
            if matches!(q.state.as_str(), "inflight" | "cleanup_inflight") {
                scan.inflight = Some(q);
                return Ok(scan);
            }
            let shape = match footprint(self, &q) {
                Ok(shape) => shape,
                Err(Error::Db(rusqlite::Error::QueryReturnedNoRows)) => {
                    scan.stop = Some(ScanStop::InvalidBody);
                    scan.next_seq = seq;
                    scan.barrier_operation = Some(q.operation);
                    break;
                }
                Err(error @ (Error::Db(_) | Error::Io(_))) => return Err(error),
                Err(_) => {
                    scan.stop = Some(ScanStop::InvalidBody);
                    scan.next_seq = seq;
                    scan.barrier_operation = Some(q.operation);
                    break;
                }
            };
            let digest = crate::sha256(q.body.as_bytes());
            let denied = denied_header
                || (matches!(q.state.as_str(), "pending" | "cleanup_pending")
                    && crate::write_permissions::targets_denied(
                        self,
                        &crate::write_permissions::namespace_targets(&q.kind, &q.body)?,
                    )?);
            let dependent = barrier.overlaps(&shape);
            let deferred = uncertain(&q.state)
                && self
                    .namespace_runtime
                    .lock()
                    .unwrap()
                    .recovery
                    .get(&q.operation)
                    .is_some_and(|r| r.digest == digest && r.next > now);
            if dependent {
                observation.dependency_blocked =
                    observation.dependency_blocked.saturating_add(1);
            }
            if denied {
                observation.permission_blocked =
                    observation.permission_blocked.saturating_add(1);
            }
            if deferred {
                observation.deferred_reconciliation =
                    observation.deferred_reconciliation.saturating_add(1);
            }
            if !dependent && !denied && !deferred {
                match q.state.as_str() {
                    // #57 pending-first: an independent mutation must never
                    // queue behind a read-only reconcile, so it wins the
                    // dispatch slot immediately.
                    "pending" | "cleanup_pending" => {
                        observation.eligible_pending =
                            observation.eligible_pending.saturating_add(1);
                        scan.pending_seq = Some(seq);
                        scan.pending = Some(q);
                        return Ok(scan);
                    }
                    "uncertain" | "cleanup_uncertain" => {
                        if scan.reconcile.is_none() {
                            scan.reconcile = Some(q.clone());
                        }
                    }
                    _ => {}
                }
            }
            // Every scanned intent constrains later candidates, including
            // conflicts, deferred reconciles and permission-gated mutations.
            barrier.add(&shape);
            intents.push(ScanIntent {
                seq,
                operation: q.operation.clone(),
                state: q.state.clone(),
                digest,
                footprint: shape,
            });
        }
        scan.exhausted = scan.stop.is_none();
        Ok(scan)
    }
    /// #57-A: merge the freshly scanned head with the cached continuation
    /// prefix, re-reading the cached middle so a committed row is pruned and a
    /// rewritten body re-parses instead of leaving a stale dependency footprint.
    #[allow(clippy::too_many_arguments)]
    fn namespace_resync_prefix(
        &self,
        cache: &mut ScanProgress,
        fresh_head: Vec<ScanIntent>,
        head_end: i64,
        observation: &mut NamespaceScheduleSnapshot,
    ) -> Result<Option<(ScanStop, Option<String>)>> {
        if head_end >= cache.cursor {
            // The fresh head covered the whole cached prefix.
            cache.intents = fresh_head;
            cache.cursor = head_end;
            return Ok(None);
        }
        let middle_start = head_end;
        let middle_end = cache.cursor;
        let mut synced = fresh_head;
        let mut cached: HashMap<i64, ScanIntent> = std::mem::take(&mut cache.intents)
            .into_iter()
            .filter(|intent| intent.seq >= middle_start && intent.seq < middle_end)
            .map(|intent| (intent.seq, intent))
            .collect();
        let rows = self
            .db
            .prepare(
                "SELECT seq,id,kind,state,body FROM namespace_ops WHERE state NOT IN ('committed','superseded') AND seq>=?1 AND seq<?2 ORDER BY seq",
            )?
            .query_map(params![middle_start, middle_end], |r| {
                Ok((
                    r.get::<_, i64>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, String>(3)?,
                    r.get::<_, String>(4)?,
                ))
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        for (seq, operation, kind, state, body) in rows {
            let digest = crate::sha256(body.as_bytes());
            let footprint = match cached.remove(&seq) {
                Some(intent) if intent.digest == digest => intent.footprint,
                _ => {
                    let q = NamespaceSelection {
                        operation: operation.clone(),
                        kind,
                        state: state.clone(),
                        body,
                    };
                    match footprint(self, &q) {
                        Ok(footprint) => footprint,
                        Err(error @ (Error::Db(_) | Error::Io(_))) => return Err(error),
                        Err(_) => {
                            return Ok(Some((ScanStop::InvalidBody, Some(operation))));
                        }
                    }
                }
            };
            synced.push(ScanIntent {
                seq,
                operation,
                state,
                digest,
                footprint,
            });
        }
        synced.sort_by_key(|intent| intent.seq);
        cache.intents = synced;
        cache.cursor = middle_end;
        Ok(None)
    }

    pub(crate) fn namespace_selection(&self) -> Result<Option<NamespaceSelection>> {
        // An in-flight request anywhere in the journal owns the serial lane.
        // This also preserves the mark_started -> fresh getter handoff.
        let inflight:Option<(String,String,String,i64,bool)>=self.db.query_row(
            "SELECT id,kind,state,length(CAST(body AS BLOB)),id IN (SELECT operation FROM access_blocked_operations) FROM namespace_ops WHERE state NOT IN ('committed','superseded') AND state IN ('inflight','cleanup_inflight') ORDER BY seq LIMIT 1",
            [],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?))).optional()?;
        if let Some((operation, kind, state, bytes, denied)) = inflight {
            let mut observation = NamespaceScheduleSnapshot {
                observed: 1,
                truncated: true,
                unresolved_barrier: true,
                ..Default::default()
            };
            let selection = if denied || !(0..=1_048_576).contains(&bytes) {
                None
            } else {
                let body = self.db.query_row(
                    "SELECT body FROM namespace_ops WHERE id=?1",
                    [&operation],
                    |r| r.get(0),
                )?;
                observation.selected = true;
                Some(NamespaceSelection {
                    operation,
                    kind,
                    state,
                    body,
                })
            };
            observation.selected_state = selection.as_ref().map(|q| q.state.clone());
            self.namespace_runtime.lock().unwrap().observed = Some((Instant::now(), observation));
            return Ok(selection);
        }
        let now = Instant::now();
        let mut progress = self.namespace_runtime.lock().unwrap().scan.take();
        let mut observation = NamespaceScheduleSnapshot::default();
        let outcome = self.namespace_selection_paged(&mut observation, &mut progress, now);
        let mut runtime = self.namespace_runtime.lock().unwrap();
        runtime.scan = progress;
        observation.observed = observation.scanned;
        observation.scan_cycles = runtime.scan_cycles;
        runtime.observed = Some((now, observation));
        outcome
    }

    /// #57-A/#57-C: bounded front window plus resumable continuation pages.
    fn namespace_selection_paged(
        &self,
        observation: &mut NamespaceScheduleSnapshot,
        progress: &mut Option<ScanProgress>,
        now: Instant,
    ) -> Result<Option<NamespaceSelection>> {
        // The fixed front window is rescanned from the journal head every round:
        // pending-first priority must never wait for the continuation cursor.
        let mut front_barrier = Barrier::default();
        let mut front_intents = Vec::new();
        let front = self.namespace_scan_window(
            0,
            LIMIT,
            BODY_BUDGET,
            &mut front_barrier,
            &mut front_intents,
            observation,
            now,
        )?;
        if let Some(selection) = front.inflight {
            observation.truncated = true;
            observation.unresolved_barrier = true;
            observation.selected = true;
            observation.selected_state = Some(selection.state.clone());
            observation.scan_cursor = progress.as_ref().map_or(0, |p| p.cursor);
            return Ok(Some(selection));
        }
        if front.stop.is_some() {
            observation.truncated = true;
        }
        if let Some(selection) = front.pending {
            if observation.truncation_reason.is_none() {
                observation.truncation_reason = front.stop.map(|stop| stop.reason().to_owned());
            }
            return Ok(self.namespace_finish_selection(
                observation,
                progress,
                now,
                Some(selection),
                front.reconcile,
            ));
        }
        let front_hard = matches!(
            front.stop,
            Some(ScanStop::OversizedBody | ScanStop::InvalidBody | ScanStop::UnresolvedState)
        );
        if front_hard {
            observation.unresolved_barrier = true;
            observation.truncation_reason = front.stop.map(|stop| stop.reason().to_owned());
            observation.barrier_operation = front.barrier_operation.clone();
        }
        if front.exhausted {
            // The whole nonterminal journal fitted in the front window: the
            // continuation cache is not needed.
            observation.scan_complete = true;
            let mut runtime = self.namespace_runtime.lock().unwrap();
            runtime.scan_cycles = runtime.scan_cycles.saturating_add(1);
            drop(runtime);
            *progress = None;
            return Ok(self.namespace_finish_selection(
                observation,
                progress,
                now,
                None,
                front.reconcile,
            ));
        }
        // Merge the fresh head into the cached continuation prefix.
        let mut cache = progress.take().unwrap_or_default();
        if front_hard {
            // The head scan stopped on a row that cannot be crossed; reusing an
            // old cache would prove later candidates against a stale prefix.
            *progress = None;
            return Ok(self.namespace_finish_selection(
                observation,
                progress,
                now,
                None,
                front.reconcile,
            ));
        }
        if let Some((stop, operation)) =
            self.namespace_resync_prefix(&mut cache, front_intents, front.next_seq, observation)?
        {
            observation.truncated = true;
            observation.unresolved_barrier = true;
            observation.truncation_reason = Some(stop.reason().to_owned());
            observation.barrier_operation = operation;
            *progress = None;
            return Ok(self.namespace_finish_selection(
                observation,
                progress,
                now,
                None,
                front.reconcile,
            ));
        }
        if cache.intents.len() > SCAN_CACHE_LIMIT {
            observation.truncated = true;
            observation.unresolved_barrier = true;
            observation.truncation_reason = Some(ScanStop::PrefixBudget.reason().to_owned());
            *progress = None;
            return Ok(self.namespace_finish_selection(
                observation,
                progress,
                now,
                None,
                front.reconcile,
            ));
        }
        // One bounded continuation page beyond the cached prefix.
        let mut barrier = Barrier::rebuild(&cache.intents);
        let tail = self.namespace_scan_window(
            cache.cursor,
            CONTINUATION_LIMIT,
            BODY_BUDGET,
            &mut barrier,
            &mut cache.intents,
            observation,
            now,
        )?;
        if tail.stop.is_some() {
            observation.truncated = true;
            if observation.truncation_reason.is_none() {
                observation.truncation_reason = tail.stop.map(|stop| stop.reason().to_owned());
            }
        }
        let tail_hard = matches!(
            tail.stop,
            Some(ScanStop::OversizedBody | ScanStop::InvalidBody | ScanStop::UnresolvedState)
        );
        if tail_hard {
            observation.unresolved_barrier = true;
            observation.barrier_operation = tail.barrier_operation.clone();
        }
        if let Some(selection) = tail.inflight {
            observation.selected = true;
            observation.selected_state = Some(selection.state.clone());
            let cursor = cache.cursor;
            *progress = Some(cache);
            observation.scan_cursor = cursor;
            return Ok(Some(selection));
        }
        if let Some(seq) = tail.pending_seq {
            // Re-examine the candidate next round: the mutation gates may hand
            // the same intent back if the local ordering gate refuses it.
            cache.cursor = seq;
        } else {
            cache.cursor = tail.next_seq;
        }
        if tail.exhausted {
            // A full continuation cycle completed; the next round rescans the
            // head and starts a new cycle, so intents that became due are
            // reconsidered instead of being skipped forever.
            observation.scan_complete = true;
            let mut runtime = self.namespace_runtime.lock().unwrap();
            runtime.scan_cycles = runtime.scan_cycles.saturating_add(1);
            drop(runtime);
            *progress = None;
        } else if cache.intents.len() > SCAN_CACHE_LIMIT {
            observation.truncated = true;
            observation.unresolved_barrier = true;
            observation.truncation_reason = Some(ScanStop::PrefixBudget.reason().to_owned());
            *progress = None;
        } else {
            *progress = Some(cache);
        }
        let reconcile = front.reconcile.or(tail.reconcile);
        Ok(self.namespace_finish_selection(observation, progress, now, tail.pending, reconcile))
    }

    /// #57-C: pending-first with a bounded reconcile aging quota.
    ///
    /// Pending work still wins the single serial lane, but a due read-only
    /// reconcile that keeps losing it is promoted once its declared aging budget
    /// expires, so neither class can starve the other and the order does not
    /// revert to the old global FIFO.
    fn namespace_finish_selection(
        &self,
        observation: &mut NamespaceScheduleSnapshot,
        progress: &mut Option<ScanProgress>,
        now: Instant,
        pending: Option<NamespaceSelection>,
        reconcile: Option<NamespaceSelection>,
    ) -> Option<NamespaceSelection> {
        let mut runtime = self.namespace_runtime.lock().unwrap();
        let budget = runtime.aging.budget.unwrap_or(RECONCILE_AGING_BUDGET);
        if let Some(reconcile) = &reconcile {
            match &runtime.aging.due_since {
                Some((operation, _)) if operation == &reconcile.operation => {}
                _ => runtime.aging.due_since = Some((reconcile.operation.clone(), now)),
            }
        }
        let due_age = runtime
            .aging
            .due_since
            .as_ref()
            .filter(|(operation, _)| reconcile.as_ref().is_some_and(|r| &r.operation == operation))
            .map(|(_, since)| now.saturating_duration_since(*since));
        observation.reconcile_due_age_ms = due_age.map(millis);
        let aged = pending.is_some() && due_age.is_some_and(|age| age >= budget);
        let selection = if aged {
            observation.executable_unscheduled =
                observation.executable_unscheduled.saturating_add(1);
            runtime.aging.aged_overrides = runtime.aging.aged_overrides.saturating_add(1);
            reconcile.clone()
        } else {
            pending.or(reconcile)
        };
        if selection.as_ref().is_some_and(|q| uncertain(&q.state)) {
            if let Some((_, since)) = runtime.aging.due_since.take() {
                let wait = now.saturating_duration_since(since);
                runtime.aging.max_wait = runtime.aging.max_wait.max(wait);
                observation.reconcile_due_age_ms = Some(millis(wait));
            }
            runtime.aging.attempts = runtime.aging.attempts.saturating_add(1);
            runtime.aging.last_attempt = Some(now);
        } else if selection.is_none() {
            runtime.aging.due_since = None;
        }
        observation.reconcile_attempts = runtime.aging.attempts;
        observation.reconcile_aged_overrides = runtime.aging.aged_overrides;
        observation.selected = selection.is_some();
        observation.selected_state = selection.as_ref().map(|q| q.state.clone());
        drop(runtime);
        observation.scan_cursor = progress.as_ref().map_or(0, |p| p.cursor);
        selection
    }

    /// #57-C: override the reconcile aging budget (default 10 s). Tests use a
    /// short budget; production keeps the declared default.
    pub fn set_namespace_reconcile_aging(&mut self, budget: Duration) {
        self.namespace_runtime.lock().unwrap().aging.budget = Some(budget);
    }
    /// #57-C test hook: simulate elapsed waiting for the oldest due reconcile
    /// without sleeping in a test. Moves its `due_since` backwards by `age`.
    pub fn age_namespace_reconcile_deadline(&mut self, age: Duration) {
        let mut runtime = self.namespace_runtime.lock().unwrap();
        if let Some((_, since)) = runtime.aging.due_since.as_mut() {
            *since = since.checked_sub(age).unwrap_or(*since);
        }
    }

    pub fn next_namespace_operation(&self) -> Result<Option<NamespaceOperation>> {
        Ok(self.namespace_selection()?.map(|q| NamespaceOperation {
            operation: q.operation,
            kind: q.kind,
            state: q.state,
        }))
    }
    /// One dispatch round of the serial namespace lane (#57).
    ///
    /// Selection itself is [`Store::namespace_selection`] (pending-first); this
    /// wrapper only accounts the round so starvation is observable without the
    /// mutation gates (which re-enter the selector between `mark_started` and the
    /// RPC) inflating the counter. The structured diagnostic is
    /// `namespace.pending_starved` and is exposed through
    /// [`Store::queue_health`].
    pub fn dispatch_namespace_operation(&mut self) -> Result<Option<NamespaceOperation>> {
        let selected = self.namespace_selection()?;
        let now = Instant::now();
        let pending_selected = selected
            .as_ref()
            .is_some_and(|q| matches!(q.state.as_str(), "pending" | "cleanup_pending"));
        let pending_exists: bool = self.db.query_row(
            "SELECT EXISTS(SELECT 1 FROM namespace_ops WHERE state IN ('pending','cleanup_pending'))",
            [],
            |r| r.get(0),
        )?;
        let mut runtime = self.namespace_runtime.lock().unwrap();
        let dispatch = &mut runtime.dispatch;
        dispatch.rounds = dispatch.rounds.saturating_add(1);
        if pending_selected {
            dispatch.consecutive_without_pending = 0;
            dispatch.last_pending_attempt = Some(now);
        } else if pending_exists {
            dispatch.consecutive_without_pending =
                dispatch.consecutive_without_pending.saturating_add(1);
            if dispatch.consecutive_without_pending == PENDING_STARVED_ROUNDS {
                dispatch.pending_starved_events = dispatch.pending_starved_events.saturating_add(1);
            }
        } else {
            // No pending work exists: the reconcile lane is not starving it.
            dispatch.consecutive_without_pending = 0;
        }
        Ok(selected.map(|q| NamespaceOperation {
            operation: q.operation,
            kind: q.kind,
            state: q.state,
        }))
    }
    /// Legacy journal inspection must retain conflicts and deferred intents.
    /// Runtime and mutation gates use scheduled_namespace_body_for_kind instead.
    pub(crate) fn namespace_body_for_kind(&self, kind: &str) -> Result<Option<(String, String)>> {
        let row: Option<(String,String,String)> = self.db.query_row(
            "SELECT kind,body,state FROM namespace_ops WHERE id NOT IN (SELECT operation FROM access_blocked_operations) AND state NOT IN ('committed','superseded') ORDER BY seq LIMIT 1",
            [], |r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).optional()?;
        Ok(row
            .filter(|(actual, _, _)| actual == kind)
            .map(|(_, body, state)| (body, state)))
    }
    pub(crate) fn scheduled_namespace_body_for_kind(
        &self,
        kind: &str,
    ) -> Result<Option<(String, String)>> {
        Ok(self
            .namespace_selection()?
            .filter(|q| q.kind == kind)
            .map(|q| (q.body, q.state)))
    }
    /// Observation/backoff is process-local; the journal's uncertain state and
    /// provenance remain durable. Restart can only retry read-only reconciliation.
    pub fn observe_namespace_unresolved(
        &mut self,
        operation: &str,
        outcome: NamespaceRecoveryObservation,
    ) -> Result<bool> {
        let row: Option<(String, String, i64)> = self
            .db
            .query_row(
                "SELECT body,state,attempts FROM namespace_ops WHERE id=?1",
                [operation],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .optional()?;
        let mut runtime = self.namespace_runtime.lock().unwrap();
        let Some((body, state, durable_attempts)) = row.filter(|(_, state, _)| uncertain(state))
        else {
            runtime.recovery.remove(operation);
            return Ok(false);
        };
        let _ = state;
        let durable_attempts = u64::try_from(durable_attempts).unwrap_or(0);
        let digest = crate::sha256(body.as_bytes());
        // #57: the durable journal counter is authoritative. A restart clears
        // the in-process map, so the next observation must continue from
        // `namespace_ops.attempts` instead of restarting the ladder at 1 (the
        // field DB would then look like a fresh linear recovery forever).
        let attempts = runtime
            .recovery
            .get(operation)
            .filter(|r| r.digest == digest)
            .map_or(durable_attempts.saturating_add(1), |r| {
                r.attempts.saturating_add(1)
            })
            .max(durable_attempts.saturating_add(1));
        let delay = match attempts {
            1 => 2,
            2 => 10,
            3 => 30,
            _ => 60,
        };
        let now = Instant::now();
        if runtime.recovery.len() >= LIMIT && !runtime.recovery.contains_key(operation) {
            if let Some(oldest) = runtime
                .recovery
                .iter()
                .min_by_key(|(_, r)| r.at)
                .map(|(id, _)| id.clone())
            {
                runtime.recovery.remove(&oldest);
            }
        }
        runtime.recovery.insert(
            operation.into(),
            Recovery {
                digest,
                attempts,
                at: now,
                next: now + Duration::from_secs(delay),
                outcome,
            },
        );
        // #9b TB2: persist the observed category and the *actual* in-memory
        // schedule so the status interface shows the same retry time the
        // scheduler uses. #57: the durable attempt counter now continues the
        // ladder across a restart instead of re-arming from 2s.
        let (category, last_error) = match outcome {
            NamespaceRecoveryObservation::TransportUnavailable => (
                crate::commit_outcome::CommitCategory::Transport,
                "namespace transport unavailable; reconciliation retried",
            ),
            NamespaceRecoveryObservation::CommitUnconfirmed => (
                crate::commit_outcome::CommitCategory::Protocol,
                "namespace commit outcome unconfirmed; reconciling without replay",
            ),
        };
        // #9 TB3: this observation runs in the same pass as the concrete
        // reconcile diagnosis written by the runtime branch ("unexpected reply
        // during rename reconcile", "transport lost during reconcile", ...).
        // Only fill in the generic reason when the row has no concrete one yet
        // (fresh row, or the `interrupted` placeholder written by the restart
        // backfill). The retry bookkeeping always advances.
        let next_retry_at =
            crate::commit_diagnostics::unix_now() + i64::try_from(delay).unwrap_or(i64::MAX);
        let changed = self.db.execute(
            "UPDATE namespace_ops SET outcome='unknown_retryable',
                    outcome_category=CASE
                        WHEN outcome IS NULL OR outcome_category='interrupted'
                        THEN ?2 ELSE outcome_category END,
                    last_error=CASE
                        WHEN outcome IS NULL OR outcome_category='interrupted'
                        THEN ?3 ELSE last_error END,
                    attempts=MAX(attempts,?4), next_retry_at=?5, updated_at=unixepoch()
              WHERE id=?1",
            params![
                operation,
                category.as_str(),
                last_error,
                i64::try_from(attempts).unwrap_or(i64::MAX),
                next_retry_at
            ],
        )?;
        if changed != 1 {
            return Err(Error::State(format!(
                "commit diagnostic target {operation} is missing from namespace_ops"
            )));
        }
        Ok(true)
    }
    pub fn clear_namespace_observation(&mut self, operation: &str) {
        self.namespace_runtime
            .lock()
            .unwrap()
            .recovery
            .remove(operation);
    }
    /// Explicit user recheck only expires process-local observation deadlines.
    /// The durable journal, origin, state and retry history stay unchanged.
    /// Selectors still enforce uncertain/read-only and dependency barriers.
    pub fn request_namespace_recheck(&self) {
        let now = Instant::now();
        for recovery in self.namespace_runtime.lock().unwrap().recovery.values_mut() {
            recovery.next = now;
        }
    }
    pub fn namespace_recovery_due(&self) -> Result<bool> {
        let runtime = self.namespace_runtime.lock().unwrap();
        let now = Instant::now();
        for (op, r) in &runtime.recovery {
            if r.next<=now && self.db.query_row(
                "SELECT EXISTS(SELECT 1 FROM namespace_ops WHERE id=?1 AND state IN ('uncertain','cleanup_uncertain'))",
                [op],|row|row.get::<_,bool>(0))? {return Ok(true);}
        }
        Ok(false)
    }
    /// Exact nonterminal counts plus a cached, explicitly partial schedule view.
    /// This never invokes dependency scheduling or a network operation.
    pub fn queue_health(&self) -> Result<QueueHealth> {
        let (namespace_unfinished,namespace_uncertain)=self.db.query_row(
            "SELECT count(*),coalesce(sum(state IN ('uncertain','cleanup_uncertain')),0) FROM namespace_ops WHERE state NOT IN ('committed','superseded')",
            [],|r|Ok((r.get::<_,i64>(0)?,r.get::<_,i64>(1)?)))?;
        let (upload_unfinished,upload_uncertain)=self.db.query_row(
            "SELECT count(*),coalesce(sum(state='uncertain'),0) FROM operations WHERE state NOT IN ('committed','superseded','abandoned')",
            [],|r|Ok((r.get::<_,i64>(0)?,r.get::<_,i64>(1)?)))?;
        let runtime = self.namespace_runtime.lock().unwrap();
        let now = Instant::now();
        let current=runtime.recovery.iter().filter_map(|(op,r)| {
            match self.db.query_row("SELECT EXISTS(SELECT 1 FROM namespace_ops WHERE id=?1 AND state IN ('uncertain','cleanup_uncertain'))",
                [op],|row|row.get::<_,bool>(0)) {Ok(true)=>Some(Ok(r)),Ok(false)=>None,Err(e)=>Some(Err(e))}
        }).collect::<std::result::Result<Vec<_>,_>>()?;
        Ok(QueueHealth {
            namespace_uncertain: namespace_uncertain as u64,
            upload_uncertain: upload_uncertain as u64,
            namespace_unfinished: namespace_unfinished as u64,
            upload_unfinished: upload_unfinished as u64,
            schedule: runtime.observed.as_ref().map(|(_, s)| s.clone()),
            schedule_age_ms: runtime
                .observed
                .as_ref()
                .map(|(at, _)| now.duration_since(*at).as_millis().min(u64::MAX as u128) as u64),
            recovery_attempts_observed: current
                .iter()
                .fold(0u64, |n, r| n.saturating_add(r.attempts)),
            next_recovery_ms: current
                .iter()
                .map(|r| {
                    r.next
                        .saturating_duration_since(now)
                        .as_millis()
                        .min(u64::MAX as u128) as u64
                })
                .min(),
            last_recovery_outcome: current.iter().max_by_key(|r| r.at).map(|r| r.outcome),
            pending_last_attempt_age_ms: runtime
                .dispatch
                .last_pending_attempt
                .map(|at| now.duration_since(at).as_millis().min(u64::MAX as u128) as u64),
            pending_starved: runtime.dispatch.consecutive_without_pending >= PENDING_STARVED_ROUNDS,
            pending_starved_events: runtime.dispatch.pending_starved_events,
            consecutive_pending_starved_rounds: runtime.dispatch.consecutive_without_pending,
            last_starvation_event: if runtime.dispatch.consecutive_without_pending
                >= PENDING_STARVED_ROUNDS
            {
                Some("namespace.pending_starved".into())
            } else {
                None
            },
            dispatch_rounds: runtime.dispatch.rounds,
            reconcile_last_attempt_age_ms: runtime
                .aging
                .last_attempt
                .map(|at| millis(now.saturating_duration_since(at))),
            reconcile_due_age_ms: runtime
                .aging
                .due_since
                .as_ref()
                .map(|(_, since)| millis(now.saturating_duration_since(*since))),
            reconcile_max_wait_ms: millis(runtime.aging.max_wait),
            reconcile_attempts: runtime.aging.attempts,
            reconcile_aged_overrides: runtime.aging.aged_overrides,
            reconcile_aging_budget_ms: millis(
                runtime.aging.budget.unwrap_or(RECONCILE_AGING_BUDGET),
            ),
        })
    }
}
