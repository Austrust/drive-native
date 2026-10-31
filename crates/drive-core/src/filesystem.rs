//! Local filesystem metadata. No method in this module performs network I/O.
use crate::{Error, Node, NodeMetadata, Result, Status, Store};
use rusqlite::{OptionalExtension, params};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct FileRecord {
    pub node: Node,
    pub ino: u64,
    pub size: u64,
    pub mtime: u64,
    pub mode: u32,
    pub status: Status,
    pub unlinked: bool,
}
// Shared with the desktop count/page projection; preserve ACL and local creation precedence.
const VISIBLE_CHILDREN: &str = "n.id NOT IN (SELECT id FROM access_blocked) AND n.parent=?1 AND n.deleted=0 AND NOT EXISTS(SELECT 1 FROM nodes l JOIN local_creations c ON c.node=l.id WHERE l.parent=n.parent AND l.name=n.name AND l.id!=n.id AND l.deleted=0)";

/// Directory enumeration never needs body state, hashes or per-file attributes.
#[derive(Debug, PartialEq, Eq)]
pub struct DirectoryEntry {
    pub ino: u64,
    pub directory: bool,
    pub name: String,
}
#[derive(Debug)]
pub struct DirectorySnapshot {
    pub entries: Vec<DirectoryEntry>,
    /// Owned allocation estimate (entry capacity and UTF-8 name capacity).
    pub bytes: usize,
}

impl Store {
    /// Build one stable directory view in a single transaction. Inodes are
    /// allocated in one INSERT SELECT, without constructing FileRecords or
    /// touching cached bodies/drafts. The byte limit is checked before allocation.
    pub fn directory_snapshot(&self, ino: u64, max_bytes: usize) -> Result<DirectorySnapshot> {
        let tx = self.db.unchecked_transaction()?;
        let id: String = tx.query_row(
            "SELECT node FROM fs_inodes WHERE ino=?1",
            [sql_unsigned(ino)?],
            |r| r.get(0),
        )?;
        let node = self.node(&id)?;
        if !node.directory {
            return Err(Error::Io(std::io::Error::from_raw_os_error(libc::ENOTDIR)));
        }
        let sql = format!(
            "SELECT count(*),coalesce(sum(length(CAST(n.name AS BLOB))),0) FROM nodes n WHERE {VISIBLE_CHILDREN}"
        );
        let (count, names) = tx.query_row(&sql, [&id], |r| {
            Ok((read_unsigned(r, 0)?, read_unsigned(r, 1)?))
        })?;
        let count =
            usize::try_from(count).map_err(|_| Error::State("directory size overflow".into()))?;
        let names =
            usize::try_from(names).map_err(|_| Error::State("directory size overflow".into()))?;
        let count = count
            .checked_add(2)
            .ok_or_else(|| Error::State("directory size overflow".into()))?;
        let bytes = count
            .checked_mul(std::mem::size_of::<DirectoryEntry>())
            .and_then(|n| n.checked_add(names))
            .and_then(|n| n.checked_add(3))
            .ok_or_else(|| Error::State("directory size overflow".into()))?;
        if bytes > max_bytes {
            return Err(Error::Io(std::io::Error::from_raw_os_error(libc::ENOMEM)));
        }
        let parent_ino = match node.parent.as_deref() {
            None => 1,
            Some(parent) => {
                self.node(parent)?;
                tx.execute("INSERT INTO fs_inodes(node) SELECT ?1 WHERE NOT EXISTS(SELECT 1 FROM fs_inodes WHERE node=?1)", [parent])?;
                tx.query_row("SELECT ino FROM fs_inodes WHERE node=?1", [parent], |r| {
                    read_unsigned(r, 0)
                })?
            }
        };
        let insert = format!(
            "INSERT INTO fs_inodes(node) SELECT n.id FROM nodes n WHERE {VISIBLE_CHILDREN} AND NOT EXISTS(SELECT 1 FROM fs_inodes i WHERE i.node=n.id) ORDER BY n.name,n.id"
        );
        tx.execute(&insert, [&id])?;
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
        let select = format!(
            "SELECT i.ino,n.directory,n.name FROM nodes n JOIN fs_inodes i ON i.node=n.id WHERE {VISIBLE_CHILDREN} ORDER BY n.name,n.id"
        );
        {
            let mut query = tx.prepare(&select)?;
            let rows = query.query_map([&id], |r| {
                Ok(DirectoryEntry {
                    ino: read_unsigned(r, 0)?,
                    directory: r.get(1)?,
                    name: r.get(2)?,
                })
            })?;
            for row in rows {
                entries.push(row?);
            }
        }
        let actual = entries.capacity() * std::mem::size_of::<DirectoryEntry>()
            + entries.iter().map(|e| e.name.capacity()).sum::<usize>();
        if actual > max_bytes {
            return Err(Error::Io(std::io::Error::from_raw_os_error(libc::ENOMEM)));
        }
        tx.commit()?;
        Ok(DirectorySnapshot {
            entries,
            bytes: actual,
        })
    }
    /// Local change token for coalescing desktop notifications. This is not a
    /// server cursor; false-positive notifications after a rollback are safe.
    pub fn local_revision(&self) -> u64 {
        self.db.total_changes()
    }
    pub fn pending_namespace_operations(&self) -> Result<i64> {
        Ok(self.db.query_row("SELECT count(*) FROM namespace_ops WHERE state NOT IN ('committed','superseded','conflict')",[],|r|r.get(0))?)
    }

