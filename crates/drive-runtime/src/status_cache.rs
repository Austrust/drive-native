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
//! - The cache is built from committed, read-only state only. The initial
//!   snapshot and every fallback rebuild read through a `Store::open_read_only`
//!   handle; journal pages are read under a short blocking `SharedStore` guard
//!   because the change journal is a TEMP object owned by the writer
//!   connection. Facts for the invalidated ids are re-read inside that same
//!   guard, so one batch always describes one committed state.
//! - Journal entries are *invalidations*, not state: a changed node is re-read
//!   from the writer connection under the guard, so replaying an update twice is
//!   harmless. Location-keyed shadowing (`local_creations`) is re-evaluated by
//!   re-reading every live id at the exact parent/name.
//! - Subtree-scoped invalidations (ACL denial, namespace overlay replacement,
//!   node moves) cannot be repaired by re-reading only the named node: rows
//!   below it may become hidden or visible. Those pages expand the published
//!   subtree and re-read its visible rows with one scoped query in the same
//!   guard; rows that disappear from that scoped result are removed. A full
//!   scan rebuild is only used when the journal itself demands it (gap,
//!   overflow, epoch change) or the backlog is huge.
//! - While a refresh is in flight the previous snapshot stays published; desktop
//!   reads may observe a slightly older committed state (the same staleness
//!   window a client poll would see anyway). The publish throttle keeps this
//!   window under ~1s.
//! - Anything the cache cannot describe (unconfirmed journal boundary, oversize
//!   profiles, persistent read errors) disables publication, so callers fall
//!   back to the authoritative scoped query.
//! - Publishing never mutates a published snapshot: the worker owns mutable
//!   working maps and clones them into a fresh `Arc<StatusProjection>`. The
//!   previous `Arc` is dropped after the cache mutex has been released, so a
//!   large projection is never destroyed on the reader path.
//!
//! The background worker owns the single `SharedStore` notification
//! subscription (`take_notifications`); no other runtime component consumes it.

use crate::service::Service;
use drive_core::{
    SharedStore,
    desktop_status::StatusSeed,
    shared_store::{CommitNotifications, StoreGuard},
    status_changes::{ChangePage, ChangeScope, JournalBudget, StatusChange, StatusCursor},
};
use std::{
    collections::{HashMap, HashSet},
    sync::{
        Arc, Mutex, TryLockError,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

/// Profiles beyond this many nodes keep using the scoped authoritative path;
/// the in-memory projection would cost more than it saves.
pub(crate) const MAX_CACHED_NODES: usize = 500_000;
/// Journal batches larger than this make a point replay more expensive than one
/// linear rebuild.
const BACKLOG_LIMIT: u64 = 50_000;
/// One journal page per guard keeps the `SharedStore` lock hold short; a page is
/// not a transaction boundary, so it is safe to publish after each page.
const BATCH_LIMIT: usize = 512;
/// Publish at most once per this interval. A burst of commits is coalesced into
/// one copy-on-write publication instead of one clone per commit; the resulting
/// committed-snapshot staleness window stays under a second for ACL flips.
const PUBLISH_INTERVAL: Duration = Duration::from_millis(250);
/// First idle wait (also the stop/wake-check interval).
const WAIT: Duration = Duration::from_millis(250);
/// Idle waits back off to this ceiling so a quiet cache takes the global writer
/// lock about once per second instead of four times.
const IDLE_WAIT_MAX: Duration = Duration::from_millis(1000);
/// A subtree refresh larger than this is cheaper to serve with one asynchronous
/// full rebuild on the read-only handle than to resolve inside the writer
/// guard; the previous snapshot keeps serving readers meanwhile.
const SUBTREE_APPLY_LIMIT: usize = 50_000;
/// Startup replay keeps going until caught up, but a synchronisation that keeps
/// writing must not starve the first publication forever.
const STARTUP_REPLAY_LIMIT: Duration = Duration::from_secs(10);
/// Budget high enough that an ordinary sync burst does not overflow the journal
/// between two cache wake-ups. The design allocates the whole journal to this
/// cache (single consumer): 65,536 entries and 16 MiB.
const JOURNAL_ENTRIES: usize = 65_536;
const JOURNAL_BYTES: usize = 16 * 1024 * 1024;

type Children = Vec<(String, bool)>;

/// Acquires the writer guard without making `Service::stop` wait behind a long
/// writer critical section. `try_lock` polling keeps the stop signal responsive
/// even while another component holds the store mutex.
fn lock_stop_aware<'a>(shared: &'a SharedStore, stop: &AtomicBool) -> Option<StoreGuard<'a>> {
    loop {
        if stop.load(Ordering::Acquire) {
            return None;
        }
        match shared.try_lock() {
            Ok(guard) => return Some(guard),
            Err(TryLockError::Poisoned(error)) => return Some(error.into_inner()),
            Err(TryLockError::WouldBlock) => std::thread::sleep(Duration::from_millis(5)),
        }
    }
}

