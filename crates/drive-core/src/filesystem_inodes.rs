//! Stable filesystem inode identities are derived, reconstructible state.
//!
//! `fs_inodes` maps authoritative nodes to the inode numbers the kernel sees.
//! Allocation used to happen lazily on the first projection of each entry,
//! which put one `INSERT` (and one `synchronous=FULL` WAL commit) inside every
//! first-scan `opendir`. A full-tree initial scan therefore paid one durable
//! commit per directory instead of one per namespace mutation.
//!
//! The mapping is pure derived state: a node row carries everything needed to
//! rebuild it, and losing an uncommitted allocation can only reassign inode
//! numbers before the next mount (no content, no namespace, no operation
//! state is affected). It is safe to allocate the whole installed namespace in
//! the SAME transaction that installs the nodes, and it is safe to backfill a
//! database installed by an older build once at mount initialization.
use super::*;
use rusqlite::{Connection, OptionalExtension};

/// Give the mount root its fixed identity (`ino=1`) when the mapping is absent.
///
/// A different node already owning inode 1 is left untouched: `ensure_fs_root`
/// rejects that state with the original error, and silently remapping a live
/// root identity is prohibited. A stale non-root mapping of the root node is
/// replaced because it was never observable as the root.
fn pin_root_inode(db: &Connection, root: &str) -> Result<bool> {
    let existing: Option<String> = db
        .query_row("SELECT node FROM fs_inodes WHERE ino=1", [], |r| r.get(0))
        .optional()?;
    if existing.is_some() {
        return Ok(false);
    }
    let stale: Option<i64> = db
        .query_row("SELECT ino FROM fs_inodes WHERE node=?1", [root], |r| {
            r.get(0)
        })
        .optional()?;
    if stale.is_some() {
        db.execute("DELETE FROM fs_inodes WHERE node=?1", [root])?;
    }
    db.execute("INSERT INTO fs_inodes(ino,node) VALUES(1,?1)", [root])?;
    Ok(true)
}

/// Allocate stable inode identities for every live node that does not have one.
///
/// Caller owns the transaction boundary. The statement order is the node id,
/// so repeated installs and crash/retry cycles produce the same assignment for
/// an unchanged namespace, and `sqlite_sequence` still allocates monotonic
/// numbers for entries that only appear later. Returns rows written.
pub(crate) fn allocate_namespace_inodes(db: &Connection, root: &str) -> Result<u64> {
    let mut written = u64::from(pin_root_inode(db, root)?);
    written += u64::try_from(
        db.execute(
            "INSERT INTO fs_inodes(node)
             SELECT n.id FROM nodes n
             WHERE n.deleted=0
               AND NOT EXISTS(SELECT 1 FROM fs_inodes i WHERE i.node=n.id)
             ORDER BY n.id",
            [],
        )?,
    )
    .unwrap_or(u64::MAX);
    Ok(written)
}

impl Store {
    /// One-time backfill for a database whose snapshot was installed before
    /// batch inode allocation existed.
    ///
    /// Mount initialization serializes Store access, so no other projection can
    /// observe a partially allocated namespace. Idempotent: a store that is
    /// already complete performs only the two bounded checks and writes nothing
    /// (no WAL frames, no fsync). Returns the number of rows written.
    pub fn ensure_filesystem_inodes(&self, root: &str) -> Result<u64> {
        let root_ok: bool = self
            .db
            .query_row("SELECT node FROM fs_inodes WHERE ino=1", [], |r| {
                r.get::<_, String>(0)
            })
            .optional()?
            .is_some_and(|node| node == root);
        let missing: bool = self.db.query_row(
            "SELECT EXISTS(SELECT 1 FROM nodes n WHERE n.deleted=0 AND NOT EXISTS(SELECT 1 FROM fs_inodes i WHERE i.node=n.id))",
            [],
            |r| r.get(0),
        )?;
        if root_ok && !missing {
            return Ok(0);
        }
        let tx = self.db.unchecked_transaction()?;
        let written = allocate_namespace_inodes(&tx, root)?;
        tx.commit()?;
        Ok(written)
    }
}

#[cfg(test)]
#[path = "filesystem_inode_allocation_tests.rs"]
mod tests;
