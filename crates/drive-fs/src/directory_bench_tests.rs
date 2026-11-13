//! Concurrency and latency probes for issue #23: an `opendir` projection must
//! not hold the writer (`SharedStore`) lock, so metadata queries (`GetStates`
//! and friends, modelled here by `SharedStore::lock()`) keep running while a
//! directory projection is being built.
//!
//! The non-ignored test proves the interaction contract; the ignored tests are
//! reproducible benchmarks used for the before/after numbers in the issue.

use super::*;
use std::sync::atomic::AtomicBool;
use std::time::{Duration, Instant};

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

/// One root directory holding `children` files, mounted as an authoritative
/// namespace with a fully allocated inode table.
fn tree(children: usize) -> (tempfile::TempDir, Arc<drive_core::SharedStore>) {
    let temp = tempfile::tempdir().unwrap();
    let mut store = Store::open(temp.path()).unwrap();
    let mut nodes = Vec::with_capacity(children + 1);
    nodes.push(node("100", None, "root", true));
    for child in 0..children {
        nodes.push(node(&format!("f{child:05}"), Some("100"), &format!("file-{child:05}"), false));
    }
    store.install_remote_snapshot(&nodes, "native-v1:9", "server").unwrap();
    store.ensure_fs_root("100").unwrap();
    (temp, Arc::new(drive_core::SharedStore::new(store)))
}

fn names(view: &View) -> Vec<String> {
    view.snapshot().entries.iter().map(|entry| entry.name.clone()).collect()
}

fn percentile(mut samples: Vec<f64>, fraction: f64) -> f64 {
    if samples.is_empty() {
        return 0.0;
    }
    samples.sort_by(|left, right| left.partial_cmp(right).unwrap());
    let rank = ((samples.len() as f64 - 1.0) * fraction).round() as usize;
    samples[rank]
}

fn bench_line(name: &str, samples: &[f64]) -> String {
    let p50 = percentile(samples.to_vec(), 0.50);
    let p95 = percentile(samples.to_vec(), 0.95);
    let max = samples.iter().copied().fold(0.0_f64, f64::max);
    format!(
        "#23 BENCH {}",
        serde_json::json!({
            "name": name,
            "iters": samples.len(),
            "p50_ms": p50,
            "p95_ms": p95,
            "max_ms": max,
        })
    )
}

/// `opendir` must not hold the writer lock while it projects: a metadata
/// caller (`GetStates` and friends, modelled by `SharedStore::lock()`) that
/// arrives mid-projection must only wait for the commit boundary, not for the
/// projection itself.
#[test]
fn opendir_does_not_hold_the_store_lock_while_projecting() {
    let (_temp, store) = tree(20_000);
    let views = Arc::new(Views::default());
    let (started_tx, started_rx) = std::sync::mpsc::channel();
    let worker = {
        let store = store.clone();
        let views = views.clone();
        std::thread::spawn(move || {
            started_tx.send(()).unwrap();
            let started = Instant::now();
            let handle = views.open(&store, 1).expect("opendir");
            (started.elapsed(), handle)
        })
    };
    started_rx.recv_timeout(Duration::from_secs(5)).unwrap();
    // Let the projection enter its build phase before contending for the lock.
    std::thread::sleep(Duration::from_millis(3));
    let started = Instant::now();
    let guard = store.lock().unwrap();
    let waited = started.elapsed();
    drop(guard);
    let (projected, handle) = worker.join().unwrap();
    views.release(1, handle).unwrap();
    assert!(views.get(1, handle).is_err());
    assert!(
        waited < Duration::from_millis(3),
        "a metadata caller waited {waited:?} while opendir projected {projected:?}"
    );
    let handle = views.open(&store, 1).expect("opendir");
    let view = views.get(1, handle).unwrap();
    assert_eq!(view.snapshot().entries.len(), 20_002);
    assert_eq!(names(&view)[2], "file-00000");
}

/// Cold projection latency over a 2,000-entry directory: the acceptance target
/// is p95 <= 300 ms; this also surfaces the lock-hold contribution.
#[test]
#[ignore = "benchmark: cold opendir projection p95 on a 2000-entry directory"]
fn opendir_dir2000_p95() {
    let (_temp, store) = tree(2000);
    let views = Views::default();
    let mut opendir_ms = Vec::new();
    for _ in 0..120 {
        views.trim_hot(None);
        let started = Instant::now();
        let handle = views.open(&store, 1).expect("opendir");
        opendir_ms.push(started.elapsed().as_secs_f64() * 1_000.0);
        views.release(1, handle).unwrap();
    }
    println!("{}", bench_line("opendir_dir2000_cold", &opendir_ms));
    assert!(percentile(opendir_ms, 0.95) <= 300.0, "opendir dir2000 p95 exceeds 300 ms");
}

