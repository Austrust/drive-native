//! Issue #90-B evidence: clean-draft metadata deletion used to run one
//! `DELETE` per row with no enclosing transaction, so under WAL +
//! `synchronous=FULL` C clean closed drafts produced C commits while the caller
//! held the Store/flights locks. These tests meter the transactions the deletion
//! pass performs (the write-amplification unit from the issue) and pin the
//! protection contract (dirty / open / quarantined drafts are never deleted).
//!
//! `commits` is instrumented at the transaction boundary of
//! [`crate::Store::prune_clean_drafts`]: the red baseline issues one autocommit
//! per deleted row, the green implementation one transaction per
//! [`GC_CLEAN_DRAFT_BATCH_SIZE`] rows.
use super::*;
use crate::{DownloadTicket, Node, NodeMetadata, Store, sha256};
use std::collections::HashSet;

fn node(id: &str, parent: Option<&str>, directory: bool) -> Node {
    Node {
        id: id.into(),
        parent: parent.map(str::to_owned),
        name: id.into(),
        version: "1".into(),
        directory,
        metadata: Some(NodeMetadata {
            size: 3,
            mtime: 1,
            mode: if directory { 0o755 } else { 0o644 },
        }),
    }
}

/// Create `clean` clean closed drafts (+ bodies), `dirty` dirty drafts and
/// `open` clean closed drafts whose body is referenced by a live draft handle.
/// Returns the clean tokens and the tokens that must survive pruning.
fn draft_fixture(
    dir: &tempfile::TempDir,
    store: &mut Store,
    clean: usize,
    dirty: usize,
    open: usize,
) -> (Vec<String>, Vec<String>) {
    let drafts = dir.path().join("drafts");
    std::fs::create_dir_all(&drafts).unwrap();
    let mut clean_tokens = Vec::new();
    let mut protected_tokens = Vec::new();
    let total = clean + dirty + open;
    let mut statement = String::from("BEGIN;");
    for index in 0..total {
        let token = uuid::Uuid::new_v4().to_string();
        let dirty_flag = usize::from(index >= clean && index < clean + dirty);
        statement.push_str(&format!(
            "INSERT INTO drafts(node,token,base,generation,checkpoint,dirty) \
             VALUES('draft-node-{index}','{token}','1',1,0,{dirty_flag});"
        ));
        std::fs::write(drafts.join(&token), b"draft-body").unwrap();
        if index < clean {
            clean_tokens.push(token);
        } else {
            protected_tokens.push(token);
        }
    }
    statement.push_str("COMMIT;");
    store.db.execute_batch(&statement).unwrap();
    // The `open` tail keeps a live handle map entry, which is exactly what
    // `open_draft`/`begin_draft` retain for the lifetime of a `DraftHandle`.
    for token in protected_tokens.iter().skip(dirty) {
        store
            .handles
            .lock()
            .unwrap()
            .insert(format!("draft:{token}"), ("draft-node-open".into(), 1));
    }
    (clean_tokens, protected_tokens)
}

fn remaining_draft_rows(store: &Store) -> HashSet<String> {
    let mut q = store.db.prepare("SELECT token FROM drafts").unwrap();
    let rows = q
        .query_map([], |r| r.get::<_, String>(0))
        .unwrap()
        .collect::<std::result::Result<HashSet<_>, _>>()
        .unwrap();
    rows
}

#[test]
fn clean_draft_delete_commits_are_batched() {
    let batch = GC_CLEAN_DRAFT_BATCH_SIZE;
    for clean in [100usize, 1_000, 5_000] {
        let dir = tempfile::tempdir().unwrap();
        let mut store = Store::open(dir.path()).unwrap();
        let (clean_tokens, protected) = draft_fixture(&dir, &mut store, clean, 3, 2);

        let mut rows = 0u64;
        let mut commits = 0u64;
        loop {
            let pruned = store.prune_clean_drafts(usize::MAX).unwrap();
            rows += pruned.rows;
            commits += pruned.commits;
            if pruned.rows == 0 {
                break;
            }
        }

        assert_eq!(rows, clean as u64, "every clean closed draft must be deleted");
        // #90-B red assertion: the pre-fix path commits once per row
        // (`commits == clean`). Batched deletion converges to ceil(C/batch).
        assert_eq!(
            commits,
            clean.div_ceil(batch) as u64,
            "clean={clean}: per-row autocommit would make {clean} commits; \
             bounded-batch deletion must make ceil({clean}/{batch})"
        );

        let rows_now = remaining_draft_rows(&store);
        for token in &protected {
            assert!(
                rows_now.contains(token),
                "dirty/open draft row must never be deleted"
            );
            assert!(
                dir.path().join("drafts").join(token).exists(),
                "dirty/open draft body must never be unlinked"
            );
        }
        for token in &clean_tokens {
            assert!(
                !dir.path().join("drafts").join(token).exists(),
                "a deleted clean draft must take its body with it"
            );
        }
    }
}

