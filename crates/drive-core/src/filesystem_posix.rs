//! POSIX attributes without desktop sync-state, pin ancestry or content opens.
//! The caller serializes Store access. Dirty bodies still contribute actual
//! local size/time; this projection never hydrates content or changes queues.
//!
//! Both hot projections are single prepared statements:
//! * `posix_record_by_inode` (getattr) resolves inode, namespace visibility,
//!   body/draft state, local mode and local atime in one round trip.
//! * `posix_lookup_by_inode` (lookup) additionally resolves the parent
//!   directory and the child name in one round trip, with the ambiguity check
//!   folded into the same statement.
//! The earlier implementation built the complete parent projection first and
//! then repeated the child attribute join, executing up to sixteen statements
//! and two recursive accessibility checks per lookup.
use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PosixRecord {
    pub id: String,
    pub ino: u64,
    pub directory: bool,
    pub size: u64,
    pub mtime: u64,
    pub atime: u64,
    pub atime_nsec: u32,
    pub mode: u32,
    pub unlinked: bool,
    pub blocks: u64,
    /// Safe to cache in the kernel for a bounded TTL: the attributes do not
    /// depend on a mutable local body, an open draft, or an unlinked identity.
    pub cacheable: bool,
}

/// True when the filesystem root marker exists (the mount was initialized).
const ROOT_INITIALIZED: &str = "EXISTS(SELECT 1 FROM fs_inodes WHERE ino=1)";
/// The same predicate as a standalone statement.
const ROOT_INITIALIZED_QUERY: &str = "SELECT EXISTS(SELECT 1 FROM fs_inodes WHERE ino=1)";

/// `id` is not inside an access-denied subtree. Mirrors `Store::is_inaccessible`
/// exactly, including the namespace-overlay remote parent edge, but keeps the
/// empty-denial fast path so a mount without ACL denials pays only the
/// uncorrelated `EXISTS` probe.
fn not_inaccessible(id: &str) -> String {
    format!(
        "(NOT EXISTS(SELECT 1 FROM access_denied) OR NOT (WITH RECURSIVE ancestors(id) AS (
            SELECT {id}.id
            UNION SELECT n.parent FROM nodes n JOIN ancestors a ON n.id=a.id WHERE n.parent IS NOT NULL
            UNION SELECT json_extract(o.remote,'$.parent') FROM namespace_overlay o JOIN ancestors a ON o.node=a.id WHERE json_extract(o.remote,'$.parent') IS NOT NULL)
        SELECT EXISTS(SELECT 1 FROM ancestors a JOIN access_denied d ON d.node=a.id)))"
    )
}

/// All attributes and identity needed by `PosixRecord`, joined from one node
/// alias. `alias` must expose `id`.
fn record_columns() -> &'static str {
    "n.id,n.directory,m.size,m.mtime,COALESCE(l.mode,m.mode),COALESCE(n.dirty_blob,n.blob),n.dirty_blob IS NOT NULL,d.token,u.node IS NOT NULL,a.seconds,a.nanoseconds,i.ino"
}
fn record_joins() -> &'static str {
    "LEFT JOIN node_metadata m ON m.node=n.id
     LEFT JOIN local_modes l ON l.node=n.id
     LEFT JOIN drafts d ON d.node=n.id AND d.dirty=1
     LEFT JOIN local_unlinks u ON u.node=n.id
     LEFT JOIN local_atimes a ON a.node=n.id"
}

fn getattr_sql() -> &'static str {
    static SQL: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    SQL.get_or_init(|| {
        format!(
            "SELECT {columns} FROM fs_inodes i
             JOIN nodes n ON n.id=i.node
             {joins}
             WHERE i.ino=?1
               AND (n.deleted=0 OR u.node IS NOT NULL)
               AND {visible}
               AND {ROOT_INITIALIZED}",
            columns = record_columns(),
            joins = record_joins(),
            visible = not_inaccessible("n"),
        )
    })
}

