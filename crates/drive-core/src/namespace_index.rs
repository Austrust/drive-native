//! Bounded, read-only namespace projection. SQLite remains authoritative.
//!
//! A returned Arc owns its generation; refreshing never invalidates a Rust
//! reference. Callers must hold the SharedStore projection read lease while
//! selecting AND publishing index results. This module's Store revision is
//! `total_changes`, not the independent SharedStore commit generation.
//! `None` means "use the SQL path", never authoritative ENOENT.
use crate::{Error, Result, Store, filesystem::{DirectoryViewStamp, PosixRecord}};
use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex, RwLock, atomic::{AtomicU64, Ordering}},
    time::{Duration, Instant},
};

pub const MAX_ENTRIES: usize = 2_000_000;
pub const MAX_BYTES: usize = 512 * 1024 * 1024;
const BUILD_BUDGET: Duration = Duration::from_secs(1);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RefreshOutcome { Fresh, Refreshed, Degraded }
#[derive(Clone, Debug)]
pub enum InvalidateHint {
    All,
    Node { node_id: String, parent_dir_ino: Option<u64> },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FileKind { File, Directory }
#[derive(Clone, Debug)]
pub struct EntrySlot {
    pub ino: u64,
    pub node: Arc<str>,
    pub parent: u64,
    pub name: Arc<str>,
    pub directory: bool,
    pub size: u64,
    pub mtime: u64,
    pub atime: (u64, u32),
    pub mode: u16,
    pub blocks: u64,
    pub dirty: bool,
    pub preview: bool,
}
#[derive(Clone, Debug)]
pub struct DirEntryLite { pub ino: u64, pub name: Arc<str>, pub kind: FileKind }
#[derive(Clone, Debug)]
pub struct DirBlock {
    pub ino: u64,
    /// Legacy cookies: '.', '..', then children ordered by name / identity.
    /// Existing open handles retain this Arc through rename/delete/refresh.
    pub entries: Vec<DirEntryLite>,
    pub version: u64,
    pub built_at_gen: u64,
}

pub(crate) struct ProjectedEntry {
    pub id: String,
    pub ino: Option<u64>,
    pub name: String,
    pub parent_ino: Option<u64>,
    pub root: bool,
    pub parent_visible: bool,
    pub visible_child: bool,
    pub directory: bool,
    pub attributes: Option<PosixRecord>,
}

#[derive(Clone, Copy)]
struct Budget { entries: usize, bytes: usize, duration: Duration }
impl Default for Budget {
    fn default() -> Self {
        Self { entries: MAX_ENTRIES, bytes: MAX_BYTES, duration: BUILD_BUDGET }
    }
}

#[derive(Debug)]
struct Snapshot {
    revision: u64,
    invalidation: u64,
    stamp: Option<DirectoryViewStamp>,
    by_ino: HashMap<u64, Arc<EntrySlot>>,
    dirs: HashMap<u64, Arc<DirBlock>>,
    bytes: usize,
    partial: bool,
    degraded: bool,
}
impl Snapshot {
    fn empty(revision: u64, invalidation: u64, stamp: Option<DirectoryViewStamp>) -> Self {
        Self {
            revision, invalidation, stamp, by_ino: HashMap::new(), dirs: HashMap::new(),
            bytes: 0, partial: true, degraded: true,
        }
    }
}
#[derive(Debug)]
struct Inner {
    snapshot: RwLock<Arc<Snapshot>>,
    invalidation: AtomicU64,
    builder: Mutex<()>,
}
/// Cloning an index shares freshness and invalidation; it does not fork state.
#[derive(Clone, Debug)]
pub struct NamespaceIndex { inner: Arc<Inner> }

pub fn build(store: &Store) -> Result<NamespaceIndex> { NamespaceIndex::build(store) }

impl NamespaceIndex {
    pub fn build(store: &Store) -> Result<Self> {
        Self::build_with_budget(store, Budget::default())
    }
    fn build_with_budget(store: &Store, budget: Budget) -> Result<Self> {
        let snapshot = load(store, 0, budget)?;
        Ok(Self { inner: Arc::new(Inner {
            snapshot: RwLock::new(Arc::new(snapshot)),
            invalidation: AtomicU64::new(0), builder: Mutex::new(()),
        }) })
    }
    fn snapshot(&self) -> Option<Arc<Snapshot>> {
        self.inner.snapshot.read().ok().map(|snapshot| Arc::clone(&snapshot))
    }
    fn current(&self) -> Option<Arc<Snapshot>> {
        let snapshot = self.snapshot()?;
        let epoch = self.inner.invalidation.load(Ordering::Acquire);
        (!snapshot.degraded && snapshot.invalidation == epoch && epoch != u64::MAX)
            .then_some(snapshot)
    }
    pub fn entry_by_ino(&self, ino: u64) -> Option<Arc<EntrySlot>> {
        self.current()?.by_ino.get(&ino).cloned()
    }
    pub fn lookup(&self, parent_ino: u64, name: &str) -> Option<Arc<EntrySlot>> {
        let snapshot = self.current()?;
        let block = snapshot.dirs.get(&parent_ino)?;
        // The POSIX lookup API does not interpret '.' or '..'. FUSE resolves
        // those specially; never synthesize a new namespace result here.
        let children = block.entries.get(2..)?;
        let slot = children.binary_search_by(|entry| entry.name.as_ref().cmp(name)).ok()?;
        // Ambiguous remote names remain SQL/state errors rather than choosing
        // an arbitrary identity from the sorted array.
        if slot > 0 && children[slot - 1].name.as_ref() == name
            || slot + 1 < children.len() && children[slot + 1].name.as_ref() == name {
            return None;
        }
        snapshot.by_ino.get(&children[slot].ino).cloned()
    }
    pub fn dir_block(&self, ino: u64) -> Option<Arc<DirBlock>> {
        self.current()?.dirs.get(&ino).cloned()
    }
    pub fn revision(&self) -> u64 { self.snapshot().map_or(0, |snapshot| snapshot.revision) }
    pub fn bytes(&self) -> usize { self.snapshot().map_or(0, |snapshot| snapshot.bytes) }
    pub fn is_partial(&self) -> bool { self.snapshot().is_none_or(|snapshot| snapshot.partial) }
    pub fn is_degraded(&self) -> bool { self.current().is_none() }

