//! Durable commit diagnostics (#9b TB2).
//!
//! TB1 added the storage columns (`outcome`, `outcome_category`, `last_error`,
//! `next_retry_at`, `updated_at`) to `operations`, `namespace_ops` and
//! `transfers`, plus the read-only `commit_diagnostics` view. TB2 writes them
//! from the same drive-core transaction that changes the operation state, so a
//! restart can never lose the reason an operation is waiting, and exposes the
//! rows through [`list`], [`open_unknown`] and [`needs_attention`].
//!
//! `items` may include `abandoned` rows (the terminal #14 state) so the user can
//! inspect why automatic reconciliation stopped, but `abandoned` is a terminal
//! queue outcome: it is not counted as open-unknown and does not force attention.

use rusqlite::{params, Connection};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::commit_outcome::{CommitCategory, CommitDiagnostic, CommitOutcome};
use crate::{Error, Result, Store};

/// One durable diagnostics row. Mirrors the `commit_diagnostics` view.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitDiagnosticRecord {
    pub scope: String,
    pub operation: String,
    pub node: Option<String>,
    pub kind: String,
    pub state: String,
    pub outcome: CommitOutcome,
    pub category: CommitCategory,
    pub last_error: Option<String>,
    pub attempts: u32,
    pub next_retry_at: i64,
    pub updated_at: i64,
}

pub(crate) fn unix_now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| i64::try_from(duration.as_secs()).unwrap_or(i64::MAX))
        .unwrap_or(0)
}

fn invalid(message: String) -> Error {
    Error::State(message)
}

/// Write one diagnostic row inside the caller's transaction. `table` is the
/// journal table that owns the commitment (`operations`, `namespace_ops` or
/// `transfers`); the caller has already checked that `operation` exists.
///
/// #57: the durable attempt counter is written here as well, and only ever
/// moves forward (`MAX`). A restart clears the process-local reconciliation
/// map, so `namespace_ops.attempts` is the only counter that can describe the
/// real retry history (and the one `commit_diagnostics`/`GetIssues`/CLI read).
/// For `operations` the projection reads `reconcile_attempts`, which the upload
/// reconcile budget already owns; the same forward-only write keeps the two in
/// agreement without letting a terminal `confirmed`/`conflict` diagnostic
/// rewind or inflate the budget.
pub(crate) fn record(
    tx: &Connection,
    table: &str,
    operation: &str,
    diagnostic: &CommitDiagnostic,
) -> Result<()> {
    let outcome = diagnostic.outcome.as_str();
    let category = diagnostic.category.as_str();
    let error = diagnostic.last_error.as_deref();
    let attempts = i64::from(diagnostic.attempts);
    let changed = match table {
        "operations" => tx.execute(
            "UPDATE operations SET outcome=?2, outcome_category=?3, last_error=?4,
                    next_retry_at=?5, updated_at=unixepoch(),
                    reconcile_attempts=CASE
                        WHEN ?2 IN ('unknown_retryable','unknown_fatal')
                        THEN MAX(reconcile_attempts,?6) ELSE reconcile_attempts END
             WHERE id=?1",
            params![
                operation,
                outcome,
                category,
                error,
                diagnostic.next_retry_at,
                attempts
            ],
        )?,
        "namespace_ops" => tx.execute(
            "UPDATE namespace_ops SET outcome=?2, outcome_category=?3, last_error=?4,
                    next_retry_at=?5, updated_at=unixepoch(),
                    attempts=MAX(attempts,?6) WHERE id=?1",
            params![
                operation,
                outcome,
                category,
                error,
                diagnostic.next_retry_at,
                attempts
            ],
        )?,
        "transfers" => tx.execute(
            "UPDATE transfers SET outcome=?2, outcome_category=?3, last_error=?4,
                    next_retry_at=?5 WHERE id=?1",
            params![
                operation,
                outcome,
                category,
                error,
                diagnostic.next_retry_at
            ],
        )?,
        _ => return Err(invalid(format!("unknown commit journal {table}"))),
    };
    if changed != 1 {
        return Err(invalid(format!(
            "commit diagnostic target {operation} is missing from {table}"
        )));
    }
    Ok(())
}

/// Remove the diagnostic of an operation that is provably not committed (a
/// permission refusal returned it to `pending`) or whose budget was reset.
pub(crate) fn clear_tx(tx: &Connection, table: &str, operation: &str) -> Result<()> {
    let changed = match table {
        "operations" => tx.execute(
            "UPDATE operations SET outcome=NULL, outcome_category=NULL, last_error=NULL,
                    next_retry_at=0, updated_at=unixepoch() WHERE id=?1",
            [operation],
        )?,
        "namespace_ops" => tx.execute(
            "UPDATE namespace_ops SET outcome=NULL, outcome_category=NULL, last_error=NULL,
                    next_retry_at=0, updated_at=unixepoch() WHERE id=?1",
            [operation],
        )?,
        "transfers" => tx.execute(
            "UPDATE transfers SET outcome=NULL, outcome_category=NULL, last_error=NULL,
                    next_retry_at=0 WHERE id=?1",
            [operation],
        )?,
        _ => return Err(invalid(format!("unknown commit journal {table}"))),
    };
    if changed != 1 {
        return Err(invalid(format!(
            "commit diagnostic target {operation} is missing from {table}"
        )));
    }
    Ok(())
}

