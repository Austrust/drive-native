//! Bulk read-only seeds and local details for desktop aggregation. No inode
//! allocation or body reads.
use crate::Result;
use crate::{Store, download_jobs::DownloadFailure, state::Status};
use rusqlite::params;
use std::collections::{HashMap, HashSet};
#[derive(Clone)]
pub struct StatusSeed {
    pub id: String,
    pub parent: Option<String>,
    pub directory: bool,
    pub pinned: bool,
    pub overlay: bool,
    pub detailed: bool,
}
/// Result of [`Store::desktop_status_acl_refresh`]: the scoped re-read an
/// `access_denied` journal change needs.
#[derive(Clone)]
pub struct AclRefresh {
    /// Every id whose visibility may have changed, i.e. the closure of the
    /// denied rows under the `access_blocked` edges.
    pub affected: Vec<String>,
    /// Currently visible seeds inside `affected`, plus the ancestor chains that
    /// keep inherited flags resolvable.
    pub seeds: Vec<StatusSeed>,
}
/// Which read answers one desktop status batch.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum SeedScope {
    /// The batch's scoped set is the whole visible namespace, so the linear full
    /// read is cheaper. A root page batch can still name a strict subset of the
    /// rows the full read returns: a node whose only path to the root crosses a
    /// tombstoned ancestor is not reachable from the requested (visible) roots,
    /// while the full visibility rule includes it. Those extra rows are sibling
    /// branches of the requested page, never rendered by the desktop tree, so the
    /// cost win is taken without changing any requested row.
    Full,
    /// The scoped recursive read of these JSON-encoded roots.
    Scoped(String),
}
/// Projection shared by the full and scoped reads so both report identical rules.
const SEED_COLUMNS: &str = "n.id,n.parent,n.directory,n.pinned,EXISTS(SELECT 1 FROM namespace_overlay o WHERE o.node=n.id), (n.blob IS NOT NULL OR n.dirty_blob IS NOT NULL OR n.conflict!=0 OR EXISTS(SELECT 1 FROM drafts d WHERE d.node=n.id) OR EXISTS(SELECT 1 FROM transfers t WHERE t.node=n.id AND t.direction='download'))";
/// Visibility rules shared by the full and scoped reads.
const SEED_FILTERS: &str = "n.deleted=0 AND n.id NOT IN (SELECT id FROM access_blocked) AND NOT EXISTS(SELECT 1 FROM nodes l JOIN local_creations c ON c.node=l.id WHERE l.parent=n.parent AND l.name=n.name AND l.id!=n.id AND l.deleted=0)";
impl Store {
    /// Full visible namespace. Only rebuild/reconciliation callers should use this;
    /// interactive status batches use [`Self::desktop_status_seeds_for`].
    pub fn desktop_status_seeds(&self) -> Result<Vec<StatusSeed>> {
        self.collect_seeds(&format!("SELECT {SEED_COLUMNS} FROM nodes n WHERE {SEED_FILTERS}"), &[])
    }
    /// Seeds for the union of the requested roots' subtrees and the roots' ancestor
    /// chains. Ancestor rows keep inherited pin/overlay flags resolvable; sibling
    /// branches are never read.
    ///
    /// The tree walk is put first with CROSS JOIN so SQLite uses the recursive
    /// frontier as the outer loop (see docs/CANDIDATE_QUERY_ORDER.md) and the
    /// (parent,name,deleted) index serves each downward step.
    ///
    /// A namespace root (`parent IS NULL`) in `roots`, or a batch naming every
    /// visible child of the namespace root, makes the scoped walk expand the whole
    /// namespace one index lookup at a time. That costs several times the linear
    /// scan (issue #67: 755 ms / 70.8 MB for the 147-directory root page at 294k
    /// nodes), so those batches use the full projection instead. The requested
    /// rows are identical; the root-page batch may additionally return nodes whose
    /// only root path crosses a tombstoned ancestor, exactly as the full read
    /// always has (see [`SeedScope::Full`]). Consumers render only the requested
    /// subtrees, so those extra rows never change a reply.
    pub fn desktop_status_seeds_for(&self, roots: &[String]) -> Result<Vec<StatusSeed>> {
        if roots.is_empty() {
            return Ok(Vec::new());
        }
        match self.seed_scope(roots)? {
            SeedScope::Full => self.desktop_status_seeds(),
            SeedScope::Scoped(encoded) => {
                self.collect_seeds(&scoped_seed_sql(), &[&encoded])
            }
        }
    }
    /// Decide which read answers a status batch.
    fn seed_scope(&self, roots: &[String]) -> Result<SeedScope> {
        let encoded = serde_json::to_string(roots)?;
        let namespace_root: bool = self.db.query_row(
            "SELECT EXISTS(SELECT 1 FROM nodes WHERE parent IS NULL AND id IN (SELECT value FROM json_each(?1)))",
            [&encoded],
            |row| row.get(0),
        )?;
        if namespace_root {
            return Ok(SeedScope::Full);
        }
        if self.covers_namespace_children(roots, &encoded)? {
            return Ok(SeedScope::Full);
        }
        Ok(SeedScope::Scoped(encoded))
    }
    /// True when `roots` names exactly every visible child of the namespace root.
    ///
    /// That is the desktop root page: a batch of all top-level directories (and
    /// files) requested by the GUI at once. The scoped read would then expand the
    /// entire namespace through one recursive subtree walk per child, which is
    /// several times the linear scan (issue #67: 755 ms / 70.8 MB for 147
    /// directories at 294k nodes). `up` additionally contributes the root row, so
    /// the scoped row set equals the full projection and only the cost improves.
    ///
    /// Any ambiguity (duplicate ids, missing/extra children, a namespace with more
    /// than one root) keeps the historical scoped read; a false negative only
    /// misses the optimization, never a row.
    fn covers_namespace_children(&self, roots: &[String], encoded: &str) -> Result<bool> {
        let unique: HashSet<&str> = roots.iter().map(String::as_str).collect();
        if unique.len() != roots.len() {
            return Ok(false);
        }
        let namespaces: Vec<String> = {
            let mut q = self
                .db
                .prepare("SELECT id FROM nodes WHERE parent IS NULL AND deleted=0")?;
            q.query_map([], |row| row.get::<_, String>(0))?
                .collect::<std::result::Result<Vec<_>, _>>()?
        };
        if namespaces.len() != 1 {
            return Ok(false);
        }
        let sql = format!(
            "SELECT (SELECT count(*) FROM nodes n WHERE n.parent=?1 AND {SEED_FILTERS}), \
                    (SELECT count(*) FROM nodes n WHERE n.parent=?1 \
                       AND n.id IN (SELECT value FROM json_each(?2)) AND {SEED_FILTERS})"
        );
        let (children, named_children): (i64, i64) = self.db.query_row(
            &sql,
            params![&namespaces[0], encoded],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )?;
        Ok(children == named_children && usize::try_from(children).ok() == Some(roots.len()))
    }
    /// One seed row for `id`, or `None` when the node is missing, deleted,
    /// access-denied or shadowed by a local creation at the same name.
    ///
    /// This is the incremental-refresh counterpart of [`Self::desktop_status_seeds`]:
    /// callers re-read only the nodes named by a committed journal change.
    pub fn desktop_status_seed(&self, id: &str) -> Result<Option<StatusSeed>> {
        let mut q = self.db.prepare(&format!(
            "SELECT {SEED_COLUMNS} FROM nodes n WHERE n.id=?1 AND {SEED_FILTERS}"
        ))?;
        let mut rows = q.query_map([id], seed_from_row)?;
        match rows.next() {
            Some(row) => Ok(Some(row?)),
            None => Ok(None),
        }
    }
    /// Scoped re-read for `access_denied` journal invalidations.
    ///
    /// `access_blocked` is a recursive view over three edges: a denied id, the
    /// local `nodes.parent` edge and the overlay `remote.parent` edge. A denial
    /// therefore changes visibility only inside the closure below the named id
    /// in exactly that graph, plus the ancestor chains whose inherited flags may
    /// change. Re-reading that scope keeps an ACL edit proportional to the
    /// affected subtree instead of the namespace size.
    ///
    /// Returns `Ok(None)` when the closure is wider than `limit` ids. The outer
    /// `LIMIT` also stops the recursive expansion early, so an oversized ACL
    /// change never materialises the whole namespace; callers must then fall back
    /// to a full rebuild. `seeds` is never wider than `affected` plus its
    /// ancestor chains.
    pub fn desktop_status_acl_refresh(
        &self,
        denied: &[String],
        limit: usize,
    ) -> Result<Option<AclRefresh>> {
        if denied.is_empty() {
            return Ok(Some(AclRefresh {
                affected: Vec::new(),
                seeds: Vec::new(),
            }));
        }
        let denied = serde_json::to_string(denied)?;
        let cap = limit.saturating_add(1) as i64;
        let affected = {
            let mut q = self.db.prepare_cached(ACL_SCOPE_SQL)?;
            q.query_map(params![&denied, cap], |row| row.get::<_, String>(0))?
                .collect::<std::result::Result<Vec<_>, _>>()?
        };
        if affected.len() > limit {
            return Ok(None);
        }
        let seeds = {
            let sql = acl_seed_sql();
            let mut q = self.db.prepare_cached(&sql)?;
            q.query_map(params![&denied], seed_from_row)?
                .collect::<std::result::Result<Vec<_>, _>>()?
        };
        Ok(Some(AclRefresh { affected, seeds }))
    }
    /// Ids of every live node at exactly `(parent, name)`, without the
    /// visibility rules.
    ///
    /// Shadowing is a property of the location, not of a node: inserting or
    /// removing a `local_creations` row hides or reveals every node stored at
    /// the same parent/name. Turning a location change into point reads means
    /// first enumerating every node there (`None` = namespace root) and then
    /// re-reading each node with [`Self::desktop_status_seed`]: the shadowed
    /// remote row is not returned by any visible seed query, but it still has to
    /// be revisited so a stale projection row can be removed.
    pub fn desktop_status_ids_at(&self, parent: Option<&str>, name: &str) -> Result<Vec<String>> {
        let sql = match parent {
            Some(_) => "SELECT n.id FROM nodes n WHERE n.parent=?1 AND n.name=?2 AND n.deleted=0",
            None => "SELECT n.id FROM nodes n WHERE n.parent IS NULL AND n.name=?1 AND n.deleted=0",
        };
        let mut q = self.db.prepare(sql)?;
        let params: &[&str] = match parent {
            Some(parent) => &[parent, name],
            None => &[name],
        };
        Ok(q.query_map(rusqlite::params_from_iter(params.iter().copied()), |r| {
            r.get(0)
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?)
    }
    fn collect_seeds(&self, sql: &str, params: &[&str]) -> Result<Vec<StatusSeed>> {
        let mut q = self.db.prepare(sql)?;
        Ok(q
            .query_map(
                rusqlite::params_from_iter(params.iter().copied()),
                seed_from_row,
            )?
            .collect::<std::result::Result<Vec<_>, _>>()?)
    }
}

fn seed_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<StatusSeed> {
    Ok(StatusSeed {
        id: row.get(0)?,
        parent: row.get(1)?,
        directory: row.get(2)?,
        pinned: row.get(3)?,
        overlay: row.get(4)?,
        detailed: row.get(5)?,
    })
}

/// Local detail of one node, as consumed by the desktop status aggregation.
/// `offline` mirrors a successful `open_cached` or dirty-draft open; `failure`
/// is the currently effective download failure record.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StatusDetail {
    pub status: Status,
    pub offline: bool,
    pub failure: Option<DownloadFailure>,
}

struct NodeFacts {
    blob: Option<String>,
    dirty_blob: Option<String>,
    local: Option<String>,
    remote: String,
    dirty: bool,
    conflict: bool,
    deleted: bool,
    base_version: Option<String>,
}

/// Batch equivalent of the per-node desktop detail read: one SELECT family for
/// a whole status batch instead of three to four point queries per node.
///
/// `nodes` is `(id, directory)`. The returned map only contains nodes whose
/// detail is fully known. An id is absent when the single-node path must run
/// instead (the node vanished, a blob file is missing, a draft is unopenable, or
/// a failure record does not parse), so callers keep the single-node error
/// reporting for those ids and for the whole batch when this call errors.
///
/// Query order matters: the recursive walks start from `json_each` so the
/// `nodes` primary key serves every frontier step, matching the scoped seed
/// query. Only the requested ids are read; nothing outside the batch is walked.
pub fn desktop_status_details(
    store: &Store,
    nodes: &[(String, bool)],
) -> Result<HashMap<String, StatusDetail>> {
    if nodes.is_empty() {
        return Ok(HashMap::new());
    }
    let ids: Vec<&str> = nodes.iter().map(|(id, _)| id.as_str()).collect();
    let ids = serde_json::to_string(&ids)?;

    let mut facts: HashMap<String, NodeFacts> = HashMap::with_capacity(nodes.len());
    {
        let mut q = store.db.prepare(
            "SELECT n.id,n.blob,n.dirty_blob,n.local_version,n.remote_version,\
             n.dirty_blob IS NOT NULL,n.conflict,n.deleted,n.base_version \
             FROM json_each(?1) j CROSS JOIN nodes n ON n.id=j.value",
        )?;
        let mut rows = q.query([&ids])?;
        while let Some(row) = rows.next()? {
            facts.insert(
                row.get(0)?,
                NodeFacts {
                    blob: row.get(1)?,
                    dirty_blob: row.get(2)?,
                    local: row.get(3)?,
                    remote: row.get(4)?,
                    dirty: row.get(5)?,
                    conflict: row.get(6)?,
                    deleted: row.get(7)?,
                    base_version: row.get(8)?,
                },
            );
        }
    }

    // Dirty draft rows. `open_dirty_draft` reads the first row for the node, so
    // keep the first token per node and ignore any extras.
    let mut draft_tokens: HashMap<String, String> = HashMap::new();
    {
        let mut q = store.db.prepare(
            "SELECT d.node,d.token FROM json_each(?1) j JOIN drafts d ON d.node=j.value \
             WHERE d.dirty=1 ORDER BY d.node,d.rowid",
        )?;
        let mut rows = q.query([&ids])?;
        while let Some(row) = rows.next()? {
            draft_tokens.entry(row.get(0)?).or_insert(row.get(1)?);
        }
    }
    let mut draft_conflict_nodes: HashSet<String> = HashSet::new();
    {
        let mut q = store.db.prepare(
            "SELECT DISTINCT d.node FROM json_each(?1) j JOIN drafts d ON d.node=j.value \
             JOIN nodes n ON n.id=d.node WHERE d.dirty=1 \
             AND NOT EXISTS(SELECT 1 FROM local_unlinks WHERE node=d.node) \
             AND (d.base!=n.remote_version OR n.deleted=1)",
        )?;
        let mut rows = q.query([&ids])?;
        while let Some(row) = rows.next()? {
            draft_conflict_nodes.insert(row.get(0)?);
        }
    }

    // `namespace::blocks_upload` for every requested node: the ids themselves
    // plus every `nodes.parent` ancestor, then an overlay lookup on that path.
    let mut blocking_roots: HashSet<String> = HashSet::new();
    {
        let mut q = store.db.prepare(
            "WITH RECURSIVE parents(root,id) AS (\
             SELECT j.value,j.value FROM json_each(?1) j \
             UNION \
             SELECT p.root,n.parent FROM nodes n JOIN parents p ON p.id=n.id WHERE n.parent IS NOT NULL\
             ) SELECT DISTINCT p.root FROM parents p \
             WHERE EXISTS(SELECT 1 FROM namespace_overlay o WHERE o.node=p.id)",
        )?;
        let mut rows = q.query([&ids])?;
        while let Some(row) = rows.next()? {
            blocking_roots.insert(row.get(0)?);
        }
    }

    // `Store::is_pinned` for every requested node (ancestor-or-self pinned).
    let mut pinned_roots: HashSet<String> = HashSet::new();
    {
        let mut q = store.db.prepare(
            "WITH RECURSIVE ancestors(root,id,parent,pinned) AS (\
             SELECT j.value,n.id,n.parent,n.pinned FROM json_each(?1) j JOIN nodes n ON n.id=j.value \
             UNION \
             SELECT a.root,n.id,n.parent,n.pinned FROM nodes n JOIN ancestors a ON n.id=a.parent\
             ) SELECT root FROM ancestors GROUP BY root HAVING MAX(pinned)=1",
        )?;
        let mut rows = q.query([&ids])?;
        while let Some(row) = rows.next()? {
            pinned_roots.insert(row.get(0)?);
        }
    }

    // `Store::is_inaccessible` for every requested node. The common case has an
    // empty access_denied table, so skip the ancestor walk entirely.
    let mut inaccessible_roots: HashSet<String> = HashSet::new();
    let access_denied: bool =
        store
            .db
            .query_row("SELECT EXISTS(SELECT 1 FROM access_denied)", [], |r| r.get(0))?;
    if access_denied {
        let mut q = store.db.prepare(
            "WITH RECURSIVE ancestors(root,id) AS (\
             SELECT j.value,n.id FROM json_each(?1) j JOIN nodes n ON n.id=j.value \
             UNION \
             SELECT a.root,n.parent FROM nodes n JOIN ancestors a ON n.id=a.id WHERE n.parent IS NOT NULL \
             UNION \
             SELECT a.root,json_extract(o.remote,'$.parent') FROM namespace_overlay o \
             JOIN ancestors a ON o.node=a.id WHERE json_extract(o.remote,'$.parent') IS NOT NULL\
             ) SELECT DISTINCT a.root FROM ancestors a JOIN access_denied d ON d.node=a.id",
        )?;
        let mut rows = q.query([&ids])?;
        while let Some(row) = rows.next()? {
            inaccessible_roots.insert(row.get(0)?);
        }
    }

    // `Store::download_failure` for every requested node: the first transfer row
    // is the one the single-node read returns.
    let mut failure_errors: HashMap<String, String> = HashMap::new();
    {
        let mut q = store.db.prepare(
            "SELECT t.node,t.error FROM transfers t \
             JOIN json_each(?1) j ON j.value=t.node \
             JOIN nodes n ON n.id=t.node AND n.remote_version=t.version \
             WHERE t.direction='download' AND t.error IS NOT NULL AND n.deleted=0 \
             AND (n.blob IS NULL OR n.local_version!=n.remote_version) ORDER BY t.rowid",
        )?;
        let mut rows = q.query([&ids])?;
        while let Some(row) = rows.next()? {
            failure_errors.entry(row.get(0)?).or_insert(row.get(1)?);
        }
    }

    let blobs = store.root.join("blobs");
    let drafts = store.root.join("drafts");
    let mut out: HashMap<String, StatusDetail> = HashMap::with_capacity(facts.len());
    for (id, directory) in nodes {
        let Some(facts) = facts.get(id) else {
            continue;
        };
        let conflict = facts.conflict || draft_conflict_nodes.contains(id);
        let status = if conflict {
            Status::Conflict
        } else if facts.deleted {
            Status::Deleted
        } else if facts.dirty || draft_tokens.contains_key(id) || blocking_roots.contains(id) {
            Status::PendingUpload
        } else {
            let pinned = pinned_roots.contains(id);
            match facts.blob.as_deref() {
                Some(blob) => {
                    if !blobs.join(blob).is_file() {
                        continue;
                    }
                    if facts.local.as_deref() != Some(facts.remote.as_str()) {
                        Status::Updating
                    } else if pinned {
                        Status::PinnedComplete
                    } else {
                        Status::LocalComplete
                    }
                }
                None => {
                    if pinned {
                        Status::Pinning
                    } else {
                        Status::OnlineOnly
                    }
                }
            }
        };
        let offline = if *directory {
            false
        } else {
            let mut offline = false;
            if let Some(token) = draft_tokens.get(id) {
                if uuid::Uuid::parse_str(token).is_err()
                    || std::fs::File::open(drafts.join(token)).is_err()
                {
                    // The single-node read turns this into a detail error.
                    continue;
                }
                offline = true;
            }
            // `open_cached`: readable coalesced body with a known version, unless
            // the node is inaccessible or a non-conflicted delete.
            if !offline && !inaccessible_roots.contains(id) && !(facts.deleted && !facts.conflict) {
                let body = facts.dirty_blob.as_deref().or(facts.blob.as_deref());
                let version = facts
                    .base_version
                    .as_deref()
                    .or(facts.local.as_deref());
                if let (Some(blob), Some(_)) = (body, version) {
                    if std::fs::File::open(blobs.join(blob)).is_ok() {
                        offline = true;
                    }
                }
            }
            offline
        };
        let failure = if *directory || matches!(&status, Status::Conflict | Status::PendingUpload) {
            None
        } else if let Some(raw) = failure_errors.get(id) {
            let Ok(failure) = serde_json::from_str::<DownloadFailure>(raw) else {
                // The single-node read reports unsupported records as errors.
                continue;
            };
            if failure.schema != 1 || failure.attempts == 0 {
                continue;
            }
            Some(failure)
        } else {
            None
        };
        out.insert(
            id.clone(),
            StatusDetail {
                status,
                offline,
                failure,
            },
        );
    }
    Ok(out)
}

/// Mount-relative location of one namespace node, resolved by
/// [`desktop_status_ancestry_paths`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StatusPath {
    /// Mount-relative path; `.` is the namespace root.
    pub path: String,
    pub directory: bool,
    /// Mount-relative paths of every resolvable ancestor, nearest first.
    pub ancestors: Vec<String>,
}

