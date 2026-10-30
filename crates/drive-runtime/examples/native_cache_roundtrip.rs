//! Real NAS, Qt settings, protected eviction, rehydration and locked restart.
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
    io::{Read, Write},
    os::unix::fs::PermissionsExt,
    time::Duration,
};
#[path = "support/service.rs"]
mod support;
use support::*;
async fn cache(proxy: &zbus::Proxy<'_>) -> Result<serde_json::Value> {
    let raw: String = proxy.call("GetCacheSettings", &()).await?;
    Ok(serde_json::from_str(&raw)?)
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
    let root = format!("{}/cache-budget-{}", args[1], uuid::Uuid::new_v4());
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
    for (i, name) in ["A.bin", "B.bin", "C.bin"].iter().enumerate() {
        std::fs::write(&snapshot, vec![65 + i as u8; 600 * 1024])?;
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
    for (i, name) in ["A.bin", "B.bin", "C.bin"].iter().enumerate() {
        assert_eq!(
            std::fs::read(mount.join(name))?,
            vec![65 + i as u8; 600 * 1024]
        );
    }
    let mut held = std::fs::File::open(mount.join("B.bin"))?;
    let mut one = [0];
    held.read_exact(&mut one)?;
    assert_eq!(one, [66]);
    let _: () = proxy.call("SetPinned", &("C.bin", true)).await?;
    wait_state(&proxy, "C.bin", "pinned_complete").await?;
    let _: () = proxy.call("Pause", &()).await?;
    let downloads = overview(&proxy).await?["downloads_started"].clone();
    let gui = std::process::Command::new("/opt/drive-native-build/gui-venv/bin/python")
        .arg(std::path::Path::new(&args[3]).join("tests/cache_gui_live.py"))
        .arg(&args[4])
        .env("PYTHONPATH", &args[3])
        .env("XDG_CONFIG_HOME", scratch.path().join("gui-config"))
        .stdin(std::process::Stdio::null())
        .status()?;
    if !gui.success() {
        return Err("cache GUI action failed".into());
    }
    assert_eq!(cache(&proxy).await?["usage"]["limit_bytes"], 1048576);
    assert_eq!(overview(&proxy).await?["downloads_started"], downloads);
    std::fs::write(output,serde_json::json!({"passed":false,"scope":root,"stage":"GUI saved; verifying paused retention"}).to_string())?;
    // Exceed the actual publication grace while paused; local icon queries must
    // neither evict contents nor count as user opens for the later LRU decision.
    for i in 0..13 {
        tokio::time::sleep(Duration::from_secs(5)).await;
        assert_eq!(states(&proxy, "A.bin").await?["status"], "local_complete");
        if i % 3 == 0 {
            println!("paused retention: {}s", (i + 1) * 5);
        }
    }
    let _: () = proxy.call("Resume", &()).await?;
    wait(&proxy, "online after pause", |v| {
        v["connection"] == "online"
    })
    .await?;
    wait_state(&proxy, "A.bin", "online_only").await?;
    assert_eq!(states(&proxy, "B.bin").await?["status"], "local_complete");
    assert_eq!(states(&proxy, "C.bin").await?["status"], "pinned_complete");
    let usage = cache(&proxy).await?;
    assert!(usage["usage"]["over_target_bytes"].as_u64().unwrap() > 0);
    drop(held);
    wait_state(&proxy, "B.bin", "online_only").await?;
    assert_eq!(cache(&proxy).await?["usage"]["over_target_bytes"], 0);
    println!(
        "online LRU eviction retained pinned and open contents; released closed handle next pass"
    );
    let _: () = proxy.call("Pause", &()).await?;
    assert_eq!(std::fs::read(mount.join("C.bin"))?, vec![67; 600 * 1024]);
    let paused_downloads = overview(&proxy).await?["downloads_started"].clone();
    for _ in 0..6 {
        assert!(std::fs::read(mount.join("A.bin")).is_err());
    }
    assert_eq!(
        overview(&proxy).await?["downloads_started"],
        paused_downloads
    );
    println!("stage: paused reads verified; checking remote identity");
    std::fs::write(output, serde_json::json!({"passed":false,"scope":root,"stage":"paused reads verified; checking remote identity"}).to_string())?;
    let nodes = client.list_dir(home.view_id, &root).await?;
    assert_eq!(nodes.len(), 3);
    for (name, receipt) in ["A.bin", "B.bin", "C.bin"].iter().zip(receipts.iter()) {
        let node = nodes
            .iter()
            .find(|n| n.name == *name)
            .ok_or("remote file was removed")?;
        assert_eq!(node.file_id, receipt.file_id);
        assert_eq!(node.sync_id, receipt.version.sync_id);
        assert_eq!(node.max_id, receipt.version.max_id);
        assert_eq!(node.hash, receipt.hash);
    }
    let _: () = proxy.call("Resume", &()).await?;
    wait(&proxy, "online for rehydration", |v| {
        v["connection"] == "online"
    })
    .await?;
    println!("stage: online rehydration");
    std::fs::write(
        output,
        serde_json::json!({"passed":false,"scope":root,"stage":"online rehydration"}).to_string(),
    )?;
    assert_eq!(std::fs::read(mount.join("A.bin"))?, vec![65; 600 * 1024]);
    let _: () = proxy.call("Pause", &()).await?;
    stop(&proxy, &mut daemon).await?;
    println!("stage: locked restart");
    std::fs::write(
        output,
        serde_json::json!({"passed":false,"scope":root,"stage":"locked restart"}).to_string(),
    )?;
    daemon = spawn(&executable, &config, &data, &mount, &log)?;
    wait(&proxy, "locked cached restart", |v| {
        v["connection"] == "locked" && v["mounted"] == true
    })
    .await?;
    assert_eq!(cache(&proxy).await?["usage"]["limit_bytes"], 1048576);
    assert_eq!(std::fs::read(mount.join("C.bin"))?, vec![67; 600 * 1024]);
    assert_eq!(std::fs::read(mount.join("A.bin"))?, vec![65; 600 * 1024]);
    stop(&proxy, &mut daemon).await?;
    assert!(!std::fs::read_to_string(log)?.contains(&password));
    std::fs::write(
        output,
        serde_json::to_vec_pretty(
            &serde_json::json!({"passed":true,"scope":root,"actual_qt_cache_settings":true,"settings_queries_zero_downloads":true,"paused_retention_seconds":65,"online_lru_eviction":true,"open_file_protected":true,"pinned_file_protected":true,"soft_limit_exceeded_for_protected_data":true,"closed_file_reclaimed_next_pass":true,"remote_identity_version_hash_unchanged":true,"offline_missing_file_returns_error":true,"native_rehydration_after_eviction":true,"locked_restart_policy_and_cache_retained":true,"mint_tested":false}),
        )?,
    )?;
    println!(
        "PASS native cache budget, Qt settings, protected eviction, NAS preservation and restart"
    );
    Ok(())
}
