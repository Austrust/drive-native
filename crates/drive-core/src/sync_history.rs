//! Persistent completion history. Entries are metadata, never content or raw
//! transport errors. Workflow triggers execute in the writer's transaction;
//! reading the history cannot synthesize a successful synchronization.
use crate::{Error, Result, Store};
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};

pub const DEFAULT_RETENTION_DAYS: u32 = 30;
pub const DEFAULT_RETENTION_RECORDS: u32 = 10_000;
const MAX_RETENTION_DAYS: u32 = 3_650;
const MAX_RETENTION_RECORDS: u32 = 1_000_000;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SyncHistoryRecord {
    pub id: i64,
    pub seq: i64,
    pub mount: String,
    pub node: Option<String>,
    pub name: String,
    pub path_at_event: String,
    pub action: String,
    pub result: String,
    pub attempts: u32,
    pub error_kind: Option<String>,
    pub error_summary: Option<String>,
    pub started_at: Option<i64>,
    pub completed_at: i64,
    pub detail: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HistoryQuery {
    pub mount: Option<String>,
    pub result: Option<String>,
    pub name: Option<String>,
    pub cursor: Option<String>,
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryPage {
    pub records: Vec<SyncHistoryRecord>,
    pub next_cursor: Option<String>,
}

#[derive(Debug, Clone)]
pub enum ClearHistory {
    OlderThanDays { days: Option<u32> },
    All,
}

impl<'de> Deserialize<'de> for ClearHistory {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        // An internally tagged unit variant discards its remaining map, even
        // with deny_unknown_fields. An empty struct variant validates it.
        #[derive(Deserialize)]
        #[serde(tag = "mode", rename_all = "snake_case", deny_unknown_fields)]
        enum Wire {
            OlderThanDays { days: Option<u32> },
            All {},
        }
        Ok(match Wire::deserialize(deserializer)? {
            Wire::OlderThanDays { days } => Self::OlderThanDays { days },
            Wire::All {} => Self::All,
        })
    }
}

fn invalid() -> Error {
    Error::State("invalid sync history request".into())
}

fn now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        .min(i64::MAX as u64) as i64
}

/// Called inside Store::open's existing atomic migration, after its backup.
///
/// `repairing` is true for a brand-new or pre-v28 store. A legacy store has no
/// durable row counter yet, so it is rebuilt from the table once here; after
/// that the append/delete triggers keep it exact and the hot path never counts
/// the retained window.
pub(crate) fn migrate(db: &Connection, repairing: bool) -> Result<()> {
    db.execute_batch(
        "CREATE TABLE IF NOT EXISTS sync_history(
          id INTEGER PRIMARY KEY AUTOINCREMENT,
          seq INTEGER NOT NULL UNIQUE,
          mount TEXT NOT NULL, node TEXT, name TEXT NOT NULL,
          path_at_event TEXT NOT NULL, action TEXT NOT NULL,
          result TEXT NOT NULL CHECK(result IN ('ok','failed','cancelled','pending')),
          attempts INTEGER NOT NULL DEFAULT 1 CHECK(attempts>=1),
          error_kind TEXT,error_summary TEXT,started_at INTEGER,completed_at INTEGER NOT NULL,
          detail TEXT, operation_key TEXT NOT NULL UNIQUE);
         CREATE INDEX IF NOT EXISTS sync_history_mount_time ON sync_history(mount,completed_at DESC);
         CREATE INDEX IF NOT EXISTS sync_history_name ON sync_history(name COLLATE NOCASE);
         CREATE INDEX IF NOT EXISTS sync_history_result ON sync_history(result,completed_at DESC);
         CREATE INDEX IF NOT EXISTS sync_history_time ON sync_history(completed_at);
         CREATE INDEX IF NOT EXISTS sync_history_mount_seq ON sync_history(mount,seq DESC);
         CREATE INDEX IF NOT EXISTS sync_history_result_seq ON sync_history(result,seq DESC);
         INSERT OR IGNORE INTO metadata(key,value) VALUES
           ('sync_history_sequence','0'),('sync_history_days','30'),('sync_history_records','10000');
         INSERT OR IGNORE INTO metadata(key,value) VALUES ('sync_history_rows','0');",
    )?;
    if repairing {
        // A pre-v28 store has rows but no counter yet. Rebuild it once inside the
        // same migration transaction; the triggers below keep it exact afterwards.
        db.execute(
            "UPDATE metadata SET value=(SELECT COUNT(*) FROM sync_history)
             WHERE key='sync_history_rows'",
            [],
        )?;
    }
    // Durable triggers cover *all* writers, including reconciliation and the
    // bridge. They store only a controlled classification, never error bodies.
    workflow_trigger(db, "history_upload", "operations", "NEW.kind='upload'",
        "'upload:'||NEW.id", "NEW.node", "'upload'", false)?;
    workflow_trigger(db, "history_namespace", "namespace_ops",
        "NEW.kind IN ('rename','remove')", "'namespace:'||NEW.id", "NEW.node",
        "CASE WHEN NEW.kind='remove' THEN 'delete'
          WHEN json_extract(NEW.body,'$.node.parent') IS NOT json_extract(NEW.body,'$.parent') THEN 'move' ELSE 'rename' END", false)?;
    workflow_trigger(db, "history_download", "transfers", "NEW.direction='download'",
        "'download:'||NEW.id", "NEW.node", "'download'", true)?;
    // Permission diagnostics for compound namespace operations must also be
    // resolved when later reconciliation confirms or conflicts the operation.
    workflow_trigger(db, "history_permission_namespace_extra", "namespace_ops",
        "NEW.kind IN ('mkdir','replace')", "'namespace:'||NEW.id", "NEW.node", "NEW.kind", false)?;
    // Removing a transfer is not in itself success: cancellation, stale versions,
    // and explicitly clearing retries use the same operation. Require a published
    // matching body before labeling the record as completed.
    let success = "EXISTS(SELECT 1 FROM nodes n WHERE n.id=OLD.node AND n.deleted=0 AND n.local_version=OLD.version AND n.blob IS NOT NULL)
        OR EXISTS(SELECT 1 FROM sync_history WHERE operation_key='download:'||OLD.id AND result='ok')";
    let body = append_sql("'download:'||OLD.id", "OLD.node", "'download'",
        &format!("CASE WHEN {success} THEN 'ok' ELSE 'cancelled' END"), "NULL", "0", "NULL");
    db.execute_batch(&format!(
        "CREATE TRIGGER IF NOT EXISTS history_download_removed AFTER DELETE ON transfers
         WHEN OLD.direction='download' BEGIN {body} END;"
    ))?;
    // Retained-window bookkeeping runs inside the same triggers as the append so
    // no writer can grow the table past the retention setting. The row count is
    // a durable counter, never `COUNT(*)`: each append and each durable delete
    // adjusts one metadata row, so an update costs the same at 10 rows and at the
    // retained cap. Age and cap eviction share one DELETE so the trigger program
    // has a constant, small statement count.
    //
    // v27 stores used a different prune trigger; drop and recreate them so the
    // upgrade installs the bounded program. Only the history table is touched.
    let evict = "DELETE FROM sync_history
        WHERE completed_at < unixepoch()-86400*(SELECT CAST(value AS INTEGER) FROM metadata WHERE key='sync_history_days')
           OR (seq=(SELECT MIN(seq) FROM sync_history)
               AND (SELECT CAST(value AS INTEGER) FROM metadata WHERE key='sync_history_rows')
                 > (SELECT CAST(value AS INTEGER) FROM metadata WHERE key='sync_history_records'))";
    db.execute_batch(&format!(
        "DROP TRIGGER IF EXISTS history_prune_insert;
         DROP TRIGGER IF EXISTS history_prune_update;
         DROP TRIGGER IF EXISTS history_prune_delete;
         CREATE TRIGGER history_prune_insert AFTER INSERT ON sync_history BEGIN
           UPDATE metadata SET value=CAST(value AS INTEGER)+1 WHERE key='sync_history_rows';
           {evict};
         END;
         CREATE TRIGGER history_prune_update AFTER UPDATE ON sync_history BEGIN {evict}; END;
         CREATE TRIGGER history_prune_delete AFTER DELETE ON sync_history BEGIN
           UPDATE metadata SET value=CAST(value AS INTEGER)-1 WHERE key='sync_history_rows';
         END;"
    ))?;
    prune(db, now())?;
    Ok(())
}

