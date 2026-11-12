//! Projection-oracle and incremental-correctness tests for the status cache.
//!
//! The oracle is the unchanged full seed query: after every committed change the
//! published projection must be row-for-row equal to a fresh
//! `Store::open_read_only` scan of the same database.
use super::{
    CacheConfig, CacheState, DegradeReason, FaultKind, Faults, MAX_CACHED_NODES, StatusCache,
    StatusCacheMetrics, StatusProjection, Worker,
};
use crate::service::{Profile, Service};
use drive_core::{
    DownloadTicket, Node, NodeMetadata, Store, desktop_status::StatusSeed,
    status_changes::JournalBudget,
};
use std::{
    path::PathBuf,
    sync::{Arc, atomic::Ordering, mpsc},
    time::{Duration, Instant},
};

type SeedTuple = (String, Option<String>, bool, bool, bool, bool);

fn node(id: &str, parent: Option<&str>, directory: bool) -> Node {
    Node {
        id: id.into(),
        parent: parent.map(str::to_owned),
        name: id.into(),
        version: "v1".into(),
        directory,
        metadata: Some(NodeMetadata {
            size: 3,
            mtime: 1,
            mode: if directory { 0o755 } else { 0o644 },
        }),
    }
}

fn profile() -> Profile {
    Profile {
        schema: 1,
        task_id: uuid::Uuid::new_v4().to_string(),
        host: "example.invalid".into(),
        tls_name: "example.invalid".into(),
        trusted_ca_pem: Vec::new(),
        username: "test".into(),
        device_id: uuid::Uuid::new_v4().to_string(),
        scope: drive_core::native::Scope {
            view_id: 1,
            node_id: 1,
            file_id: "root".into(),
            path: "/test".into(),
        },
        mount: PathBuf::from("/tmp/drive-native-test-mount"),
        writable: false,
        local_policy: None,
        writable_source: None,
        exclude_patterns: Vec::new(),
        exclude_mode: drive_core::exclude::ExcludeMode::default(),
    }
}

fn fixture() -> (tempfile::TempDir, Arc<Service>) {
    let dir = tempfile::tempdir().unwrap();
    let mut store = Store::open(dir.path()).unwrap();
    store
        .install_remote_snapshot(
            &[
                node("root", None, true),
                node("folder", Some("root"), true),
                node("one", Some("folder"), false),
                node("two", Some("folder"), false),
            ],
            "1",
            "server",
        )
        .unwrap();
    store.ensure_fs_root("root").unwrap();
    let service = Service::new(profile(), store, PathBuf::from("/nonexistent-worker")).unwrap();
    (dir, service)
}

fn start(service: &Arc<Service>, limit: usize) -> (Arc<StatusCache>, std::thread::JoinHandle<()>) {
    StatusCache::start_with_limit(service.clone(), limit)
}

fn tuple(seed: &StatusSeed) -> SeedTuple {
    (
        seed.id.clone(),
        seed.parent.clone(),
        seed.directory,
        seed.pinned,
        seed.overlay,
        seed.detailed,
    )
}

fn sorted(mut rows: Vec<SeedTuple>) -> Vec<SeedTuple> {
    rows.sort();
    rows
}

fn full_rows(service: &Service) -> Vec<SeedTuple> {
    let store = Store::open_read_only(&service.bridge.store.root()).unwrap();
    sorted(
        store
            .desktop_status_seeds()
            .unwrap()
            .iter()
            .map(tuple)
            .collect(),
    )
}

/// Visible child edges of `parent`, sorted the way a full seed scan orders them.
fn full_children(service: &Service, parent: &str) -> Vec<(String, bool)> {
    let store = Store::open_read_only(&service.bridge.store.root()).unwrap();
    let mut children: Vec<(String, bool)> = store
        .desktop_status_seeds()
        .unwrap()
        .iter()
        .filter(|seed| seed.parent.as_deref() == Some(parent))
        .map(|seed| (seed.id.clone(), seed.directory))
        .collect();
    children.sort_by(|left, right| left.0.cmp(&right.0));
    children
}

/// Synthetic projection rows for the structural resident test. Installing them
/// through `Worker::install_full` needs no SQLite fixture: the test only compares
/// the worker's resident structures with the immutable base it publishes.
fn synthetic_seeds(count: usize) -> Vec<StatusSeed> {
    const DIRS: usize = 20;
    assert!(count > 1 + DIRS, "count too small");
    let mut rows = Vec::with_capacity(count);
    rows.push(StatusSeed {
        id: "root".into(),
        parent: None,
        directory: true,
        pinned: false,
        overlay: false,
        detailed: false,
    });
    for index in 0..DIRS {
        rows.push(StatusSeed {
            id: format!("d{index:03}"),
            parent: Some("root".into()),
            directory: true,
            pinned: false,
            overlay: false,
            detailed: false,
        });
    }
    for index in 0..count - 1 - DIRS {
        rows.push(StatusSeed {
            id: format!("f{index:06}"),
            parent: Some(format!("d{:03}", index % DIRS)),
            directory: false,
            pinned: index % 17 == 0,
            overlay: false,
            detailed: index % 13 == 0,
        });
    }
    rows
}

fn projection_rows(projection: &StatusProjection) -> Vec<SeedTuple> {
    sorted(projection.seeds().map(tuple).collect())
}

fn wait_ready(cache: &StatusCache) -> Arc<StatusProjection> {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if let Some(projection) = cache.snapshot() {
            return projection;
        }
        assert!(
            Instant::now() < deadline,
            "no initial status projection within 5s"
        );
        std::thread::sleep(Duration::from_millis(5));
    }
}

/// Waits for a publication newer than `previous_generation`, then checks the one
/// authoritative oracle scan. Used by the latency evidence test where polling
/// the full scan would dominate the numbers.
fn wait_generation(
    cache: &StatusCache,
    previous_generation: u64,
    label: &str,
) -> Arc<StatusProjection> {
    let deadline = Instant::now() + Duration::from_secs(8);
    loop {
        if let Some(projection) = cache.snapshot() {
            if projection.generation() > previous_generation {
                return projection;
            }
        }
        assert!(
            Instant::now() < deadline,
            "no projection newer than generation {previous_generation} after {label}"
        );
        std::thread::sleep(Duration::from_millis(5));
    }
}

/// Waits until a projection newer than `previous_generation` matches a fresh
/// full scan. Returns that projection.
fn wait_matches(
    service: &Service,
    cache: &StatusCache,
    label: &str,
    previous_generation: u64,
) -> Arc<StatusProjection> {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if let Some(projection) = cache.snapshot() {
            let generation_advanced = projection.generation() > previous_generation;
            if generation_advanced && projection_rows(&projection) == full_rows(service) {
                return projection;
            }
        }
        if Instant::now() >= deadline {
            let published = cache.snapshot();
            match published {
                Some(projection) => panic!(
                    "projection did not converge after {label}: generation {} rows {:?} != full {:?}",
                    projection.generation(),
                    projection_rows(&projection),
                    full_rows(service)
                ),
                None => panic!("projection disappeared after {label}"),
            }
        }
        std::thread::sleep(Duration::from_millis(5));
    }
}

