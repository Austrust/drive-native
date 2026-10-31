//! Serialized Store boundaries with nonblocking, coalesced commit invalidation.
//! The notification is a wakeup, never an atom/delta or a publication receipt.
use crate::{Store, status_changes::StatusCursor};
use std::{
    fmt,
    ops::{Deref, DerefMut},
    sync::{
        Arc, LockResult, Mutex, MutexGuard, PoisonError, TryLockError, TryLockResult,
        atomic::{AtomicU64, Ordering},
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
        let boundary = boundary(&store).ok();
        let unverified = boundary.is_none();
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
                total_changes,
                unverified,
            }),
            receiver: Mutex::new(Some(receiver)),
        }
    }
    pub fn change_state(&self) -> CommitState {
        state(self.signal.state.load(Ordering::Acquire))
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
    fn guard<'a>(&'a self, guard: MutexGuard<'a, Tracked>, poisoned: bool) -> StoreGuard<'a> {
        StoreGuard {
            shared: self,
            guard: Some(guard),
            mutable_access: false,
            poisoned,
        }
    }
    pub fn lock(&self) -> LockResult<StoreGuard<'_>> {
        match self.inner.lock() {
            Ok(guard) => Ok(self.guard(guard, false)),
            Err(error) => Err(PoisonError::new(self.guard(error.into_inner(), true))),
        }
    }
    pub fn try_lock(&self) -> TryLockResult<StoreGuard<'_>> {
        match self.inner.try_lock() {
            Ok(guard) => Ok(self.guard(guard, false)),
            Err(TryLockError::WouldBlock) => Err(TryLockError::WouldBlock),
            Err(TryLockError::Poisoned(error)) => Err(TryLockError::Poisoned(PoisonError::new(
                self.guard(error.into_inner(), true),
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
impl Drop for StoreGuard<'_> {
    fn drop(&mut self) {
        let Some(mut guard) = self.guard.take() else {
            return;
        };
        let hint = guard.store.db.total_changes();
        let poisoned = self.poisoned || std::thread::panicking();
        let before = self.shared.change_state();
        let mut changed = false;
        if hint != guard.total_changes
            || self.mutable_access
            || guard.unverified
            || poisoned
            || !guard.store.db.is_autocommit()
        {
            match boundary(&guard.store) {
                Ok(next) => {
                    changed = guard.boundary.as_ref() != Some(&next);
                    guard.boundary = Some(next);
                    guard.unverified = poisoned;
                }
                Err(_) => {
                    // Never expose an internal/uncommitted sequence as a commit.
                    // Keep the last known boundary for a subsequent rollback or
                    // repair, and invalidate before releasing Store ownership.
                    changed = hint != guard.total_changes;
                    guard.unverified = true;
                }
            }
        }
        guard.total_changes = hint;
        let verified = !guard.unverified;
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

#[cfg(test)]
#[path = "shared_store_tests.rs"]
mod tests;
