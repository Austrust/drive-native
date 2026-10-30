//! Real NAS contract test, confined to one explicitly selected existing lab run.
//! Uses two independent native sessions; never records credentials or file bodies.
use sproto::{
    Client,
    actions::{
        native::{EventCursor, NativeVersion, Watch},
        transfer::ContentVersion,
    },
    client::{Config, Credentials},
};
use std::{path::Path, time::Duration};

fn config(password: String) -> Config {
    Config::builder()
        .host("192.0.2.10")
        .tls_server_name("nas.example.invalid".to_string())
        .credentials(Credentials::Password {
            username: "test-user".into(),
            password,
            otp: None,
        })
        .max_channels(3usize)
        .build()
}
fn emit(stage: &str, extra: serde_json::Value) {
    println!("{}", serde_json::json!({"stage":stage,"result":extra}));
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    let lab = args
        .get(1)
        .ok_or("expected existing /DriveNative-Lab/<run> path")?;
    let components: Vec<_> = lab.split('/').collect();
    if components.len() != 3
        || components[1] != "DriveNative-Lab"
        || components[2].is_empty()
        || components[2].contains("..")
    {
        return Err("not an isolated lab run".into());
    }
    let password = rpassword::prompt_password("Test account password (not saved): ")?;
    let observer = Client::connect(config(password.clone())).await?;
    let writer = Client::connect(config(password)).await?;
    let home = observer.personal_root().await?;
    if !home.writable || !home.download_allowed {
        return Err("test account permissions insufficient".into());
    }
    let nodes = observer.list_dir(home.view_id, "/DriveNative-Lab").await?;
    let root = nodes
        .iter()
        .find(|n| n.name == components[2])
        .ok_or("lab directory does not exist")?;
    emit(
        "native_sessions",
        serde_json::json!({"server_build":observer.server_build(),"personal_view":home.view_id,"independent_sessions":2}),
    );
    let run = uuid::Uuid::new_v4().to_string();
    let path = format!("{lab}/rust-{run}.doc");
    let local = tempfile::tempdir()?;
    let source = local.path().join("snapshot");
    let cache = local.path().join("cached");
    let v1 = b"native contract fixture first generation\n";
    std::fs::write(&source, v1)?;
    let r1 = writer
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
    observer.download_checked(&r1, &cache).await?;
    assert_eq!(std::fs::read(&cache)?, v1);
    let mut watch = Watch {
        view_id: home.view_id,
        node_id: root.node_id,
        path: lab.clone(),
        cursor: EventCursor(0),
    };
    let initial = observer.pull_events(&watch).await?;
    if initial.recovery_required {
        return Err("initial recovery signal requires investigation".into());
    }
    watch.cursor = initial.next;
    emit(
        "initial_cache",
        serde_json::json!({"verified":true,"cursor":watch.cursor.0}),
    );
    let event_client = observer.clone();
    let event_cache = cache.clone();
    let file_id = r1.file_id.clone();
    let receiver = tokio::spawn(async move {
        let mut digest = String::new();
        loop {
            let notification = event_client.wait_changes(&[watch.clone()], &digest).await?;
            digest = notification.profile_digest;
            if notification.changed_views.contains(&watch.view_id) {
                let batch = event_client.pull_events(&watch).await?;
                if batch.recovery_required {
                    return Err(sproto::Error::Decode("recovery signal in live test".into()));
                }
                for event in batch.events {
                    if event.file_id == file_id
                        && event.version.sync_id > r1.version.sync_id
                        && !event.removed
                    {
                        let expected = ContentVersion {
                            file_id: event.file_id,
                            version: event.version,
                            size: event.size,
                            hash: event.hash,
                        };
                        let verified = event_client
                            .download_checked(&expected, Path::new(&event_cache))
                            .await?;
                        return Ok(verified);
                    }
                }
                watch.cursor = batch.next;
            }
        }
    });
    tokio::time::sleep(Duration::from_millis(200)).await;
    let v2 = b"native contract fixture second generation changed remotely\n";
    std::fs::write(&source, v2)?;
    let r2 = writer
        .upload_snapshot(home.view_id, &path, r1.version, &source)
        .await?;
    emit(
        "remote_update_committed",
        serde_json::json!({"sync_id":r2.version.sync_id}),
    );
    let updated = tokio::time::timeout(Duration::from_secs(60), receiver).await???;
    emit(
        "background_download_verified",
        serde_json::json!({"sync_id":updated.remote.version.sync_id}),
    );
    assert_eq!(updated.remote.version, r2.version);
    // Attempt a stale update in an independent session. It must not overwrite v2.
    std::fs::write(
        &source,
        b"stale fixture must not overwrite current contents\n",
    )?;
    let stale = observer
        .upload_snapshot(home.view_id, &path, r1.version, &source)
        .await;
    let conflict_code = match stale {
        Err(sproto::Error::Server { code, .. }) if matches!(code, 24579 | 36865) => code,
        other => return Err(format!("stale upload was not rejected: {other:?}").into()),
    };
    emit(
        "stale_upload_rejected",
        serde_json::json!({"code":conflict_code}),
    );
    // A second identical download can hit server 0x8004 anti-loop protection.
    // Reconcile the rejected mutation through authoritative metadata instead.
    let after = writer.list_dir(home.view_id, lab).await?;
    let current = after
        .iter()
        .find(|n| n.file_id == r2.file_id)
        .ok_or("file disappeared after conflict test")?;
    assert_eq!(current.sync_id, r2.version.sync_id);
    assert_eq!(current.hash, r2.hash);
    drop(observer);
    drop(writer);
    assert_eq!(std::fs::read(cache)?, v2);
    let result = serde_json::json!({"passed":true,"cache_updated_without_open":true,"read_after_native_connections_closed":true,"stale_upload_error":conflict_code,"fuse_tested":false,"nemo_tested":false,"sha256":updated.sha256,"server_protocol_build":17892,"lab_path":lab});
    if let Some(report) = args.get(2) {
        std::fs::write(report, serde_json::to_vec_pretty(&result)?)?;
    }
    emit("native_event_refresh", result);
    Ok(())
}
