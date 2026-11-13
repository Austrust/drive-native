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
//! overflow or a rejected notification suspends new attribute leases for one
//! old-lease window instead of closing the mount for the rest of the session.
//! The window is quiet (no in-flight cacheable reply, then the previous TTL
//! plus a short grace) so every pre-degradation answer has aged out; afterwards
//! the hub resumes publishing on its own, without a restart or remount.
//! A wholesale generation invalidation (one message for a whole snapshot
//! commit) uses the same suspended window. Both the runtime feeder and this hub
//! must propagate loss. The hub is inert until a notifier is installed.
use fuser::Notifier;
use std::{
    collections::HashMap,
    ffi::OsStr,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU8, AtomicU64, AtomicUsize, Ordering},
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
/// Slack added to the previous lease TTL before publishing resumes. It absorbs
/// scheduler/clock granularity between the last cacheable reply and recovery.
const DEGRADED_GRACE_MS: u64 = 100;

enum Invalidate {
    Entry(u64, String),
    Inode(u64),
}

/// Why the hub suspended publishing cacheable metadata.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DegradeReason {
    /// The bounded invalidation queue rejected a message.
    QueueOverflow = 1,
    /// A whole-cache (snapshot generation) invalidation is being delivered.
    WholeCache = 2,
    /// The kernel rejected a notification, so a change may be unobservable.
    DeliveryFailure = 3,
}
impl DegradeReason {
    /// Stable label for the structured `event=publication_degraded` line.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::QueueOverflow => "queue_overflow",
            Self::WholeCache => "whole_cache",
            Self::DeliveryFailure => "delivery_failure",
        }
    }
    fn encode(self) -> u8 {
        self as u8
    }
    fn decode(value: u8) -> Option<Self> {
        match value {
            1 => Some(Self::QueueOverflow),
            2 => Some(Self::WholeCache),
            3 => Some(Self::DeliveryFailure),
            _ => None,
        }
    }
}

/// Non-blocking observation token for a published metadata reply. It also keeps
/// the reply in flight: a degradation only resumes leases once every reply that
/// could carry a pre-degradation answer has been written to the kernel.
pub struct ReplyPermit {
    shared: Arc<Shared>,
}
impl Drop for ReplyPermit {
    fn drop(&mut self) {
        self.shared.in_flight.fetch_sub(1, Ordering::Release);
        self.shared
            .last_reply_ms
            .fetch_max(elapsed_ms(&self.shared), Ordering::AcqRel);
    }
}

struct Shared {
    notifier: Mutex<Option<Notifier>>,
    /// Test-only deterministic late-reply injection (feature `test-seams`).
    #[cfg(any(test, feature = "test-seams"))]
    reply_delay: Mutex<Option<Duration>>,
    active: AtomicBool,
    /// Mirrors `active && !degraded && ttl_ms > 0`: one relaxed load on the hot
    /// reply path so a disabled cache costs no gate or clock work.
    caching: AtomicBool,
    /// Cacheable publication is suspended; cleared after a quiet lease window.
    degraded: AtomicBool,
    degraded_reason: AtomicU8,
    /// Latest degradation event, in monotonic milliseconds since `start`.
    degraded_since_ms: AtomicU64,
    /// Duration of the last completed degraded quiet window, in milliseconds
    /// (0 before the first recovery). Part of the #27 health accessor.
    last_degraded_ms: AtomicU64,
    /// Completion of the latest cacheable reply, in monotonic milliseconds.
    last_reply_ms: AtomicU64,
    /// Replies holding a permit; degradation resumes only at zero.
    in_flight: AtomicUsize,
    ttl_ms: AtomicU64,
    published: AtomicU64,
    invalidations: AtomicU64,
    delivered: AtomicU64,
    failures: AtomicU64,
    overflows: AtomicU64,
    recoveries: AtomicU64,
    start: Instant,
    /// Serializes degradation/recovery transitions only; never taken while
    /// holding the notifier lock, and never taken by a reply path.
    recovery: Mutex<()>,
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
    /// Degradation transitions: a committed change may have been unobservable.
    pub failures: u64,
    /// Degradations caused by a rejected invalidation/notification.
    pub overflows: u64,
    /// Automatic recoveries after a quiet lease window (no restart needed).
    pub recoveries: u64,
    pub active: bool,
    pub degraded: bool,
    pub reason: Option<DegradeReason>,
}

