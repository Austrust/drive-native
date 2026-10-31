//! Local progress and timing only. Never authorizes recovery or advances cursors.
use drive_core::snapshot_progress::{Phase as IndexPhase, Progress};
use serde::Serialize;
use std::{
    collections::VecDeque,
    sync::{
        Mutex,
        atomic::{AtomicU64, Ordering},
    },
    time::{Instant, SystemTime, UNIX_EPOCH},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    Starting,
    Authenticating,
    Configuring,
    Binding,
    CatchingUp,
    Recovering,
    Seeding,
    Enumerating,
    Fencing,
    Importing,
    Waiting,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RecoveryReason {
    InitialIndex,
    PermissionRecheck,
    CursorInvalidated,
    FilteredTypeChanged,
}
#[derive(Clone, Serialize)]
pub struct Span {
    pub phase: Phase,
    pub started_unix_ms: u64,
    pub elapsed_ms: u64,
    pub units: u64,
    pub items: u64,
}
#[derive(Serialize)]
pub struct Snapshot {
    pub active: bool,
    pub attempt: u64,
    pub started_unix_ms: u64,
    pub elapsed_ms: u64,
    pub phase: Phase,
    pub phase_started_unix_ms: u64,
    pub phase_elapsed_ms: u64,
    pub since_progress_ms: Option<u64>,
    pub units: u64,
    pub items: u64,
    pub snapshot_attempt: u8,
    pub recovery_reason: Option<RecoveryReason>,
    pub cursor_advances: u64,
    pub events_applied: u64,
    pub changed_without_advance: u64,
    pub consecutive_changed_without_advance: u64,
    pub recent_stages: Vec<Span>,
    pub phase_totals_ms: Vec<(Phase, u64)>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn changed_notification_without_advancement_is_distinct_from_idle_wait() {
        let tracker = Tracker::default();
        tracker.begin();
        for _ in 0..500 {
            tracker.pulled(true, 2, true);
        }
        assert_eq!(tracker.snapshot().changed_without_advance, 0);
        assert_eq!(tracker.snapshot().events_applied, 1000);
        tracker.pulled(false, 0, true);
        tracker.pulled(false, 0, true);
        assert_eq!(tracker.snapshot().consecutive_changed_without_advance, 2);
        tracker.pulled(false, 0, false);
        assert_eq!(tracker.snapshot().consecutive_changed_without_advance, 0);
        assert_eq!(tracker.snapshot().changed_without_advance, 2);
    }

    #[test]
    fn stages_are_bounded_and_completed_stage_timing_survives_event_churn() {
        let tracker = Tracker::default();
        tracker.begin();
        tracker.phase(Phase::Authenticating);
        tracker.activity();
        tracker.phase(Phase::Enumerating);
        tracker.index(Progress {
            attempt: 1,
            phase: IndexPhase::Enumerate,
            units: 3,
            items: 42,
        });
        assert_eq!(tracker.snapshot().items, 42);
        // A coalesced retry may first arrive in the same phase as the old one.
        tracker.index(Progress {
            attempt: 2,
            phase: IndexPhase::Enumerate,
            units: 0,
            items: 1,
        });
        assert_eq!(tracker.snapshot().recent_stages.last().unwrap().items, 42);
        assert_eq!(tracker.snapshot().snapshot_attempt, 2);
        for _ in 0..100 {
            tracker.phase(Phase::CatchingUp);
            tracker.phase(Phase::Waiting);
        }
        let state = tracker.snapshot();
        assert_eq!(state.recent_stages.len(), 16);
        assert!(
            state
                .phase_totals_ms
                .iter()
                .any(|(p, _)| *p == Phase::Authenticating)
        );
        assert!(state.phase_totals_ms.len() <= 12);
    }

    #[test]
    fn observation_is_not_activity_and_session_end_freezes_elapsed() {
        let tracker = Tracker::default();
        tracker.begin();
        let revision = tracker.revision();
        assert!(tracker.snapshot().since_progress_ms.is_none());
        let _ = tracker.snapshot();
        let _ = tracker.snapshot();
        assert_eq!(tracker.revision(), revision);
        tracker.activity();
        assert!(tracker.snapshot().since_progress_ms.is_some());
        tracker.end();
        let before = tracker.snapshot();
        std::thread::sleep(std::time::Duration::from_millis(5));
        assert_eq!(before.elapsed_ms, tracker.snapshot().elapsed_ms);
        tracker.begin();
        assert_eq!(tracker.snapshot().attempt, 2);
        assert_eq!(tracker.snapshot().events_applied, 0);
    }
}
struct State {
    active: bool,
    attempt: u64,
    started: Instant,
    started_unix_ms: u64,
    phase: Phase,
    phase_started: Instant,
    phase_unix_ms: u64,
    last_progress: Option<Instant>,
    ended: Option<Instant>,
    units: u64,
    items: u64,
    snapshot_attempt: u8,
    reason: Option<RecoveryReason>,
    cursor_advances: u64,
    events_applied: u64,
    changed_without_advance: u64,
    streak: u64,
    history: VecDeque<Span>,
    totals: Vec<(Phase, u64)>,
}
fn unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .min(u64::MAX as u128) as u64
}
fn ms(duration: std::time::Duration) -> u64 {
    duration.as_millis().min(u64::MAX as u128) as u64
}
impl State {
    fn new(attempt: u64, active: bool) -> Self {
        let now = Instant::now();
        let unix = unix_ms();
        Self {
            active,
            attempt,
            started: now,
            started_unix_ms: unix,
            phase: Phase::Starting,
            phase_started: now,
            phase_unix_ms: unix,
            last_progress: None,
            ended: None,
            units: 0,
            items: 0,
            snapshot_attempt: 0,
            reason: None,
            cursor_advances: 0,
            events_applied: 0,
            changed_without_advance: 0,
            streak: 0,
            history: VecDeque::new(),
            totals: Vec::new(),
        }
    }
    fn phase(&mut self, phase: Phase, restart: bool) {
        if self.phase == phase && !restart {
            return;
        }
        let elapsed = ms(self.phase_started.elapsed());
        if let Some((_, total)) = self.totals.iter_mut().find(|(p, _)| *p == self.phase) {
            *total = total.saturating_add(elapsed);
        } else {
            self.totals.push((self.phase, elapsed));
        }
        self.history.push_back(Span {
            phase: self.phase,
            started_unix_ms: self.phase_unix_ms,
            elapsed_ms: ms(self.phase_started.elapsed()),
            units: self.units,
            items: self.items,
        });
        if self.history.len() > 16 {
            self.history.pop_front();
        }
        self.phase = phase;
        self.phase_started = Instant::now();
        self.phase_unix_ms = unix_ms();
        self.units = 0;
        self.items = 0;
    }
}
pub struct Tracker {
    state: Mutex<State>,
    revision: AtomicU64,
}
impl Default for Tracker {
    fn default() -> Self {
        Self {
            state: Mutex::new(State::new(0, false)),
            revision: AtomicU64::new(0),
        }
    }
}
impl Tracker {
    pub fn revision(&self) -> u64 {
        self.revision.load(Ordering::Relaxed)
    }
    fn changed(&self) {
        self.revision.fetch_add(1, Ordering::Relaxed);
    }
    pub fn begin(&self) {
        let mut s = self.state.lock().unwrap();
        let attempt = s.attempt.saturating_add(1);
        *s = State::new(attempt, true);
        self.changed();
    }
    pub fn phase(&self, phase: Phase) {
        let mut s = self.state.lock().unwrap();
        if s.phase != phase {
            s.phase(phase, false);
            self.changed();
        }
    }
    pub fn activity(&self) {
        self.state.lock().unwrap().last_progress = Some(Instant::now());
        self.changed();
    }
    pub fn end(&self) {
        let mut s = self.state.lock().unwrap();
        s.active = false;
        s.ended = Some(Instant::now());
        self.changed();
    }
    pub fn recovery(&self, reason: RecoveryReason) {
        let mut s = self.state.lock().unwrap();
        s.reason = Some(reason);
        s.snapshot_attempt = 0;
        s.phase(Phase::Recovering, false);
        self.changed();
    }
    pub fn index(&self, p: Progress) {
        let mut s = self.state.lock().unwrap();
        let phase = match p.phase {
            IndexPhase::Seed => Phase::Seeding,
            IndexPhase::Enumerate => Phase::Enumerating,
            IndexPhase::Fence => Phase::Fencing,
        };
        let advanced = s.snapshot_attempt != p.attempt
            || s.phase != phase
            || p.units > s.units
            || p.items > s.items;
        let retry = s.snapshot_attempt != p.attempt;
        s.snapshot_attempt = p.attempt;
        s.phase(phase, retry);
        s.units = p.units;
        s.items = p.items;
        // A phase boundary alone is not proof of completed network work.
        if advanced && p.units > 0 {
            s.last_progress = Some(Instant::now());
        }
        if advanced {
            self.changed();
        }
    }
    pub fn pulled(&self, advanced: bool, events: usize, changed_notification: bool) {
        let mut s = self.state.lock().unwrap();
        if advanced {
            s.cursor_advances = s.cursor_advances.saturating_add(1);
            s.events_applied = s.events_applied.saturating_add(events as u64);
            s.last_progress = Some(Instant::now());
            s.streak = 0;
        } else if changed_notification {
            s.changed_without_advance = s.changed_without_advance.saturating_add(1);
            s.streak = s.streak.saturating_add(1);
        } else {
            s.streak = 0;
        }
        s.units = s.cursor_advances;
        s.items = s.events_applied;
        self.changed();
    }
    pub fn snapshot(&self) -> Snapshot {
        let s = self.state.lock().unwrap();
        let now = s.ended.unwrap_or_else(Instant::now);
        let elapsed = ms(now.saturating_duration_since(s.phase_started));
        let mut totals = s.totals.clone();
        if let Some((_, total)) = totals.iter_mut().find(|(p, _)| *p == s.phase) {
            *total = total.saturating_add(elapsed);
        } else {
            totals.push((s.phase, elapsed));
        }
        Snapshot {
            active: s.active,
            attempt: s.attempt,
            started_unix_ms: s.started_unix_ms,
            elapsed_ms: ms(now.saturating_duration_since(s.started)),
            phase: s.phase,
            phase_started_unix_ms: s.phase_unix_ms,
            phase_elapsed_ms: elapsed,
            since_progress_ms: s
                .last_progress
                .map(|last| ms(now.saturating_duration_since(last))),
            units: s.units,
            items: s.items,
            snapshot_attempt: s.snapshot_attempt,
            recovery_reason: s.reason,
            cursor_advances: s.cursor_advances,
            events_applied: s.events_applied,
            changed_without_advance: s.changed_without_advance,
            consecutive_changed_without_advance: s.streak,
            recent_stages: s.history.iter().cloned().collect(),
            phase_totals_ms: totals,
        }
    }
}
