//! Real native NAS through a temporary loopback private-CA TLS boundary.
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
    os::unix::fs::{MetadataExt, PermissionsExt},
    sync::atomic::Ordering,
};
#[path = "support/service.rs"]
mod support;
use support::*;
const CA: &str = include_str!("../../sproto/tests/fixtures/trust/ca.pem");
const CERT: &str = include_str!("../../sproto/tests/fixtures/trust/server.pem");
const KEY: &str = include_str!("../../sproto/tests/fixtures/trust/server-key.pem");
fn gui(
    desktop: &str,
    mode: &str,
    cert: &std::path::Path,
    screenshot: &str,
    config: &std::path::Path,
) -> Result<()> {
    let status = std::process::Command::new("/opt/drive-native-build/gui-venv/bin/python")
        .arg(std::path::Path::new(desktop).join("tests/trust_gui_live.py"))
        .arg(mode)
        .arg(cert)
        .arg(screenshot)
        .env("PYTHONPATH", desktop)
        .env("XDG_CONFIG_HOME", config)
        .stdin(std::process::Stdio::null())
        .status()?;
    if !status.success() {
        return Err("real Qt trust action failed".into());
    }
    Ok(())
}
#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() -> Result<()> {
    assert_ne!(unsafe { libc::geteuid() }, 0);
    drive_runtime::disable_core_dumps()?;
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 5 || args[1] != "/DriveNative-Lab/20261028T124720-1e31763a" {
        return Err(
            "expected authorized lab, new result, desktop directory, screenshot prefix".into(),
        );
    }
    std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&args[2])?
        .write_all(b"{\"passed\":false}")?;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:6690").await?; // fail if occupied; never replace a listener
    let relay = sproto::trust_relay::start(listener, CERT, KEY).await?;
    let negative = sproto::Client::connect(
        Config::builder()
            .host("127.0.0.1")
            .tls_server_name("localhost".to_owned())
            .credentials(Credentials::Password {
                username: "synthetic-unused".into(),
                password: "synthetic-unused".into(),
                otp: None,
            })
            .build(),
    )
    .await;
    if let Err(ref error) = negative {
        eprintln!("pre-authentication test result: {error}");
    }
    assert!(
        matches!(negative,Err(sproto::Error::Io(ref e)) if e.kind()==std::io::ErrorKind::InvalidData)
            || matches!(negative, Err(sproto::Error::Tls(_)))
    );
    assert_eq!(relay.accepted.load(Ordering::Relaxed), 0);
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
    let root = format!("{}/trust-{}", args[1], uuid::Uuid::new_v4());
    println!("test_subtree={root}");
    std::fs::write(
        &args[2],
        serde_json::json!({"passed":false,"scope":root}).to_string(),
    )?;
    let receipt = client.mkdir_native(home.view_id, &root).await?;
    let scratch = tempfile::tempdir()?;
    let cert = scratch.path().join("test-ca.pem");
    std::fs::write(&cert, CA)?;
    let snapshot = scratch.path().join("body");
    std::fs::write(&snapshot, b"cached through verified private TLS")?;
    let file = client
        .upload_snapshot(
            home.view_id,
            &format!("{root}/A.doc"),
            NativeVersion {
                sync_id: 0,
                max_id: 0,
            },
            &snapshot,
        )
        .await?;
    let mount = scratch.path().join("mount");
    std::fs::create_dir(&mount)?;
    let profile = Profile {
        schema: 1,
        task_id: uuid::Uuid::new_v4().to_string(),
        host: "127.0.0.1".into(),
        tls_name: "localhost".into(),
        trusted_ca_pem: vec![],
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
    let gui_config = scratch.path().join("gui-config");
    let bin = std::env::current_exe()?
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_owned();
    let conn = zbus::Connection::session().await?;
    let proxy = zbus::Proxy::new(&conn, BUS_NAME, OBJECT_PATH, BUS_NAME).await?;
    let mut daemon = spawn(&bin.join("drive-native-fsd"), &config, &data, &mount, &log)?;
    wait(&proxy, "locked", |v| v["connection"] == "locked").await?;
    let _: () = proxy.call("Unlock", &(password.clone(),)).await?;
    wait(&proxy, "untrusted private CA rejected", |v| {
        v["connection"] == "needs_attention" && v["error"] == "certificate_or_configuration"
    })
    .await?;
    assert_eq!(relay.accepted.load(Ordering::Relaxed), 0);
    gui(
        &args[3],
        "add",
        &cert,
        &format!("{}-add.png", args[4]),
        &gui_config,
    )?;
    wait(&proxy, "private CA native connection", |v| {
        v["connection"] == "online" && v["mounted"] == true
    })
    .await?;
    assert_eq!(
        std::fs::read(mount.join("A.doc"))?,
        b"cached through verified private TLS"
    );
    let inode = std::fs::metadata(mount.join("A.doc"))?.ino();
    let pid = daemon.child.id();
    gui(
        &args[3],
        "remove",
        &cert,
        &format!("{}-remove.png", args[4]),
        &gui_config,
    )?;
    wait(&proxy, "removing CA rejects next connection", |v| {
        v["connection"] == "needs_attention"
    })
    .await?;
    assert_eq!(daemon.child.id(), pid);
    assert_eq!(overview(&proxy).await?["mounted"], true);
    assert_eq!(std::fs::metadata(mount.join("A.doc"))?.ino(), inode);
    assert_eq!(
        std::fs::read(mount.join("A.doc"))?,
        b"cached through verified private TLS"
    );
    gui(
        &args[3],
        "add",
        &cert,
        &format!("{}-restore.png", args[4]),
        &gui_config,
    )?;
    wait(&proxy, "CA restoration reconnects", |v| {
        v["connection"] == "online"
    })
    .await?;
    // Exercise the same private-stdin login helper used by the setup wizard.
    let mut helper = std::process::Command::new(bin.join("drive-native-setup"))
        .arg("--private-stdin")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()?;
    {
        let mut input = helper.stdin.take().unwrap();
        let login = serde_json::json!({"command":"login","host":"127.0.0.1","tls_name":"localhost","trusted_ca_pem":[CA],"username":"test-user","password":password,"device_id":uuid::Uuid::new_v4().to_string()});
        serde_json::to_writer(&mut input, &login)?;
        input.write_all(b"\n{\"command\":\"quit\"}\n")?;
    }
    let result = helper.wait_with_output()?;
    assert!(result.status.success());
    let output = String::from_utf8(result.stdout)?;
    assert!(!output.contains(&password));
    let response: serde_json::Value =
        serde_json::from_str(output.lines().next().ok_or("setup response absent")?)?;
    assert_eq!(response["ok"], true);
    stop(&proxy, &mut daemon).await?;
    drop(daemon);
    let mut daemon = spawn(&bin.join("drive-native-fsd"), &config, &data, &mount, &log)?;
    wait(&proxy, "locked cached restart", |v| {
        v["connection"] == "locked" && v["mounted"] == true
    })
    .await?;
    let settings: String = proxy.call("GetTrustSettings", &()).await?;
    let settings: serde_json::Value = serde_json::from_str(&settings)?;
    assert_eq!(settings["settings"]["trusted_ca_pem"][0], CA);
    assert_eq!(
        std::fs::read(mount.join("A.doc"))?,
        b"cached through verified private TLS"
    );
    let _: () = proxy.call("Unlock", &(password.clone(),)).await?;
    wait(&proxy, "persisted private CA connection", |v| {
        v["connection"] == "online"
    })
    .await?;
    let _: () = proxy.call("Pause", &()).await?;
    assert_eq!(
        std::fs::read(mount.join("A.doc"))?,
        b"cached through verified private TLS"
    );
    let listed = client.list_dir(home.view_id, &root).await?;
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].file_id, file.file_id);
    assert_eq!(listed[0].sync_id, file.version.sync_id);
    assert_eq!(listed[0].max_id, file.version.max_id);
    stop(&proxy, &mut daemon).await?;
    assert!(!std::fs::read_to_string(log)?.contains(&password));
    std::fs::write(
        &args[2],
        serde_json::to_vec_pretty(
            &serde_json::json!({"passed":true,"scope":root,"test_route":"loopback private-CA TLS -> verified NAS TLS -> real native protocol","untrusted_chain_rejected_before_native_auth":true,"actual_qt_import_remove_save":true,"extra_ca_reaches_native_worker":true,"removal_rejects_reconnection":true,"mounted_cache_and_inode_survive_trust_change":true,"restoration_reconnects_without_unmount":true,"private_stdin_setup_helper_with_private_ca":true,"trust_persists_locked_restart":true,"native_reconnect_uses_persisted_ca":true,"nas_file_identity_version_unchanged":true,"verified_relay_connections":relay.accepted.load(Ordering::Relaxed),"nas_certificate_configuration_changed":false,"mint_tested":false}),
        )?,
    )?;
    println!(
        "PASS private native TLS trust rejection/import/removal/restoration, GUI, setup helper, mounted cache and locked restart"
    );
    Ok(())
}