#[test]
fn clean_draft_prune_respects_open_handle_and_quarantine() {
    let dir = tempfile::tempdir().unwrap();
    let mut store = Store::open(dir.path()).unwrap();
    store
        .install_remote_snapshot(
            &[node("root", None, true), node("file", Some("root"), false)],
            "1",
            "server",
        )
        .unwrap();
    store.ensure_fs_root("root").unwrap();
    store
        .publish_download(
            &DownloadTicket {
                id: "file".into(),
                version: "1".into(),
            },
            b"old",
            &sha256(b"old"),
        )
        .unwrap();
    let source = store.open_cached("file").unwrap();
    // A clean (dirty=0) draft that is still open must keep its row and body.
    let draft = store.begin_draft("file", "1", Some(&source)).unwrap();
    let token = draft.token.clone();
    let pruned = store.prune_clean_drafts(usize::MAX).unwrap();
    assert_eq!(pruned.rows, 0, "an open clean draft is not deletable");
    assert!(remaining_draft_rows(&store).contains(&token));
    assert!(dir.path().join("drafts").join(&token).exists());
    // Once the handle closes the same draft is reclaimable.
    drop(draft);
    drop(source);
    let pruned = store.prune_clean_drafts(usize::MAX).unwrap();
    assert_eq!(pruned.rows, 1);
    assert!(!remaining_draft_rows(&store).contains(&token));
    assert!(!dir.path().join("drafts").join(&token).exists());

    // Recoverable quarantine bodies are a different directory and must never be
    // touched by the clean-draft deletion or the unreferenced-file sweep.
    let quarantine = dir.path().join("drafts_quarantine");
    std::fs::create_dir_all(&quarantine).unwrap();
    let quarantined = uuid::Uuid::new_v4().to_string();
    std::fs::write(quarantine.join(&quarantined), b"recoverable").unwrap();
    store.prune_unreferenced_draft_files(usize::MAX).unwrap();
    assert!(quarantine.join(&quarantined).exists());
}

#[test]
fn clean_draft_prune_resumes_after_partial_pass_and_reopen() {
    let batch = GC_CLEAN_DRAFT_BATCH_SIZE;
    let dir = tempfile::tempdir().unwrap();
    let mut store = Store::open(dir.path()).unwrap();
    let _ = draft_fixture(&dir, &mut store, 300, 0, 0);

    let first = store.prune_clean_drafts(100).unwrap();
    assert_eq!(first.rows, 100);
    assert_eq!(first.commits, 1, "a 100-row budget is one 100-row transaction");
    assert_eq!(remaining_draft_rows(&store).len(), 200);

    // Crash/stop between batches: the committed batches are durable and a
    // reopened Store continues from the remaining rows.
    drop(store);
    let mut store = Store::open(dir.path()).unwrap();
    assert_eq!(remaining_draft_rows(&store).len(), 200);
    let second = store.prune_clean_drafts(usize::MAX).unwrap();
    assert_eq!(second.rows, 200);
    assert_eq!(second.commits, 200usize.div_ceil(batch) as u64);
    assert!(remaining_draft_rows(&store).is_empty());
}

