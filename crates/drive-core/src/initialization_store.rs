//! Durable, private initialization candidates. None of these tables supplies
//! filesystem metadata or authorizes remote writes before atomic final install.
use crate::{Batch, Change, Error, Node, Result, Store};
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct InitializationBinding {
    pub task_id: String,
    pub scope: serde_json::Value,
    pub restore: String,
    pub start_cursor: String,
    pub expected_cursor: String,
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct InitializationBudget {
    pub nodes: u64,
    pub node_bytes: u64,
    pub event_batches: u64,
    pub event_bytes: u64,
}
impl Default for InitializationBudget {
    fn default() -> Self {
        Self {
            nodes: 500_000,
            node_bytes: 512 * 1024 * 1024,
            event_batches: 4096,
            event_bytes: 128 * 1024 * 1024,
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct InitializationRun {
    pub run_id: String,
    pub epoch: u64,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct InitializationDirectoryTicket {
    pub run: InitializationRun,
    pub directory: String,
    pub generation: u64,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct InitializationRootTicket {
    run: InitializationRun,
    generation: u64,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct InitializationFenceTicket {
    run: InitializationRun,
    generation: u64,
    cursor: String,
}
impl InitializationFenceTicket {
    /// The exact event boundary captured with issuance, without a second Store read.
    pub fn cursor(&self) -> &str {
        &self.cursor
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct InitializationStatus {
    pub run: InitializationRun,
    pub binding: InitializationBinding,
    pub state: String,
    pub received_cursor: String,
    pub applied_cursor: String,
    pub received_sequence: u64,
    pub applied_sequence: u64,
    pub root_needs_refresh: bool,
    pub node_count: u64,
    pub node_bytes: u64,
    pub error: Option<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct InitializationCoverage {
    pub directory: String,
    pub state: String,
    pub generation: u64,
}

pub(crate) fn migrate(db: &Connection) -> Result<()> {
    db.execute_batch(
        "CREATE TABLE IF NOT EXISTS initialization_run(
        slot INTEGER PRIMARY KEY CHECK(slot=1), run_id TEXT UNIQUE NOT NULL,
        epoch INTEGER NOT NULL CHECK(epoch>0), format INTEGER NOT NULL CHECK(format=1),
        binding TEXT NOT NULL, base_restore TEXT NOT NULL, budget TEXT NOT NULL,
        state TEXT NOT NULL CHECK(state IN ('active','failed','ready')),
        received TEXT NOT NULL, applied TEXT NOT NULL,
        received_seq INTEGER NOT NULL DEFAULT 0, applied_seq INTEGER NOT NULL DEFAULT 0,
        directory_clock INTEGER NOT NULL DEFAULT 0 CHECK(directory_clock>=0),
        root_clock INTEGER NOT NULL DEFAULT 0 CHECK(root_clock>=0), root_ticket INTEGER,
        fence_clock INTEGER NOT NULL DEFAULT 0 CHECK(fence_clock>=0), fence_ticket INTEGER,
        root_refresh INTEGER NOT NULL DEFAULT 0, fenced TEXT, error TEXT,
        node_count INTEGER NOT NULL DEFAULT 0, node_bytes INTEGER NOT NULL DEFAULT 0,
        event_bytes INTEGER NOT NULL DEFAULT 0);
      CREATE TABLE IF NOT EXISTS initialization_entries(
        run TEXT NOT NULL REFERENCES initialization_run(run_id) ON DELETE CASCADE,
        id TEXT NOT NULL, parent TEXT, name TEXT NOT NULL, directory INTEGER NOT NULL,
        body TEXT NOT NULL, bytes INTEGER NOT NULL, PRIMARY KEY(run,id), UNIQUE(run,parent,name));
      CREATE INDEX IF NOT EXISTS initialization_parent ON initialization_entries(run,parent);
      CREATE TABLE IF NOT EXISTS initialization_directories(
        run TEXT NOT NULL, id TEXT NOT NULL, generation INTEGER NOT NULL DEFAULT 0,
        state TEXT NOT NULL CHECK(state IN ('pending','loading','received','denied','failed')),
        receipt_seq INTEGER, receipt_hash TEXT, PRIMARY KEY(run,id),
        FOREIGN KEY(run,id) REFERENCES initialization_entries(run,id) ON DELETE CASCADE);
      CREATE TABLE IF NOT EXISTS initialization_events(
        run TEXT NOT NULL REFERENCES initialization_run(run_id) ON DELETE CASCADE,
        seq INTEGER NOT NULL, from_cursor TEXT NOT NULL, to_cursor TEXT NOT NULL,
        body TEXT NOT NULL, digest TEXT NOT NULL, bytes INTEGER NOT NULL,
        PRIMARY KEY(run,seq));",
    )?;
    Ok(())
}

/// A persisted fence is not a new-session proof. Invalidate prior process
/// messages before exposing the Store, preserving every completed listing and
/// event receipt. Failed runs stay failed/blocked until explicit native resume.
pub(crate) fn recover_open(db: &Connection) -> Result<()> {
    let tx = db.unchecked_transaction()?;
    let old: Option<u64> = tx
        .query_row(
            "SELECT epoch FROM initialization_run WHERE state!='ready'",
            [],
            |r| read_counter(r, 0),
        )
        .optional()?;
    if let Some(epoch) = old {
        let next = epoch
            .checked_add(1)
            .ok_or_else(|| invalid("epoch exhausted on reopen"))?;
        tx.execute(
            "UPDATE initialization_run SET epoch=?1,root_ticket=NULL,fenced=NULL,fence_ticket=NULL WHERE slot=1",
            [number(next)?],
        )?;
        tx.execute("UPDATE initialization_directories SET state='pending',receipt_seq=NULL,receipt_hash=NULL WHERE state IN ('loading','failed')",[])?;
    }
    tx.commit()?;
    Ok(())
}

fn invalid(message: &str) -> Error {
    Error::State(format!("initialization: {message}"))
}
fn number(value: u64) -> Result<i64> {
    i64::try_from(value).map_err(|_| invalid("counter overflow"))
}
fn read_counter(row: &rusqlite::Row<'_>, index: usize) -> rusqlite::Result<u64> {
    let value: i64 = row.get(index)?;
    u64::try_from(value).map_err(|_| rusqlite::Error::IntegralValueOutOfRange(index, value))
}
fn read_optional_counter(row: &rusqlite::Row<'_>, index: usize) -> rusqlite::Result<Option<u64>> {
    let value: Option<i64> = row.get(index)?;
    value
        .map(|value| {
            u64::try_from(value).map_err(|_| rusqlite::Error::IntegralValueOutOfRange(index, value))
        })
        .transpose()
}
fn cursor(value: &str) -> Result<u64> {
    let raw = value
        .strip_prefix("native-v1:")
        .ok_or_else(|| invalid("invalid native cursor"))?;
    let result = raw
        .parse::<u64>()
        .map_err(|_| invalid("invalid native cursor"))?;
    if result.to_string() != raw {
        return Err(invalid("noncanonical native cursor"));
    }
    Ok(result)
}
fn root_id(binding: &InitializationBinding) -> Result<&str> {
    binding
        .scope
        .get("file_id")
        .and_then(|v| v.as_str())
        .filter(|v| !v.is_empty())
        .ok_or_else(|| invalid("missing root identity"))
}
fn validate_budget(budget: InitializationBudget) -> Result<()> {
    let max = InitializationBudget::default();
    if budget.nodes == 0
        || budget.nodes > max.nodes
        || budget.node_bytes == 0
        || budget.node_bytes > max.node_bytes
        || budget.event_batches == 0
        || budget.event_batches > max.event_batches
        || budget.event_bytes == 0
        || budget.event_bytes > max.event_bytes
    {
        return Err(invalid("invalid budget"));
    }
    Ok(())
}
fn validate_binding(db: &Connection, binding: &InitializationBinding) -> Result<()> {
    validate_binding_against(db, binding, &binding.restore)
}
fn validate_binding_against(
    db: &Connection,
    binding: &InitializationBinding,
    base_restore: &str,
) -> Result<()> {
    if binding.task_id.is_empty()
        || binding.task_id.len() > 256
        || binding.restore.is_empty()
        || binding.restore.len() > 4096
        || serde_json::to_vec(&binding.scope)?.len() > 16384
    {
        return Err(invalid("invalid binding"));
    }
    root_id(binding)?;
    cursor(&binding.start_cursor)?;
    if !binding.expected_cursor.is_empty() {
        cursor(&binding.expected_cursor)?;
    }
    let saved: String = db.query_row(
        "SELECT value FROM metadata WHERE key='native_scope'",
        [],
        |r| r.get(0),
    )?;
    let restore: String = db.query_row(
        "SELECT value FROM metadata WHERE key='server_restore_id'",
        [],
        |r| r.get(0),
    )?;
    let committed: String = db.query_row(
        "SELECT value FROM metadata WHERE key='committed_cursor'",
        [],
        |r| r.get(0),
    )?;
    if serde_json::from_str::<serde_json::Value>(&saved)? != binding.scope
        || restore != base_restore
        || committed != binding.expected_cursor
    {
        return Err(invalid("scope, server or committed cursor changed"));
    }
    Ok(())
}
fn validate_root(binding: &InitializationBinding, node: &Node) -> Result<()> {
    crate::state::valid_node(node)?;
    let path = binding
        .scope
        .get("path")
        .and_then(|p| p.as_str())
        .ok_or_else(|| invalid("missing root path"))?;
    if node.id != root_id(binding)?
        || node.parent.is_some()
        || !node.directory
        || (path != "/" && path.rsplit('/').next() != Some(node.name.as_str()))
    {
        return Err(invalid("root identity mismatch"));
    }
    Ok(())
}
fn status(db: &Connection) -> Result<Option<InitializationStatus>> {
    let row=db.query_row("SELECT run_id,epoch,binding,state,received,applied,received_seq,applied_seq,root_refresh,node_count,node_bytes,error FROM initialization_run WHERE slot=1",[],|r|{
        Ok((r.get::<_,String>(0)?,read_counter(r,1)?,r.get::<_,String>(2)?,r.get::<_,String>(3)?,r.get::<_,String>(4)?,r.get::<_,String>(5)?,read_counter(r,6)?,read_counter(r,7)?,r.get::<_,bool>(8)?,read_counter(r,9)?,read_counter(r,10)?,r.get::<_,Option<String>>(11)?))
    }).optional()?;
    row.map(
        |(
            run_id,
            epoch,
            binding,
            state,
            received_cursor,
            applied_cursor,
            received_sequence,
            applied_sequence,
            root_needs_refresh,
            node_count,
            node_bytes,
            error,
        )| {
            Ok(InitializationStatus {
                run: InitializationRun { run_id, epoch },
                binding: serde_json::from_str(&binding)?,
                state,
                received_cursor,
                applied_cursor,
                received_sequence,
                applied_sequence,
                root_needs_refresh,
                node_count,
                node_bytes,
                error,
            })
        },
    )
    .transpose()
}
fn checked(
    db: &Connection,
    run: &InitializationRun,
    allow_failed: bool,
) -> Result<InitializationStatus> {
    let s = status(db)?.ok_or_else(|| invalid("run absent"))?;
    if s.run != *run || s.state == "ready" || (!allow_failed && s.state != "active") {
        return Err(invalid("stale or inactive run"));
    }
    let base_restore: String = db.query_row(
        "SELECT base_restore FROM initialization_run WHERE slot=1",
        [],
        |r| r.get(0),
    )?;
    validate_binding_against(db, &s.binding, &base_restore)?;
    Ok(s)
}
fn budget(db: &Connection, run: &str) -> Result<InitializationBudget> {
    let body: String = db.query_row(
        "SELECT budget FROM initialization_run WHERE run_id=?1",
        [run],
        |r| r.get(0),
    )?;
    let b = serde_json::from_str(&body)?;
    validate_budget(b)?;
    Ok(b)
}
fn clear_fence(db: &Connection, run: &str) -> Result<()> {
    db.execute(
        "UPDATE initialization_run SET fenced=NULL,fence_ticket=NULL WHERE run_id=?1",
        [run],
    )?;
    Ok(())
}
fn insert_node(db: &Connection, run: &str, node: &Node) -> Result<()> {
    crate::state::valid_node(node)?;
    let body = serde_json::to_string(node)?;
    if body.len() > 1_048_576 || node.name.len() > 255 {
        return Err(invalid("node budget exceeded"));
    }
    db.execute("INSERT INTO initialization_entries(run,id,parent,name,directory,body,bytes) VALUES(?1,?2,?3,?4,?5,?6,?7)",params![run,node.id,node.parent,node.name,node.directory,body,body.len() as i64])?;
    db.execute("UPDATE initialization_run SET node_count=node_count+1,node_bytes=node_bytes+?2 WHERE run_id=?1",params![run,body.len() as i64])?;
    if node.directory {
        db.execute(
            "INSERT INTO initialization_directories(run,id,state) VALUES(?1,?2,'pending')",
            params![run, node.id],
        )?;
    }
    Ok(())
}
fn delete_subtree(db: &Connection, run: &str, id: &str) -> Result<()> {
    let (count,bytes):(i64,i64)=db.query_row("WITH RECURSIVE tree(id) AS(SELECT ?2 UNION SELECT n.id FROM initialization_entries n JOIN tree t ON n.parent=t.id WHERE n.run=?1) SELECT count(*),coalesce(sum(bytes),0) FROM initialization_entries WHERE run=?1 AND id IN tree",params![run,id],|r|Ok((r.get(0)?,r.get(1)?)))?;
    db.execute("WITH RECURSIVE tree(id) AS(SELECT ?2 UNION SELECT n.id FROM initialization_entries n JOIN tree t ON n.parent=t.id WHERE n.run=?1) DELETE FROM initialization_entries WHERE run=?1 AND id IN tree",params![run,id])?;
    db.execute("UPDATE initialization_run SET node_count=node_count-?2,node_bytes=node_bytes-?3 WHERE run_id=?1",params![run,count,bytes])?;
    Ok(())
}
fn check_node_budget(db: &Connection, run: &str) -> Result<()> {
    let b = budget(db, run)?;
    let (count, bytes): (u64, u64) = db.query_row(
        "SELECT node_count,node_bytes FROM initialization_run WHERE run_id=?1",
        [run],
        |r| Ok((read_counter(r, 0)?, read_counter(r, 1)?)),
    )?;
    if count > b.nodes || bytes > b.node_bytes {
        return Err(invalid("candidate budget exceeded"));
    }
    Ok(())
}
fn load_node(db: &Connection, run: &str, id: &str) -> Result<Node> {
    let body: String = db.query_row(
        "SELECT body FROM initialization_entries WHERE run=?1 AND id=?2",
        params![run, id],
        |r| r.get(0),
    )?;
    Ok(serde_json::from_str(&body)?)
}
fn invalidate(db: &Connection, run: &str, id: &str, recursive: bool) -> Result<()> {
    let changed = if recursive {
        db.execute("WITH RECURSIVE tree(id) AS(SELECT ?2 UNION SELECT n.id FROM initialization_entries n JOIN tree t ON n.parent=t.id WHERE n.run=?1) UPDATE initialization_directories SET state='pending',receipt_seq=NULL,receipt_hash=NULL WHERE run=?1 AND id IN tree",params![run,id])?
    } else {
        db.execute("UPDATE initialization_directories SET state='pending',receipt_seq=NULL,receipt_hash=NULL WHERE run=?1 AND id=?2",params![run,id])?
    };
    if changed == 0 {
        return Err(invalid("directory coverage absent"));
    }
    clear_fence(db, run)
}

impl Store {
    pub fn initialization_writeback_blocked(&self) -> Result<bool> {
        Ok(self.db.query_row(
            "SELECT EXISTS(SELECT 1 FROM initialization_run WHERE state!='ready')",
            [],
            |r| r.get(0),
        )?)
    }
    pub fn initialization_status(&self) -> Result<Option<InitializationStatus>> {
        status(&self.db)
    }
    pub fn begin_initialization(
        &mut self,
        binding: &InitializationBinding,
        root: &Node,
        b: InitializationBudget,
    ) -> Result<InitializationRun> {
        validate_budget(b)?;
        validate_root(binding, root)?;
        let tx = self.db.transaction()?;
        validate_binding(&tx, binding)?;
        if tx.query_row(
            "SELECT EXISTS(SELECT 1 FROM initialization_run WHERE state!='ready')",
            [],
            |r| r.get::<_, bool>(0),
        )? {
            return Err(invalid(
                "active run must be resumed or explicitly restarted",
            ));
        }
        let old: Option<u64> = tx
            .query_row(
                "SELECT epoch FROM initialization_run WHERE slot=1",
                [],
                |r| read_counter(r, 0),
            )
            .optional()?;
        let epoch = old
            .unwrap_or(0)
            .checked_add(1)
            .ok_or_else(|| invalid("epoch exhausted"))?;
        number(epoch)?;
        let run = InitializationRun {
            run_id: uuid::Uuid::new_v4().to_string(),
            epoch,
        };
        tx.execute("DELETE FROM initialization_run", [])?;
        tx.execute("INSERT INTO initialization_run(slot,run_id,epoch,format,binding,base_restore,budget,state,received,applied) VALUES(1,?1,?2,1,?3,?6,?4,'active',?5,?5)",params![run.run_id,number(epoch)?,serde_json::to_string(binding)?,serde_json::to_string(&b)?,binding.start_cursor,binding.restore])?;
        insert_node(&tx, &run.run_id, root)?;
        check_node_budget(&tx, &run.run_id)?;
        tx.commit()?;
        Ok(run)
    }
    /// Only after reauthentication and root/scope/restore validation by the
    /// native caller. In-flight listings are restarted, never resumed by offset.
    pub fn resume_initialization(
        &mut self,
        run: &InitializationRun,
        binding: &InitializationBinding,
    ) -> Result<InitializationRun> {
        let tx = self.db.transaction()?;
        let s = checked(&tx, run, true)?;
        if &s.binding != binding {
            return Err(invalid("resume binding mismatch"));
        }
        let epoch = run
            .epoch
            .checked_add(1)
            .ok_or_else(|| invalid("epoch exhausted"))?;
        number(epoch)?;
        tx.execute("UPDATE initialization_run SET epoch=?1,state='active',root_ticket=NULL,fenced=NULL,fence_ticket=NULL,error=NULL WHERE slot=1",[number(epoch)?])?;
        tx.execute("UPDATE initialization_directories SET state='pending',receipt_seq=NULL,receipt_hash=NULL WHERE run=?1 AND state IN ('loading','failed')",[&run.run_id])?;
        tx.commit()?;
        Ok(InitializationRun {
            run_id: run.run_id.clone(),
            epoch,
        })
    }
    /// Discard only a failed/unusable private candidate after the native owner
    /// revalidates the same selected root. The gate never opens between runs;
    /// even a server-restore change leaves old authoritative data untouched.
    pub fn restart_initialization(
        &mut self,
        run: &InitializationRun,
        binding: &InitializationBinding,
        root: &Node,
        b: InitializationBudget,
    ) -> Result<InitializationRun> {
        validate_budget(b)?;
        validate_root(binding, root)?;
        let tx = self.db.transaction()?;
        let old = checked(&tx, run, true)?;
        if old.binding.task_id != binding.task_id
            || old.binding.scope != binding.scope
            || old.binding.expected_cursor != binding.expected_cursor
        {
            return Err(invalid(
                "restart cannot change task, scope or authoritative base",
            ));
        }
        let base: String = tx.query_row(
            "SELECT base_restore FROM initialization_run WHERE slot=1",
            [],
            |r| r.get(0),
        )?;
        validate_binding_against(&tx, binding, &base)?;
        let epoch = run
            .epoch
            .checked_add(1)
            .ok_or_else(|| invalid("epoch exhausted"))?;
        number(epoch)?;
        let next = InitializationRun {
            run_id: uuid::Uuid::new_v4().to_string(),
            epoch,
        };
        tx.execute("DELETE FROM initialization_run", [])?;
        tx.execute("INSERT INTO initialization_run(slot,run_id,epoch,format,binding,base_restore,budget,state,received,applied) VALUES(1,?1,?2,1,?3,?4,?5,'active',?6,?6)",params![next.run_id,number(epoch)?,serde_json::to_string(binding)?,base,serde_json::to_string(&b)?,binding.start_cursor])?;
        insert_node(&tx, &next.run_id, root)?;
        check_node_budget(&tx, &next.run_id)?;
        tx.commit()?;
        Ok(next)
    }
    pub fn fail_initialization(&mut self, run: &InitializationRun, code: &str) -> Result<()> {
        if code.is_empty()
            || code.len() > 96
            || !code
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
        {
            return Err(invalid("invalid error category"));
        }
        let tx = self.db.transaction()?;
        checked(&tx, run, true)?;
        tx.execute(
            "UPDATE initialization_run SET state='failed',fenced=NULL,fence_ticket=NULL,error=?1 WHERE slot=1",
            [code],
        )?;
        tx.commit()?;
        Ok(())
    }
    pub fn initialization_coverage(
        &self,
        run: &InitializationRun,
    ) -> Result<Vec<InitializationCoverage>> {
        checked(&self.db, run, true)?;
        Ok(self.db.prepare("SELECT id,state,generation FROM initialization_directories WHERE run=?1 ORDER BY id")?.query_map([&run.run_id],|r|Ok(InitializationCoverage{directory:r.get(0)?,state:r.get(1)?,generation:read_counter(r,2)?}))?.collect::<std::result::Result<Vec<_>,_>>()?)
    }
    pub fn initialization_pending_directories(
        &self,
        run: &InitializationRun,
        limit: usize,
    ) -> Result<Vec<Node>> {
        checked(&self.db, run, false)?;
        if limit == 0 || limit > 4096 {
            return Err(invalid("invalid page limit"));
        }
        let bodies=self.db.prepare("SELECT n.body FROM initialization_entries n JOIN initialization_directories d ON d.run=n.run AND d.id=n.id WHERE n.run=?1 AND d.state IN ('pending','failed') ORDER BY n.id LIMIT ?2")?.query_map(params![run.run_id,limit as i64],|r|r.get::<_,String>(0))?.collect::<std::result::Result<Vec<_>,_>>()?;
        bodies
            .into_iter()
            .map(|b| Ok(serde_json::from_str(&b)?))
            .collect()
    }
    pub fn initialization_node(&self, run: &InitializationRun, id: &str) -> Result<Node> {
        checked(&self.db, run, true)?;
        load_node(&self.db, &run.run_id, id)
    }
}

#[path = "initialization_store_ops.rs"]
mod ops;
pub(crate) use ops::{
    InitializationCommit, check_install_gate, finish_in_transaction, installation_restore,
};
#[cfg(test)]
#[path = "initialization_store_tests.rs"]
mod tests;
