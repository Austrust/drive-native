use drive_core::{directories::DirectoryOrigin, upload_origin::UploadOrigin, *};
fn fixture() -> (tempfile::TempDir, Store, String, UploadOrigin) {
    let dir = tempfile::tempdir().unwrap();
    let mut s = Store::open(dir.path()).unwrap();
    let root = Node {
        id: "1".into(),
        parent: None,
        name: "lab".into(),
        version: "root-v1".into(),
        directory: true,
        metadata: None,
    };
    s.install_remote_snapshot(&[root], "native-v1:10", "server")
        .unwrap();
    s.bind_native_scope("server", r#"{"path":"/lab"}"#).unwrap();
    s.ensure_fs_root("1").unwrap();
    let node = s.create_local_file("1", "A", 0o644).unwrap();
    let op = s.save_local(&node.id, b"local").unwrap();
    s.creation_saved(&node.id).unwrap();
    let origin = UploadOrigin {
        schema: 1,
        node: node.id,
        parent_file: "1".into(),
        path: "/lab/A".into(),
        sha256: sha256(b"local"),
        boundary: DirectoryOrigin {
            cursor: "native-v1:10".into(),
            committer: "a".repeat(64),
            restore: "server".into(),
        },
    };
    (dir, s, op, origin)
}
#[test]
fn provenance_survives_restart_before_and_after_submission() {
    let (dir, mut s, op, origin) = fixture();
    s.prepare_upload_origin(&op, &origin, &origin.sha256)
        .unwrap();
    drop(s);
    let mut s = Store::open(dir.path()).unwrap();
    assert_eq!(s.upload_origin(&op).unwrap(), Some(origin.clone()));
    assert_eq!(s.upload_job(&op).unwrap().state, "pending");
    s.mark_upload_started(&op).unwrap();
    drop(s);
    let mut s = Store::open(dir.path()).unwrap();
    assert_eq!(s.upload_job(&op).unwrap().state, "uncertain");
    assert_eq!(s.upload_origin(&op).unwrap(), Some(origin.clone()));
    assert!(
        s.prepare_upload_origin(&op, &origin, &origin.sha256)
            .is_err()
    );
}
#[test]
fn different_path_content_identity_or_superseded_save_cannot_reuse_provenance() {
    let (_dir, mut s, op, origin) = fixture();
    for change in 0..3 {
        let mut bad = origin.clone();
        match change {
            0 => bad.node = "local:other".into(),
            1 => bad.path = "/lab/B".into(),
            _ => bad.sha256 = "f".repeat(64),
        };
        assert!(s.prepare_upload_origin(&op, &bad, &origin.sha256).is_err());
    }
    let _new = s.save_local(&origin.node, b"newer").unwrap();
    assert!(
        s.prepare_upload_origin(&op, &origin, &origin.sha256)
            .is_err()
    );
}
#[test]
fn schema_12_upgrade_preserves_unknown_writes_without_fabricating_origin() {
    let (dir, mut s, op, origin) = fixture();
    s.mark_upload_started(&op).unwrap();
    drop(s);
    let db = rusqlite::Connection::open(dir.path().join("state.sqlite")).unwrap();
    db.execute_batch("DROP TABLE upload_origins;PRAGMA user_version=12;")
        .unwrap();
    drop(db);
    let s = Store::open(dir.path()).unwrap();
    assert_eq!(s.upload_job(&op).unwrap().state, "uncertain");
    assert!(s.upload_origin(&op).unwrap().is_none());
    assert!(s.open_cached(&origin.node).is_ok());
    assert!(std::fs::read_dir(dir.path()).unwrap().any(|e| {
        e.unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with(&format!(
                "state-before-v{}-",
                drive_core::STATE_SCHEMA_VERSION
            ))
    }));
}
