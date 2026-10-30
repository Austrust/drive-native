//! Real Qt resolution of a native mkdir collision, confined to the authorized lab.
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
    io::Read,
    os::unix::fs::{MetadataExt, PermissionsExt},
    time::{Duration, Instant},
};
#[path = "support/service.rs"]
mod support;
use support::*;
async fn verify(
    client: &Client,
    view: u64,
    path: &str,
    name: &str,
    expected: &[u8],
    scratch: &std::path::Path,
) -> Result<usize> {
    let list = client.list_dir(view, path).await?;
    let n = list
        .iter()
        .find(|n| n.name == name)
        .ok_or("NAS file absent")?;
    let spec = sproto::actions::transfer::ContentVersion {
        file_id: n.file_id.clone(),
        version: NativeVersion {
            sync_id: n.sync_id,
            max_id: n.max_id,
        },
        size: n.file_size,
        hash: n.hash.clone(),
    };
    let dest = scratch.join(uuid::Uuid::new_v4().to_string());
    let mut retries = 0;
    for attempt in 0..5 {
        match client.download_checked(&spec, &dest).await {
            Ok(_) => break,
            Err(sproto::Error::Server { code: 32772, .. }) if attempt < 4 => {
                retries += 1;
                tokio::time::sleep(Duration::from_secs(2 << attempt)).await;
            }
            Err(e) => return Err(e.into()),
        }
    }
    assert_eq!(std::fs::read(dest)?, expected);
    Ok(retries)
}
#[tokio::main]
async fn main() -> Result<()> {
    assert_ne!(unsafe { libc::geteuid() }, 0);
    let limit = libc::rlimit {
        rlim_cur: 0,
        rlim_max: 0,
    };
    if unsafe { libc::setrlimit(libc::RLIMIT_CORE, &limit) } != 0 {
        return Err("cannot disable core dumps".into());
    }
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 5 || args[1] != "/DriveNative-Lab/20261028T124720-1e31763a" {
        return Err("expected authorized lab, new result, desktop directory, screenshot".into());
    }
    use std::io::Write;
    std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&args[2])?
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
    let root = format!("{}/directory-conflict-{}", args[1], uuid::Uuid::new_v4());
    std::fs::write(
        &args[2],
        serde_json::json!({"passed":false,"scope":root}).to_string(),
    )?;
    println!("test_subtree={root}");
    let receipt = client.mkdir_native(home.view_id, &root).await?;
    let scratch = tempfile::tempdir()?;
    let mount = scratch.path().join("mount");
    std::fs::create_dir(&mount)?;
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
    let conn = zbus::Connection::session().await?;
    let proxy = zbus::Proxy::new(&conn, BUS_NAME, OBJECT_PATH, BUS_NAME).await?;
    let mut daemon = spawn(&executable, &config, &data, &mount, &log)?;
    wait(&proxy, "locked", |v| v["connection"] == "locked").await?;
    let _: () = proxy.call("Unlock", &(password.clone(),)).await?;
    wait(&proxy, "mounted online", |v| {
        v["connection"] == "online" && v["mounted"] == true
    })
    .await?;
    let _: () = proxy.call("Pause", &()).await?;
    std::fs::create_dir_all(mount.join("A/B"))?;
    std::fs::write(mount.join("A/B/note.txt"), b"my offline tree")?;
    std::fs::write(mount.join("A/empty.bin"), b"")?;
    let local_ino = std::fs::metadata(mount.join("A"))?.ino();
    let child_ino = std::fs::metadata(mount.join("A/B/note.txt"))?.ino();
    let mut held = std::fs::File::open(mount.join("A/B/note.txt"))?;
    let remote = client
        .mkdir_native(home.view_id, &format!("{root}/A"))
        .await?;
    let snapshot = scratch.path().join("remote-body");
    std::fs::write(&snapshot, b"other device tree")?;
    let remote_child = client
        .upload_snapshot(
            home.view_id,
            &format!("{root}/A/remote.txt"),
            NativeVersion {
                sync_id: 0,
                max_id: 0,
            },
            &snapshot,
        )
        .await?;
    let before = client.list_dir(home.view_id, &root).await?;
    let before = before
        .iter()
        .find(|n| n.name == "A")
        .ok_or("remote directory absent")?
        .clone();
    let _: () = proxy.call("Resume", &()).await?;
    let deadline = Instant::now() + Duration::from_secs(75);
    loop {
        let raw: String = proxy.call("GetIssues", &("",)).await?;
        let page: serde_json::Value = serde_json::from_str(&raw)?;
        if page["entries"][0]["can_keep_directory"] == true {
            assert_eq!(page["entries"][0]["reason"], "directory_creation_conflict");
            break;
        }
        if Instant::now() > deadline {
            println!("issue_state={page}");
            return Err("directory conflict did not become resolvable".into());
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    let _: () = proxy.call("Pause", &()).await?;
    let downloads = overview(&proxy).await?["downloads_started"].clone();
    let gui = std::process::Command::new("/opt/drive-native-build/gui-venv/bin/python")
        .arg(std::path::Path::new(&args[3]).join("tests/conflict_gui_live.py"))
        .arg(&args[4])
        .arg("directory_creation_conflict")
        .arg("A (local conflict)")
        .env("PYTHONPATH", &args[3])
        .env("XDG_CONFIG_HOME", scratch.path().join("gui-config"))
        .stdin(std::process::Stdio::null())
        .status()?;
    assert!(gui.success(), "Qt directory action failed");
    assert_eq!(overview(&proxy).await?["downloads_started"], downloads);
    assert_eq!(
        std::fs::metadata(mount.join("A (local conflict)"))?.ino(),
        local_ino
    );
    assert_eq!(
        std::fs::metadata(mount.join("A (local conflict)/B/note.txt"))?.ino(),
        child_ino
    );
    let mut held_bytes = Vec::new();
    held.read_to_end(&mut held_bytes)?;
    assert_eq!(held_bytes, b"my offline tree");
    drop(held);
    assert!(std::fs::read(mount.join("A/B/note.txt")).is_err());
    if let Ok(meta) = std::fs::metadata(mount.join("A")) {
        assert_ne!(meta.ino(), local_ino);
    }
    stop(&proxy, &mut daemon).await?;
    drop(daemon);
    let mut daemon = spawn(&executable, &config, &data, &mount, &log)?;
    wait(&proxy, "locked remount", |v| {
        v["connection"] == "locked" && v["mounted"] == true
    })
    .await?;
    assert_eq!(
        std::fs::read(mount.join("A (local conflict)/B/note.txt"))?,
        b"my offline tree"
    );
    assert_eq!(
        std::fs::read(mount.join("A (local conflict)/empty.bin"))?,
        b""
    );
    let _: () = proxy.call("Unlock", &(password.clone(),)).await?;
    wait_state(&proxy, "A (local conflict)/B/note.txt", "local_complete").await?;
    wait_state(&proxy, "A (local conflict)/empty.bin", "local_complete").await?;
    wait_state(&proxy, "A/remote.txt", "online_only").await?;
    assert_eq!(
        std::fs::read(mount.join("A/remote.txt"))?,
        b"other device tree"
    );
    let _: () = proxy.call("Pause", &()).await?;
    assert_eq!(
        std::fs::read(mount.join("A/remote.txt"))?,
        b"other device tree"
    );
    assert_eq!(
        std::fs::read(mount.join("A (local conflict)/B/note.txt"))?,
        b"my offline tree"
    );
    let listed = client.list_dir(home.view_id, &root).await?;
    assert_eq!(listed.len(), 2);
    let original = listed
        .iter()
        .find(|n| n.name == "A")
        .ok_or("original directory absent")?;
    assert_eq!(original.file_id, remote.file_id);
    assert_eq!(original.sync_id, before.sync_id);
    assert_eq!(original.max_id, before.max_id);
    let copy = listed
        .iter()
        .find(|n| n.name == "A (local conflict)")
        .ok_or("new directory absent")?;
    assert_ne!(copy.file_id, original.file_id);
    let original_children = client.list_dir(home.view_id, &format!("{root}/A")).await?;
    assert_eq!(original_children.len(), 1);
    assert_eq!(original_children[0].file_id, remote_child.file_id);
    assert_eq!(original_children[0].sync_id, remote_child.version.sync_id);
    assert_eq!(original_children[0].max_id, remote_child.version.max_id);
    let mut retries = 0;
    retries += verify(
        &client,
        home.view_id,
        &format!("{root}/A"),
        "remote.txt",
        b"other device tree",
        scratch.path(),
    )
    .await?;
    retries += verify(
        &client,
        home.view_id,
        &format!("{root}/A (local conflict)/B"),
        "note.txt",
        b"my offline tree",
        scratch.path(),
    )
    .await?;
    retries += verify(
        &client,
        home.view_id,
        &format!("{root}/A (local conflict)"),
        "empty.bin",
        b"",
        scratch.path(),
    )
    .await?;
    stop(&proxy, &mut daemon).await?;
    assert!(!std::fs::read_to_string(log)?.contains(&password));
    std::fs::write(
        &args[2],
        serde_json::to_vec_pretty(
            &serde_json::json!({"passed":true,"scope":root,"native_directory_collision":true,"actual_qt_keep_directory_both":true,"old_path_exposes_remote_identity":true,"local_tree_and_child_inodes_preserved":true,"open_child_handle_survives_resolution":true,"resolution_zero_downloads":true,"locked_restart_preserves_tree":true,"nested_mkdir_then_children_upload":true,"empty_file_uploaded":true,"remote_directory_and_child_identity_version_unchanged":true,"independent_native_content_verified":true,"offline_both_trees_readable":true,"independent_download_retries":retries,"mint_tested":false}),
        )?,
    )?;
    println!(
        "PASS native directory collision, actual Qt action, stable inodes/open handle, locked restart, nested uploads and both NAS trees"
    );
    Ok(())
}
