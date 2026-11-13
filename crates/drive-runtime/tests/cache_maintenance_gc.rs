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
    DownloadTicket, Node, SharedStore, Store, sha256,
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
    println!(
        "BOUNDED_SCAN n={N} windows={} flights_windows={} max_window_blobs={} max_store_hold_ns={} total_store_hold_ns={} removed={} removed_bytes={}",
        stats.store_lock_windows,
        stats.flights_lock_windows,
        stats.max_blobs_per_window,
        stats.max_store_lock_hold_ns,
        stats.total_store_lock_hold_ns,
        stats.blobs_removed,
        stats.removed_bytes,
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
    println!("CURSOR_RESUME n={n} calls={calls} resumed={resumed}");
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
    println!(
        "CLEAN_DRAFT c={C} rows={} commits={} windows={} max_store_hold_ns={}",
        stats.clean_draft_rows,
        stats.clean_draft_commits,
        stats.store_lock_windows,
        stats.max_store_lock_hold_ns,
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
    println!("NO_PROGRESS_DEFER deferred={} windows={}", stats.deferred, stats.store_lock_windows);
}

/// Heavy mechanism evidence (#90-A): the same tree, same machine, compares the
/// legacy "one trim per call" hold against the bounded windows. Compile in the
/// build slot; run on the remote heavy (exclusive timing) slot only.
///
///   cargo test -p drive-runtime --locked --test cache_maintenance_gc -- --ignored --nocapture
#[test]
#[ignore = "heavy mechanism timing; run on the remote heavy slot"]
fn gc_maintenance_legacy_vs_bounded_hold_heavy() {
    use std::time::Instant;
    for n in [1_000usize, 10_000] {
        // Baseline shape: one Store-lock hold runs the complete legacy trim
        // (collect_garbage + content_bytes + cache_usage) over the whole
        // directory. `flights` is private to the bridge, so the Store half is
        // measured here; the legacy code also holds `flights` across the same
        // span and acquires it without measurable work.
        let (dir, bridge) = bridge_fixture();
        write_orphan_blobs(&dir, n);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let started = Instant::now();
        {
            let mut store = bridge.store.lock().unwrap();
            store
                .trim_cache_protected(
                    now,
                    &std::collections::HashSet::new(),
                    &std::collections::HashSet::new(),
                )
                .unwrap();
        }
        let legacy_hold = started.elapsed();
        assert_eq!(blob_files(&dir), 0);

        // Bounded shape: lock-free discovery + 128-candidate re-verified windows.
        let (dir, bridge) = bridge_fixture();
        write_orphan_blobs(&dir, n);
        let mut calls = 0;
        let started = Instant::now();
        let mut max_hold_ns = 0u64;
        let mut total_hold_ns = 0u64;
        let mut windows = 0u32;
        let mut removed = 0u64;
        loop {
            let stats = bridge.maintain_cache().unwrap().maintenance;
            calls += 1;
            max_hold_ns = max_hold_ns.max(stats.max_store_lock_hold_ns);
            total_hold_ns = total_hold_ns.saturating_add(stats.total_store_lock_hold_ns);
            windows = windows.saturating_add(stats.store_lock_windows);
            removed = removed.saturating_add(stats.blobs_removed);
            if blob_files(&dir) == 0 || calls > 64 {
                break;
            }
        }
        let bounded_elapsed = started.elapsed();
        println!(
            "HEAVY_A n={n} legacy_single_hold_ms={:.3} bounded_max_hold_ms={:.3} bounded_total_hold_ms={:.3} windows={windows} calls={calls} elapsed_ms={:.3} removed={removed}",
            legacy_hold.as_secs_f64() * 1e3,
            max_hold_ns as f64 / 1e6,
            total_hold_ns as f64 / 1e6,
            bounded_elapsed.as_secs_f64() * 1e3,
        );
        assert_eq!(blob_files(&dir), 0);
    }
}

/// #90-A acceptance: while one maintenance round is progressing, other Store
/// users must keep acquiring the lock between windows (the legacy shape held it
/// for the whole trim).
#[test]
fn background_store_users_progress_between_windows() {
    let (dir, bridge) = bridge_fixture();
    const N: usize = 4_096;
    write_orphan_blobs(&dir, N);
    let counter = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let stop = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let worker = {
        let bridge = bridge.clone();
        let counter = counter.clone();
        let stop = stop.clone();
        std::thread::spawn(move || {
            use std::sync::atomic::Ordering;
            while !stop.load(Ordering::Relaxed) {
                {
                    let store = bridge.store.lock().unwrap();
                    let _ = store.local_revision();
                }
                counter.fetch_add(1, Ordering::Relaxed);
            }
        })
    };
    let mut calls = 0;
    while blob_files(&dir) > 0 && calls < 16 {
        bridge.maintain_cache().unwrap();
        calls += 1;
    }
    stop.store(true, std::sync::atomic::Ordering::Relaxed);
    worker.join().unwrap();
    assert_eq!(blob_files(&dir), 0);
    let acquisitions = counter.load(std::sync::atomic::Ordering::Relaxed);
    assert!(
        acquisitions > 0,
        "another Store user must acquire the lock between bounded windows"
    );
    println!("BACKGROUND_PROGRESS acquisitions={acquisitions} calls={calls}");
}

/// #90-A regression guard: the legacy `maintain_cache` mapped each released
/// cache body to its inode under the Store lock and queued one
/// `Invalidation::Inode` after dropping the locks, so the kernel drops the
/// now-invalid cached body. The bounded path must keep that contract.
#[test]
fn bounded_trim_publishes_inode_invalidations_for_released_ids() {
    let (_dir, bridge) = bridge_fixture();
    {
        let mut store = bridge.store.lock().unwrap();
        store
            .install_remote_snapshot(
                &[
                    Node {
                        id: "root".into(),
                        parent: None,
                        name: "root".into(),
                        version: "1".into(),
                        directory: true,
                        metadata: None,
                    },
                    Node {
                        id: "file".into(),
                        parent: Some("root".into()),
                        name: "file".into(),
                        version: "1".into(),
                        directory: false,
                        metadata: Some(drive_core::NodeMetadata {
                            size: 11,
                            mtime: 1,
                            mode: 0o644,
                        }),
                    },
                ],
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
        // A previously opened cached file already has an inode mapping.
        assert!(store.record("file").unwrap().ino > 0);
        // The 60 s recent-publication grace would hide a just-published body
        // from the eviction path; expire it so this test drives the release
        // step and its invalidation publication.
        store.expire_cache_publication_for_test("file").unwrap();
        store.set_cache_limit(1).unwrap();
    }
    let trim = bridge.maintain_cache().unwrap();
    assert_eq!(
        trim.released.len(),
        1,
        "expected the one over-limit cached body to be released: {:?}",
        trim.usage
    );
    assert_eq!(
        trim.maintenance.released_inode_invalidations,
        trim.released.len() as u64,
        "every released id must publish exactly one Inode invalidation"
    );
    println!(
        "RELEASE_INVALIDATIONS released={} invalidations={}",
        trim.released.len(),
        trim.maintenance.released_inode_invalidations
    );
}