fn workflow_trigger(db: &Connection, name: &str, table: &str, filter: &str,
    key: &str, node: &str, action: &str, download: bool) -> Result<()> {
    let result = "CASE NEW.state WHEN 'committed' THEN 'ok' WHEN 'conflict' THEN 'failed'
        WHEN 'failed' THEN 'failed' WHEN 'retry' THEN 'failed' WHEN 'superseded' THEN 'cancelled'
        WHEN 'obsolete' THEN 'cancelled' ELSE 'pending' END";
    let error = if download {
        "CASE WHEN NEW.state IN ('failed','retry') THEN CASE WHEN json_valid(NEW.error) THEN CASE
           WHEN json_extract(NEW.error,'$.kind') IN ('transport','session_expired','server_rejected','invalid_content','local_storage')
           THEN json_extract(NEW.error,'$.kind') ELSE 'download_failed' END ELSE 'download_failed' END ELSE NULL END"
    } else {
        "CASE NEW.state WHEN 'uncertain' THEN 'uncertain_commit' WHEN 'conflict' THEN 'conflict' ELSE NULL END"
    };
    let body = append_sql(key, node, action, result, error,
        "CASE WHEN NEW.state='inflight' THEN 1 ELSE 0 END", "NULL");
    let changed=if download { "NEW.state!=OLD.state OR NEW.error IS NOT OLD.error" } else { "NEW.state!=OLD.state" };
    let columns=if download { "state,error" } else { "state" };
    let attempts=if download { format!("UPDATE sync_history SET attempts=MAX(attempts,
        CASE WHEN json_valid(NEW.error) THEN COALESCE(CAST(json_extract(NEW.error,'$.attempts') AS INTEGER),1) ELSE 1 END)
        WHERE operation_key={key};") } else { String::new() };
    db.execute_batch(&format!(
        "CREATE TRIGGER IF NOT EXISTS {name} AFTER UPDATE OF {columns} ON {table}
         WHEN ({filter}) AND ({changed}) AND NEW.state IN
           ('inflight','uncertain','committed','conflict','failed','retry','superseded','obsolete')
         BEGIN {body} {attempts} END;"
    ))?;
    Ok(())
}

// These inputs are SQL expressions owned by this module, never user strings.
fn append_sql(key: &str, node: &str, action: &str, result: &str, error: &str,
    attempted: &str, detail: &str) -> String {
    let path = format!("COALESCE((WITH RECURSIVE p(id,parent,path,visited) AS (
        SELECT id,parent,CASE WHEN parent IS NULL THEN '' ELSE name END,'/'||hex(id)||'/' FROM nodes WHERE id={node}
        UNION ALL SELECT n.id,n.parent,CASE WHEN n.parent IS NULL THEN p.path ELSE n.name||'/'||p.path END,p.visited||hex(n.id)||'/'
          FROM nodes n JOIN p ON p.parent=n.id
          WHERE instr(p.visited,'/'||hex(n.id)||'/')=0)
        SELECT path FROM p WHERE parent IS NULL LIMIT 1),'')");
    format!(
        "UPDATE metadata SET value=CAST(value AS INTEGER)+1 WHERE key='sync_history_sequence';
         INSERT INTO sync_history(seq,mount,node,name,path_at_event,action,result,attempts,
           error_kind,error_summary,started_at,completed_at,detail,operation_key)
         VALUES((SELECT CAST(value AS INTEGER) FROM metadata WHERE key='sync_history_sequence'),
           COALESCE((SELECT node FROM fs_inodes WHERE ino=1),
             (SELECT json_extract(value,'$.file_id') FROM metadata WHERE key='native_scope' AND json_valid(value)),
             (SELECT id FROM nodes WHERE parent IS NULL ORDER BY id LIMIT 1),''),
           {node},COALESCE((SELECT name FROM nodes WHERE id={node}),''),{path},{action},{result},1,
           {error},{error},CASE WHEN {attempted}>0 THEN unixepoch() ELSE NULL END,unixepoch(),{detail},{key})
         ON CONFLICT(operation_key) DO UPDATE SET seq=excluded.seq,
           name=excluded.name,path_at_event=CASE WHEN excluded.path_at_event='' THEN sync_history.path_at_event ELSE excluded.path_at_event END,
           result=excluded.result,attempts=sync_history.attempts+({attempted}),
           error_kind=excluded.error_kind,error_summary=excluded.error_summary,
           started_at=COALESCE(sync_history.started_at,excluded.started_at),
           completed_at=excluded.completed_at,detail=COALESCE(excluded.detail,sync_history.detail);"
    )
}

/// History rows collected while one inbox batch is applied, written by a single
/// multi-value statement before that application transaction commits.
///
/// The *content* of every row is captured at the moment its event is applied:
/// `path_at_event` and `name` follow the namespace as it stands after that
/// event, exactly like the previous one-row-per-event helper. Only the INSERT,
/// the retention reservation and the sequence assignment are deferred, so the
/// durability rule "history rows commit with the namespace write and the cursor
/// never moves past missing history" is unchanged.
pub(crate) struct RemoteHistoryBatch {
    rows: Vec<RemoteHistoryRow>,
}

struct RemoteHistoryRow {
    node: String,
    name: String,
    path: String,
    root: String,
    operation_key: String,
}

/// Bound one multi-value statement to this many rows. Six bound values per row
/// keeps a full statement far below SQLite's variable limit.
const MAX_ROWS_PER_INSERT: usize = 500;

