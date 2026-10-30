//! Explicit live test: native notifications -> durable inbox -> verified blobs.
//! Mutations are restricted to an existing user-selected isolated lab run.
use drive_core::{
    DownloadTicket, Status, Store,
    native::{NativeReadSession, Scope},
};
use sproto::{
    Client,
    actions::native::NativeVersion,
    client::{Config, Credentials},
};
use std::{io::Read, time::Duration};

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
fn read(store: &Store, id: &str) -> std::io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    store
        .open_cached(id)
        .map_err(std::io::Error::other)?
        .read_to_end(&mut bytes)?;
    Ok(bytes)
}
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    let lab = args
        .get(1)
        .ok_or("expected existing /DriveNative-Lab/<run>")?;
    let parts: Vec<_> = lab.split('/').collect();
    if parts.len() != 3
        || parts[1] != "DriveNative-Lab"
        || parts[2].is_empty()
        || parts[2].contains("..")
        || lab.contains('\0')
    {
        return Err("invalid isolated lab path".into());
    }
    let password = rpassword::prompt_password("Test account password (not saved): ")?;
    let observer = Client::connect(config(password.clone())).await?;
    let writer = Client::connect(config(password)).await?;
    let home = observer.personal_root().await?;
    if !home.writable || !home.download_allowed {
        return Err("insufficient personal Drive permissions".into());
    }
    let roots = observer.list_dir(home.view_id, "/DriveNative-Lab").await?;
    let root = roots
        .iter()
        .find(|n| n.name == parts[2])
        .ok_or("lab root missing")?;
    let scope = Scope {
        view_id: home.view_id,
        node_id: root.node_id,
        file_id: root.file_id.clone(),
        path: lab.clone(),
    };
    let session = NativeReadSession::new(observer, scope)?;
    let scratch = tempfile::tempdir()?;
    let state = scratch.path().join("state");
    let source = scratch.path().join("source");
    let mut store = Store::open(&state)?;
    session.bind(&mut store)?;
    let remote = format!("{lab}/state-{}.doc", uuid::Uuid::new_v4());
    let old = b"durable native state first generation\n";
    let new = b"durable native state updated by independent session\n";
    std::fs::write(&source, old)?;
    let first = writer
        .upload_snapshot(
            home.view_id,
            &remote,
            NativeVersion {
                sync_id: 0,
                max_id: 0,
            },
            &source,
        )
        .await?;
    let initial = session
        .pull(&store.cursor()?)
        .await?
        .ok_or("missing bootstrap events")?;
    store.receive(&initial)?;
    // Simulate fsd restart after durable receipt but before application.
    drop(store);
    let mut store = Store::open(&state)?;
    session.bind(&mut store)?;
    store.drain()?;
    let node = store.node(&first.file_id)?;
    let mut content = session.download(&node, scratch.path()).await?;
    store.publish_download_reader(
        &DownloadTicket {
            id: node.id.clone(),
            version: node.version,
        },
        &mut content.file,
        content.size,
        &content.sha256,
    )?;
    assert_eq!(read(&store, &first.file_id)?, old);
    assert!(!store.is_pinned(&first.file_id)?);
    let old_handle = store.open_cached(&first.file_id)?;
    let cursor = store.cursor()?;
    println!("initial_cache_persisted");
    let receiver = async {
        let mut cursor = cursor;
        let mut digest = String::new();
        loop {
            let notification = session.wait(&cursor, &digest).await?;
            digest = notification.profile_digest;
            if notification.changed_views.contains(&home.view_id) {
                if let Some(batch) = session.pull(&cursor).await? {
                    store.receive(&batch)?;
                    store.drain()?;
                    cursor = store.cursor()?;
                    let tickets = store.downloads_needed()?;
                    for ticket in tickets {
                        let node = store.node(&ticket.id)?;
                        let mut content = session.download(&node, scratch.path()).await?;
                        store.publish_download_reader(
                            &ticket,
                            &mut content.file,
                            content.size,
                            &content.sha256,
                        )?;
                        if ticket.id == first.file_id {
                            return Ok::<_, drive_core::Error>(());
                        }
                    }
                }
            }
        }
    };
    let mutation = async {
        tokio::time::sleep(Duration::from_millis(250)).await;
        std::fs::write(&source, new)?;
        let committed = writer
            .upload_snapshot(home.view_id, &remote, first.version, &source)
            .await?;
        Ok::<_, Box<dyn std::error::Error>>(committed)
    };
    let (refresh, commit) = tokio::join!(
        tokio::time::timeout(Duration::from_secs(60), receiver),
        mutation
    );
    refresh??;
    commit?;
    assert_eq!(store.status(&first.file_id)?, Status::LocalComplete);
    let mut old_handle = old_handle;
    let mut held = Vec::new();
    old_handle.read_to_end(&mut held)?;
    assert_eq!(held, old);
    drop(old_handle);
    drop(session);
    drop(writer);
    drop(store);
    // No live network sessions remain; reconstruct local state from disk.
    let store = Store::open(&state)?;
    assert_eq!(read(&store, &first.file_id)?, new);
    assert!(!store.is_pinned(&first.file_id)?);
    let result = serde_json::json!({"passed":true,"native_events_to_durable_state":true,"inbox_recovered_after_restart":true,"unpinned_background_refresh":true,"open_handle_retained_old_version":true,"offline_read_after_restart":true,"native_connections_closed_before_offline_read":true,"fuse_tested":false,"nemo_tested":false,"lab_path":lab});
    if let Some(report) = args.get(2) {
        std::fs::write(report, serde_json::to_vec_pretty(&result)?)?;
    }
    println!("{result}");
    Ok(())
}
