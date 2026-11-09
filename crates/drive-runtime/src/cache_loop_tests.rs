//! Cache-worker tests for issue #15.
//!
//! Every local write bumps `local_revision`, so the old loop re-ran the whole
//! `cache_candidates()` scan for each edit. The worker now debounces
//! revision-only trims while policy/limit edits and over-limit pressure still
//! apply immediately. No network peer is involved.
use super::*;
use drive_core::{Node, NodeMetadata};
use std::os::unix::fs::PermissionsExt;

const CA: &str = include_str!("../../sproto/tests/fixtures/trust/ca.pem");

fn profile() -> Profile {
    Profile {
        schema: 1,
        task_id: uuid::Uuid::new_v4().to_string(),
        host: "unused.invalid".into(),
        tls_name: "unused.invalid".into(),
        trusted_ca_pem: vec![CA.into()],
        username: "synthetic".into(),
        device_id: uuid::Uuid::new_v4().to_string(),
        scope: drive_core::native::Scope {
            view_id: 1,
            node_id: 1,
            file_id: "root".into(),
            path: "/test".into(),
        },
        mount: PathBuf::from("/tmp/cache-loop-test-unused"),
        writable: false,
        local_policy: None,
        writable_source: None,
        exclude_patterns: Vec::new(),
    }
}

fn fixture() -> (tempfile::TempDir, Arc<Service>) {
    let d = tempfile::tempdir().unwrap();
    std::fs::set_permissions(d.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
    let mut store = Store::open(d.path()).unwrap();
    let nodes = [
        Node {
            id: "root".into(),
            parent: None,
            name: "root".into(),
            version: "v1".into(),
            directory: true,
            metadata: Some(NodeMetadata {
                size: 0,
                mtime: 1,
                mode: 0o755,
            }),
        },
        Node {
            id: "one".into(),
            parent: Some("root".into()),
            name: "one".into(),
            version: "v1".into(),
            directory: false,
            metadata: Some(NodeMetadata {
                size: 3,
                mtime: 1,
                mode: 0o644,
            }),
        },
    ];
    store
        .install_remote_snapshot(&nodes, "1", "server")
        .unwrap();
    store.ensure_fs_root("root").unwrap();
    store
        .publish_download(
            &drive_core::DownloadTicket {
                id: "one".into(),
                version: "v1".into(),
            },
            b"old",
            &drive_core::sha256(b"old"),
        )
        .unwrap();
    let service = Service::new(profile(), store, PathBuf::from("/unused-worker")).unwrap();
    service.trust_settings().save(d.path()).unwrap();
    (d, service)
}

fn set_limit(service: &Service, limit: u64) {
    service
        .bridge
        .store
        .lock()
        .unwrap()
        .set_cache_limit(limit)
        .unwrap();
}

fn wait_for(mut condition: impl FnMut() -> bool, deadline: Duration) {
    let until = Instant::now() + deadline;
    while !condition() {
        assert!(
            Instant::now() < until,
            "condition not met within {deadline:?}"
        );
        std::thread::sleep(Duration::from_millis(5));
    }
}

fn runs(service: &Service) -> usize {
    service.cache_trim_runs.load(Ordering::Relaxed)
}

#[test]
fn cache_tick_skips_unchanged_revision_and_debounces_new_ones() {
    let (_d, service) = fixture();
    set_limit(&service, 4096);
    let mut state = CacheLoopState::default();
    assert!(matches!(service.cache_tick(&mut state), CacheTick::Ran(None)));
    assert_eq!(runs(&service), 1);
    assert!(matches!(service.cache_tick(&mut state), CacheTick::Skipped));
    service
        .bridge
        .store
        .lock()
        .unwrap()
        .save_local("one", b"new")
        .unwrap();
    assert!(
        matches!(service.cache_tick(&mut state), CacheTick::Skipped),
        "a revision inside the debounce window must not re-run the candidate scan"
    );
    // Pretend the window elapsed; the pending revision now runs one trim.
    state.last_trim = Some(Instant::now() - CACHE_TRIM_MIN_INTERVAL - Duration::from_secs(1));
    assert!(matches!(service.cache_tick(&mut state), CacheTick::Ran(None)));
    assert_eq!(runs(&service), 2);
}

#[test]
fn cache_tick_policy_change_is_never_debounced() {
    let (_d, service) = fixture();
    set_limit(&service, 4096);
    let mut state = CacheLoopState::default();
    assert!(matches!(service.cache_tick(&mut state), CacheTick::Ran(None)));
    let before = runs(&service);
    service.cache_policy_changed();
    assert!(
        matches!(service.cache_tick(&mut state), CacheTick::Ran(None)),
        "an explicit cache policy change must not be debounced"
    );
    assert_eq!(runs(&service), before + 1);
}

#[test]
fn cache_tick_reruns_on_limit_change_and_skips_without_a_limit() {
    let (_d, service) = fixture();
    // Default limit 0 disables trimming entirely.
    let mut state = CacheLoopState::default();
    assert!(matches!(service.cache_tick(&mut state), CacheTick::Skipped));
    assert_eq!(runs(&service), 0);
    set_limit(&service, 4096);
    assert!(matches!(service.cache_tick(&mut state), CacheTick::Ran(None)));
    assert_eq!(runs(&service), 1);
    assert!(matches!(service.cache_tick(&mut state), CacheTick::Skipped));
    set_limit(&service, 8192);
    assert!(
        matches!(service.cache_tick(&mut state), CacheTick::Ran(None)),
        "a cache limit edit must trigger a trim even without a revision change"
    );
    assert_eq!(runs(&service), 2);
}

#[test]
fn cache_tick_retries_while_over_target() {
    let (_d, service) = fixture();
    set_limit(&service, 4096);
    let mut state = CacheLoopState::default();
    assert!(matches!(service.cache_tick(&mut state), CacheTick::Ran(None)));
    let before = runs(&service);
    // Over-target pressure bypasses both the revision check and the debounce.
    state.pressure = true;
    assert!(matches!(service.cache_tick(&mut state), CacheTick::Ran(None)));
    assert_eq!(runs(&service), before + 1);
}

#[test]
fn cache_loop_applies_policy_change_immediately_and_skips_unchanged_revisions() {
    let (_d, service) = fixture();
    set_limit(&service, 4096);
    service.control.lock().unwrap().state = ConnectionState::Online;
    let worker = {
        let service = service.clone();
        std::thread::spawn(move || service.cache_loop_with(Duration::from_millis(20)))
    };
    wait_for(|| runs(&service) >= 1, Duration::from_secs(2));
    let settled = runs(&service);
    std::thread::sleep(Duration::from_millis(300));
    assert_eq!(
        runs(&service),
        settled,
        "an unchanged revision must not re-trim on every loop wake"
    );
    // The policy edit notifies the loop; it must trim without waiting for the
    // interval or a revision change.
    service.cache_policy_changed();
    wait_for(
        || runs(&service) == settled + 1,
        Duration::from_secs(2),
    );
    service.stop.store(true, Ordering::Release);
    service.changed.notify_all();
    worker.join().unwrap();
}

#[test]
fn cache_loop_does_not_trim_while_offline() {
    let (_d, service) = fixture();
    set_limit(&service, 4096);
    // Default state is Locked: no trim may run.
    let worker = {
        let service = service.clone();
        std::thread::spawn(move || service.cache_loop_with(Duration::from_millis(20)))
    };
    std::thread::sleep(Duration::from_millis(250));
    assert_eq!(runs(&service), 0, "an offline worker must not trim");
    service.stop.store(true, Ordering::Release);
    service.changed.notify_all();
    worker.join().unwrap();
}