fn lookup_sql() -> &'static str {
    static SQL: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    SQL.get_or_init(|| {
        // The left row is the parent directory (one row: `fs_inodes.node` is
        // unique); the right side is at most the visible children with this
        // name. Zero rows therefore means "parent not visible", one row means
        // "parent visible, child present or absent", two rows mean ambiguous.
        format!(
            "SELECT p.directory,{columns} FROM fs_inodes f
             JOIN nodes p ON p.id=f.node
             LEFT JOIN nodes n ON n.parent=p.id AND n.name=?2 AND n.deleted=0
               AND (NOT EXISTS(SELECT 1 FROM access_denied) OR n.id NOT IN (SELECT id FROM access_blocked))
               AND {child_visible}
               AND NOT EXISTS(SELECT 1 FROM nodes s JOIN local_creations c ON c.node=s.id
                              WHERE s.parent=n.parent AND s.name=n.name AND s.id!=n.id AND s.deleted=0)
             {joins}
             LEFT JOIN fs_inodes i ON i.node=n.id
             WHERE f.ino=?1
               AND (p.deleted=0 OR EXISTS(SELECT 1 FROM local_unlinks WHERE node=p.id))
               AND {parent_visible}
               AND {ROOT_INITIALIZED}",
            columns = record_columns(),
            joins = record_joins(),
            child_visible = not_inaccessible("n"),
            parent_visible = not_inaccessible("p"),
        )
    })
}

impl Store {
    /// The legacy parent-id entry point keeps its exact lookup contract; the
    /// FUSE hot path uses `posix_lookup_by_inode` and skips this resolution.
    pub fn posix_lookup(&self, parent: &str, name: &str) -> Result<PosixRecord> {
        let mapped: Option<u64> = self
            .db
            .prepare_cached("SELECT ino FROM fs_inodes WHERE node=?1")?
            .query_row([parent], |r| read_unsigned(r, 0))
            .optional()?;
        let ino = match mapped {
            Some(ino) => ino,
            None => {
                // A node created after install has no derived identity yet. The
                // node lookup keeps the original visibility/ACL errors.
                self.node(parent)?;
                self.db
                    .execute("INSERT INTO fs_inodes(node) VALUES(?1)", [parent])?;
                u64::try_from(self.db.last_insert_rowid())
                    .map_err(|_| Error::State("invalid inode".into()))?
            }
        };
        self.posix_lookup_by_inode(ino, name)
    }

    /// One statement: parent directory visibility, child name resolution and
    /// the complete child attribute projection.
    pub fn posix_lookup_by_inode(&self, parent_ino: u64, name: &str) -> Result<PosixRecord> {
        let mut query = self.db.prepare_cached(lookup_sql())?;
        let mut rows = query.query(params![sql_unsigned(parent_ino)?, name])?;
        let Some(row) = rows.next()? else {
            return Err(self.missing_projection(parent_ino)?);
        };
        let parent_directory: bool = row.get(0)?;
        if !parent_directory {
            return Err(Error::Io(std::io::Error::from_raw_os_error(libc::ENOTDIR)));
        }
        let child = projection(row, 1)?;
        if rows.next()?.is_some() {
            return Err(Error::State(
                "ambiguous namespace; recovery required".into(),
            ));
        }
        let Some(child) = child else {
            return Err(rusqlite::Error::QueryReturnedNoRows.into());
        };
        child.finish(self)
    }

    pub fn posix_record_by_inode(&self, ino: u64) -> Result<PosixRecord> {
        let mut query = self.db.prepare_cached(getattr_sql())?;
        match query
            .query_row([sql_unsigned(ino)?], |r| projection(r, 0))
            .optional()?
        {
            Some(Some(record)) => record.finish(self),
            Some(None) => Err(self.missing_projection(ino)?),
            None => Err(self.missing_projection(ino)?),
        }
    }

    /// Missing-mapping error taxonomy: an unknown inode is not found, but a
    /// mapped node in a store whose root marker is gone is a state failure.
    fn missing_projection(&self, ino: u64) -> Result<Error> {
        let initialized: bool = self
            .db
            .prepare_cached(ROOT_INITIALIZED_QUERY)?
            .query_row([], |r| r.get(0))?;
        if !initialized {
            let mapped: bool = self
                .db
                .prepare_cached("SELECT EXISTS(SELECT 1 FROM fs_inodes WHERE ino=?1)")?
                .query_row([sql_unsigned(ino)?], |r| r.get(0))?;
            if mapped {
                return Ok(Error::State("filesystem root not initialized".into()));
            }
        }
        Ok(rusqlite::Error::QueryReturnedNoRows.into())
    }
}

