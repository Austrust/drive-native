//! Reuse one bulk metadata read per status batch instead of N recursive SQL walks.
use drive_core::{desktop_status::StatusSeed, Store};
use std::{
    collections::{HashMap, HashSet},
    io,
};
type Detail = (
    drive_core::Status,
    bool,
    Option<drive_core::download_jobs::DownloadFailure>,
);
pub(super) struct StatusTree {
    details: HashMap<String, Result<Detail, ()>>,
    rows: HashMap<String, StatusSeed>,
    children: HashMap<String, Vec<(String, bool)>>,
    inherited: HashMap<String, (bool, bool)>,
}
impl StatusTree {
    pub fn load(store: &Store, roots: &[String]) -> io::Result<Self> {
        let rows = store.desktop_status_seeds().map_err(io::Error::other)?;
        let mut children: HashMap<String, Vec<(String, bool)>> = HashMap::new();
        for n in &rows {
            if let Some(p) = &n.parent {
                children
                    .entry(p.clone())
                    .or_default()
                    .push((n.id.clone(), n.directory));
            }
        }
        let mut wanted = HashSet::new();
        let mut pending = roots.to_vec();
        while let Some(id) = pending.pop() {
            if wanted.insert(id.clone()) {
                if let Some(entries) = children.get(&id) {
                    pending.extend(entries.iter().map(|(id, _)| id.clone()));
                }
            }
        }
        let mut details = HashMap::new();
        for n in rows.iter().filter(|n| n.detailed && wanted.contains(&n.id)) {
            let detail = (|| -> io::Result<Detail> {
                let own = store.status(&n.id).map_err(io::Error::other)?;
                let offline = !n.directory
                    && (store
                        .open_dirty_draft(&n.id)
                        .map_err(io::Error::other)?
                        .is_some()
                        || store.open_cached(&n.id).is_ok());
                let failure = if n.directory
                    || matches!(
                        own,
                        drive_core::Status::Conflict | drive_core::Status::PendingUpload
                    ) {
                    None
                } else {
                    store.download_failure(&n.id).map_err(io::Error::other)?
                };
                Ok((own, offline, failure))
            })();
            details.insert(n.id.clone(), detail.map_err(|_| ()));
        }
        Ok(Self {
            details,
            rows: rows.into_iter().map(|n| (n.id.clone(), n)).collect(),
            children,
            inherited: HashMap::new(),
        })
    }
    fn inherited(&mut self, id: &str, seen: &mut HashSet<String>) -> io::Result<(bool, bool)> {
        if let Some(v) = self.inherited.get(id) {
            return Ok(*v);
        }
        if seen.len() > 128 || !seen.insert(id.into()) {
            return Err(io::Error::other("status ancestry cycle"));
        }
        let n = self
            .rows
            .get(id)
            .ok_or_else(|| io::Error::other("status identity absent"))?;
        let (parent, pin, overlay) = (n.parent.clone(), n.pinned, n.overlay);
        let (a, b) = match parent {
            Some(p) => self.inherited(&p, seen)?,
            None => (false, false),
        };
        let value = (pin || a, overlay || b);
        self.inherited.insert(id.into(), value);
        Ok(value)
    }
    pub fn flags(&mut self, id: &str) -> io::Result<(bool, bool, bool)> {
        let (pin, overlay) = self.inherited(id, &mut HashSet::new())?;
        Ok((pin, self.rows.get(id).unwrap().detailed, overlay))
    }
    pub fn detail(&self, id: &str) -> Option<Result<Detail, ()>> {
        self.details.get(id).cloned()
    }
    pub fn children(&self, id: &str) -> Vec<(String, bool)> {
        self.children.get(id).cloned().unwrap_or_default()
    }
}