impl RemoteHistoryBatch {
    pub(crate) fn new() -> Self {
        Self { rows: Vec::new() }
    }

    /// Collect the history row for one discovered server event. This is
    /// deliberately not attached to status journal invalidations (local changes
    /// also invalidate that journal and are not remote synchronization events).
    pub(crate) fn push_remote_event(
        &mut self,
        db: &Connection,
        event: &crate::Event,
    ) -> Result<()> {
        let node = match &event.change {
            crate::Change::Upsert(node) => node.id.as_str(),
            crate::Change::Delete { id } => id.as_str(),
        };
        let local: String = db.query_row(
            "SELECT COALESCE((SELECT node FROM remote_aliases WHERE remote=?1),?1)",
            [node],
            |r| r.get(0),
        )?;
        let (name, path, root) = namespace_path_at_event(db, &local)?;
        self.rows.push(RemoteHistoryRow {
            node: local,
            name,
            path,
            root,
            operation_key: format!("remote:{}", event.id),
        });
        Ok(())
    }

    /// Write the collected rows. Must run inside the caller's transaction,
    /// before it commits and before `committed_cursor` advances.
    pub(crate) fn finish(self, db: &Connection) -> Result<()> {
        if self.rows.is_empty() {
            return Ok(());
        }
        // Two events cannot share `remote:{id}` (`seen` rejects a reused id with
        // different content) but keep the multi-value statement safe if a caller
        // ever does: the last row wins, like two sequential UPSERTs.
        let mut rows = Vec::with_capacity(self.rows.len());
        let mut index: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
        for row in self.rows {
            if let Some(&at) = index.get(&row.operation_key) {
                rows[at] = row;
            } else {
                index.insert(row.operation_key.clone(), rows.len());
                rows.push(row);
            }
        }
        // Steady-state retention is reserved once per batch instead of once per
        // event. The AFTER INSERT trigger keeps the hard cap for every other
        // writer (and remains the fallback for a batch larger than the cap).
        reserve(db, rows.len())?;
        // Read the sequence base *after* the event loop: namespace workflow
        // triggers started by this batch's own resolutions advance it too.
        let base: i64 = db.query_row(
            "SELECT CAST(value AS INTEGER) FROM metadata WHERE key='sync_history_sequence'",
            [],
            |r| r.get(0),
        )?;
        let mount: Option<String> = db.query_row(
            "SELECT COALESCE((SELECT node FROM fs_inodes WHERE ino=1),
               (SELECT CASE WHEN json_valid(value) THEN json_extract(value,'$.file_id') END
                  FROM metadata WHERE key='native_scope'))",
            [],
            |r| r.get(0),
        )?;
        let add = i64::try_from(rows.len()).unwrap_or(i64::MAX);
        let mut next = base;
        for chunk in rows.chunks(MAX_ROWS_PER_INSERT) {
            let mut sql = String::from(
                "INSERT INTO sync_history(seq,mount,node,name,path_at_event,action,result,
                   attempts,completed_at,operation_key) VALUES ",
            );
            for at in 0..chunk.len() {
                if at > 0 {
                    sql.push(',');
                }
                sql.push_str("(?,?,?,?,?,'remote_change','ok',1,unixepoch(),?)");
            }
            sql.push_str(
                " ON CONFLICT(operation_key) DO UPDATE SET seq=excluded.seq,
                   result=excluded.result,error_kind=excluded.error_kind,
                   error_summary=excluded.error_summary,completed_at=excluded.completed_at",
            );
            let mut values: Vec<rusqlite::types::Value> = Vec::with_capacity(chunk.len() * 6);
            for row in chunk {
                next = next
                    .checked_add(1)
                    .ok_or_else(|| Error::State("sync history sequence exhausted".into()))?;
                values.push(next.into());
                values.push(match mount.as_deref() {
                    Some(mount) => mount.to_owned().into(),
                    None => row.root.clone().into(),
                });
                values.push(row.node.clone().into());
                values.push(row.name.clone().into());
                values.push(row.path.clone().into());
                values.push(row.operation_key.clone().into());
            }
            db.execute(&sql, rusqlite::params_from_iter(values))?;
        }
        db.execute(
            "UPDATE metadata SET value=CAST(value AS INTEGER)+?1
               WHERE key='sync_history_sequence'",
            [add],
        )?;
        Ok(())
    }
}

/// One-time per batch retention reservation. Expiring day-old rows and deleting
/// the excess from the oldest end in single statements replaces the per-event
/// trigger eviction; the trigger stays installed as the hard cap for all other
/// writers.
fn reserve(db: &Connection, add: usize) -> Result<()> {
    db.execute(
        "DELETE FROM sync_history
          WHERE completed_at < unixepoch()-86400*(SELECT CAST(value AS INTEGER) FROM metadata WHERE key='sync_history_days')",
        [],
    )?;
    let cap: i64 = db
        .query_row(
            "SELECT CAST(value AS INTEGER) FROM metadata WHERE key='sync_history_records'",
            [],
            |r| r.get(0),
        )
        .optional()?
        .unwrap_or(i64::from(DEFAULT_RETENTION_RECORDS));
    let rows: i64 = db
        .query_row(
            "SELECT CAST(value AS INTEGER) FROM metadata WHERE key='sync_history_rows'",
            [],
            |r| r.get(0),
        )
        .optional()?
        .unwrap_or(0);
    let excess = rows
        .saturating_add(i64::try_from(add).unwrap_or(i64::MAX))
        .saturating_sub(cap);
    if excess > 0 {
        db.execute(
            "DELETE FROM sync_history WHERE seq IN
               (SELECT seq FROM sync_history ORDER BY seq LIMIT ?1)",
            [excess],
        )?;
    }
    Ok(())
}

/// Name, mount-relative path and walked root of `node`, using the committed
/// node rows as they stand at this instant. This is the same walk `record()`
/// performs; it is kept separate so a batch captures `path_at_event` when the
/// event applies instead of recomputing it at commit time.
fn namespace_path_at_event(db: &Connection, node: &str) -> Result<(String, String, String)> {
    let mut parts = Vec::new();
    let mut cursor = Some(node.to_owned());
    let mut seen = std::collections::HashSet::new();
    let mut name = String::new();
    let mut root = String::new();
    while let Some(id) = cursor.take() {
        if !seen.insert(id.clone()) {
            return Err(Error::State("namespace path cycle".into()));
        }
        let row: Option<(Option<String>, String)> = db
            .query_row("SELECT parent,name FROM nodes WHERE id=?1", [&id], |r| {
                Ok((r.get(0)?, r.get(1)?))
            })
            .optional()?;
        let Some((parent, current_name)) = row else {
            break;
        };
        if name.is_empty() {
            name = current_name.clone();
        }
        if parent.is_some() {
            parts.push(current_name);
        } else {
            root = id;
        }
        cursor = parent;
    }
    parts.reverse();
    Ok((name, parts.join("/"), root))
}

