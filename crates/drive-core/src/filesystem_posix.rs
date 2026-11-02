//! POSIX attributes without desktop sync-state, pin ancestry or content opens.
//! The caller serializes Store access. Dirty bodies still contribute actual
//! local size/time; this projection never hydrates content or changes queues.
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
}

impl Store {
    pub fn posix_record_by_inode(&self, ino: u64) -> Result<PosixRecord> {
        let id: String = self
            .db
            .prepare_cached("SELECT node FROM fs_inodes WHERE ino=?1")?
            .query_row([sql_unsigned(ino)?], |r| r.get(0))?;
        self.posix_record(&id, true)
    }
    pub fn posix_lookup(&self, parent: &str, name: &str) -> Result<PosixRecord> {
        // Retain the exact ACL/local-creation-shadow/ambiguity lookup contract.
        // Two IDs suffice to reject ambiguity; do not construct a Node (and
        // query its attributes) only to immediately repeat that projection.
        static SQL: std::sync::OnceLock<String> = std::sync::OnceLock::new();
        let sql = SQL.get_or_init(|| {
            format!("SELECT n.id FROM nodes n WHERE {VISIBLE_CHILDREN} AND n.name=?2 LIMIT 2")
        });
        let mut query = self.db.prepare_cached(sql)?;
        let mut rows = query.query(params![parent, name])?;
        let id: String = rows
            .next()?
            .ok_or(rusqlite::Error::QueryReturnedNoRows)?
            .get(0)?;
        if rows.next()?.is_some() {
            return Err(Error::State(
                "ambiguous namespace; recovery required".into(),
            ));
        }
        drop(rows);
        self.posix_record(&id, false)
    }
    fn posix_record(&self, id: &str, orphan: bool) -> Result<PosixRecord> {
        if self.is_inaccessible(id)? {
            return Err(rusqlite::Error::QueryReturnedNoRows.into());
        }
        let (directory, size, mtime, mode, blob, dirty, conflict, deleted, draft, draft_base, remote, unlinked, local_atime, local_atime_nsec):
            (bool, Option<i64>, Option<i64>, Option<u32>, Option<String>, bool, bool, bool, Option<String>, Option<String>, String, bool, Option<i64>, Option<i64>) =
            self.db.prepare_cached("SELECT n.directory,m.size,m.mtime,COALESCE(l.mode,m.mode),COALESCE(n.dirty_blob,n.blob),n.dirty_blob IS NOT NULL,n.conflict,n.deleted,d.token,d.base,n.remote_version,u.node IS NOT NULL,a.seconds,a.nanoseconds FROM nodes n LEFT JOIN node_metadata m ON m.node=n.id LEFT JOIN local_modes l ON l.node=n.id LEFT JOIN drafts d ON d.node=n.id AND d.dirty=1 LEFT JOIN local_unlinks u ON u.node=n.id LEFT JOIN local_atimes a ON a.node=n.id WHERE n.id=?1 AND (n.deleted=0 OR (?2 AND u.node IS NOT NULL))")?
            .query_row(params![id,orphan], |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?,r.get(6)?,r.get(7)?,r.get(8)?,r.get(9)?,r.get(10)?,r.get(11)?,r.get(12)?,r.get(13)?)))?;
        let unsigned =
            |n: i64| u64::try_from(n).map_err(|_| Error::State("negative POSIX metadata".into()));
        if size.is_none() && blob.is_none() && !directory {
            return Err(Error::State(
                "file metadata missing; native snapshot recovery required".into(),
            ));
        }
        let mut size = unsigned(size.unwrap_or(0))?;
        let mut mtime = unsigned(mtime.unwrap_or(0))?;
        let allocated = blob.is_some() || draft.is_some();
        let modified = |stat: &std::fs::Metadata| -> Result<u64> {
            Ok(stat
                .modified()?
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs())
        };
        if let Some(blob) = blob {
            let stat = std::fs::metadata(self.root.join("blobs").join(blob))?;
            size = stat.len();
            // Same timestamp precedence as FileRecord, without computing pin,
            // cache/version sync state or opening/registering a DraftHandle.
            let draft_conflict = draft.is_some()
                && !unlinked
                && (draft_base.as_deref() != Some(remote.as_str()) || deleted);
            if !conflict
                && !draft_conflict
                && !deleted
                && (dirty || draft.is_some() || crate::namespace::blocks_upload(&self.db, id)?)
            {
                mtime = modified(&stat)?;
            }
        }
        if let Some(token) = draft {
            if uuid::Uuid::parse_str(&token).is_err() {
                return Err(Error::State("invalid draft identity".into()));
            }
            let stat = std::fs::metadata(self.root.join("drafts").join(token))?;
            size = stat.len();
            mtime = modified(&stat)?;
        }
        let initialized: bool = self
            .db
            .prepare_cached("SELECT EXISTS(SELECT 1 FROM fs_inodes WHERE ino=1)")?
            .query_row([], |r| r.get(0))?;
        if !initialized {
            return Err(Error::State("filesystem root not initialized".into()));
        }
        let existing = self
            .db
            .prepare_cached("SELECT ino FROM fs_inodes WHERE node=?1")?
            .query_row([id], |r| read_unsigned(r, 0))
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
        let (atime, atime_nsec) = match (local_atime, local_atime_nsec) {
            (None, None) => (mtime, 0),
            (Some(seconds), Some(nanos)) if (0..1_000_000_000).contains(&nanos) => {
                (unsigned(seconds)?, nanos as u32)
            }
            _ => return Err(Error::State("invalid POSIX access time".into())),
        };
        Ok(PosixRecord {
            id: id.into(),
            ino,
            directory,
            size,
            mtime,
            atime,
            atime_nsec,
            mode: mode.unwrap_or(if directory { 0o755 } else { 0o644 }) & 0o777,
            unlinked,
            blocks: if allocated { size.div_ceil(512) } else { 0 },
        })
    }
}
