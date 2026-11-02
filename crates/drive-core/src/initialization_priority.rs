//! Optional queue hints for the current deferred ordinary-file page. Paths can
//! nominate an existing pending directory, never create or authorize a node.
use super::*;
const PAGE_BYTES: u64 = 256 * 1024;
const DEPENDENCIES: usize = 8;

#[derive(Clone, Debug, PartialEq, Eq)]
struct Head {
    run: InitializationRun,
    sequence: u64,
    batch_digest: String,
    hint_digest: Option<String>,
    bytes: u64,
}
#[derive(Clone, Debug)]
struct Dependency {
    parent: String,
    components: Vec<String>,
}
#[derive(Default)]
pub struct InitializationPriorityCache {
    head: Option<Head>,
    dependencies: Vec<Dependency>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InitializationPriorityPreparation {
    Prepared,
    Reused,
    Unavailable,
}

fn head(db: &Connection, s: &InitializationStatus) -> Result<Option<Head>> {
    if s.received_sequence == s.applied_sequence {
        return Ok(None);
    }
    let sequence = s
        .applied_sequence
        .checked_add(1)
        .ok_or_else(|| invalid("event sequence exhausted"))?;
    let row=db.query_row("SELECT e.digest,e.bytes,h.digest,h.bytes,e.from_cursor,e.to_cursor FROM initialization_events e LEFT JOIN initialization_event_hints h ON h.run=e.run AND h.seq=e.seq WHERE e.run=?1 AND e.seq=?2",
        params![s.run.run_id,number(sequence)?],|r|Ok((r.get::<_,String>(0)?,read_counter(r,1)?,r.get::<_,Option<String>>(2)?,read_optional_counter(r,3)?,r.get::<_,String>(4)?,r.get::<_,String>(5)?))).optional()?;
    let Some((batch_digest, bytes, hint_digest, hint_bytes, from, to)) = row else {
        return Err(invalid("candidate pending event gap"));
    };
    if from != s.applied_cursor || cursor(&to)? <= cursor(&from)? {
        return Err(invalid("event priority cursor mismatch"));
    }
    let bytes = bytes
        .checked_add(hint_bytes.unwrap_or(0))
        .ok_or_else(|| invalid("event priority byte overflow"))?;
    Ok(Some(Head {
        run: s.run.clone(),
        sequence,
        batch_digest,
        hint_digest,
        bytes,
    }))
}

fn suggested(
    db: &Connection,
    s: &InitializationStatus,
    dependency: &Dependency,
) -> Result<Option<(Node, String)>> {
    if s.root_needs_refresh {
        return Ok(None);
    }
    // Once the metadata parent exists, event application, not this accelerator,
    // decides its next rechecks. Do not prioritize a guessed replacement ID.
    if db.query_row(
        "SELECT EXISTS(SELECT 1 FROM initialization_entries WHERE run=?1 AND id=?2)",
        params![s.run.run_id, dependency.parent],
        |r| r.get::<_, bool>(0),
    )? {
        return Ok(None);
    }
    let root = root_id(&s.binding)?;
    let mut current = load_node(db, &s.run.run_id, root)?;
    validate_root(&s.binding, &current)?;
    let mut path = s
        .binding
        .scope
        .get("path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| invalid("missing scope path"))?
        .to_owned();
    for depth in 0..=dependency.components.len() {
        crate::state::valid_node(&current)?;
        if !current.directory {
            return Ok(None);
        }
        let native = if current.id == root {
            s.binding
                .scope
                .get("node_id")
                .and_then(|v| v.as_u64())
                .filter(|id| *id != 0)
        } else {
            read_native_identity(db, &s.run.run_id, &current.id)?
        };
        if native.is_none() {
            return Ok(None);
        }
        let denied = db.query_row(
            "SELECT EXISTS(SELECT 1 FROM preview_denied WHERE run=?1 AND id=?2)",
            params![s.run.run_id, current.id],
            |r| r.get::<_, bool>(0),
        )?;
        let root_invalid=db.query_row("SELECT EXISTS(SELECT 1 FROM preview_visibility WHERE run=?1 AND id=?2 AND root_invalid!=0)",params![s.run.run_id,root],|r|r.get::<_,bool>(0))?;
        if denied || root_invalid {
            return Ok(None);
        }
        let coverage = db
            .query_row(
                "SELECT state FROM initialization_directories WHERE run=?1 AND id=?2",
                params![s.run.run_id, current.id],
                |r| r.get::<_, String>(0),
            )
            .optional()?;
        match coverage.as_deref() {
            Some("pending" | "failed") => return Ok(Some((current, path))),
            Some("received") => {}
            _ => return Ok(None),
        }
        let Some(component) = dependency.components.get(depth) else {
            return Ok(None);
        };
        let body = db
            .query_row(
                "SELECT id,body FROM initialization_entries WHERE run=?1 AND parent=?2 AND name=?3",
                params![s.run.run_id, current.id, component],
                |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)),
            )
            .optional()?;
        let Some((id, body)) = body else {
            return Ok(None);
        };
        let child: Node = serde_json::from_str(&body)?;
        if child.id != id
            || child.parent.as_deref() != Some(current.id.as_str())
            || child.name != *component
        {
            return Err(invalid("priority candidate edge mismatch"));
        }
        if path != "/" {
            path.push('/');
        }
        path.push_str(component);
        current = child;
    }
    Ok(None)
}