/// Called by content publication *before* its transaction commits. This also
/// covers foreground downloads with no durable transfer row. A durable transfer
/// identity merges retry/publication/deletion, while a later download of the same
/// version after cache release remains a distinct operation.
pub(crate) fn record_download_publication(db: &Connection, node: &str, version: &str) -> Result<()> {
    let transfer: Option<String> = db.query_row("SELECT id FROM transfers WHERE node=?1 AND version=?2 AND direction='download'",
        params![node,version],|r|r.get(0)).optional()?;
    let key=transfer.map(|id|format!("download:{id}")).unwrap_or_else(||format!("publication:{}",uuid::Uuid::new_v4()));
    record(db, &key, node, "download", "ok", None)
}

fn record(db: &Connection, key: &str, node: &str, action: &str, result: &str,
    error: Option<&str>) -> Result<()> {
    // Parameterized single statements: callers already own the transaction.
    db.execute("UPDATE metadata SET value=CAST(value AS INTEGER)+1 WHERE key='sync_history_sequence'", [])?;
    let mut parts = Vec::new();
    let mut cursor = Some(node.to_owned());
    let mut seen = std::collections::HashSet::new();
    let mut name = String::new();
    let mut root = String::new();
    while let Some(id) = cursor.take() {
        if !seen.insert(id.clone()) { return Err(Error::State("namespace path cycle".into())); }
        let row: Option<(Option<String>, String)> = db.query_row(
            "SELECT parent,name FROM nodes WHERE id=?1", [&id], |r| Ok((r.get(0)?,r.get(1)?))).optional()?;
        let Some((parent, current_name)) = row else { break; };
        if name.is_empty() { name = current_name.clone(); }
        if parent.is_some() { parts.push(current_name); } else { root = id; }
        cursor = parent;
    }
    parts.reverse();
    let mount: Option<String> = db.query_row("SELECT COALESCE((SELECT node FROM fs_inodes WHERE ino=1),
        (SELECT CASE WHEN json_valid(value) THEN json_extract(value,'$.file_id') END FROM metadata WHERE key='native_scope'))",
        [], |r| r.get(0))?;
    db.execute("INSERT INTO sync_history(seq,mount,node,name,path_at_event,action,result,attempts,error_kind,error_summary,completed_at,operation_key)
        VALUES((SELECT CAST(value AS INTEGER) FROM metadata WHERE key='sync_history_sequence'),?1,?2,?3,?4,?5,?6,1,?7,?7,?8,?9)
        ON CONFLICT(operation_key) DO UPDATE SET seq=excluded.seq,result=excluded.result,
          error_kind=excluded.error_kind,error_summary=excluded.error_summary,completed_at=excluded.completed_at",
        params![mount.unwrap_or(root),node,name,parts.join("/"),action,result,error,now(),key])?;
    Ok(())
}

pub(crate) fn record_permission_response(db: &Connection, key: &str, node: &str,
    action: &str, native_action: &str, code: u32) -> Result<()> {
    record(db, key, node, action, "pending", Some("permission_commit_uncertain"))?;
    let detail = serde_json::json!({"permission_response": {"action": native_action, "code": code}});
    db.execute("UPDATE sync_history SET error_summary='服务器返回权限相关响应，提交状态待核对',detail=?2 WHERE operation_key=?1",
        params![key,detail.to_string()])?;
    Ok(())
}

/// Record how many automatic read-only reconciliation passes an uncertain
/// upload has used. Only updates an existing row: the first submission already
/// created it, and this must never fabricate a history row for other work.
pub(crate) fn note_reconcile_attempt(db: &Connection, key: &str, attempts: u32) -> Result<()> {
    db.execute(
        "UPDATE sync_history SET detail=json_patch(COALESCE(detail,'{}'), json_object('reconcile_attempts', ?2)) WHERE operation_key=?1",
        params![key, i64::from(attempts)],
    )?;
    Ok(())
}

/// A locally deleted, never confirmed creation stops being retried. History
/// keeps the permission evidence and explains why the queue gave up; the
/// operation type/result stays failure-shaped so the GUI can surface it.
/// `reason` distinguishes an automatic stop from a user-facing discard.
pub(crate) fn record_abandoned_upload(
    db: &Connection,
    key: &str,
    node: &str,
    attempts: u32,
    reason: &str,
) -> Result<()> {
    record(
        db,
        key,
        node,
        "upload",
        "failed",
        Some("permission_commit_uncertain"),
    )?;
    let summary = match reason {
        "user_discarded" => {
            "已放弃本次本地待上传并删除本地文件；服务器创建结果未确认，已停止重试"
        }
        _ => "本地文件已删除；服务器创建结果未确认，已停止自动重试",
    };
    let detail = serde_json::json!({
        "abandoned": reason,
        "reconcile_attempts": attempts,
    });
    db.execute(
        "UPDATE sync_history SET error_summary=?2, detail=json_patch(COALESCE(detail,'{}'), ?3) WHERE operation_key=?1",
        params![key, summary, detail.to_string()],
    )?;
    Ok(())
}

/// The bounded read-only reconciliation budget is spent without a confirmed
/// result. The operation becomes a durable terminal `abandoned` row so it can
/// no longer block pending work; local content is retained and history explains
/// the stop. A user recheck re-arms it (see `reset_upload_reconcile_budget`).
pub(crate) fn record_reconcile_exhausted(
    db: &Connection,
    key: &str,
    node: &str,
    attempts: u32,
) -> Result<()> {
    record(
        db,
        key,
        node,
        "upload",
        "failed",
        Some("permission_commit_uncertain"),
    )?;
    let detail = serde_json::json!({
        "abandoned": "reconcile_exhausted",
        "reconcile_attempts": attempts,
    });
    db.execute(
        "UPDATE sync_history SET error_summary=?2, detail=json_patch(COALESCE(detail,'{}'), ?3) WHERE operation_key=?1",
        params![
            key,
            "服务器提交结果未确认；自动核对已暂停，本地内容保留",
            detail.to_string()
        ],
    )?;
    Ok(())
}

/// Missing, unparsable, or out-of-range persisted settings are treated as absent
/// so a read-only query cannot fail on a corrupt metadata row; the defaults apply.
fn retention_value(db: &Connection, key: &str, maximum: u32) -> Result<Option<u32>> {
    let value: Option<String> = db
        .query_row("SELECT value FROM metadata WHERE key=?1", [key], |r| r.get(0))
        .optional()?;
    Ok(value
        .and_then(|value| value.parse::<u32>().ok())
        .filter(|value| (1..=maximum).contains(value)))
}

fn prune(db: &Connection, at: i64) -> Result<usize> {
    let removed = db.execute("DELETE FROM sync_history WHERE completed_at < ?1-86400*(SELECT CAST(value AS INTEGER) FROM metadata WHERE key='sync_history_days')", [at])?;
    Ok(removed + db.execute("DELETE FROM sync_history WHERE id IN (SELECT id FROM sync_history ORDER BY seq DESC LIMIT -1 OFFSET (SELECT CAST(value AS INTEGER) FROM metadata WHERE key='sync_history_records'))", [])?)
}

