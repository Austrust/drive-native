//! Read-only metadata diagnostics for the authorized snapshot fixture.
use sproto::{
    Client,
    client::{Config, Credentials},
};
async fn connect(password: &str) -> Result<Client, Box<dyn std::error::Error>> {
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
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    unsafe {
        let limit = libc::rlimit {
            rlim_cur: 0,
            rlim_max: 0,
        };
        libc::setrlimit(libc::RLIMIT_CORE, &limit);
    }
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 3
        || !args[1]
            .strip_prefix("/DriveNative-Lab/20261028T124720-1e31763a/snapshot-")
            .is_some_and(|suffix| uuid::Uuid::parse_str(suffix).is_ok())
    {
        return Err("expected authorized snapshot fixture and new output path".into());
    }
    use std::io::Write;
    let mut output = std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&args[2])?;
    let password = rpassword::prompt_password("Test account password (not saved): ")?;
    let client = connect(&password).await?;
    drop(password);
    let home = client.personal_root().await?;
    let nodes = client.list_dir(home.view_id, &args[1]).await?;
    let unusual:Vec<_>=nodes.iter().filter(|n|n.sync_id==0||n.max_id==0).take(4).map(|n|serde_json::json!({"name":n.name,"file_id":n.file_id,"sync_id":n.sync_id,"max_id":n.max_id,"size":n.file_size,"hash":n.hash,"type":format!("{:?}",n.file_type)})).collect();
    let result = serde_json::json!({"scope":args[1],"nodes":nodes.len(),"zero_sync":nodes.iter().filter(|n|n.sync_id==0).count(),"zero_max":nodes.iter().filter(|n|n.max_id==0).count(),"samples":unusual});
    output.write_all(&serde_json::to_vec_pretty(&result)?)?;
    println!("{result}");
    Ok(())
}