#[test]
fn failed_clean_draft_batch_rolls_back_and_retry_finishes() {
    let dir = tempfile::tempdir().unwrap();
    let mut store = Store::open(dir.path()).unwrap();
    let _ = draft_fixture(&dir, &mut store, 300, 1, 0);
    let ordered: Vec<String> = {
        let mut q = store.db.prepare("SELECT token FROM drafts ORDER BY token").unwrap();
        q.query_map([], |r| r.get::<_, String>(0))
            .unwrap()
            .collect::<std::result::Result<_, _>>()
            .unwrap()
    };
    // Abort on a token inside the second batch (batch size 128).
    let abort_token = ordered[150].clone();
    store
        .db
        .execute_batch(&format!(
            "CREATE TEMP TRIGGER abort_clean_draft_delete BEFORE DELETE ON drafts \
             WHEN OLD.token='{abort_token}' BEGIN SELECT RAISE(ABORT,'injected'); END;"
        ))
        .unwrap();
    let failed = store.prune_clean_drafts(usize::MAX).unwrap_err();
    assert!(
        failed.to_string().contains("injected") || failed.to_string().contains("injected"),
        "the batch error must propagate instead of being swallowed: {failed}"
    );
    // The first batch committed, the failed batch rolled back as a unit.
    // 300 clean rows + 1 dirty row: exactly the first 128-row batch committed.
    let remaining = remaining_draft_rows(&store);
    assert_eq!(remaining.len(), 301 - 128);
    for token in &remaining {
        assert!(
            dir.path().join("drafts").join(token).exists(),
            "the rolled-back batch must keep every body"
        );
    }
    let draft_bodies = std::fs::read_dir(dir.path().join("drafts"))
        .unwrap()
        .count();
    assert_eq!(draft_bodies, remaining.len());
    // The dirty draft is still protected.
    assert_eq!(
        store
            .db
            .query_row("SELECT COUNT(*) FROM drafts WHERE dirty=1", [], |r| r
                .get::<_, i64>(0))
            .unwrap(),
        1
    );
    // Drop the injected failure: the remaining 172 rows finish in ceil(172/128)=2
    // transactions and the failure did not corrupt the journal.
    store
        .db
        .execute_batch("DROP TRIGGER abort_clean_draft_delete;")
        .unwrap();
    let resumed = store.prune_clean_drafts(usize::MAX).unwrap();
    assert_eq!(resumed.rows, 300 - 128);
    assert_eq!(resumed.commits, 172usize.div_ceil(GC_CLEAN_DRAFT_BATCH_SIZE) as u64);
}

/// #90-A: the lock-free scan may only *discover* candidates. Every deletion is
/// re-verified against the current reference set under the Store lock, so a
/// reference created between discovery and the delete window always wins.
#[test]
fn gc_reclaim_reverifies_references_after_lock_free_discovery() {
    let dir = tempfile::tempdir().unwrap();
    let mut store = Store::open(dir.path()).unwrap();
    store
        .install_remote_snapshot(
            &[
                node("root", None, true),
                node("live", Some("root"), false),
                node("other", Some("root"), false),
            ],
            "1",
            "server",
        )
        .unwrap();
    store.ensure_fs_root("root").unwrap();
    store
        .publish_download(
            &DownloadTicket {
                id: "live".into(),
                version: "1".into(),
            },
            b"live-body",
            &sha256(b"live-body"),
        )
        .unwrap();
    let blob: String = store
        .db
        .query_row("SELECT blob FROM nodes WHERE id='live'", [], |r| r.get(0))
        .unwrap();
    let path = dir.path().join("blobs").join(&blob);

    // 1. Discovery snapshot happens while the body is referenced.
    let inventory = crate::cache_policy::scan_blob_inventory(dir.path()).unwrap();
    assert!(inventory.entries.iter().any(|entry| entry.name == blob));

    // 2. The reference disappears after discovery.
    store
        .db
        .execute(
            "UPDATE nodes SET blob=NULL,local_version=NULL WHERE id='live'",
            [],
        )
        .unwrap();
    // 3. ...but a new reference is created before the locked window. The stale
    //    candidate snapshot must not delete it.
    store
        .db
        .execute(
            "UPDATE nodes SET blob=?1, local_version='1' WHERE id='live'",
            [&blob],
        )
        .unwrap();
    let reclaim = store
        .gc_reclaim_blobs(&inventory.entries, &HashSet::new())
        .unwrap();
    assert_eq!(reclaim.removed, 0, "a re-created reference must survive");
    assert!(path.exists());

    // 3b. A shared reference on another node protects the body just as well.
    store
        .db
        .execute("UPDATE nodes SET blob=NULL,local_version=NULL WHERE id='live'", [])
        .unwrap();
    store
        .db
        .execute(
            "UPDATE nodes SET blob=?1, local_version='1' WHERE id='other'",
            [&blob],
        )
        .unwrap();
    let reclaim = store
        .gc_reclaim_blobs(&inventory.entries, &HashSet::new())
        .unwrap();
    assert_eq!(reclaim.removed, 0, "a shared blob must survive");
    assert!(path.exists());
    store
        .db
        .execute("UPDATE nodes SET blob=NULL,local_version=NULL WHERE id='other'", [])
        .unwrap();

    // 4. An open cached handle also protects the body after its DB pointer goes
    //    away (the unlinked-old-handle case).
    store
        .db
        .execute(
            "UPDATE nodes SET blob=?1, local_version='1' WHERE id='live'",
            [&blob],
        )
        .unwrap();
    let handle = store.open_cached("live").unwrap();
    store
        .db
        .execute("UPDATE nodes SET blob=NULL,local_version=NULL WHERE id='live'", [])
        .unwrap();
    let reclaim = store
        .gc_reclaim_blobs(&inventory.entries, &HashSet::new())
        .unwrap();
    assert_eq!(reclaim.removed, 0, "an open handle must survive");
    assert!(path.exists());
    drop(handle);

    // 5. With every protection gone the same candidate is reclaimed, and the
    //    logical bytes are reported.
    let reclaim = store
        .gc_reclaim_blobs(&inventory.entries, &HashSet::new())
        .unwrap();
    assert_eq!(reclaim.removed, 1);
    assert_eq!(reclaim.removed_bytes, b"live-body".len() as u64);
    assert!(!path.exists());
}

