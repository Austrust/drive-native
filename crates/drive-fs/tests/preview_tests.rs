//! Explicit real-kernel preview gates. No NAS, worker, or external credentials.
use drive_core::initialization_events::InitializationDirectoryEntry;
use drive_core::initialization_store::{
    InitializationBinding, InitializationBudget, InitializationRun,
};
use drive_core::{Node, NodeMetadata, ReadHandle, SharedStore, Store};
use std::{
    fs::{self, File, OpenOptions},
    os::{
        fd::AsRawFd,
        unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt},
    },
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

fn node(id: &str, parent: Option<&str>, name: &str, directory: bool) -> Node {
    Node{id:id.into(),parent:parent.map(str::to_owned),name:name.into(),directory,
        version:serde_json::json!({"schema":1,"restore":"server","view":1,"root":"100","file":id,"sync":100,"max":100,"size":3,"hash":"synthetic","directory":directory}).to_string(),
        metadata:Some(NodeMetadata{size:3,mtime:100,mode:if directory{0o755}else{0o644}})}
}
fn root() -> Node {
    node("100", None, "Lab", true)
}
fn folder() -> Node {
    node("200", Some("100"), "Dir", true)
}
fn doc() -> Node {
    node("101", Some("100"), "A.doc", false)
}
fn list(s: &mut Store, run: &InitializationRun, id: &str, children: Vec<Node>) {
    let t = s.begin_initialization_directory(run, id).unwrap();
    // Fixed fixture identities independent from file IDs and version tokens.
    let entries = children
        .into_iter()
        .enumerate()
        .map(|(i, node)| InitializationDirectoryEntry {
            native_node_id: match node.id.as_str() {
                "200" => 50,
                "101" => 51,
                _ => 1000 + i as u64,
            },
            node,
        })
        .collect::<Vec<_>>();
    s.store_initialization_native_directory(&t, 1, &entries)
        .unwrap();
}
struct CountHydrator(AtomicUsize);
impl drive_fs::Hydrator for CountHydrator {
    fn hydrate(&self, _: &Node) -> Result<Arc<ReadHandle>, i32> {
        self.0.fetch_add(1, Ordering::SeqCst);
        Err(libc::EIO)
    }
}
struct Fixture {
    _temp: tempfile::TempDir,
    store: Arc<SharedStore>,
    run: InitializationRun,
    mount: PathBuf,
    hits: Arc<CountHydrator>,
}
impl Fixture {
    fn cold() -> Self {
        assert_ne!(unsafe { libc::getuid() }, 0, "run as ordinary user");
        let temp = tempfile::tempdir().unwrap();
        let mount = temp.path().join("mount");
        fs::create_dir(&mount).unwrap();
        let mut s = Store::open(temp.path().join("state")).unwrap();
        let scope = serde_json::json!({"view_id":1,"node_id":2,"file_id":"100","path":"/Lab"});
        s.bind_native_scope("server", &scope.to_string()).unwrap();
        let b = InitializationBinding {
            task_id: "kernel-preview-fixture".into(),
            scope,
            restore: "server".into(),
            start_cursor: "native-v1:100".into(),
            expected_cursor: "".into(),
rules_digest: String::new(),
        };
        let run = s
            .begin_initialization(&b, &root(), InitializationBudget::default())
            .unwrap();
        Self {
            _temp: temp,
            store: Arc::new(SharedStore::new(s)),
            run,
            mount,
            hits: Arc::new(CountHydrator(AtomicUsize::new(0))),
        }
    }
    fn mount(&self) -> fuser::BackgroundSession {
        let fs = drive_fs::CacheFilesystem::new(
            self.store.clone(),
            "100",
            unsafe { libc::getuid() },
            unsafe { libc::getgid() },
        )
        .unwrap()
        .with_writes_enabled()
        .with_hydrator(self.hits.clone());
        fuser::spawn_mount2(
            fs,
            &self.mount,
            &[
                fuser::MountOption::RW,
                fuser::MountOption::DefaultPermissions,
                fuser::MountOption::NoDev,
                fuser::MountOption::NoSuid,
            ],
        )
        .unwrap()
    }
    fn root_list(&self) {
        list(
            &mut self.store.lock().unwrap(),
            &self.run,
            "100",
            vec![folder(), doc()],
        );
    }
    fn pending_unchanged(&self) {
        let s = self.store.lock().unwrap();
        assert_eq!(s.pending_operations().unwrap(), 0);
        assert_eq!(s.cursor().unwrap(), "");
        assert!(s.node("101").is_err());
        assert!(s.open_dirty_draft("101").unwrap().is_none());
        assert_eq!(self.hits.0.load(Ordering::SeqCst), 0);
    }
}
fn error<T>(r: std::io::Result<T>, allowed: &[i32]) {
    match r {
        Err(e) => assert!(
            allowed.contains(&e.raw_os_error().unwrap_or(0)),
            "unexpected {e}"
        ),
        Ok(_) => panic!("preview operation succeeded"),
    }
}
fn names(p: &Path) -> std::io::Result<Vec<String>> {
    fs::read_dir(p)?
        .map(|e| e.map(|e| e.file_name().to_string_lossy().into_owned()))
        .collect()
}
fn page(fd: &File) -> std::io::Result<usize> {
    let mut b = [0u8; 512];
    let n = unsafe {
        libc::syscall(
            libc::SYS_getdents64,
            fd.as_raw_fd(),
            b.as_mut_ptr(),
            b.len(),
        )
    };
    if n < 0 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(n as usize)
    }
}
fn dir_open(p: &Path) -> File {
    OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_DIRECTORY)
        .open(p)
        .unwrap()
}
fn large() -> Vec<Node> {
    (0..100)
        .map(|i| {
            node(
                &format!("leaf{i}"),
                Some("200"),
                &format!("leaf-{i:03}"),
                false,
            )
        })
        .collect()
}