/// Resolve `ids` (plus the ancestor chains they need) to mount-relative paths
/// with one recursive read. Missing nodes — deleted, repository-only or
/// access-filtered — are omitted; a journal consumer keeps the recorded old
/// parent/name for those.
pub fn desktop_status_ancestry_paths(
    store: &Store,
    ids: &[String],
) -> Result<HashMap<String, StatusPath>> {
    let mut out = HashMap::new();
    if ids.is_empty() {
        return Ok(out);
    }
    let encoded = serde_json::to_string(ids)?;
    let sql = "WITH RECURSIVE targets(id) AS (SELECT value FROM json_each(?1)), \
         ancestry(id,parent,name,directory) AS ( \
           SELECT n.id,n.parent,n.name,n.directory FROM nodes n WHERE n.id IN (SELECT id FROM targets) \
           UNION SELECT n.id,n.parent,n.name,n.directory FROM nodes n JOIN ancestry a ON n.id=a.parent WHERE a.parent IS NOT NULL) \
         SELECT id,parent,name,directory FROM ancestry";
    let mut facts: HashMap<String, (Option<String>, String, bool)> = HashMap::new();
    {
        let mut query = store.db.prepare_cached(sql)?;
        let mut rows = query.query_map([&encoded], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, Option<String>>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, i64>(3)? != 0,
            ))
        })?;
        for row in rows {
            let (id, parent, name, directory) = row?;
            facts.insert(id, (parent, name, directory));
        }
    }
    let mut resolved: HashMap<String, Option<String>> = HashMap::new();
    for id in ids {
        let Some((parent, _, directory)) = facts.get(id) else {
            continue;
        };
        let Some(path) = resolve_status_path(&facts, id, &mut resolved, 0) else {
            continue;
        };
        let mut ancestors = Vec::new();
        let mut current = parent.clone();
        let mut guard = 0usize;
        while let Some(pid) = current {
            if guard >= 128 {
                break;
            }
            guard += 1;
            let Some(path) = resolve_status_path(&facts, &pid, &mut resolved, 0) else {
                break;
            };
            ancestors.push(path);
            current = facts.get(&pid).and_then(|(parent, _, _)| parent.clone());
        }
        out.insert(
            id.clone(),
            StatusPath {
                path,
                directory: *directory,
                ancestors,
            },
        );
    }
    Ok(out)
}

