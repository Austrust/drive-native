//! The durable wall-clock deadline survives restart. During one process, a
//! wall-clock correction must not shorten an already published retry delay.
use drive_core::{DownloadTicket, download_jobs::DownloadFailure};
use std::{collections::HashMap, time::{Duration, Instant}};

#[derive(Default)]
pub(crate) struct RetryClock {
    gates: HashMap<(String, String), Gate>,
}
struct Gate {
    failure: DownloadFailure,
    not_before: Option<Instant>,
}
impl RetryClock {
    pub fn clear(&mut self, ticket: &DownloadTicket) {
        self.gates.remove(&(ticket.id.clone(), ticket.version.clone()));
    }
    /// Call under the Store guard that publishes this exact failure. `wall`
    /// is the timestamp passed to fail_download, not a later wall-clock read.
    pub fn publish(&mut self, ticket: &DownloadTicket, failure: Option<&DownloadFailure>, wall: u64, now: Instant) {
        self.gates.retain(|(id, version), gate| {
            (id != &ticket.id || version == &ticket.version)
                && gate.not_before.is_none_or(|at| at > now)
        });
        self.clear(ticket);
        if let Some(failure) = failure.filter(|f| f.retry_at.is_some()) {
            let remaining = failure.retry_at.unwrap().saturating_sub(wall);
            self.gates.insert((ticket.id.clone(), ticket.version.clone()), Gate {
                failure: failure.clone(),
                not_before: now.checked_add(Duration::from_secs(remaining)),
            });
        }
    }
    /// A restart seeds from the persisted remaining delay once. Reconnects
    /// retain this object; an explicit retry/reset removes its failure record.
    pub fn ready(&mut self, ticket: &DownloadTicket, failure: Option<&DownloadFailure>, wall: u64, now: Instant) -> bool {
        let Some(failure) = failure else {
            self.clear(ticket);
            return true;
        };
        let Some(retry_at) = failure.retry_at else {
            self.clear(ticket);
            return false;
        };
        let key = (ticket.id.clone(), ticket.version.clone());
        if self.gates.get(&key).is_none_or(|gate| gate.failure != *failure) {
            self.publish(ticket, Some(failure), wall, now);
        }
        wall >= retry_at && self.gates[&key].not_before.is_some_and(|at| now >= at)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use drive_core::download_jobs::DownloadFailureKind;
    fn ticket() -> DownloadTicket { DownloadTicket { id: "file".into(), version: "v1".into() } }
    fn failure(attempts: u32, retry_at: Option<u64>) -> DownloadFailure {
        DownloadFailure { schema: 1, kind: DownloadFailureKind::Transport, server_code: None, attempts, retry_at }
    }
    #[test]
    fn forward_wall_jump_cannot_shorten_a_published_delay() {
        let mut clock = RetryClock::default(); let ticket = ticket(); let f = failure(1, Some(102));
        let start = Instant::now();
        clock.publish(&ticket, Some(&f), 100, start);
        assert!(!clock.ready(&ticket, Some(&f), 500, start + Duration::from_millis(899)));
        assert!(!clock.ready(&ticket, Some(&f), 500, start + Duration::from_millis(1999)));
        assert!(clock.ready(&ticket, Some(&f), 500, start + Duration::from_secs(2)));
    }
    #[test]
    fn restart_remaining_delay_is_seeded_once_and_reset_releases_it() {
        let mut clock = RetryClock::default(); let ticket = ticket(); let f = failure(1, Some(102));
        let start = Instant::now();
        assert!(!clock.ready(&ticket, Some(&f), 101, start));
        assert!(!clock.ready(&ticket, Some(&f), 500, start + Duration::from_millis(999)));
        assert!(clock.ready(&ticket, Some(&f), 500, start + Duration::from_secs(1)));
        clock.publish(&ticket, Some(&f), 100, start);
        assert!(clock.ready(&ticket, None, 100, start));
        assert!(clock.gates.is_empty());
    }
    #[test]
    fn new_attempts_versions_and_terminal_failures_do_not_reuse_an_old_gate() {
        let mut clock = RetryClock::default(); let mut ticket = ticket(); let start = Instant::now();
        clock.publish(&ticket, Some(&failure(1, Some(102))), 100, start);
        let later = failure(2, Some(105));
        assert!(!clock.ready(&ticket, Some(&later), 101, start));
        assert!(!clock.ready(&ticket, Some(&later), 105, start + Duration::from_secs(3)));
        assert!(clock.ready(&ticket, Some(&later), 105, start + Duration::from_secs(4)));
        ticket.version = "v2".into();
        clock.publish(&ticket, Some(&failure(1, Some(102))), 100, start);
        assert_eq!(clock.gates.len(), 1);
        assert!(!clock.ready(&ticket, Some(&failure(5, None)), 900, start + Duration::from_secs(30)));
        assert!(clock.gates.is_empty());
    }
    #[test]
    fn backward_wall_jump_does_not_bypass_the_durable_deadline() {
        let mut clock = RetryClock::default(); let ticket = ticket(); let f = failure(1, Some(102));
        let start = Instant::now(); clock.publish(&ticket, Some(&f), 100, start);
        assert!(!clock.ready(&ticket, Some(&f), 101, start + Duration::from_secs(3)));
        assert!(clock.ready(&ticket, Some(&f), 102, start + Duration::from_secs(3)));
    }
}
