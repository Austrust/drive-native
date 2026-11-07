//! Scoped desktop seed equivalence and read-volume evidence.
use super::{SEED_COLUMNS, SEED_FILTERS, StatusSeed, scoped_seed_sql};
use crate::{Node, Store};
use std::collections::{HashMap, HashSet};

fn node(id: &str, parent: Option<&str>, directory: bool) -> Node {
    Node {
        id: id.into(),
        parent: parent.map(str::to_owned),
        name: id.into(),
        version: "v".into(),
        directory,
        metadata: None,
    }
}

/// Visible full-namespace rows plus the raw parent/child edges the expected
/// scoped set is derived from. The full query is the unchanged reference.
fn small_fixture() -> (tempfile::TempDir, Store) {
    let dir = tempfile::tempdir().unwrap();
    let s = Store::open(dir.path()).unwrap();
    let mut s = s;
    s.install_remote_snapshot(
        &[
            node("root", None, true),
            node("a", Some("root"), true),
            node("a1", Some("a"), false),
            node("a2", Some("a"), true),
            node("a2a", Some("a2"), false),
            node("b", Some("root"), true),
            node("b1", Some("b"), false),
            node("c", Some("root"), false),
            node("pinned", Some("root"), true),
            node("p1", Some("pinned"), false),
            node("ov", Some("root"), true),
            node("ov1", Some("ov"), false),
            node("den", Some("root"), true),
            node("den1", Some("den"), false),
            node("gone", Some("root"), true),
            node("gone_child", Some("gone"), false),
            node("shadowed", Some("a"), false),
        ],
        "1",
        "server",
    )
    .unwrap();
    s.db.execute("UPDATE nodes SET pinned=1 WHERE id='pinned'", [])
        .unwrap();
    s.db.execute(
        "INSERT INTO namespace_overlay(node,remote) VALUES('ov','{}')",
        [],
    )
    .unwrap();
    s.db.execute("INSERT INTO access_denied(node) VALUES('den')", [])
        .unwrap();
    // Detailed flags: blob, dirty blob, conflict, draft, download transfer.
    s.db.execute("UPDATE nodes SET blob='blob-a1' WHERE id='a1'", [])
        .unwrap();
    s.db.execute("UPDATE nodes SET dirty_blob='blob-b1' WHERE id='b1'", [])
        .unwrap();
    s.db.execute("UPDATE nodes SET conflict=1 WHERE id='c'", [])
        .unwrap();
    s.db.execute(
        "INSERT INTO drafts(node,token,base,generation,checkpoint,dirty) VALUES('a2a','tok','v',1,0,0)",
        [],
    )
    .unwrap();
    s.db.execute(
        "INSERT INTO transfers(id,node,version,direction,state) VALUES('tr','b1','v','download','pending')",
        [],
    )
    .unwrap();
    s.db.execute("UPDATE nodes SET deleted=1 WHERE id='gone'", [])
        .unwrap();
    // A local creation with the same parent/name shadows the remote node.
    s.db.execute(
        "INSERT INTO nodes(id,parent,name,remote_version,directory) VALUES('local_shadow','a','shadowed','l',0)",
        [],
    )
    .unwrap();
    s.db.execute(
        "INSERT INTO local_creations(node,ready) VALUES('local_shadow',1)",
        [],
    )
    .unwrap();
    (dir, s)
}

fn raw_tree(s: &Store) -> (HashMap<String, Option<String>>, HashMap<String, Vec<String>>) {
    let mut q = s.db.prepare("SELECT id,parent FROM nodes").unwrap();
    let rows: Vec<(String, Option<String>)> = q
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
        .unwrap()
        .collect::<std::result::Result<_, _>>()
        .unwrap();
    let mut parents = HashMap::new();
    let mut children: HashMap<String, Vec<String>> = HashMap::new();
    for (id, parent) in rows {
        if let Some(parent) = &parent {
            children.entry(parent.clone()).or_default().push(id.clone());
        }
        parents.insert(id, parent);
    }
    (parents, children)
}

fn expected_ids(
    roots: &[String],
    parents: &HashMap<String, Option<String>>,
    children: &HashMap<String, Vec<String>>,
    visible: &HashSet<String>,
) -> HashSet<String> {
    let mut set = HashSet::new();
    let mut stack = roots.to_vec();
    while let Some(id) = stack.pop() {
        if set.insert(id.clone()) {
            if let Some(kids) = children.get(&id) {
                stack.extend(kids.iter().cloned());
            }
        }
    }
    for root in roots {
        let mut current = Some(root.clone());
        while let Some(id) = current {
            set.insert(id.clone());
            current = parents.get(&id).cloned().flatten();
        }
    }
    set.retain(|id| visible.contains(id));
    set
}

