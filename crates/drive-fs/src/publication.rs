//! Metadata publication ordering and cache invalidation.
//!
//! When the mount answers LOOKUP/GETATTR with a non-zero entry/attr TTL, the
//! kernel may cache the answer. Two things must then be true:
//!
//! 1. A reply that was computed before a state change must never re-arm a
//!    stale cache entry after the invalidation for that change was delivered.
//! 2. Every change that can alter cached metadata must send an invalidation.
//!
//! The publication gate serializes (1): reply paths hold a read permit from
//! before projecting until their reply has been written to `/dev/fuse`;
//! invalidation delivery takes the write permit, so it waits for every in-flight
//! reply. Both directions are full `write(2)` calls on the same FUSE device
//! processed in order by the kernel, so a notification that starts after the
//! last stale reply completes is observed after it.
//!
//! Invalidation delivery is asynchronous (a bounded queue and one worker thread)
//! so mutation paths never block on the gate while they hold the Store lock.
//! If the queue overflows or the kernel rejects a notification the hub fails
//! closed: `ttl()` returns zero and later replies carry zero TTL, so stale
//! entries age out within the previous (bounded) TTL.
//!
//! The hub is inert until a session notifier is installed. Replies written
//! before that point carry zero TTL, so the window between mount creation and
//! notifier installation cannot publish a cacheable answer.
use fuser::Notifier;
use std::{
    ffi::OsStr,
    sync::{
        Arc, Mutex, RwLock, RwLockReadGuard,
        atomic::{AtomicBool, AtomicU64, Ordering},
        mpsc::{self, SyncSender},
    },
    time::Duration,
};

/// Environment override for the metadata TTL. `0` disables caching entirely.
pub const TTL_ENV: &str = "DRIVE_NATIVE_FS_TTL_MS";
/// Approved default: bounded 2 s metadata freshness.
pub const DEFAULT_TTL_MS: u64 = 2_000;
const INVALIDATION_QUEUE: usize = 4096;

enum Invalidate {
    Entry(u64, String),
    Inode(u64),
}

struct Shared {
    gate: RwLock<()>,
    notifier: Mutex<Option<Notifier>>,
    active: AtomicBool,
    failed: AtomicBool,
    ttl_ms: AtomicU64,
    published: AtomicU64,
    invalidations: AtomicU64,
    delivered: AtomicU64,
    failures: AtomicU64,
}

/// Observable hub counters (tests/diagnostics only; no paths or names).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NotifyStats {
    pub published_replies: u64,
    pub invalidations: u64,
    /// Invalidations that passed the publication gate (delivery attempted).
    pub delivered: u64,
    pub failures: u64,
    pub active: bool,
    pub failed: bool,
}

#[derive(Clone)]
pub struct NotifyHub {
    shared: Arc<Shared>,
    queue: SyncSender<Invalidate>,
    // Dropping the sender ends the delivery thread once all clones are gone.
    _thread: Arc<Worker>,
}

struct Worker {
    thread: Mutex<Option<std::thread::JoinHandle<()>>>,
}
impl Drop for Worker {
    fn drop(&mut self) {
        // The channel closes before Drop runs (sender owned by NotifyHub);
        // joining keeps tests deterministic.
        if let Some(thread) = self.thread.lock().unwrap().take() {
            let _ = thread.join();
        }
    }
}

fn fail_closed(shared: &Shared) {
    shared.failed.store(true, Ordering::Release);
    shared.failures.fetch_add(1, Ordering::Relaxed);
}

