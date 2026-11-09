use rusqlite::{Connection, OpenFlags, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
#[path = "sync_history.rs"]
pub mod sync_history;
use std::{
    collections::HashMap,
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("database: {0}")]
    Db(#[from] rusqlite::Error),
    #[error("local I/O: {0}")]
    Io(#[from] std::io::Error),
    #[error("journal format: {0}")]
    Json(#[from] serde_json::Error),
    #[error("state precondition: {0}")]
    State(String),
    #[error("content hash mismatch")]
    Integrity,
    #[error("content is not available offline")]
    Offline,
    #[error("unsupported native capability: {0}")]
    Unsupported(&'static str),
    #[cfg(feature = "native")]
    #[error("native snapshot changed during enumeration")]
    SnapshotChanged,
    #[cfg(feature = "native")]
    #[error("selected native root is unavailable or has changed identity")]
    RootUnavailable,
    #[cfg(feature = "native")]
    #[error("native protocol: {0}")]
    Native(#[from] sproto::Error),
}
pub type Result<T> = std::result::Result<T, Error>;
impl Error {
    pub fn is_not_found(&self) -> bool {
        matches!(self, Self::Db(rusqlite::Error::QueryReturnedNoRows))
            || matches!(self, Self::Io(e) if e.kind() == std::io::ErrorKind::NotFound)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct NodeMetadata {
    pub size: u64,
    pub mtime: u64,
    pub mode: u32,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Node {
    pub id: String,
    pub parent: Option<String>,
    pub name: String,
    pub version: String,
    pub directory: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<NodeMetadata>,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Change {
    Upsert(Node),
    Delete { id: String },
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Event {
    pub id: String,
    pub change: Change,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Batch {
    pub from: String,
    pub to: String,
    pub events: Vec<Event>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Status {
    OnlineOnly,
    LocalComplete,
    PinnedComplete,
    Updating,
    Pinning,
    PendingUpload,
    Conflict,
    Deleted,
}
#[derive(Debug)]
pub struct DownloadTicket {
    pub id: String,
    pub version: String,
}
/// Result of releasing one node's local body. `Skipped` means the node had no
/// cached body to reclaim; it is a no-op rather than a successful release.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReleaseOutcome {
    Released,
    Skipped,
}
/// Offline availability summary over non-directory nodes. `state` is one of
/// `empty`, `pinned`, `online_only`, `partial`, or `complete`; it describes
/// what the local cache can serve while the server is unreachable.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CacheAvailability {
    pub state: &'static str,
    pub total: u64,
    pub offline: u64,
    pub pinned: u64,
}

struct WriterLock(File);
impl Drop for WriterLock {
    fn drop(&mut self) {
        // FUSE helpers can inherit the open file description between fork and
        // exec. Explicit unlock ends ownership without waiting for their close.
        let _ = self.0.unlock();
    }
}
pub struct Store {
    pub(crate) db: Connection,
    pub(crate) root: PathBuf,
    pub(crate) handles: Arc<Mutex<HashMap<String, (String, usize)>>>,
    // Shared with the `SharedStore` and every read-only handle it opens: the
    // scheduler observations describe the writer's domain, not one connection.
    pub(crate) namespace_runtime: Arc<Mutex<crate::namespace_schedule::NamespaceRuntime>>,
    /// Deferred LRU recency for cached file opens. The FUSE open path must not
    /// append WAL frames or fsync; entries are merged into `cache_access` in
    /// one transaction before a budget/trim decision and on drop. Maintained as
    /// an amortized O(1) ledger because this is the hottest FUSE path.
    pub(crate) cache_recency: Mutex<crate::cache_policy::CacheRecency>,
    // Issue #24 counters; the Store writer lock serializes access, so plain
    // integers are cheaper than another atomic layer.
    pub(crate) draft_stats: DraftWriteStats,
    /// Writer-local effective-decision memo (#26); see `PermissionCache`. It is
    /// invalidated wholesale whenever a transaction commits, so it never
    /// outlives a permission, policy or namespace write.
    pub(crate) permission_cache: Mutex<crate::write_permissions::PermissionCache>,
    // Fields drop in order: close SQLite before releasing writer ownership.
    // Read-only handles own no lock and therefore carry None.
    _writer_lock: Option<WriterLock>,
    // #28: incremental remove-barrier index; empty on read-only handles.
    pub(crate) barrier: crate::removals::RemoveBarrierCache,
}

/// Counters that make the draft write path observable without tracing every
/// syscall. Diagnostic only: no behaviour depends on these values.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DraftWriteStats {
    /// Content writes routed through a draft handle.
    pub writes: u64,
    /// Durable draft-metadata statements: dirty-epoch marks plus checkpoint
    /// updates (checkpoint advances share the snapshot transaction).
    pub metadata_commits: u64,
    /// fsync calls on the mutable draft file itself (begin copy, checkpoint).
    pub draft_file_fsyncs: u64,
    /// Checkpoints that produced an upload snapshot.
    pub checkpoints: u64,
}

/// A handle retains its immutable version even if a newer blob is published.
pub struct ReadHandle {
    file: File,
    blob: String,
    version: String,
    handles: Arc<Mutex<HashMap<String, (String, usize)>>>,
}
impl ReadHandle {
    pub fn version(&self) -> &str {
        &self.version
    }
    pub fn size(&self) -> std::io::Result<u64> {
        Ok(self.file.metadata()?.len())
    }
    #[cfg(unix)]
    pub fn read_at(&self, buffer: &mut [u8], offset: u64) -> std::io::Result<usize> {
        use std::os::unix::fs::FileExt;
        self.file.read_at(buffer, offset)
    }
}
impl Read for ReadHandle {
    fn read(&mut self, out: &mut [u8]) -> std::io::Result<usize> {
        self.file.read(out)
    }
}
impl Drop for ReadHandle {
    fn drop(&mut self) {
        let mut h = self.handles.lock().unwrap();
        if let Some((_, n)) = h.get_mut(&self.blob) {
            *n -= 1;
            if *n == 0 {
                h.remove(&self.blob);
            }
        }
    }
}

/// Version of the durable state contract, including journal semantics.
pub const STATE_SCHEMA_VERSION: i64 = 29;

/// Automatic read-only reconciliation attempts allowed for one uncertain
/// upload. The budget is durable so a restart cannot restart an unbounded
/// retry loop; a user-requested recheck resets it deliberately.
pub const MAX_UPLOAD_RECONCILE_ATTEMPTS: u32 = 5;

/// Backoff before the next automatic read-only reconciliation pass. The fifth
/// attempt is not scheduled: the operation keeps its uncertain state but the
/// queue stops selecting it until the budget is reset.
fn upload_reconcile_delay(attempts: u32) -> i64 {
    match attempts {
        0 | 1 => 10,
        2 => 30,
        3 => 120,
        4 => 300,
        _ => 0,
    }
}

/// Candidate nodes for downloads without rescanning `nodes`: stale cached
/// content (`nodes_cached`), pinned subtrees (`nodes_pinned` seed plus the
/// parent index for recursion), and download-intent transfers (unique prefix).
/// Each branch carries the columns the original filter needs, so the outer
/// statement never joins `nodes` again and cannot degrade into a full scan;
/// the original filter still runs over the (small) candidate set. Per-candidate
/// protection checks still run in `downloads_needed`.
const DOWNLOAD_CANDIDATES: &str = "\
WITH RECURSIVE pinned_nodes(id,remote_version,cached,local_version,directory,deleted,dirty_blob) AS (
    SELECT id,remote_version,blob IS NOT NULL,local_version,directory,deleted,dirty_blob
      FROM nodes WHERE pinned=1
    UNION
    SELECT n.id,n.remote_version,n.blob IS NOT NULL,n.local_version,n.directory,n.deleted,n.dirty_blob
      FROM nodes n JOIN pinned_nodes p ON n.parent=p.id),
candidates(id,remote_version,cached,local_version,directory,deleted,dirty_blob) AS (
    SELECT id,remote_version,blob IS NOT NULL,local_version,directory,deleted,dirty_blob
      FROM nodes WHERE blob IS NOT NULL AND local_version != remote_version
    UNION
    SELECT id,remote_version,cached,local_version,directory,deleted,dirty_blob FROM pinned_nodes
    UNION
    SELECT n.id,n.remote_version,n.blob IS NOT NULL,n.local_version,n.directory,n.deleted,n.dirty_blob
      FROM transfers t JOIN nodes n ON n.id=t.node AND n.remote_version=t.version
     WHERE t.direction='download' AND t.state IN ('pending','inflight','retry','failed'))
SELECT id,remote_version,cached FROM candidates c
WHERE directory=0 AND deleted=0 AND id NOT IN (SELECT id FROM access_blocked)
  AND dirty_blob IS NULL AND (cached=0 OR local_version!=remote_version)";

/// True when `table` already carries `column`. The schema 29 upgrade guards
/// every `ALTER TABLE` with this check so an interrupted upgrade can be re-run
/// without failing on the already-added columns.
fn table_has_column(db: &Connection, table: &str, column: &str) -> Result<bool> {
    let mut query = db.prepare(&format!("PRAGMA table_info({table})"))?;
    let mut rows = query.query([])?;
    while let Some(row) = rows.next()? {
        if row.get::<_, String>(1)? == column {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Add only the columns a schema 29 database is missing.
fn add_missing_columns(db: &Connection, table: &str, columns: &[(&str, &str)]) -> Result<()> {
    for (name, definition) in columns {
        if !table_has_column(db, table, name)? {
            db.execute_batch(&format!("ALTER TABLE {table} ADD COLUMN {definition};"))?;
        }
    }
    Ok(())
}

/// Schema 29 migration (#9 TB1): one forward-only migration that adds the
/// durable commit-diagnostic columns (#9), the recovery journal (#10) and the
/// platform state push queue (#11). This is deliberately migration-only: no
/// write path consumes the new columns yet (TB2/TB3 do that after #14 lands).
///
/// Runs inside `Store::open`'s single migration transaction, after the module
/// migrations and before `user_version` is raised, so a failure rolls back the
/// whole upgrade and leaves the VACUUM INTO backup as the only artifact.
///
/// Link to #14: legacy `uncertain` rows are backfilled as
/// `unknown_retryable`/`interrupted`. #14 later introduces an explicit
/// `abandoned` terminal state for exhausted reconcile budgets; TB2 remaps those
/// old rows to `unknown_fatal`/`reconcile_exhausted`. TB1 intentionally neither
/// emits nor depends on `abandoned`.
fn migrate_v29(db: &Connection, version: i64) -> Result<()> {
    if version >= 29 {
        return Ok(());
    }
    // (a) operations: unified commit diagnosis (#9 core).
    add_missing_columns(
        db,
        "operations",
        &[
            (
                "outcome",
                "outcome TEXT CHECK(outcome IN ('confirmed','conflict','unknown_retryable','unknown_fatal'))",
            ),
            ("outcome_category", "outcome_category TEXT"),
            ("last_error", "last_error TEXT"),
            ("next_retry_at", "next_retry_at INTEGER NOT NULL DEFAULT 0"),
            ("updated_at", "updated_at INTEGER NOT NULL DEFAULT 0"),
        ],
    )?;
    // (b) namespace_ops: the same diagnosis plus a durable attempt counter. The
    // runtime currently keeps namespace backoff in a process-local HashMap
    // (`namespace_schedule::NamespaceRuntime`); TB2 moves it onto these columns.
    add_missing_columns(
        db,
        "namespace_ops",
        &[
            ("attempts", "attempts INTEGER NOT NULL DEFAULT 0"),
            (
                "outcome",
                "outcome TEXT CHECK(outcome IN ('confirmed','conflict','unknown_retryable','unknown_fatal'))",
            ),
            ("outcome_category", "outcome_category TEXT"),
            ("last_error", "last_error TEXT"),
            ("next_retry_at", "next_retry_at INTEGER NOT NULL DEFAULT 0"),
            ("updated_at", "updated_at INTEGER NOT NULL DEFAULT 0"),
        ],
    )?;
    // (c) transfers: downloads report through the same four-state exit. The
    // legacy `error` JSON stays intact for read-only compatibility.
    add_missing_columns(
        db,
        "transfers",
        &[
            (
                "outcome",
                "outcome TEXT CHECK(outcome IN ('confirmed','conflict','unknown_retryable','unknown_fatal'))",
            ),
            ("outcome_category", "outcome_category TEXT"),
            ("last_error", "last_error TEXT"),
            ("next_retry_at", "next_retry_at INTEGER NOT NULL DEFAULT 0"),
        ],
    )?;
    // (d) #11: the platform push state is orthogonal to the sync state. Columns
    // and queues only; the Windows adapter that consumes them is future work.
    if !table_has_column(db, "nodes", "in_sync")? {
        db.execute_batch(
            "ALTER TABLE nodes ADD COLUMN in_sync INTEGER NOT NULL DEFAULT 0 CHECK(in_sync IN (0,1));",
        )?;
    }
    db.execute_batch(
        "
        -- Durable migration ledger (fixes the old IF NOT EXISTS + direct
        -- user_version chain that silently skipped future ALTERs).
        CREATE TABLE IF NOT EXISTS schema_migrations(
            version INTEGER PRIMARY KEY, applied_at INTEGER NOT NULL);
        INSERT OR IGNORE INTO schema_migrations(version,applied_at) VALUES
            (28,unixepoch()),(29,unixepoch());

        -- (e) #10: recovery runs keep retrying/recovering distinct from the
        -- needs-attention terminal state. No consumer in TB1.
        CREATE TABLE IF NOT EXISTS recovery_runs(
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            cause TEXT NOT NULL CHECK(cause IN
                ('initial_index','permission_recheck','cursor_invalidated',
                 'filtered_type_changed','snapshot_changed','rescan_later',
                 'merge_mode','root_unavailable')),
            status TEXT NOT NULL CHECK(status IN
                ('retrying','recovering','needs_attention','recovered')),
            attempts INTEGER NOT NULL DEFAULT 0,
            last_error TEXT,
            next_retry_at INTEGER NOT NULL DEFAULT 0,
            started_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL);
        CREATE INDEX IF NOT EXISTS recovery_open
            ON recovery_runs(status,next_retry_at)
            WHERE status IN ('retrying','recovering');

        -- (f) #11: pushable state changes, delivered idempotently by the
        -- platform adapter (CFAPI on Windows). The table is the durable tail;
        -- FUSE invalidation stays the Linux transport.
        CREATE TABLE IF NOT EXISTS platform_state_changes(
            seq INTEGER PRIMARY KEY AUTOINCREMENT,
            node TEXT NOT NULL,
            kind TEXT NOT NULL CHECK(kind IN
                ('placeholder','hydrate','hydrate_progress','dehydrate',
                 'in_sync','conflict','delete')),
            payload TEXT,
            created_at INTEGER NOT NULL,
            delivered_at INTEGER,
            attempts INTEGER NOT NULL DEFAULT 0,
            last_error TEXT);
        CREATE INDEX IF NOT EXISTS platform_state_pending
            ON platform_state_changes(seq) WHERE delivered_at IS NULL;

        -- (g) One read-only join for the status interface: the authoritative
        -- state stays in each journal, this view only normalises the diagnosis.
        CREATE VIEW IF NOT EXISTS commit_diagnostics AS
            SELECT 'operation' AS scope, o.id, o.node, o.kind AS op_kind, o.state,
                   o.outcome, o.outcome_category, o.last_error,
                   o.reconcile_attempts AS attempts, o.next_retry_at
              FROM operations o WHERE o.outcome IS NOT NULL
            UNION ALL
            SELECT 'namespace', n.id, n.node, n.kind, n.state,
                   n.outcome, n.outcome_category, n.last_error,
                   n.attempts, n.next_retry_at
              FROM namespace_ops n WHERE n.outcome IS NOT NULL
            UNION ALL
            SELECT 'transfer', t.id, t.node, t.direction, t.state,
                   t.outcome, t.outcome_category, t.last_error,
                   0, t.next_retry_at
              FROM transfers t WHERE t.outcome IS NOT NULL;

        UPDATE metadata SET value='29' WHERE key='schema_version';
        ",
    )?;
    // (h) Legacy backfill. Idempotent: every statement is gated on
    // `outcome IS NULL`, so a re-run (or the fresh-database path, which has no
    // rows) cannot overwrite a diagnosis written by TB2 later.
    //
    // `state='inflight'` rows are still inflight here: Store::open rewrites them
    // to `uncertain` after the migration commits, so they need the diagnosis now
    // rather than being left without one. `state='pending'` rows were never
    // submitted and deliberately keep `outcome=NULL`: there is no commit result
    // to report yet.
    db.execute_batch(
        "
        UPDATE operations SET outcome='unknown_retryable',
               outcome_category='interrupted',
               last_error='interrupted by restart; awaiting reconciliation',
               next_retry_at=unixepoch(), updated_at=unixepoch()
         WHERE outcome IS NULL AND state='uncertain';
        UPDATE operations SET outcome='unknown_retryable',
               outcome_category='interrupted',
               last_error='submitted before restart; outcome unknown',
               next_retry_at=unixepoch(), updated_at=unixepoch()
         WHERE outcome IS NULL AND state='inflight';
        UPDATE operations SET outcome='confirmed', outcome_category='committed',
               updated_at=unixepoch()
         WHERE outcome IS NULL AND state='committed';
        UPDATE operations SET outcome='conflict', outcome_category='server_rejected',
               updated_at=unixepoch()
         WHERE outcome IS NULL AND state='conflict';
        UPDATE operations SET outcome='confirmed', outcome_category='local_abandoned',
               updated_at=unixepoch()
         WHERE outcome IS NULL AND state='superseded';

        UPDATE namespace_ops SET outcome='unknown_retryable',
               outcome_category='interrupted',
               last_error='interrupted by restart; awaiting reconciliation',
               next_retry_at=unixepoch(), updated_at=unixepoch()
         WHERE outcome IS NULL AND state IN ('uncertain','cleanup_uncertain');
        UPDATE namespace_ops SET outcome='unknown_retryable',
               outcome_category='interrupted',
               last_error='submitted before restart; outcome unknown',
               next_retry_at=unixepoch(), updated_at=unixepoch()
         WHERE outcome IS NULL AND state IN ('inflight','cleanup_inflight');
        UPDATE namespace_ops SET outcome='confirmed', outcome_category='committed',
               updated_at=unixepoch()
         WHERE outcome IS NULL AND state='committed';
        UPDATE namespace_ops SET outcome='conflict', outcome_category='server_rejected',
               updated_at=unixepoch()
         WHERE outcome IS NULL AND state='conflict';
        UPDATE namespace_ops SET outcome='confirmed', outcome_category='local_abandoned',
               updated_at=unixepoch()
         WHERE outcome IS NULL AND state='superseded';

        UPDATE transfers SET outcome='unknown_retryable',
               outcome_category=CASE json_extract(error,'$.kind')
                   WHEN 'server_rejected' THEN 'server_rejected'
                   WHEN 'invalid_content' THEN 'protocol'
                   WHEN 'local_storage' THEN 'local_storage'
                   WHEN 'session_expired' THEN 'timeout'
                   ELSE 'transport' END,
               last_error=COALESCE(json_extract(error,'$.kind'),'unknown'),
               next_retry_at=COALESCE(CAST(json_extract(error,'$.retry_at') AS INTEGER),0)
         WHERE outcome IS NULL AND state IN ('retry','failed');
        UPDATE transfers SET outcome='unknown_retryable',
               outcome_category='interrupted',
               last_error='interrupted by restart',
               next_retry_at=unixepoch()
         WHERE outcome IS NULL AND state='inflight';
        ",
    )?;
    Ok(())
}

impl Store {
    /// One store per account + sync root. Independent accounts never share event IDs or cursors.
    pub fn open(root: impl AsRef<Path>) -> Result<Self> {
        let root = root.as_ref().to_owned();
        fs::create_dir_all(root.join("blobs"))?;
        let writer_lock = std::fs::OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(root.join("writer.lock"))?;
        writer_lock.try_lock().map_err(|_| {
            Error::State("state directory already has a writer or locking is unavailable".into())
        })?;
        let writer_lock = WriterLock(writer_lock);
        let db = Connection::open(root.join("state.sqlite"))?;
        db.pragma_update(None, "journal_mode", "WAL")?;
        db.pragma_update(None, "synchronous", "FULL")?;
        db.pragma_update(None, "foreign_keys", "ON")?;
        db.busy_timeout(std::time::Duration::from_secs(5))?;
        let version: i64 = db.pragma_query_value(None, "user_version", |r| r.get(0))?;
        if version > STATE_SCHEMA_VERSION {
            return Err(Error::State("database is newer than this build".into()));
        }
        crate::download_publication::cleanup_staging(&root)?;
        if version > 0 && version < STATE_SCHEMA_VERSION {
            // SQLite writes a coherent snapshot including committed WAL contents.
            let backup = root.join(format!(
                "state-before-v{}-{}.sqlite",
                STATE_SCHEMA_VERSION,
                uuid::Uuid::new_v4()
            ));
            db.execute("VACUUM INTO ?1", [backup.to_string_lossy().as_ref()])?;
        }
        db.execute_batch("
            BEGIN IMMEDIATE;
            CREATE TABLE IF NOT EXISTS metadata (key TEXT PRIMARY KEY, value TEXT NOT NULL);
            INSERT OR IGNORE INTO metadata VALUES ('received_cursor',''),('committed_cursor','');
            CREATE TABLE IF NOT EXISTS inbox (seq INTEGER PRIMARY KEY, body TEXT UNIQUE NOT NULL, done INTEGER NOT NULL DEFAULT 0);
            CREATE TABLE IF NOT EXISTS seen (id TEXT PRIMARY KEY, body TEXT NOT NULL);
            CREATE TABLE IF NOT EXISTS nodes (
                id TEXT PRIMARY KEY, parent TEXT, name TEXT NOT NULL, remote_version TEXT NOT NULL,
                directory INTEGER NOT NULL, local_version TEXT, blob TEXT,
                dirty_blob TEXT, base_version TEXT, dirty_generation INTEGER NOT NULL DEFAULT 0,
                pinned INTEGER NOT NULL DEFAULT 0, deleted INTEGER NOT NULL DEFAULT 0,
                conflict INTEGER NOT NULL DEFAULT 0);
            CREATE INDEX IF NOT EXISTS nodes_parent_name_live ON nodes(parent,name,deleted);
            CREATE INDEX IF NOT EXISTS nodes_conflict ON nodes(id) WHERE conflict=1;
            CREATE INDEX IF NOT EXISTS nodes_pinned ON nodes(id) WHERE pinned=1;
            CREATE INDEX IF NOT EXISTS nodes_cached ON nodes(id,local_version,remote_version) WHERE blob IS NOT NULL;
            CREATE TABLE IF NOT EXISTS operations (
                id TEXT PRIMARY KEY, node TEXT NOT NULL, kind TEXT NOT NULL, base TEXT NOT NULL,
                blob TEXT, generation INTEGER NOT NULL, state TEXT NOT NULL DEFAULT 'pending',
                reconcile_attempts INTEGER NOT NULL DEFAULT 0,
                reconcile_at INTEGER NOT NULL DEFAULT 0);
            CREATE INDEX IF NOT EXISTS operations_node ON operations(node);
            CREATE TABLE IF NOT EXISTS transfers (
                id TEXT PRIMARY KEY, node TEXT NOT NULL, version TEXT NOT NULL,
                direction TEXT NOT NULL, state TEXT NOT NULL, error TEXT,
                UNIQUE(node,version,direction));
            CREATE TABLE IF NOT EXISTS fs_inodes(ino INTEGER PRIMARY KEY AUTOINCREMENT, node TEXT UNIQUE NOT NULL);
            CREATE TABLE IF NOT EXISTS node_metadata(node TEXT PRIMARY KEY,size INTEGER NOT NULL,mtime INTEGER NOT NULL,mode INTEGER NOT NULL);

            CREATE TABLE IF NOT EXISTS access_denied(node TEXT PRIMARY KEY);
            CREATE VIEW IF NOT EXISTS access_blocked AS
              WITH RECURSIVE blocked(id) AS (SELECT node FROM access_denied UNION SELECT n.id FROM nodes n JOIN blocked b ON n.parent=b.id UNION SELECT o.node FROM namespace_overlay o JOIN blocked b ON json_extract(o.remote,'$.parent')=b.id) SELECT id FROM blocked;
            CREATE VIEW IF NOT EXISTS access_blocked_operations AS
              SELECT o.id AS operation FROM namespace_ops o WHERE o.node IN (SELECT id FROM access_blocked)
              OR EXISTS(SELECT 1 FROM json_tree(o.body) j WHERE j.key='id' AND j.value IN (SELECT id FROM access_blocked));
            CREATE TABLE IF NOT EXISTS drafts(node TEXT PRIMARY KEY,token TEXT UNIQUE NOT NULL,base TEXT NOT NULL,generation INTEGER NOT NULL,checkpoint INTEGER NOT NULL,dirty INTEGER NOT NULL);
            CREATE TABLE IF NOT EXISTS remote_aliases(remote TEXT PRIMARY KEY,node TEXT UNIQUE NOT NULL);
            CREATE TABLE IF NOT EXISTS local_creations(node TEXT PRIMARY KEY,ready INTEGER NOT NULL DEFAULT 0);
            CREATE TABLE IF NOT EXISTS namespace_ops(seq INTEGER PRIMARY KEY AUTOINCREMENT,id TEXT UNIQUE NOT NULL,node TEXT NOT NULL,body TEXT NOT NULL,state TEXT NOT NULL,kind TEXT NOT NULL DEFAULT 'rename');
            CREATE INDEX IF NOT EXISTS namespace_ops_node ON namespace_ops(node);
            CREATE TABLE IF NOT EXISTS namespace_overlay(node TEXT PRIMARY KEY,remote TEXT NOT NULL);
            CREATE TABLE IF NOT EXISTS content_version_aliases(node TEXT NOT NULL,old TEXT NOT NULL,new TEXT NOT NULL,PRIMARY KEY(node,old));
            CREATE TABLE IF NOT EXISTS upload_origins(operation TEXT PRIMARY KEY,body TEXT NOT NULL);
            CREATE TABLE IF NOT EXISTS upload_paths(operation TEXT PRIMARY KEY,path TEXT NOT NULL);
            CREATE TABLE IF NOT EXISTS local_unlinks(node TEXT PRIMARY KEY,confirmed INTEGER NOT NULL);
            CREATE TABLE IF NOT EXISTS cache_access(seq INTEGER PRIMARY KEY AUTOINCREMENT,node TEXT UNIQUE NOT NULL);
            CREATE TABLE IF NOT EXISTS cache_published(node TEXT PRIMARY KEY,at INTEGER NOT NULL);
            CREATE TABLE IF NOT EXISTS local_modes(node TEXT PRIMARY KEY,mode INTEGER NOT NULL CHECK(mode>=0 AND mode<=511));
            CREATE TABLE IF NOT EXISTS local_atimes(node TEXT PRIMARY KEY REFERENCES nodes(id) ON DELETE CASCADE ON UPDATE CASCADE,seconds INTEGER NOT NULL CHECK(seconds>=0),nanoseconds INTEGER NOT NULL CHECK(nanoseconds>=0 AND nanoseconds<1000000000));
            CREATE TABLE IF NOT EXISTS upload_times(operation TEXT PRIMARY KEY REFERENCES operations(id) ON DELETE CASCADE,mtime INTEGER NOT NULL CHECK(mtime>=0 AND mtime<=4294967295));
            CREATE INDEX IF NOT EXISTS namespace_unfinished ON namespace_ops(seq) WHERE state NOT IN ('committed','superseded');
            CREATE INDEX IF NOT EXISTS operations_unfinished ON operations(id) WHERE state NOT IN ('committed','superseded');
            CREATE TABLE IF NOT EXISTS cache_releases(node TEXT PRIMARY KEY,version TEXT NOT NULL,state TEXT NOT NULL CHECK(state IN ('pending','retained','committed','reclaimed')),attempts INTEGER NOT NULL DEFAULT 0,retry_at INTEGER NOT NULL DEFAULT 0);
            CREATE TABLE IF NOT EXISTS engine_bridge_intents(key TEXT PRIMARY KEY,node TEXT NOT NULL,body TEXT NOT NULL,baseline TEXT,phase TEXT NOT NULL CHECK(phase IN ('prepared','succeeded','failed','applied')),attempts INTEGER NOT NULL DEFAULT 0,retry_at INTEGER NOT NULL DEFAULT 0,error TEXT);
            CREATE TABLE IF NOT EXISTS engine_bridge_tombstones(node TEXT PRIMARY KEY,cutoff TEXT NOT NULL);
            CREATE TABLE IF NOT EXISTS engine_upload_stages(operation TEXT PRIMARY KEY,source_key TEXT NOT NULL UNIQUE,source TEXT NOT NULL,state TEXT NOT NULL CHECK(state IN ('staged','claimed','receipted','applied','uncertain')),attempt TEXT UNIQUE,receipt TEXT,error TEXT);
        ")?;
        let has_kind = {
            let mut q = db.prepare("PRAGMA table_info(namespace_ops)")?;
            let cols = q
                .query_map([], |r| r.get::<_, String>(1))?
                .collect::<std::result::Result<Vec<_>, _>>()?;
            cols.iter().any(|c| c == "kind")
        };
        if !has_kind {
            db.execute_batch(
                "ALTER TABLE namespace_ops ADD COLUMN kind TEXT NOT NULL DEFAULT 'rename';",
            )?;
        }
        // Schema 25 predates the durable upload reconciliation budget. Add the
        // columns in place; existing operations start with no attempt recorded.
        let operation_columns = {
            let mut q = db.prepare("PRAGMA table_info(operations)")?;
            q.query_map([], |r| r.get::<_, String>(1))?
                .collect::<std::result::Result<Vec<_>, _>>()?
        };
        for (name, definition) in [
            (
                "reconcile_attempts",
                "reconcile_attempts INTEGER NOT NULL DEFAULT 0",
            ),
            ("reconcile_at", "reconcile_at INTEGER NOT NULL DEFAULT 0"),
        ] {
            if !operation_columns
                .iter()
                .any(|column| column.as_str() == name)
            {
                db.execute_batch(&format!(
                    "ALTER TABLE operations ADD COLUMN {definition};"
                ))?;
            }
        }
        // Schema 28 index candidates. They are created after every `ALTER TABLE`
        // above because `operations.reconcile_at` only exists on v25+ databases,
        // and they are idempotent so an interrupted upgrade simply re-creates
        // them on the next open.
        //
        // `operations_countable` is the exact predicate of `pending_operations`
        // and the cache candidate guard; the previous partial index keyed on
        // `id` could satisfy existence tests but not this aggregate.
        db.execute_batch(
            "CREATE INDEX IF NOT EXISTS operations_countable ON operations(id) WHERE state IN ('pending','inflight','uncertain');
             CREATE INDEX IF NOT EXISTS operations_node_active ON operations(node) WHERE state IN ('pending','inflight','uncertain','conflict');
             CREATE INDEX IF NOT EXISTS operations_blob_active ON operations(blob) WHERE blob IS NOT NULL AND state IN ('pending','inflight','uncertain','conflict');
             CREATE INDEX IF NOT EXISTS operations_uncertain_reconcile ON operations(reconcile_at) WHERE kind='upload' AND state='uncertain';
             CREATE INDEX IF NOT EXISTS inbox_pending ON inbox(seq) WHERE done=0;",
        )?;
        crate::initialization_store::migrate(&db)?;
        crate::preview::migrate(&db)?;
        // The `repairing` flag is about the v28 durable sync-history row counter
        // and stays pinned to the literal 28: when the schema moves to 29 a v28
        // store must not be treated as a legacy (pre-counter) store again.
        sync_history::migrate(&db, version < 28)?;
        crate::write_permissions::migrate(&db)?;
        migrate_v29(&db, version)?;
        db.pragma_update(None, "user_version", STATE_SCHEMA_VERSION)?;
        db.execute_batch("COMMIT;")?;
        crate::initialization_store::recover_open(&db)?;
        db.execute(
            "UPDATE namespace_ops SET state='uncertain' WHERE state='inflight'",
            [],
        )?;
        db.execute("UPDATE namespace_ops SET state='cleanup_uncertain' WHERE state='cleanup_inflight' AND kind='replace'",[])?;
        db.execute(
            "UPDATE operations SET state='uncertain' WHERE state='inflight'",
            [],
        )?;
        db.execute(
            "UPDATE transfers SET state='pending' WHERE direction='download' AND state='inflight'",
            [],
        )?;
        // Derived state starts after startup recovery. TEMP objects and a fresh
        // epoch keep the durable schema and external readers unchanged.
        crate::status_changes::install(&db)?;
        crate::removals::install_barrier_versions(&db)?;
        let mut store = Self {
            _writer_lock: Some(writer_lock),
            db,
            root,
            handles: Arc::new(Mutex::new(HashMap::new())),
            namespace_runtime: Arc::new(Mutex::new(Default::default())),
            cache_recency: Mutex::new(crate::cache_policy::CacheRecency::new()),
            draft_stats: DraftWriteStats::default(),
            permission_cache: Mutex::new(crate::write_permissions::PermissionCache::new()),
            barrier: Default::default(),
        };
        // Bound bookkeeping written before schema 28 (superseded operations,
        // old terminal namespace history, applied inbox receipts, duplicate
        // committed generations) once at startup after the upgrade. The durable
        // marker keeps later opens on a large database from re-scanning history
        // before the steady-state paths have anything to remove.
        store.prune_storage_if_due()?;
        Ok(store)
    }
    /// Second handle for read-only work next to a live writer. Unlike
    /// [`Store::open`] this takes no `writer.lock`, creates no files and never
    /// migrates: it only opens the existing database read-only and observes
    /// committed WAL snapshots, so callers can drop it and fall back to the
    /// serialized `SharedStore` boundary at any time. It fails when the state
    /// database is missing, does not match this build's schema or is not in WAL
    /// mode; those cases keep the one-writer contract intact.
    pub fn open_read_only(root: &Path) -> Result<Self> {
        Self::open_read_only_with(root, Arc::new(Mutex::new(Default::default())))
    }
    /// Read-only handle whose scheduler diagnostics come from the live writer's
    /// `NamespaceRuntime` instead of a private empty copy. The SQLite connection
    /// is still a plain committed-WAL snapshot: only observations are shared.
    pub(crate) fn open_read_only_observing(
        root: &Path,
        namespace_runtime: Arc<Mutex<crate::namespace_schedule::NamespaceRuntime>>,
    ) -> Result<Self> {
        Self::open_read_only_with(root, namespace_runtime)
    }
    fn open_read_only_with(
        root: &Path,
        namespace_runtime: Arc<Mutex<crate::namespace_schedule::NamespaceRuntime>>,
    ) -> Result<Self> {
        let root = root.to_owned();
        let database = root.join("state.sqlite");
        if !database.is_file() {
            return Err(Error::State("state database does not exist".into()));
        }
        let db = Connection::open_with_flags(
            &database,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )?;
        // Same wait budget as the writer: WAL readers rarely contend, and a
        // bounded wait keeps one slow checkpoint from surfacing as an I/O error.
        db.busy_timeout(std::time::Duration::from_secs(5))?;
        let version: i64 = db.pragma_query_value(None, "user_version", |r| r.get(0))?;
        if version > STATE_SCHEMA_VERSION {
            return Err(Error::State("database is newer than this build".into()));
        }
        if version != STATE_SCHEMA_VERSION {
            return Err(Error::State(
                "state database is not at the current schema; reopen it with Store::open".into(),
            ));
        }
        let mode: String = db.pragma_query_value(None, "journal_mode", |r| r.get(0))?;
        if !mode.eq_ignore_ascii_case("wal") {
            return Err(Error::State("state database is not in WAL mode".into()));
        }
        // Defence in depth for a connection whose file flags were bypassed:
        // every accidental write is rejected before it can touch the database.
        db.pragma_update(None, "query_only", true)?;
        Ok(Self {
            _writer_lock: None,
            db,
            root,
            handles: Arc::new(Mutex::new(HashMap::new())),
            namespace_runtime,
            cache_recency: Mutex::new(crate::cache_policy::CacheRecency::new()),
            draft_stats: DraftWriteStats::default(),
            permission_cache: Mutex::new(crate::write_permissions::PermissionCache::new()),
            barrier: Default::default(),
        })
    }
    /// Memoized effective decision for the writer connection (#26). Read-only
    /// handles return `None`: they cannot observe later commits, so caching
    /// there could resurrect a revoked decision.
    pub(crate) fn permission_cache_lookup(
        &self,
        root: &str,
        node: &str,
        action: crate::write_permissions::Action,
    ) -> Option<crate::write_permissions::Decision> {
        if self._writer_lock.is_none() {
            return None;
        }
        let epoch = self.db.total_changes();
        self.permission_cache
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .lookup(epoch, root, node, action)
    }
    pub(crate) fn permission_cache_store(
        &self,
        root: &str,
        node: &str,
        action: crate::write_permissions::Action,
        decision: crate::write_permissions::Decision,
    ) {
        if self._writer_lock.is_none() {
            return;
        }
        let epoch = self.db.total_changes();
        self.permission_cache
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .store(epoch, root, node, action, decision);
    }
    /// Cumulative hit/miss counters of the writer-local decision memo.
    pub fn permission_cache_stats(&self) -> crate::write_permissions::PermissionCacheStats {
        let cache = self
            .permission_cache
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        crate::write_permissions::PermissionCacheStats {
            hits: cache.hits,
            misses: cache.misses,
        }
    }
    /// The state directory, including `state.sqlite` and `blobs`.
    pub fn root(&self) -> &Path {
        &self.root
    }
    pub fn cursor(&self) -> Result<String> {
        Ok(self.db.query_row(
            "SELECT value FROM metadata WHERE key='committed_cursor'",
            [],
            |r| r.get(0),
        )?)
    }
    pub fn skipped_symlink_count(&self) -> Result<u64> {
        let count: Option<String> = self
            .db
            .query_row(
                "SELECT value FROM metadata WHERE key='skipped_symlinks'",
                [],
                |r| r.get(0),
            )
            .optional()?;
        Ok(count.and_then(|s| s.parse().ok()).unwrap_or(0))
    }
    pub fn inaccessible_count(&self) -> Result<i64> {
        Ok(self
            .db
            .query_row("SELECT count(*) FROM access_denied", [], |r| r.get(0))?)
    }
    pub fn is_inaccessible(&self, id: &str) -> Result<bool> {
        Ok(self.db.query_row("SELECT CASE WHEN NOT EXISTS(SELECT 1 FROM access_denied) THEN 0 ELSE (
            WITH RECURSIVE ancestors(id) AS (SELECT ?1 UNION SELECT n.parent FROM nodes n JOIN ancestors a ON n.id=a.id WHERE n.parent IS NOT NULL UNION SELECT json_extract(o.remote,'$.parent') FROM namespace_overlay o JOIN ancestors a ON o.node=a.id WHERE json_extract(o.remote,'$.parent') IS NOT NULL)
            SELECT EXISTS(SELECT 1 FROM ancestors a JOIN access_denied d ON d.node=a.id)) END", [id], |r| r.get(0))?)
    }
    pub fn node(&self, id: &str) -> Result<Node> {
        self.read_node(id, false)
    }
    pub(crate) fn node_for_handle(&self, id: &str) -> Result<Node> {
        self.read_node(id, true)
    }
    fn read_node(&self, id: &str, orphan: bool) -> Result<Node> {
        // Walk this node's ancestors, not the entire retained denied subtree,
        // on the hot getattr/lookup path.
        if self.is_inaccessible(id)? {
            return Err(rusqlite::Error::QueryReturnedNoRows.into());
        }
        let mut node = self.db.query_row(
            "SELECT id,parent,name,remote_version,directory FROM nodes WHERE id=?1 AND (deleted=0 OR (?2 AND EXISTS(SELECT 1 FROM local_unlinks WHERE node=?1)))",
            params![id,orphan],
            |r| {
                Ok(Node {
                    id: r.get(0)?,
                    parent: r.get(1)?,
                    name: r.get(2)?,
                    version: r.get(3)?,
                    directory: r.get(4)?,
                    metadata: None,
                })
            },
        )?;
        node.metadata = self.metadata(id)?;
        Ok(node)
    }

    /// A restored server database invalidates old cursors; never silently bind it
    /// to an existing queue of local writes or wipe cached/dirty data.
    pub fn bind_native_server(&mut self, identity: &str) -> Result<()> {
        if identity.is_empty() {
            return Err(Error::State("empty server restore identity".into()));
        }
        let old: Option<String> = self
            .db
            .query_row(
                "SELECT value FROM metadata WHERE key='server_restore_id'",
                [],
                |r| r.get(0),
            )
            .optional()?;
        match old {
            Some(old) if old != identity => Err(Error::State(
                "server restore identity changed; reconciliation required".into(),
            )),
            Some(_) => Ok(()),
            None => {
                self.db.execute(
                    "INSERT INTO metadata VALUES('server_restore_id',?1)",
                    [identity],
                )?;
                Ok(())
            }
        }
    }

    /// Bind both scope and server in one transaction before any events are imported.
    pub fn bind_native_scope(&mut self, server: &str, scope: &str) -> Result<()> {
        if server.is_empty() || scope.is_empty() {
            return Err(Error::State("empty native binding".into()));
        }
        let tx = self.db.transaction()?;
        for (key, value) in [("server_restore_id", server), ("native_scope", scope)] {
            let previous: Option<String> = tx
                .query_row("SELECT value FROM metadata WHERE key=?1", [key], |r| {
                    r.get(0)
                })
                .optional()?;
            if previous.as_deref().is_some_and(|old| old != value) {
                return Err(Error::State(
                    "native server or sync scope changed; reconciliation required".into(),
                ));
            }
            tx.execute(
                "INSERT OR IGNORE INTO metadata VALUES(?1,?2)",
                params![key, value],
            )?;
        }
        tx.commit()?;
        Ok(())
    }

    /// Immutable, namespaced local integration marker. Never a server cursor.
    pub fn extension_marker(&self, name: &str) -> Result<Option<String>> {
        let key = format!("extension:{name}");
        Ok(self
            .db
            .query_row("SELECT value FROM metadata WHERE key=?1", [key], |r| {
                r.get(0)
            })
            .optional()?)
    }
    pub fn claim_extension_marker(&mut self, name: &str, value: &str) -> Result<()> {
        if name.is_empty() || name.len() > 128 || value.len() > 4096 {
            return Err(Error::State("invalid local extension marker".into()));
        }
        let key = format!("extension:{name}");
        let tx = self.db.transaction()?;
        tx.execute(
            "INSERT OR IGNORE INTO metadata(key,value) VALUES(?1,?2)",
            params![key, value],
        )?;
        let existing: String =
            tx.query_row("SELECT value FROM metadata WHERE key=?1", [key], |r| {
                r.get(0)
            })?;
        if existing != value {
            return Err(Error::State(
                "local extension marker already claimed".into(),
            ));
        }
        tx.commit()?;
        Ok(())
    }

    /// Store the full batch before processing it. Receipt and consumption have distinct cursors.
    pub fn receive(&mut self, batch: &Batch) -> Result<()> {
        if self.initialization_writeback_blocked()? {
            return Err(Error::State(
                "initialization: candidate active; ordinary inbox blocked".into(),
            ));
        }
        if batch.to == batch.from {
            return Err(Error::State(
                "empty cursor advancement; handle long-poll timeout outside inbox".into(),
            ));
        }
        let body = serde_json::to_string(batch)?;
        let tx = self.db.transaction()?;
        let duplicate: bool = tx.query_row(
            "SELECT EXISTS(SELECT 1 FROM inbox WHERE body=?1)",
            [&body],
            |r| r.get(0),
        )?;
        if duplicate {
            return Ok(());
        }
        let received: String = tx.query_row(
            "SELECT value FROM metadata WHERE key='received_cursor'",
            [],
            |r| r.get(0),
        )?;
        if batch.from != received {
            // Applied receipts are deleted from the inbox (see `apply_next`), so
            // the duplicate check above cannot see a batch that has already been
            // committed. A resend whose target cursor equals the committed cursor
            // is the last applied batch being replayed: ignore it so redelivery
            // stays idempotent even after the receipt is gone. Anything else is a
            // genuine gap.
            let committed: String = tx.query_row(
                "SELECT value FROM metadata WHERE key='committed_cursor'",
                [],
                |r| r.get(0),
            )?;
            if batch.to == committed {
                return Ok(());
            }
            return Err(Error::State(
                "event cursor gap; reconciliation required".into(),
            ));
        }
        tx.execute("INSERT INTO inbox(body) VALUES(?1)", [body])?;
        tx.execute(
            "UPDATE metadata SET value=?1 WHERE key='received_cursor'",
            [&batch.to],
        )?;
        tx.commit()?;
        Ok(())
    }

    pub fn apply_next(&mut self) -> Result<bool> {
        if self.initialization_writeback_blocked()? {
            return Err(Error::State(
                "initialization: candidate active; ordinary inbox blocked".into(),
            ));
        }
        let next: Option<(i64, String)> = self
            .db
            .query_row(
                "SELECT seq,body FROM inbox WHERE done=0 ORDER BY seq LIMIT 1",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()?;
        let Some((seq, body)) = next else {
            return Ok(false);
        };
        let batch: Batch = serde_json::from_str(&body)?;
        let tx = self.db.transaction()?;
        let cursor: String = tx.query_row(
            "SELECT value FROM metadata WHERE key='committed_cursor'",
            [],
            |r| r.get(0),
        )?;
        if cursor != batch.from {
            return Err(Error::State("inbox cursor mismatch".into()));
        }
        for event in &batch.events {
            let payload = serde_json::to_string(&event.change)?;
            let prior: Option<String> = tx
                .query_row("SELECT body FROM seen WHERE id=?1", [&event.id], |r| {
                    r.get(0)
                })
                .optional()?;
            if let Some(prior) = prior {
                if prior != payload {
                    return Err(Error::State(
                        "event identity reused with different content".into(),
                    ));
                }
                continue;
            }
            let change = crate::creations::map_change(&tx, &event.change)?;
            if let Some(change) = crate::namespace::overlay_change(&tx, &change)? {
                match &change {
                    Change::Upsert(n) => {
                        valid_node(n)?;
                        // Do not let a move create a cycle in the persistent directory tree.
                        let cycle:bool=tx.query_row("WITH RECURSIVE parents(id) AS (SELECT ?1 UNION SELECT n.parent FROM nodes n JOIN parents p ON n.id=p.id WHERE n.parent IS NOT NULL) SELECT EXISTS(SELECT 1 FROM parents WHERE id=?2)",params![n.parent,n.id],|r|r.get(0))?;
                        if cycle {
                            return Err(Error::State("directory cycle".into()));
                        }
                        tx.execute("INSERT INTO nodes(id,parent,name,remote_version,directory) VALUES (?1,?2,?3,?4,?5)
                        ON CONFLICT(id) DO UPDATE SET parent=excluded.parent,name=excluded.name,remote_version=excluded.remote_version,directory=excluded.directory,deleted=CASE WHEN EXISTS(SELECT 1 FROM local_unlinks WHERE node=nodes.id) THEN 1 ELSE 0 END,
                        conflict=CASE WHEN nodes.dirty_blob IS NOT NULL AND nodes.base_version!=excluded.remote_version AND NOT EXISTS(SELECT 1 FROM local_unlinks WHERE node=nodes.id) THEN 1 ELSE nodes.conflict END",
                        params![n.id,n.parent,n.name,n.version,n.directory])?;
                        crate::filesystem::write_metadata(&tx, n)?;
                        // The next mount must backfill any node this event
                        // created; the O(1) hint is only a hint.
                        crate::filesystem::mark_inode_map_incomplete(&tx)?;
                    }
                    Change::Delete { id } => {
                        tx.execute("WITH RECURSIVE subtree(id) AS (SELECT ?1 UNION SELECT n.id FROM nodes n JOIN subtree s ON n.parent=s.id)
                        UPDATE nodes SET deleted=CASE WHEN id IN (SELECT node FROM local_unlinks) THEN 1 WHEN id IN (SELECT node FROM namespace_overlay) THEN 0 ELSE 1 END,conflict=CASE WHEN id IN (SELECT node FROM local_unlinks) THEN conflict WHEN dirty_blob IS NOT NULL OR id IN (SELECT node FROM namespace_overlay) THEN 1 ELSE conflict END WHERE id IN subtree",[id])?;
                    }
                }
            }
            tx.execute(
                "INSERT INTO seen VALUES (?1,?2)",
                params![event.id, payload],
            )?;
            sync_history::record_remote_event(&tx, event)?;
        }
        // An applied receipt is never read again: deleting it in the same
        // transaction keeps the inbox bounded (the `inbox_pending` index only
        // ever walks unapplied rows). Resent bodies cannot reappear here because
        // the cursor contract rejects a regressed batch before insertion.
        tx.execute("DELETE FROM inbox WHERE seq=?1", [seq])?;
        tx.execute(
            "UPDATE metadata SET value=?1 WHERE key='committed_cursor'",
            [batch.to],
        )?;
        tx.commit()?;
        Ok(true)
    }
    pub fn drain(&mut self) -> Result<()> {
        while self.apply_next()? {}
        Ok(())
    }

    pub fn is_pinned(&self, id: &str) -> Result<bool> {
        Ok(self.db.query_row(
            "WITH RECURSIVE ancestors(id,parent,pinned) AS (
            SELECT id,parent,pinned FROM nodes WHERE id=?1 UNION
            SELECT n.id,n.parent,n.pinned FROM nodes n JOIN ancestors a ON a.parent=n.id)
            SELECT EXISTS(SELECT 1 FROM ancestors WHERE pinned=1)",
            [id],
            |r| r.get(0),
        )?)
    }
    pub fn pin(&mut self, id: &str, value: bool) -> Result<()> {
        if self.is_inaccessible(id)? {
            return Err(Error::Io(std::io::Error::from_raw_os_error(libc::EACCES)));
        }
        if self.db.execute(
            "UPDATE nodes SET pinned=?2 WHERE id=?1 AND deleted=0",
            params![id, value],
        )? != 1
        {
            return Err(Error::State("pin target missing".into()));
        }
        Ok(())
    }
    pub fn downloads_needed(&self) -> Result<Vec<DownloadTicket>> {
        let mut q=self.db.prepare(DOWNLOAD_CANDIDATES)?;
        let candidates = q
            .query_map([], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, bool>(2)?,
                ))
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        let mut result = Vec::new();
        for (id, version, cached) in candidates {
            if self.cache_release_pending(&DownloadTicket {
                id: id.clone(),
                version: version.clone(),
            })? {
                continue;
            }
            if self.dirty_draft(&id)? || crate::namespace::blocks_upload(&self.db, &id)? {
                continue;
            }
            if cached
                || self.is_pinned(&id)?
                || self.has_download_intent(&DownloadTicket {
                    id: id.clone(),
                    version: version.clone(),
                })?
            {
                result.push(DownloadTicket { id, version });
            }
        }
        Ok(result)
    }
    /// Compatibility helper for small callers; production transfers use the bounded reader API.
    pub fn publish_download(
        &mut self,
        ticket: &DownloadTicket,
        bytes: &[u8],
        expected_sha256: &str,
    ) -> Result<()> {
        self.publish_download_reader(
            ticket,
            &mut std::io::Cursor::new(bytes),
            bytes.len() as u64,
            expected_sha256,
        )
    }
    /// Imports a verified content descriptor without allocating the whole file.
    /// The file must still match the ticket at commit time; old bytes survive errors.
    pub fn publish_download_reader<R: Read>(
        &mut self,
        ticket: &DownloadTicket,
        source: &mut R,
        size: u64,
        expected_sha256: &str,
    ) -> Result<()> {
        let body = self
            .prepare_download(ticket)?
            .prepare(source, size, expected_sha256, || false)?;
        self.publish_prepared_download(body)
    }
    pub(crate) fn write_blob(&self, bytes: &[u8]) -> Result<String> {
        self.write_blob_reader(&mut std::io::Cursor::new(bytes), None)
    }
    pub(crate) fn write_blob_reader<R: Read>(
        &self,
        source: &mut R,
        expected: Option<(u64, &str)>,
    ) -> Result<String> {
        let parent = self.root.join("blobs");
        let mut tmp = tempfile::NamedTempFile::new_in(&parent)?;
        let mut buffer = [0u8; 65536];
        let mut hash = Sha256::new();
        let mut count = 0u64;
        loop {
            let n = source.read(&mut buffer)?;
            if n == 0 {
                break;
            }
            count = count.checked_add(n as u64).ok_or(Error::Integrity)?;
            if expected.is_some_and(|(size, _)| count > size) {
                return Err(Error::Integrity);
            }
            tmp.write_all(&buffer[..n])?;
            hash.update(&buffer[..n]);
        }
        if let Some((size, digest)) = expected {
            if count != size || format!("{:x}", hash.finalize()) != digest {
                return Err(Error::Integrity);
            }
        }
        tmp.as_file().sync_all()?;
        let name = uuid::Uuid::new_v4().to_string();
        tmp.persist(parent.join(&name)).map_err(|e| e.error)?;
        #[cfg(unix)]
        File::open(parent)?.sync_all()?;
        // A crash before the following DB update may leave an unreferenced blob, never a half file.
        Ok(name)
    }
    pub fn open_cached(&self, id: &str) -> Result<ReadHandle> {
        if self.is_inaccessible(id)? {
            return Err(Error::Io(std::io::Error::from_raw_os_error(libc::EACCES)));
        }
        let row:Option<(Option<String>,bool,bool,Option<String>)>=self.db.query_row("SELECT COALESCE(dirty_blob,blob),deleted,conflict,COALESCE(base_version,local_version) FROM nodes WHERE id=?1",[id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?))).optional()?;
        let Some((Some(blob), deleted, conflict, Some(version))) = row else {
            return Err(Error::Offline);
        };
        if deleted && !conflict {
            return Err(Error::Offline);
        }
        let file = File::open(self.root.join("blobs").join(&blob))?;
        self.handles
            .lock()
            .unwrap()
            .entry(blob.clone())
            .or_insert_with(|| (id.to_owned(), 0))
            .1 += 1;
        Ok(ReadHandle {
            file,
            blob,
            version,
            handles: self.handles.clone(),
        })
    }
    pub fn status(&self, id: &str) -> Result<Status> {
        let (blob,local,remote,dirty,conflict,deleted):(Option<String>,Option<String>,String,bool,bool,bool)=self.db.query_row("SELECT blob,local_version,remote_version,dirty_blob IS NOT NULL,conflict,deleted FROM nodes WHERE id=?1",[id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?)))?;
        if conflict || self.draft_conflicts(id)? {
            return Ok(Status::Conflict);
        }
        if deleted {
            return Ok(Status::Deleted);
        }
        if dirty || self.dirty_draft(id)? || crate::namespace::blocks_upload(&self.db, id)? {
            return Ok(Status::PendingUpload);
        }
        let pinned = self.is_pinned(id)?;
        if let Some(blob) = blob {
            if !self.root.join("blobs").join(blob).is_file() {
                return Err(Error::State("referenced content missing on disk".into()));
            }
        } else {
            return Ok(if pinned {
                Status::Pinning
            } else {
                Status::OnlineOnly
            });
        }
        if local.as_deref() != Some(remote.as_str()) {
            return Ok(Status::Updating);
        }
        Ok(if pinned {
            Status::PinnedComplete
        } else {
            Status::LocalComplete
        })
    }
    /// Local-only cache policy. This never inserts a remote delete operation.
    /// Physical reclamation is separate and protects snapshots held by open handles.
    pub fn release(&mut self, id: &str) -> Result<ReleaseOutcome> {
        if self.cache_is_protected(id)? {
            return Err(Error::State(
                "content has a protected local reference".into(),
            ));
        }
        let cached: Option<(bool, bool, bool)> = self
            .db
            .query_row(
                "SELECT blob IS NOT NULL, dirty_blob IS NOT NULL, conflict=1 FROM nodes WHERE id=?1",
                [id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .optional()?;
        let Some((has_blob, dirty, conflict)) = cached else {
            return Err(Error::State("content is dirty, conflicted, or absent".into()));
        };
        if dirty || conflict {
            return Err(Error::State("content is dirty, conflicted, or absent".into()));
        }
        if !has_blob {
            // There is no local body left to reclaim, so report the no-op as
            // skipped instead of the fake success behind issue #19. Releasing an
            // online-only path still withdraws a pending or failed local read
            // intent, matching the body path below and keeping the queue clean.
            self.db.execute(
                "DELETE FROM transfers WHERE node=?1 AND direction='download'",
                [id],
            )?;
            return Ok(ReleaseOutcome::Skipped);
        }
        let tx = self.db.transaction()?;
        if tx.execute("UPDATE nodes SET blob=NULL,local_version=NULL WHERE id=?1 AND blob IS NOT NULL AND dirty_blob IS NULL AND conflict=0",[id])?!=1 {return Err(Error::State("content is dirty, conflicted, or absent".into()));}
        tx.execute(
            "DELETE FROM transfers WHERE node=?1 AND direction='download'",
            [id],
        )?;
        tx.commit()?;
        Ok(ReleaseOutcome::Released)
    }

    /// Reclaim only unreferenced generated blobs. Journaled writes, conflicts and
    /// open old versions remain protected. No remote operation is created.
    pub fn collect_garbage(&mut self) -> Result<u64> {
        // Retired rows may still point at the body they used to cache. A shared
        // pointer means the live row keeps the file below, so dropping the stale
        // pointer is safe; conflicted recovery rows keep theirs readable.
        self.db.execute(
            "UPDATE nodes SET blob=NULL,local_version=NULL WHERE deleted=1 AND conflict=0 AND dirty_blob IS NULL AND blob IS NOT NULL",
            [],
        )?;
        // `abandoned` is #14's terminal upload state, but it is only terminal for
        // automatic reconcile: an explicit retry re-arms it as `uncertain`, so its
        // body still has to survive garbage collection.
        let mut q=self.db.prepare("SELECT blob FROM nodes WHERE blob IS NOT NULL AND (deleted=0 OR conflict=1) UNION SELECT dirty_blob FROM nodes WHERE dirty_blob IS NOT NULL UNION SELECT blob FROM operations WHERE blob IS NOT NULL AND state IN ('pending','inflight','uncertain','conflict','abandoned') UNION SELECT json_extract(body,'$.blob') FROM namespace_ops WHERE kind='replace' AND state NOT IN ('committed','superseded')")?;
        let mut keep = q
            .query_map([], |r| r.get::<_, String>(0))?
            .collect::<std::result::Result<std::collections::HashSet<_>, _>>()?;
        keep.extend(self.handles.lock().unwrap().keys().cloned());
        let parent = self.root.join("blobs");
        let mut removed = 0;
        for entry in fs::read_dir(&parent)? {
            let entry = entry?;
            let Some(name) = entry.file_name().to_str().map(str::to_owned) else {
                continue;
            };
            if keep.contains(&name)
                || uuid::Uuid::parse_str(&name).is_err()
                || !entry.file_type()?.is_file()
            {
                continue;
            }
            let size = entry.metadata()?.len();
            fs::remove_file(entry.path())?;
            removed += size;
        }
        #[cfg(unix)]
        File::open(parent)?.sync_all()?;
        // Closed clean drafts duplicate a confirmed immutable version. Dirty or
        // open drafts retain their own files, including unlinked old handles.
        let open_drafts: std::collections::HashSet<String> = self
            .handles
            .lock()
            .unwrap()
            .keys()
            .filter_map(|k| k.strip_prefix("draft:").map(str::to_owned))
            .collect();
        let rows = {
            let mut q = self.db.prepare("SELECT token,dirty FROM drafts")?;
            q.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, bool>(1)?)))?
                .collect::<std::result::Result<Vec<_>, _>>()?
        };
        let mut retained = open_drafts.clone();
        for (token, dirty) in rows {
            if dirty || open_drafts.contains(&token) {
                retained.insert(token);
            } else {
                self.db
                    .execute("DELETE FROM drafts WHERE token=?1 AND dirty=0", [&token])?;
            }
        }
        let draft_dir = self.root.join("drafts");
        match fs::read_dir(&draft_dir) {
            Ok(entries) => {
                for entry in entries {
                    let entry = entry?;
                    let Some(name) = entry.file_name().to_str().map(str::to_owned) else {
                        continue;
                    };
                    if retained.contains(&name)
                        || uuid::Uuid::parse_str(&name).is_err()
                        || !entry.file_type()?.is_file()
                    {
                        continue;
                    }
                    let size = entry.metadata()?.len();
                    fs::remove_file(entry.path())?;
                    removed += size;
                }
                #[cfg(unix)]
                File::open(draft_dir)?.sync_all()?;
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(e.into()),
        }
        Ok(removed)
    }

    /// Persist scheduler intent before starting network I/O. Duplicate tickets coalesce.
    pub fn queue_download(&mut self, ticket: &DownloadTicket) -> Result<String> {
        let id = uuid::Uuid::new_v4().to_string();
        self.db.execute("INSERT OR IGNORE INTO transfers(id,node,version,direction,state) VALUES(?1,?2,?3,'download','pending')",params![id,ticket.id,ticket.version])?;
        Ok(self.db.query_row(
            "SELECT id FROM transfers WHERE node=?1 AND version=?2 AND direction='download'",
            params![ticket.id, ticket.version],
            |r| r.get(0),
        )?)
    }
    /// Models a complete application save. FUSE random writes / rename replacement are not implemented here.
    pub fn save_local(&mut self, id: &str, bytes: &[u8]) -> Result<String> {
        let base: String = self.db.query_row(
            "SELECT COALESCE(base_version,local_version,remote_version) FROM nodes WHERE id=?1",
            [id],
            |r| r.get(0),
        )?;
        self.save_local_from_version(id, &base, bytes)
    }
    /// Filesystem callers must pass the version captured by the write handle.
    pub fn save_local_from_version(
        &mut self,
        id: &str,
        opened_version: &str,
        bytes: &[u8],
    ) -> Result<String> {
        let blob = self.write_blob(bytes)?;
        self.commit_local_blob(id, opened_version, blob, None)
    }
    /// FUSE uses an immutable/locked draft reader; no whole-file allocation.
    pub fn save_local_reader_from_version<R: Read>(
        &mut self,
        id: &str,
        opened_version: &str,
        source: &mut R,
    ) -> Result<String> {
        let blob = self.write_blob_reader(source, None)?;
        self.commit_local_blob(id, opened_version, blob, None)
    }
    /// Preserve the mutable draft timestamp on its immutable queued snapshot.
    /// The byte copy creates a new inode; copying only bytes loses utimens.
    ///
    /// `draft_checkpoint` folds the draft's `checkpoint=generation` advance into
    /// the snapshot transaction: one commit instead of two, and a crash between
    /// the two writes can no longer leave a snapshot whose draft row disagrees.
    pub(crate) fn save_draft_file_from_version(
        &mut self,
        id: &str,
        base: &str,
        source: &mut File,
        draft_checkpoint: Option<(&str, i64)>,
    ) -> Result<String> {
        let modified = source.metadata()?.modified()?;
        let blob = self.write_blob_reader(source, None)?;
        let snapshot = File::open(self.root.join("blobs").join(&blob))?;
        snapshot.set_modified(modified)?;
        snapshot.sync_all()?;
        self.commit_local_blob(id, base, blob, draft_checkpoint)
    }
    fn commit_local_blob(
        &mut self,
        id: &str,
        opened_version: &str,
        blob: String,
        draft_checkpoint: Option<(&str, i64)>,
    ) -> Result<String> {
        let mtime = crate::timestamps::native_mtime(
            std::fs::metadata(self.root.join("blobs").join(&blob))?.modified()?,
        )?;
        let normalized = crate::namespace::normalized_version(&self.db, id, opened_version)?;
        let opened_version = normalized.as_str();
        let tx = self.db.transaction()?;
        let (_remote,base,generation,deleted,directory):(String,Option<String>,i64,bool,bool)=tx.query_row("SELECT remote_version,base_version,dirty_generation,deleted,directory FROM nodes WHERE id=?1",[id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?)))?;
        if deleted || directory {
            return Err(Error::State("cannot save deleted file or directory".into()));
        }
        let base = base.unwrap_or_else(|| opened_version.to_string());
        let op = uuid::Uuid::new_v4().to_string();
        tx.execute("UPDATE nodes SET dirty_blob=?2,base_version=?3,dirty_generation=?4,conflict=CASE WHEN ?3!=remote_version OR ?5!=?3 THEN 1 ELSE conflict END WHERE id=?1",params![id,blob,base,generation+1,opened_version])?;
        tx.execute(
            "UPDATE operations SET state='superseded' WHERE node=?1 AND state IN ('pending','abandoned')",
            [id],
        )?;
        tx.execute("INSERT INTO operations(id,node,kind,base,blob,generation) VALUES(?1,?2,'upload',?3,?4,?5)",params![op,id,base,blob,generation+1])?;
        tx.execute(
            "INSERT INTO upload_times(operation,mtime) VALUES(?1,?2)",
            params![op, mtime],
        )?;
        if let Some((token, generation)) = draft_checkpoint {
            let changed = tx.execute(
                "UPDATE drafts SET checkpoint=?3 WHERE node=?1 AND token=?2",
                params![id, token, generation],
            )?;
            if changed != 1 {
                return Err(Error::State(
                    "draft checkpoint row vanished during snapshot commit".into(),
                ));
            }
        }
        tx.commit()?;
        Ok(op)
    }
    pub fn mark_upload_started(&mut self, op: &str) -> Result<()> {
        let node: String =
            self.db
                .query_row("SELECT node FROM operations WHERE id=?1", [op], |r| {
                    r.get(0)
                })?;
        if crate::namespace::blocks_upload(&self.db, &node)? {
            return Err(Error::State("namespace operation precedes upload".into()));
        }
        // A possibly committed operation must be reconciled before another upload of the same file.
        let path = crate::namespace::node_path(&self.db, &node)?;
        // Only a remove that owns this target (same identity, deleted ancestor,
        // or overlapping path) may precede the upload. A global remove barrier
        // let unrelated pending removes, such as freedesktop trash entries,
        // block every ordinary upload.
        if let Some(blocked) = self.upload_remove_barrier(&node, &path)? {
            return Err(Error::State(format!(
                "pending remove of {} blocks upload of {}",
                blocked.path, path
            )));
        }
        let tx = self.db.transaction()?;
        let changed=tx.execute("UPDATE operations SET state='inflight' WHERE id=?1 AND state='pending' AND node NOT IN (SELECT id FROM access_blocked) AND NOT EXISTS(SELECT 1 FROM operations other WHERE other.node=operations.node AND other.state IN ('inflight','uncertain','abandoned')) AND EXISTS(SELECT 1 FROM nodes n WHERE n.id=operations.node AND n.remote_version=operations.base AND n.deleted=0 AND n.conflict=0) AND NOT EXISTS(SELECT 1 FROM drafts d WHERE d.node=operations.node AND d.generation!=d.checkpoint) AND NOT EXISTS(SELECT 1 FROM local_creations c WHERE c.node=operations.node AND c.ready=0)",[op])?;
        if changed != 1 {
            return Err(Error::State(
                "upload needs conflict resolution or commit reconciliation".into(),
            ));
        }
        tx.execute(
            "INSERT INTO upload_paths(operation,path) VALUES(?1,?2)",
            params![op, path],
        )?;
        tx.commit()?;
        Ok(())
    }
    pub fn mark_upload_uncertain(&mut self, op: &str) -> Result<()> {
        if self.db.execute(
            "UPDATE operations SET state='uncertain' WHERE id=?1 AND state='inflight'",
            [op],
        )? != 1
        {
            return Err(Error::State("operation not inflight".into()));
        }
        Ok(())
    }
    /// Call only with a verified native commit/reconciliation result, never an HTTP/TCP success alone.
    pub fn confirm_upload(&mut self, op: &str, confirmed_version: &str) -> Result<()> {
        self.confirm_upload_identity(op, confirmed_version, None)
    }
    pub fn confirm_upload_identity(
        &mut self,
        op: &str,
        confirmed_version: &str,
        remote_identity: Option<&str>,
    ) -> Result<()> {
        if confirmed_version.is_empty() {
            return Err(Error::State("empty confirmed version".into()));
        }
        let tx = self.db.transaction()?;
        let (id, base, blob, generation, state): (String, String, String, i64, String) = tx
            .query_row(
                "SELECT node,base,blob,generation,state FROM operations WHERE id=?1",
                [op],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
            )?;
        if state == "committed" {
            return Ok(());
        }
        if state != "inflight" && state != "uncertain" {
            return Err(Error::State("operation has not been submitted".into()));
        }
        let (remote, deleted): (String, bool) = tx.query_row(
            "SELECT remote_version,deleted FROM nodes WHERE id=?1",
            [&id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )?;
        if (deleted && !crate::removals::is_unlinked(&tx, &id)?)
            || (remote != base && remote != confirmed_version)
        {
            tx.execute("UPDATE nodes SET conflict=1 WHERE id=?1", [&id])?;
            tx.execute("UPDATE operations SET state='conflict' WHERE id=?1", [op])?;
            tx.commit()?;
            return Err(Error::State(
                "remote changed before commit reconciliation; preserve local content".into(),
            ));
        }
        if let Some(remote) = remote_identity {
            if let Err(error) =
                crate::creations::bind_created_identity(&tx, &id, remote, confirmed_version)
            {
                tx.execute("UPDATE nodes SET conflict=1 WHERE id=?1", [&id])?;
                tx.execute("UPDATE operations SET state='conflict' WHERE id=?1", [op])?;
                tx.commit()?;
                return Err(error);
            }
        }
        // An empty local creation can be uploaded while an application still
        // holds its original immutable empty handle. A receipt for those exact
        // same bytes changes identity/version, not the content behind that handle.
        // Never alias a genuinely older body to a different uploaded snapshot.
        let unchanged: bool = tx.query_row(
            "SELECT local_version=?2 AND blob=?3 FROM nodes WHERE id=?1",
            params![id, base, blob],
            |r| r.get::<_, Option<bool>>(0).map(|v| v.unwrap_or(false)),
        )?;
        if unchanged && base != confirmed_version {
            tx.execute(
                "INSERT OR REPLACE INTO content_version_aliases(node,old,new) VALUES(?1,?2,?3)",
                params![id, base, confirmed_version],
            )?;
        }
        tx.execute(
            "UPDATE nodes SET remote_version=?2,local_version=?2,blob=?3,conflict=0,
            dirty_blob=CASE WHEN dirty_generation=?4 THEN NULL ELSE dirty_blob END,
            base_version=CASE WHEN dirty_generation=?4 THEN NULL ELSE ?2 END WHERE id=?1",
            params![id, confirmed_version, blob, generation],
        )?;
        tx.execute(
            "UPDATE operations SET base=?2 WHERE node=?1 AND state='pending' AND generation>?3",
            params![id, confirmed_version, generation],
        )?;
        tx.execute("UPDATE drafts SET base=?2,dirty=CASE WHEN EXISTS(SELECT 1 FROM local_unlinks WHERE node=?1) THEN dirty WHEN generation=checkpoint AND ?3=(SELECT dirty_generation FROM nodes WHERE id=?1) THEN 0 ELSE dirty END WHERE node=?1",params![id,confirmed_version,generation])?;
        crate::namespace::rebase_jobs(&tx, &id, &base, confirmed_version)?;
        let overlay: Option<String> = tx
            .query_row(
                "SELECT remote FROM namespace_overlay WHERE node=?1",
                [&id],
                |r| r.get(0),
            )
            .optional()?;
        if let Some(body) = overlay {
            if let Some(mut remote) = serde_json::from_str::<Option<Node>>(&body)? {
                if remote.version == base {
                    remote.version = confirmed_version.into();
                    tx.execute(
                        "UPDATE namespace_overlay SET remote=?2 WHERE node=?1",
                        params![id, serde_json::to_string(&Some(remote))?],
                    )?;
                }
            }
        }
        tx.execute("UPDATE operations SET state='committed' WHERE id=?1", [op])?;
        // Only the newest committed generation per node is observable (the
        // `unsubmitted` predicate is an existence check), so the previous
        // generation is retired in the same transaction instead of being kept
        // forever. The delete is bookkeeping: the journal does not publish it.
        crate::retention::drop_committed_generations(&tx, &id, op)?;
        tx.execute("UPDATE node_metadata SET mtime=(SELECT mtime FROM upload_times WHERE operation=?1) WHERE node=?2 AND EXISTS(SELECT 1 FROM upload_times WHERE operation=?1)",params![op,id])?;
        tx.commit()?;
        Ok(())
    }
    /// Install a fully validated authoritative snapshot after cursor loss. The
    /// caller must finish enumeration/catch-up first. Cache, pins, open handles,
    /// and every local operation survive; only derived event inbox state resets.
    pub fn install_remote_snapshot(
        &mut self,
        nodes: &[Node],
        cursor: &str,
        server: &str,
    ) -> Result<()> {
        self.install_remote_snapshot_with_access(nodes, cursor, server, &[])
    }
    /// Replace the access-denied set without installing a snapshot.
    ///
    /// This is the incremental counterpart of the `inaccessible` argument of
    /// [`Store::install_remote_snapshot_with_access`]: the difference against the
    /// rows already stored is applied in one transaction, so the status journal
    /// records one `access_denied` subtree change per edge instead of resetting
    /// the journal for a whole snapshot.
    ///
    /// Test/diagnostic support only: production code changes access through the
    /// sync snapshot path, and other crates must not treat this as a stable
    /// contract.
    pub fn set_access_denied(&mut self, denied: &[String]) -> Result<()> {
        use std::collections::HashSet;
        let mut wanted = HashSet::new();
        for id in denied {
            if id.is_empty() {
                return Err(Error::State("invalid access denied identity".into()));
            }
            if !wanted.insert(id.as_str()) {
                return Err(Error::State("duplicate access denied identity".into()));
            }
        }
        let tx = self.db.transaction()?;
        let stored = tx
            .prepare("SELECT node FROM access_denied ORDER BY node")?
            .query_map([], |row| row.get::<_, String>(0))?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        let stored_set = stored.iter().map(String::as_str).collect::<HashSet<_>>();
        for id in &stored {
            if !wanted.contains(id.as_str()) {
                tx.execute("DELETE FROM access_denied WHERE node=?1", [id])?;
            }
        }
        let mut insert = tx.prepare("INSERT OR IGNORE INTO access_denied(node) VALUES(?1)")?;
        for id in denied {
            if !stored_set.contains(id.as_str()) {
                insert.execute([id])?;
            }
        }
        drop(insert);
        tx.commit()?;
        Ok(())
    }
    /// Install a recovery snapshot together with the set of subtrees whose access
    /// was denied.
    ///
    /// The snapshot itself replaces the status journal (see
    /// [`crate::status_changes::replace_snapshot`]), so consumers rebuild from it;
    /// the `inaccessible` set is stored in the same transaction. `inaccessible`
    /// must name directory nodes that are present in `nodes`, have a parent, and
    /// are unique.
    pub(crate) fn install_remote_snapshot_with_access(
        &mut self,
        nodes: &[Node],
        cursor: &str,
        server: &str,
        inaccessible: &[String],
    ) -> Result<()> {
        self.install_remote_snapshot_with_initialization(nodes, cursor, server, inaccessible, None)
    }
    pub(crate) fn install_remote_snapshot_with_initialization(
        &mut self,
        nodes: &[Node],
        cursor: &str,
        server: &str,
        inaccessible: &[String],
        initialization: Option<&crate::initialization_store::InitializationCommit>,
    ) -> Result<()> {
        crate::initialization_store::check_install_gate(&self.db, initialization)?;
        use std::collections::{HashMap, HashSet};
        let inaccessible = inaccessible
            .iter()
            .map(|id| self.local_identity(id))
            .collect::<Result<Vec<_>>>()?;
        let mapped = self.mapped_snapshot(nodes)?;
        let nodes = &mapped;
        if nodes.is_empty() || nodes.len() > 1_000_000 || cursor.is_empty() || server.is_empty() {
            return Err(Error::State("invalid recovery snapshot envelope".into()));
        }
        let mut by_id = HashMap::new();
        let mut names = HashSet::new();
        let mut roots = Vec::new();
        for n in nodes {
            valid_node(n)?;
            if by_id.insert(n.id.as_str(), n).is_some()
                || !names.insert((n.parent.as_deref(), n.name.as_str()))
            {
                return Err(Error::State("ambiguous snapshot identity/path".into()));
            }
            if n.parent.is_none() {
                roots.push(n);
            }
        }
        if roots.len() != 1 || !roots[0].directory {
            return Err(Error::State("snapshot needs one directory root".into()));
        }
        let root_id = roots[0].id.clone();
        let mut denied_unique = HashSet::new();
        for id in &inaccessible {
            if !denied_unique.insert(id)
                || !by_id
                    .get(id.as_str())
                    .is_some_and(|n| n.directory && n.parent.is_some())
            {
                return Err(Error::State("invalid inaccessible subtree identity".into()));
            }
        }
        let saved_scope: Option<String> = self
            .db
            .query_row(
                "SELECT value FROM metadata WHERE key='native_scope'",
                [],
                |r| r.get(0),
            )
            .optional()?;
        if let Some(scope) = saved_scope {
            let scope: serde_json::Value = serde_json::from_str(&scope)?;
            if scope.get("file_id").and_then(|s| s.as_str()) != Some(roots[0].id.as_str()) {
                return Err(Error::State(
                    "recovery cannot silently replace selected root".into(),
                ));
            }
        }
        let mut verified = HashSet::new();
        verified.insert(roots[0].id.as_str());
        for n in nodes {
            let mut visiting = HashSet::new();
            let mut current = n;
            while !verified.contains(current.id.as_str()) {
                if !visiting.insert(current.id.as_str()) {
                    return Err(Error::State("snapshot directory cycle".into()));
                }
                let parent = current
                    .parent
                    .as_deref()
                    .and_then(|p| by_id.get(p))
                    .ok_or_else(|| Error::State("snapshot parent missing".into()))?;
                if !parent.directory {
                    return Err(Error::State("snapshot parent is not directory".into()));
                }
                current = parent;
            }
            verified.extend(visiting);
        }
        crate::status_changes::replace_snapshot(&mut self.db, |tx| {
            crate::initialization_store::check_install_gate(tx, initialization)?;
            tx.execute_batch("CREATE TEMP TABLE recovery_ids(id TEXT PRIMARY KEY);")?;
            {
                let mut insert = tx.prepare("INSERT INTO recovery_ids VALUES(?1)")?;
                for n in nodes {
                    insert.execute([&n.id])?;
                }
            }
            // Permission loss is not deletion. Keep previous denied roots that the
            // server now omits, until the same identity is positively enumerated.
            tx.execute(
                "DELETE FROM access_denied WHERE node IN (SELECT id FROM recovery_ids)",
                [],
            )?;
            {
                let mut insert = tx.prepare("INSERT OR IGNORE INTO access_denied VALUES(?1)")?;
                for id in &inaccessible {
                    insert.execute([id])?;
                }
            }
            tx.execute(
                "INSERT OR IGNORE INTO recovery_ids SELECT id FROM access_blocked",
                [],
            )?;
            // Only absent objects become tombstones. Dirty content remains visible as
            // a conflict and stays protected from garbage collection.
            tx.execute("UPDATE nodes SET deleted=1,conflict=CASE WHEN dirty_blob IS NOT NULL AND id NOT IN (SELECT node FROM local_unlinks) THEN 1 ELSE conflict END WHERE id NOT IN (SELECT id FROM recovery_ids) AND id NOT IN (SELECT node FROM local_creations) AND id NOT IN (SELECT node FROM namespace_overlay)",[])?;
            // A missing/replaced remote object must not leave a pending upload
            // eligible for automatic recreation. Keep its blob for conflict resolution.
            tx.execute("UPDATE operations SET state='conflict' WHERE kind='upload' AND state='pending' AND node IN (SELECT id FROM nodes WHERE deleted=1 AND conflict=1)", [])?;
            tx.execute("UPDATE namespace_overlay SET remote='null' WHERE node NOT IN (SELECT id FROM recovery_ids)",[])?;
            tx.execute("UPDATE nodes SET conflict=1 WHERE id IN (SELECT node FROM namespace_overlay WHERE remote='null') AND id NOT IN (SELECT node FROM local_unlinks)",[])?;
            {
                let mut upsert = tx.prepare("INSERT INTO nodes(id,parent,name,remote_version,directory) VALUES(?1,?2,?3,?4,?5)
                ON CONFLICT(id) DO UPDATE SET parent=excluded.parent,name=excluded.name,remote_version=excluded.remote_version,directory=excluded.directory,deleted=CASE WHEN EXISTS(SELECT 1 FROM local_unlinks WHERE node=nodes.id) THEN 1 ELSE 0 END,
                conflict=CASE WHEN nodes.dirty_blob IS NOT NULL AND nodes.base_version!=excluded.remote_version AND NOT EXISTS(SELECT 1 FROM local_unlinks WHERE node=nodes.id) THEN 1 ELSE nodes.conflict END")?;
                let mut metadata = crate::filesystem::SnapshotMetadataWriter::new(tx)?;
                for original in nodes {
                    let Some(Change::Upsert(n)) =
                        crate::namespace::overlay_change(tx, &Change::Upsert(original.clone()))?
                    else {
                        continue;
                    };
                    upsert.execute(params![n.id, n.parent, n.name, n.version, n.directory])?;
                    metadata.write(&n)?;
                }
            }
            // Stale-flag heal (issue #6 follow-up): a snapshot installed while a
            // namespace operation was in flight marks its overlay member as
            // conflicted, and the later mkdir/rename commit binds the remote
            // identity without clearing the flag. A bound, live, untouched
            // directory with no overlay row, local creation, unlink, pending
            // operation or draft has nothing left to resolve, so the flag cannot
            // describe a real conflict. This is not conflict resolution: real
            // conflicts keep both their flag and their recovery entry (see
            // docs/STALE_CONFLICTS.md).
            tx.execute(
                "UPDATE nodes SET conflict=0
                 WHERE directory=1 AND conflict=1 AND deleted=0
                   AND id IN (SELECT id FROM recovery_ids)
                   AND id IN (SELECT node FROM remote_aliases)
                   AND remote_version != ?1
                   AND dirty_blob IS NULL
                   AND id NOT IN (SELECT node FROM local_creations)
                   AND id NOT IN (SELECT node FROM namespace_overlay)
                   AND id NOT IN (SELECT node FROM local_unlinks)
                   AND NOT EXISTS(SELECT 1 FROM namespace_ops WHERE node=nodes.id AND state NOT IN ('committed','superseded'))
                   AND NOT EXISTS(SELECT 1 FROM operations WHERE node=nodes.id AND state NOT IN ('committed','superseded'))
                   AND NOT EXISTS(SELECT 1 FROM drafts WHERE node=nodes.id)",
                params![crate::creations::CREATE_VERSION],
            )?;
            tx.execute(
                "UPDATE operations SET state='uncertain' WHERE state='inflight'",
                [],
            )?;
            tx.execute("UPDATE transfers SET state='obsolete' WHERE direction='download' AND NOT EXISTS(SELECT 1 FROM nodes n WHERE n.id=transfers.node AND n.remote_version=transfers.version AND n.deleted=0)",[])?;
            tx.execute("DELETE FROM inbox", [])?;
            tx.execute("DELETE FROM seen", [])?;
            tx.execute(
                "UPDATE metadata SET value=?1 WHERE key IN ('received_cursor','committed_cursor')",
                [cursor],
            )?;
            tx.execute("INSERT INTO metadata(key,value) VALUES('server_restore_id',?1) ON CONFLICT(key) DO UPDATE SET value=excluded.value",[server])?;
            if let Some(initialization) = initialization {
                // Preview promotion runs inside this boundary and owns the
                // inode numbers already handed to readers, so batch allocation
                // must come after it to observe those mappings.
                crate::initialization_store::finish_in_transaction(tx, initialization, cursor)?;
            }
            // Stable filesystem inode identities for the whole installed
            // namespace, in the same transaction that installs the nodes.
            // `fs_inodes` is derived, reconstructible state: a crash before
            // this commit loses no node/content/operation data and the next
            // mount backfills the mapping. Doing it here keeps first-scan
            // `opendir` read-only instead of paying one synchronous=FULL WAL
            // commit per directory.
            crate::filesystem::allocate_namespace_inodes(tx, &root_id)?;
            crate::filesystem::mark_inode_map_complete(tx)?;
            tx.execute_batch("DROP TABLE recovery_ids;")?;
            Ok(())
        })
    }

    /// Uncertain uploads that may take one more read-only reconciliation pass.
    /// Exhausted rows stay uncertain for safety but are not selected again.
    ///
    /// Ordered by the durable retry clock, not `rowid`: the v28 partial index on
    /// `reconcile_at` can then answer the range scan without a table scan and
    /// without a temporary sort. The clock is the fairness signal anyway — the
    /// oldest due operation is reconciled first.
    pub fn upload_reconcile_operations(&self, now: i64) -> Result<Vec<String>> {
        let mut q = self.db.prepare(
            "SELECT id FROM operations WHERE kind='upload' AND node NOT IN (SELECT id FROM access_blocked) AND state='uncertain' AND reconcile_attempts < ?1 AND reconcile_at <= ?2 ORDER BY reconcile_at",
        )?;
        Ok(q.query_map(params![i64::from(MAX_UPLOAD_RECONCILE_ATTEMPTS), now], |r| {
            r.get::<_, String>(0)
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?)
    }

    pub fn upload_reconcile_due(&self, now: i64) -> Result<bool> {
        Ok(self.db.query_row(
            "SELECT EXISTS(SELECT 1 FROM operations WHERE kind='upload' AND node NOT IN (SELECT id FROM access_blocked) AND state='uncertain' AND reconcile_attempts < ?1 AND reconcile_at <= ?2)",
            params![i64::from(MAX_UPLOAD_RECONCILE_ATTEMPTS), now],
            |r| r.get(0),
        )?)
    }

    /// Record one automatic read-only reconciliation attempt. The operation
    /// must still be uncertain; an already committed or superseded operation is
    /// never rewritten. History detail is updated in the same transaction.
    ///
    /// The final attempt is terminal: no further read-only pass may be selected,
    /// so the operation moves to `abandoned` instead of parking as a permanent
    /// `uncertain` row that would block pending work forever. Local content and
    /// the permission evidence are retained; a user recheck re-arms the row.
    pub fn note_upload_reconcile_attempt(&mut self, operation: &str, now: i64) -> Result<u32> {
        let tx = self.db.transaction()?;
        let (node, state, attempts): (String, String, i64) = tx.query_row(
            "SELECT node,state,reconcile_attempts FROM operations WHERE id=?1 AND kind='upload'",
            [operation],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )?;
        if state != "uncertain" {
            return Err(Error::State(
                "upload reconcile requires an uncertain operation".into(),
            ));
        }
        let attempts = u32::try_from(attempts.saturating_add(1)).unwrap_or(u32::MAX);
        if attempts >= MAX_UPLOAD_RECONCILE_ATTEMPTS {
            tx.execute(
                "UPDATE operations SET state='abandoned', reconcile_attempts=?2, reconcile_at=0 WHERE id=?1 AND state='uncertain'",
                params![operation, i64::from(attempts)],
            )?;
            crate::sync_history::record_reconcile_exhausted(
                &tx,
                &format!("upload:{operation}"),
                &node,
                attempts,
            )?;
        } else {
            tx.execute(
                "UPDATE operations SET reconcile_attempts=?2, reconcile_at=?3 WHERE id=?1 AND state='uncertain'",
                params![
                    operation,
                    i64::from(attempts),
                    now.saturating_add(upload_reconcile_delay(attempts))
                ],
            )?;
            crate::sync_history::note_reconcile_attempt(
                &tx,
                &format!("upload:{operation}"),
                attempts,
            )?;
        }
        tx.commit()?;
        Ok(attempts)
    }

    /// A user-requested recheck grants every uncertain upload a fresh budget.
    /// Exhausted (`abandoned`) rows are re-armed to `uncertain` with a zeroed
    /// budget; they are not selectable again until this re-arm. The history row
    /// loses its `abandoned` marker so the GUI stops presenting a terminal state.
    pub fn reset_upload_reconcile_budget(&mut self) -> Result<usize> {
        let tx = self.db.transaction()?;
        let abandoned: Vec<String> = {
            let mut q = tx.prepare(
                "SELECT id FROM operations WHERE kind='upload' AND state='abandoned'",
            )?;
            q.query_map([], |r| r.get::<_, String>(0))?
                .collect::<std::result::Result<Vec<_>, _>>()?
        };
        let changed = tx.execute(
            "UPDATE operations SET state='uncertain', reconcile_attempts=0, reconcile_at=0 WHERE kind='upload' AND state IN ('uncertain','abandoned')",
            [],
        )?;
        for operation in &abandoned {
            tx.execute(
                "UPDATE sync_history SET error_summary='已重新开始只读核对，服务器提交结果待确认', \
                 detail=json_patch(json_remove(COALESCE(detail,'{}'), '$.abandoned'), json_object('reconcile_attempts', 0)) \
                 WHERE operation_key=?1",
                [format!("upload:{operation}")],
            )?;
        }
        tx.commit()?;
        Ok(changed)
    }

    /// Local unlink tombstones a node; operation-side retirement uses this to
    /// decide whether a user-authorized deletion already happened.
    pub fn is_locally_deleted(&self, node: &str) -> Result<bool> {
        Ok(self.db.query_row(
            "SELECT EXISTS(SELECT 1 FROM nodes WHERE id=?1 AND deleted=1)",
            [node],
            |r| r.get(0),
        )?)
    }

    /// Retire an unconfirmed creation whose local path the user already deleted.
    /// The caller must own the deletion decision; this only accepts an
    /// uncertain/abandoned upload of an unbound creation and returns false
    /// otherwise, leaving state untouched. The uncertain result is never
    /// replayed and no terminal state value is invented: the operation becomes
    /// superseded with explanatory history.
    pub fn retire_unconfirmed_creation(&mut self, operation: &str) -> Result<bool> {
        let row: Option<(String, String)> = self
            .db
            .query_row(
                "SELECT node,state FROM operations WHERE id=?1 AND kind='upload'",
                [operation],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()?;
        let Some((node, state)) = row else {
            return Ok(false);
        };
        if !matches!(state.as_str(), "uncertain" | "abandoned")
            || !self.is_locally_deleted(&node)?
        {
            return Ok(false);
        }
        if !crate::removals::creation_unbound(&self.db, &node)? {
            return Ok(false);
        }
        let tx = self.db.transaction()?;
        crate::removals::retire_creation_tx(&tx, operation, &node, "local_deleted")?;
        tx.commit()?;
        Ok(true)
    }

    /// A user-authorized discard of an unconfirmed local creation. Only a
    /// never-bound local creation with an uncertain or exhausted upload
    /// qualifies: the local file is deleted, the operation is retired with
    /// explanatory history, and no server request is ever issued. Bound files
    /// are refused because their server object stores a different version and
    /// only a recheck can resolve the outcome.
    pub fn abandon_unconfirmed_creation(&mut self, node: &str) -> Result<bool> {
        if self.is_locally_deleted(node)? {
            return Ok(false);
        }
        let row: Option<String> = self
            .db
            .query_row(
                "SELECT id FROM operations WHERE kind='upload' AND node=?1 AND state IN ('uncertain','abandoned') ORDER BY rowid LIMIT 1",
                [node],
                |r| r.get(0),
            )
            .optional()?;
        let Some(operation) = row else {
            return Ok(false);
        };
        if !crate::removals::creation_unbound(&self.db, node)? {
            return Ok(false);
        }
        let tx = self.db.transaction()?;
        crate::removals::retire_creation_tx(&tx, &operation, node, "user_discarded")?;
        tx.commit()?;
        Ok(true)
    }

    /// Uploads whose bounded automatic reconciliation stopped without a
    /// confirmed server result. They are terminal for the pending-work counter
    /// but still explainable in the issue list and re-armable by a recheck.
    pub fn paused_upload_operations(&self) -> Result<i64> {
        Ok(self.db.query_row(
            "SELECT count(*) FROM operations WHERE kind='upload' AND state='abandoned'",
            [],
            |r| r.get(0),
        )?)
    }

    /// Offline availability summary from local metadata only. Directories are
    /// not countable entries; deleted and access-blocked nodes never count as
    /// available. A dirty draft is available because it preserves local bytes.
    pub fn cache_availability(&self) -> Result<CacheAvailability> {
        // One sequential pass answers both counters. The previous shape folded
        // the recursive pinned CTE into the same scan, so SQLite built automatic
        // partial indexes for every recursion step: ~11x slower at 210k nodes
        // even though the result is three numbers (issue #21).
        let (total, offline) = self.db.query_row(
            "SELECT COALESCE(SUM(CASE WHEN n.directory=0 THEN 1 ELSE 0 END),0), \
                    COALESCE(SUM(CASE WHEN n.directory=0 AND COALESCE(n.dirty_blob,n.blob) IS NOT NULL \
                        AND n.id NOT IN (SELECT id FROM access_blocked) THEN 1 ELSE 0 END),0) \
             FROM nodes n WHERE n.deleted=0",
            [],
            |r| Ok((r.get::<_, i64>(0)? as u64, r.get::<_, i64>(1)? as u64)),
        )?;
        // Directories are not countable entries, so carry `directory` through the
        // pinned recursion and count the co-routine's output. `id IN pinned_tree`
        // would materialize the tree and re-scan all nodes once per aggregate.
        let pinned: i64 = self.db.query_row(
            "WITH RECURSIVE pinned_tree(id,directory) AS (\
                SELECT id,directory FROM nodes WHERE pinned=1 AND deleted=0 \
                UNION \
                SELECT n.id,n.directory FROM nodes n INDEXED BY nodes_parent_name_live \
                    JOIN pinned_tree p ON n.parent=p.id WHERE n.deleted=0) \
             SELECT count(*) FROM pinned_tree WHERE directory=0",
            [],
            |r| r.get(0),
        )?;
        let pinned = pinned as u64;
        let state = if total == 0 {
            "empty"
        } else if pinned > 0 && pinned == total && offline == total {
            "pinned"
        } else if offline == 0 {
            "online_only"
        } else if offline < total {
            "partial"
        } else {
            "complete"
        };
        Ok(CacheAvailability {
            state,
            total,
            offline,
            pinned,
        })
    }

    pub fn pending_operations(&self) -> Result<i64> {
        Ok(self.db.query_row(
            "SELECT count(*) FROM operations WHERE state IN ('pending','inflight','uncertain')",
            [],
            |r| r.get(0),
        )?)
    }
}
pub(crate) fn valid_node(n: &Node) -> Result<()> {
    if n.id.is_empty()
        || n.version.is_empty()
        || n.name.is_empty()
        || matches!(n.name.as_str(), "." | "..")
        || n.name.contains('/')
        || n.name.contains('\0')
        || n.parent.as_deref() == Some(n.id.as_str())
    {
        return Err(Error::State(
            "invalid node identity, version, or name".into(),
        ));
    }
    Ok(())
}
pub fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[cfg(test)]
mod cache_availability_tests {
    use super::*;

    fn node(id: &str, parent: Option<&str>, name: &str, version: &str, directory: bool) -> Node {
        Node {
            id: id.into(),
            parent: parent.map(str::to_owned),
            name: name.into(),
            version: version.into(),
            directory,
            metadata: Some(NodeMetadata {
                size: 3,
                mtime: 1,
                mode: 0o644,
            }),
        }
    }

    fn fixture() -> (tempfile::TempDir, Store) {
        let d = tempfile::tempdir().unwrap();
        let mut s = Store::open(d.path()).unwrap();
        s.install_remote_snapshot(
            &[
                node("root", None, "root", "r", true),
                node("a", Some("root"), "a.doc", "v1", false),
                node("b", Some("root"), "b.doc", "v1", false),
            ],
            "1",
            "server",
        )
        .unwrap();
        s.ensure_fs_root("root").unwrap();
        (d, s)
    }

    fn publish(s: &mut Store, id: &str, bytes: &[u8]) {
        s.publish_download(
            &DownloadTicket {
                id: id.into(),
                version: "v1".into(),
            },
            bytes,
            &sha256(bytes),
        )
        .unwrap();
    }

    #[test]
    fn reports_empty_online_only_partial_complete_and_pinned() {
        let d = tempfile::tempdir().unwrap();
        let s = Store::open(d.path()).unwrap();
        assert_eq!(
            s.cache_availability().unwrap(),
            CacheAvailability {
                state: "empty",
                total: 0,
                offline: 0,
                pinned: 0,
            }
        );
        drop(s);
        let (_d, mut s) = fixture();
        assert_eq!(s.cache_availability().unwrap().state, "online_only");
        publish(&mut s, "a", b"a");
        let c = s.cache_availability().unwrap();
        assert_eq!((c.state, c.total, c.offline, c.pinned), ("partial", 2, 1, 0));
        publish(&mut s, "b", b"b");
        assert_eq!(s.cache_availability().unwrap().state, "complete");
        s.pin("root", true).unwrap();
        let c = s.cache_availability().unwrap();
        assert_eq!((c.state, c.total, c.offline, c.pinned), ("pinned", 2, 2, 2));
        s.install_remote_snapshot(
            &[
                node("root", None, "root", "r2", true),
                node("a", Some("root"), "a.doc", "v1", false),
            ],
            "2",
            "server",
        )
        .unwrap();
        let c = s.cache_availability().unwrap();
        assert_eq!((c.state, c.total, c.offline, c.pinned), ("pinned", 1, 1, 1));
    }

    #[test]
    fn excludes_access_denied_nodes_from_offline_availability() {
        let d = tempfile::tempdir().unwrap();
        let mut s = Store::open(d.path()).unwrap();
        let nodes = [
            node("root", None, "root", "r", true),
            node("a", Some("root"), "a.doc", "v1", false),
            node("folder", Some("root"), "folder", "v1", true),
            node("b", Some("folder"), "b.doc", "v1", false),
        ];
        s.install_remote_snapshot(&nodes, "1", "server").unwrap();
        s.ensure_fs_root("root").unwrap();
        publish(&mut s, "a", b"a");
        publish(&mut s, "b", b"b");
        assert_eq!(s.cache_availability().unwrap().state, "complete");
        s.install_remote_snapshot_with_access(&nodes, "2", "server", &["folder".into()])
            .unwrap();
        let c = s.cache_availability().unwrap();
        assert_eq!((c.state, c.total, c.offline, c.pinned), ("partial", 2, 1, 0));
    }
}
