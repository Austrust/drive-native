use super::*;
use drive_core::initialization_store::{InitializationBinding, InitializationBudget};

fn node(id: &str, parent: Option<&str>, name: &str, directory: bool) -> Node {
    Node {
        id: id.into(),
        parent: parent.map(str::to_owned),
        name: name.into(),
        directory,
        version: "v1".into(),
        metadata: Some(drive_core::NodeMetadata {
            size: if directory { 0 } else { 3 },
            mtime: 1,
            mode: if directory { 0o755 } else { 0o644 },
        }),
    }
}
fn fixture() -> (tempfile::TempDir, drive_core::SharedStore) {
    let temp = tempfile::tempdir().unwrap();
    let mut store = Store::open(temp.path()).unwrap();
    store
        .install_remote_snapshot(
            &[
                node("100", None, "root", true),
                node("101", Some("100"), "a", false),
                node("200", Some("100"), "dir", true),
                node("201", Some("200"), "child", false),
            ],
            "native-v1:7",
            "server",
        )
        .unwrap();
    store.ensure_fs_root("100").unwrap();
    (temp, drive_core::SharedStore::new(store))
}
fn open(views: &Views, store: &drive_core::SharedStore) -> (u64, Arc<View>) {
    let handle = views.open(store, 1).unwrap();
    (handle, views.get(1, handle).unwrap())
}
fn names(view: &View) -> Vec<String> {
    view.snapshot()
        .entries
        .iter()
        .map(|entry| entry.name.clone())
        .collect()
}

#[test]
fn closed_hot_view_reuses_allocations_and_semantic_commit_invalidates_it() {
    let (_temp, store) = fixture();
    let views = Views::default();
    let (fh, first) = open(&views, &store);
    let weak = Arc::downgrade(&first);
    let bytes = first.snapshot().bytes;
    drop(first);
    views.release(1, fh).unwrap();
    assert!(weak.upgrade().is_some());
    let (fh, second) = open(&views, &store);
    assert!(Arc::ptr_eq(&weak.upgrade().unwrap(), &second));
    assert_eq!(views.charged.load(Ordering::Acquire), bytes);
    drop(second);
    views.release(1, fh).unwrap();
    store
        .lock()
        .unwrap()
        .create_local_directory("100", "added", 0o755)
        .unwrap();
    let (fh, current) = open(&views, &store);
    assert!(weak.upgrade().is_none());
    assert!(names(&current).contains(&"added".into()));
    drop(current);
    views.release(1, fh).unwrap();
    views.trim_hot(None);
    assert_eq!(views.charged.load(Ordering::Acquire), 0);
}

#[test]
fn hot_cache_is_bounded_and_eviction_does_not_destroy_held_cookies() {
    let (_temp, store) = fixture();
    {
        let mut s = store.lock().unwrap();
        for i in 0..HOT_VIEWS + 12 {
            s.create_local_directory("100", &format!("d{i:04}"), 0o755)
                .unwrap();
        }
    }
    let views = Views::default();
    let (root, held) = open(&views, &store);
    let children = held
        .snapshot()
        .entries
        .iter()
        .filter(|e| e.directory && e.name.starts_with('d'))
        .map(|e| e.ino)
        .collect::<Vec<_>>();
    for ino in children {
        let fh = views.open(&store, ino).unwrap();
        views.release(ino, fh).unwrap();
    }
    let hot = views.hot.lock().unwrap();
    assert!(hot.views.len() <= HOT_VIEWS);
    assert!(hot.bytes <= HOT_BYTES);
    drop(hot);
    assert!(views.get(1, root).is_ok());
    assert_eq!(held.snapshot().entries.len(), HOT_VIEWS + 16);
    views.release(1, root).unwrap();
    drop(held);
    views.trim_hot(None);
}

