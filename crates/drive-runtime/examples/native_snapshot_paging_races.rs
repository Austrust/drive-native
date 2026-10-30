//! Real mutations between page replies; only a fresh authorized lab subtree.
use drive_core::{
    Error, Store,
    native::{NativeReadSession, Scope},
};
use sproto::{
    Client,
    actions::{
        list_observer::{PausedPage, with_page_observer},
        namespace::NamespaceTarget,
        native::NativeVersion,
    },
    client::{Config, Credentials},
};
use std::{
    collections::BTreeMap,
    io::{Read, Write},
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
async fn mutate(
    client: &Client,
    scope: &Scope,
    mode: &str,
    body: &std::path::Path,
) -> sproto::Result<serde_json::Value> {
    let mut nodes = client.list_dir(scope.view_id, &scope.path).await?;
    nodes.retain(|n| !n.is_removed);
    nodes.sort_by(|a, b| a.name.cmp(&b.name));
    assert!(nodes.len() > 1000);
    if mode == "add-file" {
        let name = format!("added-{}.txt", uuid::Uuid::new_v4());
        let receipt = client
            .upload_snapshot(
                scope.view_id,
                &format!("{}/{}", scope.path, name),
                NativeVersion {
                    sync_id: 0,
                    max_id: 0,
                },
                body,
            )
            .await?;
        return Ok(serde_json::json!({"action":"create","id":receipt.file_id,"name":name}));
    }
    let index = match mode {
        "rename-last-to-first" => nodes.len() - 1,
        "rename-within-page" | "continuous" => 10,
        _ => 0,
    };
    let node = &nodes[index];
    let target =
        NamespaceTarget::from_node(scope.view_id, format!("{}/{}", scope.path, node.name), node)?;
    if mode == "delete-first" {
        client.remove_native(&target).await?;
        return Ok(serde_json::json!({"action":"delete","id":node.file_id,"old_name":node.name}));
    }
    let name = match mode {
        "rename-last-to-first" => format!("!moved-{}.txt", uuid::Uuid::new_v4()),
        "rename-within-page" | "continuous" => {
            format!("{}-moved-{}", node.name, uuid::Uuid::new_v4())
        }
        _ => format!(
            "{}~moved-{}",
            nodes.last().unwrap().name,
            uuid::Uuid::new_v4()
        ),
    };
    let receipt = client
        .rename_native(&target, &format!("{}/{}", scope.path, name))
        .await?;
    assert_eq!(receipt.file_id, node.file_id);
    Ok(
        serde_json::json!({"action":"rename","id":node.file_id,"old_name":node.name,"new_name":name}),
    )
}
#[tokio::main]
async fn main() -> Result<()> {
    assert_ne!(unsafe { libc::geteuid() }, 0);
    drive_runtime::disable_core_dumps()?;
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 3 || args[1] != "/DriveNative-Lab/20261028T124720-1e31763a" {
        return Err("expected authorized lab and new result path".into());
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
    assert!(home.writable && home.download_allowed);
    let root = format!("{}/paging-races-{}", args[1], uuid::Uuid::new_v4());
    let receipt = a.mkdir_native(home.view_id, &root).await?;
    println!("test_subtree={root}");
    let scope = Scope {
        view_id: home.view_id,
        node_id: receipt.node_id,
        file_id: receipt.file_id,
        path: root.clone(),
    };
    let tmp = tempfile::tempdir()?;
    let source = tmp.path().join("empty");
    std::fs::write(&source, b"")?;
    let mut uploads = tokio::task::JoinSet::new();
    let mut completed = 0;
    for i in 0..1002 {
        let client = a.clone();
        let file = source.clone();
        let path = format!("{root}/f{i:04}.txt");
        let view = home.view_id;
        uploads.spawn(async move {
            client
                .upload_snapshot(
                    view,
                    &path,
                    NativeVersion {
                        sync_id: 0,
                        max_id: 0,
                    },
                    &file,
                )
                .await
        });
        if uploads.len() >= 4 {
            uploads.join_next().await.unwrap()??;
            completed += 1;
        }
        if completed > 0 && completed % 100 == 0 {
            println!("created_empty_fixtures={completed}");
            std::fs::write(output,serde_json::json!({"passed":false,"scope":root,"stage":"fixtures","created":completed}).to_string())?;
        }
    }
    while let Some(done) = uploads.join_next().await {
        done??;
        completed += 1;
    }
    assert_eq!(completed, 1002);
    let session = NativeReadSession::new(a.clone(), scope.clone())?;
    let mut cases = Vec::new();
    for mode in [
        "delete-first",
        "rename-first-to-last",
        "rename-last-to-first",
        "rename-within-page",
        "add-file",
        "rename-first-to-last-auto",
        "continuous",
    ] {
        std::fs::write(
            output,
            serde_json::to_vec_pretty(&serde_json::json!({
            "passed":false,"scope":root,"stage":mode,"completed_cases":cases}))?,
        )?;
        let baseline = session.recovery_snapshot().await?;
        let mut store = Store::open(tmp.path().join(mode))?;
        session.bind(&mut store)?;
        store.install_native_snapshot(&baseline, "")?;
        store.ensure_fs_root(&scope.file_id)?;
        let mut files: Vec<_> = baseline.nodes.iter().filter(|n| !n.directory).collect();
        files.sort_by(|a, b| a.name.cmp(&b.name));
        let draft_id = files[500].id.clone();
        let pin_id = files[600].id.clone();
        let draft = store.save_local(&draft_id, b"local draft must survive rejected pagination")?;
        store.pin(&pin_id, true)?;
        let prior = store.cursor()?;
        let (tx, mut rx) = tokio::sync::mpsc::channel::<PausedPage>(1);
        let client = b.clone();
        let watched = scope.clone();
        let body = source.clone();
        let observer = tokio::spawn(async move {
            let mut pages = Vec::new();
            let mut mutations = Vec::new();
            while let Some(pause) = rx.recv().await {
                let page = pause.boundary;
                pages.push(
                    serde_json::json!({"path":page.path,"offset":page.offset,"count":page.count,
                    "total":page.total,"first":page.first_name,"last":page.last_name}),
                );
                if page.path == watched.path
                    && page.offset == 0
                    && page.total > page.count
                    && (mutations.is_empty() || mode == "continuous")
                {
                    assert_eq!(page.count, 1000);
                    match mutate(&client, &watched, mode, &body).await {
                        Ok(m) => mutations.push(m),
                        Err(error) => {
                            let _ = pause.resume.send(Err(error));
                            return Err("independent mutation failed");
                        }
                    }
                }
                let _ = pause.resume.send(Ok(()));
            }
            Ok((pages, mutations))
        });
        let automatic = mode.ends_with("-auto") || mode == "continuous";
        let candidate = if automatic {
            with_page_observer(tx, session.recovery_snapshot()).await
        } else {
            with_page_observer(tx, session.snapshot_candidate()).await
        };
        let (pages, mutations) = observer.await?.map_err(std::io::Error::other)?;
        assert_eq!(mutations.len(), if mode == "continuous" { 3 } else { 1 });
        assert_eq!(
            store.cursor()?,
            prior,
            "unaccepted candidate changed local state"
        );
        let (accepted, rejection) = match candidate {
            Ok(snapshot) if automatic => {
                assert!(session.snapshot_unchanged(&snapshot).await?);
                (Some(snapshot), "internal-retry-and-fence")
            }
            Ok(snapshot) => {
                assert!(
                    !session.snapshot_unchanged(&snapshot).await?,
                    "mixed pages accepted"
                );
                (None, "event-fence")
            }
            Err(Error::SnapshotChanged) if !automatic => {
                (None, "page-error-with-native-event-proof")
            }
            Err(Error::Native(sproto::Error::Timeout(
                "snapshot repeatedly changed; retry recovery later",
            ))) if mode == "continuous" => (None, "three-attempt-limit"),
            Err(error) => return Err(error.into()),
        };
        let stable = match accepted {
            Some(snapshot) => snapshot,
            None => session.recovery_snapshot().await?,
        };
        let remote = b.list_dir(home.view_id, &root).await?;
        let expected: BTreeMap<_, _> = remote
            .iter()
            .filter(|n| !n.is_removed)
            .map(|n| {
                (
                    n.file_id.clone(),
                    (
                        n.name.clone(),
                        n.sync_id,
                        n.max_id,
                        n.file_size,
                        n.hash.clone(),
                    ),
                )
            })
            .collect();
        let actual: BTreeMap<_, _> = stable
            .nodes
            .iter()
            .filter(|n| !n.directory)
            .map(|n| {
                let v: serde_json::Value = serde_json::from_str(&n.version).unwrap();
                assert_eq!(n.parent.as_deref(), Some(scope.file_id.as_str()));
                (
                    n.id.clone(),
                    (
                        n.name.clone(),
                        v["sync"].as_u64().unwrap(),
                        v["max"].as_u64().unwrap(),
                        v["size"].as_u64().unwrap(),
                        v["hash"].as_str().unwrap().to_string(),
                    ),
                )
            })
            .collect();
        assert_eq!(actual, expected);
        assert_eq!(stable.nodes.len(), expected.len() + 1);
        store.install_native_snapshot(&stable, &prior)?;
        assert!(store.is_pinned(&pin_id)?);
        let mut job = store.upload_job(&draft)?;
        assert_eq!(job.state, "pending");
        let mut bytes = Vec::new();
        job.file.read_to_end(&mut bytes)?;
        assert_eq!(bytes, b"local draft must survive rejected pagination");
        assert_eq!(store.cursor()?, stable.cursor);
        cases.push(serde_json::json!({"mode":mode,"passed":true,"rejection":rejection,"pages":pages,
            "mutations":mutations,"recovered_files":expected.len(),"all_identities_versions_names_equal":true,
            "prior_cursor_preserved_until_install":true,"local_draft_and_pin_preserved":true,
            "installed_cursor":stable.cursor}));
        println!(
            "PASS {mode}: {rejection}; {} exact file identities",
            expected.len()
        );
    }
    std::fs::write(
        output,
        serde_json::to_vec_pretty(&serde_json::json!({"passed":true,"scope":root,
        "created_empty_files":1002,"cases":cases,"barrier_between_actual_pages":true,
        "independent_native_mutation_session":true,"body_downloads":0,"fuse_runtime_tested":false,
        "nas_restore_or_permissions_changed":false,"candidate_release":false}))?,
    )?;
    println!("PASS actual paginated native snapshot races and preserved local state");
    Ok(())
}
