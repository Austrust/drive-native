//! Real FUSE replacement, coordinator/unread wire acknowledgement loss and cleanup.
#[path = "support/commit_observation.rs"]
mod commit_observation;
use drive_core::{
    Status, Store,
    native::{NativeReadSession, NativeWriteOutcome, Scope},
};
use drive_runtime::{
    bridge::{Bridge, WorkerProcess},
    wire::Login,
};
use sproto::{
    Client,
    actions::native::NativeVersion,
    client::{Config, Credentials},
};
use std::{
    io::Write,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
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
#[derive(Clone, Copy)]
enum Case {
    LocalSource,
    RemoteSource,
    ForeignTarget,
    ChangedSource,
}
impl Case {
    fn name(self) -> &'static str {
        match self {
            Self::LocalSource => "local-source",
            Self::RemoteSource => "remote-source",
            Self::ForeignTarget => "foreign-target",
            Self::ChangedSource => "changed-source",
        }
    }
    fn remote_source(self) -> bool {
        !matches!(self, Self::LocalSource)
    }
}
async fn run_case(
    author: &Client,
    observer: &Client,
    password: &str,
    view: u64,
    run: &str,
    case: Case,
    unread: bool,
) -> Result<serde_json::Value> {
    let path = format!("{run}/{}", case.name());
    let dir = author.mkdir_native(view, &path).await?;
    let scope = Scope {
        view_id: view,
        node_id: dir.node_id,
        file_id: dir.file_id,
        path: path.clone(),
    };
    let scratch = tempfile::tempdir()?;
    let state = scratch.path().join("data");
    let mount = scratch.path().join("mount");
    std::fs::create_dir(&mount)?;
    let source_file = scratch.path().join("body");
    let bytes = b"atomic replacement content from this local source";
    std::fs::write(&source_file, bytes)?;
    let hash = drive_core::sha256(bytes);
    let remote_source = if case.remote_source() {
        Some(
            author
                .upload_snapshot(
                    view,
                    &format!("{path}/Temporary"),
                    NativeVersion {
                        sync_id: 0,
                        max_id: 0,
                    },
                    &source_file,
                )
                .await?,
        )
    } else {
        None
    };
    let session = NativeReadSession::new(author.clone(), scope.clone())?;
    let mut store = Store::open(&state)?;
    session.bind(&mut store)?;
    while let Some(batch) = session.pull(&store.cursor()?).await? {
        store.receive(&batch)?;
        while store.apply_next()? {}
    }
    store.ensure_fs_root(&scope.file_id)?;
    if let Some(remote) = &remote_source {
        let n = store.node(&remote.file_id)?;
        store.publish_download(
            &drive_core::DownloadTicket {
                id: n.id,
                version: n.version,
            },
            bytes,
            &hash,
        )?;
    }
    let shared = Arc::new(Mutex::new(store));
    let fs = drive_fs::CacheFilesystem::new(
        shared.clone(),
        &scope.file_id,
        unsafe { libc::getuid() },
        unsafe { libc::getgid() },
    )?
    .with_writes_enabled();
    let mounted = fuser::spawn_mount2(
        fs,
        &mount,
        &[
            fuser::MountOption::RW,
            fuser::MountOption::DefaultPermissions,
            fuser::MountOption::NoDev,
            fuser::MountOption::NoSuid,
        ],
    )?;
    std::fs::write(mount.join("Destination"), b"unsent target")?;
    if !case.remote_source() {
        std::fs::write(mount.join("Temporary"), bytes)?;
    }
    let source = shared
        .lock()
        .unwrap()
        .lookup(&scope.file_id, "Temporary")?
        .node;
    let inode = shared.lock().unwrap().record(&source.id)?.ino;
    std::fs::rename(mount.join("Temporary"), mount.join("Destination"))?;
    assert_eq!(std::fs::read(mount.join("Destination"))?, bytes);
    mounted.join();
    let mut store = Arc::try_unwrap(shared)
        .map_err(|_| "filesystem retained state")?
        .into_inner()
        .unwrap();
    let job = store
        .next_replace()?
        .ok_or("FUSE did not journal replacement")?
        .job;
    assert!(job.target_remote.is_none());
    assert_eq!(job.source.id, source.id);
    let origin = session
        .prepare_upload_creation(
            &job.target,
            &store.node(job.target.parent.as_deref().unwrap())?,
            &format!("{path}/Destination"),
            &hash,
            &store.cursor()?,
        )
        .await?
        .ok_or("unexpected target")?;
    store.prepare_replace_origin(&job.operation, origin.clone(), &hash)?;
    store.mark_replace_started(&job.operation)?;
    let job = store.next_replace()?.unwrap().job;
    if matches!(case, Case::ForeignTarget) {
        observer
            .upload_snapshot(
                view,
                &format!("{path}/Destination"),
                NativeVersion {
                    sync_id: 0,
                    max_id: 0,
                },
                &source_file,
            )
            .await?;
    } else if unread {
        let result = author
            .research_upload_identity_drop_receipt_after(
                &job.creation_origin
                    .as_ref()
                    .ok_or("missing target creation origin")?
                    .parent_file,
                Some("Destination"),
                NativeVersion {
                    sync_id: 0,
                    max_id: 0,
                },
                &source_file,
                commit_observation::file_after(
                    observer,
                    view,
                    &path,
                    "Destination",
                    bytes.len() as u64,
                    0,
                ),
            )
            .await;
        commit_observation::assert_unread_fault(result);
    } else {
        assert!(matches!(
            session
                .replace_target(&job, &source_file, &hash, scratch.path(), false)
                .await?,
            NativeWriteOutcome::Created { .. }
        ));
    }
    // No confirmation enters the coordinator's journal. A new Store must recover
    // the inflight target before any cleanup of the existing remote source.
    let before = observer.list_dir(view, &path).await?;
    let target = before
        .iter()
        .find(|n| n.name == "Destination")
        .ok_or("target absent after injection")?
        .clone();
    if case.remote_source() {
        assert!(before.iter().any(|n| n.name == "Temporary"));
    }
    let mut changed_source = None;
    if matches!(case, Case::ChangedSource) {
        let current = before.iter().find(|n| n.name == "Temporary").unwrap();
        std::fs::write(
            &source_file,
            b"another device changed the source before cleanup",
        )?;
        changed_source = Some(
            observer
                .upload_snapshot(
                    view,
                    &format!("{path}/Temporary"),
                    NativeVersion {
                        sync_id: current.sync_id,
                        max_id: current.max_id,
                    },
                    &source_file,
                )
                .await?,
        );
    }
    drop(session);
    drop(store);
    let shared = Arc::new(Mutex::new(Store::open(&state)?));
    let q = shared.lock().unwrap().next_replace()?.unwrap();
    assert_eq!(q.state, "uncertain");
    assert_eq!(q.job.creation_origin, Some(origin));
    let executable = std::env::current_exe()?
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("drive-native-worker");
    let mut worker = WorkerProcess::start(&executable)?;
    let bridge = Arc::new(Bridge::connect(
        shared.clone(),
        worker.rpc.clone(),
        Login {
            host: "192.0.2.10".into(),
            tls_name: "nas.example.invalid".into(),
            trusted_ca_pem: Vec::new(),
            username: "test-user".into(),
            password: password.into(),
            device_id: uuid::Uuid::new_v4().to_string(),
            scope: scope.clone(),
        },
    )?);
    let fs = drive_fs::CacheFilesystem::new(
        shared.clone(),
        &scope.file_id,
        unsafe { libc::getuid() },
        unsafe { libc::getgid() },
    )?
    .with_writes_enabled()
    .with_hydrator(bridge.clone());
    let mounted = fuser::spawn_mount2(
        fs,
        &mount,
        &[
            fuser::MountOption::RW,
            fuser::MountOption::DefaultPermissions,
            fuser::MountOption::NoDev,
            fuser::MountOption::NoSuid,
        ],
    )?;
    bridge.set_notifier(mounted.notifier());
    let deadline = Instant::now() + Duration::from_secs(40);
    loop {
        bridge.namespace_pending()?;
        let state = shared.lock().unwrap().next_replace()?.map(|q| q.state);
        if state.is_none() || state.as_deref() == Some("conflict") {
            break;
        }
        if Instant::now() > deadline {
            return Err(format!(
                "replacement recovery deadline in {}: {state:?}",
                case.name()
            )
            .into());
        }
        std::thread::sleep(Duration::from_secs(2));
    }
    assert_eq!(
        shared
            .lock()
            .unwrap()
            .lookup(&scope.file_id, "Destination")?
            .ino,
        inode
    );
    assert_eq!(std::fs::read(mount.join("Destination"))?, bytes);
    let after = observer.list_dir(view, &path).await?;
    let current = after
        .iter()
        .find(|n| n.name == "Destination")
        .ok_or("target disappeared")?;
    assert_eq!(
        (
            &current.file_id,
            current.sync_id,
            current.max_id,
            &current.hash
        ),
        (&target.file_id, target.sync_id, target.max_id, &target.hash)
    );
    let final_state;
    if matches!(case, Case::ForeignTarget) {
        assert_eq!(shared.lock().unwrap().status(&source.id)?, Status::Conflict);
        assert_ne!(
            shared.lock().unwrap().local_identity(&target.file_id)?,
            source.id
        );
        let old = remote_source.as_ref().unwrap();
        let current = after
            .iter()
            .find(|n| n.name == "Temporary")
            .ok_or("foreign target caused source deletion")?;
        assert_eq!(
            (&current.file_id, current.sync_id, current.max_id),
            (&old.file_id, old.version.sync_id, old.version.max_id)
        );
        final_state = "conflict";
    } else if matches!(case, Case::ChangedSource) {
        assert_eq!(
            shared.lock().unwrap().local_identity(&target.file_id)?,
            source.id
        );
        assert_eq!(shared.lock().unwrap().status(&source.id)?, Status::Conflict);
        let changed = changed_source.unwrap();
        let current = after
            .iter()
            .find(|n| n.name == "Temporary")
            .ok_or("changed source was deleted")?;
        assert_eq!(
            (
                &current.file_id,
                current.sync_id,
                current.max_id,
                &current.hash
            ),
            (
                &changed.file_id,
                changed.version.sync_id,
                changed.version.max_id,
                &changed.hash
            )
        );
        final_state = "conflict";
    } else {
        assert!(shared.lock().unwrap().next_replace()?.is_none());
        assert_eq!(
            shared.lock().unwrap().local_identity(&target.file_id)?,
            source.id
        );
        assert!(!after.iter().any(|n| n.name == "Temporary"));
        let edited = b"saved again after replacement recovery";
        std::fs::write(mount.join("Destination"), edited)?;
        let deadline = Instant::now() + Duration::from_secs(30);
        loop {
            if bridge.upload_pending(false)? > 0 {
                break;
            }
            if Instant::now() > deadline {
                return Err("post-recovery edit did not upload".into());
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        let listed = observer.list_dir(view, &path).await?;
        let n = listed.iter().find(|n| n.name == "Destination").unwrap();
        assert_eq!(n.file_id, target.file_id);
        let dest = scratch.path().join("verified");
        observer
            .download_checked(
                &sproto::actions::transfer::ContentVersion {
                    file_id: n.file_id.clone(),
                    version: NativeVersion {
                        sync_id: n.sync_id,
                        max_id: n.max_id,
                    },
                    size: n.file_size,
                    hash: n.hash.clone(),
                },
                &dest,
            )
            .await?;
        assert_eq!(std::fs::read(dest)?, edited);
        final_state = "committed";
    }
    mounted.join();
    worker.stop();
    drop(bridge);
    drop(shared);
    let store = Store::open(&state)?;
    if final_state == "conflict" {
        assert_eq!(store.next_replace()?.unwrap().state, "conflict");
    } else {
        assert!(store.next_replace()?.is_none());
    }
    println!("PASS {}: {final_state}", case.name());
    Ok(
        serde_json::json!({"case":case.name(),"scope":path,"passed":true,"actual_fuse_rename":true,"inflight_journal_reopened":true,"new_worker":true,"stable_source_inode":true,"target_version_unchanged_during_reconciliation":true,
            "target_before":{"id":target.file_id,"sync":target.sync_id,"max":target.max_id,"hash":target.hash},
            "target_after":{"id":current.file_id,"sync":current.sync_id,"max":current.max_id,"hash":current.hash},"remote_source_retained_until_target_confirmation":case.remote_source(),"state":final_state,"wire_response_discarded":unread && !matches!(case, Case::ForeignTarget),"independent_commit_observed_before_socket_drop":unread && !matches!(case, Case::ForeignTarget)}),
    )
}
#[tokio::main]
async fn main() -> Result<()> {
    assert_ne!(unsafe { libc::geteuid() }, 0);
    unsafe {
        let l = libc::rlimit {
            rlim_cur: 0,
            rlim_max: 0,
        };
        libc::setrlimit(libc::RLIMIT_CORE, &l);
    }
    let args: Vec<_> = std::env::args().collect();
    if !(args.len() == 3 || (args.len() == 4 && args[3] == "--unread-reply"))
        || args[1] != "/DriveNative-Lab/20261028T124720-1e31763a"
    {
        return Err("expected authorized lab and new result".into());
    }
    let unread = args.len() == 4;
    std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&args[2])?
        .write_all(b"{\"passed\":false}")?;
    let password = rpassword::prompt_password("Test account password (not saved): ")?;
    let author = connect(&password).await?;
    let observer = connect(&password).await?;
    let home = author.personal_root().await?;
    assert!(home.writable && home.download_allowed);
    let root = format!("{}/replace-origin-{}", args[1], uuid::Uuid::new_v4());
    author.mkdir_native(home.view_id, &root).await?;
    println!("test_subtree={root}");
    let mut results = Vec::new();
    for case in [
        Case::LocalSource,
        Case::RemoteSource,
        Case::ForeignTarget,
        Case::ChangedSource,
    ] {
        std::fs::write(&args[2],serde_json::json!({"passed":false,"scope":root,"stage":case.name(),"completed":results}).to_string())?;
        results.push(
            run_case(
                &author,
                &observer,
                &password,
                home.view_id,
                &root,
                case,
                unread,
            )
            .await?,
        );
    }
    std::fs::write(
        &args[2],
        serde_json::to_vec_pretty(
            &serde_json::json!({"passed":true,"scope":root,"cases":results,"candidate_release":false,"mint_tested":false}),
        )?,
    )?;
    println!("PASS native atomic-save provenance and ordered cleanup recovery");
    Ok(())
}
