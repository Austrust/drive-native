//! Actual kernel interruption; synthetic contents, no NAS and no user folders.
use drive_core::{Node, NodeMetadata, ReadHandle, Store};
use drive_fs::{Hydrator, ReadCancellation};
use std::{
    process::{Command, Stdio},
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::{Duration, Instant},
};
struct Waiting {
    started: AtomicUsize,
    cancelled: AtomicUsize,
}
impl Hydrator for Waiting {
    fn hydrate(&self, _: &Node) -> Result<Arc<ReadHandle>, i32> {
        panic!("read must pass cancellation")
    }
    fn hydrate_cancellable(
        &self,
        _: &Node,
        token: &ReadCancellation,
    ) -> Result<Arc<ReadHandle>, i32> {
        self.started.fetch_add(1, Ordering::Release);
        let end = Instant::now() + Duration::from_secs(8);
        while !token.is_cancelled() && Instant::now() < end {
            std::thread::sleep(Duration::from_millis(5));
        }
        if token.is_cancelled() {
            self.cancelled.fetch_add(1, Ordering::Release);
            Err(libc::EINTR)
        } else {
            Err(libc::ETIMEDOUT)
        }
    }
}
#[test]
#[ignore = "requires ordinary Linux user and /dev/fuse"]
fn fatal_signal_cancels_kernel_read_and_frees_hydration_worker() {
    let uid = unsafe { libc::getuid() };
    assert_ne!(uid, 0);
    let dir = tempfile::tempdir().unwrap();
    let mount = dir.path().join("mount");
    std::fs::create_dir(&mount).unwrap();
    let mut store = Store::open(dir.path().join("data")).unwrap();
    let nodes = vec![
        Node {
            id: "root".into(),
            parent: None,
            name: "root".into(),
            version: "1".into(),
            directory: true,
            metadata: Some(NodeMetadata {
                size: 0,
                mtime: 1,
                mode: 0o755,
            }),
        },
        Node {
            id: "file".into(),
            parent: Some("root".into()),
            name: "A.doc".into(),
            version: "1".into(),
            directory: false,
            metadata: Some(NodeMetadata {
                size: 1024,
                mtime: 1,
                mode: 0o644,
            }),
        },
    ];
    store
        .install_remote_snapshot(&nodes, "1", "server")
        .unwrap();
    let waiting = Arc::new(Waiting {
        started: AtomicUsize::new(0),
        cancelled: AtomicUsize::new(0),
    });
    let fs = drive_fs::CacheFilesystem::new(Arc::new(drive_core::SharedStore::new(store)), "root", uid, unsafe {
        libc::getgid()
    })
    .unwrap()
    .with_hydrator(waiting.clone());
    let session = fuser::spawn_mount2(
        fs,
        &mount,
        &[
            fuser::MountOption::RO,
            fuser::MountOption::DefaultPermissions,
            fuser::MountOption::NoDev,
            fuser::MountOption::NoSuid,
        ],
    )
    .unwrap();
    for round in 1..=8 {
        let mut child = Command::new("cat")
            .arg(mount.join("A.doc"))
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(3);
        while waiting.started.load(Ordering::Acquire) < round {
            assert!(Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(5));
        }
        let start = Instant::now();
        assert_eq!(unsafe { libc::kill(child.id() as i32, libc::SIGTERM) }, 0);
        while child.try_wait().unwrap().is_none() {
            assert!(start.elapsed() < Duration::from_secs(2));
            std::thread::sleep(Duration::from_millis(5));
        }
        while waiting.cancelled.load(Ordering::Acquire) < round {
            assert!(start.elapsed() < Duration::from_secs(2));
            std::thread::sleep(Duration::from_millis(5));
        }
        assert_eq!(std::fs::metadata(mount.join("A.doc")).unwrap().len(), 1024);
    }
    session.join();
    assert_eq!(waiting.cancelled.load(Ordering::Acquire), 8);
}
