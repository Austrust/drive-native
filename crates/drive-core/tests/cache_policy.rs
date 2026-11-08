use drive_core::*;
use std::collections::HashSet;
fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
fn node(id: &str) -> Node {
    Node {
        id: id.into(),
        parent: if id == "root" {
            None
        } else {
            Some("root".into())
        },
        name: id.into(),
        version: "1".into(),
        directory: id == "root",
        metadata: None,
    }
}
fn setup() -> (tempfile::TempDir, Store) {
    let dir = tempfile::tempdir().unwrap();
    let mut s = Store::open(dir.path()).unwrap();
    s.install_remote_snapshot(
        &[node("root"), node("a"), node("b"), node("c"), node("d")],
        "1",
        "server",
    )
    .unwrap();
    for id in ["a", "b", "c", "d"] {
        s.publish_download(
            &DownloadTicket {
                id: id.into(),
                version: "1".into(),
            },
            b"body",
            &sha256(b"body"),
        )
        .unwrap();
    }
    (dir, s)
}
#[test]
fn default_disabled_and_lru_survives_restart_without_icon_query_touching_it() {
    let (dir, mut s) = setup();
    assert_eq!(s.cache_limit().unwrap(), 0);
    assert!(
        s.trim_cache(now() + 120, &HashSet::new())
            .unwrap()
            .released
            .is_empty()
    );
    s.note_cache_use("c").unwrap();
    s.note_cache_use("a").unwrap();
    s.note_cache_use("b").unwrap();
    s.note_cache_use("d").unwrap();
    // Status providers open a short local read handle; that must not affect LRU.
    drop(s.open_cached("c").unwrap());
    s.set_cache_limit(8).unwrap();
    drop(s);
    let mut s = Store::open(dir.path()).unwrap();
    let result = s.trim_cache(now() + 120, &HashSet::new()).unwrap();
    assert_eq!(result.released, vec!["c", "a"]);
    assert_eq!(result.usage.content_bytes, 8);
    assert_eq!(result.reclaimed_bytes, 8);
    assert_eq!(s.pending_operations().unwrap(), 0);
    assert_eq!(s.pending_namespace_operations().unwrap(), 0);
    assert_eq!(s.status("a").unwrap(), Status::OnlineOnly);
    assert_eq!(s.status("d").unwrap(), Status::LocalComplete);
}
#[test]
fn recent_download_active_transfer_and_open_handle_are_protected() {
    let (_dir, mut s) = setup();
    s.set_cache_limit(1).unwrap();
    assert!(
        s.trim_cache(now(), &HashSet::new())
            .unwrap()
            .released
            .is_empty()
    );
    let handle = s.open_cached("a").unwrap();
    let active = HashSet::from(["b".into()]);
    let result = s.trim_cache(now() + 120, &active).unwrap();
    assert_eq!(result.released, vec!["c", "d"]);
    assert_eq!(result.usage.over_target_bytes, 7);
    assert_eq!(result.usage.protected_bytes, 8);
    drop(handle);
    assert!(
        s.trim_cache(now() + 120, &HashSet::new())
            .unwrap()
            .released
            .contains(&"a".into())
    );
}
#[test]
fn inherited_pin_and_pending_or_conflicted_writes_cannot_be_evicted() {
    let (_dir, mut s) = setup();
    s.set_cache_limit(1).unwrap();
    s.pin("root", true).unwrap();
    assert!(
        s.trim_cache(now() + 120, &HashSet::new())
            .unwrap()
            .released
            .is_empty()
    );
    s.pin("root", false).unwrap();
    s.pin("a", true).unwrap();
    let op = s.save_local("b", b"pending").unwrap();
    s.mark_upload_started(&op).unwrap();
    s.save_local("c", b"conflict").unwrap();
    let mut nodes = vec![node("root"), node("a"), node("b"), node("c"), node("d")];
    nodes[3].version = "2".into();
    s.install_remote_snapshot(&nodes, "2", "server").unwrap();
    let result = s.trim_cache(now() + 120, &HashSet::new()).unwrap();
    assert_eq!(result.released, vec!["d"]);
    assert!(s.release("b").is_err());
    assert!(s.release("c").is_err());
    assert_eq!(s.status("c").unwrap(), Status::Conflict);
    use std::io::Read;
    let mut body = Vec::new();
    s.open_cached("b").unwrap().read_to_end(&mut body).unwrap();
    assert_eq!(body, b"pending");
}
#[test]
fn stale_complete_version_remains_available_when_update_has_not_finished() {
    let (_dir, mut s) = setup();
    let mut nodes = vec![node("root"), node("a"), node("b"), node("c"), node("d")];
    nodes[1].version = "2".into();
    s.install_remote_snapshot(&nodes, "2", "server").unwrap();
    s.set_cache_limit(1).unwrap();
    let result = s.trim_cache(now() + 120, &HashSet::new()).unwrap();
    assert!(!result.released.contains(&"a".into()));
    assert_eq!(s.status("a").unwrap(), Status::Updating);
    assert!(s.open_cached("a").is_ok());
}
#[test]
fn garbage_collection_preserves_dirty_and_open_drafts_then_reclaims_closed_clean_copy() {
    let (_dir, mut s) = setup();
    let mut source = s.open_cached("a").unwrap();
    let clean = s.begin_draft("a", "1", Some(&mut source)).unwrap();
    drop(source);
    let mut source = s.open_cached("b").unwrap();
    let dirty = s.begin_draft("b", "1", Some(&mut source)).unwrap();
    drop(source);
    s.draft_write(&dirty, 0, b"edit", false).unwrap();
    drop(dirty);
    assert_eq!(s.collect_garbage().unwrap(), 0);
    drop(clean);
    assert_eq!(s.collect_garbage().unwrap(), 4);
    let dirty = s.open_dirty_draft("b").unwrap().unwrap();
    let mut buf = [0; 4];
    dirty.read_at(&mut buf, 0).unwrap();
    assert_eq!(&buf, b"edit");
    assert_eq!(
        s.cache_usage(now() + 120, &HashSet::new())
            .unwrap()
            .content_bytes,
        20
    );
}
#[test]
fn schema_11_upgrade_keeps_contents_and_creates_backup_and_policy_tables() {
    let (dir, s) = setup();
    drop(s);
    let db = rusqlite::Connection::open(dir.path().join("state.sqlite")).unwrap();
    db.execute_batch("DROP TABLE cache_access;DROP TABLE cache_published;PRAGMA user_version=11;")
        .unwrap();
    drop(db);
    let s = Store::open(dir.path()).unwrap();
    assert!(s.open_cached("a").is_ok());
    assert_eq!(s.cache_limit().unwrap(), 0);
    s.note_cache_use("a").unwrap();
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

/// Hot-path guard: `note_cache_use` runs for every successful cached open, so
/// its cost must not scale with the number of files remembered. A `Vec` ring
/// scanned on every hit degrades quadratically for large working sets; the
/// recency ledger must stay amortized O(1) while still persisting the exact
/// last-use order that the cache LRU consumes.
#[test]
fn recency_bookkeeping_stays_flat_for_large_working_sets() {
    const DISTINCT: usize = 65_536;
    const HITS: usize = 65_536;
    let dir = tempfile::tempdir().unwrap();
    let mut s = Store::open(dir.path()).unwrap();
    let mut nodes = Vec::with_capacity(DISTINCT + 1);
    nodes.push(node("root"));
    for i in 0..DISTINCT {
        nodes.push(node(&format!("f{i:06}")));
    }
    s.install_remote_snapshot(&nodes, "1", "server").unwrap();
    let ids: Vec<String> = (0..DISTINCT).map(|i| format!("f{i:06}")).collect();
    drop(nodes);

    let started = std::time::Instant::now();
    for id in &ids {
        s.note_cache_use(id).unwrap();
    }
    // Sliding-window hits: worst case for a scan when the id sits near the end.
    for i in 0..HITS {
        s.note_cache_use(&ids[(i * 7919) % DISTINCT]).unwrap();
    }
    let elapsed = started.elapsed();
    println!(
        "RECENCY_BENCH distinct={DISTINCT} repeat_hits={HITS} elapsed_us={} per_call_ns={}",
        elapsed.as_micros(),
        elapsed.as_nanos() / ((DISTINCT + HITS) as u128)
    );
    assert!(
        elapsed < std::time::Duration::from_secs(1),
        "note_cache_use took {elapsed:?} for {DISTINCT} distinct + {HITS} repeat hits"
    );

    // The deferred flush must reproduce the global last-use order exactly.
    let mut last_use: std::collections::HashMap<&str, usize> = ids
        .iter()
        .enumerate()
        .map(|(position, id)| (id.as_str(), position))
        .collect();
    for i in 0..HITS {
        last_use.insert(ids[(i * 7919) % DISTINCT].as_str(), DISTINCT + i);
    }
    let mut expected: Vec<(&str, usize)> = last_use.into_iter().collect();
    expected.sort_unstable_by_key(|(_, position)| *position);
    let expected: Vec<String> = expected.into_iter().map(|(id, _)| id.to_owned()).collect();

    drop(s);
    let observer = rusqlite::Connection::open(dir.path().join("state.sqlite")).unwrap();
    let order: Vec<String> = observer
        .prepare("SELECT node FROM cache_access ORDER BY seq")
        .unwrap()
        .query_map([], |row| row.get(0))
        .unwrap()
        .collect::<std::result::Result<_, _>>()
        .unwrap();
    assert_eq!(order.len(), expected.len());
    assert_eq!(order, expected);
}

fn wal_state(dir: &std::path::Path) -> (u64, Option<std::time::SystemTime>) {
    match std::fs::symlink_metadata(dir.join("state.sqlite-wal")) {
        Ok(metadata) => (metadata.len(), metadata.modified().ok()),
        Err(_) => (0, None),
    }
}

fn data_version(db: &rusqlite::Connection) -> i64 {
    db.query_row("PRAGMA data_version", [], |row| row.get(0))
        .unwrap()
}

/// Evidence for issue #22 acceptance 1: the store-touching part of a cached
/// read-only open (LRU recency) must not append a WAL frame, commit, or fsync.
/// `PRAGMA data_version` on an independent reader observes commits by the
/// writer; WAL length observes frames that a commit would append. Only closing
/// the store is allowed to fold the deferred list into one bounded transaction.
#[test]
fn cached_read_only_open_bookkeeping_does_not_write_or_fsync_the_store() {
    let (dir, s) = setup();
    let observer = rusqlite::Connection::open(dir.path().join("state.sqlite")).unwrap();
    let baseline_data = data_version(&observer);
    let baseline_wal = wal_state(dir.path());
    s.note_cache_use("c").unwrap();
    s.note_cache_use("a").unwrap();
    s.note_cache_use("b").unwrap();
    s.note_cache_use("d").unwrap();
    assert_eq!(
        data_version(&observer),
        baseline_data,
        "read-only open committed to the state database"
    );
    assert_eq!(
        wal_state(dir.path()),
        baseline_wal,
        "read-only open appended WAL frames"
    );

    // Deferred, single commit at close: still the same durable LRU order that
    // the synchronous per-open writes used to produce.
    drop(s);
    assert!(data_version(&observer) > baseline_data);
    assert!(wal_state(dir.path()).0 > baseline_wal.0);
    let order: Vec<String> = observer
        .prepare("SELECT node FROM cache_access ORDER BY seq")
        .unwrap()
        .query_map([], |row| row.get(0))
        .unwrap()
        .collect::<std::result::Result<_, _>>()
        .unwrap();
    assert_eq!(order, vec!["c", "a", "b", "d"]);
}
