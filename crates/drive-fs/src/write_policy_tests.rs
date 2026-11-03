//! Write gating matrix for explicit read-only mounts and per-directory
//! permissions. Synthetic SQLite only; no mount or network.
use super::*;
use drive_core::write_permissions::WritePermission;
use drive_core::{Node, NodeMetadata};

fn node(id: &str, parent: Option<&str>, directory: bool) -> Node {
    Node {
        id: id.into(),
        parent: parent.map(str::to_owned),
        name: id.into(),
        version: "1".into(),
        directory,
        metadata: Some(NodeMetadata {
            size: if directory { 0 } else { 1 },
            mtime: 1,
            mode: if directory { 0o755 } else { 0o644 },
        }),
    }
}
fn fixture() -> (
    tempfile::TempDir,
    Arc<drive_core::SharedStore>,
    CacheFilesystem,
) {
    let uid = unsafe { libc::getuid() };
    assert_ne!(uid, 0);
    let temp = tempfile::tempdir().unwrap();
    let mut store = Store::open(temp.path().join("state")).unwrap();
    store
        .install_remote_snapshot(
            &[
                node("root", None, true),
                node("sub", Some("root"), true),
                node("file", Some("sub"), false),
                node("plain", Some("root"), false),
            ],
            "1",
            "synthetic",
        )
        .unwrap();
    let store = Arc::new(drive_core::SharedStore::new(store));
    let fs = CacheFilesystem::new(store.clone(), "root", uid, unsafe { libc::getgid() }).unwrap();
    (temp, store, fs)
}
fn ino(store: &Arc<drive_core::SharedStore>, id: &str) -> u64 {
    store.lock().unwrap().record(id).unwrap().ino
}
fn set(store: &Arc<drive_core::SharedStore>, node: &str, permission: WritePermission) {
    store
        .lock()
        .unwrap()
        .set_directory_permission(node, permission, "test")
        .unwrap();
}

#[test]
fn explicit_read_only_mount_keeps_erofs_for_every_target() {
    let (_d, store, fs) = fixture();
    for id in ["root", "sub", "file", "plain"] {
        assert_eq!(
            fs.write_denial(ino(&store, id)),
            Some(libc::EROFS),
            "node {id}"
        );
    }
}

#[test]
fn per_directory_mount_denies_unknown_and_honours_ancestor_decisions() {
    let (_d, store, fs) = fixture();
    let fs = fs.with_write_policy(WritePolicy::PerDirectory {
        fallback: WritePermission::Denied,
    });
    // Unknown is conservative: a read-only remote scope grants nothing.
    assert_eq!(fs.write_denial(ino(&store, "file")), Some(libc::EACCES));
    assert_eq!(fs.write_denial(ino(&store, "plain")), Some(libc::EACCES));

    set(&store, "root", WritePermission::Allowed);
    assert_eq!(fs.write_denial(ino(&store, "file")), None);
    assert_eq!(fs.write_denial(ino(&store, "plain")), None);

    set(&store, "sub", WritePermission::Denied);
    assert_eq!(fs.write_denial(ino(&store, "file")), Some(libc::EACCES));
    assert_eq!(fs.write_denial(ino(&store, "plain")), None);

    // A nearer allow wins over the denied ancestor.
    set(&store, "sub", WritePermission::Allowed);
    assert_eq!(fs.write_denial(ino(&store, "file")), None);
}

#[test]
fn per_request_reads_apply_permission_changes_without_remount() {
    let (_d, store, fs) = fixture();
    let fs = fs.with_write_policy(WritePolicy::PerDirectory {
        fallback: WritePermission::Denied,
    });
    let file = ino(&store, "file");
    assert_eq!(fs.write_denial(file), Some(libc::EACCES));
    let before = store.lock().unwrap().write_policy_revision().unwrap();
    set(&store, "root", WritePermission::Allowed);
    assert_eq!(fs.write_denial(file), None);
    assert!(store.lock().unwrap().write_policy_revision().unwrap() > before);
    set(&store, "root", WritePermission::Denied);
    assert_eq!(fs.write_denial(file), Some(libc::EACCES));
    fs.store
        .lock()
        .unwrap()
        .clear_directory_permission("root")
        .unwrap();
    assert_eq!(fs.write_denial(file), Some(libc::EACCES));
}

#[test]
fn writable_fixture_default_allows_until_explicit_denial() {
    let (_d, store, fs) = fixture();
    let fs = fs.with_writes_enabled();
    assert_eq!(fs.write_denial(ino(&store, "file")), None);
    set(&store, "sub", WritePermission::Denied);
    assert_eq!(fs.write_denial(ino(&store, "file")), Some(libc::EACCES));
    // Explicit unknown is a decision, not a missing one: conservative deny.
    set(&store, "sub", WritePermission::Unknown);
    assert_eq!(fs.write_denial(ino(&store, "file")), Some(libc::EACCES));
    assert_eq!(fs.write_denial(ino(&store, "plain")), None);
}

#[test]
fn unresolved_target_fails_closed() {
    let (_d, _store, fs) = fixture();
    let fs = fs.with_write_policy(WritePolicy::PerDirectory {
        fallback: WritePermission::Allowed,
    });
    assert_eq!(fs.write_denial(u64::MAX), Some(libc::EACCES));
}
