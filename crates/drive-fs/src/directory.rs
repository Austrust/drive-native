//! Stable directory cookies. Only opendir projects SQLite; continuation and
//! release use immutable views. Warm continuations avoid Store; preview
//! continuations validate their revocable capability while sending one reply.
use super::*;
use drive_core::filesystem::DirectorySnapshot;
use drive_core::{namespace_index::{DirBlock, FileKind}, shared_store::CommitState};
use std::sync::{
    Weak,
    atomic::{AtomicUsize, Ordering},
};

const MAX_BYTES: usize = 64 * 1024 * 1024;
const MAX_HANDLES: usize = 4096;
const HOT_VIEWS: usize = 256;
const HOT_BYTES: usize = 8 * 1024 * 1024;
const HOT_VIEW_BYTES: usize = 128 * 1024;
const BUILD_ATTEMPTS: usize = 3;
/// Waits for an in-flight writer to publish its commit, then returns the
/// semantic generation under which the read-only projection can be trusted.
/// This only ever waits for the commit boundary, never for the projection: the
/// writer lock is released by its owner before SQL is built here (issue #23).
fn settle(store: &drive_core::SharedStore) -> CommitState {
    if let Some(lease) = store.try_projection_read() {
        let token = store.change_state();
        drop(lease);
        return token;
    }
    let guard = store.lock().unwrap_or_else(|poison| poison.into_inner());
    let token = store.change_state();
    drop(guard);
    token
}

