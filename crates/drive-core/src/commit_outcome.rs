//! Unified vocabulary for the outcome of a server-side write commit (#9).
//!
//! Every write path — upload, rename/mkdir/remove/replace and download — must
//! eventually report exactly one of four outcomes: confirmed, conflict,
//! unknown-but-retryable, unknown-and-terminal. The current tree has no such
//! single exit: each journal (`operations`, `namespace_ops`, `transfers`,
//! `engine_bridge_intents`, `engine_upload_stages`, ...) maintains its own
//! claimed/uncertain/receipted/applied states and its own backoff ladder.
//!
//! This module is the type layer of tracer bullet TB1. It defines the shared
//! vocabulary, the durable diagnostic row shape and the unified retry ladder.
//! It does **not** change behaviour: no write path reads these types yet; TB2
//! persists diagnostics from `drive-runtime` and TB3 removes the remaining
//! silent reconcile branches. The durable home of the same data is schema v29:
//! the `outcome` / `outcome_category` / `last_error` / `next_retry_at` columns
//! on `operations`, `namespace_ops` and `transfers`, plus the read-only
//! `commit_diagnostics` view. Keep the enum spellings in sync with the `CHECK`
//! constraints in `state::migrate_v29` and with [`CommitOutcome::as_str`].
//!
//! Link to #14: legacy `uncertain` rows are backfilled as
//! `unknown_retryable`/`interrupted`. Once #14 introduces an explicit
//! `abandoned` terminal state for exhausted reconcile budgets, TB2 remaps those
//! rows to `unknown_fatal`/`reconcile_exhausted`; TB1 deliberately does not
//! emit or depend on the string `abandoned`.

use serde::{Deserialize, Serialize};

/// The only four allowed commit outcomes. There is deliberately no `Other`:
/// an unclassifiable result must be reduced to one of these upstream (any
/// failure that is not provably confirmed/conflict is unknown).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommitOutcome {
    /// The server confirmed the mutation, possibly under a different version.
    Confirmed,
    /// The remote changed first, or the server explicitly rejected the
    /// mutation: the local side must reconcile before anything is replayed.
    Conflict,
    /// The commit is unknown, but a later automatic attempt is justified.
    UnknownRetryable,
    /// The commit is unknown and must not be retried automatically. It stays
    /// visible for audit and manual retry/abandon.
    UnknownFatal,
}

impl CommitOutcome {
    pub const ALL: [Self; 4] = [
        Self::Confirmed,
        Self::Conflict,
        Self::UnknownRetryable,
        Self::UnknownFatal,
    ];

    /// Stable storage/status-interface spelling (also the `CHECK` value).
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Confirmed => "confirmed",
            Self::Conflict => "conflict",
            Self::UnknownRetryable => "unknown_retryable",
            Self::UnknownFatal => "unknown_fatal",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|outcome| outcome.as_str() == value)
    }

    /// True while the client still expects a later automatic retry.
    pub const fn is_open(self) -> bool {
        matches!(self, Self::UnknownRetryable)
    }

    /// True when the item must be surfaced to the user as needing action.
    pub const fn needs_attention(self) -> bool {
        matches!(self, Self::Conflict | Self::UnknownFatal)
    }
}

impl std::fmt::Display for CommitOutcome {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// Why a commit ended in an unknown or conflicting outcome. The category is
/// diagnostic only; it never replaces the durable journal state. Values are
/// stored in `outcome_category` (TEXT, no CHECK) so TB2/TB3 can extend the set
/// without another schema migration when a new write path appears.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommitCategory {
    /// The transport failed before or after the request was written.
    Transport,
    /// The request or response timed out.
    Timeout,
    /// The server returned a permission-related response (e.g. 0x9004); the
    /// mutation may or may not have been applied.
    PermissionUncertain,
    /// The server explicitly rejected the mutation.
    ServerRejected,
    /// The protocol contract was violated or an unknown response arrived.
    Protocol,
    /// The event cursor gap requires reinitialization (#10 owns the recovery).
    CursorGap,
    /// The server required a snapshot rebuild (#10 owns the recovery).
    SnapshotChanged,
    /// A local read/write of the source file failed before submission.
    LocalStorage,
    /// The durable reconcile budget is exhausted; manual action is required.
    ReconcileExhausted,
    /// A restart interrupted a submitted mutation; outcome is unknown.
    Interrupted,
    /// Terminal success (backfill only; live paths use `outcome='confirmed'`).
    Committed,
    /// The local intent was deliberately abandoned/superseded.
    LocalAbandoned,
}