/// Writer-lock handoff while a directory projection loop runs. A `GetStates`
/// style caller only needs `SharedStore::lock()`; its p95 wait is the
/// projection's lock-hold time before the fix.
#[test]
#[ignore = "benchmark: GetStates lock handoff p95 while opendir runs"]
fn getstates_lock_p95_while_opendir_runs() {
    let (_temp, store) = tree(2000);
    let views = Arc::new(Views::default());
    let stop = Arc::new(AtomicBool::new(false));
    let worker = {
        let store = store.clone();
        let views = views.clone();
        let stop = stop.clone();
        std::thread::spawn(move || {
            let mut opendir_ms = Vec::new();
            while !stop.load(Ordering::Relaxed) {
                views.trim_hot(None);
                let started = Instant::now();
                let handle = views.open(&store, 1).expect("opendir");
                opendir_ms.push(started.elapsed().as_secs_f64() * 1_000.0);
                views.release(1, handle).unwrap();
            }
            opendir_ms
        })
    };
    let mut lock_ms = Vec::new();
    for _ in 0..120 {
        let started = Instant::now();
        let guard = store.lock().unwrap();
        lock_ms.push(started.elapsed().as_secs_f64() * 1_000.0);
        drop(guard);
        std::thread::sleep(Duration::from_millis(1));
    }
    stop.store(true, Ordering::Relaxed);
    let opendir_ms = worker.join().unwrap();
    println!("{}", bench_line("opendir_mixed_dir2000", &opendir_ms));
    println!("{}", bench_line("getstates_lock_handoff", &lock_ms));
}

/// Two sibling authoritative directories with a file in each, fully inode
/// mapped (the read-only projection path requires that).
fn two_directories() -> (tempfile::TempDir, Arc<drive_core::SharedStore>) {
    let temp = tempfile::tempdir().unwrap();
    let mut store = Store::open(temp.path()).unwrap();
    let nodes = vec![
        node("100", None, "root", true),
        node("200", Some("100"), "first", true),
        node("201", Some("100"), "second", true),
        node("202", Some("200"), "first-file", false),
        node("203", Some("201"), "second-file", false),
    ];
    store
        .install_remote_snapshot(&nodes, "native-v1:1", "server")
        .unwrap();
    store.ensure_fs_root("100").unwrap();
    (temp, Arc::new(drive_core::SharedStore::new(store)))
}

fn ino(store: &drive_core::SharedStore, id: &str) -> u64 {
    store.lock().unwrap().record(id).unwrap().ino
}

fn spawn_open(
    views: Arc<Views>,
    store: Arc<drive_core::SharedStore>,
    dir: u64,
) -> std::thread::JoinHandle<(Result<(), i32>, Option<Arc<View>>)> {
    std::thread::spawn(move || match views.open(&store, dir) {
        Ok(handle) => {
            let view = views.get(dir, handle).ok();
            views.release(dir, handle).unwrap();
            (Ok(()), view)
        }
        Err(error) => (Err(error), None),
    })
}

/// Residual #23: opening one directory must not wait for an unrelated
/// in-flight projection. Before per-directory single-flight this blocks on the
/// per-mount global builder mutex; the bounded receive makes the red state a
/// deterministic assertion failure instead of a hung test.
#[test]
fn opening_a_different_directory_does_not_wait_for_an_inflight_projection() {
    let _serial = test_hooks::serial();
    test_hooks::reset();
    let (_temp, store) = two_directories();
    let first = ino(&store, "200");
    let second = ino(&store, "201");
    assert_ne!(first, second);
    let views = Arc::new(Views::default());
    let pause = test_hooks::pause(first);

    let first_opener = spawn_open(views.clone(), store.clone(), first);
    assert!(
        pause.wait_reached(Duration::from_secs(10)),
        "paused projection of the first directory never started"
    );

    let (done_tx, done_rx) = std::sync::mpsc::channel();
    let second_opener = {
        let views = views.clone();
        let store = store.clone();
        std::thread::spawn(move || {
            let result = views.open(&store, second);
            let _ = done_tx.send(());
            match result {
                Ok(handle) => {
                    let view = views.get(second, handle).ok();
                    views.release(second, handle).unwrap();
                    (Ok(()), view)
                }
                Err(error) => (Err(error), None),
            }
        })
    };
    let unrelated_completed = done_rx.recv_timeout(Duration::from_millis(750));

    // Always unblock the paused projection before asserting, so a red run
    // still joins both threads instead of hanging.
    pause.release();
    let first_result = first_opener.join().unwrap().0;
    let second_result = second_opener.join().unwrap().0;
    first_result.expect("first directory open");
    second_result.expect("second directory open");
    assert!(
        unrelated_completed.is_ok(),
        "opening a second directory was blocked by an unrelated in-flight projection"
    );
}

/// Residual #23: concurrent opens of the same directory share one projection
/// and one published view; only the first projects.
#[test]
fn concurrent_opens_of_one_directory_share_one_projection() {
    let _serial = test_hooks::serial();
    test_hooks::reset();
    let (_temp, store) = two_directories();
    let dir = ino(&store, "200");
    let views = Arc::new(Views::default());
    let pause = test_hooks::pause(dir);

    let first_opener = spawn_open(views.clone(), store.clone(), dir);
    assert!(
        pause.wait_reached(Duration::from_secs(10)),
        "paused projection of the shared directory never started"
    );
    let second_opener = spawn_open(views.clone(), store.clone(), dir);

    pause.release();
    let (first_result, first_view) = first_opener.join().unwrap();
    let (second_result, second_view) = second_opener.join().unwrap();
    first_result.expect("first open");
    second_result.expect("second open");
    assert_eq!(
        test_hooks::starts(dir),
        1,
        "a concurrent open of the same directory projected it a second time"
    );
    let (first_view, second_view) = (
        first_view.expect("first handle exposes the view"),
        second_view.expect("second handle exposes the view"),
    );
    assert!(
        Arc::ptr_eq(&first_view, &second_view),
        "concurrent opens of one directory did not share the published view"
    );
    let names = names(&first_view);
    assert!(names.contains(&"first-file".to_string()), "{names:?}");
}
