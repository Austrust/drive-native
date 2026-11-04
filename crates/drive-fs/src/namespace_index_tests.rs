//! Written in G1; execution belongs to the independent G4 verifier.
use super::*;
use drive_core::{NodeMetadata, SharedStore};

fn fixture() -> (tempfile::TempDir, Arc<SharedStore>, Arc<Binding>, NotifyHub) {
    let temp = tempfile::tempdir().unwrap();
    let mut store = Store::open(temp.path()).unwrap();
    let node = |id: &str, parent: Option<&str>, directory: bool| Node {
        id: id.into(), name: id.into(), parent: parent.map(str::to_owned),
        version: "v1".into(), directory,
        metadata: Some(NodeMetadata { size: if directory { 0 } else { 3 },
            mtime: 123, mode: if directory { 0o755 } else { 0o644 } }),
    };
    store.install_remote_snapshot(&[node("root", None, true), node("a", Some("root"), false)], "1", "synthetic").unwrap();
    store.ensure_fs_root("root").unwrap();
    store.ensure_filesystem_inodes("root").unwrap();
    let index = Arc::new(NamespaceIndex::build(&store).unwrap());
    let store = Arc::new(SharedStore::new(store));
    let binding = Arc::new(Binding::new(index, &store));
    let notify = NotifyHub::with_ttl_ms(1000);
    (temp, store, binding, notify)
}

#[test]
fn namespace_hit_preserves_posix_projection_and_never_leases_names() {
    let (_temp, store, binding, notify) = fixture();
    let expected = store.lock().unwrap().posix_lookup_by_inode(1, "a").unwrap();
    let lease = binding.lease(&store, &notify).unwrap();
    let slot = binding.index.lookup(1, "a").unwrap();
    let actual = attr(&slot, 42, 43);
    assert_eq!((actual.ino, actual.size, actual.blocks, actual.perm),
        (expected.ino, expected.size, expected.blocks, expected.mode as u16));
    assert_eq!((actual.uid, actual.gid), (42, 43));
    assert_eq!(actual.mtime, UNIX_EPOCH + Duration::from_secs(expected.mtime));
    notify.activate_ttl_for_tests(1000);
    assert_eq!(preview::reply_ttls(&notify, true, true), (ENTRY_TTL, Duration::from_millis(1000)));
    assert!(ENTRY_TTL.is_zero());
    // A metadata publication lease excludes writers without taking Store.
    assert!(matches!(store.try_lock(), Err(std::sync::TryLockError::WouldBlock)));
    drop(lease);
}

#[test]
fn dropped_notification_cannot_keep_an_old_generation_current() {
    let (_temp, store, binding, notify) = fixture();
    // No consumer subscribes to or drains commit notifications.
    store.lock().unwrap().create_local_directory("root", "added", 0o755).unwrap();
    assert!(binding.lease(&store, &notify).is_none());
    binding.refresh(&store);
    let lease = binding.lease(&store, &notify).unwrap();
    assert!(binding.index.lookup(1, "added").unwrap().directory);
    drop(lease);
}

#[test]
fn local_atime_invalidates_even_without_semantic_status_notification() {
    let (_temp, store, binding, notify) = fixture();
    let semantic = store.change_state();
    store.lock().unwrap().set_local_atime("a", 234, 567).unwrap();
    assert_eq!(semantic, store.change_state());
    assert!(binding.lease(&store, &notify).is_none());
    binding.refresh(&store);
    let lease = binding.lease(&store, &notify).unwrap();
    assert_eq!(binding.index.lookup(1, "a").unwrap().atime, (234, 567));
    drop(lease);
}

#[test]
fn notify_failure_forces_sql_even_at_unchanged_revision() {
    let (_temp, store, binding, notify) = fixture();
    assert!(binding.lease(&store, &notify).is_some());
    notify.invalidate_delivery_failed();
    assert!(binding.lease(&store, &notify).is_none());
    binding.refresh(&store);
    assert!(binding.lease(&store, &notify).is_none());
    assert!(store.lock().unwrap().posix_lookup_by_inode(1, "a").is_ok());
}

#[test]
fn dirty_draft_is_not_an_index_attribute_hit() {
    let (_temp, store, binding, notify) = fixture();
    let draft = { let mut guard = store.lock().unwrap();
        let draft = guard.begin_draft("a", "v1", None).unwrap();
        guard.draft_write(&draft, 0, b"modified local bytes", false).unwrap(); draft };
    binding.refresh(&store);
    let lease = binding.lease(&store, &notify).unwrap();
    assert!(binding.index.lookup(1, "a").is_none());
    assert!(binding.index.dir_block(1).unwrap().entries.iter().any(|entry| entry.name.as_ref() == "a"));
    drop(lease);
    let actual = store.lock().unwrap().posix_lookup_by_inode(1, "a").unwrap();
    assert_eq!(actual.size, 20);
    assert!(!actual.cacheable);
    drop(draft);
}

#[test]
fn busy_projection_falls_back_without_waiting_for_store() {
    let (_temp, store, binding, notify) = fixture();
    let held = store.lock().unwrap();
    assert!(binding.lease(&store, &notify).is_none());
    drop(held);
    assert!(binding.lease(&store, &notify).is_some());
}

#[test]
fn separate_mount_indexes_do_not_share_generations_or_identities() {
    let (_a, first, a, notify_a) = fixture();
    let (_b, second, b, notify_b) = fixture();
    first.lock().unwrap().create_local_directory("root", "only-first", 0o755).unwrap();
    a.refresh(&first);
    let lease = b.lease(&second, &notify_b).unwrap();
    assert!(b.index.lookup(1, "only-first").is_none());
    drop(lease);
    let lease = a.lease(&first, &notify_a).unwrap();
    assert!(a.index.lookup(1, "only-first").is_some());
    drop(lease);
}

#[test]
fn explicit_hint_refreshes_without_a_store_revision_change() {
    let (_temp, store, binding, notify) = fixture();
    let state = store.projection_state();
    binding.index.invalidate(drive_core::namespace_index::InvalidateHint::All);
    assert!(binding.lease(&store, &notify).is_none());
    assert!(binding.needs_refresh(&store));
    binding.refresh(&store);
    assert_eq!(store.projection_state(), state);
    assert!(binding.lease(&store, &notify).is_some());
}
