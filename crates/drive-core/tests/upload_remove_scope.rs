//! Upload start must depend on remove intents that own its target, not on any
//! unrelated pending remove (issue #6: freedesktop trash blocked every upload).
use drive_core::removals::RemoveJob;
use drive_core::*;

fn n(id: &str, parent: Option<&str>, name: &str, version: &str, directory: bool) -> Node {
    Node {
        id: id.into(),
        parent: parent.map(str::to_owned),
        name: name.into(),
        version: version.into(),
        directory,
        metadata: Some(NodeMetadata {
            size: if directory { 0 } else { 3 },
            mtime: 1,
            mode: if directory { 0o755 } else { 0o644 },
        }),
    }
}
fn fixture() -> (tempfile::TempDir, Store) {
    let d = tempfile::tempdir().unwrap();
    let mut s = Store::open(d.path()).unwrap();
    s.install_remote_snapshot(
        &[
            n("1", None, "root", "r1", true),
            n("2", Some("1"), "A.doc", "v-2", false),
        ],
        "1",
        "server",
    )
    .unwrap();
    s.ensure_fs_root("1").unwrap();
    s.publish_download(
        &DownloadTicket {
            id: "2".into(),
            version: "v-2".into(),
        },
        b"old",
        &sha256(b"old"),
    )
    .unwrap();
    (d, s)
}

/// The freedesktop trash directory is a local file-manager artifact. A remove
/// pending inside it is independent of ordinary uploads at other paths.
#[test]
fn unrelated_trash_remove_does_not_block_ordinary_upload() {
    let (_d, mut s) = fixture();
    // A file manager created the freedesktop trash tree inside the mount.
    s.install_remote_snapshot(
        &[
            n("1", None, "root", "r1", true),
            n("2", Some("1"), "A.doc", "v-2", false),
            n("3", Some("1"), ".Trash-1000", "t-3", true),
            n("4", Some("3"), "files", "f-4", true),
            n("5", Some("4"), "junk.bin", "j-5", false),
        ],
        "2",
        "server",
    )
    .unwrap();
    s.unlink_local("4", "junk.bin", "5", false).unwrap();
    let remove = s.next_remove().unwrap().unwrap();
    assert_eq!(remove.job.path, ".Trash-1000/files/junk.bin");
    assert_eq!(remove.state, "pending");

    let normal = s.create_local_file("1", "normal.doc", 0o644).unwrap();
    s.creation_saved(&normal.id).unwrap();
    let upload = s.upload_operations("pending").unwrap().remove(0);
    s.mark_upload_started(&upload).unwrap();
    assert_eq!(s.upload_job(&upload).unwrap().relative_path, "normal.doc");
    assert_eq!(
        s.next_remove().unwrap().unwrap().job.operation,
        remove.job.operation,
        "the unrelated remove intent must stay queued"
    );
}

/// Re-creating a deleted name keeps the ordering dependency: the server must
/// see the remove before the replacement upload.
#[test]
fn recreated_same_name_upload_still_waits_for_pending_remove() {
    let (_d, mut s) = fixture();
    s.unlink_local("1", "A.doc", "2", false).unwrap();
    let replacement = s.create_local_file("1", "A.doc", 0o644).unwrap();
    s.creation_saved(&replacement.id).unwrap();
    let upload = s.upload_operations("pending").unwrap().remove(0);
    let blocked = s.mark_upload_started(&upload).unwrap_err();
    assert!(
        blocked.to_string().contains("pending remove"),
        "unexpected barrier: {blocked}"
    );

    let remove = s.next_remove().unwrap().unwrap();
    s.mark_remove_started(&remove.job.operation).unwrap();
    s.confirm_remove(&remove.job.operation, "2").unwrap();
    s.mark_upload_started(&upload).unwrap();
}

/// An uncertain remove owns the target exactly as a pending one does.
#[test]
fn uncertain_same_path_remove_still_blocks_upload() {
    let (_d, mut s) = fixture();
    s.unlink_local("1", "A.doc", "2", false).unwrap();
    let replacement = s.create_local_file("1", "A.doc", 0o644).unwrap();
    s.creation_saved(&replacement.id).unwrap();
    let upload = s.upload_operations("pending").unwrap().remove(0);
    let remove = s.next_remove().unwrap().unwrap();
    s.mark_remove_started(&remove.job.operation).unwrap();
    s.mark_remove_uncertain(&remove.job.operation).unwrap();
    assert!(s.mark_upload_started(&upload).is_err());
    s.confirm_remove(&remove.job.operation, "2").unwrap();
    s.mark_upload_started(&upload).unwrap();
}

/// Deleted ancestors remain dependencies even without a namespace overlay and
/// even when the journal captured a now-stale path.
#[test]
fn deleted_ancestor_remove_blocks_upload_by_identity() {
    let (d, mut s) = fixture();
    s.install_remote_snapshot(
        &[
            n("1", None, "root", "r1", true),
            n("2", Some("1"), "A.doc", "v-2", false),
            n("3", Some("1"), "folder", "d-3", true),
            n("4", Some("3"), "child.doc", "v-4", false),
        ],
        "2",
        "server",
    )
    .unwrap();
    s.publish_download(
        &DownloadTicket {
            id: "4".into(),
            version: "v-4".into(),
        },
        b"kid",
        &sha256(b"kid"),
    )
    .unwrap();
    let upload = s.save_local("4", b"edited").unwrap();
    // Queue an authoritative remove for the still-present ancestor. The alias
    // path is deliberately stale so only identity ancestry can catch this.
    let db = rusqlite::Connection::open(d.path().join("state.sqlite")).unwrap();
    let job = RemoveJob {
        operation: "remove-ancestor".into(),
        node: n("3", Some("1"), "folder", "d-3", true),
        path: "old-folder".into(),
        cursor: "native-v1:9".into(),
    };
    db.execute(
        "INSERT INTO namespace_ops(id,node,body,state,kind) VALUES(?1,?2,?3,'pending','remove')",
        rusqlite::params![
            job.operation,
            job.node.id,
            serde_json::to_string(&job).unwrap()
        ],
    )
    .unwrap();
    drop(db);
    let blocked = s.mark_upload_started(&upload).unwrap_err();
    assert!(
        blocked.to_string().contains("pending remove"),
        "unexpected barrier: {blocked}"
    );
}
