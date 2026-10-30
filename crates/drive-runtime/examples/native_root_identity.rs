//! Read-only diagnosis after explicit mutations of a fresh lab task root.
use drive_core::{
    Change,
    native::{NativeReadSession, Scope},
};
use sproto::{
    Client,
    actions::{namespace::NamespaceTarget, native::NativeVersion},
    client::{Config, Credentials},
};
use std::io::Write;
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
async fn observe(
    client: &Client,
    session: &NativeReadSession,
    scope: &Scope,
    cursor: &str,
) -> serde_json::Value {
    let raw = match client.pull_events(&session.watch(cursor).unwrap()).await {
        Ok(batch) => {
            serde_json::json!({"next":batch.next.0,"recovery_required":batch.recovery_required,
            "events":batch.events.iter().map(|e|serde_json::json!({"id":e.file_id,"path":e.path,
                "parent":e.parent_id,"kind":format!("{:?}",e.kind),"removed":e.removed})).collect::<Vec<_>>() })
        }
        Err(sproto::Error::Server { code, .. }) => serde_json::json!({"server_error":code}),
        Err(_) => serde_json::json!({"other_protocol_error":true}),
    };
    let mapped = match session.pull(cursor).await {
        Ok(batch) => {
            let root_delete = batch.as_ref().is_some_and(|b| {
                b.events
                    .iter()
                    .any(|e| matches!(&e.change,Change::Delete{id} if id==&scope.file_id))
            });
            serde_json::json!({"accepted":true,"would_delete_root":root_delete,
                "changes":batch.map(|b|b.events.len())})
        }
        Err(error) => {
            serde_json::json!({"accepted":false,"error":format!("{error}"),"would_delete_root":false})
        }
    };
    serde_json::json!({"raw":raw,"mapped":mapped})
}
#[tokio::main]
async fn main() -> Result<()> {
    assert_ne!(unsafe { libc::geteuid() }, 0);
    drive_runtime::disable_core_dumps()?;
    let args: Vec<_> = std::env::args().collect();
    if !(args.len() == 3 || args.len() == 4)
        || args[1] != "/DriveNative-Lab/20261028T124720-1e31763a"
    {
        return Err("expected lab and new output".into());
    }
    std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&args[2])?
        .write_all(b"{\"completed\":false}")?;
    let password = rpassword::prompt_password("Test account password (not saved): ")?;
    let a = connect(&password).await?;
    let b = connect(&password).await?;
    drop(password);
    if args.len() == 4 {
        let value: serde_json::Value = serde_json::from_slice(&std::fs::read(&args[3])?)?;
        let scope: Scope = serde_json::from_value(value["scope"].clone())?;
        if !scope
            .path
            .starts_with(&format!("{}/root-service-", args[1]))
        {
            return Err("not an authorized root fixture".into());
        }
        let session = NativeReadSession::new(a.clone(), scope.clone())?;
        let mut cursor = "native-v1:0".to_owned();
        let mut pages = Vec::new();
        for _ in 0..32 {
            let batch = a.pull_events(&session.watch(&cursor)?).await?;
            pages.push(observe(&a, &session, &scope, &cursor).await);
            let next = format!("native-v1:{}", batch.next.0);
            if next == cursor {
                break;
            }
            cursor = next;
        }
        std::fs::write(
            &args[2],
            serde_json::to_vec_pretty(
                &serde_json::json!({"scope":scope,"read_only":true,"pages":pages}),
            )?,
        )?;
        return Ok(());
    }
    let home = a.personal_root().await?;
    assert!(home.writable);
    let run = format!("{}/root-identity-{}", args[1], uuid::Uuid::new_v4());
    a.mkdir_native(home.view_id, &run).await?;
    println!("test_subtree={run}");
    let scratch = tempfile::tempdir()?;
    let source = scratch.path().join("source");
    std::fs::write(&source, b"root identity test fixture")?;
    let mut cases = Vec::new();
    for mode in ["rename-root", "delete-root"] {
        let path = format!("{run}/{mode}");
        let receipt = a.mkdir_native(home.view_id, &path).await?;
        a.upload_snapshot(
            home.view_id,
            &format!("{path}/A.txt"),
            NativeVersion {
                sync_id: 0,
                max_id: 0,
            },
            &source,
        )
        .await?;
        let scope = Scope {
            view_id: home.view_id,
            node_id: receipt.node_id,
            file_id: receipt.file_id,
            path: path.clone(),
        };
        let session = NativeReadSession::new(a.clone(), scope.clone())?;
        let baseline = session.recovery_snapshot().await?;
        let data = scratch.path().join(mode);
        let mut store = drive_core::Store::open(&data)?;
        session.bind(&mut store)?;
        while let Some(batch) = session.pull(&store.cursor()?).await? {
            store.receive(&batch)?;
            store.drain()?;
        }
        store.ensure_fs_root(&scope.file_id)?;
        let node = store.create_local_file(&scope.file_id, "Queued.txt", 0o644)?;
        let body = b"prepared local content before root changed";
        let hash = drive_core::sha256(body);
        std::fs::write(&source, body)?;
        let operation = store.save_local(&node.id, body)?;
        store.creation_saved(&node.id)?;
        let queued_path = format!("{path}/Queued.txt");
        let origin = session
            .prepare_upload_creation(
                &node,
                &store.node(&scope.file_id)?,
                &queued_path,
                &hash,
                &store.cursor()?,
            )
            .await?
            .ok_or("missing origin")?;
        store.prepare_upload_origin(&operation, &origin, &hash)?;
        let current = b
            .list_dir(home.view_id, &run)
            .await?
            .into_iter()
            .find(|n| n.file_id == scope.file_id)
            .ok_or("selected root missing")?;
        let target = NamespaceTarget::from_node(home.view_id, path.clone(), &current)?;
        if mode == "rename-root" {
            let destination = format!("{run}/Moved");
            let moved = b.rename_native(&target, &destination).await?;
            assert_eq!(moved.file_id, scope.file_id);
            let after_move = observe(&a, &session, &scope, &baseline.cursor).await;
            let replacement = b.mkdir_native(home.view_id, &path).await?;
            assert_ne!(replacement.file_id, scope.file_id);
            let after_reuse = observe(&a, &session, &scope, &baseline.cursor).await;
            let queued_session = NativeReadSession::new(a.clone(), scope.clone())?;
            let write = queued_session
                .upload_with_origin(
                    &node,
                    &queued_path,
                    &source,
                    &hash,
                    scratch.path(),
                    Some(&origin),
                )
                .await;
            let outcome = match write {
                Ok(drive_core::native::NativeWriteOutcome::Created { .. }) => "created".to_owned(),
                Ok(_) => "other-outcome".to_owned(),
                Err(e) => format!("error:{e}"),
            };
            let replacement_has_queued = b
                .list_dir(home.view_id, &path)
                .await?
                .iter()
                .any(|n| n.name == "Queued.txt" && !n.is_removed);
            let moved_has_queued = b
                .list_dir(home.view_id, &destination)
                .await?
                .iter()
                .any(|n| n.name == "Queued.txt" && !n.is_removed);
            let mut local = vec![0; body.len()];
            let count = store.open_cached(&node.id)?.read_at(&mut local, 0)?;
            let local_draft_preserved = count == body.len() && local == body;
            cases.push(serde_json::json!({"mode":mode,"original_root":scope.file_id,"replacement_root":replacement.file_id,
                "queued_write":{"outcome":outcome,"replacement_has_queued":replacement_has_queued,"moved_has_queued":moved_has_queued,"local_draft_preserved":local_draft_preserved},"original_path":path,"new_path":destination,"after_move":after_move,"after_path_reuse":after_reuse}));
        } else {
            b.remove_native(&target).await?;
            assert!(
                !b.list_dir(home.view_id, &run)
                    .await?
                    .iter()
                    .any(|n| n.file_id == scope.file_id && !n.is_removed)
            );
            cases.push(
                serde_json::json!({"mode":mode,"original_root":scope.file_id,
                "after_delete":observe(&a,&session,&scope,&baseline.cursor).await}),
            );
        }
        println!("observed {mode}");
        std::fs::write(
            &args[2],
            serde_json::to_vec_pretty(
                &serde_json::json!({"completed":false,"scope":run,"cases":cases}),
            )?,
        )?;
    }
    std::fs::write(
        &args[2],
        serde_json::to_vec_pretty(&serde_json::json!({"completed":true,"scope":run,
        "cases":cases,"translated_batches_were_not_applied":true,"independent_native_sessions":true}))?,
    )?;
    Ok(())
}
