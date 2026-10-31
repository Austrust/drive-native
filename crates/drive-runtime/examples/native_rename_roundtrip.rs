//! Real NAS + ordinary-user FUSE rename integration, confined to a new lab subtree.
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
use std::{
    io::Read,
    os::unix::fs::{FileExt, MetadataExt},
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
    let root = format!("{lab}/fuse-rename-{}", uuid::Uuid::new_v4());
    let directory = client.mkdir_native(home.view_id, &root).await?;
    let scope = Scope {
        view_id: home.view_id,
        node_id: directory.node_id,
        file_id: directory.file_id,
        path: root.clone(),
    };
    let scratch = tempfile::tempdir()?;
    let snapshot = scratch.path().join("snapshot");
    let mount = scratch.path().join("mount");
    std::fs::create_dir(&mount)?;
    let original = b"old content before rename\n";
    std::fs::write(&snapshot, original)?;
    let zero = NativeVersion {
        sync_id: 0,
        max_id: 0,
    };
    let file = client
        .upload_snapshot(home.view_id, &format!("{root}/A.doc"), zero, &snapshot)
        .await?;
    client
        .mkdir_native(home.view_id, &format!("{root}/folder"))
        .await?;
    let child = client
        .upload_snapshot(
            home.view_id,
            &format!("{root}/folder/child.doc"),
            zero,
            &snapshot,
        )
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
    let store = Arc::new(drive_core::SharedStore::new(Store::open(scratch.path().join("state"))?));
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
            fuser::MountOption::FSName("drive-native-rename-test".into()),
        ],
    )?;
    bridge.set_notifier(mounted.notifier());
    assert_eq!(std::fs::read(mount.join("A.doc"))?, original);
    assert_eq!(std::fs::read(mount.join("folder/child.doc"))?, original);
    let mut open = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(mount.join("A.doc"))?;
    let inode = open.metadata()?.ino();
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
    std::fs::rename(mount.join("A.doc"), mount.join("B.doc"))?;
    wait(&store, &file.file_id, &bridge).await?;
    assert_eq!(std::fs::metadata(mount.join("B.doc"))?.ino(), inode);
    let mut old = Vec::new();
    open.read_to_end(&mut old)?;
    assert_eq!(old, original);
    let mut new_reader = std::fs::File::open(mount.join("B.doc"))?;
    let edited = b"edit from handle opened before rename\n";
    open.set_len(0)?;
    open.write_all_at(edited, 0)?;
    open.sync_all()?;
    let mut simultaneous = Vec::new();
    new_reader.read_to_end(&mut simultaneous)?;
    assert_eq!(simultaneous, edited);
    drop(new_reader);
    wait(&store, &file.file_id, &bridge).await?;
    verify(
        &client,
        home.view_id,
        &root,
        "B.doc",
        &file.file_id,
        edited,
        scratch.path(),
    )
    .await?;
    println!("online_rename_kept_inode_old_handle_read_and_write_passed");
    worker.stop();
    assert!(events.join().unwrap().is_err());
    std::fs::rename(mount.join("B.doc"), mount.join("C.doc"))?;
    std::fs::rename(mount.join("C.doc"), mount.join("D.doc"))?;
    let offline = b"offline edit through original handle after two moves\n";
    open.set_len(0)?;
    open.write_all_at(offline, 0)?;
    open.sync_all()?;
    assert_eq!(std::fs::read(mount.join("D.doc"))?, offline);
    assert_eq!(std::fs::metadata(mount.join("D.doc"))?.ino(), inode);
    std::fs::rename(mount.join("folder"), mount.join("moved-folder"))?;
    let childbody = b"offline child edit after moving parent directory\n";
    let childfile = std::fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(mount.join("moved-folder/child.doc"))?;
    childfile.write_all_at(childbody, 0)?;
    childfile.sync_all()?;
    drop(childfile);
    worker = WorkerProcess::start(&executable)?;
    bridge.reconnect(worker.rpc.clone(), login())?;
    events = {
        let b = bridge.clone();
        std::thread::spawn(move || b.event_loop())
    };
    wait(&store, &file.file_id, &bridge).await?;
    wait(&store, &child.file_id, &bridge).await?;
    verify(
        &client,
        home.view_id,
        &root,
        "D.doc",
        &file.file_id,
        offline,
        scratch.path(),
    )
    .await?;
    verify(
        &client,
        home.view_id,
        &format!("{root}/moved-folder"),
        "child.doc",
        &child.file_id,
        childbody,
        scratch.path(),
    )
    .await?;
    let listing = client.list_dir(home.view_id, &root).await?;
    assert!(
        !listing
            .iter()
            .any(|n| matches!(n.name.as_str(), "A.doc" | "B.doc" | "C.doc" | "folder"))
    );
    assert!(store.lock().unwrap().next_rename()?.is_none());
    println!("offline_chained_file_and_directory_moves_then_upload_passed");
    drop(open);
    stop.store(true, Ordering::Release);
    bridge.signal_upload();
    upload.join().unwrap();
    worker.stop();
    let _ = events.join().unwrap();
    mounted.join();
    drop(bridge);
    drop(store);
    let store = Store::open(scratch.path().join("state"))?;
    assert_eq!(store.node(&file.file_id)?.name, "D.doc");
    assert!(store.next_rename()?.is_none());
    assert_eq!(store.status(&file.file_id)?, Status::LocalComplete);
    let result = serde_json::json!({"passed":true,"uid":uid,"test_subtree":root,"native_protocol":true,"actual_fuse_rename":true,"stable_inode":true,"pre_and_post_rename_handles_share_local_writes":true,"old_handle_read_and_write_after_commit":true,"offline_chained_rename":true,"offline_directory_move_and_child_write":true,"independent_native_readback":true,"unmount_restart_preserves_confirmed_paths":true,"rename_overwrite":false,"unlink":false,"gui_nemo":false});
    if let Some(path) = args.get(2) {
        std::fs::write(path, serde_json::to_vec_pretty(&result)?)?;
    }
    println!("{result}");
    Ok(())
}
