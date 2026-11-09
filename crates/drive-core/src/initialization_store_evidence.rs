//! Private evidence sidecars. They never alter authoritative nodes or cursors.
use super::*;

#[cfg(test)]
thread_local! {
    static PAGE_HASHES: std::cell::Cell<u64> = std::cell::Cell::new(0);
    static RECEIPT_PARSES: std::cell::Cell<u64> = std::cell::Cell::new(0);
    static RECEIPT_VALIDATIONS: std::cell::Cell<u64> = std::cell::Cell::new(0);
}
/// Hash one authoritative page body. Tests count these calls to prove that a
/// page translated and recorded in this process is not hashed again on apply.
pub(crate) fn page_hash(bytes: &[u8]) -> String {
    #[cfg(test)]
    PAGE_HASHES.with(|count| count.set(count.get() + 1));
    crate::sha256(bytes)
}
/// Parse one authoritative page body, counted by tests alongside `page_hash`.
fn page_parse<T: serde::de::DeserializeOwned>(body: &str) -> Result<T> {
    #[cfg(test)]
    RECEIPT_PARSES.with(|count| count.set(count.get() + 1));
    Ok(serde_json::from_str(body)?)
}
#[cfg(test)]
pub(crate) fn page_work() -> (u64, u64, u64) {
    (
        PAGE_HASHES.with(|count| count.get()),
        RECEIPT_PARSES.with(|count| count.get()),
        RECEIPT_VALIDATIONS.with(|count| count.get()),
    )
}
#[cfg(test)]
pub(crate) fn reset_page_work() {
    PAGE_HASHES.with(|count| count.set(0));
    RECEIPT_PARSES.with(|count| count.set(0));
    RECEIPT_VALIDATIONS.with(|count| count.set(0));
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct EventEnvelope {
    schema: u8,
    run_id: String,
    sequence: u64,
    batch_digest: String,
    scope: serde_json::Value,
    restore: String,
    hints: Vec<InitializationEventHint>,
}
impl EventEnvelope {
    pub(super) fn hint(&self, ordinal: usize) -> Option<&InitializationEventHint> {
        self.hints.get(ordinal)
    }
    pub(super) fn hints(&self) -> &[InitializationEventHint] {
        &self.hints
    }
    pub(super) fn matches_page(&self, page: &InitializationEventPage) -> bool {
        self.scope == page.scope && self.restore == page.restore && self.hints == page.hints
    }
}
fn validate_page(
    binding: &InitializationBinding,
    batch: &Batch,
    scope: &serde_json::Value,
    restore: &str,
    hints: &[InitializationEventHint],
) -> Result<()> {
    crate::initialization_events::validate_event_page_parts(batch, scope, restore, hints)?;
    if *scope != binding.scope || restore != binding.restore {
        return Err(invalid("event hint session binding mismatch"));
    }
    let root = root_id(binding)?;
    let selected = binding
        .scope
        .get("path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| invalid("event scope path missing"))?;
    let inside = |path: &str| {
        selected == "/"
            || path == selected
            || path
                .strip_prefix(selected)
                .is_some_and(|tail| tail.starts_with('/'))
    };
    for (event, hint) in batch.events.iter().zip(hints) {
        let target = hint.destination_path.as_deref().unwrap_or(&hint.event_path);
        if hint.event_path.len() > 16384
            || target.len() > 16384
            || !inside(&hint.event_path) && !inside(target)
        {
            return Err(invalid("event hint outside selected scope"));
        }
        match &event.change {
            Change::Upsert(node) => {
                crate::state::valid_node(node)?;
                if !inside(target)
                    || (node.id == root) != (target == selected)
                    || (node.id == root) != node.parent.is_none()
                {
                    return Err(invalid("event hint target identity mismatch"));
                }
            }
            Change::Delete { .. } => {
                if !hint.metadata_removed
                    && hint.kind != InitializationEventKind::Remove
                    && !(hint.kind == InitializationEventKind::Move && !inside(target))
                {
                    return Err(invalid("event hint does not explain deletion"));
                }
            }
        }
    }
    Ok(())
}

pub(super) fn encode_event_page(
    binding: &InitializationBinding,
    run: &str,
    sequence: u64,
    batch_digest: &str,
    page: &InitializationEventPage,
) -> Result<String> {
    validate_page(
        binding,
        &page.batch,
        &page.scope,
        &page.restore,
        &page.hints,
    )?;
    Ok(serde_json::to_string(&EventEnvelope {
        schema: 1,
        run_id: run.into(),
        sequence,
        batch_digest: batch_digest.into(),
        scope: page.scope.clone(),
        restore: page.restore.clone(),
        hints: page.hints.clone(),
    })?)
}

/// Check stored bytes, digest, receipt boundary and optional hint binding before
/// exposing either source. Legacy receipts remain explicitly without evidence.
pub(super) fn read_event_receipt(
    db: &Connection,
    status: &InitializationStatus,
    sequence: u64,
) -> Result<(Batch, Option<EventEnvelope>)> {
    if sequence == 0 || sequence > status.received_sequence {
        return Err(invalid("event receipt outside received boundary"));
    }
    let (body,hash,bytes,from,to):(String,String,u64,String,String)=db.query_row(
        "SELECT body,digest,bytes,from_cursor,to_cursor FROM initialization_events WHERE run=?1 AND seq=?2",
        params![status.run.run_id,number(sequence)?],|r|Ok((r.get(0)?,r.get(1)?,read_counter(r,2)?,r.get(3)?,r.get(4)?)))?;
    if bytes != body.len() as u64 || page_hash(body.as_bytes()) != hash {
        return Err(invalid("event receipt digest mismatch"));
    }
    let batch: Batch = page_parse(&body)?;
    if batch.from != from || batch.to != to || cursor(&to)? <= cursor(&from)? {
        return Err(invalid("event receipt cursor mismatch"));
    }
    let side: Option<(String, String, u64)> = db
        .query_row(
            "SELECT body,digest,bytes FROM initialization_event_hints WHERE run=?1 AND seq=?2",
            params![status.run.run_id, number(sequence)?],
            |r| Ok((r.get(0)?, r.get(1)?, read_counter(r, 2)?)),
        )
        .optional()?;
    let page = if let Some((body, digest, bytes)) = side {
        if bytes != body.len() as u64 || page_hash(body.as_bytes()) != digest {
            return Err(invalid("event hint digest mismatch"));
        }
        let envelope: EventEnvelope = page_parse(&body)?;
        if envelope.schema != 1
            || envelope.run_id != status.run.run_id
            || envelope.sequence != sequence
            || envelope.batch_digest != hash
        {
            return Err(invalid("event hint receipt binding mismatch"));
        }
        #[cfg(test)]
        RECEIPT_VALIDATIONS.with(|count| count.set(count.get() + 1));
        validate_page(
            &status.binding,
            &batch,
            &envelope.scope,
            &envelope.restore,
            &envelope.hints,
        )?;
        Some(envelope)
    } else {
        None
    };
    Ok((batch, page))
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct NativeIdentity {
    schema: u8,
    run_id: String,
    file_id: String,
    node_digest: String,
    native_node_id: u64,
}
pub(crate) fn read_native_identity(db: &Connection, run: &str, id: &str) -> Result<Option<u64>> {
    let row:Option<(String,String,u64,String)>=db.query_row(
        "SELECT m.body,m.digest,m.bytes,n.body FROM initialization_native_nodes m JOIN initialization_entries n ON n.run=m.run AND n.id=m.id WHERE m.run=?1 AND m.id=?2",
        params![run,id],|r|Ok((r.get(0)?,r.get(1)?,read_counter(r,2)?,r.get(3)?))).optional()?;
    let Some((body, digest, bytes, node)) = row else {
        return Ok(None);
    };
    if bytes != body.len() as u64 || crate::sha256(body.as_bytes()) != digest {
        return Err(invalid("native candidate identity digest mismatch"));
    }
    let identity: NativeIdentity = serde_json::from_str(&body)?;
    if identity.schema != 1
        || identity.run_id != run
        || identity.file_id != id
        || identity.native_node_id == 0
        || identity.node_digest != crate::sha256(node.as_bytes())
    {
        return Err(invalid("native candidate identity binding mismatch"));
    }
    Ok(Some(identity.native_node_id))
}
pub(super) fn store_native_identity(
    db: &Connection,
    run: &str,
    entry: &InitializationDirectoryEntry,
) -> Result<()> {
    if entry.native_node_id == 0 {
        return Err(invalid("zero native candidate identity"));
    }
    let node: String = db.query_row(
        "SELECT body FROM initialization_entries WHERE run=?1 AND id=?2",
        params![run, entry.node.id],
        |r| r.get(0),
    )?;
    let body = serde_json::to_string(&NativeIdentity {
        schema: 1,
        run_id: run.into(),
        file_id: entry.node.id.clone(),
        node_digest: crate::sha256(node.as_bytes()),
        native_node_id: entry.native_node_id,
    })?;
    let digest = crate::sha256(body.as_bytes());
    // Called only for newly inserted entries. Charge the sidecar to the entry's
    // byte ledger, so existing subtree deletion returns its complete budget.
    db.execute(
        "INSERT INTO initialization_native_nodes(run,id,body,digest,bytes) VALUES(?1,?2,?3,?4,?5)",
        params![run, entry.node.id, body, digest, body.len() as i64],
    )?;
    db.execute(
        "UPDATE initialization_entries SET bytes=bytes+?3 WHERE run=?1 AND id=?2",
        params![run, entry.node.id, body.len() as i64],
    )?;
    db.execute(
        "UPDATE initialization_run SET node_bytes=node_bytes+?2 WHERE run_id=?1",
        params![run, body.len() as i64],
    )?;
    Ok(())
}

impl Store {
    pub fn initialization_event_page(
        &self,
        run: &InitializationRun,
        sequence: u64,
    ) -> Result<Option<InitializationEventPage>> {
        let status = checked(&self.db, run, false)?;
        let (batch, evidence) = read_event_receipt(&self.db, &status, sequence)?;
        Ok(evidence.map(|e| InitializationEventPage {
            batch,
            scope: e.scope,
            restore: e.restore,
            hints: e.hints,
        }))
    }
    pub fn initialization_native_node_id(
        &self,
        run: &InitializationRun,
        id: &str,
    ) -> Result<Option<u64>> {
        let status = checked(&self.db, run, false)?;
        let node = load_node(&self.db, &run.run_id, id)?;
        if id == root_id(&status.binding)? {
            validate_root(&status.binding, &node)?;
            return status
                .binding
                .scope
                .get("node_id")
                .and_then(|v| v.as_u64())
                .filter(|n| *n != 0)
                .map(Some)
                .ok_or_else(|| invalid("selected root native identity missing"));
        }
        read_native_identity(&self.db, &run.run_id, id)
    }
}
