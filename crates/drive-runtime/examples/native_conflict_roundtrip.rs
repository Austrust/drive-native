//! Real native conflict -> Qt action -> local restart -> independent NAS verification.
use drive_runtime::{
    desktop::{BUS_NAME, OBJECT_PATH},
    service::Profile,
};
use sproto::{
    Client,
    actions::native::NativeVersion,
    client::{Config, Credentials},
};
use std::{os::unix::fs::PermissionsExt, time::Duration};
#[path = "support/service.rs"]
mod support;
use support::*;
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
    let creation = args.len() == 6 && args[5] == "--creation";
    if (!creation && args.len() != 5) || args[1] != "/DriveNative-Lab/20261028T124720-1e31763a" {
        return Err("expected authorized lab, new result, desktop directory, screenshot".into());
    }
    let output = &args[2];
    use std::io::Write;
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
    let root = format!("{}/conflict-gui-{}", args[1], uuid::Uuid::new_v4());
    std::fs::write(
        output,
        serde_json::json!({"passed":false,"scope":root}).to_string(),
    )?;
    println!("test_subtree={root}");
    let receipt = client.mkdir_native(home.view_id, &root).await?;
    let scratch = tempfile::tempdir()?;
    let snapshot = scratch.path().join("snapshot");
    let mount = scratch.path().join("mount");
    std::fs::create_dir(&mount)?;
    std::fs::write(&snapshot, b"initial")?;
    let first = if creation {
        None
    } else {
        Some(
            client
                .upload_snapshot(
                    home.view_id,
                    &format!("{root}/A.doc"),
                    NativeVersion {
                        sync_id: 0,
                        max_id: 0,
                    },
                    &snapshot,
                )
                .await?,
        )
    };
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
    wait(&proxy, "online", |v| {
        v["connection"] == "online" && v["mounted"] == true
    })
    .await?;
    if !creation {
        assert_eq!(std::fs::read(mount.join("A.doc"))?, b"initial");
    }
    let _: () = proxy.call("Pause", &()).await?;
    std::fs::write(mount.join("A.doc"), b"my offline edit")?;
    // Populate the old dentry and read cache before the GUI retires this inode.
    assert_eq!(std::fs::read(mount.join("A.doc"))?, b"my offline edit");
    use std::os::unix::fs::MetadataExt;
    let local_inode = std::fs::metadata(mount.join("A.doc"))?.ino();
    std::fs::write(&snapshot, b"other device edit")?;
    let remote = client
        .upload_snapshot(
            home.view_id,
            &format!("{root}/A.doc"),
            first.as_ref().map_or(
                NativeVersion {
                    sync_id: 0,
                    max_id: 0,
                },
                |n| n.version,
            ),
            &snapshot,
        )
        .await?;
    let _: () = proxy.call("Resume", &()).await?;
    wait_state(&proxy, "A.doc", "conflict").await?;
    let deadline = std::time::Instant::now() + Duration::from_secs(75);
    loop {
        let raw: String = proxy.call("GetIssues", &("",)).await?;
        let page: serde_json::Value = serde_json::from_str(&raw)?;
        if page["entries"][0]["can_keep_both"] == true {
            break;
        }
        if std::time::Instant::now() > deadline {
            println!("conflict_decision_state={page}");
            return Err("conflict never became safe to resolve".into());
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    let _: () = proxy.call("Pause", &()).await?;
    let before = overview(&proxy).await?["downloads_started"].clone();
    let issues: String = proxy.call("GetIssues", &("",)).await?;
    let issues: serde_json::Value = serde_json::from_str(&issues)?;
    assert_eq!(issues["entries"].as_array().unwrap().len(), 1);
    assert_eq!(issues["entries"][0]["can_keep_both"], true);
    let reason = if creation {
        "creation_conflict"
    } else {
        "content_conflict"
    };
    assert_eq!(issues["entries"][0]["reason"], reason);
    assert_eq!(overview(&proxy).await?["downloads_started"], before);
    let gui = std::process::Command::new("/opt/drive-native-build/gui-venv/bin/python")
        .arg(std::path::Path::new(&args[3]).join("tests/conflict_gui_live.py"))
        .arg(&args[4])
        .arg(reason)
        .env("PYTHONPATH", &args[3])
        .env("XDG_CONFIG_HOME", scratch.path().join("gui-config"))
        .stdin(std::process::Stdio::null())
        .status()?;
    assert!(gui.success(), "real Qt conflict action failed");
    std::fs::write(
        output,
        serde_json::json!({"passed":false,"scope":root,"stage":"GUI resolved; restarting"})
            .to_string(),
    )?;
    assert_eq!(
        std::fs::read(mount.join("A (local conflict).doc"))?,
        b"my offline edit"
    );
    if creation {
        // The peer may not yet be indexed, but the retired local inode/content
        // must never remain accessible through the old name.
        if let Ok(meta) = std::fs::metadata(mount.join("A.doc")) {
            assert_ne!(meta.ino(), local_inode);
        }
        assert!(
            std::fs::read(mount.join("A.doc")).is_err(),
            "old local cache leaked through retired dentry"
        );
    }
    stop(&proxy, &mut daemon).await?;
    drop(daemon);
    let mut daemon = spawn(&executable, &config, &data, &mount, &log)?;
    wait(&proxy, "locked restart", |v| {
        v["connection"] == "locked" && v["mounted"] == true
    })
    .await?;
    assert_eq!(
        std::fs::read(mount.join("A (local conflict).doc"))?,
        b"my offline edit"
    );
    let _: () = proxy.call("Unlock", &(password.clone(),)).await?;
    wait_state(&proxy, "A (local conflict).doc", "local_complete").await?;
    if creation {
        // Never-cached remote content is still online-only until the first read.
        wait_state(&proxy, "A.doc", "online_only").await?;
        assert_eq!(std::fs::read(mount.join("A.doc"))?, b"other device edit");
    }
    wait_state(&proxy, "A.doc", "local_complete").await?;
    let _: () = proxy.call("Pause", &()).await?;
    assert_eq!(std::fs::read(mount.join("A.doc"))?, b"other device edit");
    assert_eq!(
        std::fs::read(mount.join("A (local conflict).doc"))?,
        b"my offline edit"
    );
    std::fs::write(output,serde_json::json!({"passed":false,"scope":root,"stage":"locked restart and offline content passed; checking NAS"}).to_string())?;
    let listed = client.list_dir(home.view_id, &root).await?;
    assert_eq!(listed.len(), 2);
    let original = listed
        .iter()
        .find(|n| n.name == "A.doc")
        .ok_or("original absent")?;
    assert_eq!(original.file_id, remote.file_id);
    assert_eq!(original.sync_id, remote.version.sync_id);
    assert_eq!(original.max_id, remote.version.max_id);
    assert_ne!(
        listed
            .iter()
            .find(|n| n.name == "A (local conflict).doc")
            .unwrap()
            .file_id,
        original.file_id
    );
    let mut verify_retries = 0;
    for (name, expected) in [
        ("A.doc", &b"other device edit"[..]),
        ("A (local conflict).doc", &b"my offline edit"[..]),
    ] {
        let n = listed
            .iter()
            .find(|n| n.name == name)
            .ok_or("copy absent")?;
        let verify = scratch
            .path()
            .join(format!("verify-{}", uuid::Uuid::new_v4()));
        let spec = sproto::actions::transfer::ContentVersion {
            file_id: n.file_id.clone(),
            version: NativeVersion {
                sync_id: n.sync_id,
                max_id: n.max_id,
            },
            size: n.file_size,
            hash: n.hash.clone(),
        };
        for attempt in 0..5 {
            match client.download_checked(&spec, &verify).await {
                Ok(_) => break,
                Err(sproto::Error::Server { code: 32772, .. }) if attempt < 4 => {
                    verify_retries += 1;
                    tokio::time::sleep(Duration::from_secs(2 << attempt)).await;
                }
                Err(error) => return Err(error.into()),
            }
        }
        assert_eq!(std::fs::read(verify)?, expected);
    }
    stop(&proxy, &mut daemon).await?;
    assert!(!std::fs::read_to_string(log)?.contains(&password));
    std::fs::write(
        output,
        serde_json::to_vec_pretty(
            &serde_json::json!({"passed":true,"scope":root,"native_two_sided_conflict":true,"conflict_kind":reason,"creation_mode":creation,"retired_dentry_no_old_content":creation,"qt_button_dbus_resolution":true,"issue_listing_zero_downloads":true,"local_resolution_survives_locked_restart":true,"copy_upload_after_restart":true,"original_remote_identity_and_version_unchanged":true,"independent_remote_content_verified":true,"independent_download_retries":verify_retries,"offline_both_versions_readable":true,"mint_tested":false}),
        )?,
    )?;
    println!(
        "PASS native conflict, real Qt KeepBoth, locked restart, both NAS versions and offline files"
    );
    tokio::time::sleep(Duration::from_millis(50)).await;
    Ok(())
}
