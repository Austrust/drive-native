//! Committed status projection cache for desktop reads (issue #8, #25).
//!
//! `GetStates` used to rebuild the desktop status tree from `nodes` on every
//! request. That read is linear in the tree size and competes with the writer
//! for the global `SharedStore` mutex. This cache keeps the committed
//! `StatusSeed` rows of the namespace in memory and publishes them as an
//! immutable `Arc<StatusProjection>` so desktop requests can answer from a
//! snapshot without scanning SQLite or taking the global lock.
//!
//! # Issue #25: bounded lock hold, cheap publication, recoverable degradation
//!
//! The first version of this cache did all of a page's work under one
//! `SharedStore` guard: journal read, per-id seed re-reads, subtree scoped
//! queries and even the journal prune. A 512-change page could keep the writer
//! excluded for thousands of SQLite statements, and a publication deep-cloned
//! the whole projection (hundreds of thousands of `String`s) up to four times a
//! second. Any failure disabled the cache forever with no way to observe or
//! recover it.
//!
//! The current design keeps the same correctness model but bounds each part:
//! - The writer guard only covers the TEMP journal work: a deferred prune of the
//!   previously applied page, the watermark read and one `read_status_changes`
//!   page. Statement counts there are independent of the namespace size. The
//!   measured hold is recorded per page in a latency histogram and compared to
//!   [`LOCK_BUDGET`]; the worker also yields the lock between backlog pages
//!   (`BACKLOG_YIELD`) instead of re-acquiring it back-to-back.
//! - Facts for the invalidated ids and subtree scopes are re-read through the
//!   worker's own `Store::open_read_only` handle, outside the writer guard. The
//!   journal entry is an invalidation, so a re-read may observe newer committed
//!   facts; replaying a later page only re-reads current facts and is idempotent.
//! - Publication is base + sparse delta: the worker keeps one immutable base
//!   snapshot (`Arc` shared between every projection and the worker itself)
//!   plus only the rows/child edges that changed since it, and answers its own
//!   queries through a merged view over the two — so the projection is resident
//!   exactly once. A delta wider than `max(len / 8, COMPACT_MIN_DELTA)` triggers
//!   one compaction, which rebuilds the base once instead of cloning per
//!   publish.
//! - Degradation is a visible, retryable state. Every former "disable" (missing
//!   notifications, read-only open failure, bootstrap failure, oversize
//!   profile, rebuild failure) and a run of read failures moves the cache to
//!   `Degraded` with a reason; the last good snapshot keeps serving readers and
//!   the worker retries with exponential backoff. A successful cycle returns to
//!   `Running`; generation keeps increasing across recoveries.
//! - `access_denied` invalidations are resolved with one scoped query over the
//!   `access_blocked` closure (local parent + overlay remote-parent edges)
//!   instead of a full-namespace rebuild, so an ACL edit costs the affected
//!   subtree, not the namespace. An oversized closure still falls back to a
//!   rebuild.
//!
//! While a refresh is in flight the previous snapshot stays published; desktop
//! reads may observe a slightly older committed state (the same staleness window
//! a client poll would see anyway). The publish throttle keeps this window under
//! ~1s, and readers never block on the worker.
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
        atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering},
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
/// one publication instead of one copy-on-write publication per commit; the
/// resulting committed-snapshot staleness window stays under a second for ACL
/// flips.
const PUBLISH_INTERVAL: Duration = Duration::from_millis(250);
/// First idle wait (also the stop/wake-check interval).
const WAIT: Duration = Duration::from_millis(250);
/// Idle waits back off to this ceiling so a quiet cache takes the global writer
/// lock about once per second instead of four times.
const IDLE_WAIT_MAX: Duration = Duration::from_millis(1000);
/// A subtree refresh larger than this is cheaper to serve with one asynchronous
/// full rebuild on the read-only handle than to expand and re-read under the
/// worker; the previous snapshot keeps serving readers meanwhile.
const SUBTREE_APPLY_LIMIT: usize = 50_000;
/// Startup replay keeps going until caught up, but a synchronisation that keeps
/// writing must not starve the first publication forever.
const STARTUP_REPLAY_LIMIT: Duration = Duration::from_secs(10);
/// Budget high enough that an ordinary sync burst does not overflow the journal
/// between two cache wake-ups. The design allocates the whole journal to this
/// cache (single consumer): 65,536 entries and 16 MiB.
const JOURNAL_ENTRIES: usize = 65_536;
const JOURNAL_BYTES: usize = 16 * 1024 * 1024;
/// Hard budget for one journal page's writer-guard hold. The guard only covers
/// the TEMP journal (deferred prune, watermark, one page), so exceeding this is a
/// regression the metrics must expose rather than a cost the design accepts.
const LOCK_BUDGET: Duration = Duration::from_millis(50);
/// How long the worker sleeps between backlog pages. The previous design read
/// page after page back-to-back, which could starve the writer for a long time
/// even though each individual hold was short.
const BACKLOG_YIELD: Duration = Duration::from_millis(10);
/// First retry delay after entering the degraded state.
const RETRY_INITIAL: Duration = Duration::from_millis(100);
/// Retry delay ceiling while the cache stays degraded.
const RETRY_MAX: Duration = Duration::from_secs(5);
/// Consecutive journal read errors that turn into a degraded state. A single
/// transient error is recoverable in place.
const READ_ERROR_LIMIT: u32 = 5;
/// Minimum sparse-delta width before a compaction; also the floor below which
/// deltas are too small to be worth rebasing.
const COMPACT_MIN_DELTA: usize = 512;

/// Upper bounds (microseconds) of the lock-hold histogram buckets.
const LOCK_BUCKETS_US: [u64; 16] = [
    50, 100, 200, 400, 800, 1600, 3200, 6400, 12_800, 25_600, 51_200, 102_400, 204_800, 512_000,
    1_024_000, u64::MAX,
];

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

/// Sleeps in short slices so `Service::stop` stays responsive even during a
/// retry backoff. Returns false when the worker must stop.
fn sleep_stop_aware(duration: Duration, stop: &AtomicBool) -> bool {
    let deadline = Instant::now() + duration;
    loop {
        if stop.load(Ordering::Acquire) {
            return false;
        }
        let now = Instant::now();
        if now >= deadline {
            return true;
        }
        std::thread::sleep((deadline - now).min(Duration::from_millis(20)));
    }
}

/// Immutable part of a projection: a complete snapshot taken at a rebuild or at
/// a compaction. Successive projections share this `Arc` instead of cloning it.
#[derive(Default)]
struct ProjectionBase {
    rows: HashMap<String, StatusSeed>,
    /// Parent id -> visible children, sorted by child id (matches the
    /// primary-key order of a full seed scan).
    children: HashMap<String, Children>,
    len: usize,
}

impl ProjectionBase {
    fn from_rows(rows: Vec<StatusSeed>) -> Self {
        let len = rows.len();
        let mut row_map = HashMap::with_capacity(len);
        for row in rows {
            row_map.insert(row.id.clone(), row);
        }
        let children = build_children(&row_map);
        Self {
            rows: row_map,
            children,
            len,
        }
    }

    /// Materialises a merged view (`base` plus the sparse deltas) into one
    /// immutable snapshot. Used by compaction: the worker replaces its shared
    /// base here instead of keeping a third full copy of the projection.
    fn from_view(
        base: &ProjectionBase,
        rows: &HashMap<String, Option<StatusSeed>>,
        len: usize,
    ) -> Self {
        let mut merged: HashMap<String, StatusSeed> = HashMap::with_capacity(len);
        for (id, seed) in &base.rows {
            if !rows.contains_key(id) {
                merged.insert(id.clone(), seed.clone());
            }
        }
        for (id, seed) in rows {
            if let Some(seed) = seed {
                merged.insert(id.clone(), seed.clone());
            }
        }
        debug_assert_eq!(merged.len(), len);
        let children = build_children(&merged);
        Self {
            rows: merged,
            children,
            len,
        }
    }

