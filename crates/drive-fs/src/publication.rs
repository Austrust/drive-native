//! Bounded attribute caching and asynchronous kernel invalidation.
//!
//! Positive entry TTL is always zero, including LOOKUP and READDIRPLUS. A
//! bounded name cache can otherwise redirect a new open to an object moved to
//! another path; an inode-only OPEN cannot recheck the caller's original name.
//! Optional kernel INC_EPOCH and directory quieting do not prove all late
//! dentry revalidation/PLUS replies safe on supported Linux kernels.
//!
//! Only clean authoritative attributes use the configured TTL. Attribute
//! invalidation relies on the kernel attr_version protection; preview, dirty
//! bodies and handle-adjusted attributes remain uncached. Cached attributes may
//! remain visible until notification delivery or TTL expiry. Existing open
//! handles retain their POSIX identity/content semantics; reads are not a new
//! path-based authorization check.
//!
//! Delivery runs without a lock that a FUSE reply needs: a notification may
//! wait for a kernel operation which is itself waiting for that reply. Queue
//! overflow or a hard notification failure stops new attribute leases; it does
//! not retroactively revoke outstanding leases. Both the runtime feeder and
//! this hub must propagate loss. The hub is inert until a notifier is installed.
use fuser::Notifier;
use std::{
    collections::HashMap,
    ffi::OsStr,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU8, AtomicU64, Ordering},
        mpsc::{self, SyncSender},
    },
    time::{Duration, Instant},
};

/// Attribute TTL override; entry TTL remains zero for every value. `0` disables both.
pub const TTL_ENV: &str = "DRIVE_NATIVE_FS_TTL_MS";
/// Default bounded attribute freshness; never a positive-name lease.
pub const DEFAULT_TTL_MS: u64 = 1_000;
/// Names always revalidate, independently of attribute caching configuration.
pub const ENTRY_TTL: Duration = Duration::ZERO;
const INVALIDATION_QUEUE: usize = 65_536;

enum Invalidate {
    Entry(u64, String),
    Inode(u64),
}

/// Non-blocking observation token for a published metadata reply.
pub struct ReplyPermit;