impl CommitCategory {
    pub const ALL: [Self; 12] = [
        Self::Transport,
        Self::Timeout,
        Self::PermissionUncertain,
        Self::ServerRejected,
        Self::Protocol,
        Self::CursorGap,
        Self::SnapshotChanged,
        Self::LocalStorage,
        Self::ReconcileExhausted,
        Self::Interrupted,
        Self::Committed,
        Self::LocalAbandoned,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Transport => "transport",
            Self::Timeout => "timeout",
            Self::PermissionUncertain => "permission_uncertain",
            Self::ServerRejected => "server_rejected",
            Self::Protocol => "protocol",
            Self::CursorGap => "cursor_gap",
            Self::SnapshotChanged => "snapshot_changed",
            Self::LocalStorage => "local_storage",
            Self::ReconcileExhausted => "reconcile_exhausted",
            Self::Interrupted => "interrupted",
            Self::Committed => "committed",
            Self::LocalAbandoned => "local_abandoned",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|category| category.as_str() == value)
    }
}

impl std::fmt::Display for CommitCategory {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// One durable commit diagnosis. Mirrors the v29 `commit_diagnostics` view and
/// is the shape TB2 writes after every write-path branch. `next_retry_at` is a
/// unix-seconds timestamp; `0` means "no automatic retry scheduled".
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitDiagnostic {
    pub outcome: CommitOutcome,
    pub category: CommitCategory,
    pub last_error: Option<String>,
    pub attempts: u32,
    pub next_retry_at: i64,
}

impl CommitDiagnostic {
    /// Server confirmed the commit (possibly under a new version).
    pub fn confirmed() -> Self {
        Self {
            outcome: CommitOutcome::Confirmed,
            category: CommitCategory::Committed,
            last_error: None,
            attempts: 1,
            next_retry_at: 0,
        }
    }

    /// The remote changed first or the server rejected the mutation.
    pub fn conflict(last_error: Option<String>) -> Self {
        Self {
            outcome: CommitOutcome::Conflict,
            category: CommitCategory::ServerRejected,
            last_error,
            attempts: 1,
            next_retry_at: 0,
        }
    }

    /// Unknown, but a later automatic attempt is justified.
    pub fn unknown_retryable(
        category: CommitCategory,
        last_error: Option<String>,
        attempts: u32,
        next_retry_at: i64,
    ) -> Self {
        Self {
            outcome: CommitOutcome::UnknownRetryable,
            category,
            last_error,
            attempts,
            next_retry_at,
        }
    }

    /// Unknown and terminal until the user acts (or a manual retry is asked).
    pub fn unknown_fatal(
        category: CommitCategory,
        last_error: Option<String>,
        attempts: u32,
    ) -> Self {
        Self {
            outcome: CommitOutcome::UnknownFatal,
            category,
            last_error,
            attempts,
            next_retry_at: 0,
        }
    }

    pub const fn is_open(&self) -> bool {
        self.outcome.is_open()
    }