    /// Test-only estimate of the heap bytes this snapshot owns. The #67 W3
    /// resident tests compare it with the worker's estimate to prove the worker
    /// keeps one shared copy of the projection instead of a second full clone.
    #[cfg(test)]
    fn resident_bytes_estimate(&self) -> usize {
        rows_bytes_estimate(&self.rows) + children_bytes_estimate(&self.children)
    }
}

/// Test-only: approximate heap bytes of `parent -> visible children` edges.
#[cfg(test)]
fn children_bytes_estimate(children: &HashMap<String, Children>) -> usize {
    std::mem::size_of::<HashMap<String, Children>>()
        + children.capacity() * (std::mem::size_of::<(String, Children)>() + 1)
        + children
            .iter()
            .map(|(parent, list)| {
                parent.capacity()
                    + list.capacity() * (std::mem::size_of::<(String, bool)>() + 1)
                    + list.iter().map(|(child, _)| child.capacity()).sum::<usize>()
            })
            .sum::<usize>()
}

/// Test-only: approximate heap bytes of the row map (keys, bucket table and the
/// per-row `id`/`parent` strings; `StatusSeed` itself lives inline in the table).
#[cfg(test)]
fn rows_bytes_estimate(rows: &HashMap<String, StatusSeed>) -> usize {
    std::mem::size_of::<HashMap<String, StatusSeed>>()
        + rows.capacity() * (std::mem::size_of::<(String, StatusSeed)>() + 1)
        + rows
            .iter()
            .map(|(key, seed)| {
                key.capacity()
                    + seed.id.capacity()
                    + seed.parent.as_ref().map_or(0, String::capacity)
            })
            .sum::<usize>()
}

/// Test-only: approximate heap bytes of the sparse row delta.
#[cfg(test)]
fn row_delta_bytes_estimate(delta: &HashMap<String, Option<StatusSeed>>) -> usize {
    std::mem::size_of::<HashMap<String, Option<StatusSeed>>>()
        + delta.capacity() * (std::mem::size_of::<(String, Option<StatusSeed>)>() + 1)
        + delta
            .iter()
            .map(|(id, seed)| {
                id.capacity()
                    + seed.as_ref().map_or(0, |seed| {
                        seed.id.capacity() + seed.parent.as_ref().map_or(0, String::capacity)
                    })
            })
            .sum::<usize>()
}

/// Test-only: approximate heap bytes of the sparse child-edge delta.
#[cfg(test)]
fn child_delta_bytes_estimate(delta: &HashMap<String, ChildDelta>) -> usize {
    std::mem::size_of::<HashMap<String, ChildDelta>>()
        + delta.capacity() * (std::mem::size_of::<(String, ChildDelta)>() + 1)
        + delta
            .iter()
            .map(|(parent, child)| {
                parent.capacity()
                    + child.removed.capacity() * (std::mem::size_of::<String>() + 1)
                    + child.removed.iter().map(String::capacity).sum::<usize>()
                    + child.changed.capacity() * (std::mem::size_of::<(String, bool)>() + 1)
                    + child.changed.keys().map(String::capacity).sum::<usize>()
            })
            .sum::<usize>()
}

/// Child-edge changes for one parent since the base snapshot.
#[derive(Clone, Default)]
struct ChildDelta {
    removed: HashSet<String>,
    changed: HashMap<String, bool>,
}

impl ChildDelta {
    fn is_empty(&self) -> bool {
        self.removed.is_empty() && self.changed.is_empty()
    }

    fn entries(&self) -> usize {
        self.removed.len() + self.changed.len()
    }
}

/// Sparse changes applied on top of a [`ProjectionBase`].
#[derive(Clone, Default)]
struct ProjectionDelta {
    /// Rows whose value differs from the base (`None` = removed).
    rows: HashMap<String, Option<StatusSeed>>,
    /// Parent id -> changed child edges.
    children: HashMap<String, ChildDelta>,
}

impl ProjectionDelta {
    fn entries(&self) -> usize {
        self.rows.len() + self.children.values().map(ChildDelta::entries).sum::<usize>()
    }
}

/// Immutable committed view of the desktop status seeds.
///
/// A projection is a base snapshot plus the sparse delta accumulated since it;
/// publishing never clones the whole map. `Arc` identity of `base` is shared
/// across projections until a compaction rebases it.
pub(crate) struct StatusProjection {
    generation: u64,
    base: Arc<ProjectionBase>,
    delta: Arc<ProjectionDelta>,
    len: usize,
}

impl StatusProjection {
    #[cfg(test)]
    pub(crate) fn from_rows(rows: Vec<StatusSeed>, generation: u64) -> Self {
        let base = ProjectionBase::from_rows(rows);
        Self {
            generation,
            len: base.len,
            base: Arc::new(base),
            delta: Arc::new(ProjectionDelta::default()),
        }
    }

    pub(crate) fn seed(&self, id: &str) -> Option<&StatusSeed> {
        match self.delta.rows.get(id) {
            Some(next) => next.as_ref(),
            None => self.base.rows.get(id),
        }
    }

    /// Visible children of `id`, sorted by child id.
    ///
    /// The base list is filtered by the sparse child delta and the changed/new
    /// edges are merged back in. Callers always clone the returned vector (the
    /// desktop tree needs an owned batch per directory), so returning `Children`
    /// keeps the immutable-formula simple without extra copying on this path.
    pub(crate) fn children(&self, id: &str) -> Children {
        merge_children(
            self.base.children.get(id).map(Vec::as_slice).unwrap_or(&[]),
            self.delta.children.get(id),
        )
    }

    /// Every visible seed row (base rows not overridden by the delta, then the
    /// delta's inserted/replaced rows).
    #[cfg(test)]
    pub(crate) fn seeds(&self) -> impl Iterator<Item = &StatusSeed> + '_ {
        self.base
            .rows
            .iter()
            .filter(|(id, _)| !self.delta.rows.contains_key(*id))
            .map(|(_, seed)| seed)
            .chain(self.delta.rows.values().filter_map(Option::as_ref))
    }

    /// Whether two projections share their immutable base snapshot. This is the
    /// property that replaces the per-publish deep clone (issue #25).
    #[cfg(test)]
    pub(crate) fn base_shared_with(&self, other: &StatusProjection) -> bool {
        Arc::ptr_eq(&self.base, &other.base)
    }

    /// Test-only estimate of the heap bytes owned by this projection's immutable
    /// base snapshot. Publications share one base, so this is the single-copy
    /// reference the worker's resident estimate is compared against (#67 W3).
    #[cfg(test)]
    pub(crate) fn base_resident_bytes_estimate(&self) -> usize {
        self.base.resident_bytes_estimate()
    }

    pub(crate) fn len(&self) -> usize {
        self.len
    }

    pub(crate) fn generation(&self) -> u64 {
        self.generation
    }
}

/// Read-only merged view of an immutable [`ProjectionBase`] plus the sparse
/// deltas applied on top of it. The worker answers its own queries through this
/// view instead of keeping a second full copy of the base (#67 W3).
struct ProjectionView<'a> {
    base: &'a ProjectionBase,
    rows: &'a HashMap<String, Option<StatusSeed>>,
    children: &'a HashMap<String, ChildDelta>,
}

