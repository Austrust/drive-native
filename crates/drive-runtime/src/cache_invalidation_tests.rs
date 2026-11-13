//! Issue #90-A regression guard: the legacy `maintain_cache` queued one
//! `Invalidation::Inode` per released cache body after dropping the
//! flights/Store locks. The bounded rewrite must keep that notification
//! contract on every return path, including a round resumed across calls.
//!
//! The test-only capture is wired directly into `Bridge::enqueue_invalidation`,
//! so a regression that stops calling it fails here even if every other
//! maintenance assertion stays green.
use super::*;
use drive_core::{DownloadTicket, Node, NodeMetadata, SharedStore, Store, sha256};

fn node(id: &str, parent: Option<&str>, directory: bool) -> Node {
    Node {
        id: id.into(),
        parent: parent.map(str::to_owned),
        name: id.into(),
        version: "1".into(),
        directory,
        metadata: Some(NodeMetadata {
            size: 11,
            mtime: 1,
            mode: if directory { 0o755 } else { 0o644 },
        }),
    }
}

fn blob_files(dir: &tempfile::TempDir) -> usize {
    std::fs::read_dir(dir.path().join("blobs")).unwrap().count()
}

fn inode_invalidations(bridge: &Bridge) -> Vec<u64> {
    bridge
        .test_invalidations()
        .into_iter()
        .filter_map(|invalidation| match invalidation {
            Invalidation::Inode(ino) => Some(ino),
            _ => None,
        })
        .collect()
}

/// One cacheable body over a 1-byte limit. The publication grace is expired so
/// the eviction path selects it immediately, and the file has an inode mapping
/// like any previously opened cached file.
fn bridge_with_releasable_body() -> (tempfile::TempDir, Arc<Bridge>, u64) {
    let dir = tempfile::tempdir().unwrap();
    let mut store = Store::open(dir.path()).unwrap();
    std::fs::create_dir_all(dir.path().join("blobs")).unwrap();
    std::fs::create_dir_all(dir.path().join("drafts")).unwrap();
    store
        .install_remote_snapshot(
            &[node("root", None, true), node("file", Some("root"), false)],
            "1",
            "server",
        )
        .unwrap();
    store.ensure_fs_root("root").unwrap();
    store
        .publish_download(
            &DownloadTicket {
                id: "file".into(),
                version: "1".into(),
            },
            b"cached-body",
            &sha256(b"cached-body"),
        )
        .unwrap();
    let ino = store.record("file").unwrap().ino;
    store.expire_cache_publication_for_test("file").unwrap();
    store.set_cache_limit(1).unwrap();
    let bridge = Arc::new(Bridge::offline(Arc::new(SharedStore::new(store))).unwrap());
    (dir, bridge, ino)
}

#[test]
fn released_body_publishes_its_inode_invalidation_once() {
    let (_dir, bridge, ino) = bridge_with_releasable_body();
    let trim = bridge.maintain_cache().unwrap();
    assert_eq!(trim.released, vec!["file".to_owned()]);
    assert_eq!(
        inode_invalidations(&bridge),
        vec![ino],
        "a released cache body must hand its inode invalidation to the notification path"
    );
    // A follow-up maintenance round has nothing left to release: the
    // invalidation must not be queued again.
    let follow_up = bridge.maintain_cache().unwrap();
    assert!(follow_up.released.is_empty(), "{:?}", follow_up.maintenance);
    assert_eq!(
        inode_invalidations(&bridge)
            .into_iter()
            .filter(|candidate| *candidate == ino)
            .count(),
        1,
        "the released inode must be invalidated exactly once"
    );
}

#[test]
fn resumed_round_publishes_each_released_inode_once() {
    let (dir, bridge, ino) = bridge_with_releasable_body();
    // More candidates than one call's window budget: the first calls return
    // mid-round, and the release + its invalidation happen in a resumed call.
    let n = drive_core::cache_policy::GC_BLOB_BATCH_SIZE
        * drive_core::cache_policy::GC_MAX_WINDOWS_PER_CALL
        + 512;
    for _ in 0..n {
        let name = uuid::Uuid::new_v4().to_string();
        std::fs::write(dir.path().join("blobs").join(name), b"orphan").unwrap();
    }
    let mut calls = 0;
    let mut saw_resumed = false;
    loop {
        let trim = bridge.maintain_cache().unwrap();
        calls += 1;
        saw_resumed |= trim.maintenance.resumed;
        if blob_files(&dir) == 1 {
            break;
        }
        assert!(calls < 32, "bounded windows must eventually reclaim the orphans");
    }
    assert!(saw_resumed, "the round must have been resumed across calls");
    assert_eq!(
        inode_invalidations(&bridge),
        vec![ino],
        "a release in a resumed round must emit its inode invalidation exactly once"
    );
    assert_eq!(blob_files(&dir), 1, "the released body file awaits the next GC round");
}