/// Immutable committed view of the desktop status seeds.
pub(crate) struct StatusProjection {
    generation: u64,
    rows: HashMap<String, StatusSeed>,
    /// Parent id -> visible children, sorted by child id (matches the
    /// primary-key order of a full seed scan).
    children: HashMap<String, Children>,
    len: usize,
}

impl StatusProjection {
    pub(crate) fn from_rows(rows: Vec<StatusSeed>, generation: u64) -> Self {
        let len = rows.len();
        let mut row_map = HashMap::with_capacity(len);
        for row in rows {
            row_map.insert(row.id.clone(), row);
        }
        let children = build_children(&row_map);
        Self {
            generation,
            rows: row_map,
            children,
            len,
        }
    }

    pub(crate) fn seed(&self, id: &str) -> Option<&StatusSeed> {
        self.rows.get(id)
    }

    pub(crate) fn children(&self, id: &str) -> &[(String, bool)] {
        self.children.get(id).map(Vec::as_slice).unwrap_or(&[])
    }

    pub(crate) fn len(&self) -> usize {
        self.len
    }

    pub(crate) fn generation(&self) -> u64 {
        self.generation
    }
}

fn build_children(rows: &HashMap<String, StatusSeed>) -> HashMap<String, Children> {
    let mut children: HashMap<String, Children> = HashMap::new();
    for row in rows.values() {
        if let Some(parent) = &row.parent {
            children
                .entry(parent.clone())
                .or_default()
                .push((row.id.clone(), row.directory));
        }
    }
    for list in children.values_mut() {
        list.sort_by(|a, b| a.0.cmp(&b.0));
    }
    children
}

/// Background owner of the published projection.
pub(crate) struct StatusCache {
    current: Mutex<Option<Arc<StatusProjection>>>,
    disabled: AtomicBool,
    /// Observability counters (used by the ignored cost tests and reports).
    rebuilds: std::sync::atomic::AtomicU64,
    publishes: std::sync::atomic::AtomicU64,
}

impl StatusCache {
    /// Starts the cache worker. The service owns the returned cache handle and
    /// task; `Service::stop` terminates the worker within two wake intervals.
    pub(crate) fn start(service: Arc<Service>) -> (Arc<Self>, std::thread::JoinHandle<()>) {
        Self::start_with_limit(service, MAX_CACHED_NODES)
    }

    /// Test/operations seam: same worker with a custom projection-size bound.
    pub(crate) fn start_with_limit(
        service: Arc<Service>,
        max_nodes: usize,
    ) -> (Arc<Self>, std::thread::JoinHandle<()>) {
        Self::start_with(service, max_nodes, BACKLOG_LIMIT)
    }