impl<'a> ProjectionView<'a> {
    /// The visible row for `id`: the delta wins, `None` there means removed.
    fn row(&self, id: &str) -> Option<&StatusSeed> {
        match self.rows.get(id) {
            Some(next) => next.as_ref(),
            None => self.base.rows.get(id),
        }
    }

    fn has_row(&self, id: &str) -> bool {
        self.row(id).is_some()
    }

    /// Visible children of `id`, merged exactly like a published projection's.
    fn children(&self, id: &str) -> Children {
        merge_children(
            self.base.children.get(id).map(Vec::as_slice).unwrap_or(&[]),
            self.children.get(id),
        )
    }

    /// Whether `id` currently has at least one visible child edge. Equivalent to
    /// the old `children.contains_key(id)` (that map only ever holds non-empty
    /// lists), without materialising the full list on the common no-delta path.
    fn has_visible_children(&self, id: &str) -> bool {
        match self.children.get(id) {
            None => self
                .base
                .children
                .get(id)
                .is_some_and(|children| !children.is_empty()),
            Some(_) => !self.children(id).is_empty(),
        }
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

/// Merges one parent's base child list with the sparse delta recorded for it.
///
/// Both the published [`StatusProjection`] and the worker's [`ProjectionView`]
/// call this one function, so a snapshot and the worker can never diverge in
/// removal, insertion or sort-order semantics (#67 W3).
fn merge_children(base: &[(String, bool)], delta: Option<&ChildDelta>) -> Children {
    let Some(delta) = delta.filter(|delta| !delta.is_empty()) else {
        return base.to_vec();
    };
    let mut out: Children = Vec::with_capacity(base.len() + delta.changed.len());
    for (child, directory) in base {
        if delta.removed.contains(child) || delta.changed.contains_key(child) {
            continue;
        }
        out.push((child.clone(), *directory));
    }
    let mut added: Children = delta
        .changed
        .iter()
        .map(|(child, directory)| (child.clone(), *directory))
        .collect();
    added.sort_by(|left, right| left.0.cmp(&right.0));
    out.extend(added);
    out.sort_by(|left, right| left.0.cmp(&right.0));
    out
}

fn same_seed(left: &StatusSeed, right: &StatusSeed) -> bool {
    left.id == right.id
        && left.parent == right.parent
        && left.directory == right.directory
        && left.pinned == right.pinned
        && left.overlay == right.overlay
        && left.detailed == right.detailed
}

/// Lifecycle phase observable through [`StatusCache::metrics`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum CacheState {
    /// No snapshot published yet.
    #[default]
    Starting,
    /// Worker is following the journal and publishing snapshots.
    Running,
    /// Last known snapshot (if any) keeps serving; the worker retries.
    Degraded,
}

/// Why the cache is degraded. Every path that used to permanently disable the
/// projection now reports one of these and retries.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DegradeReason {
    /// `SharedStore::take_notifications` was rejected (another consumer claimed
    /// the subscription or the worker lost it).
    NotificationsUnavailable,
    /// Opening the read-only handle failed.
    ReadOnlyOpenFailed,
    /// Bootstrap scan or the startup journal replay failed.
    BootstrapFailed,
    /// Full-scan rebuild failed.
    RebuildFailed,
    /// The profile grew past the configured projection cap.
    TooManyNodes,
    /// Repeated journal read failures.
    ReadFailed,
}

/// Snapshot of the cache's observable state. Restarting the worker is never
/// required to make progress again; callers can poll this to surface the
/// degraded reason and the coming retry.
#[derive(Clone, Debug)]
pub(crate) struct StatusCacheMetrics {
    pub state: CacheState,
    pub degraded_reason: Option<DegradeReason>,
    /// Milliseconds spent continuously degraded (0 while running).
    pub degraded_ms: u64,
    /// Consecutive failed recovery attempts.
    pub recovery_attempts: u32,
    pub next_retry_in_ms: Option<u64>,
    pub degraded_events: u64,
    pub recovered_events: u64,
    /// Generation of the currently published snapshot.
    pub generation: u64,
    pub rows: usize,
    /// Journal pages resolved so far.
    pub pages: u64,
    pub lock_holds: u64,
    pub lock_hold_p99_us: u64,
    pub lock_hold_max_us: u64,
    pub lock_budget_us: u64,
    /// Holds that exceeded [`LOCK_BUDGET`]; a non-zero value is a regression.
    pub lock_budget_violations: u64,
    /// Pages that were followed by a deliberate writer-lock yield.
    pub backlog_yields: u64,
    pub read_errors: u64,
    pub publishes: u64,
    pub rebuilds: u64,
    /// Scoped `access_denied` refreshes (instead of full rebuilds).
    pub acl_refreshes: u64,
    /// Full base rebases performed by publication.
    pub compactions: u64,
    /// Total sparse delta entries published across all publications.
    pub publish_delta_entries: u64,
    /// Publications that reused an existing base snapshot instead of cloning it.
    pub base_shares: u64,
}

#[derive(Default)]
struct CacheStatus {
    state: CacheState,
    reason: Option<DegradeReason>,
    since: Option<Instant>,
    attempts: u32,
    next_retry: Option<Instant>,
}

/// Test-only failure injection. Each counter permits that many upcoming calls to
/// fail; production builds always carry `None`.
#[derive(Default)]
struct Faults {
    take_notifications: AtomicU32,
    open_read_only: AtomicU32,
    bootstrap: AtomicU32,
    rebuild: AtomicU32,
    read: AtomicU32,
}

#[derive(Clone, Copy)]
enum FaultKind {
    TakeNotifications,
    OpenReadOnly,
    Bootstrap,
    Rebuild,
    Read,
}

impl Faults {
    fn counter(&self, kind: FaultKind) -> &AtomicU32 {
        match kind {
            FaultKind::TakeNotifications => &self.take_notifications,
            FaultKind::OpenReadOnly => &self.open_read_only,
            FaultKind::Bootstrap => &self.bootstrap,
            FaultKind::Rebuild => &self.rebuild,
            FaultKind::Read => &self.read,
        }
    }

    fn consume(&self, kind: FaultKind) -> bool {
        let counter = self.counter(kind);
        let mut remaining = counter.load(Ordering::Relaxed);
        while remaining > 0 {
            match counter.compare_exchange_weak(
                remaining,
                remaining - 1,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => return true,
                Err(current) => remaining = current,
            }
        }
        false
    }
}

/// Worker timing/limits. Production uses [`CacheConfig::production`]; tests
/// shrink the timings and attach [`Faults`] to exercise degradation/recovery.
#[derive(Clone)]
pub(crate) struct CacheConfig {
    max_nodes: usize,
    backlog_limit: u64,
    publish_interval: Duration,
    backlog_yield: Duration,
    lock_budget: Duration,
    retry_initial: Duration,
    retry_max: Duration,
    faults: Option<Arc<Faults>>,
}

impl CacheConfig {
    fn production(max_nodes: usize, backlog_limit: u64) -> Self {
        Self {
            max_nodes,
            backlog_limit,
            publish_interval: PUBLISH_INTERVAL,
            backlog_yield: BACKLOG_YIELD,
            lock_budget: LOCK_BUDGET,
            retry_initial: RETRY_INITIAL,
            retry_max: RETRY_MAX,
            faults: None,
        }
    }
}

