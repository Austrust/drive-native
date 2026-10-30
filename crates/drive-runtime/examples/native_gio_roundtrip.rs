//! Real GIO -> ordinary-user FUSE -> native NAS write and offline queue checks.
#[path = "support/service.rs"]
mod support;
use drive_runtime::{
    desktop::{BUS_NAME, OBJECT_PATH},
    service::Profile,
};
use sproto::{
    Client,
    actions::{native::NativeVersion, transfer::ContentVersion},
    client::{Config, Credentials},
};
use std::{os::unix::fs::PermissionsExt, path::Path, process::Command, time::Duration};
use support::*;

fn gio(helper: &Path, modules: &Path, mount: &Path, action: &str) -> Result<()> {
    let output = Command::new("/usr/bin/timeout")
        .args(["--signal=TERM", "--kill-after=5", "60", "/usr/bin/python3"])
        .arg(helper)
        .arg(mount)
        .arg(action)
        .env_remove("GIO_USE_VFS")
        .env("GIO_EXTRA_MODULES", modules)
        .env_remove("DRIVE_NATIVE_METADATA_ROOT")
        .output()?;
    print!("{}", String::from_utf8_lossy(&output.stdout));
    if !output.status.success() {
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        return Err(format!("GIO stage failed: {action}").into());
    }
    Ok(())
}
async fn verify(
    client: &Client,
    view: u64,
    dir: &str,
    name: &str,
    identity: Option<&str>,
    body: &[u8],
    scratch: &Path,
) -> Result<ContentVersion> {
    let nodes = client.list_dir(view, dir).await?;
    let node = nodes
        .into_iter()
        .find(|n| n.name == name)
        .ok_or("remote file missing")?;
    if let Some(id) = identity {
        assert_eq!(node.file_id, id);
    }
    let expected = ContentVersion {
        file_id: node.file_id,
        version: NativeVersion {
            sync_id: node.sync_id,
            max_id: node.max_id,
        },
        size: node.file_size,
        hash: node.hash,
    };
    let file = scratch.join(format!("verify-{}", uuid::Uuid::new_v4()));
    for attempt in 0..5 {
        match client.download_checked(&expected, &file).await {
            Ok(_) => {
                assert_eq!(std::fs::read(&file)?, body);
                return Ok(expected);
            }
            Err(sproto::Error::Server { code: 32772, .. }) if attempt < 4 => {
                tokio::time::sleep(Duration::from_secs(2 << attempt)).await;
            }
            Err(error) => return Err(error.into()),
        }
    }
    unreachable!()
}
#[tokio::main]
async fn main() -> Result<()> {
    assert_ne!(unsafe { libc::geteuid() }, 0);
    unsafe {
        let limit = libc::rlimit {
            rlim_cur: 0,
            rlim_max: 0,
        };
        assert_eq!(libc::setrlimit(libc::RLIMIT_CORE, &limit), 0);
    }
    let args: Vec<_> = std::env::args().collect();
    let lab = args.get(1).ok_or("expected authorized laboratory")?;
    let parts: Vec<_> = lab.split('/').collect();
    if parts.len() != 3
        || parts[0] != ""
        || parts[1] != "DriveNative-Lab"
        || parts[2].is_empty()
        || parts[2].contains("..")
        || lab.contains('\0')
    {
        return Err("invalid laboratory scope".into());
    }
    let report_path = Path::new(args.get(2).ok_or("expected report path")?);
    let helper = Path::new(args.get(3).ok_or("expected GIO helper")?).canonicalize()?;
    let modules = Path::new(args.get(4).ok_or("expected module directory")?).canonicalize()?;
    assert!(modules.join("libdrive-native-probe.so").is_file());
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
    let root = format!("{lab}/gio-{}", uuid::Uuid::new_v4());
    let receipt = client.mkdir_native(home.view_id, &root).await?;
    let scratch = tempfile::tempdir()?.keep();
    println!("test_state={}", scratch.display());
    println!("test_subtree={root}");
    let mount = scratch.join("mount");
    std::fs::create_dir(&mount)?;
    let snapshot = scratch.join("snapshot");
    std::fs::write(&snapshot, b"initial-v1")?;
    let seed = client
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
    let config = scratch.join("profile.json");
    std::fs::write(&config, serde_json::to_vec_pretty(&profile)?)?;
    std::fs::set_permissions(&config, std::fs::Permissions::from_mode(0o600))?;
    let data = scratch.join("data");
    let log = scratch.join("fsd.log");
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
    wait(&proxy, "online", |v| {
        v["connection"] == "online" && v["mounted"] == true
    })
    .await?;
    assert_eq!(overview(&proxy).await?["downloads_started"], 0);
    gio(&helper, &modules, &mount, "read")?;
    wait_state(&proxy, "A.doc", "local_complete").await?;
    gio(&helper, &modules, &mount, "save")?;
    wait(&proxy, "saved queue drained", |v| {
        v["pending_operations"] == 0
    })
    .await?;
    wait_state(&proxy, "A.doc", "local_complete").await?;
    verify(
        &client,
        home.view_id,
        &root,
        "A.doc",
        Some(&seed.file_id),
        b"online-v2",
        &scratch,
    )
    .await?;
    println!("PASS GIO asynchronous replace and cancellation; native commit verified");
    gio(&helper, &modules, &mount, "rename")?;
    wait(&proxy, "renamed queue drained", |v| {
        v["pending_operations"] == 0
    })
    .await?;
    wait_state(&proxy, "Renamed.doc", "local_complete").await?;
    let before = verify(
        &client,
        home.view_id,
        &root,
        "Renamed.doc",
        Some(&seed.file_id),
        b"online-v2",
        &scratch,
    )
    .await?;
    assert!(
        !client
            .list_dir(home.view_id, &root)
            .await?
            .iter()
            .any(|n| n.name == "A.doc")
    );
    let _: () = proxy.call("Pause", &()).await?;
    gio(&helper, &modules, &mount, "offline")?;
    wait_state(&proxy, "Renamed.doc", "pending_upload").await?;
    assert!(
        overview(&proxy).await?["pending_operations"]
            .as_u64()
            .unwrap()
            > 0
    );
    let nodes = client.list_dir(home.view_id, &root).await?;
    assert!(!nodes.iter().any(|n| n.name == "Offline"));
    let old = nodes
        .iter()
        .find(|n| n.name == "Renamed.doc")
        .ok_or("remote original missing")?;
    assert_eq!(old.sync_id, before.version.sync_id);
    assert_eq!(old.hash, before.hash);
    let _: () = proxy.call("Resume", &()).await?;
    wait(&proxy, "resumed", |v| v["connection"] == "online").await?;
    wait(&proxy, "offline queue drained", |v| {
        v["pending_operations"] == 0
    })
    .await?;
    wait_state(&proxy, "Renamed.doc", "local_complete").await?;
    wait_state(&proxy, "Offline/Created.txt", "local_complete").await?;
    verify(
        &client,
        home.view_id,
        &root,
        "Renamed.doc",
        Some(&seed.file_id),
        b"offline-v3",
        &scratch,
    )
    .await?;
    verify(
        &client,
        home.view_id,
        &format!("{root}/Offline"),
        "Created.txt",
        None,
        b"offline-created",
        &scratch,
    )
    .await?;
    println!(
        "PASS offline GIO save, directory and async create; native replay and desktop states verified"
    );
    gio(&helper, &modules, &mount, "copy")?;
    wait(&proxy, "copy drained", |v| v["pending_operations"] == 0).await?;
    wait_state(&proxy, "Copied.doc", "local_complete").await?;
    let copy = verify(
        &client,
        home.view_id,
        &root,
        "Copied.doc",
        None,
        b"offline-v3",
        &scratch,
    )
    .await?;
    assert_ne!(copy.file_id, seed.file_id);
    gio(&helper, &modules, &mount, "delete")?;
    wait(&proxy, "delete drained", |v| v["pending_operations"] == 0).await?;
    let nodes = client.list_dir(home.view_id, &root).await?;
    assert_eq!(nodes.len(), 1);
    assert_eq!(nodes[0].name, "Renamed.doc");
    assert_eq!(nodes[0].file_id, seed.file_id);
    stop(&proxy, &mut daemon).await?;
    assert!(!std::fs::read_to_string(&log)?.contains(&password));
    let report = serde_json::json!({"result":"passed","subtree":root,"ordinary_uid":unsafe{libc::geteuid()},
        "native_server":true,"gio_adapter":true,"automatic_gio_selection":true,"separate_fsd_process":true,
        "async_save_remote_content_verified":true,"cancelled_save_preserves_body":true,
        "rename_remote_identity_preserved":true,"offline_save_pending_status":true,
        "offline_directory_async_creation_replayed":true,"remote_content_independently_downloaded":true,
        "async_copy_new_remote_identity":true,"delete_file_and_directory_confirmed":true,
        "full_gvfs_or_mint_acceptance":false});
    std::fs::write(report_path, serde_json::to_vec_pretty(&report)?)?;
    println!("PASS GIO copy and delete native commits; daemon stopped cleanly");
    Ok(())
}
