//! Explicit kernel test for cached metadata: READDIRPLUS capability, bounded
//! TTL, invalidation and the data-plane authority. Requires an ordinary Linux
//! user and /dev/fuse; run explicitly with `--ignored`.
use drive_core::{Node, NodeMetadata, Store};
use drive_fs::publication::NotifyHub;
use std::{
    ffi::OsStr,
    io,
    path::Path,
    sync::Arc,
    time::{Duration, Instant},
};

fn node(id: &str, parent: Option<&str>, directory: bool, size: u64) -> Node {
    Node {
        id: id.into(),
        parent: parent.map(str::to_owned),
        name: id.rsplit('/').next().unwrap().to_owned(),
        version: "v1".into(),
        directory,
        metadata: Some(NodeMetadata {
            size,
            mtime: 1_700_000_000,
            mode: if directory { 0o755 } else { 0o644 },
        }),
    }
}

struct Mount {
    store: Arc<drive_core::SharedStore>,
    hub: NotifyHub,
    session: Option<fuser::BackgroundSession>,
    dir: tempfile::TempDir,
}

impl Mount {
    fn new(entries: &[Node]) -> Mount {
        let uid = unsafe { libc::getuid() };
        let gid = unsafe { libc::getgid() };
        assert_ne!(uid, 0, "run as the unprivileged test user");
        let dir = tempfile::tempdir().unwrap();
        let mount = dir.path().join("mount");
        std::fs::create_dir(&mount).unwrap();
        let mut store = Store::open(dir.path().join("state")).unwrap();
        store.install_remote_snapshot(entries, "1", "s").unwrap();
        let store = Arc::new(drive_core::SharedStore::new(store));
        let hub = NotifyHub::with_ttl_ms(1000);
        let fs = drive_fs::CacheFilesystem::new(store.clone(), "root", uid, gid)
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
                fuser::MountOption::FSName("drive-native-readdirplus".into()),
            ],
        )
        .unwrap();
        hub.install(session.notifier());
        Mount {
            store,
            hub,
            session: Some(fuser::BackgroundSession::new(session).unwrap()),
            dir,
        }
    }
    fn root(&self) -> std::path::PathBuf {
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

fn names_and_sizes(dir: &Path) -> Vec<(String, u64)> {
    let mut entries = std::fs::read_dir(dir)
        .unwrap()
        .map(|entry| {
            let entry = entry.unwrap();
            let metadata = std::fs::metadata(entry.path()).unwrap();
            (entry.file_name().to_string_lossy().into_owned(), metadata.len())
        })
        .collect::<Vec<_>>();
    entries.sort();
    entries
}

#[test]
#[ignore = "requires ordinary Linux user and /dev/fuse; run explicitly"]
fn readdirplus_pages_stay_consistent_with_plain_metadata_and_invalidate() {
    let mut entries = vec![node("root", None, true, 0)];
    for group in 0..20 {
        entries.push(node(&format!("g{group:02}"), Some("root"), true, 0));
    }
    for group in 0..20 {
        for file in 0..40 {
            entries.push(node(
                &format!("g{group:02}/f{file:02}.dat"),
                Some(&format!("g{group:02}")),
                false,
                file as u64,
            ));
        }
    }
    let mount = Mount::new(&entries);
    let root = mount.root();
    // Two full traversals with the bounded TTL active; readdirplus must page
    // identically and the second traversal must agree with the first.
    let first = walk(&root);
    let second = walk(&root);
    // root plus the 20 group directories
    assert_eq!(first.len(), 21);
    assert_eq!(first, second);
    assert_eq!(mount.hub.stats().failures, 0);
    // Creating through the mount invalidates the directory: the new name is
    // immediately visible in a fresh traversal (negative entries are not cached).
    std::fs::write(root.join("g00").join("late.dat"), b"x").unwrap();
    let started = Instant::now();
    loop {
        let names = names_and_sizes(&root.join("g00"));
        if names.iter().any(|(name, _)| name == "late.dat") {
            break;
        }
        assert!(
            started.elapsed() < Duration::from_secs(3),
            "new name not visible after invalidation"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
    // Deleting the file invalidates again: the old name disappears.
    std::fs::remove_file(root.join("g00").join("late.dat")).unwrap();
    let started = Instant::now();
    loop {
        if !names_and_sizes(&root.join("g00"))
            .iter()
            .any(|(name, _)| name == "late.dat")
        {
            break;
        }
        assert!(
            started.elapsed() < Duration::from_secs(3),
            "deleted name still visible after invalidation"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
    // A rename changes names at both ends; both must converge.
    std::fs::rename(
        root.join("g01").join("f00.dat"),
        root.join("g01").join("renamed.dat"),
    )
    .unwrap();
    let started = Instant::now();
    loop {
        let names = names_and_sizes(&root.join("g01"));
        let renamed = names.iter().any(|(name, _)| name == "renamed.dat");
        let old = names.iter().any(|(name, _)| name == "f00.dat");
        if renamed && !old {
            break;
        }
        assert!(
            started.elapsed() < Duration::from_secs(3),
            "rename not visible after invalidation"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
    // The data plane remains authoritative: after the remote-side delete the
    // Store no longer resolves the object.
    let store = mount.store.lock().unwrap();
    assert!(store.lookup("g01", "renamed.dat").is_ok());
    drop(store);
}

fn walk(root: &Path) -> Vec<(String, Vec<(String, u64)>)> {
    let mut result = Vec::new();
    let mut directories = vec![root.to_path_buf()];
    while let Some(directory) = directories.pop() {
        let mut children = Vec::new();
        for entry in std::fs::read_dir(&directory).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            let metadata = std::fs::metadata(&path).unwrap();
            let name = entry.file_name().to_string_lossy().into_owned();
            if metadata.is_dir() {
                directories.push(path);
            }
            children.push((name, metadata.len()));
        }
        children.sort();
        result.push((
            directory
                .strip_prefix(root)
                .unwrap()
                .to_string_lossy()
                .into_owned(),
            children,
        ));
    }
    result.sort();
    result
}

#[allow(dead_code)]
fn display(name: &OsStr) -> String {
    name.to_string_lossy().into_owned()
}

#[allow(dead_code)]
fn result_io() -> io::Result<()> {
    Ok(())
}

#[test]
#[ignore = "requires ordinary Linux user and /dev/fuse; run explicitly"]
fn readdirplus_continuation_does_not_wait_for_a_busy_store() {
    // Kernel regression for the directory continuation contract: READDIRPLUS
    // attribute pages must degrade to dirent-only entries, never block on the
    // authoritative Store (in-flight commits/indexing can hold it for seconds).
    let mut entries = vec![node("root", None, true, 0)];
    for index in 0..2000 {
        entries.push(node(&format!("item-{index:04}"), Some("root"), false, 0));
    }
    let mount = Mount::new(&entries);
    let root = mount.root();
    // Open the directory stream before the Store is busy: OPENDIR itself takes
    // the Store lock; the continuation contract is about getdents.
    let mut directory = std::fs::read_dir(&root).unwrap();
    let mut names = (0..5)
        .map(|_| directory.next().unwrap().unwrap().file_name())
        .collect::<Vec<_>>();
    let guard = mount.store.lock().unwrap();
    let started = Instant::now();
    names.extend(
        directory
            .map(|entry| entry.unwrap().file_name())
            .collect::<Vec<_>>(),
    );
    let elapsed = started.elapsed();
    assert!(
        elapsed < Duration::from_secs(3),
        "readdirplus continuation blocked on the Store for {elapsed:?}"
    );
    assert_eq!(names.len(), 2000);
    drop(guard);
    // With the Store available the same stream is complete and consistent.
    let again = std::fs::read_dir(&root).unwrap().count();
    assert_eq!(again, 2000);
    assert_eq!(mount.hub.stats().failures, 0);
}
