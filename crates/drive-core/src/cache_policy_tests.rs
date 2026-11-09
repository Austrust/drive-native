//! Candidate-selection equivalence and read-volume evidence for the set-based
//! cache policy query. The legacy per-candidate pipeline is only compiled in
//! tests (`cache_candidates_legacy`).
use super::{CACHE_CANDIDATES_SQL, CACHE_RECENCY_CAPACITY, CacheRecency};
use crate::{DownloadTicket, Node, Status, Store, sha256};
use std::collections::HashSet;

fn unix_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

fn node(id: &str, parent: Option<&str>, directory: bool) -> Node {
    Node {
        id: id.into(),
        parent: parent.map(str::to_owned),
        name: id.into(),
        version: "1".into(),
        directory,
        metadata: None,
    }
}

fn publish(s: &mut Store, id: &str, body: &[u8]) {
    s.publish_download(
        &DownloadTicket {
            id: id.into(),
            version: "1".into(),
        },
        body,
        &sha256(body),
    )
    .unwrap();
}

fn blob_of(s: &Store, id: &str) -> String {
    s.db.query_row("SELECT blob FROM nodes WHERE id=?1", [id], |r| r.get(0))
        .unwrap()
}

/// Every protection category from `cache_is_protected` plus the structural
/// visibility rules, in one namespace.
fn fixture() -> (tempfile::TempDir, Store) {
    let dir = tempfile::tempdir().unwrap();
    let mut s = Store::open(dir.path()).unwrap();
    s.install_remote_snapshot(
        &[
            node("root", None, true),
            node("plain", Some("root"), false),
            node("plain2", Some("root"), false),
            node("plain3", Some("root"), false),
            node("shared_a", Some("root"), false),
            node("shared_b", Some("root"), false),
            node("pin", Some("root"), true),
            node("pin_child", Some("pin"), false),
            node("ov", Some("root"), true),
            node("ov_child", Some("ov"), false),
            node("den", Some("root"), true),
            node("den_child", Some("den"), false),
            node("local", Some("root"), false),
            node("draft_dirty", Some("root"), false),
            node("draft_clean", Some("root"), false),
            node("op_node", Some("root"), false),
            node("eng_node", Some("root"), false),
            node("blob_op", Some("root"), false),
            node("blob_ns", Some("root"), false),
            node("recent", Some("root"), false),
            node("active", Some("root"), false),
            node("handle", Some("root"), false),
            node("conflict", Some("root"), false),
            node("dirty_blob", Some("root"), false),
            node("updating", Some("root"), false),
            node("deleted", Some("root"), false),
            node("missing", Some("root"), false),
        ],
        "1",
        "server",
    )
    .unwrap();
    for id in [
        "plain",
        "plain2",
        "plain3",
        "shared_a",
        "shared_b",
        "pin_child",
        "ov_child",
        "den_child",
        "local",
        "draft_dirty",
        "draft_clean",
        "op_node",
        "eng_node",
        "blob_op",
        "blob_ns",
        "recent",
        "active",
        "handle",
        "conflict",
        "dirty_blob",
        "updating",
        "deleted",
        "missing",
    ] {
        publish(&mut s, id, format!("body-{id}").as_bytes());
    }
    // Same content reference on two nodes: neither may be reclaimed.
    s.db.execute(
        "UPDATE nodes SET blob=(SELECT blob FROM nodes WHERE id='shared_a') WHERE id='shared_b'",
        [],
    )
    .unwrap();
    // A dirty blob reference protects the candidate that owns that content.
    s.db.execute(
        "UPDATE nodes SET dirty_blob=?1 WHERE id='dirty_blob'",
        [blob_of(&s, "plain3")],
    )
    .unwrap();
    s.db.execute(
        "INSERT INTO drafts(node,token,base,generation,checkpoint,dirty) VALUES('draft_dirty','tok_dirty','1',1,0,1)",
        [],
    )
    .unwrap();
    // A clean draft is not a protection.
    s.db.execute(
        "INSERT INTO drafts(node,token,base,generation,checkpoint,dirty) VALUES('draft_clean','tok_clean','1',1,0,0)",
        [],
    )
    .unwrap();
    s.db.execute(
        "INSERT INTO operations(id,node,kind,base,blob,generation,state) VALUES('op_node_pending','op_node','upload','1',NULL,1,'pending')",
        [],
    )
    .unwrap();
    s.db.execute(
        "INSERT INTO operations(id,node,kind,base,blob,generation,state) VALUES('op_blob_protected','root','upload','1',?1,1,'uncertain')",
        [blob_of(&s, "blob_op")],
    )
    .unwrap();
    s.db.execute(
        "INSERT INTO namespace_ops(id,node,body,state,kind) VALUES('ns_replace','root',?1,'pending','replace')",
        [serde_json::json!({"blob": blob_of(&s, "blob_ns")}).to_string()],
    )
    .unwrap();
    s.db.execute(
        "INSERT INTO engine_bridge_intents(key,node,body,baseline,phase) VALUES('intent1','remote-node','{}',NULL,'prepared')",
        [],
    )
    .unwrap();
    s.db.execute(
        "INSERT INTO remote_aliases(remote,node) VALUES('remote-node','eng_node')",
        [],
    )
    .unwrap();
    s.db.execute(
        "INSERT INTO local_creations(node,ready) VALUES('local',1)",
        [],
    )
    .unwrap();
    s.db.execute("INSERT INTO access_denied(node) VALUES('den')", [])
        .unwrap();
    s.db.execute(
        "INSERT INTO namespace_overlay(node,remote) VALUES('ov','{}')",
        [],
    )
    .unwrap();
    s.db.execute("UPDATE nodes SET pinned=1 WHERE id='pin'", [])
        .unwrap();
    s.db.execute("UPDATE nodes SET conflict=1 WHERE id='conflict'", [])
        .unwrap();
    s.db.execute("UPDATE nodes SET local_version='0' WHERE id='updating'", [])
        .unwrap();
    s.db.execute("UPDATE nodes SET deleted=1 WHERE id='deleted'", [])
        .unwrap();
    (dir, s)
}