/// Consolidated publication-gate health (issue #27 residual): the existing
/// counters plus the state a health surface needs, in one non-blocking call.
/// Deliberately not an IPC/D-Bus/JSON contract yet; `stats()` stays the
/// compatibility accessor for existing callers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NotifyHealth {
    pub stats: NotifyStats,
    /// Unix-ms TTL configured for attribute leases (0 disables caching).
    pub ttl_ms: u64,
    /// Milliseconds since the current degradation started; `None` when healthy.
    pub degraded_for_ms: Option<u64>,
    /// Duration of the last completed degraded window; 0 if never degraded.
    pub last_degraded_ms: u64,
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

fn elapsed_ms(shared: &Shared) -> u64 {
    u64::try_from(shared.start.elapsed().as_millis()).unwrap_or(u64::MAX)
}

#[cfg(test)]
thread_local! {
    /// Test seam standing in for the daemon's stderr consumer (journald).
    static CAPTURED_EVENTS: std::cell::RefCell<Vec<String>> =
        const { std::cell::RefCell::new(Vec::new()) };
}

/// Emit one structured `key=value` lifecycle line. The daemon writes it to
/// stderr, which systemd forwards to the journal; unit tests capture the same
/// line on the emitting thread instead of parsing process stderr.
fn emit_event(line: String) {
    #[cfg(test)]
    CAPTURED_EVENTS.with(|events| events.borrow_mut().push(line.clone()));
    eprintln!("{line}");
}

#[cfg(test)]
pub(crate) fn drain_captured_events() -> Vec<String> {
    CAPTURED_EVENTS.with(|events| std::mem::take(&mut *events.borrow_mut()))
}

/// The moment the previous lease window is provably over: the later of the
/// degradation event and the last completed cacheable reply, plus the TTL that
/// reply could have carried, plus a short grace.
fn quiet_deadline_ms(shared: &Shared, ttl_ms: u64) -> u64 {
    let from = shared
        .degraded_since_ms
        .load(Ordering::Acquire)
        .max(shared.last_reply_ms.load(Ordering::Acquire));
    from.saturating_add(ttl_ms).saturating_add(DEGRADED_GRACE_MS)
}

/// Suspend cacheable publication. Repeated events extend the quiet window but
/// only the first transition counts as a failure. A hub without an installed
/// notifier never granted a lease; its degradation flag still gates the index
/// (a committed change may already be unobservable) and `install` clears it.
fn begin_degraded(shared: &Shared, reason: DegradeReason) {
    let _guard = shared.recovery.lock().unwrap();
    shared.caching.store(false, Ordering::Release);
    let transition = !shared.degraded.swap(true, Ordering::AcqRel);
    if transition {
        shared.failures.fetch_add(1, Ordering::Relaxed);
        crate::diagnostics::count(crate::diagnostics::Stage::TtlDegraded);
    }
    let previous_reason = DegradeReason::decode(shared.degraded_reason.load(Ordering::Acquire));
    shared
        .degraded_reason
        .store(reason.encode(), Ordering::Release);
    let degraded_since_ms = elapsed_ms(shared);
    shared
        .degraded_since_ms
        .store(degraded_since_ms, Ordering::Release);
    // One line per transition or reason change, never per repeated event: a
    // degradation storm (every dropped invalidation extends the window) must
    // not flood journald. The recovery line carries the final counters.
    if transition || previous_reason != Some(reason) {
        emit_event(format!(
            "drive-native-fs: event=publication_degraded reason={} transition={} overflows={} failures={} invalidations={} delivered={} replies_in_flight={} degraded_since_ms={}",
            reason.as_str(),
            transition,
            shared.overflows.load(Ordering::Relaxed),
            shared.failures.load(Ordering::Relaxed),
            shared.invalidations.load(Ordering::Relaxed),
            shared.delivered.load(Ordering::Relaxed),
            shared.in_flight.load(Ordering::Acquire),
            degraded_since_ms,
        ));
    }
}