enum ViewOrigin {
    Authority,
    Preview(drive_core::preview::PreviewStamp),
    Indexed { block: Option<Arc<DirBlock>>, token: CommitState, bytes: usize },
}
pub(super) struct View {
    origin: ViewOrigin,
    ino: u64,
    snapshot: Option<DirectorySnapshot>,
    reclaimer: Arc<Reclaimer>,
}
impl View {
    fn snapshot(&self) -> &DirectorySnapshot {
        self.snapshot.as_ref().unwrap()
    }
    fn indexed(&self) -> Option<(&Arc<DirBlock>, CommitState)> {
        match &self.origin {
            ViewOrigin::Indexed { block: Some(block), token, .. } => Some((block, *token)),
            _ => None,
        }
    }
}
impl Drop for View {
    fn drop(&mut self) {
        if let Some(snapshot) = self.snapshot.take() {
            self.reclaimer.release(snapshot);
        }
        if let ViewOrigin::Indexed { block, bytes, .. } = &mut self.origin {
            if let Some(block) = block.take() { self.reclaimer.release_index(block, *bytes); }
        }
    }
}
enum RetiredView {
    Sql(DirectorySnapshot),
    Index(Arc<DirBlock>, usize),
}
impl From<DirectorySnapshot> for RetiredView {
    fn from(snapshot: DirectorySnapshot) -> Self { Self::Sql(snapshot) }
}
struct Reclaimer {
    sender: Option<SyncSender<RetiredView>>,
    thread: Option<std::thread::JoinHandle<()>>,
    charged: Arc<AtomicUsize>,
    allocated: Arc<AtomicUsize>,
}
fn reclaim(snapshot: impl Into<RetiredView>, charged: &AtomicUsize, allocated: &AtomicUsize) {
    let bytes = match snapshot.into() {
        RetiredView::Sql(snapshot) => { let bytes = snapshot.bytes; drop(snapshot); bytes },
        RetiredView::Index(block, bytes) => { drop(block); bytes },
    };
    // Return capacity only after the allocations are actually released.
    charged.fetch_sub(bytes, Ordering::AcqRel);
    allocated.fetch_sub(1, Ordering::AcqRel);
}
impl Reclaimer {
    fn new(charged: Arc<AtomicUsize>, allocated: Arc<AtomicUsize>) -> Self {
        let (sender, receiver) = sync_channel(MAX_HANDLES);
        let bytes = charged.clone();
        let views = allocated.clone();
        let thread = std::thread::spawn(move || {
            while let Ok(snapshot) = receiver.recv() {
                reclaim(snapshot, &bytes, &views);
            }
        });
        Self {
            sender: Some(sender),
            thread: Some(thread),
            charged,
            allocated,
        }
    }
    fn release(&self, snapshot: DirectorySnapshot) {
        self.send(RetiredView::Sql(snapshot));
    }
    fn release_index(&self, block: Arc<DirBlock>, bytes: usize) {
        self.send(RetiredView::Index(block, bytes));
    }
    fn send(&self, snapshot: RetiredView) {
        // Each allocated view sends exactly once. Allocated (including queued)
        // views are capped at channel capacity, so a normal send cannot be full.
        // On exceptional receiver failure still release memory, never leak it.
        if let Err(error) = self.sender.as_ref().unwrap().try_send(snapshot) {
            let snapshot = match error {
                std::sync::mpsc::TrySendError::Full(snapshot)
                | std::sync::mpsc::TrySendError::Disconnected(snapshot) => snapshot,
            };
            reclaim(snapshot, &self.charged, &self.allocated);
        }
    }
}
impl Drop for Reclaimer {
    fn drop(&mut self) {
        self.sender.take();
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}
struct Table {
    next: u64,
    handles: HashMap<u64, Arc<View>>,
}
#[derive(Default)]
struct Hot {
    views: std::collections::VecDeque<(u64, CommitState, Arc<View>)>,
    bytes: usize,
    revision: Option<CommitState>,
}
pub(super) struct Views {
    table: Mutex<Table>,
    // Semantic commits conservatively invalidate all directories. Candidate
    // receipts and pure inode allocation leave reusable views alone.
    cached: Mutex<HashMap<u64, (CommitState, Weak<View>)>>,
    hot: Mutex<Hot>,
    /// In-flight projection guards, one per directory (issue #23 residual ①).
    /// Concurrent opens of one directory share the guard and therefore one
    /// projection; distinct directories never wait for each other. Only active
    /// projections are kept (`flight`/`release_flight` drop idle entries).
    building: Mutex<HashMap<u64, Arc<Mutex<()>>>>,
    charged: Arc<AtomicUsize>,
    allocated: Arc<AtomicUsize>,
    reclaimer: Arc<Reclaimer>,
}
impl Default for Views {
    fn default() -> Self {
        let charged = Arc::new(AtomicUsize::new(0));
        let allocated = Arc::new(AtomicUsize::new(0));
        let reclaimer = Arc::new(Reclaimer::new(charged.clone(), allocated.clone()));
        Self {
            table: Mutex::new(Table {
                next: 1,
                handles: HashMap::new(),
            }),
            cached: Mutex::new(HashMap::new()),
            hot: Mutex::new(Hot::default()),
            building: Mutex::new(HashMap::new()),
            charged,
            allocated,
            reclaimer,
        }
    }
}
impl Views {
    /// Shares the original handle/byte/allocation caps and reclamation queue.
    /// A failed per-directory try-lock is a normal SQL-path fallback, never
    /// dispatcher wait: only a projection of the same directory blocks it.
    fn open_index(&self, block: Arc<DirBlock>, token: CommitState) -> Option<Result<u64, i32>> {
        let ino = block.ino;
        let flight = self.flight(ino);
        let Ok(_building) = flight.try_lock() else { return None; };
        let result = self.attach_index_view(block, token);
        drop(_building);
        self.release_flight(ino, &flight);
        result
    }
    fn attach_index_view(&self, block: Arc<DirBlock>, token: CommitState) -> Option<Result<u64, i32>> {
        let mut table = self.table.lock().unwrap();
        if table.handles.len() >= MAX_HANDLES || table.next == u64::MAX {
            return Some(Err(libc::EMFILE));
        }
        // A held old-generation block remains charged until the reclaimer has
        // actually released it. Charge conservatively even when Arc is shared.
        // The estimate is precomputed by the index builder (issue #23
        // residual), so a hot open only reads one field while holding the
        // handles mutex and the projection lease.
        let bytes = block.estimated_bytes;
        if bytes > MAX_BYTES.saturating_sub(self.charged.load(Ordering::Acquire))
            || self.allocated.load(Ordering::Acquire) >= MAX_HANDLES {
            return Some(Err(libc::ENOMEM));
        }
        self.charged.fetch_add(bytes, Ordering::AcqRel);
        self.allocated.fetch_add(1, Ordering::AcqRel);
        let ino = block.ino;
        let view = Arc::new(View { origin: ViewOrigin::Indexed { block: Some(block), token, bytes },
            ino, snapshot: None, reclaimer: self.reclaimer.clone() });
        let fh = table.next;
        table.next += 1;
        table.handles.insert(fh, view);
        Some(Ok(fh))
    }
    // Closed small views can be reclaimed synchronously on a metadata worker.
    // Held/in-flight views keep their original immutable cookies and charge.
    fn evict(view: Arc<View>) {
        if let Ok(mut view) = Arc::try_unwrap(view) {
            let snapshot = view.snapshot.take().unwrap();
            reclaim(snapshot, &view.reclaimer.charged, &view.reclaimer.allocated);
        }
    }
    fn trim_hot(&self, revision: Option<&CommitState>) {
        let removed = {
            let mut hot = self.hot.lock().unwrap_or_else(|poison| poison.into_inner());
            if revision.is_some() && hot.revision.as_ref() == revision {
                return;
            }
            hot.revision = revision.copied();
            let mut removed = Vec::new();
            let mut i = 0;
            while i < hot.views.len() {
                if revision.is_none_or(|r| r != &hot.views[i].1) {
                    let (_, _, view) = hot.views.remove(i).unwrap();
                    hot.bytes -= view.snapshot().bytes;
                    removed.push(view);
                } else {
                    i += 1;
                }
            }
            removed
        };
        for view in removed {
            Self::evict(view);
        }
    }
    fn retain_hot(&self, ino: u64, revision: CommitState, view: Arc<View>) {
        if view.snapshot().bytes > HOT_VIEW_BYTES {
            return;
        }
        let removed = {
            let mut hot = self.hot.lock().unwrap();
            let mut removed = Vec::new();
            if let Some(index) = hot.views.iter().position(|(id, _, _)| *id == ino) {
                let (_, _, old) = hot.views.remove(index).unwrap();
                hot.bytes -= old.snapshot().bytes;
                removed.push(old);
            }
            hot.bytes += view.snapshot().bytes;
            hot.views.push_back((ino, revision, view));
            while hot.views.len() > HOT_VIEWS || hot.bytes > HOT_BYTES {
                let (_, _, old) = hot.views.pop_front().unwrap();
                hot.bytes -= old.snapshot().bytes;
                removed.push(old);
            }
            removed
        };
        for view in removed {
            Self::evict(view);
        }
    }
    /// Projects a directory without ever running SQL under the writer guard.
    /// The guard is only observed for a bounded moment so a rename committed by
    /// an in-flight writer is not hidden by a cached view; a held lock without
    /// a commit must not stall the projection (the #23 regression).
    ///
    /// Single-flight is per directory (issue #23 residual ①): concurrent opens
    /// of the *same* directory share one read-only projection, while distinct
    /// directories project in parallel instead of serializing on a per-mount
    /// builder mutex.
    fn open(&self, store: &drive_core::SharedStore, ino: u64) -> Result<u64, i32> {
        {
            let table = self.table.lock().unwrap_or_else(|poison| poison.into_inner());
            if table.handles.len() >= MAX_HANDLES || table.next == u64::MAX {
                return Err(libc::EMFILE);
            }
        }
        if let Some(view) = self.cached_handle(store, ino) {
            diagnostics::count(diagnostics::Stage::DirectoryCacheHit);
            return self.attach(view);
        }
        diagnostics::count(diagnostics::Stage::DirectoryCacheMiss);
        let wait = diagnostics::span(diagnostics::Stage::DirectoryBuilderWait);
        let flight = self.flight(ino);
        let building = flight.lock().unwrap_or_else(|poison| poison.into_inner());
        drop(wait);
        // A concurrent open of this directory may have published while this
        // one waited for the flight guard: share that projection instead of
        // projecting the same directory twice.
        if let Some(view) = self.cached_handle(store, ino) {
            diagnostics::count(diagnostics::Stage::DirectoryCacheHit);
            let handle = self.attach(view);
            drop(building);
            self.release_flight(ino, &flight);
            return handle;
        }
        let mut attempt = 0;
        loop {
            if self.allocated.load(Ordering::Acquire) >= MAX_HANDLES {
                return Err(libc::ENOMEM);
            }
            if self.charged.load(Ordering::Acquire) > MAX_BYTES - HOT_BYTES {
                self.trim_hot(None);
            }
            let token = store.change_state();
            if !token.verified {
                break;
            }
            let budget = MAX_BYTES.saturating_sub(self.charged.load(Ordering::Acquire));
            #[cfg(test)]
            test_hooks::before_projection(ino);
            let projection = diagnostics::span(diagnostics::Stage::DirectoryProjection);
            let built = Self::build_readonly(store, ino, budget);
            drop(projection);
            match built {
                Ok(Some((snapshot, origin))) => {
                    attempt += 1;
                    let after = store.change_state();
                    if after != token || !after.verified {
                        if attempt < BUILD_ATTEMPTS {
                            continue;
                        }
                        break;
                    }
                    let bytes = snapshot.bytes;
                    if bytes > MAX_BYTES.saturating_sub(self.charged.load(Ordering::Acquire)) {
                        if self.hot_nonempty() && attempt < BUILD_ATTEMPTS {
                            self.trim_hot(None);
                            continue;
                        }
                        return Err(libc::ENOMEM);
                    }
                    self.charged.fetch_add(bytes, Ordering::AcqRel);
                    self.allocated.fetch_add(1, Ordering::AcqRel);
                    let view = Arc::new(View {
                        origin,
                        ino,
                        snapshot: Some(snapshot),
                        reclaimer: self.reclaimer.clone(),
                    });
                    self.publish(ino, after, view.clone());
                    let handle = self.attach(view);
                    drop(building);
                    self.release_flight(ino, &flight);
                    return handle;
                }
                Ok(None) => break,
                Err(number) => {
                    attempt += 1;
                    if number == libc::ENOMEM
                        && self.hot_nonempty()
                        && attempt < BUILD_ATTEMPTS
                    {
                        self.trim_hot(None);
                        continue;
                    }
                    return Err(number);
                }
            }
        }
        drop(building);
        self.release_flight(ino, &flight);
        let (handle, view) = self.open_locked(store, ino)?;
        if let Some(view) = view {
            let token = store.change_state();
            if token.verified {
                self.publish(ino, token, view);
            }
        }
        Ok(handle)
    }
    /// Cheap-view cache probe for the committed semantic generation. Shared by
    /// the pre-flight fast path and the post-single-flight re-check so a
    /// waiter reuses the projection published while it waited.
    fn cached_handle(&self, store: &drive_core::SharedStore, ino: u64) -> Option<Arc<View>> {
        let token = settle(store);
        if !token.verified {
            return None;
        }
        self.trim_hot(Some(&token));
        let view = {
            let cached = self.cached.lock().unwrap_or_else(|poison| poison.into_inner());
            cached.get(&ino).and_then(|(revision, view)| {
                (revision == &token).then(|| view.upgrade()).flatten()
            })?
        };
        self.retain_hot(ino, token, view.clone());
        Some(view)
    }
    /// In-flight projection guard for one directory. Idle entries are dropped
    /// as they are observed, so the map tracks active projections rather than
    /// every directory ever opened.
    fn flight(&self, ino: u64) -> Arc<Mutex<()>> {
        let mut building = self.building.lock().unwrap_or_else(|poison| poison.into_inner());
        building.retain(|_, flight| Arc::strong_count(flight) > 1);
        if let Some(flight) = building.get(&ino) {
            return flight.clone();
        }
        let flight = Arc::new(Mutex::new(()));
        building.insert(ino, flight.clone());
        flight
    }
    /// Drop the idle single-flight entry when this caller was its last user.
    fn release_flight(&self, ino: u64, flight: &Arc<Mutex<()>>) {
        let mut building = self.building.lock().unwrap_or_else(|poison| poison.into_inner());
        if Arc::strong_count(flight) == 2 {
            building.remove(&ino);
        }
    }
    /// Publishes an authoritative view for reuse. The token is the semantic
    /// generation the projection was built under, not a local `total_changes`
    /// counter, so inode allocation alone never invalidates it.
    fn publish(&self, ino: u64, token: CommitState, view: Arc<View>) {
        {
            let mut cached = self.cached.lock().unwrap_or_else(|poison| poison.into_inner());
            cached.retain(|_, (_, view)| view.strong_count() != 0);
            cached.insert(ino, (token, Arc::downgrade(&view)));
        }
        self.retain_hot(ino, token, view);
    }
    /// One read-only projection attempt. `Ok(None)` asks the caller to fall back
    /// to the locked writer path (preview mounts, unmapped inodes, read-only
    /// open failure); errors are the errno the locked projection would return.
    fn build_readonly(
        store: &drive_core::SharedStore,
        ino: u64,
        budget: usize,
    ) -> Result<Option<(DirectorySnapshot, ViewOrigin)>, i32> {
        let root = store.root();
        let Ok(readonly) = drive_core::Store::open_read_only(&root) else {
            return Ok(None);
        };
        match readonly.preview_record_by_inode(ino) {
            Ok(_) => return Ok(None),
            Err(error) if error.is_not_found() => {}
            Err(_) => return Ok(None),
        }
        if permissions::inode(&readonly, ino, permissions::Action::List).is_err() {
            return Ok(None);
        }
        match readonly.directory_snapshot_read(ino, budget) {
            Ok(Some(snapshot)) => Ok(Some((snapshot, ViewOrigin::Authority))),
            Ok(None) => Ok(None),
            Err(error) => Err(errno(&error)),
        }
    }
    /// Locked fallback for projections that need the writer: preview mounts,
    /// unconfirmed boundaries and namespaces whose inodes are not mapped yet.
    /// Mirrors the pre-#23 behavior and never publishes into the cheap cache.
    fn open_locked(
        &self,
        store: &drive_core::SharedStore,
        ino: u64,
    ) -> Result<(u64, Option<Arc<View>>), i32> {
        let wait = diagnostics::span(diagnostics::Stage::DirectoryBuilderWait);
        let flight = self.flight(ino);
        let _building = flight.lock().unwrap_or_else(|poison| poison.into_inner());
        drop(wait);
        let store = store.lock().unwrap_or_else(|poison| poison.into_inner());
        let authority = match store.preview_record_by_inode(ino) {
            Ok(_) => false,
            Err(error) if error.is_not_found() => true,
            Err(error) => return Err(errno(&error)),
        };
        if authority {
            permissions::inode(&store, ino, permissions::Action::List)
                .map_err(|error| errno(&error))?;
        }
        let _revision = store.directory_view_stamp().map_err(|error| errno(&error))?;
        if self.allocated.load(Ordering::Acquire) >= MAX_HANDLES
            || self.charged.load(Ordering::Acquire) > MAX_BYTES - HOT_BYTES
        {
            self.trim_hot(None);
        }
        if self.allocated.load(Ordering::Acquire) >= MAX_HANDLES {
            return Err(libc::ENOMEM);
        }
        let remaining = MAX_BYTES.saturating_sub(self.charged.load(Ordering::Acquire));
        let projection = diagnostics::span(diagnostics::Stage::DirectoryProjection);
        let project = |budget| -> Result<_, i32> {
            if authority {
                let snapshot = if diagnostics::enabled() {
                    let (snapshot, timings) = store
                        .directory_snapshot_measured(ino, budget)
                        .map_err(|error| errno(&error))?;
                    diagnostics::directory(&timings);
                    snapshot
                } else {
                    store.directory_snapshot(ino, budget).map_err(|error| errno(&error))?
                };
                Ok((snapshot, ViewOrigin::Authority))
            } else {
                let view = store
                    .preview_directory_snapshot(ino, budget)
                    .map_err(|error| errno(&error))?;
                Ok((view.snapshot, ViewOrigin::Preview(view.stamp)))
            }
        };
        let (snapshot, origin) = match project(remaining) {
            Err(libc::ENOMEM) if self.hot_nonempty() => {
                self.trim_hot(None);
                project(MAX_BYTES.saturating_sub(self.charged.load(Ordering::Acquire)))?
            }
            result => result?,
        };
        drop(projection);
        self.charged.fetch_add(snapshot.bytes, Ordering::AcqRel);
        self.allocated.fetch_add(1, Ordering::AcqRel);
        let view = Arc::new(View {
            origin,
            ino,
            snapshot: Some(snapshot),
            reclaimer: self.reclaimer.clone(),
        });
        let handle = self.attach(view.clone())?;
        Ok((handle, authority.then_some(view)))
    }
    fn hot_nonempty(&self) -> bool {
        !self
            .hot
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
            .views
            .is_empty()
    }
    fn attach(&self, view: Arc<View>) -> Result<u64, i32> {
        let mut table = self.table.lock().unwrap_or_else(|poison| poison.into_inner());
        if table.handles.len() >= MAX_HANDLES || table.next == u64::MAX {
            return Err(libc::EMFILE);
        }
        let handle = table.next;
        table.next += 1;
        table.handles.insert(handle, view);
        Ok(handle)
    }
    fn get(&self, ino: u64, fh: u64) -> Result<Arc<View>, i32> {
        self.table
            .lock()
            .unwrap()
            .handles
            .get(&fh)
            .filter(|v| v.ino == ino)
            .cloned()
            .ok_or(libc::EBADF)
    }
    fn release(&self, ino: u64, fh: u64) -> Result<(), i32> {
        let removed = {
            let mut table = self.table.lock().unwrap();
            if !table.handles.get(&fh).is_some_and(|v| v.ino == ino) {
                return Err(libc::EBADF);
            }
            table.handles.remove(&fh)
        };
        drop(removed);
        Ok(())
    }
}

pub(super) fn open(
    fs: &mut CacheFilesystem,
    ino: u64,
    flags: i32,
    reply: ReplyOpen,
    deadline: request::RequestDeadline,
) {
    diagnostics::count(diagnostics::Stage::OpendirRequests);
    if flags & libc::O_ACCMODE != libc::O_RDONLY {
        reply.error(libc::EISDIR);
        return;
    }
    if let Some(binding) = &fs.namespace_index {
        let _index_time = diagnostics::span(diagnostics::Stage::IndexReadUs);
        if let Some(lease) = binding.lease(&fs.store, &fs.notify) {
            if let Some(block) = binding.index.dir_block(ino) {
                if let Some(result) = fs.directories.open_index(block, lease.state()) {
                    match result {
                        Ok(fh) => { diagnostics::count(diagnostics::Stage::IndexHits); reply.opened(fh, 0); },
                        Err(e) => reply.error(e),
                    }
                    drop(lease);
                    return;
                }
            }
        }
        diagnostics::count(diagnostics::Stage::IndexFallback);
    }
    fs.refresh_index_later();
    let views = fs.directories.clone();
    let store = fs.store.clone();
    let queued = diagnostics::span(diagnostics::Stage::OpendirQueue);
    fs.queue.submit(Box::new(move || {
        let _deadline = deadline;
        drop(queued);
        match views.open(&store, ino) {
            Ok(fh) => reply.opened(fh, 0),
            Err(e) => reply.error(e),
        }
    }));
}
/// Attribute chunks are computed until the reply buffer is full; 128 keeps
/// wasted rows bounded while amortizing the per-chunk Store statement.
const PLUS_PAGE: usize = 128;

/// Issue #50: true when `name` is excluded from the directory `parent`.
///
/// Cookie positions are intentionally left untouched when an entry is skipped:
/// the kernel requires cookies to stay stable across continuation pages, and
/// gaps are legal for READDIR/READDIRPLUS.
fn excluded(
    index: Option<&drive_core::exclude::ExclusionIndex>,
    parent: u64,
    name: &str,
) -> bool {
    index.is_some_and(|index| index.entry_excluded(parent, name))
}


/// READDIRPLUS: names/cookies come from the immutable directory view, bounded
/// attribute pages from one Store statement. Attributes use the same policy as
/// GETATTR (attr changes are invalidated with INVAL_INODE); positive entry TTLs
/// are suppressed while the directory was touched by a recent change, which
/// compresses the readdirplus late-install window. Preview views publish
/// zero-TTL placeholder entries only.
pub(super) fn read_plus(
    fs: &mut CacheFilesystem,
    ino: u64,
    fh: u64,
    offset: i64,
    mut reply: fuser::ReplyDirectoryPlus,
    deadline: request::RequestDeadline,
) {
    diagnostics::count(diagnostics::Stage::ReaddirPlusRequests);
    let Ok(offset) = usize::try_from(offset) else {
        reply.error(libc::EINVAL);
        return;
    };
    // Issue #50: snapshot the index once; matching never takes the Store lock.
    let exclusion = fs.exclusion_index();
    if let (Some(binding), Ok(view)) = (&fs.namespace_index, fs.directories.get(ino, fh)) {
        if let Some((block, opened)) = view.indexed() {
            let _index_time = diagnostics::span(diagnostics::Stage::IndexReadUs);
            if let Some(lease) = binding.lease(&fs.store, &fs.notify) {
                if lease.state() == opened {
                    if offset > block.entries.len() { reply.error(libc::EINVAL); return; }
                    let publication = fs.notify.reply_guard();
                    for (position, entry) in block.entries.iter().enumerate().skip(offset) {
                        let cookie = (position + 1) as i64;
                        if excluded(exclusion.as_deref(), ino, &entry.name) {
                            continue;
                        }
                        let full = match binding.index.entry_by_ino(entry.ino) {
                            Some(slot) => {
                                let (_, attr_ttl) = preview::reply_ttls(&fs.notify, true, true);
                                diagnostics::ttls(Some(ENTRY_TTL), Some(attr_ttl));
                                reply.add_with_timeouts(entry.ino, cookie, entry.name.as_ref(),
                                    &ENTRY_TTL, &attr_ttl, &namespace_index_glue::attr(&slot, fs.uid, fs.gid), 0)
                            }
                            None => {
                                // Dirty/missing attributes are not a negative
                                // namespace answer. The kernel re-looks up this
                                // dirent through the ordinary authority path.
                                diagnostics::count(diagnostics::Stage::IndexMisses);
                                reply.add_without_lookup(entry.ino, cookie, entry.name.as_ref(), kind(entry.kind))
                            }
                        };
                        if full { break; }
                    }
                    diagnostics::count(diagnostics::Stage::IndexHits);
                    reply.ok();
                    drop(publication);
                    drop(lease);
                    return;
                }
            }
            diagnostics::count(diagnostics::Stage::IndexFallback);
        }
    }
    fs.refresh_index_later();
    let views = fs.directories.clone();
    let store = fs.store.clone();
    let notify = fs.notify.clone();
    let (uid, gid) = (fs.uid, fs.gid);
    let exclusion = exclusion.clone();
    let queued = diagnostics::span(diagnostics::Stage::ReaddirPlusQueue);
    fs.read_queue.submit(Box::new(move || {
        let _deadline = deadline;
        drop(queued);
        let view = match views.get(ino, fh) {
            Ok(view) => view,
            Err(e) => {
                reply.error(e);
                return;
            }
        };
        let publication = notify.reply_guard();
        let permission_lease=if matches!(&view.origin,ViewOrigin::Preview(_)){None}else{store.permission_clear_lease()};
        if store.has_permission_denials() {
            send_acl_plus(&store,&notify,&view,offset,uid,gid,exclusion.as_deref(),reply);
            return;
        }
        if let Some((block, _)) = view.indexed() {
            send_index_plus_sql(&store, &notify, block, offset, uid, gid, exclusion.as_deref(), reply);
            drop(publication);
            return;
        }
        let entries = &view.snapshot().entries;
        if offset > entries.len() {
            reply.error(libc::EINVAL);
            return;
        }
        match &view.origin {
            ViewOrigin::Authority => {
                let mut index = offset;
                while index < entries.len() {
                    let end = (index + PLUS_PAGE).min(entries.len());
                    let slots = entries[index..end]
                        .iter()
                        .map(|entry| drive_core::filesystem::DirectorySlot {
                            ino: entry.ino,
                            name: &entry.name,
                            directory: entry.directory,
                        })
                        .collect::<Vec<_>>();
                    let projection = diagnostics::span(diagnostics::Stage::ReaddirPlusProjection);
                    // Never wait for the Store on a directory continuation: an
                    // in-flight commit/indexing can hold it for seconds while the
                    // kernel expects the stream to keep flowing (the
                    // continuation-and-seek contract). When it is busy we publish
                    // the immutable view's names/cookies with zero-TTL
                    // deny-by-default placeholders; the kernel re-looks them up
                    // through the ordinary metadata path.
                    let records = match store.try_lock() {
                        Ok(guard) => match guard.directory_posix_page(ino, &slots) {
                            Ok(records) => Some(records),
                            Err(e) => {
                                drop(projection);
                                reply.error(errno(&e));
                                drop(publication);
                                return;
                            }
                        },
                        Err(std::sync::TryLockError::WouldBlock) => None,
                        Err(std::sync::TryLockError::Poisoned(_)) => {
                            drop(projection);
                            reply.error(libc::EIO);
                            drop(publication);
                            return;
                        }
                    };
                    drop(projection);
                    let stable = records.is_some() && !notify.directory_unstable(ino);
                    let mut full = false;
                    for (position, entry) in entries[index..end].iter().enumerate() {
                        if excluded(exclusion.as_deref(), ino, &entry.name) {
                            continue;
                        }
                        let record = records
                            .as_ref()
                            .and_then(|records| records[position].as_ref());
                        let cookie = (index + position + 1) as i64;
                        let kind = if entry.directory {
                            FileType::Directory
                        } else {
                            FileType::RegularFile
                        };
                        let Some(record) = record else {
                            // Busy Store (or a stale slot): emit a dirent-only
                            // entry, which the kernel treats as a plain readdir
                            // item and never caches. The stream keeps flowing and
                            // the next stat re-looks the name up.
                            if reply.add_without_lookup(entry.ino, cookie, &entry.name, kind) {
                                full = true;
                                break;
                            }
                            continue;
                        };
                        let attr = preview::posix_attr(record, uid, gid);
                        let (mut entry_ttl, attr_ttl) =
                            preview::reply_ttls(&notify, record.cacheable, true);
                        if !stable {
                            entry_ttl = Duration::ZERO;
                        }
                        diagnostics::ttls(Some(entry_ttl), Some(attr_ttl));
                        if reply.add_with_timeouts(
                            entry.ino,
                            cookie,
                            &entry.name,
                            &entry_ttl,
                            &attr_ttl,
                            &attr,
                            0,
                        ) {
                            full = true;
                            break;
                        }
                    }
                    if full {
                        break;
                    }
                    index = end;
                }
            }
            ViewOrigin::Preview(stamp) => {
                // Candidate names are revocable metadata. Even a dirent-only
                // reply must validate its stamp under the same Store guard
                // through send; the authority busy-Store fallback is unsafe here.
                preview::publish_view(&store, stamp, |valid| {
                    if let Err(e) = valid {
                        reply.error(errno(&e));
                        return;
                    }
                    for (position, entry) in entries[offset..].iter().enumerate() {
                        if excluded(exclusion.as_deref(), ino, &entry.name) {
                            continue;
                        }
                        let kind = if entry.directory {
                            FileType::Directory
                        } else {
                            FileType::RegularFile
                        };
                        if reply.add_without_lookup(
                            entry.ino,
                            (offset + position + 1) as i64,
                            &entry.name,
                            kind,
                        ) {
                            break;
                        }
                    }
                    reply.ok();
                });
                drop(publication);
                return;
            }
            ViewOrigin::Indexed { .. } => unreachable!("indexed continuation handled above"),
        }
        reply.ok();
        drop(permission_lease);
        drop(publication);
    }));
}

pub(super) fn read(
    fs: &mut CacheFilesystem,
    ino: u64,
    fh: u64,
    offset: i64,
    reply: ReplyDirectory,
    deadline: request::RequestDeadline,
) {
    diagnostics::count(diagnostics::Stage::ReaddirRequests);
    let Ok(offset) = usize::try_from(offset) else {
        reply.error(libc::EINVAL);
        return;
    };
    // Issue #50: snapshot the index once; matching never takes the Store lock.
    let exclusion = fs.exclusion_index();
    if let (Some(binding), Ok(view)) = (&fs.namespace_index, fs.directories.get(ino, fh)) {
        if let Some((block, opened)) = view.indexed() {
            let _index_time = diagnostics::span(diagnostics::Stage::IndexReadUs);
            if let Some(lease) = binding.lease(&fs.store, &fs.notify) {
                if lease.state() == opened {
                    diagnostics::count(diagnostics::Stage::IndexHits);
                    send_index_page(block, offset, exclusion.as_deref(), reply);
                    drop(lease);
                    return;
                }
            }
            diagnostics::count(diagnostics::Stage::IndexFallback);
        }
    }
    fs.refresh_index_later();
    let views = fs.directories.clone();
    let store = fs.store.clone();
    let exclusion = exclusion.clone();
    let queued = diagnostics::span(diagnostics::Stage::ReaddirQueue);
    fs.read_queue.submit(Box::new(move || {
        let _deadline = deadline;
        drop(queued);
        match views.get(ino, fh) {
            Ok(view) => {
                let permission_lease=if matches!(&view.origin,ViewOrigin::Preview(_)){None}else{store.permission_clear_lease()};
                if store.has_permission_denials() {
                    let guard=match store.try_lock(){Ok(g)=>g,Err(_)=>{reply.error(libc::EAGAIN);return;}};
                    if let Err(e)=permissions::inode(&guard,ino,permissions::Action::List){reply.error(errno(&e));return;}
                    if let ViewOrigin::Preview(stamp)=&view.origin {if let Err(e)=guard.preview_validate_view(stamp){reply.error(errno(&e));return;}}
                    if let Some((block,_))=view.indexed(){send_index_page(block,offset,exclusion.as_deref(),reply);}else{send_page(&view,offset,exclusion.as_deref(),reply);}
                    drop(guard);return;
                }
                match &view.origin {
                    ViewOrigin::Authority => send_page(&view, offset, exclusion.as_deref(), reply),
                    ViewOrigin::Preview(stamp) => {
                        let guard = store.lock().unwrap();
                        match guard.preview_validate_view(stamp) {
                            Ok(()) => send_page(&view, offset, exclusion.as_deref(), reply),
                            Err(e) => reply.error(errno(&e)),
                        }
                        // Keep the same guard through the single bounded reply.
                        drop(guard);
                    }
                    ViewOrigin::Indexed { block: Some(block), .. } => {
                        // Revalidate the directory when available; immutable
                        // names/cookies retain .9's opened-view semantics. New
                        // lookup/open calls still revalidate moved/deleted names.
                        let guard = match store.try_lock() {
                            Ok(guard) => guard,
                            // The authority handle already owns these names.
                            // Match .9's nonblocking continuation contract;
                            // subsequent lookup/open still revalidates identity.
                            Err(std::sync::TryLockError::WouldBlock) => {
                                send_index_page(block, offset, exclusion.as_deref(), reply);
                                return;
                            }
                            Err(std::sync::TryLockError::Poisoned(_)) => { reply.error(libc::EIO); return; }
                        };
                        match guard.directory_identity(ino) {
                            Ok(_) => send_index_page(block, offset, exclusion.as_deref(), reply),
                            Err(e) => reply.error(errno(&e)),
                        }
                        drop(guard);
                    }
                    ViewOrigin::Indexed { block: None, .. } => reply.error(libc::EBADF),
                }
                drop(permission_lease);
            }
            Err(e) => reply.error(e),
        }
    }));
}
fn kind(kind: FileKind) -> FileType {
    match kind { FileKind::Directory => FileType::Directory, FileKind::File => FileType::RegularFile }
}
/// Borrowed READDIRPLUS origin for the ACL path (issue #26 residual). Both
/// origins are paged directly instead of being flattened into an owned list
/// first, so a continuation only materializes the slots of its current batch.
enum PlusOrigin<'a> {
    Indexed(&'a DirBlock),
    Snapshot(&'a DirectorySnapshot),
}
impl<'a> PlusOrigin<'a> {
    fn new(view: &'a View) -> Self {
        match view.indexed() {
            Some((block, _)) => Self::Indexed(block),
            None => Self::Snapshot(view.snapshot()),
        }
    }
    fn len(&self) -> usize {
        match self {
            Self::Indexed(block) => block.entries.len(),
            Self::Snapshot(snapshot) => snapshot.entries.len(),
        }
    }
    /// One borrowed slot of the current page batch.
    fn slot(&self, position: usize) -> drive_core::filesystem::DirectorySlot<'_> {
        match self {
            Self::Indexed(block) => {
                let entry = &block.entries[position];
                drive_core::filesystem::DirectorySlot {
                    ino: entry.ino,
                    name: entry.name.as_ref(),
                    directory: entry.kind == FileKind::Directory,
                }
            }
            Self::Snapshot(snapshot) => {
                let entry = &snapshot.entries[position];
                drive_core::filesystem::DirectorySlot {
                    ino: entry.ino,
                    name: &entry.name,
                    directory: entry.directory,
                }
            }
        }
    }
    fn kind(&self, position: usize) -> FileType {
        match self {
            Self::Indexed(block) => kind(block.entries[position].kind),
            Self::Snapshot(snapshot) => {
                if snapshot.entries[position].directory {
                    FileType::Directory
                } else {
                    FileType::RegularFile
                }
            }
        }
    }
}
/// One bounded ACL page batch. `PLUS_PAGE` bounds the slots materialized per
/// Store statement; the reply may span several batches when the kernel buffer
/// is large, and stops as soon as it is full.
fn acl_page_range(index: usize, len: usize) -> std::ops::Range<usize> {
    index..(index + PLUS_PAGE).min(len)
}

// ACL publication holds Store through the bounded reply. Known revocations
// cannot use the immutable-name busy-Store fallback of ordinary scans. Pages
// are projected in bounded batches instead of one SQL round-trip per entry, so
// a 2000-entry directory keeps the Store guard for a handful of page queries
// rather than ~50k per-entry statements (#26).
fn send_acl_plus(store:&drive_core::SharedStore,notify:&NotifyHub,view:&View,offset:usize,uid:u32,gid:u32,exclusion:Option<&drive_core::exclude::ExclusionIndex>,mut reply:ReplyDirectoryPlus) {
    let guard=match store.try_lock(){Ok(g)=>g,Err(_)=>{reply.error(libc::EAGAIN);return;}};
    if let Err(e)=permissions::inode(&guard,view.ino,permissions::Action::List){reply.error(errno(&e));return;}
    if let ViewOrigin::Preview(stamp)=&view.origin {if let Err(e)=guard.preview_validate_view(stamp){reply.error(errno(&e));return;}}
    // Issue #26 residual: page over the borrowed origin. Only the current
    // batch's slots are materialized; `offset`/EOF are decided before any page
    // work. The directory List permission, preview stamp, publication guard and
    // page-level ACL projection above/below are unchanged.
    let origin = PlusOrigin::new(view);
    let len = origin.len();
    if offset>len{reply.error(libc::EINVAL);return;}
    let parent=match guard.posix_record_by_inode(view.ino){Ok(record)=>record.id,Err(error)=>{reply.error(errno(&error));return;}};
    let cache_before=guard.permission_cache_stats();
    let mut index=offset;
    while index<len {
        let range=acl_page_range(index,len);
        let end=range.end;
        let slots=(range.start..range.end).map(|position|origin.slot(position)).collect::<Vec<_>>();
        // Issue #26 accounting: the bounded page batch is all this reply
        // materializes before the ACL projection (pre-fix: every entry).
        diagnostics::record_acl_materialized_reply(slots.len() as u64);
        diagnostics::record_acl_batch(slots.len() as u64);
        let page=match guard.directory_posix_page_for_acl(view.ino,&slots){
            Ok(page)=>page,Err(error)=>{reply.error(errno(&error));return;}
        };
        let mut records:Vec<drive_core::filesystem::PosixRecord>=Vec::new();
        let mut slot_records:Vec<Option<usize>>=vec![None;slots.len()];
        let mut dots:Vec<Option<drive_core::filesystem::PosixRecord>>=vec![None;slots.len()];
        for (position,slot) in slots.iter().enumerate() {
            let Some(record)=page[position].clone() else {continue;};
            if slot.name=="."||slot.name==".." {dots[position]=Some(record);}
            else {slot_records[position]=Some(records.len());records.push(record);}
        }
        let mut batched=true;
        let allowed=match drive_core::write_permissions::project_read_page(&guard,&parent,&mut records) {
            Ok(allowed)=>allowed,
            Err(_)=>{
                // Exact fallback keeps the old per-entry contract on any page
                // shape the batch path does not cover (e.g. overlay ancestry).
                batched=false;
                let mut allowed=Vec::with_capacity(records.len());
                for record in records.iter_mut() {allowed.push(permissions::project(&guard,record).is_ok());}
                allowed
            }
        };
        if batched {
            diagnostics::add_count(diagnostics::Stage::ReaddirPlusBatchedPages,1);
            diagnostics::add_count(diagnostics::Stage::ReaddirPlusBatchedEntries,records.len() as u64);
        }
        let mut full=false;
        for position in 0..slots.len() {
            let slot=&slots[position];
            let (ino,name)=(slot.ino,slot.name);
            if excluded(exclusion, view.ino, name) { continue; }
            let cookie=(index+position+1) as i64;
            let record=if let Some(dot)=dots[position].take() {
                let mut dot=dot;
                match permissions::project(&guard,&mut dot) {Ok(())=>Some(dot),Err(_)=>None}
            } else if let Some(next)=slot_records[position] {
                if allowed.get(next).copied().unwrap_or(false) {Some(records[next].clone())} else {None}
            } else {None};
            let full_entry=match record {
                Some(record)=>{
                    let(_,ttl)=preview::reply_ttls(notify,record.cacheable,true);
                    reply.add_with_timeouts(ino,cookie,name,&ENTRY_TTL,&ttl,&preview::posix_attr(&record,uid,gid),0)
                }
                None=>reply.add_without_lookup(ino,cookie,name,origin.kind(index+position)),
            };
            if full_entry {full=true;break;}
        }
        if full {break;}
        index=end;
    }
    let cache_after=guard.permission_cache_stats();
    diagnostics::add_count(diagnostics::Stage::PermissionCacheHits,cache_after.hits.saturating_sub(cache_before.hits));
    diagnostics::add_count(diagnostics::Stage::PermissionCacheMisses,cache_after.misses.saturating_sub(cache_before.misses));
    diagnostics::acl_summary(&guard);
    reply.ok();drop(guard);
}
fn send_index_page(block: &DirBlock, offset: usize, exclusion: Option<&drive_core::exclude::ExclusionIndex>, mut reply: ReplyDirectory) {
    let _span = diagnostics::span(diagnostics::Stage::ReaddirReply);
    if offset > block.entries.len() { reply.error(libc::EINVAL); return; }
    for (position, entry) in block.entries.iter().enumerate().skip(offset) {
        if excluded(exclusion, block.ino, &entry.name) { continue; }
        if reply.add(entry.ino, (position + 1) as i64, kind(entry.kind), entry.name.as_ref()) { break; }
    }
    reply.ok();
}
fn send_index_plus_sql(store: &drive_core::SharedStore, notify: &NotifyHub,
    block: &DirBlock, offset: usize, uid: u32, gid: u32,
    exclusion: Option<&drive_core::exclude::ExclusionIndex>, mut reply: ReplyDirectoryPlus) {
    if offset > block.entries.len() { reply.error(libc::EINVAL); return; }
    let guard = match store.try_lock() {
        Ok(guard) => guard,
        Err(std::sync::TryLockError::WouldBlock) => {
            // No old attribute is ever published. dirent-only PLUS preserves
            // .9 continuation/seek behavior while the authority writer is busy.
            for (position, entry) in block.entries.iter().enumerate().skip(offset) {
                if excluded(exclusion, block.ino, &entry.name) { continue; }
                if reply.add_without_lookup(entry.ino, (position + 1) as i64,
                    entry.name.as_ref(), kind(entry.kind)) { break; }
            }
            reply.ok();
            return;
        }
        Err(std::sync::TryLockError::Poisoned(_)) => { reply.error(libc::EIO); return; }
    };
    // Includes EOF: revocation cannot be bypassed with an empty page.
    match guard.directory_identity(block.ino) {
        Ok(_) => (),
        Err(e) => { reply.error(errno(&e)); return; },
    };
    let mut index = offset;
    while index < block.entries.len() {
        let end = (index + PLUS_PAGE).min(block.entries.len());
        let slots = block.entries[index..end].iter().map(|entry| drive_core::filesystem::DirectorySlot {
            ino: entry.ino, name: entry.name.as_ref(), directory: entry.kind == FileKind::Directory,
        }).collect::<Vec<_>>();
        let records = match guard.directory_posix_page(block.ino, &slots) {
            Ok(records) => records,
            Err(e) => { reply.error(errno(&e)); return; },
        };
        for (position, entry) in block.entries[index..end].iter().enumerate() {
            if excluded(exclusion, block.ino, &entry.name) { continue; }
            let cookie = (index + position + 1) as i64;
            let full = if let Some(record) = &records[position] {
                let (_, attr_ttl) = preview::reply_ttls(notify, record.cacheable, true);
                diagnostics::ttls(Some(ENTRY_TTL), Some(attr_ttl));
                reply.add_with_timeouts(entry.ino, cookie, entry.name.as_ref(), &ENTRY_TTL,
                    &attr_ttl, &preview::posix_attr(record, uid, gid), 0)
            } else {
                reply.add_without_lookup(entry.ino, cookie, entry.name.as_ref(), kind(entry.kind))
            };
            if full { reply.ok(); return; }
        }
        index = end;
    }
    reply.ok();
    drop(guard);
}
fn send_page(view: &View, offset: usize, exclusion: Option<&drive_core::exclude::ExclusionIndex>, mut reply: ReplyDirectory) {
    let _span = diagnostics::span(diagnostics::Stage::ReaddirReply);
    if offset > view.snapshot().entries.len() {
        reply.error(libc::EINVAL);
        return;
    }
    for (index, entry) in view.snapshot().entries.iter().enumerate().skip(offset) {
        if excluded(exclusion, view.ino, &entry.name) { continue; }
        let kind = if entry.directory {
            FileType::Directory
        } else {
            FileType::RegularFile
        };
        if reply.add(entry.ino, (index + 1) as i64, kind, &entry.name) {
            break;
        }
    }
    reply.ok();
}
pub(super) fn release(fs: &mut CacheFilesystem, ino: u64, fh: u64, reply: ReplyEmpty) {
    let _span = diagnostics::span(diagnostics::Stage::Releasedir);
    // Kernel close is not retried if a bounded work queue rejects its job.
    // Remove the handle directly so overload cannot permanently leak a view.
    // The final reference transfers the snapshot to its capacity-reserved
    // reclaimer; large deallocation does not run on the FUSE dispatcher.
    match fs.directories.release(ino, fh) {
        Ok(()) => reply.ok(),
        Err(e) => reply.error(e),
    }
}

#[cfg(test)]
#[path = "directory_view_stamp_tests.rs"]
mod stamp_tests;

/// Deterministic seam for the #23 per-directory single-flight tests: pause one
/// directory's read-only projection while another directory (or a second handle
/// of the same directory) is opened. Test-only; never compiled into the daemon.
#[cfg(test)]
pub(super) mod test_hooks {
    use std::{
        collections::HashMap,
        sync::{Arc, Condvar, Mutex, MutexGuard, OnceLock},
        time::{Duration, Instant},
    };

