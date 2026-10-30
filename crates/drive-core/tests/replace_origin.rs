use drive_core::{commit_origin::CommitOrigin, upload_origin::UploadOrigin, *};
fn fixture() -> (
    tempfile::TempDir,
    Store,
    replacements::ReplaceJob,
    UploadOrigin,
) {
    let d = tempfile::tempdir().unwrap();
    let mut s = Store::open(d.path()).unwrap();
    s.install_remote_snapshot(
        &[Node {
            id: "1".into(),
            parent: None,
            name: "lab".into(),
            version: "root".into(),
            directory: true,
            metadata: None,
        }],
        "native-v1:10",
        "server",
    )
    .unwrap();
    s.bind_native_scope("server", r#"{"path":"/lab"}"#).unwrap();
    s.ensure_fs_root("1").unwrap();
    let source = s.create_local_file("1", "Temp", 0o644).unwrap();
    s.save_local(&source.id, b"new").unwrap();
    s.creation_saved(&source.id).unwrap();
    let target = s.create_local_file("1", "Target", 0o644).unwrap();
    s.save_local(&target.id, b"old").unwrap();
    s.creation_saved(&target.id).unwrap();
    s.replace_local("1", "Temp", "1", "Target", &source.id, &target.id)
        .unwrap();
    let job = s.next_replace().unwrap().unwrap().job;
    let origin = UploadOrigin {
        schema: 1,
        node: target.id,
        parent_file: "1".into(),
        path: "/lab/Target".into(),
        sha256: sha256(b"new"),
        boundary: CommitOrigin {
            cursor: "native-v1:10".into(),
            committer: "a".repeat(64),
            restore: "server".into(),
        },
    };
    (d, s, job, origin)
}
#[test]
fn replacement_origin_is_durable_and_cannot_change_after_submission() {
    let (d, mut s, job, origin) = fixture();
    let inode = s.record(&job.source.id).unwrap().ino;
    s.prepare_replace_origin(&job.operation, origin.clone(), &origin.sha256)
        .unwrap();
    drop(s);
    let mut s = Store::open(d.path()).unwrap();
    assert_eq!(
        s.next_replace().unwrap().unwrap().job.creation_origin,
        Some(origin.clone())
    );
    s.mark_replace_started(&job.operation).unwrap();
    drop(s);
    let mut s = Store::open(d.path()).unwrap();
    let q = s.next_replace().unwrap().unwrap();
    assert_eq!(q.state, "uncertain");
    assert_eq!(q.job.creation_origin, Some(origin.clone()));
    assert!(
        s.prepare_replace_origin(&job.operation, origin.clone(), &origin.sha256)
            .is_err()
    );
    assert_eq!(s.lookup("1", "Target").unwrap().ino, inode);
    assert_eq!(s.collect_garbage().unwrap(), 0);
}
#[test]
fn replacement_provenance_rejects_different_node_path_hash_and_queue_head() {
    let (_d, mut s, job, origin) = fixture();
    for index in 0..4 {
        let mut bad = origin.clone();
        let mut operation = job.operation.clone();
        match index {
            0 => bad.node = job.source.id.clone(),
            1 => bad.path = "/lab/Temp".into(),
            2 => bad.sha256 = "b".repeat(64),
            _ => operation = "other".into(),
        };
        assert!(
            s.prepare_replace_origin(&operation, bad, &origin.sha256)
                .is_err()
        );
    }
    assert!(
        s.next_replace()
            .unwrap()
            .unwrap()
            .job
            .creation_origin
            .is_none()
    );
}
#[test]
fn schema_13_uncertain_replace_keeps_content_without_inventing_provenance() {
    let (d, mut s, job, origin) = fixture();
    s.mark_replace_started(&job.operation).unwrap();
    drop(s);
    let db = rusqlite::Connection::open(d.path().join("state.sqlite")).unwrap();
    db.execute(
        "UPDATE namespace_ops SET body=json_remove(body,'$.creation_origin') WHERE id=?1",
        [&job.operation],
    )
    .unwrap();
    db.execute_batch("PRAGMA user_version=13;").unwrap();
    drop(db);
    let mut s = Store::open(d.path()).unwrap();
    let q = s.next_replace().unwrap().unwrap();
    assert_eq!(q.state, "uncertain");
    assert!(q.job.creation_origin.is_none());
    assert!(
        s.prepare_replace_origin(&job.operation, origin.clone(), &origin.sha256)
            .is_err()
    );
    let mut contents = Vec::new();
    use std::io::Read;
    s.open_cached(&job.source.id)
        .unwrap()
        .read_to_end(&mut contents)
        .unwrap();
    assert_eq!(contents, b"new");
    assert!(std::fs::read_dir(d.path()).unwrap().any(|e| {
        e.unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with(&format!("state-before-v{}-", STATE_SCHEMA_VERSION))
    }));
}
