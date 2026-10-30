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
    let root = format!("{lab}/fuse-replace-{}", uuid::Uuid::new_v4());
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
    let original = b"old document before atomic save\n";
    let initial = b"new version from existing remote temp\n";
    let zero = NativeVersion {
        sync_id: 0,
        max_id: 0,
    };
    std::fs::write(&snapshot, original)?;
    let target = client
        .upload_snapshot(home.view_id, &format!("{root}/A.doc"), zero, &snapshot)
        .await?;
    let conflict = client
        .upload_snapshot(
            home.view_id,
            &format!("{root}/conflict.doc"),
            zero,
            &snapshot,
        )
        .await?;
    std::fs::write(&snapshot, initial)?;
    let source = client
        .upload_snapshot(
            home.view_id,
            &format!("{root}/existing.tmp"),
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
            fuser::MountOption::FSName("drive-native-replace-test".into()),
        ],
    )?;
    bridge.set_notifier(mounted.notifier());
    assert_eq!(std::fs::read(mount.join("A.doc"))?, original);
    assert_eq!(std::fs::read(mount.join("existing.tmp"))?, initial);
    assert_eq!(std::fs::read(mount.join("conflict.doc"))?, original);
    let mut old = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(mount.join("A.doc"))?;
    let source_handle = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(mount.join("existing.tmp"))?;
    let inode = source_handle.metadata()?.ino();
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
    std::fs::rename(mount.join("existing.tmp"), mount.join("A.doc"))?;
    wait_queue(&store, &bridge).await?;
    wait(&store, &source.file_id, &bridge).await?;
    assert_eq!(std::fs::metadata(mount.join("A.doc"))?.ino(), inode);
    assert_eq!(old.metadata()?.nlink(), 0);
    let mut oldbody = Vec::new();
    old.read_to_end(&mut oldbody)?;
    assert_eq!(oldbody, original);
    verify(
        &client,
        home.view_id,
        &root,
        "A.doc",
        &target.file_id,
        initial,
        scratch.path(),
    )
    .await?;
    let listed = client.list_dir(home.view_id, &root).await?;
    assert!(
        !listed
            .iter()
            .any(|n| n.name == "existing.tmp" || n.file_id == source.file_id)
    );
    let edited = b"edit through original source handle after replacement\n";
    source_handle.set_len(0)?;
    source_handle.write_all_at(edited, 0)?;
    source_handle.sync_all()?;
    wait(&store, &source.file_id, &bridge).await?;
    old.set_len(0)?;
    old.write_all_at(b"old target orphan write", 0)?;
    old.sync_all()?;
    verify(
        &client,
        home.view_id,
        &root,
        "A.doc",
        &target.file_id,
        edited,
        scratch.path(),
    )
    .await?;
    println!("existing_source_replace_retains_remote_target_identity_and_both_handles_passed");
    drop(old);
    drop(source_handle);
    // A later remote update must map to the surviving local source inode.
    let latest = client
        .list_dir(home.view_id, &root)
        .await?
        .into_iter()
        .find(|n| n.name == "A.doc")
        .ok_or("target absent")?;
    let remote_new = b"remote edit after replacement binding\n";
    std::fs::write(&snapshot, remote_new)?;
    let updated = client
        .upload_snapshot(
            home.view_id,
            &format!("{root}/A.doc"),
            NativeVersion {
                sync_id: latest.sync_id,
                max_id: latest.max_id,
            },
            &snapshot,
        )
        .await?;
    let deadline = Instant::now() + Duration::from_secs(60);
    loop {
        let n = store.lock().unwrap().node(&source.file_id)?;
        let v: serde_json::Value = serde_json::from_str(&n.version)?;
        if v["sync"].as_u64() == Some(updated.version.sync_id)
            && store.lock().unwrap().status(&source.file_id)? == Status::LocalComplete
        {
            break;
        }
        if Instant::now() > deadline {
            return Err("background update after identity rebind deadline".into());
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    assert_eq!(std::fs::read(mount.join("A.doc"))?, remote_new);
    println!("remote_background_update_after_identity_rebind_passed");
    worker.stop();
    let _ = events.join().unwrap();
    let first = b"offline atomic save one\n";
    let second = b"offline atomic save two\n";
    std::fs::write(mount.join("first.tmp"), first)?;
    std::fs::rename(mount.join("first.tmp"), mount.join("A.doc"))?;
    std::fs::write(mount.join("second.tmp"), second)?;
    let last_inode = std::fs::metadata(mount.join("second.tmp"))?.ino();
    std::fs::rename(mount.join("second.tmp"), mount.join("A.doc"))?;
    let last_id = store
        .lock()
        .unwrap()
        .lookup(&scope.file_id, "A.doc")?
        .node
        .id;
    std::fs::write(mount.join("new-source.tmp"), b"offline entirely new save")?;
    std::fs::write(mount.join("new-target.doc"), b"discarded unsent target")?;
    std::fs::rename(mount.join("new-source.tmp"), mount.join("new-target.doc"))?;
    let new_id = store
        .lock()
        .unwrap()
        .lookup(&scope.file_id, "new-target.doc")?
        .node
        .id;
    worker = WorkerProcess::start(&executable)?;
    bridge.reconnect(worker.rpc.clone(), login())?;
    events = {
        let b = bridge.clone();
        std::thread::spawn(move || b.event_loop())
    };
    wait_queue(&store, &bridge).await?;
    wait(&store, &last_id, &bridge).await?;
    wait(&store, &new_id, &bridge).await?;
    assert_eq!(std::fs::metadata(mount.join("A.doc"))?.ino(), last_inode);
    verify(
        &client,
        home.view_id,
        &root,
        "A.doc",
        &target.file_id,
        second,
        scratch.path(),
    )
    .await?;
    let listed = client.list_dir(home.view_id, &root).await?;
    assert!(!listed.iter().any(|n| n.name.ends_with(".tmp")));
    let new_remote = listed
        .iter()
        .find(|n| n.name == "new-target.doc")
        .ok_or("new target absent")?;
    verify(
        &client,
        home.view_id,
        &root,
        "new-target.doc",
        &new_remote.file_id,
        b"offline entirely new save",
        scratch.path(),
    )
    .await?;
    println!("two_offline_atomic_saves_and_two_unsent_files_coalesce_passed");
    worker.stop();
    let _ = events.join().unwrap();
    std::fs::write(mount.join("conflict.tmp"), b"local atomic save conflict")?;
    std::fs::rename(mount.join("conflict.tmp"), mount.join("conflict.doc"))?;
    let conflict_local = store
        .lock()
        .unwrap()
        .lookup(&scope.file_id, "conflict.doc")?
        .node
        .id;
    let other = b"remote concurrent edit must survive\n";
    std::fs::write(&snapshot, other)?;
    client
        .upload_snapshot(
            home.view_id,
            &format!("{root}/conflict.doc"),
            conflict.version,
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
        if store.lock().unwrap().status(&conflict_local)? == Status::Conflict {
            break;
        }
        if Instant::now() > deadline {
            return Err("replacement conflict deadline".into());
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    assert_eq!(
        std::fs::read(mount.join("conflict.doc"))?,
        b"local atomic save conflict"
    );
    verify(
        &client,
        home.view_id,
        &root,
        "conflict.doc",
        &conflict.file_id,
        other,
        scratch.path(),
    )
    .await?;
    println!("replacement_conflict_preserves_remote_and_local_bodies_passed");
    stop.store(true, Ordering::Release);
    bridge.signal_upload();
    upload.join().unwrap();
    worker.stop();
    let _ = events.join().unwrap();
    mounted.join();
    drop(bridge);
    drop(store);
    let store = Store::open(scratch.path().join("state"))?;
    assert_eq!(store.lookup(&scope.file_id, "A.doc")?.ino, last_inode);
    assert_eq!(store.status(&conflict_local)?, Status::Conflict);
    let result = serde_json::json!({"passed":true,"uid":uid,"test_subtree":root,"native_protocol":true,"actual_fuse_rename_overwrite":true,"target_not_deleted_before_commit":true,"existing_remote_source_cleaned_after_target_commit":true,"remote_target_identity_preserved":true,"source_inode_and_handle_preserved":true,"old_target_handle_preserved":true,"later_upload_after_identity_rebind":true,"background_update_after_identity_rebind":true,"two_offline_atomic_saves":true,"two_unsent_files_one_server_creation":true,"concurrent_remote_and_local_content_preserved":true,"restart_preserves_inode_and_conflict":true,"libreoffice_application_tested":false,"gui_nemo":false});
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
                s.next_replace()?,
                bridge.last_error.lock().unwrap()
            )
            .into());
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}
