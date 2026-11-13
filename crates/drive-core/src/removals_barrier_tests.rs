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
/// Field-shaped backlog for the end-to-end regression: the overwhelming majority
/// of intents are still `pending`, with the small `uncertain` and `conflict`
/// minority seen in the `.2` incident (3,114 pending / 55 uncertain / 4 conflict,
/// scaled here to 10k total). Seeding writes the same serialized [`RemoveJob`]
/// body and journal columns as `unlink_local`, but in one transaction: a public
/// API seed would be 10k synchronous=FULL commits (an I/O benchmark), and the
/// mixed shape is unreachable through the mutation gates while an `uncertain`
/// head blocks later pending intents (that is #57's dispatch defect, not #28).
fn mixed_backlog_fixture(
    pending: usize,
    uncertain: usize,
    conflict: usize,
) -> (tempfile::TempDir, Store) {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(dir.path()).unwrap();
    store
        .db
        .execute(
            "INSERT INTO nodes(id,parent,name,remote_version,directory) VALUES('root',NULL,'root','v1',1)",
            [],
        )
        .unwrap();
    store
        .db
        .execute(
            "INSERT INTO nodes(id,parent,name,remote_version,directory) VALUES('uploads','root','uploads','v1',1)",
            [],
        )
        .unwrap();
    let tx = store.db.unchecked_transaction().unwrap();
    let mut index = 0usize;
    for (state, count) in [
        ("pending", pending),
        ("uncertain", uncertain),
        ("conflict", conflict),
    ] {
        for _ in 0..count {
            let id = format!("n{index:05}");
            let name = format!("removed-{index:05}");
            let operation = format!("op{index:05}");
            let job = RemoveJob {
                operation: operation.clone(),
                node: Node {
                    id: id.clone(),
                    parent: Some("root".to_owned()),
                    name: name.clone(),
                    version: "v1".to_owned(),
                    directory: false,
                    metadata: None,
                },
                path: name.clone(),
                cursor: String::new(),
            };
            tx.execute(
                "INSERT INTO nodes(id,parent,name,remote_version,directory,deleted) VALUES(?1,'root',?2,'v1',0,1)",
                rusqlite::params![id, name],
            )
            .unwrap();
            tx.execute(
                "INSERT INTO namespace_ops(id,node,body,state,kind) VALUES(?1,?2,?3,?4,'remove')",
                rusqlite::params![
                    operation,
                    id,
                    serde_json::to_string(&job).unwrap(),
                    state
                ],
            )
            .unwrap();
            index += 1;
        }
    }
    tx.commit().unwrap();
    (dir, store)
}

/// Pending uploads for locally created files under an unrelated directory, built
/// through the production creation path (`create_local_file` + `creation_saved`).
fn prepare_uploads(store: &mut Store, count: usize) -> Vec<String> {
    let mut nodes = Vec::new();
    for index in 0..count {
        let node = store
            .create_local_file("uploads", &format!("up-{index:02}.doc"), 0o644)
            .unwrap();
        store.creation_saved(&node.id).unwrap();
        nodes.push(node.id);
    }
    nodes
        .iter()
        .map(|node| {
            store
                .db
                .query_row(
                    "SELECT id FROM operations WHERE kind='upload' AND node=?1 AND state='pending'",
                    [node.as_str()],
                    |row| row.get(0),
                )
                .unwrap()
        })
        .collect()
}

