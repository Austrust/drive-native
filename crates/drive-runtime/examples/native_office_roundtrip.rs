//! Real NAS + ordinary-user FUSE atomic replacement integration, confined to a new lab subtree.
use drive_core::{Status, Store, native::Scope};
use drive_runtime::{
    bridge::{Bridge, WorkerProcess},
    wire::Login,
};
use sproto::{
    Client,
    actions::native::NativeVersion,
    client::{Config, Credentials},
};
use std::{path::Path, process::Command};
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};
async fn wait(
    store: &Arc<drive_core::SharedStore>,
    id: &str,
    bridge: &Bridge,
) -> Result<(), Box<dyn std::error::Error>> {
    let deadline = Instant::now() + Duration::from_secs(60);
    loop {
        let status = store.lock().unwrap().status(id)?;
        if status == Status::LocalComplete {
            return Ok(());
        }
        if Instant::now() > deadline {
            return Err(format!(
                "rename/save deadline: {status:?}; {:?}",
                bridge.last_error.lock().unwrap()
            )
            .into());
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}
async fn verify(
    client: &Client,
    view: u64,
    path: &str,
    name: &str,
    id: &str,
    body: &[u8],
    stage: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let nodes = client.list_dir(view, path).await?;
    let n = nodes
        .iter()
        .find(|n| n.name == name)
        .ok_or("renamed remote path absent")?;
    assert_eq!(n.file_id, id);
    let expected = sproto::actions::transfer::ContentVersion {
        file_id: n.file_id.clone(),
        version: NativeVersion {
            sync_id: n.sync_id,
            max_id: n.max_id,
        },
        size: n.file_size,
        hash: n.hash.clone(),
    };
    let file = stage.join(format!("verify-{}", uuid::Uuid::new_v4()));
    client.download_checked(&expected, &file).await?;
    assert_eq!(std::fs::read(file)?, body);
    Ok(())
}
fn office(
    helper: &Path,
    action: &str,
    document: &Path,
    expected: &str,
    text: Option<&str>,
    trace: Option<&Path>,
) -> drive_core::Result<()> {
    let mut cmd = Command::new("/usr/bin/timeout");
    cmd.args(["--signal=TERM", "--kill-after=5", "120", "/usr/bin/python3"])
        .arg(helper)
        .arg(action)
        .arg(document)
        .arg(expected);
    if let Some(text) = text {
        cmd.arg("--text").arg(text);
    }
    if let Some(trace) = trace {
        cmd.arg("--trace").arg(trace);
    }
    let result = cmd.output()?;
    print!("{}", String::from_utf8_lossy(&result.stdout));
    if !result.status.success() {
        eprintln!("{}", String::from_utf8_lossy(&result.stderr));
        return Err(drive_core::Error::State(
            "LibreOffice command failed".into(),
        ));
    }
    Ok(())
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let uid = unsafe { libc::getuid() };
    let gid = unsafe { libc::getgid() };
    assert_ne!(uid, 0);
    let args: Vec<_> = std::env::args().collect();
    let lab = args.get(1).ok_or("expected authorized lab")?;
    let parts: Vec<_> = lab.split('/').collect();
    if parts.len() != 3
        || parts[0] != ""
        || parts[1] != "DriveNative-Lab"
        || parts[2].is_empty()
        || parts[2].contains("..")
        || lab.contains('\0')
    {
        return Err("invalid lab".into());
    }
    let password = rpassword::prompt_password("Test account password (not saved): ")?;
    let client = Client::connect(
        Config::builder()
            .host("192.0.2.10")
            .tls_server_name("nas.example.invalid".to_owned())
            .credentials(Credentials::Password {
                username: "test-user".into(),
                password: password.clone(),
                otp: None,
            })
            .build(),
    )
    .await?;
    let home = client.personal_root().await?;
    assert!(home.writable && home.download_allowed);
    assert!(
        client
            .list_dir(home.view_id, "/DriveNative-Lab")
            .await?
            .iter()
            .any(|n| n.name == parts[2])
    );
    let root = format!("{lab}/writer-{}", uuid::Uuid::new_v4());
    let directory = client.mkdir_native(home.view_id, &root).await?;
    let scope = Scope {
        view_id: home.view_id,
        node_id: directory.node_id,
        file_id: directory.file_id,
        path: root.clone(),
    };
    let scratch = tempfile::tempdir()?.keep();
    println!("application_test_state={}", scratch.display());
    let snapshot = scratch.as_path().join("snapshot.docx");
    let helper = std::path::Path::new(args.get(3).ok_or("UNO helper required")?);
    office(
        helper,
        "create",
        &snapshot,
        "Initial Writer document",
        None,
        None,
    )?;
    let mount = scratch.as_path().join("mount");
    std::fs::create_dir(&mount)?;
    let zero = NativeVersion {
        sync_id: 0,
        max_id: 0,
    };
    let target = client
        .upload_snapshot(home.view_id, &format!("{root}/A.docx"), zero, &snapshot)
        .await?;
    let executable = std::env::current_exe()?
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("drive-native-worker");
    let device = uuid::Uuid::new_v4().to_string();
    let login = || Login {
        host: "192.0.2.10".into(),
        tls_name: "nas.example.invalid".into(),
        trusted_ca_pem: Vec::new(),
        username: "test-user".into(),
        password: password.clone(),
        device_id: device.clone(),
        scope: scope.clone(),
    };
    let mut worker = WorkerProcess::start(&executable)?;
    let store = Arc::new(drive_core::SharedStore::new(Store::open(scratch.as_path().join("state"))?));
    let bridge = Arc::new(Bridge::connect(store.clone(), worker.rpc.clone(), login())?);
    bridge.pull()?;
    let fs = drive_fs::CacheFilesystem::new(store.clone(), &scope.file_id, uid, gid)?
        .with_writes_enabled()
        .with_hydrator(bridge.clone());
    let mounted = fuser::spawn_mount2(
        fs,
        &mount,
        &[
            fuser::MountOption::RW,
            fuser::MountOption::DefaultPermissions,
            fuser::MountOption::NoDev,
            fuser::MountOption::NoSuid,
            fuser::MountOption::FSName("drive-native-writer-test".into()),
        ],
    )?;
    bridge.set_notifier(mounted.notifier());
    let mut events = {
        let b = bridge.clone();
        std::thread::spawn(move || b.event_loop())
    };
    let stop = Arc::new(AtomicBool::new(false));
    let upload = {
        let b = bridge.clone();
        let stop = stop.clone();
        std::thread::spawn(move || b.upload_loop(&stop))
    };
    let path = mount.join("A.docx");
    office(
        helper,
        "edit",
        &path,
        "Initial Writer document",
        Some("Online Writer save"),
        None,
    )?;
    wait_queue(&store, &bridge).await?;
    let id = store
        .lock()
        .unwrap()
        .lookup(&scope.file_id, "A.docx")?
        .node
        .id;
    wait(&store, &id, &bridge).await?;
    let saved = std::fs::read(&path)?;
    verify(
        &client,
        home.view_id,
        &root,
        "A.docx",
        &target.file_id,
        &saved,
        scratch.as_path(),
    )
    .await?;
    println!("actual_writer_online_save_native_commit_verified");
    // Independent native session changes the document while this mount stays online.
    office(
        helper,
        "edit",
        &snapshot,
        "Initial Writer document",
        Some("Remote change without reopening mount"),
        None,
    )?;
    let remote = client
        .list_dir(home.view_id, &root)
        .await?
        .into_iter()
        .find(|n| n.name == "A.docx")
        .ok_or("target absent")?;
    let changed = client
        .upload_snapshot(
            home.view_id,
            &format!("{root}/A.docx"),
            NativeVersion {
                sync_id: remote.sync_id,
                max_id: remote.max_id,
            },
            &snapshot,
        )
        .await?;
    let deadline = Instant::now() + Duration::from_secs(60);
    loop {
        let s = store.lock().unwrap();
        let n = s.node(&id)?;
        let v: serde_json::Value = serde_json::from_str(&n.version)?;
        let ready = v["sync"].as_u64() == Some(changed.version.sync_id)
            && s.status(&id)? == Status::LocalComplete;
        drop(s);
        if ready {
            break;
        }
        if Instant::now() > deadline {
            return Err("Writer background refresh deadline".into());
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    worker.stop();
    let _ = events.join().unwrap();
    office(
        helper,
        "read",
        &path,
        "Remote change without reopening mount",
        None,
        None,
    )?;
    println!("actual_writer_offline_reads_background_refreshed_document");
    office(
        helper,
        "edit",
        &path,
        "Remote change without reopening mount",
        Some("Offline Writer save one"),
        None,
    )?;
    office(
        helper,
        "edit",
        &path,
        "Offline Writer save one",
        Some("Offline Writer save two"),
        None,
    )?;
    office(helper, "read", &path, "Offline Writer save two", None, None)?;
    worker = WorkerProcess::start(&executable)?;
    bridge.reconnect(worker.rpc.clone(), login())?;
    events = {
        let b = bridge.clone();
        std::thread::spawn(move || b.event_loop())
    };
    wait_queue(&store, &bridge).await?;
    let id = store
        .lock()
        .unwrap()
        .lookup(&scope.file_id, "A.docx")?
        .node
        .id;
    wait(&store, &id, &bridge).await?;
    let saved = std::fs::read(&path)?;
    verify(
        &client,
        home.view_id,
        &root,
        "A.docx",
        &target.file_id,
        &saved,
        scratch.as_path(),
    )
    .await?;
    let names = client.list_dir(home.view_id, &root).await?;
    assert_eq!(
        names.len(),
        1,
        "temporary or lock files remain on NAS: {names:?}"
    );
    assert_eq!(names[0].name, "A.docx");
    println!("actual_writer_two_offline_saves_reconnect_native_commit_verified");
    stop.store(true, Ordering::Release);
    bridge.signal_upload();
    upload.join().unwrap();
    worker.stop();
    let _ = events.join().unwrap();
    mounted.join();
    drop(bridge);
    drop(store);
    let result = serde_json::json!({"passed":true,"uid":uid,"test_subtree":root,"application":"LibreOffice Writer 24.2.7.2","actual_docx_save":true,"online_native_commit":true,"background_refresh_without_reopen":true,"worker_stopped_before_offline_open":true,"two_offline_application_saves":true,"reconnect_native_commit":true,"remote_target_identity_preserved":true,"no_remote_temp_or_lock_files":true,"gui_nemo":false});
    if let Some(path) = args.get(2) {
        std::fs::write(path, serde_json::to_vec_pretty(&result)?)?;
    }
    println!("{result}");
    Ok(())
}
async fn wait_queue(
    store: &Arc<drive_core::SharedStore>,
    bridge: &Bridge,
) -> Result<(), Box<dyn std::error::Error>> {
    let deadline = Instant::now() + Duration::from_secs(75);
    loop {
        if store.lock().unwrap().next_namespace_kind()?.is_none() {
            return Ok(());
        }
        if Instant::now() > deadline {
            let s = store.lock().unwrap();
            return Err(format!(
                "replacement deadline: {:?}; {:?}",
                s.next_replace()?,
                bridge.last_error.lock().unwrap()
            )
            .into());
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}
