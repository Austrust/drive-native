//! Offline synthetic fixture for the ORIGINAL FSearch application. No NAS or
//! protocol worker is configured. Local mode is a separate comparison filesystem,
//! never a surrogate for the Drive mount being indexed.
//!
//! Two generators are supported:
//! * `GROUPS FILES_PER_GROUP WIDE_FILES`: deterministic synthetic tree.
//! * `fuse-manifest|local-manifest MANIFEST.tsv`: a saved namespace manifest,
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
const MAX_ENTRIES: usize = 500_000;

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
        if path.contains('\0') || (path != "." && path.split('/').any(|part| matches!(part, "" | "." | ".."))) {
            return Err(format!("manifest line {}: unsafe relative path", index + 1).into());
        }
        let (rel, id, parent, name) = if path == "." {
            if root_seen {
                return Err("duplicate root entry".into());
            }
            if !directory { return Err("manifest root must be a directory".into()); }
            root_seen = true;
            (String::new(), "root".to_string(), None, "root".to_string())
        } else {
            if path.is_empty() || path.starts_with('/') || path.ends_with('/') || path.contains("//") {
                return Err(format!("manifest line {}: invalid path {path:?}", index + 1).into());
            }
            let (parent, name) = match path.rsplit_once('/') {
                Some((parent, name)) => (format!("manifest:{parent}"), name.to_string()),
                None => ("root".to_string(), path.to_string()),
            };
            if name.is_empty() {
                return Err(format!("manifest line {}: empty name", index + 1).into());
            }
            (path.to_string(), format!("manifest:{path}"), Some(parent), name)
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
    let directories = entries.iter().filter(|(_, node)| node.directory)
        .map(|(_, node)| node.id.as_str()).collect::<HashSet<_>>();
    if entries.iter().any(|(_, node)| node.parent.as_ref().is_some_and(|id| !directories.contains(id.as_str()))) {
        return Err("manifest has a missing or non-directory parent".into());
    }
    // Local references are built parent-first even for an unordered manifest.
    entries.sort_by(|a, b| a.0.matches('/').count().cmp(&b.0.matches('/').count()).then(a.0.cmp(&b.0)));
    Ok(entries)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    let manifest_mode = args.get(2).is_some_and(|mode| mode == "fuse-manifest" || mode == "local-manifest");
    if manifest_mode {
        if args.len() != 4 {
            return Err("usage: fsearch_fixture RUN_DIR fuse-manifest|local-manifest MANIFEST.tsv".into());
        }
    } else if args.len() != 6 {
        return Err(
            "usage: fsearch_fixture RUN_DIR fuse|fuse-writable|local GROUPS FILES_PER_GROUP WIDE_FILES"
                .into(),
        );
    }
    let run = PathBuf::from(&args[1]);
    let local = args[2] == "local" || args[2] == "local-manifest";
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
    let mut index_status = serde_json::json!({"index_enabled":false});
    if local {
        for (path, node) in &entries {
            if path.is_empty() {
                continue;
            }
            let target = mount.join(path);
            if node.directory {
                std::fs::create_dir(&target)?;
            } else {
                // A truthful sparse local reference, never an empty stand-in
                // for the separately mounted Drive filesystem under test.
                std::fs::File::create(&target)?.set_len(node.metadata.as_ref().unwrap().size)?;
            }
        }
        for (path, node) in entries.iter().rev() {
            std::fs::File::open(mount.join(path))?.set_times(
                std::fs::FileTimes::new()
                    .set_modified(UNIX_EPOCH + Duration::from_secs(node.metadata.as_ref().unwrap().mtime)),
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
        let mut fs = CacheFilesystem::new(shared.clone(), "root", unsafe { libc::getuid() }, unsafe {
            libc::getgid()
        })?
        .with_hydrator(no_network.clone())
        .with_notify_hub(notify.clone());
        // The exact same fixture source can build against .9 by supplying
        // --cfg drive_fs_baseline; it then omits only the unavailable NSI API.
        #[cfg(not(drive_fs_baseline))]
        if drive_fs::namespace_index_enabled() {
            let started = std::time::Instant::now();
            let index = { let guard = shared.lock().unwrap();
                drive_core::namespace_index::NamespaceIndex::build(&guard) };
            match index {
                Ok(index) => {
                    index_status = serde_json::json!({"index_enabled":true,
                        "index_degraded":index.is_degraded(),"index_partial":index.is_partial(),
                        "index_bytes":index.bytes(),"index_build_us":started.elapsed().as_micros()});
                    fs = fs.with_namespace_index(Arc::new(index));
                }
                Err(_) => index_status = serde_json::json!({"index_enabled":true,"index_degraded":true,
                    "index_build_us":started.elapsed().as_micros()}),
            }
        }
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
        serde_json::json!({"ready":true,"mode":args[2],"manifest":manifest_mode,"namespace_index":index_status,"files":entries.iter().filter(|(_,n)|!n.directory).count(),"directories_including_root":entries.iter().filter(|(_,n)|n.directory).count()}),
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

#[cfg(test)]
mod manifest_tests {
    use super::*;
    fn parse(text: &str) -> Result<Vec<(String, Node)>, Box<dyn std::error::Error>> {
        let temp = tempfile::tempdir()?;
        let path = temp.path().join("manifest.tsv");
        std::fs::write(&path, text)?;
        manifest_entries(&path)
    }
    #[test]
    fn manifest_preserves_sizes_times_and_orders_parents_first() {
        let entries = parse("nested/file\tf\t987654321\t1700000123\n.\td\t0\t1700000000\nnested\td\t0\t1700000001\nroot\tf\t17\t1700000002\n").unwrap();
        assert_eq!(entries[0].0, "");
        let file = entries.iter().find(|(path, _)| path == "nested/file").unwrap();
        let metadata = file.1.metadata.as_ref().unwrap();
        assert_eq!((metadata.size, metadata.mtime), (987654321, 1700000123));
        assert_ne!(entries.iter().find(|(path, _)| path == "root").unwrap().1.id, "root");
    }
    #[test]
    fn manifest_rejects_escape_components_before_creating_any_reference() {
        for path in ["../outside", "a/../../outside", "./file", "/absolute", "a//file"] {
            assert!(parse(&format!(".\td\t0\t1\n{path}\tf\t4\t1\n")).is_err());
        }
    }
    #[test]
    fn manifest_requires_real_directory_parents_and_root() {
        assert!(parse(".\tf\t4\t1\n").is_err());
        assert!(parse(".\td\t0\t1\na/file\tf\t4\t1\n").is_err());
        assert!(parse(".\td\t0\t1\na\tf\t4\t1\na/file\tf\t4\t1\n").is_err());
    }
}