impl NotifyHub {
    /// Read the configured TTL: `DRIVE_NATIVE_FS_TTL_MS`, default 2 s.
    pub fn from_env() -> Self {
        let ttl_ms = std::env::var(TTL_ENV)
            .ok()
            .and_then(|value| value.trim().parse::<u64>().ok())
            .unwrap_or(DEFAULT_TTL_MS);
        Self::with_ttl_ms(ttl_ms)
    }
    pub fn with_ttl_ms(ttl_ms: u64) -> Self {
        let (queue, receiver) = mpsc::sync_channel::<Invalidate>(INVALIDATION_QUEUE);
        let shared = Arc::new(Shared {
            gate: RwLock::new(()),
            notifier: Mutex::new(None),
            active: AtomicBool::new(false),
            failed: AtomicBool::new(false),
            ttl_ms: AtomicU64::new(ttl_ms),
            published: AtomicU64::new(0),
            invalidations: AtomicU64::new(0),
            delivered: AtomicU64::new(0),
            failures: AtomicU64::new(0),
        });
        let worker_shared = shared.clone();
        let thread = std::thread::spawn(move || {
            while let Ok(message) = receiver.recv() {
                // Wait for every in-flight reply to be written before telling
                // the kernel the old answer is stale.
                let _published = worker_shared
                    .gate
                    .write()
                    .unwrap_or_else(|poison| poison.into_inner());
                // Past the gate: every earlier reply has been written out.
                worker_shared.delivered.fetch_add(1, Ordering::Relaxed);
                if worker_shared.failed.load(Ordering::Acquire) {
                    continue;
                }
                let notifier = worker_shared.notifier.lock().unwrap();
                let Some(notifier) = notifier.as_ref() else {
                    continue;
                };
                let result = match message {
                    Invalidate::Entry(parent, name) => {
                        notifier.inval_entry(parent, OsStr::new(&name))
                    }
                    Invalidate::Inode(ino) => notifier.inval_inode(ino, 0, 0),
                };
                if result.is_err() {
                    // A lost invalidation must never leave a positive-TTL
                    // answer behind: stop publishing cacheable metadata.
                    fail_closed(&worker_shared);
                }
            }
        });
        Self {
            shared,
            queue,
            _thread: Arc::new(Worker {
                thread: Mutex::new(Some(thread)),
            }),
        }
    }
    /// Install the session notifier; before this every reply must stay at TTL 0.
    pub fn install(&self, notifier: Notifier) {
        *self.shared.notifier.lock().unwrap() = Some(notifier);
        self.shared.active.store(true, Ordering::Release);
    }
    /// Stop publishing or delivering: the mount is going away.
    pub fn deactivate(&self) {
        self.shared.active.store(false, Ordering::Release);
        self.shared.notifier.lock().unwrap().take();
    }
    /// Enable a TTL without a real notifier (ordering/fallback tests only).
    #[doc(hidden)]
    pub fn activate_ttl_for_tests(&self, ttl_ms: u64) {
        self.shared.ttl_ms.store(ttl_ms, Ordering::Relaxed);
        self.shared.active.store(true, Ordering::Release);
    }
    /// Model a rejected notification / unsupported kernel (tests only).
    #[doc(hidden)]
    pub fn force_failure_for_tests(&self) {
        fail_closed(&self.shared);
    }
    /// Configured TTL, or zero when caching is not publishable.
    pub fn ttl(&self) -> Duration {
        if !self.shared.active.load(Ordering::Acquire)
            || self.shared.failed.load(Ordering::Acquire)
        {
            return Duration::ZERO;
        }
        Duration::from_millis(self.shared.ttl_ms.load(Ordering::Relaxed))
    }
    pub fn is_caching(&self) -> bool {
        !self.ttl().is_zero()
    }
    /// Hold this across a metadata reply's `write(2)` when caching is active.
    pub fn reply_guard(&self) -> Option<RwLockReadGuard<'_, ()>> {
        if !self.is_caching() {
            return None;
        }
        self.shared.published.fetch_add(1, Ordering::Relaxed);
        Some(
            self.shared
                .gate
                .read()
                .unwrap_or_else(|poison| poison.into_inner()),
        )
    }
    pub fn invalidate_entry(&self, parent_ino: u64, name: &str) {
        self.shared.invalidations.fetch_add(1, Ordering::Relaxed);
        if self
            .queue
            .try_send(Invalidate::Entry(parent_ino, name.to_owned()))
            .is_err()
        {
            fail_closed(&self.shared);
        }
    }
    pub fn invalidate_inode(&self, ino: u64) {
        self.shared.invalidations.fetch_add(1, Ordering::Relaxed);
        if self.queue.try_send(Invalidate::Inode(ino)).is_err() {
            fail_closed(&self.shared);
        }
    }
    pub fn stats(&self) -> NotifyStats {
        NotifyStats {
            published_replies: self.shared.published.load(Ordering::Relaxed),
            invalidations: self.shared.invalidations.load(Ordering::Relaxed),
            delivered: self.shared.delivered.load(Ordering::Relaxed),
            failures: self.shared.failures.load(Ordering::Relaxed),
            active: self.shared.active.load(Ordering::Acquire),
            failed: self.shared.failed.load(Ordering::Acquire),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicUsize;

    #[test]
    fn invalidation_delivery_waits_for_an_in_flight_reply_publication() {
        let hub = NotifyHub::with_ttl_ms(2000);
        hub.activate_ttl_for_tests(2000);
        let guard = hub.reply_guard().expect("caching active");
        hub.invalidate_entry(1, "name");
        // The delivery thread blocks on the gate write permit while the reply
        // publication is open.
        std::thread::sleep(Duration::from_millis(80));
        assert_eq!(hub.stats().delivered, 0);
        drop(guard);
        let deadline = std::time::Instant::now() + Duration::from_secs(2);
        while hub.stats().delivered == 0 && std::time::Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(5));
        }
        assert_eq!(hub.stats().delivered, 1);
        assert!(hub.stats().invalidations >= 1);
        // The unused AtomicUsize import is gone with the old design.
        let _ = AtomicUsize::new(0);
    }

    #[test]
    fn zero_ttl_and_failure_disable_caching() {
        let hub = NotifyHub::with_ttl_ms(0);
        assert!(hub.ttl().is_zero());
        assert!(hub.reply_guard().is_none());
        hub.activate_ttl_for_tests(1500);
        assert_eq!(hub.ttl(), Duration::from_millis(1500));
        hub.deactivate();
        assert!(hub.ttl().is_zero());
        // A rejected/failing invalidation falls back to zero TTL: earlier
        // cacheable answers then age out within the previous bound.
        hub.activate_ttl_for_tests(1500);
        hub.force_failure_for_tests();
        assert!(hub.ttl().is_zero());
        assert!(hub.reply_guard().is_none());
        let stats = hub.stats();
        assert!(stats.failed);
        assert_eq!(stats.failures, 1);
    }
}