#[test]
fn set_based_candidates_match_legacy_pipeline() {
    let (_dir, s) = fixture();
    let now = unix_now() + 120;
    // Within the 60s publication grace window, so eviction must skip it.
    s.db.execute(
        "INSERT OR REPLACE INTO cache_published(node,at) VALUES('recent',?1)",
        [(now - 30) as i64],
    )
    .unwrap();
    let handle = s.open_cached("handle").unwrap();
    let active = HashSet::from(["active".to_string()]);
    let new = s.cache_candidates(now, &active).unwrap();
    let legacy = s.cache_candidates_legacy(now, &active).unwrap();
    assert_eq!(new, legacy);
    drop(handle);
    let ids: HashSet<&str> = new.iter().map(|(id, _)| id.as_str()).collect();
    for id in ["plain", "plain2", "draft_clean", "missing"] {
        assert!(ids.contains(id), "expected reclaimable candidate {id}");
    }
    for id in [
        "shared_a", "shared_b", "pin_child", "ov_child", "den_child", "local", "draft_dirty",
        "op_node", "eng_node", "blob_op", "blob_ns", "plain3", "recent", "active", "handle",
        "conflict", "dirty_blob", "updating", "deleted",
    ] {
        assert!(!ids.contains(id), "protected candidate {id} was selected");
    }
}

#[test]
fn missing_content_error_matches_legacy() {
    let (dir, s) = fixture();
    let now = unix_now() + 120;
    let blob = blob_of(&s, "missing");
    std::fs::remove_file(dir.path().join("blobs").join(&blob)).unwrap();
    let legacy = s
        .cache_candidates_legacy(now, &HashSet::new())
        .unwrap_err();
    let new = s.cache_candidates(now, &HashSet::new()).unwrap_err();
    assert_eq!(legacy.to_string(), new.to_string());
    assert!(new.to_string().contains("referenced content missing on disk"));
}

#[cfg(unix)]
#[test]
fn non_regular_content_error_matches_legacy() {
    let (dir, s) = fixture();
    let now = unix_now() + 120;
    let target = dir.path().join("blobs").join(blob_of(&s, "plain2"));
    let link = dir.path().join("blobs").join(blob_of(&s, "plain"));
    std::fs::remove_file(&link).unwrap();
    std::os::unix::fs::symlink(&target, &link).unwrap();
    let legacy = s
        .cache_candidates_legacy(now, &HashSet::new())
        .unwrap_err();
    let new = s.cache_candidates(now, &HashSet::new()).unwrap_err();
    assert_eq!(legacy.to_string(), new.to_string());
    assert!(new.to_string().contains("invalid cache content file"));
}