#[test]
fn gc_reclaim_keeps_operations_abandoned_and_namespace_replace_refs() {
    let dir = tempfile::tempdir().unwrap();
    let mut store = Store::open(dir.path()).unwrap();
    store
        .install_remote_snapshot(
            &[node("root", None, true), node("live", Some("root"), false)],
            "1",
            "server",
        )
        .unwrap();
    store.ensure_fs_root("root").unwrap();
    store
        .publish_download(
            &DownloadTicket {
                id: "live".into(),
                version: "1".into(),
            },
            b"shared-body",
            &sha256(b"shared-body"),
        )
        .unwrap();
    let blob: String = store
        .db
        .query_row("SELECT blob FROM nodes WHERE id='live'", [], |r| r.get(0))
        .unwrap();
    let inventory = crate::cache_policy::scan_blob_inventory(dir.path()).unwrap();
    store
        .db
        .execute(
            "UPDATE nodes SET blob=NULL,local_version=NULL WHERE id='live'",
            [],
        )
        .unwrap();

    // Pending upload operation protects the body, abandoned keeps protecting it
    // (#14: an explicit retry can re-arm the same content).
    store
        .db
        .execute(
            "INSERT INTO operations(id,node,kind,base,blob,generation,state) \
             VALUES('op','live','upload','1',?1,1,'pending')",
            [&blob],
        )
        .unwrap();
    assert_eq!(
        store
            .gc_reclaim_blobs(&inventory.entries, &HashSet::new())
            .unwrap()
            .removed,
        0
    );
    store
        .db
        .execute("UPDATE operations SET state='abandoned' WHERE id='op'", [])
        .unwrap();
    assert_eq!(
        store
            .gc_reclaim_blobs(&inventory.entries, &HashSet::new())
            .unwrap()
            .removed,
        0,
        "abandoned recovery data must survive GC"
    );
    store.db.execute("DELETE FROM operations WHERE id='op'", []).unwrap();

    // A pending namespace replacement protects the blob until it commits.
    store
        .db
        .execute(
            "INSERT INTO namespace_ops(id,node,body,state,kind) \
             VALUES('ns','root',?1,'pending','replace')",
            [serde_json::json!({ "blob": blob }).to_string()],
        )
        .unwrap();
    assert_eq!(
        store
            .gc_reclaim_blobs(&inventory.entries, &HashSet::new())
            .unwrap()
            .removed,
        0
    );
    store
        .db
        .execute("UPDATE namespace_ops SET state='committed' WHERE id='ns'", [])
        .unwrap();
    assert_eq!(
        store
            .gc_reclaim_blobs(&inventory.entries, &HashSet::new())
            .unwrap()
            .removed,
        1,
        "the committed replacement no longer references the body"
    );
}

