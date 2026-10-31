//! Paged read-only WAL initialization, deliberately separate from publication.
//! A completed snapshot is historical: its journal cursor still needs catch-up.
use crate::{
    Error, SharedStore,
    shared_store::CommitState,
    status_changes::{JournalWatermark, StatusCursor},
    status_projection::{self, ProjectionBudget, RebuildReason, StatusAtom, StatusSnapshot, Stop},
};
use rusqlite::{Connection, OpenFlags, OptionalExtension};
use std::{
    path::PathBuf,
    sync::TryLockError,
    time::{Duration, Instant},
};

#[derive(Clone, Copy, Debug)]
pub struct SnapshotBudget {
    pub max_nodes: usize,
    /// Cumulative logical output plus temporary projection allowances, not RSS.
    pub max_bytes: usize,
    pub max_work: usize,
    pub max_elapsed: Duration,
    pub page_nodes: usize,
}
impl Default for SnapshotBudget {
    fn default() -> Self {
        Self {
            // Match IndexBudget's node limit; do not reuse small delta budgets.
            max_nodes: 500_000,
            max_bytes: 512 * 1024 * 1024,
            max_work: 64_000_000,
            max_elapsed: Duration::from_secs(120),
            page_nodes: 512,
        }
    }
}
impl SnapshotBudget {
    fn validate(self) -> Result<(), SnapshotError> {
        if self.max_nodes == 0
            || self.max_nodes > 500_000
            || self.max_bytes == 0
            || self.max_bytes > 1024 * 1024 * 1024
            || self.max_work == 0
            || self.max_work > 128_000_000
            || self.max_elapsed.is_zero()
            || self.max_elapsed > Duration::from_secs(300)
            || self.page_nodes == 0
            || self.page_nodes > 4096
        {
            return Err(SnapshotError::InvalidBudget);
        }
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SnapshotError {
    #[error("Store is busy; retry snapshot initialization later")]
    Busy,
    #[error("Store boundary is unverified or poisoned")]
    Unverified,
    #[error("snapshot requires a file-backed WAL database")]
    NotWal,
    #[error("invalid status snapshot budget")]
    InvalidBudget,
    #[error("status snapshot budget exhausted")]
    Budget,
    #[error("status snapshot has invalid visible ancestry")]
    InvalidGraph,
    #[error("status snapshot is not fully read")]
    Incomplete,
    #[error("status snapshot is already closed")]
    Closed,
    #[error(transparent)]
    Store(#[from] Error),
    #[error(transparent)]
    Sql(#[from] rusqlite::Error),
}

/// Identity of the single writer serialization domain, not a server identity.
/// The journal epoch changes on Store reopen/replacement, even at the same path.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SnapshotSource {
    pub database: PathBuf,
    pub journal_epoch: String,
    pub commit: CommitState,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SnapshotProgress {
    pub scanned_nodes: usize,
    pub visible_nodes: usize,
    pub accounted_bytes: usize,
    pub complete: bool,
}
#[derive(Debug)]
pub struct CompletedSnapshot {
    pub source: SnapshotSource,
    pub snapshot: StatusSnapshot,
}

/// Owns a SQLite read transaction, never a Store guard. Drive it on a background
/// owner and drop it on cancellation; do not retain an idle reader across waits.
/// Every page checks the cumulative deadline. No atoms escape before completion.
pub struct StatusSnapshotReader {
    db: Option<Connection>,
    source: SnapshotSource,
    watermark: JournalWatermark,
    budget: SnapshotBudget,
    started: Instant,
    after: Option<String>,
    atoms: Vec<StatusAtom>,
    scanned: usize,
    bytes: usize,
    work: usize,
    complete: bool,
}

impl SharedStore {
    /// Uses try_lock for Store ownership and zero SQLite busy timeout. Local
    /// canonicalize/open/first-read I/O still occurs under the same boundary as
    /// the TEMP journal capture; this is not a wall-clock latency guarantee.
    pub fn try_begin_status_snapshot(
        &self,
        budget: SnapshotBudget,
    ) -> Result<StatusSnapshotReader, SnapshotError> {
        budget.validate()?;
        let started = Instant::now();
        let store = match self.try_lock() {
            Ok(guard) => guard,
            Err(TryLockError::WouldBlock) => return Err(SnapshotError::Busy),
            Err(TryLockError::Poisoned(_)) => return Err(SnapshotError::Unverified),
        };
        let commit = self.change_state();
        if !commit.verified || !store.db.is_autocommit() {
            return Err(SnapshotError::Unverified);
        }
        let watermark = store.status_change_watermark()?;
        let path = store
            .db
            .path()
            .filter(|s| !s.is_empty())
            .ok_or(SnapshotError::NotWal)?;
        let database = std::fs::canonicalize(path).map_err(Error::from)?;
        let db = Connection::open_with_flags(
            &database,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )?;
        db.busy_timeout(Duration::ZERO)?;
        db.pragma_update(None, "query_only", true)?;
        let mode: String = db.pragma_query_value(None, "journal_mode", |r| r.get(0))?;
        if !mode.eq_ignore_ascii_case("wal") {
            return Err(SnapshotError::NotWal);
        }
        db.execute_batch("BEGIN DEFERRED")?;
        // BEGIN alone only disables autocommit. Reading the real main btree,
        // including an empty table, establishes the WAL end mark before Store
        // unlock. SELECT 1 or a TEMP-only read cannot establish that boundary.
        let _: Option<i64> = db
            .query_row(
                "SELECT length(id) FROM main.nodes ORDER BY id LIMIT 1",
                [],
                |r| r.get(0),
            )
            .optional()?;
        if started.elapsed() > budget.max_elapsed {
            return Err(SnapshotError::Budget);
        }
        let source = SnapshotSource {
            database,
            journal_epoch: watermark.cursor.epoch.clone(),
            commit,
        };
        drop(store);
        Ok(StatusSnapshotReader {
            db: Some(db),
            source,
            watermark,
            budget,
            started,
            after: None,
            atoms: Vec::new(),
            scanned: 0,
            bytes: 0,
            work: 0,
            complete: false,
        })
    }
}
impl StatusSnapshotReader {
    pub fn source(&self) -> &SnapshotSource {
        &self.source
    }
    pub fn cursor(&self) -> &StatusCursor {
        &self.watermark.cursor
    }
    pub fn progress(&self) -> SnapshotProgress {
        SnapshotProgress {
            scanned_nodes: self.scanned,
            visible_nodes: self.atoms.len(),
            accounted_bytes: self.bytes,
            complete: self.complete,
        }
    }
    pub fn next_page(&mut self) -> Result<SnapshotProgress, SnapshotError> {
        let result = self.read_page();
        if result.is_err() {
            self.close();
            self.atoms = Vec::new();
            self.complete = false;
        }
        result
    }
    fn read_page(&mut self) -> Result<SnapshotProgress, SnapshotError> {
        let db = self.db.as_ref().ok_or(SnapshotError::Closed)?;
        let remaining = self
            .budget
            .max_elapsed
            .checked_sub(self.started.elapsed())
            .filter(|d| !d.is_zero())
            .ok_or(SnapshotError::Budget)?;
        if self.complete {
            return Ok(self.progress());
        }
        let bytes = self
            .budget
            .max_bytes
            .checked_sub(self.bytes)
            .filter(|n| *n > 0)
            .ok_or(SnapshotError::Budget)?;
        let work = self
            .budget
            .max_work
            .checked_sub(self.work)
            .filter(|n| *n > 0)
            .ok_or(SnapshotError::Budget)?;
        let page = status_projection::snapshot_page(
            db,
            self.after.as_deref(),
            self.budget.page_nodes,
            ProjectionBudget {
                max_nodes: self.budget.max_nodes,
                max_bytes: bytes,
                max_work: work,
                max_elapsed: remaining,
                ..ProjectionBudget::default()
            },
        )
        .map_err(|error| match error {
            Stop::Rebuild(RebuildReason::Budget) => SnapshotError::Budget,
            Stop::Rebuild(_) => SnapshotError::InvalidGraph,
            Stop::Error(error) => SnapshotError::Store(error),
        })?;
        let scanned = self
            .scanned
            .checked_add(page.scanned)
            .ok_or(SnapshotError::Budget)?;
        if scanned > self.budget.max_nodes || self.started.elapsed() > self.budget.max_elapsed {
            return Err(SnapshotError::Budget);
        }
        self.scanned = scanned;
        self.bytes += page.bytes;
        self.work += page.work;
        if page.last.is_some() {
            self.after = page.last;
        }
        self.atoms.extend(page.atoms);
        self.complete = page.exhausted;
        Ok(self.progress())
    }
    /// Only the captured cursor is attached. The caller must catch up on the
    /// writer's journal, handling gap/epoch changes before publishing readiness.
    pub fn finish(mut self) -> Result<CompletedSnapshot, SnapshotError> {
        if self.db.is_none() {
            return Err(SnapshotError::Closed);
        }
        if !self.complete {
            return Err(SnapshotError::Incomplete);
        }
        if self.started.elapsed() > self.budget.max_elapsed {
            return Err(SnapshotError::Budget);
        }
        self.close();
        Ok(CompletedSnapshot {
            source: self.source.clone(),
            snapshot: StatusSnapshot {
                through: self.watermark.cursor.clone(),
                atoms: std::mem::take(&mut self.atoms),
            },
        })
    }
    pub fn cancel(mut self) {
        self.close();
    }
    fn close(&mut self) {
        if let Some(db) = self.db.take() {
            let _ = db.execute_batch("ROLLBACK");
            // Closing is definitive even if explicit ROLLBACK reports an error.
            drop(db);
        }
    }
}
impl Drop for StatusSnapshotReader {
    fn drop(&mut self) {
        self.close();
    }
}

#[cfg(test)]
#[path = "status_snapshot_tests.rs"]
mod tests;
