//! #28 evidence: an unfinished-remove backlog must not make the upload barrier
//! scan every queued intent. The bench builds a realistic queue (one durable
//! pending remove intent per deleted node, with the serialized `RemoveJob` body)
//! and measures the barrier for an unrelated upload target.
use super::*;
use crate::{Node, Store};
use std::time::{Duration, Instant};

fn fixture(removes: usize) -> (tempfile::TempDir, Store) {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(dir.path()).unwrap();
    store
        .db
        .execute(
            "INSERT INTO nodes(id,parent,name,remote_version,directory) VALUES('root',NULL,'root','v1',1)",
            [],
        )
        .unwrap();
    let tx = store.db.unchecked_transaction().unwrap();
    for i in 0..removes {
        let id = format!("n{i:05}");
        let name = format!("removed-{i:05}");
        tx.execute(
            "INSERT INTO nodes(id,parent,name,remote_version,directory) VALUES(?1,'root',?2,'v1',0)",
            rusqlite::params![id, name],
        )
        .unwrap();
        let job = RemoveJob {
            operation: format!("op{i:05}"),
            node: Node {
                id: id.clone(),
                parent: Some("root".to_owned()),
                name: name.clone(),
                version: "v1".to_owned(),
                directory: false,
                metadata: None,
            },
            path: name,
            cursor: String::new(),
        };
        tx.execute(
            "INSERT INTO namespace_ops(id,node,body,state,kind) VALUES(?1,?2,?3,'pending','remove')",
            rusqlite::params![job.operation, id, serde_json::to_string(&job).unwrap()],
        )
        .unwrap();
    }
    // The upload target never overlaps a queued remove.
    tx.execute(
        "INSERT INTO nodes(id,parent,name,remote_version,directory) VALUES('target','root','target.doc','v1',0)",
        [],
    )
    .unwrap();
    tx.commit().unwrap();
    (dir, store)
}

fn target_barrier(store: &mut Store) -> Option<UploadRemoveBarrier> {
    store.upload_remove_barrier("target", "target.doc").unwrap()
}

fn median(mut samples: Vec<Duration>) -> Duration {
    samples.sort();
    samples[samples.len() / 2]
}

#[test]
fn remove_barrier_cost_does_not_scale_with_the_remove_backlog() {
    let (_small_dir, mut small) = fixture(1_000);
    let (_large_dir, mut large) = fixture(10_000);
    let warm_small = Instant::now();
    assert!(target_barrier(&mut small).is_none());
    let warm_small = warm_small.elapsed();
    let warm_large = Instant::now();
    assert!(target_barrier(&mut large).is_none());
    let warm_large = warm_large.elapsed();
    let sample = |store: &mut Store| {
        (0..7)
            .map(|_| {
                let start = Instant::now();
                assert!(target_barrier(store).is_none());
                start.elapsed()
            })
            .collect::<Vec<_>>()
    };
    let small_ms = median(sample(&mut small));
    let large_ms = median(sample(&mut large));
    eprintln!(
        "#28 BENCH backlog_1000={:.3}ms backlog_10000={:.3}ms ratio={:.1}",
        small_ms.as_secs_f64() * 1e3,
        large_ms.as_secs_f64() * 1e3,
        large_ms.as_secs_f64() / small_ms.as_secs_f64().max(1e-9)
    );
    let (calls, rebuilds, last_us, rows) = large.remove_barrier_stats();
    eprintln!(
        "#28 REBUILD first_1000={:.3}ms first_10000={:.3}ms",
        warm_small.as_secs_f64() * 1e3,
        warm_large.as_secs_f64() * 1e3
    );
    eprintln!("#28 STAT large calls={calls} rebuilds={rebuilds} last_us={last_us} rows={rows}");
    assert!(
        large_ms < small_ms * 3 && large_ms < Duration::from_millis(2),
        "remove barrier grows with the backlog: 1k={small_ms:?} 10k={large_ms:?}"
    );
}

#[test]
fn remove_barrier_answers_follow_the_queued_removes() {
    let (_dir, mut store) = fixture(0);
    let insert_node = |store: &Store, id: &str, parent: &str, name: &str, directory: bool| {
        store
            .db
            .execute(
                "INSERT INTO nodes(id,parent,name,remote_version,directory) VALUES(?1,?2,?3,'v1',?4)",
                rusqlite::params![id, parent, name, directory],
            )
            .unwrap();
    };
    let queue = |store: &Store, operation: &str, node: &str, parent: &str, name: &str| {
        let job = RemoveJob {
            operation: operation.to_owned(),
            node: Node {
                id: node.to_owned(),
                parent: Some(parent.to_owned()),
                name: name.to_owned(),
                version: "v1".to_owned(),
                directory: true,
                metadata: None,
            },
            path: name.to_owned(),
            cursor: String::new(),
        };
        store
            .db
            .execute(
                "INSERT INTO namespace_ops(id,node,body,state,kind) VALUES(?1,?2,?3,'pending','remove')",
                rusqlite::params![operation, node, serde_json::to_string(&job).unwrap()],
            )
            .unwrap();
    };
    // A queued removal of an ancestor directory blocks uploads below it by
    // identity, independently of captured paths.
    insert_node(&store, "dir", "root", "dir", true);
    insert_node(&store, "leaf", "dir", "leaf.doc", false);
    queue(&store, "op-dir", "dir", "root", "dir");
    let blocked = store
        .upload_remove_barrier("leaf", "dir/leaf.doc")
        .unwrap()
        .expect("removal of the ancestor directory blocks the upload");
    assert_eq!(blocked.operation, "op-dir");

    // A permission-blocked operation is not part of the barrier.
    store
        .db
        .execute("INSERT INTO access_denied(node) VALUES('dir')", [])
        .unwrap();
    assert!(store
        .upload_remove_barrier("leaf", "dir/leaf.doc")
        .unwrap()
        .is_none());
    store.db.execute("DELETE FROM access_denied", []).unwrap();
    assert!(store
        .upload_remove_barrier("leaf", "dir/leaf.doc")
        .unwrap()
        .is_some());

    // Committing the queued remove releases the barrier.
    store
        .db
        .execute(
            "UPDATE namespace_ops SET state='committed' WHERE id='op-dir'",
            [],
        )
        .unwrap();
    assert!(store
        .upload_remove_barrier("leaf", "dir/leaf.doc")
        .unwrap()
        .is_none());
}

