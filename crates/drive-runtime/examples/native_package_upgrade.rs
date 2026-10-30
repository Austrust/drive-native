//! Installed-package upgrade/removal/reinstall with real pending data. Explicit NAS test only.
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
    time::{Duration, Instant},
};
#[path = "support/service.rs"]
mod support;
use support::*;
async fn gate(base: &Path, name: &str) -> Result<()> {
    std::fs::write(base.join(format!("{name}.ready")), b"ready")?;
    println!("package gate ready: {name}");
    let start = Instant::now();
    while !base.join(format!("{name}.done")).exists() {
        if start.elapsed() > Duration::from_secs(900) {
            return Err(format!("package gate timed out: {name}").into());
        }
        tokio::time::sleep(Duration::from_millis(250)).await;
    }
    Ok(())
}
fn schema(path: &Path) -> Result<u64> {
    let out=std::process::Command::new("/usr/bin/python3").arg("-c").arg("import sqlite3,sys; c=sqlite3.connect('file:'+sys.argv[1]+'?mode=ro',uri=True); print(c.execute('PRAGMA user_version').fetchone()[0])").arg(path).output()?;
    if !out.status.success() {
        return Err("schema inspection failed".into());
    }
    Ok(std::str::from_utf8(&out.stdout)?.trim().parse()?)
}
fn save(path: &Path, body: &[u8], create: bool) -> Result<()> {
    let mut f = std::fs::OpenOptions::new()
        .write(true)
        .create(create)
        .truncate(true)
        .open(path)?;
    f.write_all(body)?;
    f.sync_all()?;
    Ok(())
}
async fn verify(
    client: &Client,
    view: u64,
    root: &str,
    name: &str,
    body: &[u8],
    base: &Path,
) -> Result<()> {
    let items = client.list_dir(view, root).await?;
    let n = items
        .iter()
        .find(|n| n.name == name)
        .ok_or("remote fixture absent")?;
    let expected = sproto::actions::transfer::ContentVersion {
        file_id: n.file_id.clone(),
        version: NativeVersion {
            sync_id: n.sync_id,
            max_id: n.max_id,
        },
        size: n.file_size,
        hash: n.hash.clone(),
    };
    let target = base.join(format!("readback-{}", uuid::Uuid::new_v4()));
    client.download_checked(&expected, &target).await?;
    assert_eq!(std::fs::read(target)?, body);
    Ok(())
}
#[tokio::main]
async fn main() -> Result<()> {
    assert_ne!(unsafe { libc::geteuid() }, 0);
    drive_runtime::disable_core_dumps()?;
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 4 || args[1] != "/DriveNative-Lab/20261028T124720-1e31763a" {
        return Err("expected authorized lab, new result and private test directory".into());
    }
    let output = Path::new(&args[2]);
    std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(output)?
        .write_all(b"{\"passed\":false}")?;
    let base = Path::new(&args[3]);
    assert!(base.is_absolute());
    std::fs::create_dir(base)?;
    std::fs::set_permissions(base, std::fs::Permissions::from_mode(0o700))?;
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
    let root = format!("{}/package-upgrade-{}", args[1], uuid::Uuid::new_v4());
    println!("test_subtree={root}");
    std::fs::write(
        output,
        serde_json::json!({"passed":false,"scope":root,"stage":"old package fixture"}).to_string(),
    )?;
    let directory = client.mkdir_native(home.view_id, &root).await?;
    let source = base.join("source");
    let original = b"original cached document from installed package";
    std::fs::write(&source, original)?;
    for name in ["A.doc", "Pinned.doc"] {
        client
            .upload_snapshot(
                home.view_id,
                &format!("{root}/{name}"),
                NativeVersion {
                    sync_id: 0,
                    max_id: 0,
                },
                &source,
            )
            .await?;
    }
    let mount = base.join("mount");
    std::fs::create_dir(&mount)?;
    let profile = Profile {
        schema: 1,
        task_id: uuid::Uuid::new_v4().to_string(),
        host: "192.0.2.10".into(),
        tls_name: "nas.example.invalid".into(),
        trusted_ca_pem: vec![],
        username: "test-user".into(),
        device_id: uuid::Uuid::new_v4().to_string(),
        scope: drive_core::native::Scope {
            view_id: home.view_id,
            node_id: directory.node_id,
            file_id: directory.file_id,
            path: root.clone(),
        },
        mount: mount.clone(),
        writable: true,
    };
    let config = base.join("profile.json");
    let mut old_profile = serde_json::to_value(&profile)?;
    old_profile
        .as_object_mut()
        .unwrap()
        .remove("trusted_ca_pem");
    std::fs::write(&config, serde_json::to_vec(&old_profile)?)?;
    std::fs::set_permissions(&config, std::fs::Permissions::from_mode(0o600))?;
    let data = base.join("data");
    let database = data
        .join("drive-native")
        .join(&profile.task_id)
        .join("state.sqlite");
    let log = base.join("fsd.log");
    let executable = Path::new("/usr/lib/drive-native/drive-native-fsd");
    let bus = zbus::Connection::session().await?;
    let proxy = zbus::Proxy::new(&bus, BUS_NAME, OBJECT_PATH, BUS_NAME).await?;
    let mut daemon = spawn(executable, &config, &data, &mount, &log)?;
    wait(&proxy, "old package locked", |v| {
        v["connection"] == "locked"
    })
    .await?;
    let _: () = proxy.call("Unlock", &(password.clone(),)).await?;
    wait(&proxy, "old package online", |v| {
        v["connection"] == "online" && v["mounted"] == true
    })
    .await?;
    assert_eq!(std::fs::read(mount.join("A.doc"))?, original);
    let _: () = proxy.call("SetPinned", &("Pinned.doc", true)).await?;
    wait_state(&proxy, "Pinned.doc", "pinned_complete").await?;
    let _: () = proxy.call("Pause", &()).await?;
    let edited = b"offline edit before actual deb upgrade";
    let created = b"offline creation before actual deb upgrade";
    save(&mount.join("A.doc"), edited, false)?;
    save(&mount.join("Created.doc"), created, true)?;
    assert_eq!(states(&proxy, "A.doc").await?["status"], "pending_upload");
    assert_eq!(
        states(&proxy, "Created.doc").await?["status"],
        "pending_upload"
    );
    stop(&proxy, &mut daemon).await?;
    let old_schema = schema(&database)?;
    assert_eq!(old_schema, 10);
    std::fs::write(output,serde_json::json!({"passed":false,"scope":root,"stage":"old package stopped with pending data","old_schema":old_schema}).to_string())?;
    gate(base, "upgrade").await?;
    daemon = spawn(executable, &config, &data, &mount, &log)?;
    wait(&proxy, "new package locked", |v| {
        v["connection"] == "locked" && v["mounted"] == true
    })
    .await?;
    assert_eq!(std::fs::read(mount.join("A.doc"))?, edited);
    assert_eq!(std::fs::read(mount.join("Created.doc"))?, created);
    assert_eq!(std::fs::read(mount.join("Pinned.doc"))?, original);
    assert_eq!(states(&proxy, "A.doc").await?["status"], "pending_upload");
    assert_eq!(
        states(&proxy, "Created.doc").await?["status"],
        "pending_upload"
    );
    assert_eq!(
        states(&proxy, "Pinned.doc").await?["status"],
        "pinned_complete"
    );
    assert_eq!(schema(&database)?, 14);
    let raw: String = proxy.call("GetTransferSettings", &()).await?;
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&raw)?["limits"],
        serde_json::json!({"download_bps":0,"upload_bps":0})
    );
    let _: () = proxy.call("Unlock", &(password.clone(),)).await?;
    wait(&proxy, "new package online", |v| {
        v["connection"] == "online"
    })
    .await?;
    wait_state(&proxy, "A.doc", "local_complete").await?;
    wait_state(&proxy, "Created.doc", "local_complete").await?;
    verify(&client, home.view_id, &root, "A.doc", edited, base).await?;
    verify(&client, home.view_id, &root, "Created.doc", created, base).await?;
    let _: () = proxy.call("Pause", &()).await?;
    let uninstall_draft = b"unsent draft retained across package removal and reinstall";
    save(&mount.join("Created.doc"), uninstall_draft, false)?;
    stop(&proxy, &mut daemon).await?;
    let db_hash = drive_core::sha256(&std::fs::read(&database)?);
    let profile_hash = drive_core::sha256(&std::fs::read(&config)?);
    gate(base, "remove").await?;
    assert!(!executable.exists());
    assert_eq!(drive_core::sha256(&std::fs::read(&database)?), db_hash);
    assert_eq!(drive_core::sha256(&std::fs::read(&config)?), profile_hash);
    gate(base, "reinstall").await?;
    assert!(executable.exists());
    daemon = spawn(executable, &config, &data, &mount, &log)?;
    wait(&proxy, "reinstalled package locked", |v| {
        v["connection"] == "locked" && v["mounted"] == true
    })
    .await?;
    assert_eq!(std::fs::read(mount.join("Created.doc"))?, uninstall_draft);
    assert_eq!(
        states(&proxy, "Created.doc").await?["status"],
        "pending_upload"
    );
    assert_eq!(std::fs::read(mount.join("Pinned.doc"))?, original);
    stop(&proxy, &mut daemon).await?;
    assert!(!std::fs::read_to_string(&log)?.contains(&password));
    std::fs::write(
        output,
        serde_json::to_vec_pretty(
            &serde_json::json!({"passed":true,"scope":root,"old_schema":old_schema,"new_schema":14,"installed_packages_in_clean_chroot":true,"actual_fuse_cache_and_pin":true,"offline_edit_and_create_survived_upgrade":true,"migrated_uploads_independently_verified":true,"remove_kept_database_and_profile_bytes":true,"reinstall_kept_unsent_draft_and_pinned_content":true,"locked_offline_mount_after_reinstall":true,"user_id":unsafe{libc::geteuid()},"mint_tested":false}),
        )?,
    )?;
    println!(
        "PASS installed deb upgrade, native pending uploads, purge and reinstalled offline draft"
    );
    Ok(())
}
