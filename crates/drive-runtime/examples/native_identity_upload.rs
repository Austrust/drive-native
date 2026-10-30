//! Explicit Windows identity-address experiments in a fresh authorized lab tree.
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
#[tokio::main]
async fn main() -> Result<()> {
    assert_ne!(unsafe { libc::geteuid() }, 0);
    drive_runtime::disable_core_dumps()?;
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 3 || args[1] != "/DriveNative-Lab/20261028T124720-1e31763a" {
        return Err("expected authorized lab and new result".into());
    }
    std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&args[2])?
        .write_all(b"{\"passed\":false}")?;
    let password = rpassword::prompt_password("Test account password (not saved): ")?;
    let a = connect(&password).await?;
    let b = connect(&password).await?;
    drop(password);
    let home = a.personal_root().await?;
    assert!(home.writable);
    let run = format!("{}/identity-upload-{}", args[1], uuid::Uuid::new_v4());
    a.mkdir_native(home.view_id, &run).await?;
    println!("test_subtree={run}");
    let original = format!("{run}/Original");
    let moved = format!("{run}/Moved");
    let parent = a.mkdir_native(home.view_id, &original).await?;
    let temp = tempfile::tempdir()?;
    let source = temp.path().join("source");
    std::fs::write(&source, b"identity-anchored initial content")?;
    let first = a
        .upload_snapshot_at_identity(
            &parent.file_id,
            Some("A.txt"),
            NativeVersion {
                sync_id: 0,
                max_id: 0,
            },
            &source,
        )
        .await?;
    let child = b
        .list_dir(home.view_id, &original)
        .await?
        .into_iter()
        .find(|n| n.name == "A.txt")
        .ok_or("missing child")?;
    assert_eq!(first.file_id, child.file_id);
    let current = b
        .list_dir(home.view_id, &run)
        .await?
        .into_iter()
        .find(|n| n.file_id == parent.file_id)
        .ok_or("missing root")?;
    b.rename_native(
        &NamespaceTarget::from_node(home.view_id, original.clone(), &current)?,
        &moved,
    )
    .await?;
    let replacement = b.mkdir_native(home.view_id, &original).await?;
    assert_ne!(replacement.file_id, parent.file_id);
    std::fs::write(&source, b"content after original root moved")?;
    let second = a
        .upload_snapshot_at_identity(
            &parent.file_id,
            Some("B.txt"),
            NativeVersion {
                sync_id: 0,
                max_id: 0,
            },
            &source,
        )
        .await?;
    let updated = a
        .upload_snapshot_at_identity(&child.file_id, None, first.version, &source)
        .await?;
    assert_eq!(updated.file_id, child.file_id);
    let after = b.list_dir(home.view_id, &moved).await?;
    assert!(
        after
            .iter()
            .any(|n| n.file_id == second.file_id && n.name == "B.txt")
    );
    assert!(after.iter().any(|n| n.file_id == updated.file_id
        && n.name == "A.txt"
        && n.sync_id == updated.version.sync_id));
    assert!(b.list_dir(home.view_id, &original).await?.is_empty());
    let content = b
        .download_checked(&updated, &temp.path().join("verified"))
        .await?;
    assert_eq!(
        content.sha256,
        drive_core::sha256(b"content after original root moved")
    );
    let current = b
        .list_dir(home.view_id, &run)
        .await?
        .into_iter()
        .find(|n| n.file_id == parent.file_id)
        .ok_or("moved parent missing")?;
    b.remove_native(&NamespaceTarget::from_node(
        home.view_id,
        moved.clone(),
        &current,
    )?)
    .await?;
    let parent_error = match a
        .upload_snapshot_at_identity(
            &parent.file_id,
            Some("AfterDelete.txt"),
            NativeVersion {
                sync_id: 0,
                max_id: 0,
            },
            &source,
        )
        .await
    {
        Ok(r) => {
            serde_json::json!({"accepted":true,"file_id":r.file_id,"sync_id":r.version.sync_id})
        }
        Err(e) => serde_json::json!({"accepted":false,"error":format!("{e}")}),
    };
    let file_error = match a
        .upload_snapshot_at_identity(&child.file_id, None, updated.version, &source)
        .await
    {
        Ok(r) => {
            serde_json::json!({"accepted":true,"file_id":r.file_id,"sync_id":r.version.sync_id})
        }
        Err(e) => serde_json::json!({"accepted":false,"error":format!("{e}")}),
    };
    assert!(b.list_dir(home.view_id, &original).await?.is_empty());
    let old_parent_live = b
        .list_dir(home.view_id, &run)
        .await?
        .iter()
        .any(|n| n.file_id == parent.file_id && !n.is_removed);
    std::fs::write(
        &args[2],
        serde_json::to_vec_pretty(&serde_json::json!({"passed":true,"scope":run,
        "view_id":home.view_id,"original_parent":parent.file_id,"replacement_parent":replacement.file_id,
        "initial_child":first.file_id,"child_after_move":second.file_id,"updated_child":updated.file_id,
        "full_content_verified":true,"replacement_untouched":true,"deleted_parent_error":parent_error,
        "deleted_file_error":file_error,"old_parent_live_after_deleted_id_requests":old_parent_live,"method":"native upload path id:<parent>/<leaf> and id:<file>; independent native session verifies"}))?,
    )?;
    println!("PASS live identity upload and replacement isolation; deleted-ID responses recorded");
    Ok(())
}