#[cfg(target_os = "linux")]
fn io_rchar() -> u64 {
    let text = std::fs::read_to_string("/proc/self/io").unwrap();
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("rchar: ") {
            return rest.trim().parse().unwrap();
        }
    }
    panic!("rchar missing from /proc/self/io");
}

/// A cache whose candidates are spread over a 60k-node namespace, like the
/// reported profile. Protection rules are mixed in so both implementations do
/// real work; the only difference measured is how often `nodes` is read.
#[cfg(target_os = "linux")]
fn perf_fixture() -> (tempfile::TempDir, Store) {
    let dir = tempfile::tempdir().unwrap();
    drop(Store::open(dir.path()).unwrap());
    let now = unix_now() + 120;
    {
        let db = rusqlite::Connection::open(dir.path().join("state.sqlite")).unwrap();
        db.execute_batch(
            r#"BEGIN;
            INSERT INTO nodes(id,parent,name,remote_version,directory) VALUES('root',NULL,'root','v',1);
            WITH RECURSIVE i(x) AS (SELECT 0 UNION ALL SELECT x+1 FROM i WHERE x<99)
              INSERT INTO nodes(id,parent,name,remote_version,directory) SELECT 'd'||x,'root','d'||x,'v',1 FROM i;
            WITH RECURSIVE k(y) AS (SELECT 0 UNION ALL SELECT y+1 FROM k WHERE y<59999)
              INSERT INTO nodes(id,parent,name,remote_version,directory,local_version,blob)
                SELECT 'f'||y,'d'||(y/600),'f'||y,'v',0,
                       CASE WHEN (y%600)<4 THEN 'v' END,
                       CASE WHEN (y%600)<4 THEN 'b'||y END FROM k;
            WITH RECURSIVE c(i) AS (SELECT 0 UNION ALL SELECT i+1 FROM c WHERE i<29)
              INSERT INTO nodes(id,parent,name,remote_version,directory)
                SELECT 'deep'||i, CASE WHEN i=0 THEN 'root' ELSE 'deep'||(i-1) END, 'deep'||i,'v',1 FROM c;
            INSERT INTO nodes(id,parent,name,remote_version,directory) VALUES('target','deep29','target','v',1);
            WITH RECURSIVE t(i) AS (SELECT 0 UNION ALL SELECT i+1 FROM t WHERE i<49)
              INSERT INTO nodes(id,parent,name,remote_version,directory,local_version,blob)
                SELECT 't'||i,'target','t'||i,'v',0,'v','tb'||i FROM t;
            INSERT INTO nodes(id,parent,name,remote_version,directory) VALUES('pin','root','pin','v',1);
            WITH RECURSIVE p(i) AS (SELECT 0 UNION ALL SELECT i+1 FROM p WHERE i<99)
              INSERT INTO nodes(id,parent,name,remote_version,directory,local_version,blob)
                SELECT 'pin'||i,'pin','pin'||i,'v',0,'v','pb'||i FROM p;
            UPDATE nodes SET pinned=1 WHERE id='pin';
            INSERT INTO nodes(id,parent,name,remote_version,directory) VALUES('ov','root','ov','v',1);
            WITH RECURSIVE o(i) AS (SELECT 0 UNION ALL SELECT i+1 FROM o WHERE i<199)
              INSERT INTO nodes(id,parent,name,remote_version,directory,local_version,blob)
                SELECT 'ov'||i,'ov','ov'||i,'v',0,'v','ob'||i FROM o;
            INSERT INTO namespace_overlay(node,remote) VALUES('ov','{}');
            INSERT INTO nodes(id,parent,name,remote_version,directory) VALUES('den','root','den','v',1);
            WITH RECURSIVE d(x) AS (SELECT 0 UNION ALL SELECT x+1 FROM d WHERE x<49)
              INSERT INTO nodes(id,parent,name,remote_version,directory,local_version,blob)
                SELECT 'den'||x,'den','den'||x,'v',0,'v','db'||x FROM d;
            INSERT INTO access_denied(node) VALUES('den');
            INSERT INTO drafts(node,token,base,generation,checkpoint,dirty) VALUES('f0','tok','v',1,0,1);
            INSERT INTO operations(id,node,kind,base,blob,generation,state) VALUES('o1','f1','upload','v','b1',1,'pending');
            INSERT INTO operations(id,node,kind,base,blob,generation,state) VALUES('o2','root','upload','v','b603',1,'uncertain');
            INSERT INTO namespace_ops(id,node,body,state,kind) VALUES('n1','root','{"blob":"b1200"}','pending','replace');
            INSERT INTO engine_bridge_intents(key,node,body,baseline,phase) VALUES('e1','f3','{}',NULL,'prepared');
            INSERT INTO local_creations(node,ready) VALUES('f4',1);
            UPDATE nodes SET dirty_blob='b602' WHERE id='f2';
            UPDATE nodes SET blob='sharedblob',local_version='v' WHERE id IN ('f600','f601');
            INSERT INTO cache_access(node) VALUES('t1');
            UPDATE nodes SET local_version='v',blob='b8' WHERE id='f8';
            COMMIT;"#,
        )
        .unwrap();
        db.execute(
            "INSERT OR REPLACE INTO cache_published(node,at) VALUES('f1201',?1)",
            [(now - 30) as i64],
        )
        .unwrap();
        let mut q = db
            .prepare("SELECT DISTINCT blob FROM nodes WHERE blob IS NOT NULL")
            .unwrap();
        let blobs: Vec<String> = q
            .query_map([], |r| r.get(0))
            .unwrap()
            .collect::<std::result::Result<_, _>>()
            .unwrap();
        drop(q);
        let blobs_dir = dir.path().join("blobs");
        for blob in blobs {
            std::fs::write(blobs_dir.join(blob), b"0123456789abcdef").unwrap();
        }
    }
    let s = Store::open(dir.path()).unwrap();
    (dir, s)
}