/// Stop must be observed by the worker and the join must be bounded (design:
/// ≤2s).
fn stop_and_join(service: &Arc<Service>, task: std::thread::JoinHandle<()>) {
    service.stop.store(true, Ordering::Release);
    let (done, wait) = mpsc::channel();
    std::thread::spawn(move || {
        let _ = task.join();
        let _ = done.send(());
    });
    assert!(
        wait.recv_timeout(Duration::from_secs(2)).is_ok(),
        "status cache thread did not stop within 2s"
    );
}

#[test]
fn projection_shape_and_sorted_children_are_stable() {
    // Deliberately unsorted input: children must be published in child-id order,
    // matching the primary-key order of the authoritative scan.
    let rows = vec![
        StatusSeed {
            id: "root".into(),
            parent: None,
            directory: true,
            pinned: false,
            overlay: false,
            detailed: false,
        },
        StatusSeed {
            id: "z".into(),
            parent: Some("root".into()),
            directory: false,
            pinned: false,
            overlay: false,
            detailed: false,
        },
        StatusSeed {
            id: "child".into(),
            parent: Some("root".into()),
            directory: true,
            pinned: true,
            overlay: true,
            detailed: true,
        },
        StatusSeed {
            id: "a".into(),
            parent: Some("root".into()),
            directory: false,
            pinned: false,
            overlay: false,
            detailed: false,
        },
    ];
    let projection = StatusProjection::from_rows(rows, 7);
    assert_eq!(projection.len(), 4);
    assert_eq!(projection.generation(), 7);
    assert_eq!(
        projection.children("root"),
        &[
            ("a".to_owned(), false),
            ("child".to_owned(), true),
            ("z".to_owned(), false)
        ]
    );
    assert!(projection.children("child").is_empty());
    assert_eq!(projection.seed("root").unwrap().directory, true);
    assert!(projection.seed("missing").is_none());
}

#[test]
fn startup_projection_matches_the_full_scan() {
    let (_dir, service) = fixture();
    let (cache, task) = start(&service, MAX_CACHED_NODES);
    let projection = wait_ready(&cache);
    assert_eq!(projection_rows(&projection), full_rows(&service));
    assert_eq!(projection.len(), 4);
    assert_eq!(projection.children("root").len(), 1);
    assert_eq!(projection.children("folder").len(), 2);
    assert!(projection.generation() >= 1);
    stop_and_join(&service, task);
}

#[test]
fn idle_cache_does_not_republish() {
    let (_dir, service) = fixture();
    let (cache, task) = start(&service, MAX_CACHED_NODES);
    let projection = wait_ready(&cache);
    let generation = projection.generation();
    std::thread::sleep(Duration::from_millis(900));
    let idle = cache.snapshot().unwrap();
    assert_eq!(
        idle.generation(),
        generation,
        "idle wake-ups must not publish a new generation"
    );
    stop_and_join(&service, task);
}

#[test]
fn journal_updates_flow_into_the_projection() {
    let (_dir, service) = fixture();
    let (cache, task) = start(&service, MAX_CACHED_NODES);
    let mut projection = wait_ready(&cache);

    // nodes.pinned.
    {
        let mut store = service.bridge.store.lock().unwrap();
        store.pin("folder", true).unwrap();
    }
    projection = wait_matches(&service, &cache, "pin", projection.generation());
    assert!(projection.seed("folder").unwrap().pinned);

    // nodes.dirty_blob + drafts (detailed flag and content row).
    {
        let mut store = service.bridge.store.lock().unwrap();
        store.save_local("one", b"draft-content").unwrap();
    }
    projection = wait_matches(&service, &cache, "save_local", projection.generation());
    assert!(projection.seed("one").unwrap().detailed);

    // transfers (a queued download makes the node detailed).
    {
        let mut store = service.bridge.store.lock().unwrap();
        store
            .queue_download(&DownloadTicket {
                id: "two".into(),
                version: "v1".into(),
            })
            .unwrap();
    }
    projection = wait_matches(&service, &cache, "queue_download", projection.generation());
    assert!(projection.seed("two").unwrap().detailed);

    // nodes added by a committed remote batch.
    {
        let mut store = service.bridge.store.lock().unwrap();
        store
            .receive(&drive_core::Batch {
                from: "1".into(),
                to: "2".into(),
                events: vec![drive_core::Event {
                    id: "e2".into(),
                    change: drive_core::Change::Upsert(node("three", Some("folder"), false)),
                }],
            })
            .unwrap();
        store.drain().unwrap();
    }
    projection = wait_matches(&service, &cache, "remote add", projection.generation());
    assert!(projection.seed("three").is_some());

    stop_and_join(&service, task);
}

#[test]
fn remote_moves_and_deletes_update_locations_and_rows() {
    let (_dir, service) = fixture();
    let (cache, task) = start(&service, MAX_CACHED_NODES);
    let mut projection = wait_ready(&cache);

    // Moving a node under a different parent/name must refresh both location
    // keys: the old key drops the child and the new key gains it.
    {
        let mut store = service.bridge.store.lock().unwrap();
        let mut moved = node("one", Some("root"), false);
        moved.name = "moved".into();
        store
            .receive(&drive_core::Batch {
                from: "1".into(),
                to: "2".into(),
                events: vec![drive_core::Event {
                    id: "e2".into(),
                    change: drive_core::Change::Upsert(moved),
                }],
            })
            .unwrap();
        store.drain().unwrap();
    }
    projection = wait_matches(&service, &cache, "remote move", projection.generation());
    assert_eq!(
        projection.seed("one").unwrap().parent.as_deref(),
        Some("root")
    );
    assert!(projection.children("folder").iter().all(|(id, _)| id != "one"));
    assert!(projection.children("root").iter().any(|(id, _)| id == "one"));

    // A remote delete names only the node; the row and its child entry must go.
    {
        let mut store = service.bridge.store.lock().unwrap();
        store
            .receive(&drive_core::Batch {
                from: "2".into(),
                to: "3".into(),
                events: vec![drive_core::Event {
                    id: "e3".into(),
                    change: drive_core::Change::Delete {
                        id: "one".into(),
                    },
                }],
            })
            .unwrap();
        store.drain().unwrap();
    }
    projection = wait_matches(&service, &cache, "remote delete", projection.generation());
    assert!(projection.seed("one").is_none());
    assert!(projection.children("root").iter().all(|(id, _)| id != "one"));

    stop_and_join(&service, task);
}

