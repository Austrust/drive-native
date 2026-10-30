//! Native identity remove: conditional behavior and independent event evidence.
use sproto::{
    Client,
    actions::{
        list::NodeInfo, namespace::NamespaceTarget, native::NativeVersion, transfer::ContentVersion,
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
async fn at(c: &Client, view: u64, parent: &str, name: &str) -> Result<Option<NodeInfo>> {
    let nodes = match c.list_dir(view, parent).await {
        Ok(n) => n,
        Err(sproto::Error::Server {
            code: 12291 | 32769,
            ..
        }) => return Ok(None),
        Err(e) => return Err(e.into()),
    };
    Ok(nodes.into_iter().find(|n| n.name == name && !n.is_removed))
}
fn summary(n: Option<&NodeInfo>) -> serde_json::Value {
    match n {
        None => serde_json::Value::Null,
        Some(n) => {
            serde_json::json!({"id":n.file_id,"sync":n.sync_id,"max":n.max_id,"hash":n.hash,"size":n.file_size,"type":format!("{:?}",n.file_type)})
        }
    }
}
async fn rename(c: &Client, view: u64, parent: &str, name: &str, to: &str) -> Result<()> {
    let n = at(c, view, parent, name)
        .await?
        .ok_or("fixture source missing")?;
    c.rename_native(
        &NamespaceTarget::from_node(view, format!("{parent}/{name}"), &n)?,
        to,
    )
    .await?;
    Ok(())
}
async fn create(c: &Client, view: u64, path: &str, body: &Path) -> Result<()> {
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
async fn verify_body(c: &Client, n: &NodeInfo, expected: &[u8], temp: &Path) -> Result<()> {
    let v = ContentVersion {
        file_id: n.file_id.clone(),
        version: NativeVersion {
            sync_id: n.sync_id,
            max_id: n.max_id,
        },
        size: n.file_size,
        hash: n.hash.clone(),
    };
    let out = temp.join(uuid::Uuid::new_v4().to_string());
    c.download_checked(&v, &out).await?;
    assert_eq!(std::fs::read(out)?, expected);
    Ok(())
}
#[tokio::main]
async fn main() -> Result<()> {
    use sproto::actions::{
        list::FileType,
        namespace::RemoveReconciliation,
        native::{EventCursor, EventKind, Watch},
    };
    assert_ne!(unsafe { libc::geteuid() }, 0);
    drive_runtime::disable_core_dumps()?;
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 3 || args[1] != "/DriveNative-Lab/20261028T124720-1e31763a" {
        return Err("expected authorized lab and new output".into());
    }
    std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&args[2])?
        .write_all(b"{\"completed\":false}")?;
    let password = rpassword::prompt_password("Test account password (not saved): ")?;
    let a = connect(&password).await?;
    let b = connect(&password).await?;
    let home = a.personal_root().await?;
    assert!(home.writable);
    let view = home.view_id;
    let run = format!("{}/remove-identity-{}", args[1], uuid::Uuid::new_v4());
    a.mkdir_native(view, &run).await?;
    println!("test_subtree={run}");
    let temp = tempfile::tempdir()?;
    let local = temp.path().join("source");
    let foreign = temp.path().join("foreign");
    let body = b"original identity remove fixture";
    let other = b"independent content must survive";
    std::fs::write(&local, body)?;
    std::fs::write(&foreign, other)?;
    let mut results = vec![];
    for mode in [
        "normal-file",
        "normal-directory",
        "renamed-and-replaced",
        "deleted-and-replaced",
        "updated",
        "directory-child-added",
        "parent-moved-and-replaced",
        "repeat-after-confirmed",
        "unread-reply",
    ] {
        let root = format!("{run}/{mode}");
        a.mkdir_native(view, &root).await?;
        let parent = format!("{root}/Parent");
        a.mkdir_native(view, &parent).await?;
        let path = format!("{parent}/A");
        let directory = matches!(mode, "normal-directory" | "directory-child-added");
        if directory {
            a.mkdir_native(view, &path).await?;
        } else {
            create(&a, view, &path, &local).await?;
        }
        let original = at(&a, view, &parent, "A").await?.ok_or("source missing")?;
        let target = NamespaceTarget::from_node(view, path.clone(), &original)?;
        let (rparent, rname) = root.rsplit_once('/').ok_or("invalid run root")?;
        let root_node = at(&a, view, rparent, rname)
            .await?
            .ok_or("watch root missing")?;
        let mut watch = Watch {
            view_id: view,
            node_id: root_node.node_id,
            path: root.clone(),
            cursor: EventCursor(0),
        };
        loop {
            let events = a.pull_events(&watch).await?;
            assert!(!events.recovery_required);
            if events.next == watch.cursor {
                break;
            }
            watch.cursor = events.next;
        }
        match mode {
            "renamed-and-replaced" => {
                rename(&b, view, &parent, "A", &format!("{parent}/Moved")).await?;
                create(&b, view, &path, &foreign).await?;
            }
            "deleted-and-replaced" => {
                b.remove_native(&target).await?;
                create(&b, view, &path, &foreign).await?;
            }
            "updated" => {
                b.upload_snapshot(view, &path, target.base, &foreign)
                    .await?;
            }
            "directory-child-added" => {
                create(&b, view, &format!("{path}/Foreign"), &foreign).await?;
            }
            "parent-moved-and-replaced" => {
                rename(&b, view, &root, "Parent", &format!("{root}/Parent-moved")).await?;
                b.mkdir_native(view, &parent).await?;
                create(&b, view, &path, &foreign).await?;
            }
            _ => {}
        }
        let (protected_parent, protected_name) = if mode == "directory-child-added" {
            (path.as_str(), "Foreign")
        } else {
            (parent.as_str(), "A")
        };
        let protected = matches!(
            mode,
            "renamed-and-replaced"
                | "deleted-and-replaced"
                | "updated"
                | "directory-child-added"
                | "parent-moved-and-replaced"
        );
        let before = if protected {
            at(&b, view, protected_parent, protected_name).await?
        } else {
            None
        };
        let response = if mode == "unread-reply" {
            let observed = async {
                loop {
                    if matches!(
                        b.reconcile_remove(&target, &watch).await?,
                        RemoveReconciliation::TombstoneObserved { .. }
                    ) {
                        break;
                    }
                    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                }
                Ok(())
            };
            let r = a
                .research_remove_identity_unread_after(&target, observed)
                .await;
            assert!(matches!(
                r,
                Err(sproto::Error::Timeout(
                    "injected unread identity remove reply"
                ))
            ));
            serde_json::json!({"accepted":false,"reply_deliberately_unread":true,"independent_commit_observed":true})
        } else {
            match a.remove_at_identity(&target).await {
                Ok(_) => serde_json::json!({"accepted":true}),
                Err(e) => serde_json::json!({"accepted":false,"error":e.to_string()}),
            }
        };
        let current = at(&b, view, &parent, "A").await?;
        let moved = at(&b, view, &parent, "Moved").await?;
        let moved_parent = at(&b, view, &format!("{root}/Parent-moved"), "A").await?;
        let after = if protected {
            at(&b, view, protected_parent, protected_name).await?
        } else {
            None
        };
        if protected {
            assert_eq!(
                summary(before.as_ref()),
                summary(after.as_ref()),
                "protected metadata changed: {mode}"
            );
            verify_body(
                &b,
                after.as_ref().ok_or("protected entry removed")?,
                other,
                temp.path(),
            )
            .await?;
        }
        let mut events = vec![];
        loop {
            let batch = b.pull_events(&watch).await?;
            assert!(!batch.recovery_required);
            let next = batch.next;
            events.extend(batch.events);
            if next == watch.cursor {
                break;
            }
            watch.cursor = next;
        }
        let tombstone = events.iter().any(|e| {
            e.kind == EventKind::Remove
                && e.file_id == original.file_id
                && e.version.sync_id > target.base.sync_id
        });
        let accepted = response["accepted"] == true;
        if matches!(
            mode,
            "normal-file" | "normal-directory" | "repeat-after-confirmed"
        ) {
            assert!(
                accepted && current.is_none() && tombstone,
                "ordinary remove must be confirmed"
            );
        }
        if matches!(
            mode,
            "updated" | "directory-child-added" | "renamed-and-replaced"
        ) {
            assert!(!accepted, "stale remove must be rejected");
        }
        if mode == "renamed-and-replaced" {
            assert!(
                moved
                    .as_ref()
                    .is_some_and(|n| n.file_id == original.file_id)
            );
            verify_body(&b, moved.as_ref().unwrap(), body, temp.path()).await?;
        }
        let retry = if mode == "repeat-after-confirmed" {
            create(&b, view, &path, &foreign).await?;
            let replacement = at(&b, view, &parent, "A")
                .await?
                .ok_or("replacement missing")?;
            let r = a.remove_at_identity(&target).await;
            let remaining = at(&b, view, &parent, "A")
                .await?
                .ok_or("retry deleted replacement")?;
            assert_eq!(summary(Some(&replacement)), summary(Some(&remaining)));
            verify_body(&b, &remaining, other, temp.path()).await?;
            Some(
                serde_json::json!({"accepted":r.is_ok(),"error":r.err().map(|e|e.to_string()),"replacement_preserved":true}),
            )
        } else {
            None
        };
        let mut rewind = watch.clone();
        rewind.cursor = EventCursor(target.base.sync_id);
        let recovered = if mode == "unread-reply" {
            Some(connect(&password).await?)
        } else {
            None
        };
        let verifier = recovered.as_ref().unwrap_or(&b);
        let reconciliation = match verifier.reconcile_remove(&target, &rewind).await? {
            RemoveReconciliation::TombstoneObserved { .. } => "tombstone-observed",
            RemoveReconciliation::StillPresent(_) => "still-present",
            RemoveReconciliation::AwaitingEvidence => "awaiting-evidence",
            RemoveReconciliation::HistoryGap => "history-gap",
            RemoveReconciliation::ChangedDuringVerification => "changed-during-verification",
        };
        if matches!(mode, "parent-moved-and-replaced" | "unread-reply") {
            assert_eq!(reconciliation, "tombstone-observed");
            assert!(tombstone);
        }
        results.push(serde_json::json!({"mode":mode,"request":{"action":"remove","path":format!("id:{}",original.file_id),"sync_id":target.base.sync_id,"max_id":target.base.max_id,"view_id_omitted":true},"response":response,"current":summary(current.as_ref()),"moved_source":summary(moved.as_ref()),"source_under_moved_parent":summary(moved_parent.as_ref()),"protected_metadata_unchanged":if protected {Some(true)} else {None},"post_base_tombstone_observed":tombstone,"reconciliation":reconciliation,"repeat":retry}));
        std::fs::write(
            &args[2],
            serde_json::to_vec_pretty(
                &serde_json::json!({"completed":false,"scope":run,"view_id":view,"cases":results}),
            )?,
        )?;
        println!("{mode}: observed; protected content checks passed");
        assert_eq!(directory, original.file_type == FileType::Dir);
    }
    std::fs::write(
        &args[2],
        serde_json::to_vec_pretty(
            &serde_json::json!({"completed":true,"scope":run,"server":"Drive Server 4.0.3-27892","view_id":view,"cases":results}),
        )?,
    )?;
    Ok(())
}
