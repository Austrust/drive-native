//! Issue #50 phase 4 (R5): retiring a local creation must also drop the staging
//! metadata of every upload operation it supersedes, otherwise
//! `upload_paths`/`upload_origins` grow orphaned rows across permission-uncertain
//! uploads.
use super::retire_creation_tx;
use crate::Store;

fn open() -> (tempfile::TempDir, Store) {
    let dir = tempfile::tempdir().expect("tempdir");
    let store = Store::open(dir.path()).expect("open store");
    store
        .db
        .execute_batch(
            "INSERT INTO nodes(id,parent,name,remote_version,directory) VALUES('root',NULL,'root','v1',1);
             INSERT INTO nodes(id,parent,name,remote_version,directory) VALUES('file','root','A.txt','v1',0);",
        )
        .expect("fixture nodes");
    (dir, store)
}

fn upload(store: &Store, operation: &str, node: &str, state: &str) {
    store
        .db
        .execute(
            "INSERT INTO operations(id,node,kind,base,generation,state) VALUES(?1,?2,'upload','v1',1,?3)",
            rusqlite::params![operation, node, state],
        )
        .expect("operation");
    store
        .db
        .execute(
            "INSERT INTO upload_paths(operation,path) VALUES(?1,?2)",
            rusqlite::params![operation, format!("/{node}")],
        )
        .expect("upload path");
    store
        .db
        .execute(
            "INSERT INTO upload_origins(operation,body) VALUES(?1,?2)",
            rusqlite::params![operation, format!(r#"{{"id":"{node}"}}"#)],
        )
        .expect("upload origin");
}

fn scalar(store: &Store, sql: &str) -> i64 {
    store
        .db
        .query_row(sql, [], |row| row.get(0))
        .expect("scalar")
}

#[test]
fn retiring_a_creation_drops_staging_rows_of_every_superseded_upload() {
    let (_dir, store) = open();
    upload(&store, "op-abandoned", "file", "uncertain");
    // A local edit after the uncertain commit queued a second upload which the
    // tombstone supersedes as well.
    upload(&store, "op-pending", "file", "pending");

    retire_creation_tx(&store.db, "op-abandoned", "file", "permission_denied")
        .expect("retire creation");

    assert_eq!(
        scalar(
            &store,
            "SELECT count(*) FROM operations WHERE node='file' AND state='superseded'",
        ),
        2,
        "both uploads are retired"
    );
    assert_eq!(
        scalar(&store, "SELECT count(*) FROM upload_paths"),
        0,
        "no orphaned upload path rows"
    );
    assert_eq!(
        scalar(&store, "SELECT count(*) FROM upload_origins"),
        0,
        "no orphaned upload origin rows"
    );
    assert_eq!(
        scalar(&store, "SELECT deleted FROM nodes WHERE id='file'"),
        1,
        "the retired node is a local tombstone"
    );
}

#[test]
fn retiring_a_creation_keeps_unrelated_upload_staging_rows() {
    let (_dir, store) = open();
    store
        .db
        .execute(
            "INSERT INTO nodes(id,parent,name,remote_version,directory) VALUES('other','root','B.txt','v1',0)",
            [],
        )
        .expect("other node");
    upload(&store, "op-abandoned", "file", "uncertain");
    upload(&store, "op-other", "other", "pending");

    retire_creation_tx(&store.db, "op-abandoned", "file", "local_deleted").expect("retire");

    assert_eq!(
        scalar(&store, "SELECT count(*) FROM upload_paths"),
        1,
        "another node's staging row is untouched"
    );
    assert_eq!(
        scalar(
            &store,
            "SELECT count(*) FROM upload_paths WHERE operation='op-other'",
        ),
        1
    );
    assert_eq!(
        scalar(
            &store,
            "SELECT count(*) FROM operations WHERE id='op-other' AND state='pending'",
        ),
        1
    );
}
