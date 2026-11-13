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
    assert_eq!(remaining_draft_rows(&store).len(), 300 - 128);
    for token in &ordered[..128] {
        assert!(!dir.path().join("drafts").join(token).exists());
    }
    for token in &ordered[128..] {
        assert!(
            dir.path().join("drafts").join(token).exists(),
            "the rolled-back batch must keep every body"
        );
    }
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