/// RED for the #28 residual: the barrier's TEMP version trigger fired on every
/// `namespace_ops` UPDATE, including durable commit-diagnosis/attempt bookkeeping
/// that cannot change the indexed membership or paths. One bookkeeping write per
/// queued intent force-rebuilt the whole 10k-row index.
#[test]
fn barrier_version_ignores_nonsemantic_intent_updates() {
    let (_dir, mut store) = fixture(10_000);
    assert!(target_barrier(&mut store).is_none());
    let (calls, rebuilds, _last_us, rows) = store.remove_barrier_stats();
    assert_eq!((calls, rebuilds, rows), (1, 1, 10_000));

    // Exactly the columns commit_diagnostics::record/clear_tx rewrite while a
    // remove intent waits for its next attempt.
    store
        .db
        .execute(
            "UPDATE namespace_ops SET attempts=attempts+1, outcome='unknown_retryable',
                    outcome_category='interrupted', last_error='reconcile attempt',
                    next_retry_at=0, updated_at=unixepoch() WHERE kind='remove'",
            [],
        )
        .unwrap();
    assert!(target_barrier(&mut store).is_none());
    let (calls_after, rebuilds_after, _last_us, rows_after) = store.remove_barrier_stats();
    assert_eq!(calls_after, calls + 1);
    assert_eq!(
        rebuilds_after, rebuilds,
        "non-semantic namespace_ops updates must not rebuild the barrier index"
    );
    assert_eq!(rows_after, 10_000);

    // A semantic change must still invalidate the index.
    store
        .db
        .execute(
            "UPDATE namespace_ops SET state='committed' WHERE id='op05000'",
            [],
        )
        .unwrap();
    assert!(target_barrier(&mut store).is_none());
    assert_eq!(
        store.remove_barrier_stats().1,
        rebuilds_after + 1,
        "state changes must still rebuild"
    );
    assert_eq!(store.remove_barrier_stats().3, 9_999);

    // Body rewrites are the other non-semantic production update (`rebase_jobs`
    // rewrites a queued intent's captured version after an ancestor confirms).
    store
        .db
        .execute("UPDATE namespace_ops SET body=body WHERE id='op04000'", [])
        .unwrap();
    assert!(target_barrier(&mut store).is_none());
    assert_eq!(
        store.remove_barrier_stats().1,
        rebuilds_after + 1,
        "body rewrites must not rebuild the barrier index"
    );
}