    fn start_with(
        service: Arc<Service>,
        max_nodes: usize,
        backlog_limit: u64,
    ) -> (Arc<Self>, std::thread::JoinHandle<()>) {
        let cache = Arc::new(Self {
            current: Mutex::new(None),
            disabled: AtomicBool::new(false),
            rebuilds: std::sync::atomic::AtomicU64::new(0),
            publishes: std::sync::atomic::AtomicU64::new(0),
        });
        let worker = cache.clone();
        let task = std::thread::spawn(move || worker.run(&service, max_nodes, backlog_limit));
        (cache, task)
    }

    /// Test seam for the backlog-triggered rebuild path.
    #[cfg(test)]
    pub(crate) fn start_with_limits(
        service: Arc<Service>,
        max_nodes: usize,
        backlog_limit: u64,
    ) -> (Arc<Self>, std::thread::JoinHandle<()>) {
        Self::start_with(service, max_nodes, backlog_limit)
    }

    /// Number of full-scan rebuilds performed by the worker.
    #[cfg(test)]
    pub(crate) fn rebuilds(&self) -> u64 {
        self.rebuilds.load(Ordering::Relaxed)
    }

    /// Number of published projections.
    #[cfg(test)]
    pub(crate) fn publishes(&self) -> u64 {
        self.publishes.load(Ordering::Relaxed)
    }

    pub(crate) fn snapshot(&self) -> Option<Arc<StatusProjection>> {
        if self.disabled() {
            return None;
        }
        self.current
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    fn disabled(&self) -> bool {
        self.disabled.load(Ordering::Acquire)
    }

    fn disable(&self) {
        self.disabled.store(true, Ordering::Release);
        // Clearing under the lock makes snapshot() return None from now on; the
        // dropped snapshot must not be destroyed while the lock is held.
        let stale = {
            let mut current = self
                .current
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            current.take()
        };
        drop(stale);
    }

    fn publish(&self, projection: Arc<StatusProjection>) {
        // Only the swap happens under the lock; the retired projection may be
        // hundreds of megabytes and is dropped afterwards by `stale`.
        let stale = {
            let mut current = self
                .current
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            current.replace(projection)
        };
        drop(stale);
    }

    fn run(&self, service: &Arc<Service>, max_nodes: usize, backlog_limit: u64) {
        let shared = service.bridge.store.clone();
        // Single subscriber: if something else already claimed notifications we
        // cannot follow commits, so stay disabled and let callers use SQLite.
        let Ok(notifications) = shared.take_notifications() else {
            self.disable();
            return;
        };
        let Ok(store) = drive_core::Store::open_read_only(&shared.root()) else {
            self.disable();
            return;
        };
        let mut worker = Worker {
            cache: self,
            store,
            max_nodes,
            backlog_limit,
            cursor: StatusCursor {
                epoch: String::new(),
                sequence: 0,
            },
            generation: 0,
            rows: HashMap::new(),
            children: HashMap::new(),
            len: 0,
            dirty: false,
            last_publish: None,
            idle_wait: WAIT,
        };
        if !worker.bootstrap(service, &shared, &notifications) {
            self.disable();
            return;
        }
        worker.serve(service, &shared, &notifications);
    }
}

/// One page read from the journal, or the alternative maintenance action.
enum Batch {
    /// No new journal entries.
    Empty,
    /// A point replay cannot repair the page; rebuild from a full scan.
    Rebuild(StatusCursor),
    /// Fully-resolved invalidations for one committed page.
    Changes {
        updates: Vec<(String, Option<StatusSeed>)>,
        next: StatusCursor,
        has_more: bool,
    },
}

struct Worker<'a> {
    cache: &'a StatusCache,
    /// Read-only handle: initial snapshot, rebuilds and the desktop fallback all
    /// observe only committed WAL frames.
    store: drive_core::Store,
    max_nodes: usize,
    /// Unconsumed journal span that forces a full rebuild instead of a replay.
    backlog_limit: u64,
    cursor: StatusCursor,
    generation: u64,
    rows: HashMap<String, StatusSeed>,
    children: HashMap<String, Children>,
    len: usize,
    dirty: bool,
    last_publish: Option<Instant>,
    /// Idle wait, backed off from `WAIT` up to `IDLE_WAIT_MAX` so a quiet cache
    /// does not take the writer lock four times a second.
    idle_wait: Duration,
}

