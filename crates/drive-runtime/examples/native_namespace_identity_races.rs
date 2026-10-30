//! Actual namespace core preflight followed by an independent NAS mutation.
use drive_core::{
    Store,
    namespace::RenameJob,
    native::{
        NativeDirectoryOutcome, NativeReadSession, NativeRemoveOutcome, NativeRenameOutcome, Scope,
    },
    removals::RemoveJob,
};
use sproto::{
    Client,
    actions::{
        list::NodeInfo,
        namespace::NamespaceTarget,
        namespace_observer::{PausedNamespace, with_namespace_observer},
        native::NativeVersion,
    },
    client::{Config, Credentials},
};
use std::{io::Write, path::Path};
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
async fn listed(c: &Client, view: u64, path: &str) -> Result<Vec<NodeInfo>> {
    match c.list_dir(view, path).await {
        Ok(n) => Ok(n),
        Err(sproto::Error::Server {
            code: 12291 | 32769,
            ..
        }) => Ok(vec![]),
        Err(e) => Err(e.into()),
    }
}
async fn at(c: &Client, view: u64, parent: &str, name: &str) -> Result<Option<NodeInfo>> {
    Ok(listed(c, view, parent)
        .await?
        .into_iter()
        .find(|n| n.name == name && !n.is_removed))
}
async fn move_path(c: &Client, view: u64, parent: &str, name: &str, to: &str) -> Result<()> {
    let n = at(c, view, parent, name)
        .await?
        .ok_or("missing move source")?;
    c.rename_native(
        &NamespaceTarget::from_node(view, format!("{parent}/{name}"), &n)?,
        to,
    )
    .await?;
    Ok(())
}
async fn upload(c: &Client, view: u64, path: &str, body: &Path) -> Result<()> {
    c.upload_snapshot(
        view,
        path,
        NativeVersion {
            sync_id: 0,
            max_id: 0,
        },
        body,
    )
    .await?;
    Ok(())
}
fn fingerprint(n: Option<&NodeInfo>) -> serde_json::Value {
    match n {
        None => serde_json::Value::Null,
        Some(n) => {
            serde_json::json!({"id":n.file_id,"sync":n.sync_id,"max":n.max_id,"hash":n.hash,"size":n.file_size,"kind":format!("{:?}",n.file_type)})
        }
    }
}
#[tokio::main]
async fn main() -> Result<()> {
    assert_ne!(unsafe { libc::geteuid() }, 0);
    drive_runtime::disable_core_dumps()?;
    let args: Vec<_> = std::env::args().collect();
    let protected = args.get(3).is_some_and(|x| x == "--expect-mkdir-protected");
    if !(args.len() == 3 || (args.len() == 4 && protected))
        || args[1] != "/DriveNative-Lab/20261028T124720-1e31763a"
    {
        return Err(
            "expected authorized lab, new output and optional --expect-mkdir-protected".into(),
        );
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
    let home = a.personal_root().await?;
    assert!(home.writable);
    let view = home.view_id;
    let run = format!("{}/namespace-races-{}", args[1], uuid::Uuid::new_v4());
    a.mkdir_native(view, &run).await?;
    println!("test_subtree={run}");
    let scratch = tempfile::tempdir()?;
    let source = scratch.path().join("original");
    let foreign = scratch.path().join("foreign");
    std::fs::write(&source, b"original identity body")?;
    std::fs::write(&foreign, b"independent foreign body")?;
    let mut cases = vec![];
    for mode in [
        "mkdir-parent-replaced",
        "mkdir-parent-deleted",
        "rename-parent-replaced",
        "rename-source-replaced",
        "remove-source-replaced",
        "remove-directory-child-added",
    ] {
        let root = format!("{run}/{mode}");
        let r = a.mkdir_native(view, &root).await?;
        let parent_path = format!("{root}/Parent");
        let parent = a.mkdir_native(view, &parent_path).await?;
        let mkdir = mode.starts_with("mkdir");
        let rename = mode.starts_with("rename");
        let remove_dir = mode == "remove-directory-child-added";
        if !mkdir && !remove_dir {
            upload(&a, view, &format!("{root}/A.txt"), &source).await?;
        }
        let scope = Scope {
            view_id: view,
            node_id: r.node_id,
            file_id: r.file_id,
            path: root.clone(),
        };
        let session = NativeReadSession::new(a.clone(), scope.clone())?;
        let snap = session.recovery_snapshot().await?;
        let data = scratch.path().join(mode);
        let mut store = Store::open(&data)?;
        session.bind(&mut store)?;
        store.install_remote_snapshot(&snap.nodes, &snap.cursor, &snap.restore)?;
        store.ensure_fs_root(&scope.file_id)?;
        let mut directory_job = None;
        let mut rename_job = None;
        let mut remove_job = None;
        let mut child_id = None;
        if mkdir {
            let d = store.create_local_directory(&parent.file_id, "NewDir", 0o755)?;
            let child = store.create_local_file(&d.id, "draft.txt", 0o644)?;
            store.save_local(&child.id, b"unuploaded child retained")?;
            store.creation_saved(&child.id)?;
            child_id = Some(child.id);
            let q = store.next_directory()?.ok_or("directory queue absent")?;
            let prepared = store.mark_directory_started(&q.job.operation)?;
            let NativeDirectoryOutcome::Prepared(origin) =
                session.create_directory(&prepared, false).await?
            else {
                return Err("directory preparation missing".into());
            };
            store.prepare_directory(&q.job.operation, origin)?;
            directory_job = Some(store.mark_directory_started(&q.job.operation)?);
        } else if rename {
            let node = store.lookup(&scope.file_id, "A.txt")?.node;
            let dest_parent = if mode == "rename-parent-replaced" {
                parent.file_id.clone()
            } else {
                scope.file_id.clone()
            };
            rename_job = Some(RenameJob {
                operation: uuid::Uuid::new_v4().to_string(),
                node,
                source: "A.txt".into(),
                destination: if mode == "rename-parent-replaced" {
                    "Parent/B.txt"
                } else {
                    "B.txt"
                }
                .into(),
                parent: dest_parent.clone(),
                parent_node: store.node(&dest_parent)?,
                name: "B.txt".into(),
                cursor: store.cursor()?,
            });
        } else {
            let name = if remove_dir { "Parent" } else { "A.txt" };
            remove_job = Some(RemoveJob {
                operation: uuid::Uuid::new_v4().to_string(),
                node: store.lookup(&scope.file_id, name)?.node,
                path: name.into(),
                cursor: store.cursor()?,
            });
        }
        let (tx, mut rx) = tokio::sync::mpsc::channel::<PausedNamespace>(1);
        let observer = b.clone();
        let root2 = root.clone();
        let foreign2 = foreign.clone();
        let race = tokio::spawn(async move {
            let paused = rx.recv().await.ok_or("namespace did not reach barrier")?;
            let point = &paused.boundary;
            let boundary = serde_json::json!({"action":point.action,"view":point.view,"path":point.path,"source":point.source,"destination":point.destination});
            let mutation = async {
                match mode {
                    "mkdir-parent-replaced" | "rename-parent-replaced" => {
                        move_path(
                            &observer,
                            view,
                            &root2,
                            "Parent",
                            &format!("{root2}/Parent-moved"),
                        )
                        .await?;
                        observer
                            .mkdir_native(view, &format!("{root2}/Parent"))
                            .await?;
                    }
                    "mkdir-parent-deleted" => {
                        let n = at(&observer, view, &root2, "Parent")
                            .await?
                            .ok_or("parent absent")?;
                        observer
                            .remove_native(&NamespaceTarget::from_node(
                                view,
                                format!("{root2}/Parent"),
                                &n,
                            )?)
                            .await?;
                    }
                    "rename-source-replaced" | "remove-source-replaced" => {
                        move_path(
                            &observer,
                            view,
                            &root2,
                            "A.txt",
                            &format!("{root2}/Moved.txt"),
                        )
                        .await?;
                        upload(&observer, view, &format!("{root2}/A.txt"), &foreign2).await?;
                    }
                    "remove-directory-child-added" => {
                        upload(
                            &observer,
                            view,
                            &format!("{root2}/Parent/Foreign.txt"),
                            &foreign2,
                        )
                        .await?;
                    }
                    _ => unreachable!(),
                };
                Ok::<_, Box<dyn std::error::Error + Send + Sync>>(())
            }
            .await;
            match mutation {
                Ok(()) => {
                    let protected = if mode.ends_with("source-replaced") {
                        serde_json::json!([
                            fingerprint(at(&observer, view, &root2, "A.txt").await?.as_ref()),
                            fingerprint(at(&observer, view, &root2, "Moved.txt").await?.as_ref())
                        ])
                    } else if mode == "remove-directory-child-added" {
                        fingerprint(
                            at(&observer, view, &format!("{root2}/Parent"), "Foreign.txt")
                                .await?
                                .as_ref(),
                        )
                    } else {
                        serde_json::Value::Null
                    };
                    paused
                        .resume
                        .send(Ok(()))
                        .map_err(|_| "caller left barrier")?;
                    Ok((boundary, protected))
                }
                Err(e) => {
                    let _ = paused
                        .resume
                        .send(Err(sproto::Error::Decode("namespace race failed".into())));
                    Err(e)
                }
            }
        });
        let outcome = with_namespace_observer(tx, async {
            if let Some(job) = &directory_job {
                let value = session.create_directory(job, false).await;
                match value {
                    Ok(NativeDirectoryOutcome::Created { node, remote }) => {
                        store.confirm_directory(&job.operation, &node, &remote)?;
                        Ok::<String, Box<dyn std::error::Error + Send + Sync>>("created".into())
                    }
                    Ok(NativeDirectoryOutcome::Conflict) => {
                        store.conflict_directory(&job.operation)?;
                        Ok("conflict".into())
                    }
                    v => {
                        store.mark_directory_uncertain(&job.operation)?;
                        Ok(match v {
                            Err(e) => format!("error:{e}"),
                            _ => "uncertain".into(),
                        })
                    }
                }
            } else if let Some(job) = &rename_job {
                Ok(match session.rename(job, false).await {
                    Ok(NativeRenameOutcome::Confirmed { .. }) => "confirmed".into(),
                    Ok(NativeRenameOutcome::Conflict) => "conflict".into(),
                    Ok(NativeRenameOutcome::Uncertain) => "uncertain".into(),
                    Ok(_) => "refresh".into(),
                    Err(e) => format!("error:{e}"),
                })
            } else {
                Ok(
                    match session.remove(remove_job.as_ref().unwrap(), false).await {
                        Ok(NativeRemoveOutcome::Confirmed { .. }) => "confirmed".into(),
                        Ok(NativeRemoveOutcome::Conflict) => "conflict".into(),
                        Ok(NativeRemoveOutcome::Uncertain) => "uncertain".into(),
                        Err(e) => format!("error:{e}"),
                    },
                )
            }
        })
        .await?;
        let (boundary, protected_before) = race.await??;
        let replacement_new = at(
            &b,
            view,
            &parent_path,
            if mkdir { "NewDir" } else { "B.txt" },
        )
        .await?;
        let original_new = at(
            &b,
            view,
            &format!("{root}/Parent-moved"),
            if mkdir { "NewDir" } else { "B.txt" },
        )
        .await?;
        let replacement_a = at(&b, view, &root, "A.txt").await?;
        let original_a = at(&b, view, &root, "Moved.txt").await?;
        let child = at(&b, view, &parent_path, "Foreign.txt").await?;
        let protected_after = if mode.ends_with("source-replaced") {
            serde_json::json!([
                fingerprint(replacement_a.as_ref()),
                fingerprint(original_a.as_ref())
            ])
        } else if remove_dir {
            fingerprint(child.as_ref())
        } else {
            serde_json::Value::Null
        };
        assert_eq!(
            protected_before, protected_after,
            "remote identity/version/content metadata changed"
        );
        let mut reconciled = "not-required".to_owned();
        let child_preserved = if let Some(id) = child_id {
            drop(store);
            let mut s = Store::open(&data)?;
            if let Some(q) = s.next_directory()? {
                if q.state == "uncertain" {
                    reconciled = match session.create_directory(&q.job, true).await {
                        Ok(NativeDirectoryOutcome::Conflict) => {
                            s.conflict_directory(&q.job.operation)?;
                            "conflict".into()
                        }
                        Ok(NativeDirectoryOutcome::Uncertain) => "uncertain".into(),
                        Err(e) => format!("error:{e}"),
                        _ => return Err("raced directory incorrectly adopted after reopen".into()),
                    };
                }
            }
            let expected = b"unuploaded child retained";
            let mut body = vec![0; expected.len()];
            let n = s.open_cached(&id)?.read_at(&mut body, 0)?;
            n == expected.len() && body == expected
        } else {
            true
        };
        let entry = serde_json::json!({"mode":mode,"outcome":outcome,"readonly_reconciliation":reconciled,"protected_before":protected_before,"protected_after":protected_after,"wire_boundary":boundary,"replacement_received_new":replacement_new.is_some(),"original_received_new":original_new.is_some(),"source_replacement_id":replacement_a.as_ref().map(|n|&n.file_id),"original_moved_id":original_a.as_ref().map(|n|&n.file_id),"foreign_child_preserved":child.is_some(),"local_child_preserved_after_restart":child_preserved});
        println!("observed {mode}: {outcome}");
        cases.push(entry);
        std::fs::write(
            &args[2],
            serde_json::to_vec_pretty(
                &serde_json::json!({"scope":run,"completed":false,"cases":cases,"race_after_actual_preflight":true}),
            )?,
        )?;
        assert!(child_preserved);
        if mode.ends_with("source-replaced") {
            assert!(replacement_a.is_some() && original_a.is_some());
            assert_eq!(outcome, "conflict");
        }
        if remove_dir {
            assert!(child.is_some());
            assert_eq!(outcome, "conflict");
        }
        if protected && mode == "mkdir-parent-deleted" {
            assert_eq!(outcome, "conflict");
        }
        if protected && mode == "mkdir-parent-replaced" {
            assert!(replacement_new.is_none(), "replacement parent was mutated");
            assert_ne!(outcome, "created");
            assert_ne!(outcome, "confirmed");
        }
    }
    std::fs::write(
        &args[2],
        serde_json::to_vec_pretty(
            &serde_json::json!({"scope":run,"completed":true,"cases":cases,"mkdir_protected_assertions":protected,"race_after_actual_preflight":true,"fuse_test":false}),
        )?,
    )?;
    Ok(())
}