/// Resume publication once the previous lease window is fully quiet.
fn try_recover(shared: &Shared) -> bool {
    if !shared.degraded.load(Ordering::Acquire) {
        return true;
    }
    {
        let _guard = shared.recovery.lock().unwrap();
        if !shared.degraded.load(Ordering::Acquire) {
            return true;
        }
        // A reply that started before the degradation may still publish an
        // answer computed before it; its lease must expire before leases resume.
        if shared.in_flight.load(Ordering::Acquire) != 0 {
            return false;
        }
        let ttl_ms = shared.ttl_ms.load(Ordering::Relaxed);
        if elapsed_ms(shared) < quiet_deadline_ms(shared, ttl_ms) {
            return false;
        }
        shared.degraded_reason.store(0, Ordering::Release);
        shared.degraded.store(false, Ordering::Release);
        shared.recoveries.fetch_add(1, Ordering::Relaxed);
        crate::diagnostics::count(crate::diagnostics::Stage::TtlRecovered);
        let caching = shared.active.load(Ordering::Acquire) && ttl_ms > 0;
        shared.caching.store(caching, Ordering::Release);
        let degraded_ms = elapsed_ms(shared)
            .saturating_sub(shared.degraded_since_ms.load(Ordering::Acquire));
        shared.last_degraded_ms.store(degraded_ms, Ordering::Release);
        emit_event(format!(
            "drive-native-fs: event=publication_recovered degraded_ms={} recoveries={} overflows={} failures={} invalidations={} delivered={} ttl_ms={} caching={}",
            degraded_ms,
            shared.recoveries.load(Ordering::Relaxed),
            shared.overflows.load(Ordering::Relaxed),
            shared.failures.load(Ordering::Relaxed),
            shared.invalidations.load(Ordering::Relaxed),
            shared.delivered.load(Ordering::Relaxed),
            ttl_ms,
            caching,
        ));
    }
    true
}

/// Count and apply a wholesale generation invalidation. Deliberate: the caller
/// replaces every per-node invalidation of a snapshot commit.
fn note_whole_cache(shared: &Shared) {
    shared.invalidations.fetch_add(1, Ordering::Relaxed);
    crate::diagnostics::count(crate::diagnostics::Stage::TtlInvalidations);
    let touched_directories = {
        let mut touched = shared.touched.lock().unwrap();
        let count = touched.len();
        touched.clear();
        count
    };
    emit_event(format!(
        "drive-native-fs: event=generation_invalidation invalidations={} delivered={} touched_directories={}",
        shared.invalidations.load(Ordering::Relaxed),
        shared.delivered.load(Ordering::Relaxed),
        touched_directories,
    ));
    begin_degraded(shared, DegradeReason::WholeCache);
}