#[test]
fn journal_budget_is_installed_and_applied_pages_are_pruned() {
    let (_dir, service) = fixture();
    let (cache, task) = start(&service, MAX_CACHED_NODES);
    let projection = wait_ready(&cache);
    {
        let guard = service.bridge.store.lock().unwrap();
        let watermark = guard.status_change_watermark().unwrap();
        assert_eq!(watermark.budget.entries, 65_536);
        assert_eq!(watermark.budget.bytes, 16 * 1024 * 1024);
        assert_eq!(watermark.entries, 0);
    }
    {
        let mut store = service.bridge.store.lock().unwrap();
        store.pin("folder", true).unwrap();
        store.pin("one", true).unwrap();
    }
    let projection = wait_matches(&service, &cache, "prune", projection.generation());
    assert!(projection.seed("folder").unwrap().pinned);
    {
        let guard = service.bridge.store.lock().unwrap();
        let watermark = guard.status_change_watermark().unwrap();
        assert_eq!(
            watermark.entries, 0,
            "acknowledged journal pages must be pruned"
        );
    }
    stop_and_join(&service, task);
}

#[test]
fn journal_overflow_forces_a_full_rebuild() {
    let (_dir, service) = fixture();
    let (cache, task) = start(&service, MAX_CACHED_NODES);
    let mut projection = wait_ready(&cache);

    {
        let mut store = service.bridge.store.lock().unwrap();
        store
            .set_status_journal_budget(JournalBudget {
                entries: 1,
                bytes: 4096,
            })
            .unwrap();
        store.pin("folder", true).unwrap();
        store.pin("one", true).unwrap();
    }
    projection = wait_matches(&service, &cache, "overflow rebuild", projection.generation());
    assert!(projection.seed("one").unwrap().pinned);
    assert!(projection.seed("folder").unwrap().pinned);

    stop_and_join(&service, task);
}

/// Issue #25: an over-cap profile used to disable the cache permanently with no
/// observation point and no recovery. It now reports a visible degraded reason
/// (retrying in the background) and returns to service once the profile fits.
#[test]
fn oversized_profiles_degrade_the_cache_without_disabling_it() {
    let (_dir, service) = fixture();
    let (cache, task) = start(&service, 2);
    let metrics = wait_degraded(&cache, DegradeReason::TooManyNodes);
    assert_eq!(metrics.state, CacheState::Degraded);
    assert_eq!(metrics.degraded_reason, Some(DegradeReason::TooManyNodes));
    assert!(metrics.degraded_events >= 1);
    assert!(metrics.recovery_attempts >= 1);
    // No snapshot existed before the failure, but the worker keeps retrying
    // instead of shutting the projection down.
    assert!(cache.snapshot().is_none());

    // Shrink the profile below the cap: the retry cycle bootstraps and the cache
    // resumes without restarting the service.
    {
        let mut store = service.bridge.store.lock().unwrap();
        store
            .install_remote_snapshot(
                &[node("root", None, true), node("folder", Some("root"), true)],
                "2",
                "server",
            )
            .unwrap();
    }
    wait_recovered(&cache);
    wait_matches(&service, &cache, "cap recovery", 0);
    stop_and_join(&service, task);
}

