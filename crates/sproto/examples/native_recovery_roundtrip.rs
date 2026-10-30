//! Live fault injection. Only writes random files under the supplied lab run.
use md4::{Digest, Md4};
use sha2::Sha256;
use sproto::{
    Client,
    actions::{
        mutations::{UploadIntent, UploadReconciliation},
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
        .max_channels(3usize)
        .build()
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    let lab = args.get(1).ok_or("expected lab run")?;
    let parts: Vec<_> = lab.split('/').collect();
    if parts.len() != 3
        || parts[1] != "DriveNative-Lab"
        || parts[2].is_empty()
        || parts[2].contains("..")
        || lab.contains('\0')
    {
        return Err("invalid lab run".into());
    }
    let password = rpassword::prompt_password("Test account password (not saved): ")?;
    let writer = Client::connect(config(password.clone())).await?;
    let home = writer.personal_root().await?;
    if !home.writable || !home.download_allowed {
        return Err("insufficient test account permissions".into());
    }
    let roots = writer.list_dir(home.view_id, "/DriveNative-Lab").await?;
    let root = roots
        .iter()
        .find(|n| n.name == parts[2])
        .ok_or("lab root missing")?;
    let local = tempfile::tempdir()?;
    let snapshot = local.path().join("snapshot");
    let path = format!("{lab}/recovery-{}.doc", uuid::Uuid::new_v4());
    std::fs::write(&snapshot, b"initial recovery fixture\n")?;
    let initial = writer
        .upload_snapshot(
            home.view_id,
            &path,
            NativeVersion {
                sync_id: 0,
                max_id: 0,
            },
            &snapshot,
        )
        .await?;
    let mut watch = Watch {
        view_id: home.view_id,
        node_id: root.node_id,
        path: lab.clone(),
        cursor: EventCursor(0),
    };
    let initial_events = writer.pull_events(&watch).await?;
    if initial_events.recovery_required {
        return Err("initial recovery signal".into());
    }
    std::fs::write(
        local.path().join("cursor"),
        initial_events.next.0.to_string(),
    )?;
    let desired = b"complete content whose upload acknowledgement is deliberately lost\n";
    std::fs::write(&snapshot, desired)?;
    let intent = UploadIntent {
        view_id: home.view_id,
        path: path.clone(),
        original_file_id: Some(initial.file_id.clone()),
        base: initial.version,
        size: desired.len() as u64,
        md4: format!("{:x}", Md4::digest(desired)),
        sha256: format!("{:x}", Sha256::digest(desired)),
    };
    let lost = writer
        .research_upload_drop_receipt(home.view_id, &path, initial.version, &snapshot)
        .await;
    assert!(matches!(lost, Err(sproto::Error::Timeout(_))));
    drop(writer);
    println!("upload_connection_closed_before_receipt");
    let observer = Client::connect(config(password.clone())).await?;
    // Wait only for this uncertain mutation's desired metadata, never resend it.
    tokio::time::timeout(Duration::from_secs(30), async {
        loop {
            let entries = observer.list_dir(home.view_id, lab).await?;
            if entries
                .iter()
                .any(|n| n.file_id == initial.file_id && n.hash == intent.md4)
            {
                return Ok::<_, sproto::Error>(());
            }
            tokio::time::sleep(Duration::from_millis(200)).await;
        }
    })
    .await??;
    let current = match observer.reconcile_upload(&intent, local.path()).await? {
        UploadReconciliation::DesiredStatePresent(v) => v,
        other => return Err(format!("lost receipt was not reconciled: {other:?}").into()),
    };
    watch.cursor = EventCursor(std::fs::read_to_string(local.path().join("cursor"))?.parse()?);
    let batch = observer.pull_events(&watch).await?;
    assert!(!batch.recovery_required);
    assert!(
        batch
            .events
            .iter()
            .any(|e| e.file_id == initial.file_id && e.version == current.version)
    );
    watch.cursor = batch.next;
    let quiet = observer.pull_events(&watch).await?;
    assert!(quiet.events.is_empty());
    assert_eq!(quiet.next, watch.cursor);
    let mut invalid = watch.clone();
    invalid.cursor = EventCursor(u64::MAX / 2);
    assert!(matches!(
        observer.pull_events(&invalid).await,
        Err(sproto::Error::CursorReset { .. })
    ));
    // A later independent update must prevent the old uncertain operation from
    // being accepted or retried, even though it once committed successfully.
    let other = Client::connect(config(password)).await?;
    std::fs::write(
        &snapshot,
        b"later independent edit must survive reconciliation\n",
    )?;
    let later = other
        .upload_snapshot(home.view_id, &path, current.version, &snapshot)
        .await?;
    assert!(matches!(
        observer.reconcile_upload(&intent, local.path()).await?,
        UploadReconciliation::Diverged(_)
    ));
    let actual = other
        .list_dir(home.view_id, lab)
        .await?
        .into_iter()
        .find(|n| n.file_id == later.file_id)
        .ok_or("missing later edit")?;
    assert_eq!(actual.hash, later.hash);
    assert_eq!(actual.sync_id, later.version.sync_id);
    let result = serde_json::json!({"passed":true,"lost_upload_receipt_injected":true,"reconciled_with_full_sha256_download":true,"original_file_identity_preserved":true,"independent_later_edit_preserved":true,"reconnected_from_persisted_cursor":true,"duplicate_pull_empty":true,"future_cursor_rejected":true,"replayed_mutations":0,"lab_path":lab,"fuse_tested":false});
    if let Some(report) = args.get(2) {
        std::fs::write(report, serde_json::to_vec_pretty(&result)?)?;
    }
    println!("{result}");
    Ok(())
}
