//! Regression: invalidation delivery must never block metadata replies.
//!
//! A kernel `INVAL_ENTRY` can wait for an in-flight FUSE reply, so a reply path
//! that blocks on invalidation delivery while a delivery holds a lock can
//! deadlock (observed as `request_wait_answer` vs `fuse_reverse_inval_entry`).
//! These tests exercise rename/delete interleaved with stats of the old path
//! under a positive TTL and require bounded completion.
use drive_core::{Node, NodeMetadata, Store};
use drive_fs::publication::NotifyHub;
use std::{
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};

fn node(id: &str, parent: Option<&str>, directory: bool) -> Node {
    Node {
        id: id.into(),
        parent: parent.map(str::to_owned),
        name: id.rsplit('/').next().unwrap().to_owned(),
        version: "v1".into(),
        directory,
        metadata: Some(NodeMetadata {
            size: 0,
            mtime: 1_700_000_000,
            mode: if directory { 0o755 } else { 0o644 },
        }),
    }
}

struct Mount {
    hub: NotifyHub,
    session: Option<fuser::BackgroundSession>,
    dir: tempfile::TempDir,
}

impl Mount {
    fn new(entries: &[Node], ttl_ms: u64) -> Mount {
        let uid = unsafe { libc::getuid() };
        let gid = unsafe { libc::getgid() };
        assert_ne!(uid, 0, "run as the unprivileged test user");
        let dir = tempfile::tempdir().unwrap();
        let mount = dir.path().join("mount");
        std::fs::create_dir(&mount).unwrap();
        let mut store = Store::open(dir.path().join("state")).unwrap();
        store.install_remote_snapshot(entries, "1", "s").unwrap();
        let store = Arc::new(drive_core::SharedStore::new(store));
        let hub = NotifyHub::with_ttl_ms(ttl_ms);
        let fs = drive_fs::CacheFilesystem::new(store, "root", uid, gid)
            .unwrap()
            .with_writes_enabled()
            .with_notify_hub(hub.clone());
        let session = fuser::Session::new(
            fs,
            &mount,
            &[
                fuser::MountOption::RW,
                fuser::MountOption::DefaultPermissions,
                fuser::MountOption::NoDev,
                fuser::MountOption::NoSuid,
                fuser::MountOption::FSName("drive-native-deadlock".into()),
            ],
        )
        .unwrap();
        hub.install(session.notifier());
        Mount {
            hub,
            session: Some(fuser::BackgroundSession::new(session).unwrap()),
            dir,
        }
    }
    fn root(&self) -> PathBuf {
        self.dir.path().join("mount")
    }
}

impl Drop for Mount {
    fn drop(&mut self) {
        if let Some(session) = self.session.take() {
            let _ = session.join_result();
        }
    }
}

fn wait_until(deadline_ms: u64, mut check: impl FnMut() -> bool, what: &str) {
    let started = Instant::now();
    loop {
        if check() {
            return;
        }
        assert!(
            started.elapsed() < Duration::from_millis(deadline_ms),
            "{what} did not settle within {deadline_ms}ms"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[test]
#[ignore = "requires ordinary Linux user and /dev/fuse; run explicitly"]
fn rename_and_delete_under_ttl_do_not_deadlock_concurrent_old_path_stats() {
    let entries = vec![
        node("root", None, true),
        node("g", Some("root"), true),
        node("g/a.dat", Some("g"), false),
        node("g/b.dat", Some("g"), false),
        node("g/c.dat", Some("g"), false),
    ];
    let mount = Mount::new(&entries, 1000);
    let root = mount.root();
    let g = root.join("g");
    // Warm the dentries/attrs under the positive TTL.
    for name in ["a.dat", "b.dat", "c.dat"] {
        assert!(std::fs::metadata(g.join(name)).is_ok());
    }
    // Interleave rename/delete with stats of the old paths. Each operation must
    // complete promptly; a notification that waits for a reply while holding a
    // reply-side lock would hang here.
    std::fs::rename(g.join("a.dat"), g.join("a2.dat")).unwrap();
    wait_until(3000, || std::fs::metadata(g.join("a.dat")).is_err(), "old name");
    assert!(std::fs::metadata(g.join("a2.dat")).is_ok());
    std::fs::remove_file(g.join("b.dat")).unwrap();
    wait_until(3000, || std::fs::metadata(g.join("b.dat")).is_err(), "deleted name");
    assert!(std::fs::metadata(g.join("c.dat")).is_ok());
    // A burst of renames followed by stats of the old names must also finish.
    for round in 0..20 {
        let from = g.join(format!("c.dat"));
        let to = g.join(format!("c-{round}.dat"));
        std::fs::rename(&from, &to).unwrap();
        assert!(std::fs::metadata(&from).is_err());
        std::fs::rename(&to, from).unwrap();
    }
    assert_eq!(mount.hub.stats().failures, 0);
}

#[test]
#[ignore = "requires ordinary Linux user and /dev/fuse; run explicitly"]
fn invalidation_delivery_never_stalls_other_requests() {
    let entries = vec![
        node("root", None, true),
        node("g", Some("root"), true),
        node("g/a.dat", Some("g"), false),
        node("g/b.dat", Some("g"), false),
    ];
    let mount = Mount::new(&entries, 1000);
    let root = mount.root();
    // Trigger a burst of entry invalidations through local renames while other
    // metadata requests keep being served.
    let g = root.join("g");
    for round in 0..50 {
        std::fs::rename(g.join("a.dat"), g.join(format!("a-{round}.dat"))).unwrap();
        assert!(std::fs::metadata(g.join("b.dat")).is_ok());
        std::fs::rename(g.join(format!("a-{round}.dat")), g.join("a.dat")).unwrap();
        assert!(!std::fs::read_dir(&root).unwrap().next().is_none());
    }
    assert_eq!(mount.hub.stats().failures, 0);
}