    struct Hook {
        /// Directories whose next projection blocks before it starts building.
        pauses: HashMap<u64, Arc<Pause>>,
        /// Projections that entered the build phase, per directory.
        starts: HashMap<u64, usize>,
    }

    /// Names visited while re-deriving one index view's byte estimate. The
    /// production path never increments it after #23; the regression test
    /// replays the pre-#23 derivation to prove the counter is live.
    static INDEX_NAME_SCANS: std::sync::atomic::AtomicUsize =
        std::sync::atomic::AtomicUsize::new(0);
    pub(super) fn note_index_name_scan(items: usize) {
        INDEX_NAME_SCANS.fetch_add(items, std::sync::atomic::Ordering::Relaxed);
    }
    pub(super) fn index_name_scans() -> usize {
        INDEX_NAME_SCANS.load(std::sync::atomic::Ordering::Relaxed)
    }

    fn hook() -> &'static Mutex<Hook> {
        static HOOK: OnceLock<Mutex<Hook>> = OnceLock::new();
        HOOK.get_or_init(|| {
            Mutex::new(Hook {
                pauses: HashMap::new(),
                starts: HashMap::new(),
            })
        })
    }

    /// Tests using the process-global seam must hold this for their duration.
    pub(super) fn serial() -> MutexGuard<'static, ()> {
        static SERIAL: Mutex<()> = Mutex::new(());
        SERIAL.lock().unwrap_or_else(|poison| poison.into_inner())
    }

    pub(super) struct Pause {
        _ino: u64,
        reached: Mutex<bool>,
        release: Mutex<bool>,
        reached_cv: Condvar,
        release_cv: Condvar,
    }
    impl Pause {
        fn new(ino: u64) -> Self {
            Self {
                _ino: ino,
                reached: Mutex::new(false),
                release: Mutex::new(false),
                reached_cv: Condvar::new(),
                release_cv: Condvar::new(),
            }
        }
        /// Wait until the paused projection reached the build boundary.
        pub(super) fn wait_reached(&self, timeout: Duration) -> bool {
            let deadline = Instant::now() + timeout;
            let mut reached = self.reached.lock().unwrap_or_else(|poison| poison.into_inner());
            while !*reached {
                let left = deadline.saturating_duration_since(Instant::now());
                if left.is_zero() {
                    return false;
                }
                let (guard, _) = self
                    .reached_cv
                    .wait_timeout(reached, left)
                    .unwrap_or_else(|poison| poison.into_inner());
                reached = guard;
            }
            true
        }
        pub(super) fn release(&self) {
            *self.release.lock().unwrap_or_else(|poison| poison.into_inner()) = true;
            self.release_cv.notify_all();
        }
    }

    /// Clear every pause/counter. Call while holding [`serial`].
    pub(super) fn reset() {
        let mut hook = hook().lock().unwrap_or_else(|poison| poison.into_inner());
        hook.pauses.clear();
        hook.starts.clear();
        INDEX_NAME_SCANS.store(0, std::sync::atomic::Ordering::Relaxed);
    }

    /// Block the next projection of `ino` at the build boundary.
    pub(super) fn pause(ino: u64) -> Arc<Pause> {
        let pause = Arc::new(Pause::new(ino));
        hook().lock()
            .unwrap_or_else(|poison| poison.into_inner())
            .pauses
            .insert(ino, pause.clone());
        pause
    }

    pub(super) fn starts(ino: u64) -> usize {
        hook()
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
            .starts
            .get(&ino)
            .copied()
            .unwrap_or(0)
    }

    /// Called by `Views::open` immediately before one read-only projection.
    pub(super) fn before_projection(ino: u64) {
        let pause = {
            let mut hook = hook().lock().unwrap_or_else(|poison| poison.into_inner());
            *hook.starts.entry(ino).or_insert(0) += 1;
            hook.pauses.get(&ino).cloned()
        };
        let Some(pause) = pause else { return; };
        *pause.reached.lock().unwrap_or_else(|poison| poison.into_inner()) = true;
        pause.reached_cv.notify_all();
        let mut release = pause.release.lock().unwrap_or_else(|poison| poison.into_inner());
        while !*release {
            release = pause
                .release_cv
                .wait(release)
                .unwrap_or_else(|poison| poison.into_inner());
        }
    }
}

