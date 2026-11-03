//! Metadata publication ordering and cache invalidation.
//!
//! When the mount answers LOOKUP/GETATTR with a non-zero entry/attr TTL, the
//! kernel may cache the answer. Caching then relies on two mechanisms:
//!
//! 1. **Publication gate** (this module): reply paths hold a read permit from
//!    before projecting until their reply `write(2)` to `/dev/fuse` completes;
//!    invalidation delivery takes the write permit. The gate is
//!    writer-preferring so a scan full of readers cannot starve delivery. This
//!    removes the user-space class of reordering where an invalidation is
//!    written before a reply that was computed from older state.
//!
//! 2. **Bounded TTL + authoritative data path**: the gate cannot prove that the
//!    kernel finished installing a reply's cache lifetime before the
//!    invalidation for a concurrent change is processed. `write(2)` returning
//!    only orders delivery to the kernel; the waiting FUSE request can still
//!    wake afterwards and install its (older) TTL. That residual window is the
//!    same class as the rejected `fs-local-experience-8` experiment and is
//!    **not** claimed to be eliminated; the gate is an engineering measure that
//!    compresses the user-space reorder window, not a safety proof.
//!
//!    Kernel-side protection differs per cache kind:
//!    - attribute TTL is guarded by the kernel's `attr_version`: an
//!      `FUSE_NOTIFY_INVAL_INODE` bumps it, and older GETATTR replies are
//!      discarded, so attr changes must always send `INVAL_INODE`;
//!    - positive entry TTL has no equivalent guard on the kernels this product
//!      targets (Mint 22.x ships 6.8/6.11/6.14; `FUSE_NOTIFY_INC_EPOCH` exists
//!      only from 6.16). The contract for entries is therefore the bounded one
//!      below, not kernel-proven immediate consistency.
//!
//!    Worst case: a cached dentry/attribute stays visible for at most the
//!    configured TTL `t` after a change (plus an in-flight reply window).
//!    Content access (`open`/`read`/`write`) and permission decisions still
//!    consult the authoritative Store on every operation, so stale metadata
//!    cannot make a deleted, renamed or revoked object's data readable. The
//!    cache is bounded (`DRIVE_NATIVE_FS_TTL_MS`, default 1000 ms, `0`
//!    disables), configurable, and fails closed on invalidation failure
//!    (strict notifier: a swallowed ENOENT is treated as failure).
//!
//! Invalidation delivery is asynchronous (a bounded queue and one worker) so
//! mutation paths never block on the gate while holding the Store lock. If the
//! queue overflows or the kernel rejects a notification the hub fails closed:
//! `ttl()` returns zero and later replies carry zero TTL, so stale entries age
//! out within the previous (bounded) TTL. `NotifyStats` and the
//! `TtlPublishedReplies`/`TtlDelivered`/`TtlInvalidations`/`TtlFailures`
//! diagnostics expose backlog and fallback behaviour.
//!
//! The hub is inert until a session notifier is installed. Replies written
//! before that point carry zero TTL, so the window between mount creation and
//! notifier installation cannot publish a cacheable answer.
use fuser::Notifier;
use std::{
    ffi::OsStr,
    sync::{
        Arc, Condvar, Mutex, MutexGuard, PoisonError,
        atomic::{AtomicBool, AtomicU64, Ordering},
        mpsc::{self, SyncSender},
    },
    time::Duration,
};

/// Environment override for the metadata TTL. `0` disables caching entirely.
pub const TTL_ENV: &str = "DRIVE_NATIVE_FS_TTL_MS";
/// Approved default: bounded ~1 s metadata freshness.
pub const DEFAULT_TTL_MS: u64 = 1_000;
const INVALIDATION_QUEUE: usize = 65_536;

enum Invalidate {
    Entry(u64, String),
    Inode(u64),
}

/// Writer-preferring gate: readers never starve an invalidation that is waiting
/// to be delivered.
struct Gate {
    state: Mutex<GateState>,
    readers_ok: Condvar,
    writers_ok: Condvar,
}
#[derive(Default)]
struct GateState {
    readers: usize,
    writer: bool,
    waiting_writers: usize,
}
pub struct GateRead<'a>(&'a Gate);
struct GateWrite<'a>(&'a Gate);

impl Gate {
    fn new() -> Self {
        Self {
            state: Mutex::new(GateState::default()),
            readers_ok: Condvar::new(),
            writers_ok: Condvar::new(),
        }
    }
    fn read(&self) -> GateRead<'_> {
        let mut state = self.lock();
        while state.writer || state.waiting_writers > 0 {
            state = self
                .readers_ok
                .wait(state)
                .unwrap_or_else(PoisonError::into_inner);
        }
        state.readers += 1;
        GateRead(self)
    }
    fn write(&self) -> GateWrite<'_> {
        let mut state = self.lock();
        state.waiting_writers += 1;
        while state.writer || state.readers > 0 {
            state = self
                .writers_ok
                .wait(state)
                .unwrap_or_else(PoisonError::into_inner);
        }
        state.waiting_writers -= 1;
        state.writer = true;
        GateWrite(self)
    }
    fn lock(&self) -> MutexGuard<'_, GateState> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }
    fn release_read(&self) {
        let mut state = self.lock();
        state.readers -= 1;
        if state.readers == 0 && state.waiting_writers > 0 {
            self.writers_ok.notify_one();
        } else if state.waiting_writers == 0 {
            self.readers_ok.notify_all();
        }
    }
    fn release_write(&self) {
        let mut state = self.lock();
        state.writer = false;
        if state.waiting_writers > 0 {
            self.writers_ok.notify_one();
        } else {
            self.readers_ok.notify_all();
        }
    }
}
impl Drop for GateRead<'_> {
    fn drop(&mut self) {
        self.0.release_read();
    }
}
impl Drop for GateWrite<'_> {
    fn drop(&mut self) {
        self.0.release_write();
    }
}

