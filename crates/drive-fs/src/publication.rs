//! Bounded metadata caching and kernel invalidation.
//!
//! Authoritative metadata may be cached by the kernel for a bounded TTL. The
//! correctness contract is **bounded staleness**: a cached name/attribute can be
//! up to `t` (+ an in-flight reply window) stale after a change. Data access
//! (`open`/`read`/`write`) and permission decisions still consult the
//! authoritative Store on every operation, so stale metadata cannot make a
//! deleted, renamed or revoked object's data readable.
//!
//! Invalidation delivery is asynchronous and **non-blocking for FUSE reply
//! paths**: a dedicated thread writes strict `INVAL_ENTRY`/`INVAL_INODE`
//! notifications directly. There is intentionally no user-space publication
//! lock: user-space write order does not prove kernel-side cache-install order
//! (the kernel installs a reply's TTL after waking the waiting FUSE task), and a
//! blocking lock lets a kernel notification (which may wait for an in-flight
//! reply) deadlock against the reply path. Reply paths only bump counters.
//!
//! Kernel-side protection differs per cache kind:
//! - attribute TTL is guarded by the kernel's `attr_version`: an
//!   `INVAL_INODE` bumps it and older GETATTR replies are discarded, so attr
//!   changes must always send `INVAL_INODE`;
//! - positive entry TTL has no equivalent guard on the kernels this product
//!   targets (Mint 22.x ships 6.8/6.11/6.14; `FUSE_NOTIFY_INC_EPOCH` exists only
//!   from 6.16), hence the bounded contract above and the directory quieting
//!   period for READDIRPLUS.
//!
//! A full invalidation queue or a failed notification (other than the benign
//! "nothing cached" ENOENT) fails the mount closed:
//! `ttl()` becomes zero and later replies carry zero TTL, so earlier cached
//! answers age out within the previous bound. Strict notifier variants are used
//! because the lenient fuser API swallows ENOENT. `NotifyStats` and the
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
        Arc, Mutex,
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

struct Shared {
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

/// Non-blocking observation token for a published metadata reply.
pub struct ReplyPermit;

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
                // Deliberately no user-space lock here: a kernel notification
                // can wait for an in-flight FUSE reply, so blocking reply paths
                // on delivery would deadlock (ABBA). The cache contract is
                // bounded staleness + authoritative data path, not proven
                // ordering (see the module documentation).
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
                if let Err(error) = result {
                    // The kernel answers ENOENT when nothing for that name/inode
                    // is cached (or a racing install is still in flight). Under
                    // the bounded-staleness contract that is benign: there is no
                    // installed cache entry to revoke. Any other write failure
                    // means an invalidation may have been lost, so stop
                    // publishing cacheable metadata (`TTL` becomes zero and
                    // older answers age out within the previous bound).
                    let benign = error.raw_os_error() == Some(libc::ENOENT);
                    if !benign {
                        fail_closed(&worker_shared);
                    }
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
    /// Observation only, never a lock: metadata reply paths must stay
    /// non-blocking. Holding this does not order replies against invalidations;
    /// the bounded TTL and the authoritative data path are the contract.
    #[inline]
    pub fn reply_guard(&self) -> Option<ReplyPermit> {
        if !self.is_caching() {
            return None;
        }
        self.shared.published.fetch_add(1, Ordering::Relaxed);
        crate::diagnostics::count(crate::diagnostics::Stage::TtlPublishedReplies);
        Some(ReplyPermit)
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
    fn invalidation_delivery_never_blocks_metadata_replies() {
        let hub = NotifyHub::with_ttl_ms(2000);
        hub.activate_ttl_for_tests(2000);
        let permit = hub.reply_guard().expect("caching active");
        // Enqueue invalidations without any reply-side lock: both the enqueue
        // and a later reply permit stay non-blocking while delivery may still
        // be writing a kernel notification.
        for index in 0..1000 {
            hub.invalidate_entry(1, &format!("name-{index}"));
        }
        let started = std::time::Instant::now();
        let _second = hub.reply_guard().expect("caching active");
        assert!(
            started.elapsed() < Duration::from_millis(100),
            "reply path blocked on invalidation delivery"
        );
        drop(permit);
        // The delivery thread makes progress on its own.
        let deadline = std::time::Instant::now() + Duration::from_secs(2);
        while hub.stats().delivered == 0 && std::time::Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(5));
        }
        assert!(hub.stats().delivered >= 1);
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
