//! Connection-local, transactional invalidations for a future desktop index.
//! These are row changes, not server events or a second synchronization store.
//! Consumers must rebuild on epoch/gap and project the current atoms under the
//! same Store serialization boundary as the returned watermark. No UI uses this
//! journal yet, and it does not include transient flights or external file loss.
use crate::{Error, Result, Store};
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};

const MAX_ENTRIES: usize = 65_536;
const MAX_BYTES: usize = 16 * 1024 * 1024;
const ROW_OVERHEAD: usize = 160;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct JournalBudget {
    pub entries: usize,
    /// Accounted UTF-8 payload plus fixed per-row allowance, not process RSS.
    pub bytes: usize,
}
impl Default for JournalBudget {
    fn default() -> Self {
        Self {
            entries: 8192,
            bytes: 4 * 1024 * 1024,
        }
    }
}
impl JournalBudget {
    fn validate(self) -> Result<()> {
        if self.entries == 0
            || self.entries > MAX_ENTRIES
            || self.bytes == 0
            || self.bytes > MAX_BYTES
        {
            return Err(Error::State("invalid status journal budget".into()));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatusCursor {
    pub epoch: String,
    pub sequence: u64,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JournalWatermark {
    pub cursor: StatusCursor,
    /// All changes through this sequence have been discarded (prune/overflow).
    pub retained_after: u64,
    pub entries: usize,
    pub bytes: usize,
    pub overflows: u64,
    pub budget: JournalBudget,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChangeScope {
    NodeAndAncestors,
    /// Includes inherited pin/overlay/permission and namespace visibility.
    SubtreeAndAncestors,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatusChange {
    pub sequence: u64,
    pub source: String,
    pub operation: String,
    pub scope: ChangeScope,
    pub old_node: Option<String>,
    pub new_node: Option<String>,
    pub old_parent: Option<String>,
    pub new_parent: Option<String>,
    pub old_name: Option<String>,
    pub new_name: Option<String>,
    pub bytes: usize,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GapReason {
    EpochChanged,
    HistoryDiscarded,
    CursorAhead,
}
#[derive(Debug, PartialEq, Eq)]
pub enum ChangePage {
    Ready {
        changes: Vec<StatusChange>,
        next: StatusCursor,
        watermark: JournalWatermark,
        has_more: bool,
    },
    Rebuild {
        reason: GapReason,
        watermark: JournalWatermark,
    },
}

// Table, identity column, semantic columns, descendant propagation. Metadata
// rows and operations are included so the source remains useful for attributes
// and task state; inode/LRU/settings/receipt bodies deliberately are not.
const SOURCES: &[(&str, &str, &[&str], bool)] = &[
    (
        "nodes",
        "id",
        &[
            "id",
            "parent",
            "name",
            "remote_version",
            "directory",
            "local_version",
            "blob",
            "dirty_blob",
            "base_version",
            "dirty_generation",
            "pinned",
            "deleted",
            "conflict",
        ],
        true,
    ),
    (
        "drafts",
        "node",
        &["node", "token", "base", "generation", "checkpoint", "dirty"],
        false,
    ),
    (
        "transfers",
        "node",
        &["node", "version", "direction", "state", "error"],
        false,
    ),
    (
        "operations",
        "node",
        &["node", "kind", "base", "blob", "generation", "state"],
        false,
    ),
    (
        "namespace_ops",
        "node",
        &["node", "body", "state", "kind"],
        true,
    ),
    ("namespace_overlay", "node", &["node", "remote"], true),
    ("local_creations", "node", &["node", "ready"], true),
    ("local_unlinks", "node", &["node", "confirmed"], true),
    ("access_denied", "node", &["node"], true),
    ("remote_aliases", "node", &["node", "remote"], true),
    (
        "content_version_aliases",
        "node",
        &["node", "old", "new"],
        false,
    ),
    (
        "node_metadata",
        "node",
        &["node", "size", "mtime", "mode"],
        false,
    ),
    ("local_modes", "node", &["node", "mode"], false),
    (
        "cache_releases",
        "node",
        &["node", "version", "state", "attempts", "retry_at"],
        false,
    ),
    (
        "engine_bridge_intents",
        "node",
        &["node", "phase", "attempts", "retry_at", "error"],
        true,
    ),
    (
        "engine_bridge_tombstones",
        "node",
        &["node", "cutoff"],
        true,
    ),
];

pub(crate) fn install(db: &Connection) -> Result<()> {
    // REPLACE's implicit DELETE must be observed too. There are no main-schema
    // triggers introduced here; this setting belongs to this writer connection.
    db.pragma_update(None, "recursive_triggers", true)?;
    db.execute_batch(
        "CREATE TEMP TABLE dn_status_meta (
        singleton INTEGER PRIMARY KEY CHECK(singleton=1), epoch TEXT NOT NULL,
        head INTEGER NOT NULL, floor INTEGER NOT NULL, entries INTEGER NOT NULL,
        bytes INTEGER NOT NULL, overflows INTEGER NOT NULL,
        max_entries INTEGER NOT NULL, max_bytes INTEGER NOT NULL);
        CREATE TEMP TABLE dn_status_changes (
        seq INTEGER PRIMARY KEY, source TEXT NOT NULL, operation TEXT NOT NULL,
        subtree INTEGER NOT NULL, old_node TEXT, new_node TEXT,
        old_parent TEXT, new_parent TEXT, old_name TEXT, new_name TEXT,
        bytes INTEGER NOT NULL);",
    )?;
    let budget = JournalBudget::default();
    db.execute(
        "INSERT INTO dn_status_meta VALUES(1,?1,0,0,0,0,0,?2,?3)",
        params![
            uuid::Uuid::new_v4().to_string(),
            budget.entries as i64,
            budget.bytes as i64
        ],
    )?;
    for &(table, identity, columns, subtree) in SOURCES {
        for operation in ["INSERT", "UPDATE", "DELETE"] {
            let old = if operation == "INSERT" {
                "NULL".into()
            } else {
                format!("OLD.{identity}")
            };
            let new = if operation == "DELETE" {
                "NULL".into()
            } else {
                format!("NEW.{identity}")
            };
            let location = |side: &str, column: &str, absent: bool| {
                if absent {
                    "NULL".to_owned()
                } else if table == "nodes" {
                    format!("{side}.{column}")
                } else {
                    format!("(SELECT {column} FROM main.nodes WHERE id={side}.{identity})")
                }
            };
            let old_parent = location("OLD", "parent", operation == "INSERT");
            let new_parent = location("NEW", "parent", operation == "DELETE");
            let old_name = location("OLD", "name", operation == "INSERT");
            let new_name = location("NEW", "name", operation == "DELETE");
            let values = [&old, &new, &old_parent, &new_parent, &old_name, &new_name];
            let bytes = format!(
                "{}+{}",
                ROW_OVERHEAD + table.len() + operation.len(),
                values
                    .iter()
                    .map(|v| format!("coalesce(length(CAST({v} AS BLOB)),0)"))
                    .collect::<Vec<_>>()
                    .join("+")
            );
            let when = if operation == "UPDATE" {
                format!(
                    "WHEN {}",
                    columns
                        .iter()
                        .map(|c| format!("OLD.{c} IS NOT NEW.{c}"))
                        .collect::<Vec<_>>()
                        .join(" OR ")
                )
            } else {
                String::new()
            };
            // A new version, conflict or content publication changes this
            // node's contribution, not the inherited facts of every descendant.
            // Preserve subtree invalidation for identity/namespace/visibility
            // and pin changes, including mixed updates. INSERT/DELETE remain
            // conservative because existing descendants may attach/detach.
            let subtree = if table == "nodes" && operation == "UPDATE" {
                ["id", "parent", "name", "directory", "pinned", "deleted"]
                    .iter()
                    .map(|column| format!("OLD.{column} IS NOT NEW.{column}"))
                    .collect::<Vec<_>>()
                    .join(" OR ")
            } else {
                i32::from(subtree).to_string()
            };
            // No per-insert COUNT, SUM, sort, or unbounded retention. On overflow
            // discard the bounded buffer and the current row; the floor tells
            // every lagging consumer it must rebuild. All bookkeeping rolls
            // back with the triggering statement/transaction/savepoint.
            db.execute_batch(&format!(
                "CREATE TEMP TRIGGER dn_status_{table}_{operation}
                AFTER {operation} ON main.{table} {when} BEGIN
                UPDATE dn_status_meta SET head=head+1;
                INSERT INTO dn_status_changes
                SELECT head,'{table}','{operation}',({subtree}),{old},{new},
                       {old_parent},{new_parent},{old_name},{new_name},{bytes}
                FROM dn_status_meta
                WHERE entries<max_entries AND bytes+({bytes})<=max_bytes;
                UPDATE dn_status_meta SET floor=head,entries=0,bytes=0,overflows=overflows+1
                WHERE NOT EXISTS(SELECT 1 FROM dn_status_changes WHERE seq=head);
                DELETE FROM dn_status_changes WHERE seq<=(SELECT floor FROM dn_status_meta);
                UPDATE dn_status_meta SET entries=entries+1,bytes=bytes+({bytes})
                WHERE EXISTS(SELECT 1 FROM dn_status_changes WHERE seq=head);
                END;"
            ))?;
        }
    }
    Ok(())
}

fn read_integer<T: TryFrom<i64>>(row: &rusqlite::Row<'_>, column: usize) -> rusqlite::Result<T> {
    let value: i64 = row.get(column)?;
    value
        .try_into()
        .map_err(|_| rusqlite::Error::IntegralValueOutOfRange(column, value))
}

impl Store {
    fn committed_status_boundary(&self) -> Result<()> {
        if !self.db.is_autocommit() {
            return Err(Error::State(
                "status journal requires a committed Store boundary".into(),
            ));
        }
        Ok(())
    }

    pub fn status_change_watermark(&self) -> Result<JournalWatermark> {
        self.committed_status_boundary()?;
        Ok(self.db.query_row("SELECT epoch,head,floor,entries,bytes,overflows,max_entries,max_bytes FROM dn_status_meta", [], |r| {
            Ok(JournalWatermark {
                cursor: StatusCursor { epoch: r.get(0)?, sequence: read_integer(r, 1)? },
                retained_after: read_integer(r, 2)?, entries: read_integer(r, 3)?, bytes: read_integer(r, 4)?,
                overflows: read_integer(r, 5)?, budget: JournalBudget { entries: read_integer(r, 6)?, bytes: read_integer(r, 7)? },
            })
        })?)
    }

    /// Returns invalidations, not complete historical atoms. Read current node
    /// facts before releasing the external Store lock; apply the union of old
    /// and new locations. A page boundary is not a transaction boundary.
    pub fn read_status_changes(&self, after: &StatusCursor, limit: usize) -> Result<ChangePage> {
        if limit == 0 || limit > MAX_ENTRIES {
            return Err(Error::State("invalid status journal page size".into()));
        }
        let watermark = self.status_change_watermark()?;
        let reason = if after.epoch != watermark.cursor.epoch {
            Some(GapReason::EpochChanged)
        } else if after.sequence < watermark.retained_after {
            Some(GapReason::HistoryDiscarded)
        } else if after.sequence > watermark.cursor.sequence {
            Some(GapReason::CursorAhead)
        } else {
            None
        };
        if let Some(reason) = reason {
            return Ok(ChangePage::Rebuild { reason, watermark });
        }
        let mut q = self.db.prepare("SELECT seq,source,operation,subtree,old_node,new_node,old_parent,new_parent,old_name,new_name,bytes FROM dn_status_changes WHERE seq>?1 ORDER BY seq LIMIT ?2")?;
        let changes = q
            .query_map(params![after.sequence as i64, limit as i64], |r| {
                Ok(StatusChange {
                    sequence: read_integer(r, 0)?,
                    source: r.get(1)?,
                    operation: r.get(2)?,
                    scope: if r.get::<_, bool>(3)? {
                        ChangeScope::SubtreeAndAncestors
                    } else {
                        ChangeScope::NodeAndAncestors
                    },
                    old_node: r.get(4)?,
                    new_node: r.get(5)?,
                    old_parent: r.get(6)?,
                    new_parent: r.get(7)?,
                    old_name: r.get(8)?,
                    new_name: r.get(9)?,
                    bytes: read_integer(r, 10)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        let sequence = changes.last().map_or(after.sequence, |c| c.sequence);
        let has_more = sequence < watermark.cursor.sequence;
        let next = StatusCursor {
            epoch: after.epoch.clone(),
            sequence,
        };
        Ok(ChangePage::Ready {
            changes,
            next,
            watermark,
            has_more,
        })
    }

    /// Explicit acknowledgement by the future index owner. Lagging readers get
    /// a gap, never a falsely complete empty page. Does not mutate durable state.
    pub fn prune_status_changes(&mut self, through: &StatusCursor) -> Result<()> {
        let watermark = self.status_change_watermark()?;
        if through.epoch != watermark.cursor.epoch || through.sequence > watermark.cursor.sequence {
            return Err(Error::State("invalid status journal prune cursor".into()));
        }
        if through.sequence <= watermark.retained_after {
            return Ok(());
        }
        let tx = self.db.transaction()?;
        // Explicit batched pruning may aggregate the bounded retained prefix;
        // the hot-path insertion triggers never scan the buffer for accounting.
        let (entries, bytes): (i64, i64) = tx.query_row(
            "SELECT count(*),coalesce(sum(bytes),0) FROM dn_status_changes WHERE seq<=?1",
            [through.sequence as i64],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )?;
        tx.execute(
            "DELETE FROM dn_status_changes WHERE seq<=?1",
            [through.sequence as i64],
        )?;
        tx.execute(
            "UPDATE dn_status_meta SET floor=?1,entries=entries-?2,bytes=bytes-?3",
            params![through.sequence as i64, entries, bytes],
        )?;
        tx.commit()?;
        Ok(())
    }

    pub fn set_status_journal_budget(&mut self, budget: JournalBudget) -> Result<()> {
        budget.validate()?;
        if self.status_change_watermark()?.budget == budget {
            return Ok(());
        }
        let tx = self.db.transaction()?;
        tx.execute(
            "UPDATE dn_status_meta SET max_entries=?1,max_bytes=?2",
            params![budget.entries as i64, budget.bytes as i64],
        )?;
        tx.execute("UPDATE dn_status_meta SET floor=head,entries=0,bytes=0,overflows=overflows+1 WHERE entries>max_entries OR bytes>max_bytes", [])?;
        tx.execute(
            "DELETE FROM dn_status_changes WHERE seq<=(SELECT floor FROM dn_status_meta)",
            [],
        )?;
        tx.commit()?;
        Ok(())
    }
}

#[cfg(test)]
#[path = "status_changes_tests.rs"]
mod tests;
