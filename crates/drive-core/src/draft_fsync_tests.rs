//! Issue #24: the FUSE write path must not issue a durable SQLite commit per
//! write syscall. Dirty draft metadata is coalesced per dirty epoch (first edit
//! after begin/checkpoint) and checkpointed in one transaction; a crash between
//! edits must still leave a recoverable dirty draft.
use crate::*;
use std::io::Read;

const CHUNK: usize = 128 * 1024;
const CHUNKS: usize = 800; // 100 MiB in 128 KiB writes.

fn node(id: &str, version: &str, dir: bool, size: u64) -> Node {
    Node {
        id: id.into(),
        parent: if dir { None } else { Some("root".into()) },
        name: id.into(),
        version: version.into(),
        directory: dir,
        metadata: Some(NodeMetadata {
            size: if dir { 0 } else { size },
            mtime: 0,
            mode: 0o644,
        }),
    }
}

fn setup_empty(content: &[u8]) -> (tempfile::TempDir, Store) {
    let dir = tempfile::tempdir().unwrap();
    let mut store = Store::open(dir.path()).unwrap();
    store
        .install_remote_snapshot(
            &[
                node("root", "1", true, 0),
                node("big", "1", false, content.len() as u64),
            ],
            "1",
            "server",
        )
        .unwrap();
    store
        .publish_download(
            &DownloadTicket {
                id: "big".into(),
                version: "1".into(),
            },
            content,
            &sha256(content),
        )
        .unwrap();
    (dir, store)
}

fn begin(store: &mut Store) -> drafts::DraftHandle {
    let mut read = store.open_cached("big").unwrap();
    store.begin_draft("big", "1", Some(&mut read)).unwrap()
}

/// 100 MiB of sequential writes used to run one autocommit UPDATE (plus its
/// status-change trigger rows) per syscall. Only the first edit of a dirty
/// epoch may commit; later writes must reuse that durable mark.
#[test]
fn sequential_write_path_does_not_commit_metadata_per_syscall() {
    let (_dir, mut s) = setup_empty(b"");
    let h = begin(&mut s);
    assert_eq!(
        s.draft_write_stats().draft_file_fsyncs,
        2,
        "begin must durably create the mutable working copy"
    );
    let payload = vec![0x5a_u8; CHUNK];
    let before = s.db.total_changes();
    let started = std::time::Instant::now();
    for index in 0..CHUNKS {
        s.draft_write(&h, (index * CHUNK) as u64, &payload, false)
            .unwrap();
    }
    let elapsed = started.elapsed();
    let write_changes = s.db.total_changes() - before;
    let stats = s.draft_write_stats();
    eprintln!(
        "#24 sequential_write: {} writes, {} MiB, {} SQLite row changes in {elapsed:?}, \
         metadata_commits={} draft_file_fsyncs={}",
        CHUNKS,
        (CHUNKS * CHUNK) / (1024 * 1024),
        write_changes,
        stats.metadata_commits,
        stats.draft_file_fsyncs
    );
    assert!(
        write_changes <= 16,
        "100 MiB sequential write issued {write_changes} SQLite row changes; \
         a bounded write path must only persist one dirty-epoch mark"
    );
    assert_eq!(
        stats.metadata_commits, 1,
        "the whole dirty epoch must be marked by exactly one durable statement"
    );
    assert_eq!(
        stats.draft_file_fsyncs, 2,
        "content writes must not fsync the working file per syscall"
    );
    assert_eq!(s.status("big").unwrap(), Status::PendingUpload);
}

/// The coalesced dirty mark must stay durable before checkpoint: an open file
/// that disappears with unsaved edits is recovered on the next start, exactly
/// like the per-write commit it replaces.
#[test]
fn unsaved_coalesced_edits_are_recovered_after_unclean_reopen() {
    let (dir, mut s) = setup_empty(b"");
    let h = begin(&mut s);
    s.draft_write(&h, 0, b"recovered", false).unwrap();
    assert!(s.dirty_draft("big").unwrap());
    drop(h);
    drop(s);
    let mut s = Store::open(dir.path()).unwrap();
    assert!(s.dirty_draft("big").unwrap());
    let h = s.open_dirty_draft("big").unwrap().unwrap();
    let mut bytes = [0u8; 9];
    assert_eq!(h.read_at(&mut bytes, 0).unwrap(), 9);
    assert_eq!(&bytes, b"recovered");
    assert_eq!(s.checkpoint_recovered_drafts().unwrap(), 1);
    let mut uploaded = Vec::new();
    s.open_cached("big")
        .unwrap()
        .read_to_end(&mut uploaded)
        .unwrap();
    assert_eq!(uploaded, b"recovered");
}

/// A checkpoint is one transaction: saving the blob and advancing the draft
/// checkpoint must not be split into two autocommits.
#[test]
fn checkpoint_persists_dirty_edits_in_one_metadata_transaction() {
    let (_dir, mut s) = setup_empty(b"");
    let h = begin(&mut s);
    let mut payload = vec![0u8; 64 * 1024];
    payload[0] = 7;
    s.draft_write(&h, 0, &payload, false).unwrap();
    let before = s.db.total_changes();
    let op = s.checkpoint_draft(&h).unwrap().unwrap();
    let checkpoint_changes = s.db.total_changes() - before;
    let stats = s.draft_write_stats();
    assert!(
        checkpoint_changes <= 64,
        "checkpoint issued {checkpoint_changes} SQLite row changes; expected one bounded transaction"
    );
    assert_eq!(
        stats.metadata_commits, 2,
        "checkpoint must merge its row update into the snapshot commit"
    );
    assert_eq!(stats.checkpoints, 1);
    assert_eq!(stats.draft_file_fsyncs, 3, "two begin fsyncs + one barrier");
    assert!(op.starts_with("op-") || !op.is_empty());
    let mut uploaded = Vec::new();
    s.open_cached("big")
        .unwrap()
        .read_to_end(&mut uploaded)
        .unwrap();
    assert_eq!(uploaded, payload);
}

/// SETATTR(mtime) used to fsync twice (SQLite commit + an eager working-file
/// sync_all). The mtime is only durable through the checkpoint, so the single
/// barrier now belongs to the checkpoint.
#[test]
fn timestamp_edit_defers_its_single_fsync_to_checkpoint() {
    let (_dir, mut s) = setup_empty(b"old");
    let h = begin(&mut s);
    let after_begin = s.draft_write_stats();
    s.draft_set_mtime(&h, 123456).unwrap();
    let after_set_mtime = s.draft_write_stats();
    assert_eq!(
        after_set_mtime.draft_file_fsyncs, after_begin.draft_file_fsyncs,
        "SETATTR(mtime) must not fsync before the checkpoint that persists it"
    );
    assert_eq!(
        after_set_mtime.metadata_commits - after_begin.metadata_commits,
        1,
        "mtime edit must durably mark its dirty epoch"
    );
    let op = s.checkpoint_draft(&h).unwrap().unwrap();
    assert!(!op.is_empty());
    let after = s.draft_write_stats();
    assert_eq!(
        after.draft_file_fsyncs - after_begin.draft_file_fsyncs,
        1,
        "the timestamp edit costs exactly one working-file fsync"
    );
    assert_eq!(
        after.metadata_commits - after_begin.metadata_commits,
        2,
        "epoch mark + merged checkpoint advance"
    );
    assert_eq!(after.checkpoints, 1);
}
