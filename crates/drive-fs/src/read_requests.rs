//! Kernel read ownership: exactly one reply per request, and a real reply
//! deadline for issue #59.
//!
//! A plain FUSE handler cannot preempt a blocking userspace call, so the read
//! path keeps its worker call as-is. What it *can* do is stop the caller from
//! waiting for it: one shared [`ReplyClaim`] is raced by the worker result, the
//! deadline watchdog and the interrupt path, and only the winner gets to send
//! the single kernel reply. When the budget expires first, the watchdog answers
//! a defined error, marks the request answered and every later worker result is
//! discarded instead of being sent as a second reply.
//!
//! The same [`ReadCancellation`] token carries the deadline to the hydration
//! worker (see [`crate::Hydrator`]), which lets the runtime skip work that is
//! already expired and stop waiting for a reply nobody will accept.

use fuser::ReplyData;
use std::{
    collections::{BinaryHeap, HashMap},
    sync::{
        Arc, Condvar, Mutex, Weak,
        atomic::{AtomicBool, AtomicU8, Ordering},
    },
    time::{Duration, Instant},
};

/// Who owns the one kernel reply of a read request.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ReplyOwner {
    Pending = 0,
    Worker = 1,
    Deadline = 2,
    Abort = 3,
}
impl ReplyOwner {
    fn from_code(code: u8) -> Self {
        match code {
            1 => Self::Worker,
            2 => Self::Deadline,
            3 => Self::Abort,
            _ => Self::Pending,
        }
    }
}

/// Single-shot completion gate. The first claimer wins; everybody else must
/// drop its result. A `compare_exchange` (not a test-and-set) keeps the owner
/// readable for metrics, so a late worker can tell a deadline loss from a
/// user-interrupt loss.
#[derive(Debug, Default)]
struct ReplyClaim(AtomicU8);
impl ReplyClaim {
    fn try_claim(&self, owner: ReplyOwner) -> bool {
        self.0
            .compare_exchange(
                ReplyOwner::Pending as u8,
                owner as u8,
                Ordering::AcqRel,
                Ordering::Acquire,
            )
            .is_ok()
    }
    fn owner(&self) -> ReplyOwner {
        ReplyOwner::from_code(self.0.load(Ordering::Acquire))
    }
}

/// Coordination token shared by the FUSE request, its queue jobs and the
/// runtime hydration worker (issue #59).
///
/// `Default` produces an unbounded token that never expires, which keeps the
/// runtime worker tests and non-read callers unchanged. A read request gets
/// [`ReadCancellation::with_budget`]; from then on `is_expired`,
/// `should_abandon` and `remaining` are meaningful to the runtime, including
/// while it waits for a transfer.
#[derive(Clone, Debug)]
pub struct ReadCancellation(Arc<CancelState>);
#[derive(Debug)]
struct CancelState {
    cancelled: AtomicBool,
    answered: AtomicBool,
    started: Instant,
    /// `None` means "no deadline": the token only carries cancellation.
    budget: Option<Duration>,
}
impl Default for ReadCancellation {
    fn default() -> Self {
        Self(Arc::new(CancelState {
            cancelled: AtomicBool::new(false),
            answered: AtomicBool::new(false),
            started: Instant::now(),
            budget: None,
        }))
    }
}
impl ReadCancellation {
    /// Token bound to one FUSE request's opcode budget. `started` is the
    /// dispatcher timestamp of the request, so the budget covers queueing,
    /// Store waits, the worker call and the reply.
    pub fn with_budget(started: Instant, budget: Duration) -> Self {
        Self(Arc::new(CancelState {
            cancelled: AtomicBool::new(false),
            answered: AtomicBool::new(false),
            started,
            budget: Some(budget),
        }))
    }
    pub fn is_cancelled(&self) -> bool {
        self.0.cancelled.load(Ordering::Acquire)
    }
    pub fn cancel(&self) {
        self.0.cancelled.store(true, Ordering::Release);
    }
    /// The opcode budget, or `None` for an unbounded token.
    pub fn budget(&self) -> Option<Duration> {
        self.0.budget
    }
    pub fn started(&self) -> Instant {
        self.0.started
    }
    pub fn deadline(&self) -> Option<Instant> {
        self.0.budget.map(|budget| self.0.started + budget)
    }
    /// Time left before the reply deadline (zero once exhausted).
    pub fn remaining(&self) -> Option<Duration> {
        self.deadline()
            .map(|deadline| deadline.saturating_duration_since(Instant::now()))
    }
    pub fn is_expired(&self) -> bool {
        self.deadline()
            .is_some_and(|deadline| Instant::now() >= deadline)
    }
    /// True when doing more expensive work for this request is pointless:
    /// either the caller cancelled it or its reply deadline already passed.
    pub fn should_abandon(&self) -> bool {
        self.is_cancelled() || self.is_expired()
    }
    /// True once some path claimed the one kernel reply (metrics/tests).
    pub fn is_answered(&self) -> bool {
        self.0.answered.load(Ordering::Acquire)
    }
    fn mark_answered(&self) {
        self.0.answered.store(true, Ordering::Release);
    }
    /// Defined error for an abandoned request, mirroring the pre-existing
    /// interrupt semantics (EINTR) and the pre-existing read-timeout error
    /// (EIO) instead of inventing a new errno.
    pub fn abandoned_error(&self) -> Option<i32> {
        if self.is_cancelled() {
            Some(libc::EINTR)
        } else if self.is_expired() {
            Some(libc::EIO)
        } else {
            None
        }
    }
}

