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
    io::Read,
    sync::{Arc, atomic::Ordering},
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
    let absent_name = format!("uncached-{}.doc", uuid::Uuid::new_v4());
    let old = b"native worker first revision\n";
    let new = b"native worker updated remotely while file stays closed\n";
    std::fs::write(&source, old)?;
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
    writer
        .upload_snapshot(
            home.view_id,
            &format!("{lab}/{absent_name}"),
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
    let mut worker = WorkerProcess::start(&worker_path)?;
    let store = Arc::new(drive_core::SharedStore::new(Store::open(scratch.path().join("state"))?));
    let password_for_restart = password.clone();
    let device_id = uuid::Uuid::new_v4().to_string();
    let bridge = Arc::new(Bridge::connect(
        store.clone(),
        worker.rpc.clone(),
        Login {
            host: "192.0.2.10".into(),
            tls_name: "nas.example.invalid".into(),
            trusted_ca_pem: Vec::new(),
            username: "test-user".into(),
            password,
            device_id: device_id.clone(),
            scope: scope.clone(),
        },
    )?);
    bridge.pull()?;
    let filesystem = drive_fs::CacheFilesystem::new(store.clone(), &scope.file_id, uid, gid)?
        .with_hydrator(bridge.clone());
    let mount_session = fuser::spawn_mount2(
        filesystem,
        &mount,
        &[
            fuser::MountOption::RO,
            fuser::MountOption::DefaultPermissions,
            fuser::MountOption::NoDev,
            fuser::MountOption::NoSuid,
            fuser::MountOption::FSName("drive-native-live-test".into()),
        ],
    )?;
    bridge.set_notifier(mount_session.notifier());
    assert_eq!(
        std::fs::metadata(mount.join(&name))?.len(),
        old.len() as u64
    );
    let _ = std::fs::read_dir(&mount)?.collect::<Result<Vec<_>, _>>()?;
    assert_eq!(bridge.downloads.load(Ordering::Relaxed), 0);
    println!("native_metadata_mounted_without_body_download");
    let events = {
        let bridge = bridge.clone();
        std::thread::spawn(move || bridge.event_loop())
    };
    let stop_transfers = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let transfers = {
        let b = bridge.clone();
        let stop = stop_transfers.clone();
        std::thread::spawn(move || b.upload_loop(&stop))
    };
    let readers = (0..8)
        .map(|_| {
            let path = mount.join(&name);
            std::thread::spawn(move || std::fs::read(path))
        })
        .collect::<Vec<_>>();
    for reader in readers {
        assert_eq!(reader.join().unwrap()?, old)
    }
    assert_eq!(bridge.downloads.load(Ordering::Relaxed), 1);
    let old_version = store.lock().unwrap().node(&first.file_id)?.version;
    assert!(!store.lock().unwrap().is_pinned(&first.file_id)?);
    println!("concurrent_on_demand_read_passed_single_native_transfer");
    std::fs::write(&source, new)?;
    writer
        .upload_snapshot(home.view_id, &remote, first.version, &source)
        .await?;
    let deadline = Instant::now() + Duration::from_secs(60);
    loop {
        let ready = {
            let s = store.lock().unwrap();
            s.node(&first.file_id)?.version != old_version
                && s.status(&first.file_id)? == Status::LocalComplete
        };
        if ready {
            break;
        }
        if Instant::now() > deadline {
            return Err("background refresh deadline exceeded".into());
        }
        tokio::time::sleep(Duration::from_millis(30)).await;
    }
    assert_eq!(bridge.downloads.load(Ordering::Relaxed), 2);
    println!("unpinned_background_refresh_completed_without_reopening");
    worker.stop();
    drop(writer);
    assert!(events.join().unwrap().is_err());
    assert_eq!(std::fs::read(mount.join(&name))?, new);
    let error = std::fs::read(mount.join(&absent_name)).unwrap_err();
    assert_eq!(error.raw_os_error(), Some(libc::ENETDOWN));
    worker = WorkerProcess::start(&worker_path)?;
    bridge.reconnect(
        worker.rpc.clone(),
        Login {
            host: "192.0.2.10".into(),
            tls_name: "nas.example.invalid".into(),
            trusted_ca_pem: Vec::new(),
            username: "test-user".into(),
            password: password_for_restart,
            device_id,
            scope,
        },
    )?;
    bridge.pull()?;
    assert_eq!(std::fs::read(mount.join(&absent_name))?, old);
    assert_eq!(std::fs::read(mount.join(&name))?, new);
    println!("worker_reconnected_without_remount_and_uncached_file_downloaded");
    worker.stop();
    stop_transfers.store(true, Ordering::Release);
    bridge.signal_upload();
    transfers.join().unwrap();
    mount_session.join();
    drop(bridge);
    drop(store);
    let s = Store::open(scratch.path().join("state"))?;
    let mut bytes = Vec::new();
    s.open_cached(&first.file_id)?.read_to_end(&mut bytes)?;
    assert_eq!(bytes, new);
    let result = serde_json::json!({"passed":true,"uid":uid,"worker_pid":worker.pid(),"native_worker_separate_process":true,"fd_content_transfer":true,"directory_browsing_downloads":0,"concurrent_readers":8,"foreground_native_transfers":1,"unpinned_background_refresh_without_reopen":true,"worker_killed_before_offline_mounted_read":true,"uncached_offline_error":"ENETDOWN","state_reopened_after_unmount":true,"worker_reconnected_without_remount":true,"lab_path":lab,"writable_fuse":false,"nemo_tested":false});
    if let Some(report) = args.get(2) {
        std::fs::write(report, serde_json::to_vec_pretty(&result)?)?
    }
    println!("{result}");
    Ok(())
}