type SeedTuple = (String, Option<String>, bool, bool, bool, bool);

fn tuple(seed: &StatusSeed) -> SeedTuple {
    (
        seed.id.clone(),
        seed.parent.clone(),
        seed.directory,
        seed.pinned,
        seed.overlay,
        seed.detailed,
    )
}

fn tuples(seeds: &[StatusSeed]) -> Vec<SeedTuple> {
    let mut rows: Vec<SeedTuple> = seeds.iter().map(tuple).collect();
    rows.sort();
    rows
}

#[test]
fn scoped_seed_rows_match_the_full_projection_of_tree_and_ancestors() {
    let (_dir, s) = small_fixture();
    let full = s.desktop_status_seeds().unwrap();
    let visible: HashSet<String> = full.iter().map(|seed| seed.id.clone()).collect();
    let (parents, children) = raw_tree(&s);
    for roots in [
        vec!["root"],
        vec!["a"],
        vec!["a2"],
        vec!["a1"],
        vec!["a2a"],
        vec!["a", "b"],
        vec!["c", "p1"],
        vec!["ov"],
        vec!["den"],
        vec!["den1"],
        vec!["gone"],
        vec!["gone_child"],
        vec!["shadowed"],
        vec!["local_shadow"],
        vec!["root", "a1"],
    ] {
        let roots: Vec<String> = roots.into_iter().map(str::to_owned).collect();
        let scoped = s.desktop_status_seeds_for(&roots).unwrap();
        let ids = expected_ids(&roots, &parents, &children, &visible);
        let expected: Vec<StatusSeed> = full
            .iter()
            .filter(|seed| ids.contains(&seed.id))
            .cloned()
            .collect();
        assert_eq!(tuples(&scoped), tuples(&expected), "roots {roots:?}");
    }
    assert!(s.desktop_status_seeds_for(&[]).unwrap().is_empty());
}

