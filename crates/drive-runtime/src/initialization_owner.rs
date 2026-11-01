//! FSD-owned initialization. Replies are repeatable reads, not checkpoints;
//! the next request is issued only after the prior receipt is committed locally.
use super::*;
use drive_core::{
    Batch,
    initialization_store::{
        InitializationApplyResult, InitializationBinding, InitializationBudget, InitializationRun,
    },
    native::{
        InitializationBody as Body, InitializationReply, InitializationRequest as Request, Scope,
    },
    snapshot_progress::{Phase, Progress},
};
use std::{fmt, io::Read, time::Instant};

#[derive(Clone)]
pub(super) struct Session {
    task: String,
    scope: Scope,
    restore: String,
}

#[derive(Debug)]
enum Recheck {
    Cursor,
    Filtered,
    Directory,
}
impl fmt::Display for Recheck {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("native initialization requires a verified recheck")
    }
}
impl std::error::Error for Recheck {}
fn recheck(error: &io::Error) -> Option<&Recheck> {
    error.get_ref()?.downcast_ref()
}
fn invalid(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}
fn number(cursor: &str) -> io::Result<u64> {
    cursor
        .strip_prefix("native-v1:")
        .and_then(|n| n.parse().ok())
        .ok_or_else(|| invalid("invalid initialization cursor"))
}

impl Bridge {
    pub(crate) fn enable_initialization_owner(&self, task: &str) -> io::Result<()> {
        uuid::Uuid::parse_str(task).map_err(|_| invalid("invalid initialization task identity"))?;
        let mut current = self.initialization_task.lock().unwrap();
        if current.as_deref().is_some_and(|old| old != task) {
            return Err(invalid("initialization task changed"));
        }
        *current = Some(task.into());
        self.session_pending.store(true, Ordering::Release);
        Ok(())
    }

    pub(super) fn attach_initialization_session(
        &self,
        rpc: Arc<Rpc>,
        restore: &str,
        raw_scope: &str,
    ) -> io::Result<()> {
        let task = self
            .initialization_task
            .lock()
            .unwrap()
            .clone()
            .ok_or_else(|| invalid("initialization task absent"))?;
        let scope: Scope = serde_json::from_str(raw_scope)?;
        if restore.is_empty() || !rpc.is_connected() {
            return Err(io::ErrorKind::NotConnected.into());
        }
        // Never hold Service::Control here: Pause must be able to cancel the old
        // RPC while a mutation drains. No network operation holds this writer.
        let _writes = self.recovery_gate.write().unwrap();
        if !rpc.is_connected() {
            return Err(io::ErrorKind::NotConnected.into());
        }
        let mut store = self.store.lock().unwrap();
        if let Some((old_restore, old_scope)) =
            store.native_scope_binding().map_err(io::Error::other)?
        {
            if serde_json::from_str::<Scope>(&old_scope)? != scope {
                return Err(invalid("native selected scope changed"));
            }
            if old_restore == restore {
                store
                    .bind_native_scope(restore, &old_scope)
                    .map_err(io::Error::other)?;
            }
            // A new restore is held behind the session barrier. Only a verified
            // root and candidate completion may replace authoritative metadata.
        } else {
            store
                .bind_native_scope(restore, raw_scope)
                .map_err(io::Error::other)?;
        }
        self.session_pending.store(true, Ordering::Release);
        *self.initialization_session.lock().unwrap() = Some(Session {
            task,
            scope,
            restore: restore.into(),
        });
        let old = std::mem::replace(&mut *self.connection.lock().unwrap(), rpc);
        old.close();
        *self.last_error.lock().unwrap() = None;
        Ok(())
    }

