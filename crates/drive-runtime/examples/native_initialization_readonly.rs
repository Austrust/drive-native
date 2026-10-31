//! Production snapshot path, read-only NAS, isolated ephemeral local state.
//! No mounts, write operations, credentials in arguments, or file names in output.
use drive_core::{
    Store,
    native::{NativeReadSession, Scope},
    snapshot_progress,
};
use serde::Deserialize;
use serde_json::json;
use sproto::{
    Client,
    client::{Config, Credentials},
};
use std::{
    io::{Read, Write},
    os::unix::fs::OpenOptionsExt,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Input {
    host: String,
    tls_name: String,
    username: String,
    scope: Scope,
    #[serde(default)]
    trusted_ca_pem: Vec<String>,
}

async fn run() -> Result<(), &'static str> {
    let args: Vec<_> = std::env::args_os().collect();
    if args.len() != 3 {
        return Err("expected_nonsecret_config_and_new_report");
    }
    let mut data = Vec::new();
    std::fs::File::open(&args[1])
        .map_err(|_| "configuration")?
        .take(1_048_577)
        .read_to_end(&mut data)
        .map_err(|_| "configuration")?;
    if data.len() > 1_048_576 {
        return Err("configuration_budget");
    }
    let input: Input = serde_json::from_slice(&data).map_err(|_| "configuration")?;
    let ca = sproto::trust::parse_trusted_pem(&input.trusted_ca_pem)
        .map_err(|_| "trust_configuration")?;
    let output = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&args[2])
        .map_err(|_| "new_report_required")?;
    let output = Arc::new(Mutex::new(output));
    let write = |value: serde_json::Value| {
        let mut file = output.lock().unwrap();
        writeln!(file, "{value}").map_err(|_| "report_write")?;
        file.flush().map_err(|_| "report_write")
    };
    write(json!({"stage":"awaiting_secret","nas_writes":0,"mounts":0}))?;
    let password =
        rpassword::prompt_password("Password (not saved): ").map_err(|_| "secret_input")?;
    let client = tokio::time::timeout(
        Duration::from_secs(90),
        Client::connect(
            Config::builder()
                .host(input.host)
                .tls_server_name(input.tls_name)
                .credentials(Credentials::Password {
                    username: input.username,
                    password,
                    otp: None,
                })
                .trusted_ca_der(ca)
                .max_channels(4usize)
                .build(),
        ),
    )
    .await
    .map_err(|_| "connection_deadline")?
    .map_err(|_| "connection_failed")?;
    let server_build = client.server_build();
    let session = NativeReadSession::new(client, input.scope).map_err(|_| "scope_configuration")?;
    let local = tempfile::tempdir().map_err(|_| "isolated_store")?;
    let mut store = Store::open(local.path()).map_err(|_| "isolated_store")?;
    session.bind(&mut store).map_err(|_| "scope_binding")?;
    let old_cursor = store.cursor().map_err(|_| "isolated_cursor")?;
    let begin = Instant::now();
    let observed = Arc::new(Mutex::new((
        None,
        Instant::now(),
        0u64,
        None::<f64>,
        0u64,
        0u64,
    )));
    let out = output.clone();
    let observations = observed.clone();
    let observer: snapshot_progress::Observer = Arc::new(move |p| {
        let mut state = observations.lock().unwrap();
        state.2 += 1;
        if p.phase == snapshot_progress::Phase::Enumerate && p.units > 0 && state.3.is_none() {
            state.3 = Some(begin.elapsed().as_secs_f64());
        }
        if p.phase == snapshot_progress::Phase::Seed {
            state.4 = state.4.max(p.units);
        }
        if p.phase == snapshot_progress::Phase::Watermark {
            state.5 = state.5.max(p.units);
        }
        if state.0 != Some((p.attempt, p.phase)) || state.1.elapsed() >= Duration::from_secs(5) {
            state.0 = Some((p.attempt, p.phase));
            state.1 = Instant::now();
            let value = json!({"stage":"index_progress","elapsed_s":begin.elapsed().as_secs_f64(),"progress":p});
            let mut file = out.lock().unwrap();
            let _ = writeln!(file, "{value}");
            let _ = file.flush();
        }
    });
    let result = tokio::time::timeout(
        Duration::from_secs(3600),
        snapshot_progress::scope(observer, session.recovery_snapshot()),
    )
    .await;
    let snapshot = match result {
        Ok(Ok(snapshot)) => snapshot,
        failure => {
            let category = match failure {
                Err(_) => "overall_deadline",
                Ok(Err(drive_core::Error::RootUnavailable)) => "root_unavailable",
                Ok(Err(drive_core::Error::Native(sproto::Error::Server { .. }))) => "server",
                Ok(Err(drive_core::Error::Native(sproto::Error::Timeout(_)))) => "native_timeout",
                Ok(Err(drive_core::Error::Unsupported(_))) => "unsupported_contract",
                _ => "snapshot_failed",
            };
            write(
                json!({"stage":"ended","success":false,"category":category,"elapsed_s":begin.elapsed().as_secs_f64(),
                "cursor_unchanged":store.cursor().map_err(|_| "isolated_cursor")?==old_cursor,"nas_writes":0}),
            )?;
            return Err(category);
        }
    };
    let snapshot_seconds = begin.elapsed().as_secs_f64();
    let nodes = snapshot.nodes.len();
    let inaccessible = snapshot.inaccessible.len();
    let import = Instant::now();
    store
        .install_native_snapshot(&snapshot, &old_cursor)
        .map_err(|_| "isolated_install")?;
    let matched = store.cursor().map_err(|_| "isolated_cursor")? == snapshot.cursor;
    let measurements = observed.lock().unwrap();
    write(
        json!({"stage":"completed","success":matched,"server_build":server_build,"nodes":nodes,
        "inaccessible_directories":inaccessible,"snapshot_seconds":snapshot_seconds,"import_seconds":import.elapsed().as_secs_f64(),
        "progress_observations":measurements.2,"first_directory_list_seconds":measurements.3,
        "legacy_seed_pages_max":measurements.4,"watermark_steps":measurements.5,
        "durable_cursor_matches_snapshot":matched,"first_directory_list_is_not_fuse_visibility":true,
        "nas_writes":0,"mounts":0,"original_state_changed":false}),
    )?;
    output
        .lock()
        .unwrap()
        .sync_all()
        .map_err(|_| "report_sync")?;
    if !matched {
        return Err("cursor_mismatch");
    }
    println!("Read-only initialization completed; sanitized report saved.");
    Ok(())
}
#[tokio::main]
async fn main() {
    unsafe {
        let limit = libc::rlimit {
            rlim_cur: 0,
            rlim_max: 0,
        };
        libc::setrlimit(libc::RLIMIT_CORE, &limit);
    }
    if let Err(category) = run().await {
        eprintln!("read-only probe: {category}");
        std::process::exit(1);
    }
}