/// Background owner of the published projection.
pub(crate) struct StatusCache {
    current: Mutex<Option<Arc<StatusProjection>>>,
    status: Mutex<CacheStatus>,
    /// Monotonic across worker restarts: a recovery must not publish a lower
    /// generation than the snapshot it replaces.
    generation: AtomicU64,
    rebuilds: AtomicU64,
    publishes: AtomicU64,
    degraded_events: AtomicU64,
    recovered_events: AtomicU64,
    lock_holds: AtomicU64,
    lock_hold_max: AtomicU64,
    lock_buckets: [AtomicU64; LOCK_BUCKETS_US.len()],
    lock_budget_violations: AtomicU64,
    backlog_yields: AtomicU64,
    pages: AtomicU64,
    read_errors: AtomicU64,
    acl_refreshes: AtomicU64,
    compactions: AtomicU64,
    publish_delta_entries: AtomicU64,
    base_shares: AtomicU64,
    /// Test-only (#67 W3): heap bytes the worker reported for its projection
    /// state right before the last publication. The resident regression tests
    /// compare it with the shared base's single-copy estimate.
    #[cfg(test)]
    worker_resident_bytes: AtomicU64,
    config: CacheConfig,
    /// Test-only capture of the structured lifecycle lines the product writes to
    /// stderr; lets tests assert the same events without hijacking stderr.
    #[cfg(test)]
    health_events: Mutex<Vec<String>>,
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
        Self::spawn(service, CacheConfig::production(max_nodes, backlog_limit))
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

    /// Test seam: full timing/fault control.
    #[cfg(test)]
    pub(crate) fn start_with_config(
        service: Arc<Service>,
        config: CacheConfig,
    ) -> (Arc<Self>, std::thread::JoinHandle<()>) {
        Self::spawn(service, config)
    }