fn resolve_status_path(
    facts: &HashMap<String, (Option<String>, String, bool)>,
    id: &str,
    memo: &mut HashMap<String, Option<String>>,
    depth: u8,
) -> Option<String> {
    if let Some(cached) = memo.get(id) {
        return cached.clone();
    }
    if depth > 128 {
        return None;
    }
    let (parent, name, _) = facts.get(id)?;
    let path = match parent {
        None => ".".to_string(),
        Some(parent) => {
            let base = resolve_status_path(facts, parent, memo, depth + 1)?;
            if base == "." {
                name.clone()
            } else {
                format!("{base}/{name}")
            }
        }
    };
    memo.insert(id.to_owned(), Some(path.clone()));
    Some(path)
}

/// Scoped seed SQL, kept separate so tests can inspect its query plan.
fn scoped_seed_sql() -> String {
    format!(
        "WITH RECURSIVE roots(id) AS (SELECT value FROM json_each(?1)), \
         down(id) AS (SELECT id FROM roots UNION SELECT n.id FROM down d CROSS JOIN nodes n ON n.parent=d.id), \
         up(id) AS (SELECT id FROM roots UNION SELECT n.parent FROM up u CROSS JOIN nodes n ON n.id=u.id WHERE n.parent IS NOT NULL), \
         scope(id) AS MATERIALIZED (SELECT id FROM down UNION SELECT id FROM up) \
         SELECT {SEED_COLUMNS} FROM scope s CROSS JOIN nodes n ON n.id=s.id WHERE {SEED_FILTERS}"
    )
}

