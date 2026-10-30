//! Explicit native/FUSE cancellation in a new authorized UUID subtree.
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
    io::Write,
    os::unix::fs::PermissionsExt,
    path::Path,
    process::{Child, Command, Stdio},
    time::Duration,
};
#[path = "support/service.rs"]
mod support;
use support::*;
struct Reader(Child);
impl Reader {
    fn start(path: &Path, out: &Path) -> Result<Self> {
        Ok(Self(
            Command::new("cat")
                .arg(path)
                .stdin(Stdio::null())
                .stdout(Stdio::from(std::fs::File::create(out)?))
                .stderr(Stdio::null())
                .spawn()?,
        ))
    }
    async fn interrupt(&mut self) -> Result<()> {
        if self.0.try_wait()?.is_some() {
            return Err("reader completed before interruption".into());
        }
        if unsafe { libc::kill(self.0.id() as i32, libc::SIGTERM) } != 0 {
            return Err(std::io::Error::last_os_error().into());
        }
        let start = std::time::Instant::now();
        loop {
            if let Some(status) = self.0.try_wait()? {
                assert!(!status.success());
                return Ok(());
            }
            if start.elapsed() > Duration::from_secs(2) {
                return Err("interrupted reader did not exit promptly".into());
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }
    async fn finish(&mut self) -> Result<()> {
        let start = std::time::Instant::now();
        loop {
            if let Some(status) = self.0.try_wait()? {
                if !status.success() {
                    return Err("remaining reader failed".into());
                }
                return Ok(());
            }
            if start.elapsed() > Duration::from_secs(60) {
                return Err("remaining reader timed out".into());
            }
            tokio::time::sleep(Duration::from_millis(40)).await;
        }
    }
}
impl Drop for Reader {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}
async fn transfers(p: &zbus::Proxy<'_>) -> Result<serde_json::Value> {
    let raw: String = p.call("GetTransfers", &()).await?;
    Ok(serde_json::from_str(&raw)?)
}
async fn partial(p: &zbus::Proxy<'_>, name: &str) -> Result<serde_json::Value> {
    let start = std::time::Instant::now();
    loop {
        let value = transfers(p).await?;
        if value["entries"].as_array().unwrap().iter().any(|e| {
            e["path"] == name
                && e["progress"]["bytes"]
                    .as_u64()
                    .is_some_and(|n| n >= 65536 && n < e["progress"]["total"].as_u64().unwrap_or(0))
        }) {
            return Ok(value);
        }
        if start.elapsed() > Duration::from_secs(30) {
            return Err(format!("no partial native progress for {name}: {value}").into());
        }
        tokio::time::sleep(Duration::from_millis(40)).await;
    }
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
    if args.len() != 3 || args[1] != "/DriveNative-Lab/20261028T124720-1e31763a" {
        return Err("expected authorized lab and new result".into());
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
    let root = format!("{}/cancel-{}", args[1], uuid::Uuid::new_v4());
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
    for (i, name) in ["Cancel.bin", "Shared.bin", "Pinned.bin"]
        .iter()
        .enumerate()
    {
        std::fs::write(&snapshot, vec![65 + i as u8; 512 * 1024])?;
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
    let raw: String = proxy.call("GetTransferSettings", &()).await?;
    let policy: serde_json::Value = serde_json::from_str(&raw)?;
    let raw=serde_json::json!({"schema":1,"task_id":profile.task_id,"expected_version":policy["version"],"limits":{"download_bps":65536,"upload_bps":0}}).to_string();
    let _: String = proxy.call("SetTransferSettings", &(raw,)).await?;
    let mut evidence = Vec::new();
    let initial = overview(&proxy).await?;
    let mut abandoned =
        Reader::start(&mount.join("Cancel.bin"), &scratch.path().join("cancelled"))?;
    evidence.push(partial(&proxy, "Cancel.bin").await?);
    let start = std::time::Instant::now();
    abandoned.interrupt().await?;
    let cancelled_ms = start.elapsed().as_millis();
    let deadline = std::time::Instant::now() + Duration::from_secs(3);
    loop {
        let entries = transfers(&proxy).await?;
        if entries["entries"].as_array().unwrap().is_empty() {
            break;
        }
        if std::time::Instant::now() > deadline {
            std::fs::write(
                output,
                serde_json::to_vec_pretty(
                    &serde_json::json!({"passed":false,"scope":root,"stage":"abandoned transfer did not stop","transfers":entries,"state":states(&proxy,"Cancel.bin").await?,"overview":overview(&proxy).await?,"reader_exit_ms":cancelled_ms}),
                )?,
            )?;
            return Err("abandoned transfer did not stop".into());
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
    wait_state(&proxy, "Cancel.bin", "online_only").await?;
    assert_eq!(
        overview(&proxy).await?["download_diagnostics"]["failures_seen"],
        initial["download_diagnostics"]["failures_seen"]
    );
    assert_eq!(
        std::fs::metadata(scratch.path().join("cancelled"))?.len(),
        0
    );

    // A subsequent operation must use a clean native channel after abort.
    let mut reopened = Reader::start(&mount.join("Cancel.bin"), &scratch.path().join("reopened"))?;
    reopened.finish().await?;
    assert_eq!(
        std::fs::read(scratch.path().join("reopened"))?,
        vec![65; 512 * 1024]
    );
    let before_shared = overview(&proxy).await?["downloads_started"]
        .as_u64()
        .unwrap();
    let mut first = Reader::start(&mount.join("Shared.bin"), &scratch.path().join("first"))?;
    let mut second = Reader::start(&mount.join("Shared.bin"), &scratch.path().join("second"))?;
    evidence.push(partial(&proxy, "Shared.bin").await?);
    first.interrupt().await?;
    second.finish().await?;
    assert_eq!(
        std::fs::read(scratch.path().join("second"))?,
        vec![66; 512 * 1024]
    );
    assert_eq!(
        overview(&proxy).await?["downloads_started"]
            .as_u64()
            .unwrap()
            - before_shared,
        1
    );

    let _: () = proxy.call("SetPinned", &("Pinned.bin", true)).await?;
    evidence.push(partial(&proxy, "Pinned.bin").await?);
    let mut reader = Reader::start(
        &mount.join("Pinned.bin"),
        &scratch.path().join("pinned-reader"),
    )?;
    tokio::time::sleep(Duration::from_millis(250)).await;
    reader.interrupt().await?;
    wait_state(&proxy, "Pinned.bin", "pinned_complete").await?;
    // Native metadata notification still works after cancelling a data channel.
    client
        .mkdir_native(home.view_id, &format!("{root}/AfterCancel"))
        .await?;
    let deadline = std::time::Instant::now() + Duration::from_secs(10);
    while !mount.join("AfterCancel").is_dir() {
        if std::time::Instant::now() > deadline {
            return Err("native events stalled after cancellation".into());
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    let _: () = proxy.call("Pause", &()).await?;
    assert_eq!(
        std::fs::read(mount.join("Pinned.bin"))?,
        vec![67; 512 * 1024]
    );
    assert_eq!(
        std::fs::read(mount.join("Shared.bin"))?,
        vec![66; 512 * 1024]
    );
    let final_overview = overview(&proxy).await?;
    stop(&proxy, &mut daemon).await?;
    assert!(!std::fs::read_to_string(&log)?.contains(&password));
    std::fs::write(
        output,
        serde_json::to_vec_pretty(
            &serde_json::json!({"passed":true,"scope":root,"actual_kernel_signal":true,"abandoned_reader_exit_ms":cancelled_ms,"last_reader_transfer_removed":true,"no_failure_or_empty_cache_after_cancel":true,"later_download_clean_native_channel":true,"shared_readers_one_download":true,"pinned_download_survived_reader_cancel":true,"native_events_after_cancel":true,"offline_contents_verified":true,"progress_samples":evidence,"overview":final_overview,"windows_runtime_compared":false}),
        )?,
    )?;
    println!("PASS real native/FUSE cancellation, shared readers, pin and event survival");
    Ok(())
}