struct Pending {
    id: u64,
    state: Weak<ReadState>,
    cancel: ReadCancellation,
    claim: ReplyClaim,
    reply: Mutex<Option<ReplyData>>,
}
impl Pending {
    fn take_reply(&self, owner: ReplyOwner) -> Option<ReplyData> {
        if !self.claim.try_claim(owner) {
            return None;
        }
        self.cancel.mark_answered();
        self.reply
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
            .take()
    }
    /// Deadline path: answer the defined error, then drop. A worker that
    /// finishes later loses the claim and its result is discarded.
    fn expire(&self) {
        let Some(reply) = self.take_reply(ReplyOwner::Deadline) else {
            return;
        };
        crate::request::record_deadline_fired();
        reply.error(libc::EIO);
    }
}
impl Drop for Pending {
    fn drop(&mut self) {
        if let Some(state) = self.state.upgrade() {
            state
                .registry
                .lock()
                .unwrap_or_else(|poison| poison.into_inner())
                .remove(&self.id);
        }
    }
}

/// One deadline-ordered entry. Ordered reversed so `BinaryHeap` is a min-heap
/// on `(deadline, request id)`.
struct Entry {
    at: Instant,
    id: u64,
    pending: Weak<Pending>,
}
impl PartialEq for Entry {
    fn eq(&self, other: &Self) -> bool {
        (self.at, self.id) == (other.at, other.id)
    }
}
impl Eq for Entry {}
impl PartialOrd for Entry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Entry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.at.cmp(&self.at).then_with(|| other.id.cmp(&self.id))
    }
}

#[derive(Default)]
struct DeadlineQueue {
    heap: BinaryHeap<Entry>,
}
impl DeadlineQueue {
    /// Drop entries whose request already answered or vanished. Amortized
    /// O(log n) per request: a compaction only happens after roughly one
    /// unanswered-set worth of new registrations.
    fn compact(&mut self) {
        self.heap = std::mem::take(&mut self.heap)
            .into_iter()
            .filter(|entry| {
                entry
                    .pending
                    .upgrade()
                    .is_some_and(|pending| pending.claim.owner() == ReplyOwner::Pending)
            })
            .collect();
    }
}

struct ReadState {
    registry: Mutex<HashMap<u64, Weak<Pending>>>,
    deadlines: Mutex<DeadlineQueue>,
    wake: Condvar,
    stop: AtomicBool,
    thread: Mutex<Option<std::thread::JoinHandle<()>>>,
}

