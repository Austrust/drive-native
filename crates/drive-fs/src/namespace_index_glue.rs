//! A mount-local binding between an immutable index generation and Store's
//! projection publication token. A hit holds only a short projection lease,
//! never the SQLite writer. All misses retain the authoritative .9 path.
use super::*;
use drive_core::{namespace_index::{EntrySlot, NamespaceIndex, RefreshOutcome}, shared_store::CommitState};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

pub const INDEX_ENV: &str = "DRIVE_NATIVE_FS_INDEX";
pub fn enabled() -> bool { std::env::var_os(INDEX_ENV).is_none_or(|value| value != "0") }

pub(super) struct Binding {
    pub index: Arc<NamespaceIndex>,
    token: AtomicU64,
    refresh: Mutex<()>,
    requested: AtomicBool,
    ready: AtomicBool,
}
fn word(state: CommitState) -> u64 {
    if state.revision >= (1 << 63) { u64::MAX }
    else { (state.revision << 1) | u64::from(!state.verified) }
}
impl Binding {
    pub fn new(index: Arc<NamespaceIndex>, store: &drive_core::SharedStore) -> Self {
        let binding = Self { index, token: AtomicU64::new(u64::MAX), refresh: Mutex::new(()), requested: AtomicBool::new(false), ready: AtomicBool::new(false) };
        binding.refresh(store);
        binding
    }
    fn needs_refresh(&self, store: &drive_core::SharedStore) -> bool {
        word(store.projection_state()) != self.token.load(Ordering::Acquire)
            // A valid snapshot explicitly invalidated by a hint needs refresh.
            // A transient failed builder which left an old snapshot valid also
            // needs retry. A stable budget-degraded generation does not.
            || self.ready.load(Ordering::Acquire) == self.index.is_degraded()
    }
    pub fn schedule(self: &Arc<Self>, store: Arc<drive_core::SharedStore>, queue: &LocalQueue) {
        if !self.needs_refresh(&store)
            || self.requested.swap(true, Ordering::AcqRel) { return; }
        struct Pending(Arc<Binding>);
        impl Drop for Pending { fn drop(&mut self) { self.0.requested.store(false, Ordering::Release); } }
        let pending = Pending(self.clone());
        queue.submit(Box::new(move || { pending.0.refresh(&store); drop(pending); }));
    }
    /// No Store acquisition occurs under this lease. The caller must retain it
    /// until its single FUSE reply has been sent, including test delay seams.
    pub fn lease<'a>(&self, store: &'a drive_core::SharedStore, notify: &NotifyHub)
        -> Option<drive_core::shared_store::ProjectionReadGuard<'a>> {
        if notify.delivery_failed() || !self.ready.load(Ordering::Acquire) || self.index.is_degraded() {
            diagnostics::count(diagnostics::Stage::IndexDegraded);
            return None;
        }
        let lease = store.try_projection_read()?;
        if store.has_permission_denials() { return None; }
        let state = lease.state();
        if !state.verified || word(state) != self.token.load(Ordering::Acquire) {
            diagnostics::count(diagnostics::Stage::IndexMisses);
            return None;
        }
        Some(lease)
    }
    /// Called only on initialization or a metadata worker, never on the FUSE
    /// dispatcher. Rebuilding may consume the bounded index-load budget.
    pub fn refresh(&self, store: &drive_core::SharedStore) {
        if !self.needs_refresh(store) {
            return;
        }
        let Ok(_refresh) = self.refresh.try_lock() else { return; };
        let Ok(guard) = store.try_lock() else { return; };
        let started = std::time::Instant::now();
        let outcome = self.index.ensure_fresh(&guard);
        // finish_projection captures the published token while the exclusive
        // projection lease is still held. Reading a token after plain drop
        // could bind this index to a different concurrent writer's generation.
        let state = guard.finish_projection();
        if outcome == RefreshOutcome::Refreshed {
            diagnostics::value(diagnostics::Stage::IndexBuildUs, started.elapsed().as_micros().min(u64::MAX as u128) as u64);
        }
        diagnostics::value(diagnostics::Stage::IndexBytes, self.index.bytes() as u64);
        if self.index.is_partial() { diagnostics::count(diagnostics::Stage::IndexPartial); }
        if outcome == RefreshOutcome::Degraded || !state.verified {
            diagnostics::count(diagnostics::Stage::IndexDegraded);
        }
        // A failed builder can leave an old snapshot present. Never bind it to
        // this newer Store token merely because is_degraded() is still false.
        self.ready.store(outcome != RefreshOutcome::Degraded && state.verified, Ordering::Release);
        self.token.store(word(state), Ordering::Release);
    }
}

pub(super) fn attr(slot: &EntrySlot, uid: u32, gid: u32) -> FileAttr {
    let time = UNIX_EPOCH.checked_add(Duration::from_secs(slot.mtime)).unwrap_or(UNIX_EPOCH);
    FileAttr {
        ino: slot.ino, size: slot.size, blocks: slot.blocks,
        atime: UNIX_EPOCH.checked_add(Duration::new(slot.atime.0, slot.atime.1)).unwrap_or(UNIX_EPOCH),
        mtime: time, ctime: time, crtime: UNIX_EPOCH,
        kind: if slot.directory { FileType::Directory } else { FileType::RegularFile },
        perm: slot.mode, nlink: if slot.directory { 2 } else { 1 },
        uid, gid, rdev: 0, blksize: 4096, flags: 0,
    }
}

#[cfg(test)]
#[path = "namespace_index_tests.rs"]
mod tests;