    /// Directory containing durable local contents; space is local, not NAS quota.
    pub fn storage_directory(&self) -> &std::path::Path {
        &self.root
    }
    /// POSIX permissions belong to this Linux inode. They are not NAS ACL writes.
    /// An independent table prevents remote content updates resetting chmod.
    pub fn set_local_mode(&mut self, id: &str, mode: u32) -> Result<()> {
        if mode & !0o777 != 0 {
            return Err(Error::Io(std::io::Error::from_raw_os_error(libc::EINVAL)));
        }
        self.node_for_handle(id)?;
        self.db.execute("INSERT INTO local_modes(node,mode) VALUES(?1,?2) ON CONFLICT(node) DO UPDATE SET mode=excluded.mode",params![id,mode])?;
        Ok(())
    }

    pub fn ensure_fs_root(&self, id: &str) -> Result<()> {
        let root = self.node(id)?;
        if root.parent.is_some() || !root.directory {
            return Err(Error::State(
                "mount root is not the indexed directory root".into(),
            ));
        }
        let existing: Option<String> = self
            .db
            .query_row("SELECT node FROM fs_inodes WHERE ino=1", [], |r| r.get(0))
            .optional()?;
        if existing.as_deref().is_some_and(|n| n != id) {
            return Err(Error::State("mount root identity changed".into()));
        }
        if existing.is_none() {
            self.db
                .execute("INSERT INTO fs_inodes(ino,node) VALUES(1,?1)", [id])?;
        }
        let actual: i64 =
            self.db
                .query_row("SELECT ino FROM fs_inodes WHERE node=?1", [id], |r| {
                    r.get(0)
                })?;
        if actual != 1 {
            return Err(Error::State("root inode was not initialized first".into()));
        }
        Ok(())
    }
    pub fn metadata(&self, id: &str) -> Result<Option<NodeMetadata>> {
        Ok(self
            .db
            .query_row(
                "SELECT size,mtime,mode FROM node_metadata WHERE node=?1",
                [id],
                |r| {
                    Ok(NodeMetadata {
                        size: read_unsigned(r, 0)?,
                        mtime: read_unsigned(r, 1)?,
                        mode: r.get(2)?,
                    })
                },
            )
            .optional()?)
    }
    pub fn record(&self, id: &str) -> Result<FileRecord> {
        self.record_impl(id, false)
    }
    pub fn record_for_handle(&self, id: &str) -> Result<FileRecord> {
        self.record_impl(id, true)
    }
    fn record_impl(&self, id: &str, orphan: bool) -> Result<FileRecord> {
        let node = if orphan {
            self.node_for_handle(id)?
        } else {
            self.node(id)?
        };
        let initialized: bool = self.db.query_row(
            "SELECT EXISTS(SELECT 1 FROM fs_inodes WHERE ino=1)",
            [],
            |r| r.get(0),
        )?;
        if !initialized {
            return Err(Error::State("filesystem root not initialized".into()));
        }
        // INSERT OR IGNORE still advances AUTOINCREMENT and fsyncs the WAL.
        // Existing inode queries must remain read-only, including getattr.
        let existing = self
            .db
            .query_row("SELECT ino FROM fs_inodes WHERE node=?1", [id], |r| {
                read_unsigned(r, 0)
            })
            .optional()?;
        let ino = match existing {
            Some(ino) => ino,
            None => {
                self.db
                    .execute("INSERT INTO fs_inodes(node) VALUES(?1)", [id])?;
                u64::try_from(self.db.last_insert_rowid())
                    .map_err(|_| Error::State("invalid inode".into()))?
            }
        };
        let meta = node.metadata.clone().unwrap_or(NodeMetadata {
            size: 0,
            mtime: 0,
            mode: if node.directory { 0o755 } else { 0o644 },
        });
        let blob: Option<String> = self.db.query_row(
            "SELECT COALESCE(dirty_blob,blob) FROM nodes WHERE id=?1",
            [id],
            |r| r.get(0),
        )?;
        if blob.is_none() && node.metadata.is_none() && !node.directory {
            return Err(Error::State(
                "file metadata missing; native snapshot recovery required".into(),
            ));
        }
        let mut size = meta.size;
        let mut mtime = meta.mtime;
        if let Some(blob) = blob {
            let stat = std::fs::metadata(self.root.join("blobs").join(blob))?;
            size = stat.len();
            if self.status(id)? == Status::PendingUpload {
                mtime = stat
                    .modified()?
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
            }
        }
        if let Some(draft) = self.open_dirty_draft(id)? {
            size = draft.size()?;
            mtime = draft.mtime()?;
        }
        let mode = self
            .db
            .query_row("SELECT mode FROM local_modes WHERE node=?1", [id], |r| {
                r.get::<_, u32>(0)
            })
            .optional()?
            .unwrap_or(meta.mode & 0o777);
        Ok(FileRecord {
            node,
            ino,
            size,
            mtime,
            mode,
            status: self.status(id)?,
            unlinked: self.locally_unlinked(id)?,
        })
    }
    pub fn record_by_inode(&self, ino: u64) -> Result<FileRecord> {
        let id: String = self.db.query_row(
            "SELECT node FROM fs_inodes WHERE ino=?1",
            [sql_unsigned(ino)?],
            |r| r.get(0),
        )?;
        self.record_for_handle(&id)
    }
    pub fn children(&self, parent: &str) -> Result<Vec<FileRecord>> {
        if !self.node(parent)?.directory {
            return Err(Error::State("not a directory".into()));
        }
        let mut q = self
            .db
            .prepare("SELECT n.id FROM nodes n WHERE n.id NOT IN (SELECT id FROM access_blocked) AND n.parent=?1 AND n.deleted=0 AND NOT EXISTS(SELECT 1 FROM nodes l JOIN local_creations c ON c.node=l.id WHERE l.parent=n.parent AND l.name=n.name AND l.id!=n.id AND l.deleted=0) ORDER BY n.name,n.id")?;
        let ids = q
            .query_map([parent], |r| r.get::<_, String>(0))?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        let tx = self.db.unchecked_transaction()?;
        let records = ids
            .iter()
            .map(|id| self.record(id))
            .collect::<Result<Vec<_>>>()?;
        tx.commit()?;
        Ok(records)
    }
    /// Desktop aggregation needs node metadata, never inode allocation or body handles.
    pub fn child_nodes(&self, parent: &str) -> Result<Vec<Node>> {
        self.child_nodes_page(parent, 0, usize::MAX)
    }
    /// Local metadata only: paginate before constructing nodes. No inode or body IO.
    /// Pages reflect the current store state, not a stable cross-request snapshot.
    pub fn child_nodes_page(&self, parent: &str, offset: usize, limit: usize) -> Result<Vec<Node>> {
        if !self.node(parent)?.directory {
            return Err(Error::State("not a directory".into()));
        }
        let sql = format!(
            "SELECT n.id FROM nodes n WHERE {VISIBLE_CHILDREN} ORDER BY n.name,n.id LIMIT ?2 OFFSET ?3"
        );
        let mut q = self.db.prepare(&sql)?;
        let ids = q
            .query_map(
                params![
                    parent,
                    i64::try_from(limit).unwrap_or(i64::MAX),
                    i64::try_from(offset).unwrap_or(i64::MAX)
                ],
                |r| r.get::<_, String>(0),
            )?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        ids.iter().map(|id| self.node(id)).collect()
    }
    /// Count the same visible set used by child_nodes_page, including local shadows.
    pub fn child_node_count(&self, parent: &str) -> Result<usize> {
        if !self.node(parent)?.directory {
            return Err(Error::State("not a directory".into()));
        }
        let sql = format!("SELECT count(*) FROM nodes n WHERE {VISIBLE_CHILDREN}");
        let count: i64 = self.db.query_row(&sql, [parent], |r| r.get(0))?;
        usize::try_from(count).map_err(|_| Error::State("invalid child count".into()))
    }
    pub fn lookup(&self, parent: &str, name: &str) -> Result<FileRecord> {
        self.record(&self.lookup_node(parent, name)?.id)
    }
    pub fn lookup_node(&self, parent: &str, name: &str) -> Result<Node> {
        let mut q = self
            .db
            .prepare("SELECT n.id FROM nodes n WHERE n.id NOT IN (SELECT id FROM access_blocked) AND n.parent=?1 AND n.name=?2 AND n.deleted=0 AND NOT EXISTS(SELECT 1 FROM nodes l JOIN local_creations c ON c.node=l.id WHERE l.parent=n.parent AND l.name=n.name AND l.id!=n.id AND l.deleted=0)")?;
        let ids = q
            .query_map(params![parent, name], |r| r.get::<_, String>(0))?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        if ids.len() > 1 {
            return Err(Error::State(
                "ambiguous namespace; recovery required".into(),
            ));
        }
        let id = ids.first().ok_or(rusqlite::Error::QueryReturnedNoRows)?;
        self.node(id)
    }
}

pub(crate) fn write_metadata(tx: &rusqlite::Transaction<'_>, n: &Node) -> Result<()> {
    if let Some(m) = &n.metadata {
        if m.mode & !0o777 != 0 {
            return Err(Error::State("unsupported filesystem mode bits".into()));
        }
        tx.execute("INSERT INTO node_metadata(node,size,mtime,mode) VALUES(?1,?2,?3,?4) ON CONFLICT(node) DO UPDATE SET size=excluded.size,mtime=excluded.mtime,mode=excluded.mode",params![n.id,sql_unsigned(m.size)?,sql_unsigned(m.mtime)?,m.mode])?;
    } else {
        tx.execute("DELETE FROM node_metadata WHERE node=?1", [&n.id])?;
    }
    Ok(())
}

fn read_unsigned(row: &rusqlite::Row<'_>, column: usize) -> rusqlite::Result<u64> {
    let value: i64 = row.get(column)?;
    value
        .try_into()
        .map_err(|_| rusqlite::Error::IntegralValueOutOfRange(column, value))
}
fn sql_unsigned(value: u64) -> Result<i64> {
    value
        .try_into()
        .map_err(|_| Error::State("filesystem integer exceeds SQLite signed range".into()))
}
