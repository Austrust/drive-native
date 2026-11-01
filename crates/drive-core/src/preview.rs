//! Cold-empty initialization metadata only. No candidate enters generic Node,
//! body, draft or namespace APIs. Callers serialize view validation and reply
//! with the same Store guard; a preview stamp is not a remote authorization.
use crate::filesystem::{DirectoryEntry, DirectorySnapshot};
use crate::initialization_store::{self as init, InitializationStatus};
use crate::{Error, Node, Result, Store};
use rusqlite::{Connection, OptionalExtension, params};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PreviewOrigin {
    pub run_id: String,
    pub epoch: u64,
    pub root_id: String,
}
#[derive(Clone, Debug)]
pub struct PreviewRecord {
    pub id: String,
    pub name: String,
    pub directory: bool,
    pub ino: u64,
    pub size: u64,
    pub mtime: u64,
    pub mode: u32,
    pub origin: PreviewOrigin,
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct CoverageToken {
    id: String,
    native: u64,
    generation: i64,
    receipt: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PreviewStamp {
    origin: PreviewOrigin,
    directory: String,
    members_revision: u64,
    chain: Vec<CoverageToken>,
}
impl PreviewStamp {
    /// Opaque, local comparison token, never a server cursor or permission.
    pub fn fingerprint(&self) -> String {
        crate::sha256(format!("preview-v1:{self:?}").as_bytes())
    }
}
#[derive(Debug)]
pub struct PreviewDirectorySnapshot {
    pub snapshot: DirectorySnapshot,
    pub stamp: PreviewStamp,
}
#[derive(Debug)]
pub struct PreviewPage {
    pub entries: Vec<PreviewRecord>,
    pub total: usize,
    pub stamp: PreviewStamp,
}

fn error(code: i32) -> Error {
    Error::Io(std::io::Error::from_raw_os_error(code))
}
fn invalid(message: &str) -> Error {
    Error::State(format!("preview: {message}"))
}
fn sql_number(n: u64) -> Result<i64> {
    i64::try_from(n).map_err(|_| invalid("inode overflow"))
}
fn read_u64(row: &rusqlite::Row<'_>, n: usize) -> rusqlite::Result<u64> {
    let value: i64 = row.get(n)?;
    u64::try_from(value).map_err(|_| rusqlite::Error::IntegralValueOutOfRange(n, value))
}
pub(crate) fn migrate(db: &Connection) -> Result<()> {
    let had_root_marker = db
        .prepare("PRAGMA table_info(preview_visibility)")?
        .query_map([], |r| r.get::<_, String>(1))?
        .collect::<std::result::Result<Vec<_>, _>>()?
        .iter()
        .any(|name| name == "root_invalid");
    db.execute_batch("CREATE TABLE IF NOT EXISTS preview_mount(
      slot INTEGER PRIMARY KEY CHECK(slot=1),task TEXT NOT NULL,scope TEXT NOT NULL,root TEXT NOT NULL,
      allocated INTEGER NOT NULL DEFAULT 0 CHECK(typeof(allocated)='integer' AND allocated BETWEEN 0 AND 500000),
      bytes INTEGER NOT NULL DEFAULT 0 CHECK(typeof(bytes)='integer' AND bytes BETWEEN 0 AND 536870912));
      CREATE TABLE IF NOT EXISTS preview_inodes(
      ino INTEGER PRIMARY KEY REFERENCES fs_inodes(ino),reserved TEXT UNIQUE NOT NULL,
      run TEXT NOT NULL,node TEXT NOT NULL,native TEXT NOT NULL,
      UNIQUE(run,node,native));
      CREATE TRIGGER IF NOT EXISTS preview_inode_charge AFTER INSERT ON preview_inodes BEGIN
        UPDATE preview_mount SET allocated=allocated+1,bytes=bytes+length(CAST(new.node AS BLOB))+length(CAST(new.reserved AS BLOB))+length(CAST(new.run AS BLOB))+length(CAST(new.native AS BLOB))+64 WHERE slot=1;
      END;
      CREATE TRIGGER IF NOT EXISTS preview_inode_uncharge AFTER DELETE ON preview_inodes BEGIN
        UPDATE preview_mount SET allocated=allocated-1,bytes=bytes-length(CAST(old.node AS BLOB))-length(CAST(old.reserved AS BLOB))-length(CAST(old.run AS BLOB))-length(CAST(old.native AS BLOB))-64 WHERE slot=1;
      END;
      CREATE TABLE IF NOT EXISTS preview_visibility(
      run TEXT NOT NULL,id TEXT NOT NULL,revision INTEGER NOT NULL CHECK(typeof(revision)='integer' AND revision>=0),
      root_invalid INTEGER NOT NULL DEFAULT 0 CHECK(root_invalid IN (0,1)),
      PRIMARY KEY(run,id),FOREIGN KEY(run,id) REFERENCES initialization_entries(run,id) ON DELETE CASCADE);
      CREATE TABLE IF NOT EXISTS preview_denied(
      run TEXT NOT NULL,id TEXT NOT NULL,PRIMARY KEY(run,id),
      FOREIGN KEY(run,id) REFERENCES initialization_entries(run,id) ON DELETE CASCADE);
      CREATE TRIGGER IF NOT EXISTS preview_denied_capture AFTER UPDATE OF state ON initialization_directories
      WHEN new.state='denied' BEGIN INSERT OR IGNORE INTO preview_denied(run,id) VALUES(new.run,new.id); END;
      CREATE TRIGGER IF NOT EXISTS preview_denied_clear AFTER UPDATE OF state ON initialization_directories
      WHEN new.state='received' AND NOT EXISTS(SELECT 1 FROM initialization_run r WHERE r.run_id=new.run AND json_extract(r.binding,'$.scope.file_id')=new.id)
      BEGIN DELETE FROM preview_denied WHERE run=new.run AND id=new.id; END;
      CREATE TRIGGER IF NOT EXISTS preview_denied_visibility AFTER INSERT ON preview_denied BEGIN
        INSERT INTO preview_visibility(run,id,revision)
          SELECT run,parent,1 FROM initialization_entries WHERE run=new.run AND id=new.id AND parent IS NOT NULL
          ON CONFLICT(run,id) DO UPDATE SET revision=revision+1;
      END;
      CREATE TRIGGER IF NOT EXISTS preview_allowed_visibility AFTER DELETE ON preview_denied BEGIN
        INSERT INTO preview_visibility(run,id,revision)
          SELECT run,parent,1 FROM initialization_entries WHERE run=old.run AND id=old.id AND parent IS NOT NULL
          ON CONFLICT(run,id) DO UPDATE SET revision=revision+1;
      END;
      INSERT OR IGNORE INTO preview_denied(run,id) SELECT run,id FROM initialization_directories WHERE state='denied';")?;
    let has_root_marker = db
        .prepare("PRAGMA table_info(preview_visibility)")?
        .query_map([], |r| r.get::<_, String>(1))?
        .collect::<std::result::Result<Vec<_>, _>>()?
        .iter()
        .any(|name| name == "root_invalid");
    if !has_root_marker {
        db.execute_batch("ALTER TABLE preview_visibility ADD COLUMN root_invalid INTEGER NOT NULL DEFAULT 0 CHECK(root_invalid IN (0,1));")?;
    }
    if !had_root_marker {
        // Older candidates did not distinguish a probe from a known change.
        // Conservatively require one actual root validation for those rows.
        db.execute_batch("INSERT INTO preview_visibility(run,id,revision,root_invalid)
          SELECT r.run_id,n.id,0,1 FROM initialization_run r JOIN initialization_entries n
          ON n.run=r.run_id AND n.id=json_extract(r.binding,'$.scope.file_id') WHERE r.root_refresh=1
          ON CONFLICT(run,id) DO UPDATE SET root_invalid=1;")?;
    }
    Ok(())
}

fn cold(db: &Connection) -> Result<bool> {
    // A missing root in a populated or damaged old database is not a new mount.
    let unsafe_state: bool = db.query_row("SELECT
      EXISTS(SELECT 1 FROM nodes) OR EXISTS(SELECT 1 FROM drafts)
      OR EXISTS(SELECT 1 FROM local_creations) OR EXISTS(SELECT 1 FROM operations)
      OR EXISTS(SELECT 1 FROM namespace_ops) OR EXISTS(SELECT 1 FROM namespace_overlay)
      OR EXISTS(SELECT 1 FROM local_unlinks) OR EXISTS(SELECT 1 FROM upload_origins)
      OR EXISTS(SELECT 1 FROM engine_bridge_intents) OR EXISTS(SELECT 1 FROM engine_upload_stages)
      OR EXISTS(SELECT 1 FROM metadata WHERE key IN ('committed_cursor','received_cursor') AND value!='')",[],|r|r.get(0))?;
    if unsafe_state {
        return Ok(false);
    }
    // A sealed cold mount has no authoritative Node through which the normal
    // filesystem can allocate an inode. Preview allocation checks each reserved
    // mapping; promotion first installs Nodes in its same transaction. Avoid an
    // O(all allocated inodes) absence proof on every getattr/readdir continuation.
    if db.query_row("SELECT EXISTS(SELECT 1 FROM preview_mount)", [], |r| {
        r.get::<_, bool>(0)
    })? {
        return Ok(true);
    }
    let unreserved:bool=db.query_row("SELECT EXISTS(SELECT 1 FROM fs_inodes f LEFT JOIN preview_inodes p ON p.ino=f.ino WHERE p.ino IS NULL OR f.node!=p.reserved)",[],|r|r.get(0))?;
    Ok(!unreserved)
}
fn session(db: &Connection) -> Result<Option<InitializationStatus>> {
    if !cold(db)? {
        return Ok(None);
    }
    let run = db
        .query_row(
            "SELECT run_id,epoch FROM initialization_run WHERE state!='ready'",
            [],
            |r| {
                Ok(crate::initialization_store::InitializationRun {
                    run_id: r.get(0)?,
                    epoch: read_u64(r, 1)?,
                })
            },
        )
        .optional()?;
    run.map(|run| {
        let s = init::checked(db, &run, true)?;
        check_mount_binding(db, &s)?;
        Ok(s)
    })
    .transpose()
}
fn required_session(db: &Connection) -> Result<InitializationStatus> {
    session(db)?.ok_or_else(|| error(libc::ESTALE))
}
fn origin(s: &InitializationStatus) -> Result<PreviewOrigin> {
    Ok(PreviewOrigin {
        run_id: s.run.run_id.clone(),
        epoch: s.run.epoch,
        root_id: init::root_id(&s.binding)?.into(),
    })
}
fn identity(db: &Connection, s: &InitializationStatus, node: &Node) -> Result<u64> {
    if node.id == init::root_id(&s.binding)? {
        init::validate_root(&s.binding, node)?;
        return s
            .binding
            .scope
            .get("node_id")
            .and_then(|n| n.as_u64())
            .filter(|n| *n != 0)
            .ok_or_else(|| invalid("root native identity missing"));
    }
    init::preview_native_identity(db, &s.run.run_id, &node.id)?.ok_or_else(|| error(libc::EAGAIN))
}
fn coverage(
    db: &Connection,
    s: &InitializationStatus,
    node: &Node,
) -> Result<(String, i64, Option<String>)> {
    db.query_row("SELECT state,generation,receipt_hash FROM initialization_directories WHERE run=?1 AND id=?2",
        params![s.run.run_id,node.id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).map_err(Into::into)
}
fn ready_token(db: &Connection, s: &InitializationStatus, node: &Node) -> Result<CoverageToken> {
    deny_guard(db, s, &node.id)?;
    let (state, generation, receipt) = coverage(db, s, node)?;
    if state == "denied" {
        return Err(error(libc::EACCES));
    }
    if state != "received" {
        return Err(error(libc::EAGAIN));
    }
    if generation <= 0 {
        return Err(invalid("invalid received generation"));
    }
    Ok(CoverageToken {
        id: node.id.clone(),
        native: identity(db, s, node)?,
        generation,
        receipt: receipt.ok_or_else(|| invalid("received directory lacks receipt"))?,
    })
}
fn metadata_guard(db: &Connection, s: &InitializationStatus, node: &Node) -> Result<()> {
    deny_guard(db, s, init::root_id(&s.binding)?)?;
    deny_guard(db, s, &node.id)?;
    if root_invalid(db, s)? {
        return Err(error(libc::EAGAIN));
    }
    crate::state::valid_node(node)?;
    identity(db, s, node)?;
    if node.directory && coverage(db, s, node)?.0 == "denied" {
        return Err(error(libc::EACCES));
    }
    if let Some(parent) = &node.parent {
        for ancestor in init::directory_chain(db, &s.run.run_id, &s.binding, parent)? {
            ready_token(db, s, &ancestor)?;
        }
    } else {
        init::validate_root(&s.binding, node)?;
    }
    Ok(())
}
fn stamp(db: &Connection, s: &InitializationStatus, node: &Node) -> Result<PreviewStamp> {
    if !node.directory {
        return Err(error(libc::ENOTDIR));
    }
    deny_guard(db, s, init::root_id(&s.binding)?)?;
    if root_invalid(db, s)? {
        return Err(error(libc::EAGAIN));
    }
    let chain = init::directory_chain(db, &s.run.run_id, &s.binding, &node.id)?
        .iter()
        .map(|n| ready_token(db, s, n))
        .collect::<Result<Vec<_>>>()?;
    let members_revision = db
        .query_row(
            "SELECT revision FROM preview_visibility WHERE run=?1 AND id=?2",
            params![s.run.run_id, node.id],
            |r| read_u64(r, 0),
        )
        .optional()?
        .unwrap_or(0);
    Ok(PreviewStamp {
        origin: origin(s)?,
        directory: node.id.clone(),
        members_revision,
        chain,
    })
}
fn mapped_node(db: &Connection, s: &InitializationStatus, ino: u64) -> Result<Node> {
    let row:Option<(String,String,String,String,String)>=db.query_row(
        "SELECT p.run,p.node,p.native,p.reserved,f.node FROM preview_inodes p JOIN fs_inodes f ON f.ino=p.ino WHERE p.ino=?1",
        [sql_number(ino)?],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?))).optional()?;
    let Some((run, id, native, reserved, key)) = row else {
        return Err(error(libc::ENOENT));
    };
    if run != s.run.run_id || reserved != key {
        return Err(error(libc::ESTALE));
    }
    let node = init::load_node(db, &run, &id).map_err(|e| {
        if e.is_not_found() {
            error(libc::ESTALE)
        } else {
            e
        }
    })?;
    if identity(db, s, &node)?.to_string() != native {
        return Err(error(libc::ESTALE));
    }
    Ok(node)
}
fn allocate(db: &Connection, s: &InitializationStatus, node: &Node) -> Result<u64> {
    let native = identity(db, s, node)?.to_string();
    if let Some((ino,reserved,key))=db.query_row(
        "SELECT p.ino,p.reserved,f.node FROM preview_inodes p JOIN fs_inodes f ON f.ino=p.ino WHERE p.run=?1 AND p.node=?2 AND p.native=?3",
        params![s.run.run_id,node.id,native],|r|Ok((read_u64(r,0)?,r.get::<_,String>(1)?,r.get::<_,String>(2)?))).optional()? {
        if reserved!=key {return Err(invalid("reserved inode changed"));} return Ok(ino);
    }
    let reserved = format!("\0drive-native-preview:{}", uuid::Uuid::new_v4());
    let ino = if node.id == init::root_id(&s.binding)? {
        let previous:Option<(String,String)>=db.query_row("SELECT f.node,p.reserved FROM fs_inodes f LEFT JOIN preview_inodes p ON p.ino=f.ino WHERE f.ino=1",[],|r|Ok((r.get(0)?,r.get(1)?))).optional()?;
        if let Some((key, old)) = previous {
            if key != old {
                return Err(invalid("root inode not reserved"));
            }
            db.execute("DELETE FROM preview_inodes WHERE ino=1", [])?;
            db.execute("UPDATE fs_inodes SET node=?1 WHERE ino=1", [&reserved])?;
        } else {
            db.execute("INSERT INTO fs_inodes(ino,node) VALUES(1,?1)", [&reserved])?;
        }
        1
    } else {
        if !db.query_row(
            "SELECT EXISTS(SELECT 1 FROM preview_inodes WHERE ino=1 AND run=?1)",
            [&s.run.run_id],
            |r| r.get::<_, bool>(0),
        )? {
            return Err(error(libc::EAGAIN));
        }
        db.execute("INSERT INTO fs_inodes(node) VALUES(?1)", [&reserved])?;
        u64::try_from(db.last_insert_rowid()).map_err(|_| invalid("inode overflow"))?
    };
    db.execute(
        "INSERT INTO preview_inodes(ino,reserved,run,node,native) VALUES(?1,?2,?3,?4,?5)",
        params![sql_number(ino)?, reserved, s.run.run_id, node.id, native],
    )?;
    Ok(ino)
}
fn record(
    db: &Connection,
    s: &InitializationStatus,
    node: &Node,
    ino: u64,
) -> Result<PreviewRecord> {
    metadata_guard(db, s, node)?;
    metadata_record(s, node, ino)
}
fn metadata_record(s: &InitializationStatus, node: &Node, ino: u64) -> Result<PreviewRecord> {
    let metadata = node
        .metadata
        .as_ref()
        .ok_or_else(|| invalid("metadata missing"))?;
    Ok(PreviewRecord {
        id: node.id.clone(),
        name: node.name.clone(),
        directory: node.directory,
        ino,
        size: metadata.size,
        mtime: metadata.mtime,
        mode: metadata.mode & 0o555,
        origin: origin(s)?,
    })
}
// Parent and all ancestors were already verified by stamp(), in this same
// transaction. Do not repeat the identical chain for every item on a GUI page.
fn direct_child_guard(
    db: &Connection,
    s: &InitializationStatus,
    parent: &str,
    node: &Node,
) -> Result<()> {
    deny_guard(db, s, &node.id)?;
    crate::state::valid_node(node)?;
    if node.parent.as_deref() != Some(parent) {
        return Err(invalid("direct child parent mismatch"));
    }
    identity(db, s, node)?;
    if node.directory && coverage(db, s, node)?.0 == "denied" {
        return Err(error(libc::EACCES));
    }
    Ok(())
}
const CHILDREN: &str = "n.run=?1 AND n.parent=?2 AND NOT EXISTS(SELECT 1 FROM preview_denied d WHERE d.run=n.run AND d.id=n.id)";
fn deny_guard(db: &Connection, s: &InitializationStatus, id: &str) -> Result<()> {
    if db.query_row(
        "SELECT EXISTS(SELECT 1 FROM preview_denied WHERE run=?1 AND id=?2)",
        params![s.run.run_id, id],
        |r| r.get::<_, bool>(0),
    )? {
        return Err(error(libc::EACCES));
    }
    Ok(())
}
fn child_count(db: &Connection, s: &InitializationStatus, id: &str) -> Result<(usize, usize)> {
    let (count,names):(u64,u64)=db.query_row(&format!("SELECT count(*),coalesce(sum(length(CAST(n.name AS BLOB))),0) FROM initialization_entries n WHERE {CHILDREN}"),params![s.run.run_id,id],|r|Ok((read_u64(r,0)?,read_u64(r,1)?)))?;
    Ok((
        usize::try_from(count).map_err(|_| invalid("count overflow"))?,
        usize::try_from(names).map_err(|_| invalid("name bytes overflow"))?,
    ))
}
impl Store {
    fn preview_storage_safe(&self) -> Result<bool> {
        use std::os::unix::{fs::OpenOptionsExt, io::AsRawFd};
        let opened = std::fs::OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_DIRECTORY | libc::O_NOFOLLOW)
            .open(self.root.join("drafts"));
        let directory = match opened {
            Ok(directory) => directory,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(true),
            Err(e) if matches!(e.raw_os_error(), Some(libc::ELOOP) | Some(libc::ENOTDIR)) => {
                return Ok(false);
            }
            Err(e) => return Err(e.into()),
        };
        // Inspect only one entry through our no-follow directory descriptor.
        // A lost DB row must not hide surviving unsent content from cold checks.
        let mut entries = std::fs::read_dir(format!("/proc/self/fd/{}", directory.as_raw_fd()))?;
        Ok(entries.next().transpose()?.is_none())
    }
    fn required_preview_session(&self, db: &Connection) -> Result<InitializationStatus> {
        if !self.preview_storage_safe()? {
            return Err(error(libc::EACCES));
        }
        required_session(db)
    }
    /// Only the authenticated owner may call this on a confirmed root failure.
    /// Reconnect/resume cannot erase it; an actual verified root reply can.
    pub fn revoke_initialization_preview_root(
        &mut self,
        run: &crate::initialization_store::InitializationRun,
    ) -> Result<()> {
        let tx = self.db.transaction()?;
        let s = init::checked(&tx, run, true)?;
        revoke_root_in_transaction(&tx, &s)?;
        tx.commit()?;
        Ok(())
    }
    pub fn preview_available(&self, root: &str) -> Result<bool> {
        if !self.preview_storage_safe()? {
            return Ok(false);
        }
        let Some(s) = session(&self.db)? else {
            return Ok(false);
        };
        if init::root_id(&s.binding)? != root {
            return Ok(false);
        }
        deny_guard(&self.db, &s, root)?;
        if root_invalid(&self.db, &s)? {
            return Ok(false);
        }
        let node = init::load_node(&self.db, &s.run.run_id, root)?;
        metadata_guard(&self.db, &s, &node)?;
        check_mount_binding(&self.db, &s)?;
        Ok(true)
    }
    pub fn preview_mount_root(&self, root: &str) -> Result<Option<PreviewRecord>> {
        if !self.preview_storage_safe()? {
            return Ok(None);
        }
        let tx = self.db.unchecked_transaction()?;
        let Some(s) = session(&tx)? else {
            return Ok(None);
        };
        if init::root_id(&s.binding)? != root {
            return Ok(None);
        }
        deny_guard(&tx, &s, root)?;
        if root_invalid(&tx, &s)? {
            return Ok(None);
        }
        let node = init::load_node(&tx, &s.run.run_id, root)?;
        metadata_guard(&tx, &s, &node)?;
        check_mount_binding(&tx, &s)?;
        if !tx.query_row("SELECT EXISTS(SELECT 1 FROM preview_mount)", [], |r| {
            r.get::<_, bool>(0)
        })? {
            tx.execute("INSERT INTO preview_mount(slot,task,scope,root,allocated,bytes) SELECT 1,?1,?2,?3,count(*),coalesce(sum(length(CAST(node AS BLOB))+length(CAST(reserved AS BLOB))+length(CAST(run AS BLOB))+length(CAST(native AS BLOB))+64),0) FROM preview_inodes",params![s.binding.task_id,serde_json::to_string(&s.binding.scope)?,root])?;
        }
        let ino = allocate(&tx, &s, &node)?;
        let record = record(&tx, &s, &node, ino)?;
        tx.commit()?;
        Ok(Some(record))
    }
    pub fn preview_record_by_inode(&self, ino: u64) -> Result<PreviewRecord> {
        if !self.db.query_row(
            "SELECT EXISTS(SELECT 1 FROM preview_inodes WHERE ino=?1)",
            [sql_number(ino)?],
            |r| r.get::<_, bool>(0),
        )? {
            return Err(error(libc::ENOENT));
        }
        let s = self.required_preview_session(&self.db)?;
        if ino == 1 {
            // The FSD can outlive a failed candidate run. Root rebinding must
            // work without a GUI or another mount attempt; old stamps retain
            // their origin and are rejected separately.
            let root = init::root_id(&s.binding)?.to_owned();
            return self
                .preview_mount_root(&root)?
                .ok_or_else(|| error(libc::ESTALE));
        }
        let node = mapped_node(&self.db, &s, ino)?;
        record(&self.db, &s, &node, ino)
    }
    pub fn preview_lookup(&self, parent_ino: u64, name: &str) -> Result<PreviewRecord> {
        if name.is_empty()
            || matches!(name, "." | "..")
            || name.contains('/')
            || name.contains('\0')
        {
            return Err(error(libc::ENOENT));
        }
        let tx = self.db.unchecked_transaction()?;
        let s = self.required_preview_session(&tx)?;
        let parent = mapped_node(&tx, &s, parent_ino)?;
        stamp(&tx, &s, &parent)?;
        let body: Option<String> = tx
            .query_row(
                "SELECT body FROM initialization_entries WHERE run=?1 AND parent=?2 AND name=?3",
                params![s.run.run_id, parent.id, name],
                |r| r.get(0),
            )
            .optional()?;
        let node: Node = serde_json::from_str(&body.ok_or_else(|| error(libc::ENOENT))?)?;
        metadata_guard(&tx, &s, &node)?;
        let ino = allocate(&tx, &s, &node)?;
        let record = record(&tx, &s, &node, ino)?;
        tx.commit()?;
        Ok(record)
    }
    pub fn preview_children_page(
        &self,
        parent_ino: u64,
        offset: usize,
        limit: usize,
    ) -> Result<PreviewPage> {
        if limit == 0 || limit > 256 {
            return Err(error(libc::EINVAL));
        }
        let tx = self.db.unchecked_transaction()?;
        let s = self.required_preview_session(&tx)?;
        let parent = mapped_node(&tx, &s, parent_ino)?;
        let stamp = stamp(&tx, &s, &parent)?;
        let (total, _) = child_count(&tx, &s, &parent.id)?;
        let bodies=tx.prepare(&format!("SELECT n.body FROM initialization_entries n WHERE {CHILDREN} ORDER BY n.name,n.id LIMIT ?3 OFFSET ?4"))?
            .query_map(params![s.run.run_id,parent.id,limit as i64,i64::try_from(offset).map_err(|_|error(libc::EINVAL))?],|r|r.get::<_,String>(0))?
            .collect::<std::result::Result<Vec<_>,_>>()?;
        let entries = bodies
            .iter()
            .map(|body| {
                let node: Node = serde_json::from_str(body)?;
                direct_child_guard(&tx, &s, &parent.id, &node)?;
                let ino = allocate(&tx, &s, &node)?;
                metadata_record(&s, &node, ino)
            })
            .collect::<Result<Vec<_>>>()?;
        tx.commit()?;
        Ok(PreviewPage {
            entries,
            total,
            stamp,
        })
    }
    pub fn preview_directory_snapshot(
        &self,
        ino: u64,
        max_bytes: usize,
    ) -> Result<PreviewDirectorySnapshot> {
        let tx = self.db.unchecked_transaction()?;
        let s = self.required_preview_session(&tx)?;
        let node = mapped_node(&tx, &s, ino)?;
        let stamp = stamp(&tx, &s, &node)?;
        let (count, names) = child_count(&tx, &s, &node.id)?;
        let count = count
            .checked_add(2)
            .ok_or_else(|| invalid("count overflow"))?;
        let bytes = count
            .checked_mul(std::mem::size_of::<DirectoryEntry>())
            .and_then(|n| n.checked_add(names))
            .and_then(|n| n.checked_add(3))
            .ok_or_else(|| invalid("view bytes overflow"))?;
        if bytes > max_bytes {
            return Err(error(libc::ENOMEM));
        }
        let parent_ino = if let Some(parent) = &node.parent {
            allocate(&tx, &s, &init::load_node(&tx, &s.run.run_id, parent)?)?
        } else {
            1
        };
        let mut entries = Vec::with_capacity(count);
        entries.push(DirectoryEntry {
            ino,
            directory: true,
            name: ".".into(),
        });
        entries.push(DirectoryEntry {
            ino: parent_ino,
            directory: true,
            name: "..".into(),
        });
        {
            let mut query = tx.prepare(&format!(
                "SELECT n.body FROM initialization_entries n WHERE {CHILDREN} ORDER BY n.name,n.id"
            ))?;
            let mut rows = query.query(params![s.run.run_id, node.id])?;
            while let Some(row) = rows.next()? {
                let body: String = row.get(0)?;
                let child: Node = serde_json::from_str(&body)?;
                direct_child_guard(&tx, &s, &node.id, &child)?;
                entries.push(DirectoryEntry {
                    ino: allocate(&tx, &s, &child)?,
                    directory: child.directory,
                    name: child.name,
                });
            }
        }
        let actual = entries.capacity() * std::mem::size_of::<DirectoryEntry>()
            + entries.iter().map(|e| e.name.capacity()).sum::<usize>();
        if actual > max_bytes {
            return Err(error(libc::ENOMEM));
        }
        tx.commit()?;
        Ok(PreviewDirectorySnapshot {
            snapshot: DirectorySnapshot {
                entries,
                bytes: actual,
            },
            stamp,
        })
    }
    pub fn preview_validate_view(&self, expected: &PreviewStamp) -> Result<()> {
        let s = self.required_preview_session(&self.db)?;
        if origin(&s)? != expected.origin {
            return Err(error(libc::ESTALE));
        }
        let node = init::load_node(&self.db, &s.run.run_id, &expected.directory).map_err(|e| {
            if e.is_not_found() {
                error(libc::ESTALE)
            } else {
                e
            }
        })?;
        let actual = stamp(&self.db, &s, &node).map_err(|e| match &e {
            Error::Io(io) if io.raw_os_error() == Some(libc::EAGAIN) => error(libc::ESTALE),
            _ => e,
        })?;
        if actual != *expected {
            return Err(error(libc::ESTALE));
        }
        Ok(())
    }
}
fn check_mount_binding(db: &Connection, s: &InitializationStatus) -> Result<()> {
    let old: Option<(String, String, String)> = db
        .query_row(
            "SELECT task,scope,root FROM preview_mount WHERE slot=1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .optional()?;
    if let Some((task, scope, root)) = old {
        if task != s.binding.task_id
            || serde_json::from_str::<serde_json::Value>(&scope)? != s.binding.scope
            || root != init::root_id(&s.binding)?
        {
            return Err(invalid("mount binding changed"));
        }
    }
    Ok(())
}

/// Invoked inside the existing final nodes/cursor/ready transaction. Never
/// promotes stale reservations for removed/replaced identities.
pub(crate) fn promote_in_transaction(db: &Connection, s: &InitializationStatus) -> Result<()> {
    if !db.query_row("SELECT EXISTS(SELECT 1 FROM preview_mount)", [], |r| {
        r.get::<_, bool>(0)
    })? {
        return Ok(());
    }
    check_mount_binding(db, s)?;
    // Completion must also succeed if no reader visited root after a candidate
    // restart. This is still inside the atomic, fully validated final install.
    let root_node = init::load_node(db, &s.run.run_id, init::root_id(&s.binding)?)?;
    allocate(db, s, &root_node)?;
    let mappings = db
        .prepare("SELECT ino,reserved,node,native FROM preview_inodes WHERE run=?1 ORDER BY ino")?
        .query_map([&s.run.run_id], |r| {
            Ok((
                read_u64(r, 0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
            ))
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    let mut root = false;
    for (ino, reserved, id, native) in mappings {
        let node = match init::load_node(db, &s.run.run_id, &id) {
            Ok(n) => n,
            Err(e) if e.is_not_found() => continue,
            Err(e) => return Err(e),
        };
        if identity(db, s, &node)?.to_string() != native {
            continue;
        }
        let key: String = db.query_row(
            "SELECT node FROM fs_inodes WHERE ino=?1",
            [sql_number(ino)?],
            |r| r.get(0),
        )?;
        if key != reserved {
            return Err(invalid("promotion inode reservation changed"));
        }
        if !db.query_row(
            "SELECT EXISTS(SELECT 1 FROM nodes WHERE id=?1 AND deleted=0)",
            [&id],
            |r| r.get::<_, bool>(0),
        )? {
            return Err(invalid("promotion target absent"));
        }
        db.execute(
            "UPDATE fs_inodes SET node=?2 WHERE ino=?1",
            params![sql_number(ino)?, id],
        )?;
        db.execute(
            "DELETE FROM preview_inodes WHERE ino=?1",
            [sql_number(ino)?],
        )?;
        if ino == 1 {
            root = true;
        }
    }
    if !root {
        return Err(invalid("preview root was not promoted"));
    }
    // Retire stale reservations after a successful promotion. AUTOINCREMENT
    // keeps their numbers from being reused; no content or live Node is removed.
    let stale = db
        .prepare(
            "SELECT p.ino,p.reserved,f.node FROM preview_inodes p JOIN fs_inodes f ON f.ino=p.ino",
        )?
        .query_map([], |r| {
            Ok((
                read_u64(r, 0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
            ))
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    for (ino, reserved, key) in stale {
        if reserved != key {
            return Err(invalid("stale reservation changed"));
        }
        db.execute(
            "DELETE FROM preview_inodes WHERE ino=?1",
            [sql_number(ino)?],
        )?;
        db.execute(
            "DELETE FROM fs_inodes WHERE ino=?1 AND node=?2",
            params![sql_number(ino)?, reserved],
        )?;
    }
    db.execute("DELETE FROM preview_mount", [])?;
    Ok(())
}

pub(crate) fn verified_root_in_transaction(
    db: &Connection,
    run: &crate::initialization_store::InitializationRun,
) -> Result<()> {
    let s = init::checked(db, run, true)?;
    db.execute(
        "DELETE FROM preview_denied WHERE run=?1 AND id=?2",
        params![run.run_id, init::root_id(&s.binding)?],
    )?;
    db.execute(
        "UPDATE preview_visibility SET root_invalid=0 WHERE run=?1 AND id=?2",
        params![run.run_id, init::root_id(&s.binding)?],
    )?;
    Ok(())
}

fn root_invalid(db: &Connection, s: &InitializationStatus) -> Result<bool> {
    Ok(db
        .query_row(
            "SELECT root_invalid FROM preview_visibility WHERE run=?1 AND id=?2",
            params![s.run.run_id, init::root_id(&s.binding)?],
            |r| r.get(0),
        )
        .optional()?
        .unwrap_or(false))
}
/// An observed root change invalidates the prior proof. Merely requesting a
/// refresh does not: root_refresh still blocks the final commit in either case.
pub(crate) fn invalidate_root_in_transaction(
    db: &Connection,
    s: &InitializationStatus,
) -> Result<()> {
    db.execute("INSERT INTO preview_visibility(run,id,revision,root_invalid) VALUES(?1,?2,0,1) ON CONFLICT(run,id) DO UPDATE SET root_invalid=1",params![s.run.run_id,init::root_id(&s.binding)?])?;
    Ok(())
}

pub(crate) fn revoke_root_in_transaction(db: &Connection, s: &InitializationStatus) -> Result<()> {
    db.execute(
        "INSERT OR IGNORE INTO preview_denied(run,id) VALUES(?1,?2)",
        params![s.run.run_id, init::root_id(&s.binding)?],
    )?;
    db.execute("UPDATE initialization_run SET root_refresh=1,root_ticket=NULL,fenced=NULL,fence_ticket=NULL WHERE run_id=?1",[&s.run.run_id])?;
    Ok(())
}

#[cfg(all(test, feature = "native"))]
#[path = "preview_tests.rs"]
mod tests;