impl Worker<'_> {
    /// Snapshot, journal subscription and replay, then publish the first
    /// projection. The journal boundary is captured before the snapshot; the
    /// snapshot may already contain newer commits, and replaying those entries
    /// is idempotent.
    fn bootstrap(
        &mut self,
        service: &Arc<Service>,
        shared: &SharedStore,
        notifications: &CommitNotifications,
    ) -> bool {
        let stop = &service.stop;
        let cursor = {
            let mut guard = match lock_stop_aware(shared, stop) {
                Some(guard) => guard,
                None => return false,
            };
            let budget = JournalBudget {
                entries: JOURNAL_ENTRIES,
                bytes: JOURNAL_BYTES,
            };
            if guard.set_status_journal_budget(budget).is_err() {
                return false;
            }
            match guard.status_change_watermark() {
                Ok(watermark) => watermark.cursor,
                Err(_) => return false,
            }
        };
        let rows = match self.store.desktop_status_seeds() {
            Ok(rows) => rows,
            Err(_) => return false,
        };
        if !self.install_full(rows) {
            return false;
        }
        self.cursor = cursor;
        // The first publication is bounded by `STARTUP_REPLAY_LIMIT`: a sync
        // that keeps writing must not keep desktop reads on the authoritative
        // path forever. Reaching the deadline publishes the committed snapshot
        // we have (slightly stale, converged by the serve loop), which is the
        // same window a client poll would observe.
        let deadline = Instant::now() + STARTUP_REPLAY_LIMIT;
        loop {
            if stop.load(Ordering::Acquire) {
                return false;
            }
            if !notifications.state().verified {
                let _ = notifications.recv_timeout(WAIT);
                continue;
            }
            match self.read_batch(shared, stop) {
                Ok(Some(Batch::Empty)) => break,
                Ok(Some(Batch::Changes {
                    updates,
                    next,
                    has_more: _,
                })) => {
                    self.apply(updates);
                    self.cursor = next;
                    if self.len > self.max_nodes {
                        return false;
                    }
                }
                Ok(Some(Batch::Rebuild(cursor))) => {
                    if !self.rebuild(cursor, stop) {
                        return false;
                    }
                }
                Ok(None) => return false,
                Err(_) => {
                    let _ = notifications.recv_timeout(WAIT);
                }
            }
            if Instant::now() >= deadline {
                break;
            }
        }
        self.publish_now();
        true
    }

    /// Main maintenance loop. Idle wake-ups are cheap and publish nothing.
    fn serve(
        &mut self,
        service: &Arc<Service>,
        shared: &SharedStore,
        notifications: &CommitNotifications,
    ) {
        let stop = &service.stop;
        loop {
            if stop.load(Ordering::Acquire) {
                return;
            }
            let mut wait = true;
            if notifications.state().verified {
                match self.read_batch(shared, stop) {
                    Ok(Some(Batch::Empty)) => {}
                    Ok(Some(Batch::Changes {
                        updates,
                        next,
                        has_more,
                    })) => {
                        self.apply(updates);
                        self.cursor = next;
                        // The cap is enforced after every applied batch: an
                        // incremental burst must not grow a published projection
                        // past what the caller was promised it would cache.
                        if self.len > self.max_nodes {
                            self.cache.disable();
                            return;
                        }
                        self.dirty = true;
                        self.idle_wait = WAIT;
                        self.publish_if_due();
                        wait = !has_more;
                    }
                    Ok(Some(Batch::Rebuild(cursor))) => {
                        if !self.rebuild(cursor, stop) {
                            self.cache.disable();
                            return;
                        }
                        self.publish_now();
                        wait = false;
                    }
                    Ok(None) => return,
                    Err(_) => {}
                }
            }
            if self.publish_if_due() {
                wait = true;
            }
            if wait {
                let _ = notifications.recv_timeout(self.next_wait());
                self.publish_if_due();
            }
        }
    }

    /// Reads one committed journal page and resolves its invalidations while the
    /// writer is still excluded. Everything a page touches (old/new node ids,
    /// old/new parent+name keys) is re-read from the writer connection inside
    /// this guard, so no cross-batch write can be observed half-applied. For
    /// subtree-scoped invalidations the visible rows below the affected roots are
    /// re-read with one scoped query in the same guard; rows that disappeared
    /// from that result are removed. `Ok(None)` means the stop signal arrived
    /// while waiting for the writer guard.
    fn read_batch(
        &mut self,
        shared: &SharedStore,
        stop: &AtomicBool,
    ) -> drive_core::Result<Option<Batch>> {
        let mut guard = match lock_stop_aware(shared, stop) {
            Some(guard) => guard,
            None => return Ok(None),
        };
        let watermark = guard.status_change_watermark()?;
        if watermark
            .cursor
            .sequence
            .saturating_sub(self.cursor.sequence)
            > self.backlog_limit
        {
            return Ok(Some(Batch::Rebuild(watermark.cursor)));
        }
        let (changes, next, has_more) = match guard.read_status_changes(&self.cursor, BATCH_LIMIT)? {
            ChangePage::Ready {
                changes,
                next,
                has_more,
                ..
            } => (changes, next, has_more),
            ChangePage::Rebuild { watermark, .. } => {
                return Ok(Some(Batch::Rebuild(watermark.cursor)));
            }
        };
        if changes.is_empty() {
            return Ok(Some(Batch::Empty));
        }
        let mut ids: HashSet<String> = HashSet::new();
        let mut keys: HashSet<(Option<String>, String)> = HashSet::new();
        let mut subtree_roots: Vec<String> = Vec::new();
        for change in &changes {
            if let Some(id) = &change.old_node {
                ids.insert(id.clone());
            }
            if let Some(id) = &change.new_node {
                ids.insert(id.clone());
            }
            if let Some(name) = &change.old_name {
                keys.insert((change.old_parent.clone(), name.clone()));
            }
            if let Some(name) = &change.new_name {
                keys.insert((change.new_parent.clone(), name.clone()));
            }
            if change.scope == ChangeScope::SubtreeAndAncestors
                && self.subtree_needs_refresh(change)
            {
                // A namespace-root location (`parent IS NULL`) would make the
                // scoped query take its full-scan fallback; never do that inside
                // the writer guard.
                if change.new_parent.is_none() || change.old_parent.is_none() {
                    return Ok(Some(Batch::Rebuild(watermark.cursor)));
                }
                if let Some(id) = change.new_node.as_ref().or(change.old_node.as_ref()) {
                    subtree_roots.push(id.clone());
                }
            }
        }
        subtree_roots.sort_unstable();
        subtree_roots.dedup();
        let mut updates: Vec<(String, Option<StatusSeed>)> =
            Vec::with_capacity(ids.len() + keys.len() + subtree_roots.len());
        let mut seen: HashSet<String> = HashSet::with_capacity(ids.len() + keys.len());
        let mut walked: HashSet<String> = HashSet::new();
        let mut subtree_seeds: Vec<StatusSeed> = Vec::new();
        if !subtree_roots.is_empty() {
            // Old visible rows below the roots come from the published children
            // map; the scoped query then tells which of them are still visible.
            let mut stack = subtree_roots.clone();
            while let Some(id) = stack.pop() {
                if !walked.insert(id.clone()) {
                    continue;
                }
                if let Some(children) = self.children.get(&id) {
                    for (child, _) in children {
                        stack.push(child.clone());
                    }
                }
            }
            if walked.len() > SUBTREE_APPLY_LIMIT {
                return Ok(Some(Batch::Rebuild(watermark.cursor)));
            }
            subtree_seeds = guard.desktop_status_seeds_for(&subtree_roots)?;
            if subtree_seeds.len() > SUBTREE_APPLY_LIMIT {
                return Ok(Some(Batch::Rebuild(watermark.cursor)));
            }
        }
        for id in ids {
            let seed = guard.desktop_status_seed(&id)?;
            seen.insert(id.clone());
            updates.push((id, seed));
        }
        for (parent, name) in keys {
            for id in guard.desktop_status_ids_at(parent.as_deref(), &name)? {
                if !seen.insert(id.clone()) {
                    continue;
                }
                let seed = guard.desktop_status_seed(&id)?;
                updates.push((id, seed));
            }
        }
        // `desktop_status_seeds_for` includes the roots' ancestor chain, so those
        // rows (needed by `StatusTree::inherited`) are re-read as well.
        let mut visible: HashSet<String> = HashSet::with_capacity(subtree_seeds.len());
        for seed in subtree_seeds {
            visible.insert(seed.id.clone());
            if seen.insert(seed.id.clone()) {
                updates.push((seed.id.clone(), Some(seed)));
            }
        }
        for id in walked {
            if !visible.contains(&id) {
                updates.push((id, None));
            }
        }
        // A page is not a transaction boundary, so the acknowledged prefix can be
        // pruned as soon as it has been fully resolved. Pruning edits only the
        // TEMP journal (floor/entries/bytes), never durable state.
        guard.prune_status_changes(&next)?;
        Ok(Some(Batch::Changes {
            updates,
            next,
            has_more,
        }))
    }

    /// Subtree-scoped invalidations whose descendants' visibility can change and
    /// therefore cannot be repaired by refreshing the named node alone.
    ///
    /// Seed rows are row-local; the inherited part is the `access_blocked`
    /// filter (it walks `nodes.parent` and the overlay remote-parent edges). A
    /// denial, an overlay relocation/removal or a directory move can hide or
    /// reveal the whole subtree below the named node.
    fn subtree_needs_refresh(&self, change: &StatusChange) -> bool {
        match change.source.as_str() {
            "access_denied" => true,
            // Overlay edits can only hide descendants when the overlay node
            // already has projected children; plain file overlays are row-local.
            "namespace_overlay" => change
                .new_node
                .as_ref()
                .or(change.old_node.as_ref())
                .is_some_and(|id| self.children.contains_key(id)),
            // A move changes the inherited `access_blocked` chain below the
            // moved directory. Inserts/deletes and renames do not.
            "nodes" => {
                change.old_node.is_some()
                    && change.new_node.is_some()
                    && change.old_parent != change.new_parent
                    && change
                        .new_node
                        .as_ref()
                        .is_some_and(|id| self.children.contains_key(id))
            }
            _ => false,
        }
    }

    /// Rebuilds the working projection from a fresh committed scan. The previous
    /// snapshot keeps serving readers until [`Self::publish_now`].
    fn rebuild(&mut self, cursor: StatusCursor, stop: &AtomicBool) -> bool {
        if stop.load(Ordering::Acquire) {
            return false;
        }
        self.cache.rebuilds.fetch_add(1, Ordering::Relaxed);
        let rows = match self.store.desktop_status_seeds() {
            Ok(rows) => rows,
            Err(_) => return false,
        };
        // A scan of a large namespace is the longest uninterrupted step the
        // worker performs; stop requests must not wake up only after it.
        if stop.load(Ordering::Acquire) {
            return false;
        }
        if !self.install_full(rows) {
            return false;
        }
        // The scan may already include commits newer than `cursor`; replaying
        // them later only re-reads current facts, so it stays idempotent.
        self.cursor = cursor;
        true
    }

    fn install_full(&mut self, rows: Vec<StatusSeed>) -> bool {
        if rows.len() > self.max_nodes {
            return false;
        }
        self.len = rows.len();
        self.rows = HashMap::with_capacity(rows.len());
        for row in rows {
            self.rows.insert(row.id.clone(), row);
        }
        self.children = build_children(&self.rows);
        self.dirty = true;
        true
    }

    fn apply(&mut self, updates: Vec<(String, Option<StatusSeed>)>) {
        for (id, next) in updates {
            self.set_row(&id, next);
        }
    }

    fn set_row(&mut self, id: &str, next: Option<StatusSeed>) {
        let previous = self.rows.get(id).cloned();
        match (previous, next) {
            (Some(previous), Some(next)) => {
                if previous.parent != next.parent || previous.directory != next.directory {
                    if let Some(parent) = &previous.parent {
                        self.remove_child(parent, id);
                    }
                    self.upsert_child(&next);
                }
                self.rows.insert(id.to_owned(), next);
            }
            (Some(previous), None) => {
                if let Some(parent) = &previous.parent {
                    self.remove_child(parent, id);
                }
                self.rows.remove(id);
                self.len = self.len.saturating_sub(1);
            }
            (None, Some(next)) => {
                self.upsert_child(&next);
                self.rows.insert(id.to_owned(), next);
                self.len += 1;
            }
            (None, None) => {}
        }
        // `id` is the map key; keep the stored row id authoritative.
        debug_assert!(self.rows.get(id).is_none_or(|row| row.id == id));
    }

    fn upsert_child(&mut self, seed: &StatusSeed) {
        let Some(parent) = &seed.parent else {
            return;
        };
        let children = self.children.entry(parent.clone()).or_default();
        let index = children.binary_search_by(|(child, _)| child.as_str().cmp(seed.id.as_str()));
        let entry = (seed.id.clone(), seed.directory);
        match index {
            Ok(index) => children[index] = entry,
            Err(index) => children.insert(index, entry),
        }
    }

    fn remove_child(&mut self, parent: &str, id: &str) {
        let Some(children) = self.children.get_mut(parent) else {
            return;
        };
        if let Ok(index) = children.binary_search_by(|(child, _)| child.as_str().cmp(id)) {
            children.remove(index);
        }
        if children.is_empty() {
            self.children.remove(parent);
        }
    }

    fn publish_now(&mut self) {
        self.generation += 1;
        let projection = Arc::new(StatusProjection {
            generation: self.generation,
            rows: self.rows.clone(),
            children: self.children.clone(),
            len: self.len,
        });
        self.cache.publish(projection);
        self.cache.publishes.fetch_add(1, Ordering::Relaxed);
        self.last_publish = Some(Instant::now());
        self.dirty = false;
    }

    /// Publishes the accumulated changes if the throttle window has elapsed.
    /// Returns whether a projection was published.
    fn publish_if_due(&mut self) -> bool {
        if !self.dirty {
            return false;
        }
        if let Some(last) = self.last_publish {
            if last.elapsed() < PUBLISH_INTERVAL {
                return false;
            }
        }
        self.publish_now();
        true
    }

    /// Wait before the next wake-up. Dirty state waits only for the publish
    /// throttle; idle state backs off so a quiet cache stops taking the writer
    /// lock four times per second.
    fn next_wait(&mut self) -> Duration {
        if self.dirty {
            let remaining = self.last_publish.map_or(Duration::ZERO, |last| {
                PUBLISH_INTERVAL
                    .saturating_sub(last.elapsed())
                    .max(Duration::from_millis(1))
            });
            return remaining;
        }
        self.idle_wait = (self.idle_wait * 2).min(IDLE_WAIT_MAX);
        self.idle_wait
    }
}

#[cfg(test)]
#[path = "status_cache_tests.rs"]
mod tests;
