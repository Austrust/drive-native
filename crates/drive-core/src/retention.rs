//! Bounded retention for durable bookkeeping tables.
//!
//! `operations`, `namespace_ops`, `inbox` and the upload staging tables are
//! written by the sync engine, but only a bounded tail of them is ever
//! consulted: the pending queue, the uncertain-upload reconciler and the
//! diagnostic history. Without cleanup a store that uploads or renames files
//! keeps one row per operation for its whole lifetime, and every enumerating
//! query pays for that history.
//!
//! Every limit here is count-based, never wall-clock-based, so a prune is
//! deterministic. Rows are removed only when no query can observe them any
//! more:
//!
//! * `operations` in `superseded` state have no consumer at all;
//! * only the newest `committed` operation per node is consulted (the
//!   `unsubmitted` predicates are existence checks), so previous generations
//!   are dropped when the next one commits and once for legacy databases;
//! * terminal `namespace_ops` (committed/superseded by the namespace engine)
//!   are diagnostic only and keep the newest [`NAMESPACE_TERMINAL_RETENTION`]
//!   rows;
//! * applied `inbox` rows are deleted as soon as `apply_next` commits them;
//! * upload staging/origin/path rows whose operation no longer exists are
//!   orphans.
//!
//! Deleting journal sources must not look like a namespace change. Every
//! cleanup statement therefore runs through
//! [`crate::status_changes::without_journal`], which suppresses the TEMP
//! journal triggers for the duration of one transaction. The semantic journal
//! cursor and the namespace watermark stay exactly where they were, so no
//! projection (namespace index, desktop status, inode map) is invalidated by
//! retention.

use crate::{Result, Store};
use rusqlite::{OptionalExtension, Transaction, params};

/// Terminal `namespace_ops` rows retained for diagnostics. Older terminal rows
/// have no consumer: every namespace query filters
/// `state NOT IN ('committed','superseded')`.
pub(crate) const NAMESPACE_TERMINAL_RETENTION: i64 = 10_000;

/// Metadata key recording that the one-time legacy committed-operation dedupe
/// already ran on this store. `metadata` is not a journal source.
const COMMITTED_DEDUPE_FLAG: &str = "storage_v28_committed_deduped";

/// Metadata key recording that the upgrade-time storage cleanup already ran.
/// Bound history is maintained afterwards by commits, `apply_next` and the
/// runtime idle tick, so later opens do not rescan.
const STORAGE_PRUNED_FLAG: &str = "storage_v28_pruned";

/// Counts of the rows one [`Store::prune_storage`] call removed. All fields are
/// additive; the fields are exposed for the acceptance tests and diagnostics.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RetentionStats {
    /// Terminal `namespace_ops` rows removed beyond the retention window.
    pub namespace_ops: u64,
    /// Superseded `operations` rows removed.
    pub operations_superseded: u64,
    /// Previous committed `operations` generations removed (legacy dedupe).
    pub operations_committed: u64,
    /// Applied `inbox` receipts removed.
    pub inbox: u64,
    /// Applied or orphaned `engine_upload_stages` rows removed.
    pub upload_stages: u64,
    /// Orphaned `upload_origins` rows removed.
    pub upload_origins: u64,
    /// Orphaned `upload_paths` rows removed.
    pub upload_paths: u64,
}