impl Store {
    /// Cache only a small, validated ordinary-file head. Oversized or legacy
    /// pages continue through the existing complete event-application path.
    pub fn prepare_initialization_priority(
        &self,
        run: &InitializationRun,
        cache: &mut InitializationPriorityCache,
    ) -> Result<InitializationPriorityPreparation> {
        let s = checked(&self.db, run, false)?;
        let observed = head(&self.db, &s)?;
        if observed == cache.head {
            return Ok(if cache.dependencies.is_empty() {
                InitializationPriorityPreparation::Unavailable
            } else {
                InitializationPriorityPreparation::Reused
            });
        }
        let mut dependencies = Vec::new();
        if let Some(ref h) = observed {
            if h.bytes <= PAGE_BYTES && h.hint_digest.is_some() {
                // Check actual storage size before loading body strings. This
                // is done only for a new small header, never on cached reuse or
                // an already oversized page. Normal event application remains
                // responsible for full receipt integrity on every application.
                let actual:u64=self.db.query_row("SELECT length(CAST(e.body AS BLOB))+coalesce(length(CAST(h.body AS BLOB)),0) FROM initialization_events e LEFT JOIN initialization_event_hints h ON h.run=e.run AND h.seq=e.seq WHERE e.run=?1 AND e.seq=?2",params![run.run_id,number(h.sequence)?],|r|read_counter(r,0))?;
                if actual != h.bytes {
                    return Err(invalid("event priority receipt byte mismatch"));
                }
                let (batch, evidence) = read_event_receipt(&self.db, &s, h.sequence)?;
                if batch.from != s.applied_cursor {
                    return Err(invalid("candidate applied cursor mismatch"));
                }
                let evidence = evidence.ok_or_else(|| invalid("priority hint disappeared"))?;
                let scope = s
                    .binding
                    .scope
                    .get("path")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| invalid("missing scope path"))?;
                let mut ordinary = true;
                let mut parents = HashSet::new();
                for (ordinal, event) in batch.events.iter().enumerate() {
                    let Change::Upsert(node) = &event.change else {
                        ordinary = false;
                        break;
                    };
                    if node.directory || node.id == root_id(&s.binding)? {
                        ordinary = false;
                        break;
                    }
                    match load_node(&self.db, &run.run_id, &node.id) {
                        Ok(old) if old.directory => {
                            ordinary = false;
                            break;
                        }
                        Ok(_) => {}
                        Err(e) if e.is_not_found() => {}
                        Err(e) => return Err(e),
                    }
                    let parent = node
                        .parent
                        .as_ref()
                        .ok_or_else(|| invalid("priority missing file parent"))?;
                    match load_node(&self.db, &run.run_id, parent) {
                        Ok(_) => continue,
                        Err(e) if e.is_not_found() => {}
                        Err(e) => return Err(e),
                    }
                    if dependencies.len() == DEPENDENCIES || !parents.insert(parent.clone()) {
                        continue;
                    }
                    let hint = evidence
                        .hint(ordinal)
                        .ok_or_else(|| invalid("priority missing hint"))?;
                    let target = hint.destination_path.as_deref().unwrap_or(&hint.event_path);
                    let prefix = format!("{}/", scope.trim_end_matches('/'));
                    let relative = target
                        .strip_prefix(&prefix)
                        .ok_or_else(|| invalid("priority target outside scope"))?;
                    let mut components = relative.split('/').map(str::to_owned).collect::<Vec<_>>();
                    components.pop();
                    if components.len() > 128 {
                        continue;
                    }
                    dependencies.push(Dependency {
                        parent: parent.clone(),
                        components,
                    });
                }
                if !ordinary {
                    dependencies.clear();
                }
            }
        }
        cache.head = observed;
        cache.dependencies = dependencies;
        Ok(if cache.dependencies.is_empty() {
            InitializationPriorityPreparation::Unavailable
        } else {
            InitializationPriorityPreparation::Prepared
        })
    }

    /// One optional existing directory; call at most once per ordinary window.
    /// Header identity is rechecked in the same transaction as all path/ACL
    /// lookups and issuance. No coverage is invalidated or marked complete.
    pub fn claim_initialization_priority(
        &mut self,
        run: &InitializationRun,
        cache: &InitializationPriorityCache,
    ) -> Result<Option<InitializationDirectoryClaim>> {
        let tx = self.db.transaction()?;
        let s = checked(&tx, run, false)?;
        if cache.head.is_none() || head(&tx, &s)? != cache.head {
            return Ok(None);
        }
        for dependency in &cache.dependencies {
            if let Some((node, path)) = suggested(&tx, &s, dependency)? {
                // Revalidate the same bounded authoritative candidate chain as
                // normal claim, independent of the historical path suggestion.
                let chain = directory_chain(&tx, &run.run_id, &s.binding, &node.id)?;
                if chain.first() != Some(&node) {
                    return Err(invalid("priority candidate changed"));
                }
                let ticket = ops::begin_directory_in_transaction(&tx, run, &node.id)?;
                tx.commit()?;
                return Ok(Some(InitializationDirectoryClaim { node, path, ticket }));
            }
        }
        Ok(None)
    }
}
