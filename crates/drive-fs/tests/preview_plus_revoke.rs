//! PLUS preview authorization while Store is busy. Run with DRIVE_NATIVE_FS_DIAGNOSTICS=1.
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
        let hub = drive_fs::publication::NotifyHub::with_ttl_ms(1000);
        let fs = drive_fs::CacheFilesystem::new(
            self.store.clone(),
            "100",
            unsafe { libc::getuid() },
            unsafe { libc::getgid() },
        )
        .unwrap()
        .with_writes_enabled()
        .with_hydrator(self.hits.clone())
        .with_notify_hub(hub.clone());
        let session = fuser::Session::new(
            fs,
            &self.mount,
            &[
                fuser::MountOption::RW,
                fuser::MountOption::DefaultPermissions,
                fuser::MountOption::NoDev,
                fuser::MountOption::NoSuid,
            ],
        ).unwrap();
        hub.install(session.notifier());
        fuser::BackgroundSession::new(session).unwrap()
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

#[test]
#[ignore="ordinary Linux user, /dev/fuse and DRIVE_NATIVE_FS_DIAGNOSTICS=1; explicit kernel validation"]
fn preview_plus_cannot_disclose_denied_names_while_store_is_busy() {
    use std::sync::mpsc;
    use std::time::Duration;
    assert_eq!(std::env::var("DRIVE_NATIVE_FS_DIAGNOSTICS").as_deref(),Ok("1"));
    let f=Fixture::cold(); f.root_list(); let session=f.mount();
    let dir=dir_open(&f.mount);
    assert!(page(&dir).unwrap()>0);
    let plus_count=||drive_fs::diagnostics::snapshot().into_iter()
        .find(|s|matches!(s.stage,drive_fs::diagnostics::Stage::ReaddirPlusQueue)).unwrap().count;
    assert!(plus_count()>0,"must exercise actual READDIRPLUS, not plain READDIR");
    assert_eq!(unsafe{libc::lseek(dir.as_raw_fd(),0,libc::SEEK_SET)},0);
    let before=plus_count();
    let mut guard=f.store.lock().unwrap();
    let t=guard.begin_initialization_directory(&f.run,"200").unwrap();
    guard.store_initialization_denied(&t,1).unwrap();
    let (tx,rx)=mpsc::channel();
    let thread=std::thread::spawn(move||{tx.send(page(&dir)).unwrap();});
    let early=rx.recv_timeout(Duration::from_millis(150));
    drop(guard);
    thread.join().unwrap();
    assert!(matches!(early,Err(mpsc::RecvTimeoutError::Timeout)),"busy Store cannot permit old preview names");
    error(rx.recv_timeout(Duration::from_secs(3)).unwrap(),&[libc::EACCES,libc::ESTALE]);
    assert!(plus_count()>before,"revocation check must run through PLUS");
    f.pending_unchanged();
    session.join();
}
