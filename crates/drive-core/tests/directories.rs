use drive_core::*;
fn root() -> Node {
    Node {
        id: "1".into(),
        parent: None,
        name: "root".into(),
        version: "root1".into(),
        directory: true,
        metadata: Some(NodeMetadata {
            size: 0,
            mtime: 1,
            mode: 0o755,
        }),
    }
}
fn fixture() -> (tempfile::TempDir, Store) {
    let d = tempfile::tempdir().unwrap();
    let mut s = Store::open(d.path()).unwrap();
    s.install_remote_snapshot(&[root()], "1", "server").unwrap();
    s.ensure_fs_root("1").unwrap();
    (d, s)
}
fn confirm(s: &mut Store, remote: &str) {
    let q = s.next_directory().unwrap().unwrap();
    let mut job = s.mark_directory_started(&q.job.operation).unwrap();
    job.node.version = format!("dir-{remote}");
    s.confirm_directory(&q.job.operation, &job.node, remote)
        .unwrap();
}
#[test]
fn nested_offline_directories_keep_inode_and_block_children_until_confirmed() {
    let (d, mut s) = fixture();
    let a = s.create_local_directory("1", "A", 0o750).unwrap();
    let inode = s.record(&a.id).unwrap().ino;
    let b = s.create_local_directory(&a.id, "B", 0o755).unwrap();
    let file = s.create_local_file(&b.id, "file", 0o644).unwrap();
    let op = s.save_local(&file.id, b"offline child").unwrap();
    s.creation_saved(&file.id).unwrap();
    assert!(s.mark_upload_started(&op).is_err());
    assert_eq!(s.status(&a.id).unwrap(), Status::PendingUpload);
    drop(s);
    let mut s = Store::open(d.path()).unwrap();
    assert_eq!(s.record(&a.id).unwrap().ino, inode);
    confirm(&mut s, "10");
    assert!(s.mark_upload_started(&op).is_err());
    let q = s.next_directory().unwrap().unwrap();
    let job = s.mark_directory_started(&q.job.operation).unwrap();
    assert_eq!(job.parent.version, "dir-10");
    let mut confirmed = job.node;
    confirmed.version = "dir-11".into();
    s.confirm_directory(&q.job.operation, &confirmed, "11")
        .unwrap();
    assert!(s.next_namespace_kind().unwrap().is_none());
    s.mark_upload_started(&op).unwrap();
    assert_eq!(s.upload_job(&op).unwrap().relative_path, "A/B/file");
    assert_eq!(s.local_identity("10").unwrap(), a.id);
    assert_eq!(s.local_identity("11").unwrap(), b.id);
}
#[test]
fn create_rename_remove_preserves_order_and_does_not_replay_unknown_mkdir() {
    let (d, mut s) = fixture();
    let a = s.create_local_directory("1", "A", 0o755).unwrap();
    s.rename_local("1", "A", "1", "B", false).unwrap();
    s.unlink_local("1", "B", &a.id, true).unwrap();
    let q = s.next_directory().unwrap().unwrap();
    s.mark_directory_started(&q.job.operation).unwrap();
    drop(s);
    let mut s = Store::open(d.path()).unwrap();
    let q = s.next_directory().unwrap().unwrap();
    assert_eq!(q.state, "uncertain");
    assert!(s.mark_directory_started(&q.job.operation).is_err());
    let mut confirmed = q.job.node;
    confirmed.version = "d1".into();
    s.confirm_directory(&q.job.operation, &confirmed, "10")
        .unwrap();
    let r = s.next_rename().unwrap().unwrap();
    assert_eq!(r.job.node.version, "d1");
    s.mark_rename_started(&r.job.operation).unwrap();
    let mut moved = confirmed;
    moved.name = "B".into();
    moved.version = "d2".into();
    s.confirm_rename(&r.job.operation, &moved, true).unwrap();
    let r = s.next_remove().unwrap().unwrap();
    assert_eq!(r.job.node.version, "d2");
    assert_eq!(r.job.path, "B");
    s.mark_remove_started(&r.job.operation).unwrap();
    s.confirm_remove(&r.job.operation, &a.id).unwrap();
    assert!(s.children("1").unwrap().is_empty());
}
#[test]
fn directory_early_echo_folds_children_without_losing_local_inode() {
    let (_d, mut s) = fixture();
    let a = s.create_local_directory("1", "A", 0o755).unwrap();
    let inode = s.record(&a.id).unwrap().ino;
    let q = s.next_directory().unwrap().unwrap();
    s.mark_directory_started(&q.job.operation).unwrap();
    let mut echo = a.clone();
    echo.id = "10".into();
    echo.version = "d1".into();
    let child = Node {
        id: "20".into(),
        parent: Some("10".into()),
        name: "remote-child".into(),
        version: "v1".into(),
        directory: false,
        metadata: Some(NodeMetadata {
            size: 5,
            mtime: 1,
            mode: 0o644,
        }),
    };
    s.receive(&Batch {
        from: "1".into(),
        to: "2".into(),
        events: vec![
            Event {
                id: "a".into(),
                change: Change::Upsert(echo),
            },
            Event {
                id: "b".into(),
                change: Change::Upsert(child),
            },
        ],
    })
    .unwrap();
    s.drain().unwrap();
    let mut confirmed = a.clone();
    confirmed.version = "d1".into();
    s.confirm_directory(&q.job.operation, &confirmed, "10")
        .unwrap();
    assert_eq!(s.lookup("1", "A").unwrap().ino, inode);
    assert_eq!(s.children(&a.id).unwrap().len(), 1);
    assert!(s.node("10").is_err());
}
#[test]
fn moving_existing_file_into_unconfirmed_parent_uses_confirmed_parent_version() {
    let (_d, mut s) = fixture();
    let file = Node {
        id: "20".into(),
        parent: Some("1".into()),
        name: "file".into(),
        version: "v1".into(),
        directory: false,
        metadata: Some(NodeMetadata {
            size: 3,
            mtime: 1,
            mode: 0o644,
        }),
    };
    s.install_remote_snapshot(&[root(), file], "2", "server")
        .unwrap();
    let a = s.create_local_directory("1", "A", 0o755).unwrap();
    s.rename_local("1", "file", &a.id, "file", false).unwrap();
    confirm(&mut s, "10");
    let q = s.next_rename().unwrap().unwrap();
    assert_eq!(q.job.node.version, "v1");
    assert_eq!(q.job.parent_node.version, "dir-10");
    assert_eq!(q.job.destination, "A/file");
}
#[test]
fn mkdir_type_collision_and_invalid_name_are_rejected_before_journaling() {
    let (_d, mut s) = fixture();
    s.create_local_file("1", "occupied", 0o644).unwrap();
    assert!(s.create_local_directory("1", "occupied", 0o755).is_err());
    assert!(s.create_local_directory("1", "../escape", 0o755).is_err());
    assert!(s.next_namespace_kind().unwrap().is_none());
}

