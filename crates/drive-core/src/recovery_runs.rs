//! Persistent journal for automatic recovery runs (#10).
//!
//! Schema v29 already defines the `recovery_runs` table (added by #9 TB1, see
//! `state.rs`); this module is its only writer. A run starts when the runtime
//! decides to rebuild the local snapshot in response to a `RecoveryRequired`
//! signal, records every automatic attempt, and ends either `recovered` or
//! `needs_attention` once the bounded retry budget is spent. Desktop clients
//! consume the journal through the additive `recovery` object, so they can
//! tell "retrying", "recovering" and "needs attention" apart.

use rusqlite::{params, OptionalExtension};
use serde::Serialize;

use crate::state::{Error, Result, Store};

/// Automatic attempts allowed before a run is surfaced for a human.
pub const MAX_RECOVERY_ATTEMPTS: u32 = 3;

/// Cause names shared with the `recovery_runs` CHECK constraint.
pub const CAUSE_INITIAL_INDEX: &str = "initial_index";
pub const CAUSE_PERMISSION_RECHECK: &str = "permission_recheck";
pub const CAUSE_CURSOR_INVALIDATED: &str = "cursor_invalidated";
pub const CAUSE_FILTERED_TYPE_CHANGED: &str = "filtered_type_changed";
pub const CAUSE_SNAPSHOT_CHANGED: &str = "snapshot_changed";
pub const CAUSE_RESCAN_LATER: &str = "rescan_later";
pub const CAUSE_MERGE_MODE: &str = "merge_mode";
pub const CAUSE_ROOT_UNAVAILABLE: &str = "root_unavailable";

/// Every cause accepted by the schema CHECK constraint.
pub const CAUSES: [&str; 8] = [
    CAUSE_INITIAL_INDEX,
    CAUSE_PERMISSION_RECHECK,
    CAUSE_CURSOR_INVALIDATED,
    CAUSE_FILTERED_TYPE_CHANGED,
    CAUSE_SNAPSHOT_CHANGED,
    CAUSE_RESCAN_LATER,
    CAUSE_MERGE_MODE,
    CAUSE_ROOT_UNAVAILABLE,
];

/// Status names shared with the `recovery_runs` CHECK constraint.
pub const STATUS_RETRYING: &str = "retrying";
pub const STATUS_RECOVERING: &str = "recovering";
pub const STATUS_NEEDS_ATTENTION: &str = "needs_attention";
pub const STATUS_RECOVERED: &str = "recovered";

/// Every status accepted by the schema CHECK constraint.
pub const STATUSES: [&str; 4] = [
    STATUS_RETRYING,
    STATUS_RECOVERING,
    STATUS_NEEDS_ATTENTION,
    STATUS_RECOVERED,
];

/// Worker error-category prefix that marks a `RecoveryRequired` signal (#10).
///
/// The token travels through the existing content-free worker category channel
/// (`Response::Failed{category}`); the supervisor and the pull path classify it
/// as recoverable instead of turning it into a human-attention error.
pub const TOKEN_PREFIX: &str = "native_recovery_required:";

/// A rescan is required; rebuild the snapshot and keep the run automatic.
pub const TOKEN_RESCAN_LATER: &str = "native_recovery_required:rescan_later";
/// The server entered merge mode; rebuild the snapshot and label the cause.
pub const TOKEN_MERGE_MODE: &str = "native_recovery_required:merge_mode";
/// The committed snapshot must be replaced because the local copy drifted.
pub const TOKEN_SNAPSHOT_CHANGED: &str = "native_recovery_required:snapshot_changed";
/// The event journal cursor cannot be closed; rebuild the snapshot.
pub const TOKEN_CURSOR_GAP: &str = "native_recovery_required:cursor_gap";

/// Map a worker category token to the durable recovery cause.
pub fn token_cause(token: &str) -> Option<&'static str> {
    match token {
        TOKEN_RESCAN_LATER => Some(CAUSE_RESCAN_LATER),
        TOKEN_MERGE_MODE => Some(CAUSE_MERGE_MODE),
        TOKEN_SNAPSHOT_CHANGED => Some(CAUSE_SNAPSHOT_CHANGED),
        TOKEN_CURSOR_GAP => Some(CAUSE_CURSOR_INVALIDATED),
        _ => None,
    }
}

/// Map a category token suffix (`rescan_later`, `merge_mode`,
/// `snapshot_changed`, `cursor_gap`) to the durable recovery cause.
pub fn reason_cause(reason: &str) -> Option<&'static str> {
    match reason {
        "rescan_later" => Some(CAUSE_RESCAN_LATER),
        "merge_mode" => Some(CAUSE_MERGE_MODE),
        "snapshot_changed" => Some(CAUSE_SNAPSHOT_CHANGED),
        "cursor_gap" => Some(CAUSE_CURSOR_INVALIDATED),
        _ => None,
    }
}

/// Map a durable recovery cause back to its worker category token.
pub fn cause_token(cause: &str) -> Option<&'static str> {
    match cause {
        CAUSE_RESCAN_LATER => Some(TOKEN_RESCAN_LATER),
        CAUSE_MERGE_MODE => Some(TOKEN_MERGE_MODE),
        CAUSE_SNAPSHOT_CHANGED => Some(TOKEN_SNAPSHOT_CHANGED),
        CAUSE_CURSOR_INVALIDATED => Some(TOKEN_CURSOR_GAP),
        _ => None,
    }
}

/// Milliseconds since the Unix epoch, saturating instead of panicking.
pub(crate) fn unix_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|elapsed| i64::try_from(elapsed.as_millis()).unwrap_or(i64::MAX))
        .unwrap_or(0)
}

