//! Local progress and timing only. Never authorizes recovery or advances cursors.
use drive_core::snapshot_progress::{Phase as IndexPhase, Progress, RetryReason};
use serde::Serialize;
use std::{
    collections::VecDeque,
    sync::{
        Mutex,
        atomic::{AtomicU64, Ordering},
    },
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
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
    Watermark,
    Seeding,
    Enumerating,
    Fencing,
    Reconciling,
    Retrying,
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
    SnapshotChanged,
    RescanLater,
    MergeMode,
    RootUnavailable,
}
/// Issue #58: catch-up and attempt-lifetime supervision limits. Production
/// values come from [`Limits::from_env`] (all optional); tests override them
/// programmatically with sub-second values so no test sleeps for minutes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Limits {
    /// Total wall-clock budget for one continuous catch-up episode.
    pub catch_up_budget: Duration,
    /// No business progress for longer than this is a no-progress condition.
    pub no_progress_timeout: Duration,
    /// Consecutive `changed=true` pulls that applied nothing are also a
    /// no-progress condition even when the timeout has not elapsed.
    pub no_progress_streak: u64,
    /// An attempt must live at least this long before it can be declared
    /// stalled; a fresh reconnect is not a stall.
    pub attempt_stall_min_elapsed: Duration,
    /// Business-silence threshold for an attempt (keep-alives do not count).
    pub attempt_no_progress_timeout: Duration,
}
pub const CATCH_UP_BUDGET: Duration = Duration::from_secs(600);
pub const NO_PROGRESS_TIMEOUT: Duration = Duration::from_secs(120);
pub const NO_PROGRESS_STREAK: u64 = 3;
pub const ATTEMPT_STALL_MIN_ELAPSED: Duration = Duration::from_secs(30 * 60);
pub const ATTEMPT_NO_PROGRESS_TIMEOUT: Duration = Duration::from_secs(10 * 60);
/// While catch-up or an attempt heartbeat is pending, the event loop must wake
/// out of the notification long poll often enough to run the progress
/// watchdog. This only ever shortens `wire::NOTIFICATION_IDLE`; it never
/// changes the receive-idle budget itself.
pub const WATCHDOG_WAIT_CEILING: Duration = Duration::from_secs(5);
impl Default for Limits {
    fn default() -> Self {
        Self {
            catch_up_budget: CATCH_UP_BUDGET,
            no_progress_timeout: NO_PROGRESS_TIMEOUT,
            no_progress_streak: NO_PROGRESS_STREAK,
            attempt_stall_min_elapsed: ATTEMPT_STALL_MIN_ELAPSED,
            attempt_no_progress_timeout: ATTEMPT_NO_PROGRESS_TIMEOUT,
        }
    }
}
impl Limits {
    /// Optional environment overrides, parsed as integer milliseconds (the
    /// streak as a count). Missing, zero or invalid values keep the default.
    pub fn from_env() -> Self {
        fn millis(name: &str, default: Duration) -> Duration {
            std::env::var(name)
                .ok()
                .and_then(|value| value.trim().parse::<u64>().ok())
                .filter(|millis| *millis > 0)
                .map(Duration::from_millis)
                .unwrap_or(default)
        }
        let mut limits = Self::default();
        limits.catch_up_budget = millis("DRIVE_NATIVE_CATCHUP_BUDGET_MS", limits.catch_up_budget);
        limits.no_progress_timeout = millis(
            "DRIVE_NATIVE_NO_PROGRESS_TIMEOUT_MS",
            limits.no_progress_timeout,
        );
        limits.attempt_stall_min_elapsed = millis(
            "DRIVE_NATIVE_ATTEMPT_STALL_MIN_ELAPSED_MS",
            limits.attempt_stall_min_elapsed,
        );
        limits.attempt_no_progress_timeout = millis(
            "DRIVE_NATIVE_ATTEMPT_NO_PROGRESS_TIMEOUT_MS",
            limits.attempt_no_progress_timeout,
        );
        if let Some(streak) = std::env::var("DRIVE_NATIVE_NO_PROGRESS_STREAK")
            .ok()
            .and_then(|value| value.trim().parse::<u64>().ok())
            .filter(|streak| *streak > 0)
        {
            limits.no_progress_streak = streak;
        }
        limits
    }
}