#[cfg(feature = "native")]
fn version(sync: u64, max: u64) -> String {
    serde_json::json!({"schema":1,"restore":"server","view":54,"root":"1","file":"10","sync":sync,"max":max,"size":0,"hash":"","directory":true}).to_string()
}
#[test]
#[cfg(feature = "native")]
fn directory_subtree_event_rebases_moves_but_not_pending_deletion() {
    let (_d, mut s) = fixture();
    let mut dir = root();
    dir.id = "10".into();
    dir.parent = Some("1".into());
    dir.name = "A".into();
    dir.version = version(2, 2);
    s.install_remote_snapshot(&[root(), dir.clone()], "2", "server")
        .unwrap();
    s.rename_local("1", "A", "1", "B", false).unwrap();
    s.rename_local("1", "B", "1", "C", false).unwrap();
    s.unlink_local("1", "C", "10", true).unwrap();
    dir.version = version(2, 3);
    s.receive(&Batch {
        from: "2".into(),
        to: "3".into(),
        events: vec![Event {
            id: "advance".into(),
            change: Change::Upsert(dir.clone()),
        }],
    })
    .unwrap();
    s.drain().unwrap();
    let q = s.next_rename().unwrap().unwrap();
    s.mark_rename_started(&q.job.operation).unwrap();
    let q = s.next_rename().unwrap().unwrap();
    assert_eq!(q.job.node.version, version(2, 3));
    dir.name = "B".into();
    dir.version = version(4, 4);
    s.confirm_rename(&q.job.operation, &dir, true).unwrap();
    let q = s.next_rename().unwrap().unwrap();
    assert_eq!(q.job.node.version, version(4, 4));
    s.mark_rename_started(&q.job.operation).unwrap();
    dir.name = "C".into();
    dir.version = version(5, 5);
    s.confirm_rename(&q.job.operation, &dir, true).unwrap();
    // Unseen children still protect rmdir: the original deletion intent never
    // inherits a broader subtree baseline merely from a directory move refresh.
    assert_eq!(
        s.next_remove().unwrap().unwrap().job.node.version,
        version(2, 2)
    );
}
#[test]
#[cfg(feature = "native")]
fn pre_mutation_directory_observation_is_durable_and_rejects_own_revision_changes() {
    let (d, mut s) = fixture();
    let mut dir = root();
    dir.id = "10".into();
    dir.parent = Some("1".into());
    dir.name = "A".into();
    dir.version = version(2, 2);
    s.install_remote_snapshot(&[root(), dir.clone()], "2", "server")
        .unwrap();
    s.rename_local("1", "A", "1", "B", false).unwrap();
    let q = s.next_rename().unwrap().unwrap();
    s.mark_rename_started(&q.job.operation).unwrap();
    dir.version = version(3, 3);
    assert!(s.refresh_directory_rename(&q.job.operation, &dir).is_err());
    dir.version = version(2, 3);
    s.refresh_directory_rename(&q.job.operation, &dir).unwrap();
    drop(s);
    let mut s = Store::open(d.path()).unwrap();
    let q = s.next_rename().unwrap().unwrap();
    assert_eq!(q.state, "pending");
    assert_eq!(q.job.node.version, version(2, 3));
    assert_eq!(s.lookup("1", "B").unwrap().node.id, "10");
    s.mark_rename_started(&q.job.operation).unwrap();
}