    fn session(&self) -> io::Result<Session> {
        self.initialization_session
            .lock()
            .unwrap()
            .clone()
            .ok_or_else(|| invalid("authenticated initialization session absent"))
    }
    pub(super) fn current(&self, rpc: &Arc<Rpc>) -> io::Result<()> {
        if rpc.is_connected() && Arc::ptr_eq(rpc, &self.connection()) {
            Ok(())
        } else {
            Err(io::ErrorKind::NotConnected.into())
        }
    }
    pub(super) fn owner_candidate_pending(&self) -> io::Result<bool> {
        if self.initialization_task.lock().unwrap().is_none() {
            return Ok(false);
        }
        let session = self.session()?;
        let store = self.store.lock().unwrap();
        Ok(store
            .initialization_writeback_blocked()
            .map_err(io::Error::other)?
            || store
                .native_scope_binding()
                .map_err(io::Error::other)?
                .is_none_or(|(restore, _)| restore != session.restore))
    }
    pub(super) fn finish_session_catchup(&self, rpc: &Arc<Rpc>) -> io::Result<()> {
        if self.initialization_task.lock().unwrap().is_none() {
            return Ok(());
        }
        let _writes = self.recovery_gate.write().unwrap();
        self.current(rpc)?;
        let session = self.session()?;
        let store = self.store.lock().unwrap();
        if store
            .initialization_writeback_blocked()
            .map_err(io::Error::other)?
            || store
                .native_scope_binding()
                .map_err(io::Error::other)?
                .is_none_or(|(restore, _)| restore != session.restore)
        {
            return Err(invalid("initialization is not committed"));
        }
        self.session_pending.store(false, Ordering::Release);
        drop(store);
        self.signal_upload();
        Ok(())
    }
    pub(super) fn owner_initial_pull(&self) -> io::Result<bool> {
        let event = self.event_gate.lock().unwrap();
        let reason = if self.owner_candidate_pending()? {
            Some(crate::initialization::RecoveryReason::InitialIndex)
        } else {
            let store = self.store.lock().unwrap();
            if store.cursor().map_err(io::Error::other)?.is_empty() {
                Some(crate::initialization::RecoveryReason::InitialIndex)
            } else if store.inaccessible_count().map_err(io::Error::other)? > 0 {
                Some(crate::initialization::RecoveryReason::PermissionRecheck)
            } else {
                None
            }
        };
        if let Some(reason) = reason {
            self.run_initialization_owner(reason)
        } else {
            drop(event);
            self.pull()
        }
    }