#[cfg(target_os = "linux")]
#[test]
#[ignore = "read-volume evidence; run explicitly on the remote Linux builder"]
fn candidate_query_read_volume_is_bounded() {
    let (_dir, s) = perf_fixture();
    let now = unix_now() + 120;
    let active = HashSet::from(["t0".to_string()]);

    let rchar_before = io_rchar();
    let new_started = std::time::Instant::now();
    let new = s.cache_candidates(now, &active).unwrap();
    let new_elapsed = new_started.elapsed();
    let rchar_after_new = io_rchar();
    let new_read = rchar_after_new - rchar_before;

    let legacy_started = std::time::Instant::now();
    let legacy = s.cache_candidates_legacy(now, &active).unwrap();
    let legacy_elapsed = legacy_started.elapsed();
    let legacy_read = io_rchar() - rchar_after_new;

    assert_eq!(new, legacy);
    assert!(new_read.saturating_mul(10) < legacy_read);
    println!("NEW:    candidates={} elapsed={new_elapsed:?} rchar={new_read} bytes", new.len());
    println!("LEGACY: candidates={} elapsed={legacy_elapsed:?} rchar={legacy_read} bytes", legacy.len());

    let mut q = s
        .db
        .prepare(&format!("EXPLAIN QUERY PLAN {CACHE_CANDIDATES_SQL}"))
        .unwrap();
    let plan: Vec<String> = q
        .query_map([], |r| r.get::<_, String>(3))
        .unwrap()
        .collect::<std::result::Result<_, _>>()
        .unwrap();
    println!("CANDIDATE PLAN:");
    for line in plan {
        println!("  {line}");
    }
}