impl Store {
    /// Remove durable bookkeeping rows that no query can observe any more.
    ///
    /// This is called at startup (which cleans stores written before schema 28)
    /// and periodically from the runtime's idle tick. It is idempotent and does
    /// not touch the semantic journal, the namespace watermark, node rows or
    /// any pending/uncertain operation.
    ///
    /// The legacy committed-operation dedupe is a full table scan, so it runs at
    /// most once per store and is recorded in `metadata`.
    pub fn prune_storage(&mut self) -> Result<RetentionStats> {
        let mut stats = RetentionStats::default();
        // `inbox` is not a journal source. Every `done=1` row has been applied
        // successfully; new rows are only inserted for unapplied events.
        stats.inbox = self.db.execute("DELETE FROM inbox WHERE done=1", [])? as u64;
        let dedupe = self
            .db
            .query_row(
                "SELECT value FROM metadata WHERE key=?1",
                [COMMITTED_DEDUPE_FLAG],
                |row| row.get::<_, String>(0),
            )
            .optional()?
            .is_none();
        let cleaned = crate::status_changes::without_journal(&mut self.db, |tx| {
            let mut stats = RetentionStats {
                inbox: stats.inbox,
                ..RetentionStats::default()
            };
            // No consumer reads superseded operations: they are historical
            // copies kept only to preserve the pending queue's view of a node
            // while a newer operation is active.
            stats.operations_superseded =
                tx.execute("DELETE FROM operations WHERE state='superseded'", [])? as u64;
            // Legacy stores may hold one committed row per successful upload of
            // a node. Only the newest committed row per node can be observed.
            if dedupe {
                stats.operations_committed = tx.execute(
                    "DELETE FROM operations WHERE state='committed'
                     AND rowid NOT IN (SELECT MAX(rowid) FROM operations
                                       WHERE state='committed' GROUP BY node)",
                    [],
                )? as u64;
            }
            // Keep the newest terminal namespace window; older terminal rows
            // are not read by any query. `OFFSET` names the (N+1)-th newest row,
            // so `seq <= cutoff` leaves exactly `NAMESPACE_TERMINAL_RETENTION`.
            stats.namespace_ops = tx.execute(
                "DELETE FROM namespace_ops WHERE state IN ('committed','superseded')
                 AND seq <= COALESCE((SELECT seq FROM namespace_ops
                                     WHERE state IN ('committed','superseded')
                                     ORDER BY seq DESC LIMIT 1 OFFSET ?1), -1)",
                params![NAMESPACE_TERMINAL_RETENTION],
            )? as u64;
            // Upload staging rows are consumed by the bridge while the
            // operation exists. Applied rows and rows whose operation vanished
            // cannot be claimed again.
            stats.upload_stages = tx.execute(
                "DELETE FROM engine_upload_stages
                 WHERE state='applied'
                    OR NOT EXISTS (SELECT 1 FROM operations o
                                   WHERE o.id=engine_upload_stages.operation)",
                [],
            )? as u64;
            stats.upload_origins = tx.execute(
                "DELETE FROM upload_origins
                 WHERE NOT EXISTS (SELECT 1 FROM operations o
                                   WHERE o.id=upload_origins.operation)",
                [],
            )? as u64;
            stats.upload_paths = tx.execute(
                "DELETE FROM upload_paths
                 WHERE NOT EXISTS (SELECT 1 FROM operations o
                                   WHERE o.id=upload_paths.operation)",
                [],
            )? as u64;
            Ok(stats)
        })?;
        if dedupe {
            self.db.execute(
                "INSERT OR REPLACE INTO metadata(key,value) VALUES(?1,'28')",
                [COMMITTED_DEDUPE_FLAG],
            )?;
        }
        stats.namespace_ops = cleaned.namespace_ops;
        stats.operations_superseded = cleaned.operations_superseded;
        stats.operations_committed = cleaned.operations_committed;
        stats.upload_stages = cleaned.upload_stages;
        stats.upload_origins = cleaned.upload_origins;
        stats.upload_paths = cleaned.upload_paths;
        Ok(stats)
    }

    /// Run the startup cleanup only once per store version.
    ///
    /// Rebuilding the duplicate-generation window and deleting old terminal
    /// history are cheap in steady state but a one-off scan of a large legacy
    /// database; a durable marker keeps every later open from paying it before
    /// the commit and idle paths have anything to remove.
    pub(crate) fn prune_storage_if_due(&mut self) -> Result<()> {
        let done: bool = self
            .db
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM metadata WHERE key=?1)",
                [STORAGE_PRUNED_FLAG],
                |row| row.get(0),
            )?;
        if done {
            return Ok(());
        }
        self.prune_storage()?;
        self.db.execute(
            "INSERT OR REPLACE INTO metadata(key,value) VALUES(?1,'28')",
            [STORAGE_PRUNED_FLAG],
        )?;
        Ok(())
    }
}

/// Remove previous committed generations of one node inside an operation's
/// commit transaction.
///
/// A node keeps at most one committed operation: the `unsubmitted` predicates
/// and the conflict diagnostics only ask whether *any* committed generation
/// exists. Dropping the rest keeps `operations` bounded without waiting for a
/// restart. The delete is bookkeeping, so it is hidden from the semantic
/// journal; a rollback of the commit restores the flag and every row.
pub(crate) fn drop_committed_generations(
    tx: &Transaction<'_>,
    node: &str,
    keep: &str,
) -> Result<u64> {
    crate::status_changes::without_journal_in(tx, || {
        Ok(tx.execute(
            "DELETE FROM operations
             WHERE node=?1 AND state='committed' AND id<>?2",
            params![node, keep],
        )? as u64)
    })
}

#[cfg(test)]
#[path = "retention_tests.rs"]
mod tests;