/// All durable diagnostics, oldest retry first.
pub fn list(store: &Store, limit: usize) -> Result<Vec<CommitDiagnosticRecord>> {
    let limit = i64::try_from(limit.clamp(1, 500)).unwrap_or(500);
    let mut statement = store.db.prepare_cached(
        "SELECT scope, operation, node, kind, state, outcome, outcome_category,
                last_error, attempts, next_retry_at, updated_at
           FROM commit_diagnostics
          ORDER BY CASE outcome WHEN 'conflict' THEN 0 WHEN 'unknown_retryable' THEN 1 ELSE 2 END,
                   next_retry_at, scope, operation
          LIMIT ?1",
    )?;
    let rows = statement.query_map([limit], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, Option<String>>(2)?,
            row.get::<_, String>(3)?,
            row.get::<_, String>(4)?,
            row.get::<_, String>(5)?,
            row.get::<_, String>(6)?,
            row.get::<_, Option<String>>(7)?,
            row.get::<_, i64>(8)?,
            row.get::<_, i64>(9)?,
            row.get::<_, i64>(10)?,
        ))
    })?;
    let mut diagnostics = Vec::new();
    for row in rows {
        let (
            scope,
            operation,
            node,
            kind,
            state,
            outcome,
            category,
            last_error,
            attempts,
            next_retry_at,
            updated_at,
        ) = row?;
        let outcome = CommitOutcome::parse(&outcome)
            .ok_or_else(|| invalid(format!("unknown stored commit outcome {outcome}")))?;
        let category = CommitCategory::parse(&category)
            .ok_or_else(|| invalid(format!("unknown stored commit category {category}")))?;
        diagnostics.push(CommitDiagnosticRecord {
            scope,
            operation,
            node,
            kind,
            state,
            outcome,
            category,
            last_error,
            attempts: u32::try_from(attempts).unwrap_or(u32::MAX),
            next_retry_at,
            updated_at,
        });
    }
    Ok(diagnostics)
}

/// Count of live unknowns that still have an automatic retry scheduled.
pub fn open_unknown(store: &Store) -> Result<i64> {
    store
        .db
        .query_row(
            "SELECT COUNT(*) FROM commit_diagnostics WHERE outcome='unknown_retryable'",
            [],
            |row| row.get(0),
        )
        .map_err(Into::into)
}

/// Count of rows that need a user decision: conflicts and fatal unknowns. The
/// terminal `abandoned` state is deliberately excluded (it is explained, not
/// waiting for a decision).
pub fn needs_attention(store: &Store) -> Result<i64> {
    store
        .db
        .query_row(
            "SELECT COUNT(*) FROM commit_diagnostics
              WHERE outcome='conflict'
                 OR (outcome='unknown_fatal' AND state<>'abandoned')",
            [],
            |row| row.get(0),
        )
        .map_err(Into::into)
}

/// Rebuild diagnostics that a restart or a legacy database left empty. Rows
/// that already carry an outcome are never rewritten, so repeated opens are
/// idempotent.
pub(crate) fn backfill_startup(db: &Connection) -> Result<()> {
    // #14's terminal state predates TB2. Its budget was exhausted, so it must
    // not keep counting as open unknown work, but the row stays visible.
    // (Superseded rows are mapped by the v29 migration / retire paths.)
    db.execute_batch(
        "UPDATE operations
            SET outcome='unknown_retryable', outcome_category='interrupted',
                last_error='interrupted by restart; awaiting reconciliation',
                next_retry_at=unixepoch(), updated_at=unixepoch()
          WHERE outcome IS NULL AND state='uncertain';
         UPDATE namespace_ops
            SET outcome='unknown_retryable', outcome_category='interrupted',
                last_error='interrupted by restart; awaiting reconciliation',
                next_retry_at=unixepoch(), updated_at=unixepoch()
          WHERE outcome IS NULL AND state IN ('uncertain','cleanup_uncertain');
         UPDATE operations
            SET outcome='confirmed', outcome_category='local_abandoned',
                last_error='reconcile budget exhausted before diagnostics existed;
                            no automatic retry',
                next_retry_at=0, updated_at=unixepoch()
          WHERE outcome IS NULL AND state='abandoned';
         UPDATE namespace_ops
            SET outcome='confirmed', outcome_category='local_abandoned',
                last_error='reconcile budget exhausted before diagnostics existed;
                            no automatic retry',
                next_retry_at=0, updated_at=unixepoch()
          WHERE outcome IS NULL AND state='abandoned';",
    )?;
    Ok(())
}
