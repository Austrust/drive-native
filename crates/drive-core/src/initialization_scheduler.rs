//! Bounded candidate scheduling. A scan cursor is a queue hint, never a proof
//! that coverage or an event boundary is complete.
use super::*;

const SCAN_LIMIT: usize = 64;
const CLAIM_LIMIT: usize = 16;
pub(super) const FIRST_PAGE: &str = "SELECT id FROM initialization_directories INDEXED BY initialization_pending WHERE run=?1 AND state IN ('pending','failed') ORDER BY id LIMIT ?2";
pub(super) const NEXT_PAGE: &str = "SELECT id FROM initialization_directories INDEXED BY initialization_pending WHERE run=?1 AND state IN ('pending','failed') AND id>?2 ORDER BY id LIMIT ?3";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InitializationScanCursor {
    run: InitializationRun,
    after: String,
}

#[derive(Clone, Debug)]
pub struct InitializationDirectoryClaim {
    pub node: Node,
    pub path: String,
    pub ticket: InitializationDirectoryTicket,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct InitializationClaimStats {
    pub examined: usize,
    pub ancestor_promotions: usize,
    pub coalesced: usize,
    pub blocked_loading: usize,
    /// A previous claim is not necessarily duplicate work: an event, interrupted
    /// request or replacement may legitimately require another complete list.
    pub previously_claimed: usize,
}

#[derive(Clone, Debug)]
pub struct InitializationClaimPage {
    pub jobs: Vec<InitializationDirectoryClaim>,
    /// None means this indexed scan reached its end, not initialization ready.
    /// Discard the cursor after applying events or committing directory receipts;
    /// mutations can create earlier keys which a continuation would skip.
    pub next: Option<InitializationScanCursor>,
    pub stats: InitializationClaimStats,
}

fn pending_ids(db: &Connection, run: &str, after: Option<&str>) -> Result<Vec<String>> {
    let mut statement = db.prepare(if after.is_some() {
        NEXT_PAGE
    } else {
        FIRST_PAGE
    })?;
    let mut rows = if let Some(after) = after {
        statement.query(params![run, after, (SCAN_LIMIT + 1) as i64])?
    } else {
        statement.query(params![run, (SCAN_LIMIT + 1) as i64])?
    };
    let mut ids = Vec::new();
    while let Some(row) = rows.next()? {
        ids.push(row.get(0)?);
    }
    Ok(ids)
}

fn chain_path(binding: &InitializationBinding, chain: &[Node]) -> Result<String> {
    let root_path = binding
        .scope
        .get("path")
        .and_then(|p| p.as_str())
        .ok_or_else(|| invalid("missing scope path"))?;
    if !root_path.starts_with('/')
        || root_path.contains('\0')
        || (root_path != "/"
            && root_path[1..]
                .split('/')
                .any(|part| part.is_empty() || matches!(part, "." | "..")))
    {
        return Err(invalid("noncanonical scope path"));
    }
    let mut path = root_path.to_owned();
    for node in chain.iter().rev().skip(1) {
        if path != "/" {
            path.push('/');
        }
        path.push_str(&node.name);
    }
    Ok(path)
}

impl Store {
    /// Select and claim up to sixteen independent directory requests in one
    /// transaction. Only the pending partial index and bounded ancestry paths
    /// are read; no complete candidate tree or native identity map is loaded.
    /// The caller must still use the existing final coverage/fence validation.
    pub fn claim_initialization_directories(
        &mut self,
        run: &InitializationRun,
        limit: usize,
        after: Option<&InitializationScanCursor>,
    ) -> Result<InitializationClaimPage> {
        if limit == 0 || limit > CLAIM_LIMIT || after.is_some_and(|c| c.run != *run) {
            return Err(invalid("invalid initialization claim page"));
        }
        let tx = self.db.transaction()?;
        let state = checked(&tx, run, false)?;
        let candidates = pending_ids(&tx, &run.run_id, after.map(|c| c.after.as_str()))?;
        let mut stats = InitializationClaimStats::default();
        let mut jobs = Vec::new();
        let mut selected = HashSet::new();
        let mut last = None;
        let mut clock: u64 = tx.query_row(
            "SELECT directory_clock FROM initialization_run WHERE run_id=?1",
            [&run.run_id],
            |r| read_counter(r, 0),
        )?;
        for id in candidates.iter().take(SCAN_LIMIT) {
            if jobs.len() == limit {
                break;
            }
            stats.examined += 1;
            last = Some(id.clone());
            let chain = directory_chain(&tx, &run.run_id, &state.binding, id)?;
            let mut highest = None;
            let mut blocked = false;
            let mut duplicate = false;
            for (index, node) in chain.iter().enumerate() {
                let coverage: String = tx.query_row(
                    "SELECT state FROM initialization_directories WHERE run=?1 AND id=?2",
                    params![run.run_id, node.id],
                    |r| r.get(0),
                )?;
                match coverage.as_str() {
                    "pending" | "failed" => highest = Some(index),
                    "loading" => {
                        blocked = true;
                        duplicate |= selected.contains(&node.id);
                    }
                    "received" => {}
                    _ => return Err(invalid("pending directory below unavailable coverage")),
                }
            }
            if blocked {
                if duplicate {
                    stats.coalesced += 1;
                } else {
                    stats.blocked_loading += 1;
                }
                continue;
            }
            let index = highest.ok_or_else(|| invalid("pending directory state changed"))?;
            let node = chain[index].clone();
            let path = chain_path(&state.binding, &chain[index..])?;
            if index != 0 {
                stats.ancestor_promotions += 1;
            }
            let previous: u64 = tx.query_row(
                "SELECT generation FROM initialization_directories WHERE run=?1 AND id=?2",
                params![run.run_id, node.id],
                |r| read_counter(r, 0),
            )?;
            stats.previously_claimed += usize::from(previous != 0);
            clock = clock
                .checked_add(1)
                .ok_or_else(|| invalid("directory generation exhausted"))?;
            number(clock)?;
            tx.execute("UPDATE initialization_directories SET generation=?3,state='loading',receipt_seq=NULL,receipt_hash=NULL WHERE run=?1 AND id=?2",params![run.run_id,node.id,number(clock)?])?;
            selected.insert(node.id.clone());
            jobs.push(InitializationDirectoryClaim {
                ticket: InitializationDirectoryTicket {
                    run: run.clone(),
                    directory: node.id.clone(),
                    generation: clock,
                },
                node,
                path,
            });
        }
        if !jobs.is_empty() {
            tx.execute(
                "UPDATE initialization_run SET directory_clock=?2 WHERE run_id=?1",
                params![run.run_id, number(clock)?],
            )?;
            clear_fence(&tx, &run.run_id)?;
        }
        let next = if stats.examined < candidates.len() {
            last.map(|after| InitializationScanCursor {
                run: run.clone(),
                after,
            })
        } else {
            None
        };
        tx.commit()?;
        Ok(InitializationClaimPage { jobs, next, stats })
    }
}