struct Shared {
    gate: Gate,
    notifier: Mutex<Option<Notifier>>,
    /// Test-only deterministic late-reply injection (feature `test-seams`).
    #[cfg(any(test, feature = "test-seams"))]
    reply_delay: Mutex<Option<Duration>>,
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
        if let Some(thread) = self.thread.lock().unwrap().take() {
            let _ = thread.join();
        }
    }
}

fn fail_closed(shared: &Shared) {
    if !shared.failed.swap(true, Ordering::AcqRel) {
        shared.failures.fetch_add(1, Ordering::Relaxed);
        crate::diagnostics::count(crate::diagnostics::Stage::TtlFailures);
    }
}

impl NotifyHub {
    /// Read the configured TTL: `DRIVE_NATIVE_FS_TTL_MS`, default 1 s.
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
            gate: Gate::new(),
            notifier: Mutex::new(None),
            #[cfg(any(test, feature = "test-seams"))]
            reply_delay: Mutex::new(None),
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
                // Wait for every in-flight reply to finish writing. This orders
                // user-space publication; the kernel-side late-install window is
                // covered by the bounded TTL + authoritative data path (see the
                // module documentation).
                let _published = worker_shared.gate.write();
                worker_shared.delivered.fetch_add(1, Ordering::Relaxed);
                crate::diagnostics::count(crate::diagnostics::Stage::TtlDelivered);
                if worker_shared.failed.load(Ordering::Acquire) {
                    continue;
                }
                let notifier = worker_shared.notifier.lock().unwrap();
                let Some(notifier) = notifier.as_ref() else {
                    continue;
                };
                // Strict variants: the lenient fuser API swallows ENOENT,
                // which would hide a dropped invalidation. Any error fails the
                // mount closed to zero TTL.
                let result = match message {
                    Invalidate::Entry(parent, name) => {
                        notifier.inval_entry_strict(parent, OsStr::new(&name))
                    }
                    Invalidate::Inode(ino) => notifier.inval_inode_strict(ino, 0, 0),
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
    /// Test-only: hold a metadata reply for `delay` after its projection and
    /// publication permit, before the reply is written to the kernel. This makes
    /// the late-reply ordering race deterministic. Compiled only under
    /// `cfg(test)` / the `test-seams` feature; never in release builds.
    #[cfg(any(test, feature = "test-seams"))]
    pub fn reply_delay_for_tests(&self, delay: Option<Duration>) {
        *self.shared.reply_delay.lock().unwrap() = delay;
    }
    #[cfg(any(test, feature = "test-seams"))]
    pub(crate) fn apply_reply_delay(&self) {
        let delay = *self.shared.reply_delay.lock().unwrap();
        if let Some(delay) = delay {
            std::thread::sleep(delay);
        }
    }
    #[cfg(not(any(test, feature = "test-seams")))]
    #[inline]
    pub(crate) fn apply_reply_delay(&self) {}

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
    pub fn reply_guard(&self) -> Option<GateRead<'_>> {
        if !self.is_caching() {
            return None;
        }
        self.shared.published.fetch_add(1, Ordering::Relaxed);
        crate::diagnostics::count(crate::diagnostics::Stage::TtlPublishedReplies);
        Some(self.shared.gate.read())
    }
    pub fn invalidate_entry(&self, parent_ino: u64, name: &str) {
        self.shared.invalidations.fetch_add(1, Ordering::Relaxed);
        crate::diagnostics::count(crate::diagnostics::Stage::TtlInvalidations);
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
        crate::diagnostics::count(crate::diagnostics::Stage::TtlInvalidations);
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

    #[test]
    fn invalidation_delivery_waits_for_an_in_flight_reply_publication() {
        let hub = NotifyHub::with_ttl_ms(2000);
        hub.activate_ttl_for_tests(2000);
        let guard = hub.reply_guard().expect("caching active");
        hub.invalidate_entry(1, "name");
        // Delivery blocks on the gate while the reply publication is open.
        std::thread::sleep(Duration::from_millis(80));
        assert_eq!(hub.stats().delivered, 0);
        drop(guard);
        let deadline = std::time::Instant::now() + Duration::from_secs(2);
        while hub.stats().delivered == 0 && std::time::Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(5));
        }
        assert_eq!(hub.stats().delivered, 1);
        assert!(hub.stats().invalidations >= 1);
    }

    #[test]
    fn a_waiting_invalidation_blocks_new_readers_instead_of_starving() {
        let hub = NotifyHub::with_ttl_ms(2000);
        hub.activate_ttl_for_tests(2000);
        // Hold one reader so the writer starts waiting.
        let first = hub.reply_guard().expect("caching active");
        hub.invalidate_entry(1, "name");
        std::thread::sleep(Duration::from_millis(30));
        // With writer preference the next reader cannot overtake the writer.
        let start = std::time::Instant::now();
        let second = std::thread::spawn({
            let hub = hub.clone();
            move || {
                let _guard = hub.reply_guard();
                start.elapsed()
            }
        });
        std::thread::sleep(Duration::from_millis(40));
        assert!(!second.is_finished(), "new reader overtook waiting writer");
        drop(first);
        let waited = second.join().unwrap();
        // The key assertion is the ordering above; the wait itself is
        // scheduled by the OS, so only require a non-trivial block.
        assert!(waited >= Duration::from_millis(20), "waited {waited:?}");
        let deadline = std::time::Instant::now() + Duration::from_secs(2);
        while hub.stats().delivered == 0 && std::time::Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(5));
        }
        assert_eq!(hub.stats().delivered, 1);
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
