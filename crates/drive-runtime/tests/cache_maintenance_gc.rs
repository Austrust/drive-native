//! Issue #90-A/B integration evidence at the runtime boundary.
//!
//! One `Bridge::maintain_cache` call used to perform the complete
//! `collect_garbage` -> `content_bytes` -> optional second GC -> `cache_usage`
//! sequence while holding the flights lock *and* the Store lock, so every blob /
//! draft directory walk, deletion and directory sync (and the C clean-draft
//! autocommits) shared one uninterrupted critical section. These tests drive the
//! real maintenance path and read its additive `CacheMaintenanceStats`:
//!
//! * the blob work must be split into lock windows bounded by
//!   `GC_BLOB_BATCH_SIZE`, with a restartable cursor for larger directories;
//! * clean-draft deletions must converge on one transaction per
//!   `GC_CLEAN_DRAFT_BATCH_SIZE` rows, metered separately from the blob work;
//! * a no-progress pressure round must defer instead of re-walking the
//!   directories on every wakeup.
use drive_core::{
    Node, SharedStore, Store,
    cache_policy::{GC_BLOB_BATCH_SIZE, GC_CLEAN_DRAFT_BATCH_SIZE, GC_MAX_WINDOWS_PER_CALL},
};
use drive_runtime::bridge::Bridge;
use std::sync::Arc;

fn bridge_fixture() -> (tempfile::TempDir, Arc<Bridge>) {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(dir.path()).unwrap();
    std::fs::create_dir_all(dir.path().join("blobs")).unwrap();
    std::fs::create_dir_all(dir.path().join("drafts")).unwrap();
    let bridge = Bridge::offline(Arc::new(SharedStore::new(store))).unwrap();
    // Large limit: exercise the collection/accounting path, not the release
    // branch (which is bounded to 256 candidates by the existing policy).
    bridge.store.lock().unwrap().set_cache_limit(1 << 40).unwrap();
    (dir, Arc::new(bridge))
}

fn write_orphan_blobs(dir: &tempfile::TempDir, count: usize) {
    for _ in 0..count {
        let name = uuid::Uuid::new_v4().to_string();
        std::fs::write(dir.path().join("blobs").join(name), b"orphan-body").unwrap();
    }
}

fn blob_files(dir: &tempfile::TempDir) -> usize {
    std::fs::read_dir(dir.path().join("blobs")).unwrap().count()
}

#[test]
fn gc_blob_reclaim_is_split_into_bounded_lock_windows() {
    let (dir, bridge) = bridge_fixture();
    const N: usize = 1_000;
    write_orphan_blobs(&dir, N);
    let trim = bridge.maintain_cache().unwrap();
    let stats = trim.maintenance;
    assert_eq!(blob_files(&dir), 0, "unreferenced blobs must be reclaimed");
    assert!(
        stats.blobs_discovered >= N as u64,
        "the round must account for the discovered volume: {stats:?}"
    );
    let expected_windows = stats.blobs_discovered.div_ceil(GC_BLOB_BATCH_SIZE as u64);
    assert!(
        stats.store_lock_windows as u64 >= expected_windows,
        "one uninterrupted Store lock hold covered {} blobs; bounded windows need >= {expected_windows} locks: {stats:?}",
        stats.blobs_discovered
    );
    assert!(
        stats.flights_lock_windows as u64 >= expected_windows,
        "flights lock must also be re-acquired per window, got {} for {expected_windows} windows: {stats:?}",
        stats.flights_lock_windows
    );
    assert!(
        stats.max_blobs_per_window <= GC_BLOB_BATCH_SIZE as u64,
        "a single lock window examined {} blobs (batch {}): {stats:?}",
        stats.max_blobs_per_window,
        GC_BLOB_BATCH_SIZE
    );
}

#[test]
fn gc_round_resumes_from_cursor_for_large_directories() {
    let (dir, bridge) = bridge_fixture();
    // More candidates than one call's window budget can re-verify.
    let n = GC_BLOB_BATCH_SIZE * GC_MAX_WINDOWS_PER_CALL * 2;
    write_orphan_blobs(&dir, n);
    let first = bridge.maintain_cache().unwrap().maintenance;
    assert!(
        blob_files(&dir) > 0,
        "the first call must stop at its bounded window budget, not run the whole directory in one hold: {first:?}"
    );
    assert_eq!(first.resumed, false);
    let mut calls = 1;
    let mut resumed = false;
    while blob_files(&dir) > 0 && calls < 16 {
        let stats = bridge.maintain_cache().unwrap().maintenance;
        resumed |= stats.resumed;
        calls += 1;
    }
    assert_eq!(blob_files(&dir), 0, "the persisted cursor must finish the round");
    assert!(resumed, "a later call must report resuming the interrupted round");
    assert!(calls < 16, "bounded windows must finish a {} blob directory", n);
}

#[test]
fn clean_draft_commits_are_batched_under_maintenance() {
    let (_dir, bridge) = bridge_fixture();
    const C: usize = 300;
    let ids: Vec<String> = (0..C).map(|index| format!("file-{index}")).collect();
    {
        let mut store = bridge.store.lock().unwrap();
        let mut nodes = vec![Node {
            id: "root".into(),
            parent: None,
            name: "root".into(),
            version: "1".into(),
            directory: true,
            metadata: None,
        }];
        nodes.extend(ids.iter().map(|id| Node {
            id: id.clone(),
            parent: Some("root".into()),
            name: id.clone(),
            version: "1".into(),
            directory: false,
            metadata: None,
        }));
        store.install_remote_snapshot(&nodes, "1", "server").unwrap();
        store.ensure_fs_root("root").unwrap();
        // Real draft lifecycle: each begin_draft inserts a clean (dirty=0) row
        // and a working body; dropping the handle closes it without a write.
        for id in &ids {
            let draft = store.begin_draft(id, "1", None).unwrap();
            drop(draft);
        }
    }
    let trim = bridge.maintain_cache().unwrap();
    let stats = trim.maintenance;
    assert_eq!(
        stats.clean_draft_rows, C as u64,
        "every clean closed draft row must be deleted: {stats:?}"
    );
    assert_eq!(
        stats.clean_draft_commits,
        C.div_ceil(GC_CLEAN_DRAFT_BATCH_SIZE) as u64,
        "#90-B red: per-row autocommit made {C} commits; bounded batches must make ceil({C}/{GC_CLEAN_DRAFT_BATCH_SIZE})"
    );
    assert_eq!(
        std::fs::read_dir(bridge.store.root().join("drafts"))
            .unwrap()
            .count(),
        0,
        "a deleted clean draft must take its body with it"
    );
}

#[test]
fn no_progress_pressure_is_deferred_instead_of_rescanned() {
    let (_dir, bridge) = bridge_fixture();
    // Nothing reclaimable: the first round scans and reclaims zero bytes.
    let first = bridge.maintain_cache().unwrap();
    assert!(!first.maintenance.deferred);
    // The cache loop keeps calling while `over_target_bytes > 0`; an unchanged
    // revision with no reclaimable progress must not repeat the directory walk.
    let second = bridge.maintain_cache().unwrap();
    let stats = second.maintenance;
    assert!(
        stats.deferred,
        "a no-progress pressure round must be deferred instead of re-scanning: {stats:?}"
    );
    assert_eq!(stats.store_lock_windows, 0, "deferred work takes no Store lock");
    assert_eq!(stats.blobs_examined, 0, "deferred work examines no blobs");
}
