//! Per-request deadlines, bounded metadata Store waits and per-opcode metrics
//! for issue #59. Everything here is process-local, allocation-free on the
//! request path and never holds a lock while emitting an observation.
//!
//! The FUSE layer cannot preempt a blocking userspace call (no kernel-level
//! cancellation exists for a plain handler thread). What this module provides
//! is therefore: a bounded wait where the blocking point is controllable (the
//! shared Store lock on metadata reads), a per-opcode server-side deadline that
//! detects an over-budget reply, one structured `fuse.request_timeout` event
//! per late request, and the latency/timeout/lock-wait aggregates that
//! `GetOverview` exposes. It deliberately does not claim to cancel a blocked
//! worker RPC.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

pub const MAX_BACKGROUND_DEFAULT: u16 = 64;
pub const CONGESTION_THRESHOLD_DEFAULT: u16 = 48;
const METADATA_DEADLINE_MS: u64 = 5_000;
const READ_DEADLINE_MS: u64 = 30_000;
const WRITE_DEADLINE_MS: u64 = 60_000;
const READDIR_DEADLINE_MS: u64 = 10_000;
const METADATA_LOCK_WAIT_MS: u64 = 50;

/// Requested kernel queue tuning. `congestion_threshold` is normalized to
/// `1..=max_background` exactly like the fuser default derivation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FuseTuning {
    pub max_background: u16,
    pub congestion_threshold: u16,
}
impl Default for FuseTuning {
    fn default() -> Self {
        Self {
            max_background: MAX_BACKGROUND_DEFAULT,
            congestion_threshold: CONGESTION_THRESHOLD_DEFAULT,
        }
    }
}
impl FuseTuning {
    /// Parse the bounded integer form used by the environment/profile
    /// overrides. Invalid or zero values fall back to the default so a typo can
    /// never turn into an unbounded kernel queue or a panic.
    pub fn parse(max_background: Option<&str>, congestion_threshold: Option<&str>) -> Self {
        fn bounded(value: Option<&str>, fallback: u16) -> u16 {
            value
                .and_then(|text| text.trim().parse::<u16>().ok())
                .filter(|value| *value > 0)
                .unwrap_or(fallback)
        }
        let max_background = bounded(max_background, MAX_BACKGROUND_DEFAULT).max(1);
        let congestion_threshold = bounded(congestion_threshold, CONGESTION_THRESHOLD_DEFAULT);
        Self {
            max_background,
            congestion_threshold: congestion_threshold.min(max_background).max(1),
        }
    }
    fn from_env() -> Self {
        let max = std::env::var("DRIVE_NATIVE_FUSE_MAX_BACKGROUND").ok();
        let congestion = std::env::var("DRIVE_NATIVE_FUSE_CONGESTION_THRESHOLD").ok();
        Self::parse(max.as_deref(), congestion.as_deref())
    }
}