/// One row of the `recovery_runs` journal.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RecoveryRun {
    pub id: i64,
    pub cause: String,
    pub status: String,
    pub attempts: u32,
    pub last_error: Option<String>,
    pub next_retry_at: i64,
    pub started_at: i64,
    pub updated_at: i64,
}

impl RecoveryRun {
    fn from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Self> {
        let attempts: i64 = row.get(3)?;
        Ok(Self {
            id: row.get(0)?,
            cause: row.get(1)?,
            status: row.get(2)?,
            attempts: u32::try_from(attempts).unwrap_or(u32::MAX),
            last_error: row.get(4)?,
            next_retry_at: row.get(5)?,
            started_at: row.get(6)?,
            updated_at: row.get(7)?,
        })
    }
}

const COLUMNS: &str = "id,cause,status,attempts,last_error,next_retry_at,started_at,updated_at";

fn validate_cause(cause: &str) -> Result<()> {
    if CAUSES.contains(&cause) {
        Ok(())
    } else {
        Err(Error::State(format!("invalid recovery cause: {cause}")))
    }
}

fn validate_status(status: &str) -> Result<()> {
    if STATUSES.contains(&status) {
        Ok(())
    } else {
        Err(Error::State(format!("invalid recovery status: {status}")))
    }
}

impl Store {
    /// Open (or resume) the single automatic recovery run.
    ///
    /// `recovery_runs` is a single-flight journal: while a run is open it is
    /// returned unchanged instead of stacking a second row. `recovery_runs`
    /// rows are only written under the existing store writer lock, so this
    /// method does not need a transaction of its own.
    pub fn begin_recovery_run(&self, cause: &str) -> Result<RecoveryRun> {
        validate_cause(cause)?;
        let now = unix_ms();
        if let Some(mut run) = self.open_recovery_run()? {
            if run.cause == cause && run.status != STATUS_RECOVERING {
                self.db.execute(
                    "UPDATE recovery_runs SET status=?1, updated_at=?2 WHERE id=?3",
                    params![STATUS_RECOVERING, now, run.id],
                )?;
                run.status = STATUS_RECOVERING.into();
                run.updated_at = now;
            }
            return Ok(run);
        }
        self.db.execute(
            "INSERT INTO recovery_runs(cause, status, attempts, started_at, updated_at)
             VALUES(?1, ?2, 0, ?3, ?3)",
            params![cause, STATUS_RECOVERING, now],
        )?;
        Ok(RecoveryRun {
            id: self.db.last_insert_rowid(),
            cause: cause.into(),
            status: STATUS_RECOVERING.into(),
            attempts: 0,
            last_error: None,
            next_retry_at: 0,
            started_at: now,
            updated_at: now,
        })
    }

    /// The currently open run (`recovering` or `retrying`), if any.
    pub fn open_recovery_run(&self) -> Result<Option<RecoveryRun>> {
        Ok(self
            .db
            .query_row(
                &format!(
                    "SELECT {COLUMNS} FROM recovery_runs
                     WHERE status IN ('{STATUS_RETRYING}','{STATUS_RECOVERING}')
                     ORDER BY id DESC LIMIT 1"
                ),
                [],
                RecoveryRun::from_row,
            )
            .optional()?)
    }

    /// Record the outcome of one automatic attempt.
    pub fn update_recovery_run(
        &self,
        id: i64,
        status: &str,
        attempts: u32,
        last_error: Option<&str>,
        next_retry_at: i64,
    ) -> Result<()> {
        validate_status(status)?;
        self.db.execute(
            "UPDATE recovery_runs
             SET status=?1, attempts=?2, last_error=?3, next_retry_at=?4, updated_at=?5
             WHERE id=?6",
            params![
                status,
                i64::from(attempts),
                last_error,
                next_retry_at,
                unix_ms(),
                id
            ],
        )?;
        Ok(())
    }

    /// Close the run as `recovered` or `needs_attention`.
    pub fn finish_recovery_run(&self, id: i64, status: &str) -> Result<()> {
        validate_status(status)?;
        self.db.execute(
            "UPDATE recovery_runs SET status=?1, updated_at=?2 WHERE id=?3",
            params![status, unix_ms(), id],
        )?;
        Ok(())
    }

    /// Every open run, oldest first.
    pub fn open_recovery_runs(&self) -> Result<Vec<RecoveryRun>> {
        let mut query = self.db.prepare(&format!(
            "SELECT {COLUMNS} FROM recovery_runs
             WHERE status IN ('{STATUS_RETRYING}','{STATUS_RECOVERING}')
             ORDER BY id"
        ))?;
        let rows = query.query_map([], RecoveryRun::from_row)?;
        let mut runs = Vec::new();
        for row in rows {
            runs.push(row?);
        }
        Ok(runs)
    }

    /// The newest run in any status.
    pub fn latest_recovery_run(&self) -> Result<Option<RecoveryRun>> {
        Ok(self
            .db
            .query_row(
                &format!("SELECT {COLUMNS} FROM recovery_runs ORDER BY id DESC LIMIT 1"),
                [],
                RecoveryRun::from_row,
            )
            .optional()?)
    }

    /// Keep the newest `keep` finished runs, preserving every open run.
    pub fn prune_recovery_runs(&self, keep: u32) -> Result<usize> {
        let removed = self.db.execute(
            &format!(
                "DELETE FROM recovery_runs
                 WHERE status NOT IN ('{STATUS_RETRYING}','{STATUS_RECOVERING}')
                   AND id NOT IN (
                     SELECT id FROM recovery_runs
                     WHERE status NOT IN ('{STATUS_RETRYING}','{STATUS_RECOVERING}')
                     ORDER BY id DESC LIMIT ?1
                   )"
            ),
            params![i64::from(keep)],
        )?;
        Ok(removed)
    }
}
