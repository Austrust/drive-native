//! Opt-in process-local aggregates, without paths, inode IDs or per-call logs.
//! Histogram bounds are powers of two microseconds; quantiles are upper bounds.
use std::sync::{
    OnceLock,
    atomic::{AtomicU64, Ordering},
};
use std::time::Instant;

#[derive(Clone, Copy, Debug)]
#[repr(usize)]
pub enum Stage {
    LookupQueue,
    LookupStoreWait,
    LookupProjection,
    GetattrQueue,
    GetattrStoreWait,
    GetattrProjection,
    OpendirQueue,
    DirectoryBuilderWait,
    DirectoryStoreWait,
    DirectoryProjection,
    ReaddirQueue,
    ReaddirReply,
    Releasedir,
    DirectoryCacheHit,
    DirectoryCacheMiss,
    DirectoryIdentity,
    DirectoryCount,
    DirectoryInodes,
    DirectoryCollect,
    DirectoryCommit,
    /// Rows written into `fs_inodes` by one directory projection. `total_us`
    /// aggregates rows, not microseconds (0 = snapshot install preallocated).
    DirectoryInodeWrites,
    /// One opendir whose projection committed without writing any WAL frame.
    DirectoryCommitNoop,
    /// One lookup answered on the FUSE dispatcher thread (no queue wakeup).
    LookupInline,
    /// One getattr answered on the FUSE dispatcher thread (no queue wakeup).
    GetattrInline,
    /// Metadata replies published under a non-zero TTL (publication permit).
    TtlPublishedReplies,
    /// Invalidation deliveries attempted past the publication gate.
    TtlDelivered,
    /// Invalidations queued (may lag delivery).
    TtlInvalidations,
    /// Invalidation failures (notification rejected/unsupported or queue
    /// overflow). Non-zero means the mount suspended cacheable publication for
    /// one quiet lease window; it recovers on its own (see `TtlDegraded`).
    TtlFailures,
    /// One transition into the degraded quiet window (leases suspended).
    TtlDegraded,
    /// One automatic recovery out of the degraded quiet window.
    TtlRecovered,
    /// LOOKUP replies published with a non-zero positive-entry TTL.
    EntryTtlNonZero,
    /// LOOKUP replies published with a zero entry TTL.
    EntryTtlZero,
    /// Replies published with a non-zero attribute TTL.
    AttrTtlNonZero,
    /// Replies published with a zero attribute TTL.
    AttrTtlZero,
    /// Replies that could publish at least one non-zero TTL (cache effective).
    CacheableReplies,
    /// One READDIRPLUS reply served from a warm authoritative view.
    ReaddirPlusQueue,
    /// One bounded attribute page projection for READDIRPLUS.
    ReaddirPlusProjection,
    IndexHits,
    IndexMisses,
    IndexFallback,
    IndexDegraded,
    IndexBuildUs,
    /// Aggregate allocation samples, in bytes (not microseconds).
    IndexBytes,
    IndexPartial,
    PermissionDenied,
    SpaceForbidden,
    LookupRequests,
    GetattrRequests,
    OpendirRequests,
    ReaddirRequests,
    ReaddirPlusRequests,
    /// Elapsed time for index attempts/replies, not a CPU-time attribution.
    IndexReadUs,
}
const STAGES: [Stage; 52] = [
    Stage::LookupQueue,
    Stage::LookupStoreWait,
    Stage::LookupProjection,
    Stage::GetattrQueue,
    Stage::GetattrStoreWait,
    Stage::GetattrProjection,
    Stage::OpendirQueue,
    Stage::DirectoryBuilderWait,
    Stage::DirectoryStoreWait,
    Stage::DirectoryProjection,
    Stage::ReaddirQueue,
    Stage::ReaddirReply,
    Stage::Releasedir,
    Stage::DirectoryCacheHit,
    Stage::DirectoryCacheMiss,
    Stage::DirectoryIdentity,
    Stage::DirectoryCount,
    Stage::DirectoryInodes,
    Stage::DirectoryCollect,
    Stage::DirectoryCommit,
    Stage::DirectoryInodeWrites,
    Stage::DirectoryCommitNoop,
    Stage::LookupInline,
    Stage::GetattrInline,
    Stage::TtlPublishedReplies,
    Stage::TtlDelivered,
    Stage::TtlInvalidations,
    Stage::TtlFailures,
    Stage::EntryTtlNonZero,
    Stage::EntryTtlZero,
    Stage::AttrTtlNonZero,
    Stage::AttrTtlZero,
    Stage::CacheableReplies,
    Stage::ReaddirPlusQueue,
    Stage::ReaddirPlusProjection,
    Stage::IndexHits, Stage::IndexMisses, Stage::IndexFallback,
    Stage::IndexDegraded, Stage::IndexBuildUs, Stage::IndexBytes,
    Stage::IndexPartial, Stage::PermissionDenied, Stage::SpaceForbidden,
    Stage::LookupRequests, Stage::GetattrRequests, Stage::OpendirRequests,
    Stage::ReaddirRequests, Stage::ReaddirPlusRequests, Stage::IndexReadUs,
    Stage::TtlDegraded, Stage::TtlRecovered,
];
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
}
static TOTALS: [Aggregate; 52] = [const { Aggregate::new() }; 52];
static ENABLED: OnceLock<bool> = OnceLock::new();
pub(crate) fn enabled() -> bool {
    *ENABLED
        .get_or_init(|| std::env::var_os("DRIVE_NATIVE_FS_DIAGNOSTICS").is_some_and(|s| s == "1"))
}
pub struct Span {
    stage: Stage,
    start: Option<Instant>,
}
pub fn span(stage: Stage) -> Span {
    Span {
        stage,
        start: enabled().then(Instant::now),
    }
}
pub fn count(stage: Stage) {
    if enabled() {
        record(stage, 0);
    }
}
pub(crate) fn value(stage: Stage, value: u64) {
    if enabled() { record(stage, value); }
}
/// Count one metadata publication's entry/attr TTLs (counts only, no paths).
pub(crate) fn ttls(entry: Option<std::time::Duration>, attr: Option<std::time::Duration>) {
    if !enabled() {
        return;
    }
    if let Some(entry) = entry {
        count(if entry.is_zero() {
            Stage::EntryTtlZero
        } else {
            Stage::EntryTtlNonZero
        });
    }
    if let Some(attr) = attr {
        count(if attr.is_zero() {
            Stage::AttrTtlZero
        } else {
            Stage::AttrTtlNonZero
        });
    }
    if entry.is_some_and(|ttl| !ttl.is_zero()) || attr.is_some_and(|ttl| !ttl.is_zero()) {
        count(Stage::CacheableReplies);
    }
}

