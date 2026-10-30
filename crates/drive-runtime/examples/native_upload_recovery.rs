//! Lost coordinator/unread wire acknowledgement, updates and foreign-create rejection.
#[path = "support/commit_observation.rs"]
mod commit_observation;
use drive_core::{
    Status, Store,
    native::{NativeReadSession, NativeWriteOutcome, Scope},
};
use drive_runtime::{
    bridge::{Bridge, WorkerProcess},
    wire::Login,
};
use sproto::{
    Client,
    actions::native::NativeVersion,
    client::{Config, Credentials},
};
use std::{
    io::Write,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
async fn connect(password: &str) -> Result<Client> {
    Ok(Client::connect(
        Config::builder()
            .host("192.0.2.10")
            .tls_server_name("nas.example.invalid".to_owned())
            .credentials(Credentials::Password {
                username: "test-user".into(),
                password: password.into(),
                otp: None,
            })
            .build(),
    )
    .await?)
}
#[tokio::main]
async fn main() -> Result<()> {
    assert_ne!(unsafe { libc::geteuid() }, 0);
    unsafe {
        let l = libc::rlimit {
            rlim_cur: 0,
            rlim_max: 0,
        };
        libc::setrlimit(libc::RLIMIT_CORE, &l);
    }
    let args: Vec<_> = std::env::args().collect();
    if !(args.len() == 3 || (args.len() == 4 && args[3] == "--unread-reply"))
        || args[1] != "/DriveNative-Lab/20261028T124720-1e31763a"
    {
        return Err("expected authorized lab and new result path".into());
    }
    let unread = args.len() == 4;
    std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&args[2])?
        .write_all(b"{\"passed\":false}")?;
    let password = rpassword::prompt_password("Test account password (not saved): ")?;
    let author = connect(&password).await?;
    let observer = connect(&password).await?;
    let home = author.personal_root().await?;
    assert!(home.writable && home.download_allowed);
    let root = format!("{}/file-origin-{}", args[1], uuid::Uuid::new_v4());
    println!("test_subtree={root}");
    let receipt = author.mkdir_native(home.view_id, &root).await?;
    let scope = Scope {
        view_id: home.view_id,
        node_id: receipt.node_id,
        file_id: receipt.file_id,
        path: root.clone(),
    };
    let session = NativeReadSession::new(author.clone(), scope.clone())?;
    let scratch = tempfile::tempdir()?;
    let data = scratch.path().join("data");
    let source = scratch.path().join("source");
    let mount = scratch.path().join("mount");
    std::fs::create_dir(&mount)?;
    let body = b"same bytes are not evidence of who created the file";
    std::fs::write(&source, body)?;
    let hash = drive_core::sha256(body);
    if unread {
        std::fs::write(&source, b"base before uncertain existing-file update")?;
        author
            .upload_snapshot(
                home.view_id,
                &format!("{root}/ExistingUpdate.txt"),
                NativeVersion {
                    sync_id: 0,
                    max_id: 0,
                },
                &source,
            )
            .await?;
        std::fs::write(&source, body)?;
    }
    let mut store = Store::open(&data)?;
    session.bind(&mut store)?;
    while let Some(batch) = session.pull(&store.cursor()?).await? {
        store.receive(&batch)?;
        while store.apply_next()? {}
    }
    store.ensure_fs_root(&scope.file_id)?;
    let mut cases = Vec::new();
    let names = if unread {
        vec![
            "LostAck.txt",
            "ExistingUpdate.txt",
            "ForeignSame.txt",
            "LegacyUnknown.txt",
        ]
    } else {
        vec!["LostAck.txt", "ForeignSame.txt", "LegacyUnknown.txt"]
    };
    for name in &names {
        let name = *name;
        let existing = name == "ExistingUpdate.txt";
        let node = if existing {
            store.lookup(&scope.file_id, name)?.node
        } else {
            store.create_local_file(&scope.file_id, name, 0o644)?
        };
        let inode = store.record(&node.id)?.ino;
        let op = store.save_local(&node.id, body)?;
        if !existing {
            store.creation_saved(&node.id)?;
        }
        let path = format!("{root}/{name}");
        let origin = if !existing && name != "LegacyUnknown.txt" {
            let origin = session
                .prepare_upload_creation(
                    &node,
                    &store.node(&scope.file_id)?,
                    &path,
                    &hash,
                    &store.cursor()?,
                )
                .await?
                .ok_or("unexpected existing target")?;
            store.prepare_upload_origin(&op, &origin, &hash)?;
            Some(origin)
        } else {
            None
        };
        store.mark_upload_started(&op)?;
        if unread && (name == "LostAck.txt" || existing) {
            let base = if existing {
                let value: serde_json::Value = serde_json::from_str(&node.version)?;
                NativeVersion {
                    sync_id: value["sync"].as_u64().ok_or("missing base sync")?,
                    max_id: value["max"].as_u64().ok_or("missing base max")?,
                }
            } else {
                NativeVersion {
                    sync_id: 0,
                    max_id: 0,
                }
            };
            let identity = if existing {
                serde_json::from_str::<serde_json::Value>(&node.version)?["file"]
                    .as_str()
                    .ok_or("missing file identity")?
                    .to_owned()
            } else {
                origin
                    .as_ref()
                    .ok_or("missing creation origin")?
                    .parent_file
                    .clone()
            };
            let result = author
                .research_upload_identity_drop_receipt_after(
                    &identity,
                    if existing { None } else { Some(name) },
                    base,
                    &source,
                    commit_observation::file_after(
                        &observer,
                        home.view_id,
                        &root,
                        name,
                        body.len() as u64,
                        base.sync_id,
                    ),
                )
                .await;
            commit_observation::assert_unread_fault(result);
            println!("PASS independently observed unread upload: {name}");
        } else if name == "LostAck.txt" {
            let result = session
                .upload_with_origin(
                    &node,
                    &path,
                    &source,
                    &hash,
                    scratch.path(),
                    origin.as_ref(),
                )
                .await?;
            assert!(matches!(result, NativeWriteOutcome::Created { .. }));
            // Deliberately discard the acknowledgement; leave the journal inflight
            // and reopen it, as after fsd termination before confirming the reply.
        } else {
            observer
                .upload_snapshot(
                    home.view_id,
                    &path,
                    NativeVersion {
                        sync_id: 0,
                        max_id: 0,
                    },
                    &source,
                )
                .await?;
        }
        cases.push((node, op, inode, origin));
    }
    let before = observer.list_dir(home.view_id, &root).await?;
    assert_eq!(before.len(), names.len());
    std::fs::write(&args[2],serde_json::json!({"passed":false,"scope":root,"stage":"commits observed; reopening journal"}).to_string())?;
    drop(session);
    drop(author);
    drop(store);
    let store = Arc::new(Mutex::new(Store::open(&data)?));
    for (_, op, _, origin) in &cases {
        assert_eq!(store.lock().unwrap().upload_job(op)?.state, "uncertain");
        assert_eq!(&store.lock().unwrap().upload_origin(op)?, origin);
    }
    let exe = std::env::current_exe()?
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("drive-native-worker");
    let mut worker = WorkerProcess::start(&exe)?;
    let bridge = Arc::new(Bridge::connect(
        store.clone(),
        worker.rpc.clone(),
        Login {
            host: "192.0.2.10".into(),
            tls_name: "nas.example.invalid".into(),
            trusted_ca_pem: Vec::new(),
            username: "test-user".into(),
            password: password.clone(),
            device_id: uuid::Uuid::new_v4().to_string(),
            scope: scope.clone(),
        },
    )?);
    let fs = drive_fs::CacheFilesystem::new(
        store.clone(),
        &scope.file_id,
        unsafe { libc::getuid() },
        unsafe { libc::getgid() },
    )?
    .with_writes_enabled()
    .with_hydrator(bridge.clone());
    let mounted = fuser::spawn_mount2(
        fs,
        &mount,
        &[
            fuser::MountOption::RW,
            fuser::MountOption::DefaultPermissions,
            fuser::MountOption::NoDev,
            fuser::MountOption::NoSuid,
        ],
    )?;
    bridge.set_notifier(mounted.notifier());
    let deadline = Instant::now() + Duration::from_secs(40);
    let mut count = 0;
    loop {
        count += bridge.upload_pending(true)?;
        if count == if unread { 2 } else { 1 } {
            break;
        }
        if Instant::now() > deadline {
            return Err("creation reconciliation did not confirm owned file".into());
        }
        std::thread::sleep(Duration::from_secs(2));
    }
    let owned = &cases[0];
    let foreign = cases
        .iter()
        .find(|c| c.0.name == "ForeignSame.txt")
        .unwrap();
    let legacy = cases
        .iter()
        .find(|c| c.0.name == "LegacyUnknown.txt")
        .unwrap();
    if unread {
        let existing = cases
            .iter()
            .find(|c| c.0.name == "ExistingUpdate.txt")
            .unwrap();
        assert_eq!(
            store.lock().unwrap().upload_job(&existing.1)?.state,
            "committed"
        );
        assert_eq!(
            store.lock().unwrap().record(&existing.0.id)?.ino,
            existing.2
        );
        let remote = before
            .iter()
            .find(|n| n.name == "ExistingUpdate.txt")
            .unwrap();
        assert_eq!(
            store.lock().unwrap().local_identity(&remote.file_id)?,
            existing.0.id
        );
    }
    assert_eq!(store.lock().unwrap().record(&owned.0.id)?.ino, owned.2);
    let remote = before.iter().find(|n| n.name == "LostAck.txt").unwrap();
    assert_eq!(
        store.lock().unwrap().local_identity(&remote.file_id)?,
        owned.0.id
    );
    assert_eq!(
        store.lock().unwrap().upload_job(&owned.1)?.state,
        "committed"
    );
    assert_eq!(
        store.lock().unwrap().status(&foreign.0.id)?,
        Status::Conflict
    );
    assert_eq!(
        store.lock().unwrap().upload_job(&legacy.1)?.state,
        "uncertain"
    );
    for (node, _, _, _) in &cases {
        assert_eq!(std::fs::read(mount.join(&node.name))?, body);
    }
    let after = observer.list_dir(home.view_id, &root).await?;
    for a in &before {
        let b = after
            .iter()
            .find(|b| b.name == a.name)
            .ok_or("remote disappeared")?;
        assert_eq!(
            (&a.file_id, a.sync_id, a.max_id, &a.hash),
            (&b.file_id, b.sync_id, b.max_id, &b.hash)
        );
    }
    println!(
        "PASS own lost acknowledgement recovered; equal-content foreign and legacy creations not adopted"
    );
    // A new file through the real FUSE/worker path also exercises read-only
    // preparation and durable storage before the ordinary upload.
    std::fs::write(mount.join("Normal.txt"), b"normal prepared FUSE creation")?;
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        if bridge.upload_pending(false)? > 0 {
            break;
        }
        if Instant::now() > deadline {
            return Err("normal prepared upload did not finish".into());
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    let changed = b"edit after restored remote identity";
    std::fs::write(mount.join("LostAck.txt"), changed)?;
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        if bridge.upload_pending(false)? > 0 {
            break;
        }
        if Instant::now() > deadline {
            return Err("update after recovery did not finish".into());
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    let listed = observer.list_dir(home.view_id, &root).await?;
    let current = listed.iter().find(|n| n.name == "LostAck.txt").unwrap();
    assert_eq!(current.file_id, remote.file_id);
    let dest = scratch.path().join("verify");
    observer
        .download_checked(
            &sproto::actions::transfer::ContentVersion {
                file_id: current.file_id.clone(),
                version: NativeVersion {
                    sync_id: current.sync_id,
                    max_id: current.max_id,
                },
                size: current.file_size,
                hash: current.hash.clone(),
            },
            &dest,
        )
        .await?;
    assert_eq!(std::fs::read(&dest)?, changed);
    mounted.join();
    worker.stop();
    drop(bridge);
    drop(store);
    let store = Store::open(&data)?;
    assert_eq!(store.status(&foreign.0.id)?, Status::Conflict);
    assert_eq!(store.upload_job(&legacy.1)?.state, "uncertain");
    std::fs::write(
        &args[2],
        serde_json::to_vec_pretty(
            &serde_json::json!({"passed":true,"scope":root,"server":"Drive 4.0.3-27892","coordinator_acknowledgement_discarded":!unread,"coordinator_result_not_persisted":true,"wire_response_discarded":unread,"independent_commit_observed_before_socket_drop":unread,"existing_update_unread_recovered":unread,"unread_targets":if unread { vec!["LostAck.txt", "ExistingUpdate.txt"] } else { vec![] },
                "remote_before_reconcile":before.iter().map(|n| serde_json::json!({"name":n.name,"id":n.file_id,"sync":n.sync_id,"max":n.max_id,"hash":n.hash})).collect::<Vec<_>>(),
                "remote_after_reconcile":after.iter().map(|n| serde_json::json!({"name":n.name,"id":n.file_id,"sync":n.sync_id,"max":n.max_id,"hash":n.hash})).collect::<Vec<_>>(),
                "journal_reopened_inflight_to_uncertain":true,"new_authenticated_worker":true,"own_event_author_and_full_content_verified":true,"stable_local_inode":true,"foreign_identical_content_rejected":true,"legacy_missing_provenance_retained":true,"reconciliation_did_not_mutate_remote_versions":true,"fuse_normal_creation_prepared":true,"edit_after_recovery_uses_same_remote_identity":true,"conflict_and_unknown_state_survive_restart":true,"mint_tested":false}),
        )?,
    )?;
    println!("PASS native upload provenance, FUSE, journal restart and edit-after-recovery");
    Ok(())
}
