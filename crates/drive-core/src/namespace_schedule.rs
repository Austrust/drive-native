//! Bounded, serial namespace scheduling. Unknown commits keep their intents;
//! only operations proved independent of every earlier unfinished intent pass.
use crate::{Error, Result, Store};
use rusqlite::OptionalExtension;
use serde::Serialize;
use std::{
    collections::{HashMap, HashSet},
    time::{Duration, Instant},
};

const LIMIT: usize = 256;
const BODY_BUDGET: usize = 4 * 1024 * 1024;
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
#[derive(Default)]
pub(crate) struct NamespaceRuntime {
    recovery: HashMap<String, Recovery>,
    observed: Option<(Instant, NamespaceScheduleSnapshot)>,
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
#[derive(Default)]
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
impl Store {
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
            self.namespace_runtime.lock().unwrap().observed = Some((Instant::now(), observation));
            return Ok(selection);
        }
        let headers=self.db.prepare(
            "SELECT id,kind,state,length(CAST(body AS BLOB)),id IN (SELECT operation FROM access_blocked_operations) FROM namespace_ops WHERE state NOT IN ('committed','superseded') ORDER BY seq LIMIT 257")?
            .query_map([],|r|Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?,r.get::<_,String>(2)?,r.get::<_,i64>(3)?,r.get::<_,bool>(4)?)))?
            .collect::<std::result::Result<Vec<_>,_>>()?;
        let header_count = headers.len();
        let mut observation = NamespaceScheduleSnapshot {
            truncated: headers.len() > LIMIT,
            ..Default::default()
        };
        let mut preceding = Vec::<Footprint>::new();
        let mut total = 0usize;
        let mut selection = None;
        let now = Instant::now();
        for (operation, kind, state, bytes, denied) in headers.into_iter().take(LIMIT) {
            observation.observed += 1;
            let bytes = usize::try_from(bytes)
                .map_err(|_| Error::State("negative namespace body size".into()))?;
            total = total.saturating_add(bytes);
            if bytes > 1024 * 1024 || total > BODY_BUDGET {
                observation.truncated = true;
                observation.unresolved_barrier = true;
                break;
            }
            let body = self.db.query_row(
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
            if !matches!(
                q.state.as_str(),
                "pending"
                    | "cleanup_pending"
                    | "uncertain"
                    | "cleanup_uncertain"
                    | "conflict"
                    | "inflight"
                    | "cleanup_inflight"
            ) {
                observation.unresolved_barrier = true;
                break;
            }
            // Keep at most one native mutation in flight, including callers
            // that re-enter a selector between mark_started and sending RPC.
            if matches!(q.state.as_str(), "inflight" | "cleanup_inflight") {
                selection = Some(q);
                observation.selected = true;
                break;
            }
            let shape = match footprint(self, &q) {
                Ok(shape) => shape,
                Err(Error::Db(rusqlite::Error::QueryReturnedNoRows)) => {
                    observation.unresolved_barrier = true;
                    break;
                }
                Err(error @ (Error::Db(_) | Error::Io(_))) => return Err(error),
                Err(_) => {
                    observation.unresolved_barrier = true;
                    break;
                }
            };
            let denied = denied || (matches!(q.state.as_str(), "pending" | "cleanup_pending")
                && crate::write_permissions::targets_denied(self,&crate::write_permissions::namespace_targets(&q.kind,&q.body)?)?);
            let dependent = preceding.iter().any(|prior| prior.overlaps(&shape));
            let deferred = uncertain(&q.state)
                && self
                    .namespace_runtime
                    .lock()
                    .unwrap()
                    .recovery
                    .get(&q.operation)
                    .is_some_and(|r| r.digest == crate::sha256(q.body.as_bytes()) && r.next > now);
            if dependent {
                observation.dependency_blocked += 1;
            }
            if denied {
                observation.permission_blocked += 1;
            }
            if deferred {
                observation.deferred_reconciliation += 1;
            }
            if !dependent
                && !denied
                && !deferred
                && matches!(
                    q.state.as_str(),
                    "pending" | "cleanup_pending" | "uncertain" | "cleanup_uncertain"
                )
            {
                selection = Some(q);
                observation.selected = true;
                break;
            }
            // Conflict, uncertain and permission-gated intents still constrain
            // every later operation. Shared read-only parents alone do not.
            preceding.push(shape);
        }
        observation.truncated |= observation.observed < header_count as u64;
        self.namespace_runtime.lock().unwrap().observed = Some((now, observation));
        Ok(selection)
    }
    pub fn next_namespace_operation(&self) -> Result<Option<NamespaceOperation>> {
        Ok(self.namespace_selection()?.map(|q| NamespaceOperation {
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
        let row: Option<(String, String)> = self
            .db
            .query_row(
                "SELECT body,state FROM namespace_ops WHERE id=?1",
                [operation],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()?;
        let mut runtime = self.namespace_runtime.lock().unwrap();
        let Some((body, state)) = row.filter(|(_, state)| uncertain(state)) else {
            runtime.recovery.remove(operation);
            return Ok(false);
        };
        let _ = state;
        let digest = crate::sha256(body.as_bytes());
        let attempts = runtime
            .recovery
            .get(operation)
            .filter(|r| r.digest == digest)
            .map_or(1, |r| r.attempts.saturating_add(1));
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
            "SELECT count(*),coalesce(sum(state='uncertain'),0) FROM operations WHERE state NOT IN ('committed','superseded')",
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
        })
    }
}