/// Why a structured `initialization.no_progress` event was emitted.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NoProgressReason {
    /// Consecutive `changed=true` pulls applied no events.
    ChangedWithoutAdvance,
    /// Too long without cursor advance / applied events / index units.
    Timeout,
    /// The catch-up budget ran out while the initialization never settled.
    BudgetExhausted,
}
/// Bounded, content-free description of the last no-progress observation.
/// Keep-alive transport bytes and `changed_without_advance` never refresh it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct NoProgress {
    pub phase: Phase,
    pub streak: u64,
    pub last_progress_age_ms: u64,
    pub items: u64,
    pub reason: NoProgressReason,
}
/// Result of asking to enter the catching-up phase.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CatchUpDecision {
    Enter,
    /// The total catch-up budget is spent: the caller must keep serving cached
    /// data and incremental pulls instead of claiming `catching_up` again.
    BudgetExhausted,
}
/// Business-progress stall reason recorded on the attempt recovery path.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StallReason {
    NoBusinessProgress,
}
impl StallReason {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::NoBusinessProgress => "no_business_progress",
        }
    }
}
/// Pure attempt-heartbeat predicate so the recovery policy is testable without
/// a wall clock: an attempt is stalled only when it is old, business-dead, and
/// still owes work. Keep-alive reads never update `last_business_progress`.
pub fn attempt_stall_reason(
    elapsed: Duration,
    last_business_progress_age: Duration,
    pending_work: bool,
    limits: &Limits,
) -> Option<StallReason> {
    (pending_work
        && elapsed > limits.attempt_stall_min_elapsed
        && last_business_progress_age > limits.attempt_no_progress_timeout)
        .then_some(StallReason::NoBusinessProgress)
}
#[derive(Clone, Serialize)]
pub struct Span {
    pub phase: Phase,
    pub started_unix_ms: u64,
    pub elapsed_ms: u64,
    pub units: u64,
    pub items: u64,
    pub retry_reason: Option<RetryReason>,
}
#[derive(Serialize)]
pub struct Snapshot {
    /// Process-lifetime counters survive ordinary network reconnection. They
    /// are observations, independent of the persistent completion checkpoint.
    pub work: crate::initialization_metrics::Snapshot,
    pub active: bool,
    pub attempt: u64,
    pub started_unix_ms: u64,
    pub elapsed_ms: u64,
    pub phase: Phase,
    pub phase_started_unix_ms: u64,
    pub phase_elapsed_ms: u64,
    pub since_progress_ms: Option<u64>,
    /// Issue #58: truth of the catch-up state and its supervision. `phase` can
    /// be `catching_up` only while `catching_up` is true; after the budget is
    /// spent the runtime keeps pulling incrementally but reports
    /// `catching_up=false` with the observed backlog.
    pub catching_up: bool,
    pub catch_up_budget_ms: u64,
    pub budget_remaining_ms: u64,
    /// Age of the last unit of *business* progress (cursor advance, applied
    /// events or index units). Keep-alives never refresh it.
    pub last_progress_age_ms: Option<u64>,
    pub last_business_progress_at_unix_ms: Option<u64>,
    pub backlog_estimate: Option<u64>,
    pub no_progress: Option<NoProgress>,
    pub no_progress_events: u64,
    pub units: u64,
    pub items: u64,
    pub snapshot_attempt: u8,
    pub retry_count: u64,
    pub last_retry_reason: Option<RetryReason>,
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
    fn retry_cause_survives_next_attempt_without_counting_as_completed_work() {
        let tracker = Tracker::default();
        tracker.begin();
        let retry = Progress {
            attempt: 1,
            phase: IndexPhase::Retry,
            units: 0,
            items: 0,
            retry_reason: Some(RetryReason::ParentUnavailable),
        };
        tracker.index(retry);
        tracker.index(retry);
        assert_eq!(tracker.snapshot().retry_count, 1);
        assert!(tracker.snapshot().since_progress_ms.is_none());
        tracker.index(Progress {
            attempt: 2,
            phase: IndexPhase::Seed,
            units: 0,
            items: 0,
            retry_reason: None,
        });
        let state = tracker.snapshot();
        assert_eq!(
            state.last_retry_reason,
            Some(RetryReason::ParentUnavailable)
        );
        assert_eq!(
            state.recent_stages.last().unwrap().retry_reason,
            state.last_retry_reason
        );
        tracker.begin();
        assert_eq!(tracker.snapshot().retry_count, 0);
        assert!(tracker.snapshot().last_retry_reason.is_none());
    }

