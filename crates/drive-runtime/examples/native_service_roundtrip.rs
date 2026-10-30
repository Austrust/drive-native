//! Explicit real NAS + ordinary-user FUSE + session D-Bus lifecycle validation.
//! Only a new subtree below the supplied authorized laboratory is modified.
use drive_runtime::{
    desktop::{BUS_NAME, OBJECT_PATH},
    service::Profile,
};
use sproto::{
    Client,
    actions::native::NativeVersion,
    client::{Config, Credentials},
};
use std::{
    os::unix::fs::PermissionsExt,
    time::{Duration, Instant},
};
#[path = "support/service.rs"]
mod support;
use support::*;
fn worker_pid(parent: u32) -> Result<Option<u32>> {
    // Linux children are owned by the thread that spawned them. The network
    // worker is spawned by the supervisor thread, not the async main thread.
    for task in std::fs::read_dir(format!("/proc/{parent}/task"))? {
        let task = task?;
        let Ok(children) = std::fs::read_to_string(task.path().join("children")) else {
            continue;
        };
        for child in children.split_whitespace() {
            let pid: u32 = child.parse()?;
            if std::fs::read_link(format!("/proc/{pid}/exe"))
                .is_ok_and(|p| p.file_name().is_some_and(|n| n == "drive-native-worker"))
            {
                return Ok(Some(pid));
            }
        }
    }
    Ok(None)
}
#[tokio::main]
async fn main() -> Result<()> {
    assert_ne!(unsafe { libc::geteuid() }, 0);
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
        return Err("invalid lab scope".into());
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
    let root = format!("{lab}/service-{}", uuid::Uuid::new_v4());
    let receipt = client.mkdir_native(home.view_id, &root).await?;
    let scratch = tempfile::tempdir()?;
    let snapshot = scratch.path().join("snapshot");
    let mount = scratch.path().join("mount");
    std::fs::create_dir(&mount)?;
    let zero = NativeVersion {
        sync_id: 0,
        max_id: 0,
    };
    std::fs::write(&snapshot, b"original-v1")?;
    let first = client
        .upload_snapshot(
            home.view_id,
            &format!("{root}/A.doc"),
            zero.clone(),
            &snapshot,
        )
        .await?;
    client
        .mkdir_native(home.view_id, &format!("{root}/Pinned"))
        .await?;
    std::fs::write(&snapshot, b"pinned-v1")?;
    client
        .upload_snapshot(
            home.view_id,
            &format!("{root}/Pinned/B.txt"),
            zero.clone(),
            &snapshot,
        )
        .await?;
    let profile = Profile {
        schema: 1,
        task_id: uuid::Uuid::new_v4().to_string(),
        host: "192.0.2.10".into(),
        tls_name: "nas.example.invalid".into(),
        trusted_ca_pem: Vec::new(),
        username: "test-user".into(),
        device_id: uuid::Uuid::new_v4().to_string(),
        scope: drive_core::native::Scope {
            view_id: home.view_id,
            node_id: receipt.node_id,
            file_id: receipt.file_id,
            path: root.clone(),
        },
        mount: mount.clone(),
        writable: true,
    };
    let config = scratch.path().join("profile.json");
    std::fs::write(&config, serde_json::to_vec_pretty(&profile)?)?;
    std::fs::set_permissions(&config, std::fs::Permissions::from_mode(0o600))?;
    let data = scratch.path().join("data");
    let log = scratch.path().join("fsd.log");
    let executable = std::env::current_exe()?
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("drive-native-fsd");
    let connection = zbus::Connection::session().await?;
    let proxy = zbus::Proxy::new(&connection, BUS_NAME, OBJECT_PATH, BUS_NAME).await?;
    let mut daemon = spawn(&executable, &config, &data, &mount, &log)?;
    wait(&proxy, "locked before login", |v| {
        v["connection"] == "locked"
    })
    .await?;
    let _: () = proxy.call("Unlock", &(password.clone(),)).await?;
    wait(&proxy, "online mounted", |v| {
        v["connection"] == "online" && v["mounted"] == true
    })
    .await?;
    let before = overview(&proxy).await?;
    assert_eq!(before["downloads_started"], 0);
    let _ = std::fs::read_dir(&mount)?.collect::<std::io::Result<Vec<_>>>()?;
    assert_eq!(states(&proxy, "A.doc").await?["status"], "online_only");
    assert_eq!(overview(&proxy).await?["downloads_started"], 0);
    assert_eq!(std::fs::read(mount.join("A.doc"))?, b"original-v1");
    wait_state(&proxy, "A.doc", "local_complete").await?;
    println!("PASS persistent daemon login, metadata-only browsing and native hydration");

    let mut signals = proxy.receive_signal("StatusChanged").await?;
    std::fs::write(&snapshot, b"background-v2")?;
    client
        .upload_snapshot(
            home.view_id,
            &format!("{root}/A.doc"),
            first.version,
            &snapshot,
        )
        .await?;
    wait(&proxy, "background download without reopen", |v| {
        v["downloads_started"].as_u64().unwrap_or(0) >= 2
    })
    .await?;
    wait_state(&proxy, "A.doc", "local_complete").await?;
    use zbus::export::futures_core::Stream;
    let signal = tokio::time::timeout(
        Duration::from_secs(5),
        std::future::poll_fn(|cx| std::pin::Pin::new(&mut signals).poll_next(cx)),
    )
    .await?;
    assert!(signal.is_some());
    let _: () = proxy.call("Pause", &()).await?;
    assert_eq!(overview(&proxy).await?["connection"], "paused");
    assert_eq!(std::fs::read(mount.join("A.doc"))?, b"background-v2");
    std::fs::write(mount.join("A.doc"), b"offline-v3")?;
    wait_state(&proxy, "A.doc", "pending_upload").await?;
    let _: () = proxy.call("Resume", &()).await?;
    wait(&proxy, "resume", |v| v["connection"] == "online").await?;
    wait_state(&proxy, "A.doc", "local_complete").await?;
    let remote = client
        .list_dir(home.view_id, &root)
        .await?
        .into_iter()
        .find(|n| n.name == "A.doc")
        .ok_or("remote A missing")?;
    let verify = scratch.path().join("verify");
    client
        .download_checked(
            &sproto::actions::transfer::ContentVersion {
                file_id: remote.file_id,
                version: NativeVersion {
                    sync_id: remote.sync_id,
                    max_id: remote.max_id,
                },
                size: remote.file_size,
                hash: remote.hash,
            },
            &verify,
        )
        .await?;
    assert_eq!(std::fs::read(&verify)?, b"offline-v3");
    println!(
        "PASS status signal, unopened background refresh, paused cached read and offline write replay"
    );
    let old_worker = worker_pid(daemon.child.id())?.ok_or("native child absent")?;
    assert_eq!(unsafe { libc::kill(old_worker as i32, libc::SIGKILL) }, 0);
    assert_eq!(std::fs::read(mount.join("A.doc"))?, b"offline-v3");
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        if worker_pid(daemon.child.id())?.is_some_and(|pid| pid != old_worker)
            && overview(&proxy).await?["connection"] == "online"
        {
            break;
        }
        if Instant::now() > deadline {
            return Err("worker crash reconnect deadline".into());
        }
        tokio::time::sleep(Duration::from_millis(75)).await;
    }
    println!(
        "PASS killed network worker reconnects automatically while cached mount stays available"
    );

    let _: () = proxy.call("SetPinned", &("Pinned", true)).await?;
    wait_state(&proxy, "Pinned", "pinned_complete").await?;
    std::fs::write(&snapshot, b"future pinned child")?;
    client
        .upload_snapshot(
            home.view_id,
            &format!("{root}/Pinned/C.txt"),
            zero,
            &snapshot,
        )
        .await?;
    wait_state(&proxy, "Pinned/C.txt", "pinned_complete").await?;
    assert_eq!(states(&proxy, "Pinned").await?["status"], "pinned_complete");
    let _: () = proxy.call("SetPinned", &("Pinned", false)).await?;
    assert_eq!(states(&proxy, "Pinned").await?["status"], "local_complete");
    let released: String = proxy.call("ReleaseSpace", &("Pinned",)).await?;
    let released: serde_json::Value = serde_json::from_str(&released)?;
    assert_eq!(released["released"].as_array().unwrap().len(), 2);
    assert!(
        released["protected_or_failed"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert_eq!(states(&proxy, "Pinned").await?["status"], "online_only");
    assert_eq!(
        client
            .list_dir(home.view_id, &format!("{root}/Pinned"))
            .await?
            .len(),
        2
    );
    println!(
        "PASS pin wakes immediately, future child inherits, unpin retains cache, release keeps NAS files"
    );

    drop(signals);
    drop(proxy);
    drop(connection);
    let connection = zbus::Connection::session().await?;
    let proxy = zbus::Proxy::new(&connection, BUS_NAME, OBJECT_PATH, BUS_NAME).await?;
    assert_eq!(overview(&proxy).await?["connection"], "online");
    assert_eq!(std::fs::read(mount.join("A.doc"))?, b"offline-v3");
    println!("PASS closing desktop D-Bus connection does not stop service or mount");
    stop(&proxy, &mut daemon).await?;
    drop(daemon);
    let mut daemon = spawn(&executable, &config, &data, &mount, &log)?;
    wait(&proxy, "offline restart mounted before unlock", |v| {
        v["connection"] == "locked" && v["mounted"] == true
    })
    .await?;
    assert_eq!(std::fs::read(mount.join("A.doc"))?, b"offline-v3");
    assert!(std::fs::read(mount.join("Pinned/B.txt")).is_err());
    stop(&proxy, &mut daemon).await?;
    let diagnostics = std::fs::read_to_string(&log)?;
    assert!(!diagnostics.contains(&password));
    let report = serde_json::json!({"result":"passed","subtree":root,"ordinary_uid":unsafe{libc::geteuid()},
        "real_native_server":true,"dbus_session":true,"separate_fsd_process":true,"native_hydration":true,
        "metadata_browsing_zero_downloads":true,"background_refresh_without_reopen":true,"status_signal":true,
        "pause_retains_mount":true,"worker_crash_auto_reconnect":true,"desktop_disconnect_retains_service":true,"offline_write_reconnect":true,"pin_wakeup_and_new_child":true,
        "release_keeps_remote":true,"offline_restart_before_unlock":true,"mint_gui_verified":false});
    if let Some(path) = args.get(2) {
        std::fs::write(path, serde_json::to_vec_pretty(&report)?)?;
    }
    println!("PASS offline fsd restart without credentials; all session-bus service checks passed");
    Ok(())
}
