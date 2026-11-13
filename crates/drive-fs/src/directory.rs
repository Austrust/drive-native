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
    builder: Mutex<()>,
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
            builder: Mutex::new(()),
            charged,
            allocated,
            reclaimer,
        }
    }
}
impl Views {
    /// Shares the original handle/byte/allocation caps and reclamation queue.
    /// A failed try-lock is a normal SQL-path fallback, never dispatcher wait.
    fn open_index(&self, block: Arc<DirBlock>, token: CommitState) -> Option<Result<u64, i32>> {
        let Ok(_building) = self.builder.try_lock() else { return None; };
        let mut table = self.table.lock().unwrap();
        if table.handles.len() >= MAX_HANDLES || table.next == u64::MAX {
            return Some(Err(libc::EMFILE));
        }
        // A held old-generation block remains charged until the reclaimer has
        // actually released it. Charge conservatively even when Arc is shared.
        let bytes = std::mem::size_of::<DirBlock>()
            .saturating_add(block.entries.capacity() * std::mem::size_of::<drive_core::namespace_index::DirEntryLite>())
            .saturating_add(block.entries.iter().map(|e| e.name.len()).sum::<usize>());
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
    fn open(&self, store: &drive_core::SharedStore, ino: u64) -> Result<u64, i32> {
        {
            let table = self.table.lock().unwrap_or_else(|poison| poison.into_inner());
            if table.handles.len() >= MAX_HANDLES || table.next == u64::MAX {
                return Err(libc::EMFILE);
            }
        }
        let token = settle(store);
        if token.verified {
            self.trim_hot(Some(&token));
            let cached = {
                let cached = self.cached.lock().unwrap_or_else(|poison| poison.into_inner());
                cached.get(&ino).and_then(|(revision, view)| {
                    (revision == &token).then(|| view.upgrade()).flatten()
                })
            };
            if let Some(view) = cached {
                diagnostics::count(diagnostics::Stage::DirectoryCacheHit);
                self.retain_hot(ino, token, view.clone());
                return self.attach(view);
            }
        }
        diagnostics::count(diagnostics::Stage::DirectoryCacheMiss);
        let wait = diagnostics::span(diagnostics::Stage::DirectoryBuilderWait);
        let building = self.builder.lock().unwrap_or_else(|poison| poison.into_inner());
        drop(wait);
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
        let (handle, view) = self.open_locked(store, ino)?;
        if let Some(view) = view {
            let token = store.change_state();
            if token.verified {
                self.publish(ino, token, view);
            }
        }
        Ok(handle)
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
        let _building = self.builder.lock().unwrap_or_else(|poison| poison.into_inner());
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
// ACL publication holds Store through the bounded reply. Known revocations
// cannot use the immutable-name busy-Store fallback of ordinary scans. Pages
// are projected in bounded batches instead of one SQL round-trip per entry, so
// a 2000-entry directory keeps the Store guard for a handful of page queries
// rather than ~50k per-entry statements (#26).
fn send_acl_plus(store:&drive_core::SharedStore,notify:&NotifyHub,view:&View,offset:usize,uid:u32,gid:u32,exclusion:Option<&drive_core::exclude::ExclusionIndex>,mut reply:ReplyDirectoryPlus) {
    let guard=match store.try_lock(){Ok(g)=>g,Err(_)=>{reply.error(libc::EAGAIN);return;}};
    if let Err(e)=permissions::inode(&guard,view.ino,permissions::Action::List){reply.error(errno(&e));return;}
    if let ViewOrigin::Preview(stamp)=&view.origin {if let Err(e)=guard.preview_validate_view(stamp){reply.error(errno(&e));return;}}
    let entries:Vec<_>=if let Some((block,_))=view.indexed(){block.entries.iter().map(|e|(e.ino,e.name.as_ref(),kind(e.kind))).collect()}
        else{view.snapshot().entries.iter().map(|e|(e.ino,e.name.as_str(),if e.directory{FileType::Directory}else{FileType::RegularFile})).collect()};
    if offset>entries.len(){reply.error(libc::EINVAL);return;}
    let parent=match guard.posix_record_by_inode(view.ino){Ok(record)=>record.id,Err(error)=>{reply.error(errno(&error));return;}};
    let cache_before=guard.permission_cache_stats();
    let mut index=offset;
    while index<entries.len() {
        let end=(index+PLUS_PAGE).min(entries.len());
        let chunk=&entries[index..end];
        let slots=chunk.iter().map(|(ino,name,directory)|drive_core::filesystem::DirectorySlot{
            ino:*ino,name,directory:*directory==FileType::Directory,
        }).collect::<Vec<_>>();
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
            let (ino,name,kind)=&chunk[position];
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
                    reply.add_with_timeouts(*ino,cookie,name,&ENTRY_TTL,&ttl,&preview::posix_attr(&record,uid,gid),0)
                }
                None=>reply.add_without_lookup(*ino,cookie,name,*kind),
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

#[cfg(test)]
#[path = "directory_bench_tests.rs"]
mod bench_tests;

#[cfg(test)]
mod index_tests {
    use super::*;
    fn block() -> Arc<DirBlock> {
        Arc::new(DirBlock { ino: 1, version: 0, built_at_gen: 3, entries: vec![
            drive_core::namespace_index::DirEntryLite { ino: 1, name: Arc::from("."), kind: FileKind::Directory },
            drive_core::namespace_index::DirEntryLite { ino: 1, name: Arc::from(".."), kind: FileKind::Directory },
            drive_core::namespace_index::DirEntryLite { ino: 2, name: Arc::from("file"), kind: FileKind::File },
        ] })
    }
    fn token() -> CommitState { CommitState { revision: 4, verified: true } }
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
