//! Reuse one bulk metadata read per status batch instead of N recursive SQL walks.
use crate::status_cache::StatusProjection;
use drive_core::{
    desktop_status::{desktop_status_details, StatusDetail, StatusSeed},
    Store,
};
use std::{
    collections::{HashMap, HashSet},
    io,
};
type Detail = (
    drive_core::Status,
    bool,
    Option<drive_core::download_jobs::DownloadFailure>,
);
type Children = HashMap<String, Vec<(String, bool)>>;
/// Where the seed rows of one batch come from.
enum Backing<'p> {
    /// Authoritative scoped read of the requested subtrees and ancestor chains.
    Loaded {
        rows: HashMap<String, StatusSeed>,
        children: Children,
    },
    /// Committed in-memory projection (issue #8). Only borrowed for the duration
    /// of the request; `wanted` is the subtree visited by this batch.
    Projected {
        projection: &'p StatusProjection,
        wanted: HashSet<String>,
    },
}
pub(super) struct StatusTree<'p> {
    details: HashMap<String, Result<Detail, ()>>,
    backing: Backing<'p>,
    inherited: HashMap<String, (bool, bool)>,
}
/// Desktop status detail for one seed. Both backings share this so a projected
/// batch reports exactly what the authoritative scoped batch reports.
fn detail_for(store: &Store, seed: &StatusSeed) -> io::Result<Detail> {
    let own = store.status(&seed.id).map_err(io::Error::other)?;
    let offline = !seed.directory
        && (store
            .open_dirty_draft(&seed.id)
            .map_err(io::Error::other)?
            .is_some()
            || store.open_cached(&seed.id).is_ok());
    let failure = if seed.directory
        || matches!(
            own,
            drive_core::Status::Conflict | drive_core::Status::PendingUpload
        ) {
        None
    } else {
        store.download_failure(&seed.id).map_err(io::Error::other)?
    };
    Ok((own, offline, failure))
}
fn to_detail(detail: &StatusDetail) -> Detail {
    (
        detail.status.clone(),
        detail.offline,
        detail.failure.clone(),
    )
}
/// Fill the detail map for `seeds` with one batch read (issue #30) instead of
/// three to four point queries per detailed node. Ids the batch cannot answer
/// fall back to [`detail_for`], which preserves the single-node error reporting.
fn load_details<'a>(
    store: &Store,
    seeds: impl Iterator<Item = &'a StatusSeed>,
) -> HashMap<String, Result<Detail, ()>> {
    let seeds: Vec<&StatusSeed> = seeds.collect();
    let nodes: Vec<(String, bool)> = seeds
        .iter()
        .map(|seed| (seed.id.clone(), seed.directory))
        .collect();
    let batch = desktop_status_details(store, &nodes).ok();
    let mut details = HashMap::with_capacity(seeds.len());
    for seed in seeds {
        let detail = match batch.as_ref().and_then(|batch| batch.get(&seed.id)) {
            Some(detail) => Ok(to_detail(detail)),
            None => detail_for(store, seed).map_err(|_| ()),
        };
        details.insert(seed.id.clone(), detail);
    }
    details
}
impl<'p> StatusTree<'p> {
    pub fn load(store: &Store, roots: &[String]) -> io::Result<Self> {
        // Only the requested roots' subtree plus their ancestor chain is read; a
        // full-namespace read would dominate every status batch on large profiles.
        let rows = store
            .desktop_status_seeds_for(roots)
            .map_err(io::Error::other)?;
        let mut children: Children = HashMap::new();
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
        let details = load_details(
            store,
            rows.iter()
                .filter(|n| n.detailed && wanted.contains(&n.id)),
        );
        Ok(Self {
            details,
            backing: Backing::Loaded {
                rows: rows.into_iter().map(|n| (n.id.clone(), n)).collect(),
                children,
            },
            inherited: HashMap::new(),
        })
    }
    /// Same batch semantics as [`Self::load`], answered from the committed
    /// projection without a scoped SQLite read. A root (or any link of its
    /// ancestor chain) missing from the projection is an error so the caller can
    /// fall back to the authoritative scoped read instead of reporting a silently
    /// truncated tree.
    pub(super) fn load_with_projection(
        store: &Store,
        projection: &'p StatusProjection,
        roots: &[String],
    ) -> io::Result<Self> {
        let mut wanted = HashSet::new();
        let mut pending = roots.to_vec();
        while let Some(id) = pending.pop() {
            if wanted.insert(id.clone()) {
                pending.extend(projection.children(&id).iter().map(|(id, _)| id.clone()));
            }
        }
        for root in roots {
            let mut cursor = Some(root.as_str());
            let mut hops = 0usize;
            while let Some(id) = cursor {
                hops += 1;
                if hops > 128 {
                    return Err(io::Error::other("status ancestry cycle"));
                }
                let seed = projection
                    .seed(id)
                    .ok_or_else(|| io::Error::other("status identity absent"))?;
                cursor = seed.parent.as_deref();
            }
        }
        let mut detailed_seeds: Vec<&StatusSeed> = Vec::with_capacity(wanted.len());
        for id in &wanted {
            let seed = projection
                .seed(id)
                .ok_or_else(|| io::Error::other("status identity absent"))?;
            if seed.detailed {
                detailed_seeds.push(seed);
            }
        }
        let details = load_details(store, detailed_seeds.into_iter());
        Ok(Self {
            details,
            backing: Backing::Projected { projection, wanted },
            inherited: HashMap::new(),
        })
    }
    fn seed(&self, id: &str) -> Option<&StatusSeed> {
        match &self.backing {
            Backing::Loaded { rows, .. } => rows.get(id),
            Backing::Projected { projection, .. } => projection.seed(id),
        }
    }
    fn batch_children(&self, id: &str) -> Vec<(String, bool)> {
        match &self.backing {
            Backing::Loaded { children, .. } => children.get(id).cloned().unwrap_or_default(),
            // A projected batch only walks children inside this batch's subtree,
            // mirroring the scoped read which never returns sibling branches.
            Backing::Projected { projection, wanted } => projection
                .children(id)
                .iter()
                .filter(|(child, _)| wanted.contains(child))
                .cloned()
                .collect(),
        }
    }
    fn inherited(&mut self, id: &str, seen: &mut HashSet<String>) -> io::Result<(bool, bool)> {
        if let Some(v) = self.inherited.get(id) {
            return Ok(*v);
        }
        if seen.len() > 128 || !seen.insert(id.into()) {
            return Err(io::Error::other("status ancestry cycle"));
        }
        let n = self
            .seed(id)
            .cloned()
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
        let detailed = self
            .seed(id)
            .ok_or_else(|| io::Error::other("status identity absent"))?
            .detailed;
        Ok((pin, detailed, overlay))
    }
    pub fn detail(&self, id: &str) -> Option<Result<Detail, ()>> {
        self.details.get(id).cloned()
    }
    pub fn children(&self, id: &str) -> Vec<(String, bool)> {
        self.batch_children(id)
    }
}
