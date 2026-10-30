//! Real native NAS, actual Qt progress, shared limits and FUSE offline freshness.
use drive_runtime::{
    desktop::{BUS_NAME, OBJECT_PATH},
    service::Profile,
};
use sproto::{
    Client,
    actions::native::NativeVersion,
    client::{Config, Credentials},
};
use std::{io::Write, os::unix::fs::PermissionsExt, time::Duration};
#[path = "support/service.rs"]
mod support;
use support::*;
async fn json_call(p: &zbus::Proxy<'_>, method: &str) -> Result<serde_json::Value> {
    let raw: String = p.call(method, &()).await?;
    Ok(serde_json::from_str(&raw)?)
}
fn gui(args: &[String], scratch: &std::path::Path, mode: &str) -> Result<std::process::Child> {
    Ok(
        std::process::Command::new("/opt/drive-native-build/gui-venv/bin/python")
            .arg(std::path::Path::new(&args[3]).join("tests/transfer_gui_live.py"))
            .arg(mode)
            .arg(&args[4])
            .env("PYTHONPATH", &args[3])
            .env("XDG_CONFIG_HOME", scratch.join("gui-config"))
            .stdin(std::process::Stdio::null())
            .spawn()?,
    )
}
async fn sample(
    p: &zbus::Proxy<'_>,
    samples: &mut Vec<serde_json::Value>,
    start: std::time::Instant,
) -> Result<bool> {
    let transfer = json_call(p, "GetTransfers").await?;
    let partial = transfer["entries"].as_array().unwrap().iter().any(|e| {
        e["progress"]["bytes"]
            .as_u64()
            .is_some_and(|b| b > 0 && b < e["progress"]["total"].as_u64().unwrap_or(0))
    });
    if !transfer["entries"].as_array().unwrap().is_empty() {
        samples.push(serde_json::json!({"elapsed_ms":start.elapsed().as_millis(),"transfers":transfer,"A":states(p,"A.bin").await?,"B":states(p,"B.bin").await?}));
    }
    Ok(partial)
}
#[tokio::main]
async fn main() -> Result<()> {
    assert_ne!(unsafe { libc::geteuid() }, 0);
    unsafe {
        let limit = libc::rlimit {
            rlim_cur: 0,
            rlim_max: 0,
        };
        libc::setrlimit(libc::RLIMIT_CORE, &limit);
    }
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 5 || args[1] != "/DriveNative-Lab/20261028T124720-1e31763a" {
        return Err("expected authorized lab, new result, desktop directory, screenshot".into());
    }
    let output = &args[2];
    std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output)?
        .write_all(b"{\"passed\":false}")?;
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
    let root = format!("{}/transfers-{}", args[1], uuid::Uuid::new_v4());
    println!("test_subtree={root}");
    std::fs::write(
        output,
        serde_json::json!({"passed":false,"scope":root,"stage":"create fixture"}).to_string(),
    )?;
    let receipt = client.mkdir_native(home.view_id, &root).await?;
    let scratch = tempfile::tempdir()?;
    let snapshot = scratch.path().join("source");
    let mount = scratch.path().join("mount");
    std::fs::create_dir(&mount)?;
    let mut receipts = Vec::new();
    for (i, name) in ["A.bin", "B.bin"].iter().enumerate() {
        std::fs::write(&snapshot, vec![65 + i as u8; 256 * 1024])?;
        receipts.push(
            client
                .upload_snapshot(
                    home.view_id,
                    &format!("{root}/{name}"),
                    NativeVersion {
                        sync_id: 0,
                        max_id: 0,
                    },
                    &snapshot,
                )
                .await?,
        );
    }
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
    std::fs::write(&config, serde_json::to_vec(&profile)?)?;
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
    wait(&proxy, "locked", |v| v["connection"] == "locked").await?;
    let _: () = proxy.call("Unlock", &(password.clone(),)).await?;
    wait(&proxy, "online mount", |v| {
        v["connection"] == "online" && v["mounted"] == true
    })
    .await?;
    let _: () = proxy.call("Pause", &()).await?;
    if !gui(&args, scratch.path(), "configure")?.wait()?.success() {
        return Err("Qt configuration failed".into());
    }
    assert_eq!(overview(&proxy).await?["connection"], "paused");
    assert_eq!(
        json_call(&proxy, "GetTransferSettings").await?["limits"]["download_bps"],
        65536
    );
    let _: () = proxy.call("Resume", &()).await?;
    wait(&proxy, "online with limit", |v| v["connection"] == "online").await?;
    let start = std::time::Instant::now();
    let readers: Vec<_> = ["A.bin", "B.bin"]
        .iter()
        .map(|name| {
            let path = mount.join(name);
            std::thread::spawn(move || std::fs::read(path))
        })
        .collect();
    let mut samples = Vec::new();
    let mut partial = false;
    let mut gui_child = None;
    let mut event_started = false;
    let mut event_during_transfer = false;
    while readers.iter().any(|r| !r.is_finished()) {
        let seen = sample(&proxy, &mut samples, start).await?;
        partial |= seen;
        if seen && gui_child.is_none() {
            gui_child = Some(gui(&args, scratch.path(), "observe")?);
        }
        if seen && !event_started {
            client
                .mkdir_native(home.view_id, &format!("{root}/DuringTransfer"))
                .await?;
            event_started = true;
        }
        if event_started
            && states(&proxy, "DuringTransfer").await?["error"].is_null()
            && mount.join("DuringTransfer").is_dir()
        {
            event_during_transfer = true;
        }
        if start.elapsed() > Duration::from_secs(100) {
            return Err("concurrent download deadline".into());
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    for (i, reader) in readers.into_iter().enumerate() {
        assert_eq!(reader.join().unwrap()?, vec![65 + i as u8; 256 * 1024]);
    }
    let download_seconds = start.elapsed().as_secs_f64();
    std::fs::write(
        output,
        serde_json::to_vec_pretty(
            &serde_json::json!({"passed":false,"scope":root,"stage":"downloads complete","partial":partial,"event_during_transfer":event_during_transfer,"download_seconds":download_seconds,"samples":samples}),
        )?,
    )?;
    assert!(partial && event_during_transfer);
    assert!(download_seconds >= 7.5, "shared download budget violated");
    assert!(
        gui_child
            .ok_or("no actual partial transfer")?
            .wait()?
            .success()
    );
    wait_state(&proxy, "A.bin", "local_complete").await?;
    wait_state(&proxy, "B.bin", "local_complete").await?;
    println!("shared download budget and actual Qt progress passed: {download_seconds:.2}s");
    let _: () = proxy.call("Pause", &()).await?;
    let edited = vec![91; 256 * 1024];
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(mount.join("A.bin"))?;
    file.write_all(&edited)?;
    file.sync_all()?;
    drop(file);
    assert_ne!(states(&proxy, "A.bin").await?["status"], "local_complete");
    let _: () = proxy.call("Resume", &()).await?;
    wait(&proxy, "online upload", |v| v["connection"] == "online").await?;
    let upload_start = std::time::Instant::now();
    let mut upload_partial = false;
    loop {
        upload_partial |= sample(&proxy, &mut samples, start).await?;
        if states(&proxy, "A.bin").await?["status"] == "local_complete" {
            break;
        }
        if upload_start.elapsed() > Duration::from_secs(100) {
            return Err("upload deadline".into());
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    let upload_seconds = upload_start.elapsed().as_secs_f64();
    std::fs::write(
        output,
        serde_json::to_vec_pretty(
            &serde_json::json!({"passed":false,"scope":root,"stage":"upload complete","upload_partial":upload_partial,"upload_seconds":upload_seconds,"samples":samples}),
        )?,
    )?;
    assert!(upload_partial);
    assert!(upload_seconds >= 7.0, "upload limit violated");
    let nodes = client.list_dir(home.view_id, &root).await?;
    let n = nodes.iter().find(|n| n.name == "A.bin").unwrap();
    let expected = sproto::actions::transfer::ContentVersion {
        file_id: n.file_id.clone(),
        version: NativeVersion {
            sync_id: n.sync_id,
            max_id: n.max_id,
        },
        size: n.file_size,
        hash: n.hash.clone(),
    };
    let checked = scratch.path().join("remote-readback");
    client.download_checked(&expected, &checked).await?;
    assert_eq!(std::fs::read(checked)?, edited);
    println!("offline write, limited upload and independent readback passed: {upload_seconds:.2}s");
    // Remote edit while the mount remains online; never open local A until after pause.
    let updated = vec![97; 512 * 1024];
    std::fs::write(&snapshot, &updated)?;
    client
        .upload_snapshot(
            home.view_id,
            &format!("{root}/A.bin"),
            expected.version,
            &snapshot,
        )
        .await?;
    let background_start = std::time::Instant::now();
    let mut background_partial = false;
    let mut changed = None;
    loop {
        let seen = sample(&proxy, &mut samples, start).await?;
        background_partial |= seen;
        if seen && changed.is_none() {
            changed = Some(gui(&args, scratch.path(), "change")?);
        }
        if background_partial
            && states(&proxy, "A.bin").await?["status"] == "local_complete"
            && json_call(&proxy, "GetTransfers").await?["entries"]
                .as_array()
                .unwrap()
                .is_empty()
        {
            break;
        }
        if background_start.elapsed() > Duration::from_secs(100) {
            return Err("background refresh deadline".into());
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    assert!(
        changed
            .ok_or("no background byte progress")?
            .wait()?
            .success()
    );
    assert_eq!(
        json_call(&proxy, "GetTransferSettings").await?["limits"]["download_bps"],
        131072
    );
    let _: () = proxy.call("Pause", &()).await?;
    assert_eq!(std::fs::read(mount.join("A.bin"))?, updated);
    assert_eq!(std::fs::read(mount.join("B.bin"))?, vec![66; 256 * 1024]);
    stop(&proxy, &mut daemon).await?;
    daemon = spawn(&executable, &config, &data, &mount, &log)?;
    wait(&proxy, "locked restart", |v| {
        v["connection"] == "locked" && v["mounted"] == true
    })
    .await?;
    let limits = json_call(&proxy, "GetTransferSettings").await?;
    assert_eq!(
        limits["limits"],
        serde_json::json!({"download_bps":131072,"upload_bps":32768})
    );
    assert_eq!(std::fs::read(mount.join("A.bin"))?, updated);
    assert!(
        json_call(&proxy, "GetTransfers").await?["entries"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    stop(&proxy, &mut daemon).await?;
    assert!(!std::fs::read_to_string(log)?.contains(&password));
    std::fs::write(
        output,
        serde_json::to_vec_pretty(
            &serde_json::json!({"passed":true,"scope":root,"server":"DSM 7.4.1 / Drive Server 4.0.3-27892","actual_qt_settings_and_live_progress":true,"concurrent_download_seconds":download_seconds,"upload_seconds":upload_seconds,"event_received_during_throttled_body":event_during_transfer,"offline_upload_independent_readback":true,"cached_unpinned_background_update_without_reopen":true,"live_gui_limit_change":true,"paused_offline_new_content":true,"locked_restart_cache_and_policy":true,"samples":samples,"mint_tested":false}),
        )?,
    )?;
    println!(
        "PASS native FUSE transfer progress, shared pacing, Qt changes and offline background freshness"
    );
    Ok(())
}