    fn initialization_read(
        &self,
        rpc: &Arc<Rpc>,
        session: &Session,
        request: Request,
        deadline: Instant,
    ) -> io::Result<Body> {
        self.current(rpc)?;
        let timeout = deadline
            .checked_duration_since(Instant::now())
            .filter(|d| !d.is_zero())
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::TimedOut,
                    "initialization work budget exhausted",
                )
            })?
            .min(Duration::from_secs(900));
        let response = rpc.call(Command::Initialization(request), timeout)?;
        let (size, hash) = match response.body {
            Response::Initialization { size, sha256 } if size <= 512 * 1024 * 1024 => {
                (size, sha256)
            }
            Response::Failed { category } if category == "native_cursor_reset" => {
                return Err(io::Error::new(io::ErrorKind::InvalidData, Recheck::Cursor));
            }
            Response::Failed { category } if category == "native_filtered_type_changed" => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    Recheck::Filtered,
                ));
            }
            Response::Failed { category } if category == "native_snapshot_changed" => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    Recheck::Directory,
                ));
            }
            body => return Err(operation_error(body)),
        };
        let descriptor = response
            .descriptor
            .ok_or_else(|| invalid("initialization descriptor absent"))?;
        let reply = decode_reply(std::fs::File::from(descriptor), size, &hash)?;
        if reply.scope != session.scope || reply.restore != session.restore {
            return Err(invalid("initialization response session mismatch"));
        }
        self.current(rpc)?;
        Ok(reply.body)
    }

    fn apply_candidate_events(
        &self,
        rpc: &Arc<Rpc>,
        run: &InitializationRun,
        batch: Option<Batch>,
        directory_race: bool,
    ) -> io::Result<bool> {
        let _writes = self.recovery_gate.write().unwrap();
        self.current(rpc)?;
        let mut store = self.store.lock().unwrap();
        let mut changed = false;
        if let Some(batch) = batch {
            let status = store
                .initialization_status()
                .map_err(io::Error::other)?
                .ok_or_else(|| invalid("candidate absent"))?;
            if status.run != *run {
                return Err(invalid("candidate changed"));
            }
            changed = store
                .record_initialization_events(
                    run,
                    status
                        .received_sequence
                        .checked_add(1)
                        .ok_or_else(|| invalid("receipt overflow"))?,
                    &batch,
                )
                .map_err(io::Error::other)?;
        }
        // Crash after receipt before apply is safe: replay the saved body before
        // requesting another page, never skip to a fresh server watermark.
        let awaiting = loop {
            let pending = store
                .initialization_next_pending_sequence(run)
                .map_err(io::Error::other)?;
            let Some(sequence) = pending else {
                if directory_race && !changed {
                    return Err(invalid("coverage race without pending receipt"));
                }
                break false;
            };
            let applied = if directory_race {
                store.apply_initialization_events_after_directory_race(run, sequence)
            } else {
                store.apply_initialization_events(run, sequence)
            };
            match applied.map_err(io::Error::other)? {
                InitializationApplyResult::AwaitingCoverage => {
                    // Do not retry the same durable batch under the Store lock.
                    // Enumerate before attempting application again.
                    break true;
                }
                InitializationApplyResult::Applied => changed = true,
                InitializationApplyResult::AlreadyApplied => {
                    return Err(invalid("candidate pending event already applied"));
                }
            }
        };
        drop(store);
        if changed {
            self.initialization.activity();
        }
        Ok(awaiting)
    }

    pub(super) fn run_initialization_owner(
        &self,
        reason: crate::initialization::RecoveryReason,
    ) -> io::Result<bool> {
        let rpc = self.connection();
        let session = self.session()?;
        self.session_pending.store(true, Ordering::Release);
        self.initialization.recovery(reason);
        let deadline = Instant::now() + Duration::from_secs(3600);
        let mut restart = false;
        let mut coverage_escapes = 0usize;
        for attempt in 1..=3 {
            let result = self.initialize_candidate(
                &rpc,
                &session,
                deadline,
                restart,
                attempt,
                &mut coverage_escapes,
            );
            match result {
                Ok(()) => return Ok(true),
                Err(error)
                    if matches!(recheck(&error), Some(Recheck::Cursor | Recheck::Filtered))
                        && attempt < 3 =>
                {
                    restart = true;
                }
                Err(error) => {
                    if !crate::connection_diagnostics::retryable(&error)
                        && self.current(&rpc).is_ok()
                    {
                        let _writes = self.recovery_gate.write().unwrap();
                        self.current(&rpc)?;
                        let mut store = self.store.lock().unwrap();
                        if let Some(status) =
                            store.initialization_status().map_err(io::Error::other)?
                        {
                            if status.state != "ready"
                                && status.binding.task_id == session.task
                                && status.binding.restore == session.restore
                            {
                                store
                                    .fail_initialization(&status.run, "initialization_rejected")
                                    .map_err(io::Error::other)?;
                            }
                        }
                    }
                    return Err(error);
                }
            }
        }
        unreachable!()
    }

    fn initialize_candidate(
        &self,
        rpc: &Arc<Rpc>,
        session: &Session,
        deadline: Instant,
        restart: bool,
        attempt: u8,
        coverage_escapes: &mut usize,
    ) -> io::Result<()> {
        let existing = self
            .store
            .lock()
            .unwrap()
            .initialization_status()
            .map_err(io::Error::other)?;
        let resume = existing.as_ref().is_some_and(|state| {
            state.state != "ready" && !restart && state.binding.restore == session.restore
        });
        let (run, resumed) = if resume {
            let _writes = self.recovery_gate.write().unwrap();
            self.current(rpc)?;
            let mut store = self.store.lock().unwrap();
            let state = existing.as_ref().unwrap();
            if state.binding.task_id != session.task
                || state.binding.scope != serde_json::to_value(&session.scope)?
            {
                return Err(invalid("candidate task or scope changed"));
            }
            (
                store
                    .resume_initialization(&state.run, &state.binding)
                    .map_err(io::Error::other)?,
                true,
            )
        } else {
            self.initialization
                .phase(crate::initialization::Phase::Watermark);
            let (cursor, root) =
                match self.initialization_read(rpc, session, Request::Seed, deadline)? {
                    Body::Seed { cursor, root } => (cursor, root),
                    _ => return Err(invalid("expected initialization seed")),
                };
            self.initialization.activity();
            let _writes = self.recovery_gate.write().unwrap();
            self.current(rpc)?;
            let mut store = self.store.lock().unwrap();
            let expected_cursor = store.cursor().map_err(io::Error::other)?;
            let binding = InitializationBinding {
                task_id: session.task.clone(),
                scope: serde_json::to_value(&session.scope)?,
                restore: session.restore.clone(),
                start_cursor: cursor,
                expected_cursor,
            };
            match store.initialization_status().map_err(io::Error::other)? {
                Some(status) if status.state != "ready" => {
                    if status.binding.task_id != session.task
                        || status.binding.scope != binding.scope
                    {
                        return Err(invalid("candidate task or scope changed"));
                    }
                    (
                        store
                            .restart_initialization(
                                &status.run,
                                &binding,
                                &root,
                                InitializationBudget::default(),
                            )
                            .map_err(io::Error::other)?,
                        false,
                    )
                }
                _ => (
                    store
                        .begin_initialization_verified_restore(
                            &binding,
                            &root,
                            InitializationBudget::default(),
                        )
                        .map_err(io::Error::other)?,
                    false,
                ),
            }
        };
        let (status, same_restore) = {
            let store = self.store.lock().unwrap();
            (
                store
                    .initialization_status()
                    .map_err(io::Error::other)?
                    .ok_or_else(|| invalid("candidate absent"))?,
                store
                    .native_scope_binding()
                    .map_err(io::Error::other)?
                    .is_some_and(|(restore, _)| restore == session.restore),
            )
        };
        if same_restore
            && !status.binding.expected_cursor.is_empty()
            && number(&status.binding.start_cursor)? < number(&status.binding.expected_cursor)?
        {
            // A low watermark is not reset evidence. Require the real native
            // reset response for the exact old committed cursor on this session.
            match self.initialization_read(
                rpc,
                session,
                Request::Pull {
                    cursor: status.binding.expected_cursor.clone(),
                },
                deadline,
            ) {
                Err(error) if matches!(recheck(&error), Some(Recheck::Cursor)) => {
                    let _writes = self.recovery_gate.write().unwrap();
                    self.current(rpc)?;
                    let mut store = self.store.lock().unwrap();
                    let base = store
                        .native_scope_binding()
                        .map_err(io::Error::other)?
                        .ok_or_else(|| invalid("native binding absent"))?;
                    if base.0 == session.restore {
                        store
                            .authorize_initialization_cursor_reset(
                                &run,
                                &status.binding.expected_cursor,
                            )
                            .map_err(io::Error::other)?;
                    }
                }
                Ok(_) => return Err(invalid("lower watermark without verified native reset")),
                Err(error) => return Err(error),
            }
        }
        if resumed {
            self.refresh_candidate_root(rpc, session, &run, deadline)?;
        }
        let mut awaiting_coverage = self.apply_candidate_events(rpc, &run, None, false)?;
        let mut directories = 0u64;
        let mut pages = 0usize;
        loop {
            // Native events, not a periodic full-tree scan. Interleave durable
            // catchup with a bounded amount of directory work.
            self.initialization
                .phase(crate::initialization::Phase::CatchingUp);
            for _ in 0..16 {
                // Do not accumulate more event pages while a received page
                // depends on directories that enumeration has not reached yet.
                if awaiting_coverage {
                    break;
                }
                let state = self
                    .store
                    .lock()
                    .unwrap()
                    .initialization_status()
                    .map_err(io::Error::other)?
                    .ok_or_else(|| invalid("candidate absent"))?;
                match self.initialization_read(
                    rpc,
                    session,
                    Request::Pull {
                        cursor: state.received_cursor,
                    },
                    deadline,
                )? {
                    Body::Events(None) => break,
                    Body::Events(Some(batch)) => {
                        pages += 1;
                        if pages > 4096 {
                            return Err(invalid("initialization event budget exceeded"));
                        }
                        awaiting_coverage =
                            self.apply_candidate_events(rpc, &run, Some(batch), false)?;
                    }
                    _ => return Err(invalid("expected candidate events")),
                }
            }
            let state = self
                .store
                .lock()
                .unwrap()
                .initialization_status()
                .map_err(io::Error::other)?
                .ok_or_else(|| invalid("candidate absent"))?;
            if state.root_needs_refresh {
                self.refresh_candidate_root(rpc, session, &run, deadline)?;
            }
            let directories_before = directories;
            let mut coverage_race = false;
            for _ in 0..4 {
                let jobs = {
                    let _writes = self.recovery_gate.write().unwrap();
                    self.current(rpc)?;
                    let mut store = self.store.lock().unwrap();
                    let nodes = store
                        .initialization_pending_directories(&run, 4)
                        .map_err(io::Error::other)?;
                    let mut jobs = Vec::new();
                    for node in nodes {
                        let path = store
                            .initialization_directory_path(&run, &node.id)
                            .map_err(io::Error::other)?;
                        let ticket = store
                            .begin_initialization_directory(&run, &node.id)
                            .map_err(io::Error::other)?;
                        jobs.push((node, path, ticket));
                    }
                    jobs
                };
                if jobs.is_empty() {
                    break;
                }
                self.initialization
                    .phase(crate::initialization::Phase::Enumerating);
                // At most four network reads; the sole owner commits receipts.
                // No Store/recovery guard is held across network waits. Events
                // are applied only between these bounded groups of requests.
                let receipts = std::thread::scope(|threads| {
                    let mut pending = Vec::new();
                    for (node, path, mut ticket) in jobs {
                        let run = &run;
                        pending.push(threads.spawn(move || {
                            let result = (|| {
                                for attempt in 0..3 {
                                    match self.initialization_read(
                                        rpc,
                                        session,
                                        Request::Directory {
                                            node: node.clone(),
                                            path: path.clone(),
                                        },
                                        deadline,
                                    ) {
                                        Err(error)
                                            if matches!(
                                                recheck(&error),
                                                Some(Recheck::Directory)
                                            ) && attempt < 2 =>
                                        {
                                            let _writes = self.recovery_gate.write().unwrap();
                                            self.current(rpc)?;
                                            ticket = self
                                                .store
                                                .lock()
                                                .unwrap()
                                                .begin_initialization_directory(run, &node.id)
                                                .map_err(io::Error::other)?;
                                        }
                                        result => return result,
                                    }
                                }
                                unreachable!()
                            })();
                            (node, ticket, result)
                        }));
                    }
                    pending
                        .into_iter()
                        .map(|thread| {
                            thread
                                .join()
                                .map_err(|_| io::Error::other("initialization read task failed"))
                        })
                        .collect::<io::Result<Vec<_>>>()
                })?;
                for (node, ticket, result) in receipts {
                    let count = {
                        let _writes = self.recovery_gate.write().unwrap();
                        self.current(rpc)?;
                        let mut store = self.store.lock().unwrap();
                        if !store
                            .initialization_directory_ticket_current(&ticket)
                            .map_err(io::Error::other)?
                        {
                            continue;
                        }
                        let body = match result {
                            Err(error)
                                if awaiting_coverage
                                    && matches!(recheck(&error), Some(Recheck::Directory)) =>
                            {
                                coverage_race = true;
                                continue;
                            }
                            other => other?,
                        };
                        let (identity, children, denied) = match body {
                            Body::Directory {
                                identity,
                                children,
                                denied,
                            } => (identity, children, denied),
                            _ => return Err(invalid("expected directory receipt")),
                        };
                        if identity != node.id || (denied && !children.is_empty()) {
                            return Err(invalid("directory receipt identity mismatch"));
                        }
                        if denied {
                            store
                                .store_initialization_denied(&ticket, 1)
                                .map_err(io::Error::other)?;
                        } else {
                            store
                                .store_initialization_directory(&ticket, 1, &children)
                                .map_err(io::Error::other)?;
                        }
                        store
                            .initialization_status()
                            .map_err(io::Error::other)?
                            .ok_or_else(|| invalid("candidate absent"))?
                            .node_count
                    };
                    directories += 1;
                    self.initialization.index(Progress {
                        attempt,
                        phase: Phase::Enumerate,
                        units: directories,
                        items: count,
                        retry_reason: None,
                    });
                }
                if coverage_race {
                    break;
                }
            }
            if coverage_race {
                // All reads in this group have joined. Invalidate after their
                // receipts so none can overwrite the newly scheduled recheck.
                if *coverage_escapes >= 16 {
                    return Err(invalid("initialization coverage race budget exceeded"));
                }
                *coverage_escapes += 1;
                awaiting_coverage = self.apply_candidate_events(rpc, &run, None, true)?;
                continue;
            }
            // A newly listed parent can resolve a deferred page. If it remains
            // missing after complete coverage, core schedules the original
            // conservative recheck; it never silently discards the event.
            awaiting_coverage = self.apply_candidate_events(rpc, &run, None, false)?;
            if awaiting_coverage {
                if directories == directories_before {
                    return Err(invalid(
                        "initialization coverage dependency made no progress",
                    ));
                }
                continue;
            }
            if !self
                .store
                .lock()
                .unwrap()
                .initialization_pending_directories(&run, 1)
                .map_err(io::Error::other)?
                .is_empty()
            {
                continue;
            }
            let ticket = {
                let _writes = self.recovery_gate.write().unwrap();
                self.current(rpc)?;
                self.store
                    .lock()
                    .unwrap()
                    .begin_initialization_fence(&run)
                    .map_err(io::Error::other)?
            };
            self.initialization
                .phase(crate::initialization::Phase::Fencing);
            match self.initialization_read(
                rpc,
                session,
                Request::Pull {
                    cursor: ticket.cursor().into(),
                },
                deadline,
            )? {
                Body::Events(Some(batch)) => {
                    awaiting_coverage =
                        self.apply_candidate_events(rpc, &run, Some(batch), false)?;
                    continue;
                }
                Body::Events(None) => {}
                _ => return Err(invalid("expected initialization fence")),
            }
            let invalidations = {
                let _writes = self.recovery_gate.write().unwrap();
                self.current(rpc)?;
                self.initialization
                    .phase(crate::initialization::Phase::Importing);
                let mut store = self.store.lock().unwrap();
                store
                    .confirm_initialization_fence(&ticket, ticket.cursor())
                    .map_err(io::Error::other)?;
                let mut invalidations = store.snapshot_invalidations().map_err(io::Error::other)?;
                store
                    .complete_initialization(&run)
                    .map_err(io::Error::other)?;
                invalidations.extend(store.snapshot_invalidations().map_err(io::Error::other)?);
                self.session_pending.store(false, Ordering::Release);
                invalidations
            };
            for (inode, parent, name) in invalidations {
                if let Some(inode) = inode {
                    let _ = self.notifications.send(Invalidation::Inode(inode));
                }
                if let Some(parent) = parent {
                    let _ = self.notifications.send(Invalidation::Entry(parent, name));
                }
            }
            self.signal_upload();
            self.initialization.activity();
            return Ok(());
        }
    }
    fn refresh_candidate_root(
        &self,
        rpc: &Arc<Rpc>,
        session: &Session,
        run: &InitializationRun,
        deadline: Instant,
    ) -> io::Result<()> {
        let ticket = {
            let _writes = self.recovery_gate.write().unwrap();
            self.current(rpc)?;
            self.store
                .lock()
                .unwrap()
                .begin_initialization_root_refresh(run)
                .map_err(io::Error::other)?
        };
        let node = match self.initialization_read(rpc, session, Request::Root, deadline)? {
            Body::Root { node } => node,
            _ => return Err(invalid("expected current root")),
        };
        let _writes = self.recovery_gate.write().unwrap();
        self.current(rpc)?;
        self.store
            .lock()
            .unwrap()
            .refresh_initialization_root(&ticket, &node)
            .map_err(io::Error::other)?;
        self.initialization.activity();
        Ok(())
    }
}

