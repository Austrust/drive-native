//! Desktop status sampling used by the filesystem daemon.
//!
//! The daemon polls a handful of counters on every tick. The same sample also
//! decides which desktop signals are emitted: `status_changed` when anything
//! observable moved, and `status_changed2` with `overview_changed` when the GUI
//! must re-read `GetOverview`/`GetIssues` (navigation counts and tray badge).

/// One tick of the fields a desktop client can observe.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StatusSample {
    /// Revision of the control plane (mount state, paused flags, ...).
    pub control_revision: u64,
    /// Queued/active download counter.
    pub downloads: usize,
    /// Number of active download entries with a transfer in flight.
    pub active_downloads: usize,
    /// Last surfaced transfer error, if any.
    pub last_error: Option<String>,
    /// Monotonic revision of transfer progress.
    pub transfer_revision: u64,
    /// Committed status watermark: only committed namespace/status atoms move
    /// it, unlike the total change counter.
    pub commit_revision: u64,
    /// Whether the committed status projection is verified.
    pub commit_verified: bool,
    /// Revision of the automatic recovery journal (#10). A change forces the
    /// desktop to re-read the additive `recovery` overview object.
    pub recovery_revision: u64,
}

impl StatusSample {
    /// True when the sample differs at all; the daemon then emits
    /// `status_changed` and evaluates the revisioned status delta.
    pub fn differs(&self, previous: &Self) -> bool {
        self != previous
    }

    /// Whether `GetOverview`/`GetIssues` may report different counts.
    ///
    /// `commit_revision` is the committed status watermark; background
    /// reconciliation advances it without touching the other counters, and the
    /// overview projects exactly that status state. Omitting it here leaves the
    /// tray badge and navigation counts stale after pending/uncertain items are
    /// cleared (#53).
    pub fn overview_changed(&self, previous: &Self) -> bool {
        self.control_revision != previous.control_revision
            || self.downloads != previous.downloads
            || self.active_downloads != previous.active_downloads
            || self.last_error != previous.last_error
            || self.transfer_revision != previous.transfer_revision
            || self.commit_revision != previous.commit_revision
            || self.commit_verified != previous.commit_verified
            || self.recovery_revision != previous.recovery_revision
    }

    /// Whether the committed status journal watermark moved; only then does the
    /// daemon have a revisioned delta to emit.
    pub fn semantic_changed(&self, previous: &Self) -> bool {
        self.commit_revision != previous.commit_revision
            || self.commit_verified != previous.commit_verified
    }
}

#[cfg(test)]
mod tests {
    use super::StatusSample;

    fn sample() -> StatusSample {
        StatusSample {
            control_revision: 3,
            downloads: 0,
            active_downloads: 0,
            last_error: None,
            transfer_revision: 11,
            commit_revision: 7,
            commit_verified: true,
            recovery_revision: 0,
        }
    }

    #[test]
    fn semantic_watermark_advance_requires_an_overview_refresh() {
        // #53: reconciliation clears pending/uncertain status atoms and only
        // advances the committed watermark. The GUI must re-read GetOverview so
        // the tray badge and navigation counts return to zero.
        let previous = sample();
        let mut current = sample();
        current.commit_revision += 1;
        assert!(current.differs(&previous));
        assert!(current.semantic_changed(&previous));
        assert!(
            current.overview_changed(&previous),
            "a committed status watermark change must refresh the overview"
        );
    }

    #[test]
    fn identical_ticks_emit_nothing() {
        let previous = sample();
        let current = sample();
        assert!(!current.differs(&previous));
        assert!(!current.semantic_changed(&previous));
        assert!(!current.overview_changed(&previous));
    }

    #[test]
    fn overview_only_counters_refresh_without_a_semantic_delta() {
        let previous = sample();
        let mutations: [fn(&mut StatusSample); 4] = [
            |sample| sample.downloads += 1,
            |sample| sample.active_downloads += 1,
            |sample| sample.transfer_revision += 1,
            |sample| sample.last_error = Some("network".into()),
        ];
        for mutate in mutations {
            let mut current = sample();
            mutate(&mut current);
            assert!(current.differs(&previous));
            assert!(current.overview_changed(&previous));
            assert!(!current.semantic_changed(&previous));
        }
    }

    #[test]
    fn control_revision_alone_refreshes_overview_without_a_delta() {
        let previous = sample();
        let mut current = sample();
        current.control_revision += 1;
        assert!(current.overview_changed(&previous));
        assert!(!current.semantic_changed(&previous));
    }

    #[test]
    fn commit_verification_change_is_semantic_and_refreshes_overview() {
        let previous = sample();
        let mut current = sample();
        current.commit_verified = false;
        assert!(current.semantic_changed(&previous));
        assert!(current.overview_changed(&previous));
    }

    #[test]
    fn recovery_revision_refreshes_overview_without_a_semantic_delta() {
        // #10: automatic recovery begins/retries/ends without changing the
        // committed status watermark, so the sample must carry its own revision
        // or the GUI keeps a stale "recovering"/"needs attention" object.
        let previous = sample();
        let mut current = sample();
        current.recovery_revision += 1;
        assert!(current.differs(&previous));
        assert!(current.overview_changed(&previous));
        assert!(!current.semantic_changed(&previous));
    }
}