#[test]
#[ignore = "requires ordinary Linux user and /dev/fuse; run explicitly"]
fn cold_preview_browses_received_parent_and_pending_is_not_empty() {
    let f = Fixture::cold();
    let session = f.mount();
    assert!(fs::metadata(&f.mount).unwrap().is_dir());
    error(names(&f.mount), &[libc::EAGAIN]);
    f.root_list();
    let children = names(&f.mount).unwrap();
    assert!(children.contains(&"Dir".into()) && children.contains(&"A.doc".into()));
    error(names(&f.mount.join("Dir")), &[libc::EAGAIN]);
    error(fs::metadata(f.mount.join("Dir/missing")), &[libc::EAGAIN]);
    let t = f
        .store
        .lock()
        .unwrap()
        .begin_initialization_directory(&f.run, "200")
        .unwrap();
    error(names(&f.mount.join("Dir")), &[libc::EAGAIN]);
    f.store
        .lock()
        .unwrap()
        .store_initialization_native_directory(&t, 1, &[])
        .unwrap();
    assert!(names(&f.mount.join("Dir")).unwrap().is_empty());
    f.pending_unchanged();
    session.join();
}

#[test]
#[ignore = "requires ordinary Linux user and /dev/fuse; run explicitly"]
fn preview_content_and_all_supported_mutations_reject_without_hydration_or_journal() {
    let f = Fixture::cold();
    f.root_list();
    let session = f.mount();
    let a = f.mount.join("A.doc");
    let d = f.mount.join("Dir");
    let n = f.mount.join("new");
    error(File::open(&a), &[libc::EAGAIN]);
    error(
        OpenOptions::new().write(true).open(&a),
        &[libc::EACCES, libc::EROFS],
    );
    error(
        OpenOptions::new().write(true).truncate(true).open(&a),
        &[libc::EACCES, libc::EROFS],
    );
    error(fs::write(&a, b"replace"), &[libc::EACCES, libc::EROFS]);
    error(fs::write(&n, b"new"), &[libc::EACCES, libc::EROFS]);
    error(fs::create_dir(&n), &[libc::EACCES, libc::EROFS]);
    error(fs::remove_file(&a), &[libc::EACCES, libc::EROFS]);
    error(fs::remove_dir(&d), &[libc::EACCES, libc::EROFS]);
    error(fs::rename(&a, &n), &[libc::EACCES, libc::EROFS]);
    error(
        fs::set_permissions(&a, fs::Permissions::from_mode(0o644)),
        &[libc::EACCES, libc::EROFS],
    );
    f.pending_unchanged();
    session.join();
}

#[test]
#[ignore = "requires ordinary Linux user and /dev/fuse; run explicitly"]
fn preview_directory_continuation_is_revoked_after_denial() {
    let f = Fixture::cold();
    f.root_list();
    list(&mut f.store.lock().unwrap(), &f.run, "200", large());
    let session = f.mount();
    let dir = dir_open(&f.mount.join("Dir"));
    assert!(page(&dir).unwrap() > 0);
    {
        let mut s = f.store.lock().unwrap();
        let t = s.begin_initialization_directory(&f.run, "200").unwrap();
        s.store_initialization_denied(&t, 1).unwrap();
    }
    error(page(&dir), &[libc::EACCES, libc::ESTALE]);
    f.pending_unchanged();
    drop(dir);
    session.join();
}

#[test]
#[ignore = "requires ordinary Linux user and /dev/fuse; run explicitly"]
fn promotion_preserves_mount_and_inode_and_enables_authoritative_content_and_writes() {
    let f = Fixture::cold();
    f.root_list();
    list(&mut f.store.lock().unwrap(), &f.run, "200", vec![]);
    let session = f.mount();
    let old = dir_open(&f.mount);
    let ino = fs::metadata(f.mount.join("A.doc")).unwrap().ino();
    assert!(page(&old).unwrap() > 0);
    {
        let mut s = f.store.lock().unwrap();
        let fence = s.begin_initialization_fence(&f.run).unwrap();
        s.confirm_initialization_fence(&fence, "native-v1:100")
            .unwrap();
        s.complete_initialization(&f.run).unwrap();
        s.publish_download(
            &drive_core::DownloadTicket {
                id: "101".into(),
                version: doc().version,
            },
            b"new",
            &drive_core::sha256(b"new"),
        )
        .unwrap();
    }
    assert_eq!(
        unsafe { libc::lseek(old.as_raw_fd(), 0, libc::SEEK_SET) },
        0
    );
    error(page(&old), &[libc::ESTALE]);
    assert_eq!(fs::metadata(f.mount.join("A.doc")).unwrap().ino(), ino);
    assert_eq!(fs::read(f.mount.join("A.doc")).unwrap(), b"new");
    fs::write(f.mount.join("created"), b"local").unwrap();
    assert_eq!(fs::read(f.mount.join("created")).unwrap(), b"local");
    assert!(f.store.lock().unwrap().pending_operations().unwrap() > 0);
    assert_eq!(f.hits.0.load(Ordering::SeqCst), 0);
    drop(old);
    session.join();
}