#[cfg(test)]
#[path = "initialization_owner_tests.rs"]
mod tests;

fn decode_reply(file: std::fs::File, size: u64, hash: &str) -> io::Result<InitializationReply> {
    let metadata = file.metadata()?;
    if !metadata.is_file() || metadata.len() != size {
        return Err(invalid("invalid initialization descriptor"));
    }
    use sha2::{Digest, Sha256};
    struct CheckedRead<'a> {
        file: std::io::Take<std::fs::File>,
        digest: &'a mut Sha256,
        count: &'a mut u64,
    }
    impl Read for CheckedRead<'_> {
        fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
            let count = self.file.read(buffer)?;
            self.digest.update(&buffer[..count]);
            *self.count += count as u64;
            Ok(count)
        }
    }
    let mut digest = Sha256::new();
    let mut count = 0;
    // Hash the exact bytes parsed, without retaining another entire JSON
    // allocation for every one of the four concurrent directory replies.
    let reply: InitializationReply = serde_json::from_reader(std::io::BufReader::with_capacity(
        64 * 1024,
        CheckedRead {
            file: file.take(size + 1),
            digest: &mut digest,
            count: &mut count,
        },
    ))?;
    if count != size || format!("{:x}", digest.finalize()) != hash {
        return Err(invalid("initialization payload integrity mismatch"));
    }
    Ok(reply)
}