    #[test]
    fn watermark_and_local_reconciliation_have_distinct_observed_stages() {
        let tracker = Tracker::default();
        tracker.begin();
        for (index, phase) in [
            (IndexPhase::Watermark, Phase::Watermark),
            (IndexPhase::Reconcile, Phase::Reconciling),
        ] {
            tracker.index(Progress {
                retry_reason: None,
                attempt: 1,
                phase: index,
                units: 2,
                items: 3,
            });
            assert_eq!(tracker.snapshot().phase, phase);
            assert_eq!(tracker.snapshot().units, 2);
            assert!(tracker.snapshot().active);
        }
    }

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
            retry_reason: None,
            attempt: 1,
            phase: IndexPhase::Enumerate,
            units: 3,
            items: 42,
        });
        assert_eq!(tracker.snapshot().items, 42);
        // A coalesced retry may first arrive in the same phase as the old one.
        tracker.index(Progress {
            retry_reason: None,
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
        assert!(state.phase_totals_ms.len() <= 14);
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
    fn short_limits(budget: Duration, timeout: Duration, streak: u64) -> Limits {
        Limits {
            catch_up_budget: budget,
            no_progress_timeout: timeout,
            no_progress_streak: streak,
            ..Limits::default()
        }
    }

    #[test]
    fn production_default_limits_match_the_issue_contract() {
        let limits = Limits::default();
        assert_eq!(limits.catch_up_budget, Duration::from_secs(600));
        assert_eq!(limits.no_progress_timeout, Duration::from_secs(120));
        assert_eq!(limits.no_progress_streak, 3);
        assert_eq!(limits.attempt_stall_min_elapsed, Duration::from_secs(1800));
        assert_eq!(limits.attempt_no_progress_timeout, Duration::from_secs(600));
    }

    #[test]
    fn catch_up_budget_exhaustion_is_explicit_and_journalled() {
        let tracker = Tracker::default();
        tracker.configure_limits(short_limits(
            Duration::from_millis(100),
            Duration::from_secs(60),
            10,
        ));
        tracker.begin();
        assert_eq!(tracker.begin_catch_up(), CatchUpDecision::Enter);
        // One changed notification with nothing applied: an observed backlog.
        tracker.pulled(false, 0, true);
        tracker.shift_clock_back_for_test(Duration::from_millis(150));
        assert_eq!(tracker.begin_catch_up(), CatchUpDecision::BudgetExhausted);
        let snapshot = tracker.snapshot();
        assert!(!snapshot.catching_up);
        assert_eq!(snapshot.budget_remaining_ms, 0);
        assert_eq!(snapshot.catch_up_budget_ms, 100);
        assert_eq!(snapshot.backlog_estimate, Some(1));
        let event = snapshot
            .no_progress
            .expect("budget exhaustion is a no-progress signal");
        assert_eq!(event.reason, NoProgressReason::BudgetExhausted);
        assert_eq!(event.streak, 1);
        assert!(event.last_progress_age_ms >= 150);
        assert_eq!(snapshot.no_progress_events, 1);
        // Once spent, the same episode never claims catching_up again.
        assert_eq!(tracker.begin_catch_up(), CatchUpDecision::BudgetExhausted);
        // A quiet pull settles the episode and restores the full budget.
        tracker.pulled(false, 0, false);
        assert_eq!(tracker.snapshot().budget_remaining_ms, 100);
        assert_eq!(tracker.begin_catch_up(), CatchUpDecision::Enter);
        assert!(tracker.snapshot().catching_up);
    }

    #[test]
    fn changed_notification_streak_emits_one_structured_event_per_episode() {
        let tracker = Tracker::default();
        tracker.configure_limits(short_limits(
            Duration::from_secs(600),
            Duration::from_secs(60),
            3,
        ));
        tracker.begin();
        tracker.begin_catch_up();
        for _ in 0..2 {
            tracker.pulled(false, 0, true);
        }
        assert!(tracker.no_progress_check().is_none());
        tracker.pulled(false, 0, true);
        let event = tracker
            .no_progress_check()
            .expect("streak threshold must emit");
        assert_eq!(event.reason, NoProgressReason::ChangedWithoutAdvance);
        assert_eq!(event.streak, 3);
        assert!(
            tracker.no_progress_check().is_none(),
            "one event per episode"
        );
        assert_eq!(tracker.snapshot().no_progress_events, 1);
        // A real cursor advance counts as progress and reboots the watchdog.
        tracker.pulled(true, 2, true);
        for _ in 0..3 {
            tracker.pulled(false, 0, true);
        }
        assert!(tracker.no_progress_check().is_some());
        assert_eq!(tracker.snapshot().no_progress_events, 2);
    }

    #[test]
    fn no_progress_timeout_emits_even_without_a_changed_streak() {
        let tracker = Tracker::default();
        tracker.configure_limits(short_limits(
            Duration::from_secs(600),
            Duration::from_millis(50),
            100,
        ));
        tracker.begin();
        tracker.begin_catch_up();
        tracker.shift_clock_back_for_test(Duration::from_millis(80));
        // The attempt budget is not what fires here.
        assert!(!tracker.enforce_catch_up_budget());
        let event = tracker.no_progress_check().expect("timeout must emit");
        assert_eq!(event.reason, NoProgressReason::Timeout);
        assert!(event.last_progress_age_ms >= 80);
    }

    #[test]
    fn attempt_heartbeat_requires_elapsed_and_business_silence_and_owed_work() {
        let limits = Limits {
            attempt_stall_min_elapsed: Duration::from_secs(1800),
            attempt_no_progress_timeout: Duration::from_secs(600),
            ..Limits::default()
        };
        let old = Duration::from_secs(1801);
        let silent = Duration::from_secs(601);
        assert_eq!(
            attempt_stall_reason(old, silent, true, &limits),
            Some(StallReason::NoBusinessProgress)
        );
        assert_eq!(
            attempt_stall_reason(old, silent, false, &limits),
            None,
            "a healthy idle attempt is not a stall"
        );
        assert_eq!(
            attempt_stall_reason(Duration::from_secs(1799), silent, true, &limits),
            None
        );
        assert_eq!(
            attempt_stall_reason(old, Duration::from_secs(599), true, &limits),
            None
        );
    }

    #[test]
    fn simulated_100k_event_advances_never_trigger_the_progress_watchdog() {
        let tracker = Tracker::default();
        tracker.configure_limits(short_limits(
            Duration::from_secs(60),
            Duration::from_secs(2),
            2,
        ));
        tracker.begin();
        for batch in 0..1000u64 {
            assert_eq!(
                tracker.begin_catch_up(),
                CatchUpDecision::Enter,
                "batch {batch}"
            );
            // 100 applied events with an advancing cursor per batch: exactly
            // the progress definition (cursor_advances>0 || events_applied>0).
            tracker.pulled(true, 100, true);
            assert!(
                tracker.no_progress_check().is_none(),
                "progress-based watchdog fired on batch {batch}"
            );
        }
        let snapshot = tracker.snapshot();
        assert_eq!(snapshot.events_applied, 100_000);
        assert_eq!(snapshot.cursor_advances, 1_000);
        assert!(snapshot.catching_up);
        assert!(snapshot.no_progress.is_none());
        assert!(snapshot.budget_remaining_ms > 0);
    }

    #[test]
    fn changed_notifications_never_refresh_the_business_heartbeat() {
        let tracker = Tracker::default();
        tracker.configure_limits(Limits {
            attempt_stall_min_elapsed: Duration::from_millis(100),
            attempt_no_progress_timeout: Duration::from_millis(100),
            ..Limits::default()
        });
        tracker.begin();
        tracker.begin_catch_up();
        tracker.shift_clock_back_for_test(Duration::from_millis(300));
        assert_eq!(
            tracker.attempt_stall(true),
            Some(StallReason::NoBusinessProgress)
        );
        // A changed notification with nothing applied is not progress...
        tracker.pulled(false, 5, true);
        assert_eq!(
            tracker.attempt_stall(true),
            Some(StallReason::NoBusinessProgress)
        );
        // ...but a committed cursor advance is.
        tracker.pulled(true, 5, true);
        assert_eq!(tracker.attempt_stall(true), None);
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
    /// Issue #58: business-only heartbeat. `activity()` and real pull/index
    /// advances refresh it; notification keep-alives do not.
    last_business_progress: Option<Instant>,
    last_business_progress_unix_ms: Option<u64>,
    /// Start of the current continuous catch-up episode; `None` means the last
    /// pull settled the session (`batch=None` with no pending change).
    catch_up_started: Option<Instant>,
    budget_exhausted: bool,
    backlog_estimate: Option<u64>,
    no_progress: Option<NoProgress>,
    no_progress_events: u64,
    no_progress_reported: bool,
    ended: Option<Instant>,
    units: u64,
    items: u64,
    snapshot_attempt: u8,
    retry_count: u64,
    last_retry_reason: Option<RetryReason>,
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
            last_business_progress: Some(now),
            last_business_progress_unix_ms: Some(unix),
            catch_up_started: None,
            budget_exhausted: false,
            backlog_estimate: None,
            no_progress: None,
            no_progress_events: 0,
            no_progress_reported: false,
            ended: None,
            units: 0,
            items: 0,
            snapshot_attempt: 0,
            retry_count: 0,
            last_retry_reason: None,
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
            retry_reason: if self.phase == Phase::Retrying {
                self.last_retry_reason
            } else {
                None
            },
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
    /// Spend this episode's total catch-up budget. Returns the journal payload
    /// for the caller to print outside the lock.
    fn exhaust_catch_up_budget(&mut self, now: Instant, limits: &Limits) -> serde_json::Value {
        self.budget_exhausted = true;
        let age = self.last_business_progress.map_or_else(
            || now.saturating_duration_since(self.started),
            |last| now.saturating_duration_since(last),
        );
        if !self.no_progress_reported {
            self.no_progress_reported = true;
            self.no_progress_events = self.no_progress_events.saturating_add(1);
            self.no_progress = Some(NoProgress {
                phase: Phase::CatchingUp,
                streak: self.streak,
                last_progress_age_ms: ms(age),
                items: self.items,
                reason: NoProgressReason::BudgetExhausted,
            });
        }
        serde_json::json!({
            "event": "initialization.catch_up_budget_exhausted",
            "budget_ms": ms(limits.catch_up_budget),
            "last_progress_age_ms": ms(age),
            "streak": self.streak,
            "items": self.items,
            "backlog_estimate": self.backlog_estimate,
        })
    }
}
pub struct Tracker {
    pub work: crate::initialization_metrics::Metrics,
    state: Mutex<State>,
    revision: AtomicU64,
    // Issue #58 limits are atomics so every method takes exactly one lock and
    // tests can reconfigure them before starting a deterministic run.
    catch_up_budget_ms: AtomicU64,
    no_progress_timeout_ms: AtomicU64,
    no_progress_streak: AtomicU64,
    attempt_stall_min_elapsed_ms: AtomicU64,
    attempt_no_progress_timeout_ms: AtomicU64,
}
impl Default for Tracker {
    fn default() -> Self {
        // Production and test harnesses configure the supervision deadlines
        // through the documented `DRIVE_NATIVE_*` variables; defaults apply when
        // unset.
        let limits = Limits::from_env();
        Self {
            work: crate::initialization_metrics::Metrics::default(),
            state: Mutex::new(State::new(0, false)),
            revision: AtomicU64::new(0),
            catch_up_budget_ms: AtomicU64::new(limits.catch_up_budget.as_millis() as u64),
            no_progress_timeout_ms: AtomicU64::new(limits.no_progress_timeout.as_millis() as u64),
            no_progress_streak: AtomicU64::new(limits.no_progress_streak),
            attempt_stall_min_elapsed_ms: AtomicU64::new(
                limits.attempt_stall_min_elapsed.as_millis() as u64,
            ),
            attempt_no_progress_timeout_ms: AtomicU64::new(
                limits.attempt_no_progress_timeout.as_millis() as u64,
            ),
        }
    }
}
impl Tracker {
    /// Replace the supervision limits (production reads [`Limits::from_env`];
    /// tests pass sub-second values).
    pub fn configure_limits(&self, limits: Limits) {
        self.catch_up_budget_ms
            .store(limits.catch_up_budget.as_millis() as u64, Ordering::Relaxed);
        self.no_progress_timeout_ms.store(
            limits.no_progress_timeout.as_millis() as u64,
            Ordering::Relaxed,
        );
        self.no_progress_streak
            .store(limits.no_progress_streak, Ordering::Relaxed);
        self.attempt_stall_min_elapsed_ms.store(
            limits.attempt_stall_min_elapsed.as_millis() as u64,
            Ordering::Relaxed,
        );
        self.attempt_no_progress_timeout_ms.store(
            limits.attempt_no_progress_timeout.as_millis() as u64,
            Ordering::Relaxed,
        );
    }
    pub fn limits(&self) -> Limits {
        Limits {
            catch_up_budget: Duration::from_millis(self.catch_up_budget_ms.load(Ordering::Relaxed)),
            no_progress_timeout: Duration::from_millis(
                self.no_progress_timeout_ms.load(Ordering::Relaxed),
            ),
            no_progress_streak: self.no_progress_streak.load(Ordering::Relaxed),
            attempt_stall_min_elapsed: Duration::from_millis(
                self.attempt_stall_min_elapsed_ms.load(Ordering::Relaxed),
            ),
            attempt_no_progress_timeout: Duration::from_millis(
                self.attempt_no_progress_timeout_ms.load(Ordering::Relaxed),
            ),
        }
    }
    /// Enter the catching-up phase unless the episode's total budget is spent.
    /// Called by the event loop before every pull; the first call after the
    /// session settled starts a new episode.
    pub fn begin_catch_up(&self) -> CatchUpDecision {
        let limits = self.limits();
        let journal = {
            let mut s = self.state.lock().unwrap();
            let now = Instant::now();
            let Some(started) = s.catch_up_started else {
                s.catch_up_started = Some(now);
                s.phase(Phase::CatchingUp, false);
                return CatchUpDecision::Enter;
            };
            if now.saturating_duration_since(started) < limits.catch_up_budget {
                s.phase(Phase::CatchingUp, false);
                return CatchUpDecision::Enter;
            }
            if s.budget_exhausted {
                return CatchUpDecision::BudgetExhausted;
            }
            s.exhaust_catch_up_budget(now, &limits)
        };
        // Journal-visible structured event: the runtime stopped claiming
        // `catching_up` and keeps serving cached data plus incremental pulls.
        eprintln!("{journal}");
        self.changed();
        CatchUpDecision::BudgetExhausted
    }
    /// Spend an already-expired budget without waiting for the next pull. The
    /// attempt watchdog calls this while the event loop is blocked in a
    /// keep-alive-renewed notification wait, so `catching_up=false` is timely.
    pub fn enforce_catch_up_budget(&self) -> bool {
        let limits = self.limits();
        let journal = {
            let mut s = self.state.lock().unwrap();
            let Some(started) = s.catch_up_started else {
                return false;
            };
            if s.budget_exhausted {
                return false;
            }
            let now = Instant::now();
            if now.saturating_duration_since(started) < limits.catch_up_budget {
                return false;
            }
            s.exhaust_catch_up_budget(now, &limits)
        };
        eprintln!("{journal}");
        self.changed();
        true
    }
    /// True while initialization/catch-up has not settled. This is the
    /// "still owes business progress" input for the attempt heartbeat; it is
    /// not refreshed by keep-alive reads.
    pub fn catch_up_pending(&self) -> bool {
        self.state.lock().unwrap().catch_up_started.is_some()
    }
    /// Emit the structured `initialization.no_progress` event once per catch-up
    /// episode when keep-alives/`changed_without_advance` are the only reads.
    pub fn no_progress_check(&self) -> Option<NoProgress> {
        let limits = self.limits();
        let event = {
            let mut s = self.state.lock().unwrap();
            if s.catch_up_started.is_none() {
                return None;
            }
            let now = Instant::now();
            let age = s.last_business_progress.map_or_else(
                || now.saturating_duration_since(s.started),
                |last| now.saturating_duration_since(last),
            );
            let streak_triggered = s.streak >= limits.no_progress_streak;
            let timeout_triggered = age > limits.no_progress_timeout;
            if (!streak_triggered && !timeout_triggered) || s.no_progress_reported {
                return None;
            }
            s.no_progress_reported = true;
            s.no_progress_events = s.no_progress_events.saturating_add(1);
            let event = NoProgress {
                phase: s.phase,
                streak: s.streak,
                last_progress_age_ms: ms(age),
                items: s.items,
                reason: if streak_triggered {
                    NoProgressReason::ChangedWithoutAdvance
                } else {
                    NoProgressReason::Timeout
                },
            };
            s.no_progress = Some(event.clone());
            event
        };
        eprintln!(
            "{}",
            serde_json::json!({
                "event": "initialization.no_progress",
                "phase": event.phase,
                "streak": event.streak,
                "last_progress_age_ms": event.last_progress_age_ms,
                "items": event.items,
                "reason": event.reason,
            })
        );
        self.changed();
        Some(event)
    }
    /// Attempt heartbeat: an old attempt that still owes work but has recorded
    /// no business progress for longer than the configured threshold is
    /// stalled. The supervisor turns this into one bounded rebuild.
    pub fn attempt_stall(&self, pending_work: bool) -> Option<StallReason> {
        if !pending_work {
            return None;
        }
        let limits = self.limits();
        let s = self.state.lock().unwrap();
        if !s.active {
            return None;
        }
        let now = Instant::now();
        let elapsed = now.saturating_duration_since(s.started);
        let age = s
            .last_business_progress
            .map_or(elapsed, |last| now.saturating_duration_since(last));
        attempt_stall_reason(elapsed, age, pending_work, &limits)
    }
    /// While an attempt heartbeat is pending, bound the notification long poll
    /// so the watchdog runs. Never extends `NOTIFICATION_IDLE`.
    pub fn notification_wait_bound(&self, pending_work: bool) -> Option<Duration> {
        if !pending_work {
            return None;
        }
        let limits = self.limits();
        let active = self.state.lock().unwrap().active;
        if !active {
            return None;
        }
        Some(
            (limits.attempt_no_progress_timeout / 2)
                .clamp(Duration::from_millis(20), WATCHDOG_WAIT_CEILING),
        )
    }
    pub fn last_business_progress_at_unix_ms(&self) -> Option<u64> {
        self.state.lock().unwrap().last_business_progress_unix_ms
    }
    /// Test-only clock injection: shift every recorded instant into the past so
    /// attempt/budget deadlines can be exercised without sleeping.
    #[cfg(test)]
    pub(crate) fn shift_clock_back_for_test(&self, distance: Duration) {
        fn shift(instant: &mut Instant, distance: Duration) {
            if let Some(earlier) = instant.checked_sub(distance) {
                *instant = earlier;
            }
        }
        let mut s = self.state.lock().unwrap();
        shift(&mut s.started, distance);
        shift(&mut s.phase_started, distance);
        if let Some(last) = s.last_progress.as_mut() {
            shift(last, distance);
        }
        if let Some(last) = s.last_business_progress.as_mut() {
            shift(last, distance);
        }
        if let Some(started) = s.catch_up_started.as_mut() {
            shift(started, distance);
        }
    }
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
        let now = Instant::now();
        let mut s = self.state.lock().unwrap();
        s.last_progress = Some(now);
        s.last_business_progress = Some(now);
        s.last_business_progress_unix_ms = Some(unix_ms());
        drop(s);
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
        s.retry_count = 0;
        s.last_retry_reason = None;
        // The fresh snapshot is a new attempt at catching up: the previous
        // episode's budget/no-progress bookkeeping must not poison it.
        s.catch_up_started = None;
        s.budget_exhausted = false;
        s.backlog_estimate = None;
        s.no_progress_reported = false;
        s.streak = 0;
        s.phase(Phase::Recovering, false);
        self.changed();
    }
    pub fn index(&self, p: Progress) {
        let mut s = self.state.lock().unwrap();
        let phase = match p.phase {
            IndexPhase::Watermark => Phase::Watermark,
            IndexPhase::Seed => Phase::Seeding,
            IndexPhase::Enumerate => Phase::Enumerating,
            IndexPhase::Fence => Phase::Fencing,
            IndexPhase::Reconcile => Phase::Reconciling,
            IndexPhase::Retry => Phase::Retrying,
        };
        let advanced = s.snapshot_attempt != p.attempt
            || s.phase != phase
            || p.units > s.units
            || p.items > s.items;
        let retry = s.snapshot_attempt != p.attempt;
        let diagnosed_retry = p.retry_reason.is_some() && (retry || s.phase != Phase::Retrying);
        s.snapshot_attempt = p.attempt;
        s.phase(phase, retry);
        s.units = p.units;
        s.items = p.items;
        if let Some(reason) = p.retry_reason {
            s.last_retry_reason = Some(reason);
            if diagnosed_retry {
                s.retry_count = s.retry_count.saturating_add(1);
            }
        }
        // A phase boundary alone is not proof of completed network work.
        if advanced && p.units > 0 {
            let now = Instant::now();
            s.last_progress = Some(now);
            s.last_business_progress = Some(now);
            s.last_business_progress_unix_ms = Some(unix_ms());
            s.no_progress_reported = false;
        }
        if advanced {
            self.changed();
        }
    }
    /// Record one ordinary pull result. `advanced` means the committed cursor
    /// moved; `changed_notification` means the long poll claimed a change even
    /// when the pull applied nothing.
    pub fn pulled(&self, advanced: bool, events: usize, changed_notification: bool) {
        let mut s = self.state.lock().unwrap();
        if advanced {
            let now = Instant::now();
            s.cursor_advances = s.cursor_advances.saturating_add(1);
            s.events_applied = s.events_applied.saturating_add(events as u64);
            s.last_progress = Some(now);
            s.last_business_progress = Some(now);
            s.last_business_progress_unix_ms = Some(unix_ms());
            s.streak = 0;
            // Applied events are no longer a backlog lower bound.
            s.backlog_estimate = None;
            s.no_progress_reported = false;
        } else if changed_notification {
            s.changed_without_advance = s.changed_without_advance.saturating_add(1);
            s.streak = s.streak.saturating_add(1);
            // A changed notification with nothing applied is at least this many
            // unobserved business changes; keep it as the backlog lower bound.
            s.backlog_estimate = Some(s.streak);
        } else {
            // A quiet pull (`batch=None` with no pending change) settles the
            // continuous catch-up episode: the next pull starts a fresh budget.
            s.streak = 0;
            s.catch_up_started = None;
            s.budget_exhausted = false;
            s.backlog_estimate = None;
            s.no_progress_reported = false;
        }
        s.units = s.cursor_advances;
        s.items = s.events_applied;
        self.changed();
    }
    pub fn snapshot(&self) -> Snapshot {
        let s = self.state.lock().unwrap();
        let now = s.ended.unwrap_or_else(Instant::now);
        let elapsed = ms(now.saturating_duration_since(s.phase_started));
        let catch_up_budget_ms = self.catch_up_budget_ms.load(Ordering::Relaxed);
        let catch_up_elapsed_ms = s
            .catch_up_started
            .map(|started| ms(now.saturating_duration_since(started)))
            .unwrap_or(0);
        let last_progress_age_ms = s
            .last_business_progress
            .map(|last| ms(now.saturating_duration_since(last)));
        let mut totals = s.totals.clone();
        if let Some((_, total)) = totals.iter_mut().find(|(p, _)| *p == s.phase) {
            *total = total.saturating_add(elapsed);
        } else {
            totals.push((s.phase, elapsed));
        }
        Snapshot {
            work: self.work.snapshot(),
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
            catching_up: s.catch_up_started.is_some() && !s.budget_exhausted,
            catch_up_budget_ms,
            budget_remaining_ms: catch_up_budget_ms.saturating_sub(catch_up_elapsed_ms),
            last_progress_age_ms,
            last_business_progress_at_unix_ms: s.last_business_progress_unix_ms,
            backlog_estimate: s.backlog_estimate,
            no_progress: s.no_progress.clone(),
            no_progress_events: s.no_progress_events,
            units: s.units,
            items: s.items,
            snapshot_attempt: s.snapshot_attempt,
            retry_count: s.retry_count,
            last_retry_reason: s.last_retry_reason,
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