    pub const fn needs_attention(&self) -> bool {
        self.outcome.needs_attention()
    }
}

/// Unified backoff ladder (#9 §4.4). `attempts` is the number of automatic
/// attempts already recorded (0 = first failure). `Some(seconds)` schedules the
/// next automatic attempt; `None` means "do not retry automatically — surface
/// the item for manual retry/abandon".
pub fn retry_delay(category: CommitCategory, attempts: u32) -> Option<u32> {
    match category {
        // Connection-level failures: retry promptly, cap at 30s. Attention for
        // a persistently offline backend is decided by #10, not here.
        CommitCategory::Transport | CommitCategory::Timeout | CommitCategory::Interrupted => {
            Some(match attempts {
                0 => 0,
                1 => 2,
                2 => 4,
                3 => 8,
                4 => 16,
                _ => 30,
            })
        }
        // Permission responses: the server may have applied the mutation, so
        // probe slowly and stop after a bounded budget (#14).
        CommitCategory::PermissionUncertain => match attempts {
            0 => Some(10),
            1 => Some(30),
            2 => Some(120),
            3 => Some(300),
            4 => Some(600),
            _ => None,
        },
        // Explicit rejection or exhausted budget: manual only.
        CommitCategory::ServerRejected
        | CommitCategory::ReconcileExhausted
        | CommitCategory::Committed
        | CommitCategory::LocalAbandoned => None,
        // Contract/local failures: retry a few times, then ask the user.
        CommitCategory::Protocol | CommitCategory::LocalStorage => match attempts {
            0 => Some(60),
            1 => Some(300),
            2 => Some(900),
            _ => None,
        },
        // Recovery signals (#10): reinitialize promptly, then slow down. The
        // only outcomes that may still stop for the user are authentication,
        // certificate and root failures, which have their own categories.
        CommitCategory::CursorGap | CommitCategory::SnapshotChanged => Some(match attempts {
            0 => 0,
            1 => 2,
            2 => 5,
            3 => 10,
            4 => 30,
            _ => 60,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn outcome_round_trips_through_storage_spelling() {
        for outcome in CommitOutcome::ALL {
            assert_eq!(CommitOutcome::parse(outcome.as_str()), Some(outcome));
        }
        assert_eq!(CommitOutcome::parse("other"), None);
        assert_eq!(CommitOutcome::parse(""), None);
    }

    #[test]
    fn category_round_trips_through_storage_spelling() {
        for category in CommitCategory::ALL {
            assert_eq!(CommitCategory::parse(category.as_str()), Some(category));
        }
        assert_eq!(CommitCategory::parse("unknown"), None);
    }

    #[test]
    fn only_unknown_retryable_stays_open() {
        assert!(CommitOutcome::UnknownRetryable.is_open());
        assert!(!CommitOutcome::Confirmed.is_open());
        assert!(!CommitOutcome::Conflict.is_open());
        assert!(!CommitOutcome::UnknownFatal.is_open());
        assert!(CommitOutcome::Conflict.needs_attention());
        assert!(CommitOutcome::UnknownFatal.needs_attention());
        assert!(!CommitOutcome::UnknownRetryable.needs_attention());
        assert!(!CommitOutcome::Confirmed.needs_attention());
    }

    #[test]
    fn retry_ladder_matches_the_unified_table() {
        // Transport/Timeout/Interrupted: prompt, then capped at 30s.
        assert_eq!(retry_delay(CommitCategory::Transport, 0), Some(0));
        assert_eq!(retry_delay(CommitCategory::Timeout, 1), Some(2));
        assert_eq!(retry_delay(CommitCategory::Interrupted, 2), Some(4));
        assert_eq!(retry_delay(CommitCategory::Transport, 4), Some(16));
        assert_eq!(retry_delay(CommitCategory::Transport, 9), Some(30));
        // Permission: 10/30/120/300/600, then terminal.
        assert_eq!(
            retry_delay(CommitCategory::PermissionUncertain, 0),
            Some(10)
        );
        assert_eq!(
            retry_delay(CommitCategory::PermissionUncertain, 4),
            Some(600)
        );
        assert_eq!(retry_delay(CommitCategory::PermissionUncertain, 5), None);
        // Manual-only categories never schedule a retry.
        assert_eq!(retry_delay(CommitCategory::ServerRejected, 0), None);
        assert_eq!(retry_delay(CommitCategory::ReconcileExhausted, 0), None);
        assert_eq!(retry_delay(CommitCategory::Committed, 0), None);
        // Protocol/local storage: 60/300/900, then terminal.
        assert_eq!(retry_delay(CommitCategory::Protocol, 0), Some(60));
        assert_eq!(retry_delay(CommitCategory::LocalStorage, 2), Some(900));
        assert_eq!(retry_delay(CommitCategory::Protocol, 3), None);
        // Recovery signals keep retrying (attention is #10's decision).
        assert_eq!(retry_delay(CommitCategory::CursorGap, 0), Some(0));
        assert_eq!(retry_delay(CommitCategory::SnapshotChanged, 4), Some(30));
        assert_eq!(retry_delay(CommitCategory::CursorGap, 6), Some(60));
    }

    #[test]
    fn diagnostics_carry_terminal_flags() {
        let confirmed = CommitDiagnostic::confirmed();
        assert_eq!(confirmed.outcome, CommitOutcome::Confirmed);
        assert_eq!(confirmed.category, CommitCategory::Committed);
        assert!(!confirmed.needs_attention());
        let fatal = CommitDiagnostic::unknown_fatal(
            CommitCategory::ReconcileExhausted,
            Some("reconcile attempts exhausted".into()),
            5,
        );
        assert!(fatal.needs_attention());
        assert_eq!(fatal.next_retry_at, 0);
        let retryable = CommitDiagnostic::unknown_retryable(
            CommitCategory::Transport,
            Some("transport".into()),
            2,
            16,
        );
        assert!(retryable.is_open());
        assert_eq!(retryable.next_retry_at, 16);
    }
}