#[cfg(test)]
#[path = "directory_bench_tests.rs"]
mod bench_tests;

#[cfg(test)]
mod index_tests {
    use super::*;
    fn block() -> Arc<DirBlock> {
        let entries = vec![
            drive_core::namespace_index::DirEntryLite { ino: 1, name: Arc::from("."), kind: FileKind::Directory },
            drive_core::namespace_index::DirEntryLite { ino: 1, name: Arc::from(".."), kind: FileKind::Directory },
            drive_core::namespace_index::DirEntryLite { ino: 2, name: Arc::from("file"), kind: FileKind::File },
        ];
        let estimated_bytes = expected_bytes(&entries);
        Arc::new(DirBlock { ino: 1, version: 0, built_at_gen: 3, entries, estimated_bytes })
    }
    fn token() -> CommitState { CommitState { revision: 4, verified: true } }
    /// The byte estimate charged per index view. Issue #23 residual moved the
    /// O(N) name-walk from every hot open into the block builder; keep the
    /// documented formula (struct + Vec capacity + name bytes) as the oracle.
    fn expected_bytes(entries: &Vec<drive_core::namespace_index::DirEntryLite>) -> usize {
        std::mem::size_of::<DirBlock>()
            .saturating_add(entries.capacity() * std::mem::size_of::<drive_core::namespace_index::DirEntryLite>())
            .saturating_add(entries.iter().map(|entry| entry.name.len()).sum::<usize>())
    }
    /// Exact pre-#23 hot-open derivation, kept test-only. It walks every entry
    /// name through the regression counter, which is what makes the "zero
    /// visits" assertion above falsifiable.
    fn replay_legacy_index_view_bytes(block: &DirBlock) -> usize {
        test_hooks::note_index_name_scan(block.entries.len());
        std::mem::size_of::<DirBlock>()
            .saturating_add(block.entries.capacity() * std::mem::size_of::<drive_core::namespace_index::DirEntryLite>())
            .saturating_add(block.entries.iter().map(|entry| entry.name.len()).sum::<usize>())
    }
    /// Issue #23 residual: a hot index open must be O(1) in the directory size.
    /// The red state was a per-open full name walk (H x N visits) inside the
    /// handles mutex and the projection lease. The test asserts the precomputed
    /// contract directly: the builder's `estimated_bytes` equals the documented
    /// old formula, and every hot open charges exactly that amount without
    /// changing it (so no re-derivation can silently disagree with it).
    #[test]
    fn hot_index_opens_do_not_rescan_every_entry_name() {
        let _serial = test_hooks::serial();
        test_hooks::reset();
        const CHILDREN: usize = 10_000;
        const OPENS: usize = 100;
        let entries = {
            let mut entries = Vec::with_capacity(CHILDREN + 2);
            entries.push(drive_core::namespace_index::DirEntryLite { ino: 1, name: Arc::from(".."), kind: FileKind::Directory });
            entries.push(drive_core::namespace_index::DirEntryLite { ino: 1, name: Arc::from("."), kind: FileKind::Directory });
            entries.extend((0..CHILDREN).map(|i| drive_core::namespace_index::DirEntryLite {
                ino: 2 + i as u64,
                name: Arc::from(format!("file-{i:05}")),
                kind: FileKind::File,
            }));
            entries
        };
        let expected = expected_bytes(&entries);
        let big = Arc::new(DirBlock {
            ino: 1,
            version: 0,
            built_at_gen: 3,
            estimated_bytes: expected,
            entries,
        });
        let views = Views::default();
        // Keep every handle open so the charge is deterministic: the reclaimer
        // returns capacity asynchronously on release and is not part of this
        // accounting assertion.
        for _ in 0..OPENS {
            views.open_index(big.clone(), token()).unwrap().unwrap();
        }
        // Issue #23 regression contract: the hot open path never visits entry
        // names. The pre-#23 code produced OPENS x CHILDREN visits here (red
        // evidence branch `evidence/fs-red23`, 1,000,200 visits).
        assert_eq!(
            test_hooks::index_name_scans(),
            0,
            "hot index opens walked entry names again"
        );
        // Conservative per-View accounting is unchanged: one charge per open,
        // each equal to the precomputed estimate.
        assert_eq!(
            views.charged.load(Ordering::Acquire),
            expected.saturating_mul(OPENS),
            "hot index opens no longer charge exactly the precomputed estimate"
        );
        // Prove the counter above is a real observation, not a dead assertion:
        // replaying the exact pre-#23 derivation goes through the same counter.
        assert_eq!(
            replay_legacy_index_view_bytes(&big),
            expected,
            "the precomputed estimate must equal the legacy formula"
        );
        assert_eq!(
            test_hooks::index_name_scans(),
            big.entries.len(),
            "the name-visit counter no longer observes a legacy-style derivation"
        );
        assert_eq!(
            big.estimated_bytes, expected,
            "the builder's estimate drifted from the documented byte formula"
        );
    }
    #[test]
    fn index_views_share_the_original_byte_and_handle_budget() {
        let views = Views::default();
        views.charged.store(MAX_BYTES, Ordering::Release);
        assert_eq!(views.open_index(block(), token()).unwrap(), Err(libc::ENOMEM));
        views.charged.store(0, Ordering::Release);
        views.allocated.store(MAX_HANDLES, Ordering::Release);
        assert_eq!(views.open_index(block(), token()).unwrap(), Err(libc::ENOMEM));
        views.allocated.store(0, Ordering::Release);
        let fh = views.open_index(block(), token()).unwrap().unwrap();
        let view = views.get(1, fh).unwrap();
        { let mut table = views.table.lock().unwrap();
          for handle in 2..=MAX_HANDLES as u64 { table.handles.insert(handle, view.clone()); }
          table.next = MAX_HANDLES as u64 + 1; }
        assert_eq!(views.open_index(block(), token()).unwrap(), Err(libc::EMFILE));
        assert_eq!(views.release(2, fh), Err(libc::EBADF));
    }
    #[test]
    fn old_index_view_keeps_cookies_and_releases_through_reserved_reclaimer() {
        let views = Views::default();
        let original = block();
        let fh = views.open_index(original.clone(), token()).unwrap().unwrap();
        let reader = views.get(1, fh).unwrap();
        assert!(Arc::ptr_eq(reader.indexed().unwrap().0, &original));
        assert_eq!(reader.indexed().unwrap().1, token());
        let charged = views.charged.clone();
        let allocated = views.allocated.clone();
        views.release(1, fh).unwrap();
        assert!(charged.load(Ordering::Acquire) > 0);
        drop(views);
        assert_eq!(reader.indexed().unwrap().0.entries[2].name.as_ref(), "file");
        drop(reader);
        assert_eq!(charged.load(Ordering::Acquire), 0);
        assert_eq!(allocated.load(Ordering::Acquire), 0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn shared_views_keep_old_namespace_and_release_their_budget() {
        let temp = tempfile::tempdir().unwrap();
        let mut store = Store::open(temp.path()).unwrap();
        let node = |id: &str| Node {
            id: id.into(),
            name: id.into(),
            parent: (id != "root").then(|| "root".into()),
            directory: id == "root",
            version: "1".into(),
            metadata: None,
        };
        store
            .install_remote_snapshot(&[node("root"), node("a")], "1", "s")
            .unwrap();
        store.ensure_fs_root("root").unwrap();
        let store = drive_core::SharedStore::new(store);
        let views = Views::default();
        let first = views.open(&store, 1).unwrap();
        let second = views.open(&store, 1).unwrap();
        let a = views.get(1, first).unwrap();
        let b = views.get(1, second).unwrap();
        assert!(Arc::ptr_eq(&a, &b));
        assert_eq!(views.charged.load(Ordering::Acquire), a.snapshot().bytes);
        store
            .lock()
            .unwrap()
            .create_local_directory("root", "added", 0o755)
            .unwrap();
        let next = views.open(&store, 1).unwrap();
        let c = views.get(1, next).unwrap();
        assert_eq!(a.snapshot().entries.len(), 3);
        assert_eq!(c.snapshot().entries.len(), 4);
        assert!(!Arc::ptr_eq(&a, &c));
        assert!(matches!(views.get(999, first), Err(libc::EBADF)));
        assert_eq!(views.release(999, first), Err(libc::EBADF));
        views.release(1, first).unwrap();
        views.release(1, second).unwrap();
        views.release(1, next).unwrap();
        assert!(views.charged.load(Ordering::Acquire) > 0); // in-flight readers still own views
        drop((a, b, c));
        // Closed authority views now have a separately bounded hot owner.
        views.trim_hot(None);
        let deadline = std::time::Instant::now() + Duration::from_secs(2);
        while views.allocated.load(Ordering::Acquire) != 0 && std::time::Instant::now() < deadline {
            std::thread::yield_now();
        }
        assert_eq!(views.charged.load(Ordering::Acquire), 0);
        assert_eq!(views.allocated.load(Ordering::Acquire), 0);
        assert!(matches!(views.get(1, first), Err(libc::EBADF)));
    }

    #[test]
    fn exhausted_budgets_reject_new_views_without_losing_existing_handles() {
        let temp = tempfile::tempdir().unwrap();
        let mut store = Store::open(temp.path()).unwrap();
        store
            .install_remote_snapshot(
                &[Node {
                    id: "root".into(),
                    name: "root".into(),
                    parent: None,
                    directory: true,
                    version: "1".into(),
                    metadata: None,
                }],
                "1",
                "s",
            )
            .unwrap();
        store.ensure_fs_root("root").unwrap();
        let store = drive_core::SharedStore::new(store);
        let views = Views::default();
        views.charged.store(MAX_BYTES, Ordering::Release);
        let revision = store.lock().unwrap().local_revision();
        assert_eq!(views.open(&store, 1), Err(libc::ENOMEM));
        assert_eq!(store.lock().unwrap().local_revision(), revision);
        assert!(views.table.lock().unwrap().handles.is_empty());
        views.charged.store(0, Ordering::Release);
        views.allocated.store(MAX_HANDLES, Ordering::Release);
        assert_eq!(views.open(&store, 1), Err(libc::ENOMEM));
        views.allocated.store(0, Ordering::Release);
        let fh = views.open(&store, 1).unwrap();
        let view = views.get(1, fh).unwrap();
        {
            let mut table = views.table.lock().unwrap();
            for handle in 2..=MAX_HANDLES as u64 {
                table.handles.insert(handle, view.clone());
            }
            table.next = MAX_HANDLES as u64 + 1;
        }
        assert_eq!(views.open(&store, 1), Err(libc::EMFILE));
        assert_eq!(views.get(1, fh).unwrap().snapshot().entries.len(), 2);
        views.release(1, 2).unwrap();
        assert!(views.open(&store, 1).is_ok());
        assert_eq!(views.charged.load(Ordering::Acquire), view.snapshot().bytes);
    }

    #[test]
    fn reclaim_channel_holds_every_reserved_view_without_returning_capacity_early() {
        let (sender, receiver) = sync_channel(MAX_HANDLES);
        let charged = Arc::new(AtomicUsize::new(0));
        let allocated = Arc::new(AtomicUsize::new(0));
        // Deliberately leave the receiver idle to model a stalled reclaimer.
        let reclaimer = Arc::new(Reclaimer {
            sender: Some(sender),
            thread: None,
            charged: charged.clone(),
            allocated: allocated.clone(),
        });
        let bytes = std::mem::size_of::<drive_core::filesystem::DirectoryEntry>() + 1;
        for _ in 0..MAX_HANDLES {
            let snapshot = DirectorySnapshot {
                entries: vec![drive_core::filesystem::DirectoryEntry {
                    ino: 1,
                    directory: true,
                    name: ".".into(),
                }],
                bytes,
            };
            charged.fetch_add(bytes, Ordering::AcqRel);
            allocated.fetch_add(1, Ordering::AcqRel);
            drop(View {
                origin: ViewOrigin::Authority,
                ino: 1,
                snapshot: Some(snapshot),
                reclaimer: reclaimer.clone(),
            });
        }
        assert_eq!(charged.load(Ordering::Acquire), MAX_HANDLES * bytes);
        assert_eq!(allocated.load(Ordering::Acquire), MAX_HANDLES);
        for _ in 0..MAX_HANDLES {
            reclaim(receiver.try_recv().unwrap(), &charged, &allocated);
        }
        assert!(receiver.try_recv().is_err());
        assert_eq!(charged.load(Ordering::Acquire), 0);
        assert_eq!(allocated.load(Ordering::Acquire), 0);
    }

    #[test]
    fn last_reader_can_outlive_the_handle_table_and_drain_reclaimer() {
        let temp = tempfile::tempdir().unwrap();
        let mut store = Store::open(temp.path()).unwrap();
        store
            .install_remote_snapshot(
                &[Node {
                    id: "root".into(),
                    name: "root".into(),
                    parent: None,
                    directory: true,
                    version: "1".into(),
                    metadata: None,
                }],
                "1",
                "s",
            )
            .unwrap();
        store.ensure_fs_root("root").unwrap();
        let views = Views::default();
        let fh = views.open(&drive_core::SharedStore::new(store), 1).unwrap();
        let reader = views.get(1, fh).unwrap();
        let charged = views.charged.clone();
        let allocated = views.allocated.clone();
        drop(views);
        assert_eq!(reader.snapshot().entries.len(), 2);
        assert!(charged.load(Ordering::Acquire) > 0);
        drop(reader); // last reclaimer owner closes channel and joins its drain
        assert_eq!(charged.load(Ordering::Acquire), 0);
        assert_eq!(allocated.load(Ordering::Acquire), 0);
    }
}

#[cfg(test)]
mod acl_page_tests {
    use super::*;
    use drive_core::filesystem::{DirectoryEntry, DirectorySnapshot};

    fn indexed(children: usize) -> DirBlock {
        let mut entries = Vec::with_capacity(children + 2);
        entries.push(drive_core::namespace_index::DirEntryLite { ino: 1, name: Arc::from(".."), kind: FileKind::Directory });
        entries.push(drive_core::namespace_index::DirEntryLite { ino: 1, name: Arc::from("."), kind: FileKind::Directory });
        entries.extend((0..children).map(|index| drive_core::namespace_index::DirEntryLite {
            ino: 2 + index as u64,
            name: Arc::from(format!("file-{index:05}")),
            kind: FileKind::File,
        }));
        DirBlock {
            ino: 1,
            version: 0,
            built_at_gen: 3,
            estimated_bytes: 0,
            entries,
        }
    }
    fn snapshot(children: usize) -> DirectorySnapshot {
        let mut entries = Vec::with_capacity(children + 2);
        entries.push(DirectoryEntry { ino: 1, directory: true, name: "..".into() });
        entries.push(DirectoryEntry { ino: 1, directory: true, name: ".".into() });
        entries.extend((0..children).map(|index| DirectoryEntry {
            ino: 2 + index as u64,
            directory: false,
            name: format!("file-{index:05}"),
        }));
        DirectorySnapshot { entries, bytes: 0 }
    }

    /// Issue #26 residual: both origins are paged through borrowed batches. The
    /// batch bound is what stops a continuation from materializing the whole
    /// directory; the exact coverage proves paging did not change the cookie
    /// positions or skip entries.
    #[test]
    fn both_origins_page_with_bounded_borrowed_batches() {
        const CHILDREN: usize = 300;
        let indexed = indexed(CHILDREN);
        let snapshot = snapshot(CHILDREN);
        for origin in [PlusOrigin::Indexed(&indexed), PlusOrigin::Snapshot(&snapshot)] {
            assert_eq!(origin.len(), CHILDREN + 2);
            assert_eq!(origin.slot(0).name, "..");
            assert!(origin.slot(0).directory);
            assert_eq!(origin.slot(1).name, ".");
            assert!(!origin.slot(2).directory);
            assert_eq!(origin.kind(0), FileType::Directory);
            assert_eq!(origin.kind(2), FileType::RegularFile);
            let mut position = 0;
            let mut seen = 0;
            let mut max_batch = 0;
            while position < origin.len() {
                let range = acl_page_range(position, origin.len());
                let batch = range.len();
                assert!(batch <= PLUS_PAGE, "page batch {batch} exceeds PLUS_PAGE");
                max_batch = max_batch.max(batch);
                seen += batch;
                position = range.end;
            }
            assert_eq!(seen, CHILDREN + 2, "paging did not cover the directory exactly once");
            assert_eq!(max_batch, PLUS_PAGE);
            // offset == N is an empty first batch (EOF), offset > N is rejected
            // by the caller before any batch is built.
            assert_eq!(acl_page_range(origin.len(), origin.len()).len(), 0);
            assert_eq!(origin.slot(CHILDREN + 1).ino, 2 + CHILDREN as u64 - 1);
        }
    }
}
