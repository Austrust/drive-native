//! Serialized Store boundaries with nonblocking, coalesced commit invalidation.
//! The notification is a wakeup, never an atom/delta or a publication receipt.
use crate::{
    Store,
    status_changes::{NamespaceWatermark, StatusCursor},
};
use std::{
    collections::HashMap,
    fmt,
    ops::{Deref, DerefMut},
    path::PathBuf,
    sync::{
        Arc, LockResult, Mutex, MutexGuard, PoisonError, TryLockError, TryLockResult,
        RwLock, RwLockReadGuard, RwLockWriteGuard,
        atomic::{AtomicBool, AtomicU64, Ordering},
        mpsc::{self, Receiver, RecvTimeoutError, SyncSender, TryRecvError},
    },
    time::Duration,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CommitState {
    /// Local invalidation generation, not a server or journal cursor.
    pub revision: u64,
    /// False after an uncommitted boundary, journal failure or poisoned access.
    pub verified: bool,
}
fn state(word: u64) -> CommitState {
    CommitState {
        revision: word >> 1,
        verified: word & 1 == 0,
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Boundary {
    cursor: StatusCursor,
    overflows: u64,
}
fn boundary(store: &Store) -> crate::Result<Boundary> {
    let watermark = store.status_change_watermark()?;
    Ok(Boundary {
        cursor: watermark.cursor,
        overflows: watermark.overflows,
    })
}
struct Tracked {
    store: Store,
    boundary: Option<Boundary>,
    /// Last confirmed namespace semantic watermark. The projection token is
    /// derived from this value, never from `total_changes`: LRU, settings,
    /// receipt and TEMP writes must not invalidate a namespace projection while
    /// name/identity/visibility/attribute writes still must.
    namespace: Option<NamespaceWatermark>,
    total_changes: u64,
    unverified: bool,
}

/// There is deliberately no access to the underlying Mutex: every acquired
/// Store guard, including poison recovery, must pass through the same boundary.
pub struct SharedStore {
    // Fields drop in declaration order: invalidate the domain before SQLite
    // or writer ownership teardown. Receivers retain only the atomic state.
    signal: OwnerSignal,
    inner: Mutex<Tracked>,
    receiver: Mutex<Option<Receiver<()>>>,
    // Independent of semantic desktop notifications: includes atime/inode and
    // every SQLite mutation. Lock order is projection -> inner everywhere.
    projection: RwLock<()>,
    projection_word: AtomicU64,
    permission_denials: AtomicBool,
    permission_publication: RwLock<()>,
    // Captured before the Store moves behind the mutex so read-only handles can
    // be opened beside a live writer without acquiring either lock. A whole
    // Store replacement (DerefMut) may move the root, so the publication is
    // refreshed from the writer boundary before readers may trust it.
    root: Mutex<PathBuf>,
    // Shared with the writer's Store: off-lock readers must see the same open
    // handle set the writer protects in `cache_usage`.
    handles: Arc<Mutex<HashMap<String, (String, usize)>>>,
    // Same reason as `handles`: desktop overview serves `queue_health` from a
    // read-only connection, and its scheduler observations describe the writer's
    // domain rather than that throwaway connection.
    namespace_runtime: Arc<Mutex<crate::namespace_schedule::NamespaceRuntime>>,
    // Lock-free mirror of `Store::local_revision()` (`total_changes`), published
    // whenever a writer guard finishes, so off-lock projections keep the field's
    // historical meaning instead of reading 0 from their own connection.
    local_revision: AtomicU64,
}

/// A short metadata publication lease. It never holds SQLite/Store; callers
/// must not acquire Store until this lease has been dropped.
pub struct ProjectionReadGuard<'a> {
    _guard: RwLockReadGuard<'a, ()>,
    state: CommitState,
}
impl ProjectionReadGuard<'_> {
    pub fn state(&self) -> CommitState { self.state }
}
struct OwnerSignal {
    state: Arc<AtomicU64>,
    wake: SyncSender<()>,
}
impl Drop for OwnerSignal {
    fn drop(&mut self) {
        let previous = state(self.state.load(Ordering::Acquire));
        let next = previous.revision.saturating_add(1).min(u64::MAX >> 1);
        self.state.store((next << 1) | 1, Ordering::Release);
        let _ = self.wake.try_send(());
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AlreadySubscribed;
impl fmt::Display for AlreadySubscribed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Store commit notifications already claimed")
    }
}
impl std::error::Error for AlreadySubscribed {}

/// One subscriber only. A full wake channel coalesces; read state() after each
/// receive and capture the journal under Store, never count received messages.
pub struct CommitNotifications {
    receiver: Receiver<()>,
    state: Arc<AtomicU64>,
}
impl CommitNotifications {
    pub fn state(&self) -> CommitState {
        state(self.state.load(Ordering::Acquire))
    }
    pub fn recv_timeout(&self, timeout: Duration) -> Result<(), RecvTimeoutError> {
        self.receiver.recv_timeout(timeout)
    }
    pub fn try_recv(&self) -> Result<(), TryRecvError> {
        self.receiver.try_recv()
    }
}
impl SharedStore {
    pub fn new(store: Store) -> Self {
        let root = store.root().to_owned();
        // Cloned before the Store moves into the mutex: read-only readers need
        // the same handle map to answer cache protection without a Store lock.
        let handles = store.handles.clone();
        let namespace_runtime = store.namespace_runtime.clone();
        let permission_denials = crate::write_permissions::has_denials(&store).unwrap_or(true);
        let boundary = boundary(&store).ok();
        // A projection may only be trusted when both the semantic journal and
        // the namespace watermark were readable at construction time.
        let namespace = store.namespace_watermark().ok();
        let unverified = boundary.is_none() || namespace.is_none();
        let total_changes = store.db.total_changes();
        let (wake, receiver) = mpsc::sync_channel(1);
        if unverified {
            let _ = wake.try_send(());
        }
        Self {
            signal: OwnerSignal {
                state: Arc::new(AtomicU64::new(if unverified { 3 } else { 0 })),
                wake,
            },
            inner: Mutex::new(Tracked {
                store,
                boundary,
                namespace,
                total_changes,
                unverified,
            }),
            receiver: Mutex::new(Some(receiver)),
            projection: RwLock::new(()),
            projection_word: AtomicU64::new(if unverified { 1 } else { 0 }),
            permission_denials: AtomicBool::new(permission_denials),
            permission_publication: RwLock::new(()),
            root: Mutex::new(root),
            handles,
            namespace_runtime,
            local_revision: AtomicU64::new(total_changes),
        }
    }
    /// The state directory this handle owns. Answered from the captured path;
    /// it never acquires the Store or projection lock.
    pub fn root(&self) -> PathBuf {
        self.root
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }
    /// Read-only connection beside the live writer that shares the writer's
    /// handle map and scheduler observations. The SQLite side still observes
    /// only committed WAL, takes no `writer.lock` and cannot write.
    pub fn read_only(&self) -> crate::Result<Store> {
        Store::open_read_only_observing(&self.root, self.namespace_runtime.clone())
    }
    /// Lock-free mirror of [`Store::local_revision`] (SQLite `total_changes`).
    /// Updated when a writer guard publishes its boundary, so desktop overview
    /// can read it without acquiring the Store.
    pub fn local_revision(&self) -> u64 {
        self.local_revision.load(Ordering::Acquire)
    }
    /// Number of open cached/draft handles recorded by the writer (the sum of
    /// all reference counts). Answered from the shared handle map alone; it
    /// never acquires the Store or projection lock.
    pub fn open_handle_count(&self) -> usize {
        self.handles
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .values()
            .map(|(_, references)| *references)
            .sum()
    }
    pub fn change_state(&self) -> CommitState {
        state(self.signal.state.load(Ordering::Acquire))
    }
    /// Observation only. A reply must additionally hold try_projection_read().
    pub fn projection_state(&self) -> CommitState {
        state(self.projection_word.load(Ordering::Acquire))
    }
    /// Conservative fast-path guard, published under the projection write lease.
    pub fn has_permission_denials(&self) -> bool {
        self.permission_denials.load(Ordering::Acquire)
    }
    /// Independent of ordinary Store work: cached reads remain nonblocking
    /// during metadata work, but the first denial waits for bounded replies.
    /// Never block acquiring Store while retaining this guard. Directory
    /// continuations may try_lock for a read-only projection (no ACL mutation).
    pub fn permission_clear_lease(&self) -> Option<RwLockReadGuard<'_, ()>> {
        let guard=self.permission_publication.read().ok()?;
        (!self.has_permission_denials()).then_some(guard)
    }
    pub fn try_projection_read(&self) -> Option<ProjectionReadGuard<'_>> {
        let guard = self.projection.try_read().ok()?;
        Some(ProjectionReadGuard { _guard: guard, state: self.projection_state() })
    }
    pub fn take_notifications(&self) -> Result<CommitNotifications, AlreadySubscribed> {
        let mut receiver = self.receiver.lock().unwrap_or_else(PoisonError::into_inner);
        receiver
            .take()
            .map(|receiver| CommitNotifications {
                receiver,
                state: self.signal.state.clone(),
            })
            .ok_or(AlreadySubscribed)
    }
    fn guard<'a>(&'a self, guard: MutexGuard<'a, Tracked>, projection: RwLockWriteGuard<'a, ()>, poisoned: bool) -> StoreGuard<'a> {
        StoreGuard {
            shared: self,
            guard: Some(guard),
            mutable_access: false,
            poisoned,
            _projection: projection,
        }
    }
    pub fn lock(&self) -> LockResult<StoreGuard<'_>> {
        let (projection, projection_poisoned) = match self.projection.write() {
            Ok(guard) => (guard, false),
            Err(error) => (error.into_inner(), true),
        };
        match self.inner.lock() {
            Ok(guard) if !projection_poisoned => Ok(self.guard(guard, projection, false)),
            Ok(guard) => Err(PoisonError::new(self.guard(guard, projection, true))),
            Err(error) => Err(PoisonError::new(self.guard(error.into_inner(), projection, true))),
        }
    }
    pub fn try_lock(&self) -> TryLockResult<StoreGuard<'_>> {
        let (projection, projection_poisoned) = match self.projection.try_write() {
            Ok(guard) => (guard, false),
            Err(TryLockError::WouldBlock) => return Err(TryLockError::WouldBlock),
            Err(TryLockError::Poisoned(error)) => (error.into_inner(), true),
        };
        match self.inner.try_lock() {
            Ok(guard) if !projection_poisoned => Ok(self.guard(guard, projection, false)),
            Ok(guard) => Err(TryLockError::Poisoned(PoisonError::new(self.guard(guard, projection, true)))),
            Err(TryLockError::WouldBlock) => Err(TryLockError::WouldBlock),
            Err(TryLockError::Poisoned(error)) => Err(TryLockError::Poisoned(PoisonError::new(
                self.guard(error.into_inner(), projection, true),
            ))),
        }
    }
    pub fn is_poisoned(&self) -> bool {
        self.inner.is_poisoned()
    }
    pub fn into_inner(self) -> LockResult<Store> {
        // Unique ownership removes the serialization domain. A retained
        // notification receiver must not keep regarding its old view as ready.
        drop(self.signal);
        let result = self
            .inner
            .into_inner()
            .map(|tracked| tracked.store)
            .map_err(|error| PoisonError::new(error.into_inner().store));
        result
    }
}
pub struct StoreGuard<'a> {
    shared: &'a SharedStore,
    guard: Option<MutexGuard<'a, Tracked>>,
    mutable_access: bool,
    poisoned: bool,
    _projection: RwLockWriteGuard<'a, ()>,
}
impl fmt::Debug for StoreGuard<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StoreGuard").finish_non_exhaustive()
    }
}
impl Deref for StoreGuard<'_> {
    type Target = Store;
    fn deref(&self) -> &Store {
        &self.guard.as_ref().expect("live Store guard").store
    }
}
impl DerefMut for StoreGuard<'_> {
    fn deref_mut(&mut self) -> &mut Store {
        // Also catches replacement of the entire Store through DerefMut, even
        // when the new connection happens to have the same total_changes.
        self.mutable_access = true;
        &mut self.guard.as_mut().expect("live Store guard").store
    }
}
impl StoreGuard<'_> {
    /// Publish the completed boundary and capture its exact projection token
    /// before releasing the write lease. This binds a rebuilt index without a
    /// race against a second Store writer between drop and token observation.
    pub fn finish_projection(mut self) -> CommitState {
        self.finish();
        self.shared.projection_state()
    }
    fn finish(&mut self) {
        let Some(mut guard) = self.guard.take() else {
            return;
        };
        let hint = guard.store.db.total_changes();
        let poisoned = self.poisoned || std::thread::panicking();
        let before = self.shared.change_state();
        let projection_before = self.shared.projection_state();
        // A Store replaced through DerefMut may point at a different directory:
        // publish the new root and invalidate every projection derived from the
        // old namespace before releasing the write lease.
        let root_changed = {
            let current = guard.store.root();
            let mut published = self.shared.root.lock().unwrap_or_else(PoisonError::into_inner);
            if published.as_path() != current {
                *published = current.to_owned();
                true
            } else {
                false
            }
        };
        let mut projection_changed = poisoned || root_changed;
        let mut changed = root_changed;
        if hint != guard.total_changes
            || self.mutable_access
            || guard.unverified
            || poisoned
            || !guard.store.db.is_autocommit()
        {
            // The projection token is the namespace semantic watermark, not
            // total_changes: LRU, settings, receipt and TEMP writes must not
            // force a projection rebuild. An open transaction or an unreadable
            // watermark fails closed instead of publishing a guess.
            let mut verified = true;
            if guard.store.db.is_autocommit() {
                match guard.store.namespace_watermark() {
                    Ok(next) => {
                        projection_changed |= guard.namespace.as_ref() != Some(&next);
                        guard.namespace = Some(next);
                    }
                    Err(_) => {
                        projection_changed = true;
                        verified = false;
                    }
                }
            } else {
                projection_changed = true;
                verified = false;
            }
            match boundary(&guard.store) {
                Ok(next) => {
                    changed = guard.boundary.as_ref() != Some(&next);
                    guard.boundary = Some(next);
                }
                Err(_) => {
                    // Never expose an internal/uncommitted sequence as a commit.
                    // Keep the last known boundary for a subsequent rollback or
                    // repair, and invalidate before releasing Store ownership.
                    changed = hint != guard.total_changes;
                    verified = false;
                }
            }
            guard.unverified = !verified || poisoned;
        }
        changed |= root_changed;
        let sql_changed = hint != guard.total_changes || self.mutable_access || poisoned;
        guard.total_changes = hint;
        // Publish the same value `Store::local_revision` would answer under this
        // guard, before any off-lock reader can ask for it.
        self.shared
            .local_revision
            .store(hint, Ordering::Release);
        let verified = !guard.unverified;
        if projection_changed || projection_before.verified != verified {
            let next = projection_before.revision.checked_add(1).filter(|n| *n < (1 << 63));
            self.shared.projection_word.store(
                next.map_or(u64::MAX, |n| (n << 1) | u64::from(!verified)), Ordering::Release,
            );
        }
        if sql_changed || projection_before.verified != verified {
            // `directory_permissions` is deliberately outside the namespace
            // journal, so denial publication follows the SQLite write hint
            // rather than the projection watermark: the first denial must wait
            // for bounded cached replies before it becomes visible.
            let denied=crate::write_permissions::has_denials(&guard.store).unwrap_or(true);
            if denied != self.shared.has_permission_denials() {
                let _publication=self.shared.permission_publication.write().unwrap_or_else(PoisonError::into_inner);
                self.shared.permission_denials.store(denied,Ordering::Release);
            }
        }
        changed |= before.verified != verified;
        if changed {
            // Store serialization makes this the sole publisher. One atomic
            // word keeps validity and revision coherent for lock-free readers.
            // Exhaustion fails closed instead of wrapping to an old generation.
            let next = before.revision.checked_add(1).filter(|n| *n < (1 << 63));
            let word = next.map_or(u64::MAX, |n| (n << 1) | u64::from(!verified));
            self.shared.signal.state.store(word, Ordering::Release);
        }
        drop(guard);
        if changed {
            // No callback, external lock or blocking send after the boundary.
            let _ = self.shared.signal.wake.try_send(());
        }
    }
}
impl Drop for StoreGuard<'_> {
    fn drop(&mut self) { self.finish(); }
}

#[cfg(test)]
#[path = "shared_store_tests.rs"]
mod tests;
