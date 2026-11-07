//! Bulk read-only seeds for desktop aggregation. No inode allocation or body reads.
use crate::{Result, Store};
#[derive(Clone)]
pub struct StatusSeed {
    pub id: String,
    pub parent: Option<String>,
    pub directory: bool,
    pub pinned: bool,
    pub overlay: bool,
    pub detailed: bool,
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
    /// A namespace root (`parent IS NULL`) in `roots` means the scoped walk would
    /// cover the whole database one index lookup at a time. That costs several
    /// times the linear scan on a large namespace, so those batches use the full
    /// projection instead; the returned row set is the same because the scoped
    /// set is then the entire namespace.
    pub fn desktop_status_seeds_for(&self, roots: &[String]) -> Result<Vec<StatusSeed>> {
        if roots.is_empty() {
            return Ok(Vec::new());
        }
        let roots = serde_json::to_string(roots)?;
        let namespace_root: bool = self.db.query_row(
            "SELECT EXISTS(SELECT 1 FROM nodes WHERE parent IS NULL AND id IN (SELECT value FROM json_each(?1)))",
            [&roots],
            |row| row.get(0),
        )?;
        if namespace_root {
            return self.desktop_status_seeds();
        }
        self.collect_seeds(&scoped_seed_sql(), &[&roots])
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

#[cfg(test)]
#[path = "desktop_status_tests.rs"]
mod tests;
