use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
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
    // Fields drop in order: close SQLite before releasing writer ownership.
    _writer_lock: WriterLock,
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
pub const STATE_SCHEMA_VERSION: i64 = 18;

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
            CREATE TABLE IF NOT EXISTS operations (
                id TEXT PRIMARY KEY, node TEXT NOT NULL, kind TEXT NOT NULL, base TEXT NOT NULL,
                blob TEXT, generation INTEGER NOT NULL, state TEXT NOT NULL DEFAULT 'pending');
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
            CREATE TABLE IF NOT EXISTS namespace_overlay(node TEXT PRIMARY KEY,remote TEXT NOT NULL);
            CREATE TABLE IF NOT EXISTS content_version_aliases(node TEXT NOT NULL,old TEXT NOT NULL,new TEXT NOT NULL,PRIMARY KEY(node,old));
            CREATE TABLE IF NOT EXISTS upload_origins(operation TEXT PRIMARY KEY,body TEXT NOT NULL);
            CREATE TABLE IF NOT EXISTS upload_paths(operation TEXT PRIMARY KEY,path TEXT NOT NULL);
            CREATE TABLE IF NOT EXISTS local_unlinks(node TEXT PRIMARY KEY,confirmed INTEGER NOT NULL);
            CREATE TABLE IF NOT EXISTS cache_access(seq INTEGER PRIMARY KEY AUTOINCREMENT,node TEXT UNIQUE NOT NULL);
            CREATE TABLE IF NOT EXISTS cache_published(node TEXT PRIMARY KEY,at INTEGER NOT NULL);
            CREATE TABLE IF NOT EXISTS local_modes(node TEXT PRIMARY KEY,mode INTEGER NOT NULL CHECK(mode>=0 AND mode<=511));
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
        db.pragma_update(None, "user_version", STATE_SCHEMA_VERSION)?;
        db.execute_batch("COMMIT;")?;
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
        Ok(Self {
            _writer_lock: writer_lock,
            db,
            root,
            handles: Arc::new(Mutex::new(HashMap::new())),
        })
    }
    pub fn cursor(&self) -> Result<String> {
        Ok(self.db.query_row(
            "SELECT value FROM metadata WHERE key='committed_cursor'",
            [],
            |r| r.get(0),
        )?)
    }
    pub fn skipped_symlink_count(&self) -> Result<u64> {
        let count: Option<String> = self.db.query_row("SELECT value FROM metadata WHERE key='skipped_symlinks'", [], |r| r.get(0)).optional()?;
        Ok(count.and_then(|s| s.parse().ok()).unwrap_or(0))
    }
    pub fn inaccessible_count(&self) -> Result<i64> {
        Ok(self.db.query_row("SELECT count(*) FROM access_denied", [], |r| r.get(0))?)
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
        if self.is_inaccessible(id)? { return Err(rusqlite::Error::QueryReturnedNoRows.into()); }
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
        Ok(self.db.query_row("SELECT value FROM metadata WHERE key=?1", [key], |r| r.get(0)).optional()?)
    }
    pub fn claim_extension_marker(&mut self, name: &str, value: &str) -> Result<()> {
        if name.is_empty() || name.len()>128 || value.len()>4096 {
            return Err(Error::State("invalid local extension marker".into()));
        }
        let key = format!("extension:{name}");
        let tx=self.db.transaction()?;
        tx.execute("INSERT OR IGNORE INTO metadata(key,value) VALUES(?1,?2)",params![key,value])?;
        let existing:String=tx.query_row("SELECT value FROM metadata WHERE key=?1",[key],|r|r.get(0))?;
        if existing!=value {return Err(Error::State("local extension marker already claimed".into()));}
        tx.commit()?;Ok(())
    }

    /// Store the full batch before processing it. Receipt and consumption have distinct cursors.
    pub fn receive(&mut self, batch: &Batch) -> Result<()> {
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
        }
        tx.execute("UPDATE inbox SET done=1 WHERE seq=?1", [seq])?;
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
        if self.is_inaccessible(id)? { return Err(Error::Io(std::io::Error::from_raw_os_error(libc::EACCES))); }
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
        let mut q=self.db.prepare("WITH RECURSIVE pinned_nodes(id) AS (SELECT id FROM nodes WHERE pinned=1 UNION SELECT n.id FROM nodes n JOIN pinned_nodes p ON n.parent=p.id) SELECT n.id,n.remote_version,n.blob IS NOT NULL FROM nodes n WHERE n.directory=0 AND n.deleted=0 AND n.id NOT IN (SELECT id FROM access_blocked) AND n.dirty_blob IS NULL AND (n.blob IS NULL OR n.local_version!=n.remote_version) AND (n.blob IS NOT NULL OR n.id IN (SELECT id FROM pinned_nodes) OR EXISTS(SELECT 1 FROM transfers t WHERE t.node=n.id AND t.version=n.remote_version AND t.direction='download' AND t.state IN ('pending','inflight','retry','failed')))")?;
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
            if self.cache_release_pending(&DownloadTicket{id:id.clone(),version:version.clone()})? {
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
        if self.dirty_draft(&ticket.id)? {
            return Err(Error::State(
                "download cannot replace a local working copy".into(),
            ));
        }

        let blob = self.write_blob_reader(source, Some((size, expected_sha256)))?;
        let tx = self.db.transaction()?;
        let changed=tx.execute("UPDATE nodes SET blob=?3,local_version=?2 WHERE id=?1 AND remote_version=?2 AND dirty_blob IS NULL AND deleted=0 AND directory=0",params![ticket.id,ticket.version,blob])?;
        if changed != 1 {
            return Err(Error::State(
                "download belongs to obsolete identity/version or locally modified file".into(),
            ));
        }
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            .min(i64::MAX as u64) as i64;
        tx.execute("INSERT INTO cache_published(node,at) VALUES(?1,?2) ON CONFLICT(node) DO UPDATE SET at=excluded.at",params![ticket.id,now])?;
        tx.commit()?;
        Ok(())
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
        if self.is_inaccessible(id)? { return Err(Error::Io(std::io::Error::from_raw_os_error(libc::EACCES))); }
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
    pub fn release(&mut self, id: &str) -> Result<()> {
        if self.cache_is_protected(id)? {
            return Err(Error::State(
                "content has a protected local reference".into(),
            ));
        }
        let tx = self.db.transaction()?;
        if tx.execute("UPDATE nodes SET blob=NULL,local_version=NULL WHERE id=?1 AND dirty_blob IS NULL AND conflict=0",[id])?!=1 {return Err(Error::State("content is dirty, conflicted, or absent".into()));}
        tx.execute(
            "DELETE FROM transfers WHERE node=?1 AND direction='download'",
            [id],
        )?;
        tx.commit()?;
        Ok(())
    }

    /// Reclaim only unreferenced generated blobs. Journaled writes, conflicts and
    /// open old versions remain protected. No remote operation is created.
    pub fn collect_garbage(&mut self) -> Result<u64> {
        let mut q=self.db.prepare("SELECT blob FROM nodes WHERE blob IS NOT NULL UNION SELECT dirty_blob FROM nodes WHERE dirty_blob IS NOT NULL UNION SELECT blob FROM operations WHERE blob IS NOT NULL AND state IN ('pending','inflight','uncertain','conflict') UNION SELECT json_extract(body,'$.blob') FROM namespace_ops WHERE kind='replace' AND state NOT IN ('committed','superseded')")?;
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
        self.commit_local_blob(id, opened_version, blob)
    }
    /// FUSE uses an immutable/locked draft reader; no whole-file allocation.
    pub fn save_local_reader_from_version<R: Read>(
        &mut self,
        id: &str,
        opened_version: &str,
        source: &mut R,
    ) -> Result<String> {
        let blob = self.write_blob_reader(source, None)?;
        self.commit_local_blob(id, opened_version, blob)
    }
    fn commit_local_blob(
        &mut self,
        id: &str,
        opened_version: &str,
        blob: String,
    ) -> Result<String> {
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
            "UPDATE operations SET state='superseded' WHERE node=?1 AND state='pending'",
            [id],
        )?;
        tx.execute("INSERT INTO operations(id,node,kind,base,blob,generation) VALUES(?1,?2,'upload',?3,?4,?5)",params![op,id,base,blob,generation+1])?;
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
        let tx = self.db.transaction()?;
        let changed=tx.execute("UPDATE operations SET state='inflight' WHERE id=?1 AND state='pending' AND node NOT IN (SELECT id FROM access_blocked) AND NOT EXISTS(SELECT 1 FROM operations other WHERE other.node=operations.node AND other.state IN ('inflight','uncertain')) AND EXISTS(SELECT 1 FROM nodes n WHERE n.id=operations.node AND n.remote_version=operations.base AND n.deleted=0 AND n.conflict=0) AND NOT EXISTS(SELECT 1 FROM drafts d WHERE d.node=operations.node AND d.generation!=d.checkpoint) AND NOT EXISTS(SELECT 1 FROM local_creations c WHERE c.node=operations.node AND c.ready=0) AND NOT EXISTS(SELECT 1 FROM namespace_ops WHERE kind='remove' AND id NOT IN (SELECT operation FROM access_blocked_operations) AND state NOT IN ('committed','superseded'))",[op])?;
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
    pub(crate) fn install_remote_snapshot_with_access(
        &mut self, nodes: &[Node], cursor: &str, server: &str, inaccessible: &[String],
    ) -> Result<()> {
        use std::collections::{HashMap, HashSet};
        let inaccessible = inaccessible.iter().map(|id| self.local_identity(id)).collect::<Result<Vec<_>>>()?;
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
        let mut denied_unique = HashSet::new();
        for id in &inaccessible {
            if !denied_unique.insert(id) || !by_id.get(id.as_str()).is_some_and(|n| n.directory && n.parent.is_some()) {
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
        let tx = self.db.transaction()?;
        tx.execute_batch("CREATE TEMP TABLE recovery_ids(id TEXT PRIMARY KEY);")?;
        for n in nodes {
            tx.execute("INSERT INTO recovery_ids VALUES(?1)", [&n.id])?;
        }
        // Permission loss is not deletion. Keep previous denied roots that the
        // server now omits, until the same identity is positively enumerated.
        tx.execute("DELETE FROM access_denied WHERE node IN (SELECT id FROM recovery_ids)", [])?;
        for id in &inaccessible { tx.execute("INSERT OR IGNORE INTO access_denied VALUES(?1)", [id])?; }
        tx.execute("INSERT OR IGNORE INTO recovery_ids SELECT id FROM access_blocked", [])?;
        // Only absent objects become tombstones. Dirty content remains visible as
        // a conflict and stays protected from garbage collection.
        tx.execute("UPDATE nodes SET deleted=1,conflict=CASE WHEN dirty_blob IS NOT NULL AND id NOT IN (SELECT node FROM local_unlinks) THEN 1 ELSE conflict END WHERE id NOT IN (SELECT id FROM recovery_ids) AND id NOT IN (SELECT node FROM local_creations) AND id NOT IN (SELECT node FROM namespace_overlay)",[])?;
        // A missing/replaced remote object must not leave a pending upload
        // eligible for automatic recreation. Keep its blob for conflict resolution.
        tx.execute("UPDATE operations SET state='conflict' WHERE kind='upload' AND state='pending' AND node IN (SELECT id FROM nodes WHERE deleted=1 AND conflict=1)", [])?;
        tx.execute("UPDATE namespace_overlay SET remote='null' WHERE node NOT IN (SELECT id FROM recovery_ids)",[])?;
        tx.execute("UPDATE nodes SET conflict=1 WHERE id IN (SELECT node FROM namespace_overlay WHERE remote='null') AND id NOT IN (SELECT node FROM local_unlinks)",[])?;
        for original in nodes {
            let Some(Change::Upsert(n)) =
                crate::namespace::overlay_change(&tx, &Change::Upsert(original.clone()))?
            else {
                continue;
            };
            tx.execute("INSERT INTO nodes(id,parent,name,remote_version,directory) VALUES(?1,?2,?3,?4,?5)
                ON CONFLICT(id) DO UPDATE SET parent=excluded.parent,name=excluded.name,remote_version=excluded.remote_version,directory=excluded.directory,deleted=CASE WHEN EXISTS(SELECT 1 FROM local_unlinks WHERE node=nodes.id) THEN 1 ELSE 0 END,
                conflict=CASE WHEN nodes.dirty_blob IS NOT NULL AND nodes.base_version!=excluded.remote_version AND NOT EXISTS(SELECT 1 FROM local_unlinks WHERE node=nodes.id) THEN 1 ELSE nodes.conflict END",
                params![n.id,n.parent,n.name,n.version,n.directory])?;
            crate::filesystem::write_metadata(&tx, &n)?;
        }
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
        tx.execute_batch("DROP TABLE recovery_ids;")?;
        tx.commit()?;
        Ok(())
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