#[cfg(test)]
mod payload_tests {
    use super::*;
    use std::io::{Seek, Write};
    fn file(bytes: &[u8]) -> std::fs::File {
        let mut file = tempfile::tempfile().unwrap();
        file.write_all(bytes).unwrap();
        file.rewind().unwrap();
        file
    }
    #[test]
    fn checked_stream_rejects_corruption_length_and_trailing_messages() {
        let data=br#"{"scope":{"view_id":1,"node_id":1,"file_id":"100","path":"/Lab"},"restore":"server","body":{"Events":null}}"#;
        assert!(decode_reply(file(data), data.len() as u64, &drive_core::sha256(data)).is_ok());
        assert!(decode_reply(file(data), data.len() as u64, &"0".repeat(64)).is_err());
        assert!(
            decode_reply(file(data), data.len() as u64 - 1, &drive_core::sha256(data)).is_err()
        );
        let extra = [data.as_slice(), b" {}"].concat();
        assert!(
            decode_reply(
                file(&extra),
                extra.len() as u64,
                &drive_core::sha256(&extra)
            )
            .is_err()
        );
        let broken = b"{broken";
        assert!(
            decode_reply(
                file(broken),
                broken.len() as u64,
                &drive_core::sha256(broken)
            )
            .is_err()
        );
    }
}
