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
    pub fn desktop_status_seeds_for(&self, roots: &[String]) -> Result<Vec<StatusSeed>> {
        if roots.is_empty() {
            return Ok(Vec::new());
        }
        let roots = serde_json::to_string(roots)?;
        self.collect_seeds(&scoped_seed_sql(), &[&roots])
    }
    fn collect_seeds(&self, sql: &str, params: &[&str]) -> Result<Vec<StatusSeed>> {
        let mut q = self.db.prepare(sql)?;
        Ok(q
            .query_map(rusqlite::params_from_iter(params.iter().copied()), |r| {
                Ok(StatusSeed {
                    id: r.get(0)?,
                    parent: r.get(1)?,
                    directory: r.get(2)?,
                    pinned: r.get(3)?,
                    overlay: r.get(4)?,
                    detailed: r.get(5)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?)
    }
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
