//! Issue #50 write-path evidence: create, mkdir, rename, unlink, write and
//! setattr against excluded paths answer ENOENT and never enqueue a local
//! operation. The mounted tests need `/dev/fuse` and an ordinary Linux user, so
//! they are `#[ignore]`d and run explicitly like the other kernel regressions.
use super::*;
use drive_core::exclude::{ExcludeMode, ExcludeRules, ExclusionState};
use drive_core::NodeMetadata;
use std::io::Write;
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

fn open_fs(
    nodes: &[Node],
    patterns: &[&str],
) -> (tempfile::TempDir, Arc<drive_core::SharedStore>, CacheFilesystem) {
    open_fs_mode(nodes, patterns, ExcludeMode::Hidden)
}

fn open_fs_mode(
    nodes: &[Node],
    patterns: &[&str],
    mode: ExcludeMode,
) -> (tempfile::TempDir, Arc<drive_core::SharedStore>, CacheFilesystem) {
    let temp = tempfile::tempdir().unwrap();
    let mut store = Store::open(temp.path().join("state")).unwrap();
    store
        .install_remote_snapshot(nodes, "1", "synthetic")
        .unwrap();
    store.ensure_fs_root(ROOT).unwrap();
    store.ensure_filesystem_inodes(ROOT).unwrap();
    let store = Arc::new(drive_core::SharedStore::new(store));
    let state = Arc::new(ExclusionState::with_mode(
        ExcludeRules::compile(patterns).expect("valid test rules"),
        mode,
    ));
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
    .with_writes_enabled()
    .with_exclusion_state(state);
    (temp, store, fs)
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
            fuser::MountOption::FSName("drive-native-exclude-write-test".into()),
        ],
    )
    .unwrap();
    (mount, session)
}

fn assert_enoent<T>(result: std::io::Result<T>, what: &str) {
    let error = result.err().unwrap_or_else(|| panic!("{what} must fail"));
    assert_eq!(
        error.raw_os_error(),
        Some(libc::ENOENT),
        "{what} must answer ENOENT, got {error:?}"
    );
}

#[test]
fn local_only_mode_allows_write_guards_and_marks_quiesce_targets() {
    let (_temp, store, fs) = open_fs_mode(
        &snapshot(),
        &[r"^Pinned$", r"\.iso$"],
        ExcludeMode::LocalOnly,
    );
    let (pinned_dir, root_ino) = {
        let guard = store.lock().unwrap();
        (
            guard.lookup(ROOT, "Pinned").unwrap().ino,
            guard.record(ROOT).unwrap().ino,
        )
    };
    assert!(!fs.inode_excluded(pinned_dir));
    assert!(!fs.entry_excluded(root_ino, "Pinned"));
    assert!(!fs.child_excluded(root_ino, "blocked.iso", false));
    assert!(!fs.child_excluded(root_ino, "Pinned", true));
    // The write side still resolves the same paths for the local-only quiesce.
    assert!(fs.local_only_inode(pinned_dir));
    assert!(fs.local_only_child(root_ino, "blocked.iso", false));
    assert!(!fs.local_only_child(root_ino, "A.txt", false));
}

/// Issue #56B: writes under a `local_only` rule succeed, stay readable and
/// queue no upload/namespace operation.
#[test]
#[ignore = "requires ordinary Linux user and /dev/fuse"]
fn local_only_write_paths_stay_local_without_operations() {
    let (temp, store, fs) = open_fs_mode(&snapshot(), &[r"^Pinned$"], ExcludeMode::LocalOnly);
    let (mount, session) = mount_at(&temp, fs);
    std::fs::write(mount.join("Pinned/new.txt"), b"local").unwrap();
    assert_eq!(
        std::fs::read(mount.join("Pinned/new.txt")).unwrap(),
        b"local"
    );
    let (id, pending) = {
        let s = store.lock().unwrap();
        (
            s.lookup("300", "new.txt").expect("local node").node.id,
            s.upload_operations("pending").unwrap(),
        )
    };
    assert!(!id.is_empty());
    assert!(
        pending.is_empty(),
        "local_only write must not queue an upload: {pending:?}"
    );
    assert_eq!(
        store.lock().unwrap().pending_namespace_operations().unwrap(),
        0
    );
    drop(session);
}

#[test]
#[ignore = "requires ordinary Linux user and /dev/fuse"]
fn excluded_write_paths_answer_enoent_without_local_operations() {
    let (temp, store, fs) = open_fs(&snapshot(), &[r"^Pinned$", r"\.iso$"]);
    let (mount, _session) = mount_at(&temp, fs);

    assert_enoent(std::fs::write(mount.join("blocked.iso"), b"x"), "create file");
    assert_enoent(
        std::fs::create_dir(mount.join("Pinned/newdir")),
        "mkdir below excluded dir",
    );
    assert_enoent(
        std::fs::write(mount.join("Pinned/new.txt"), b"x"),
        "create inside excluded dir",
    );
    assert_enoent(
        std::fs::rename(mount.join("A.txt"), mount.join("blocked.iso")),
        "rename into excluded name",
    );
    assert_enoent(
        std::fs::rename(
            mount.join("Pinned/keep.txt"),
            mount.join("renamed.txt"),
        ),
        "rename out of excluded dir",
    );
    assert_enoent(
        std::fs::remove_file(mount.join("Pinned/keep.txt")),
        "unlink inside excluded dir",
    );
    assert_enoent(
        std::fs::remove_file(mount.join("blocked.iso")),
        "unlink excluded name",
    );

    // The excluded view never leaked into the authoritative Store: the snapshot
    // nodes remain untouched (and hidden), while no candidate/operation for the
    // rejected writes exists.
    let guard = store.lock().unwrap();
    assert!(guard.lookup(ROOT, "blocked.iso").is_err());
    assert!(guard.lookup(ROOT, "newdir").is_err());
    assert!(guard.lookup(ROOT, "new.txt").is_err());
    assert!(guard.lookup(ROOT, "A.txt").is_ok());
    assert!(guard.node("300").is_ok(), "excluded snapshot node is hidden, not deleted");
    assert_eq!(guard.pending_operations().unwrap(), 0);
    assert!(guard.downloads_needed().unwrap().is_empty());
}

#[test]
#[ignore = "requires ordinary Linux user and /dev/fuse"]
fn empty_rules_keep_the_unfiltered_write_baseline() {
    let (temp, store, fs) = open_fs(&snapshot(), &[]);
    let (mount, _session) = mount_at(&temp, fs);

    let mut file = std::fs::File::create(mount.join("ok.txt")).unwrap();
    file.write_all(b"hello").unwrap();
    drop(file);

    std::fs::create_dir(mount.join("okdir")).unwrap();

    assert_eq!(std::fs::read(mount.join("ok.txt")).unwrap(), b"hello");
    assert!(mount.join("okdir").is_dir());

    let guard = store.lock().unwrap();
    assert!(guard.lookup(ROOT, "ok.txt").is_ok());
    assert!(guard.lookup(ROOT, "okdir").is_ok());
}
