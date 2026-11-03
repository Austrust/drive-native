//! Minimal per-directory write permissions. The native lookup contract exposes
//! no per-node ACL, so a missing decision is explicitly `unknown` and callers
//! must fail closed. A decision always comes from a configured or observed
//! source; this module never invents remote capability.
use crate::{Error, Result, Store};
use rusqlite::{OptionalExtension, params};
use serde::{Deserialize, Serialize};

/// Effective write permission for one node.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WritePermission {
    Allowed,
    Denied,
    /// No authoritative source has decided yet. Conservative callers deny.
    Unknown,
}
impl WritePermission {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Allowed => "allowed",
            Self::Denied => "denied",
            Self::Unknown => "unknown",
        }
    }
    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "allowed" => Ok(Self::Allowed),
            "denied" => Ok(Self::Denied),
            "unknown" => Ok(Self::Unknown),
            _ => Err(Error::State("invalid write permission".into())),
        }
    }
}

/// Result of resolving a node against the directory policy chain.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct EffectiveWritePermission {
    pub permission: WritePermission,
    /// Node that supplied the decision; `None` means no explicit policy exists.
    pub source: Option<String>,
    /// Configured origin label (for example `setup`, `file_station`, `observed`).
    pub origin: Option<String>,
}
impl EffectiveWritePermission {
    pub fn unknown() -> Self {
        Self {
            permission: WritePermission::Unknown,
            source: None,
            origin: None,
        }
    }
}

fn valid_origin(origin: &str) -> bool {
    !origin.is_empty() && origin.len() <= 128 && !origin.chars().any(char::is_control)
}

impl Store {
    /// Record a directory write decision. New decisions invalidate readers
    /// through the durable revision, never through a TTL cache.
    pub fn set_directory_permission(
        &mut self,
        node: &str,
        permission: WritePermission,
        origin: &str,
    ) -> Result<()> {
        if !valid_origin(origin) {
            return Err(Error::State("invalid write permission origin".into()));
        }
        let exists: bool = self.db.query_row(
            "SELECT EXISTS(SELECT 1 FROM nodes WHERE id=?1)",
            [node],
            |r| r.get(0),
        )?;
        if !exists {
            return Err(Error::State("unknown write permission node".into()));
        }
        let tx = self.db.transaction()?;
        tx.execute(
            "INSERT INTO directory_permissions(node,permission,origin,updated) VALUES(?1,?2,?3,unixepoch())
             ON CONFLICT(node) DO UPDATE SET permission=excluded.permission,origin=excluded.origin,updated=excluded.updated",
            params![node, permission.as_str(), origin],
        )?;
        bump_write_policy_revision(&tx)?;
        tx.commit()?;
        Ok(())
    }
    /// Remove an explicit decision. The node falls back to its nearest ancestor
    /// or to `unknown`.
    pub fn clear_directory_permission(&mut self, node: &str) -> Result<bool> {
        let tx = self.db.transaction()?;
        let changed = tx.execute("DELETE FROM directory_permissions WHERE node=?1", [node])?;
        if changed > 0 {
            bump_write_policy_revision(&tx)?;
        }
        tx.commit()?;
        Ok(changed > 0)
    }
    /// Resolve the nearest explicit decision on the node or one of its
    /// ancestors. Never guesses from a sibling or from the share capability.
    pub fn effective_write_permission(&self, node: &str) -> Result<EffectiveWritePermission> {
        let mut current = Some(node.to_owned());
        for _ in 0..=256 {
            let Some(id) = current else {
                return Ok(EffectiveWritePermission::unknown());
            };
            let row: Option<(String, String)> = self
                .db
                .query_row(
                    "SELECT permission,origin FROM directory_permissions WHERE node=?1",
                    [&id],
                    |r| Ok((r.get(0)?, r.get(1)?)),
                )
                .ok();
            if let Some((permission, origin)) = row {
                return Ok(EffectiveWritePermission {
                    permission: WritePermission::parse(&permission)?,
                    source: Some(id),
                    origin: Some(origin),
                });
            }
            // Missing node rows also end the chain: fail closed as unknown.
            current = self
                .db
                .query_row("SELECT parent FROM nodes WHERE id=?1", [&id], |r| {
                    r.get::<_, Option<String>>(0)
                })
                .ok()
                .flatten();
        }
        Err(Error::State("write permission ancestry too deep".into()))
    }
    /// Durable change counter; readers re-read the policy when it advances.
    pub fn write_policy_revision(&self) -> Result<u64> {
        let value: String = self.db.query_row(
            "SELECT value FROM metadata WHERE key='write_policy_revision'",
            [],
            |r| r.get(0),
        )?;
        value
            .parse::<u64>()
            .map_err(|_| Error::State("invalid write policy revision".into()))
    }
    /// Whether any directory inside `root` is explicitly writable. This is the
    /// only input that may promote a remote read-only scope to a RW mount.
    /// Explicit decisions are rare, so enumerate them and verify scope by
    /// ancestry instead of walking the whole node subtree on every check.
    pub fn has_allowed_directory_permission(&self, root: &str) -> Result<bool> {
        let ids: Vec<String> = {
            let mut q = self.db.prepare(
                "SELECT node FROM directory_permissions WHERE permission='allowed' LIMIT 257",
            )?;
            q.query_map([], |r| r.get(0))?
                .collect::<std::result::Result<Vec<_>, _>>()?
        };
        if ids.len() > 256 {
            return Err(Error::State("write permission decisions too large".into()));
        }
        for id in ids {
            if !self.is_within(root, &id)? {
                continue;
            }
            if self.effective_write_permission(&id)?.permission == WritePermission::Allowed {
                return Ok(true);
            }
        }
        Ok(false)
    }
    /// Bounded ancestry check: the node is `root` or one of its descendants.
    fn is_within(&self, root: &str, node: &str) -> Result<bool> {
        let mut current = Some(node.to_owned());
        for _ in 0..=256 {
            let Some(id) = current else {
                return Ok(false);
            };
            if id == root {
                return Ok(true);
            }
            current = self
                .db
                .query_row("SELECT parent FROM nodes WHERE id=?1", [&id], |r| {
                    r.get::<_, Option<String>>(0)
                })
                .optional()?
                .flatten();
        }
        Err(Error::State("write permission ancestry too deep".into()))
    }
}

pub(crate) fn bump_write_policy_revision(tx: &rusqlite::Transaction<'_>) -> Result<()> {
    tx.execute(
        "INSERT OR IGNORE INTO metadata(key,value) VALUES('write_policy_revision','0')",
        [],
    )?;
    tx.execute(
        "UPDATE metadata SET value=CAST(CAST(value AS INTEGER)+1 AS TEXT) WHERE key='write_policy_revision'",
        [],
    )?;
    Ok(())
}
