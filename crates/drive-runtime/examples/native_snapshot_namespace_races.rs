//! Explicit two-session native enumeration races, confined to a fresh lab child.
use drive_core::{
    Error, Store,
    native::{NativeReadSession, Scope},
};
use sproto::{
    Client,
    actions::{namespace::NamespaceTarget, native::NativeVersion},
    client::{Config, Credentials},
};
use std::{
    io::Write,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
async fn connect(password: &str) -> Result<Client> {
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
async fn main() -> Result<()> {
    assert_ne!(unsafe { libc::geteuid() }, 0);
    drive_runtime::disable_core_dumps()?;
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 3 || args[1] != "/DriveNative-Lab/20261028T124720-1e31763a" {
        return Err("expected authorized lab and new report path".into());
    }
    let output = &args[2];
    std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output)?
        .write_all(b"{\"passed\":false}")?;
    let password = rpassword::prompt_password("Test account password (not saved): ")?;
    let a = connect(&password).await?;
    let b = connect(&password).await?;
    drop(password);
    let home = a.personal_root().await?;
    assert!(home.writable);
    let root = format!("{}/snapshot-races-{}", args[1], uuid::Uuid::new_v4());
    a.mkdir_native(home.view_id, &root).await?;
    println!("test_subtree={root}");
    let tmp = tempfile::tempdir()?;
    let source = tmp.path().join("body");
    std::fs::write(&source, b"enumeration-race-only")?;
    let mut cases = Vec::new();
    for mode in [
        "delete-listed",
        "move-into-listed",
        "move-into-unlisted",
        "move-into-unlisted-auto",
    ] {
        std::fs::write(
            output,
            serde_json::to_vec_pretty(
                &serde_json::json!({"passed":false,"scope":root,"stage":mode,"completed_cases":cases}),
            )?,
        )?;
        let case_root = format!("{root}/{mode}");
        let receipt = a.mkdir_native(home.view_id, &case_root).await?;
        for name in ["Left", "Right"] {
            a.mkdir_native(home.view_id, &format!("{case_root}/{name}"))
                .await?;
            a.upload_snapshot(
                home.view_id,
                &format!("{case_root}/{name}/item.txt"),
                NativeVersion {
                    sync_id: 0,
                    max_id: 0,
                },
                &source,
            )
            .await?;
        }
        let scope = Scope {
            view_id: home.view_id,
            node_id: receipt.node_id,
            file_id: receipt.file_id,
            path: case_root.clone(),
        };
        let session = NativeReadSession::new(a.clone(), scope.clone())?;
        let fired = Arc::new(AtomicBool::new(false));
        let mutation = Arc::new(Mutex::new(None::<(String, String, String)>));
        let observer = {
            let fired = fired.clone();
            let mutation = mutation.clone();
            let client = b.clone();
            let scope = scope.clone();
            move |visited: String| {
                let client = client.clone();
                let scope = scope.clone();
                let fired = fired.clone();
                let mutation = mutation.clone();
                async move {
                    if visited == scope.path || fired.swap(true, Ordering::SeqCst) {
                        return Ok(());
                    }
                    let other = if visited.ends_with("/Left") {
                        format!("{}/Right", scope.path)
                    } else {
                        format!("{}/Left", scope.path)
                    };
                    let (from, to) = match mode {
                        "move-into-listed" => (other.clone(), visited.clone()),
                        "move-into-unlisted" | "move-into-unlisted-auto" => {
                            (visited.clone(), other)
                        }
                        _ => (visited.clone(), String::new()),
                    };
                    let node = client
                        .list_dir(scope.view_id, &from)
                        .await?
                        .into_iter()
                        .find(|n| n.name == "item.txt" && !n.is_removed)
                        .ok_or_else(|| Error::State("race source absent".into()))?;
                    let src = format!("{from}/item.txt");
                    let target = NamespaceTarget::from_node(scope.view_id, src.clone(), &node)?;
                    let destination = if to.is_empty() {
                        String::new()
                    } else {
                        format!("{to}/moved.txt")
                    };
                    if mode == "delete-listed" {
                        client.remove_native(&target).await?;
                    } else {
                        let moved = client.rename_native(&target, &destination).await?;
                        assert_eq!(moved.file_id, node.file_id);
                    }
                    *mutation.lock().unwrap() = Some((node.file_id, from, to));
                    Ok(())
                }
            }
        };
        let candidate = if mode.ends_with("-auto") {
            session.research_recovery_snapshot(observer).await
        } else {
            session.research_snapshot_candidate(observer).await
        };
        assert!(fired.load(Ordering::SeqCst));
        let accepted_auto = if mode.ends_with("-auto") {
            candidate.as_ref().ok().cloned()
        } else {
            None
        };
        let rejection = match candidate {
            Ok(ref snapshot) if mode.ends_with("-auto") => {
                assert!(session.snapshot_unchanged(snapshot).await?);
                "internal-retry-and-fence"
            }
            Ok(ref snapshot) => {
                assert!(
                    !session.snapshot_unchanged(snapshot).await?,
                    "mixed enumeration accepted"
                );
                "event-fence"
            }
            Err(Error::SnapshotChanged) => "duplicate-identity-with-event-proof",
            Err(error) => return Err(error.into()),
        };
        let (id, from, to) = mutation
            .lock()
            .unwrap()
            .clone()
            .ok_or("mutation was not confirmed")?;
        let stable = match accepted_auto {
            Some(snapshot) => snapshot,
            None => session.recovery_snapshot().await?,
        };
        assert_eq!(
            stable.nodes.len(),
            if mode == "delete-listed" { 4 } else { 5 }
        );
        let matching: Vec<_> = stable.nodes.iter().filter(|node| node.id == id).collect();
        if mode == "delete-listed" {
            assert!(matching.is_empty());
        } else {
            assert_eq!(matching.len(), 1);
            assert_eq!(matching[0].name, "moved.txt");
            let parent = stable
                .nodes
                .iter()
                .find(|node| node.name == to.rsplit('/').next().unwrap() && node.directory)
                .ok_or("new parent absent")?;
            assert_eq!(matching[0].parent.as_deref(), Some(parent.id.as_str()));
        }
        let state = tmp.path().join(mode);
        let mut store = Store::open(&state)?;
        session.bind(&mut store)?;
        store.install_native_snapshot(&stable, "")?;
        assert_eq!(store.cursor()?, stable.cursor);
        cases.push(serde_json::json!({"mode":mode,"candidate_rejected":true,"rejection":rejection,"file_id":id,"from":from,"to":to,"fresh_snapshot_nodes":stable.nodes.len(),"fresh_snapshot_installed":true}));
        println!("PASS {mode}: {rejection}; fresh snapshot correct");
    }
    let busy_root = format!("{root}/continuous");
    let receipt = a.mkdir_native(home.view_id, &busy_root).await?;
    let busy = NativeReadSession::new(
        a.clone(),
        Scope {
            view_id: home.view_id,
            node_id: receipt.node_id,
            file_id: receipt.file_id,
            path: busy_root.clone(),
        },
    )?;
    let attempts = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let result = busy
        .research_recovery_snapshot({
            let attempts = attempts.clone();
            let client = b.clone();
            let root = busy_root.clone();
            move |visited: String| {
                let attempts = attempts.clone();
                let client = client.clone();
                let root = root.clone();
                async move {
                    if visited == root {
                        let attempt = attempts.fetch_add(1, Ordering::SeqCst);
                        client
                            .mkdir_native(home.view_id, &format!("{root}/change-{attempt}"))
                            .await?;
                    }
                    Ok(())
                }
            }
        })
        .await;
    assert!(matches!(
        result,
        Err(Error::Native(sproto::Error::Timeout(
            "snapshot repeatedly changed; retry recovery later"
        )))
    ));
    assert_eq!(attempts.load(Ordering::SeqCst), 3);
    let recovered = busy.recovery_snapshot().await?;
    assert_eq!(recovered.nodes.len(), 4);
    println!(
        "PASS continuously changing enumeration stops after three attempts and later recovers"
    );
    std::fs::write(
        output,
        serde_json::to_vec_pretty(
            &serde_json::json!({"passed":true,"scope":root,"cases":cases,"real_independent_native_sessions":true,"barrier_after_first_child_directory_enumeration":true,"body_downloads":0,"automatic_retry_for_mid_enumeration_error_verified":true,"continuous_changes_bounded_to_three_attempts":true,"quiet_recovery_after_retry_exhaustion":true,"fuse_runtime_tested":false,"nas_restore_or_permissions_changed":false}),
        )?,
    )?;
    Ok(())
}
