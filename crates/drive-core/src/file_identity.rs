//! Platform file identity contract (issue #11, step 1).
//!
//! The authoritative node id is the store UUID-like id (`nodes.id`); the
//! kernel-facing file identity is platform specific: a filesystem inode number
//! on Linux/FUSE, a `FILE_ID_128` in the CFAPI sync root on Windows. The
//! mapping is derived state (like `fs_inodes` already is) and never part of the
//! synced namespace, so a node can be re-mounted on another platform.
//!
//! Step 1 only extracts the read contract and a Linux adapter that delegates to
//! the existing `fs_inodes` state; it does **not** add the v30
//! `platform_file_ids` table, change any u64 ino interface (`FileRecord.ino`,
//! `filesystem_posix`, `namespace_index`, FUSE) or touch the FUSE callbacks.
//! [`PlatformFileId`] currently mirrors the exact `fs_inodes` integers already
//! visible to the kernel, so behavior is unchanged. Step 2 adds the persisted
//! 16-byte platform id column and the Windows CFAPI adapter behind the same
//! trait, then migrates the direct call sites listed in `docs/WINDOWS_PORT.md`.
//!
//! Retained direct call sites for step 1 (and why):
//!
//! - `Store::record_by_inode` / `record_impl`: the FUSE getattr/read path must
//!   keep its exact SQL count and error shape; step 2 routes them through
//!   [`FileIdentitySource`] once platform ids are persisted.
//! - `filesystem_posix` lookup/projection: hot-path JOINs returning `ino` as
//!   part of one statement; changing them now would duplicate the identity
//!   column before v30 exists.
//! - `namespace_index`, `preview` and `native_snapshot`: in-memory maps keyed by
//!   u64 and preview foreign keys into `fs_inodes`; they are the regression
//!   surface step 2 must keep compatible (see `WINDOWS_PORT.md` §5.1).
use crate::{Error, Result, Store};
use rusqlite::OptionalExtension;
use serde::{Deserialize, Serialize};
use std::fmt;

#[cfg(test)]
#[path = "file_identity_tests.rs"]
mod tests;

/// Opaque platform file identity exposed to the core.
///
/// Linux keeps its existing `u64` inode numbering in step 1 (the value the
/// kernel already sees), so this is a `#[repr(transparent)]` name for that
/// number. Step 2 will move the persisted representation to 16 opaque bytes
/// while [`PlatformFileId::as_u64`] keeps being the Linux projection.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
#[repr(transparent)]
pub struct PlatformFileId(u64);

impl PlatformFileId {
    /// Fixed identity of a mount root. Every supported backing store pins the
    /// root to a stable, low-numbered identity so `..`/self lookups never move.
    pub const ROOT: Self = Self(1);

    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn as_u64(self) -> u64 {
        self.0
    }
}

impl From<u64> for PlatformFileId {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl From<PlatformFileId> for u64 {
    fn from(value: PlatformFileId) -> Self {
        value.0
    }
}

impl fmt::Display for PlatformFileId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

fn from_sql(ino: i64) -> Result<PlatformFileId> {
    u64::try_from(ino)
        .map(PlatformFileId)
        .map_err(|_| Error::State("invalid platform file identity".into()))
}

fn sql_inode(id: PlatformFileId) -> Result<i64> {
    i64::try_from(id.as_u64())
        .map_err(|_| Error::State("platform file identity out of range".into()))
}

/// Read contract between the core and a platform file-identity provider.
///
/// All lookups are read-only and return `Ok(None)` for "no identity recorded
/// yet"; they must never allocate (the FUSE getattr path relies on this).
/// Errors stay in the crate's existing [`Error`] so Linux keeps its current
/// error shape and a Windows adapter maps platform errors at its boundary.
pub trait FileIdentitySource {
    /// Platform identity recorded for `node`, if any.
    fn id_for_node(&self, node: &str) -> Result<Option<PlatformFileId>>;

    /// Node that owns `id`, if any.
    fn node_for_id(&self, id: PlatformFileId) -> Result<Option<String>>;

    /// Fixed root identity; `Ok(None)` before the mount root is pinned.
    fn root_id(&self) -> Result<Option<PlatformFileId>>;
}

/// Linux adapter: delegates the contract to the existing `fs_inodes` state.
///
/// Every method is a read-only mirror of SQL already used elsewhere, and the
/// forwarding helpers call the existing mount-time entry points unchanged, so
/// this adapter is an equivalence layer, not a behavior change.
pub struct StoreFileIdentity<'a> {
    store: &'a Store,
}

impl<'a> StoreFileIdentity<'a> {
    pub const fn new(store: &'a Store) -> Self {
        Self { store }
    }

    /// Forward the one-time mount backfill (`fs_inodes` allocation, unchanged).
    pub fn ensure_namespace_identities(&self, root: &str) -> Result<u64> {
        self.store.ensure_filesystem_inodes(root)
    }

    /// Forward the mount-time root pin (`ino=1`, unchanged).
    pub fn pin_root_identity(&self, root: &str) -> Result<()> {
        self.store.ensure_fs_root(root)
    }
}

impl FileIdentitySource for StoreFileIdentity<'_> {
    fn id_for_node(&self, node: &str) -> Result<Option<PlatformFileId>> {
        let raw: Option<i64> = self
            .store
            .db
            .query_row("SELECT ino FROM fs_inodes WHERE node=?1", [node], |r| {
                r.get(0)
            })
            .optional()?;
        raw.map(from_sql).transpose()
    }

    fn node_for_id(&self, id: PlatformFileId) -> Result<Option<String>> {
        Ok(self
            .store
            .db
            .query_row(
                "SELECT node FROM fs_inodes WHERE ino=?1",
                [sql_inode(id)?],
                |r| r.get(0),
            )
            .optional()?)
    }

    fn root_id(&self) -> Result<Option<PlatformFileId>> {
        let raw: Option<i64> = self
            .store
            .db
            .query_row("SELECT ino FROM fs_inodes WHERE ino=1", [], |r| r.get(0))
            .optional()?;
        raw.map(from_sql).transpose()
    }
}

impl Store {
    /// Read-only platform identity view over this store's `fs_inodes` state.
    pub fn file_identity(&self) -> StoreFileIdentity<'_> {
        StoreFileIdentity::new(self)
    }
}