/// Bounded batch of read ownership handles for one mount. The deadline
/// watchdog is one process-local thread per mount, started on first read and
/// joined when the filesystem is dropped.
pub(super) struct ReadRequests {
    state: Arc<ReadState>,
}
impl Default for ReadRequests {
    fn default() -> Self {
        Self {
            state: Arc::new(ReadState {
                registry: Mutex::new(HashMap::new()),
                deadlines: Mutex::new(DeadlineQueue::default()),
                wake: Condvar::new(),
                stop: AtomicBool::new(false),
                thread: Mutex::new(None),
            }),
        }
    }
}
impl Drop for ReadRequests {
    fn drop(&mut self) {
        self.state.stop.store(true, Ordering::Release);
        self.state.wake.notify_all();
        let thread = self
            .state
            .thread
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
            .take();
        if let Some(thread) = thread {
            let _ = thread.join();
        }
    }
}
impl ReadRequests {
    fn lock(queue: &Mutex<DeadlineQueue>) -> std::sync::MutexGuard<'_, DeadlineQueue> {
        queue.lock().unwrap_or_else(|poison| poison.into_inner())
    }
    fn ensure_watchdog(state: &Arc<ReadState>) {
        let mut thread = state
            .thread
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        if thread.is_some() {
            return;
        }
        let owned = state.clone();
        match std::thread::Builder::new()
            .name("drive-fs-read-deadline".into())
            .spawn(move || Self::watchdog(owned))
        {
            Ok(handle) => *thread = Some(handle),
            // Without a watchdog the pre-#59 "detect after the fact" behavior
            // remains (the request still replies when its worker returns); do
            // not turn a thread-spawn failure into a data-plane failure.
            Err(_) => crate::request::record_watchdog_unavailable(),
        }
    }
    fn watchdog(state: Arc<ReadState>) {
        let mut queue = Self::lock(&state.deadlines);
        loop {
            if state.stop.load(Ordering::Acquire) {
                return;
            }
            let now = Instant::now();
            match queue.heap.peek().map(|entry| (entry.at, entry.pending.clone())) {
                Some((at, pending)) if at <= now => {
                    queue.heap.pop();
                    drop(queue);
                    if let Some(pending) = pending.upgrade() {
                        pending.expire();
                    }
                    queue = Self::lock(&state.deadlines);
                }
                Some((at, _)) => {
                    let wait = at.saturating_duration_since(now);
                    let (guard, _) = state
                        .wake
                        .wait_timeout(queue, wait)
                        .unwrap_or_else(|poison| poison.into_inner());
                    queue = guard;
                }
                None => {
                    queue = state
                        .wake
                        .wait(queue)
                        .unwrap_or_else(|poison| poison.into_inner());
                }
            }
        }
    }
    /// Register one read with its opcode budget. `started` must be the
    /// dispatcher timestamp of the FUSE request so queueing counts against the
    /// budget.
    pub(super) fn register_measured(
        &self,
        id: u64,
        reply: ReplyData,
        started: Instant,
        budget: Duration,
    ) -> ReadReply {
        Self::ensure_watchdog(&self.state);
        let cancel = ReadCancellation::with_budget(started, budget);
        let pending = Arc::new(Pending {
            id,
            state: Arc::downgrade(&self.state),
            cancel,
            claim: ReplyClaim::default(),
            reply: Mutex::new(Some(reply)),
        });
        {
            let mut registry = self
                .state
                .registry
                .lock()
                .unwrap_or_else(|poison| poison.into_inner());
            registry.insert(id, Arc::downgrade(&pending));
        }
        {
            let mut queue = Self::lock(&self.state.deadlines);
            let live = self
                .state
                .registry
                .lock()
                .unwrap_or_else(|poison| poison.into_inner())
                .len();
            if queue.heap.len() > live.saturating_mul(2).saturating_add(64) {
                queue.compact();
            }
            queue.heap.push(Entry {
                at: started + budget,
                id,
                pending: Arc::downgrade(&pending),
            });
        }
        self.state.wake.notify_one();
        ReadReply(pending)
    }
    pub fn cancel_all(&self) {
        let pending = self
            .state
            .registry
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
            .values()
            .filter_map(Weak::upgrade)
            .collect::<Vec<_>>();
        for pending in pending {
            pending.cancel.cancel();
            if let Some(reply) = pending.take_reply(ReplyOwner::Abort) {
                crate::request::record_cancelled();
                reply.error(libc::EINTR);
            }
        }
    }
    pub fn interrupt(&self, id: u64) -> bool {
        let pending = self
            .state
            .registry
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
            .get(&id)
            .and_then(Weak::upgrade);
        if let Some(pending) = pending {
            pending.cancel.cancel();
            if let Some(reply) = pending.take_reply(ReplyOwner::Abort) {
                crate::request::record_cancelled();
                reply.error(libc::EINTR);
            }
            true
        } else {
            false
        }
    }
}
/// Kernel read reply with exactly one send. Every method returns whether this
/// call produced the kernel reply; a `false` result means a competing path
/// (deadline watchdog or interrupt) already answered and the result must be
/// dropped, which is surfaced as a counter instead of a second reply.
pub(super) struct ReadReply(Arc<Pending>);
impl ReadReply {
    pub fn cancellation(&self) -> ReadCancellation {
        self.0.cancel.clone()
    }
    pub fn is_answered(&self) -> bool {
        self.0.claim.owner() != ReplyOwner::Pending
    }
    /// True when the deadline watchdog claimed the reply: a late worker result
    /// must be discarded, not published.
    pub fn deadline_won(&self) -> bool {
        self.0.claim.owner() == ReplyOwner::Deadline
    }
    /// Record that a competing path already answered instead of this one.
    pub fn record_lost(&self) {
        self.lost();
    }
    fn lost(&self) {
        if self.deadline_won() {
            crate::request::record_late_result();
        } else {
            crate::request::record_cancelled();
        }
    }
    pub fn data(self, data: &[u8]) -> bool {
        match self.0.take_reply(ReplyOwner::Worker) {
            Some(reply) => {
                reply.data(data);
                true
            }
            None => {
                self.lost();
                false
            }
        }
    }
    pub fn error(self, error: i32) -> bool {
        match self.0.take_reply(ReplyOwner::Worker) {
            Some(reply) => {
                reply.error(error);
                true
            }
            None => {
                self.lost();
                false
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicUsize;

    #[test]
    fn a_cancellation_token_without_a_budget_never_expires() {
        let token = ReadCancellation::default();
        std::thread::sleep(Duration::from_millis(2));
        assert!(!token.is_expired());
        assert!(!token.should_abandon());
        assert!(token.remaining().is_none());
        token.cancel();
        assert!(token.is_cancelled());
        assert!(token.should_abandon());
        assert_eq!(token.abandoned_error(), Some(libc::EINTR));
    }

    #[test]
    fn a_budgeted_token_reports_expiry_and_the_read_error() {
        let token = ReadCancellation::with_budget(Instant::now(), Duration::from_millis(5));
        assert!(!token.should_abandon());
        std::thread::sleep(Duration::from_millis(10));
        assert!(token.is_expired());
        assert!(token.should_abandon());
        assert_eq!(token.abandoned_error(), Some(libc::EIO));
        assert!(token.remaining().unwrap().is_zero());
        // Cancellation wins over expiry so a user interrupt keeps EINTR.
        token.cancel();
        assert_eq!(token.abandoned_error(), Some(libc::EINTR));
    }

    #[test]
    fn the_reply_claim_has_exactly_one_winner_under_contention() {
        let rounds = 200;
        for _ in 0..rounds {
            let claim = Arc::new(ReplyClaim::default());
            let winners = Arc::new(AtomicUsize::new(0));
            let mut threads = Vec::new();
            for owner in [ReplyOwner::Worker, ReplyOwner::Deadline, ReplyOwner::Abort] {
                let claim = claim.clone();
                let winners = winners.clone();
                threads.push(std::thread::spawn(move || {
                    if claim.try_claim(owner) {
                        winners.fetch_add(1, Ordering::Relaxed);
                    }
                }));
            }
            for thread in threads {
                thread.join().unwrap();
            }
            assert_eq!(
                winners.load(Ordering::Relaxed),
                1,
                "more than one path would have answered the same kernel reply"
            );
        }
    }

    #[test]
    fn the_deadline_watchdog_expires_an_unanswered_request() {
        let requests = ReadRequests::default();
        let started = Instant::now();
        let pending = Arc::new(Pending {
            id: 7,
            state: Arc::downgrade(&requests.state),
            cancel: ReadCancellation::with_budget(started, Duration::from_millis(20)),
            claim: ReplyClaim::default(),
            reply: Mutex::new(None),
        });
        requests
            .state
            .registry
            .lock()
            .unwrap()
            .insert(7, Arc::downgrade(&pending));
        requests
            .state
            .deadlines
            .lock()
            .unwrap()
            .heap
            .push(Entry {
                at: started + Duration::from_millis(20),
                id: 7,
                pending: Arc::downgrade(&pending),
            });
        ReadRequests::ensure_watchdog(&requests.state);
        let deadline = Instant::now() + Duration::from_secs(2);
        while pending.claim.owner() != ReplyOwner::Deadline {
            assert!(
                Instant::now() < deadline,
                "the watchdog never claimed an expired request"
            );
            std::thread::sleep(Duration::from_millis(2));
        }
        assert!(pending.cancel.is_answered());
        assert!(!pending.claim.try_claim(ReplyOwner::Worker));
    }
}
