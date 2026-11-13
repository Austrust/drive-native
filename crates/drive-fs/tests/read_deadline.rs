//! Issue #59 kernel end-to-end: a cache-miss read whose hydration worker never
//! returns must still get a defined reply at its deadline, exactly once, and a
//! late hydration result must be discarded. Queued work that expired before it
//! started must not execute, and the non-preemptible hydration count must stay
//! bounded by the data-plane worker pool.
//!
//! Synthetic fixture only: no NAS, no user directories. Requires an ordinary
//! Linux user and `/dev/fuse`; run explicitly, serially:
//!
//! ```text
//! cargo test -p drive-fs --all-features --locked --test read_deadline -- \
//!     --ignored --nocapture --test-threads=1
//! ```
use drive_core::{sha256, DownloadTicket, Node, NodeMetadata, ReadHandle, SharedStore, Store};
use drive_fs::request::{self, FuseTuning, RequestBudgets};
use drive_fs::{CacheFilesystem, Hydrator, ReadCancellation};
use std::{
    sync::{
        Arc, Condvar, Mutex,
        atomic::{AtomicBool, AtomicUsize, Ordering},
        mpsc,
    },
    time::{Duration, Instant},
};

const READ_BUDGET: Duration = Duration::from_millis(300);
const BODY: usize = 4096;
/// One FUSE mount per test; `request::install` is process-global, so kernel
/// tests in this binary must not overlap.
static SERIAL: Mutex<()> = Mutex::new(());
fn serial() -> std::sync::MutexGuard<'static, ()> {
    SERIAL.lock().unwrap_or_else(|poison| poison.into_inner())
}

fn node(id: &str, parent: Option<&str>, directory: bool, size: u64) -> Node {
    Node {
        id: id.into(),
        parent: parent.map(str::to_owned),
        name: id.into(),
        version: "1".into(),
        directory,
        metadata: Some(NodeMetadata {
            size,
            mtime: 1_700_000_000,
            mode: if directory { 0o755 } else { 0o644 },
        }),
    }
}

/// Test-only worker RPC: every hydration blocks until the test releases it,
/// exactly like a runtime download that stays connected but never returns.
struct SlowHydrator {
    store: Arc<SharedStore>,
    calls: AtomicUsize,
    abandoned_before_start: AtomicUsize,
    released: Mutex<bool>,
    cv: Condvar,
    fail_after_release: AtomicBool,
    body: Vec<u8>,
}
impl SlowHydrator {
    fn new(store: Arc<SharedStore>, fail_after_release: bool) -> Self {
        Self {
            store,
            calls: AtomicUsize::new(0),
            abandoned_before_start: AtomicUsize::new(0),
            released: Mutex::new(false),
            cv: Condvar::new(),
            fail_after_release: AtomicBool::new(fail_after_release),
            body: vec![7; BODY],
        }
    }
    fn release(&self) {
        *self.released.lock().unwrap_or_else(|poison| poison.into_inner()) = true;
        self.cv.notify_all();
    }
    fn calls(&self) -> usize {
        self.calls.load(Ordering::SeqCst)
    }
}
impl Hydrator for SlowHydrator {
    fn hydrate(&self, node: &Node) -> Result<Arc<ReadHandle>, i32> {
        self.hydrate_cancellable(node, &ReadCancellation::default())
    }
    fn hydrate_cancellable(
        &self,
        node: &Node,
        cancel: &ReadCancellation,
    ) -> Result<Arc<ReadHandle>, i32> {
        // The runtime-side contract under test: a request whose reply is gone is
        // not started at all (here the token is the only observable signal).
        if cancel.should_abandon() {
            self.abandoned_before_start.fetch_add(1, Ordering::SeqCst);
            return Err(cancel.abandoned_error().unwrap_or(libc::EINTR));
        }
        self.calls.fetch_add(1, Ordering::SeqCst);
        let mut released = self.released.lock().unwrap_or_else(|poison| poison.into_inner());
        while !*released {
            released = self
                .cv
                .wait_timeout(released, Duration::from_millis(20))
                .unwrap_or_else(|poison| poison.into_inner())
                .0;
        }
        drop(released);
        if self.fail_after_release.load(Ordering::SeqCst) {
            return Err(libc::EIO);
        }
        let ticket = DownloadTicket {
            id: node.id.clone(),
            version: node.version.clone(),
        };
        self.store
            .lock()
            .unwrap()
            .publish_download(&ticket, &self.body, &sha256(&self.body))
            .map_err(|_| libc::EIO)?;
        let handle = self.store.lock().unwrap().open_cached(&node.id).map_err(|_| libc::EIO)?;
        Ok(Arc::new(handle))
    }
}

