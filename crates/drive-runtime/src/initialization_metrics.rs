//! Process-lifetime, bounded observations. Never used to decide coverage or commit.
//! No remote identities, paths, cursor values, credentials or content are retained.
use serde::Serialize;
use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
    time::Instant,
};

pub type Observer = Arc<dyn Fn(sproto::diagnostics::Snapshot) + Send + Sync>;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Stage {
    RpcSeed,
    RpcRoot,
    RpcDirectory,
    RpcPull,
    PayloadDecode,
    StoreWait,
    RecoveryWait,
    Scheduling,
    DirectoryCommit,
    EventCollect,
    EventApply,
    RootCommit,
    FinalCheck,
    FinalCommit,
    QueueSample,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Counter {
    Resumed,
    Restarted,
    PullNoAdvance,
    PullPages,
    PullEvents,
    AppliedPages,
    AwaitingCoverage,
    DirectoryCommitted,
    DirectoryDenied,
    DirectoryRetry,
    StaleReceipt,
    CoverageRace,
    FenceAttempts,
    Checkpoints,
    ClaimPages,
    ClaimExamined,
    ClaimAncestorPromotions,
    ClaimCoalesced,
    ClaimBlockedLoading,
    ClaimPreviouslyClaimed,
    ClaimNew,
    ClaimScanYields,
}
#[derive(Clone, Default, Serialize)]
pub struct Total {
    pub count: u64,
    pub total_us: u64,
    pub max_us: u64,
}
#[derive(Clone, Serialize)]
pub struct InFlight {
    pub stage: Stage,
    pub elapsed_ms: u64,
}
#[derive(Clone, Serialize)]
pub struct CommandSample {
    pub sequence: u64,
    pub protocol_request_elapsed_us: u64,
    pub request: Option<sproto::diagnostics::RequestSample>,
    pub pages: u64,
    pub items: u64,
}
#[derive(Clone, Default, Serialize)]
pub struct Snapshot {
    pub queue: Option<drive_core::initialization_store::InitializationQueueSummary>,
    pub queue_sample_age_ms: Option<u64>,
    pub stages: Vec<(Stage, Total)>,
    pub counters: Vec<(Counter, u64)>,
    pub in_flight: Vec<InFlight>,
    pub max_in_flight: usize,
    pub native: sproto::diagnostics::Snapshot,
    pub native_commands: u64,
    pub native_merge_rejected: u64,
    pub recent_commands: VecDeque<CommandSample>,
}
#[derive(Default)]
struct State {
    snapshot: Snapshot,
    next: u64,
    active: Vec<(u64, Stage, Instant)>,
    queue_at: Option<Instant>,
}
#[derive(Default)]
pub struct Metrics {
    state: Mutex<State>,
}
pub struct Span<'a> {
    metrics: &'a Metrics,
    id: u64,
    stage: Stage,
    start: Instant,
}
impl Metrics {
    pub fn queue_due(&self) -> bool {
        self.state
            .lock()
            .unwrap()
            .queue_at
            .is_none_or(|time| time.elapsed().as_secs() >= 10)
    }
    pub fn queue(&self, queue: drive_core::initialization_store::InitializationQueueSummary) {
        let mut s = self.state.lock().unwrap();
        s.snapshot.queue = Some(queue);
        s.queue_at = Some(Instant::now());
    }
    pub fn span(&self, stage: Stage) -> Span<'_> {
        let start = Instant::now();
        let mut s = self.state.lock().unwrap();
        s.next = s.next.wrapping_add(1);
        let id = s.next;
        // Production has at most four reads and one local writer. Cap defensive
        // diagnostics storage independently of the number of remote directories.
        if s.active.len() < 64 {
            s.active.push((id, stage, start));
        }
        s.snapshot.max_in_flight = s.snapshot.max_in_flight.max(s.active.len());
        Span {
            metrics: self,
            id,
            stage,
            start,
        }
    }
    pub fn count(&self, key: Counter, n: u64) {
        let mut s = self.state.lock().unwrap();
        if let Some((_, count)) = s.snapshot.counters.iter_mut().find(|(k, _)| *k == key) {
            *count = count.saturating_add(n);
        } else {
            s.snapshot.counters.push((key, n));
        }
    }
    pub fn native(&self, snapshot: sproto::diagnostics::Snapshot) {
        let mut s = self.state.lock().unwrap();
        if !s.snapshot.native.merge(&snapshot) {
            s.snapshot.native_merge_rejected = s.snapshot.native_merge_rejected.saturating_add(1);
            return;
        }
        s.snapshot.native_commands = s.snapshot.native_commands.saturating_add(1);
        let sample = CommandSample {
            sequence: s.snapshot.native_commands,
            protocol_request_elapsed_us: snapshot
                .requests
                .iter()
                .fold(0u64, |sum, r| sum.saturating_add(r.elapsed_us)),
            request: snapshot
                .last_failure
                .clone()
                .or(snapshot.last_request.clone()),
            pages: snapshot.list_pages,
            items: snapshot.list_items,
        };
        if s.snapshot.recent_commands.len() == 64 {
            s.snapshot.recent_commands.pop_front();
        }
        s.snapshot.recent_commands.push_back(sample);
    }
    pub fn snapshot(&self) -> Snapshot {
        let s = self.state.lock().unwrap();
        let mut result = s.snapshot.clone();
        result.queue_sample_age_ms = s
            .queue_at
            .map(|at| at.elapsed().as_millis().min(u64::MAX as u128) as u64);
        result.in_flight = s
            .active
            .iter()
            .map(|(_, stage, start)| InFlight {
                stage: *stage,
                elapsed_ms: start.elapsed().as_millis().min(u64::MAX as u128) as u64,
            })
            .collect();
        result
    }
}
impl Drop for Span<'_> {
    fn drop(&mut self) {
        let us = self.start.elapsed().as_micros().min(u64::MAX as u128) as u64;
        let mut s = self.metrics.state.lock().unwrap();
        s.active.retain(|(id, _, _)| *id != self.id);
        if !s
            .snapshot
            .stages
            .iter()
            .any(|(stage, _)| *stage == self.stage)
        {
            s.snapshot.stages.push((self.stage, Total::default()));
        }
        let total = &mut s
            .snapshot
            .stages
            .iter_mut()
            .find(|(stage, _)| *stage == self.stage)
            .unwrap()
            .1;
        total.count = total.count.saturating_add(1);
        total.total_us = total.total_us.saturating_add(us);
        total.max_us = total.max_us.max(us);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stalled_spans_visible_and_early_error_drop_recorded() {
        let metrics = Metrics::default();
        let span = metrics.span(Stage::RpcDirectory);
        assert_eq!(metrics.snapshot().in_flight.len(), 1);
        drop(span);
        assert!(metrics.snapshot().in_flight.is_empty());
        assert_eq!(metrics.snapshot().stages[0].1.count, 1);
        for _ in 0..500 {
            metrics.native(Default::default());
        }
        assert_eq!(metrics.snapshot().native_commands, 500);
        assert_eq!(metrics.snapshot().recent_commands.len(), 64);
    }
}
