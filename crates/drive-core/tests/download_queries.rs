//! Query-shape regression tests for `Store::downloads_needed`: the union of
//! candidate sources must select exactly the rows the pre-fix single-statement
//! scan selected, while the plan starts from the small v27 indexes.

use drive_core::*;
use rusqlite::Connection;

/// Frozen pre-fix statement, byte for byte the query embedded in
/// `Store::downloads_needed` before issue #15/#16.
const OLD_DOWNLOADS: &str = "WITH RECURSIVE pinned_nodes(id) AS (SELECT id FROM nodes WHERE pinned=1 UNION SELECT n.id FROM nodes n JOIN pinned_nodes p ON n.parent=p.id) SELECT n.id,n.remote_version,n.blob IS NOT NULL FROM nodes n WHERE n.directory=0 AND n.deleted=0 AND n.id NOT IN (SELECT id FROM access_blocked) AND n.dirty_blob IS NULL AND (n.blob IS NULL OR n.local_version!=n.remote_version) AND (n.blob IS NOT NULL OR n.id IN (SELECT id FROM pinned_nodes) OR EXISTS(SELECT 1 FROM transfers t WHERE t.node=n.id AND t.version=n.remote_version AND t.direction='download' AND t.state IN ('pending','inflight','retry','failed')))";

/// Verbatim copy of the production constant. `candidate_query_matches_the_production_constant`
/// fails if the two drift apart.
const NEW_DOWNLOADS: &str = "\
WITH RECURSIVE pinned_nodes(id,remote_version,cached,local_version,directory,deleted,dirty_blob) AS (
    SELECT id,remote_version,blob IS NOT NULL,local_version,directory,deleted,dirty_blob
      FROM nodes WHERE pinned=1
    UNION
    SELECT n.id,n.remote_version,n.blob IS NOT NULL,n.local_version,n.directory,n.deleted,n.dirty_blob
      FROM nodes n JOIN pinned_nodes p ON n.parent=p.id),
candidates(id,remote_version,cached,local_version,directory,deleted,dirty_blob) AS (
    SELECT id,remote_version,blob IS NOT NULL,local_version,directory,deleted,dirty_blob
      FROM nodes WHERE blob IS NOT NULL AND local_version != remote_version
    UNION
    SELECT id,remote_version,cached,local_version,directory,deleted,dirty_blob FROM pinned_nodes
    UNION
    SELECT n.id,n.remote_version,n.blob IS NOT NULL,n.local_version,n.directory,n.deleted,n.dirty_blob
      FROM transfers t JOIN nodes n ON n.id=t.node AND n.remote_version=t.version
     WHERE t.direction='download' AND t.state IN ('pending','inflight','retry','failed'))
SELECT id,remote_version,cached FROM candidates c
WHERE directory=0 AND deleted=0 AND id NOT IN (SELECT id FROM access_blocked)
  AND dirty_blob IS NULL AND (cached=0 OR local_version!=remote_version)";