#[test]
#[ignore = "requires ordinary Linux user and /dev/fuse; run explicitly"]
fn warm_authoritative_mount_does_not_read_the_pending_candidate() {
    let f = Fixture::cold();
    f.root_list();
    {
        let mut s = f.store.lock().unwrap();
        list(&mut s, &f.run, "200", vec![]);
        let fence = s.begin_initialization_fence(&f.run).unwrap();
        s.confirm_initialization_fence(&fence, "native-v1:100")
            .unwrap();
        s.complete_initialization(&f.run).unwrap();
        s.publish_download(
            &drive_core::DownloadTicket {
                id: "101".into(),
                version: doc().version,
            },
            b"old",
            &drive_core::sha256(b"old"),
        )
        .unwrap();
        let mut binding = s.initialization_status().unwrap().unwrap().binding;
        binding.expected_cursor = "native-v1:100".into();
        binding.start_cursor = "native-v1:200".into();
        s.begin_initialization(&binding, &root(), InitializationBudget::default())
            .unwrap();
        assert!(!s.preview_available("100").unwrap());
    }
    let session = f.mount();
    assert!(names(&f.mount).unwrap().contains(&"A.doc".into()));
    assert_eq!(fs::read(f.mount.join("A.doc")).unwrap(), b"old");
    // Local drafts remain valid in a warm tree even while remote writes are gated.
    fs::write(f.mount.join("draft"), b"local").unwrap();
    assert_eq!(fs::read(f.mount.join("draft")).unwrap(), b"local");
    assert!(
        f.store
            .lock()
            .unwrap()
            .initialization_writeback_blocked()
            .unwrap()
    );
    assert_eq!(f.hits.0.load(Ordering::SeqCst), 0);
    session.join();
}

#[test]
#[ignore = "requires ordinary Linux user and /dev/fuse; run explicitly"]
fn parent_preview_stream_is_revoked_when_a_listed_child_is_denied() {
    let f = Fixture::cold();
    f.root_list();
    let session = f.mount();
    let old = dir_open(&f.mount);
    assert!(page(&old).unwrap() > 0);
    {
        let mut s = f.store.lock().unwrap();
        let t = s.begin_initialization_directory(&f.run, "200").unwrap();
        s.store_initialization_denied(&t, 1).unwrap();
    }
    assert_eq!(
        unsafe { libc::lseek(old.as_raw_fd(), 0, libc::SEEK_SET) },
        0
    );
    error(page(&old), &[libc::ESTALE, libc::EACCES]);
    assert!(!names(&f.mount).unwrap().contains(&"Dir".into()));
    f.pending_unchanged();
    drop(old);
    session.join();
}

#[test]
#[ignore = "requires ordinary Linux user and /dev/fuse; run explicitly"]
fn restarted_candidate_recovers_same_mount_without_promoting_old_preview_handles() {
    let f = Fixture::cold();
    f.root_list();
    let session = f.mount();
    let old = dir_open(&f.mount);
    assert!(page(&old).unwrap() > 0);
    let next = {
        let mut s = f.store.lock().unwrap();
        let b = s.initialization_status().unwrap().unwrap().binding;
        s.restart_initialization(&f.run, &b, &root(), InitializationBudget::default())
            .unwrap()
    };
    assert_eq!(
        unsafe { libc::lseek(old.as_raw_fd(), 0, libc::SEEK_SET) },
        0
    );
    error(page(&old), &[libc::ESTALE]);
    error(names(&f.mount), &[libc::EAGAIN]);
    list(&mut f.store.lock().unwrap(), &next, "100", vec![doc()]);
    assert_eq!(names(&f.mount).unwrap(), vec!["A.doc"]);
    {
        let mut s = f.store.lock().unwrap();
        let t = s.begin_initialization_fence(&next).unwrap();
        s.confirm_initialization_fence(&t, "native-v1:100").unwrap();
        s.complete_initialization(&next).unwrap();
    }
    fs::write(f.mount.join("after-restart"), b"saved").unwrap();
    assert_eq!(fs::read(f.mount.join("after-restart")).unwrap(), b"saved");
    assert_eq!(f.hits.0.load(Ordering::SeqCst), 0);
    drop(old);
    session.join();
}
