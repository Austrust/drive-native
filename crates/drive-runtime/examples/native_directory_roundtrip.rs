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
use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};
async fn wait(
    store: &Arc<Mutex<Store>>,
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
    let root = format!("{lab}/fuse-directory-{}", uuid::Uuid::new_v4());
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
    let original = b"existing file moved into a newly created directory";
    let zero = NativeVersion {
        sync_id: 0,
        max_id: 0,
    };
    std::fs::write(&snapshot, original)?;
    let target = client
        .upload_snapshot(home.view_id, &format!("{root}/A.doc"), zero, &snapshot)
        .await?;
    // Deliberately challenge the server with a same-name file in this isolated
    // fixture. Preflight listing alone cannot guard a remote creation race.
    let collision_result = client
        .mkdir_native(home.view_id, &format!("{root}/A.doc"))
        .await;
    let listed = client.list_dir(home.view_id, &root).await?;
    let unchanged = listed
        .iter()
        .find(|n| n.name == "A.doc")
        .ok_or("file disappeared after mkdir collision")?;
    assert_eq!(unchanged.file_type, sproto::actions::list::FileType::File);
    assert_eq!(unchanged.file_id, target.file_id);
    assert_eq!(unchanged.sync_id, target.version.sync_id);
    assert_eq!(unchanged.max_id, target.version.max_id);
    assert!(
        collision_result.is_err(),
        "mkdir over file unexpectedly returned a directory receipt"
    );
    println!("native_mkdir_same_name_file_rejected_without_changing_identity_or_version");
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
    let store = Arc::new(Mutex::new(Store::open(scratch.path().join("state"))?));
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
            fuser::MountOption::FSName("drive-native-directory-test".into()),
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
    assert_eq!(std::fs::read(mount.join("A.doc"))?, original);
    std::fs::create_dir(mount.join("Online"))?;
    std::fs::write(mount.join("Online/note.txt"), b"online nested content")?;
    let online = store
        .lock()
        .unwrap()
        .lookup(&scope.file_id, "Online")?
        .node
        .id;
    let note = store.lock().unwrap().lookup(&online, "note.txt")?.node.id;
    wait_queue(&store, &bridge).await?;
    wait(&store, &note, &bridge).await?;
    let listed = client
        .list_dir(home.view_id, &format!("{root}/Online"))
        .await?;
    let n = listed
        .iter()
        .find(|n| n.name == "note.txt")
        .ok_or("online child absent")?;
    verify(
        &client,
        home.view_id,
        &format!("{root}/Online"),
        "note.txt",
        &n.file_id,
        b"online nested content",
        scratch.path(),
    )
    .await?;
    std::fs::rename(mount.join("A.doc"), mount.join("Online/A.doc"))?;
    wait_queue(&store, &bridge).await?;
    println!("native_fuse_online_mkdir_child_upload_and_move_verified");
    worker.stop();
    let _ = events.join().unwrap();
    std::fs::create_dir_all(mount.join("Offline/Deep"))?;
    let parent = store
        .lock()
        .unwrap()
        .lookup(&scope.file_id, "Offline")?
        .node
        .id;
    let deep = store.lock().unwrap().lookup(&parent, "Deep")?.node.id;
    std::fs::write(
        mount.join("Offline/Deep/new.txt"),
        b"offline child survives parent rename",
    )?;
    let child = store.lock().unwrap().lookup(&deep, "new.txt")?.node.id;
    std::fs::rename(mount.join("Online/A.doc"), mount.join("Offline/Deep/A.doc"))?;
    std::fs::rename(mount.join("Offline"), mount.join("Renamed"))?;
    std::fs::create_dir(mount.join("CreateThenRemove"))?;
    std::fs::remove_dir(mount.join("CreateThenRemove"))?;
    worker = WorkerProcess::start(&executable)?;
    bridge.reconnect(worker.rpc.clone(), login())?;
    events = {
        let b = bridge.clone();
        std::thread::spawn(move || b.event_loop())
    };
    wait_queue(&store, &bridge).await?;
    wait(&store, &child, &bridge).await?;
    let listed = client
        .list_dir(home.view_id, &format!("{root}/Renamed/Deep"))
        .await?;
    assert_eq!(listed.len(), 2);
    let n = listed
        .iter()
        .find(|n| n.name == "new.txt")
        .ok_or("offline child absent")?;
    verify(
        &client,
        home.view_id,
        &format!("{root}/Renamed/Deep"),
        "new.txt",
        &n.file_id,
        b"offline child survives parent rename",
        scratch.path(),
    )
    .await?;
    verify(
        &client,
        home.view_id,
        &format!("{root}/Renamed/Deep"),
        "A.doc",
        &target.file_id,
        original,
        scratch.path(),
    )
    .await?;
    assert!(
        !client
            .list_dir(home.view_id, &root)
            .await?
            .iter()
            .any(|n| n.name == "CreateThenRemove" || n.name == "Offline")
    );
    println!(
        "native_fuse_offline_nested_mkdir_existing_file_move_parent_rename_and_remove_verified"
    );
    worker.stop();
    let _ = events.join().unwrap();
    std::fs::create_dir(mount.join("Collision"))?;
    let collision = store
        .lock()
        .unwrap()
        .lookup(&scope.file_id, "Collision")?
        .node
        .id;
    std::fs::write(mount.join("Collision/local.txt"), b"retain local child")?;
    let remote = client
        .mkdir_native(home.view_id, &format!("{root}/Collision"))
        .await?;
    worker = WorkerProcess::start(&executable)?;
    bridge.reconnect(worker.rpc.clone(), login())?;
    events = {
        let b = bridge.clone();
        std::thread::spawn(move || b.event_loop())
    };
    let deadline = Instant::now() + Duration::from_secs(45);
    loop {
        if store.lock().unwrap().status(&collision)? == Status::Conflict {
            break;
        }
        if Instant::now() > deadline {
            return Err("directory collision deadline".into());
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    assert_eq!(
        std::fs::read(mount.join("Collision/local.txt"))?,
        b"retain local child"
    );
    assert!(
        client
            .list_dir(home.view_id, &format!("{root}/Collision"))
            .await?
            .is_empty()
    );
    let current = client.list_dir(home.view_id, &root).await?;
    assert_eq!(
        current
            .iter()
            .find(|n| n.name == "Collision")
            .unwrap()
            .file_id,
        remote.file_id
    );
    println!("preexisting_remote_directory_conflicts_without_adopting_or_uploading_children");
    stop.store(true, Ordering::Release);
    bridge.signal_upload();
    upload.join().unwrap();
    worker.stop();
    let _ = events.join().unwrap();
    mounted.join();
    drop(bridge);
    drop(store);
    let s = Store::open(scratch.path().join("state"))?;
    assert_eq!(s.status(&collision)?, Status::Conflict);
    let result = serde_json::json!({"passed":true,"uid":uid,"test_subtree":root,"native_protocol":true,"native_mkdir_existing_file_preserved":true,"fuse_online_mkdir":true,"child_upload_waits_for_parent":true,"offline_nested_creation":true,"move_existing_file_into_unconfirmed_parent":true,"rename_new_directory_before_reconnect":true,"create_then_remove_directory":true,"remote_directory_collision_preserves_both":true,"restart_preserves_conflict":true,"lost_directory_receipt_auto_reconciliation":false});
    if let Some(path) = args.get(2) {
        std::fs::write(path, serde_json::to_vec_pretty(&result)?)?;
    }
    println!("{result}");
    Ok(())
}
async fn wait_queue(
    store: &Arc<Mutex<Store>>,
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
                s.next_namespace_kind()?,
                bridge.last_error.lock().unwrap()
            )
            .into());
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}
