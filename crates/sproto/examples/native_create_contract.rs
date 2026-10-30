//! Explicit native creation race test; mutations confined to the authorized lab.
use sproto::{
    Client,
    actions::native::NativeVersion,
    client::{Config, Credentials},
};
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
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    let lab = args.get(1).ok_or("expected isolated lab")?;
    let parts: Vec<_> = lab.split('/').collect();
    if parts.len() != 3
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
    if !home.writable {
        return Err("permission denied".into());
    }
    if !a
        .list_dir(home.view_id, "/DriveNative-Lab")
        .await?
        .iter()
        .any(|n| n.name == parts[2])
    {
        return Err("lab missing".into());
    }
    let dir = tempfile::tempdir()?;
    let source_a = dir.path().join("a");
    let source_b = dir.path().join("b");
    let body_a = b"creation race writer a\n";
    let body_b = b"creation race writer b\n";
    std::fs::write(&source_a, body_a)?;
    std::fs::write(&source_b, body_b)?;
    let zero = NativeVersion {
        sync_id: 0,
        max_id: 0,
    };
    let name = format!("create-race-{}.doc", uuid::Uuid::new_v4());
    let path = format!("{lab}/{name}");
    let (ra, rb) = tokio::join!(
        a.upload_snapshot(home.view_id, &path, zero, &source_a),
        b.upload_snapshot(home.view_id, &path, zero, &source_b)
    );
    let code = |r: &sproto::Result<sproto::actions::transfer::ContentVersion>| match r {
        Ok(_) => 0,
        Err(sproto::Error::Server { code, .. }) => *code,
        Err(_) => u32::MAX,
    };
    let codes = [code(&ra), code(&rb)];
    println!("concurrent_create_codes={codes:?}");
    assert_eq!(
        codes.iter().filter(|&&c| c == 0).count(),
        1,
        "creation must not overwrite another successful create"
    );
    let (winner, body) = match (ra, rb) {
        (Ok(v), Err(_)) => (v, body_a.as_slice()),
        (Err(_), Ok(v)) => (v, body_b.as_slice()),
        _ => unreachable!(),
    };
    let nodes = a.list_dir(home.view_id, lab).await?;
    let matches: Vec<_> = nodes.iter().filter(|n| n.name == name).collect();
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].file_id, winner.file_id);
    assert_eq!(matches[0].hash, winner.hash);
    a.download_checked(&winner, &dir.path().join("verify"))
        .await?;
    assert_eq!(std::fs::read(dir.path().join("verify"))?, body);
    let repeated = b
        .upload_snapshot(home.view_id, &path, zero, &source_b)
        .await;
    let repeat_code = code(&repeated);
    assert_ne!(repeat_code, 0);
    let nodes = a.list_dir(home.view_id, lab).await?;
    let n = nodes
        .iter()
        .find(|n| n.name == name)
        .ok_or("winner missing")?;
    assert_eq!(n.file_id, winner.file_id);
    assert_eq!(n.hash, winner.hash);
    assert_eq!(n.sync_id, winner.version.sync_id);
    let result = serde_json::json!({"passed":true,"lab_path":lab,"two_independent_sessions":true,"concurrent_create_codes":codes,"existing_path_create_rejected_code":repeat_code,"winner_content_verified":true,"winner_version_preserved":true});
    if let Some(out) = args.get(2) {
        std::fs::write(out, serde_json::to_vec_pretty(&result)?)?
    }
    println!("{result}");
    Ok(())
}