/// Per-opcode server-side deadline budgets. Overridable for tests and
/// constrained deployments; defaults follow the issue proposal.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RequestBudgets {
    pub metadata: Duration,
    pub read: Duration,
    pub write: Duration,
    pub readdir: Duration,
}
impl Default for RequestBudgets {
    fn default() -> Self {
        Self {
            metadata: Duration::from_millis(METADATA_DEADLINE_MS),
            read: Duration::from_millis(READ_DEADLINE_MS),
            write: Duration::from_millis(WRITE_DEADLINE_MS),
            readdir: Duration::from_millis(READDIR_DEADLINE_MS),
        }
    }
}
impl RequestBudgets {
    fn from_env() -> Self {
        fn millis(name: &str, fallback: Duration) -> Duration {
            std::env::var(name)
                .ok()
                .and_then(|text| text.trim().parse::<u64>().ok())
                .filter(|value| *value > 0)
                .map(Duration::from_millis)
                .unwrap_or(fallback)
        }
        let default = Self::default();
        Self {
            metadata: millis("DRIVE_NATIVE_FUSE_DEADLINE_METADATA_MS", default.metadata),
            read: millis("DRIVE_NATIVE_FUSE_DEADLINE_READ_MS", default.read),
            write: millis("DRIVE_NATIVE_FUSE_DEADLINE_WRITE_MS", default.write),
            readdir: millis("DRIVE_NATIVE_FUSE_DEADLINE_READDIR_MS", default.readdir),
        }
    }
    pub fn for_op(self, op: Op) -> Duration {
        match op {
            Op::Lookup | Op::Getattr | Op::Opendir | Op::Statfs => self.metadata,
            Op::Readdir | Op::ReaddirPlus => self.readdir,
            Op::Read => self.read,
            Op::Open
            | Op::Write
            | Op::Setattr
            | Op::Create
            | Op::Mkdir
            | Op::Remove
            | Op::Rename
            | Op::Flush
            | Op::Release => self.write,
            Op::Other => self.metadata,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(usize)]
pub enum Op {
    Lookup,
    Getattr,
    Opendir,
    Readdir,
    ReaddirPlus,
    Open,
    Read,
    Write,
    Setattr,
    Create,
    Mkdir,
    Remove,
    Rename,
    Flush,
    Release,
    Statfs,
    Other,
}
const OP_COUNT: usize = 17;
const OPS: [Op; OP_COUNT] = [
    Op::Lookup,
    Op::Getattr,
    Op::Opendir,
    Op::Readdir,
    Op::ReaddirPlus,
    Op::Open,
    Op::Read,
    Op::Write,
    Op::Setattr,
    Op::Create,
    Op::Mkdir,
    Op::Remove,
    Op::Rename,
    Op::Flush,
    Op::Release,
    Op::Statfs,
    Op::Other,
];
impl Op {
    pub fn as_str(self) -> &'static str {
        match self {
            Op::Lookup => "lookup",
            Op::Getattr => "getattr",
            Op::Opendir => "opendir",
            Op::Readdir => "readdir",
            Op::ReaddirPlus => "readdirplus",
            Op::Open => "open",
            Op::Read => "read",
            Op::Write => "write",
            Op::Setattr => "setattr",
            Op::Create => "create",
            Op::Mkdir => "mkdir",
            Op::Remove => "unlink",
            Op::Rename => "rename",
            Op::Flush => "flush",
            Op::Release => "release",
            Op::Statfs => "statfs",
            Op::Other => "other",
        }
    }
}

struct Aggregate {
    count: AtomicU64,
    total_us: AtomicU64,
    max_us: AtomicU64,
    buckets: [AtomicU64; 32],
}
impl Aggregate {
    const fn new() -> Self {
        Self {
            count: AtomicU64::new(0),
            total_us: AtomicU64::new(0),
            max_us: AtomicU64::new(0),
            buckets: [const { AtomicU64::new(0) }; 32],
        }
    }
    fn record(&self, us: u64) {
        self.count.fetch_add(1, Ordering::Relaxed);
        self.total_us.fetch_add(us, Ordering::Relaxed);
        self.max_us.fetch_max(us, Ordering::Relaxed);
        let bucket = if us <= 1 {
            0
        } else {
            (64 - (us - 1).leading_zeros() as usize).min(31)
        };
        self.buckets[bucket].fetch_add(1, Ordering::Relaxed);
    }
    fn summary(&self) -> HistogramSummary {
        let count = self.count.load(Ordering::Relaxed);
        let buckets: [u64; 32] =
            std::array::from_fn(|index| self.buckets[index].load(Ordering::Relaxed));
        HistogramSummary {
            count,
            total_us: self.total_us.load(Ordering::Relaxed),
            max_us: self.max_us.load(Ordering::Relaxed),
            p50_us: quantile(&buckets, count, 50),
            p95_us: quantile(&buckets, count, 95),
            p99_us: quantile(&buckets, count, 99),
        }
    }
}
struct OpAggregate {
    latency: Aggregate,
    timeouts: AtomicU64,
    retryable: AtomicU64,
}
impl OpAggregate {
    const fn new() -> Self {
        Self {
            latency: Aggregate::new(),
            timeouts: AtomicU64::new(0),
            retryable: AtomicU64::new(0),
        }
    }
}
/// Upper bound of the power-of-two histogram bucket that contains the
/// quantile, in microseconds. `0` means "no samples".
fn quantile(buckets: &[u64; 32], count: u64, percent: u64) -> u64 {
    if count == 0 {
        return 0;
    }
    let target = (count.saturating_mul(percent) + 99) / 100;
    let mut seen = 0;
    for (index, bucket) in buckets.iter().enumerate() {
        seen += bucket;
        if seen >= target {
            return 1u64 << index;
        }
    }
    1u64 << 31
}

static REQUESTS: [OpAggregate; OP_COUNT] = [const { OpAggregate::new() }; OP_COUNT];
static LOCK_WAITS: Aggregate = Aggregate::new();
static LOCK_WAIT_TIMEOUTS: AtomicU64 = AtomicU64::new(0);
static REQUEST_TIMEOUTS: AtomicU64 = AtomicU64::new(0);
static RETRYABLE_REPLIES: AtomicU64 = AtomicU64::new(0);
static DEGRADED_EVENTS: AtomicU64 = AtomicU64::new(0);
static TUNING: AtomicU64 = AtomicU64::new(0);
static EFFECTIVE_TUNING: AtomicU64 = AtomicU64::new(0);
static BUDGET_METADATA_MS: AtomicU64 = AtomicU64::new(METADATA_DEADLINE_MS);
static BUDGET_READ_MS: AtomicU64 = AtomicU64::new(READ_DEADLINE_MS);
static BUDGET_WRITE_MS: AtomicU64 = AtomicU64::new(WRITE_DEADLINE_MS);
static BUDGET_READDIR_MS: AtomicU64 = AtomicU64::new(READDIR_DEADLINE_MS);
static LOCK_WAIT_MS: AtomicU64 = AtomicU64::new(METADATA_LOCK_WAIT_MS);

fn pack(tuning: FuseTuning) -> u64 {
    (u64::from(tuning.max_background) << 16) | u64::from(tuning.congestion_threshold)
}
fn unpack(word: u64) -> FuseTuning {
    FuseTuning {
        max_background: (word >> 16) as u16,
        congestion_threshold: word as u16,
    }
}

/// Install the configured (environment) tuning and budgets. Idempotent from the
/// caller's point of view: the last call wins, which keeps tests and mounts of
/// a single process observable. Called before the FUSE session is created.
pub fn configure() -> FuseTuning {
    let tuning = FuseTuning::from_env();
    let budgets = RequestBudgets::from_env();
    install(tuning, budgets);
    tuning
}
/// Explicit form used by tests and by the future profile/CLI override.
pub fn install(tuning: FuseTuning, budgets: RequestBudgets) {
    let tuning = FuseTuning::parse(
        Some(&tuning.max_background.to_string()),
        Some(&tuning.congestion_threshold.to_string()),
    );
    TUNING.store(pack(tuning), Ordering::Release);
    EFFECTIVE_TUNING.store(pack(tuning), Ordering::Release);
    BUDGET_METADATA_MS.store(budgets.metadata.as_millis() as u64, Ordering::Release);
    BUDGET_READ_MS.store(budgets.read.as_millis() as u64, Ordering::Release);
    BUDGET_WRITE_MS.store(budgets.write.as_millis() as u64, Ordering::Release);
    BUDGET_READDIR_MS.store(budgets.readdir.as_millis() as u64, Ordering::Release);
}
/// Record the tuning the FUSE kernel connection accepted (`FUSE_INIT`). The
/// kernel may clamp values, so this is published from the init callback.
pub fn publish_effective(tuning: FuseTuning) {
    EFFECTIVE_TUNING.store(pack(tuning), Ordering::Release);
}
pub fn configured_tuning() -> FuseTuning {
    // `0` means nothing installed in this process yet (no mounted filesystem):
    // report the documented defaults rather than an empty tuning word.
    match TUNING.load(Ordering::Acquire) {
        0 => FuseTuning::default(),
        word => unpack(word),
    }
}
pub fn effective_tuning() -> FuseTuning {
    match EFFECTIVE_TUNING.load(Ordering::Acquire) {
        0 => FuseTuning::default(),
        word => unpack(word),
    }
}
pub fn budgets() -> RequestBudgets {
    RequestBudgets {
        metadata: Duration::from_millis(BUDGET_METADATA_MS.load(Ordering::Relaxed)),
        read: Duration::from_millis(BUDGET_READ_MS.load(Ordering::Relaxed)),
        write: Duration::from_millis(BUDGET_WRITE_MS.load(Ordering::Relaxed)),
        readdir: Duration::from_millis(BUDGET_READDIR_MS.load(Ordering::Relaxed)),
    }
}
/// Bound used by the metadata read paths when they need the serialized Store.
pub fn metadata_lock_wait() -> Duration {
    Duration::from_millis(LOCK_WAIT_MS.load(Ordering::Relaxed))
}

fn record_latency(op: Op, elapsed: Duration) {
    REQUESTS[op as usize]
        .latency
        .record(elapsed.as_micros() as u64);
}
fn record_timeout(op: Op, elapsed: Duration, detail: &str) {
    REQUESTS[op as usize]
        .timeouts
        .fetch_add(1, Ordering::Relaxed);
    REQUEST_TIMEOUTS.fetch_add(1, Ordering::Relaxed);
    DEGRADED_EVENTS.fetch_add(1, Ordering::Relaxed);
    // One structured line per late request: journald keeps it, monitoring can
    // match the `fuse.request_timeout` token, and no path is stored in process
    // memory or in the aggregate snapshot.
    let budget = budgets().for_op(op);
    eprintln!(
        "drive-native fuse.request_timeout op={} elapsed_ms={} budget_ms={} {}",
        op.as_str(),
        elapsed.as_millis(),
        budget.as_millis(),
        detail
    );
}
/// Count one reply turned into a retryable error because a bounded wait
/// expired. Emitted together with the `fuse.degraded_reason` event.
pub fn record_retryable(op: Op) {
    REQUESTS[op as usize]
        .retryable
        .fetch_add(1, Ordering::Relaxed);
    RETRYABLE_REPLIES.fetch_add(1, Ordering::Relaxed);
}
/// Record the degraded reason once per bounded-wait expiry. Callers pass a
/// short, path-free detail (inode/parent ids at most).
pub fn record_store_busy(op: Op, elapsed: Duration, detail: &str) {
    LOCK_WAIT_TIMEOUTS.fetch_add(1, Ordering::Relaxed);
    DEGRADED_EVENTS.fetch_add(1, Ordering::Relaxed);
    eprintln!(
        "drive-native fuse.degraded_reason=store_lock_timeout op={} lock_wait_ms={} {}",
        op.as_str(),
        elapsed.as_millis(),
        detail
    );
}
/// Observation of a completed bounded wait for the shared Store (metadata
/// path). Feeds the lock-wait histogram exposed through `GetOverview`.
pub fn record_lock_wait(elapsed: Duration) {
    LOCK_WAITS.record(elapsed.as_micros() as u64);
}

/// RAII deadline for one FUSE handler invocation.
///
/// The guard is created on the dispatcher thread and moved into the job that
/// finally replies. It records the reply latency and, when the request took
/// longer than its opcode budget, one timeout event. Dropping an unanswered
/// reply is already an error; the guard never changes the reply outcome by
/// itself (except where the caller explicitly checks `expired`).
pub struct RequestDeadline {
    op: Op,
    started: Instant,
    budget: Duration,
    finished: bool,
}
impl RequestDeadline {
    pub fn new(op: Op) -> Self {
        Self {
            op,
            started: Instant::now(),
            budget: budgets().for_op(op),
            finished: false,
        }
    }
    pub fn elapsed(&self) -> Duration {
        self.started.elapsed()
    }
    /// True once the opcode budget is exhausted. Callers use this only at
    /// controllable blocking points (a ready result is still returned; forcing
    /// EAGAIN after a successful projection would be worse than answering).
    pub fn expired(&self) -> bool {
        self.started.elapsed() >= self.budget
    }
    /// Record a reply that exceeded the budget with a short detail string
    /// (inode/parent only). Consumes the guard so the timeout is counted once.
    pub fn timeout(mut self, detail: &str) {
        let elapsed = self.started.elapsed();
        record_latency(self.op, elapsed);
        record_timeout(self.op, elapsed, detail);
        self.finished = true;
    }
}
impl Drop for RequestDeadline {
    fn drop(&mut self) {
        if self.finished {
            return;
        }
        let elapsed = self.started.elapsed();
        record_latency(self.op, elapsed);
        if elapsed >= self.budget {
            // The handler already replied (or dropped its reply, which fuser
            // answers with EIO); the request still exceeded its deadline.
            record_timeout(self.op, elapsed, "detail=unavailable");
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HistogramSummary {
    pub count: u64,
    pub total_us: u64,
    pub max_us: u64,
    pub p50_us: u64,
    pub p95_us: u64,
    pub p99_us: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OpLatency {
    pub op: &'static str,
    pub latency: HistogramSummary,
    pub timeouts: u64,
    pub retryable: u64,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FuseMetrics {
    pub configured_tuning: FuseTuning,
    pub effective_tuning: FuseTuning,
    pub budgets_ms: [(&'static str, u64); 4],
    pub ops: Vec<OpLatency>,
    pub request_timeouts: u64,
    pub retryable_replies: u64,
    pub lock_wait: HistogramSummary,
    pub lock_wait_timeouts: u64,
    pub degraded_events: u64,
}
pub fn snapshot() -> FuseMetrics {
    let budgets = budgets();
    FuseMetrics {
        configured_tuning: configured_tuning(),
        effective_tuning: effective_tuning(),
        budgets_ms: [
            ("metadata", budgets.metadata.as_millis() as u64),
            ("read", budgets.read.as_millis() as u64),
            ("write", budgets.write.as_millis() as u64),
            ("readdir", budgets.readdir.as_millis() as u64),
        ],
        ops: OPS
            .into_iter()
            .map(|op| OpLatency {
                op: op.as_str(),
                latency: REQUESTS[op as usize].latency.summary(),
                timeouts: REQUESTS[op as usize].timeouts.load(Ordering::Relaxed),
                retryable: REQUESTS[op as usize].retryable.load(Ordering::Relaxed),
            })
            .collect(),
        request_timeouts: REQUEST_TIMEOUTS.load(Ordering::Relaxed),
        retryable_replies: RETRYABLE_REPLIES.load(Ordering::Relaxed),
        lock_wait: LOCK_WAITS.summary(),
        lock_wait_timeouts: LOCK_WAIT_TIMEOUTS.load(Ordering::Relaxed),
        degraded_events: DEGRADED_EVENTS.load(Ordering::Relaxed),
    }
}

impl FuseMetrics {
    /// Additive JSON projection for `GetOverview`. Kept here (not in the
    /// runtime) so the FUSE data plane owns its counters; the runtime only
    /// serializes. `schema` stays 1 and no IPC version changes.
    pub fn to_json(&self) -> serde_json::Value {
        let tuning = |tuning: FuseTuning| {
            serde_json::json!({
                "max_background": tuning.max_background,
                "congestion_threshold": tuning.congestion_threshold,
            })
        };
        let histogram = |summary: HistogramSummary| {
            serde_json::json!({
                "count": summary.count,
                "total_us": summary.total_us,
                "max_us": summary.max_us,
                "p50_us": summary.p50_us,
                "p95_us": summary.p95_us,
                "p99_us": summary.p99_us,
            })
        };
        let budgets: serde_json::Map<String, serde_json::Value> = self
            .budgets_ms
            .iter()
            .map(|(name, value)| ((*name).to_owned(), serde_json::json!(value)))
            .collect();
        let ops: serde_json::Map<String, serde_json::Value> = self
            .ops
            .iter()
            .map(|op| {
                (
                    op.op.to_owned(),
                    serde_json::json!({
                        "count": op.latency.count,
                        "total_us": op.latency.total_us,
                        "max_us": op.latency.max_us,
                        "p50_us": op.latency.p50_us,
                        "p95_us": op.latency.p95_us,
                        "p99_us": op.latency.p99_us,
                        "timeouts": op.timeouts,
                        "retryable": op.retryable,
                    }),
                )
            })
            .collect();
        serde_json::json!({
            "schema": 1,
            "configured_tuning": tuning(self.configured_tuning),
            "effective_tuning": tuning(self.effective_tuning),
            "request_budget_ms": serde_json::Value::Object(budgets),
            "ops": serde_json::Value::Object(ops),
            "request_timeouts": self.request_timeouts,
            "retryable_replies": self.retryable_replies,
            "lock_wait": histogram(self.lock_wait),
            "lock_wait_timeouts": self.lock_wait_timeouts,
            "degraded_events": self.degraded_events,
            "deadline_capability": "detect-and-report+bounded-store-wait",
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tuning_defaults_match_the_issue_proposal() {
        assert_eq!(FuseTuning::default().max_background, 64);
        assert_eq!(FuseTuning::default().congestion_threshold, 48);
    }

    #[test]
    fn tuning_parse_rejects_zero_and_clamps_congestion_to_max() {
        let parsed = FuseTuning::parse(Some("0"), Some("999"));
        assert_eq!(parsed.max_background, MAX_BACKGROUND_DEFAULT);
        assert_eq!(parsed.congestion_threshold, MAX_BACKGROUND_DEFAULT);
        let parsed = FuseTuning::parse(Some("32"), Some("48"));
        assert_eq!(parsed.max_background, 32);
        assert_eq!(parsed.congestion_threshold, 32);
        let parsed = FuseTuning::parse(Some("not-a-number"), Some("--1"));
        assert_eq!(parsed, FuseTuning::default());
        let parsed = FuseTuning::parse(Some("64"), Some("48"));
        assert_eq!(parsed.max_background, 64);
        assert_eq!(parsed.congestion_threshold, 48);
    }

    #[test]
    fn budgets_are_per_opcode() {
        let budgets = RequestBudgets::default();
        assert_eq!(budgets.for_op(Op::Lookup), Duration::from_secs(5));
        assert_eq!(budgets.for_op(Op::Getattr), Duration::from_secs(5));
        assert_eq!(budgets.for_op(Op::Read), Duration::from_secs(30));
        assert_eq!(budgets.for_op(Op::Write), Duration::from_secs(60));
        assert_eq!(budgets.for_op(Op::Readdir), Duration::from_secs(10));
    }

    #[test]
    fn quantile_returns_a_bucket_upper_bound() {
        let mut buckets = [0u64; 32];
        // 100 samples: 50 at 1 us (bucket 0), 50 at 1024 us (bucket 10).
        buckets[0] = 50;
        buckets[10] = 50;
        assert_eq!(quantile(&buckets, 100, 50), 1);
        assert_eq!(quantile(&buckets, 100, 95), 1024);
        assert_eq!(quantile(&buckets, 0, 99), 0);
    }

    #[test]
    fn deadline_records_latency_and_expiry() {
        let before = snapshot();
        let guard = RequestDeadline {
            op: Op::Getattr,
            started: Instant::now() - Duration::from_secs(7),
            budget: Duration::from_secs(5),
            finished: false,
        };
        drop(guard);
        let after = snapshot();
        assert_eq!(
            after.request_timeouts,
            before.request_timeouts + 1,
            "an over-budget request is counted as a timeout"
        );
        let getattr = |m: &FuseMetrics| m.ops.iter().find(|op| op.op == "getattr").unwrap().clone();
        assert_eq!(
            getattr(&after).latency.count,
            getattr(&before).latency.count + 1
        );
    }
}