/// Cost evidence for the design doc: startup snapshot, one incremental delta, a
/// forced full rebuild, and reader snapshot latency on a 60k-node namespace.
/// Run explicitly with
/// `cargo test -p drive-runtime --lib status_cache:: -- --include-ignored --nocapture`.
#[test]
#[ignore = "read-volume/latency evidence; run explicitly on the remote Linux builder"]
fn projection_startup_delta_and_rebuild_costs() {
    const DIRS: usize = 100;
    const FILES: usize = 59_900;
    let dir = tempfile::tempdir().unwrap();
    let mut store = Store::open(dir.path()).unwrap();
    let mut nodes = Vec::with_capacity(DIRS + FILES + 1);
    nodes.push(node("root", None, true));
    for index in 0..DIRS {
        nodes.push(node(&format!("d{index}"), Some("root"), true));
    }
    for index in 0..FILES {
        nodes.push(node(
            &format!("f{index}"),
            Some(&format!("d{}", index % DIRS)),
            false,
        ));
    }
    store.install_remote_snapshot(&nodes, "1", "server").unwrap();
    store.ensure_fs_root("root").unwrap();
    let profile = profile();
    let service = Service::new(profile, store, PathBuf::from("/nonexistent-worker")).unwrap();
    let namespace_rows = nodes.len();
    drop(nodes);

    // Cost of the authoritative full read the worker performs on startup and on
    // every rebuild (same query, same read-only handle).
    let scan_started = Instant::now();
    let full = full_rows(&service);
    let scan_elapsed = scan_started.elapsed();
    assert_eq!(full.len(), namespace_rows);

    let started = Instant::now();
    let (cache, task) = start(&service, MAX_CACHED_NODES);
    let projection = wait_ready(&cache);
    let startup_elapsed = started.elapsed();
    assert_eq!(projection_rows(&projection), full);

    // #67 W3: the projection must be resident once. The worker reports the heap
    // size of its projection state (shared immutable base + sparse deltas) right
    // before each publication; compare it with the base snapshot readers hold.
    let base_bytes = projection.base_resident_bytes_estimate();
    let worker_bytes = cache.worker_resident_bytes() as usize;
    println!(
        "RESIDENT projection_base_bytes={base_bytes} worker_bytes={worker_bytes} ratio={:.3}",
        worker_bytes as f64 / base_bytes as f64
    );
    assert!(
        worker_bytes <= base_bytes * 5 / 4,
        "worker holds {worker_bytes} heap bytes for a {base_bytes}-byte shared base \
         (ratio {:.2}); the projection must be resident once (#67 W3)",
        worker_bytes as f64 / base_bytes as f64
    );

    // One incremental commit: node-level invalidation only. The publication
    // throttle (250ms) means the measured latency includes at most one window.
    let before = projection.generation();
    let delta_started = Instant::now();
    {
        let mut store = service.bridge.store.lock().unwrap();
        store.pin("f1", true).unwrap();
    }
    let projection = wait_generation(&cache, before, "single delta");
    let delta_elapsed = delta_started.elapsed();
    assert!(projection.seed("f1").unwrap().pinned);
    assert_eq!(projection_rows(&projection), full_rows(&service));

    // One subtree-scoped invalidation: a directory with ~599 children moves to a
    // different parent. This exercises the incremental subtree refresh (children
    // walk + one scoped re-read) instead of a point refresh.
    let before = projection.generation();
    let subtree_started = Instant::now();
    {
        let mut store = service.bridge.store.lock().unwrap();
        let mut moved = node("d0", Some("d1"), true);
        moved.name = "d0moved".into();
        store
            .receive(&drive_core::Batch {
                from: "1".into(),
                to: "2".into(),
                events: vec![drive_core::Event {
                    id: "e2".into(),
                    change: drive_core::Change::Upsert(moved),
                }],
            })
            .unwrap();
        store.drain().unwrap();
    }
    let projection = wait_generation(&cache, before, "subtree move");
    let subtree_move_elapsed = subtree_started.elapsed();
    assert_eq!(projection.seed("d0").unwrap().parent.as_deref(), Some("d1"));
    assert!(projection.children("d0").len() > 500);
    assert_eq!(projection_rows(&projection), full_rows(&service));

    // Readers must stay fast while the worker clones a large projection: sample
    // `snapshot()` around a burst of commits that trigger publication.
    let mut worst_snapshot = Duration::ZERO;
    {
        let mut store = service.bridge.store.lock().unwrap();
        for index in 0..8 {
            let started = Instant::now();
            let _ = cache.snapshot();
            worst_snapshot = worst_snapshot.max(started.elapsed());
            store.pin(&format!("f{index}"), true).unwrap();
        }
    }
    for _ in 0..40 {
        let started = Instant::now();
        let _ = cache.snapshot();
        worst_snapshot = worst_snapshot.max(started.elapsed());
        if cache
            .snapshot()
            .is_some_and(|published| published.generation() > projection.generation())
        {
            break;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(
        worst_snapshot < Duration::from_millis(5),
        "snapshot() took {worst_snapshot:?} while publishing"
    );

    // Force the journal to overflow: the next wake must take the full-rebuild
    // path while the previous snapshot keeps serving readers. Wait for the burst
    // publication to settle first so the measurement isolates the rebuild.
    std::thread::sleep(Duration::from_millis(300));
    let before = cache
        .snapshot()
        .expect("published projection")
        .generation();
    let rebuild_started = Instant::now();
    {
        let mut store = service.bridge.store.lock().unwrap();
        store
            .set_status_journal_budget(JournalBudget {
                entries: 1,
                bytes: 4096,
            })
            .unwrap();
        store.pin("f100", true).unwrap();
        store.pin("f101", true).unwrap();
    }
    let projection = wait_generation(&cache, before, "forced rebuild");
    let rebuild_elapsed = rebuild_started.elapsed();
    assert!(projection.seed("f100").unwrap().pinned);
    assert!(projection.seed("f101").unwrap().pinned);
    assert_eq!(projection_rows(&projection), full_rows(&service));

    let rebuilds = cache.rebuilds();
    let publishes = cache.publishes();
    println!(
        "NAMESPACE rows={namespace_rows} full_scan={scan_elapsed:?} startup={startup_elapsed:?} single_delta={delta_elapsed:?} subtree_move={subtree_move_elapsed:?} forced_rebuild={rebuild_elapsed:?} projection_len={} worst_snapshot={worst_snapshot:?} rebuilds={rebuilds} publishes={publishes}",
        projection.len()
    );

    stop_and_join(&service, task);
}

/// Issue #25: growing past the node cap is a *degraded* state, not a silent
/// permanent disable. The last snapshot keeps serving stale-but-usable data and
/// the worker retries until the namespace fits the cap again.
#[test]
fn incremental_growth_past_the_cap_degrades_and_recovers() {
    let (_dir, service) = fixture();
    let (cache, task) = start(&service, 5);
    let projection = wait_ready(&cache);
    assert_eq!(projection.len(), 4);

    {
        let mut store = service.bridge.store.lock().unwrap();
        // `install_remote_snapshot` is a whole-namespace snapshot: keep the
        // existing rows and add two files that push the projection past the cap.
        let extra = [
            node("root", None, true),
            node("folder", Some("root"), true),
            node("one", Some("folder"), false),
            node("two", Some("folder"), false),
            node("x0", Some("root"), false),
            node("x1", Some("root"), false),
        ];
        store
            .install_remote_snapshot(&extra, "2", "server")
            .unwrap();
    }
    let metrics = wait_degraded(&cache, DegradeReason::TooManyNodes);
    assert_eq!(metrics.state, CacheState::Degraded);
    // The previous snapshot is still served: stale data beats no data.
    let stale = cache.snapshot().expect("degraded cache keeps its last snapshot");
    assert_eq!(stale.len(), 4);
    assert!(stale.seed("x0").is_none());

    // Shrink the profile back below the cap and let the retry cycle recover.
    {
        let mut store = service.bridge.store.lock().unwrap();
        let extra = [
            node("root", None, true),
            node("folder", Some("root"), true),
            node("one", Some("folder"), false),
            node("two", Some("folder"), false),
        ];
        store
            .install_remote_snapshot(&extra, "3", "server")
            .unwrap();
    }
    wait_recovered(&cache);
    let projection = wait_matches(&service, &cache, "cap regression", projection.generation());
    assert_eq!(projection.len(), 4);
    assert_eq!(cache.metrics().state, CacheState::Running);
    stop_and_join(&service, task);
}

#[test]
fn paged_journal_replays_more_than_one_batch() {
    let (_dir, service) = fixture();
    let (cache, task) = start(&service, MAX_CACHED_NODES);
    let projection = wait_ready(&cache);

    // A whole-namespace snapshot with 600 new rows: the journal page limit is
    // 512, so the worker must replay more than one page before it converges.
    let mut extra = vec![
        node("root", None, true),
        node("folder", Some("root"), true),
        node("one", Some("folder"), false),
        node("two", Some("folder"), false),
    ];
    for index in 0..600 {
        extra.push(node(&format!("p{index}"), Some("folder"), false));
    }
    {
        let mut store = service.bridge.store.lock().unwrap();
        store
            .install_remote_snapshot(&extra, "2", "server")
            .unwrap();
    }
    let projection = wait_matches(&service, &cache, "paged journal", projection.generation());
    assert!(projection.len() >= 600);
    stop_and_join(&service, task);
}

#[test]
fn publish_throttle_coalesces_a_burst_of_commits() {
    let (_dir, service) = fixture();
    let (cache, task) = start(&service, MAX_CACHED_NODES);
    let projection = wait_ready(&cache);
    let before = projection.generation();

    {
        let mut store = service.bridge.store.lock().unwrap();
        for index in 0..40 {
            store.pin("folder", index % 2 == 0).unwrap();
        }
    }
    let projection = wait_matches(&service, &cache, "burst", before);
    let published = projection.generation() - before;
    assert!(
        published < 20,
        "40 commits produced {published} publications; the throttle did not coalesce"
    );
    stop_and_join(&service, task);
}

#[test]
fn stop_is_bounded_while_the_writer_guard_is_held() {
    let (_dir, service) = fixture();
    let (cache, task) = start(&service, MAX_CACHED_NODES);
    let _projection = wait_ready(&cache);

    // Hold the store mutex and ask the worker to stop: the stop-aware lock loop
    // must give up polling and the join must stay bounded.
    let guard = service.bridge.store.lock().unwrap();
    service.stop.store(true, Ordering::Release);
    let (done, wait) = mpsc::channel();
    std::thread::spawn(move || {
        let _ = task.join();
        let _ = done.send(());
    });
    assert!(
        wait.recv_timeout(Duration::from_secs(2)).is_ok(),
        "status cache thread did not stop while the writer guard was held"
    );
    drop(guard);
}

#[test]
fn directory_move_keeps_the_projection_on_the_oracle() {
    let (_dir, service) = fixture();
    let (cache, task) = start(&service, MAX_CACHED_NODES);
    let mut projection = wait_ready(&cache);

    // `folder` has two children: moving it is a subtree-scoped invalidation and
    // must keep both the moved directory and its descendant rows consistent with
    // the full scan.
    {
        let mut store = service.bridge.store.lock().unwrap();
        let mut moved = node("folder", Some("root"), true);
        moved.name = "moved_folder".into();
        store
            .receive(&drive_core::Batch {
                from: "1".into(),
                to: "2".into(),
                events: vec![drive_core::Event {
                    id: "e2".into(),
                    change: drive_core::Change::Upsert(moved),
                }],
            })
            .unwrap();
        store.drain().unwrap();
    }
    projection = wait_matches(&service, &cache, "directory move", projection.generation());
    assert_eq!(
        projection.seed("folder").unwrap().parent.as_deref(),
        Some("root")
    );
    assert!(
        projection
            .children("folder")
            .iter()
            .any(|(id, _)| id == "one")
    );
    stop_and_join(&service, task);
}

#[test]
fn journal_backlog_over_the_limit_forces_a_full_rebuild() {
    let (_dir, service) = fixture();
    // Backlog seam: any span above two entries must take the rebuild path.
    let (cache, task) =
        super::StatusCache::start_with_limits(service.clone(), MAX_CACHED_NODES, 2);
    let projection = wait_ready(&cache);
    let before = projection.generation();
    let rebuilds_before = cache.rebuilds();

    {
        let mut store = service.bridge.store.lock().unwrap();
        for id in ["one", "two", "folder"] {
            store.pin(id, true).unwrap();
        }
    }
    let projection = wait_matches(&service, &cache, "backlog rebuild", before);
    assert!(
        cache.rebuilds() > rebuilds_before,
        "a journal span above the backlog limit did not force a rebuild"
    );
    assert!(projection.seed("one").unwrap().pinned);
    assert!(projection.seed("two").unwrap().pinned);
    stop_and_join(&service, task);
}

/// One public-API write path touches four journal sources at once: `nodes`,
/// `local_creations`, `namespace_overlay` and `namespace_ops`; removing the
/// local creation touches `local_unlinks` again.
#[test]
fn local_creation_and_removal_flow_into_the_projection() {
    let (_dir, service) = fixture();
    let (cache, task) = start(&service, MAX_CACHED_NODES);
    let mut projection = wait_ready(&cache);

    let created;
    {
        let mut store = service.bridge.store.lock().unwrap();
        created = store
            .create_local_directory("folder", "newdir", 0o755)
            .unwrap();
    }
    projection = wait_matches(&service, &cache, "local creation", projection.generation());
    assert!(projection.seed(&created.id).is_some());
    assert!(projection
        .children("folder")
        .iter()
        .any(|(id, directory)| id == &created.id && *directory));

    {
        let mut store = service.bridge.store.lock().unwrap();
        store
            .unlink_local("folder", "newdir", &created.id, true)
            .unwrap();
    }
    projection = wait_matches(&service, &cache, "local removal", projection.generation());
    assert!(projection.seed(&created.id).is_none());
    assert!(!projection
        .children("folder")
        .iter()
        .any(|(id, _)| id == &created.id));
    stop_and_join(&service, task);
}

/// Regression for issue #8 H1: a directory hidden by the inherited
/// `access_blocked` chain has no projected children, so "already has children"
/// alone misses the moment it becomes visible again. The invalidation trigger
/// must treat a previously hidden node as a subtree refresh; otherwise its
/// descendants are never re-projected and stay missing until a full rebuild,
/// while the directory's aggregate status is silently wrong.
#[test]
fn hidden_to_visible_subtree_invalidations_are_recognized() {
    use super::{build_children, subtree_needs_refresh};
    use drive_core::status_changes::{ChangeScope, StatusChange};
    use std::collections::HashMap;

    fn seed(id: &str, parent: Option<&str>, directory: bool) -> StatusSeed {
        StatusSeed {
            id: id.into(),
            parent: parent.map(str::to_owned),
            directory,
            pinned: false,
            overlay: false,
            detailed: false,
        }
    }

    fn change(
        source: &str,
        old_node: Option<&str>,
        new_node: Option<&str>,
        old_parent: Option<&str>,
        new_parent: Option<&str>,
    ) -> StatusChange {
        StatusChange {
            sequence: 1,
            source: source.into(),
            operation: "UPDATE".into(),
            scope: ChangeScope::SubtreeAndAncestors,
            old_node: old_node.map(str::to_owned),
            new_node: new_node.map(str::to_owned),
            old_parent: old_parent.map(str::to_owned),
            new_parent: new_parent.map(str::to_owned),
            old_name: old_node.map(str::to_owned),
            new_name: new_node.map(str::to_owned),
            bytes: 1,
        }
    }

    // `D` is denied, so hidden `p` (a directory below it) and every descendant
    // of `p` are absent from the projection; only the visible part is projected.
    let mut rows: HashMap<String, StatusSeed> = HashMap::new();
    rows.insert("root".into(), seed("root", None, true));
    rows.insert("d".into(), seed("d", Some("root"), true));
    rows.insert("f".into(), seed("f", Some("d"), false));
    let children = build_children(&rows);

    // (a) overlay edit that points hidden `p` back at the visible parent `d`.
    let overlay_reveal = change(
        "namespace_overlay",
        Some("p"),
        Some("p"),
        Some("d"),
        Some("d"),
    );
    assert!(
        subtree_needs_refresh(&rows, &children, &overlay_reveal),
        "an overlay edit that reveals a hidden directory must refresh its subtree"
    );

    // (b) move of the hidden directory out of the denied subtree.
    let move_reveal = change("nodes", Some("p"), Some("p"), Some("D"), Some("d"));
    assert!(
        subtree_needs_refresh(&rows, &children, &move_reveal),
        "moving a hidden directory into visibility must refresh its subtree"
    );

    // ACL edits are deliberately NOT subtree refreshes: `access_denied` can hide
    // rows outside the named node's local subtree through the overlay
    // remote-parent edge, so `read_batch` rebuilds for them instead.
    let acl_hide = change(
        "access_denied",
        Some("D"),
        Some("D"),
        Some("root"),
        Some("root"),
    );
    assert!(!subtree_needs_refresh(&rows, &children, &acl_hide));

    // Controls: row-local writes must not widen into a subtree query.
    let visible_file_move = change("nodes", Some("f"), Some("f"), Some("d"), Some("root"));
    assert!(!subtree_needs_refresh(
        &rows,
        &children,
        &visible_file_move
    ));
    let visible_file_overlay = change(
        "namespace_overlay",
        Some("f"),
        Some("f"),
        Some("d"),
        Some("d"),
    );
    assert!(!subtree_needs_refresh(
        &rows,
        &children,
        &visible_file_overlay
    ));
    let rename = change("nodes", Some("d"), Some("d"), Some("root"), Some("root"));
    assert!(!subtree_needs_refresh(&rows, &children, &rename));
    let insert = change("nodes", None, Some("new"), Some("d"), Some("d"));
    assert!(!subtree_needs_refresh(&rows, &children, &insert));

    // A visible directory still widens on a move.
    let visible_dir_move = change("nodes", Some("d"), Some("d"), Some("root"), Some("top"));
    assert!(subtree_needs_refresh(&rows, &children, &visible_dir_move));
}

/// Regression for queue review R1: the subtree walk's "remove rows that the
/// scoped re-read no longer returns" pass must not overwrite a row that another
/// change on the same journal page already refreshed. A directory move together
/// with a descendant move out of that directory is the minimal case.
#[test]
fn descendant_moved_out_of_a_moved_directory_stays_visible() {
    let (_dir, service) = fixture();
    // Seed one directory with one child through the committed remote path.
    {
        let mut store = service.bridge.store.lock().unwrap();
        store
            .receive(&drive_core::Batch {
                from: "1".into(),
                to: "2".into(),
                events: vec![
                    drive_core::Event {
                        id: "e1".into(),
                        change: drive_core::Change::Upsert(node("d1", Some("root"), true)),
                    },
                    drive_core::Event {
                        id: "e2".into(),
                        change: drive_core::Change::Upsert(node("x", Some("d1"), false)),
                    },
                ],
            })
            .unwrap();
        store.drain().unwrap();
    }
    let (cache, task) = start(&service, MAX_CACHED_NODES);
    let projection = wait_ready(&cache);
    assert_eq!(projection.seed("x").unwrap().parent.as_deref(), Some("d1"));

    // Both writes land on one page: the directory move first (its subtree walk
    // collects the old child and would mark it removed), then the child's own
    // move out of that directory.
    {
        let mut store = service.bridge.store.lock().unwrap();
        store
            .receive(&drive_core::Batch {
                from: "2".into(),
                to: "3".into(),
                events: vec![
                    drive_core::Event {
                        id: "e3".into(),
                        change: drive_core::Change::Upsert(node("d1", Some("folder"), true)),
                    },
                    drive_core::Event {
                        id: "e4".into(),
                        change: drive_core::Change::Upsert(node("x", Some("root"), false)),
                    },
                ],
            })
            .unwrap();
        store.drain().unwrap();
    }
    let projection = wait_matches(
        &service,
        &cache,
        "directory move with a descendant move",
        projection.generation(),
    );
    let x = projection
        .seed("x")
        .expect("a descendant moved out on the same page must stay visible");
    assert_eq!(
        x.parent.as_deref(),
        Some("root"),
        "the descendant keeps the newer location from the same page"
    );
    assert!(projection.children("root").iter().any(|(id, _)| id == "x"));
    assert!(!projection.children("d1").iter().any(|(id, _)| id == "x"));
    assert_eq!(projection_rows(&projection), full_rows(&service));
    stop_and_join(&service, task);
}

// ---------------------------------------------------------------------------
// Issue #25: bounded lock holds, sparse publications, recoverable degradation.
// ---------------------------------------------------------------------------

/// The production lock budget is kept in the test config so the budget
/// assertions exercise the shipped constant; the throttle/retry intervals are
/// shortened so tests do not sleep on the production cadence.
fn issue25_config(faults: Arc<Faults>, max_nodes: usize, backlog_limit: u64) -> CacheConfig {
    CacheConfig {
        max_nodes,
        backlog_limit,
        publish_interval: Duration::from_millis(20),
        backlog_yield: Duration::from_millis(1),
        lock_budget: super::LOCK_BUDGET,
        retry_initial: Duration::from_millis(100),
        retry_max: Duration::from_millis(250),
        faults: Some(faults),
    }
}

fn no_faults() -> Arc<Faults> {
    Arc::new(Faults::default())
}

/// Polls the observable lifecycle until the cache reports `reason`. A degraded
/// cache whose fault was consumed recovers after a short bounded retry wait, so
/// this also proves the degraded state is visible from the outside.
fn wait_degraded(cache: &StatusCache, reason: DegradeReason) -> StatusCacheMetrics {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let metrics = cache.metrics();
        if metrics.state == CacheState::Degraded && metrics.degraded_reason == Some(reason) {
            return metrics;
        }
        assert!(
            Instant::now() < deadline,
            "cache did not degrade with {reason:?}: state {:?}, reason {:?}",
            metrics.state,
            metrics.degraded_reason
        );
        std::thread::sleep(Duration::from_millis(1));
    }
}

fn wait_recovered(cache: &StatusCache) -> StatusCacheMetrics {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let metrics = cache.metrics();
        if metrics.state == CacheState::Running
            && metrics.recovered_events > 0
            && cache.snapshot().is_some()
        {
            return metrics;
        }
        assert!(
            Instant::now() < deadline,
            "cache did not recover: state {:?}, reason {:?}, recovered {}",
            metrics.state,
            metrics.degraded_reason,
            metrics.recovered_events
        );
        std::thread::sleep(Duration::from_millis(1));
    }
}

