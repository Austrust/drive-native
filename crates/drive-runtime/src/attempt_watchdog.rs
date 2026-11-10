//! Issue #58: bounded attempt-rebuild policy for business-progress stalls.
//!
//! The supervisor records `attempt_stalled` when the attempt heartbeat reports
//! no business progress while work is still owed, then rebuilds the worker with
//! a bounded delay. At most [`MAX_STALL_REBUILDS_PER_HOUR`] rebuilds are allowed
//! in a rolling hour; the next stall escalates to a user-visible
//! `needs_attention` state. A rebuild is a transport reconnect: the persisted
//! catalog/cursor/queue state is never reset by this policy.
use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

/// Delays for consecutive rebuilds. The values beyond the first three exist so
/// the policy stays bounded even if the hourly allowance is raised; with the
/// default allowance the fourth observation escalates instead.
pub(crate) const STALL_BACKOFF: [Duration; 5] = [
    Duration::from_secs(1),
    Duration::from_secs(2),
    Duration::from_secs(5),
    Duration::from_secs(15),
    Duration::from_secs(30),
];
pub(crate) const MAX_STALL_REBUILDS_PER_HOUR: usize = 3;
pub(crate) const STALL_WINDOW: Duration = Duration::from_secs(3600);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum StallAction {
    /// Rebuild after this bounded delay; the attempt is retried.
    Rebuild(Duration),
    /// The hourly allowance is exhausted: present the stall to the user.
    NeedsAttention,
}

/// Rolling-hour accounting of observed stalls. Pure over an injected `now`, so
/// the policy is tested without sleeping.
#[derive(Default)]
pub(crate) struct StallTracker {
    recent: VecDeque<Instant>,
}

impl StallTracker {
    pub(crate) fn new() -> Self {
        Self::default()
    }
    pub(crate) fn observe(&mut self, now: Instant) -> StallAction {
        self.recent
            .retain(|previous| now.saturating_duration_since(*previous) < STALL_WINDOW);
        self.recent.push_back(now);
        if self.recent.len() > MAX_STALL_REBUILDS_PER_HOUR {
            StallAction::NeedsAttention
        } else {
            StallAction::Rebuild(
                STALL_BACKOFF[(self.recent.len() - 1).min(STALL_BACKOFF.len() - 1)],
            )
        }
    }
    pub(crate) fn rebuilds_within_window(&self) -> usize {
        self.recent.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rebuild_delays_are_bounded_and_the_fourth_stall_needs_attention() {
        let base = Instant::now();
        let mut stalls = StallTracker::new();
        assert_eq!(
            stalls.observe(base),
            StallAction::Rebuild(Duration::from_secs(1))
        );
        assert_eq!(
            stalls.observe(base + Duration::from_secs(10)),
            StallAction::Rebuild(Duration::from_secs(2))
        );
        assert_eq!(
            stalls.observe(base + Duration::from_secs(20)),
            StallAction::Rebuild(Duration::from_secs(5))
        );
        // Fourth stall inside the same hour: no unbounded rebuild loop.
        assert_eq!(
            stalls.observe(base + Duration::from_secs(30)),
            StallAction::NeedsAttention
        );
        assert_eq!(stalls.rebuilds_within_window(), 4);
    }

    #[test]
    fn a_new_hour_allows_rebuilds_again_and_stale_entries_are_evicted() {
        let base = Instant::now();
        let mut stalls = StallTracker::new();
        for offset in [0u64, 10, 20, 30] {
            let _ = stalls.observe(base + Duration::from_secs(offset));
        }
        // Everything recorded before the one-hour window is gone: a genuinely
        // recovered connection is not punished by old stalls.
        assert_eq!(
            stalls.observe(base + STALL_WINDOW + Duration::from_secs(40)),
            StallAction::Rebuild(Duration::from_secs(1))
        );
        assert_eq!(stalls.rebuilds_within_window(), 1);
    }

    #[test]
    fn stalls_just_inside_the_window_still_count() {
        let base = Instant::now();
        let mut stalls = StallTracker::new();
        assert!(matches!(stalls.observe(base), StallAction::Rebuild(_)));
        assert!(matches!(
            stalls.observe(base + Duration::from_secs(1000)),
            StallAction::Rebuild(_)
        ));
        assert!(matches!(
            stalls.observe(base + Duration::from_secs(2000)),
            StallAction::Rebuild(_)
        ));
        assert_eq!(
            stalls.observe(base + Duration::from_secs(3000)),
            StallAction::NeedsAttention
        );
    }
}
