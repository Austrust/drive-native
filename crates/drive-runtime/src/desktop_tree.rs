//! Reuse one bulk metadata read per status batch instead of N recursive SQL walks.
use drive_core::{desktop_status::StatusSeed, Store};
use std::{
    collections::{HashMap, HashSet},
    io,
};
pub(super) struct StatusTree {
    rows: HashMap<String, StatusSeed>,
    children: HashMap<String, Vec<(String, bool)>>,
    inherited: HashMap<String, (bool, bool)>,
}
impl StatusTree {
    pub fn load(store: &Store) -> io::Result<Self> {
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
        Ok(Self {
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
    pub fn children(&self, id: &str) -> Vec<(String, bool)> {
        self.children.get(id).cloned().unwrap_or_default()
    }
}