/// Closure of the named denial rows under the two edges `access_blocked` propagates
/// over (local `nodes.parent`, overlay `remote.parent`). The outer `LIMIT` caps the
/// output so an oversized denial is detected without first materialising the whole
/// namespace.
const ACL_SCOPE_SQL: &str = "WITH RECURSIVE affected(id) AS (SELECT value FROM json_each(?1) \
     UNION SELECT n.id FROM affected a CROSS JOIN nodes n ON n.parent=a.id \
     UNION SELECT o.node FROM affected a CROSS JOIN namespace_overlay o ON json_extract(o.remote,'$.parent')=a.id) \
     SELECT id FROM affected LIMIT ?2";

/// Visible seeds inside the ACL closure plus their ancestor chains.
fn acl_seed_sql() -> String {
    format!(
        "WITH RECURSIVE affected(id) AS (SELECT value FROM json_each(?1) \
         UNION SELECT n.id FROM affected a CROSS JOIN nodes n ON n.parent=a.id \
         UNION SELECT o.node FROM affected a CROSS JOIN namespace_overlay o ON json_extract(o.remote,'$.parent')=a.id), \
         up(id) AS (SELECT id FROM affected UNION SELECT n.parent FROM up u CROSS JOIN nodes n ON n.id=u.id WHERE n.parent IS NOT NULL) \
         SELECT {SEED_COLUMNS} FROM up s CROSS JOIN nodes n ON n.id=s.id WHERE {SEED_FILTERS}"
    )
}

#[cfg(test)]
#[path = "desktop_status_tests.rs"]
mod tests;