/// The recency ledger is the hottest open-path data structure: verify exact
/// last-use ordering, duplicate compaction and bounded eviction without SQLite.
/// This is the unit-level guard for the scan/timing test in `tests/cache_policy.rs`.
#[test]
fn recency_ledger_orders_deduplicates_and_evicts_in_bounded_space() {
    let mut recency = CacheRecency::new();
    for id in ["a", "b", "a", "c", "b"] {
        recency.note(id);
    }
    assert_eq!(recency.take(), vec!["a", "c", "b"]);
    assert_eq!(recency.take(), Vec::<String>::new());

    // A hot id must not grow the eviction queue without bound.
    for _ in 0..CACHE_RECENCY_CAPACITY * 2 + 4096 {
        recency.note("hot");
    }
    assert!(recency.order.len() <= CACHE_RECENCY_CAPACITY * 2 + 64);
    assert_eq!(recency.take(), vec!["hot"]);

    // Writes past capacity drop only the oldest distinct entries.
    let mut recency = CacheRecency::new();
    for i in 0..CACHE_RECENCY_CAPACITY + 2 {
        recency.note(&format!("n{i:06}"));
    }
    let pending = recency.take();
    assert_eq!(pending.len(), CACHE_RECENCY_CAPACITY);
    assert_eq!(pending[0], "n000002");
    assert_eq!(
        pending.last().unwrap(),
        &format!("n{:06}", CACHE_RECENCY_CAPACITY + 1)
    );
}

fn cached_live_and_deleted() -> (tempfile::TempDir, Store) {
    let dir = tempfile::tempdir().unwrap();
    let mut s = Store::open(dir.path()).unwrap();
    s.install_remote_snapshot(
        &[
            node("root", None, true),
            node("live", Some("root"), false),
            node("gone", Some("root"), false),
        ],
        "1",
        "server",
    )
    .unwrap();
    publish(&mut s, "live", b"live-body");
    publish(&mut s, "gone", b"gone-body");
    s.db.execute("UPDATE nodes SET deleted=1 WHERE id='gone'", [])
        .unwrap();
    (dir, s)
}

fn node_blob(s: &Store, id: &str) -> Option<String> {
    s.db.query_row("SELECT blob FROM nodes WHERE id=?1", [id], |r| r.get(0))
        .unwrap()
}

#[test]
fn garbage_collection_reclaims_deleted_node_blob_and_leaves_live_content_reclaimable() {
    let (dir, mut s) = cached_live_and_deleted();
    let blobs = dir.path().join("blobs");
    let live_blob = blob_of(&s, "live");
    let gone_blob = blob_of(&s, "gone");
    assert!(blobs.join(&live_blob).is_file());
    assert!(blobs.join(&gone_blob).is_file());

    let reclaimed = s.collect_garbage().unwrap();
    assert_eq!(reclaimed, b"gone-body".len() as u64);
    assert!(
        !blobs.join(&gone_blob).exists(),
        "a blob only referenced by a deleted node must be reclaimed"
    );
    assert!(blobs.join(&live_blob).is_file(), "live content must survive");
    assert!(
        node_blob(&s, "gone").is_none(),
        "the deleted node must not keep a dangling blob pointer"
    );

    // Every byte left in the cache is a clean, complete body the policy may
    // reclaim; nothing is stranded as permanent protection.
    let usage = s.cache_usage(unix_now() + 120, &HashSet::new()).unwrap();
    assert_eq!(usage.content_bytes, b"live-body".len() as u64);
    assert_eq!(usage.reclaimable_bytes, b"live-body".len() as u64);
    assert_eq!(usage.protected_bytes, 0);
    assert_eq!(s.status("live").unwrap(), Status::LocalComplete);

    // No orphan files, and another pass finds nothing to collect.
    assert_eq!(std::fs::read_dir(&blobs).unwrap().count(), 1);
    assert_eq!(s.collect_garbage().unwrap(), 0);
}

#[test]
fn garbage_collection_never_reclaims_a_blob_still_referenced_by_a_live_node() {
    let (dir, mut s) = cached_live_and_deleted();
    let blobs = dir.path().join("blobs");
    let live_blob = blob_of(&s, "live");
    let gone_blob = blob_of(&s, "gone");
    // Simulate a shared pointer: the row being retired also references the
    // body that a live node still owns.
    s.db.execute(
        "UPDATE nodes SET blob=?1 WHERE id='gone'",
        [live_blob.as_str()],
    )
    .unwrap();
    s.db
        .execute("UPDATE nodes SET deleted=1 WHERE id='gone'", [])
        .unwrap();

    let reclaimed = s.collect_garbage().unwrap();
    // Only the former, now-unreferenced body of the deleted row is collected.
    assert_eq!(reclaimed, b"gone-body".len() as u64);
    assert!(
        blobs.join(&live_blob).is_file(),
        "a blob shared with a live node must never be reclaimed"
    );
    assert!(!blobs.join(&gone_blob).exists());
    assert!(node_blob(&s, "gone").is_none());

    assert_eq!(s.status("live").unwrap(), Status::LocalComplete);
    use std::io::Read;
    let mut body = Vec::new();
    s.open_cached("live").unwrap().read_to_end(&mut body).unwrap();
    assert_eq!(body, b"live-body");
}

