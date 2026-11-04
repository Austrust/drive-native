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
        let mut previous = self.inner.invalidation.load(Ordering::Acquire);
        loop {
            match self.inner.invalidation.compare_exchange_weak(
                previous, previous.saturating_add(1), Ordering::AcqRel, Ordering::Acquire,
            ) {
                Ok(_) => break,
                Err(actual) => previous = actual,
            }
        }
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
    let mut children: HashMap<u64, Vec<(Arc<str>, DirEntryLite)>> = HashMap::new();
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
        let node: Arc<str> = Arc::from(row.id);
        if row.directory && row.parent_visible && parent != 0 {
            directories.insert(ino, parent);
        }
        if !row.root && row.visible_child && parent != 0 {
            children.entry(parent).or_default().push((Arc::clone(&node), DirEntryLite {
                ino, name: Arc::clone(&name),
                kind: if row.directory { FileKind::Directory } else { FileKind::File },
            }));
        }
        if let Some(record) = row.attributes {
            snapshot.by_ino.insert(ino, Arc::new(EntrySlot {
                ino, node, parent, name, directory: row.directory,
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
        let mut ordered = children.remove(&ino).unwrap_or_default();
        ordered.sort_unstable_by(|a, b| a.1.name.cmp(&b.1.name).then(a.0.cmp(&b.0)));
        let mut entries: Vec<_> = ordered.into_iter().map(|(_, entry)| entry).collect();
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
    if start.elapsed() > budget.duration {
        return Ok(Snapshot::empty(revision, epoch, Some(stamp)));
    }
    snapshot.bytes = charged;
    Ok(snapshot)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Batch, Change, DownloadTicket, Event, Node, NodeMetadata};

    fn node(id: &str, parent: Option<&str>, directory: bool) -> Node {
        Node {
            id: id.into(), parent: parent.map(Into::into), name: id.into(),
            version: "1".into(), directory,
            metadata: Some(NodeMetadata {
                size: if directory { 0 } else { 3 }, mtime: 1_700_000_000,
                mode: if directory { 0o755 } else { 0o644 },
            }),
        }
    }
    fn fixture() -> (tempfile::TempDir, Store) {
        let path = tempfile::tempdir().unwrap();
        let mut store = Store::open(path.path()).unwrap();
        store.install_remote_snapshot(&[
            node("root", None, true), node("A", Some("root"), true),
            node("B", Some("root"), true), node("file", Some("A"), false),
        ], "1", "synthetic").unwrap();
        store.ensure_fs_root("root").unwrap();
        (path, store)
    }
    fn ino_for(store: &Store, id: &str) -> Result<u64> {
        let ino: i64 = store.db.query_row("SELECT ino FROM fs_inodes WHERE node=?1", [id],
            |row| row.get(0))?;
        u64::try_from(ino).map_err(|_| Error::State("invalid fixture inode".into()))
    }
    fn assert_record(entry: &EntrySlot, expected: &PosixRecord) {
        assert_eq!(entry.ino, expected.ino);
        assert_eq!(entry.node.as_ref(), expected.id);
        assert_eq!(entry.directory, expected.directory);
        assert_eq!(entry.size, expected.size);
        assert_eq!(entry.mtime, expected.mtime);
        assert_eq!(entry.atime, (expected.atime, expected.atime_nsec));
        assert_eq!(u32::from(entry.mode), expected.mode);
        assert_eq!(entry.blocks, expected.blocks);
        assert!(!entry.dirty && !entry.preview);
    }

    #[test]
    fn batch_projection_matches_sql_without_writes_or_hydration() {
        let (path, store) = fixture();
        let before = store.local_revision();
        let index = build(&store).unwrap();
        assert!(!index.is_degraded());
        assert_eq!(index.revision(), before);
        assert!(index.bytes() > 0 && index.bytes() <= MAX_BYTES);
        for parent in [1, ino_for(&store, "A").unwrap(), ino_for(&store, "B").unwrap()] {
            let expected = store.directory_snapshot(parent, MAX_BYTES).unwrap();
            let actual = index.dir_block(parent).unwrap();
            assert_eq!(actual.entries.len(), expected.entries.len());
            for (a, e) in actual.entries.iter().zip(&expected.entries) {
                assert_eq!((a.ino, a.name.as_ref(), a.kind == FileKind::Directory),
                    (e.ino, e.name.as_str(), e.directory));
                assert_record(&index.entry_by_ino(a.ino).unwrap(), &store.posix_record_by_inode(e.ino).unwrap());
            }
        }
        assert_eq!(store.local_revision(), before);
        assert!(store.downloads_needed().unwrap().is_empty());
        assert_eq!(std::fs::read_dir(path.path().join("blobs")).unwrap().count(), 0);
    }

    #[test]
    fn immutable_arc_survives_refresh_but_new_lookup_uses_new_generation() {
        let (_path, mut store) = fixture();
        let index = build(&store).unwrap();
        let parent = ino_for(&store, "A").unwrap();
        let old = index.lookup(parent, "file").unwrap();
        let old_directory = index.dir_block(parent).unwrap();
        let mut changed = node("file", Some("B"), false);
        changed.name = "renamed".into();
        changed.version = "2".into();
        changed.metadata.as_mut().unwrap().size = 71;
        store.receive(&Batch { from: "1".into(), to: "2".into(), events: vec![Event {
            id: "rename".into(), change: Change::Upsert(changed),
        }] }).unwrap();
        store.drain().unwrap();
        assert_eq!(index.ensure_fresh(&store), RefreshOutcome::Refreshed);
        assert!(index.lookup(parent, "file").is_none());
        let fresh = index.lookup(ino_for(&store, "B").unwrap(), "renamed").unwrap();
        assert_eq!((fresh.ino, fresh.size), (old.ino, 71));
        assert_eq!((old.name.as_ref(), old.size), ("file", 3));
        assert_eq!(old_directory.entries[2].name.as_ref(), "file");
        assert_ne!(old_directory.version, index.dir_block(parent).unwrap().version);
    }

    #[test]
    fn explicit_hint_invalidates_clones_even_without_revision_change() {
        let (_path, store) = fixture();
        let index = build(&store).unwrap();
        let cloned = index.clone();
        cloned.invalidate(InvalidateHint::Node { node_id: "file".into(), parent_dir_ino: None });
        assert!(index.is_degraded());
        assert!(index.entry_by_ino(1).is_none());
        assert!(index.dir_block(1).is_none());
        assert_eq!(index.ensure_fresh(&store), RefreshOutcome::Refreshed);
        assert!(!cloned.is_degraded());
        assert_eq!(cloned.ensure_fresh(&store), RefreshOutcome::Fresh);
    }

    #[test]
    fn uncommitted_boundary_cannot_publish_and_rollback_recovers() {
        let (_path, store) = fixture();
        let index = build(&store).unwrap();
        store.db.execute_batch("BEGIN; UPDATE node_metadata SET size=88 WHERE node='file';").unwrap();
        assert_eq!(index.ensure_fresh(&store), RefreshOutcome::Degraded);
        assert!(index.entry_by_ino(1).is_none());
        assert!(NamespaceIndex::build(&store).is_err());
        store.db.execute_batch("ROLLBACK").unwrap();
        assert_eq!(index.ensure_fresh(&store), RefreshOutcome::Refreshed);
        assert_eq!(index.entry_by_ino(ino_for(&store, "file").unwrap()).unwrap().size, 3);
    }

    #[test]
    fn explicit_atime_refresh_does_not_depend_on_status_journal() {
        let (_path, mut store) = fixture();
        let index = build(&store).unwrap();
        let stamp = store.directory_view_stamp().unwrap();
        store.set_local_atime("file", 1_777_000_000, 42).unwrap();
        assert_eq!(store.directory_view_stamp().unwrap(), stamp);
        assert_eq!(index.ensure_fresh(&store), RefreshOutcome::Refreshed);
        assert_eq!(index.entry_by_ino(ino_for(&store, "file").unwrap()).unwrap().atime,
            (1_777_000_000, 42));
    }

    #[test]
    fn dirty_body_attributes_are_never_served_from_index() {
        let (_path, mut store) = fixture();
        let draft = store.begin_draft("file", "1", None).unwrap();
        store.draft_write(&draft, 0, b"local draft", false).unwrap();
        let inode = ino_for(&store, "file").unwrap();
        let parent = ino_for(&store, "A").unwrap();
        let index = build(&store).unwrap();
        assert!(index.is_partial());
        assert!(index.entry_by_ino(inode).is_none());
        assert!(index.lookup(parent, "file").is_none());
        assert_eq!(index.dir_block(parent).unwrap().entries[2].ino, inode);
        assert_eq!(store.posix_record_by_inode(inode).unwrap().size, 11);
        // External changes to an already-dirty draft need not bump SQLite.
        let file = std::fs::read_dir(store.root.join("drafts")).unwrap().next().unwrap().unwrap();
        std::fs::OpenOptions::new().write(true).open(file.path()).unwrap().set_len(41).unwrap();
        assert_eq!(index.ensure_fresh(&store), RefreshOutcome::Fresh);
        assert!(index.entry_by_ino(inode).is_none());
        assert_eq!(store.posix_record_by_inode(inode).unwrap().size, 41);
    }

    #[test]
    fn clean_blob_size_and_local_mode_use_posix_precedence() {
        let (_path, mut store) = fixture();
        store.publish_download(&DownloadTicket { id: "file".into(), version: "1".into() },
            b"abc", &crate::sha256(b"abc")).unwrap();
        store.db.execute("UPDATE node_metadata SET size=999 WHERE node='file'", []).unwrap();
        store.set_local_mode("file", 0o600).unwrap();
        let inode = ino_for(&store, "file").unwrap();
        let index = build(&store).unwrap();
        let entry = index.entry_by_ino(inode).unwrap();
        assert_record(&entry, &store.posix_record_by_inode(inode).unwrap());
        assert_eq!((entry.size, entry.mode, entry.blocks), (3, 0o600, 1));
    }

    #[test]
    fn acl_overlay_edges_and_revocation_are_not_bypassed() {
        let (_path, store) = fixture();
        let index = build(&store).unwrap();
        let parent = ino_for(&store, "A").unwrap();
        let inode = ino_for(&store, "file").unwrap();
        let remote = node("file", Some("B"), false);
        store.db.execute("INSERT INTO namespace_overlay(node,remote) VALUES('file',?1)",
            [serde_json::to_string(&remote).unwrap()]).unwrap();
        store.db.execute("INSERT INTO access_denied(node) VALUES('B')", []).unwrap();
        assert_eq!(index.ensure_fresh(&store), RefreshOutcome::Refreshed);
        assert!(index.entry_by_ino(inode).is_none());
        assert!(index.lookup(parent, "file").is_none());
        assert_eq!(index.dir_block(parent).unwrap().entries.len(), 2);
        store.db.execute("DELETE FROM access_denied", []).unwrap();
        assert_eq!(index.ensure_fresh(&store), RefreshOutcome::Refreshed);
        assert!(index.lookup(parent, "file").is_some());
    }

    #[test]
    fn missing_inode_forces_directory_fallback_without_allocating() {
        let (_path, store) = fixture();
        let parent = ino_for(&store, "A").unwrap();
        store.db.execute("DELETE FROM fs_inodes WHERE node='file'", []).unwrap();
        let revision = store.local_revision();
        let index = build(&store).unwrap();
        assert!(index.is_partial());
        assert!(index.dir_block(parent).is_none());
        assert!(index.lookup(parent, "file").is_none());
        assert_eq!(store.local_revision(), revision);
        assert_eq!(store.db.query_row("SELECT count(*) FROM fs_inodes WHERE node='file'", [],
            |row| row.get::<_, i64>(0)).unwrap(), 0);
    }

    #[test]
    fn budgets_discard_partial_generations_and_do_not_retry_unchanged_state() {
        let (_path, store) = fixture();
        for budget in [
            Budget { entries: 1, ..Budget::default() },
            Budget { bytes: 1, ..Budget::default() },
            Budget { duration: Duration::ZERO, ..Budget::default() },
        ] {
            let index = NamespaceIndex::build_with_budget(&store, budget).unwrap();
            assert!(index.is_partial() && index.is_degraded());
            assert_eq!(index.bytes(), 0);
            assert!(index.entry_by_ino(1).is_none());
            assert!(index.dir_block(1).is_none());
            assert_eq!(index.ensure_fresh(&store), RefreshOutcome::Degraded);
        }
    }

    #[test]
    fn local_creation_shadows_remote_without_synthesizing_attributes() {
        let (_path, mut store) = fixture();
        let local = store.create_local_file("A", "new-name", 0o644).unwrap();
        let mut remote = node("file", Some("A"), false);
        remote.name = "new-name".into();
        remote.version = "2".into();
        store.receive(&Batch { from: "1".into(), to: "2".into(), events: vec![Event {
            id: "colliding-remote".into(), change: Change::Upsert(remote),
        }] }).unwrap();
        store.drain().unwrap();
        let parent = ino_for(&store, "A").unwrap();
        let expected = store.directory_snapshot(parent, MAX_BYTES).unwrap();
        let index = build(&store).unwrap();
        let actual = index.dir_block(parent).unwrap();
        assert_eq!(actual.entries.len(), expected.entries.len());
        assert_eq!(actual.entries[2].ino, ino_for(&store, &local.id).unwrap());
        assert_eq!(actual.entries[2].name.as_ref(), "new-name");
    }

    #[test]
    fn selected_subdirectory_is_normalized_to_authoritative_inode_one() {
        // native::project maps the selected root's parent to None. An index
        // must use the same scope as ensure_fs_root, not its remote ancestor.
        let path = tempfile::tempdir().unwrap();
        let mut store = Store::open(path.path()).unwrap();
        store.install_remote_snapshot(&[
            node("selected-A", None, true), node("file", Some("selected-A"), false),
        ], "1", "synthetic").unwrap();
        store.ensure_fs_root("selected-A").unwrap();
        let index = build(&store).unwrap();
        assert_eq!(index.entry_by_ino(1).unwrap().node.as_ref(), "selected-A");
        assert_eq!(index.entry_by_ino(1).unwrap().parent, 1);
        assert_eq!(index.dir_block(1).unwrap().entries[1].ino, 1);
        assert!(index.lookup(1, "file").is_some());
    }

    #[test]
    fn ambiguous_names_use_sql_error_and_preserve_directory_order() {
        let (_path, store) = fixture();
        store.db.execute("INSERT INTO nodes(id,parent,name,remote_version,directory)
            VALUES('other','A','file','1',0)", []).unwrap();
        store.db.execute("INSERT INTO node_metadata VALUES('other',3,1700000000,420)", []).unwrap();
        store.db.execute("INSERT INTO fs_inodes(node) VALUES('other')", []).unwrap();
        let parent = ino_for(&store, "A").unwrap();
        let index = build(&store).unwrap();
        assert!(index.lookup(parent, "file").is_none());
        assert!(store.posix_lookup_by_inode(parent, "file").is_err());
        let expected = store.directory_snapshot(parent, MAX_BYTES).unwrap();
        let actual = index.dir_block(parent).unwrap();
        assert_eq!(actual.entries.iter().map(|entry| entry.ino).collect::<Vec<_>>(),
            expected.entries.iter().map(|entry| entry.ino).collect::<Vec<_>>());
    }
}
