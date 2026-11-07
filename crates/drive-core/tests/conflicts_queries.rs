//! Regression coverage for the issue-candidate query behind `Store::sync_issues`.
//!
//! Until schema 27 the candidate set was a single `SELECT ... FROM nodes` whose
//! three correlated `EXISTS` subqueries forced a full `nodes` scan and, without
//! a `node` index on `operations`/`namespace_ops`, a full scan of both tables
//! for every node row (issue #16). The replacement is a four-arm `UNION` that
//! starts from the small partial indexes. These tests pin the two properties
//! that matter: the new set is exactly the old set, and the plan does not scan
//! the large tables.

use drive_core::*;
use rusqlite::Connection;
use std::collections::BTreeSet;

/// Frozen pre-#16 shape, kept verbatim as the equivalence reference.
const OLD_CANDIDATES: &str = "
    SELECT id FROM nodes n WHERE id>?1 AND (conflict=1
        OR EXISTS(SELECT 1 FROM drafts d WHERE d.node=n.id AND d.dirty=1
            AND (d.base!=n.remote_version OR n.deleted=1)
            AND NOT EXISTS(SELECT 1 FROM local_unlinks WHERE node=n.id))
        OR EXISTS(SELECT 1 FROM operations WHERE node=n.id AND state='uncertain')
        OR EXISTS(SELECT 1 FROM namespace_ops WHERE node=n.id
            AND state IN ('uncertain','cleanup_uncertain','conflict')))
    ORDER BY id LIMIT 257";

/// Copy of `ISSUE_CANDIDATES` from `crates/drive-core/src/conflicts.rs`.
/// `candidate_query_matches_the_production_constant` keeps this copy honest.
const NEW_CANDIDATES: &str = "
    SELECT id FROM (
        SELECT id FROM nodes WHERE conflict=1
        UNION
        SELECT n.id FROM nodes n JOIN drafts d ON d.node=n.id
         WHERE d.dirty=1 AND (d.base!=n.remote_version OR n.deleted=1)
           AND NOT EXISTS(SELECT 1 FROM local_unlinks WHERE node=n.id)
        UNION
        SELECT o.node FROM operations o INDEXED BY operations_unfinished
         WHERE o.state='uncertain' AND o.state NOT IN ('committed','superseded')
           AND EXISTS(SELECT 1 FROM nodes n WHERE n.id=o.node)
        UNION
        SELECT o.node FROM namespace_ops o INDEXED BY namespace_unfinished
         WHERE o.state IN ('uncertain','cleanup_uncertain','conflict')
           AND o.state NOT IN ('committed','superseded')
           AND EXISTS(SELECT 1 FROM nodes n WHERE n.id=o.node)
    )
    WHERE id>?1 ORDER BY id LIMIT 257";