#[test]
fn gc_reclaim_verifies_candidate_identity_on_disk() {
    let dir = tempfile::tempdir().unwrap();
    let mut store = Store::open(dir.path()).unwrap();
    let name = uuid::Uuid::new_v4().to_string();
    std::fs::write(dir.path().join("blobs").join(&name), b"abcdef").unwrap();
    // A size mismatch means the body changed since discovery: keep, rediscover.
    let stale = crate::cache_policy::BlobEntry {
        name: name.clone(),
        bytes: 3,
    };
    let reclaim = store.gc_reclaim_blobs(&[stale], &HashSet::new()).unwrap();
    assert_eq!(reclaim.removed, 0);
    assert!(dir.path().join("blobs").join(&name).exists());
    // The current identity (UUID + regular file + same size) is reclaimed.
    let inventory = crate::cache_policy::scan_blob_inventory(dir.path()).unwrap();
    let reclaim = store.gc_reclaim_blobs(&inventory.entries, &HashSet::new()).unwrap();
    assert_eq!(reclaim.removed, 1);
    assert!(!dir.path().join("blobs").join(&name).exists());
}

#[test]
fn scan_blob_inventory_skips_non_blob_entries() {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(dir.path()).unwrap();
    std::fs::write(dir.path().join("blobs").join("not-a-uuid"), b"x").unwrap();
    let real = uuid::Uuid::new_v4().to_string();
    std::fs::write(dir.path().join("blobs").join(&real), b"1234").unwrap();
    let inventory = crate::cache_policy::scan_blob_inventory(dir.path()).unwrap();
    assert_eq!(inventory.entries.len(), 1);
    assert_eq!(inventory.entries[0].name, real);
    assert_eq!(inventory.entries[0].bytes, 4);
    assert_eq!(inventory.total_bytes, 4);
    drop(store);
}

/// Heavy mechanism evidence (#90-B): C clean drafts deleted by the old
/// per-row autocommit loop vs the bounded batch transactions, measured in the
/// same process/tree/profile (WAL + `synchronous=FULL`). Compile in the build
/// slot; run on the remote heavy slot only.
///
///   cargo test -p drive-core --all-features --locked --lib \
///     clean_draft_delete_legacy_vs_batched_heavy -- --ignored --nocapture
#[test]
#[ignore = "heavy mechanism timing; run on the remote heavy slot"]
fn clean_draft_delete_legacy_vs_batched_heavy() {
    let batch = GC_CLEAN_DRAFT_BATCH_SIZE;
    for clean in [1_000usize, 5_000] {
        // Legacy shape: one autocommit per deleted row, exactly the pre-#90
        // cycle. This is a private measurement loop, not product code.
        let dir = tempfile::tempdir().unwrap();
        let mut store = Store::open(dir.path()).unwrap();
        let _ = draft_fixture(&dir, &mut store, clean, 0, 0);
        let tokens: Vec<String> = {
            let mut q = store
                .db
                .prepare("SELECT token FROM drafts ORDER BY token")
                .unwrap();
            q.query_map([], |r| r.get::<_, String>(0))
                .unwrap()
                .collect::<std::result::Result<_, _>>()
                .unwrap()
        };
        let started = std::time::Instant::now();
        for token in &tokens {
            store
                .db
                .execute("DELETE FROM drafts WHERE token=?1 AND dirty=0", [token])
                .unwrap();
        }
        let legacy_elapsed = started.elapsed();
        assert!(remaining_draft_rows(&store).is_empty());

        // Bounded shape: one transaction per batch.
        let dir = tempfile::tempdir().unwrap();
        let mut store = Store::open(dir.path()).unwrap();
        let _ = draft_fixture(&dir, &mut store, clean, 0, 0);
        let started = std::time::Instant::now();
        let mut rows = 0u64;
        let mut commits = 0u64;
        loop {
            let pruned = store.prune_clean_drafts(usize::MAX).unwrap();
            rows += pruned.rows;
            commits += pruned.commits;
            if pruned.rows == 0 {
                break;
            }
        }
        let batched_elapsed = started.elapsed();
        assert_eq!(rows, clean as u64);
        assert_eq!(commits, clean.div_ceil(batch) as u64);
        println!(
            "HEAVY_B c={clean} legacy_commits={clean} legacy_ms={:.3} batched_commits={} batched_ms={:.3}",
            legacy_elapsed.as_secs_f64() * 1e3,
            commits,
            batched_elapsed.as_secs_f64() * 1e3,
        );
    }
}