    pub fn invalidate(&self, _hint: InvalidateHint) {
        // Hints are optional and may be lost. Conservatively invalidate the
        // whole generation; never claim unrefreshed siblings are current.
        let _ = self.inner.invalidation.fetch_update(Ordering::AcqRel, Ordering::Acquire,
            |value| Some(value.saturating_add(1)));
    }

    pub fn ensure_fresh(&self, store: &Store) -> RefreshOutcome {
        let Ok(_builder) = self.inner.builder.try_lock() else {
            // Another rebuild is in progress or poisoned. Never wait behind
            // a large namespace load in a FUSE request.
            return RefreshOutcome::Degraded;
        };
        let stamp = match store.directory_view_stamp() {
            Ok(stamp) => stamp,
            Err(_) => { self.invalidate(InvalidateHint::All); return RefreshOutcome::Degraded; }
        };
        let revision = store.local_revision();
        let epoch = self.inner.invalidation.load(Ordering::Acquire);
        if epoch == u64::MAX { return RefreshOutcome::Degraded; }
        if let Some(snapshot) = self.snapshot() {
            if snapshot.invalidation == epoch && snapshot.revision == revision
                && snapshot.stamp.as_ref() == Some(&stamp) {
                // A budget failure is stable for this generation. Do not
                // repeatedly spend 1s rebuilding on each lookup/EOF request.
                return if snapshot.degraded { RefreshOutcome::Degraded } else { RefreshOutcome::Fresh };
            }
        }
        self.invalidate(InvalidateHint::All);
        let epoch = self.inner.invalidation.load(Ordering::Acquire);
        if epoch == u64::MAX { return RefreshOutcome::Degraded; }
        let snapshot = load(store, epoch, Budget::default()).unwrap_or_else(|_| {
            Snapshot::empty(revision, epoch, Some(stamp))
        });
        let degraded = snapshot.degraded;
        let Ok(mut destination) = self.inner.snapshot.write() else { return RefreshOutcome::Degraded; };
        // An invalidation arriving during load must not be swallowed by the
        // new publication. Its epoch mismatch keeps every read on SQL.
        if self.inner.invalidation.load(Ordering::Acquire) != epoch {
            return RefreshOutcome::Degraded;
        }
        *destination = Arc::new(snapshot);
        if degraded { RefreshOutcome::Degraded } else { RefreshOutcome::Refreshed }
    }
}

fn budget_error() -> Error {
    Error::Io(std::io::Error::from_raw_os_error(libc::ENOMEM))
}
fn load(store: &Store, epoch: u64, budget: Budget) -> Result<Snapshot> {
    let start = Instant::now();
    let stamp = store.directory_view_stamp()?;
    let revision = store.local_revision();
    // The transaction covers all joins and ACL/shadow membership. It is read
    // only: unlike posix_lookup/directory_snapshot, no inode is allocated.
    let transaction = store.db.unchecked_transaction()?;
    let initialized: bool = transaction.query_row(
        "SELECT EXISTS(SELECT 1 FROM fs_inodes i JOIN nodes n ON n.id=i.node
         WHERE i.ino=1 AND n.directory=1 AND n.parent IS NULL AND n.deleted=0)",
        [], |row| row.get(0),
    )?;
    if !initialized {
        return Err(Error::State("namespace index requires an authoritative filesystem root".into()));
    }
    let mut snapshot = Snapshot {
        revision, invalidation: epoch, stamp: Some(stamp.clone()),
        by_ino: HashMap::new(), dirs: HashMap::new(), bytes: 0,
        partial: false, degraded: false,
    };
    let mut children: HashMap<u64, Vec<DirEntryLite>> = HashMap::new();
    let mut directories: HashMap<u64, u64> = HashMap::new();
    let mut incomplete: HashSet<u64> = HashSet::new();
    let mut count = 0usize;
    // Conservative allocation charge includes map slack, Arc headers and the
    // temporary construction maps, not merely UTF-8 payload length.
    let mut charged = 0usize;
    let result = store.visit_namespace_index(|row| {
        count = count.checked_add(1).ok_or_else(budget_error)?;
        charged = charged.checked_add(
            320usize.saturating_add(row.id.len()).saturating_add(row.name.len()),
        ).ok_or_else(budget_error)?;
        if count > budget.entries || charged > budget.bytes
            || (count & 255 == 0 && start.elapsed() > budget.duration) {
            return Err(budget_error());
        }
        let Some(ino) = row.ino else {
            snapshot.partial = true;
            if let Some(parent) = row.parent_ino { incomplete.insert(parent); }
            return Ok(());
        };
        let parent = if row.root { ino } else { row.parent_ino.unwrap_or(0) };
        let name: Arc<str> = Arc::from(row.name);
        if row.directory && row.parent_visible && parent != 0 {
            directories.insert(ino, parent);
        }
        if !row.root && row.visible_child && parent != 0 {
            children.entry(parent).or_default().push(DirEntryLite {
                ino, name: Arc::clone(&name),
                kind: if row.directory { FileKind::Directory } else { FileKind::File },
            });
        }
        if let Some(record) = row.attributes {
            snapshot.by_ino.insert(ino, Arc::new(EntrySlot {
                ino, node: Arc::from(row.id), parent, name, directory: row.directory,
                size: record.size, mtime: record.mtime,
                atime: (record.atime, record.atime_nsec), mode: record.mode as u16,
                blocks: record.blocks, dirty: false, preview: false,
            }));
        } else {
            snapshot.partial = true;
        }
        Ok(())
    });
    if let Err(error) = result {
        // Budget exhaustion discards the entire partial generation. Other
        // failures propagate so the caller can record a build-failure metric.
        if matches!(&error, Error::Io(error) if error.raw_os_error() == Some(libc::ENOMEM)) {
            transaction.commit()?;
            return Ok(Snapshot::empty(revision, epoch, Some(stamp)));
        }
        return Err(error);
    }
    for (ino, parent) in directories {
        if incomplete.contains(&ino) { continue; }
        let mut entries = children.remove(&ino).unwrap_or_default();
        entries.sort_unstable_by(|a, b| a.name.cmp(&b.name).then(a.ino.cmp(&b.ino)));
        // reserve first so moving dot entries does not grow on every push.
        entries.reserve(2);
        entries.insert(0, DirEntryLite { ino: parent, name: Arc::from(".."), kind: FileKind::Directory });
        entries.insert(0, DirEntryLite { ino, name: Arc::from("."), kind: FileKind::Directory });
        charged = charged.saturating_add(128 + entries.capacity() * std::mem::size_of::<DirEntryLite>());
        if charged > budget.bytes || start.elapsed() > budget.duration {
            transaction.commit()?;
            return Ok(Snapshot::empty(revision, epoch, Some(stamp)));
        }
        snapshot.dirs.insert(ino, Arc::new(DirBlock {
            ino, entries, version: epoch, built_at_gen: revision,
        }));
    }
    transaction.commit()?;
    if store.local_revision() != revision || store.directory_view_stamp()? != stamp {
        return Err(Error::State("namespace changed during index build".into()));
    }
    snapshot.bytes = charged;
    Ok(snapshot)
}
