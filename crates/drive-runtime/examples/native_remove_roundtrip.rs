//! Real NAS + ordinary-user FUSE unlink integration, confined to a new lab subtree.
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
    let root = format!("{lab}/fuse-remove-{}", uuid::Uuid::new_v4());
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
    let original = b"original content for unlink test\n";
    std::fs::write(&snapshot, original)?;
    let zero = NativeVersion {
        sync_id: 0,
        max_id: 0,
    };
    let file = client
        .upload_snapshot(home.view_id, &format!("{root}/A.doc"), zero, &snapshot)
        .await?;
    client
        .upload_snapshot(
            home.view_id,
            &format!("{root}/offline.doc"),
            zero,
            &snapshot,
        )
        .await?;
    client
        .upload_snapshot(
            home.view_id,
            &format!("{root}/missing.doc"),
            zero,
            &snapshot,
        )
        .await?;
    client
        .mkdir_native(home.view_id, &format!("{root}/folder"))
        .await?;
    client
        .upload_snapshot(
            home.view_id,
            &format!("{root}/folder/child.doc"),
            zero,
            &snapshot,
        )
        .await?;
    client
        .mkdir_native(home.view_id, &format!("{root}/guard"))
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
            fuser::MountOption::FSName("drive-native-remove-test".into()),
        ],
    )?;
    bridge.set_notifier(mounted.notifier());
    assert_eq!(std::fs::read(mount.join("A.doc"))?, original);
    let open = std::fs::OpenOptions::new()
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
    std::fs::remove_file(mount.join("A.doc"))?;
    wait_queue(&store, &bridge).await?;
    assert_eq!(open.metadata()?.nlink(), 0);
    open.set_len(0)?;
    open.write_all_at(b"orphan content survives remote delete", 0)?;
    open.sync_all()?;
    assert!(
        !client
            .list_dir(home.view_id, &root)
            .await?
            .iter()
            .any(|n| n.file_id == file.file_id)
    );
    let newbody = b"independent same-path replacement\n";
    std::fs::write(mount.join("A.doc"), newbody)?;
    let new_id = store
        .lock()
        .unwrap()
        .lookup(&scope.file_id, "A.doc")?
        .node
        .id;
    wait(&store, &new_id, &bridge).await?;
    assert_ne!(std::fs::metadata(mount.join("A.doc"))?.ino(), inode);
    let replacement = client
        .list_dir(home.view_id, &root)
        .await?
        .into_iter()
        .find(|n| n.name == "A.doc")
        .ok_or("replacement absent")?;
    assert_ne!(replacement.file_id, file.file_id);
    open.write_all_at(b"another orphan write", 0)?;
    open.sync_all()?;
    verify(
        &client,
        home.view_id,
        &root,
        "A.doc",
        &replacement.file_id,
        newbody,
        scratch.path(),
    )
    .await?;
    assert!(
        store
            .lock()
            .unwrap()
            .upload_operations("pending")?
            .is_empty()
    );
    println!("online_delete_tombstone_old_handle_write_and_same_path_recreate_passed");
    let mut missing = std::fs::File::open(mount.join("missing.doc"))?;
    std::fs::remove_file(mount.join("missing.doc"))?;
    let mut bytes = Vec::new();
    missing.read_to_end(&mut bytes)?;
    assert_eq!(bytes, original);
    drop(missing);
    wait_queue(&store, &bridge).await?;
    println!("already_open_uncached_handle_hydrates_before_unlink_passed");
    worker.stop();
    let _ = events.join().unwrap();
    std::fs::rename(mount.join("offline.doc"), mount.join("moved.doc"))?;
    std::fs::remove_file(mount.join("moved.doc"))?;
    std::fs::remove_file(mount.join("folder/child.doc"))?;
    std::fs::remove_dir(mount.join("folder"))?;
    std::fs::write(mount.join("temporary.doc"), b"never submitted")?;
    std::fs::remove_file(mount.join("temporary.doc"))?;
    worker = WorkerProcess::start(&executable)?;
    bridge.reconnect(worker.rpc.clone(), login())?;
    events = {
        let b = bridge.clone();
        std::thread::spawn(move || b.event_loop())
    };
    wait_queue(&store, &bridge).await?;
    let listed = client.list_dir(home.view_id, &root).await?;
    assert!(!listed.iter().any(|n| matches!(
        n.name.as_str(),
        "offline.doc" | "moved.doc" | "folder" | "temporary.doc" | "missing.doc"
    )));
    println!("offline_rename_then_delete_child_then_rmdir_and_unsent_file_delete_passed");
    worker.stop();
    let _ = events.join().unwrap();
    // The client sees an empty directory. Another session adds an unseen child
    // after our offline unlink: rmdir must detect it instead of deleting it.
    let guard_id = store
        .lock()
        .unwrap()
        .lookup(&scope.file_id, "guard")?
        .node
        .id;
    std::fs::remove_dir(mount.join("guard"))?;
    client
        .upload_snapshot(
            home.view_id,
            &format!("{root}/guard/foreign.doc"),
            zero,
            &snapshot,
        )
        .await?;
    worker = WorkerProcess::start(&executable)?;
    bridge.reconnect(worker.rpc.clone(), login())?;
    events = {
        let b = bridge.clone();
        std::thread::spawn(move || b.event_loop())
    };
    let deadline = Instant::now() + Duration::from_secs(60);
    loop {
        if store.lock().unwrap().status(&guard_id)? == Status::Conflict {
            break;
        }
        if Instant::now() > deadline {
            return Err("rmdir conflict deadline".into());
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    let children = client
        .list_dir(home.view_id, &format!("{root}/guard"))
        .await?;
    assert!(children.iter().any(|n| n.name == "foreign.doc"));
    println!("rmdir_preserves_unseen_remote_child_as_conflict_passed");
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
    assert_eq!(store.status(&file.file_id)?, Status::Deleted);
    assert_eq!(store.status(&guard_id)?, Status::Conflict);
    assert_eq!(
        store.next_remove()?.ok_or("conflict intent lost")?.state,
        "conflict"
    );
    let result = serde_json::json!({"passed":true,"uid":uid,"test_subtree":root,"native_protocol":true,"actual_fuse_unlink_and_rmdir":true,"deletion_requires_identity_tombstone":true,"open_handle_read_write_and_nlink_zero":true,"same_path_recreation_keeps_new_identity":true,"open_uncached_file_hydrated_before_unlink":true,"offline_rename_then_delete":true,"offline_child_remove_then_rmdir":true,"unsent_local_file_delete":true,"unseen_remote_child_preserved":true,"conflict_survives_restart":true,"independent_native_readback":true,"gui_nemo":false});
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
                "namespace deadline: rename={:?} remove={:?} bridge={:?}",
                s.next_rename()?,
                s.next_remove()?,
                bridge.last_error.lock().unwrap()
            )
            .into());
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}