fn normalize(sql: &str) -> String {
    sql.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn node(id: &str, parent: Option<&str>, version: &str) -> Node {
    Node {
        id: id.into(),
        parent: parent.map(str::to_owned),
        name: if parent.is_some() {
            "A.doc".into()
        } else {
            "root".into()
        },
        version: version.into(),
        directory: parent.is_none(),
        metadata: Some(NodeMetadata {
            size: 3,
            mtime: 1,
            mode: 0o644,
        }),
    }
}

/// Creates a task with one root directory and `leaves` file children, then
/// reopens the store. Raw state is seeded between the two opens.
fn fixture(leaves: usize) -> (tempfile::TempDir, Connection, usize) {
    let d = tempfile::tempdir().unwrap();
    let mut s = Store::open(d.path()).unwrap();
    let mut nodes = vec![node("root", None, "r")];
    for i in 0..leaves {
        nodes.push(Node {
            id: leaf(i),
            parent: Some("root".into()),
            name: format!("f{i:03}.txt"),
            version: "v1".into(),
            directory: false,
            metadata: Some(NodeMetadata {
                size: 3,
                mtime: 1,
                mode: 0o644,
            }),
        });
    }
    s.install_remote_snapshot(&nodes, "1", "server").unwrap();
    s.ensure_fs_root("root").unwrap();
    drop(s);
    let db = Connection::open(d.path().join("state.sqlite")).unwrap();
    (d, db, leaves)
}

fn leaf(i: usize) -> String {
    format!("a{i:03}")
}

/// Walks one candidate query with the same strict `id>cursor` keyset paging the
/// store uses (257 row lookahead) and returns the full ordered identity list.
fn walk(mut page: impl FnMut(&str) -> Vec<String>) -> Vec<String> {
    let mut out = Vec::new();
    let mut after = String::new();
    loop {
        let ids = page(&after);
        let done = ids.len() < 257;
        if let Some(last) = ids.last() {
            after = last.clone();
        }
        out.extend(ids);
        if done {
            break;
        }
    }
    out
}

fn page(db: &Connection, sql: &str, after: &str) -> Vec<String> {
    let mut q = db.prepare(sql).unwrap();
    let rows = q.query_map([after], |r| r.get::<_, String>(0)).unwrap();
    rows.collect::<std::result::Result<Vec<_>, _>>().unwrap()
}

fn walk_old(db: &Connection) -> Vec<String> {
    walk(|after| page(db, OLD_CANDIDATES, after))
}

fn walk_new(db: &Connection) -> Vec<String> {
    walk(|after| page(db, NEW_CANDIDATES, after))
}

/// One node per source at `i % 10`, plus a no-issue decoy at `i % 10 == 9`.
fn seed_sources(db: &Connection, leaves: usize) -> BTreeSet<String> {
    let mut expected = BTreeSet::new();
    for i in 0..leaves {
        let id = leaf(i);
        match i % 10 {
            0 => {
                db.execute(
                    "UPDATE nodes SET conflict=1, dirty_blob='conflict-blob', local_version='v1' WHERE id=?1",
                    [&id],
                )
                .unwrap();
                expected.insert(id);
            }
            1 => {
                db.execute(
                    "INSERT INTO drafts(node,token,base,generation,checkpoint,dirty) VALUES(?1,?2,'v0',1,0,1)",
                    rusqlite::params![&id, format!("token-{id}")],
                )
                .unwrap();
                expected.insert(id);
            }
            2 => {
                // A dirty draft with base equal to the remote version and no
                // deletion is not an issue; the local unlink exclusion is
                // exercised on a draft that would otherwise be one.
                db.execute(
                    "INSERT INTO drafts(node,token,base,generation,checkpoint,dirty) VALUES(?1,?2,'v0',1,0,1)",
                    rusqlite::params![&id, format!("token-{id}")],
                )
                .unwrap();
                db.execute("INSERT INTO local_unlinks(node,confirmed) VALUES(?1,0)", [&id])
                    .unwrap();
            }
            3 => {
                db.execute(
                    "INSERT INTO operations(id,node,kind,base,blob,generation,state) VALUES(?1,?2,'upload','v1',NULL,1,'uncertain')",
                    rusqlite::params![format!("op-{id}"), &id],
                )
                .unwrap();
                expected.insert(id);
            }
            4 => {
                db.execute(
                    "INSERT INTO operations(id,node,kind,base,blob,generation,state) VALUES(?1,?2,'upload','v1',NULL,1,'committed')",
                    rusqlite::params![format!("op-{id}"), &id],
                )
                .unwrap();
            }
            5 => {
                db.execute(
                    "INSERT INTO namespace_ops(id,node,body,state,kind) VALUES(?1,?2,'{}','uncertain','rename')",
                    rusqlite::params![format!("ns-{id}"), &id],
                )
                .unwrap();
                expected.insert(id);
            }
            6 => {
                db.execute(
                    "INSERT INTO namespace_ops(id,node,body,state,kind) VALUES(?1,?2,'{}','cleanup_uncertain','rename')",
                    rusqlite::params![format!("ns-{id}"), &id],
                )
                .unwrap();
                expected.insert(id);
            }
            7 => {
                db.execute(
                    "INSERT INTO namespace_ops(id,node,body,state,kind) VALUES(?1,?2,'{}','conflict','rename')",
                    rusqlite::params![format!("ns-{id}"), &id],
                )
                .unwrap();
                expected.insert(id);
            }
            8 => {
                db.execute(
                    "INSERT INTO namespace_ops(id,node,body,state,kind) VALUES(?1,?2,'{}','committed','rename')",
                    rusqlite::params![format!("ns-{id}"), &id],
                )
                .unwrap();
            }
            _ => {}
        }
    }
    // An operation that names a node outside the snapshot must not be reported:
    // the old query started from `nodes` and the new arms keep that filter.
    db.execute(
        "INSERT INTO operations(id,node,kind,base,blob,generation,state) VALUES('op-ghost','ghost','upload','v1',NULL,1,'uncertain')",
        [],
    )
    .unwrap();
    expected
}

#[test]
fn candidate_query_matches_the_production_constant() {
    let source = normalize(include_str!("../src/conflicts.rs"));
    let sql = normalize(NEW_CANDIDATES);
    assert!(
        source.contains(&sql),
        "tests/conflicts_queries.rs NEW_CANDIDATES drifted from conflicts.rs ISSUE_CANDIDATES"
    );
}

#[test]
fn old_and_new_candidate_sets_agree_on_every_seeded_source() {
    let (d, db, leaves) = fixture(200);
    let seeded = seed_sources(&db, leaves);
    drop(db);
    let store = Store::open(d.path()).unwrap();
    let db = Connection::open(d.path().join("state.sqlite")).unwrap();

    let old = walk_old(&db);
    let new = walk_new(&db);
    assert_eq!(old, new, "old and new candidate sets differ");
    assert_eq!(
        new.iter().cloned().collect::<BTreeSet<_>>(),
        seeded,
        "candidate set does not match the seeded issue sources"
    );

    // Cursor equality at every position, including cursors that fall between
    // candidates: the union must keep the strict id>cursor keyset contract.
    let mut cursors = vec![String::new()];
    cursors.extend((0..leaves).map(leaf));
    for after in cursors {
        assert_eq!(
            page(&db, OLD_CANDIDATES, &after),
            page(&db, NEW_CANDIDATES, &after),
            "page mismatch after cursor {after:?}"
        );
    }
    drop(db);
    drop(store);
}

#[test]
fn union_pages_sync_issues_across_the_257_lookahead_boundary() {
    let (d, db, leaves) = fixture(260);
    // 130 conflict nodes followed by 130 dirty drafts: both sources return a
    // `SyncIssue`, so the page entries must equal the candidate pages.
    for i in 0..leaves {
        let id = leaf(i);
        if i < 130 {
            db.execute(
                "UPDATE nodes SET conflict=1, dirty_blob='conflict-blob', local_version='v1' WHERE id=?1",
                [&id],
            )
            .unwrap();
        } else {
            db.execute(
                "INSERT INTO drafts(node,token,base,generation,checkpoint,dirty) VALUES(?1,?2,'v0',1,0,1)",
                rusqlite::params![&id, format!("token-{id}")],
            )
            .unwrap();
        }
    }
    drop(db);
    let store = Store::open(d.path()).unwrap();

    let first = store.sync_issues("root", "").unwrap();
    assert_eq!(first.entries.len(), 256);
    assert_eq!(first.next.as_deref(), Some(leaf(255).as_str()));
    let second = store
        .sync_issues("root", first.next.as_deref().unwrap())
        .unwrap();
    assert_eq!(second.entries.len(), 4);
    assert_eq!(second.next, None);

    let mut seen: Vec<String> = first
        .entries
        .iter()
        .chain(second.entries.iter())
        .map(|e| e.id.clone())
        .collect();
    seen.sort();
    let expected: Vec<String> = (0..leaves).map(leaf).collect();
    assert_eq!(seen, expected);
}

#[test]
fn candidate_query_plan_uses_the_union_indexes() {
    let (d, db, leaves) = fixture(64);
    seed_sources(&db, leaves);
    drop(db);
    let _store = Store::open(d.path()).unwrap();
    let db = Connection::open(d.path().join("state.sqlite")).unwrap();

    let plan_lines = |sql: &str| -> Vec<String> {
        let mut stmt = db.prepare(&format!("EXPLAIN QUERY PLAN {sql}")).unwrap();
        let rows = stmt.query_map([""], |r| r.get::<_, String>(3)).unwrap();
        rows.collect::<std::result::Result<Vec<_>, _>>().unwrap()
    };

    let old_plan = plan_lines(OLD_CANDIDATES);
    let new_plan = plan_lines(NEW_CANDIDATES);
    println!("old candidate plan:\n{}", old_plan.join("\n"));
    println!("new candidate plan:\n{}", new_plan.join("\n"));

    let joined = new_plan.join("\n");
    assert!(
        joined.contains("nodes_conflict"),
        "conflict arm does not use the partial nodes_conflict index:\n{joined}"
    );
    assert!(
        joined.contains("operations_unfinished"),
        "operations arm does not use the partial operations_unfinished index:\n{joined}"
    );
    assert!(
        joined.contains("namespace_unfinished"),
        "namespace arm does not use the partial namespace_unfinished index:\n{joined}"
    );

    // `SCAN (subquery-N)` is the UNION co-routine, not a table scan. `drafts`
    // is the only base table without a usable index here and holds open drafts
    // only, never the 294k-node table.
    let full_scans: Vec<&String> = new_plan
        .iter()
        .filter(|line| {
            let line = line.trim();
            line.starts_with("SCAN ")
                && !line.contains("USING")
                && !line.contains("drafts")
                && !line.contains("(subquery-")
        })
        .collect();
    assert!(
        full_scans.is_empty(),
        "unindexed large-table scan(s) in the candidate plan: {full_scans:?}\n{joined}"
    );

    // Per-identity lookups used by `sync_issue` must be covered by the new
    // non-partial indexes, which is what keeps a single issue cheap.
    let operation_lookup = plan_lines(
        "SELECT 1 FROM operations WHERE node=?1 AND state IN ('inflight','uncertain')",
    );
    let namespace_lookup = plan_lines(
        "SELECT 1 FROM namespace_ops WHERE node=?1 AND state IN ('uncertain','cleanup_uncertain','inflight','cleanup_inflight')",
    );
    assert!(
        operation_lookup.join("\n").contains("operations_node"),
        "operations per-node lookup does not use operations_node: {operation_lookup:?}"
    );
    assert!(
        namespace_lookup.join("\n").contains("namespace_ops_node"),
        "namespace_ops per-node lookup does not use namespace_ops_node: {namespace_lookup:?}"
    );
}

/// Kernel view of bytes read by this process; the throughput question in the
/// issue is about DB page reads, not wall clock alone.
fn rchar() -> Option<u64> {
    let text = std::fs::read_to_string("/proc/self/io").ok()?;
    text.lines()
        .find_map(|line| line.strip_prefix("rchar:"))?
        .trim()
        .parse()
        .ok()
}

#[test]
#[ignore = "294k-node cost probe; run with --ignored --nocapture"]
fn issue_query_cost_on_a_294k_node_database() {
    const NODES: usize = 294_000;
    let d = tempfile::tempdir().unwrap();
    let mut s = Store::open(d.path()).unwrap();
    s.install_remote_snapshot(&[node("root", None, "r")], "1", "server")
        .unwrap();
    s.ensure_fs_root("root").unwrap();
    drop(s);
    {
        let mut db = Connection::open(d.path().join("state.sqlite")).unwrap();
        let tx = db.transaction().unwrap();
        {
            let mut nodes = tx
                .prepare(
                    "INSERT INTO nodes(id,parent,name,remote_version,directory,conflict,pinned) VALUES(?1,'root',?2,'v1',0,?3,0)",
                )
                .unwrap();
            let mut operations = tx
                .prepare(
                    "INSERT INTO operations(id,node,kind,base,generation,state) VALUES(?1,?2,'upload','v1',1,?3)",
                )
                .unwrap();
            let mut namespace = tx
                .prepare(
                    "INSERT INTO namespace_ops(id,node,body,state,kind) VALUES(?1,?2,'{}',?3,'rename')",
                )
                .unwrap();
            let mut drafts = tx
                .prepare(
                    "INSERT INTO drafts(node,token,base,generation,checkpoint,dirty) VALUES(?1,?2,'v0',1,0,1)",
                )
                .unwrap();
            for i in 0..NODES {
                let id = format!("n{i:06}");
                nodes
                    .execute(rusqlite::params![&id, format!("f{i:06}.txt"), i % 1000 == 0])
                    .unwrap();
                let operation = if i % 10_000 == 0 { "uncertain" } else { "committed" };
                operations
                    .execute(rusqlite::params![format!("op{i:06}"), &id, operation])
                    .unwrap();
                let namespace_op = if i % 20_000 == 0 { "uncertain" } else { "committed" };
                namespace
                    .execute(rusqlite::params![format!("ns{i:06}"), &id, namespace_op])
                    .unwrap();
                if i % 5_000 == 1 {
                    drafts
                        .execute(rusqlite::params![&id, format!("t{i:06}")])
                        .unwrap();
                }
            }
        }
        tx.commit().unwrap();
    }
    let store = Store::open(d.path()).unwrap();
    let db = Connection::open(d.path().join("state.sqlite")).unwrap();
    let plan_lines = |sql: &str| -> Vec<String> {
        let mut stmt = db.prepare(&format!("EXPLAIN QUERY PLAN {sql}")).unwrap();
        let rows = stmt.query_map([""], |r| r.get::<_, String>(3)).unwrap();
        rows.collect::<std::result::Result<Vec<_>, _>>().unwrap()
    };

    println!("v27 candidate plan:\n{}", plan_lines(NEW_CANDIDATES).join("\n"));

    // Warm page cache/statements before measuring so the number reflects the
    // query, not the first touch of the file.
    let warm = store.sync_issues("root", "").unwrap();
    let before = rchar();
    let start = std::time::Instant::now();
    let first_page = store.sync_issues("root", "").unwrap();
    let elapsed = start.elapsed();
    let after = rchar();
    println!(
        "sync_issues 294k nodes: entries={} next={:?} elapsed={elapsed:?} rchar_delta={:?} (warm entries={})",
        first_page.entries.len(),
        first_page.next,
        after
            .zip(before)
            .map(|(after, before)| after.saturating_sub(before)),
        warm.entries.len()
    );

    let before = rchar();
    let start = std::time::Instant::now();
    let candidates = page(&db, NEW_CANDIDATES, "");
    let elapsed = start.elapsed();
    let after = rchar();
    println!(
        "v27 candidate query 294k nodes: ids={} elapsed={elapsed:?} rchar_delta={:?}",
        candidates.len(),
        after.zip(before).map(|(after, before)| after.saturating_sub(before))
    );

    // Same database, same indexes, pre-fix SQL shape: this is the cost issue
    // #16 paid on every GetIssues call.
    let before = rchar();
    let start = std::time::Instant::now();
    let old = page(&db, OLD_CANDIDATES, "");
    let elapsed = start.elapsed();
    let after = rchar();
    println!(
        "v27-schema old candidate query 294k nodes: ids={} elapsed={elapsed:?} rchar_delta={:?}",
        old.len(),
        after.zip(before).map(|(after, before)| after.saturating_sub(before))
    );

    // Simulated pre-fix schema: the old shape relies on the very indexes the
    // v27 upgrade adds for its correlated `EXISTS` subqueries. Only the plan is
    // printed: executing the old shape here is quadratic in the number of
    // candidates and would take hours, which is exactly the issue #16 cost.
    db.execute_batch(
        "DROP INDEX nodes_conflict; DROP INDEX nodes_pinned; DROP INDEX nodes_cached; DROP INDEX operations_node; DROP INDEX namespace_ops_node;",
    )
    .unwrap();
    println!(
        "v26 (indexes dropped) old candidate plan:\n{}",
        plan_lines(OLD_CANDIDATES).join("\n")
    );
    db.execute_batch(
        "CREATE INDEX nodes_conflict ON nodes(id) WHERE conflict=1; CREATE INDEX nodes_pinned ON nodes(id) WHERE pinned=1; CREATE INDEX nodes_cached ON nodes(id,local_version,remote_version) WHERE blob IS NOT NULL; CREATE INDEX operations_node ON operations(node); CREATE INDEX namespace_ops_node ON namespace_ops(node);",
    )
    .unwrap();
    drop(db);
    drop(store);
}

#[test]
fn zero_issue_store_returns_an_empty_page() {
    let (d, db, _) = fixture(0);
    drop(db);
    let store = Store::open(d.path()).unwrap();
    let page = store.sync_issues("root", "").unwrap();
    assert!(page.entries.is_empty());
    assert_eq!(page.next, None);
}

#[test]
fn oversized_issue_cursor_is_rejected_before_the_candidate_query() {
    let (d, db, _) = fixture(0);
    drop(db);
    let store = Store::open(d.path()).unwrap();
    let after = "a".repeat(8193);
    match store.sync_issues("root", &after) {
        Err(Error::State(message)) => assert_eq!(message, "invalid issue cursor"),
        _ => panic!("expected the invalid-cursor error"),
    }
    // Exactly the documented budget is still accepted.
    let page = store.sync_issues("root", &"a".repeat(8192)).unwrap();
    assert!(page.entries.is_empty());
    assert_eq!(page.next, None);
}

#[test]
fn missing_root_is_an_error_instead_of_an_empty_page() {
    // The old implementation resolved `root` first and propagated the lookup
    // error; the union query must not silently turn that into an empty page.
    let d = tempfile::tempdir().unwrap();
    let store = Store::open(d.path()).unwrap();
    match store.sync_issues("root", "") {
        Err(Error::Db(_)) => {}
        _ => panic!("expected the node lookup to fail"),
    }
}