struct Fixture {
    dir: tempfile::TempDir,
    session: Option<fuser::BackgroundSession>,
    store: Arc<SharedStore>,
    hydrator: Arc<SlowHydrator>,
}
impl Fixture {
    fn mount(children: usize, fail_after_release: bool) -> Self {
        let uid = unsafe { libc::getuid() };
        let gid = unsafe { libc::getgid() };
        assert_ne!(uid, 0, "run as the unprivileged test user");
        let dir = tempfile::tempdir().unwrap();
        let mount = dir.path().join("mount");
        std::fs::create_dir(&mount).unwrap();
        let mut store = Store::open(dir.path().join("state")).unwrap();
        let mut nodes = vec![node("root", None, true, 0)];
        for child in 0..children {
            nodes.push(node(&format!("slow{child:02}"), Some("root"), false, BODY as u64));
        }
        store.install_remote_snapshot(&nodes, "1", "synthetic").unwrap();
        store.ensure_fs_root("root").unwrap();
        let store = Arc::new(SharedStore::new(store));
        let hydrator = Arc::new(SlowHydrator::new(store.clone(), fail_after_release));
        let fs = CacheFilesystem::new(store.clone(), "root", uid, gid)
            .unwrap()
            .with_hydrator(hydrator.clone());
        // Issue #59: enforce a real, short reply deadline for this process. The
        // kernel e2e binary is serialized, so the global budget is per test.
        // Install after CacheFilesystem::new() (which calls request::configure()
        // and would otherwise reset the budgets to the environment defaults)
        // and before the mount.
        request::install(
            FuseTuning::default(),
            RequestBudgets {
                read: READ_BUDGET,
                ..RequestBudgets::default()
            },
        );
        let session = fuser::spawn_mount2(
            fs,
            &mount,
            &[
                fuser::MountOption::RO,
                fuser::MountOption::FSName("drive-native-read-deadline".into()),
            ],
        )
        .unwrap();
        Self {
            dir,
            session: Some(session),
            store,
            hydrator,
        }
    }
    fn root(&self) -> std::path::PathBuf {
        self.dir.path().join("mount")
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        // Never let a panicking test hang the process on unmount: a pending
        // FUSE READ keeps the session alive until its hydration returns, so
        // release the (test-only) worker first, then bound the unmount join.
        self.hydrator.release();
        if let Some(session) = self.session.take() {
            let (done, finished) = mpsc::channel();
            std::thread::spawn(move || {
                let _ = done.send(session.join_result().is_ok());
            });
            let _ = finished.recv_timeout(Duration::from_secs(10));
        }
        // Keep the store alive until the mount is gone: a late worker holding
        // the Arc must not observe a dropped state directory.
        let _ = &self.store;
    }
}

fn thread_count() -> u64 {
    std::fs::read_to_string("/proc/self/status")
        .unwrap_or_default()
        .lines()
        .find_map(|line| line.strip_prefix("Threads:"))
        .and_then(|rest| rest.trim().parse().ok())
        .unwrap_or(0)
}

fn read_error(path: std::path::PathBuf) -> (Option<i32>, Duration) {
    let started = Instant::now();
    let error = std::fs::read(&path).map(|_| ()).map_err(|e| e.raw_os_error());
    (error.unwrap_err(), started.elapsed())
}

/// AC: a 60 s / never-returning worker RPC with a short budget must reply
/// end-to-end at the budget, exactly once (EIO), and the late body must be
/// discarded rather than sent as a second reply or installed into the handle.
#[test]
#[ignore = "requires ordinary Linux user and /dev/fuse; run explicitly"]
fn slow_worker_replies_at_the_deadline_and_discards_the_late_body() {
    let _serial = serial();
    let fixture = Fixture::mount(1, false);
    let before = request::snapshot();
    assert_eq!(
        before.budgets_ms.iter().find(|(name, _)| *name == "read").map(|(_, value)| *value),
        Some(READ_BUDGET.as_millis() as u64),
        "the configured read budget did not reach the request path (stale build or install ordering)"
    );
    let path = fixture.root().join("slow00");
    let (tx, rx) = mpsc::channel();
    let reader = std::thread::spawn(move || {
        let _ = tx.send(read_error(path));
    });
    let (error, elapsed) = rx.recv_timeout(Duration::from_secs(3)).expect(
        "the read was never answered: the deadline did not race the blocked worker",
    );
    assert_eq!(
        error,
        Some(libc::EIO),
        "a deadline reply must use the pre-existing read-timeout error"
    );
    assert!(
        elapsed >= READ_BUDGET / 2,
        "the read was answered before its budget: {elapsed:?}"
    );
    assert!(
        elapsed < Duration::from_secs(2),
        "the read exceeded the budget + tolerance: {elapsed:?}"
    );
    let during = request::snapshot();
    assert!(
        during.deadline.fired >= before.deadline.fired + 1,
        "the deadline watchdog did not answer the request"
    );
    assert!(
        during.deadline.hydrations_inflight >= 1,
        "the blocked, non-preemptible hydration must stay observable"
    );
    assert!(
        during.deadline.hydrations_peak <= 4,
        "non-preemptible hydrations exceeded the data-plane worker bound"
    );
    // Let the worker return a valid body after the reply was already sent.
    fixture.hydrator.release();
    let deadline = Instant::now() + Duration::from_secs(3);
    loop {
        let now = request::snapshot();
        let late = now.deadline.late_results_discarded >= before.deadline.late_results_discarded + 1;
        let skipped = now.deadline.expired_task_skips > before.deadline.expired_task_skips;
        if (late || skipped) && now.deadline.hydrations_inflight == 0 {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "the late hydration result was not discarded (no late_result counter, inflight={})",
            now.deadline.hydrations_inflight
        );
        std::thread::sleep(Duration::from_millis(10));
    }
    reader.join().unwrap();
    // Exactly one reply was sent for the first read; a fresh read hydrates
    // again and returns one body. If a double reply had leaked, fuser would
    // have aborted the session or the data would be corrupted here.
    let data = std::fs::read(fixture.root().join("slow00")).expect(
        "a read after a discarded late result must recover through a fresh hydration",
    );
    assert_eq!(data.len(), BODY);
    assert!(data.iter().all(|byte| *byte == 7));
    assert!(
        request::snapshot().deadline.late_results_discarded
            >= before.deadline.late_results_discarded + 1,
        "the late hydration result was not recorded as discarded"
    );
}

