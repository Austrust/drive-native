//! FSD-owned initialization. Bounded directory reads form a pipeline; the owner
//! alone commits their receipts and advances candidate events/checkpoints.
//! Repeatable read replies are never themselves a committed checkpoint.
use super::*;
use crate::initialization_metrics::{Counter, Stage};
#[cfg(test)]
use drive_core::Batch;
use drive_core::{
    initialization_events::InitializationEventPage,
    initialization_store::{
        InitializationApplyResult, InitializationBinding, InitializationBudget,
        InitializationDirectoryClaim, InitializationPriorityCache,
        InitializationPriorityPreparation, InitializationRun,
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
    fn initialization_store(&self) -> drive_core::shared_store::StoreGuard<'_> {
        let _span = self.initialization.work.span(Stage::StoreWait);
        self.store.lock().unwrap()
    }
    fn initialization_writer(&self) -> std::sync::RwLockWriteGuard<'_, ()> {
        let _span = self.initialization.work.span(Stage::RecoveryWait);
        self.recovery_gate.write().unwrap()
    }
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
        let _writes = self.initialization_writer();
        if !rpc.is_connected() {
            return Err(io::ErrorKind::NotConnected.into());
        }
        let mut store = self.initialization_store();
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
        self.clear_all_failures();
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
        let store = self.initialization_store();
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
        let _writes = self.initialization_writer();
        self.current(rpc)?;
        let session = self.session()?;
        let store = self.initialization_store();
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
            let store = self.initialization_store();
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
        let stage = match &request {
            Request::Seed => Stage::RpcSeed,
            Request::Root => Stage::RpcRoot,
            Request::Directory { .. } => Stage::RpcDirectory,
            Request::Pull { .. } => Stage::RpcPull,
        };
        let timer = self.initialization.work.span(stage);
        let tracker = self.initialization.clone();
        let response = rpc.call_initialization(
            request,
            timeout,
            Arc::new(move |snapshot| tracker.work.native(snapshot)),
        )?;
        drop(timer);
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
        let decode = self.initialization.work.span(Stage::PayloadDecode);
        let reply = decode_reply(std::fs::File::from(descriptor), size, &hash)?;
        if reply.scope != session.scope || reply.restore != session.restore {
            return Err(invalid("initialization response session mismatch"));
        }
        if let Body::Events(Some(page)) = &reply.body {
            if page.scope != serde_json::to_value(&session.scope).map_err(io::Error::other)?
                || page.restore != session.restore
            {
                return Err(invalid("initialization event page session mismatch"));
            }
            page.validate().map_err(io::Error::other)?;
        }
        self.current(rpc)?;
        drop(decode);
        match &reply.body {
            // No watermark advancement; this is not proof of an empty wire page.
            Body::Events(None) => self.initialization.work.count(Counter::PullNoAdvance, 1),
            Body::Events(Some(page)) => {
                self.initialization.work.count(Counter::PullPages, 1);
                self.initialization
                    .work
                    .count(Counter::PullEvents, page.batch.events.len() as u64);
            }
            _ => {}
        }
        Ok(reply.body)
    }

    fn apply_candidate_events(
        &self,
        rpc: &Arc<Rpc>,
        run: &InitializationRun,
        batch: Option<InitializationEventPage>,
        directory_race: bool,
    ) -> io::Result<bool> {
        let _writes = self.initialization_writer();
        self.current(rpc)?;
        let mut store = self.initialization_store();
        let mut changed = false;
        let mut recorded_page = None;
        if let Some(batch) = &batch {
            let _span = self.initialization.work.span(Stage::EventCollect);
            let status = store
                .initialization_status()
                .map_err(io::Error::other)?
                .ok_or_else(|| invalid("candidate absent"))?;
            if status.run != *run {
                return Err(invalid("candidate changed"));
            }
            let sequence = status
                .received_sequence
                .checked_add(1)
                .ok_or_else(|| invalid("receipt overflow"))?;
            changed = store
                .record_initialization_event_page(run, sequence, batch)
                .map_err(io::Error::other)?;
            // The receipt body was just translated and validated in this process.
            // Apply it from this page instead of hashing and parsing it again.
            recorded_page = Some((sequence, batch));
        }
        // Crash after receipt before apply is safe: replay the saved body before
        // requesting another page, never skip to a fresh server watermark.
        let awaiting = loop {
            let _span = self.initialization.work.span(Stage::EventApply);
            let pending = store
                .initialization_next_pending_sequence(run)
                .map_err(io::Error::other)?;
            let Some(sequence) = pending else {
                if directory_race && !changed {
                    return Err(invalid("coverage race without pending receipt"));
                }
                break false;
            };
            let outcome = match recorded_page {
                Some((recorded, page)) if recorded == sequence => store
                    .apply_initialization_events_page_diagnosed(run, sequence, directory_race, page)
                    .map_err(io::Error::other)?,
                _ => store
                    .apply_initialization_events_diagnosed(run, sequence, directory_race)
                    .map_err(io::Error::other)?,
            };
            self.initialization.work.event_application(&outcome);
            match outcome.result {
                InitializationApplyResult::AwaitingCoverage => {
                    self.initialization.work.count(Counter::AwaitingCoverage, 1);
                    // Do not retry the same durable batch under the Store lock.
                    // Enumerate before attempting application again.
                    break true;
                }
                InitializationApplyResult::Applied => {
                    self.initialization.work.count(Counter::AppliedPages, 1);
                    changed = true;
                }
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
                        let _writes = self.initialization_writer();
                        self.current(&rpc)?;
                        let mut store = self.initialization_store();
                        if let Some(status) =
                            store.initialization_status().map_err(io::Error::other)?
                        {
                            if status.state != "ready"
                                && status.binding.task_id == session.task
                                && status.binding.scope == serde_json::to_value(&session.scope)?
                            {
                                // Explicit native root loss/denial is evidence to revoke
                                // candidate visibility, even across a server restore. A
                                // generic failed run or transport loss is not such proof.
                                // The sticky revocation survives resume; only verified
                                // root recovery (or a newly verified seed) can clear it.
                                if is_root_unavailable(&error) {
                                    store
                                        .revoke_initialization_preview_root(&status.run)
                                        .map_err(io::Error::other)?;
                                }
                                if status.binding.restore == session.restore {
                                    store
                                        .fail_initialization(&status.run, "initialization_rejected")
                                        .map_err(io::Error::other)?;
                                }
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
            .initialization_store()
            .initialization_status()
            .map_err(io::Error::other)?;
        let resume = existing.as_ref().is_some_and(|state| {
            state.state != "ready" && !restart && state.binding.restore == session.restore
        });
        let (run, resumed) = if resume {
            let _writes = self.initialization_writer();
            self.current(rpc)?;
            let mut store = self.initialization_store();
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
            let _writes = self.initialization_writer();
            self.current(rpc)?;
            let mut store = self.initialization_store();
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
            let store = self.initialization_store();
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
                    let _writes = self.initialization_writer();
                    self.current(rpc)?;
                    let mut store = self.initialization_store();
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
            self.initialization.work.count(Counter::Resumed, 1);
            self.refresh_candidate_root(rpc, session, &run, deadline)?;
        } else {
            self.initialization.work.count(Counter::Restarted, 1);
        }
        let mut awaiting_coverage = self.apply_candidate_events(rpc, &run, None, false)?;
        let mut priority_cache = InitializationPriorityCache::default();
        let mut directories = 0u64;
        let mut pages = 0usize;
        loop {
            if self.initialization.work.queue_due() {
                let store = self.initialization_store();
                let _span = self.initialization.work.span(Stage::QueueSample);
                self.initialization.work.queue(
                    store
                        .initialization_queue_summary(&run)
                        .map_err(io::Error::other)?,
                );
            }
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
                    .initialization_store()
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
                .initialization_store()
                .initialization_status()
                .map_err(io::Error::other)?
                .ok_or_else(|| invalid("candidate absent"))?;
            if state.root_needs_refresh {
                self.refresh_candidate_root(rpc, session, &run, deadline)?;
            }
            let directories_before = directories;
            let priority = if awaiting_coverage {
                self.initialization_priority_job(rpc, &run, &mut priority_cache)?
            } else {
                priority_cache = InitializationPriorityCache::default();
                None
            };
            let coverage_race = self.initialization_directory_window_with_priority(
                rpc,
                session,
                &run,
                deadline,
                awaiting_coverage,
                attempt,
                &mut directories,
                priority,
            )?;
            if coverage_race {
                // All reads in this group have joined. Invalidate after their
                // receipts so none can overwrite the newly scheduled recheck.
                if *coverage_escapes >= 16 {
                    return Err(invalid("initialization coverage race budget exceeded"));
                }
                *coverage_escapes += 1;
                self.initialization.work.count(Counter::CoverageRace, 1);
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
                .initialization_store()
                .initialization_pending_directories(&run, 1)
                .map_err(io::Error::other)?
                .is_empty()
            {
                continue;
            }
            self.initialization
                .phase(crate::initialization::Phase::Fencing);
            let ticket = {
                self.initialization.work.count(Counter::FenceAttempts, 1);
                let _writes = self.initialization_writer();
                self.current(rpc)?;
                let _span = self.initialization.work.span(Stage::FinalCheck);
                self.initialization_store()
                    .begin_initialization_fence(&run)
                    .map_err(io::Error::other)?
            };
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
                let _writes = self.initialization_writer();
                self.current(rpc)?;
                self.initialization
                    .phase(crate::initialization::Phase::Importing);
                let mut store = self.initialization_store();
                let _span = self.initialization.work.span(Stage::FinalCommit);
                store
                    .confirm_initialization_fence(&ticket, ticket.cursor())
                    .map_err(io::Error::other)?;
                let mut invalidations = store.snapshot_invalidations().map_err(io::Error::other)?;
                store
                    .complete_initialization(&run)
                    .map_err(io::Error::other)?;
                invalidations.extend(store.snapshot_invalidations().map_err(io::Error::other)?);
                self.session_pending.store(false, Ordering::Release);
                self.initialization.work.count(Counter::Checkpoints, 1);
                invalidations
            };
            // The commit replaced the whole namespace; publish one generation
            // invalidation instead of per-node messages that cannot fit the
            // bounded notification queue (~2 per node for 210k/294k nodes).
            self.enqueue_snapshot_invalidations(invalidations);
            self.signal_upload();
            self.initialization.activity();
            return Ok(());
        }
    }
    /// Four network reads, as before, but completed receipts commit immediately
    /// and refill the freed slot. Each event window admits at most sixteen jobs.
    /// Network waits never hold the Store; events still apply between windows.
    #[cfg(test)]
    fn initialization_directory_window(
        &self,
        rpc: &Arc<Rpc>,
        session: &Session,
        run: &InitializationRun,
        deadline: Instant,
        awaiting_coverage: bool,
        attempt: u8,
        directories: &mut u64,
    ) -> io::Result<bool> {
        self.initialization_directory_window_with_priority(
            rpc,
            session,
            run,
            deadline,
            awaiting_coverage,
            attempt,
            directories,
            None,
        )
    }

    fn initialization_priority_job(
        &self,
        rpc: &Arc<Rpc>,
        run: &InitializationRun,
        cache: &mut InitializationPriorityCache,
    ) -> io::Result<Option<InitializationDirectoryClaim>> {
        let _writes = self.initialization_writer();
        self.current(rpc)?;
        let mut store = self.initialization_store();
        let prepared = {
            let _span = self.initialization.work.span(Stage::PriorityPrepare);
            store
                .prepare_initialization_priority(run, cache)
                .map_err(io::Error::other)?
        };
        match prepared {
            InitializationPriorityPreparation::Prepared => {
                self.initialization.work.count(Counter::PriorityPrepared, 1)
            }
            InitializationPriorityPreparation::Reused => {
                self.initialization.work.count(Counter::PriorityReused, 1)
            }
            InitializationPriorityPreparation::Unavailable => {
                self.initialization.work.count(Counter::PrioritySkipped, 1);
                return Ok(None);
            }
        }
        let _span = self.initialization.work.span(Stage::PriorityClaim);
        let claim = store
            .claim_initialization_priority(run, cache)
            .map_err(io::Error::other)?;
        self.initialization.work.count(
            if claim.is_some() {
                Counter::PriorityClaimed
            } else {
                Counter::PrioritySkipped
            },
            1,
        );
        Ok(claim)
    }

    fn initialization_directory_window_with_priority(
        &self,
        rpc: &Arc<Rpc>,
        session: &Session,
        run: &InitializationRun,
        deadline: Instant,
        awaiting_coverage: bool,
        attempt: u8,
        directories: &mut u64,
        mut priority: Option<InitializationDirectoryClaim>,
    ) -> io::Result<bool> {
        use drive_core::initialization_store::InitializationDirectoryClaim;
        let mut coverage_race = false;
        let mut first_error = None;
        std::thread::scope(|threads| {
            let (tx, rx) = std::sync::mpsc::channel();
            let mut in_flight = 0usize;
            let mut admitted = 0usize;
            let mut handles = Vec::new();
            loop {
                let mut completed = None;
                if first_error.is_none() && !coverage_race && admitted < 16 && in_flight < 4 {
                    let mut after = None;
                    // Bound each lock hold and yield between indexed pages. An
                    // exhausted scan is not coverage: the existing fence checks
                    // every directory after the window has drained.
                    loop {
                        // Commit ready receipts before scanning another page.
                        // The following outer iteration then starts at the first
                        // key because that commit may change candidate ancestry.
                        if let Ok(receipt) = rx.try_recv() {
                            completed = Some(receipt);
                            break;
                        }
                        if Instant::now() >= deadline {
                            first_error = Some(io::Error::new(
                                io::ErrorKind::TimedOut,
                                "initialization work budget exhausted",
                            ));
                            break;
                        }
                        let priority_page = priority.is_some();
                        let page = if let Some(job) = priority.take() {
                            Ok(drive_core::initialization_store::InitializationClaimPage {
                                jobs: vec![job],
                                next: None,
                                stats: Default::default(),
                            })
                        } else {
                            (|| {
                                let _writes = self.initialization_writer();
                                self.current(rpc)?;
                                let mut store = self.initialization_store();
                                let _span = self.initialization.work.span(Stage::Scheduling);
                                store
                                    .claim_initialization_directories(
                                        run,
                                        (4 - in_flight).min(16 - admitted),
                                        after.as_ref(),
                                    )
                                    .map_err(io::Error::other)
                            })()
                        };
                        let page = match page {
                            Ok(page) => page,
                            Err(error) => {
                                first_error = Some(error);
                                break;
                            }
                        };
                        let work = &self.initialization.work;
                        if !priority_page {
                            work.count(Counter::ClaimPages, 1);
                        }
                        work.count(Counter::ClaimExamined, page.stats.examined as u64);
                        work.count(
                            Counter::ClaimAncestorPromotions,
                            page.stats.ancestor_promotions as u64,
                        );
                        work.count(Counter::ClaimCoalesced, page.stats.coalesced as u64);
                        work.count(
                            Counter::ClaimBlockedLoading,
                            page.stats.blocked_loading as u64,
                        );
                        work.count(
                            Counter::ClaimPreviouslyClaimed,
                            page.stats.previously_claimed as u64,
                        );
                        if !priority_page {
                            work.count(
                                Counter::ClaimNew,
                                page.jobs
                                    .len()
                                    .saturating_sub(page.stats.previously_claimed)
                                    as u64,
                            );
                        }
                        let found = !page.jobs.is_empty();
                        if found {
                            self.initialization
                                .phase(crate::initialization::Phase::Enumerating);
                        }
                        for job in page.jobs {
                            let tx = tx.clone();
                            admitted += 1;
                            in_flight += 1;
                            handles.push(threads.spawn(move || {
                                let InitializationDirectoryClaim {
                                    node,
                                    path,
                                    mut ticket,
                                } = job;
                                let result =
                                    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                                        for retry in 0..3 {
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
                                                    ) && retry < 2 =>
                                                {
                                                    let _writes = self.initialization_writer();
                                                    self.current(rpc)?;
                                                    let mut store = self.initialization_store();
                                                    // Another completed listing may have moved,
                                                    // replaced or revoked this subtree while this
                                                    // request was in flight. Never issue a fresh
                                                    // ticket for the new identity using the old
                                                    // captured node/path. The coordinator will
                                                    // discard this old ticket and schedule the
                                                    // current candidate independently.
                                                    if !store
                                                        .initialization_directory_ticket_current(
                                                            &ticket,
                                                        )
                                                        .map_err(io::Error::other)?
                                                    {
                                                        return Err(error);
                                                    }
                                                    ticket = store
                                                        .begin_initialization_directory(
                                                            run, &node.id,
                                                        )
                                                        .map_err(io::Error::other)?;
                                                    self.initialization
                                                        .work
                                                        .count(Counter::DirectoryRetry, 1);
                                                }
                                                result => return result,
                                            }
                                        }
                                        unreachable!()
                                    }))
                                    .unwrap_or_else(|_| {
                                        Err(io::Error::other("initialization read task failed"))
                                    });
                                let _ = tx.send((node, ticket, result));
                            }));
                        }
                        if in_flight == 4
                            || admitted == 16
                            || (page.next.is_none() && !priority_page)
                        {
                            break;
                        }
                        after = page.next;
                        if !priority_page {
                            work.count(Counter::ClaimScanYields, 1);
                        }
                        // Issuing claims changes loading/generation, not paths.
                        // Keep the keyset hint while filling vacant slots past
                        // a blocked prefix; never carry it across a receipt.
                        std::thread::yield_now();
                    }
                }
                if in_flight == 0 {
                    break;
                }
                let (node, ticket, result) = match completed.map(Ok).unwrap_or_else(|| rx.recv()) {
                    Ok(result) => result,
                    Err(_) => {
                        first_error =
                            Some(io::Error::other("initialization completion channel closed"));
                        break;
                    }
                };
                in_flight -= 1;
                let committed = (|| {
                    let _writes = self.initialization_writer();
                    self.current(rpc)?;
                    let mut store = self.initialization_store();
                    // Every receipt is persisted inside this serialized section.
                    // Count the rounds so a large tree's commit pipeline is
                    // distinguishable from its read concurrency without logs.
                    self.initialization
                        .work
                        .count(Counter::DirectoryCommitRounds, 1);
                    let _span = self.initialization.work.span(Stage::DirectoryCommit);
                    if !store
                        .initialization_directory_ticket_current(&ticket)
                        .map_err(io::Error::other)?
                    {
                        self.initialization.work.count(Counter::StaleReceipt, 1);
                        return Ok(None);
                    }
                    let body = match result {
                        Err(error)
                            if awaiting_coverage
                                && matches!(recheck(&error), Some(Recheck::Directory)) =>
                        {
                            coverage_race = true;
                            return Ok(None);
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
                        self.initialization.work.count(Counter::DirectoryDenied, 1);
                    } else {
                        store
                            .store_initialization_native_directory(&ticket, 1, &children)
                            .map_err(io::Error::other)?;
                    }
                    self.initialization
                        .work
                        .count(Counter::DirectoryCommitted, 1);
                    Ok(Some(
                        store
                            .initialization_status()
                            .map_err(io::Error::other)?
                            .ok_or_else(|| invalid("candidate absent"))?
                            .node_count,
                    ))
                })();
                match committed {
                    Ok(Some(count)) => {
                        *directories += 1;
                        self.initialization.index(Progress {
                            attempt,
                            phase: Phase::Enumerate,
                            units: *directories,
                            items: count,
                            retry_reason: None,
                        });
                    }
                    Ok(None) => {}
                    Err(error) => {
                        if first_error.is_none() {
                            first_error = Some(error);
                        }
                    }
                }
                // On one failed read, drain and persist other verified receipts
                // before reconnecting. Never discard their completed work.
            }
            for handle in handles {
                if handle.join().is_err() && first_error.is_none() {
                    first_error = Some(io::Error::other("initialization read task failed"));
                }
            }
        });
        if let Some(error) = first_error {
            Err(error)
        } else {
            Ok(coverage_race)
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
            let _writes = self.initialization_writer();
            self.current(rpc)?;
            self.initialization_store()
                .begin_initialization_root_refresh(run)
                .map_err(io::Error::other)?
        };
        let node = match self.initialization_read(rpc, session, Request::Root, deadline)? {
            Body::Root { node } => node,
            _ => return Err(invalid("expected current root")),
        };
        let _writes = self.initialization_writer();
        self.current(rpc)?;
        let _span = self.initialization.work.span(Stage::RootCommit);
        self.initialization_store()
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
