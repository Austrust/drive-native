//! Read-only native download reuse probe on the existing isolated lab fixture.
use serde_json::json;
use sproto::{
    Client, Error,
    actions::{native::NativeVersion, transfer::ContentVersion},
    client::{Config, Credentials},
};
fn config(password: &str) -> Config {
    Config::builder()
        .host("192.0.2.10")
        .tls_server_name("nas.example.invalid".to_string())
        .credentials(Credentials::Password {
            username: "test-user".into(),
            password: password.into(),
            otp: None,
        })
        .max_channels(1usize)
        .build()
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let password = rpassword::prompt_password("Test account password (not saved): ")?;
    let client = Client::connect(config(&password)).await?;
    let root = client.personal_root().await?;
    let nodes=client.list_dir(root.view_id,"/DriveNative-Lab/20261028T124720-1e31763a/service-134016b3-afcb-4d63-a130-db4023063ee8").await?;
    let n = nodes
        .iter()
        .find(|n| n.name == "A.doc")
        .ok_or("missing lab fixture")?;
    let expected = ContentVersion {
        file_id: n.file_id.clone(),
        version: NativeVersion {
            sync_id: n.sync_id,
            max_id: n.max_id,
        },
        size: n.file_size,
        hash: n.hash.clone(),
    };
    let parent = client
        .list_dir(root.view_id, "/DriveNative-Lab/20261028T124720-1e31763a")
        .await?;
    let folder = parent
        .iter()
        .find(|n| n.name == "service-134016b3-afcb-4d63-a130-db4023063ee8")
        .ok_or("missing lab folder")?;
    let watch = sproto::actions::native::Watch {
        view_id: root.view_id,
        node_id: folder.node_id,
        path:
            "/DriveNative-Lab/20261028T124720-1e31763a/service-134016b3-afcb-4d63-a130-db4023063ee8"
                .into(),
        cursor: sproto::actions::native::EventCursor(0),
    };
    let tmp = tempfile::tempdir()?;
    for mode in [
        "same_session",
        "fresh_session",
        "file_flag",
        "current_version",
        "scoped_current",
    ] {
        for iteration in 0..4 {
            let c = if mode != "fresh_session" {
                client.clone()
            } else {
                Client::connect(config(&password)).await?
            };
            let result = if mode == "scoped_current" {
                c.research_download_scoped(&expected, &tmp.path().join("content"), &watch)
                    .await
            } else if mode == "current_version" {
                c.research_download_current(&expected, &tmp.path().join("content"))
                    .await
            } else if mode == "file_flag" {
                c.research_download_file_flag(&expected, &tmp.path().join("content"))
                    .await
            } else {
                c.download_checked(&expected, &tmp.path().join("content"))
                    .await
            };
            let value = match result {
                Ok(_) => json!({"verified":true}),
                Err(Error::Server { code, reason }) => {
                    json!({"verified":false,"server_code":code,"known_repeat_reason":reason=="Repeated download request, are you download hash-mismatch?"})
                }
                Err(_) => json!({"verified":false,"other_error":true}),
            };
            println!(
                "{}",
                json!({"mode":mode,"iteration":iteration,"result":value})
            );
        }
    }
    Ok(())
}
