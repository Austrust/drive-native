//! Race after real core preflight, before the actual native request is sent.
use drive_core::{
    Store,
    native::{NativeReadSession, NativeWriteOutcome, Scope},
};
use sproto::{
    Client,
    actions::{
        namespace::NamespaceTarget,
        native::NativeVersion,
        transfer::ContentVersion,
        upload_observer::{PausedUpload, with_upload_observer},
    },
    client::{Config, Credentials},
};
use std::io::Write;
type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
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
async fn contents(
    client: &Client,
    view: u64,
    parent: &str,
    name: &str,
    scratch: &std::path::Path,
) -> Result<Option<Vec<u8>>> {
    let nodes = match client.list_dir(view, parent).await {
        Ok(n) => n,
        Err(sproto::Error::Server { code: 12291, .. }) => return Ok(None),
        Err(e) => return Err(e.into()),
    };
    let Some(n) = nodes.iter().find(|n| n.name == name && !n.is_removed) else {
        return Ok(None);
    };
    let version = ContentVersion {
        file_id: n.file_id.clone(),
        version: NativeVersion {
            sync_id: n.sync_id,
            max_id: n.max_id,
        },
        size: n.file_size,
        hash: n.hash.clone(),
    };
    let target = scratch.join(format!("check-{}", uuid::Uuid::new_v4()));
    client.download_checked(&version, &target).await?;
    Ok(Some(std::fs::read(target)?))
}
#[tokio::main]
async fn main() -> Result<()> {
    assert_ne!(unsafe { libc::geteuid() }, 0);
    drive_runtime::disable_core_dumps()?;
    let args: Vec<_> = std::env::args().collect();
    let protected = args.get(3).is_some_and(|s| s == "--expect-protected");
    if !(args.len() == 3 || (args.len() == 4 && protected))
        || args[1] != "/DriveNative-Lab/20261028T124720-1e31763a"
    {
        return Err("expected authorized lab, new result, optional --expect-protected".into());
    }
    std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&args[2])?
        .write_all(b"{\"completed\":false}")?;
    let password = rpassword::prompt_password("Test account password (not saved): ")?;
    let author = connect(&password).await?;
    let observer = connect(&password).await?;
    drop(password);
    let home = author.personal_root().await?;
    assert!(home.writable);
    let run = format!("{}/upload-races-{}", args[1], uuid::Uuid::new_v4());
    author.mkdir_native(home.view_id, &run).await?;
    println!("test_subtree={run}");
    let scratch = tempfile::tempdir()?;
    let source = scratch.path().join("payload");
    let foreign = scratch.path().join("foreign");
    let payload = b"local immutable edit awaiting native confirmation";
    let foreign_body = b"independent replacement must stay unchanged";
    let hash = drive_core::sha256(payload);
    std::fs::write(&source, payload)?;
    std::fs::write(&foreign, foreign_body)?;
    let mut cases = Vec::new();
    for mode in [
        "nested-parent-replaced",
        "root-replaced",
        "parent-deleted",
        "existing-file-replaced",
        "new-file-name-reused",
        "parent-deleted-and-replaced",
    ] {
        let root = format!("{run}/{mode}");
        let receipt = author.mkdir_native(home.view_id, &root).await?;
        let scope = Scope {
            view_id: home.view_id,
            node_id: receipt.node_id,
            file_id: receipt.file_id,
            path: root.clone(),
        };
        let nested = mode == "nested-parent-replaced" || mode.starts_with("parent-deleted");
        let parent_path = if nested {
            format!("{root}/Parent")
        } else {
            root.clone()
        };
        let parent_file = if nested {
            author
                .mkdir_native(home.view_id, &parent_path)
                .await?
                .file_id
        } else {
            scope.file_id.clone()
        };
        let name = "A.txt";
        let path = format!("{parent_path}/{name}");
        let existing = mode == "existing-file-replaced";
        if existing {
            author
                .upload_snapshot(
                    home.view_id,
                    &path,
                    NativeVersion {
                        sync_id: 0,
                        max_id: 0,
                    },
                    &foreign,
                )
                .await?;
        }
        let session = NativeReadSession::new(author.clone(), scope.clone())?;
        let snapshot = session.recovery_snapshot().await?;
        let data = scratch.path().join(mode);
        let mut store = Store::open(&data)?;
        session.bind(&mut store)?;
        store.install_remote_snapshot(&snapshot.nodes, &snapshot.cursor, &snapshot.restore)?;
        store.ensure_fs_root(&scope.file_id)?;
        let node = if existing {
            store.lookup(&parent_file, name)?.node
        } else {
            store.create_local_file(&parent_file, name, 0o644)?
        };
        let op = store.save_local(&node.id, payload)?;
        if !existing {
            store.creation_saved(&node.id)?;
        }
        let origin = if existing {
            None
        } else {
            let o = session
                .prepare_upload_creation(
                    &node,
                    &store.node(&parent_file)?,
                    &path,
                    &hash,
                    &store.cursor()?,
                )
                .await?
                .ok_or("origin not prepared")?;
            store.prepare_upload_origin(&op, &o, &hash)?;
            Some(o)
        };
        store.mark_upload_started(&op)?;
        let (tx, mut rx) = tokio::sync::mpsc::channel::<PausedUpload>(1);
        let b = observer.clone();
        let root_copy = root.clone();
        let run_copy = run.clone();
        let parent_copy = parent_path.clone();
        let parent_id = parent_file.clone();
        let foreign_copy = foreign.clone();
        let task = tokio::spawn(async move {
            let pause = rx.recv().await.ok_or("upload never reached wire barrier")?;
            let boundary =
                serde_json::json!({"view":pause.boundary.view,"path":pause.boundary.path});
            let mutation = async {
                if mode == "existing-file-replaced" {
                    let n = b
                        .list_dir(home.view_id, &root_copy)
                        .await?
                        .into_iter()
                        .find(|n| n.name == "A.txt")
                        .ok_or("source file absent")?;
                    b.rename_native(
                        &NamespaceTarget::from_node(
                            home.view_id,
                            format!("{root_copy}/A.txt"),
                            &n,
                        )?,
                        &format!("{root_copy}/Moved.txt"),
                    )
                    .await?;
                    b.upload_snapshot(
                        home.view_id,
                        &format!("{root_copy}/A.txt"),
                        NativeVersion {
                            sync_id: 0,
                            max_id: 0,
                        },
                        &foreign_copy,
                    )
                    .await?;
                } else if mode=="new-file-name-reused" {
                    b.upload_snapshot(home.view_id,&format!("{root_copy}/A.txt"),NativeVersion{sync_id:0,max_id:0},&foreign_copy).await?;
                } else {
                    let enclosing = if nested { &root_copy } else { &run_copy };
                    let n = b
                        .list_dir(home.view_id, enclosing)
                        .await?
                        .into_iter()
                        .find(|n| n.file_id == parent_id)
                        .ok_or("parent absent")?;
                    let target = NamespaceTarget::from_node(home.view_id, parent_copy.clone(), &n)?;
                    if mode.starts_with("parent-deleted") {
                        b.remove_native(&target).await?;
                        if mode=="parent-deleted-and-replaced" {
                            b.mkdir_native(home.view_id,&parent_copy).await?;
                            b.upload_snapshot(home.view_id,&format!("{parent_copy}/A.txt"),NativeVersion{sync_id:0,max_id:0},&foreign_copy).await?;
                        }
                    } else {
                        b.rename_native(&target, &format!("{parent_copy}-moved"))
                            .await?;
                        b.mkdir_native(home.view_id, &parent_copy).await?;
                    }
                }
                Ok::<_, Box<dyn std::error::Error + Send + Sync>>(())
            }
            .await;
            match mutation {
                Ok(()) => {
                    pause
                        .resume
                        .send(Ok(()))
                        .map_err(|_| "upload left barrier")?;
                    Ok(boundary)
                }
                Err(e) => {
                    let _ = pause
                        .resume
                        .send(Err(sproto::Error::Decode("test mutation failed".into())));
                    Err(e)
                }
            }
        });
        let result = with_upload_observer(
            tx,
            session.upload_with_origin(
                &node,
                &path,
                &source,
                &hash,
                scratch.path(),
                origin.as_ref(),
            ),
        )
        .await;
        let boundary = task.await??;
        let outcome = match result {
            Ok(NativeWriteOutcome::Created { node, remote }) => {
                store.confirm_upload_identity(&op, &node.version, Some(&remote))?;
                "created".to_owned()
            }
            Ok(NativeWriteOutcome::Confirmed(node)) => {
                store.confirm_upload(&op, &node.version)?;
                "confirmed".to_owned()
            }
            Ok(NativeWriteOutcome::Conflict) => {
                store.conflict_upload(&op)?;
                "conflict".to_owned()
            }
            Ok(NativeWriteOutcome::Uncertain) => {
                store.mark_upload_uncertain(&op)?;
                "uncertain".to_owned()
            }
            Err(e) => {
                store.mark_upload_uncertain(&op)?;
                format!("error:{e}")
            }
        };
        let replacement =
            contents(&observer, home.view_id, &parent_path, name, scratch.path()).await?;
        let replacement_has_payload = replacement.as_deref() == Some(payload.as_slice());
        let foreign_unchanged =
            !(existing || mode=="new-file-name-reused" || mode=="parent-deleted-and-replaced") || replacement.as_deref() == Some(foreign_body.as_slice());
        let moved_parent = if existing {
            parent_path.clone()
        } else {
            format!("{parent_path}-moved")
        };
        let moved_name = if existing { "Moved.txt" } else { name };
        let original_has_payload = if mode.starts_with("parent-deleted") || mode=="new-file-name-reused" {
            false
        } else {
            contents(
                &observer,
                home.view_id,
                &moved_parent,
                moved_name,
                scratch.path(),
            )
            .await?
            .as_deref()
                == Some(payload.as_slice())
        };
        let reconciled = if store.upload_job(&op)?.state=="uncertain" {
            match session.reconcile_with_origin(&node,&path,&source,&hash,scratch.path(),origin.as_ref()).await {
                Ok(NativeWriteOutcome::Conflict)=>{store.conflict_upload(&op)?;"conflict".to_owned()},
                Ok(NativeWriteOutcome::Uncertain)=>"uncertain".to_owned(),
                Ok(_)=>return Err("a raced upload was incorrectly accepted by reconciliation".into()),
                Err(e)=>format!("error:{e}"),
            }
        } else {"not-required".into()};
        drop(store);
        let store = Store::open(&data)?;
        let mut body = vec![0; payload.len()];
        let n = store.open_cached(&node.id)?.read_at(&mut body, 0)?;
        let draft_preserved = n == payload.len() && body == payload;
        let journal = store.upload_job(&op)?.state;
        let live_parent = if nested {
            observer.list_dir(home.view_id,&root).await?.into_iter().find(|n|n.name=="Parent" && !n.is_removed).map(|n|serde_json::json!({"file_id":n.file_id,"node_id":n.node_id,"same_identity":n.file_id==parent_file}))
        } else {
            None
        };
        cases.push(serde_json::json!({"mode":mode,"wire_boundary":boundary,"outcome":outcome,"reconciled":reconciled,"journal":journal,
            "replacement_has_payload":replacement_has_payload,"live_parent_after_race":live_parent,"foreign_unchanged":foreign_unchanged,"original_has_payload":original_has_payload,"draft_preserved_after_restart":draft_preserved}));
        std::fs::write(
            &args[2],
            serde_json::to_vec_pretty(
                &serde_json::json!({"completed":false,"scope":run,"cases":cases}),
            )?,
        )?;
        if protected {
            if mode != "parent-deleted" {
                assert!(!replacement_has_payload);
            } else {
                assert_eq!(
                    outcome, "conflict",
                    "a restored parent must not be silently committed"
                );
            }
            assert!(foreign_unchanged);
            assert!(draft_preserved);
            assert!(!matches!(outcome.as_str(), "created" | "confirmed"));
            assert!(matches!(journal.as_str(), "uncertain" | "conflict"));
        }
        println!("observed {mode}: {outcome}");
        std::fs::write(
            &args[2],
            serde_json::to_vec_pretty(
                &serde_json::json!({"completed":false,"scope":run,"cases":cases}),
            )?,
        )?;
    }
    std::fs::write(
        &args[2],
        serde_json::to_vec_pretty(
            &serde_json::json!({"completed":true,"scope":run,"cases":cases,"protected_assertions":protected,"race_after_actual_preflight":true,"ordinary_processes_not_observed":true}),
        )?,
    )?;
    Ok(())
}