pub(crate) fn directory(t: &drive_core::filesystem::DirectoryBuildTimings) {
    if enabled() {
        for (stage, us) in [
            (Stage::DirectoryIdentity, t.identity_us),
            (Stage::DirectoryCount, t.count_us),
            (Stage::DirectoryInodes, t.inode_allocation_us),
            (Stage::DirectoryInodeWrites, t.inode_writes),
            (Stage::DirectoryCollect, t.collect_us),
            (Stage::DirectoryCommit, t.commit_us),
        ] {
            record(stage, us);
        }
        // A projection that wrote no inode rows commits no WAL frame, so the
        // first-scan per-directory fsync is gone. Count those commits directly.
        if t.inode_writes == 0 {
            count(Stage::DirectoryCommitNoop);
        }
    }
}
fn record(stage: Stage, us: u64) {
    let a = &TOTALS[stage as usize];
    a.count.fetch_add(1, Ordering::Relaxed);
    a.total_us.fetch_add(us, Ordering::Relaxed);
    a.max_us.fetch_max(us, Ordering::Relaxed);
    let bucket = if us <= 1 {
        0
    } else {
        (64 - (us - 1).leading_zeros() as usize).min(31)
    };
    a.buckets[bucket].fetch_add(1, Ordering::Relaxed);
}
impl Drop for Span {
    fn drop(&mut self) {
        if let Some(start) = self.start {
            record(
                self.stage,
                start.elapsed().as_micros().min(u64::MAX as u128) as u64,
            );
        }
    }
}
#[derive(Debug)]
pub struct Sample {
    pub stage: Stage,
    pub count: u64,
    pub total_us: u64,
    pub max_us: u64,
    pub buckets: [u64; 32],
}
/// A bounded aggregate snapshot; concurrent observations are approximate.
/// No endpoint or background sampler is installed automatically.
pub fn snapshot() -> Vec<Sample> {
    STAGES
        .into_iter()
        .map(|stage| {
            let a = &TOTALS[stage as usize];
            Sample {
                stage,
                count: a.count.load(Ordering::Relaxed),
                total_us: a.total_us.load(Ordering::Relaxed),
                max_us: a.max_us.load(Ordering::Relaxed),
                buckets: std::array::from_fn(|i| a.buckets[i].load(Ordering::Relaxed)),
            }
        })
        .collect()
}