#[test]
fn queued_remove_paths_follow_ancestor_renames() {
    let (_dir, mut store) = fixture(0);
    store
        .db
        .execute(
            "INSERT INTO nodes(id,parent,name,remote_version,directory) VALUES('dir','root','dir','v1',1)",
            [],
        )
        .unwrap();
    store
        .db
        .execute(
            "INSERT INTO nodes(id,parent,name,remote_version,directory) VALUES('removed','dir','leaf.doc','v1',0)",
            [],
        )
        .unwrap();
    store
        .db
        .execute(
            "INSERT INTO nodes(id,parent,name,remote_version,directory) VALUES('uploaddir','root','uploaddir','v1',1)",
            [],
        )
        .unwrap();
    store
        .db
        .execute(
            "INSERT INTO nodes(id,parent,name,remote_version,directory) VALUES('uploadleaf','uploaddir','leaf.doc','v1',0)",
            [],
        )
        .unwrap();
    let job = RemoveJob {
        operation: "op-leaf".to_owned(),
        node: Node {
            id: "removed".to_owned(),
            parent: Some("dir".to_owned()),
            name: "leaf.doc".to_owned(),
            version: "v1".to_owned(),
            directory: false,
            metadata: None,
        },
        path: "dir/leaf.doc".to_owned(),
        cursor: String::new(),
    };
    store
        .db
        .execute(
            "INSERT INTO namespace_ops(id,node,body,state,kind) VALUES('op-leaf','removed',?1,'pending','remove')",
            [serde_json::to_string(&job).unwrap()],
        )
        .unwrap();
    assert!(store
        .upload_remove_barrier("uploadleaf", "uploaddir/leaf.doc")
        .unwrap()
        .is_none());
    // Renaming the queued remove's parent rebinds its live path so the same
    // upload now overlaps it.
    store
        .db
        .execute("UPDATE nodes SET name='uploaddir' WHERE id='dir'", [])
        .unwrap();
    let rebound = store
        .upload_remove_barrier("uploadleaf", "uploaddir/leaf.doc")
        .unwrap()
        .expect("rebound queued remove owns the upload path");
    assert_eq!(rebound.operation, "op-leaf");
    assert_eq!(rebound.path, "uploaddir/leaf.doc");
}

#[test]
fn barrier_cache_tracks_bulk_commits_with_a_large_backlog() {
    let (_dir, mut store) = fixture(10_000);
    // Warm the cache over the full backlog.
    assert!(target_barrier(&mut store).is_none());
    // Completing one queued remove releases its node.
    store
        .db
        .execute(
            "UPDATE namespace_ops SET state='committed' WHERE id='op05000'",
            [],
        )
        .unwrap();
    assert!(store
        .upload_remove_barrier("n05000", "removed-05000")
        .unwrap()
        .is_none());
    // A fresh intent for the same path must be observed incrementally.
    let job = RemoveJob {
        operation: "op-new".to_owned(),
        node: Node {
            id: "n05000".to_owned(),
            parent: Some("root".to_owned()),
            name: "removed-05000".to_owned(),
            version: "v1".to_owned(),
            directory: false,
            metadata: None,
        },
        path: "removed-05000".to_owned(),
        cursor: String::new(),
    };
    store
        .db
        .execute(
            "INSERT INTO namespace_ops(id,node,body,state,kind) VALUES('op-new','n05000',?1,'pending','remove')",
            [serde_json::to_string(&job).unwrap()],
        )
        .unwrap();
    let blocked = store
        .upload_remove_barrier("n05000", "removed-05000")
        .unwrap()
        .expect("the fresh intent blocks the same path again");
    assert_eq!(blocked.operation, "op-new");
    assert_eq!(blocked.path, "removed-05000");
    // Bulk completion must not leave a stale index behind.
    store
        .db
        .execute(
            "UPDATE namespace_ops SET state='committed' WHERE kind='remove'",
            [],
        )
        .unwrap();
    assert!(store
        .upload_remove_barrier("n05000", "removed-05000")
        .unwrap()
        .is_none());
}