/// Polls the product health channel for one line containing `needle`. Production
/// writes these lines to stderr (systemd journal); tests capture the same event
/// in-process so the content can be asserted without hijacking global stderr.
fn wait_health_event(cache: &StatusCache, needle: &str) -> String {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        if let Some(line) = cache
            .health_events()
            .into_iter()
            .find(|line| line.contains(needle))
        {
            return line;
        }
        assert!(
            Instant::now() < deadline,
            "no health event containing {needle:?}; captured {:?}",
            cache.health_events()
        );
        std::thread::sleep(Duration::from_millis(1));
    }
}

/// Issue #25: the first three permanent-disable paths (subscription, read-only
/// open, bootstrap) now surface a reason and retry to a full recovery without
/// restarting the service.
#[test]
fn startup_faults_degrade_visibly_and_recover_without_a_restart() {
    for (kind, reason) in [
        (
            FaultKind::TakeNotifications,
            DegradeReason::NotificationsUnavailable,
        ),
        (FaultKind::OpenReadOnly, DegradeReason::ReadOnlyOpenFailed),
        (FaultKind::Bootstrap, DegradeReason::BootstrapFailed),
    ] {
        let (_dir, service) = fixture();
        let faults = no_faults();
        faults.counter(kind).store(1, Ordering::Relaxed);
        let (cache, task) = StatusCache::start_with_config(
            service.clone(),
            issue25_config(faults, MAX_CACHED_NODES, super::BACKLOG_LIMIT),
        );
        let metrics = wait_degraded(&cache, reason);
        assert_eq!(metrics.state, CacheState::Degraded);
        assert_eq!(metrics.degraded_reason, Some(reason));
        assert!(metrics.degraded_events >= 1, "degradation must be counted");
        assert!(
            metrics.recovery_attempts >= 1,
            "the worker must retry while degraded"
        );
        wait_recovered(&cache);
        wait_matches(&service, &cache, "startup fault recovery", 0);
        stop_and_join(&service, task);
    }
}

