//! Effective decisions, not invented NAS ACL inheritance. Unknown permits an
//! attempt; only a confirmed remote outcome may become an observed decision.
use crate::{Error, Result, Store};
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use crate::filesystem::PosixRecord;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    Traverse,
    List,
    Read,
    CreateFile,
    CreateDir,
    Delete,
    RenameSource,
    RenameTarget,
    WriteContent,
    Setattr,
}
impl Action {
    pub const ALL: [Self; 10] = [
        Self::Traverse,
        Self::List,
        Self::Read,
        Self::CreateFile,
        Self::CreateDir,
        Self::Delete,
        Self::RenameSource,
        Self::RenameTarget,
        Self::WriteContent,
        Self::Setattr,
    ];
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Traverse => "traverse",
            Self::List => "list",
            Self::Read => "read",
            Self::CreateFile => "create_file",
            Self::CreateDir => "create_dir",
            Self::Delete => "delete",
            Self::RenameSource => "rename_source",
            Self::RenameTarget => "rename_target",
            Self::WriteContent => "write_content",
            Self::Setattr => "setattr",
        }
    }
    pub fn is_write(self) -> bool {
        !matches!(self, Self::Traverse | Self::List | Self::Read)
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Decision {
    Allow,
    Deny,
    Unknown,
}

/// Writer-local memo for effective decisions (#26). Every committed database
/// write bumps `total_changes`, which invalidates the whole map; read-only
/// handles never populate it because they cannot observe later commits.
pub(crate) struct PermissionCache {
    epoch: u64,
    decisions: HashMap<(String, String, Action), Decision>,
    pub(crate) hits: u64,
    pub(crate) misses: u64,
    /// Successful batched ACL page projections (#26).
    pub(crate) batched_pages: u64,
    /// Entries covered by those batched projections.
    pub(crate) batched_entries: u64,
}
impl PermissionCache {
    pub(crate) fn new() -> Self {
        Self {
            epoch: u64::MAX,
            decisions: HashMap::new(),
            hits: 0,
            misses: 0,
            batched_pages: 0,
            batched_entries: 0,
        }
    }
    fn roll(&mut self, epoch: u64) {
        if self.epoch != epoch {
            self.epoch = epoch;
            self.decisions.clear();
        }
    }
    pub(crate) fn lookup(
        &mut self,
        epoch: u64,
        root: &str,
        node: &str,
        action: Action,
    ) -> Option<Decision> {
        self.roll(epoch);
        match self
            .decisions
            .get(&(root.to_owned(), node.to_owned(), action))
        {
            Some(decision) => {
                self.hits += 1;
                Some(*decision)
            }
            None => {
                self.misses += 1;
                None
            }
        }
    }
    pub(crate) fn store(
        &mut self,
        epoch: u64,
        root: &str,
        node: &str,
        action: Action,
        decision: Decision,
    ) {
        self.roll(epoch);
        if self.decisions.len() >= 131_072 {
            self.decisions.clear();
        }
        self.decisions
            .insert((root.to_owned(), node.to_owned(), action), decision);
    }
    /// Count one successful batched ACL page projection and the entries it
    /// covered (#26). Only a wholly batched page calls this, so the per-entry
    /// fallback never inflates the counters.
    pub(crate) fn record_batch(&mut self, entries: u64) {
        self.batched_pages += 1;
        self.batched_entries += entries;
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PermissionCacheStats {
    pub hits: u64,
    pub misses: u64,
}
/// Process-local ACL diagnostics consumed by `drive-fs` (#26). Additive
/// counters only: no JSON/D-Bus field and no behaviour depends on them.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AclMetrics {
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub batched_pages: u64,
    pub batched_entries: u64,
}
/// Snapshot the writer-local permission memo and the batched READDIRPLUS
/// projection counters (#26). Read-only handles never populate the memo, so
/// their cache counters stay zero; the batch counters are per `Store`.
pub fn acl_metrics(store: &Store) -> AclMetrics {
    let cache = store.permission_cache_stats();
    let (batched_pages, batched_entries) = store.permission_cache_batch_stats();
    AclMetrics {
        cache_hits: cache.hits,
        cache_misses: cache.misses,
        batched_pages,
        batched_entries,
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LocalPolicy {
    ExplicitReadOnly,
    FollowRemote,
}
impl LocalPolicy {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ExplicitReadOnly => "explicit_read_only",
            Self::FollowRemote => "follow_remote",
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServerCapability {
    Unknown,
    Allowed,
    SpaceForbidden,
}
impl ServerCapability {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Unknown => "unknown",
            Self::Allowed => "allowed",
            Self::SpaceForbidden => "space_forbidden",
        }
    }
}
#[derive(Debug, Clone, Serialize)]
pub struct Policy {
    pub local_policy: LocalPolicy,
    pub writable_source: String,
    pub server_capability: ServerCapability,
    pub write_policy_revision: u64,
}
fn invalid() -> Error {
    Error::State("invalid permission state".into())
}
pub(crate) fn migrate(db: &Connection) -> Result<()> {
    db.execute_batch(
        "CREATE TABLE IF NOT EXISTS write_policy(
        root TEXT PRIMARY KEY,local_policy TEXT NOT NULL,local_policy_source TEXT NOT NULL,
        server_capability TEXT NOT NULL,server_capability_source TEXT NOT NULL,
        revision INTEGER NOT NULL DEFAULT 0,updated_at INTEGER NOT NULL);
      CREATE TABLE IF NOT EXISTS directory_permissions(
        root TEXT NOT NULL,node TEXT NOT NULL,action TEXT NOT NULL,state TEXT NOT NULL,
        source TEXT NOT NULL,verified_at INTEGER,revision INTEGER NOT NULL,
        PRIMARY KEY(root,node,action));
      CREATE INDEX IF NOT EXISTS directory_permissions_root ON directory_permissions(root,node);",
    )?;
    Ok(())
}
pub fn policy(store: &Store, root: &str) -> Result<Policy> {
    let row:Option<(String,String,String,i64)>=store.db.query_row(
        "SELECT local_policy,local_policy_source,server_capability,revision FROM write_policy WHERE root=?1",[root],
        |r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?))).optional()?;
    let Some((local, source, server, revision)) = row else {
        return Ok(Policy {
            local_policy: LocalPolicy::FollowRemote,
            writable_source: "legacy".into(),
            server_capability: ServerCapability::Unknown,
            write_policy_revision: 0,
        });
    };
    Ok(Policy {
        local_policy: match local.as_str() {
            "explicit_read_only" => LocalPolicy::ExplicitReadOnly,
            "follow_remote" => LocalPolicy::FollowRemote,
            _ => return Err(invalid()),
        },
        writable_source: source,
        server_capability: match server.as_str() {
            "unknown" => ServerCapability::Unknown,
            "allowed" => ServerCapability::Allowed,
            "space_forbidden" => ServerCapability::SpaceForbidden,
            _ => return Err(invalid()),
        },
        write_policy_revision: revision.try_into().map_err(|_| invalid())?,
    })
}
pub fn local_policy(store: &Store, root: &str) -> Result<LocalPolicy> {
    Ok(policy(store, root)?.local_policy)
}
pub fn mount_writable(store: &Store, root: &str) -> Result<bool> {
    let p = policy(store, root)?;
    Ok(p.local_policy == LocalPolicy::FollowRemote
        && p.server_capability != ServerCapability::SpaceForbidden)
}
/// Called once when a profile first meets schema25. A persisted user choice is
/// never silently replaced by an old share capability or a later reconnect.
pub fn initialize_policy(
    store: &Store,
    root: &str,
    local: LocalPolicy,
    source: &str,
) -> Result<()> {
    if !matches!(source, "legacy" | "user") || root.is_empty() {
        return Err(invalid());
    }
    store.db.execute(
        "INSERT OR IGNORE INTO write_policy VALUES(?1,?2,?3,'unknown','legacy',0,unixepoch())",
        params![root, local.as_str(), source],
    )?;
    Ok(())
}
pub fn set_local_policy(store: &Store, root: &str, local: LocalPolicy) -> Result<()> {
    initialize_policy(store, root, local, "user")?;
    store.db.execute("UPDATE write_policy SET local_policy=?2,local_policy_source='user',revision=revision+1,updated_at=unixepoch() WHERE root=?1",params![root,local.as_str()])?;
    Ok(())
}
pub fn set_local_policy_checked(
    store: &Store,
    root: &str,
    local: LocalPolicy,
    revision: u64,
) -> Result<()> {
    let revision = i64::try_from(revision).map_err(|_| invalid())?;
    if store.db.execute("UPDATE write_policy SET local_policy=?2,local_policy_source='user',revision=revision+1,updated_at=unixepoch() WHERE root=?1 AND revision=?3",params![root,local.as_str(),revision])?!=1{return Err(Error::State("permission policy changed; reload settings".into()));}
    Ok(())
}
pub fn snapshot(store: &Store, root: &str, node: &str) -> Result<serde_json::Value> {
    let mut actions = serde_json::Map::new();
    for action in Action::ALL {
        let row:Option<(String,Option<i64>,i64)>=store.db.query_row("SELECT source,verified_at,revision FROM directory_permissions WHERE root=?1 AND node=?2 AND action=?3",params![root,node,action.as_str()],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).optional()?;
        let (source, verified, revision) = row.unwrap_or(("unknown".into(), None, 0));
        actions.insert(action.as_str().into(),serde_json::json!({"state":match decide(store,root,node,action)?{Decision::Allow=>"allowed",Decision::Deny=>"denied",Decision::Unknown=>"unknown"},"source":source,"verified_at":verified,"revision":revision}));
    }
    Ok(serde_json::json!({"node":node,"policy":policy(store,root)?,"actions":actions}))
}
pub fn filesystem_root(store: &Store) -> Result<Option<String>> {
    Ok(store
        .db
        .query_row("SELECT node FROM fs_inodes WHERE ino=1", [], |r| r.get(0))
        .optional()?)
}
fn exact(store: &Store, root: &str, node: &str, action: Action) -> Result<Decision> {
    let value: Option<String> = store
        .db
        .query_row(
            "SELECT state FROM directory_permissions WHERE root=?1 AND node=?2 AND action=?3",
            params![root, node, action.as_str()],
            |r| r.get(0),
        )
        .optional()?;
    match value.as_deref() {
        None | Some("unknown") => Ok(Decision::Unknown),
        Some("allowed") => Ok(Decision::Allow),
        Some("denied") => Ok(Decision::Deny),
        _ => Err(invalid()),
    }
}
pub fn decide(store: &Store, root: &str, node: &str, action: Action) -> Result<Decision> {
    if let Some(decision) = store.permission_cache_lookup(root, node, action) {
        return Ok(decision);
    }
    let decision = decide_uncached(store, root, node, action)?;
    store.permission_cache_store(root, node, action, decision);
    Ok(decision)
}
pub(crate) fn decide_uncached(
    store: &Store,
    root: &str,
    node: &str,
    action: Action,
) -> Result<Decision> {
    if action.is_write() && !mount_writable(store, root)? {
        return Ok(Decision::Deny);
    }
    if store.is_inaccessible(node)? {
        return Ok(Decision::Deny);
    }
    // A local rename changes nodes.parent before remote confirmation. Both
    // retained ancestry graphs constrain reachability until reconciliation.
    let denied:bool=store.db.query_row("WITH RECURSIVE ancestors(id) AS (
      SELECT ?2 UNION SELECT n.parent FROM nodes n JOIN ancestors a ON n.id=a.id WHERE n.parent IS NOT NULL AND a.id!=?1
      UNION SELECT json_extract(o.remote,'$.parent') FROM namespace_overlay o JOIN ancestors a ON o.node=a.id WHERE json_extract(o.remote,'$.parent') IS NOT NULL AND a.id!=?1)
      SELECT EXISTS(SELECT 1 FROM ancestors a JOIN directory_permissions p ON p.node=a.id WHERE p.root=?1 AND p.action='traverse' AND p.state='denied')",params![root,node],|r|r.get(0))?;
    if denied {
        return Ok(Decision::Deny);
    }
    let mut seen = HashSet::new();
    let mut current = Some(node.to_owned());
    let mut in_scope = false;
    while let Some(id) = current {
        if !seen.insert(id.clone()) {
            return Err(invalid());
        }
        // Traversal is inherited as a reachability requirement, never as a
        // blanket inheritance of a parent's create/delete/write permissions.
        if (id != node || action == Action::Traverse)
            && exact(store, root, &id, Action::Traverse)? == Decision::Deny
        {
            return Ok(Decision::Deny);
        }
        if id == root {
            in_scope = true;
            break;
        }
        current = store
            .db
            .query_row("SELECT parent FROM nodes WHERE id=?1", [&id], |r| {
                r.get::<_, Option<String>>(0)
            })
            .optional()?
            .flatten();
    }
    if !in_scope {
        return Ok(Decision::Deny);
    }
    if action == Action::WriteContent {
        let parent: Option<String> = store.db.query_row(
            "SELECT parent FROM nodes WHERE id=?1 AND remote_version=?2",
            params![node, crate::creations::CREATE_VERSION], |r| r.get(0),
        ).optional()?.flatten();
        if let Some(parent) = parent {
            if decide(store, root, &parent, Action::CreateFile)? == Decision::Deny {
                return Ok(Decision::Deny);
            }
        }
    }
    exact(store, root, node, action)
}
/// Batched read projection for a READDIRPLUS ACL page (#26). Mirrors the
/// `drive-fs` `permissions::project` decision order while hoisting the repeated
/// per-entry queries into a handful of page queries: root lookup, mount policy,
/// parent reachability, per-page denied identities and permission rows.
///
/// Returns one `allowed` flag per record; `false` means the caller must publish
/// the entry without a lookup (same contract as a failed per-record project).
pub fn project_read_page(
    store: &Store,
    parent: &str,
    records: &mut [PosixRecord],
) -> Result<Vec<bool>> {
    let mut flags = vec![false; records.len()];
    if records.is_empty() {
        return Ok(flags);
    }
    let Some(root) = filesystem_root(store)? else {
        // drive-fs project() allows the entry when no filesystem root exists.
        flags.fill(true);
        return Ok(flags);
    };
    let writable = mount_writable(store, &root)?;
    // A directory whose own traversal is denied makes every child inaccessible;
    // one memoized decision covers the page instead of one per entry.
    let parent_allowed = decide(store, &root, parent, Action::Traverse)? != Decision::Deny;
    let ids: Vec<&str> = records.iter().map(|record| record.id.as_str()).collect();
    let placeholders = (1..=ids.len())
        .map(|index| format!("?{index}"))
        .collect::<Vec<_>>()
        .join(",");

    let mut denied_own = HashSet::new();
    let sql = format!("SELECT node FROM access_denied WHERE node IN ({placeholders})");
    let mut query = store.db.prepare_cached(&sql)?;
    let mut rows = query.query(rusqlite::params_from_iter(ids.iter().copied()))?;
    while let Some(row) = rows.next()? {
        denied_own.insert(row.get::<_, String>(0)?);
    }

    let mut states: HashMap<(String, Action), Decision> = HashMap::new();
    let numbered = (2..=ids.len() + 1)
        .map(|index| format!("?{index}"))
        .collect::<Vec<_>>()
        .join(",");
    let sql = format!(
        "SELECT node,action,state FROM directory_permissions
         WHERE root=?1 AND node IN ({numbered})
           AND action IN ('traverse','list','read','write_content','setattr')"
    );
    let mut query = store.db.prepare_cached(&sql)?;
    let mut params: Vec<&dyn rusqlite::ToSql> = Vec::with_capacity(ids.len() + 1);
    params.push(&root);
    for id in &ids {
        params.push(id);
    }
    let mut rows = query.query(params.as_slice())?;
    while let Some(row) = rows.next()? {
        let node: String = row.get(0)?;
        let action: String = row.get(1)?;
        let state: String = row.get(2)?;
        let action = Action::ALL
            .into_iter()
            .find(|candidate| candidate.as_str() == action)
            .ok_or_else(invalid)?;
        let decision = match state.as_str() {
            "unknown" => Decision::Unknown,
            "allowed" => Decision::Allow,
            "denied" => Decision::Deny,
            _ => return Err(invalid()),
        };
        states.insert((node, action), decision);
    }

    let mut creation = HashSet::new();
    let sql = format!(
        "SELECT id FROM nodes WHERE id IN ({placeholders}) AND remote_version=?{}",
        ids.len() + 1
    );
    let mut query = store.db.prepare_cached(&sql)?;
    let mut params: Vec<&dyn rusqlite::ToSql> = Vec::with_capacity(ids.len() + 1);
    for id in &ids {
        params.push(id);
    }
    params.push(&crate::creations::CREATE_VERSION);
    let mut rows = query.query(params.as_slice())?;
    while let Some(row) = rows.next()? {
        creation.insert(row.get::<_, String>(0)?);
    }

    // A local rename can give an entry a retained remote ancestry that the
    // page-level parent check cannot prove; those entries fall back exactly.
    let mut overlaid = HashSet::new();
    let sql = format!("SELECT node FROM namespace_overlay WHERE node IN ({placeholders})");
    let mut query = store.db.prepare_cached(&sql)?;
    let mut rows = query.query(rusqlite::params_from_iter(ids.iter().copied()))?;
    while let Some(row) = rows.next()? {
        overlaid.insert(row.get::<_, String>(0)?);
    }
    let create_file = if creation.is_empty() {
        Decision::Allow
    } else {
        decide(store, &root, parent, Action::CreateFile)?
    };

    for (index, record) in records.iter_mut().enumerate() {
        let id = record.id.clone();
        let id = id.as_str();
        if overlaid.contains(id) {
            flags[index] = project_record(store, &root, id, record, writable)?;
            continue;
        }
        if !parent_allowed || denied_own.contains(id) {
            continue;
        }
        if state(&states, id, Action::Traverse) == Decision::Deny {
            continue;
        }
        let read = if record.directory { Action::List } else { Action::Read };
        if state(&states, id, read) == Decision::Deny {
            record.mode &= !0o444;
        }
        if !writable {
            flags[index] = true;
            continue;
        }
        if record.directory {
            flags[index] = true;
            continue;
        }
        let write_denied = if creation.contains(id) && create_file == Decision::Deny {
            true
        } else {
            state(&states, id, Action::WriteContent) == Decision::Deny
        };
        if write_denied && state(&states, id, Action::Setattr) == Decision::Deny {
            record.mode &= !0o222;
        }
        flags[index] = true;
    }
    // One wholly batched page: count the projection and the entries it
    // covered (#26). The per-entry fallback in `drive-fs` never reaches here.
    store.permission_cache_batch(records.len() as u64);
    Ok(flags)
}