/// Best-effort `FUSE_NOTIFY_INC_EPOCH` (kernel >= 6.16). Unsupported kernels
/// answer EINVAL/ENOSYS, which is remembered and treated as optional; any other
/// failure suspends publication for one quiet lease window like a lost
/// invalidation.
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
                begin_degraded(shared, DegradeReason::DeliveryFailure);
            }
        }
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
            degraded: AtomicBool::new(false),
            degraded_reason: AtomicU8::new(0),
            degraded_since_ms: AtomicU64::new(0),
            last_degraded_ms: AtomicU64::new(0),
            last_reply_ms: AtomicU64::new(0),
            in_flight: AtomicUsize::new(0),
            ttl_ms: AtomicU64::new(ttl_ms),
            published: AtomicU64::new(0),
            invalidations: AtomicU64::new(0),
            delivered: AtomicU64::new(0),
            failures: AtomicU64::new(0),
            overflows: AtomicU64::new(0),
            recoveries: AtomicU64::new(0),
            start: Instant::now(),
            recovery: Mutex::new(()),
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
                if worker_shared.degraded.load(Ordering::Acquire) {
                    continue;
                }
                let notifier = worker_shared.notifier.lock().unwrap();
                let Some(notifier) = notifier.as_ref() else {
                    continue;
                };
                // Strict variants: the lenient fuser API swallows ENOENT,
                // which would hide a dropped invalidation. Any error suspends
                // cacheable publication for one quiet lease window.
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
                    // publishing cacheable metadata for a lease window.
                    if error.raw_os_error() != Some(libc::ENOENT) {
                        begin_degraded(&worker_shared, DegradeReason::DeliveryFailure);
                    }
                } else if worker_shared.ttl_ms.load(Ordering::Relaxed) != 0 {
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
        self.clear_degraded();
        self.refresh_caching();
    }
    /// Stop publishing or delivering: the mount is going away.
    pub fn deactivate(&self) {
        self.shared.active.store(false, Ordering::Release);
        self.shared.caching.store(false, Ordering::Release);
        self.shared.notifier.lock().unwrap().take();
        self.clear_degraded();
    }
    /// A new session (or a test activation) starts from a clean degradation
    /// state: the previous notifier connection and its lost changes are gone.
    fn clear_degraded(&self) {
        let _guard = self.shared.recovery.lock().unwrap();
        self.shared.degraded_reason.store(0, Ordering::Release);
        self.shared.degraded.store(false, Ordering::Release);
    }
    fn refresh_caching(&self) {
        let caching = self.shared.active.load(Ordering::Acquire)
            && !self.shared.degraded.load(Ordering::Acquire)
            && self.shared.ttl_ms.load(Ordering::Relaxed) > 0;
        self.shared.caching.store(caching, Ordering::Release);
    }
    /// Enable a TTL without a real notifier (ordering/fallback tests only).
    #[doc(hidden)]
    pub fn activate_ttl_for_tests(&self, ttl_ms: u64) {
        self.shared.ttl_ms.store(ttl_ms, Ordering::Relaxed);
        self.shared.active.store(true, Ordering::Release);
        self.clear_degraded();
        self.refresh_caching();
    }
    /// Model a rejected notification / unsupported kernel (tests only).
    #[doc(hidden)]
    pub fn force_degraded_for_tests(&self) {
        begin_degraded(&self.shared, DegradeReason::DeliveryFailure);
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

    /// Configured TTL, or zero while caching is not publishable. Polling this
    /// during a degraded window is what resumes publication once it is quiet.
    #[inline]
    pub fn ttl(&self) -> Duration {
        if !self.shared.active.load(Ordering::Acquire) {
            return Duration::ZERO;
        }
        if self.shared.degraded.load(Ordering::Acquire) && !try_recover(&self.shared) {
            return Duration::ZERO;
        }
        Duration::from_millis(self.shared.ttl_ms.load(Ordering::Relaxed))
    }
    #[inline]
    pub fn is_caching(&self) -> bool {
        // One relaxed load on the hot path; the slow path only runs when the
        // cache is configured and installed.
        if !self.shared.caching.load(Ordering::Relaxed) {
            if !self.shared.degraded.load(Ordering::Relaxed) || !try_recover(&self.shared) {
                return false;
            }
        }
        !self.ttl().is_zero()
    }
    /// A lost notification suspends index publication as well as attribute TTL
    /// until the hub recovers.
    #[inline]
    pub fn delivery_failed(&self) -> bool {
        if !self.shared.degraded.load(Ordering::Acquire) {
            return false;
        }
        !try_recover(&self.shared)
    }
    /// Observation only, never a lock: metadata reply paths must stay
    /// non-blocking. Holding this does not order replies against invalidations;
    /// the bounded TTL and the authoritative data path are the contract.
    #[inline]
    pub fn reply_guard(&self) -> Option<ReplyPermit> {
        if !self.is_caching() {
            return None;
        }
        // Reserve the in-flight slot before the authority read so a concurrent
        // degradation cannot resume leases while this answer is still pending.
        self.shared.in_flight.fetch_add(1, Ordering::Acquire);
        self.shared.published.fetch_add(1, Ordering::Relaxed);
        crate::diagnostics::count(crate::diagnostics::Stage::TtlPublishedReplies);
        Some(ReplyPermit {
            shared: self.shared.clone(),
        })
    }
    /// An upstream queue lost a committed invalidation. Suspend all new
    /// attribute leases; existing kernel leases still have their granted bound
    /// and publication resumes after that bound plus a short grace.
    pub fn invalidate_delivery_failed(&self) {
        self.shared.overflows.fetch_add(1, Ordering::Relaxed);
        begin_degraded(&self.shared, DegradeReason::QueueOverflow);
    }
    /// One generation invalidation for a whole snapshot commit: no per-node
    /// message has to fit the bounded delivery queue. It protects replies even
    /// when no notifier instance is installed, and revalidates every dentry on
    /// kernels that support epoch bumps.
    pub fn invalidate_all(&self) {
        note_whole_cache(&self.shared);
        let notifier = self.shared.notifier.lock().unwrap();
        if let Some(notifier) = notifier.as_ref() {
            bump_epoch(&self.shared, notifier);
        }
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
            self.shared.overflows.fetch_add(1, Ordering::Relaxed);
            begin_degraded(&self.shared, DegradeReason::QueueOverflow);
        }
    }
    pub fn invalidate_inode(&self, ino: u64) {
        self.mark_directory_touched(ino);
        self.shared.invalidations.fetch_add(1, Ordering::Relaxed);
        crate::diagnostics::count(crate::diagnostics::Stage::TtlInvalidations);
        if self.queue.try_send(Invalidate::Inode(ino)).is_err() {
            self.shared.overflows.fetch_add(1, Ordering::Relaxed);
            begin_degraded(&self.shared, DegradeReason::QueueOverflow);
        }
    }
    pub fn stats(&self) -> NotifyStats {
        NotifyStats {
            published_replies: self.shared.published.load(Ordering::Relaxed),
            invalidations: self.shared.invalidations.load(Ordering::Relaxed),
            delivered: self.shared.delivered.load(Ordering::Relaxed),
            failures: self.shared.failures.load(Ordering::Relaxed),
            overflows: self.shared.overflows.load(Ordering::Relaxed),
            recoveries: self.shared.recoveries.load(Ordering::Relaxed),
            active: self.shared.active.load(Ordering::Acquire),
            degraded: self.shared.degraded.load(Ordering::Acquire),
            reason: DegradeReason::decode(self.shared.degraded_reason.load(Ordering::Acquire)),
        }
    }
    /// One non-blocking health snapshot for the publication gate (#27
    /// observability). Pure observation: never waits for the recovery lock, so
    /// a daemon health probe cannot stall replies; transitions emit their own
    /// structured stderr lines (`event=publication_degraded|recovered`).
    pub fn health(&self) -> NotifyHealth {
        let degraded = self.shared.degraded.load(Ordering::Acquire);
        let degraded_since_ms = self.shared.degraded_since_ms.load(Ordering::Acquire);
        NotifyHealth {
            stats: self.stats(),
            ttl_ms: self.shared.ttl_ms.load(Ordering::Relaxed),
            degraded_for_ms: degraded
                .then(|| elapsed_ms(&self.shared).saturating_sub(degraded_since_ms)),
            last_degraded_ms: self.shared.last_degraded_ms.load(Ordering::Acquire),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wait_until(deadline: Duration, mut condition: impl FnMut() -> bool) -> bool {
        let end = Instant::now() + deadline;
        while Instant::now() < end {
            if condition() {
                return true;
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        condition()
    }

    #[test]
    fn invalidation_delivery_never_blocks_metadata_replies() {
        let hub = NotifyHub::with_ttl_ms(2000);
        hub.activate_ttl_for_tests(2000);
        let permit = hub.reply_guard().expect("caching active");
        for index in 0..1000 {
            hub.invalidate_entry(1, &format!("name-{index}"));
        }
        let started = Instant::now();
        let _second = hub.reply_guard().expect("caching active");
        assert!(
            started.elapsed() < Duration::from_millis(100),
            "reply path blocked on invalidation delivery"
        );
        drop(permit);
        assert!(wait_until(Duration::from_secs(2), || hub.stats().delivered > 0));
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
    fn zero_ttl_disables_caching_and_degradation_recovers_without_restart() {
        let hub = NotifyHub::with_ttl_ms(0);
        assert!(hub.ttl().is_zero());
        assert!(hub.reply_guard().is_none());
        hub.activate_ttl_for_tests(1500);
        assert_eq!(hub.ttl(), Duration::from_millis(1500));
        hub.deactivate();
        assert!(hub.ttl().is_zero());

        // A rejected/failing invalidation suspends new leases for one old-lease
        // window; earlier cacheable answers then age out within their bound.
        hub.activate_ttl_for_tests(30);
        hub.force_degraded_for_tests();
        assert!(hub.ttl().is_zero());
        assert!(hub.reply_guard().is_none());
        let stats = hub.stats();
        assert!(stats.degraded);
        assert_eq!(stats.failures, 1);
        assert_eq!(stats.reason, Some(DegradeReason::DeliveryFailure));

        // No restart/remount: publishing resumes on its own once quiet.
        assert!(
            wait_until(Duration::from_secs(2), || !hub.ttl().is_zero()),
            "quarantined hub did not recover after its quiet lease window"
        );
        assert!(hub.reply_guard().is_some());
        let stats = hub.stats();
        assert!(!stats.degraded);
        assert_eq!(stats.reason, None);
        assert_eq!(stats.recoveries, 1);
    }

    #[test]
    fn recovery_waits_for_replies_that_predate_the_degradation() {
        let hub = NotifyHub::with_ttl_ms(20);
        hub.activate_ttl_for_tests(20);
        let permit = hub.reply_guard().expect("caching active");
        hub.force_degraded_for_tests();
        // The lease window alone would be over by now, but the pre-degradation
        // reply is still pending and could publish an answer computed earlier.
        std::thread::sleep(Duration::from_millis(250));
        assert!(hub.ttl().is_zero(), "leases resumed with an old reply in flight");
        drop(permit);
        assert!(wait_until(Duration::from_secs(2), || !hub.ttl().is_zero()));
        assert_eq!(hub.stats().recoveries, 1);
    }

    #[test]
    fn a_whole_cache_invalidation_is_one_message_and_quarantines_the_hub() {
        let hub = NotifyHub::with_ttl_ms(20);
        hub.activate_ttl_for_tests(20);
        assert!(!hub.ttl().is_zero());
        hub.invalidate_all();
        assert!(hub.ttl().is_zero());
        assert!(
            wait_until(Duration::from_secs(2), || !hub.ttl().is_zero()),
            "whole-cache invalidation did not recover"
        );
        let stats = hub.stats();
        assert_eq!(stats.invalidations, 1);
        assert_eq!(stats.overflows, 0);
        assert_eq!(stats.recoveries, 1);
    }
    #[test]
    fn overflow_health_reports_degradation_window_and_recovery() {
        let hub = NotifyHub::with_ttl_ms(20);
        hub.activate_ttl_for_tests(20);
        assert!(hub.health().degraded_for_ms.is_none());
        hub.invalidate_delivery_failed();
        let health = hub.health();
        assert!(health.stats.degraded);
        assert_eq!(health.stats.reason, Some(DegradeReason::QueueOverflow));
        assert_eq!(health.stats.overflows, 1);
        assert!(health.degraded_for_ms.is_some());
        assert_eq!(health.last_degraded_ms, 0);
        assert_eq!(health.ttl_ms, 20);
        assert!(
            wait_until(Duration::from_secs(2), || !hub.ttl().is_zero()),
            "degraded publication never auto-recovered"
        );
        let health = hub.health();
        assert!(!health.stats.degraded);
        assert!(health.degraded_for_ms.is_none());
        assert_eq!(health.stats.recoveries, 1);
        assert_eq!(health.stats.overflows, 1);
        // The quiet window covers at least the old-lease TTL plus the grace.
        assert!(
            health.last_degraded_ms >= 100,
            "recovered after only {}ms",
            health.last_degraded_ms
        );
    }

    #[test]
    fn publication_transitions_emit_structured_stderr_events() {
        let _ = super::drain_captured_events();
        let hub = NotifyHub::with_ttl_ms(20);
        hub.activate_ttl_for_tests(20);
        hub.invalidate_delivery_failed();
        hub.invalidate_all();
        assert!(wait_until(Duration::from_secs(2), || !hub.ttl().is_zero()));
        let events = super::drain_captured_events();
        let joined = events.join("\n");
        assert!(joined.contains("event=publication_degraded"), "{joined}");
        assert!(joined.contains("reason=queue_overflow"), "{joined}");
        assert!(joined.contains("overflows=1"), "{joined}");
        assert!(joined.contains("event=generation_invalidation"), "{joined}");
        assert!(joined.contains("event=publication_recovered"), "{joined}");
        assert!(joined.contains("degraded_ms="), "{joined}");
    }

}
