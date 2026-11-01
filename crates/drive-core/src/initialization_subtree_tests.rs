//! Exercise the production recursive statements, including their VM work.
use super::*;
use rusqlite::StatementStatus;

fn fixture(runs: &[&str]) -> Connection {
    let db = Connection::open_in_memory().unwrap();
    db.pragma_update(None, "foreign_keys", true).unwrap();
    // Production permits one active candidate. This SQL-boundary fixture permits
    // two runs with colliding IDs so a missing run predicate is observable. All
    // entries/coverage tables, indexes and FK cascades come from production DDL.
    db.execute_batch("CREATE TABLE initialization_run(run_id TEXT PRIMARY KEY,node_count INTEGER NOT NULL DEFAULT 0,node_bytes INTEGER NOT NULL DEFAULT 0,fenced TEXT,fence_ticket INTEGER);").unwrap();
    migrate(&db).unwrap();
    for run in runs {
        db.execute(
            "INSERT INTO initialization_run(run_id,fenced,fence_ticket) VALUES(?1,'native-v1:9',3)",
            [run],
        )
        .unwrap();
    }
    db
}
fn add(db: &Connection, run: &str, id: &str, parent: Option<&str>, directory: bool) {
    insert_node(
        db,
        run,
        &Node {
            id: id.into(),
            parent: parent.map(str::to_owned),
            name: id.into(),
            directory,
            version: "v1".into(),
            metadata: None,
        },
    )
    .unwrap();
}
fn seal(db: &Connection) {
    db.execute("UPDATE initialization_directories SET state='received',receipt_seq=7,receipt_hash='receipt'",[]).unwrap();
}
fn ids(db: &Connection, run: &str) -> Vec<String> {
    db.prepare("SELECT id FROM initialization_entries WHERE run=?1 ORDER BY id")
        .unwrap()
        .query_map([run], |r| r.get(0))
        .unwrap()
        .collect::<std::result::Result<_, _>>()
        .unwrap()
}
fn pending(db: &Connection, run: &str) -> Vec<String> {
    db.prepare(
        "SELECT id FROM initialization_directories WHERE run=?1 AND state='pending' ORDER BY id",
    )
    .unwrap()
    .query_map([run], |r| r.get(0))
    .unwrap()
    .collect::<std::result::Result<_, _>>()
    .unwrap()
}
fn counters_match(db: &Connection, run: &str) {
    let actual: (i64, i64) = db
        .query_row(
            "SELECT count(*),coalesce(sum(bytes),0) FROM initialization_entries WHERE run=?1",
            [run],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    let recorded: (i64, i64) = db
        .query_row(
            "SELECT node_count,node_bytes FROM initialization_run WHERE run_id=?1",
            [run],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert_eq!(recorded, actual);
}
fn branches(db: &Connection, run: &str) {
    for (id, parent, directory) in [
        ("root", None, true),
        ("a", Some("root"), true),
        ("b", Some("a"), true),
        ("c", Some("a"), true),
        ("deep", Some("b"), true),
        ("leaf", Some("deep"), false),
        ("other", Some("root"), true),
        ("other-leaf", Some("other"), false),
    ] {
        add(db, run, id, parent, directory);
    }
}

#[test]
fn recursive_invalidation_visits_only_selected_branch_and_run() {
    let mut db = fixture(&["selected", "foreign"]);
    branches(&db, "selected");
    branches(&db, "foreign");
    add(&db, "foreign", "foreign-only", Some("a"), true);
    seal(&db);
    let tx = db.transaction().unwrap();
    invalidate(&tx, "selected", "a", true).unwrap();
    tx.commit().unwrap();
    assert_eq!(pending(&db, "selected"), vec!["a", "b", "c", "deep"]);
    assert!(pending(&db, "foreign").is_empty());
    let affected:i64=db.query_row("SELECT count(*) FROM initialization_directories WHERE run='selected' AND state='pending' AND receipt_seq IS NULL AND receipt_hash IS NULL",[],|r|r.get(0)).unwrap();
    assert_eq!(affected, 4);
    let unchanged:i64=db.query_row("SELECT count(*) FROM initialization_directories WHERE state='received' AND (receipt_seq!=7 OR receipt_hash!='receipt')",[],|r|r.get(0)).unwrap();
    assert_eq!(unchanged, 0);
    let fences: Vec<(String, Option<String>, Option<i64>)> = db
        .prepare("SELECT run_id,fenced,fence_ticket FROM initialization_run ORDER BY run_id")
        .unwrap()
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
        .unwrap()
        .collect::<std::result::Result<_, _>>()
        .unwrap();
    assert_eq!(
        fences,
        vec![
            ("foreign".into(), Some("native-v1:9".into()), Some(3)),
            ("selected".into(), None, None)
        ]
    );
    counters_match(&db, "selected");
    counters_match(&db, "foreign");
}

#[test]
fn subtree_delete_preserves_siblings_other_run_counters_and_coverage_cascades() {
    let mut db = fixture(&["selected", "foreign"]);
    branches(&db, "selected");
    branches(&db, "foreign");
    seal(&db);
    let foreign = ids(&db, "foreign");
    let tx = db.transaction().unwrap();
    delete_subtree(&tx, "selected", "a").unwrap();
    tx.commit().unwrap();
    assert_eq!(ids(&db, "selected"), vec!["other", "other-leaf", "root"]);
    assert_eq!(ids(&db, "foreign"), foreign);
    let coverage: i64 = db
        .query_row(
            "SELECT count(*) FROM initialization_directories WHERE run='selected'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(coverage, 2);
    counters_match(&db, "selected");
    counters_match(&db, "foreign");
    let tx = db.transaction().unwrap();
    delete_subtree(&tx, "selected", "absent").unwrap();
    tx.commit().unwrap();
    counters_match(&db, "selected");
    // Deleting the root is still complete and restricted to the selected run.
    let tx = db.transaction().unwrap();
    delete_subtree(&tx, "selected", "root").unwrap();
    tx.commit().unwrap();
    assert!(ids(&db, "selected").is_empty());
    assert_eq!(ids(&db, "foreign"), foreign);
    counters_match(&db, "selected");
}

#[test]
fn recursive_union_still_terminates_a_cycle_and_delete_transaction_rolls_back() {
    let mut db = fixture(&["selected"]);
    add(&db, "selected", "a", Some("b"), true);
    add(&db, "selected", "b", Some("a"), true);
    add(&db, "selected", "leaf", Some("b"), false);
    seal(&db);
    let tx = db.transaction().unwrap();
    invalidate(&tx, "selected", "a", true).unwrap();
    tx.commit().unwrap();
    assert_eq!(pending(&db, "selected"), vec!["a", "b"]);
    let before = ids(&db, "selected");
    {
        let tx = db.transaction().unwrap();
        delete_subtree(&tx, "selected", "a").unwrap();
        assert!(ids(&tx, "selected").is_empty()); /* rollback */
    }
    assert_eq!(ids(&db, "selected"), before);
    counters_match(&db, "selected");
    let tx = db.transaction().unwrap();
    delete_subtree(&tx, "selected", "a").unwrap();
    tx.commit().unwrap();
    assert!(ids(&db, "selected").is_empty());
    counters_match(&db, "selected");
}

fn measured(size: usize) -> [u64; 3] {
    let mut db = fixture(&["selected"]);
    {
        let tx = db.transaction().unwrap();
        for index in 0..size {
            let id = format!("n{index:06}");
            let parent = (index > 0).then(|| format!("n{:06}", (index - 1) / 4));
            add(&tx, "selected", &id, parent.as_deref(), true);
        }
        seal(&tx);
        tx.commit().unwrap();
    }
    let mut count = db.prepare(SUBTREE_TOTALS_SQL).unwrap();
    let (nodes, bytes): (i64, i64) = count
        .query_row(params!["selected", "n000000"], |r| {
            Ok((r.get(0)?, r.get(1)?))
        })
        .unwrap();
    assert_eq!(nodes, size as i64);
    assert!(bytes > 0);
    let count_steps = u64::try_from(count.get_status(StatementStatus::VmStep)).unwrap();
    drop(count);
    let mut invalidate = db.prepare(INVALIDATE_SUBTREE_SQL).unwrap();
    assert_eq!(
        invalidate.execute(params!["selected", "n000000"]).unwrap(),
        size
    );
    let invalidate_steps = u64::try_from(invalidate.get_status(StatementStatus::VmStep)).unwrap();
    drop(invalidate);
    let mut delete = db.prepare(DELETE_SUBTREE_SQL).unwrap();
    assert_eq!(
        delete.execute(params!["selected", "n000000"]).unwrap(),
        size
    );
    let delete_steps = u64::try_from(delete.get_status(StatementStatus::VmStep)).unwrap();
    drop(delete);
    assert!(ids(&db, "selected").is_empty());
    [count_steps, invalidate_steps, delete_steps]
}

#[test]
fn all_three_production_statements_scale_with_subtree_not_its_square() {
    let small = measured(256);
    let large = measured(1024);
    for ((name, small), large) in ["totals", "invalidate", "delete"]
        .into_iter()
        .zip(small)
        .zip(large)
    {
        assert!(small > 0);
        // Four times as many nodes permits six times the VM work (B-tree and
        // FK overhead), but rejects the ~16x recursive run-scan regression.
        assert!(
            large <= small * 6 + 4096,
            "{name}: 256={small}, 1024={large} VM steps"
        );
        assert!(
            large < 1024 * 1200,
            "{name}: unexpectedly high per-node work: {large}"
        );
    }
}

#[test]
fn production_recursive_plan_uses_both_run_and_parent_index_constraints() {
    let db = fixture(&["selected"]);
    branches(&db, "selected");
    for sql in [
        SUBTREE_TOTALS_SQL,
        DELETE_SUBTREE_SQL,
        INVALIDATE_SUBTREE_SQL,
    ] {
        let plan = db
            .prepare(&format!("EXPLAIN QUERY PLAN {sql}"))
            .unwrap()
            .query_map(params!["selected", "root"], |r| r.get::<_, String>(3))
            .unwrap()
            .collect::<std::result::Result<Vec<_>, _>>()
            .unwrap();
        assert!(
            plan.iter()
                .any(|line| line.contains("initialization_parent")
                    && line.contains("run=? AND parent=?")),
            "{plan:?}"
        );
    }
}
