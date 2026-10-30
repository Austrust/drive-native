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
use std::os::unix::fs::PermissionsExt;
#[path = "support/service.rs"]
mod support;
use support::*;
#[tokio::main]
async fn main() -> Result<()> {
    assert_ne!(unsafe { libc::geteuid() }, 0);
    let args: Vec<_> = std::env::args().collect();
    drive_runtime::disable_core_dumps()?;
    if args.len() != 3 || args[1] != "/DriveNative-Lab/20261028T124720-1e31763a" {
        return Err("expected authorized lab and new result".into());
    }
    use std::io::Write;
    std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&args[2])?
        .write_all(b"{\"passed\":false}")?;
    let lab = &args[1];
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
    let root = format!("{lab}/root-service-{}", uuid::Uuid::new_v4());
    println!("test_subtree={root}");
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
    let _first = client
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

    // Lose the root while online: the real native event loop must stop syncing,
    // without unmounting or deleting the downloaded contents.
    let parent = lab.to_string();
    let destination = format!("{root}-moved");
    let current = client
        .list_dir(home.view_id, &parent)
        .await?
        .into_iter()
        .find(|n| n.file_id == profile.scope.file_id)
        .ok_or("root missing")?;
    client
        .rename_native(
            &sproto::actions::namespace::NamespaceTarget::from_node(
                home.view_id,
                root.clone(),
                &current,
            )?,
            &destination,
        )
        .await?;
    let replacement = client.mkdir_native(home.view_id, &root).await?;
    assert_ne!(replacement.file_id, profile.scope.file_id);
    wait(&proxy, "root attention", |v| {
        v["connection"] == "needs_attention"
    })
    .await?;
    let stopped = overview(&proxy).await?;
    assert_eq!(stopped["error"], "native_root_unavailable");
    assert_eq!(stopped["mounted"], true);
    assert_eq!(std::fs::read(mount.join("A.doc"))?, b"original-v1");
    std::fs::write(
        mount.join("Queued.txt"),
        b"keep my local draft while the remote root is unavailable",
    )?;
    wait_state(&proxy, "Queued.txt", "pending_upload").await?;
    assert!(client.list_dir(home.view_id, &root).await?.is_empty());
    let local_state = states(&proxy, "Queued.txt").await?;
    println!("PASS root unavailable: cache and local draft readable, replacement untouched");
    stop(&proxy, &mut daemon).await?;
    drop(daemon);
    let mut daemon = spawn(&executable, &config, &data, &mount, &log)?;
    wait(&proxy, "cached restart", |v| {
        v["connection"] == "locked" && v["mounted"] == true
    })
    .await?;
    assert_eq!(std::fs::read(mount.join("A.doc"))?, b"original-v1");
    assert_eq!(
        std::fs::read(mount.join("Queued.txt"))?,
        b"keep my local draft while the remote root is unavailable"
    );
    let _: () = proxy.call("Unlock", &(password.clone(),)).await?;
    wait(&proxy, "root rejected after restart", |v| {
        v["connection"] == "needs_attention"
    })
    .await?;
    assert_eq!(overview(&proxy).await?["error"], "native_root_unavailable");
    assert!(client.list_dir(home.view_id, &root).await?.is_empty());
    println!("PASS restart: cache and local draft survive, replacement still untouched");
    // Restore the same identity, not a different folder at the same pathname.
    let replacement = client
        .list_dir(home.view_id, &parent)
        .await?
        .into_iter()
        .find(|n| n.file_id == replacement.file_id)
        .ok_or("replacement missing")?;
    client
        .remove_native(&sproto::actions::namespace::NamespaceTarget::from_node(
            home.view_id,
            root.clone(),
            &replacement,
        )?)
        .await?;
    let original = client
        .list_dir(home.view_id, &parent)
        .await?
        .into_iter()
        .find(|n| n.file_id == profile.scope.file_id)
        .ok_or("original missing")?;
    client
        .rename_native(
            &sproto::actions::namespace::NamespaceTarget::from_node(
                home.view_id,
                destination.clone(),
                &original,
            )?,
            &root,
        )
        .await?;
    let current = client
        .list_dir(home.view_id, &parent)
        .await?
        .into_iter()
        .find(|n| n.file_id == profile.scope.file_id)
        .ok_or("restored root absent")?;
    let raw = match client
        .pull_events(&sproto::actions::native::Watch {
            view_id: home.view_id,
            node_id: profile.scope.node_id,
            path: root.clone(),
            cursor: sproto::actions::native::EventCursor(0),
        })
        .await
    {
        Ok(batch) => {
            serde_json::json!({"next":batch.next.0,"events":batch.events.iter().map(|e|serde_json::json!({"file_id":e.file_id,"path":e.path,"kind":format!("{:?}",e.kind),"removed":e.removed})).collect::<Vec<_>>()})
        }
        Err(e) => serde_json::json!({"error":format!("{e}")}),
    };
    std::fs::write(
        &args[2],
        serde_json::to_vec_pretty(
            &serde_json::json!({"passed":false,"phase":"same-root-restored-before-unlock","scope":profile.scope,"current_node_id":current.node_id,"raw":raw,"cache_and_draft_survived_restart":true,"replacement_untouched":true}),
        )?,
    )?;

    let _: () = proxy.call("Unlock", &(password.clone(),)).await?;
    wait(&proxy, "same root restored", |v| {
        v["connection"] == "online"
    })
    .await?;
    wait_state(&proxy, "Queued.txt", "local_complete").await?;
    let listing = client.list_dir(home.view_id, &root).await?;
    let uploaded = listing
        .iter()
        .find(|n| n.name == "Queued.txt")
        .ok_or("draft not uploaded after restore")?;
    let verified = sproto::actions::transfer::ContentVersion {
        file_id: uploaded.file_id.clone(),
        version: NativeVersion {
            sync_id: uploaded.sync_id,
            max_id: uploaded.max_id,
        },
        size: uploaded.file_size,
        hash: uploaded.hash.clone(),
    };
    client
        .download_checked(&verified, &scratch.path().join("draft-verified"))
        .await?;
    assert_eq!(
        std::fs::read(scratch.path().join("draft-verified"))?,
        b"keep my local draft while the remote root is unavailable"
    );
    stop(&proxy, &mut daemon).await?;
    std::fs::write(
        &args[2],
        serde_json::to_vec_pretty(&serde_json::json!({"passed":true,"scope":root,
        "real_fuse":true,"real_worker_and_service":true,"root_error":stopped["error"],
        "mount_survived":stopped["mounted"],"draft_state":local_state["status"],
        "restart_preserved_cache_and_draft":true,"replacement_untouched":true,
        "same_identity_restored_and_draft_uploaded":true,"uploaded_identity":uploaded.file_id}))?,
    )?;
    println!(
        "PASS root attention, cache, offline draft, restart, same-identity restore and upload"
    );
    Ok(())
}
