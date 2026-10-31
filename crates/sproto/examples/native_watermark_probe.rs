//! Read-only, bounded probe. Config contains connection/scope but no credentials.
//! Password is read from a no-echo terminal; output is counts/timings/booleans only.
use rustls::pki_types::{CertificateDer, pem::PemObject};
use serde_json::{Value, json};
use sproto::{
    Client, Error,
    actions::native::{EventCursor, Watch},
    client::{Config, Credentials},
};
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

fn text<'a>(v: &'a Value, key: &str) -> Result<&'a str, &'static str> {
    v.get(key).and_then(Value::as_str).ok_or("configuration")
}
fn integer(v: &Value, key: &str) -> Result<u64, &'static str> {
    v.get(key).and_then(Value::as_u64).ok_or("configuration")
}
fn classify(e: &Error) -> Value {
    match e {
        Error::Server { code, .. } => json!({"category":"server", "code":code}),
        Error::Decode(_) => json!({"category":"decode"}),
        Error::InvalidConfig(_) => json!({"category":"configuration"}),
        Error::Timeout(_) => json!({"category":"timeout"}),
        Error::CursorReset { .. } => json!({"category":"cursor_reset"}),
        Error::SessionInvalid => json!({"category":"session_expired"}),
        Error::UnsupportedServer { .. } => json!({"category":"unsupported_server"}),
        _ => json!({"category":"transport_or_authentication"}),
    }
}
async fn stage<T>(
    name: &'static str,
    task: impl std::future::Future<Output = sproto::Result<T>>,
) -> Result<T, &'static str> {
    let begin = Instant::now();
    match tokio::time::timeout(Duration::from_secs(90), task).await {
        Ok(Ok(value)) => {
            println!(
                "{}",
                json!({"stage":name,"success":true,"seconds":begin.elapsed().as_secs_f64()})
            );
            Ok(value)
        }
        result => {
            let error = match result {
                Ok(Err(e)) => classify(&e),
                _ => json!({"category":"stage_deadline"}),
            };
            println!(
                "{}",
                json!({"stage":name,"success":false,"seconds":begin.elapsed().as_secs_f64(),"error":error})
            );
            Err("probe_stage_failed")
        }
    }
}
async fn run() -> Result<(), &'static str> {
    let path = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .ok_or("configuration")?;
    let data = std::fs::read(path).map_err(|_| "configuration")?;
    if data.len() > 1024 * 1024 {
        return Err("configuration");
    }
    let cfg: Value = serde_json::from_slice(&data).map_err(|_| "configuration")?;
    if cfg.get("password").is_some() || cfg.get("session").is_some() {
        return Err("credentials_in_config_rejected");
    }
    let scope = cfg.get("scope").ok_or("configuration")?;
    let watch = Watch {
        view_id: integer(scope, "view_id")?,
        node_id: integer(scope, "node_id")?,
        path: text(scope, "path")?.to_owned(),
        cursor: EventCursor(0),
    };
    let expected = text(scope, "file_id")?;
    if watch.path == "/" {
        return Err("root_parent_listing_unsupported");
    }
    let mut ca = Vec::new();
    if let Some(items) = cfg.get("trusted_ca_pem").and_then(Value::as_array) {
        for item in items {
            let pem = item.as_str().ok_or("configuration")?;
            ca.push(
                CertificateDer::from_pem_slice(pem.as_bytes())
                    .map_err(|_| "configuration")?
                    .as_ref()
                    .to_vec(),
            );
        }
    }
    println!(
        "{}",
        json!({"stage":"awaiting_secret","config_valid":true,"nas_writes":0})
    );
    let password =
        rpassword::prompt_password("Password (not saved): ").map_err(|_| "secret_input")?;
    let config = Config::builder()
        .host(text(&cfg, "host")?)
        .tls_server_name(text(&cfg, "tls_name")?.to_owned())
        .credentials(Credentials::Password {
            username: text(&cfg, "username")?.to_owned(),
            password,
            otp: None,
        })
        .trusted_ca_der(ca)
        .max_channels(1usize)
        .build();
    let client = stage("connect", Client::connect(config)).await?;
    let start = stage("current_watermark", client.current_sync_watermark(&watch)).await?;
    let node = stage(
        "selected_root",
        client.current_selected_node(watch.view_id, &watch.path, expected, watch.node_id),
    )
    .await?;
    if node.is_none() {
        return Err("selected_root_missing");
    }
    println!(
        "{}",
        json!({"stage":"root_identity","matched":true,"directory":true,"live":true})
    );
    let pull = Watch {
        cursor: start,
        ..watch.clone()
    };
    let batch = stage("pull_from_watermark", client.pull_events(&pull)).await?;
    let finish = stage(
        "watermark_after_pull",
        client.current_sync_watermark(&watch),
    )
    .await?;
    println!(
        "{}",
        json!({"stage":"completed","events":batch.events.len(),"pull_monotonic":batch.next.0>=start.0,
        "pull_advanced":batch.next.0>start.0,"watermark_monotonic":finish.0>=start.0,
        "pull_not_beyond_later_watermark":batch.next.0<=finish.0,"recovery_required":batch.recovery_required,
        "root_identity_matched":true,"nas_writes":0,"durable_cursor_writes":0,"server_build":client.server_build()})
    );
    Ok(())
}
#[tokio::main]
async fn main() {
    if let Err(category) = run().await {
        println!(
            "{}",
            json!({"stage":"ended","success":false,"category":category})
        );
        std::process::exit(1);
    }
}
