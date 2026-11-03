//! Offline synthetic fixture for the ORIGINAL FSearch application. No NAS or
//! protocol worker is configured. Local mode is a separate comparison filesystem,
//! never a surrogate for the Drive mount being indexed.
#![allow(unexpected_cfgs)]
use drive_core::{Node, NodeMetadata, ReadHandle, Store};
use drive_fs::{CacheFilesystem, Hydrator};
use std::{
    io::{BufRead, Write},
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::{Duration, UNIX_EPOCH},
};
struct NoNetwork(AtomicU64);
impl Hydrator for NoNetwork {
    fn hydrate(&self, _: &Node) -> Result<Arc<ReadHandle>, i32> {
        self.0.fetch_add(1, Ordering::Relaxed);
        Err(libc::ENETDOWN)
    }
}
fn emit(value: serde_json::Value) {
    println!("{value}");
    std::io::stdout().flush().unwrap();
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 6 {
        return Err(
            "usage: fsearch_fixture RUN_DIR fuse|fuse-writable|local GROUPS FILES_PER_GROUP WIDE_FILES".into(),
        );
    }
    let run = PathBuf::from(&args[1]);
    let local = args[2] == "local";
    let writable = args[2] == "fuse-writable";
    if !local && !writable && args[2] != "fuse" {
        return Err("invalid mode".into());
    }
    if run.exists() {
        return Err("fixture destination must be new".into());
    }
    let groups = args[3].parse::<usize>()?;
    let files = args[4].parse::<usize>()?;
    let wide = args[5].parse::<usize>()?;
    // Test-only scale bounds. The measurement matrix now includes a
    // Personal_Sync-sized tree (56,007 groups / 319,679 entries), previously
    // rejected by the 10,000-group gate; the total budget stays bounded so a
    // typo cannot allocate an unbounded tree. This example is a fixture, never
    // shipped product code.
    const MAX_ENTRIES: usize = 500_000;
    const MAX_GROUPS: usize = 100_000;
    // root + wide + 64 deep directories, and the 64 `leaf.dat` files.
    let total_entries = groups
        .checked_mul(files.saturating_add(1))
        .and_then(|n| n.checked_add(wide))
        .and_then(|n| n.checked_add(130))
        .ok_or("fixture entry count overflow")?;
    if total_entries > MAX_ENTRIES || groups > MAX_GROUPS {
        return Err(format!(
            "fixture budget exceeded: {total_entries} entries / {groups} groups (max {MAX_ENTRIES}/{MAX_GROUPS})"
        )
        .into());
    }
    std::fs::create_dir_all(&run)?;
    let mount = run.join("mount");
    std::fs::create_dir(&mount)?;
    let mut entries = Vec::<(String, Node)>::new();
    let mut add = |path: String, parent: Option<String>, directory: bool| {
        let id = if path.is_empty() {
            "root".into()
        } else {
            path.clone()
        };
        let name = if path.is_empty() {
            "root".into()
        } else {
            path.rsplit('/').next().unwrap().into()
        };
        entries.push((
            path,
            Node {
                id,
                parent,
                name,
                version: "v1".into(),
                directory,
                metadata: Some(NodeMetadata {
                    size: if directory { 0 } else { 1234 },
                    mtime: 1_700_000_000,
                    mode: if directory { 0o755 } else { 0o644 },
                }),
            },
        ));
    };
    add(String::new(), None, true);
    for group in 0..groups {
        let dir = format!("d{group:05}");
        add(dir.clone(), Some("root".into()), true);
        for file in 0..files {
            add(format!("{dir}/f{file:05}.dat"), Some(dir.clone()), false);
        }
    }
    add("wide".into(), Some("root".into()), true);
    for file in 0..wide {
        add(format!("wide/w{file:06}.dat"), Some("wide".into()), false);
    }
    let mut parent = "root".to_string();
    let mut path = String::new();
    for depth in 0..64 {
        if !path.is_empty() {
            path.push('/');
        }
        path.push_str(&format!("deep{depth:02}"));
        add(path.clone(), Some(parent), true);
        add(format!("{path}/leaf.dat"), Some(path.clone()), false);
        parent = path.clone();
    }
    let truth:Vec<_>=entries.iter().map(|(path,n)|serde_json::json!({"path":path,"directory":n.directory,"size":n.metadata.as_ref().unwrap().size,"mtime":n.metadata.as_ref().unwrap().mtime})).collect();
    std::fs::write(run.join("truth.json"), serde_json::to_vec(&truth)?)?;
    let no_network = Arc::new(NoNetwork(AtomicU64::new(0)));
    let mut session = None;
    if local {
        for (path, node) in &entries {
            if path.is_empty() {
                continue;
            }
            let target = mount.join(path);
            if node.directory {
                std::fs::create_dir(&target)?;
            } else {
                std::fs::File::create(&target)?.set_len(1234)?;
            }
        }
        for (path, _) in entries.iter().rev() {
            std::fs::File::open(mount.join(path))?.set_times(
                std::fs::FileTimes::new()
                    .set_modified(UNIX_EPOCH + Duration::from_secs(1_700_000_000)),
            )?;
        }
    } else {
        let mut store = Store::open(run.join("state"))?;
        store.install_remote_snapshot(
            &entries.iter().map(|(_, n)| n.clone()).collect::<Vec<_>>(),
            "1",
            "synthetic",
        )?;
        let shared = Arc::new(drive_core::SharedStore::new(store));
        // Metadata caching is active only after the session notifier is
        // installed; the default TTL is DRIVE_NATIVE_FS_TTL_MS (2 s).
        let notify = drive_fs::NotifyHub::from_env();
        let mut fs = CacheFilesystem::new(shared, "root", unsafe { libc::getuid() }, unsafe {
            libc::getgid()
        })?
        .with_hydrator(no_network.clone())
        .with_notify_hub(notify.clone());
        // Only the explicit functional-test mode accepts local edits.
        // Default paired scans retain the original read-only behavior.
        if writable {
            fs = fs.with_writes_enabled();
        }
        let mounted = fuser::spawn_mount2(
            fs,
            &mount,
            &[
                fuser::MountOption::DefaultPermissions,
                fuser::MountOption::NoDev,
                fuser::MountOption::NoSuid,
                fuser::MountOption::FSName("drive-native-fsearch-synthetic".into()),
            ],
        )?;
        notify.install(mounted.notifier());
        session = Some(mounted);
    }
    emit(
        serde_json::json!({"ready":true,"mode":args[2],"files":entries.iter().filter(|(_,n)|!n.directory).count(),"directories_including_root":entries.iter().filter(|(_,n)|n.directory).count()}),
    );
    for line in std::io::stdin().lock().lines() {
        match line?.as_str() {
            "sample" => {
                let mut value =
                    serde_json::json!({"hydration_attempts":no_network.0.load(Ordering::Relaxed)});
                #[cfg(drive_fs_diagnostics)]
                {
                    value["diagnostics"]=serde_json::json!(drive_fs::diagnostics::snapshot().into_iter().map(|s|serde_json::json!({"stage":format!("{:?}",s.stage),"count":s.count,"total_us":s.total_us,"max_us":s.max_us,"buckets":s.buckets})).collect::<Vec<_>>());
                }
                emit(value);
            }
            "stop" => break,
            _ => return Err("unknown fixture command".into()),
        }
    }
    drop(session.take());
    emit(
        serde_json::json!({"stopped":true,"hydration_attempts":no_network.0.load(Ordering::Relaxed)}),
    );
    Ok(())
}