/// Issue #25: repeated journal read errors (previously swallowed by
/// `Err(_) => {}`) degrade visibly and recover once reads succeed again.
#[test]
fn repeated_read_failures_degrade_and_recover() {
    let (_dir, service) = fixture();
    let faults = no_faults();
    let injected = faults.clone();
    let (cache, task) = StatusCache::start_with_config(
        service.clone(),
        issue25_config(faults, MAX_CACHED_NODES, super::BACKLOG_LIMIT),
    );
    let projection = wait_ready(&cache);
    let before = cache.metrics();
    // Five consecutive failed page reads cross READ_ERROR_LIMIT.
    injected
        .counter(FaultKind::Read)
        .store(5, Ordering::Relaxed);
    let metrics = wait_degraded(&cache, DegradeReason::ReadFailed);
    assert!(metrics.degraded_events >= 1);
    assert!(
        cache.snapshot().is_some(),
        "a degraded cache keeps serving its last snapshot"
    );
    wait_recovered(&cache);
    wait_matches(&service, &cache, "read failure recovery", projection.generation());
    assert!(cache.metrics().lock_holds > before.lock_holds);
    stop_and_join(&service, task);
}

/// Issue #25: a failed full rebuild (previously `disable(); return`) degrades
/// visibly and recovers without a restart.
#[test]
fn rebuild_failures_degrade_and_recover() {
    let (_dir, service) = fixture();
    let faults = no_faults();
    let injected = faults.clone();
    let (cache, task) = StatusCache::start_with_config(
        service.clone(),
        issue25_config(faults, MAX_CACHED_NODES, 2),
    );
    let projection = wait_ready(&cache);
    injected
        .counter(FaultKind::Rebuild)
        .store(1, Ordering::Relaxed);
    {
        // The three pins exceed the configured backlog limit and force a rebuild.
        let mut store = service.bridge.store.lock().unwrap();
        store.pin("one", true).unwrap();
        store.pin("two", true).unwrap();
        store.pin("folder", true).unwrap();
    }
    let metrics = wait_degraded(&cache, DegradeReason::RebuildFailed);
    assert!(metrics.degraded_events >= 1);
    let stale = cache
        .snapshot()
        .expect("the last snapshot survives a rebuild failure");
    assert_eq!(stale.len(), 4);
    wait_recovered(&cache);
    wait_matches(
        &service,
        &cache,
        "rebuild failure recovery",
        projection.generation(),
    );
    stop_and_join(&service, task);
}

