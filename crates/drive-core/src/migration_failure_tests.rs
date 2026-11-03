//! Regression: a failure inside the schema 23 -> 24 migration transaction must
//! roll back to the previous schema and leave local data untouched; a retry must
//! then migrate cleanly. Uses the test-only migration seam in `state.rs`.
use super::{STATE_SCHEMA_VERSION, inject_migration_failure};
use crate::write_permissions::WritePermission;
use crate::{DownloadTicket, Node, NodeMetadata, Store, sha256};
use std::io::Read;

fn node(id: &str, parent: Option<&str>, name: &str, version: &str, directory: bool) -> Node {
    Node {
        id: id.into(),
        parent: parent.map(str::to_owned),
        name: name.into(),
        version: version.into(),
        directory,
        metadata: Some(NodeMetadata {
            size: if directory { 0 } else { 3 },
            mtime: 1,
            mode: if directory { 0o755 } else { 0o600 },
        }),
    }
}

fn state_db(root: &std::path::Path) -> rusqlite::Connection {
    rusqlite::Connection::open(root.join("state.sqlite")).unwrap()
}

#[test]
fn failing_schema_23_24_migration_rolls_back_and_retries_cleanly() {
    let d = tempfile::tempdir().unwrap();
    let mut store = Store::open(d.path()).unwrap();
    store
        .install_remote_snapshot(
            &[
                node("root", None, "root", "1", true),
                node("file", Some("root"), "A.doc", "v1", false),
            ],
            "cursor1",
            "server1",
        )
        .unwrap();
    store
        .publish_download(
            &DownloadTicket {
                id: "file".into(),
                version: "v1".into(),
            },
            b"old",
            &sha256(b"old"),
        )
        .unwrap();
    store.pin("file", true).unwrap();
    let operation = store.save_local("file", b"offline changes").unwrap();
    drop(store);

    // Record durable local state, then shape the database like schema 23.
    let (blob, pending_state, pending_bytes, pinned_before) = {
        let db = state_db(d.path());
        let (blob, state): (String, String) = db
            .query_row("SELECT blob,state FROM operations WHERE id=?1", [&operation], |r| {
                Ok((r.get(0)?, r.get(1)?))
            })
            .unwrap();
        let pinned: bool = db
            .query_row("SELECT pinned FROM nodes WHERE id='file'", [], |r| r.get(0))
            .unwrap();
        db.execute_batch(
            "DROP TABLE directory_permissions;
             DELETE FROM metadata WHERE key='write_policy_revision';
             PRAGMA user_version=23;",
        )
        .unwrap();
        let version: i64 = db
            .query_row("PRAGMA user_version", [], |r| r.get(0))
            .unwrap();
        assert_eq!(version, 23, "fixture must simulate schema 23");
        let table: i64 = db
            .query_row(
                "SELECT count(*) FROM sqlite_master WHERE type='table' AND name='directory_permissions'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(table, 0);
        (blob, state, b"offline changes".to_vec(), pinned)
    };
    assert_eq!(pending_state, "pending");
    assert!(pinned_before);
    let blob_path = d.path().join("blobs").join(&blob);
    assert_eq!(std::fs::read(&blob_path).unwrap(), pending_bytes);

    // Inject a real SQL constraint failure inside the migration transaction.
    inject_migration_failure();
    let error = match Store::open(d.path()) {
        Ok(_) => panic!("injected migration must fail"),
        Err(error) => error,
    };
    assert!(
        matches!(error, crate::Error::Db(_)),
        "unexpected error: {error}"
    );

    // The half-applied migration must be gone: no version bump, no policy table,
    // no revision key, and local pending/pin/content untouched.
    {
        let db = state_db(d.path());
        let version: i64 = db
            .query_row("PRAGMA user_version", [], |r| r.get(0))
            .unwrap();
        assert_eq!(version, 23, "failed migration must not advance schema version");
        let policy_table: i64 = db
            .query_row(
                "SELECT count(*) FROM sqlite_master WHERE type='table' AND name='directory_permissions'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(policy_table, 0, "half-created policy table must roll back");
        let revision: i64 = db
            .query_row(
                "SELECT count(*) FROM metadata WHERE key='write_policy_revision'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(revision, 0, "migration metadata key must roll back");
        let (stored_blob, state): (String, String) = db
            .query_row("SELECT blob,state FROM operations WHERE id=?1", [&operation], |r| {
                Ok((r.get(0)?, r.get(1)?))
            })
            .unwrap();
        assert_eq!((stored_blob.as_str(), state.as_str()), (blob.as_str(), "pending"));
        let pinned: bool = db
            .query_row("SELECT pinned FROM nodes WHERE id='file'", [], |r| r.get(0))
            .unwrap();
        assert!(pinned, "pin must survive the failed migration");
    }
    assert_eq!(
        std::fs::read(&blob_path).unwrap(),
        pending_bytes,
        "pending body must survive the failed migration"
    );

    // A pre-migration backup was still produced (same migration contract).
    let backup = std::fs::read_dir(d.path())
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .find(|path| {
            path.file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("state-before-v24-")
        })
        .expect("migration backup must exist");
    {
        let db = rusqlite::Connection::open(&backup).unwrap();
        let version: i64 = db
            .query_row("PRAGMA user_version", [], |r| r.get(0))
            .unwrap();
        assert_eq!(version, 23);
        let integrity: String = db
            .query_row("PRAGMA integrity_check", [], |r| r.get(0))
            .unwrap();
        assert_eq!(integrity, "ok");
    }

    // Retry without the seam must migrate cleanly and preserve the same state.
    let mut store = Store::open(d.path()).expect("retry after rollback must migrate");
    assert_eq!(store.upload_job(&operation).unwrap().state, "pending");
    assert!(store.is_pinned("file").unwrap());
    let mut bytes = Vec::new();
    store
        .open_cached("file")
        .unwrap()
        .read_to_end(&mut bytes)
        .unwrap();
    assert_eq!(bytes, pending_bytes);
    assert_eq!(store.write_policy_revision().unwrap(), 0);
    assert_eq!(
        store.effective_write_permission("file").unwrap().permission,
        WritePermission::Unknown
    );
    drop(store);

    let db = state_db(d.path());
    let version: i64 = db
        .query_row("PRAGMA user_version", [], |r| r.get(0))
        .unwrap();
    assert_eq!(version, STATE_SCHEMA_VERSION);
    let decisions: i64 = db
        .query_row("SELECT count(*) FROM directory_permissions", [], |r| r.get(0))
        .unwrap();
    assert_eq!(decisions, 0, "migration must not invent write decisions");
}
