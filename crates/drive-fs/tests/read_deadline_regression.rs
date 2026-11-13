//! Issue #59 red-state kernel regression, deliberately written against the
//! *pre-existing* drive-fs API only (`request::install`, `Hydrator`,
//! `ReadCancellation::is_cancelled`). It compiles against the unfixed tree, so
//! the same test name produces a real red -> green pair:
//!
//! * unfixed: the hydration worker blocks forever and the read caller is never
//!   answered (the post-hoc `RequestDeadline` only observes the reply);
//! * fixed: the single-claim deadline watchdog answers EIO at the budget.
//!
//! Requires an ordinary Linux user and `/dev/fuse`; run explicitly, serially:
//! `cargo test -p drive-fs --all-features --locked --test read_deadline_regression -- --ignored --test-threads=1`
use drive_core::{Node, NodeMetadata, ReadHandle, SharedStore, Store};
use drive_fs::request::{self, FuseTuning, RequestBudgets};
use drive_fs::{CacheFilesystem, Hydrator, ReadCancellation};
use std::{
    sync::{Arc, Condvar, Mutex, mpsc},
    time::{Duration, Instant},
};

const READ_BUDGET: Duration = Duration::from_millis(300);

struct BlockingHydrator {
    released: Mutex<bool>,
    cv: Condvar,
}
impl Hydrator for BlockingHydrator {
    fn hydrate(&self, node: &Node) -> Result<Arc<ReadHandle>, i32> {
        self.hydrate_cancellable(node, &ReadCancellation::default())
    }
    fn hydrate_cancellable(
        &self,
        _node: &Node,
        cancel: &ReadCancellation,
    ) -> Result<Arc<ReadHandle>, i32> {
        let mut released = self.released.lock().unwrap_or_else(|poison| poison.into_inner());
        while !*released {
            released = self
                .cv
                .wait_timeout(released, Duration::from_millis(20))
                .unwrap_or_else(|poison| poison.into_inner())
                .0;
        }
        if cancel.is_cancelled() {
            return Err(libc::EINTR);
        }
        Err(libc::EIO)
    }
}

#[test]
#[ignore = "requires ordinary Linux user and /dev/fuse; run explicitly"]
fn slow_read_must_reply_within_its_deadline() {
    let uid = unsafe { libc::getuid() };
    let gid = unsafe { libc::getgid() };
    assert_ne!(uid, 0, "run as the unprivileged test user");
    let dir = tempfile::tempdir().unwrap();
    let mount = dir.path().join("mount");
    std::fs::create_dir(&mount).unwrap();
    let mut store = Store::open(dir.path().join("state")).unwrap();
    let nodes = vec![
        Node {
            id: "root".into(),
            parent: None,
            name: "root".into(),
            version: "1".into(),
            directory: true,
            metadata: Some(NodeMetadata { size: 0, mtime: 1_700_000_000, mode: 0o755 }),
        },
        Node {
            id: "slow".into(),
            parent: Some("root".into()),
            name: "slow".into(),
            version: "1".into(),
            directory: false,
            metadata: Some(NodeMetadata { size: 4096, mtime: 1_700_000_000, mode: 0o644 }),
        },
    ];
    store.install_remote_snapshot(&nodes, "1", "synthetic").unwrap();
    store.ensure_fs_root("root").unwrap();
    let store = Arc::new(SharedStore::new(store));
    let hydrator = Arc::new(BlockingHydrator {
        released: Mutex::new(false),
        cv: Condvar::new(),
    });
    let fs = CacheFilesystem::new(store, "root", uid, gid)
        .unwrap()
        .with_hydrator(hydrator.clone());
    // CacheFilesystem::new() calls request::configure(), which re-reads the
    // environment defaults; install the short budget after construction and
    // before the mount so the tests observe it.
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
            fuser::MountOption::FSName("drive-native-read-deadline-regression".into()),
        ],
    )
    .unwrap();
    let path = mount.join("slow");
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let started = Instant::now();
        let outcome = std::fs::read(&path)
            .map(|_| ())
            .map_err(|error| error.raw_os_error());
        let _ = tx.send((outcome, started.elapsed()));
    });
    let outcome = rx.recv_timeout(Duration::from_secs(3));
    // Unblock the fixture before asserting so a red run still cleans up.
    *hydrator.released.lock().unwrap() = true;
    hydrator.cv.notify_all();
    let _ = session.join_result();
    let (error, elapsed) = outcome.expect(
        "the read was never answered at its deadline: a blocked worker RPC \
         still holds the caller (issue #59 red state)",
    );
    assert_eq!(
        error,
        Err(Some(libc::EIO)),
        "the deadline reply must be the pre-existing read-timeout error"
    );
    assert!(
        elapsed < Duration::from_secs(2),
        "the read was answered after {elapsed:?}, beyond budget + tolerance"
    );
}