/// Issue #25: an `access_denied` journal change is resolved through the scoped
/// ACL re-read instead of a whole-namespace rebuild.
#[test]
fn access_denied_changes_use_the_scoped_acl_refresh() {
    let (_dir, service) = fixture();
    let (cache, task) = start(&service, MAX_CACHED_NODES);
    let projection = wait_ready(&cache);
    {
        // Incremental ACL edit: the journal records the denied edge without
        // installing a snapshot, so the cache must resolve it scoped.
        let mut store = service.bridge.store.lock().unwrap();
        store.set_access_denied(&["folder".into()]).unwrap();
    }
    let projection = wait_matches(
        &service,
        &cache,
        "access denied",
        projection.generation(),
    );
    let acl_refreshes = cache.metrics().acl_refreshes;
    assert!(acl_refreshes >= 1, "the ACL page must count a scoped refresh");
    assert!(projection.seed("root").is_some());
    assert!(projection.seed("folder").is_none());
    assert!(projection.seed("one").is_none());
    assert!(projection.seed("two").is_none());
    {
        // Restoring access is the reverse journal change and must reveal the
        // subtree through the same scoped path.
        let mut store = service.bridge.store.lock().unwrap();
        store.set_access_denied(&[]).unwrap();
    }
    let projection = wait_matches(
        &service,
        &cache,
        "access restored",
        projection.generation(),
    );
    assert!(cache.metrics().acl_refreshes > acl_refreshes);
    assert!(projection.seed("folder").is_some());
    assert!(projection.seed("one").is_some());
    assert!(projection.seed("two").is_some());
    stop_and_join(&service, task);
}

/// Issue #25: every journal page resolves inside a bounded writer-lock hold.
#[test]
fn journal_pages_report_a_bounded_lock_hold() {
    let (_dir, service) = fixture();
    let (cache, task) = start(&service, MAX_CACHED_NODES);
    let projection = wait_ready(&cache);
    {
        let mut store = service.bridge.store.lock().unwrap();
        for _ in 0..8 {
            store.pin("one", true).unwrap();
            store.pin("one", false).unwrap();
        }
    }
    wait_matches(
        &service,
        &cache,
        "bounded lock hold",
        projection.generation(),
    );
    let metrics = cache.metrics();
    assert!(metrics.lock_holds > 0);
    assert!(
        metrics.lock_hold_p99_us <= metrics.lock_budget_us,
        "p99 lock hold {}us exceeds the {}us budget",
        metrics.lock_hold_p99_us,
        metrics.lock_budget_us
    );
    assert_eq!(metrics.lock_budget_violations, 0);
    stop_and_join(&service, task);
}

/// Issue #25: publications share the immutable base instead of cloning the whole
/// table; the delta only carries the changed rows/edges.
#[test]
fn publications_share_the_base_snapshot_instead_of_cloning_it() {
    let (_dir, service) = fixture();
    let (cache, task) = start(&service, MAX_CACHED_NODES);
    let first = wait_ready(&cache);
    let mut generation = first.generation();
    for round in 0..4u32 {
        {
            let mut store = service.bridge.store.lock().unwrap();
            store.pin("folder", round % 2 == 0).unwrap();
        }
        let next = wait_generation(&cache, generation, "sparse publication");
        generation = next.generation();
    }
    let last = cache.snapshot().unwrap();
    assert!(
        first.base_shared_with(&last),
        "incremental publications must reuse the base snapshot"
    );
    let metrics = cache.metrics();
    assert_eq!(metrics.compactions, 0, "small deltas must not rebase");
    assert!(metrics.base_shares >= 4);
    assert!(
        metrics.publish_delta_entries < 32,
        "published delta entries {} should be sparse",
        metrics.publish_delta_entries
    );
    stop_and_join(&service, task);
}

/// Issue #25: a delta that grows wide rebases the base once (compaction) instead
/// of deep-cloning rows on every publish.
#[test]
fn wide_deltas_compact_once_instead_of_cloning_every_publish() {
    let (_dir, service) = fixture();
    let mut nodes = vec![
        node("root", None, true),
        node("folder", Some("root"), true),
        node("one", Some("folder"), false),
        node("two", Some("folder"), false),
    ];
    for index in 0..600 {
        nodes.push(node(&format!("n{index}"), Some("folder"), false));
    }
    {
        // Seed the namespace before the cache starts, so bootstrap takes it as
        // the base (604 rows, wide-delta threshold 512) instead of rebuilding.
        let mut store = service.bridge.store.lock().unwrap();
        store.install_remote_snapshot(&nodes, "2", "server").unwrap();
    }
    let (cache, task) = start(&service, MAX_CACHED_NODES);
    let first = wait_ready(&cache);
    assert_eq!(first.len(), 604);
    {
        // 600 distinct journal rows: the delta is wider than the 512 compaction
        // threshold but each publish still clones only the sparse delta.
        let mut store = service.bridge.store.lock().unwrap();
        for index in 0..600 {
            store.pin(&format!("n{index}"), true).unwrap();
        }
    }
    let projection = wait_matches(&service, &cache, "wide delta", first.generation());
    assert_eq!(projection.len(), 604);
    let metrics = cache.metrics();
    assert!(
        metrics.compactions >= 1,
        "a delta wider than max(len/8, 512) must compact the base"
    );
    stop_and_join(&service, task);
}