struct Shared {
    notifier: Mutex<Option<Notifier>>,
    /// Test-only deterministic late-reply injection (feature `test-seams`).
    #[cfg(any(test, feature = "test-seams"))]
    reply_delay: Mutex<Option<Duration>>,
    active: AtomicBool,
    /// Mirrors `active && !failed && ttl_ms > 0`: one relaxed load on the hot
    /// reply path so a disabled cache costs no gate or clock work.
    caching: AtomicBool,
    failed: AtomicBool,
    ttl_ms: AtomicU64,
    published: AtomicU64,
    invalidations: AtomicU64,
    delivered: AtomicU64,
    failures: AtomicU64,
    /// Directories touched by a change recently; READDIRPLUS must not publish a
    /// positive entry TTL for them until the view is stable again.
    touched: Mutex<HashMap<u64, Instant>>,
    /// 0 unknown, 1 `INC_EPOCH` supported, 2 not supported by this kernel.
    epoch: AtomicU8,
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

/// Best-effort `FUSE_NOTIFY_INC_EPOCH` (kernel >= 6.16). Unsupported kernels
/// answer EINVAL/ENOSYS, which is remembered and treated as optional; any other
/// failure falls back to zero TTL like a lost invalidation.
fn bump_epoch(shared: &Shared, notifier: &Notifier) {
    match shared.epoch.load(Ordering::Relaxed) {
        2 => return,
        1 => {}
        _ => {}
    }
    match notifier.inc_epoch() {
        Ok(()) => {
            shared.epoch.store(1, Ordering::Relaxed);
        }
        Err(error) => {
            let unsupported = matches!(
                error.raw_os_error(),
                Some(libc::EINVAL | libc::ENOSYS | libc::EOPNOTSUPP | libc::ENOTTY)
            );
            if unsupported {
                shared.epoch.store(2, Ordering::Relaxed);
            } else {
                fail_closed(shared);
            }
        }
    }
}

fn fail_closed(shared: &Shared) {
    if !shared.failed.swap(true, Ordering::AcqRel) {
        shared.caching.store(false, Ordering::Release);
        shared.failures.fetch_add(1, Ordering::Relaxed);
        crate::diagnostics::count(crate::diagnostics::Stage::TtlFailures);
    }
}

impl NotifyHub {
    /// Read attribute TTL: `DRIVE_NATIVE_FS_TTL_MS`, default 1 s; entry TTL is zero.
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
            caching: AtomicBool::new(false),
            failed: AtomicBool::new(false),
            ttl_ms: AtomicU64::new(ttl_ms),
            published: AtomicU64::new(0),
            invalidations: AtomicU64::new(0),
            delivered: AtomicU64::new(0),
            failures: AtomicU64::new(0),
            touched: Mutex::new(HashMap::new()),
            epoch: AtomicU8::new(0),
        });
        let worker_shared = shared.clone();
        let thread = std::thread::spawn(move || {
            while let Ok(message) = receiver.recv() {
                // Deliberately no user-space lock here: a kernel notification
                // can wait for an in-flight FUSE reply, so blocking reply paths
                // on delivery would deadlock (ABBA). The cache contract is
                // bounded staleness + the authoritative data path, not proven
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
                    // The kernel answers ENOENT when nothing for that name or
                    // inode is cached (or a racing install is still in flight).
                    // Under the bounded-staleness contract that is benign: there
                    // is no installed cache entry to revoke. Any other write
                    // failure means an invalidation may have been lost, so stop
                    // publishing cacheable metadata.
                    if error.raw_os_error() != Some(libc::ENOENT) {
                        fail_closed(&worker_shared);
                    }
                } else if !worker_shared.ttl_ms.load(Ordering::Relaxed).eq(&0) {
                    bump_epoch(&worker_shared, notifier);
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
        self.refresh_caching();
    }
    /// Stop publishing or delivering: the mount is going away.
    pub fn deactivate(&self) {
        self.shared.active.store(false, Ordering::Release);
        self.shared.caching.store(false, Ordering::Release);
        self.shared.notifier.lock().unwrap().take();
    }
    fn refresh_caching(&self) {
        let caching = self.shared.active.load(Ordering::Acquire)
            && !self.shared.failed.load(Ordering::Acquire)
            && self.shared.ttl_ms.load(Ordering::Relaxed) > 0;
        self.shared.caching.store(caching, Ordering::Release);
    }
    /// Enable a TTL without a real notifier (ordering/fallback tests only).
    #[doc(hidden)]
    pub fn activate_ttl_for_tests(&self, ttl_ms: u64) {
        self.shared.ttl_ms.store(ttl_ms, Ordering::Relaxed);
        self.shared.active.store(true, Ordering::Release);
        self.refresh_caching();
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

    /// Configured TTL regardless of activation: used to decide at FUSE init
    /// whether READDIRPLUS should be requested.
    pub fn configured_ttl_ms(&self) -> u64 {
        self.shared.ttl_ms.load(Ordering::Relaxed)
    }
    /// A directory touched by a change recently must not publish a positive
    /// entry TTL (compresses the readdirplus late-install window).
    pub fn directory_unstable(&self, ino: u64) -> bool {
        if !self.is_caching() {
            return false;
        }
        let ttl = self.ttl();
        let touched = self.shared.touched.lock().unwrap();
        touched
            .get(&ino)
            .is_some_and(|last| last.elapsed() < ttl)
    }
    fn mark_directory_touched(&self, ino: u64) {
        if self.configured_ttl_ms() == 0 {
            return;
        }
        let mut touched = self.shared.touched.lock().unwrap();
        if touched.len() >= 8192 {
            touched.clear();
        }
        touched.insert(ino, Instant::now());
    }
    /// Whether the kernel accepted `FUSE_NOTIFY_INC_EPOCH` (dentry epoch guard).
    pub fn epoch_supported(&self) -> bool {
        self.shared.epoch.load(Ordering::Relaxed) == 1
    }

    /// Configured TTL, or zero when caching is not publishable.
    #[inline]
    pub fn ttl(&self) -> Duration {
        if !self.shared.active.load(Ordering::Acquire)
            || self.shared.failed.load(Ordering::Acquire)
        {
            return Duration::ZERO;
        }
        Duration::from_millis(self.shared.ttl_ms.load(Ordering::Relaxed))
    }
    #[inline]
    pub fn is_caching(&self) -> bool {
        // One relaxed load on the hot path; the slow path only runs when the
        // cache is configured and installed.
        if !self.shared.caching.load(Ordering::Relaxed) {
            return false;
        }
        !self.ttl().is_zero()
    }
    /// A lost notification disables index publication as well as attribute TTL.
    #[inline]
    pub fn delivery_failed(&self) -> bool {
        self.shared.failed.load(Ordering::Acquire)
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
    /// An upstream queue lost a committed invalidation. Stop all new attribute
    /// leases; existing kernel leases still have their previously granted bound.
    pub fn invalidate_delivery_failed(&self) {
        fail_closed(&self.shared);
    }
    pub fn invalidate_entry(&self, parent_ino: u64, name: &str) {
        self.mark_directory_touched(parent_ino);
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
        self.mark_directory_touched(ino);
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
        let deadline = std::time::Instant::now() + Duration::from_secs(2);
        while hub.stats().delivered == 0 && std::time::Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(5));
        }
        assert!(hub.stats().delivered >= 1);
    }

    #[test]
    fn a_touched_directory_suppresses_entry_ttls_until_it_is_stable() {
        let hub = NotifyHub::with_ttl_ms(1000);
        hub.activate_ttl_for_tests(60);
        assert!(!hub.directory_unstable(7));
        hub.invalidate_entry(7, "child");
        assert!(hub.directory_unstable(7));
        assert!(!hub.directory_unstable(8));
        std::thread::sleep(Duration::from_millis(90));
        assert!(!hub.directory_unstable(7));
        // A configured TTL of zero tracks nothing.
        let disabled = NotifyHub::with_ttl_ms(0);
        disabled.activate_ttl_for_tests(0);
        disabled.invalidate_entry(7, "child");
        assert!(!disabled.directory_unstable(7));
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
