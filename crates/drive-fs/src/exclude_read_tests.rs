//! Issue #50 read-path evidence: excluded paths and their subtrees answer
//! ENOENT, stay out of readdir, and the inode index follows namespace rebuilds.
//!
//! The mounted tests need `/dev/fuse` and an ordinary Linux user, so they are
//! `#[ignore]`d like the other kernel regressions and are run explicitly.
use super::*;
use drive_core::exclude::{ExcludeRules, ExclusionState};
use drive_core::NodeMetadata;
use std::sync::Arc;

const ROOT: &str = "100";

fn node(id: &str, parent: Option<&str>, name: &str, directory: bool) -> Node {
    Node {
        id: id.into(),
        parent: parent.map(str::to_owned),
        name: name.into(),
        directory,
        version: "1".into(),
        metadata: Some(NodeMetadata {
            size: if directory { 0 } else { 4 },
            mtime: 1,
            mode: if directory { 0o755 } else { 0o644 },
        }),
    }
}

fn snapshot() -> Vec<Node> {
    vec![
        node(ROOT, None, "Lab", true),
        node("101", Some(ROOT), "A.txt", false),
        node("102", Some(ROOT), "install.iso", false),
        node("200", Some(ROOT), "node_modules", true),
        node("201", Some("200"), "left-pad", true),
        node("202", Some("201"), "index.js", false),
        node("300", Some(ROOT), "Pinned", true),
        node("301", Some("300"), "keep.txt", false),
    ]
}

fn state(patterns: &[&str]) -> Arc<ExclusionState> {
    Arc::new(ExclusionState::new(
        ExcludeRules::compile(patterns).expect("valid test rules"),
    ))
}

fn open_fs(
    nodes: &[Node],
    patterns: &[&str],
) -> (
    tempfile::TempDir,
    Arc<drive_core::SharedStore>,
    Arc<ExclusionState>,
    CacheFilesystem,
) {
    let temp = tempfile::tempdir().unwrap();
    let mut store = Store::open(temp.path().join("state")).unwrap();
    store
        .install_remote_snapshot(nodes, "1", "synthetic")
        .unwrap();
    store.ensure_fs_root(ROOT).unwrap();
    store.ensure_filesystem_inodes(ROOT).unwrap();
    let store = Arc::new(drive_core::SharedStore::new(store));
    let state = state(patterns);
    if !patterns.is_empty() {
        let guard = store.lock().unwrap();
        state.refresh(&guard, ROOT).unwrap();
    }
    let fs = CacheFilesystem::new(
        store.clone(),
        ROOT,
        unsafe { libc::getuid() },
        unsafe { libc::getgid() },
    )
    .unwrap()
    .with_exclusion_state(state.clone());
    (temp, store, state, fs)
}

fn mount_at(
    temp: &tempfile::TempDir,
    fs: CacheFilesystem,
) -> (std::path::PathBuf, fuser::BackgroundSession) {
    let mount = temp.path().join("mnt");
    std::fs::create_dir(&mount).unwrap();
    let session = fuser::spawn_mount2(
        fs,
        &mount,
        &[
            fuser::MountOption::DefaultPermissions,
            fuser::MountOption::NoDev,
            fuser::MountOption::NoSuid,
            fuser::MountOption::FSName("drive-native-exclude-read-test".into()),
        ],
    )
    .unwrap();
    (mount, session)
}