#[test]
fn prepared_author_survives_restart_and_cannot_be_replaced_after_uncertainty() {
    let (d, mut s) = fixture();
    let dir = s.create_local_directory("1", "prepared", 0o755).unwrap();
    let q = s.next_directory().unwrap().unwrap();
    s.mark_directory_started(&q.job.operation).unwrap();
    let origin = drive_core::directories::DirectoryOrigin {
        cursor: "native-v1:7".into(),
        committer: "a".repeat(64),
        restore: "r".into(),
    };
    s.prepare_directory(&q.job.operation, origin.clone())
        .unwrap();
    let job = s.mark_directory_started(&q.job.operation).unwrap();
    assert_eq!(job.origin, Some(origin.clone()));
    drop(s);
    let mut s = Store::open(d.path()).unwrap();
    let q = s.next_directory().unwrap().unwrap();
    assert_eq!(q.state, "uncertain");
    assert_eq!(q.job.origin, Some(origin.clone()));
    assert!(s.prepare_directory(&q.job.operation, origin).is_err());
    assert_eq!(s.lookup("1", "prepared").unwrap().node.id, dir.id);
}
#[test]
fn legacy_uncertain_job_cannot_gain_provenance_after_schema_upgrade() {
    let (d, mut s) = fixture();
    s.create_local_directory("1", "legacy", 0o755).unwrap();
    let q = s.next_directory().unwrap().unwrap();
    s.mark_directory_started(&q.job.operation).unwrap();
    drop(s);
    let db = rusqlite::Connection::open(d.path().join("state.sqlite")).unwrap();
    let body: String = db
        .query_row(
            "SELECT body FROM namespace_ops WHERE id=?1",
            [&q.job.operation],
            |r| r.get(0),
        )
        .unwrap();
    let mut value: serde_json::Value = serde_json::from_str(&body).unwrap();
    for key in ["creation_contract", "origin", "cursor"] {
        value.as_object_mut().unwrap().remove(key);
    }
    db.execute("UPDATE namespace_ops SET body=?1", [value.to_string()])
        .unwrap();
    db.pragma_update(None, "user_version", 10).unwrap();
    drop(db);
    let mut s = Store::open(d.path()).unwrap();
    let q = s.next_directory().unwrap().unwrap();
    assert_eq!(q.job.creation_contract, 0);
    assert_eq!(q.state, "uncertain");
    let origin = drive_core::directories::DirectoryOrigin {
        cursor: "native-v1:7".into(),
        committer: "a".repeat(64),
        restore: "r".into(),
    };
    assert!(s.prepare_directory(&q.job.operation, origin).is_err());
    assert!(std::fs::read_dir(d.path()).unwrap().any(|f| {
        f.unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with(&format!(
                "state-before-v{}-",
                drive_core::STATE_SCHEMA_VERSION
            ))
    }));
}

#[test]
fn lost_read_only_preparation_can_be_repeated_after_restart() {
    let (d, mut s) = fixture();
    s.create_local_directory("1", "read-only", 0o755).unwrap();
    let q = s.next_directory().unwrap().unwrap();
    s.mark_directory_started(&q.job.operation).unwrap();
    drop(s);
    let mut s = Store::open(d.path()).unwrap();
    let q = s.next_directory().unwrap().unwrap();
    assert_eq!(q.state, "uncertain");
    assert!(q.job.origin.is_none());
    s.prepare_directory(
        &q.job.operation,
        drive_core::directories::DirectoryOrigin {
            cursor: "native-v1:7".into(),
            committer: "a".repeat(64),
            restore: "r".into(),
        },
    )
    .unwrap();
    assert_eq!(s.next_directory().unwrap().unwrap().state, "pending");
}