#[test]
fn scoped_query_runs_on_a_query_only_connection() {
    let (dir, s) = small_fixture();
    let roots = vec!["a".to_owned()];
    let scoped = s.desktop_status_seeds_for(&roots).unwrap();
    let db = rusqlite::Connection::open(dir.path().join("state.sqlite")).unwrap();
    db.pragma_update(None, "query_only", "ON").unwrap();
    db.busy_timeout(std::time::Duration::from_secs(5)).unwrap();
    let mut q = db.prepare(&scoped_seed_sql()).unwrap();
    let mut rows: Vec<SeedTuple> = q
        .query_map([serde_json::to_string(&roots).unwrap()], |r| {
            Ok((
                r.get(0)?,
                r.get(1)?,
                r.get(2)?,
                r.get(3)?,
                r.get(4)?,
                r.get(5)?,
            ))
        })
        .unwrap()
        .collect::<std::result::Result<_, _>>()
        .unwrap();
    drop(q);
    rows.sort();
    assert_eq!(rows, tuples(&scoped));
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

/// Namespace close to the reported profile: ~200k nodes, one 50-child page at
/// depth 31, plus mixed pin/overlay/denied/draft flags.
#[cfg(target_os = "linux")]
fn perf_fixture() -> (tempfile::TempDir, Store) {
    let dir = tempfile::tempdir().unwrap();
    drop(Store::open(dir.path()).unwrap());
    {
        let db = rusqlite::Connection::open(dir.path().join("state.sqlite")).unwrap();
        db.execute_batch(
            r#"BEGIN;
            INSERT INTO nodes(id,parent,name,remote_version,directory) VALUES('root',NULL,'root','v',1);
            WITH RECURSIVE i(x) AS (SELECT 0 UNION ALL SELECT x+1 FROM i WHERE x<99)
              INSERT INTO nodes(id,parent,name,remote_version,directory)
                SELECT 'd'||x,'root','d'||x,'v',1 FROM i;
            WITH RECURSIVE k(y) AS (SELECT 0 UNION ALL SELECT y+1 FROM k WHERE y<199899)
              INSERT INTO nodes(id,parent,name,remote_version,directory,local_version,blob)
                SELECT 'f'||y,'d'||(y/1999),'f'||y,'v',0,
                       CASE WHEN (y%1999)<4 THEN 'v' END,
                       CASE WHEN (y%1999)<4 THEN 'b'||y END FROM k;
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
                SELECT 'p'||i,'pin','p'||i,'v',0,'v','pb'||i FROM p;
            UPDATE nodes SET pinned=1 WHERE id='pin';
            INSERT INTO nodes(id,parent,name,remote_version,directory) VALUES('ov','root','ov','v',1);
            WITH RECURSIVE o(i) AS (SELECT 0 UNION ALL SELECT i+1 FROM o WHERE i<199)
              INSERT INTO nodes(id,parent,name,remote_version,directory,local_version,blob)
                SELECT 'o'||i,'ov','o'||i,'v',0,'v','ob'||i FROM o;
            INSERT INTO namespace_overlay(node,remote) VALUES('ov','{}');
            INSERT INTO nodes(id,parent,name,remote_version,directory) VALUES('den','root','den','v',1);
            WITH RECURSIVE d(x) AS (SELECT 0 UNION ALL SELECT x+1 FROM d WHERE x<49)
              INSERT INTO nodes(id,parent,name,remote_version,directory,local_version,blob)
                SELECT 'den'||x,'den','den'||x,'v',0,'v','db'||x FROM d;
            INSERT INTO access_denied(node) VALUES('den');
            INSERT INTO nodes(id,parent,name,remote_version,directory) VALUES('local_shadow','d2','f2','l',0);
            INSERT INTO local_creations(node,ready) VALUES('local_shadow',1);
            INSERT INTO drafts(node,token,base,generation,checkpoint,dirty) VALUES('f0','tok','v',1,0,1);
            INSERT INTO transfers(id,node,version,direction,state) VALUES('tr','f4','v','download','pending');
            UPDATE nodes SET deleted=1 WHERE id IN ('f100','f101','f102','f103','f104','f105','f106','f107','f108','f109');
            COMMIT;"#,
        )
        .unwrap();
        db.execute_batch("ANALYZE;").unwrap();
    }
    let s = Store::open(dir.path()).unwrap();
    (dir, s)
}

#[cfg(target_os = "linux")]
fn scoped_expected(s: &Store, roots: &[String]) -> Vec<SeedTuple> {
    let full = s.desktop_status_seeds().unwrap();
    let mut q = s
        .db
        .prepare(
            "WITH RECURSIVE down(id) AS (SELECT ?1 UNION SELECT n.id FROM down d JOIN nodes n ON n.parent=d.id), \
             up(id) AS (SELECT ?1 UNION SELECT n.parent FROM up u JOIN nodes n ON n.id=u.id WHERE n.parent IS NOT NULL), \
             scope(id) AS (SELECT id FROM down UNION SELECT id FROM up) SELECT id FROM scope",
        )
        .unwrap();
    let ids: HashSet<String> = q
        .query_map([roots[0].as_str()], |r| r.get::<_, String>(0))
        .unwrap()
        .collect::<std::result::Result<_, _>>()
        .unwrap();
    let mut rows: Vec<SeedTuple> = full
        .iter()
        .filter(|seed| ids.contains(&seed.id))
        .map(tuple)
        .collect();
    rows.sort();
    rows
}

#[cfg(target_os = "linux")]
#[test]
#[ignore = "read-volume evidence; run explicitly on the remote Linux builder"]
fn scoped_seed_read_volume_is_bounded() {
    let (_dir, s) = perf_fixture();
    let roots = vec!["target".to_owned()];

    let rchar_before = io_rchar();
    let scoped_started = std::time::Instant::now();
    let scoped = s.desktop_status_seeds_for(&roots).unwrap();
    let scoped_elapsed = scoped_started.elapsed();
    let rchar_after_scoped = io_rchar();
    let scoped_read = rchar_after_scoped - rchar_before;

    let full_started = std::time::Instant::now();
    let full = s.desktop_status_seeds().unwrap();
    let full_elapsed = full_started.elapsed();
    let full_read = io_rchar() - rchar_after_scoped;

    assert_eq!(tuples(&scoped), scoped_expected(&s, &roots));
    assert!(full.len() >= 200_000);
    assert!(scoped_read.saturating_mul(10) < full_read);

    println!(
        "SCOPED(target+ancestors): rows={} elapsed={scoped_elapsed:?} rchar={scoped_read} bytes",
        scoped.len()
    );
    println!(
        "FULL: rows={} elapsed={full_elapsed:?} rchar={full_read} bytes",
        full.len()
    );

    let mut q = s
        .db
        .prepare(&format!(
            "EXPLAIN QUERY PLAN SELECT {SEED_COLUMNS} FROM nodes n WHERE {SEED_FILTERS}"
        ))
        .unwrap();
    let plan: Vec<String> = q
        .query_map([], |r| r.get::<_, String>(3))
        .unwrap()
        .collect::<std::result::Result<_, _>>()
        .unwrap();
    println!("FULL PLAN:");
    for line in plan {
        println!("  {line}");
    }

    let mut q = s
        .db
        .prepare(&format!("EXPLAIN QUERY PLAN {}", scoped_seed_sql()))
        .unwrap();
    let plan: Vec<String> = q
        .query_map([serde_json::to_string(&roots).unwrap()], |r| {
            r.get::<_, String>(3)
        })
        .unwrap()
        .collect::<std::result::Result<_, _>>()
        .unwrap();
    println!("SCOPED PLAN:");
    for line in plan {
        println!("  {line}");
    }
}

/// Single-root namespace slightly above the reported production size (~294k
/// nodes): the shape where walking the whole tree by index lookups loses to the
/// linear full scan.
#[cfg(target_os = "linux")]
fn whole_tree_fixture() -> (tempfile::TempDir, Store) {
    let dir = tempfile::tempdir().unwrap();
    drop(Store::open(dir.path()).unwrap());
    {
        let db = rusqlite::Connection::open(dir.path().join("state.sqlite")).unwrap();
        db.execute_batch(
            r#"BEGIN;
            INSERT INTO nodes(id,parent,name,remote_version,directory) VALUES('root',NULL,'root','v',1);
            WITH RECURSIVE i(x) AS (SELECT 0 UNION ALL SELECT x+1 FROM i WHERE x<99)
              INSERT INTO nodes(id,parent,name,remote_version,directory)
                SELECT 'd'||x,'root','d'||x,'v',1 FROM i;
            WITH RECURSIVE k(y) AS (SELECT 0 UNION ALL SELECT y+1 FROM k WHERE y<293899)
              INSERT INTO nodes(id,parent,name,remote_version,directory,local_version,blob)
                SELECT 'f'||y,'d'||(y/2939),'f'||y,'v',0,
                       CASE WHEN (y%2939)<4 THEN 'v' END,
                       CASE WHEN (y%2939)<4 THEN 'b'||y END FROM k;
            UPDATE nodes SET pinned=1 WHERE id='d0';
            INSERT INTO namespace_overlay(node,remote) VALUES('d1','{}');
            INSERT INTO access_denied(node) VALUES('d2');
            INSERT INTO drafts(node,token,base,generation,checkpoint,dirty) VALUES('f0','tok','v',1,0,1);
            INSERT INTO transfers(id,node,version,direction,state) VALUES('tr','f4','v','download','pending');
            COMMIT;"#,
        )
        .unwrap();
        db.execute_batch("ANALYZE;").unwrap();
    }
    let s = Store::open(dir.path()).unwrap();
    (dir, s)
}

/// A namespace root in `roots` must fall back to the full scan: the scoped walk
/// visits the whole tree one index lookup at a time and loses badly on 294k
/// nodes, while the row set is the same either way.
#[cfg(target_os = "linux")]
#[test]
#[ignore = "read-volume evidence; run explicitly on the remote Linux builder"]
fn namespace_root_scoped_seed_falls_back_to_the_full_scan() {
    let (_dir, s) = whole_tree_fixture();
    let roots = vec!["root".to_owned()];
    let roots_json = serde_json::to_string(&roots).unwrap();

    // The scoped walk alone, i.e. what the fallback avoids on this shape.
    let before = io_rchar();
    let direct_started = std::time::Instant::now();
    let mut q = s.db.prepare(&scoped_seed_sql()).unwrap();
    let direct_rows = q
        .query_map([&roots_json], |_| Ok(()))
        .unwrap()
        .count();
    let direct_elapsed = direct_started.elapsed();
    drop(q);
    let direct_read = io_rchar() - before;

    let before_full = io_rchar();
    let full_started = std::time::Instant::now();
    let full = s.desktop_status_seeds().unwrap();
    let full_elapsed = full_started.elapsed();
    let full_read = io_rchar() - before_full;

    let before_fallback = io_rchar();
    let fallback_started = std::time::Instant::now();
    let fallback = s.desktop_status_seeds_for(&roots).unwrap();
    let fallback_elapsed = fallback_started.elapsed();
    let fallback_read = io_rchar() - before_fallback;

    assert_eq!(tuples(&fallback), tuples(&full));
    // The fixture has 294,001 raw nodes; the access-denied subtree hides rows
    // from both queries, so compare against the visible count.
    assert!(full.len() >= 290_000, "fixture too small: {}", full.len());
    assert!(direct_rows >= 290_000, "scoped root walk missed nodes: {direct_rows}");
    assert!(
        fallback_read <= full_read + 1_048_576,
        "fallback read {fallback_read} bytes vs full {full_read} bytes"
    );
    assert!(
        fallback_read < direct_read,
        "fallback read {fallback_read} bytes vs direct scoped walk {direct_read} bytes"
    );

    println!(
        "ROOT-SCOPED(direct, pre-fallback): rows={direct_rows} elapsed={direct_elapsed:?} rchar={direct_read} bytes"
    );
    println!(
        "FULL: rows={} elapsed={full_elapsed:?} rchar={full_read} bytes",
        full.len()
    );
    println!(
        "ROOT-SCOPED(fallback): rows={} elapsed={fallback_elapsed:?} rchar={fallback_read} bytes",
        fallback.len()
    );
}
