//! Replay old/fixed recursive invalidation on an isolated database copy only.
//! Never connects to NAS; each update is rolled back, no identities are printed.
use rusqlite::{Connection, OpenFlags, StatementStatus, params};
use serde_json::json;
use std::{path::PathBuf, time::Instant};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = PathBuf::from(std::env::args_os().nth(1).ok_or("copy path required")?);
    if path.file_name().and_then(|s| s.to_str()) != Some("replay-copy.sqlite") {
        return Err("explicit isolated replay-copy.sqlite required".into());
    }
    let db = Connection::open_with_flags(&path, OpenFlags::SQLITE_OPEN_READ_WRITE)?;
    db.pragma_update(None, "journal_mode", "WAL")?;
    db.pragma_update(None, "synchronous", "FULL")?;
    db.pragma_update(None, "foreign_keys", "ON")?;
    db.pragma_update(None, "recursive_triggers", "ON")?;
    let (run, binding, count): (String, String, i64) = db.query_row(
        "SELECT run_id,binding,node_count FROM initialization_run WHERE slot=1",
        [],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
    )?;
    let binding: serde_json::Value = serde_json::from_str(&binding)?;
    let root = binding["scope"]["file_id"].as_str().ok_or("root absent")?;
    let old = "WITH RECURSIVE tree(id) AS(SELECT ?2 UNION SELECT n.id FROM initialization_entries n JOIN tree t ON n.parent=t.id WHERE n.run=?1) UPDATE initialization_directories SET state='pending',receipt_seq=NULL,receipt_hash=NULL WHERE run=?1 AND id IN tree";
    let fixed = "WITH RECURSIVE tree(id) AS(SELECT ?2 UNION SELECT n.id FROM tree t CROSS JOIN initialization_entries n INDEXED BY initialization_parent WHERE n.run=?1 AND n.parent=t.id) UPDATE initialization_directories SET state='pending',receipt_seq=NULL,receipt_hash=NULL WHERE run=?1 AND id IN tree";
    for (label, sql) in [("old", old), ("fixed", fixed)] {
        let plan = db
            .prepare(&format!("EXPLAIN QUERY PLAN {sql}"))?
            .query_map(params![run, root], |r| r.get::<_, String>(3))?
            .collect::<Result<Vec<_>, _>>()?;
        println!(
            "{}",
            json!({"stage":"begin","label":label,"sqlite":rusqlite::version(),"nodes":count,"plan":plan})
        );
        db.execute_batch("BEGIN IMMEDIATE")?;
        let mut statement = db.prepare(sql)?;
        let begin = Instant::now();
        let rows = statement.execute(params![run, root])?;
        let elapsed = begin.elapsed().as_secs_f64();
        let steps = statement.get_status(StatementStatus::VmStep);
        drop(statement);
        db.execute_batch("ROLLBACK")?;
        println!(
            "{}",
            json!({"stage":"complete","label":label,"seconds":elapsed,"rows":rows,
            "vm_steps_i32":steps,"vm_counter_may_overflow":label=="old","rolled_back":true})
        );
    }
    Ok(())
}