/// Raw attribute columns, still unvalidated. `None` means the left join found
/// no visible child (lookup-only).
struct Projection {
    id: String,
    directory: bool,
    size: Option<i64>,
    mtime: Option<i64>,
    mode: Option<u32>,
    blob: Option<String>,
    dirty: bool,
    draft: Option<String>,
    unlinked: bool,
    atime: Option<i64>,
    atime_nsec: Option<i64>,
    ino: Option<u64>,
}

fn projection(row: &rusqlite::Row<'_>, base: usize) -> rusqlite::Result<Option<Projection>> {
    let id: Option<String> = row.get(base)?;
    let Some(id) = id else {
        return Ok(None);
    };
    Ok(Some(Projection {
        id,
        directory: row.get(base + 1)?,
        size: row.get(base + 2)?,
        mtime: row.get(base + 3)?,
        mode: row.get(base + 4)?,
        blob: row.get(base + 5)?,
        dirty: row.get(base + 6)?,
        draft: row.get(base + 7)?,
        unlinked: row.get(base + 8)?,
        atime: row.get(base + 9)?,
        atime_nsec: row.get(base + 10)?,
        ino: match row.get::<_, Option<i64>>(base + 11)? {
            Some(ino) => Some(
                u64::try_from(ino).map_err(|_| {
                    rusqlite::Error::IntegralValueOutOfRange(base + 11, ino)
                })?,
            ),
            None => None,
        },
    }))
}

impl Projection {
    fn finish(self, store: &Store) -> Result<PosixRecord> {
        let unsigned =
            |n: i64| u64::try_from(n).map_err(|_| Error::State("negative POSIX metadata".into()));
        if self.size.is_none() && self.blob.is_none() && !self.directory {
            return Err(Error::State(
                "file metadata missing; native snapshot recovery required".into(),
            ));
        }
        let mut size = unsigned(self.size.unwrap_or(0))?;
        let mut mtime = unsigned(self.mtime.unwrap_or(0))?;
        let allocated = self.blob.is_some() || self.draft.is_some();
        // Bounded kernel caching is only allowed for authoritative metadata
        // that does not depend on mutable local content.
        let cacheable = !self.unlinked && self.draft.is_none() && !self.dirty;
        let modified = |stat: &std::fs::Metadata| -> Result<u64> {
            Ok(stat
                .modified()?
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs())
        };
        if let Some(blob) = self.blob {
            let stat = std::fs::metadata(store.root.join("blobs").join(blob))?;
            size = stat.len();
            // Same timestamp precedence as FileRecord, without computing pin,
            // cache/version sync state or opening/registering a DraftHandle.
            // Namespace-only pending work does not modify the file's body.
            // A dirty body keeps its own time even when it has a conflict.
            if self.dirty {
                mtime = modified(&stat)?;
            }
        }
        if let Some(token) = self.draft {
            if uuid::Uuid::parse_str(&token).is_err() {
                return Err(Error::State("invalid draft identity".into()));
            }
            let stat = std::fs::metadata(store.root.join("drafts").join(token))?;
            size = stat.len();
            mtime = modified(&stat)?;
        }
        let ino = match self.ino {
            Some(ino) => ino,
            None => {
                // A node created after install/backfill has no derived identity
                // yet. Allocate it now; this is the only write on this path.
                let initialized: bool = store
                    .db
                    .prepare_cached(ROOT_INITIALIZED_QUERY)?
                    .query_row([], |r| r.get(0))?;
                if !initialized {
                    return Err(Error::State("filesystem root not initialized".into()));
                }
                store
                    .db
                    .execute("INSERT INTO fs_inodes(node) VALUES(?1)", [&self.id])?;
                u64::try_from(store.db.last_insert_rowid())
                    .map_err(|_| Error::State("invalid inode".into()))?
            }
        };
        let (atime, atime_nsec) = match (self.atime, self.atime_nsec) {
            (None, None) => (mtime, 0),
            (Some(seconds), Some(nanos)) if (0..1_000_000_000).contains(&nanos) => {
                (unsigned(seconds)?, nanos as u32)
            }
            _ => return Err(Error::State("invalid POSIX access time".into())),
        };
        Ok(PosixRecord {
            id: self.id,
            ino,
            directory: self.directory,
            size,
            mtime,
            atime,
            atime_nsec,
            mode: self.mode.unwrap_or(if self.directory { 0o755 } else { 0o644 }) & 0o777,
            unlinked: self.unlinked,
            blocks: if allocated { size.div_ceil(512) } else { 0 },
            cacheable,
        })
    }
}