fn names(dir: &std::path::Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(dir)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

fn assert_missing(path: &std::path::Path) {
    let error = std::fs::metadata(path).unwrap_err();
    assert_eq!(
        error.kind(),
        std::io::ErrorKind::NotFound,
        "excluded path must answer ENOENT: {}",
        path.display()
    );
}

/// The inode helpers consult the current namespace index; this runs without a
/// kernel mount so the derivation is covered by the ordinary test suite.
#[test]
fn excluded_inode_and_entry_helpers_follow_the_namespace() {
    let (_temp, store, _state, fs) = open_fs(&snapshot(), &[r"(^|/)node_modules/", r"\.iso$"]);
    let (hidden_dir, hidden_file, visible, root_ino) = {
        let guard = store.lock().unwrap();
        (
            guard.lookup(ROOT, "node_modules").unwrap().ino,
            guard.lookup(ROOT, "install.iso").unwrap().ino,
            guard.lookup(ROOT, "A.txt").unwrap().ino,
            guard.record_by_inode(1).unwrap().ino,
        )
    };
    assert!(fs.inode_excluded(hidden_dir));
    assert!(fs.inode_excluded(hidden_file));
    assert!(!fs.inode_excluded(visible));
    assert!(!fs.entry_excluded(root_ino, "A.txt"));
    assert!(fs.entry_excluded(root_ino, "node_modules"));
    assert!(fs.exclusion_index().is_some());
}

#[test]
fn empty_rules_keep_the_unfiltered_fast_path() {
    let (_temp, _store, state, fs) = open_fs(&snapshot(), &[]);
    assert!(!state.is_active());
    assert!(fs.exclusion_index().is_none());
    assert!(!fs.entry_excluded(1, "node_modules"));
}

#[test]
#[ignore = "requires an ordinary Linux user and /dev/fuse; run explicitly"]
fn excluded_paths_answer_enoent_and_stay_out_of_readdir() {
    let (temp, _store, _state, fs) = open_fs(&snapshot(), &[r"(^|/)node_modules/", r"\.iso$"]);
    let (mount, session) = mount_at(&temp, fs);
    // Visible content is still served...
    assert_eq!(names(&mount), ["A.txt", "Pinned"]);
    assert!(std::fs::metadata(mount.join("A.txt")).is_ok());
    // ...while excluded entries and every descendant answer ENOENT.
    assert_missing(&mount.join("install.iso"));
    assert_missing(&mount.join("node_modules"));
    assert_missing(&mount.join("node_modules/left-pad"));
    assert_missing(&mount.join("node_modules/left-pad/index.js"));
    assert_eq!(
        std::fs::read_dir(mount.join("node_modules"))
            .unwrap_err()
            .kind(),
        std::io::ErrorKind::NotFound
    );
    drop(session);
}

#[test]
#[ignore = "requires an ordinary Linux user and /dev/fuse; run explicitly"]
fn readdir_without_rules_lists_the_full_namespace() {
    let (temp, _store, _state, fs) = open_fs(&snapshot(), &[]);
    let (mount, session) = mount_at(&temp, fs);
    assert_eq!(
        names(&mount),
        ["A.txt", "Pinned", "install.iso", "node_modules"]
    );
    assert!(std::fs::metadata(mount.join("node_modules/left-pad/index.js")).is_ok());
    drop(session);
}

#[test]
#[ignore = "requires an ordinary Linux user and /dev/fuse; run explicitly"]
fn mount_then_initialization_snapshot_refresh_keeps_excluded_paths_hidden() {
    // The mount starts from an empty namespace (only the scope root exists), like
    // a first mount before the initialization owner has installed its snapshot.
    let root_only = vec![node(ROOT, None, "Lab", true)];
    let (temp, store, state, fs) = open_fs(&root_only, &[r"(^|/)node_modules/", r"\.iso$"]);
    let (mount, session) = mount_at(&temp, fs);
    assert_eq!(names(&mount), Vec::<String>::new());

    // Initialization installs the namespace after the mount; the exclusion index
    // must be refreshed from the new namespace (Lead acceptance requirement).
    {
        let mut guard = store.lock().unwrap();
        guard
            .install_remote_snapshot(&snapshot(), "2", "synthetic")
            .unwrap();
        guard.ensure_fs_root(ROOT).unwrap();
        guard.ensure_filesystem_inodes(ROOT).unwrap();
        state.refresh(&guard, ROOT).unwrap();
    }
    assert_eq!(names(&mount), ["A.txt", "Pinned"]);
    assert_missing(&mount.join("install.iso"));
    assert_missing(&mount.join("node_modules/left-pad/index.js"));
    drop(session);
}