#[test]
fn garbage_collection_keeps_the_body_of_a_deleted_conflicted_recovery_row() {
    let (_dir, mut s) = cached_live_and_deleted();
    s.db
        .execute("UPDATE nodes SET deleted=1, conflict=1 WHERE id='gone'", [])
        .unwrap();
    let gone_blob = blob_of(&s, "gone");
    assert_eq!(s.collect_garbage().unwrap(), 0);
    assert_eq!(node_blob(&s, "gone").as_deref(), Some(gone_blob.as_str()));
    // A deleted-but-conflicted row is still readable for recovery.
    use std::io::Read;
    let mut body = Vec::new();
    s.open_cached("gone").unwrap().read_to_end(&mut body).unwrap();
    assert_eq!(body, b"gone-body");
}

#[test]
fn garbage_collection_keeps_a_deleted_nodes_blob_while_an_upload_operation_references_it() {
    let (dir, mut s) = cached_live_and_deleted();
    let blobs = dir.path().join("blobs");
    let gone_blob = blob_of(&s, "gone");
    // A deleted row can still be named by an unfinished upload operation; the
    // journal owns the body until it retires.
    s.db.execute(
        "INSERT INTO operations(id,node,kind,base,blob,generation,state) VALUES('op_pending','gone','upload','1',?1,1,'pending')",
        [gone_blob.as_str()],
    )
    .unwrap();

    assert_eq!(s.collect_garbage().unwrap(), 0);
    assert!(
        blobs.join(&gone_blob).is_file(),
        "a blob pinned by a pending operation must survive garbage collection"
    );
    assert!(node_blob(&s, "gone").is_none());

    s.db.execute("DELETE FROM operations WHERE id='op_pending'", [])
        .unwrap();
    assert_eq!(s.collect_garbage().unwrap(), b"gone-body".len() as u64);
    assert!(!blobs.join(&gone_blob).exists());
}

#[test]
fn garbage_collection_keeps_a_deleted_nodes_blob_while_a_namespace_replace_references_it() {
    let (dir, mut s) = cached_live_and_deleted();
    let blobs = dir.path().join("blobs");
    let gone_blob = blob_of(&s, "gone");
    let body = serde_json::json!({"blob": gone_blob});
    s.db.execute(
        "INSERT INTO namespace_ops(id,node,body,state,kind) VALUES('ns_pending','root',?1,'pending','replace')",
        [body.to_string()],
    )
    .unwrap();

    assert_eq!(s.collect_garbage().unwrap(), 0);
    assert!(
        blobs.join(&gone_blob).is_file(),
        "a blob pinned by a pending namespace replace must survive"
    );

    s.db.execute(
        "UPDATE namespace_ops SET state='committed' WHERE id='ns_pending'",
        [],
    )
    .unwrap();
    assert_eq!(s.collect_garbage().unwrap(), b"gone-body".len() as u64);
    assert!(!blobs.join(&gone_blob).exists());
}

#[test]
fn draft_and_cache_release_journals_carry_no_body_pointer() {
    // Neither table can pin a body file: draft bodies live in drafts/ and are
    // kept by the dirty/open-draft rules, while cache_releases only tracks a
    // remote ownership hand-off and never references a cache blob.
    let (_dir, s) = cached_live_and_deleted();
    for table in ["drafts", "cache_releases"] {
        let mut statement = s
            .db
            .prepare(&format!("PRAGMA table_info({table})"))
            .unwrap();
        let columns: HashSet<String> = statement
            .query_map([], |row| row.get::<_, String>(1))
            .unwrap()
            .map(|column| column.unwrap())
            .collect();
        assert!(!columns.is_empty(), "{table} must exist");
        assert!(
            !columns.contains("blob"),
            "{table} must not point at cache body files"
        );
    }
}
