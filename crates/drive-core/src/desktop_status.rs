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
impl Store {
    pub fn desktop_status_seeds(&self) -> Result<Vec<StatusSeed>> {
        let mut q=self.db.prepare("SELECT n.id,n.parent,n.directory,n.pinned,EXISTS(SELECT 1 FROM namespace_overlay o WHERE o.node=n.id), (n.blob IS NOT NULL OR n.dirty_blob IS NOT NULL OR n.conflict!=0 OR EXISTS(SELECT 1 FROM drafts d WHERE d.node=n.id) OR EXISTS(SELECT 1 FROM transfers t WHERE t.node=n.id AND t.direction='download')) FROM nodes n WHERE n.deleted=0 AND n.id NOT IN (SELECT id FROM access_blocked) AND NOT EXISTS(SELECT 1 FROM nodes l JOIN local_creations c ON c.node=l.id WHERE l.parent=n.parent AND l.name=n.name AND l.id!=n.id AND l.deleted=0)")?;
        Ok(q.query_map([], |r| {
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