#[test]
fn candidate_receipts_and_inode_housekeeping_reuse_the_same_held_view() {
    let (_temp, store) = fixture();
    let views = Views::default();
    let (_, first) = open(&views, &store);
    let bytes = views.charged.load(Ordering::Acquire);
    let initial_revision = store.lock().unwrap().local_revision();
    {
        let mut store = store.lock().unwrap();
        let stamp = store.directory_view_stamp().unwrap();
        store.record("201").unwrap(); // allocates an inode absent from root view
        let scope = r#"{"view_id":1,"node_id":1,"file_id":"100","path":"/"}"#;
        store.bind_native_scope("server", scope).unwrap();
        let binding = InitializationBinding {
            task_id: "synthetic-candidate".into(),
            scope: scope.parse().unwrap(),
            restore: "server".into(),
            start_cursor: "native-v1:100".into(),
            expected_cursor: "native-v1:7".into(),
rules_digest: String::new(),
        };
        let root = store.node("100").unwrap();
        let run = store
            .begin_initialization(&binding, &root, InitializationBudget::default())
            .unwrap();
        let ticket = store.begin_initialization_directory(&run, "100").unwrap();
        store
            .store_initialization_directory(
                &ticket,
                1,
                &[node("200", Some("100"), "private-dir", true)],
            )
            .unwrap();
        for _ in 0..32 {
            let ticket = store.begin_initialization_directory(&run, "200").unwrap();
            store
                .store_initialization_directory(&ticket, 1, &[])
                .unwrap();
        }
        assert!(store.local_revision() > initial_revision);
        assert_eq!(store.directory_view_stamp().unwrap(), stamp);
        assert!(store.initialization_writeback_blocked().unwrap());
    }
    let (_, second) = open(&views, &store);
    assert!(Arc::ptr_eq(&first, &second));
    assert_eq!(names(&second), [".", "..", "a", "dir"]);
    assert_eq!(views.charged.load(Ordering::Acquire), bytes);
    assert_eq!(views.allocated.load(Ordering::Acquire), 1);
}

#[test]
fn authoritative_rename_delete_and_create_rebuild_new_opens_but_hold_old_cookies() {
    let (_temp, store) = fixture();
    let views = Views::default();
    let (_, first) = open(&views, &store);
    store
        .lock()
        .unwrap()
        .rename_local("100", "a", "100", "renamed", false)
        .unwrap();
    let (_, renamed) = open(&views, &store);
    assert!(!Arc::ptr_eq(&first, &renamed));
    assert_eq!(names(&first), [".", "..", "a", "dir"]);
    assert_eq!(names(&renamed), [".", "..", "dir", "renamed"]);
    store
        .lock()
        .unwrap()
        .unlink_local("100", "renamed", "101", false)
        .unwrap();
    let (_, deleted) = open(&views, &store);
    assert!(!Arc::ptr_eq(&renamed, &deleted));
    assert_eq!(names(&deleted), [".", "..", "dir"]);
    store
        .lock()
        .unwrap()
        .create_local_directory("100", "local", 0o755)
        .unwrap();
    let (_, created) = open(&views, &store);
    assert!(!Arc::ptr_eq(&deleted, &created));
    assert_eq!(names(&created), [".", "..", "dir", "local"]);
    // Old immutable names/cookie indexes have not been rewritten in place.
    assert_eq!(names(&renamed), [".", "..", "dir", "renamed"]);
}

#[test]
fn changed_store_epoch_never_reuses_a_view_with_the_same_sequence() {
    let (_temp, store) = fixture();
    let views = Views::default();
    let (_, first) = open(&views, &store);
    let other = tempfile::tempdir().unwrap();
    let mut replacement = Store::open(other.path()).unwrap();
    replacement
        .install_remote_snapshot(
            &[
                node("100", None, "root", true),
                node("999", Some("100"), "replacement", false),
            ],
            "native-v1:7",
            "server",
        )
        .unwrap();
    replacement.ensure_fs_root("100").unwrap();
    {
        let mut old = store.lock().unwrap();
        assert_eq!(
            old.status_change_watermark().unwrap().cursor.sequence,
            replacement
                .status_change_watermark()
                .unwrap()
                .cursor
                .sequence
        );
        *old = replacement;
    }
    let (_, current) = open(&views, &store);
    assert!(!Arc::ptr_eq(&first, &current));
    assert_eq!(names(&current), [".", "..", "replacement"]);
    assert_eq!(names(&first), [".", "..", "a", "dir"]);
}

#[test]
fn cache_selection_waits_for_the_store_commit_and_reads_the_current_namespace() {
    let (_temp, store) = fixture();
    let store = Arc::new(store);
    let views = Arc::new(Views::default());
    let (_, first) = open(&views, &store);
    let mut guard = store.lock().unwrap();
    let (started, observed) = std::sync::mpsc::channel();
    let reader = {
        let store = store.clone();
        let views = views.clone();
        std::thread::spawn(move || {
            started.send(()).unwrap();
            open(&views, &store).1
        })
    };
    observed.recv_timeout(Duration::from_secs(2)).unwrap();
    guard
        .rename_local("100", "a", "100", "committed", false)
        .unwrap();
    drop(guard);
    let current = reader.join().unwrap();
    assert!(!Arc::ptr_eq(&first, &current));
    assert_eq!(names(&current), [".", "..", "committed", "dir"]);
}
