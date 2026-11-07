//! Committed status projection cache for desktop reads (issue #8).
//!
//! `GetStates` used to rebuild the desktop status tree from `nodes` on every
//! request. That read is linear in the tree size and competes with the writer
//! for the global `SharedStore` mutex. This cache keeps the committed
//! `StatusSeed` rows of the namespace in memory and publishes them as an
//! immutable `Arc<StatusProjection>` so desktop requests can answer from a
//! snapshot without scanning SQLite or taking the global lock.
//!
//! Correctness model:
//! - The cache is built from committed, read-only state only.
//! - While a build or refresh is in flight the previous snapshot stays
//!   published; desktop reads may therefore observe a slightly older committed
//!   state (the same staleness window a client poll would see anyway). The
//!   desktop fallback path is used until the first snapshot exists.
//! - Anything the cache cannot describe (missing rows, oversize profiles) is
//!   reported by not publishing a snapshot, so callers fall back to the
//!   authoritative scoped query.
//!
//! The background worker owns the single `SharedStore` notification
//! subscription (`take_notifications`); no other runtime component consumes it.

use crate::service::Service;
use drive_core::{SharedStore, desktop_status::StatusSeed};
use std::{
    collections::HashMap,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

/// Profiles beyond this many nodes keep using the scoped authoritative path;
/// the in-memory projection would cost more than it saves.
pub(crate) const MAX_CACHED_NODES: usize = 500_000;
const WAIT: Duration = Duration::from_secs(1);

/// Immutable committed view of the desktop status seeds.
pub(crate) struct StatusProjection {
    generation: u64,
    rows: HashMap<String, StatusSeed>,
    children: HashMap<String, Vec<(String, bool)>>,
}

impl StatusProjection {
    pub(crate) fn from_rows(rows: Vec<StatusSeed>, generation: u64) -> Self {
        let mut child_map: HashMap<String, Vec<(String, bool)>> = HashMap::new();
        for row in &rows {
            if let Some(parent) = &row.parent {
                child_map
                    .entry(parent.clone())
                    .or_default()
                    .push((row.id.clone(), row.directory));
            }
        }
        Self {
            generation,
            rows: rows.into_iter().map(|row| (row.id.clone(), row)).collect(),
            children: child_map,
        }
    }

    pub(crate) fn seed(&self, id: &str) -> Option<&StatusSeed> {
        self.rows.get(id)
    }

    pub(crate) fn children(&self, id: &str) -> &[(String, bool)] {
        self.children.get(id).map(Vec::as_slice).unwrap_or(&[])
    }

    pub(crate) fn len(&self) -> usize {
        self.rows.len()
    }

    pub(crate) fn generation(&self) -> u64 {
        self.generation
    }
}

/// Background owner of the published projection.
pub(crate) struct StatusCache {
    current: Mutex<Option<Arc<StatusProjection>>>,
    disabled: AtomicBool,
}

impl StatusCache {
    /// Starts the cache worker. The service owns the returned cache handle and
    /// task; `Service::stop` terminates the worker within one wake interval.
    pub(crate) fn start(
        service: Arc<Service>,
    ) -> (Arc<Self>, std::thread::JoinHandle<()>) {
        let cache = Arc::new(Self {
            current: Mutex::new(None),
            disabled: AtomicBool::new(false),
        });
        let worker = cache.clone();
        let task = std::thread::spawn(move || worker.run(&service));
        (cache, task)
    }

    pub(crate) fn snapshot(&self) -> Option<Arc<StatusProjection>> {
        self.current
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    fn run(&self, service: &Arc<Service>) {
        let shared: &SharedStore = &service.bridge.store;
        // Single subscriber: if something else already claimed notifications we
        // cannot follow commits, so stay disabled and let callers use SQLite.
        let Ok(notifications) = shared.take_notifications() else {
            self.disabled.store(true, Ordering::Release);
            return;
        };
        let mut generation = 0u64;
        loop {
            if service.stop.load(Ordering::Acquire) || self.disabled.load(Ordering::Acquire) {
                return;
            }
            let state = notifications.state();
            if state.verified {
                if let Some(projection) = self.build(shared, state.revision, generation) {
                    generation += 1;
                    *self
                        .current
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(projection);
                }
            }
            if service.stop.load(Ordering::Acquire) {
                return;
            }
            // Wait for the next commit boundary. A revision bump without a
            // notification (or a missed merge) is caught by the timeout rebuild.
            let _ = notifications.recv_timeout(WAIT);
        }
    }

    fn build(
        &self,
        shared: &SharedStore,
        revision: u64,
        generation: u64,
    ) -> Option<Arc<StatusProjection>> {
        let store = drive_core::Store::open_read_only(&shared.root()).ok()?;
        let rows = store.desktop_status_seeds().ok()?;
        if rows.len() > MAX_CACHED_NODES {
            self.disabled.store(true, Ordering::Release);
            return None;
        }
        let projection = StatusProjection::from_rows(rows, generation.max(revision));
        Some(Arc::new(projection))
    }
}