fn row_record(row: &rusqlite::Row<'_>) -> rusqlite::Result<SyncHistoryRecord> {
    let detail: Option<String> = row.get(13)?;
    Ok(SyncHistoryRecord { id: row.get(0)?, seq: row.get(1)?, mount: row.get(2)?, node: row.get(3)?,
        name: row.get(4)?, path_at_event: row.get(5)?, action: row.get(6)?, result: row.get(7)?,
        attempts: row.get(8)?, error_kind: row.get(9)?, error_summary: row.get(10)?,
        started_at: row.get(11)?, completed_at: row.get(12)?,
        detail: detail.map(|value| serde_json::from_str(&value).map_err(|e|
            rusqlite::Error::FromSqlConversionFailure(13,rusqlite::types::Type::Text,Box::new(e)))).transpose()? })
}
const COLUMNS: &str = "id,seq,mount,node,name,path_at_event,action,result,attempts,error_kind,error_summary,started_at,completed_at,detail";

impl Store {
    /// A failed preparation has not submitted a mutation. Keep the upload job
    /// pending and retain its body; history describes the failed attempt only.
    /// This method never accepts raw transport error strings or changes unknown
    /// commits back into retryable uploads.
    pub fn record_upload_preparation_failure(&mut self, operation: &str, kind: &str) -> Result<()> {
        if !matches!(kind,"network"|"server_rejected"|"local_storage"|"protocol") {
            return Err(Error::State("unsupported upload preparation failure category".into()));
        }
        let tx=self.db.transaction()?;
        let (node,state):(String,String)=tx.query_row("SELECT node,state FROM operations WHERE id=?1 AND kind='upload'",
            [operation],|r|Ok((r.get(0)?,r.get(1)?)))?;
        if state!="pending" { return Err(Error::State("upload preparation requires pending operation".into())); }
        let key=format!("upload:{operation}");
        let previous:Option<u32>=tx.query_row("SELECT attempts FROM sync_history WHERE operation_key=?1",
            [&key],|r|r.get(0)).optional()?;
        record(&tx,&key,&node,"upload","failed",Some(kind))?;
        if let Some(attempts)=previous {
            tx.execute("UPDATE sync_history SET attempts=?2 WHERE operation_key=?1",
                params![key,attempts.saturating_add(1)])?;
        }
        tx.commit()?;
        Ok(())
    }
    /// Retained identity only; callers must still validate visibility/root scope.
    pub fn sync_history_parent(&self, id: &str) -> Result<Option<String>> {
        Ok(self.db.query_row("SELECT parent FROM nodes WHERE id=?1",[id],|r| r.get::<_,Option<String>>(0)).optional()?.flatten())
    }
    pub fn sync_history(&self, query: &HistoryQuery) -> Result<HistoryPage> {
        let limit = query.limit.unwrap_or(100);
        if limit == 0 || limit > 500 || query.name.as_ref().is_some_and(|s| s.len()>4096)
            || query.mount.as_ref().is_some_and(|s| s.len()>1024)
            || query.result.as_deref().is_some_and(|s| !matches!(s,"ok"|"failed"|"cancelled"|"pending")) { return Err(invalid()); }
        let cursor = query.cursor.as_deref().map(|s| {
            if s.is_empty() || !s.bytes().all(|b| b.is_ascii_digit()) { return Err(invalid()); }
            s.parse::<i64>().ok().filter(|n| *n>0).ok_or_else(invalid)
        }).transpose()?;
        let mut statement = self.db.prepare(&format!("SELECT {COLUMNS} FROM sync_history
          WHERE (?1 IS NULL OR mount=?1) AND (?2 IS NULL OR result=?2)
            AND (?3 IS NULL OR instr(lower(name),lower(?3))>0) AND (?4 IS NULL OR seq<?4)
          ORDER BY seq DESC,id DESC LIMIT ?5"))?;
        let mut records = statement.query_map(params![query.mount,query.result,query.name,cursor,limit+1],row_record)?
            .collect::<std::result::Result<Vec<_>,_>>()?;
        let has_more = records.len()>limit as usize;
        records.truncate(limit as usize);
        let next_cursor = if has_more { records.last().map(|r|r.seq.to_string()) } else { None };
        Ok(HistoryPage { records,next_cursor })
    }
    pub fn sync_history_record(&self, id: i64) -> Result<Option<SyncHistoryRecord>> {
        if id<=0 { return Err(invalid()); }
        Ok(self.db.query_row(&format!("SELECT {COLUMNS} FROM sync_history WHERE id=?1"), [id], row_record).optional()?)
    }
    pub fn clear_sync_history(&mut self, request: &ClearHistory) -> Result<usize> {
        let tx = self.db.transaction()?;
        let removed = match request {
            ClearHistory::All => tx.execute("DELETE FROM sync_history", [])?,
            ClearHistory::OlderThanDays { days } => tx.execute("DELETE FROM sync_history WHERE completed_at < ?1", [now().saturating_sub(i64::from(days.unwrap_or(DEFAULT_RETENTION_DAYS))*86400)])?,
        };
        tx.commit()?;
        Ok(removed)
    }
    /// Read-only view of the persisted retention. The GUI shows these limits and
    /// must still work when the metadata rows are missing or damaged.
    pub fn sync_history_retention(&self) -> Result<(u32, u32)> {
        Ok((
            retention_value(&self.db, "sync_history_days", MAX_RETENTION_DAYS)?
                .unwrap_or(DEFAULT_RETENTION_DAYS),
            retention_value(&self.db, "sync_history_records", MAX_RETENTION_RECORDS)?
                .unwrap_or(DEFAULT_RETENTION_RECORDS),
        ))
    }
    /// Metadata-only change: pruning history never deletes files, drafts, cached
    /// bodies, or queued operations. Callers must still bind requests to a task.
    pub fn set_sync_history_retention(&mut self, days: u32, records: u32) -> Result<()> {
        if days == 0 || days > MAX_RETENTION_DAYS || records == 0 || records > MAX_RETENTION_RECORDS {
            return Err(invalid());
        }
        let tx=self.db.transaction()?;
        tx.execute("UPDATE metadata SET value=?1 WHERE key='sync_history_days'", [days.to_string()])?;
        tx.execute("UPDATE metadata SET value=?1 WHERE key='sync_history_records'", [records.to_string()])?;
        prune(&tx,now())?;
        tx.commit()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Batch, Change, DownloadTicket, Event, Node, NodeMetadata};

    fn fixture() -> (tempfile::TempDir, Store) {
        let dir=tempfile::tempdir().unwrap();
        let mut store=Store::open(dir.path()).unwrap();
        let make=|id:&str,parent:Option<&str>,directory:bool| Node {
            id:id.into(),parent:parent.map(str::to_owned),name:id.into(),version:"v1".into(),directory,
            metadata:Some(NodeMetadata { size:3,mtime:1,mode:if directory {0o755}else{0o644} }),
        };
        store.install_remote_snapshot(&[make("root",None,true),make("folder",Some("root"),true),make("file",Some("root"),false)],"0","server").unwrap();
        store.ensure_fs_root("root").unwrap();
        (dir,store)
    }
    fn rows(store:&Store)->Vec<SyncHistoryRecord> {
        store.sync_history(&HistoryQuery::default()).unwrap().records
    }
    fn add(store:&mut Store,key:&str,result:&str) {
        let tx=store.db.transaction().unwrap();
        record(&tx,key,"file","upload",result,None).unwrap();
        tx.commit().unwrap();
    }

    #[test]
    fn history_is_atomic_with_operation_and_is_not_built_by_queries() {
        let (_dir,mut store)=fixture();
        assert!(rows(&store).is_empty());
        {
            let tx=store.db.transaction().unwrap();
            record(&tx,"rollback","file","upload","ok",None).unwrap();
            // Dropping rolls both the result and sequence back.
        }
        assert!(rows(&store).is_empty());
        let op=store.save_local("file",b"saved bytes").unwrap();
        store.mark_upload_started(&op).unwrap();
        let pending=rows(&store);
        assert_eq!(pending.len(),1);
        assert_eq!(pending[0].result,"pending");
        store.mark_upload_uncertain(&op).unwrap();
        let uncertain=rows(&store);
        assert_eq!(uncertain[0].error_kind.as_deref(),Some("uncertain_commit"));
        assert_eq!(uncertain[0].result,"pending");
        store.confirm_upload(&op,"v2").unwrap();
        let completed=rows(&store);
        assert_eq!(completed.len(),1);
        assert_eq!(completed[0].id,pending[0].id);
        assert_eq!(completed[0].result,"ok");
        assert_eq!(completed[0].attempts,1);
        assert_eq!(completed[0].mount,"root");
        assert_eq!(completed[0].path_at_event,"file");
        assert!(completed[0].seq>uncertain[0].seq);
        assert!(completed[0].error_kind.is_none());
    }

    #[test]
    fn failed_download_retries_merge_without_claiming_cancellation_as_success() {
        let (_dir,mut store)=fixture();
        let ticket=DownloadTicket{id:"file".into(),version:"v1".into()};
        for _ in 0..2 {
            store.begin_download(&ticket).unwrap();
            store.fail_download(&ticket,crate::download_jobs::DownloadFailureKind::Transport,None,1).unwrap();
        }
        let records=rows(&store);
        assert_eq!(records.len(),1);
        assert_eq!(records[0].attempts,2);
        assert_eq!(records[0].result,"failed");
        assert_eq!(records[0].error_kind.as_deref(),Some("transport"));
        store.finish_download(&ticket).unwrap();
        assert_eq!(rows(&store)[0].result,"cancelled");
        assert!(store.open_cached("file").is_err());
    }

    #[test]
    fn downloaded_receipt_merges_with_transfer_completion() {
        let (_dir,mut store)=fixture();
        let ticket=DownloadTicket{id:"file".into(),version:"v1".into()};
        store.begin_download(&ticket).unwrap();
        store.publish_download(&ticket,b"abc",&crate::sha256(b"abc")).unwrap();
        // The publication caller is wired by integration; exercise its exact
        // transaction helper here as well as the durable transfer completion.
        let tx=store.db.transaction().unwrap();
        record_download_publication(&tx,"file","v1").unwrap();
        tx.commit().unwrap();
        store.finish_download(&ticket).unwrap();
        assert_eq!(rows(&store).len(),1);
        assert_eq!(rows(&store)[0].result,"ok");
        assert_eq!(rows(&store)[0].attempts,1);
    }

    #[test]
    fn version_keys_do_not_collide_on_delimiters() {
        let (_dir,mut store)=fixture();
        let tx=store.db.transaction().unwrap();
        record_download_publication(&tx,"a:b","c").unwrap();
        record_download_publication(&tx,"a","b:c").unwrap();
        tx.commit().unwrap();
        assert_eq!(rows(&store).len(),2);
    }

    #[test]
    fn remote_events_are_recorded_once_in_application_transaction() {
        let (_dir,mut store)=fixture();
        let mut node=store.node("file").unwrap();
        node.version="v2".into();
        let event=Event{id:"event-one".into(),change:Change::Upsert(node)};
        store.receive(&Batch{from:"0".into(),to:"1".into(),events:vec![event.clone()]}).unwrap();
        assert!(rows(&store).is_empty());
        store.drain().unwrap();
        assert_eq!(rows(&store).len(),1);
        assert_eq!(rows(&store)[0].action,"remote_change");
        store.receive(&Batch{from:"1".into(),to:"2".into(),events:vec![event]}).unwrap();
        store.drain().unwrap();
        assert_eq!(rows(&store).len(),1);
    }

    #[test]
    fn rename_move_and_remove_follow_verified_namespace_receipts() {
        let (_dir,mut store)=fixture();
        store.rename_local("root","file","root","renamed",true).unwrap();
        let rename=store.next_rename().unwrap().unwrap();
        store.mark_rename_started(&rename.job.operation).unwrap();
        let mut receipt=store.node("file").unwrap();
        receipt.version="v2".into();
        store.confirm_rename(&rename.job.operation,&receipt,true).unwrap();
        assert_eq!(rows(&store)[0].action,"rename");
        assert_eq!(rows(&store)[0].result,"ok");
        store.rename_local("root","renamed","folder","moved",true).unwrap();
        let moved=store.next_rename().unwrap().unwrap();
        store.mark_rename_started(&moved.job.operation).unwrap();
        let mut receipt=store.node("file").unwrap();
        receipt.version="v3".into();
        store.confirm_rename(&moved.job.operation,&receipt,true).unwrap();
        assert_eq!(rows(&store)[0].action,"move");
        assert_eq!(rows(&store)[0].path_at_event,"folder/moved");
        store.unlink_local("folder","moved","file",false).unwrap();
        let removed=store.next_remove().unwrap().unwrap();
        store.mark_remove_started(&removed.job.operation).unwrap();
        store.confirm_remove(&removed.job.operation,"file").unwrap();
        let records=rows(&store);
        assert_eq!(records.len(),3);
        assert_eq!(records[0].action,"delete");
        assert_eq!(records[0].result,"ok");
        assert_eq!(records[0].path_at_event,"folder/moved");
    }

    #[test]
    fn pages_filter_literal_names_and_keep_monotonic_sequence_after_clear() {
        let (_dir,mut store)=fixture();
        store.db.execute("UPDATE nodes SET name='100%_Name' WHERE id='file'",[]).unwrap();
        for i in 0..5 { add(&mut store,&format!("{i}"),if i%2==0 {"failed"}else{"ok"}); }
        let query=HistoryQuery{mount:Some("root".into()),result:Some("failed".into()),name:Some("%_nAMe".into()),limit:Some(2),..Default::default()};
        let first=store.sync_history(&query).unwrap();
        assert_eq!(first.records.len(),2);
        let second=store.sync_history(&HistoryQuery{cursor:first.next_cursor.clone(),..query}).unwrap();
        assert_eq!(second.records.len(),1);
        assert!(second.next_cursor.is_none());
        assert!(second.records[0].seq<first.records[1].seq);
        assert!(store.sync_history(&HistoryQuery{name:Some("100X".into()),..Default::default()}).unwrap().records.is_empty());
        let maximum=rows(&store)[0].seq;
        store.clear_sync_history(&ClearHistory::All).unwrap();
        add(&mut store,"new","ok");
        assert!(rows(&store)[0].seq>maximum);
        assert!(store.sync_history(&HistoryQuery{cursor:Some("-1".into()),..Default::default()}).is_err());
        assert!(store.sync_history(&HistoryQuery{limit:Some(501),..Default::default()}).is_err());
    }

    #[test]
    fn retention_and_clear_preserve_local_content_and_operations() {
        let (_dir,mut store)=fixture();
        let op=store.save_local("file",b"private unsent body").unwrap();
        store.set_sync_history_retention(30,2).unwrap();
        for i in 0..5 { add(&mut store,&format!("retention:{i}"),"failed"); }
        assert_eq!(rows(&store).len(),2);
        let bytes=store.open_cached("file").unwrap().size().unwrap();
        assert_eq!(store.clear_sync_history(&ClearHistory::All).unwrap(),2);
        assert_eq!(store.open_cached("file").unwrap().size().unwrap(),bytes);
        assert_eq!(store.upload_job(&op).unwrap().state,"pending");
        assert!(serde_json::from_str::<ClearHistory>(r#"{"mode":"older_than_days","days":-1}"#).is_err());
        assert!(serde_json::from_str::<ClearHistory>(r#"{"mode":"all","days":30}"#).is_err());
        assert!(serde_json::from_str::<ClearHistory>(r#"{"mode":"all","unexpected":true}"#).is_err());
        assert!(serde_json::from_str::<ClearHistory>(r#"{"mode":"oops"}"#).is_err());
    }

    #[test]
    fn retention_reads_defaults_rejects_invalid_settings_and_only_affects_metadata_history() {
        let (_dir,mut store)=fixture();
        assert_eq!(store.sync_history_retention().unwrap(),(DEFAULT_RETENTION_DAYS,DEFAULT_RETENTION_RECORDS));
        let op=store.save_local("file",b"private unsent body").unwrap();
        let bytes=store.open_cached("file").unwrap().size().unwrap();
        store.set_sync_history_retention(7,1).unwrap();
        assert_eq!(store.sync_history_retention().unwrap(),(7,1));
        add(&mut store,"keep","failed");
        assert_eq!(rows(&store).len(),1);
        for (days,records) in [(0,1),(3_651,1),(1,0),(1,1_000_001)] {
            assert!(store.set_sync_history_retention(days,records).is_err());
        }
        assert_eq!(store.sync_history_retention().unwrap(),(7,1));
        // Missing, corrupt, or out-of-range rows fall back to the defaults instead
        // of turning a read-only settings query into a hard failure.
        store.db.execute_batch("UPDATE metadata SET value='corrupt' WHERE key='sync_history_days';
          DELETE FROM metadata WHERE key='sync_history_records';").unwrap();
        assert_eq!(store.sync_history_retention().unwrap(),(DEFAULT_RETENTION_DAYS,DEFAULT_RETENTION_RECORDS));
        store.db.execute_batch("UPDATE metadata SET value='0' WHERE key='sync_history_days';
          INSERT INTO metadata(key,value) VALUES('sync_history_records','999999999');").unwrap();
        assert_eq!(store.sync_history_retention().unwrap(),(DEFAULT_RETENTION_DAYS,DEFAULT_RETENTION_RECORDS));
        // Retention and pruning are metadata-only: unsent content and queued
        // upload operations survive both the setting change and the prune.
        assert_eq!(store.open_cached("file").unwrap().size().unwrap(),bytes);
        assert_eq!(store.upload_job(&op).unwrap().state,"pending");
        assert_eq!(rows(&store).len(),1);
    }

    #[test]
    fn retained_window_bookkeeping_is_exact_and_never_counts_the_window() {
        let (_dir,mut store)=fixture();
        let counter=|s:&Store|->i64 { s.db.query_row(
            "SELECT CAST(value AS INTEGER) FROM metadata WHERE key='sync_history_rows'",[],|r|r.get(0)).unwrap() };
        let table=|s:&Store|->i64 { s.db.query_row("SELECT COUNT(*) FROM sync_history",[],|r|r.get(0)).unwrap() };
        for index in 0..20 { add(&mut store,&format!("k{index}"),"ok"); }
        assert_eq!(counter(&store),table(&store));
        assert_eq!(table(&store),20);
        // An append that merges into an existing operation must not double-count.
        add(&mut store,"k3","failed");
        assert_eq!(counter(&store),table(&store));
        // A durable delete outside the triggers keeps the counter honest.
        store.db.execute("DELETE FROM sync_history WHERE operation_key='k5'",[]).unwrap();
        assert_eq!(counter(&store),table(&store));
        // Lowering the cap prunes the excess; the counter follows the deletes.
        store.set_sync_history_retention(30,5).unwrap();
        assert_eq!(table(&store),5);
        assert_eq!(counter(&store),table(&store));
        // The append trigger must not scan or count the retained window.
        let trigger:String=store.db.query_row(
            "SELECT sql FROM sqlite_master WHERE name='history_prune_insert'",[],|r|r.get(0)).unwrap();
        assert!(!trigger.contains("COUNT(*)"),"append trigger must not count retained rows: {trigger}");
        assert!(trigger.contains("sync_history_rows"));
        let keys:Vec<String>={
            let mut query=store.db.prepare("SELECT operation_key FROM sync_history ORDER BY seq DESC").unwrap();
            query.query_map([],|r|r.get(0)).unwrap().collect::<std::result::Result<_,_>>().unwrap()
        };
        assert_eq!(keys,["k3","k19","k18","k17","k16"]);
    }

    #[test]
    fn schema_23_upgrade_preserves_dirty_data_and_future_schema_is_rejected() {
        let (dir,mut store)=fixture();
        store.save_local("file",b"not uploaded").unwrap();
        // Emulate the real v23 shape, not a new schema with only its version
        // number changed. These durable history objects did not exist in v23.
        store.db.execute_batch("DROP TRIGGER history_upload; DROP TRIGGER history_namespace;
          DROP TRIGGER history_permission_namespace_extra;
          DROP TRIGGER history_download; DROP TRIGGER history_download_removed;
          DROP TABLE sync_history;
          DELETE FROM metadata WHERE key LIKE 'sync_history_%';").unwrap();
        store.db.pragma_update(None,"user_version",23).unwrap();
        drop(store);
        let store=Store::open(dir.path()).unwrap();
        assert!(rows(&store).is_empty());
        assert_eq!(store.open_cached("file").unwrap().size().unwrap(),12);
        let backups=std::fs::read_dir(dir.path()).unwrap().filter_map(|e|e.ok()).filter(|e|
            e.file_name().to_string_lossy().starts_with(&format!("state-before-v{}-",crate::STATE_SCHEMA_VERSION))).count();
        assert_eq!(backups,1);
        let version:i64=store.db.pragma_query_value(None,"user_version",|r|r.get(0)).unwrap();
        assert_eq!(version,crate::STATE_SCHEMA_VERSION);
        store.db.pragma_update(None,"user_version",crate::STATE_SCHEMA_VERSION+1).unwrap();
        drop(store);
        assert!(matches!(Store::open(dir.path()),Err(Error::State(reason)) if reason.contains("newer")));
    }

    #[test]
    fn history_survives_store_restart() {
        let (dir,mut store)=fixture();
        add(&mut store,"persist","failed");
        let before=rows(&store);
        drop(store);
        let store=Store::open(dir.path()).unwrap();
        assert_eq!(rows(&store),before);
    }

    #[test]
    fn failed_migration_rolls_back_schema_and_keeps_coherent_backup() {
        let (dir,mut store)=fixture();
        store.save_local("file",b"pending survives").unwrap();
        store.db.execute_batch("DROP TRIGGER history_upload; DROP TRIGGER history_namespace;
          DROP TRIGGER history_permission_namespace_extra;
          DROP TRIGGER history_download; DROP TRIGGER history_download_removed;
          DROP TABLE sync_history;
          DELETE FROM metadata WHERE key LIKE 'sync_history_%';
          CREATE VIEW sync_history AS SELECT 1 AS incompatible;
          PRAGMA user_version=23;").unwrap();
        drop(store);
        assert!(Store::open(dir.path()).is_err());
        let db=Connection::open(dir.path().join("state.sqlite")).unwrap();
        let version:i64=db.pragma_query_value(None,"user_version",|r|r.get(0)).unwrap();
        assert_eq!(version,23);
        let dirty:bool=db.query_row("SELECT dirty_blob IS NOT NULL FROM nodes WHERE id='file'",[],|r|r.get(0)).unwrap();
        assert!(dirty);
        let metadata:i64=db.query_row("SELECT COUNT(*) FROM metadata WHERE key LIKE 'sync_history_%'",[],|r|r.get(0)).unwrap();
        assert_eq!(metadata,0);
        let backups=std::fs::read_dir(dir.path()).unwrap().filter_map(|e|e.ok()).filter(|e|
            e.file_name().to_string_lossy().starts_with(&format!("state-before-v{}-",crate::STATE_SCHEMA_VERSION))).collect::<Vec<_>>();
        assert_eq!(backups.len(),1);
        let backup=Connection::open(backups[0].path()).unwrap();
        let backup_version:i64=backup.pragma_query_value(None,"user_version",|r|r.get(0)).unwrap();
        assert_eq!(backup_version,23);
    }

    #[test]
    fn download_history_never_copies_unstructured_error_bodies() {
        let (_dir,mut store)=fixture();
        let ticket=DownloadTicket{id:"file".into(),version:"v1".into()};
        store.begin_download(&ticket).unwrap();
        store.db.execute("UPDATE transfers SET state='retry',error='synthetic-secret-body'",[]).unwrap();
        let records=rows(&store);
        assert_eq!(records[0].error_kind.as_deref(),Some("download_failed"));
        assert!(!serde_json::to_string(&records).unwrap().contains("synthetic-secret-body"));
    }

    #[test]
    fn history_does_not_impose_a_new_depth_limit_on_remote_events_or_uploads() {
        let dir=tempfile::tempdir().unwrap();
        let mut store=Store::open(dir.path()).unwrap();
        let mut nodes=vec![Node { id:"root".into(),parent:None,name:"root".into(),version:"v1".into(),directory:true,
            metadata:Some(NodeMetadata{size:0,mtime:1,mode:0o755}) }];
        let mut parent="root".to_owned();
        let mut names=Vec::new();
        for depth in 0..160 {
            let id=format!("d{depth:03}");
            names.push(id.clone());
            nodes.push(Node{id:id.clone(),parent:Some(parent),name:id.clone(),version:"v1".into(),directory:true,
                metadata:Some(NodeMetadata{size:0,mtime:1,mode:0o755})});
            parent=id;
        }
        names.push("deep-file".into());
        nodes.push(Node{id:"deep-file".into(),parent:Some(parent),name:"deep-file".into(),version:"v1".into(),directory:false,
            metadata:Some(NodeMetadata{size:3,mtime:1,mode:0o644})});
        store.install_remote_snapshot(&nodes,"0","server").unwrap();
        store.ensure_fs_root("root").unwrap();
        let mut changed=store.node("deep-file").unwrap();
        changed.version="v2".into();
        store.receive(&Batch{from:"0".into(),to:"1".into(),events:vec![Event{id:"deep-event".into(),change:Change::Upsert(changed)}]}).unwrap();
        store.drain().unwrap();
        assert_eq!(store.cursor().unwrap(),"1");
        assert_eq!(rows(&store)[0].path_at_event,names.join("/"));
        let operation=store.save_local("deep-file",b"deep edit").unwrap();
        store.mark_upload_started(&operation).unwrap();
        assert_eq!(rows(&store)[0].action,"upload");
        assert_eq!(rows(&store)[0].path_at_event,names.join("/"));
    }

    #[test]
    fn preparation_failure_merges_attempts_without_submitting_or_reclassifying_uncertain_work() {
        let (_dir,mut store)=fixture();
        let operation=store.save_local("file",b"retained local bytes").unwrap();
        store.record_upload_preparation_failure(&operation,"network").unwrap();
        store.record_upload_preparation_failure(&operation,"server_rejected").unwrap();
        assert_eq!(rows(&store).len(),1);
        assert_eq!(rows(&store)[0].attempts,2);
        assert_eq!(rows(&store)[0].result,"failed");
        assert_eq!(store.upload_job(&operation).unwrap().state,"pending");
        assert_eq!(store.open_cached("file").unwrap().size().unwrap(),20);
        assert!(store.record_upload_preparation_failure(&operation,"raw credential-like text").is_err());
        assert_eq!(rows(&store)[0].attempts,2);
        store.mark_upload_started(&operation).unwrap();
        assert_eq!(rows(&store)[0].attempts,3);
        store.mark_upload_uncertain(&operation).unwrap();
        assert!(store.record_upload_preparation_failure(&operation,"network").is_err());
        assert_eq!(store.upload_job(&operation).unwrap().state,"uncertain");
        assert_eq!(rows(&store)[0].result,"pending");
    }
}
