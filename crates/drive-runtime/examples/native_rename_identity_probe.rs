//! Native rename identity grammar: experiment, not product behavior.
use sproto::{
    Client, PObject,
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
    assert_ne!(unsafe { libc::geteuid() }, 0);
    drive_runtime::disable_core_dumps()?;
    let args: Vec<_> = std::env::args().collect();
    let destination_only = args.get(3).is_some_and(|a| a == "--destination-identity");
    let include_view = destination_only || args.get(3).is_some_and(|a| a == "--with-view");
    if !(args.len() == 3 || (args.len() == 4 && include_view))
        || args[1] != "/DriveNative-Lab/20261028T124720-1e31763a"
    {
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
    drop(password);
    let home = a.personal_root().await?;
    assert!(home.writable);
    let view = home.view_id;
    let run = format!("{}/rename-identity-{}", args[1], uuid::Uuid::new_v4());
    a.mkdir_native(view, &run).await?;
    println!("test_subtree={run}");
    let temp = tempfile::tempdir()?;
    let local = temp.path().join("source");
    let foreign = temp.path().join("foreign");
    let body = b"original body for identity rename";
    let foreign_body = b"independent content must survive";
    std::fs::write(&local, body)?;
    std::fs::write(&foreign, foreign_body)?;
    let mut results = vec![];
    for mode in [
        "normal-file",
        "normal-directory",
        "parent-moved-and-replaced",
        "parent-deleted",
        "parent-deleted-and-replaced",
        "target-occupied",
        "source-renamed-and-replaced",
        "source-deleted",
        "source-updated",
        "directory-child-added",
    ] {
        let root = format!("{run}/{mode}");
        a.mkdir_native(view, &root).await?;
        let dest = format!("{root}/Parent");
        let parent = a.mkdir_native(view, &dest).await?;
        let directory = mode == "normal-directory" || mode == "directory-child-added";
        if directory {
            a.mkdir_native(view, &format!("{root}/A")).await?;
        } else {
            create(&a, view, &format!("{root}/A"), &local).await?;
        }
        let original = at(&a, view, &root, "A").await?.ok_or("fixture A missing")?;
        let target = NamespaceTarget::from_node(view, format!("{root}/A"), &original)?;
        match mode {
            "parent-moved-and-replaced" => {
                rename(&b, view, &root, "Parent", &format!("{root}/Parent-moved")).await?;
                b.mkdir_native(view, &dest).await?;
            }
            "parent-deleted" | "parent-deleted-and-replaced" => {
                let n = at(&b, view, &root, "Parent")
                    .await?
                    .ok_or("parent missing")?;
                b.remove_native(&NamespaceTarget::from_node(view, dest.clone(), &n)?)
                    .await?;
                if mode == "parent-deleted-and-replaced" {
                    b.mkdir_native(view, &dest).await?;
                    create(&b, view, &format!("{dest}/B"), &foreign).await?;
                }
            }
            "target-occupied" => create(&b, view, &format!("{dest}/B"), &foreign).await?,
            "source-renamed-and-replaced" => {
                rename(&b, view, &root, "A", &format!("{root}/Moved")).await?;
                create(&b, view, &format!("{root}/A"), &foreign).await?;
            }
            "source-deleted" => {
                b.remove_native(&target).await?;
            }
            "source-updated" => {
                b.upload_snapshot(view, &format!("{root}/A"), target.base, &foreign)
                    .await?;
            }
            "directory-child-added" => {
                create(&b, view, &format!("{root}/A/Foreign"), &foreign).await?
            }
            _ => {}
        }
        let protected_before = match mode {
            "source-renamed-and-replaced" | "source-updated" => at(&b, view, &root, "A").await?,
            "target-occupied" | "parent-deleted-and-replaced" => at(&b, view, &dest, "B").await?,
            "directory-child-added" => at(&b, view, &format!("{root}/A"), "Foreign").await?,
            _ => None,
        };
        let response = match a
            .research_rename_identity(
                &target,
                &parent.file_id,
                "B",
                include_view,
                !destination_only,
            )
            .await
        {
            Ok(r) => {
                let mut d = serde_json::Map::new();
                for k in ["path", "name", "file_id", "parent_id", "file_type"] {
                    d.insert(
                        k.into(),
                        r.get(k)
                            .and_then(PObject::as_str)
                            .map(serde_json::Value::from)
                            .unwrap_or(serde_json::Value::Null),
                    );
                }
                for k in ["sync_id", "max_id", "node_id", "is_removed"] {
                    d.insert(
                        k.into(),
                        r.get(k)
                            .and_then(PObject::as_int)
                            .map(serde_json::Value::from)
                            .unwrap_or(serde_json::Value::Null),
                    );
                }
                serde_json::json!({"accepted":true,"receipt":d})
            }
            Err(e) => serde_json::json!({"accepted":false,"error":e.to_string()}),
        };
        let current = at(&b, view, &root, "A").await?;
        let new = at(&b, view, &dest, "B").await?;
        let moved = at(&b, view, &format!("{root}/Parent-moved"), "B").await?;
        let protected_after = match mode {
            "source-renamed-and-replaced" | "source-updated" => current.clone(),
            "target-occupied" | "parent-deleted-and-replaced" => new.clone(),
            "directory-child-added" => at(&b, view, &format!("{root}/A"), "Foreign").await?,
            _ => None,
        };
        let content_ok = if !directory {
            let matched = new
                .as_ref()
                .or(moved.as_ref())
                .filter(|n| n.file_id == original.file_id);
            if let Some(n) = matched {
                verify_body(
                    &b,
                    n,
                    if mode == "source-updated" {
                        foreign_body.as_slice()
                    } else {
                        body.as_slice()
                    },
                    temp.path(),
                )
                .await?;
                true
            } else {
                false
            }
        } else {
            false
        };
        let entry = serde_json::json!({"case":mode,"response":response,"source_after":summary(current.as_ref()),"expected_source_id":original.file_id,"destination_after":summary(new.as_ref()),"original_parent_destination":summary(moved.as_ref()),"protected_before":summary(protected_before.as_ref()),"protected_after":summary(protected_after.as_ref()),"original_content_verified_at_destination":content_ok});
        println!("observed {mode}: {}", response);
        results.push(entry);
        std::fs::write(
            &args[2],
            serde_json::to_vec_pretty(
                &serde_json::json!({"completed":false,"scope":run,"cases":results,"normal_core_used":false,"request":{"src_path":if destination_only {"<absolute-source>"}else{"id:<source>"},"dst_path":"id:<parent>/B","view_id":if include_view {serde_json::json!(view)}else{serde_json::Value::Null}}}),
            )?,
        )?;
        if let Some(n) = protected_after.as_ref() {
            verify_body(&b, n, foreign_body, temp.path()).await?;
        }
        assert_eq!(
            summary(protected_before.as_ref()),
            summary(protected_after.as_ref()),
            "protected remote metadata changed"
        );
        if mode == "normal-file" && response["accepted"] == false {
            break;
        }
    }
    std::fs::write(
        &args[2],
        serde_json::to_vec_pretty(
            &serde_json::json!({"completed":true,"scope":run,"cases":results,"normal_core_used":false,"request":{"src_path":if destination_only {"<absolute-source>"}else{"id:<source>"},"dst_path":"id:<parent>/B","view_id":if include_view {serde_json::json!(view)}else{serde_json::Value::Null}}}),
        )?,
    )?;
    Ok(())
}
