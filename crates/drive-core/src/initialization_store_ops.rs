use super::*;

pub(super) fn begin_directory_in_transaction(
    db: &Connection,
    run: &InitializationRun,
    id: &str,
) -> Result<InitializationDirectoryTicket> {
    let generation: u64 = db.query_row(
        "SELECT r.directory_clock FROM initialization_run r JOIN initialization_directories d ON d.run=r.run_id WHERE r.run_id=?1 AND d.id=?2",
        params![run.run_id,id], |r|read_counter(r,0))?;
    let generation = generation
        .checked_add(1)
        .ok_or_else(|| invalid("directory generation exhausted"))?;
    number(generation)?;
    db.execute(
        "UPDATE initialization_run SET directory_clock=?2 WHERE run_id=?1",
        params![run.run_id, number(generation)?],
    )?;
    db.execute("UPDATE initialization_directories SET generation=?3,state='loading',receipt_seq=NULL,receipt_hash=NULL WHERE run=?1 AND id=?2",params![run.run_id,id,number(generation)?])?;
    clear_fence(db, &run.run_id)?;
    Ok(InitializationDirectoryTicket {
        run: run.clone(),
        directory: id.into(),
        generation,
    })
}

impl Store {
    /// A sibling directory response may have moved or replaced this subtree.
    /// Discard that stale read rather than publishing it or failing the run.
    pub fn initialization_directory_ticket_current(
        &self,
        ticket: &InitializationDirectoryTicket,
    ) -> Result<bool> {
        checked(&self.db, &ticket.run, false)?;
        let row = self
            .db
            .query_row(
                "SELECT generation,state FROM initialization_directories WHERE run=?1 AND id=?2",
                params![ticket.run.run_id, ticket.directory],
                |r| Ok((read_counter(r, 0)?, r.get::<_, String>(1)?)),
            )
            .optional()?;
        Ok(row.is_some_and(|(generation, state)| {
            generation == ticket.generation && state == "loading"
        }))
    }
    pub fn begin_initialization_directory(
        &mut self,
        run: &InitializationRun,
        id: &str,
    ) -> Result<InitializationDirectoryTicket> {
        let tx = self.db.transaction()?;
        checked(&tx, run, false)?;
        // Run-wide allocation prevents ABA after a parent replacement deletes
        // and recreates the same directory ID/row within the same run epoch.
        let ticket = begin_directory_in_transaction(&tx, run, id)?;
        tx.commit()?;
        Ok(ticket)
    }
    /// Accept only a complete, checked direct listing. No offset/partial page is
    /// persisted as completed coverage. False is an exact idempotent duplicate.
    pub fn store_initialization_directory(
        &mut self,
        ticket: &InitializationDirectoryTicket,
        sequence: u64,
        children: &[Node],
    ) -> Result<bool> {
        self.store_initialization_listing(ticket, sequence, Some(children), None)
    }
    /// Numeric identities come from the same native NodeInfo listing as Node.
    /// They are not inferred from file IDs or embedded in content versions.
    pub fn store_initialization_native_directory(
        &mut self,
        ticket: &InitializationDirectoryTicket,
        sequence: u64,
        entries: &[InitializationDirectoryEntry],
    ) -> Result<bool> {
        if entries.len() > 500_000 || entries.iter().any(|entry| entry.native_node_id == 0) {
            return Err(invalid("invalid native directory identities"));
        }
        let children = entries
            .iter()
            .map(|entry| entry.node.clone())
            .collect::<Vec<_>>();
        self.store_initialization_listing(ticket, sequence, Some(&children), Some(entries))
    }
    /// Same receipt as [`Store::store_initialization_native_directory`], but
    /// drops children whose scope-relative path is excluded before the listing
    /// is persisted. Excluded directories therefore never enter the candidate
    /// tree and are never scheduled for enumeration or download.
    pub fn store_initialization_native_directory_filtered(
        &mut self,
        ticket: &InitializationDirectoryTicket,
        sequence: u64,
        entries: &[InitializationDirectoryEntry],
        rules: &crate::exclude::ExcludeRules,
    ) -> Result<bool> {
        if rules.is_empty() {
            return self.store_initialization_native_directory(ticket, sequence, entries);
        }
        let scope = checked(&self.db, &ticket.run, false)?
            .binding
            .scope
            .get("path")
            .and_then(|path| path.as_str())
            .ok_or_else(|| invalid("missing scope path"))?
            .to_owned();
        let absolute = self.initialization_directory_path(&ticket.run, &ticket.directory)?;
        let relative = crate::exclude::scope_relative(&scope, &absolute)
            .ok_or_else(|| invalid("directory is outside the scope"))?;
        let kept: Vec<InitializationDirectoryEntry> = entries
            .iter()
            .filter(|entry| {
                let child = crate::exclude::join_relative(&relative, &entry.node.name);
                !rules.matches(&child, entry.node.directory)
            })
            .cloned()
            .collect();
        self.store_initialization_native_directory(ticket, sequence, &kept)
    }
    pub fn store_initialization_denied(
        &mut self,
        ticket: &InitializationDirectoryTicket,
        sequence: u64,
    ) -> Result<bool> {
        self.store_initialization_listing(ticket, sequence, None, None)
    }
    fn store_initialization_listing(
        &mut self,
        ticket: &InitializationDirectoryTicket,
        sequence: u64,
        listing: Option<&[Node]>,
        native: Option<&[InitializationDirectoryEntry]>,
    ) -> Result<bool> {
        if sequence == 0 {
            return Err(invalid("zero receipt sequence"));
        }
        number(sequence)?;
        let children = listing.unwrap_or(&[]);
        if children.len() > 500_000 {
            return Err(invalid("listing count budget exceeded"));
        }
        let tx = self.db.transaction()?;
        let s = checked(&tx, &ticket.run, false)?;
        if listing.is_none() && ticket.directory == root_id(&s.binding)? {
            return Err(invalid("selected root inaccessible"));
        }
        let (generation,state,seq,hash):(u64,String,Option<u64>,Option<String>)=tx.query_row("SELECT generation,state,receipt_seq,receipt_hash FROM initialization_directories WHERE run=?1 AND id=?2",params![ticket.run.run_id,ticket.directory],|r|Ok((read_counter(r,0)?,r.get(1)?,read_optional_counter(r,2)?,r.get(3)?)))?;
        if generation != ticket.generation {
            return Err(invalid("stale directory generation"));
        }
        let mut ids = HashMap::new();
        let mut names = HashSet::new();
        let mut bytes = 0usize;
        let limits = budget(&tx, &ticket.run.run_id)?;
        for child in children {
            crate::state::valid_node(child)?;
            if child.parent.as_deref() != Some(ticket.directory.as_str())
                || child.id == root_id(&s.binding)?
                || ids.insert(child.id.as_str(), child).is_some()
                || !names.insert(&child.name)
            {
                return Err(invalid("ambiguous directory listing"));
            }
            bytes = bytes
                .checked_add(serde_json::to_vec(child)?.len())
                .ok_or_else(|| invalid("listing byte overflow"))?;
            if bytes as u64 > limits.node_bytes {
                return Err(invalid("listing byte budget exceeded"));
            }
        }
        let digest = if let Some(entries) = native {
            crate::sha256(&serde_json::to_vec(&("native-directory-v1", entries))?)
        } else {
            crate::sha256(&serde_json::to_vec(&(listing.is_none(), children))?)
        };
        let native_ids = native.map(|entries| {
            entries
                .iter()
                .map(|entry| (entry.node.id.as_str(), entry))
                .collect::<HashMap<_, _>>()
        });
        if matches!(state.as_str(), "received" | "denied") {
            if seq == Some(sequence) && hash.as_deref() == Some(&digest) {
                return Ok(false);
            }
            return Err(invalid("receipt identity reused with different contents"));
        }
        if state != "loading" {
            return Err(invalid("directory not loading"));
        }
        // Observe moves in either parent order. Collect all cross-parent IDs
        // before changing anything; child order must not erase fresh siblings.
        let mut moved = Vec::new();
        let mut old_parents = HashSet::new();
        for child in children {
            let previous = match load_node(&tx, &ticket.run.run_id, &child.id) {
                Ok(node) => node,
                Err(error) if error.is_not_found() => continue,
                Err(error) => return Err(error),
            };
            if previous.parent.as_deref() == Some(ticket.directory.as_str()) {
                continue;
            }
            let parent = previous
                .parent
                .as_deref()
                .ok_or_else(|| invalid("cannot relocate candidate root"))?;
            directory_chain(&tx, &ticket.run.run_id, &s.binding, parent)?;
            let target = directory_chain(&tx, &ticket.run.run_id, &s.binding, &ticket.directory)?;
            if target.iter().any(|node| node.id == previous.id) {
                return Err(invalid("candidate move would cross its own subtree"));
            }
            old_parents.insert(parent.to_owned());
            moved.push(previous.id);
        }
        for id in moved {
            delete_subtree(&tx, &ticket.run.run_id, &id)?;
        }
        let old = tx
            .prepare("SELECT body FROM initialization_entries WHERE run=?1 AND parent=?2")?
            .query_map(params![ticket.run.run_id, ticket.directory], |r| {
                r.get::<_, String>(0)
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        let mut retained = HashSet::new();
        for body in old {
            let old: Node = serde_json::from_str(&body)?;
            let incoming_identity = native_ids
                .as_ref()
                .and_then(|ids| ids.get(old.id.as_str()))
                .map(|entry| entry.native_node_id);
            let incoming = ids.get(old.id.as_str()).copied();
            let unchanged = incoming.is_some_and(|new| *new == old);
            let refresh_directory = incoming.is_some_and(|new| {
                old.directory
                    && new.directory
                    && old.parent == new.parent
                    && old.name == new.name
                    && incoming_identity.is_some()
            });
            // No identity proof is reused when a complete parent list removes
            // or structurally replaces this child. A corrupt obsolete sidecar
            // must not prevent deletion of the entire private subtree. Every
            // retention/refresh path still validates its digest before reuse.
            if !unchanged && !refresh_directory {
                delete_subtree(&tx, &ticket.run.run_id, &old.id)?;
                continue;
            }
            let previous_identity = read_native_identity(&tx, &ticket.run.run_id, &old.id)?;
            if unchanged && previous_identity == incoming_identity {
                retained.insert(old.id);
            } else if let Some(new) =
                incoming.filter(|_| refresh_directory && previous_identity == incoming_identity)
            {
                // A fresh listing changed only this directory's metadata/version.
                // Retain discovered identities, never their old coverage/authority.
                // Updating the identity sidecar must also rebind its Node digest.
                let body = serde_json::to_string(new)?;
                if body.len() > 1_048_576 || new.name.len() > 255 {
                    return Err(invalid("node budget exceeded"));
                }
                let previous_bytes: i64 = tx.query_row(
                    "SELECT bytes FROM initialization_entries WHERE run=?1 AND id=?2",
                    params![ticket.run.run_id, old.id],
                    |r| r.get(0),
                )?;
                tx.execute(
                    "DELETE FROM initialization_native_nodes WHERE run=?1 AND id=?2",
                    params![ticket.run.run_id, old.id],
                )?;
                tx.execute(
                    "UPDATE initialization_entries SET body=?3,bytes=?4 WHERE run=?1 AND id=?2",
                    params![ticket.run.run_id, old.id, body, body.len() as i64],
                )?;
                tx.execute(
                    "UPDATE initialization_run SET node_bytes=node_bytes-?2+?3 WHERE run_id=?1",
                    params![ticket.run.run_id, previous_bytes, body.len() as i64],
                )?;
                store_native_identity(
                    &tx,
                    &ticket.run.run_id,
                    native_ids.as_ref().unwrap()[old.id.as_str()],
                )?;
                invalidate(&tx, &ticket.run.run_id, &old.id, true)?;
                retained.insert(old.id);
            } else {
                delete_subtree(&tx, &ticket.run.run_id, &old.id)?;
            }
        }
        for child in children {
            if !retained.contains(&child.id) {
                insert_node(&tx, &ticket.run.run_id, child)?;
                if let Some(entry) = native_ids
                    .as_ref()
                    .and_then(|ids| ids.get(child.id.as_str()))
                {
                    store_native_identity(&tx, &ticket.run.run_id, entry)?;
                }
            }
        }
        // Recheck any surviving old parent and reject its in-flight response.
        // A parent reinserted in this batch must likewise remain pending.
        for parent in old_parents {
            match load_node(&tx, &ticket.run.run_id, &parent) {
                Ok(node) if node.directory => invalidate(&tx, &ticket.run.run_id, &parent, false)?,
                Ok(_) => return Err(invalid("relocation parent became a file")),
                Err(error) if error.is_not_found() => {}
                Err(error) => return Err(error),
            }
        }
        check_node_budget(&tx, &ticket.run.run_id)?;
        tx.execute("UPDATE initialization_directories SET state=?3,receipt_seq=?4,receipt_hash=?5 WHERE run=?1 AND id=?2",params![ticket.run.run_id,ticket.directory,if listing.is_some(){"received"}else{"denied"},number(sequence)?,digest])?;
        clear_fence(&tx, &ticket.run.run_id)?;
        tx.commit()?;
        Ok(true)
    }

    /// Issue immediately before a current-root request. New probes and applied
    /// root changes invalidate older replies; no caller may fabricate completion.
    pub fn begin_initialization_root_refresh(
        &mut self,
        run: &InitializationRun,
    ) -> Result<InitializationRootTicket> {
        let tx = self.db.transaction()?;
        checked(&tx, run, false)?;
        let old = tx.query_row(
            "SELECT root_clock FROM initialization_run WHERE slot=1",
            [],
            |r| read_counter(r, 0),
        )?;
        let generation = old
            .checked_add(1)
            .ok_or_else(|| invalid("root generation exhausted"))?;
        tx.execute("UPDATE initialization_run SET root_clock=?1,root_ticket=?1,root_refresh=1,fenced=NULL,fence_ticket=NULL WHERE slot=1",[number(generation)?])?;
        tx.commit()?;
        Ok(InitializationRootTicket {
            run: run.clone(),
            generation,
        })
    }
    pub fn refresh_initialization_root(
        &mut self,
        ticket: &InitializationRootTicket,
        root: &Node,
    ) -> Result<()> {
        let run = &ticket.run;
        let tx = self.db.transaction()?;
        let s = checked(&tx, run, false)?;
        let current = tx.query_row(
            "SELECT root_ticket FROM initialization_run WHERE slot=1",
            [],
            |r| read_optional_counter(r, 0),
        )?;
        if current != Some(ticket.generation) {
            return Err(invalid("stale root response"));
        }
        validate_root(&s.binding, root)?;
        let body = serde_json::to_string(root)?;
        if body.len() > 1_048_576 {
            return Err(invalid("node budget exceeded"));
        }
        let previous: i64 = tx.query_row(
            "SELECT bytes FROM initialization_entries WHERE run=?1 AND id=?2",
            params![run.run_id, root.id],
            |r| r.get(0),
        )?;
        tx.execute(
            "UPDATE initialization_entries SET body=?3,bytes=?4,name=?5 WHERE run=?1 AND id=?2",
            params![run.run_id, root.id, body, body.len() as i64, root.name],
        )?;
        tx.execute("UPDATE initialization_run SET node_bytes=node_bytes+?2-?3,root_refresh=0,root_ticket=NULL,fenced=NULL,fence_ticket=NULL WHERE run_id=?1",params![run.run_id,body.len() as i64,previous])?;
        check_node_budget(&tx, &run.run_id)?;
        crate::preview::verified_root_in_transaction(&tx, run)?;
        tx.commit()?;
        Ok(())
    }

    /// Store translated native events before acknowledging their receipt. The
    /// durable product inbox/cursor is deliberately untouched.
    pub fn record_initialization_events(
        &mut self,
        run: &InitializationRun,
        sequence: u64,
        batch: &Batch,
    ) -> Result<bool> {
        self.record_initialization_events_inner(run, sequence, batch, None)
    }
    pub fn record_initialization_event_page(
        &mut self,
        run: &InitializationRun,
        sequence: u64,
        page: &InitializationEventPage,
    ) -> Result<bool> {
        self.record_initialization_events_inner(run, sequence, &page.batch, Some(page))
    }
    fn record_initialization_events_inner(
        &mut self,
        run: &InitializationRun,
        sequence: u64,
        batch: &Batch,
        page: Option<&InitializationEventPage>,
    ) -> Result<bool> {
        number(sequence)?;
        let from = cursor(&batch.from)?;
        let to = cursor(&batch.to)?;
        if sequence == 0 || to <= from || batch.events.len() > 500_000 {
            return Err(invalid("invalid candidate event batch"));
        }
        let tx = self.db.transaction()?;
        let s = checked(&tx, run, false)?;
        let limits = budget(&tx, &run.run_id)?;
        let mut event_bytes = 0usize;
        for event in &batch.events {
            if event.id.is_empty() || event.id.len() > 4096 {
                return Err(invalid("invalid event identity"));
            }
            if let Change::Upsert(node) = &event.change {
                crate::state::valid_node(node)?;
            }
            event_bytes = event_bytes
                .checked_add(serde_json::to_vec(event)?.len())
                .ok_or_else(|| invalid("event byte overflow"))?;
            if event_bytes as u64 > limits.event_bytes {
                return Err(invalid("candidate event budget exceeded"));
            }
        }
        let body = serde_json::to_string(batch)?;
        let digest = super::evidence::page_hash(body.as_bytes());
        let hint_body = page
            .map(|page| encode_event_page(&s.binding, &run.run_id, sequence, &digest, page))
            .transpose()?;
        if let Some(old) = tx
            .query_row(
                "SELECT digest FROM initialization_events WHERE run=?1 AND seq=?2",
                params![run.run_id, number(sequence)?],
                |r| r.get::<_, String>(0),
            )
            .optional()?
        {
            let (saved_batch, saved_page) = read_event_receipt(&tx, &s, sequence)?;
            if old == digest && saved_batch == *batch {
                if let (Some(incoming), Some(saved)) = (page, saved_page.as_ref()) {
                    if !saved.matches_page(incoming) {
                        return Err(invalid("event hint evidence cannot be replaced"));
                    }
                }
                // An upgraded peer may replay a legacy receipt with hints. Its
                // original evidence remains absent; never backfill on replay.
                return Ok(false);
            }
            return Err(invalid("event sequence reused with different contents"));
        }
        if sequence
            != s.received_sequence
                .checked_add(1)
                .ok_or_else(|| invalid("event sequence exhausted"))?
            || batch.from != s.received_cursor
        {
            return Err(invalid("candidate event gap"));
        }
        let old_bytes: u64 = tx.query_row(
            "SELECT event_bytes FROM initialization_run WHERE slot=1",
            [],
            |r| read_counter(r, 0),
        )?;
        let total = old_bytes
            .checked_add(body.len() as u64)
            .and_then(|n| n.checked_add(hint_body.as_ref().map_or(0, |body| body.len() as u64)))
            .ok_or_else(|| invalid("event byte overflow"))?;
        if sequence > limits.event_batches || total > limits.event_bytes {
            return Err(invalid("candidate event budget exceeded"));
        }
        tx.execute("INSERT INTO initialization_events(run,seq,from_cursor,to_cursor,body,digest,bytes) VALUES(?1,?2,?3,?4,?5,?6,?7)",params![run.run_id,number(sequence)?,batch.from,batch.to,body,digest,body.len() as i64])?;
        if let Some(hints) = hint_body {
            tx.execute("INSERT INTO initialization_event_hints(run,seq,body,digest,bytes) VALUES(?1,?2,?3,?4,?5)",
                params![run.run_id,number(sequence)?,hints,super::evidence::page_hash(hints.as_bytes()),hints.len() as i64])?;
        }
        tx.execute("UPDATE initialization_run SET received=?2,received_seq=?3,event_bytes=?4,fenced=NULL,fence_ticket=NULL WHERE run_id=?1",params![run.run_id,batch.to,number(sequence)?,number(total)?])?;
        tx.commit()?;
        Ok(true)
    }
    pub fn pending_initialization_events(
        &self,
        run: &InitializationRun,
        limit: usize,
    ) -> Result<Vec<(u64, Batch)>> {
        let s = checked(&self.db, run, false)?;
        if limit == 0 || limit > 64 {
            return Err(invalid("invalid event page limit"));
        }
        let rows=self.db.prepare("SELECT seq FROM initialization_events WHERE run=?1 AND seq>?2 ORDER BY seq LIMIT ?3")?.query_map(params![run.run_id,number(s.applied_sequence)?,limit as i64],|r|read_counter(r,0))?.collect::<std::result::Result<Vec<_>,_>>()?;
        rows.into_iter()
            .map(|seq| Ok((seq, read_event_receipt(&self.db, &s, seq)?.0)))
            .collect()
    }
    /// The owner needs only the next durable sequence, not another full decode
    /// of its body. Detect missing receipts without skipping the gap.
    pub fn initialization_next_pending_sequence(
        &self,
        run: &InitializationRun,
    ) -> Result<Option<u64>> {
        let s = checked(&self.db, run, false)?;
        if s.applied_sequence == s.received_sequence {
            return Ok(None);
        }
        let next: Option<u64> = self.db.query_row(
            "SELECT seq FROM initialization_events WHERE run=?1 AND seq>?2 ORDER BY seq LIMIT 1",
            params![run.run_id, number(s.applied_sequence)?], |row| read_counter(row, 0),
        ).optional()?;
        let expected = s
            .applied_sequence
            .checked_add(1)
            .ok_or_else(|| invalid("event sequence exhausted"))?;
        if next != Some(expected) || expected > s.received_sequence {
            return Err(invalid("candidate pending event gap"));
        }
        Ok(next)
    }
    /// A verified directory race while waiting for coverage can depend on a
    /// later event page. Apply the durable head with a full conservative recheck
    /// so the owner can resume Pull. No event or fence boundary is skipped.
    pub fn apply_initialization_events_after_directory_race(
        &mut self,
        run: &InitializationRun,
        sequence: u64,
    ) -> Result<InitializationApplyResult> {
        self.apply_initialization_events_diagnosed(run, sequence, true)
            .map(|outcome| outcome.result)
    }
    /// Apply means that the event's required rechecks are durably scheduled,
    /// not that old event metadata was copied over a newer directory listing.
    pub fn apply_initialization_events(
        &mut self,
        run: &InitializationRun,
        sequence: u64,
    ) -> Result<InitializationApplyResult> {
        self.apply_initialization_events_diagnosed(run, sequence, false)
            .map(|outcome| outcome.result)
    }
    /// The same transaction and policy as the compatibility wrappers. Return
    /// diagnostic success only after durable obligations/cursor commit; errors
    /// never return a misleading successful plan summary.
    pub fn apply_initialization_events_diagnosed(
        &mut self,
        run: &InitializationRun,
        sequence: u64,
        directory_race: bool,
    ) -> Result<InitializationApplyOutcome> {
        self.apply_initialization_events_with(run, sequence, directory_race, None)
    }
    /// Apply the page the caller translated and durably recorded at this exact
    /// sequence in the same writer session. The page was validated by
    /// `record_initialization_event_page`; it is authoritative evidence, so the
    /// receipt body must not be read, parsed or hashed a second time.
    pub fn apply_initialization_events_page_diagnosed(
        &mut self,
        run: &InitializationRun,
        sequence: u64,
        directory_race: bool,
        page: &InitializationEventPage,
    ) -> Result<InitializationApplyOutcome> {
        self.apply_initialization_events_with(run, sequence, directory_race, Some(page))
    }
    fn apply_initialization_events_with(
        &mut self,
        run: &InitializationRun,
        sequence: u64,
        directory_race: bool,
        page: Option<&InitializationEventPage>,
    ) -> Result<InitializationApplyOutcome> {
        let tx = self.db.transaction()?;
        let s = checked(&tx, run, false)?;
        if sequence == 0 || sequence > s.received_sequence {
            return Err(invalid("event not received"));
        }
        if sequence <= s.applied_sequence {
            if page.is_none() {
                read_event_receipt(&tx, &s, sequence)?;
            }
            return Ok(InitializationApplyOutcome {
                result: InitializationApplyResult::AlreadyApplied,
                summary: InitializationApplySummary::default(),
            });
        }
        if sequence != s.applied_sequence + 1 {
            return Err(invalid("event application gap"));
        }
        let receipt;
        let (batch, hints): (&Batch, Option<&[InitializationEventHint]>) = match page {
            Some(page) => (&page.batch, Some(page.hints.as_slice())),
            None => {
                receipt = read_event_receipt(&tx, &s, sequence)?;
                (
                    &receipt.0,
                    receipt.1.as_ref().map(|envelope| envelope.hints()),
                )
            }
        };
        if batch.from != s.applied_cursor {
            return Err(invalid("candidate applied cursor mismatch"));
        }
        let root = root_id(&s.binding)?;
        // A verified durable head can prove the selected root unusable even
        // though this batch must never be applied. Persist only revocation;
        // returning Err must not roll that evidence back into a visible preview.
        for event in &batch.events {
            let rejection = match &event.change {
                Change::Delete { id } if id == root => Some(invalid("selected root removed")),
                Change::Upsert(node) if node.id == root => validate_root(&s.binding, node).err(),
                _ => None,
            };
            if let Some(rejection) = rejection {
                crate::preview::revoke_root_in_transaction(&tx, &s)?;
                tx.commit()?;
                return Err(rejection);
            }
        }
        // Plan the entire page before mutating coverage: children can precede
        // their unknown directory event. No hinted identity is inserted here.
        let mut unknown_directories = HashMap::new();
        // A bounded optional optimization. Larger pages retain the existing
        // conservative policy, rather than multiplying ancestry work per item.
        if !directory_race
            && batch.events.len() <= 4096
            && batch
                .events
                .iter()
                .filter(|event| matches!(&event.change, Change::Upsert(n) if n.directory))
                .count()
                <= 64
        {
            let mut occurrences = HashMap::<&str, usize>::new();
            let mut buckets = HashMap::<(&str, &str), usize>::new();
            for event in &batch.events {
                let id = match &event.change {
                    Change::Upsert(n) => n.id.as_str(),
                    Change::Delete { id } => id.as_str(),
                };
                *occurrences.entry(id).or_default() += 1;
                if let Change::Upsert(node) = &event.change {
                    if node.directory {
                        if let Some(parent) = node.parent.as_deref() {
                            *buckets.entry((parent, node.name.as_str())).or_default() += 1;
                        }
                    }
                }
            }
            let mut unknown_nodes = Vec::new();
            for (ordinal, event) in batch.events.iter().enumerate() {
                let Change::Upsert(node) = &event.change else {
                    continue;
                };
                if !node.directory
                    || occurrences.get(node.id.as_str()) != Some(&1)
                    || node
                        .parent
                        .as_deref()
                        .is_none_or(|parent| buckets.get(&(parent, node.name.as_str())) != Some(&1))
                {
                    continue;
                }
                match load_node(&tx, &run.run_id, &node.id) {
                    Ok(_) => continue,
                    Err(e) if e.is_not_found() => {}
                    Err(e) => return Err(e),
                }
                unknown_nodes.push((ordinal, node));
                if let Some(plan) = local_recheck::plan_unknown_directory(
                    &tx,
                    &s,
                    node,
                    hints.and_then(|hint| hint.get(ordinal)),
                )? {
                    unknown_directories.insert(node.id.as_str(), plan);
                }
            }
            // Bounded fixed point, independent of event order. Only a chain
            // already connected to a real native anchor can admit another ID.
            // Cycles, ambiguous names and chains with missing page evidence
            // remain unsupported and trigger the existing complete recheck.
            for _ in 0..unknown_nodes.len() {
                let mut progress = false;
                for &(ordinal, node) in &unknown_nodes {
                    if unknown_directories.contains_key(node.id.as_str()) {
                        continue;
                    }
                    let Some(parent_plan) = node
                        .parent
                        .as_deref()
                        .and_then(|parent| unknown_directories.get(parent))
                    else {
                        continue;
                    };
                    if let Some(plan) = local_recheck::plan_nested_unknown_directory(
                        &tx,
                        &s,
                        node,
                        hints.and_then(|hint| hint.get(ordinal)),
                        parent_plan,
                    )? {
                        unknown_directories.insert(node.id.as_str(), plan);
                        progress = true;
                    }
                }
                if !progress {
                    break;
                }
            }
        }
        let mut all = directory_race;
        let mut full_reason =
            directory_race.then_some(InitializationFullRecheckReason::DirectoryRace);
        let mut missing_file_parent = false;
        let mut root_refresh = false;
        let mut parents = HashSet::new();
        let mut local_subtrees = HashSet::new();
        for (ordinal, event) in batch.events.iter().enumerate() {
            let id = match &event.change {
                Change::Upsert(node) => &node.id,
                Change::Delete { id } => id,
            };
            let old = match load_node(&tx, &run.run_id, id) {
                Ok(n) => Some(n),
                Err(e) if e.is_not_found() => None,
                Err(e) => return Err(e),
            };
            if id == root {
                match &event.change {
                    Change::Upsert(node) => validate_root(&s.binding, node)?,
                    _ => return Err(invalid("selected root removed")),
                }
                all = true;
                full_reason = full_reason.max(Some(InitializationFullRecheckReason::Root));
                root_refresh = true;
                crate::preview::invalidate_root_in_transaction(&tx, &s)?;
                continue;
            }
            if let Some(plan) = unknown_directories.get(id.as_str()) {
                parents.insert(plan.parent.clone());
                continue;
            }
            let directory_event = old.as_ref().is_some_and(|node| node.directory)
                || matches!(&event.change, Change::Upsert(node) if node.directory);
            if matches!((&old, &event.change), (Some(previous), Change::Upsert(node)) if previous.directory != node.directory)
            {
                all = true;
                full_reason = full_reason.max(Some(InitializationFullRecheckReason::TypeMismatch));
            }
            if directory_event && !all {
                if let Some(plan) = local_recheck::plan_known_directory(
                    &tx,
                    &s,
                    event,
                    hints.and_then(|hint| hint.get(ordinal)),
                    old.as_ref(),
                )? {
                    local_subtrees.insert(plan.directory);
                    parents.extend(plan.parents);
                    continue;
                }
            }
            if old.as_ref().is_some_and(|node| node.directory) {
                all = true;
                full_reason =
                    full_reason.max(Some(InitializationFullRecheckReason::UnsupportedDirectory));
            }
            if let Some(parent) = old.as_ref().and_then(|node| node.parent.as_ref()) {
                parents.insert(parent.clone());
            }
            match &event.change {
                Change::Upsert(node) => {
                    crate::state::valid_node(node)?;
                    if node.directory {
                        all = true;
                        full_reason = full_reason
                            .max(Some(InitializationFullRecheckReason::UnsupportedDirectory));
                    }
                    let parent = node
                        .parent
                        .as_ref()
                        .ok_or_else(|| invalid("replacement root event"))?;
                    match load_node(&tx, &run.run_id, parent) {
                        Ok(node) if node.directory => {
                            parents.insert(parent.clone());
                        }
                        Ok(_) => {
                            all = true;
                            full_reason = full_reason
                                .max(Some(InitializationFullRecheckReason::TypeMismatch));
                        }
                        Err(error) if error.is_not_found() => {
                            let covered =
                                unknown_directories
                                    .get(parent.as_str())
                                    .is_some_and(|plan| {
                                        local_recheck::covers_unknown_parent_file(
                                            plan,
                                            node,
                                            hints.and_then(|hint| hint.get(ordinal)),
                                        )
                                    });
                            if !covered {
                                missing_file_parent = true;
                            }
                        }
                        Err(error) => return Err(error),
                    }
                }
                Change::Delete { .. } if old.is_none() => {
                    all = true;
                    full_reason =
                        full_reason.max(Some(InitializationFullRecheckReason::UnknownDelete));
                }
                _ => {}
            }
        }
        // Inspect the entire batch before changing coverage. Only verified
        // known-directory or existing-parent plans are local; root, unsupported
        // unknowns/mismatches and uncovered mixed dependencies remain conservative.
        // This is our scheduling policy, not an inferred Windows algorithm.
        if missing_file_parent && !all && local_subtrees.is_empty() {
            // Use the partial pending index first. Do not put both EXISTS in a
            // SQL OR: SQLite need not short circuit the loading-row scan.
            let pending: bool = tx.query_row(
                "SELECT EXISTS(SELECT 1 FROM initialization_directories WHERE run=?1 AND state IN ('pending','failed'))",
                [&run.run_id],
                |row| row.get(0),
            )?;
            let unfinished = pending || tx.query_row(
                "SELECT EXISTS(SELECT 1 FROM initialization_directories WHERE run=?1 AND state='loading')",
                [&run.run_id],
                |row| row.get::<_, bool>(0),
            )?;
            if unfinished {
                // Leave this durable receipt unapplied. In particular, neither
                // the applied cursor nor completed directory coverage changes.
                return Ok(InitializationApplyOutcome {
                    result: InitializationApplyResult::AwaitingCoverage,
                    summary: InitializationApplySummary {
                        awaiting_coverage_reason: Some(
                            InitializationCoverageWaitReason::MissingFileParent,
                        ),
                        ..InitializationApplySummary::default()
                    },
                });
            }
        }
        all |= missing_file_parent;
        if missing_file_parent {
            full_reason = full_reason.max(Some(InitializationFullRecheckReason::MissingParent));
        }
        let summary = if all {
            InitializationApplySummary {
                full_recheck_reason: full_reason,
                ..InitializationApplySummary::default()
            }
        } else {
            InitializationApplySummary {
                local_parent_plans: parents.len() as u64,
                local_subtree_plans: local_subtrees.len() as u64,
                unknown_directory_dependency_plans: unknown_directories.len() as u64,
                ..InitializationApplySummary::default()
            }
        };
        if all {
            invalidate(&tx, &run.run_id, root, true)?;
        } else {
            local_recheck::invalidate_local_subtrees(&tx, &s, &local_subtrees)?;
            for parent in parents {
                invalidate(&tx, &run.run_id, &parent, false)?;
            }
        }
        tx.execute("UPDATE initialization_run SET applied=?2,applied_seq=?3,root_refresh=max(root_refresh,?4),root_ticket=CASE WHEN ?4 THEN NULL ELSE root_ticket END,fenced=NULL,fence_ticket=NULL WHERE run_id=?1",params![run.run_id,batch.to,number(sequence)?,root_refresh])?;
        tx.commit()?;
        Ok(InitializationApplyOutcome {
            result: InitializationApplyResult::Applied,
            summary,
        })
    }
    /// Capture the candidate boundary immediately before the native stationary
    /// fence request. Every subsequent mutation invalidates this one-use ticket.
    pub fn begin_initialization_fence(
        &mut self,
        run: &InitializationRun,
    ) -> Result<InitializationFenceTicket> {
        let tx = self.db.transaction()?;
        let state = checked(&tx, run, false)?;
        let old = tx.query_row(
            "SELECT fence_clock FROM initialization_run WHERE slot=1",
            [],
            |r| read_counter(r, 0),
        )?;
        let generation = old
            .checked_add(1)
            .ok_or_else(|| invalid("fence generation exhausted"))?;
        tx.execute(
            "UPDATE initialization_run SET fence_clock=?1,fence_ticket=?1,fenced=NULL WHERE slot=1",
            [number(generation)?],
        )?;
        tx.commit()?;
        Ok(InitializationFenceTicket {
            run: run.clone(),
            generation,
            cursor: state.applied_cursor,
        })
    }
    /// Confirm only an independently validated stationary response to this exact
    /// request. A reply crossing re-listing, events or another probe is stale.
    pub fn confirm_initialization_fence(
        &mut self,
        ticket: &InitializationFenceTicket,
        at: &str,
    ) -> Result<()> {
        cursor(at)?;
        let tx = self.db.transaction()?;
        let s = checked(&tx, &ticket.run, false)?;
        let current = tx.query_row(
            "SELECT fence_ticket FROM initialization_run WHERE slot=1",
            [],
            |r| read_optional_counter(r, 0),
        )?;
        if current != Some(ticket.generation) || at != ticket.cursor {
            return Err(invalid("stale fence response"));
        }
        validate_complete(&tx, &s, at)?;
        tx.execute(
            "UPDATE initialization_run SET fenced=?1,fence_ticket=NULL WHERE slot=1",
            [at],
        )?;
        tx.commit()?;
        Ok(())
    }
    #[cfg(feature = "native")]
    pub fn complete_initialization(&mut self, run: &InitializationRun) -> Result<()> {
        let s = checked(&self.db, run, false)?;
        let at: Option<String> = self.db.query_row(
            "SELECT fenced FROM initialization_run WHERE slot=1",
            [],
            |r| r.get(0),
        )?;
        let at = at.ok_or_else(|| invalid("independent fence missing"))?;
        validate_complete(&self.db, &s, &at)?;
        let nodes = all_nodes(&self.db, &run.run_id)?;
        let denied=self.db.prepare("SELECT id FROM initialization_directories WHERE run=?1 AND state='denied' ORDER BY id")?.query_map([&run.run_id],|r|r.get::<_,String>(0))?.collect::<std::result::Result<Vec<_>,_>>()?;
        let snapshot = crate::native::RemoteSnapshot {
            scope: serde_json::from_value(s.binding.scope.clone())?,
            restore: s.binding.restore.clone(),
            cursor: at.clone(),
            nodes,
            inaccessible: denied,
            skipped_symlinks: 0,
        };
        let commit = InitializationCommit {
            run: run.clone(),
            cursor: at,
        };
        self.install_native_snapshot_for_initialization(
            &snapshot,
            &s.binding.expected_cursor,
            &commit,
        )
    }
}

fn all_nodes(db: &Connection, run: &str) -> Result<Vec<Node>> {
    let bodies = db
        .prepare("SELECT body FROM initialization_entries WHERE run=?1 ORDER BY id")?
        .query_map([run], |r| r.get::<_, String>(0))?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    bodies
        .into_iter()
        .map(|body| Ok(serde_json::from_str(&body)?))
        .collect()
}
fn validate_complete(db: &Connection, s: &InitializationStatus, at: &str) -> Result<()> {
    validate_completion_state(db, s, at)?;
    let nodes = all_nodes(db, &s.run.run_id)?;
    let mut children: HashMap<&str, Vec<&Node>> = HashMap::new();
    let root = root_id(&s.binding)?;
    let mut roots = 0;
    let mut names = HashSet::new();
    for node in &nodes {
        crate::state::valid_node(node)?;
        if !names.insert((node.parent.as_deref(), node.name.as_str())) {
            return Err(invalid("ambiguous candidate names"));
        }
        if let Some(parent) = &node.parent {
            children.entry(parent).or_default().push(node);
        } else {
            validate_root(&s.binding, node)?;
            roots += 1;
        }
    }
    if roots != 1 || nodes.len() as u64 != s.node_count {
        return Err(invalid("candidate root or count mismatch"));
    }
    let mut pending = vec![(root, 0usize)];
    let mut visited = HashSet::new();
    while let Some((id, depth)) = pending.pop() {
        if !visited.insert(id) {
            return Err(invalid("candidate cycle"));
        }
        for node in children.get(id).into_iter().flatten() {
            if depth + 1 > if node.directory { 128 } else { 129 } {
                return Err(invalid("candidate depth exceeded"));
            }
            if !node.directory && children.contains_key(node.id.as_str()) {
                return Err(invalid("candidate file has children"));
            }
            pending.push((&node.id, depth + 1));
        }
    }
    if visited.len() != nodes.len() {
        return Err(invalid("candidate parent missing or cycle"));
    }
    Ok(())
}
fn validate_completion_state(db: &Connection, s: &InitializationStatus, at: &str) -> Result<()> {
    let base_restore: String = db.query_row(
        "SELECT base_restore FROM initialization_run WHERE run_id=?1",
        [&s.run.run_id],
        |r| r.get(0),
    )?;
    // A candidate may start behind the committed boundary and catch up. It may
    // not silently reset that boundary within the same server restore domain.
    if base_restore == s.binding.restore
        && !s.binding.expected_cursor.is_empty()
        && cursor(at)? < cursor(&s.binding.expected_cursor)?
    {
        let (epoch, expected): (Option<u64>, Option<String>) = db.query_row(
            "SELECT reset_epoch,reset_expected FROM initialization_run WHERE run_id=?1",
            [&s.run.run_id],
            |r| Ok((read_optional_counter(r, 0)?, r.get(1)?)),
        )?;
        if epoch != Some(s.run.epoch)
            || expected.as_deref() != Some(s.binding.expected_cursor.as_str())
        {
            return Err(invalid("candidate cursor precedes committed boundary"));
        }
    }
    if s.root_needs_refresh
        || s.received_cursor != at
        || s.applied_cursor != at
        || s.received_sequence != s.applied_sequence
    {
        return Err(invalid("events or root refresh incomplete"));
    }
    let incomplete:bool=db.query_row("SELECT EXISTS(SELECT 1 FROM initialization_entries n LEFT JOIN initialization_directories d ON d.run=n.run AND d.id=n.id WHERE n.run=?1 AND n.directory=1 AND (d.id IS NULL OR d.state NOT IN ('received','denied'))) OR EXISTS(SELECT 1 FROM initialization_directories d LEFT JOIN initialization_entries n ON n.run=d.run AND n.id=d.id WHERE d.run=?1 AND (n.id IS NULL OR n.directory!=1)) OR EXISTS(SELECT 1 FROM initialization_directories d JOIN initialization_entries n ON n.run=d.run AND n.parent=d.id WHERE d.run=?1 AND d.state='denied')",[&s.run.run_id],|r|r.get(0))?;
    if incomplete {
        return Err(invalid("directory coverage incomplete"));
    }
    check_node_budget(db, &s.run.run_id)?;
    Ok(())
}

/// Constructible only by complete_initialization after reading the candidate.
/// The installer repeats identity validation and consumes this within its own
/// authoritative snapshot transaction; no public raw gate-clear operation exists.
pub(crate) struct InitializationCommit {
    run: InitializationRun,
    cursor: String,
}
pub(crate) fn installation_restore(
    db: &Connection,
    commit: &InitializationCommit,
) -> Result<String> {
    Ok(checked(db, &commit.run, false)?.binding.restore)
}
pub(crate) fn check_install_gate(
    db: &Connection,
    commit: Option<&InitializationCommit>,
) -> Result<()> {
    match commit {
        Some(commit) => {
            let s = checked(db, &commit.run, false)?;
            validate_completion_state(db, &s, &commit.cursor)?;
            let fenced: Option<String> = db.query_row(
                "SELECT fenced FROM initialization_run WHERE slot=1",
                [],
                |r| r.get(0),
            )?;
            if fenced.as_deref() != Some(&commit.cursor) {
                return Err(invalid("final fence changed"));
            }
            Ok(())
        }
        None => {
            if db.query_row(
                "SELECT EXISTS(SELECT 1 FROM initialization_run WHERE state!='ready')",
                [],
                |r| r.get::<_, bool>(0),
            )? {
                Err(invalid("active candidate requires atomic final install"))
            } else {
                Ok(())
            }
        }
    }
}
pub(crate) fn finish_in_transaction(
    db: &Connection,
    commit: &InitializationCommit,
    at: &str,
) -> Result<()> {
    // Called after authoritative cursor updates; checked() would intentionally
    // reject that new cursor against the old expected cursor here.
    let s = status(db)?.ok_or_else(|| invalid("run disappeared during install"))?;
    let fenced: Option<String> = db.query_row(
        "SELECT fenced FROM initialization_run WHERE slot=1",
        [],
        |r| r.get(0),
    )?;
    if s.run != commit.run
        || s.state != "active"
        || at != commit.cursor
        || fenced.as_deref() != Some(at)
        || s.applied_cursor != at
        || s.received_cursor != at
    {
        return Err(invalid("final run changed"));
    }
    crate::preview::promote_in_transaction(db, &s)?;
    db.execute(
        "UPDATE initialization_run SET state='ready',error=NULL WHERE slot=1",
        [],
    )?;
    Ok(())
}
