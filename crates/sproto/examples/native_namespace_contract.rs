//! Destructive test operations are confined to a fresh UUID subtree of the lab.
use sproto::{
    Client,
    actions::{
        list::NodeInfo,
        namespace::{NamespaceTarget, RemoveReconciliation, RenameReconciliation},
        native::{EventCursor, NativeVersion, Watch},
    },
    client::{Config, Credentials},
};
use std::time::Duration;
fn config(password: String) -> Config {
    Config::builder()
        .host("192.0.2.10")
        .tls_server_name("nas.example.invalid".to_owned())
        .credentials(Credentials::Password {
            username: "test-user".into(),
            password,
            otp: None,
        })
        .build()
}
fn code<T>(r: &sproto::Result<T>) -> u32 {
    match r {
        Ok(_) => 0,
        Err(sproto::Error::Server { code, .. }) => *code,
        Err(_) => u32::MAX,
    }
}
async fn find(
    client: &Client,
    view: u64,
    parent: &str,
    name: &str,
) -> Result<NodeInfo, Box<dyn std::error::Error>> {
    let mut nodes = client
        .list_dir(view, parent)
        .await?
        .into_iter()
        .filter(|n| n.name == name && !n.is_removed);
    let n = nodes.next().ok_or("test object missing")?;
    assert!(nodes.next().is_none());
    Ok(n)
}
fn target(view: u64, path: &str, n: &NodeInfo) -> NamespaceTarget {
    NamespaceTarget::from_node(view, path.into(), n).unwrap()
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    let lab = args.get(1).ok_or("expected authorized lab")?;
    let parts: Vec<_> = lab.split('/').collect();
    if parts.len() != 3
        || parts[0] != ""
        || parts[1] != "DriveNative-Lab"
        || parts[2].is_empty()
        || parts[2].contains("..")
        || lab.contains('\0')
    {
        return Err("invalid lab".into());
    }
    let password = rpassword::prompt_password("Test account password (not saved): ")?;
    let a = Client::connect(config(password.clone())).await?;
    let b = Client::connect(config(password)).await?;
    let home = a.personal_root().await?;
    assert!(home.writable);
    find(&a, home.view_id, "/DriveNative-Lab", parts[2]).await?;
    let root = format!("{lab}/namespace-{}", uuid::Uuid::new_v4());
    println!("test_subtree={root}");
    let dir = a.mkdir_native(home.view_id, &root).await?;
    let same_dir = b.mkdir_native(home.view_id, &root).await;
    let repeated_mkdir_code = code(&same_dir);
    let current_dir = find(&a, home.view_id, lab, root.rsplit('/').next().unwrap()).await?;
    assert_eq!(current_dir.file_id, dir.file_id);
    assert_eq!(current_dir.sync_id, dir.version.sync_id);
    if let Ok(repeated) = same_dir {
        assert_eq!(repeated.file_id, dir.file_id);
    }
    println!("mkdir and repeated directory checked: {repeated_mkdir_code}");
    let staging = tempfile::tempdir()?;
    let snapshot = staging.path().join("snapshot");
    std::fs::write(&snapshot, b"namespace writer a v1\n")?;
    let zero = NativeVersion {
        sync_id: 0,
        max_id: 0,
    };
    let path_a = format!("{root}/A.doc");
    let path_b = format!("{root}/B.doc");
    let path_c = format!("{root}/C.doc");
    let path_d = format!("{root}/D.doc");
    a.upload_snapshot(home.view_id, &path_a, zero, &snapshot)
        .await?;
    std::fs::write(&snapshot, b"independent existing destination\n")?;
    let original_b = b
        .upload_snapshot(home.view_id, &path_b, zero, &snapshot)
        .await?;
    let original_a = find(&a, home.view_id, &root, "A.doc").await?;
    let old = target(home.view_id, &path_a, &original_a);
    let collision = a.rename_native(&old, &path_b).await;
    let collision_code = code(&collision);
    assert_eq!(collision_code, 36872);
    let unchanged_b = find(&b, home.view_id, &root, "B.doc").await?;
    assert_eq!(unchanged_b.file_id, original_b.file_id);
    assert_eq!(unchanged_b.hash, original_b.hash);
    assert_eq!(unchanged_b.sync_id, original_b.version.sync_id);
    std::fs::write(&snapshot, b"namespace writer b newer version\n")?;
    b.upload_snapshot(home.view_id, &path_a, old.base, &snapshot)
        .await?;
    let stale_rename = a.rename_native(&old, &path_c).await;
    let stale_rename_code = code(&stale_rename);
    assert_ne!(
        stale_rename_code, 0,
        "stale rename must not move newer remote content"
    );
    let stale_remove = a.remove_native(&old).await;
    let stale_remove_code = code(&stale_remove);
    assert_eq!(stale_remove_code, 40962);
    let fresh = find(&a, home.view_id, &root, "A.doc").await?;
    let moved = a
        .rename_native(&target(home.view_id, &path_a, &fresh), &path_c)
        .await?;
    assert_eq!(moved.file_id, original_a.file_id);
    assert_eq!(moved.hash, fresh.hash);
    let rename_intent = target(home.view_id, &path_c, &moved);
    assert!(matches!(
        a.research_rename_discard_ack(&rename_intent, &path_d).await,
        Err(sproto::Error::Timeout(_))
    ));
    let mut lost_rename = None;
    for attempt in 0..30 {
        let observed = b.reconcile_rename(&rename_intent, &path_d).await?;
        if attempt == 0 || attempt == 29 {
            println!("rename_reconciliation={observed:?}");
        }
        if let RenameReconciliation::DesiredStateObserved(n) = observed {
            lost_rename = Some(n);
            break;
        }
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
    let renamed = lost_rename.ok_or("lost rename did not converge without replay")?;
    assert_eq!(renamed.file_id, original_a.file_id);
    assert_eq!(renamed.hash, fresh.hash);
    println!("rename collision/stale/lost reply checked: {collision_code}/{stale_rename_code}");
    let mut watch = Watch {
        view_id: home.view_id,
        node_id: dir.node_id,
        path: root.clone(),
        cursor: EventCursor(0),
    };
    watch.cursor = b.pull_events(&watch).await?.next;
    let remove_intent = target(home.view_id, &path_d, &renamed);
    assert!(matches!(
        a.research_remove_discard_ack(&remove_intent).await,
        Err(sproto::Error::Timeout(_))
    ));
    let mut removed = false;
    for _ in 0..30 {
        if matches!(
            b.reconcile_remove(&remove_intent, &watch).await?,
            RemoveReconciliation::TombstoneObserved {
                replacement: None,
                ..
            }
        ) {
            removed = true;
            break;
        }
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
    assert!(
        removed,
        "lost remove needs tombstone evidence without replay"
    );
    std::fs::write(&snapshot, b"new identity reusing deleted path\n")?;
    let replacement = b
        .upload_snapshot(home.view_id, &path_d, zero, &snapshot)
        .await?;
    assert_ne!(replacement.file_id, remove_intent.file_id);
    let replacement_preserved = matches!(a.reconcile_remove(&remove_intent,&watch).await?,
        RemoveReconciliation::TombstoneObserved{replacement:Some(n),..} if n.file_id==replacement.file_id);
    assert!(replacement_preserved);
    // Explicitly test server's own stale base protection against path reuse.
    let reused_delete = a.remove_native(&remove_intent).await;
    let reused_delete_code = code(&reused_delete);
    assert_ne!(reused_delete_code, 0);
    let preserved = find(&b, home.view_id, &root, "D.doc").await?;
    assert_eq!(preserved.file_id, replacement.file_id);
    assert_eq!(preserved.hash, replacement.hash);
    println!("lost delete, tombstone and replacement protection checked: {reused_delete_code}");
    // Parent removal behavior must be measured before exposing recursive delete.
    let parent = format!("{root}/directory-version");
    let parent_receipt = a.mkdir_native(home.view_id, &parent).await?;
    let parent_node = find(&a, home.view_id, &root, "directory-version").await?;
    let child = format!("{parent}/new-child.doc");
    b.upload_snapshot(home.view_id, &child, zero, &snapshot)
        .await?;
    let parent_after_child = find(&a, home.view_id, &root, "directory-version").await?;
    let removed_parent = a
        .remove_native(&target(home.view_id, &parent, &parent_node))
        .await;
    let directory_delete_code = code(&removed_parent);
    let parent_exists = a
        .list_dir(home.view_id, &root)
        .await?
        .iter()
        .any(|n| n.file_id == parent_receipt.file_id);
    if directory_delete_code == 0 {
        assert!(!parent_exists);
    } else {
        assert!(parent_exists);
        find(&b, home.view_id, &parent, "new-child.doc").await?;
    }
    let result = serde_json::json!({"passed":true,"lab_path":lab,"test_subtree":root,
        "two_independent_sessions":true,"fault_boundary":"server_reply_received_but_coordinator_ack_discarded","repeated_mkdir_code":repeated_mkdir_code,
        "rename_destination_exists_code":collision_code,"stale_rename_code":stale_rename_code,
        "stale_remove_code":stale_remove_code,"lost_rename_reconciled_without_replay":true,
        "lost_remove_confirmed_by_tombstone":true,"replacement_preserved":replacement_preserved,
        "reused_path_stale_delete_code":reused_delete_code,
        "parent_version_before_child":[parent_node.sync_id,parent_node.max_id],
        "parent_version_after_child":[parent_after_child.sync_id,parent_after_child.max_id],
        "directory_delete_after_child_code":directory_delete_code,"directory_still_exists":parent_exists});
    if let Some(out) = args.get(2) {
        std::fs::write(out, serde_json::to_vec_pretty(&result)?)?;
    }
    println!("{result}");
    Ok(())
}
