//! Explicit NAS/FUSE recovery test, confined to an authorized lab child.
use drive_core::{
    Status, Store,
    native::{NativeDirectoryOutcome, NativeReadSession, Scope},
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
    sync::Arc,
    time::{Duration, Instant},
};
async fn connect(password: &str) -> Result<Client, Box<dyn std::error::Error>> {
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
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    assert_ne!(unsafe { libc::getuid() }, 0);
    let args: Vec<_> = std::env::args().collect();
    let wire_unread = match args.get(3).map(String::as_str) {
        None if args.len() == 3 => false,
        Some("--wire-unread") if args.len() == 4 => true,
        _ => return Err(
            "usage: native_directory_recovery <authorized-lab> <new-result.json> [--wire-unread]"
                .into(),
        ),
    };
    let lab = args.get(1).ok_or("authorized lab required")?;
    if lab != "/DriveNative-Lab/20261028T124720-1e31763a" {
        return Err("scope not authorized for this fixture".into());
    }
    let output = args.get(2).ok_or("result path required")?;
    use std::io::Write;
    std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output)?
        .write_all(b"{\"passed\":false}\n")?;
    let password = rpassword::prompt_password("Test account password (not saved): ")?;
    let a = connect(&password).await?;
    let observer = connect(&password).await?;
    let home = a.personal_root().await?;
    assert!(home.writable && home.download_allowed);
    let root = format!("{lab}/mkdir-recovery-{}", uuid::Uuid::new_v4());
    println!("test_subtree={root}");
    std::fs::write(
        output,
        serde_json::json!({"passed":false,"scope":root,"stage":"creating fixture"}).to_string(),
    )?;
    let receipt = a.mkdir_native(home.view_id, &root).await?;
    let scope = Scope {
        view_id: home.view_id,
        node_id: receipt.node_id,
        file_id: receipt.file_id,
        path: root.clone(),
    };
    let session = NativeReadSession::new(a.clone(), scope.clone())?;
    let scratch = tempfile::tempdir()?;
    let state = scratch.path().join("state");
    let mount = scratch.path().join("mount");
    std::fs::create_dir(&mount)?;
    let mut store = Store::open(&state)?;
    session.bind(&mut store)?;
    for _ in 0..20 {
        match session.pull(&store.cursor()?).await? {
            Some(b) => {
                store.receive(&b)?;
                while store.apply_next()? {}
            }
            None => break,
        }
    }
    store.ensure_fs_root(&scope.file_id)?;
    let dir = store.create_local_directory(&scope.file_id, "LostAck", 0o755)?;
    let inode = store.record(&dir.id)?.ino;
    let child = store.create_local_file(&dir.id, "note.txt", 0o644)?;
    let bytes = b"offline child survives lost mkdir receipt and worker restart";
    let child_op = store.save_local(&child.id, bytes)?;
    store.creation_saved(&child.id)?;
    let q = store.next_directory()?.unwrap();
    let job = store.mark_directory_started(&q.job.operation)?;
    let NativeDirectoryOutcome::Prepared(origin) = session.create_directory(&job, false).await?
    else {
        return Err("missing read-only preparation".into());
    };
    assert!(
        !observer
            .list_dir(home.view_id, &root)
            .await?
            .iter()
            .any(|n| n.name == "LostAck")
    );
    store.prepare_directory(&q.job.operation, origin.clone())?;
    let sent = store.mark_directory_started(&q.job.operation)?;
    assert_eq!(sent.origin, Some(origin.clone()));
    if wire_unread {
        let result = a
            .research_mkdir_identity_discard_reply_after(&scope.file_id, "LostAck", async {
                loop {
                    let entries = observer.list_dir(home.view_id, &root).await?;
                    if entries.iter().any(|n| n.name == "LostAck" && !n.is_removed) {
                        return Ok(());
                    }
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            })
            .await;
        if !matches!(
            result,
            Err(sproto::Error::Timeout(
                "injected unread identity directory reply"
            ))
        ) {
            return Err("unread response fault did not reach independently observed commit".into());
        }
    } else {
        assert!(
            session
                .research_create_directory_discard_ack(&sent)
                .await
                .is_err()
        );
    }
    store.mark_directory_uncertain(&q.job.operation)?;
    assert!(store.mark_upload_started(&child_op).is_err());
    let deadline = Instant::now() + Duration::from_secs(15);
    let remote = loop {
        if let Some(n) = observer
            .list_dir(home.view_id, &root)
            .await?
            .into_iter()
            .find(|n| n.name == "LostAck")
        {
            break n;
        }
        if Instant::now() > deadline {
            return Err(
                "injected request did not commit; not a valid lost-ack recovery run".into(),
            );
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    };
    drop(session);
    drop(a);
    drop(store);
    let store = Arc::new(drive_core::SharedStore::new(Store::open(&state)?));
    assert_eq!(
        store.lock().unwrap().next_directory()?.unwrap().job.origin,
        Some(origin)
    );
    assert_eq!(store.lock().unwrap().record(&dir.id)?.ino, inode);
    let executable = std::env::current_exe()?
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("drive-native-worker");
    let mut worker = WorkerProcess::start(&executable)?;
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
    assert_eq!(std::fs::read(mount.join("LostAck/note.txt"))?, bytes);
    assert_eq!(bridge.namespace_pending()?, 1);
    assert_eq!(
        store.lock().unwrap().local_identity(&remote.file_id)?,
        dir.id
    );
    assert!(store.lock().unwrap().next_namespace_kind()?.is_none());
    assert_eq!(bridge.upload_pending(false)?, 1);
    assert_eq!(
        store.lock().unwrap().status(&child.id)?,
        Status::LocalComplete
    );
    let listed = observer
        .list_dir(home.view_id, &format!("{root}/LostAck"))
        .await?;
    let n = listed
        .iter()
        .find(|n| n.name == "note.txt")
        .ok_or("child upload missing")?;
    let download = scratch.path().join("verify");
    observer
        .download_checked(
            &sproto::actions::transfer::ContentVersion {
                file_id: n.file_id.clone(),
                version: NativeVersion {
                    sync_id: n.sync_id,
                    max_id: n.max_id,
                },
                size: n.file_size,
                hash: n.hash.clone(),
            },
            &download,
        )
        .await?;
    assert_eq!(std::fs::read(download)?, bytes);
    println!(
        "PASS native lost mkdir response, restarted Store/new worker, stable inode and child upload"
    );
    // A different session creates the target after our durable preparation.
    let foreign = connect(&password).await?;
    let own = NativeReadSession::new(observer.clone(), scope.clone())?;
    let (q, child2, child_op2) = {
        let mut s = store.lock().unwrap();
        let d = s.create_local_directory(&scope.file_id, "Foreign", 0o755)?;
        let f = s.create_local_file(&d.id, "private.txt", 0o600)?;
        let op = s.save_local(&f.id, b"must not enter foreign directory")?;
        s.creation_saved(&f.id)?;
        let q = s.next_directory()?.unwrap();
        (q, f, op)
    };
    let job = store
        .lock()
        .unwrap()
        .mark_directory_started(&q.job.operation)?;
    let NativeDirectoryOutcome::Prepared(origin) = own.create_directory(&job, false).await? else {
        return Err("foreign case preflight failed".into());
    };
    store
        .lock()
        .unwrap()
        .prepare_directory(&q.job.operation, origin)?;
    store
        .lock()
        .unwrap()
        .mark_directory_started(&q.job.operation)?;
    let other = foreign
        .mkdir_native(home.view_id, &format!("{root}/Foreign"))
        .await?;
    store
        .lock()
        .unwrap()
        .mark_directory_uncertain(&q.job.operation)?;
    assert_eq!(bridge.namespace_pending()?, 0);
    assert_eq!(
        store.lock().unwrap().next_directory()?.unwrap().state,
        "conflict"
    );
    assert!(
        store
            .lock()
            .unwrap()
            .mark_upload_started(&child_op2)
            .is_err()
    );
    assert_eq!(
        std::fs::read(mount.join("Foreign/private.txt"))?,
        b"must not enter foreign directory"
    );
    assert!(
        foreign
            .list_dir(home.view_id, &format!("{root}/Foreign"))
            .await?
            .is_empty()
    );
    assert_ne!(
        store.lock().unwrap().local_identity(&other.file_id)?,
        q.job.node.id
    );
    assert_eq!(
        store.lock().unwrap().node(&child2.id)?.parent.as_deref(),
        Some(q.job.node.id.as_str())
    );
    println!(
        "PASS foreign same-name directory is not adopted; local child retained and not uploaded"
    );
    drop(mounted);
    worker.stop();
    drop(bridge);
    drop(store);
    let store = Store::open(&state)?;
    assert_eq!(store.next_directory()?.unwrap().state, "conflict");
    std::fs::write(
        output,
        serde_json::to_string_pretty(
            &serde_json::json!({"passed":true,"scope":root,"native_drop_before_reply":wire_unread,"wire_response_read":!wire_unread,"independent_observation_before_socket_drop":wire_unread,"worker_ack_dropped_after_server_reply":!wire_unread,"remote_commit_independently_observed":true,"reopened_store_new_worker_reconciles":true,"same_local_inode":true,"fuse_local_child_preserved":true,"child_uploaded_after_parent_confirmation":true,"foreign_same_name_not_adopted":true,"foreign_directory_child_upload_blocked":true,"conflict_survives_restart":true,"unknown_absent_target_retry_tested":false,"legacy_unknown_auto_recovery":false}),
        )? + "\n",
    )?;
    Ok(())
}
