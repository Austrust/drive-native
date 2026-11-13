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

/// The diagnostics list ordering contract (oldest retry first, conflicts first).
///
/// The old statement sorted the whole `commit_diagnostics` view (successful
/// `confirmed` history included) on every overview. The bounded form below
/// reads each outcome class per journal as an index-ordered range, keeps only
/// its top `?1` rows, and merges them: with the total order
/// `(class, next_retry_at, scope, operation)` this is exactly the global top-k,
/// so the returned rows and their order are identical.
pub(crate) fn diagnostics_list_sql() -> &'static str {
    static SQL: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    SQL.get_or_init(|| {
        let mut sql = String::from(
            "WITH candidates(priority,scope,operation,node,kind,state,outcome,outcome_category,\
             last_error,attempts,next_retry_at,updated_at) AS (",
        );
        // (class priority, per-journal equality predicate). `confirmed` and
        // `unknown_fatal` share the last priority: the historical ordering puts
        // every other outcome in one class.
        for (priority, predicate) in [
            (0u8, "outcome='conflict'"),
            (1, "outcome='unknown_retryable'"),
            (2, "outcome='confirmed'"),
            (2, "outcome='unknown_fatal'"),
        ] {
            for table in ["operations", "namespace_ops", "transfers"] {
                let (scope, kind, attempts, updated) = match table {
                    "operations" => ("operation", "kind", "reconcile_attempts", "updated_at"),
                    "namespace_ops" => ("namespace", "kind", "attempts", "updated_at"),
                    _ => ("transfer", "direction", "0", "0"),
                };
                if !sql.ends_with("AS (") {
                    sql.push_str(" UNION ALL ");
                }
                sql.push_str(&format!(
                    "SELECT * FROM (SELECT {priority},'{scope}',id,node,{kind},state,outcome,\
                     outcome_category,last_error,{attempts},next_retry_at,{updated}                      FROM {table} WHERE {predicate} ORDER BY next_retry_at,id LIMIT ?1)"
                ));
            }
        }
        sql.push_str(
            ") SELECT scope,operation,node,kind,state,outcome,outcome_category,\
             last_error,attempts,next_retry_at,updated_at FROM candidates \
             ORDER BY priority,next_retry_at,scope,operation LIMIT ?1",
        );
        sql
    })
}

/// Both overview diagnostics counts from one statement (issue #88): the two
/// numbers are part of the same read, so a writer commit cannot be observed by
/// one count and not the other. Each count is an index range over the rows of
/// one outcome class per journal, so the retained `confirmed` history is never
/// scanned (the old form aggregated over the whole view).
pub(crate) const DIAGNOSTICS_COUNTS_SQL: &str =
    "SELECT
       (SELECT COUNT(*) FROM operations WHERE outcome='unknown_retryable')
     + (SELECT COUNT(*) FROM namespace_ops WHERE outcome='unknown_retryable')
     + (SELECT COUNT(*) FROM transfers WHERE outcome='unknown_retryable'),
       (SELECT COUNT(*) FROM operations WHERE outcome='conflict')
     + (SELECT COUNT(*) FROM namespace_ops WHERE outcome='conflict')
     + (SELECT COUNT(*) FROM transfers WHERE outcome='conflict')
     + (SELECT COUNT(*) FROM operations WHERE outcome='unknown_fatal' AND state<>'abandoned')
     + (SELECT COUNT(*) FROM namespace_ops WHERE outcome='unknown_fatal' AND state<>'abandoned')
     + (SELECT COUNT(*) FROM transfers WHERE outcome='unknown_fatal' AND state<>'abandoned')";

/// All durable diagnostics, oldest retry first.
pub fn list(store: &Store, limit: usize) -> Result<Vec<CommitDiagnosticRecord>> {
    list_on(&store.db, limit)
}

fn list_on(conn: &Connection, limit: usize) -> Result<Vec<CommitDiagnosticRecord>> {
    let limit = i64::try_from(limit.clamp(1, 500)).unwrap_or(500);
    let mut statement = conn.prepare_cached(diagnostics_list_sql())?;
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

/// One consistent read of everything the overview needs from the commit
/// diagnostics (issue #88).
///
/// The two counts and the diagnosis list come from the same read transaction,
/// so a commit cannot be half-observed. `data_version` is the connection's
/// `PRAGMA data_version` captured after the reads; `consistent` is false when
/// another connection committed while the read was running (the values are
/// still usable, but the caller must not cache them under that key).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticsSnapshot {
    /// Rows with `outcome='unknown_retryable'` (the existing `open_unknown`).
    pub unknown_retryable: i64,
    /// Rows that need a user decision: conflicts plus non-abandoned
    /// `unknown_fatal` rows (the existing `needs_attention` number; the
    /// terminal `abandoned` state is explained, not waiting for a decision).
    pub unknown_fatal: i64,
    /// The diagnosis list (`list(store, 20)` semantics).
    pub records: Vec<CommitDiagnosticRecord>,
    /// `PRAGMA data_version` observed after the read transaction; valid as a
    /// cache key only on the same connection.
    pub data_version: i64,
    /// True when the source connection did not commit between the start and end
    /// of the read, so `data_version` describes exactly this result.
    pub consistent: bool,
}

impl DiagnosticsSnapshot {
    /// Count of live unknowns that still have an automatic retry scheduled.
    pub fn open_unknown(&self) -> i64 {
        self.unknown_retryable
    }
    /// Count of rows that need a user decision (conflicts + armed fatals).
    pub fn needs_attention(&self) -> i64 {
        self.unknown_fatal
    }
}

/// Number of attempts to obtain a read that was not torn by a crossing commit.
const DIAGNOSTICS_SNAPSHOT_ATTEMPTS: usize = 2;

/// Read the overview diagnostics through one consistent read transaction.
///
/// Safe on a read-only writer-adjacent handle: it only starts a deferred read
/// transaction, never writes, and `PRAGMA data_version` is a read-only pragma.
pub fn snapshot(store: &Store) -> Result<DiagnosticsSnapshot> {
    let mut last: Option<DiagnosticsSnapshot> = None;
    for _ in 0..DIAGNOSTICS_SNAPSHOT_ATTEMPTS {
        let before = store.data_version()?;
        let (unknown_retryable, unknown_fatal, records) = {
            let transaction = store.db.unchecked_transaction()?;
            let (unknown_retryable, unknown_fatal): (i64, i64) =
                transaction.query_row(DIAGNOSTICS_COUNTS_SQL, [], |row| {
                    Ok((row.get(0)?, row.get(1)?))
                })?;
            let records = list_on(&transaction, 20)?;
            transaction.commit()?;
            (unknown_retryable, unknown_fatal, records)
        };
        let after = store.data_version()?;
        let snapshot = DiagnosticsSnapshot {
            unknown_retryable,
            unknown_fatal,
            records,
            data_version: after,
            consistent: before == after,
        };
        if snapshot.consistent {
            return Ok(snapshot);
        }
        last = Some(snapshot);
    }
    Ok(last.expect("snapshot loop always produces a result"))
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

#[cfg(test)]
#[path = "commit_diagnostics_scale_tests.rs"]
mod scale_tests;