fn normalize(sql: &str) -> String {
    sql.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn node(id: &str, parent: Option<&str>, directory: bool) -> Node {
    Node {
        id: id.into(),
        parent: parent.map(str::to_owned),
        name: format!("{id}.dat"),
        version: "1".into(),
        directory,
        metadata: None,
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Row {
    id: String,
    version: String,
    cached: bool,
}

fn rows(db: &Connection, sql: &str) -> Vec<Row> {
    let mut stmt = db.prepare(sql).unwrap();
    let mut rows = stmt
        .query_map([], |r| {
            Ok(Row {
                id: r.get(0)?,
                version: r.get(1)?,
                cached: r.get(2)?,
            })
        })
        .unwrap();
    rows.by_ref().map(|row| row.unwrap()).collect()
}

fn plan(db: &Connection, sql: &str) -> Vec<String> {
    let mut stmt = db
        .prepare(&format!("EXPLAIN QUERY PLAN {sql}"))
        .unwrap();
    let rows = stmt.query_map([], |r| r.get::<_, String>(3)).unwrap();
    rows.map(|line| line.unwrap()).collect()
}

/// Plans for the three candidate sources, so a regression names the branch that
/// lost its index instead of only the joined statement.
fn branch_plans(db: &Connection) -> Vec<(&'static str, Vec<String>)> {
    vec![
        (
            "stale",
            plan(
                db,
                "SELECT id FROM nodes WHERE blob IS NOT NULL AND local_version != remote_version",
            ),
        ),
        (
            "pinned",
            plan(
                db,
                "WITH RECURSIVE pinned_nodes(id) AS (SELECT id FROM nodes WHERE pinned=1 UNION SELECT n.id FROM nodes n JOIN pinned_nodes p ON n.parent=p.id) SELECT id FROM pinned_nodes",
            ),
        ),
        (
            "intent",
            plan(
                db,
                "SELECT t.node FROM transfers t JOIN nodes n ON n.id=t.node AND n.remote_version=t.version WHERE t.direction='download' AND t.state IN ('pending','inflight','retry','failed')",
            ),
        ),
    ]
}

/// Builds a database where every candidate source and every protection filter
/// has a representative row.
fn seeded_fixture() -> (tempfile::TempDir, Vec<Row>, Vec<Row>) {
    let dir = tempfile::tempdir().unwrap();
    let mut store = Store::open(dir.path()).unwrap();
    store
        .install_remote_snapshot(
            &[
                node("root", None, true),
                node("folder", Some("root"), true),
                node("p1", Some("folder"), false),
                node("p_draft", Some("folder"), false),
                node("stale", Some("root"), false),
                node("uptodate", Some("root"), false),
                node("plain", Some("root"), false),
                node("intent", Some("root"), false),
                node("intent_old", Some("root"), false),
                node("intent_done", Some("root"), false),
                node("dirty", Some("root"), false),
                node("dir_stale", Some("root"), true),
                node("deleted", Some("root"), false),
                node("blocked_dir", Some("root"), true),
                node("blocked_child", Some("blocked_dir"), false),
                node("release", Some("root"), false),
            ],
            "1",
            "server",
        )
        .unwrap();
    store.ensure_fs_root("root").unwrap();
    drop(store);

    let db = Connection::open(dir.path().join("state.sqlite")).unwrap();
    db.execute_batch(
        "
        UPDATE nodes SET pinned=1 WHERE id='folder';
        UPDATE nodes SET blob='blob-stale', local_version='0' WHERE id='stale';
        UPDATE nodes SET blob='blob-up', local_version='1' WHERE id='uptodate';
        UPDATE nodes SET blob='blob-dirty', local_version='0', dirty_blob='blob-dirty-next' WHERE id='dirty';
        UPDATE nodes SET blob='blob-dir', local_version='0' WHERE id='dir_stale';
        UPDATE nodes SET blob='blob-deleted', local_version='0', deleted=1 WHERE id='deleted';
        UPDATE nodes SET blob='blob-blocked', local_version='0' WHERE id='blocked_child';
        UPDATE nodes SET blob='blob-release', local_version='0' WHERE id='release';
        INSERT INTO access_denied(node) VALUES('blocked_dir');
        INSERT INTO transfers(id,node,version,direction,state) VALUES('t1','intent','1','download','pending');
        INSERT INTO transfers(id,node,version,direction,state) VALUES('t2','intent_old','0','download','pending');
        INSERT INTO transfers(id,node,version,direction,state) VALUES('t3','intent_done','1','download','committed');
        INSERT INTO cache_releases(node,version,state,attempts,retry_at) VALUES('release','1','pending',0,0);
        INSERT INTO drafts(node,token,base,generation,checkpoint,dirty) VALUES('p_draft','draft-token','0',1,0,1);
        ",
    )
    .unwrap();
    drop(db);

    // Selected by both statements: stale cached content, an unpinned child of a
    // pinned subtree, the pinned dirty draft (the loop later skips it), a live
    // download intent, and the pending cache release (the loop later skips it).
    let selected = vec![
        Row {
            id: "intent".into(),
            version: "1".into(),
            cached: false,
        },
        Row {
            id: "p1".into(),
            version: "1".into(),
            cached: false,
        },
        Row {
            id: "p_draft".into(),
            version: "1".into(),
            cached: false,
        },
        Row {
            id: "release".into(),
            version: "1".into(),
            cached: true,
        },
        Row {
            id: "stale".into(),
            version: "1".into(),
            cached: true,
        },
    ];
    // Rows the loop still filters out before returning tickets.
    let loop_skips = vec![
        Row {
            id: "p_draft".into(),
            version: "1".into(),
            cached: false,
        },
        Row {
            id: "release".into(),
            version: "1".into(),
            cached: true,
        },
    ];
    (dir, selected, loop_skips)
}

#[test]
fn candidate_query_matches_the_production_constant() {
    let production = include_str!("../src/state.rs");
    assert!(
        normalize(production).contains(&normalize(NEW_DOWNLOADS)),
        "downloads_needed query drifted from the production constant"
    );
}

#[test]
fn old_and_new_download_queries_select_exactly_the_same_rows() {
    let (dir, mut expected, loop_skips) = seeded_fixture();
    let db = Connection::open(dir.path().join("state.sqlite")).unwrap();
    let mut old = rows(&db, OLD_DOWNLOADS);
    let mut new = rows(&db, NEW_DOWNLOADS);
    old.sort();
    new.sort();
    assert_eq!(old, new, "union candidates changed the selected rows");
    expected.sort();
    assert_eq!(new, expected);

    // And the public method agrees, after its per-candidate protection checks:
    // the pending cache release and the dirty draft node are skipped.
    drop(db);
    let store = Store::open(dir.path()).unwrap();
    let mut jobs = store.downloads_needed().unwrap();
    jobs.sort_by(|a, b| a.id.cmp(&b.id));
    assert_eq!(jobs.len(), expected.len() - loop_skips.len());
    assert_eq!(
        jobs.iter().map(|ticket| &ticket.id).collect::<Vec<_>>(),
        vec!["intent", "p1", "stale"]
    );
    assert!(jobs.iter().all(|ticket| ticket.version == "1"));
    assert!(!jobs.iter().any(|ticket| ticket.id == "release"));
    assert!(!jobs.iter().any(|ticket| ticket.id == "p_draft"));
    assert!(!jobs.iter().any(|ticket| ticket.id == "uptodate"));
    assert!(!jobs.iter().any(|ticket| ticket.id == "blocked_child"));
}

#[test]
fn download_query_plan_uses_the_candidate_indexes() {
    let (dir, _, _) = seeded_fixture();
    let db = Connection::open(dir.path().join("state.sqlite")).unwrap();
    let new_plan = plan(&db, NEW_DOWNLOADS);
    println!("new downloads plan:\n{}", new_plan.join("\n"));
    let old_plan = plan(&db, OLD_DOWNLOADS);
    println!("old downloads plan:\n{}", old_plan.join("\n"));
    let branches = branch_plans(&db);
    for (name, lines) in &branches {
        println!("{name} branch plan:\n{}", lines.join("\n"));
    }
    let stale = &branches[0].1;
    let pinned = &branches[1].1;
    let intent = &branches[2].1;
    assert!(
        stale.iter().any(|line| line.contains("nodes_cached")),
        "stale branch does not start from nodes_cached: {stale:?}"
    );
    assert!(
        pinned.iter().any(|line| line.contains("nodes_pinned")),
        "pinned subtree branch does not start from nodes_pinned: {pinned:?}"
    );
    // The download-intent branch scans the (small) transfers queue once and
    // probes nodes by rowid; it must never produce a correlated per-node probe.
    assert!(
        intent.iter().any(|line| line.contains("transfers") || line.trim() == "SCAN t"),
        "download intent branch lost the transfers source: {intent:?}"
    );
    assert!(
        !intent.iter().any(|line| line.contains("CORRELATED")),
        "download intent branch became correlated again: {intent:?}"
    );
    assert!(new_plan.iter().any(|line| line.contains("nodes_cached")));
    assert!(new_plan.iter().any(|line| line.contains("nodes_pinned")));
    // The outer statement reads the small candidate CTE only: it must not
    // drive a full scan of nodes the way the frozen old statement does.
    assert!(
        !new_plan.iter().any(|line| {
            let line = line.trim();
            line == "SCAN n" || line == "SCAN nodes"
        }),
        "the rewritten statement still full-scans nodes: {new_plan:?}"
    );
    assert!(
        !new_plan.iter().any(|line| line.contains("CORRELATED")),
        "correlated subqueries survived the rewrite: {new_plan:?}"
    );
    assert!(
        old_plan.first().map(|line| line.trim()) == Some("SCAN n"),
        "the frozen old statement should still scan the nodes table: {old_plan:?}"
    );
    assert!(
        old_plan.iter().any(|line| line.contains("CORRELATED")),
        "the frozen old statement should be the correlated shape: {old_plan:?}"
    );
    assert!(
        old_plan
            .iter()
            .any(|line| line.contains("sqlite_autoindex_transfers_2")),
        "the frozen old statement should probe transfers per node: {old_plan:?}"
    );
}

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
fn downloads_needed_cost_on_a_294k_node_database() {
    const NODES: usize = 294_000;
    const PINNED_CHILDREN: usize = 200;
    let dir = tempfile::tempdir().unwrap();
    let mut store = Store::open(dir.path()).unwrap();
    store
        .install_remote_snapshot(
            &[node("root", None, true), node("folder", Some("root"), true)],
            "1",
            "server",
        )
        .unwrap();
    store.ensure_fs_root("root").unwrap();
    drop(store);
    {
        let mut db = Connection::open(dir.path().join("state.sqlite")).unwrap();
        let tx = db.transaction().unwrap();
        tx.execute("UPDATE nodes SET pinned=1 WHERE id='folder'", [])
            .unwrap();
        {
            let mut nodes = tx
                .prepare(
                    "INSERT INTO nodes(id,parent,name,remote_version,directory,blob,local_version) VALUES(?1,'root',?2,'v1',0,?3,?4)",
                )
                .unwrap();
            for i in 0..NODES {
                let id = format!("n{i:06}");
                // Every 1000th node is a stale cached file: the only rows the
                // stale branch should contribute.
                if i % 1000 == 0 {
                    nodes
                        .execute(rusqlite::params![
                            &id,
                            format!("f{i:06}.txt"),
                            format!("blob{i:06}"),
                            "v0"
                        ])
                        .unwrap();
                } else {
                    nodes
                        .execute(rusqlite::params![&id, format!("f{i:06}.txt"), None::<String>, None::<String>])
                        .unwrap();
                }
            }
            let mut pinned = tx
                .prepare(
                    "INSERT INTO nodes(id,parent,name,remote_version,directory) VALUES(?1,'folder',?2,'v1',0)",
                )
                .unwrap();
            for i in 0..PINNED_CHILDREN {
                pinned
                    .execute(rusqlite::params![format!("p{i:04}"), format!("p{i:04}.txt")])
                    .unwrap();
            }
        }
        tx.commit().unwrap();
    }

    let store = Store::open(dir.path()).unwrap();
    let _ = store.downloads_needed().unwrap(); // warm statements/page cache
    let database = dir.path().join("state.sqlite");
    let read = Connection::open(&database).unwrap();

    let before = rchar();
    let start = std::time::Instant::now();
    let old = rows(&read, OLD_DOWNLOADS);
    let old_elapsed = start.elapsed();
    let old_rchar = rchar()
        .zip(before)
        .map(|(after, before)| after.saturating_sub(before));

    let before = rchar();
    let start = std::time::Instant::now();
    let new = rows(&read, NEW_DOWNLOADS);
    let new_sql_elapsed = start.elapsed();
    let new_sql_rchar = rchar()
        .zip(before)
        .map(|(after, before)| after.saturating_sub(before));

    let before = rchar();
    let start = std::time::Instant::now();
    let jobs = store.downloads_needed().unwrap();
    let elapsed = start.elapsed();
    let rchar_delta = rchar()
        .zip(before)
        .map(|(after, before)| after.saturating_sub(before));

    assert_eq!(old, new);
    // 294 stale cached files + 200 uncached pinned children; the loop settles
    // each candidate, so it returns the same rows it selected.
    assert_eq!(jobs.len(), NODES / 1000 + PINNED_CHILDREN);
    println!(
        "downloads_needed 294k nodes: jobs={} old_sql={old_elapsed:?}/{old_rchar:?} new_sql={new_sql_elapsed:?}/{new_sql_rchar:?} downloads_needed={elapsed:?}/{rchar_delta:?}",
        jobs.len()
    );
    println!(
        "new downloads plan:\n{}",
        plan(&read, NEW_DOWNLOADS).join("\n")
    );
}