    fn spawn(service: Arc<Service>, config: CacheConfig) -> (Arc<Self>, std::thread::JoinHandle<()>) {
        let cache = Arc::new(Self {
            current: Mutex::new(None),
            status: Mutex::new(CacheStatus::default()),
            generation: AtomicU64::new(0),
            rebuilds: AtomicU64::new(0),
            publishes: AtomicU64::new(0),
            degraded_events: AtomicU64::new(0),
            recovered_events: AtomicU64::new(0),
            lock_holds: AtomicU64::new(0),
            lock_hold_max: AtomicU64::new(0),
            lock_buckets: std::array::from_fn(|_| AtomicU64::new(0)),
            lock_budget_violations: AtomicU64::new(0),
            backlog_yields: AtomicU64::new(0),
            pages: AtomicU64::new(0),
            read_errors: AtomicU64::new(0),
            acl_refreshes: AtomicU64::new(0),
            compactions: AtomicU64::new(0),
            publish_delta_entries: AtomicU64::new(0),
            base_shares: AtomicU64::new(0),
            #[cfg(test)]
            worker_resident_bytes: AtomicU64::new(0),
            config,
            #[cfg(test)]
            health_events: Mutex::new(Vec::new()),
        });
        let worker = cache.clone();
        let task = std::thread::spawn(move || worker.run(service));
        (cache, task)
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

    /// Test-only (#67 W3): the worker's last reported heap estimate for its
    /// projection state (shared immutable base plus sparse deltas).
    #[cfg(test)]
    pub(crate) fn worker_resident_bytes(&self) -> u64 {
        self.worker_resident_bytes.load(Ordering::Relaxed)
    }

    /// Test-only (#67 W3): records the worker's projection-state heap estimate
    /// right before a publication.
    #[cfg(test)]
    fn record_worker_resident_bytes(&self, bytes: usize) {
        self.worker_resident_bytes
            .store(bytes as u64, Ordering::Relaxed);
    }

    pub(crate) fn snapshot(&self) -> Option<Arc<StatusProjection>> {
        self.current
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    /// Product-facing health view: lifecycle state, degradation reason/backoff and
    /// the cost counters. Lifecycle logging reads it on every transition, and a
    /// future diagnostics/IPC surface can expose it without changing any JSON or
    /// D-Bus contract today.
    pub(crate) fn health_snapshot(&self) -> StatusCacheMetrics {
        self.metrics()
    }

    /// Test-only view of the lifecycle lines the worker emitted, in order.
    #[cfg(test)]
    pub(crate) fn health_events(&self) -> Vec<String> {
        self.health_events
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    /// Writes one structured lifecycle line to the product diagnostic channel:
    /// stderr, which the daemon's systemd unit collects in the journal (the same
    /// convention as `bridge.rs`/`shutdown.rs`). Every line carries the full
    /// health snapshot so a degraded cache can be diagnosed from logs alone; the
    /// unit tests capture the identical lines in-process instead of hijacking
    /// the global stderr stream.
    fn emit_health(&self, level: &str, event: &str, extra: &str) {
        let health = self.health_snapshot();
        let mut line = format!(
            "drive-native status cache: level={level} event={event} \
             state={:?} degraded_reason={:?} degraded_ms={} recovery_attempts={} \
             next_retry_in_ms={:?} degraded_events={} recovered_events={} \
             generation={} rows={} pages={} lock_holds={} lock_hold_p99_us={} \
             lock_hold_max_us={} lock_budget_us={} lock_budget_violations={} \
             backlog_yields={} read_errors={} publishes={} rebuilds={} \
             acl_refreshes={} compactions={} publish_delta_entries={} base_shares={}",
            health.state,
            health.degraded_reason,
            health.degraded_ms,
            health.recovery_attempts,
            health.next_retry_in_ms,
            health.degraded_events,
            health.recovered_events,
            health.generation,
            health.rows,
            health.pages,
            health.lock_holds,
            health.lock_hold_p99_us,
            health.lock_hold_max_us,
            health.lock_budget_us,
            health.lock_budget_violations,
            health.backlog_yields,
            health.read_errors,
            health.publishes,
            health.rebuilds,
            health.acl_refreshes,
            health.compactions,
            health.publish_delta_entries,
            health.base_shares,
        );
        if !extra.is_empty() {
            line.push(' ');
            line.push_str(extra);
        }
        #[cfg(test)]
        self.health_events
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(line.clone());
        eprintln!("{line}");
    }

    /// Observable lifecycle and cost counters. A snapshot keeps being served
    /// while `state == Degraded`; the reason and retry countdown explain why.
    /// The lifecycle log reads this on every state transition; tests are the
    /// other consumer.
    pub(crate) fn metrics(&self) -> StatusCacheMetrics {
        let current = self
            .current
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        let status = self
            .status
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        StatusCacheMetrics {
            state: status.state,
            degraded_reason: status.reason,
            degraded_ms: status
                .since
                .map(|since| since.elapsed().as_millis().min(u128::from(u64::MAX)) as u64)
                .unwrap_or(0),
            recovery_attempts: status.attempts,
            next_retry_in_ms: status.next_retry.map(|retry| {
                retry
                    .saturating_duration_since(Instant::now())
                    .as_millis()
                    .min(u128::from(u64::MAX)) as u64
            }),
            degraded_events: self.degraded_events.load(Ordering::Relaxed),
            recovered_events: self.recovered_events.load(Ordering::Relaxed),
            generation: current
                .as_ref()
                .map_or_else(|| self.generation.load(Ordering::Relaxed), |row| row.generation()),
            rows: current.as_ref().map_or(0, |row| row.len()),
            pages: self.pages.load(Ordering::Relaxed),
            lock_holds: self.lock_holds.load(Ordering::Relaxed),
            lock_hold_p99_us: self.lock_hold_p99_us(),
            lock_hold_max_us: self.lock_hold_max.load(Ordering::Relaxed),
            lock_budget_us: self.config.lock_budget.as_micros() as u64,
            lock_budget_violations: self.lock_budget_violations.load(Ordering::Relaxed),
            backlog_yields: self.backlog_yields.load(Ordering::Relaxed),
            read_errors: self.read_errors.load(Ordering::Relaxed),
            publishes: self.publishes.load(Ordering::Relaxed),
            rebuilds: self.rebuilds.load(Ordering::Relaxed),
            acl_refreshes: self.acl_refreshes.load(Ordering::Relaxed),
            compactions: self.compactions.load(Ordering::Relaxed),
            publish_delta_entries: self.publish_delta_entries.load(Ordering::Relaxed),
            base_shares: self.base_shares.load(Ordering::Relaxed),
        }
    }

    fn next_generation(&self) -> u64 {
        self.generation.fetch_add(1, Ordering::Relaxed) + 1
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

    fn record_lock_hold(&self, elapsed: Duration) {
        let micros = elapsed.as_micros().min(u128::from(u64::MAX)) as u64;
        self.lock_holds.fetch_add(1, Ordering::Relaxed);
        self.lock_hold_max.fetch_max(micros, Ordering::Relaxed);
        for (index, bound) in LOCK_BUCKETS_US.iter().enumerate() {
            if micros <= *bound {
                self.lock_buckets[index].fetch_add(1, Ordering::Relaxed);
                break;
            }
        }
        if elapsed > self.config.lock_budget {
            let violations = self.lock_budget_violations.fetch_add(1, Ordering::Relaxed) + 1;
            // Log the first overrun and then geometric milestones: a regression is
            // visible without letting a pathological hold flood the journal.
            if violations.is_power_of_two() {
                self.emit_health(
                    "warn",
                    "lock_budget_exceeded",
                    &format!(
                        "hold_us={micros} budget_us={} violations={violations}",
                        self.config.lock_budget.as_micros()
                    ),
                );
            }
        }
    }

    fn lock_hold_p99_us(&self) -> u64 {
        let total: u64 = self
            .lock_buckets
            .iter()
            .map(|bucket| bucket.load(Ordering::Relaxed))
            .sum();
        if total == 0 {
            return 0;
        }
        let target = (total * 99 + 99) / 100;
        let mut seen = 0u64;
        for (index, bound) in LOCK_BUCKETS_US.iter().enumerate() {
            seen += self.lock_buckets[index].load(Ordering::Relaxed);
            if seen >= target {
                return *bound;
            }
        }
        u64::MAX
    }

    fn fault(&self, kind: FaultKind) -> bool {
        self.config
            .faults
            .as_ref()
            .is_some_and(|faults| faults.consume(kind))
    }

    fn mark_running(&self) {
        let (first_ready, recovered, downtime_ms) = {
            let mut status = self
                .status
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let first_ready = status.state == CacheState::Starting;
            let recovered = status.state == CacheState::Degraded;
            let downtime_ms = if recovered {
                status
                    .since
                    .map(|since| since.elapsed().as_millis().min(u128::from(u64::MAX)) as u64)
                    .unwrap_or(0)
            } else {
                0
            };
            if recovered {
                self.recovered_events.fetch_add(1, Ordering::Relaxed);
            }
            status.state = CacheState::Running;
            status.reason = None;
            status.since = None;
            status.attempts = 0;
            status.next_retry = None;
            (first_ready, recovered, downtime_ms)
        };
        if recovered {
            self.emit_health("info", "recovered", &format!("downtime_ms={downtime_ms}"));
        } else if first_ready {
            self.emit_health("info", "ready", "");
        }
    }

    /// Enters the degraded state. Returns whether this transition is newly
    /// observable (first degradation or a changed reason) and the attempt count.
    fn degrade(&self, reason: DegradeReason) -> (bool, u32) {
        let mut status = self
            .status
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let transitioned = status.state != CacheState::Degraded;
        let reason_changed = status.reason != Some(reason);
        if transitioned {
            self.degraded_events.fetch_add(1, Ordering::Relaxed);
            status.since = Some(Instant::now());
        }
        status.state = CacheState::Degraded;
        status.reason = Some(reason);
        status.attempts = status.attempts.saturating_add(1);
        status.next_retry = None;
        (transitioned || reason_changed, status.attempts)
    }

    /// Enters the degraded state and waits before the next recovery attempt.
    /// Returns false when the service is stopping.
    fn degrade_and_wait(&self, reason: DegradeReason, stop: &AtomicBool) -> bool {
        let (transition, attempts) = self.degrade(reason);
        let wait = {
            let mut status = self
                .status
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let factor = 1u32 << status.attempts.min(6);
            let wait = self
                .config
                .retry_initial
                .saturating_mul(factor)
                .min(self.config.retry_max);
            status.next_retry = Some(Instant::now() + wait);
            wait
        };
        // The first degradation (and each reason change) is a warn event; retries
        // are recorded at debug level on geometric attempt counts, so an outage
        // cannot flood the product journal.
        if transition {
            self.emit_health(
                "warn",
                "degraded",
                &format!(
                    "reason={reason:?} attempts={attempts} next_retry_in_ms={}",
                    wait.as_millis()
                ),
            );
        } else if attempts.is_power_of_two() {
            self.emit_health(
                "debug",
                "degraded_retry",
                &format!(
                    "reason={reason:?} attempts={attempts} next_retry_in_ms={}",
                    wait.as_millis()
                ),
            );
        }
        sleep_stop_aware(wait, stop)
    }

    fn run(self: Arc<Self>, service: Arc<Service>) {
        let shared = service.bridge.store.clone();
        let stop = &service.stop;
        // Single subscriber: a rejected claim used to disable the cache forever.
        // Keep retrying with a visible degraded state instead; the authoritative
        // SQLite path serves callers while no snapshot is published.
        let notifications = loop {
            if stop.load(Ordering::Acquire) {
                return;
            }
            if self.fault(FaultKind::TakeNotifications) {
                if !self.degrade_and_wait(DegradeReason::NotificationsUnavailable, stop) {
                    return;
                }
                continue;
            }
            match shared.take_notifications() {
                Ok(notifications) => break notifications,
                Err(_) => {
                    if !self.degrade_and_wait(DegradeReason::NotificationsUnavailable, stop) {
                        return;
                    }
                }
            }
        };
        loop {
            if stop.load(Ordering::Acquire) {
                return;
            }
            match self.cycle(&service, &shared, &notifications) {
                Cycle::Stop => return,
                Cycle::Degrade(reason) => {
                    if !self.degrade_and_wait(reason, stop) {
                        return;
                    }
                }
            }
        }
    }

    /// One open → bootstrap → follow-journal cycle. A cycle hands control back to
    /// [`Self::run`] with either `Stop` or a degradation reason; the last good
    /// snapshot (if any) stays published meanwhile.
    fn cycle(
        &self,
        service: &Arc<Service>,
        shared: &SharedStore,
        notifications: &CommitNotifications,
    ) -> Cycle {
        if self.fault(FaultKind::OpenReadOnly) {
            return Cycle::Degrade(DegradeReason::ReadOnlyOpenFailed);
        }
        let store = match shared.read_only() {
            Ok(store) => store,
            Err(_) => return Cycle::Degrade(DegradeReason::ReadOnlyOpenFailed),
        };
        let mut worker = Worker::new(self, store);
        if self.fault(FaultKind::Bootstrap) {
            return Cycle::Degrade(DegradeReason::BootstrapFailed);
        }
        match worker.bootstrap(shared, &service.stop) {
            Flow::Stop => return Cycle::Stop,
            Flow::Degrade(reason) => return Cycle::Degrade(reason),
            Flow::Continue => {}
        }
        self.mark_running();
        match worker.serve(shared, notifications, &service.stop) {
            Flow::Stop => Cycle::Stop,
            Flow::Degrade(reason) => Cycle::Degrade(reason),
            Flow::Continue => Cycle::Stop,
        }
    }
}

/// Outcome of one part of the worker: keep going, stop the service, or surface a
/// degradation reason to the retry loop.
enum Flow {
    Continue,
    Stop,
    Degrade(DegradeReason),
}

enum Cycle {
    Stop,
    Degrade(DegradeReason),
}

/// One page read from the journal, or the alternative maintenance action.
enum LockedPage {
    Empty,
    Rebuild(StatusCursor),
    Ready(JournalPage),
}

/// A journal page whose invalidations still have to be resolved against the
/// read-only store (outside the writer guard).
struct JournalPage {
    changes: Vec<StatusChange>,
    next: StatusCursor,
    has_more: bool,
    /// Watermark captured with the page, used if resolving needs a rebuild.
    rebuild_cursor: StatusCursor,
}

/// One page of journal invalidations, or the alternative maintenance action.
enum Batch {
    Empty,
    Rebuild(StatusCursor),
    Changes {
        updates: Vec<(String, Option<StatusSeed>)>,
        next: StatusCursor,
        has_more: bool,
    },
}

/// Worker state for one cache cycle. Everything mutable is owned here and only
/// published through sparse deltas.
///
/// The worker never keeps a second full copy of the projection: `base` is the
/// immutable snapshot shared with every published projection, and all mutations
/// are recorded in the sparse `row_delta`/`child_delta` maps on top of it
/// (#67 W3). Queries go through [`Worker::view`].
struct Worker<'a> {
    cache: &'a StatusCache,
    store: drive_core::Store,
    config: CacheConfig,
    cursor: StatusCursor,
    base: Arc<ProjectionBase>,
    /// Visible-row changes since `base` (`None` = removed).
    row_delta: HashMap<String, Option<StatusSeed>>,
    /// Child-edge changes since `base`.
    child_delta: HashMap<String, ChildDelta>,
    len: usize,
    dirty: bool,
    last_publish: Option<Instant>,
    idle_wait: Duration,
    /// Cursor of the last page whose changes were applied but not yet pruned.
    /// Pruning is deferred to the next page so it never shares a guard with the
    /// resolution work.
    prune_pending: Option<StatusCursor>,
}

impl<'a> Worker<'a> {
    fn new(cache: &'a StatusCache, store: drive_core::Store) -> Self {
        Self {
            cache,
            store,
            config: cache.config.clone(),
            cursor: StatusCursor {
                epoch: String::new(),
                sequence: 0,
            },
            base: Arc::new(ProjectionBase::default()),
            row_delta: HashMap::new(),
            child_delta: HashMap::new(),
            len: 0,
            dirty: false,
            last_publish: None,
            idle_wait: WAIT,
            prune_pending: None,
        }
    }

    /// Read-only merged view over the shared base and the sparse deltas.
    fn view(&self) -> ProjectionView<'_> {
        ProjectionView {
            base: &self.base,
            rows: &self.row_delta,
            children: &self.child_delta,
        }
    }

    fn fault(&self, kind: FaultKind) -> bool {
        self.cache.fault(kind)
    }

    /// Test-only (#67 W3) estimate of the heap bytes owned by this worker's
    /// projection state: the shared immutable base (one copy for the whole
    /// process) plus the sparse deltas recorded since the last rebase.
    #[cfg(test)]
    fn resident_bytes_estimate(&self) -> usize {
        self.base.resident_bytes_estimate()
            + row_delta_bytes_estimate(&self.row_delta)
            + child_delta_bytes_estimate(&self.child_delta)
    }

    /// Initial snapshot plus startup journal replay up to `STARTUP_REPLAY_LIMIT`.
    /// The guard is held only for the budget/journal setup; the full scan and the
    /// replay pages read through the worker's own read-only connection.
    fn bootstrap(&mut self, shared: &SharedStore, stop: &AtomicBool) -> Flow {
        let Some(mut guard) = lock_stop_aware(shared, stop) else {
            return Flow::Stop;
        };
        let started = Instant::now();
        let setup: drive_core::Result<StatusCursor> = (|| {
            guard.set_status_journal_budget(JournalBudget {
                entries: JOURNAL_ENTRIES,
                bytes: JOURNAL_BYTES,
            })?;
            Ok(guard.status_change_watermark()?.cursor)
        })();
        self.cache.record_lock_hold(started.elapsed());
        drop(guard);
        let cursor = match setup {
            Ok(cursor) => cursor,
            Err(_) => return Flow::Degrade(DegradeReason::BootstrapFailed),
        };
        if stop.load(Ordering::Acquire) {
            return Flow::Stop;
        }
        let rows = match self.store.desktop_status_seeds() {
            Ok(rows) => rows,
            Err(_) => return Flow::Degrade(DegradeReason::BootstrapFailed),
        };
        if !self.install_full(rows) {
            return Flow::Degrade(DegradeReason::TooManyNodes);
        }
        // The scan may already include commits newer than `cursor`; replaying
        // them later only re-reads current facts, so it stays idempotent.
        self.cursor = cursor;
        let deadline = Instant::now() + STARTUP_REPLAY_LIMIT;
        loop {
            if stop.load(Ordering::Acquire) {
                return Flow::Stop;
            }
            match self.read_batch(shared, stop) {
                Ok(Some(Batch::Empty)) => break,
                Ok(Some(Batch::Changes {
                    updates,
                    next,
                    has_more,
                })) => {
                    self.apply(updates);
                    self.cursor = next;
                    if self.len > self.config.max_nodes {
                        return Flow::Degrade(DegradeReason::TooManyNodes);
                    }
                    if !has_more {
                        break;
                    }
                }
                Ok(Some(Batch::Rebuild(cursor))) => {
                    if !self.rebuild(cursor, stop) {
                        return Flow::Degrade(DegradeReason::RebuildFailed);
                    }
                    break;
                }
                Ok(None) => return Flow::Stop,
                Err(_) => return Flow::Degrade(DegradeReason::BootstrapFailed),
            }
            if Instant::now() >= deadline {
                break;
            }
        }
        self.publish_now();
        Flow::Continue
    }

    /// Main maintenance loop. Idle wake-ups are cheap and publish nothing.
    fn serve(
        &mut self,
        shared: &SharedStore,
        notifications: &CommitNotifications,
        stop: &AtomicBool,
    ) -> Flow {
        let mut read_errors = 0u32;
        loop {
            if stop.load(Ordering::Acquire) {
                return Flow::Stop;
            }
            let mut wait = true;
            if notifications.state().verified {
                match self.read_batch(shared, stop) {
                    Ok(Some(Batch::Empty)) => read_errors = 0,
                    Ok(Some(Batch::Changes {
                        updates,
                        next,
                        has_more,
                    })) => {
                        read_errors = 0;
                        self.apply(updates);
                        self.cursor = next;
                        // The cap is enforced after every applied batch: an
                        // incremental burst must not grow a published projection
                        // past what the caller was promised it would cache.
                        if self.len > self.config.max_nodes {
                            return Flow::Degrade(DegradeReason::TooManyNodes);
                        }
                        self.dirty = true;
                        self.idle_wait = WAIT;
                        self.publish_if_due();
                        wait = !has_more;
                        if has_more {
                            // Yield the writer between backlog pages; the old
                            // loop re-locked back-to-back and could starve writers
                            // for as long as the backlog lasted.
                            self.cache.backlog_yields.fetch_add(1, Ordering::Relaxed);
                            if !sleep_stop_aware(self.config.backlog_yield, stop) {
                                return Flow::Stop;
                            }
                        }
                    }
                    Ok(Some(Batch::Rebuild(cursor))) => {
                        read_errors = 0;
                        if !self.rebuild(cursor, stop) {
                            return Flow::Degrade(DegradeReason::RebuildFailed);
                        }
                        self.publish_now();
                        wait = false;
                    }
                    Ok(None) => return Flow::Stop,
                    Err(_) => {
                        self.cache.read_errors.fetch_add(1, Ordering::Relaxed);
                        read_errors += 1;
                        if read_errors >= READ_ERROR_LIMIT {
                            return Flow::Degrade(DegradeReason::ReadFailed);
                        }
                        let _ = notifications.recv_timeout(WAIT);
                    }
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

    /// Reads one journal page under the short writer guard, then resolves its
    /// invalidations against the read-only store after the guard is dropped.
    /// `Ok(None)` means the stop signal arrived while waiting for the writer.
    fn read_batch(
        &mut self,
        shared: &SharedStore,
        stop: &AtomicBool,
    ) -> drive_core::Result<Option<Batch>> {
        if self.fault(FaultKind::Read) {
            return Err(drive_core::Error::State(
                "injected status journal read failure".into(),
            ));
        }
        let Some(mut guard) = lock_stop_aware(shared, stop) else {
            return Ok(None);
        };
        let started = Instant::now();
        let outcome = self.read_locked(&mut guard);
        drop(guard);
        self.cache.record_lock_hold(started.elapsed());
        match outcome? {
            LockedPage::Empty => Ok(Some(Batch::Empty)),
            LockedPage::Rebuild(cursor) => Ok(Some(Batch::Rebuild(cursor))),
            LockedPage::Ready(page) => self.resolve(page),
        }
    }

    /// The only writer-guard work: deferred prune of the previously applied page,
    /// the watermark read and one journal page. Cost is independent of the
    /// namespace size and of the page's subtree scope (resolution happens later
    /// on the read-only handle).
    fn read_locked(&mut self, guard: &mut StoreGuard<'_>) -> drive_core::Result<LockedPage> {
        if let Some(acknowledged) = self.prune_pending.take() {
            guard.prune_status_changes(&acknowledged)?;
        }
        let watermark = guard.status_change_watermark()?;
        if watermark
            .cursor
            .sequence
            .saturating_sub(self.cursor.sequence)
            > self.config.backlog_limit
        {
            return Ok(LockedPage::Rebuild(watermark.cursor));
        }
        match guard.read_status_changes(&self.cursor, BATCH_LIMIT)? {
            ChangePage::Ready {
                changes,
                next,
                has_more,
                watermark,
            } => {
                if changes.is_empty() {
                    return Ok(LockedPage::Empty);
                }
                // A page is not a transaction boundary; acknowledging this page
                // is safe as soon as its changes have been resolved (next read).
                self.prune_pending = Some(next.clone());
                self.cache.pages.fetch_add(1, Ordering::Relaxed);
                Ok(LockedPage::Ready(JournalPage {
                    changes,
                    next,
                    has_more,
                    rebuild_cursor: watermark.cursor,
                }))
            }
            ChangePage::Rebuild { watermark, .. } => Ok(LockedPage::Rebuild(watermark.cursor)),
        }
    }

    /// Resolves one page's invalidations on the read-only connection. Journal
    /// entries are invalidations, so reading current committed facts here (even
    /// newer than the page) is safe: replaying a later page only re-reads the
    /// same facts.
    fn resolve(&self, page: JournalPage) -> drive_core::Result<Option<Batch>> {
        let view = self.view();
        let mut ids: HashSet<String> = HashSet::new();
        let mut keys: HashSet<(Option<String>, String)> = HashSet::new();
        let mut subtree_roots: Vec<String> = Vec::new();
        let mut acl_denied: Vec<String> = Vec::new();
        for change in &page.changes {
            // `access_blocked` is a view over `access_denied`, `nodes.parent` and
            // the overlay remote-parent edge. The closure below the denied id in
            // that graph is exactly the rows whose visibility can change; it is
            // re-read scoped instead of rebuilding the whole namespace (#25).
            if change.source == "access_denied" {
                if let Some(id) = change.new_node.as_ref().or(change.old_node.as_ref()) {
                    if !acl_denied.contains(id) {
                        acl_denied.push(id.clone());
                    }
                }
                continue;
            }
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
                && subtree_needs_refresh(&view, change)
            {
                // A namespace-root location (`parent IS NULL`) would make the
                // scoped query take its full-scan fallback; never do that.
                if change.new_parent.is_none() || change.old_parent.is_none() {
                    return Ok(Some(Batch::Rebuild(page.rebuild_cursor)));
                }
                if let Some(id) = change.new_node.as_ref().or(change.old_node.as_ref()) {
                    subtree_roots.push(id.clone());
                }
            }
        }
        let mut updates: Vec<(String, Option<StatusSeed>)> =
            Vec::with_capacity(ids.len() + keys.len() + subtree_roots.len());
        let mut seen: HashSet<String> = HashSet::with_capacity(ids.len() + keys.len());
        let mut walked: HashSet<String> = HashSet::new();
        let mut visible: HashSet<String> = HashSet::new();
        let mut subtree_seeds: Vec<StatusSeed> = Vec::new();
        if !subtree_roots.is_empty() {
            subtree_roots.sort_unstable();
            subtree_roots.dedup();
            // Old visible rows below the roots come from the merged view; the
            // scoped query then tells which of them are still visible.
            let mut stack = subtree_roots.clone();
            while let Some(id) = stack.pop() {
                if !walked.insert(id.clone()) {
                    continue;
                }
                for (child, _) in view.children(&id) {
                    stack.push(child.clone());
                }
            }
            if walked.len() > SUBTREE_APPLY_LIMIT {
                return Ok(Some(Batch::Rebuild(page.rebuild_cursor)));
            }
            subtree_seeds = self.store.desktop_status_seeds_for(&subtree_roots)?;
            if subtree_seeds.len() > SUBTREE_APPLY_LIMIT {
                return Ok(Some(Batch::Rebuild(page.rebuild_cursor)));
            }
        }
        for id in ids {
            if !seen.insert(id.clone()) {
                continue;
            }
            updates.push((id.clone(), self.store.desktop_status_seed(&id)?));
        }
        for (parent, name) in keys {
            for id in self.store.desktop_status_ids_at(parent.as_deref(), &name)? {
                if !seen.insert(id.clone()) {
                    continue;
                }
                updates.push((id.clone(), self.store.desktop_status_seed(&id)?));
            }
        }
        // `desktop_status_seeds_for` includes the roots' ancestor chain, so those
        // rows (needed by `StatusTree::inherited`) are re-read as well.
        for seed in subtree_seeds {
            visible.insert(seed.id.clone());
            if seen.insert(seed.id.clone()) {
                updates.push((seed.id.clone(), Some(seed)));
            }
        }
        for id in walked {
            // A row that another change on the same page already refreshed (for
            // example a descendant that moved out of the walked directory) must
            // keep that newer state; dropping it here would hide a visible row.
            if !visible.contains(&id) && !seen.contains(&id) {
                updates.push((id, None));
            }
        }
        if !acl_denied.is_empty() {
            acl_denied.sort_unstable();
            acl_denied.dedup();
            let limit = self.config.max_nodes.min(SUBTREE_APPLY_LIMIT);
            let Some(refresh) = self.store.desktop_status_acl_refresh(&acl_denied, limit)? else {
                return Ok(Some(Batch::Rebuild(page.rebuild_cursor)));
            };
            self.cache.acl_refreshes.fetch_add(1, Ordering::Relaxed);
            let mut seeds: HashMap<&str, &StatusSeed> = refresh
                .seeds
                .iter()
                .map(|seed| (seed.id.as_str(), seed))
                .collect();
            // `affected` is the closure that may have changed visibility; rows now
            // visible come back as seeds, the rest are removed from the
            // projection. Ancestor rows outside the closure are unchanged.
            for id in &refresh.affected {
                updates.push((id.clone(), seeds.remove(id.as_str()).cloned()));
            }
        }
        Ok(Some(Batch::Changes {
            updates,
            next: page.next,
            has_more: page.has_more,
        }))
    }

    /// Rebuilds the working projection from a fresh committed scan. The previous
    /// snapshot keeps serving readers until [`Self::publish_now`].
    fn rebuild(&mut self, cursor: StatusCursor, stop: &AtomicBool) -> bool {
        if stop.load(Ordering::Acquire) {
            return false;
        }
        if self.fault(FaultKind::Rebuild) {
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

    /// Replaces the working state with a fresh full scan: the immutable base
    /// becomes the only full copy and the sparse deltas are reset (#67 W3).
    fn install_full(&mut self, rows: Vec<StatusSeed>) -> bool {
        if rows.len() > self.config.max_nodes {
            return false;
        }
        let base = ProjectionBase::from_rows(rows);
        self.len = base.len;
        self.base = Arc::new(base);
        self.row_delta.clear();
        self.child_delta.clear();
        self.dirty = true;
        true
    }

    fn apply(&mut self, updates: Vec<(String, Option<StatusSeed>)>) {
        for (id, next) in updates {
            self.set_row(&id, next);
        }
    }

    fn set_row(&mut self, id: &str, next: Option<StatusSeed>) {
        let previous = self.view().row(id).cloned();
        match (previous, next) {
            (Some(previous), Some(next)) => {
                if same_seed(&previous, &next) {
                    // Row-local state did not change; keep the published delta
                    // sparse so an unchanged re-read costs nothing.
                    return;
                }
                if previous.parent != next.parent || previous.directory != next.directory {
                    if let Some(parent) = &previous.parent {
                        self.remove_child(parent, id);
                    }
                    self.upsert_child(&next);
                }
                self.row_delta.insert(id.to_owned(), Some(next));
            }
            (Some(previous), None) => {
                if let Some(parent) = &previous.parent {
                    self.remove_child(parent, id);
                }
                self.len = self.len.saturating_sub(1);
                self.row_delta.insert(id.to_owned(), None);
            }
            (None, Some(next)) => {
                self.upsert_child(&next);
                self.row_delta.insert(id.to_owned(), Some(next));
                self.len += 1;
            }
            (None, None) => {}
        }
        self.dirty = true;
        // `id` is the map key; keep the stored row id authoritative.
        debug_assert!(self.view().row(id).is_none_or(|row| row.id == id));
    }

    fn upsert_child(&mut self, seed: &StatusSeed) {
        let Some(parent) = seed.parent.as_ref() else {
            return;
        };
        let delta = self.child_delta.entry(parent.clone()).or_default();
        delta.removed.remove(&seed.id);
        delta.changed.insert(seed.id.clone(), seed.directory);
        self.dirty = true;
    }

    fn remove_child(&mut self, parent: &str, id: &str) {
        let delta = self.child_delta.entry(parent.to_owned()).or_default();
        delta.changed.remove(id);
        delta.removed.insert(id.to_owned());
        self.dirty = true;
    }

    fn delta_entries(&self) -> usize {
        self.row_delta.len() + self.child_delta.values().map(ChildDelta::entries).sum::<usize>()
    }

    /// Rebases the immutable base when the sparse delta grew wider than a
    /// fraction of the projection. One full copy here replaces one full copy per
    /// publication in the old design.
    fn compact_if_needed(&mut self) {
        let threshold = (self.len / 8).max(COMPACT_MIN_DELTA);
        if self.delta_entries() <= threshold {
            return;
        }
        // Rebase the shared snapshot from the merged view. The old base stays
        // alive only while a reader's snapshot holds it; the worker itself never
        // keeps a second full copy (#67 W3).
        self.base = Arc::new(ProjectionBase::from_view(
            &self.base,
            &self.row_delta,
            self.len,
        ));
        self.row_delta.clear();
        self.child_delta.clear();
        self.cache.compactions.fetch_add(1, Ordering::Relaxed);
    }

    fn publish_now(&mut self) {
        let generation = self.cache.next_generation();
        let delta = ProjectionDelta {
            rows: self.row_delta.clone(),
            children: self.child_delta.clone(),
        };
        self.cache
            .publish_delta_entries
            .fetch_add(delta.entries() as u64, Ordering::Relaxed);
        // The published projection shares the previous immutable base; only the
        // sparse delta is owned by this generation.
        self.cache.base_shares.fetch_add(1, Ordering::Relaxed);
        #[cfg(test)]
        self.cache
            .record_worker_resident_bytes(self.resident_bytes_estimate());
        let projection = Arc::new(StatusProjection {
            generation,
            base: Arc::clone(&self.base),
            delta: Arc::new(delta),
            len: self.len,
        });
        self.cache.publish(projection);
        self.cache.publishes.fetch_add(1, Ordering::Relaxed);
        self.last_publish = Some(Instant::now());
        self.dirty = false;
        self.compact_if_needed();
    }

    /// Publishes the accumulated changes if the throttle window has elapsed.
    /// Returns whether a projection was published.
    fn publish_if_due(&mut self) -> bool {
        if !self.dirty {
            return false;
        }
        if let Some(last) = self.last_publish {
            if last.elapsed() < self.config.publish_interval {
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
                self.config
                    .publish_interval
                    .saturating_sub(last.elapsed())
                    .max(Duration::from_millis(1))
            });
            return remaining;
        }
        self.idle_wait = (self.idle_wait * 2).min(IDLE_WAIT_MAX);
        self.idle_wait
    }
}

/// Subtree-scoped invalidations whose descendants' visibility can change and
/// therefore cannot be repaired by refreshing the named node alone.
///
/// Seed rows are row-local; the inherited part is the `access_blocked` filter
/// (it walks `nodes.parent` and the overlay remote-parent edges). An overlay
/// relocation/removal or a directory move can hide or reveal the whole subtree
/// below the named node. ACL edits are not handled here: `access_denied` can
/// hide rows outside the named node's local subtree through the overlay
/// remote-parent edge, so `resolve` re-reads the scoped `access_blocked` closure
/// instead.
///
/// A node that is currently absent from the projection may be hidden by that
/// inherited chain. When it becomes visible again (a move out of an overlay-
/// denied position, or an overlay edit that points it back at a visible parent)
/// its descendants were never projected, so the refresh must cover the whole
/// subtree even though no child edge exists yet. Refreshing only the named row
/// would leave those descendant rows missing until the next full rebuild (#8
/// H1). A move of a hidden node is therefore treated like a directory move.
fn subtree_needs_refresh(view: &ProjectionView<'_>, change: &StatusChange) -> bool {
    let Some(node) = change.new_node.as_ref().or(change.old_node.as_ref()) else {
        return false;
    };
    let was_hidden = !view.has_row(node);
    let was_visible_directory = view.has_visible_children(node);
    match change.source.as_str() {
        "namespace_overlay" => was_visible_directory || (was_hidden && change.new_node.is_some()),
        // A move changes the inherited `access_blocked` chain below the moved
        // node. Inserts/deletes and renames do not.
        "nodes" => {
            change.old_node.is_some()
                && change.new_node.is_some()
                && change.old_parent != change.new_parent
                && (was_visible_directory || was_hidden)
        }
        _ => false,
    }
}

#[cfg(test)]
#[path = "status_cache_tests.rs"]
mod tests;
