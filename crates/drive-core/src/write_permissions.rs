//! Effective decisions, not invented NAS ACL inheritance. Unknown permits an
//! attempt; only a confirmed remote outcome may become an observed decision.
use crate::{Error, Result, Store};
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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
    exact(store, root, node, action)
}
/// Distinguishes whole-mount policy from a target-specific refusal.
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
    let tx = store.db.unchecked_transaction()?;
    // Clear the terminal retry gate paired with a typed Read refusal as well.
    tx.execute("UPDATE transfers SET state='pending',error=NULL WHERE direction='download' AND state IN ('failed','retry')
      AND json_valid(error) AND json_extract(error,'$.kind')='server_rejected' AND json_extract(error,'$.server_code') IS NULL
      AND node IN (SELECT node FROM directory_permissions WHERE root=?1 AND action='read' AND state='denied' AND source='observed')",[root])?;
    tx.execute(
        "UPDATE write_policy SET revision=revision+1,updated_at=unixepoch() WHERE root=?1",
        [root],
    )?;
    tx.execute("UPDATE directory_permissions SET state='unknown',verified_at=NULL,revision=(SELECT revision FROM write_policy WHERE root=?1) WHERE root=?1 AND source='observed'",[root])?;
    tx.commit()?;
    Ok(())
}
pub fn reset_session(store: &Store) -> Result<()> {
    if let Some(root) = filesystem_root(store)? {
        reset_observed(store, &root)?;
    }
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
                targets.push((p, Action::RenameSource));
            }
            targets.push((j.node.id, Action::RenameSource));
            targets.push((j.parent_node.id, Action::RenameTarget));
        }
        "remove" => {
            let j: crate::removals::RemoveJob = serde_json::from_str(body)?;
            if let Some(p) = j.node.parent {
                targets.push((p.clone(), Action::Delete));
                if j.node.directory {
                    targets.push((p, Action::CreateDir));
                }
            }
            targets.push((j.node.id, Action::Delete));
        }
        "replace" => {
            let j: crate::replacements::ReplaceJob = serde_json::from_str(body)?;
            if let Some(p) = j.source.parent {
                targets.push((p, Action::RenameSource));
            }
            targets.push((j.source.id, Action::RenameSource));
            if let Some(p) = j.target.parent {
                targets.push((p.clone(), Action::RenameTarget));
                targets.push((p, Action::Delete));
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
    tx.execute(&format!("UPDATE {table} SET state=CASE state WHEN 'cleanup_inflight' THEN 'cleanup_pending' ELSE 'pending' END WHERE id=?1 AND state IN ('inflight','cleanup_inflight')"),[operation])?;
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
            metadata: None,
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
        assert_eq!(crate::STATE_SCHEMA_VERSION, 25);
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
                    .starts_with("state-before-v25-")
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

}