/// Issue #25 residual: a degraded cache must be visible on the product side, not
/// only in crate-internal counters. The worker writes a structured health line
/// for the degradation (reason, attempt, scheduled retry) and for the recovery,
/// and `health_snapshot()` exposes the same state to callers.
#[test]
fn degradation_and_recovery_are_product_visible() {
    let (_dir, service) = fixture();
    let faults = no_faults();
    faults
        .counter(FaultKind::TakeNotifications)
        .store(1, Ordering::Relaxed);
    let (cache, task) = StatusCache::start_with_config(
        service.clone(),
        issue25_config(faults, MAX_CACHED_NODES, super::BACKLOG_LIMIT),
    );
    // The worker retries after a first backoff, so catch the degraded window.
    let deadline = Instant::now() + Duration::from_secs(10);
    let mut health = None;
    while Instant::now() < deadline {
        let snapshot = cache.health_snapshot();
        if snapshot.state == CacheState::Degraded
            && snapshot.degraded_reason == Some(DegradeReason::NotificationsUnavailable)
        {
            health = Some(snapshot);
            break;
        }
        std::thread::sleep(Duration::from_millis(1));
    }
    let health = health.expect("health snapshot never exposed the degraded state");
    assert_eq!(
        health.degraded_reason,
        Some(DegradeReason::NotificationsUnavailable)
    );
    assert!(health.degraded_events >= 1, "degradation must be counted");
    assert!(
        health.recovery_attempts >= 1,
        "the retry attempt must be visible in the health snapshot"
    );

    let degraded = wait_health_event(&cache, "event=degraded");
    assert!(degraded.contains("level=warn"), "degraded line: {degraded}");
    assert!(
        degraded.contains("reason=NotificationsUnavailable"),
        "degraded line: {degraded}"
    );
    assert!(
        degraded.contains("attempts=1") && degraded.contains("next_retry_in_ms="),
        "degraded line: {degraded}"
    );

    wait_recovered(&cache);
    let recovered = wait_health_event(&cache, "event=recovered");
    assert!(
        recovered.contains("level=info"),
        "recovery line: {recovered}"
    );
    assert!(
        recovered.contains("downtime_ms="),
        "recovery line: {recovered}"
    );
    let health = cache.health_snapshot();
    assert_eq!(health.state, CacheState::Running);
    assert_eq!(health.degraded_reason, None);
    assert!(health.recovered_events >= 1);
    assert!(health.next_retry_in_ms.is_none());
    wait_matches(&service, &cache, "health-visible recovery", 0);
    stop_and_join(&service, task);
}

/// Issue #25 residual: writer-guard holdings over budget are a regression; the
/// product health channel must report them and the counter must stay readable.
#[test]
fn lock_budget_overruns_are_reported_on_the_product_health_channel() {
    let (_dir, service) = fixture();
    let (cache, task) = start(&service, MAX_CACHED_NODES);
    wait_ready(&cache);
    let budget = super::LOCK_BUDGET;
    cache.record_lock_hold(budget + Duration::from_millis(5));
    let line = wait_health_event(&cache, "event=lock_budget_exceeded");
    assert!(line.contains("level=warn"), "budget line: {line}");
    assert!(
        line.contains("hold_us=") && line.contains(&format!("budget_us={}", budget.as_micros())),
        "budget line: {line}"
    );
    let health = cache.health_snapshot();
    assert!(health.lock_budget_violations >= 1);
    assert!(health.lock_holds >= 1);
// ---------------------------------------------------------------------------
// #67 W3: the projection is resident once (immutable base + sparse deltas).
// ---------------------------------------------------------------------------

/// #67 W3-A: the worker must not keep a second full copy of the projection next
/// to the immutable base it publishes. `install_full` used to clone the base's
/// row and child maps into the worker, so its resident structures were ~2x the
/// published snapshot; the requirement is one shared base plus sparse deltas.
#[test]
fn worker_keeps_a_single_resident_projection_copy() {
    let (_dir, service) = fixture();
    // A stopped cache handle still exposes the worker config the structural test
    // needs; no background worker is involved in the assertion below.
    let (cache, task) = start(&service, MAX_CACHED_NODES);
    let _ = wait_ready(&cache);
    stop_and_join(&service, task);

    let store = Store::open_read_only(&service.bridge.store.root()).unwrap();
    let mut worker = Worker::new(&cache, store);
    assert!(worker.install_full(synthetic_seeds(10_000)));

    let single_copy = worker.base.resident_bytes_estimate();
    let resident = worker.resident_bytes_estimate();
    assert!(
        resident <= single_copy * 5 / 4,
        "worker holds {resident} heap bytes for a {single_copy}-byte projection \
         (ratio {:.2}); W3 requires one shared base plus sparse deltas",
        resident as f64 / single_copy as f64
    );
}

/// #67 W3-A: the worker's merged view (immutable base + sparse deltas) must
/// materialise the same rows and sorted child edges as a full scan after a
/// move-heavy delta forces a base compaction.
#[test]
fn move_heavy_compaction_keeps_rows_and_children_on_the_oracle() {
    const MOVED: usize = 300;
    let (_dir, service) = fixture();
    let mut nodes = vec![
        node("root", None, true),
        node("folder", Some("root"), true),
        node("other", Some("root"), true),
        node("one", Some("folder"), false),
        node("two", Some("folder"), false),
    ];
    for index in 0..600 {
        nodes.push(node(&format!("n{index}"), Some("folder"), false));
    }
    {
        // Seed before the cache starts so bootstrap takes the 605 rows as base.
        let mut store = service.bridge.store.lock().unwrap();
        store
            .install_remote_snapshot(&nodes, "2", "server")
            .unwrap();
    }
    let (cache, task) = start(&service, MAX_CACHED_NODES);
    let first = wait_ready(&cache);
    assert_eq!(first.len(), 605);

    // Each move rewrites one row plus one old-parent removal and one new-parent
    // insert: 300 moves produce 900 delta entries, wider than max(len/8, 512).
    {
        let mut store = service.bridge.store.lock().unwrap();
        let events: Vec<drive_core::Event> = (0..MOVED)
            .map(|index| drive_core::Event {
                id: format!("m{index}"),
                change: drive_core::Change::Upsert(node(&format!("n{index}"), Some("other"), false)),
            })
            .collect();
        store
            .receive(&drive_core::Batch {
                from: "2".into(),
                to: "3".into(),
                events,
            })
            .unwrap();
        store.drain().unwrap();
    }
    let projection = wait_matches(&service, &cache, "move-heavy delta", first.generation());
    assert!(
        cache.metrics().compactions >= 1,
        "a 900-entry delta must compact the 605-row base"
    );
    assert_eq!(projection_rows(&projection), full_rows(&service));
    for parent in ["root", "folder", "other"] {
        assert_eq!(
            projection.children(parent),
            full_children(&service, parent),
            "child edges of {parent} must match a full scan after compaction"
        );
    }

    stop_and_join(&service, task);
}
