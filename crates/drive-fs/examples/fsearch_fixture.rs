//! Offline synthetic fixture for the ORIGINAL FSearch application. No NAS or
//! protocol worker is configured. Local mode is a separate comparison filesystem,
//! never a surrogate for the Drive mount being indexed.
//!
//! Two generators are supported:
//! * `GROUPS FILES_PER_GROUP WIDE_FILES`: deterministic synthetic tree.
//! * `fuse-manifest MANIFEST.tsv`: the real namespace of an existing tree,
//!   supplied as `relative_path<TAB>d|f<TAB>size<TAB>mtime_epoch_seconds` lines
//!   (root is `.`). This only reproduces metadata; content is never hydrated.
//!   The manifest mode exists so measurement runs can use a real names/layout
//!   distribution instead of the synthetic shape.
#![allow(unexpected_cfgs)]
use drive_core::{Node, NodeMetadata, ReadHandle, Store};
use drive_fs::{CacheFilesystem, Hydrator};
use std::{
    collections::HashSet,
    fs::File,
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
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
/// Test-only scale bound: a typo cannot allocate an unbounded tree.
const MAX_ENTRIES: usize = 1_000_000;

fn synthetic_entries(
    groups: usize,
    files: usize,
    wide: usize,
) -> Result<Vec<(String, Node)>, Box<dyn std::error::Error>> {
    let total_entries = groups
        .checked_mul(files.saturating_add(1))
        .and_then(|n| n.checked_add(wide))
        .and_then(|n| n.checked_add(130))
        .ok_or("fixture entry count overflow")?;
    if total_entries > MAX_ENTRIES || groups > 100_000 {
        return Err(format!("fixture budget exceeded: {total_entries} entries / {groups} groups").into());
    }
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
    Ok(entries)
}

/// Read a real namespace manifest. The caller passes the metadata only; no file
/// is opened, so an on-demand mount can serve the naming/layout distribution of
/// an existing tree without transferring content.
fn manifest_entries(manifest: &Path) -> Result<Vec<(String, Node)>, Box<dyn std::error::Error>> {
    let file = File::open(manifest)
        .map_err(|e| format!("cannot open manifest {}: {e}", manifest.display()))?;
    let mut entries = Vec::<(String, Node)>::new();
    let mut seen = HashSet::<String>::new();
    let mut root_seen = false;
    for (index, line) in BufReader::new(file).lines().enumerate() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let mut fields = line.split('\t');
        let (Some(path), Some(kind), Some(size), Some(mtime)) = (
            fields.next(),
            fields.next(),
            fields.next(),
            fields.next(),
        ) else {
            return Err(format!("manifest line {} is not 4 tab-separated fields", index + 1).into());
        };
        if fields.next().is_some() {
            return Err(format!("manifest line {} has extra fields", index + 1).into());
        }
        let directory = match kind {
            "d" => true,
            "f" => false,
            other => return Err(format!("manifest line {}: unknown kind {other:?}", index + 1).into()),
        };
        let (rel, id, parent, name) = if path == "." {
            if root_seen {
                return Err("duplicate root entry".into());
            }
            root_seen = true;
            (String::new(), "root".to_string(), None, "root".to_string())
        } else {
            if path.is_empty() || path.starts_with('/') || path.ends_with('/') || path.contains("//") {
                return Err(format!("manifest line {}: invalid path {path:?}", index + 1).into());
            }
            let (parent, name) = match path.rsplit_once('/') {
                Some((parent, name)) => (parent.to_string(), name.to_string()),
                None => ("root".to_string(), path.to_string()),
            };
            if name.is_empty() {
                return Err(format!("manifest line {}: empty name", index + 1).into());
            }
            (path.to_string(), path.to_string(), Some(parent), name)
        };
        if !seen.insert(id.clone()) {
            return Err(format!("manifest line {}: duplicate path {path:?}", index + 1).into());
        }
        let size = size.parse::<u64>()?;
        let mtime = mtime.parse::<u64>()?;
        entries.push((
            rel,
            Node {
                id,
                parent,
                name,
                version: "v1".into(),
                directory,
                metadata: Some(NodeMetadata {
                    size: if directory { 0 } else { size },
                    mtime,
                    mode: if directory { 0o755 } else { 0o644 },
                }),
            },
        ));
        if entries.len() > MAX_ENTRIES {
            return Err(format!("manifest exceeds {MAX_ENTRIES} entries").into());
        }
    }
    if !root_seen {
        return Err("manifest has no root entry (.)".into());
    }
    Ok(entries)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    let manifest_mode = args.get(2).is_some_and(|mode| mode == "fuse-manifest");
    if manifest_mode {
        if args.len() != 4 {
            return Err("usage: fsearch_fixture RUN_DIR fuse-manifest MANIFEST.tsv".into());
        }
    } else if args.len() != 6 {
        return Err(
            "usage: fsearch_fixture RUN_DIR fuse|fuse-writable|local GROUPS FILES_PER_GROUP WIDE_FILES"
                .into(),
        );
    }
    let run = PathBuf::from(&args[1]);
    let local = !manifest_mode && args[2] == "local";
    let writable = !manifest_mode && args[2] == "fuse-writable";
    if !manifest_mode && !local && !writable && args[2] != "fuse" {
        return Err("invalid mode".into());
    }
    if run.exists() {
        return Err("fixture destination must be new".into());
    }
    let entries = if manifest_mode {
        manifest_entries(Path::new(&args[3]))?
    } else {
        synthetic_entries(
            args[3].parse::<usize>()?,
            args[4].parse::<usize>()?,
            args[5].parse::<usize>()?,
        )?
    };
    std::fs::create_dir_all(&run)?;
    let mount = run.join("mount");
    std::fs::create_dir(&mount)?;
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
            if manifest_mode { "manifest" } else { "synthetic" },
        )?;
        let shared = Arc::new(drive_core::SharedStore::new(store));
        // Metadata caching is active only after the session notifier is
        // installed; the TTL comes from DRIVE_NATIVE_FS_TTL_MS (1 s default,
        // 0 disables caching).
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
        let fsname = if manifest_mode {
            "drive-native-fsearch-manifest"
        } else {
            "drive-native-fsearch-synthetic"
        };
        let mounted = fuser::spawn_mount2(
            fs,
            &mount,
            &[
                fuser::MountOption::DefaultPermissions,
                fuser::MountOption::NoDev,
                fuser::MountOption::NoSuid,
                fuser::MountOption::FSName(fsname.into()),
            ],
        )?;
        notify.install(mounted.notifier());
        session = Some(mounted);
    }
    emit(
        serde_json::json!({"ready":true,"mode":args[2],"manifest":manifest_mode,"files":entries.iter().filter(|(_,n)|!n.directory).count(),"directories_including_root":entries.iter().filter(|(_,n)|n.directory).count()}),
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