fn state(
    states: &HashMap<(String, Action), Decision>,
    node: &str,
    action: Action,
) -> Decision {
    states
        .get(&(node.to_owned(), action))
        .copied()
        .unwrap_or(Decision::Unknown)
}

fn project_record(
    store: &Store,
    root: &str,
    node: &str,
    record: &mut PosixRecord,
    writable: bool,
) -> Result<bool> {
    if decide(store, root, node, Action::Traverse)? == Decision::Deny {
        return Ok(false);
    }
    let read = if record.directory {
        Action::List
    } else {
        Action::Read
    };
    if decide(store, root, node, read)? == Decision::Deny {
        record.mode &= !0o444;
    }
    if !writable || record.directory {
        return Ok(true);
    }
    let write_denied = decide(store, root, node, Action::WriteContent)? == Decision::Deny;
    let setattr_denied = decide(store, root, node, Action::Setattr)? == Decision::Deny;
    if write_denied && setattr_denied {
        record.mode &= !0o222;
    }
    Ok(true)
}/// Distinguishes whole-mount policy from a target-specific refusal.
pub fn enforce(store: &Store, root: &str, node: &str, action: Action) -> Result<()> {
    if action.is_write() && !mount_writable(store, root)? {
        return Err(std::io::Error::from_raw_os_error(libc::EROFS).into());
    }
    if decide(store, root, node, action)? == Decision::Deny {
        return Err(std::io::Error::from_raw_os_error(libc::EACCES).into());
    }
    Ok(())
}
pub fn record_observed(
    store: &Store,
    root: &str,
    node: &str,
    action: Action,
    allowed: bool,
) -> Result<()> {
    record(
        store,
        root,
        node,
        action,
        if allowed { "allowed" } else { "denied" },
        "observed",
    )
}
fn record(
    store: &Store,
    root: &str,
    node: &str,
    action: Action,
    state: &str,
    source: &str,
) -> Result<()> {
    if root.is_empty() || node.is_empty() {
        return Err(invalid());
    }
    let tx = store.db.unchecked_transaction()?;
    record_db(&tx, root, node, action, state, source)?;
    tx.commit()?;
    Ok(())
}
fn record_db(
    db: &Connection,
    root: &str,
    node: &str,
    action: Action,
    state: &str,
    source: &str,
) -> Result<()> {
    db.execute("INSERT OR IGNORE INTO write_policy VALUES(?1,'follow_remote','legacy','unknown','legacy',0,unixepoch())",[root])?;
    db.execute(
        "UPDATE write_policy SET revision=revision+1,updated_at=unixepoch() WHERE root=?1",
        [root],
    )?;
    db.execute("INSERT INTO directory_permissions VALUES(?1,?2,?3,?4,?5,unixepoch(),(SELECT revision FROM write_policy WHERE root=?1))
      ON CONFLICT(root,node,action) DO UPDATE SET state=excluded.state,source=excluded.source,verified_at=excluded.verified_at,revision=excluded.revision
      WHERE excluded.state='denied' OR excluded.source!='observed' OR directory_permissions.source NOT IN ('native','file_station') OR directory_permissions.state='unknown'",
      params![root,node,action.as_str(),state,source])?;
    Ok(())
}
pub fn reset_observed(store: &Store, root: &str) -> Result<()> {
    reset_evidence(store, root, true)
}
/// A native profile-change notification invalidates all cached per-node
/// evidence. It is not proof of an allow, a deny, or a whole-space capability.
/// Keep local policy and any explicit service restriction unchanged.
pub fn invalidate_remote_permissions(store: &Store, root: &str) -> Result<()> {
    reset_evidence(store, root, false)
}
fn reset_evidence(store: &Store, root: &str, observed_only: bool) -> Result<()> {
    if root.is_empty() { return Err(invalid()); }
    let tx = store.db.unchecked_transaction()?;
    // The first profile notification can precede the first observation. Make
    // its revision visible even when this Store has not yet persisted a policy.
    // INSERT OR IGNORE preserves every existing user/service choice.
    tx.execute("INSERT OR IGNORE INTO write_policy VALUES(?1,'follow_remote','legacy','unknown','legacy',0,unixepoch())", [root])?;
    // Clear the terminal retry gate paired with a typed Read refusal as well.
    tx.execute("UPDATE transfers SET state='pending',error=NULL,outcome=NULL,outcome_category=NULL,last_error=NULL,next_retry_at=0 WHERE direction='download' AND state IN ('failed','retry')
      AND json_valid(error) AND json_extract(error,'$.kind')='server_rejected' AND json_extract(error,'$.server_code') IS NULL
      AND node IN (SELECT node FROM directory_permissions WHERE root=?1 AND action='read' AND state='denied' AND (?2=0 OR source='observed'))",params![root,observed_only])?;
    tx.execute(
        "UPDATE write_policy SET revision=revision+1,updated_at=unixepoch() WHERE root=?1",
        [root],
    )?;
    tx.execute("UPDATE directory_permissions SET state='unknown',verified_at=NULL,revision=(SELECT revision FROM write_policy WHERE root=?1) WHERE root=?1 AND (?2=0 OR source='observed')",params![root,observed_only])?;
    tx.commit()?;
    Ok(())
}
pub fn reset_session(store: &Store) -> Result<()> {
    if let Some(root) = filesystem_root(store)? {
        reset_observed(store, &root)?;
    }
    Ok(())
}
/// Capture before a read request and compare while publishing its observation.
/// Any intervening evidence/policy update makes that observation conservative:
/// it may complete the call but must not resurrect a stale cached decision.
pub fn session_revision(store: &Store) -> Result<Option<(String, u64)>> {
    filesystem_root(store)?.map(|root| {
        let revision = policy(store, &root)?.write_policy_revision;
        Ok((root, revision))
    }).transpose()
}
pub fn defer_stale_read_refusal(store: &Store, ticket: &crate::DownloadTicket) -> Result<()> {
    store.db.execute("UPDATE transfers SET state='pending',error=NULL,outcome=NULL,outcome_category=NULL,last_error=NULL,next_retry_at=0 WHERE node=?1 AND version=?2 AND direction='download' AND state='inflight'",
        params![ticket.id,ticket.version])?;
    Ok(())
}
/// Targets are identities from the durable intent, not a possibly renamed path.
pub fn namespace_targets(kind: &str, body: &str) -> Result<Vec<(String, Action)>> {
    let mut targets = Vec::new();
    match kind {
        "mkdir" => {
            let j: crate::directories::DirectoryJob = serde_json::from_str(body)?;
            targets.push((j.parent.id, Action::CreateDir));
        }
        "rename" => {
            let j: crate::namespace::RenameJob = serde_json::from_str(body)?;
            if let Some(p) = j.node.parent {
                targets.push((p, Action::Traverse));
            }
            targets.push((j.node.id, Action::RenameSource));
            targets.push((j.parent_node.id, Action::RenameTarget));
        }
        "remove" => {
            let j: crate::removals::RemoveJob = serde_json::from_str(body)?;
            if let Some(p) = j.node.parent {
                targets.push((p, Action::Traverse));
            }
            targets.push((j.node.id, Action::Delete));
        }
        "replace" => {
            let j: crate::replacements::ReplaceJob = serde_json::from_str(body)?;
            if let Some(p) = j.source.parent {
                targets.push((p, Action::Traverse));
            }
            targets.push((j.source.id, Action::RenameSource));
            if let Some(p) = j.target.parent {
                targets.push((p, Action::RenameTarget));
            }
            targets.push((j.target.id, Action::Delete));
        }
        _ => return Err(invalid()),
    }
    Ok(targets)
}
pub fn targets_denied(store: &Store, targets: &[(String, Action)]) -> Result<bool> {
    let Some(root) = filesystem_root(store)? else {
        return Ok(false);
    };
    for (node, action) in targets {
        if decide(store, &root, node, *action)? == Decision::Deny {
            return Ok(true);
        }
    }
    Ok(false)
}
pub fn observe_targets(store: &Store, targets: &[(String, Action)], allowed: bool) -> Result<()> {
    let Some(root) = filesystem_root(store)? else {
        return Ok(());
    };
    for (node, action) in targets {
        record_observed(store, &root, node, *action, allowed)?;
    }
    Ok(())
}
/// A definite refusal retains the durable body and retries only after explicit
/// recheck. Never call for a transport error or an uncertain commit.
pub fn refuse_operation(
    store: &Store,
    operation: &str,
    namespace: bool,
    targets: &[(String, Action)],
    node: &str,
    action: Action,
) -> Result<()> {
    if !targets.iter().any(|(id, a)| id == node && *a == action) {
        return Err(invalid());
    }
    let Some(root) = filesystem_root(store)? else {
        return Err(invalid());
    };
    let (table, key) = if namespace {
        ("namespace_ops", format!("namespace:{operation}"))
    } else {
        ("operations", format!("upload:{operation}"))
    };
    let tx = store.db.unchecked_transaction()?;
    record_db(&tx, &root, node, action, "denied", "observed")?;
    // The server proved this attempt did not commit. An already uncertain
    // reconciliation is never reset or resent by this method.
    let changed = tx.execute(&format!("UPDATE {table} SET state=CASE state WHEN 'cleanup_inflight' THEN 'cleanup_pending' ELSE 'pending' END WHERE id=?1 AND state IN ('inflight','cleanup_inflight')"),[operation])?;
    if changed > 0 {
        // The attempt was proven not to have committed, so any previous
        // unknown diagnosis is stale: the next definite attempt starts fresh.
        crate::commit_diagnostics::clear_tx(&tx, table, operation)?;
    }
    if !namespace && changed > 0 {
        // A definite non-commit can capture the current path on the next
        // attempt. Keep content, origin and mtime intent; uncertain paths stay.
        tx.execute("DELETE FROM upload_paths WHERE operation=?1", [operation])?;
    }
    tx.execute("UPDATE sync_history SET result='failed',error_kind='permission_denied',error_summary='permission_denied',completed_at=unixepoch() WHERE operation_key=?1",[key])?;
    tx.commit()?;
    Ok(())
}
pub fn upload_targets(node: &crate::Node) -> Vec<(String, Action)> {
    if node.version == crate::creations::CREATE_VERSION {
        node.parent
            .iter()
            .cloned()
            .map(|p| (p, Action::CreateFile))
            .collect()
    } else {
        vec![(node.id.clone(), Action::WriteContent)]
    }
}

/// Preserve a direct permission-related reply without inventing a definite
/// non-commit or guessing which participant in a compound mutation was denied.
/// This operation and its bytes remain in reconciliation; no ACL row changes.
/// `0x9003` still arrives here for creations/mkdir/replace: only the
/// identity-addressed upload of an *existing* node has a proven
/// `(node, write_content)` attribution and is refused earlier through
/// `refuse_operation` (see `sproto::Error::MutationPermissionDenied`).
pub fn mutation_permission_uncertain(store: &Store, operation: &str, namespace: bool, code: u32) -> Result<()> {
    let (table, prefix) = if namespace { ("namespace_ops", "namespace") } else { ("operations", "upload") };
    let tx = store.db.unchecked_transaction()?;
    let (node, kind, state): (String, String, String) = tx.query_row(
        &format!("SELECT node,kind,state FROM {table} WHERE id=?1"), [operation],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?;
    let action = match (namespace, kind.as_str(), state.as_str()) {
        (false, "upload", "inflight") => "upload",
        (true, "mkdir" | "replace", "inflight") => "upload",
        (true, "replace", "cleanup_inflight") | (true, "remove", "inflight") => "remove",
        (true, "rename", "inflight") => "rename",
        _ => return Err(invalid()),
    };
    if !matches!((action, code), ("upload" | "remove" | "rename", 0x3002 | 0x3004)
        | ("upload", 0x9003 | 0x9004) | ("remove", 0xa003 | 0xa004)) {
        return Err(invalid());
    }
    tx.execute(&format!("UPDATE {table} SET state=CASE state WHEN 'cleanup_inflight' THEN 'cleanup_uncertain' ELSE 'uncertain' END WHERE id=?1"), [operation])?;
    crate::sync_history::record_permission_response(&tx, &format!("{prefix}:{operation}"), &node,
        if namespace { kind.as_str() } else { "upload" }, action, code)?;
    // #9b TB2: a direct permission-related reply is an unknown-retryable commit
    // outcome, not an anonymous pending row.
    let retry_after = crate::commit_diagnostics::unix_now()
        + if namespace { 2 } else { 10 };
    crate::commit_diagnostics::record(
        &tx,
        table,
        operation,
        &crate::commit_outcome::CommitDiagnostic::unknown_retryable(
            crate::commit_outcome::CommitCategory::PermissionUncertain,
            Some(format!(
                "permission response 0x{code:04x}; commit outcome unconfirmed"
            )),
            0,
            retry_after,
        ),
    )?;
    tx.commit()?;
    Ok(())
}
pub(crate) fn has_denials(store: &Store) -> Result<bool> {
    Ok(store.db.query_row(
        "SELECT EXISTS(SELECT 1 FROM directory_permissions WHERE state='denied')",
        [],
        |r| r.get(0),
    )?)
}

#[cfg(any(test, feature = "test-seams"))]
pub fn inject(
    store: &Store,
    root: &str,
    node: &str,
    action: Action,
    decision: Decision,
) -> Result<()> {
    record(
        store,
        root,
        node,
        action,
        match decision {
            Decision::Allow => "allowed",
            Decision::Deny => "denied",
            Decision::Unknown => "unknown",
        },
        "native",
    )
}
#[cfg(any(test, feature = "test-seams"))]
pub fn inject_server_capability(store: &Store, root: &str, cap: ServerCapability) -> Result<()> {
    initialize_policy(store, root, LocalPolicy::FollowRemote, "user")?;
    store.db.execute("UPDATE write_policy SET server_capability=?2,server_capability_source='native_view',revision=revision+1 WHERE root=?1",params![root,cap.as_str()])?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> (tempfile::TempDir, Store) {
        let dir = tempfile::tempdir().unwrap();
        let mut s = Store::open(dir.path()).unwrap();
        let nodes = [
            ("root", None, true),
            ("A", Some("root"), true),
            ("B", Some("root"), true),
            ("f", Some("A"), false),
        ]
        .map(|(id, p, d)| crate::Node {
            id: id.into(),
            parent: p.map(str::to_owned),
            name: id.into(),
            version: "v1".into(),
            directory: d,
            metadata: Some(crate::NodeMetadata {
                size: if d { 0 } else { 3 }, mtime: 1,
                mode: if d { 0o755 } else { 0o644 },
            }),
        });
        s.install_remote_snapshot(&nodes, "1", "server").unwrap();
        s.ensure_fs_root("root").unwrap();
        (dir, s)
    }
    #[test]
    fn first_denial_publication_waits_for_existing_bounded_reply() {
        use std::{sync::{Arc,mpsc},time::Duration};
        let(_d,s)=fixture();let shared=Arc::new(crate::SharedStore::new(s));
        let lease=shared.permission_clear_lease().unwrap();
        let(ready_tx,ready_rx)=mpsc::channel();let(done_tx,done_rx)=mpsc::channel();
        let writer=shared.clone();let thread=std::thread::spawn(move||{
            let guard=writer.lock().unwrap();inject(&guard,"root","f",Action::Read,Decision::Deny).unwrap();
            ready_tx.send(()).unwrap();drop(guard);done_tx.send(()).unwrap();
        });
        ready_rx.recv_timeout(Duration::from_secs(2)).unwrap();
        assert!(done_rx.recv_timeout(Duration::from_millis(50)).is_err());
        assert!(!shared.has_permission_denials());drop(lease);
        done_rx.recv_timeout(Duration::from_secs(2)).unwrap();thread.join().unwrap();
        assert!(shared.permission_clear_lease().is_none());assert!(shared.has_permission_denials());
    }
    #[test]
    fn cached_reply_lease_does_not_wait_for_ordinary_metadata_store() {
        let(_d,s)=fixture();let shared=crate::SharedStore::new(s);let guard=shared.lock().unwrap();
        let lease=shared.permission_clear_lease().expect("ordinary Store work must not block cached replies");
        drop(lease);drop(guard);
    }
    #[test]
    fn root_denial_does_not_become_a_child_write_policy() {
        let (_d, s) = fixture();
        inject(&s, "root", "root", Action::CreateDir, Decision::Deny).unwrap();
        inject(&s, "root", "A", Action::CreateDir, Decision::Allow).unwrap();
        inject(&s, "root", "B", Action::CreateDir, Decision::Deny).unwrap();
        assert_eq!(
            decide(&s, "root", "A", Action::CreateDir).unwrap(),
            Decision::Allow
        );
        assert_eq!(
            decide(&s, "root", "root", Action::CreateDir).unwrap(),
            Decision::Deny
        );
        assert_eq!(
            decide(&s, "root", "B", Action::CreateDir).unwrap(),
            Decision::Deny
        );
    }
    #[test]
    fn action_classes_are_independent() {
        let (_d, s) = fixture();
        inject(&s, "root", "A", Action::CreateDir, Decision::Allow).unwrap();
        inject(&s, "root", "A", Action::CreateFile, Decision::Deny).unwrap();
        inject(&s, "root", "f", Action::WriteContent, Decision::Allow).unwrap();
        inject(&s, "root", "f", Action::Delete, Decision::Deny).unwrap();
        assert!(enforce(&s, "root", "A", Action::CreateDir).is_ok());
        assert!(enforce(&s, "root", "A", Action::CreateFile).is_err());
        assert!(enforce(&s, "root", "f", Action::WriteContent).is_ok());
        assert!(enforce(&s, "root", "f", Action::Delete).is_err());
    }
    #[test]
    fn unknown_attempt_does_not_persist_allow() {
        let (_d, s) = fixture();
        assert_eq!(
            decide(&s, "root", "A", Action::CreateFile).unwrap(),
            Decision::Unknown
        );
        assert!(enforce(&s, "root", "A", Action::CreateFile).is_ok());
        assert_eq!(
            s.db.query_row("SELECT count(*) FROM directory_permissions", [], |r| r
                .get::<_, i64>(0))
                .unwrap(),
            0
        );
    }
    #[test]
    fn recheck_resets_observed_but_never_explicit_read_only() {
        let (_d, s) = fixture();
        record_observed(&s, "root", "A", Action::CreateFile, false).unwrap();
        assert!(enforce(&s, "root", "A", Action::CreateFile).is_err());
        set_local_policy(&s, "root", LocalPolicy::ExplicitReadOnly).unwrap();
        reset_observed(&s, "root").unwrap();
        assert_eq!(
            exact(&s, "root", "A", Action::CreateFile).unwrap(),
            Decision::Unknown
        );
        assert_eq!(
            local_policy(&s, "root").unwrap(),
            LocalPolicy::ExplicitReadOnly
        );
        assert!(
            matches!(enforce(&s,"root","A",Action::CreateFile),Err(Error::Io(ref e)) if e.raw_os_error()==Some(libc::EROFS))
        );
    }
    #[test]
    fn mutation_permission_hint_preserves_uncertainty_bytes_and_unknown_acl() {
        let (_d, mut s) = fixture();
        let op = s.save_local("f", b"unconfirmed content").unwrap();
        s.mark_upload_started(&op).unwrap();
        assert!(mutation_permission_uncertain(&s, &op, false, 0x9005).is_err());
        assert_eq!(s.upload_job(&op).unwrap().state, "inflight");
        mutation_permission_uncertain(&s, &op, false, 0x9004).unwrap();
        assert_eq!(s.upload_job(&op).unwrap().state, "uncertain");
        assert_eq!(decide(&s, "root", "f", Action::WriteContent).unwrap(), Decision::Unknown);
        assert!(mutation_permission_uncertain(&s, &op, false, 0x9004).is_err());
        reset_session(&s).unwrap();
        assert_eq!(s.upload_job(&op).unwrap().state, "uncertain");
        let mut bytes = Vec::new();
        std::io::Read::read_to_end(&mut s.open_cached("f").unwrap(), &mut bytes).unwrap();
        assert_eq!(bytes, b"unconfirmed content");
        let rows = s.sync_history(&crate::sync_history::HistoryQuery::default()).unwrap().records;
        assert_eq!(rows[0].error_kind.as_deref(), Some("permission_commit_uncertain"));
        assert_eq!(rows[0].result, "pending");
        s.confirm_upload(&op, "v2").unwrap();
        let rows = s.sync_history(&crate::sync_history::HistoryQuery::default()).unwrap().records;
        assert_eq!(rows[0].result, "ok");
        assert!(rows[0].error_kind.is_none());
    }
    #[test]
    fn directory_permission_hint_keeps_journal_and_reconciliation_barrier() {
        let (_d, mut s) = fixture();
        s.create_local_directory("A", "unconfirmed", 0o755).unwrap();
        let op = s.scheduled_directory().unwrap().unwrap().job.operation;
        s.mark_directory_started(&op).unwrap();
        mutation_permission_uncertain(&s, &op, true, 0x9004).unwrap();
        assert_eq!(s.scheduled_directory().unwrap().unwrap().state, "uncertain");
        assert!(s.mark_directory_started(&op).is_err());
        assert_eq!(decide(&s, "root", "A", Action::CreateDir).unwrap(), Decision::Unknown);
        let rows = s.sync_history(&crate::sync_history::HistoryQuery::default()).unwrap().records;
        assert_eq!(rows[0].action, "mkdir");
        assert_eq!(rows[0].error_kind.as_deref(), Some("permission_commit_uncertain"));
        s.conflict_directory(&op).unwrap();
        let rows = s.sync_history(&crate::sync_history::HistoryQuery::default()).unwrap().records;
        assert_eq!(rows[0].error_kind.as_deref(), Some("conflict"));
    }
    #[test]
    fn profile_change_expires_all_node_sources_but_preserves_scope_policy_and_intents() {
        for uncertain in [false, true] {
        let (_d, mut s) = fixture();
        let op = s.save_local("f", b"unconfirmed content").unwrap();
        if uncertain {
            s.mark_upload_started(&op).unwrap();
            s.mark_upload_uncertain(&op).unwrap();
        }
        let state = s.upload_job(&op).unwrap().state;
        record_observed(&s, "root", "f", Action::Read, false).unwrap();
        inject(&s, "root", "A", Action::CreateDir, Decision::Deny).unwrap();
        record(&s, "root", "B", Action::List, "allowed", "file_station").unwrap();
        record_observed(&s, "other-root", "other-node", Action::Read, false).unwrap();
        set_local_policy(&s, "root", LocalPolicy::ExplicitReadOnly).unwrap();
        inject_server_capability(&s, "root", ServerCapability::SpaceForbidden).unwrap();
        let revision = policy(&s, "root").unwrap().write_policy_revision;
        invalidate_remote_permissions(&s, "root").unwrap();
        for (node, action) in [("f", Action::Read), ("A", Action::CreateDir), ("B", Action::List)] {
            assert_eq!(exact(&s, "root", node, action).unwrap(), Decision::Unknown);
        }
        assert_eq!(exact(&s, "other-root", "other-node", Action::Read).unwrap(), Decision::Deny);
        let policy = policy(&s, "root").unwrap();
        assert_eq!(policy.local_policy, LocalPolicy::ExplicitReadOnly);
        assert_eq!(policy.server_capability, ServerCapability::SpaceForbidden);
        assert_eq!(policy.write_policy_revision, revision + 1);
        assert_eq!(s.upload_job(&op).unwrap().state, state);
        let mut bytes = Vec::new();
        std::io::Read::read_to_end(&mut s.open_cached("f").unwrap(), &mut bytes).unwrap();
        assert_eq!(bytes, b"unconfirmed content");
        assert_eq!(s.db.query_row("SELECT count(*) FROM directory_permissions WHERE root='root' AND verified_at IS NOT NULL", [], |r| r.get::<_, i64>(0)).unwrap(), 0);
        }
    }
    #[test]
    fn traverse_is_distinct_from_list() {
        let (_d, s) = fixture();
        inject(&s, "root", "A", Action::List, Decision::Deny).unwrap();
        assert!(enforce(&s, "root", "f", Action::Read).is_ok());
        inject(&s, "root", "A", Action::Traverse, Decision::Deny).unwrap();
        assert!(enforce(&s, "root", "f", Action::Read).is_err());
    }
    #[test]
    fn explicit_and_space_limits_override_child_allow() {
        let (_d, s) = fixture();
        inject(&s, "root", "A", Action::CreateDir, Decision::Allow).unwrap();
        inject_server_capability(&s, "root", ServerCapability::SpaceForbidden).unwrap();
        assert!(!mount_writable(&s, "root").unwrap());
        assert!(enforce(&s, "root", "A", Action::CreateDir).is_err());
        assert!(enforce(&s, "root", "A", Action::List).is_ok());
    }
    #[test]
    fn independent_subroot_stops_at_its_own_ancestry_boundary() {
        let (_d, s) = fixture();
        inject(&s, "A", "root", Action::Traverse, Decision::Deny).unwrap();
        inject(&s, "A", "A", Action::CreateDir, Decision::Allow).unwrap();
        assert_eq!(
            decide(&s, "A", "A", Action::CreateDir).unwrap(),
            Decision::Allow
        );
        assert!(enforce(&s, "A", "f", Action::Read).is_ok());
    }
    #[test]
    fn remote_overlay_ancestry_is_not_bypassed_by_local_move() {
        let (_d, mut s) = fixture();
        s.rename_local("A", "f", "B", "moved", false).unwrap();
        inject(&s, "root", "A", Action::Traverse, Decision::Deny).unwrap();
        assert_eq!(
            decide(&s, "root", "f", Action::Read).unwrap(),
            Decision::Deny
        );
    }
    #[test]
    fn precise_source_wins_over_a_late_observed_success() {
        let (_d, s) = fixture();
        inject(&s, "root", "f", Action::Read, Decision::Deny).unwrap();
        record_observed(&s, "root", "f", Action::Read, true).unwrap();
        assert_eq!(
            decide(&s, "root", "f", Action::Read).unwrap(),
            Decision::Deny
        );
    }
    #[test]
    fn denied_namespace_intent_does_not_block_independent_sibling() {
        let (_d, mut s) = fixture();
        s.create_local_directory("A", "new-a", 0o755).unwrap();
        let b = s.create_local_directory("B", "new-b", 0o755).unwrap();
        inject(&s, "root", "A", Action::CreateDir, Decision::Deny).unwrap();
        assert_eq!(s.scheduled_directory().unwrap().unwrap().job.node.id, b.id);
        assert_eq!(s.pending_namespace_operations().unwrap(), 2);
    }
    #[test]
    fn definite_refusal_preserves_bytes_and_uncertain_is_never_reset() {
        let (_d, mut s) = fixture();
        let op = s.save_local("f", b"unsent edit").unwrap();
        s.mark_upload_started(&op).unwrap();
        let targets = vec![("f".into(), Action::WriteContent)];
        refuse_operation(&s, &op, false, &targets, "f", Action::WriteContent).unwrap();
        assert_eq!(s.upload_job(&op).unwrap().state, "pending");
        assert!(targets_denied(&s, &targets).unwrap());
        let mut bytes = Vec::new();
        std::io::Read::read_to_end(&mut s.upload_job(&op).unwrap().file, &mut bytes).unwrap();
        assert_eq!(bytes, b"unsent edit");
        reset_session(&s).unwrap();
        assert!(!targets_denied(&s, &targets).unwrap());
        s.mark_upload_started(&op).unwrap();
        s.mark_upload_uncertain(&op).unwrap();
        refuse_operation(&s, &op, false, &targets, "f", Action::WriteContent).unwrap();
        assert_eq!(s.upload_job(&op).unwrap().state, "uncertain");
    }
    #[test]
    fn decisions_survive_restart_and_recheck_preserves_data() {
        let (d, mut s) = fixture();
        s.save_local("f", b"offline").unwrap();
        record_observed(&s, "root", "f", Action::WriteContent, false).unwrap();
        let revision = policy(&s, "root").unwrap().write_policy_revision;
        drop(s);
        let s = Store::open(d.path()).unwrap();
        assert_eq!(
            decide(&s, "root", "f", Action::WriteContent).unwrap(),
            Decision::Deny
        );
        reset_session(&s).unwrap();
        assert_eq!(
            decide(&s, "root", "f", Action::WriteContent).unwrap(),
            Decision::Unknown
        );
        assert!(policy(&s, "root").unwrap().write_policy_revision > revision);
        assert_eq!(s.pending_operations().unwrap(), 1);
    }
    #[test]
    fn schema24_migration_preserves_history_dirty_bytes_and_backup() {
        let (d, mut s) = fixture();
        let op = s.save_local("f", b"retained").unwrap();
        s.mark_upload_started(&op).unwrap();
        s.mark_upload_uncertain(&op).unwrap();
        let history = s
            .sync_history(&crate::sync_history::HistoryQuery::default())
            .unwrap()
            .records
            .len();
        drop(s);
        let db = Connection::open(d.path().join("state.sqlite")).unwrap();
        db.execute_batch(
            "DROP TABLE directory_permissions;DROP TABLE write_policy;PRAGMA user_version=24;",
        )
        .unwrap();
        drop(db);
        let s = Store::open(d.path()).unwrap();
        assert_eq!(
            s.db.query_row("PRAGMA user_version", [], |r| r.get::<_, i64>(0))
                .unwrap(),
            crate::STATE_SCHEMA_VERSION
        );
        let backup_prefix = format!("state-before-v{}-", crate::STATE_SCHEMA_VERSION);
        assert_eq!(
            s.sync_history(&crate::sync_history::HistoryQuery::default())
                .unwrap()
                .records
                .len(),
            history
        );
        assert_eq!(s.upload_job(&op).unwrap().state, "uncertain");
        let backup = std::fs::read_dir(d.path())
            .unwrap()
            .map(|e| e.unwrap().path())
            .find(|p| {
                p.file_name()
                    .unwrap()
                    .to_string_lossy()
                    .starts_with(backup_prefix.as_str())
            })
            .unwrap();
        let db = Connection::open(backup).unwrap();
        assert_eq!(
            db.query_row("PRAGMA user_version", [], |r| r.get::<_, i64>(0))
                .unwrap(),
            24
        );
        assert_eq!(
            db.query_row("PRAGMA integrity_check", [], |r| r.get::<_, String>(0))
                .unwrap(),
            "ok"
        );
    }
    #[test]
    fn schema25_migration_adds_reconcile_budget_columns_and_keeps_uncertain_upload() {
        let (d, mut s) = fixture();
        let op = s.save_local("f", b"retained").unwrap();
        s.mark_upload_started(&op).unwrap();
        s.mark_upload_uncertain(&op).unwrap();
        let history = s
            .sync_history(&crate::sync_history::HistoryQuery::default())
            .unwrap()
            .records
            .len();
        drop(s);
        let db = Connection::open(d.path().join("state.sqlite")).unwrap();
        db.execute_batch(
            // A real v25 database predates the v29 commit diagnostics view and
            // the v28 partial index, so remove both before simulating the old
            // shape: SQLite refuses to drop a column that still has dependents.
            "DROP VIEW IF EXISTS commit_diagnostics;DROP INDEX IF EXISTS operations_uncertain_reconcile;ALTER TABLE operations DROP COLUMN reconcile_attempts;ALTER TABLE operations DROP COLUMN reconcile_at;PRAGMA user_version=25;",
        )
        .unwrap();
        drop(db);
        let s = Store::open(d.path()).unwrap();
        assert_eq!(
            s.db.query_row("PRAGMA user_version", [], |r| r.get::<_, i64>(0))
                .unwrap(),
            crate::STATE_SCHEMA_VERSION
        );
        let backup_prefix = format!("state-before-v{}-", crate::STATE_SCHEMA_VERSION);
        let (attempts, at): (i64, i64) = s
            .db
            .query_row(
                "SELECT reconcile_attempts,reconcile_at FROM operations WHERE id=?1",
                [&op],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!((attempts, at), (0, 0));
        assert_eq!(s.upload_operations("uncertain").unwrap(), vec![op.clone()]);
        assert_eq!(s.pending_operations().unwrap(), 1);
        assert!(s.upload_reconcile_due(0).unwrap());
        assert_eq!(
            s.sync_history(&crate::sync_history::HistoryQuery::default())
                .unwrap()
                .records
                .len(),
            history
        );
        let backup = std::fs::read_dir(d.path())
            .unwrap()
            .map(|e| e.unwrap().path())
            .find(|p| {
                p.file_name()
                    .unwrap()
                    .to_string_lossy()
                    .starts_with(backup_prefix.as_str())
            })
            .unwrap();
        let db = Connection::open(backup).unwrap();
        assert_eq!(
            db.query_row("PRAGMA user_version", [], |r| r.get::<_, i64>(0))
                .unwrap(),
            25
        );
        assert_eq!(
            db.query_row("PRAGMA integrity_check", [], |r| r.get::<_, String>(0))
                .unwrap(),
            "ok"
        );
    }
    #[test]
    fn revision_conflict_does_not_replace_a_user_policy() {
        let (_d, s) = fixture();
        initialize_policy(&s, "root", LocalPolicy::FollowRemote, "legacy").unwrap();
        let rev = policy(&s, "root").unwrap().write_policy_revision;
        set_local_policy_checked(&s, "root", LocalPolicy::ExplicitReadOnly, rev).unwrap();
        assert!(set_local_policy_checked(&s, "root", LocalPolicy::FollowRemote, rev).is_err());
        assert_eq!(
            local_policy(&s, "root").unwrap(),
            LocalPolicy::ExplicitReadOnly
        );
    }
    #[test]
    fn malformed_pending_intent_keeps_the_existing_fail_closed_barrier() {
        let(_d,mut s)=fixture();s.create_local_directory("A","pending",0o755).unwrap();
        s.db.execute("UPDATE namespace_ops SET body='{}' WHERE state='pending'",[]).unwrap();
        assert!(s.next_namespace_operation().unwrap().is_none());
        assert!(s.queue_health().unwrap().schedule.unwrap().unresolved_barrier);
    }
    #[test]
    fn recheck_releases_only_the_typed_read_refusal_retry_gate() {
        let(_d,mut s)=fixture();let ticket=crate::DownloadTicket{id:"f".into(),version:"v1".into()};
        s.begin_download(&ticket).unwrap();s.fail_download(&ticket,crate::download_jobs::DownloadFailureKind::ServerRejected,None,1).unwrap();
        record_observed(&s,"root","f",Action::Read,false).unwrap();reset_session(&s).unwrap();
        assert!(s.download_failure("f").unwrap().is_none());
        assert_eq!(decide(&s,"root","f",Action::Read).unwrap(),Decision::Unknown);
        assert_eq!(s.db.query_row("SELECT state FROM transfers WHERE node='f'",[],|r|r.get::<_,String>(0)).unwrap(),"pending");
    }
    #[test]
    fn a_new_draft_rechecks_its_parents_create_permission() {
        let (_d, mut s) = fixture();
        let file = s.create_local_file("A", "draft", 0o600).unwrap();
        inject(&s, "root", &file.id, Action::WriteContent, Decision::Allow).unwrap();
        inject(&s, "root", "A", Action::CreateFile, Decision::Deny).unwrap();
        assert_eq!(decide(&s, "root", &file.id, Action::WriteContent).unwrap(), Decision::Deny);
        inject(&s, "root", "A", Action::CreateFile, Decision::Unknown).unwrap();
        assert_eq!(decide(&s, "root", &file.id, Action::WriteContent).unwrap(), Decision::Allow);
    }

    #[test]
    fn parent_self_denials_do_not_stall_child_remove_or_rename_journals() {
        let (_d, mut s) = fixture();
        inject(&s, "root", "A", Action::Delete, Decision::Deny).unwrap();
        inject(&s, "root", "A", Action::CreateDir, Decision::Deny).unwrap();
        inject(&s, "root", "f", Action::Delete, Decision::Allow).unwrap();
        s.unlink_local("A", "f", "f", false).unwrap();
        let job = s.scheduled_remove().unwrap().unwrap();
        assert_eq!(job.job.node.id, "f");
        inject(&s, "root", "A", Action::Traverse, Decision::Deny).unwrap();
        assert!(s.scheduled_remove().unwrap().is_none());
        inject(&s, "root", "A", Action::Traverse, Decision::Unknown).unwrap();
        inject(&s, "root", "f", Action::Delete, Decision::Deny).unwrap();
        assert!(s.scheduled_remove().unwrap().is_none());
        inject(&s, "root", "f", Action::Delete, Decision::Allow).unwrap();
        assert_eq!(s.scheduled_remove().unwrap().unwrap().job.operation, job.job.operation);

        let (_d, mut s) = fixture();
        inject(&s, "root", "A", Action::RenameSource, Decision::Deny).unwrap();
        inject(&s, "root", "B", Action::Delete, Decision::Deny).unwrap();
        s.rename_local("A", "f", "B", "moved", false).unwrap();
        assert!(s.scheduled_rename().unwrap().is_some());
        inject(&s, "root", "B", Action::RenameTarget, Decision::Deny).unwrap();
        assert!(s.scheduled_rename().unwrap().is_none());
        inject(&s, "root", "B", Action::RenameTarget, Decision::Unknown).unwrap();
        inject(&s, "root", "f", Action::RenameSource, Decision::Deny).unwrap();
        assert!(s.scheduled_rename().unwrap().is_none());
        inject(&s, "root", "f", Action::RenameSource, Decision::Allow).unwrap();
        assert!(s.scheduled_rename().unwrap().is_some());
    }

    /// The auto-learned denial path used only for an existing-node upload
    /// refused by the server's Create ACL test (wire code 0x9003). It must use
    /// the current policy revision, persist, and expire through the existing
    /// profile/recheck invalidation rather than a new mechanism.
    #[test]
    fn learned_upload_denial_uses_current_revision_persists_and_expires() {
        let (d, mut s) = fixture();
        let op = s.save_local("f", b"edited").unwrap();
        s.mark_upload_started(&op).unwrap();
        let targets = upload_targets(&s.node("f").unwrap());
        assert_eq!(targets, vec![("f".to_owned(), Action::WriteContent)]);
        let before = policy(&s, "root").unwrap().write_policy_revision;
        refuse_operation(&s, &op, false, &targets, "f", Action::WriteContent).unwrap();
        let after = policy(&s, "root").unwrap().write_policy_revision;
        assert_eq!(after, before + 1);
        assert_eq!(
            decide(&s, "root", "f", Action::WriteContent).unwrap(),
            Decision::Deny
        );
        let (source, revision, verified): (String, i64, Option<i64>) = s
            .db
            .query_row(
                "SELECT source,revision,verified_at FROM directory_permissions
                 WHERE root='root' AND node='f' AND action='write_content'",
                [],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .unwrap();
        assert_eq!(source, "observed");
        assert_eq!(revision, after as i64);
        assert!(verified.is_some());
        let rows = s
            .sync_history(&crate::sync_history::HistoryQuery::default())
            .unwrap()
            .records;
        assert!(rows
            .iter()
            .any(|r| r.result == "failed"
                && r.error_kind.as_deref() == Some("permission_denied")));
        // Durable across restart: an observed target denial is not in-memory.
        drop(s);
        let s = Store::open(d.path()).unwrap();
        assert_eq!(
            decide(&s, "root", "f", Action::WriteContent).unwrap(),
            Decision::Deny
        );
        // A native profile change invalidates observed evidence and bumps the
        // revision again; the user's policy and the target are untouched.
        let revision = policy(&s, "root").unwrap().write_policy_revision;
        invalidate_remote_permissions(&s, "root").unwrap();
        assert_eq!(exact(&s, "root", "f", Action::WriteContent).unwrap(), Decision::Unknown);
        assert_eq!(
            decide(&s, "root", "f", Action::WriteContent).unwrap(),
            Decision::Unknown
        );
        assert_eq!(
            policy(&s, "root").unwrap().write_policy_revision,
            revision + 1
        );
    }

    /// Fail closed: a definite-refusal caller cannot name a node/action pair
    /// that the operation did not authorize. This is the guard for a misbound
    /// worker reply; nothing may be learned or retried from it.
    #[test]
    fn definite_refusal_must_match_an_authorized_target() {
        let (_d, mut s) = fixture();
        let op = s.save_local("f", b"x").unwrap();
        s.mark_upload_started(&op).unwrap();
        let wrong = vec![("root".to_owned(), Action::CreateFile)];
        assert!(refuse_operation(&s, &op, false, &wrong, "f", Action::WriteContent).is_err());
        assert_eq!(
            exact(&s, "root", "f", Action::WriteContent).unwrap(),
            Decision::Unknown
        );
        assert_eq!(s.upload_job(&op).unwrap().state, "inflight");
    }

    /// Rename and remove responses keep the action-scoped hint. Their codes are
    /// ambiguous (profile read-only filtering rewrites 0x9004 to 0xa004) or
    /// unattributable (rename source/target, compound replace phases), so no
    /// directory_permissions row may be written.
    #[test]
    fn rename_and_remove_permission_hints_never_write_acl_rows() {
        let (_d, mut s) = fixture();
        s.rename_local("A", "f", "B", "moved", false).unwrap();
        let op = s.scheduled_rename().unwrap().unwrap().job.operation;
        s.mark_rename_started(&op).unwrap();
        mutation_permission_uncertain(&s, &op, true, 0x3002).unwrap();
        assert_eq!(s.scheduled_rename().unwrap().unwrap().state, "uncertain");
        assert_eq!(
            s.db.query_row("SELECT count(*) FROM directory_permissions", [], |r| r
                .get::<_, i64>(0))
                .unwrap(),
            0
        );

        let (_d, mut s) = fixture();
        s.unlink_local("A", "f", "f", false).unwrap();
        let op = s.scheduled_remove().unwrap().unwrap().job.operation;
        s.mark_remove_started(&op).unwrap();
        mutation_permission_uncertain(&s, &op, true, 0xa004).unwrap();
        assert_eq!(s.scheduled_remove().unwrap().unwrap().state, "uncertain");
        assert_eq!(
            s.db.query_row("SELECT count(*) FROM directory_permissions", [], |r| r
                .get::<_, i64>(0))
                .unwrap(),
            0
        );
    }

    /// #26 acceptance: the ACL page metrics are consumable from product code,
    /// not only as `DRIVE_NATIVE_FS_DIAGNOSTICS=1` stages. A batched page must
    /// advance `batched_pages`/`batched_entries`; the writer-local memo must
    /// report its miss and hit in `cache_misses`/`cache_hits`.
    #[test]
    fn acl_metrics_track_batched_projection_and_permission_cache() {
        let record = |id: &str, ino: u64, directory: bool| crate::filesystem::PosixRecord {
            id: id.into(),
            ino,
            directory,
            size: if directory { 0 } else { 3 },
            mtime: 1,
            atime: 0,
            atime_nsec: 0,
            mode: if directory { 0o755 } else { 0o644 },
            unlinked: false,
            blocks: 0,
            cacheable: true,
        };
        let (_d, s) = fixture();
        let mut records = vec![record("A", 2, true), record("f", 3, false)];
        let before = acl_metrics(&s);
        let allowed = project_read_page(&s, "root", &mut records).unwrap();
        assert_eq!(allowed, vec![true, true]);
        let after = acl_metrics(&s);
        assert_eq!(
            after.batched_pages,
            before.batched_pages + 1,
            "one batched ACL page must be counted"
        );
        assert_eq!(
            after.batched_entries,
            before.batched_entries + 2,
            "both projected entries must be counted"
        );
        assert!(
            after.cache_misses > before.cache_misses,
            "the parent traversal decision must fill the permission memo"
        );
        let mut again = vec![record("A", 2, true), record("f", 3, false)];
        project_read_page(&s, "root", &mut again).unwrap();
        let second = acl_metrics(&s);
        assert!(
            second.cache_hits > after.cache_hits,
            "an unchanged store generation must reuse the memoized traversal decision"
        );
        assert_eq!(second.batched_entries, before.batched_entries + 4);
    }

}
