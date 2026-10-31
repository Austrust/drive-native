//! Explicit native snapshot fence test in a newly created authorized test child.
use drive_core::{
    Store,
    native::{NativeReadSession, Scope},
};
use sproto::{
    Client,
    client::{Config, Credentials},
};
async fn connect(password: &str) -> Result<Client, Box<dyn std::error::Error>> {
    Ok(Client::connect(
        Config::builder()
            .host("192.0.2.10")
            .tls_server_name("nas.example.invalid".to_owned())
            .credentials(Credentials::Password {
                username: "test-user".into(),
                password: password.into(),
                otp: None,
            })
            .build(),
    )
    .await?)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    assert_ne!(unsafe { libc::getuid() }, 0);
    unsafe {
        let limit = libc::rlimit {
            rlim_cur: 0,
            rlim_max: 0,
        };
        libc::setrlimit(libc::RLIMIT_CORE, &limit);
    }
    let args: Vec<_> = std::env::args().collect();
    let paged = args.len() == 4 && args[3] == "--paged";
    if (!paged && args.len() != 3) || args[1] != "/DriveNative-Lab/20261028T124720-1e31763a" {
        return Err(
            "usage: native_snapshot_roundtrip <authorized-lab> <new-result.json> [--paged]".into(),
        );
    }
    let output = &args[2];
    use std::io::Write;
    std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(output)?
        .write_all(b"{\"passed\":false}\n")?;
    let password = rpassword::prompt_password("Test account password (not saved): ")?;
    let a = connect(&password).await?;
    let b = connect(&password).await?;
    let home = a.personal_root().await?;
    let root = format!("{}/snapshot-{}", args[1], uuid::Uuid::new_v4());
    println!("test_subtree={root}");
    std::fs::write(
        output,
        serde_json::json!({"passed":false,"scope":root,"stage":"create"}).to_string(),
    )?;
    let receipt = a.mkdir_native(home.view_id, &root).await?;
    let scope = Scope {
        view_id: home.view_id,
        node_id: receipt.node_id,
        file_id: receipt.file_id,
        path: root.clone(),
    };
    a.mkdir_native(home.view_id, &format!("{root}/Before"))
        .await?;
    let session = NativeReadSession::new(a.clone(), scope.clone())?;
    let file_count = if paged { 1002 } else { 0 };
    if paged {
        let sources = tempfile::tempdir()?;
        let empty = sources.path().join("empty");
        std::fs::write(&empty, b"")?;
        let started = std::time::Instant::now();
        for base in (0..file_count).step_by(4) {
            let mut jobs = tokio::task::JoinSet::new();
            for i in base..(base + 4).min(file_count) {
                let client = a.clone();
                let path = format!("{root}/page-{i:04}.txt");
                let source = empty.clone();
                let view = home.view_id;
                jobs.spawn(async move {
                    client
                        .upload_snapshot(
                            view,
                            &path,
                            sproto::actions::native::NativeVersion {
                                sync_id: 0,
                                max_id: 0,
                            },
                            &source,
                        )
                        .await
                });
            }
            while let Some(outcome) = jobs.join_next().await {
                outcome??;
            }
            if base % 100 == 0 {
                println!(
                    "pagination fixture: {}/{} files; {}s",
                    (base + 4).min(file_count),
                    file_count,
                    started.elapsed().as_secs()
                );
            }
        }
    }
    let candidate = session.snapshot_candidate().await?;
    assert_eq!(candidate.nodes.len(), 2 + file_count);
    for i in 0..file_count {
        assert!(
            candidate
                .nodes
                .iter()
                .any(|n| n.name == format!("page-{i:04}.txt"))
        );
    }
    assert!(session.snapshot_unchanged(&candidate).await?);
    b.mkdir_native(home.view_id, &format!("{root}/During"))
        .await?;
    assert!(
        !session.snapshot_unchanged(&candidate).await?,
        "changed enumeration was accepted"
    );
    let stable = session.recovery_snapshot().await?;
    assert_eq!(stable.nodes.len(), 3 + file_count);
    let tmp = tempfile::tempdir()?;
    let mut store = Store::open(tmp.path())?;
    session.bind(&mut store)?;
    store.install_native_snapshot(&stable, "")?;
    assert_eq!(store.cursor()?, stable.cursor);
    assert!(store.node(&scope.file_id)?.directory);
    std::fs::write(
        output,
        serde_json::to_vec_pretty(
            &serde_json::json!({"passed":false,"stage":"fence passed; runtime pending","scope":root,"nodes":stable.nodes.len(),"checks":["native root and complete directory enumeration","quiet event fence accepted","independent session mutation invalidates candidate","new candidate includes concurrent mutation","checked atomic installation"]}),
        )?,
    )?;
    let source = tmp.path().join("source");
    std::fs::write(&source, b"native cached initial")?;
    let remote = format!("{root}/A.doc");
    a.upload_snapshot(
        home.view_id,
        &remote,
        sproto::actions::native::NativeVersion {
            sync_id: 0,
            max_id: 0,
        },
        &source,
    )
    .await?;
    let worker_path = std::env::current_exe()?
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("drive-native-worker");
    let mut worker = drive_runtime::bridge::WorkerProcess::start(&worker_path)?;
    let state_dir = tmp.path().join("runtime");
    let shared = std::sync::Arc::new(drive_core::SharedStore::new(Store::open(&state_dir)?));
    let bridge = std::sync::Arc::new(drive_runtime::bridge::Bridge::connect(
        shared.clone(),
        worker.rpc.clone(),
        drive_runtime::wire::Login {
            host: "192.0.2.10".into(),
            tls_name: "nas.example.invalid".into(),
            trusted_ca_pem: Vec::new(),
            username: "test-user".into(),
            password: password.clone(),
            device_id: uuid::Uuid::new_v4().to_string(),
            scope: scope.clone(),
        },
    )?);
    drop(password);
    assert!(bridge.pull()?);
    let mount = tmp.path().join("mount");
    std::fs::create_dir(&mount)?;
    let fs = drive_fs::CacheFilesystem::new(
        shared.clone(),
        &scope.file_id,
        unsafe { libc::getuid() },
        unsafe { libc::getgid() },
    )?
    .with_hydrator(bridge.clone())
    .with_writes_enabled();
    let mounted = fuser::spawn_mount2(
        fs,
        &mount,
        &[
            fuser::MountOption::RW,
            fuser::MountOption::DefaultPermissions,
            fuser::MountOption::NoDev,
            fuser::MountOption::NoSuid,
        ],
    )?;
    bridge.set_notifier(mounted.notifier());
    assert_eq!(
        std::fs::read(mount.join("A.doc"))?,
        b"native cached initial"
    );
    std::fs::write(mount.join("A.doc"), b"local pending during cursor recovery")?;
    std::fs::write(mount.join("Offline.doc"), b"local create during recovery")?;
    let current = session.recovery_snapshot().await?;
    {
        let mut st = shared.lock().unwrap();
        st.pin(&scope.file_id, true)?;
        st.install_remote_snapshot(&current.nodes, "native-v1:999999999", &current.restore)?;
    }
    assert!(
        bridge.pull()?,
        "cursor reset did not install native recovery snapshot"
    );
    assert_ne!(shared.lock().unwrap().cursor()?, "native-v1:999999999");
    let pending = shared.lock().unwrap().pending_operations()?;
    assert!(pending >= 2);
    bridge.disconnect();
    worker.stop();
    assert_eq!(
        std::fs::read(mount.join("A.doc"))?,
        b"local pending during cursor recovery"
    );
    assert_eq!(
        std::fs::read(mount.join("Offline.doc"))?,
        b"local create during recovery"
    );
    let count = bridge.downloads.load(std::sync::atomic::Ordering::SeqCst);
    let entries = std::fs::read_dir(&mount)?.collect::<std::io::Result<Vec<_>>>()?;
    assert!(entries.len() >= 4);
    assert_eq!(
        bridge.downloads.load(std::sync::atomic::Ordering::SeqCst),
        count
    );
    assert!(shared.lock().unwrap().is_pinned(&scope.file_id)?);
    drop(entries);
    mounted.join();
    drop(bridge);
    drop(shared);
    let reopened = Store::open(&state_dir)?;
    let record = reopened.lookup(&scope.file_id, "A.doc")?;
    use std::io::Read;
    let mut body = Vec::new();
    reopened
        .open_cached(&record.node.id)?
        .read_to_end(&mut body)?;
    assert_eq!(body, b"local pending during cursor recovery");
    std::fs::write(
        output,
        serde_json::to_vec_pretty(
            &serde_json::json!({"passed":true,"scope":root,"nodes":current.nodes.len(),"pending_operations":pending,"pagination_files":file_count,"checks":["native snapshot quiet fence","independent mutation rejects stale candidate","initial worker descriptor bootstrap","FUSE native hydration","actual native future cursor reset response","automatic checked recovery","dirty edit and local create preserved","offline mounted reads","pin preserved","directory browsing zero downloads","SQLite restart preserves pending content"]}),
        )?,
    )?;
    println!("snapshot fence and native worker/FUSE recovery passed");
    Ok(())
}
