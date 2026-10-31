//! Explicit live integration. NAS mutations are confined to the authorized lab.
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
    io::{Read, Write},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let uid = unsafe { libc::getuid() };
    let gid = unsafe { libc::getgid() };
    if uid == 0 {
        return Err("run as an ordinary Linux user".into());
    }
    let args: Vec<_> = std::env::args().collect();
    let lab = args
        .get(1)
        .ok_or("expected isolated /DriveNative-Lab/<run>")?;
    let parts: Vec<_> = lab.split('/').collect();
    if parts.len() != 3
        || parts[1] != "DriveNative-Lab"
        || parts[2].is_empty()
        || parts[2].contains("..")
        || lab.contains('\0')
    {
        return Err("invalid lab path".into());
    }
    let password = rpassword::prompt_password("Test account password (not saved): ")?;
    let config = Config::builder()
        .host("192.0.2.10")
        .tls_server_name("nas.example.invalid".to_string())
        .credentials(Credentials::Password {
            username: "test-user".into(),
            password: password.clone(),
            otp: None,
        })
        .build();
    let writer = Client::connect(config).await?;
    let home = writer.personal_root().await?;
    if !home.writable || !home.download_allowed {
        return Err("test account lacks required Drive permissions".into());
    }
    let roots = writer.list_dir(home.view_id, "/DriveNative-Lab").await?;
    let root = roots
        .iter()
        .find(|n| n.name == parts[2])
        .ok_or("lab does not exist")?;
    let scope = Scope {
        view_id: home.view_id,
        node_id: root.node_id,
        file_id: root.file_id.clone(),
        path: lab.clone(),
    };
    let scratch = tempfile::tempdir()?;
    let source = scratch.path().join("source");
    let mount = scratch.path().join("mount");
    std::fs::create_dir(&mount)?;
    let name = format!("mounted-{}.doc", uuid::Uuid::new_v4());
    let remote = format!("{lab}/{name}");

    let initial = b"initial native writable file\n";
    std::fs::write(&source, initial)?;
    let first = writer
        .upload_snapshot(
            home.view_id,
            &remote,
            NativeVersion {
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
    let device_id = uuid::Uuid::new_v4().to_string();
    let login = || Login {
        host: "192.0.2.10".into(),
        tls_name: "nas.example.invalid".into(),
        trusted_ca_pem: Vec::new(),
        username: "test-user".into(),
        password: password.clone(),
        device_id: device_id.clone(),
        scope: scope.clone(),
    };

    let mut worker = WorkerProcess::start(&worker_path)?;
    let store = Arc::new(drive_core::SharedStore::new(Store::open(scratch.path().join("state"))?));
    let bridge = Arc::new(Bridge::connect(store.clone(), worker.rpc.clone(), login())?);
    bridge.pull()?;
    let filesystem = drive_fs::CacheFilesystem::new(store.clone(), &scope.file_id, uid, gid)?
        .with_hydrator(bridge.clone())
        .with_writes_enabled();
    let session = fuser::spawn_mount2(
        filesystem,
        &mount,
        &[
            fuser::MountOption::RW,
            fuser::MountOption::DefaultPermissions,
            fuser::MountOption::NoDev,
            fuser::MountOption::NoSuid,
            fuser::MountOption::FSName("drive-native-write-test".into()),
        ],
    )?;
    bridge.set_notifier(session.notifier());
    let local = mount.join(&name);
    assert_eq!(std::fs::read(&local)?, initial);
    let mut events = {
        let b = bridge.clone();
        std::thread::spawn(move || b.event_loop())
    };
    let stop = Arc::new(AtomicBool::new(false));
    let uploads = {
        let b = bridge.clone();
        let stop = stop.clone();
        std::thread::spawn(move || b.upload_loop(&stop))
    };
    let created_name = format!("fuse-created-{}.doc", uuid::Uuid::new_v4());
    let created_path = mount.join(&created_name);
    let mut created = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&created_path)?;
    created.write_all(b"created through FUSE before remote identity exists\n")?;
    created.sync_all()?;
    drop(created);
    let record = store
        .lock()
        .unwrap()
        .lookup(&scope.file_id, &created_name)?;
    let local_id = record.node.id.clone();
    let inode = record.ino;
    wait_status(&store, &local_id, Status::LocalComplete, &bridge).await?;
    assert_eq!(
        store
            .lock()
            .unwrap()
            .lookup(&scope.file_id, &created_name)?
            .ino,
        inode
    );
    verify(
        &writer,
        home.view_id,
        lab,
        &created_name,
        scratch.path(),
        b"created through FUSE before remote identity exists\n",
    )
    .await?;
    save(&created_path, b"updated after native identity binding\n")?;
    wait_status(&store, &local_id, Status::LocalComplete, &bridge).await?;
    verify(
        &writer,
        home.view_id,
        lab,
        &created_name,
        scratch.path(),
        b"updated after native identity binding\n",
    )
    .await?;
    println!("fuse_create_remote_identity_binding_and_second_save_passed");
    let online = b"written from actual Linux FUSE mount\n";
    save(&local, online)?;
    wait_status(&store, &first.file_id, Status::LocalComplete, &bridge).await?;
    println!("online_commit_confirmed_before_independent_readback");
    let second = verify(&writer, home.view_id, lab, &name, scratch.path(), online).await?;
    println!("online_fuse_save_native_commit_readback_passed");
    worker.stop();
    assert!(events.join().unwrap().is_err());
    let offline = b"saved offline, uploaded after worker reconnect\n";
    save(&local, offline)?;
    assert_eq!(
        store.lock().unwrap().status(&first.file_id)?,
        Status::PendingUpload
    );
    assert_eq!(std::fs::read(&local)?, offline);
    let offline_name = format!("offline-created-{}.doc", uuid::Uuid::new_v4());
    let offline_path = mount.join(&offline_name);
    let mut new_file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&offline_path)?;
    new_file.write_all(b"new file created fully offline\n")?;
    new_file.sync_all()?;
    drop(new_file);
    let offline_id = store
        .lock()
        .unwrap()
        .lookup(&scope.file_id, &offline_name)?
        .node
        .id;
    assert_eq!(
        store.lock().unwrap().status(&offline_id)?,
        Status::PendingUpload
    );
    worker = WorkerProcess::start(&worker_path)?;
    bridge.reconnect(worker.rpc.clone(), login())?;
    events = {
        let b = bridge.clone();
        std::thread::spawn(move || b.event_loop())
    };
    wait_status(&store, &first.file_id, Status::LocalComplete, &bridge).await?;
    println!("offline_commit_confirmed_before_independent_readback");
    wait_status(&store, &offline_id, Status::LocalComplete, &bridge).await?;
    verify(
        &writer,
        home.view_id,
        lab,
        &offline_name,
        scratch.path(),
        b"new file created fully offline\n",
    )
    .await?;
    println!("offline_fuse_create_and_reconnect_passed");
    let third = verify(&writer, home.view_id, lab, &name, scratch.path(), offline).await?;
    assert_ne!(second.version, third.version);
    println!("offline_fuse_save_reconnect_native_commit_passed");
    worker.stop();
    assert!(events.join().unwrap().is_err());
    let local_conflict = b"local edit must survive remote conflict\n";
    let remote_conflict = b"independent remote edit must survive local conflict\n";
    save(&local, local_conflict)?;
    std::fs::write(&source, remote_conflict)?;
    writer
        .upload_snapshot(home.view_id, &remote, third.version, &source)
        .await?;
    worker = WorkerProcess::start(&worker_path)?;
    bridge.reconnect(worker.rpc.clone(), login())?;
    events = {
        let b = bridge.clone();
        std::thread::spawn(move || b.event_loop())
    };
    wait_status(&store, &first.file_id, Status::Conflict, &bridge).await?;
    assert_eq!(std::fs::read(&local)?, local_conflict);
    verify(
        &writer,
        home.view_id,
        lab,
        &name,
        scratch.path(),
        remote_conflict,
    )
    .await?;
    println!("two_sided_edit_conflict_preserved_local_and_remote");
    stop.store(true, Ordering::Release);
    bridge.signal_upload();
    worker.stop();
    let _ = events.join().unwrap();
    uploads.join().unwrap();
    session.join();
    drop(bridge);
    drop(store);
    let s = Store::open(scratch.path().join("state"))?;
    let mut recovered = Vec::new();
    s.open_cached(&first.file_id)?.read_to_end(&mut recovered)?;
    assert_eq!(recovered, local_conflict);
    assert_eq!(s.status(&first.file_id)?, Status::Conflict);
    let result = serde_json::json!({"passed":true,"uid":uid,"lab_path":lab,"actual_fuse_write":true,"native_upload":true,"online_commit_independent_readback":true,"offline_queue_reconnect":true,"two_sided_conflict_preserved":true,"conflict_survives_unmount_restart":true,"fuse_file_create_and_update":true,"offline_fuse_create_reconnect":true,"create_rename_delete":false,"atomic_save":false,"gui_nemo":false});
    if let Some(report) = args.get(2) {
        std::fs::write(report, serde_json::to_vec_pretty(&result)?)?;
    }
    println!("{result}");
    Ok(())
}
fn save(path: &std::path::Path, body: &[u8]) -> std::io::Result<()> {
    let mut f = std::fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(path)?;
    f.write_all(body)?;
    f.sync_all()?;
    drop(f);
    Ok(())
}
async fn wait_status(
    store: &Arc<drive_core::SharedStore>,
    id: &str,
    wanted: Status,
    bridge: &Bridge,
) -> Result<(), Box<dyn std::error::Error>> {
    let deadline = Instant::now() + Duration::from_secs(60);
    loop {
        let status = store.lock().unwrap().status(id)?;
        if status == wanted {
            return Ok(());
        }
        if Instant::now() > deadline {
            return Err(format!(
                "status deadline: wanted {wanted:?}, got {status:?}; runtime {:?}",
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
    lab: &str,
    name: &str,
    dir: &std::path::Path,
    body: &[u8],
) -> Result<sproto::actions::transfer::ContentVersion, Box<dyn std::error::Error>> {
    let nodes = client.list_dir(view, lab).await?;
    let n = nodes
        .iter()
        .find(|n| n.name == name)
        .ok_or("file missing after upload")?;
    let expected = sproto::actions::transfer::ContentVersion {
        file_id: n.file_id.clone(),
        version: NativeVersion {
            sync_id: n.sync_id,
            max_id: n.max_id,
        },
        size: n.file_size,
        hash: n.hash.clone(),
    };
    let path = dir.join(format!("verify-{}", uuid::Uuid::new_v4()));
    client.download_checked(&expected, &path).await?;
    assert_eq!(std::fs::read(path)?, body);
    Ok(expected)
}