/// AC: queued work that expired before it started must not execute expensive
/// work; the number of non-preemptible hydrations stays bounded by the worker
/// pool and returns to zero after the timeout/unmount, with threads back to the
/// pre-read baseline.
#[test]
#[ignore = "requires ordinary Linux user and /dev/fuse; run explicitly"]
fn expired_queued_reads_are_not_started_and_the_worker_stays_bounded() {
    let _serial = serial();
    const READS: usize = 12;
    /// Fixed data-plane worker pool size (see `LocalQueue::new`).
    const WORKERS: usize = 4;
    let fixture = Fixture::mount(READS, true);
    assert_eq!(
        request::snapshot()
            .budgets_ms
            .iter()
            .find(|(name, _)| *name == "read")
            .map(|(_, value)| *value),
        Some(READ_BUDGET.as_millis() as u64),
        "the configured read budget did not reach the request path (stale build or install ordering)"
    );
    let baseline_threads = thread_count();
    let readers = (0..READS)
        .map(|child| {
            let path = fixture.root().join(format!("slow{child:02}"));
            std::thread::spawn(move || read_error(path))
        })
        .collect::<Vec<_>>();
    // Give every request time to reach the data plane; only the four pool
    // workers can start a hydration, the rest queue behind them.
    let deadline = Instant::now() + Duration::from_secs(5);
    while fixture.hydrator.calls() < WORKERS {
        assert!(
            Instant::now() < deadline,
            "the data-plane workers never started their hydrations"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
    // Wait well past the read budget so every queued request is expired.
    std::thread::sleep(READ_BUDGET * 2);
    let started_before_release = fixture.hydrator.calls();
    assert!(
        started_before_release <= WORKERS,
        "non-preemptible hydrations exceeded the {WORKERS}-worker bound: {started_before_release}"
    );
    let before = request::snapshot();
    fixture.hydrator.release();
    for reader in readers {
        let (error, elapsed) = reader.join().unwrap();
        assert_eq!(
            error,
            Some(libc::EIO),
            "every expired read must be answered with the deadline error"
        );
        assert!(
            elapsed < Duration::from_secs(3),
            "an expired read exceeded its budget + tolerance: {elapsed:?}"
        );
    }
    assert_eq!(
        fixture.hydrator.calls(),
        started_before_release,
        "an expired queued task executed its hydration after the workers were released"
    );
    // The four pool workers return before the remaining queued jobs are
    // observed as skipped, so wait for the counters instead of racing them.
    let deadline = Instant::now() + Duration::from_secs(3);
    let after = loop {
        let now = request::snapshot();
        if now.deadline.expired_task_skips > before.deadline.expired_task_skips {
            break now;
        }
        assert!(
            Instant::now() < deadline,
            "expired queued tasks were not observed as skipped (skips {} -> {})",
            before.deadline.expired_task_skips,
            now.deadline.expired_task_skips
        );
        std::thread::sleep(Duration::from_millis(10));
    };
    assert_eq!(
        after.deadline.hydrations_inflight, 0,
        "the hydration gauge did not return to zero after the timeout"
    );
    // Threads return to the pre-read baseline once the mount is gone.
    drop(fixture);
    let deadline = Instant::now() + Duration::from_secs(3);
    loop {
        let threads = thread_count();
        if threads <= baseline_threads + 1 {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "threads did not return to the baseline: {threads} > {baseline_threads} + 1"
        );
        std::thread::sleep(Duration::from_millis(20));
    }
}