/// RED for the #28 residual: an upload must stay bounded while a 10k field-shaped
/// remove backlog drains and each reconcile attempt rewrites durable bookkeeping.
/// The old trigger rebuilt all 10k indexed rows on every attempt (>40ms); the
/// fixed trigger ignores those writes and the upload pays only the per-target
/// lookup.
#[test]
fn upload_start_stays_bounded_with_a_large_mixed_remove_backlog() {
    let (_backlog_dir, mut backlog) = mixed_backlog_fixture(8_900, 1_000, 100);
    let (_control_dir, mut control) = mixed_backlog_fixture(0, 0, 0);
    let backlog_uploads = prepare_uploads(&mut backlog, 8);
    let control_uploads = prepare_uploads(&mut control, 8);

    // The first upload after the backlog exists pays the one-time index build;
    // it is the same cost the pre-#28 implementation paid on every upload.
    assert!(backlog.mark_upload_started(&backlog_uploads[0]).is_ok());
    assert!(control.mark_upload_started(&control_uploads[0]).is_ok());
    assert_eq!(backlog.remove_barrier_stats().3, 10_000);

    let mut backlog_samples = Vec::new();
    let mut control_samples = Vec::new();
    for index in 1..8 {
        // One durable attempt/diagnosis rewrite between uploads, the shape of
        // commit_diagnostics::record for the draining remove tail.
        backlog
            .db
            .execute(
                "UPDATE namespace_ops SET attempts=attempts+1, outcome='unknown_retryable',
                        outcome_category='interrupted', last_error='reconcile attempt',
                        next_retry_at=0, updated_at=unixepoch() WHERE id=?1",
                [format!("op{index:05}")],
            )
            .unwrap();
        let started = Instant::now();
        backlog.mark_upload_started(&backlog_uploads[index]).unwrap();
        backlog_samples.push(started.elapsed());
        let started = Instant::now();
        control.mark_upload_started(&control_uploads[index]).unwrap();
        control_samples.push(started.elapsed());
    }
    let backlog_median = median(backlog_samples);
    let control_median = median(control_samples);
    let (calls, rebuilds, last_us, rows) = backlog.remove_barrier_stats();
    eprintln!(
        "#28 E2E backlog_median={:.3}ms control_median={:.3}ms ratio={:.1} calls={calls} rebuilds={rebuilds} last_us={last_us} rows={rows}",
        backlog_median.as_secs_f64() * 1e3,
        control_median.as_secs_f64() * 1e3,
        backlog_median.as_secs_f64() / control_median.as_secs_f64().max(1e-9)
    );
    assert_eq!((calls, rows), (8, 10_000));
    assert_eq!(
        rebuilds, 1,
        "reconcile bookkeeping must not rebuild the barrier index"
    );
    // The primary bound is relative to the empty-store control: `cargo test
    // --workspace` runs test binaries in parallel under shared machine load, so
    // an absolute millisecond threshold is not a stable product signal (the same
    // call measured ~4ms isolated and ~20ms under the full-suite load). The RED
    // lever stays the O(R) rebuild ratio (RED: 6.7) plus `rebuilds == 1` above;
    // the absolute bound below only catches pathological regressions.
    let ratio = backlog_median.as_secs_f64() / control_median.as_secs_f64().max(1e-9);
    assert!(
        ratio <= 5.0,
        "backlog upload start must stay within 5x of the empty store: {ratio:.1}"
    );
    assert!(
        backlog_median < Duration::from_millis(200),
        "upload start with a 10k backlog is pathologically slow: {backlog_median:?}"
    );
    assert!(last_us > 0);

    // Unrelated paths are not blocked, while a remove that owns the upload path
    // still is.
    let blocked_node = backlog.create_local_file("uploads", "blocked.doc", 0o644).unwrap();
    backlog.creation_saved(&blocked_node.id).unwrap();
    let blocked_upload: String = backlog
        .db
        .query_row(
            "SELECT id FROM operations WHERE kind='upload' AND node=?1 AND state='pending'",
            [blocked_node.id.as_str()],
            |row| row.get(0),
        )
        .unwrap();
    let job = RemoveJob {
        operation: "op-blocked".to_owned(),
        node: blocked_node.clone(),
        path: "uploads/blocked.doc".to_owned(),
        cursor: String::new(),
    };
    backlog
        .db
        .execute(
            "INSERT INTO namespace_ops(id,node,body,state,kind) VALUES('op-blocked',?1,?2,'pending','remove')",
            rusqlite::params![blocked_node.id, serde_json::to_string(&job).unwrap()],
        )
        .unwrap();
    let error = backlog.mark_upload_started(&blocked_upload).unwrap_err();
    assert!(
        error.to_string().contains("pending remove"),
        "upload under its own queued remove must stay blocked: {error}"
    );
}

/// RED for the #28 residual: production had no observable barrier diagnostics.
/// Each rebuild now emits one structured `key=value` line to stderr for the
/// journal, and the crate-internal accessor exposes the same lines to tests.
#[test]
fn barrier_rebuild_emits_diagnostic_line_and_stats() {
    let (_dir, mut store) = fixture(1_000);
    assert!(store.remove_barrier_diagnostics().is_empty());
    assert!(target_barrier(&mut store).is_none());
    let (calls, rebuilds, last_us, rows) = store.remove_barrier_stats();
    assert_eq!((calls, rebuilds, rows), (1, 1, 1_000));
    assert!(last_us > 0);
    let lines = store.remove_barrier_diagnostics();
    assert_eq!(lines.len(), 1, "one diagnostic line per rebuild: {lines:?}");
    let line = &lines[0];
    for expected in [
        "level=info",
        "event=remove_barrier_rebuild",
        "calls=1",
        "rebuilds=1",
        "rows=1000",
        "last_us=",
    ] {
        assert!(
            line.contains(expected),
            "missing {expected:?} in diagnostic line {line:?}"
        );
    }
    // A cache hit must not emit another line.
    assert!(target_barrier(&mut store).is_none());
    assert_eq!(store.remove_barrier_diagnostics().len(), 1);
}
