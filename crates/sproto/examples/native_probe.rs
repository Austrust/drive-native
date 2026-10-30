//! Read-only live probe. No password/session or remote filenames in output.
use sproto::{
    Client,
    client::{Config, Credentials},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let password = rpassword::prompt_password("Test account password (not saved): ")?;
    let config = Config::builder()
        .host("192.0.2.10")
        .tls_server_name("nas.example.invalid".to_string())
        .credentials(Credentials::Password {
            username: "test-user".into(),
            password,
            otp: None,
        })
        .max_channels(2usize)
        .build();
    match Client::connect(config).await {
        Ok(client) => {
            println!(
                "{}",
                serde_json::json!({"stage":"native_auth","success":true,"server_build":client.server_build()})
            );
            match client.list_shares().await {
                Ok(shares) => println!(
                    "{}",
                    serde_json::json!({"stage":"native_views","success":true,"count":shares.len(),"downloadable":shares.iter().filter(|s|s.can_download).count()})
                ),
                Err(e) => eprintln!("native view enumeration failed: {e}"),
            }
        }
        Err(sproto::Error::Server { code, .. }) => {
            eprintln!(
                "{}",
                serde_json::json!({"stage":"native_auth","success":false,"server_error_code":code})
            );
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("native connection failed: {e}");
            std::process::exit(1);
        }
    }
    Ok(())
}
