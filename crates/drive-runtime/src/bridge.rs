use crate::background::{BackgroundFailure, BackgroundSource};
use crate::{rpc::Rpc, wire::*};
#[cfg(test)]
use drive_core::Store;
use drive_core::download_jobs::DownloadFailureKind;
use drive_core::{Change, DownloadTicket, Node, ReadHandle};
use std::{
    collections::HashMap,
    io,
    os::{fd::OwnedFd, unix::net::UnixStream},
    path::Path,
    process::{Child, Command as Process, Stdio},
    sync::{
        Arc, Condvar, Mutex,
        atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering},
    },
    time::{Duration, Instant},
};
pub struct WorkerProcess {
    pub rpc: Arc<Rpc>,
    child: Child,
}
impl WorkerProcess {
    pub fn start(executable: &Path) -> io::Result<Self> {
        let (a, b) = UnixStream::pair()?;
        let fd: OwnedFd = b.into();
        let mut child = Process::new(executable)
            .arg("--ipc-stdin")
            .stdin(Stdio::from(fd))
            .stdout(Stdio::null())
            .stderr(Stdio::inherit())
            .spawn()?;
        match Rpc::new(a) {
            Ok(rpc) => Ok(Self {
                rpc: Arc::new(rpc),
                child,
            }),
            Err(error) => {
                let _ = child.kill();
                let _ = child.wait();
                Err(error)
            }
        }
    }
    pub(crate) fn before_cleanup(&mut self) -> crate::connection_diagnostics::ProcessState {
        use crate::connection_diagnostics::ProcessState;
        use std::os::unix::process::ExitStatusExt;
        match self.child.try_wait() {
            Ok(None) => ProcessState::Running,
            Ok(Some(status)) => ProcessState::Exited {
                code: status.code(),
                signal: status.signal(),
            },
            Err(_) => ProcessState::QueryFailed,
        }
    }
    pub fn stop(&mut self) {
        self.rpc.close();
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
    pub fn pid(&self) -> u32 {
        self.child.id()
    }
}
impl Drop for WorkerProcess {
    fn drop(&mut self) {
        self.stop();
    }
}
enum Invalidation {
    Inode(u64),
    Entry(u64, String),
    /// The whole committed namespace changed (a snapshot generation): one
    /// message replaces a per-node invalidation list that cannot fit the queue.
    All,
}
#[derive(Default)]
struct Demand {
    readers: Vec<drive_fs::ReadCancellation>,
    keep: bool,
    foreground: bool,
    closed: bool,
}
struct Flight {
    demand: Mutex<Demand>,
    done: Mutex<Option<Result<(), i32>>>,
    changed: Condvar,
}
/// Outcome of one upload pass. `Gated` means the pass returned before it
/// inspected the queue, so it is not evidence that the queue recovered.
enum UploadPass {
    Gated,
    Ran(usize),
}
/// Result of one upload cycle: `ran` is false when every pass was gated and
/// `confirmed` counts the operations the server confirmed in this cycle.
struct CycleOutcome {
    ran: bool,
    confirmed: usize,
}
/// Idle poll interval of the upload worker. Signals and due retry deadlines
/// wake the worker earlier.
const UPLOAD_WAIT_INTERVAL: Duration = Duration::from_secs(1);
/// Upper bound on how long a hydration refresh may be deferred while uploads
/// keep progressing. `refresh_cached()` enumerates durable download intent
/// (pinned, stale, retryable) via `downloads_needed()`, so running it after
/// every confirmed upload made each write pay that scan. Deferring it keeps
/// the intent durable and bounds its worst-case latency.
const UPLOAD_REFRESH_MAX_INTERVAL: Duration = Duration::from_secs(5);
/// How often the idle upload tick prunes durable bookkeeping (superseded
/// operations, terminal namespace history, applied inbox receipts). Startup and
/// operation commits already bound the steady state; this only ages out rows a
/// long-running mount would otherwise keep until the next restart. Pruning
/// never publishes a semantic journal change, so it cannot wake the UI.
const STORAGE_PRUNE_INTERVAL: Duration = Duration::from_secs(300);
pub struct Bridge {
    pub store: Arc<drive_core::SharedStore>,
    recovery_gate: std::sync::RwLock<()>,
    event_gate: Mutex<()>,
    connection: Mutex<Arc<Rpc>>,
    initialization_task: Mutex<Option<String>>,
    initialization_session: Mutex<Option<initialization_owner::Session>>,
    session_pending: AtomicBool,
    flights: Mutex<HashMap<(String, String), Arc<Flight>>>,
    download_retry_clock: Mutex<crate::download_retry_clock::RetryClock>,
    /// Wakes flights waiting for a retry deadline when another flight
    /// publishes or clears a retry gate (#31).
    retry_wake: (Mutex<()>, Condvar),
    notify_hub: Arc<Mutex<Option<drive_fs::NotifyHub>>>,
    notifications: std::sync::mpsc::SyncSender<Invalidation>,
    /// Committed invalidations rejected by the bounded notification queue.
    /// Non-zero means the mount suspended cacheable publication for one quiet
    /// lease window and then resumed; it never needs a restart (#27).
    notification_overflows: AtomicU64,
    pub downloads: AtomicUsize,
    pub download_failures_seen: AtomicUsize,
    pub last_download_failure: Mutex<Option<(DownloadFailureKind, Option<u32>)>>,
    upload_signal: (Mutex<bool>, Condvar),
    recheck_requested: AtomicBool,
    /// Test-only observation of how many hydration refreshes ran.
    #[cfg(test)]
    refresh_runs: AtomicUsize,
    failure: Mutex<Option<BackgroundFailure>>,
    /// Last concrete reason a pending upload could not start. Unlike
    /// `failure` this is resolved state, cleared as soon as the queue is
    /// startable again, so it can name the blocking path/operation.
    pub upload_block: Mutex<Option<String>>,
    pub cache_error: Mutex<Option<String>>,
    pub initialization: Arc<crate::initialization::Tracker>,
    /// Issue #50: task-scoped exclusion patterns compiled once at profile load.
    /// Empty rules keep matching a constant-time no-op.
    exclusion: std::sync::RwLock<drive_core::exclude::ExcludeRules>,
}
impl Bridge {
    pub fn transfer_entries(&self) -> Vec<crate::transfers::TransferEntry> {
        self.connection().transfers.entries()
    }
    pub fn transfer_revision(&self) -> u64 {
        self.connection().transfers.revision()
    }

    fn connection(&self) -> Arc<Rpc> {
        self.connection.lock().unwrap().clone()
    }
    /// Stop/join the prior event loop before attaching. The mount, handles and
    /// SQLite writer survive. A changed restore requires verified initialization;
    /// a different selected scope is rejected.
    pub fn reconnect(&self, rpc: Arc<Rpc>, login: Login) -> io::Result<()> {
        let scope = serde_json::to_string(&login.scope)?;
        let restore = Self::authenticate(&rpc, login)?;
        self.attach(rpc, &restore, &scope)
    }
    /// Attach under the recovery gate, outside the supervisor control lock so
    /// pause/stop can interrupt network work. The supervisor rechecks generation.
    pub fn attach(&self, rpc: Arc<Rpc>, restore: &str, scope: &str) -> io::Result<()> {
        if self.initialization_task.lock().unwrap().is_some() {
            return self.attach_initialization_session(rpc, restore, scope);
        }
        // Initialization owners must take the write side before beginning or
        // completing a durable candidate. Keep the same gate -> Store ordering
        // as mutation dispatch, including while publishing the new connection.
        let _mutation = self.recovery_gate.read().unwrap();
        let blocked = {
            let mut store = self.store.lock().unwrap();
            store
                .bind_native_scope(restore, scope)
                .map_err(io::Error::other)?;
            drive_core::write_permissions::reset_session(&store).map_err(io::Error::other)?;
            store
                .initialization_writeback_blocked()
                .map_err(io::Error::other)?
        };
        let old = std::mem::replace(&mut *self.connection.lock().unwrap(), rpc);
        old.close();
        self.clear_all_failures();
        if !blocked {
            self.signal_upload();
        }
        Ok(())
    }

    pub fn connect(
        store: Arc<drive_core::SharedStore>,
        rpc: Arc<Rpc>,
        login: Login,
    ) -> io::Result<Self> {
        let scope = serde_json::to_string(&login.scope)?;
        let restore = Self::authenticate(&rpc, login)?;
        store
            .lock()
            .unwrap()
            .bind_native_scope(&restore, &scope)
            .map_err(io::Error::other)?;
        Self::from_connection(store, rpc)
    }
    pub fn authenticate(rpc: &Rpc, login: Login) -> io::Result<String> {
        let result = rpc.call(Command::Connect(login), Duration::from_secs(60))?;
        match result.body {
            Response::Connected { restore } => Ok(restore),
            Response::Failed { category } => Err(io::Error::new(
                match category.as_str() {
                    "transport" => io::ErrorKind::NotConnected,
                    "tls_or_configuration" => io::ErrorKind::InvalidInput,
                    _ => io::ErrorKind::PermissionDenied,
                },
                "native login was not accepted",
            )),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid login response",
            )),
        }
    }
    /// Cached files remain mountable before credentials or a network exist.
    pub fn offline(store: Arc<drive_core::SharedStore>) -> io::Result<Self> {
        let (a, b) = UnixStream::pair()?;
        let rpc = Arc::new(Rpc::new(a)?);
        rpc.close();
        drop(b);
        Self::from_connection(store, rpc)
    }
    pub fn disconnect(&self) {
        self.connection().close();
    }
    pub fn active_downloads(&self) -> Vec<String> {
        self.flights
            .lock()
            .unwrap()
            .keys()
            .map(|(id, _)| id.clone())
            .collect()
    }
    pub fn cache_usage(&self) -> io::Result<drive_core::cache_policy::CacheUsage> {
        let active = self.active_downloads().into_iter().collect();
        self.store
            .lock()
            .unwrap()
            .cache_usage(unix_seconds(), &active)
            .map_err(io::Error::other)
    }
    pub fn maintain_cache(&self) -> io::Result<drive_core::cache_policy::CacheTrim> {
        let flights = self.flights.lock().unwrap();
        let active = flights.keys().map(|(id, _)| id.clone()).collect();
        let (result, inodes) = {
            let mut store = self.store.lock().unwrap();
            let result = store
                .trim_cache(unix_seconds(), &active)
                .map_err(io::Error::other)?;
            let inodes = result
                .released
                .iter()
                .filter_map(|id| store.record(id).ok().map(|r| r.ino))
                .collect::<Vec<_>>();
            (result, inodes)
        };
        drop(flights);
        for ino in inodes {
            self.enqueue_invalidation(Invalidation::Inode(ino));
        }
        Ok(result)
    }
    pub fn release_content(&self, id: &str) -> io::Result<()> {
        let flights = self.flights.lock().unwrap();
        if flights.keys().any(|(active, _)| active == id) {
            return Err(io::Error::other("file is being downloaded"));
        }
        let mut store = self.store.lock().unwrap();
        let ino = store.record(id).map_err(io::Error::other)?.ino;
        store.release(id).map_err(io::Error::other)?;
        drop(store);
        drop(flights);
        self.enqueue_invalidation(Invalidation::Inode(ino));
        Ok(())
    }
    fn from_connection(store: Arc<drive_core::SharedStore>, rpc: Arc<Rpc>) -> io::Result<Self> {
        let notify_hub: Arc<Mutex<Option<drive_fs::NotifyHub>>> = Arc::new(Mutex::new(None));
        let target = notify_hub.clone();
        let (notifications, rx) = std::sync::mpsc::sync_channel(1024);
        std::thread::spawn(move || {
            for message in rx {
                // The hub owns the publication gate, the kernel notifier and the
                // degraded/recovery fallback; delivery order is decided there.
                let Some(hub) = target.lock().unwrap().clone() else {
                    continue;
                };
                match message {
                    Invalidation::Inode(ino) => hub.invalidate_inode(ino),
                    Invalidation::Entry(parent, name) => hub.invalidate_entry(parent, &name),
                    Invalidation::All => hub.invalidate_all(),
                }
            }
        });
        Ok(Self {
            store,
            recovery_gate: std::sync::RwLock::new(()),
            event_gate: Mutex::new(()),
            initialization_task: Mutex::new(None),
            initialization_session: Mutex::new(None),
            session_pending: AtomicBool::new(false),
            connection: Mutex::new(rpc),
            flights: Mutex::new(HashMap::new()),
            download_retry_clock: Mutex::new(crate::download_retry_clock::RetryClock::default()),
            retry_wake: (Mutex::new(()), Condvar::new()),
            notify_hub,
            notifications,
            notification_overflows: AtomicU64::new(0),
            downloads: AtomicUsize::new(0),
            download_failures_seen: AtomicUsize::new(0),
            last_download_failure: Mutex::new(None),
            upload_signal: (Mutex::new(false), Condvar::new()),
            recheck_requested: AtomicBool::new(false),
            #[cfg(test)]
            refresh_runs: AtomicUsize::new(0),
            failure: Mutex::new(None),
            upload_block: Mutex::new(None),
            cache_error: Mutex::new(None),
            initialization: Arc::new(crate::initialization::Tracker::default()),
            exclusion: std::sync::RwLock::new(drive_core::exclude::ExcludeRules::default()),
        })
    }
    /// Every upstream queue failure must suspend cacheable publication, not
    /// silently lose a committed mutation before it reaches the mount's hub.
    fn enqueue_invalidation(&self, invalidation: Invalidation) {
        if !enqueue_invalidation(&self.notifications, &self.notify_hub, invalidation) {
            self.notification_overflows.fetch_add(1, Ordering::Relaxed);
        }
    }
    /// Committed invalidations dropped by the bounded notification queue.
    /// Observable for root diagnostics; the mount recovers without a restart.
    pub fn notification_overflows(&self) -> u64 {
        self.notification_overflows.load(Ordering::Relaxed)
    }
    /// Install the profile's exclusion rules. Must be called before
    /// initialization enumerates the namespace (see `Service::new`, #50).
    pub(crate) fn set_exclude_rules(&self, rules: drive_core::exclude::ExcludeRules) {
        *self.exclusion.write().unwrap() = rules;
    }
    /// Snapshot the current rules so a worker can match without holding a lock.
    pub(crate) fn exclude_rules(&self) -> drive_core::exclude::ExcludeRules {
        self.exclusion.read().unwrap().clone()
    }
    /// Above this many per-node invalidations one generation message is cheaper
    /// and bounded: a snapshot commit touches every node, and one message per
    /// inode/entry cannot fit the bounded delivery queue (210k nodes ≈ 420k
    /// messages).
    const SNAPSHOT_GENERATION_THRESHOLD: usize = 64;
    /// Publish the invalidation set of a whole-snapshot commit (first install or
    /// recovery rebuild). Small sets keep precise per-node messages; a large
    /// set collapses into one generation invalidation so the bounded channel
    /// cannot lose any part of a commit that already replaced the namespace.
    fn enqueue_snapshot_invalidations(
        &self,
        invalidations: Vec<(Option<u64>, Option<u64>, String)>,
    ) {
        let messages = invalidations
            .iter()
            .map(|(inode, parent, _)| usize::from(inode.is_some()) + usize::from(parent.is_some()))
            .sum::<usize>();
        if messages > Self::SNAPSHOT_GENERATION_THRESHOLD {
            self.enqueue_invalidation(Invalidation::All);
            return;
        }
        for (inode, parent, name) in invalidations {
            if let Some(inode) = inode {
                self.enqueue_invalidation(Invalidation::Inode(inode));
            }
            if let Some(parent) = parent {
                self.enqueue_invalidation(Invalidation::Entry(parent, name));
            }
        }
    }
    /// Stop publishing cacheable metadata once background producers stop.
    pub fn clear_notifier(&self) {
        if let Some(hub) = self.notify_hub.lock().unwrap().take() {
            hub.deactivate();
        }
    }
    /// Compatibility for mounts that do not share the FUSE filesystem's hub:
    /// caching stays disabled because the filesystem hub is never installed.
    pub fn set_notifier(&self, notifier: fuser::Notifier) {
        let hub = drive_fs::NotifyHub::from_env();
        hub.install(notifier);
        *self.notify_hub.lock().unwrap() = Some(hub);
    }
    /// Share the mount's hub so invalidation delivery is ordered against the
    /// FUSE replies that publish cached metadata.
    pub fn set_notify_hub(&self, hub: drive_fs::NotifyHub) {
        *self.notify_hub.lock().unwrap() = Some(hub);
    }
    /// Resume persistent candidates before ordinary events. Recheck quarantined
    /// permissions at explicit session boundaries; transport retries otherwise
    /// continue from the committed cursor.
    pub fn initial_pull(&self) -> io::Result<bool> {
        if self.initialization_task.lock().unwrap().is_some() {
            return self.owner_initial_pull();
        }
        let denied = self
            .store
            .lock()
            .unwrap()
            .inaccessible_count()
            .map_err(io::Error::other)?;
        if denied > 0 {
            let _event = self.event_gate.lock().unwrap();
            self.rebuild_snapshot(crate::initialization::RecoveryReason::PermissionRecheck)
        } else {
            self.pull()
        }
    }
    pub fn pull(&self) -> io::Result<bool> {
        self.pull_after_notification(false)
    }
    fn pull_after_notification(&self, changed_notification: bool) -> io::Result<bool> {
        self.initialization
            .phase(crate::initialization::Phase::CatchingUp);
        let _event = self.event_gate.lock().unwrap();
        if self.owner_candidate_pending()? {
            return self
                .run_initialization_owner(crate::initialization::RecoveryReason::InitialIndex);
        }
        let cursor = self
            .store
            .lock()
            .unwrap()
            .cursor()
            .map_err(io::Error::other)?;
        if cursor.is_empty() {
            return self.rebuild_snapshot(crate::initialization::RecoveryReason::InitialIndex);
        }
        let rpc = self.connection();
        let response = rpc.call(Command::Pull { cursor }, Duration::from_secs(60))?;
        // Attach takes the writer. Only the current session may commit this
        // reply; never hold this guard while waiting for the network.
        let receipt = self.recovery_gate.read().unwrap();
        if self.initialization_task.lock().unwrap().is_some() {
            self.current(&rpc)?;
        }
        let batch = match response.body {
            Response::Events(batch) => batch,
            Response::Failed { category }
                if matches!(
                    category.as_str(),
                    "native_cursor_reset" | "native_filtered_type_changed"
                ) =>
            {
                drop(receipt);
                return self.rebuild_snapshot(if category == "native_cursor_reset" {
                    crate::initialization::RecoveryReason::CursorInvalidated
                } else {
                    crate::initialization::RecoveryReason::FilteredTypeChanged
                });
            }
            body => return Err(operation_error(body)),
        };
        let Some(batch) = batch else {
            self.initialization.pulled(false, 0, changed_notification);
            drop(receipt);
            self.finish_session_catchup(&rpc)?;
            return Ok(false);
        };
        let event_count = batch.events.len();
        let mut invalidations = Vec::new();
        let mut invalidated_inodes = Vec::new();
        let mut store = self.store.lock().unwrap();
        for event in &batch.events {
            let id = match &event.change {
                Change::Upsert(n) => &n.id,
                Change::Delete { id } => id,
            };
            if let Ok(old) = store.local_identity(id).and_then(|id| store.record(&id)) {
                // The old name, the changed object's attributes and the old
                // parent directory's attributes are all possibly cached.
                invalidated_inodes.push(old.ino);
                if let Some(p) = old.node.parent.as_deref() {
                    if let Ok(p) = store.local_identity(p).and_then(|p| store.record(&p)) {
                        invalidated_inodes.push(p.ino);
                        invalidations.push((p.ino, old.node.name));
                    }
                }
            }
        }
        store.receive(&batch).map_err(io::Error::other)?;
        store.drain().map_err(io::Error::other)?;
        for event in &batch.events {
            match &event.change {
                Change::Upsert(n) => {
                    if let Ok(record) = store.record(&n.id) {
                        invalidated_inodes.push(record.ino);
                    }
                    if let Some(p) = n.parent.as_deref() {
                        if let Ok(p) = store.local_identity(p).and_then(|p| store.record(&p)) {
                            invalidated_inodes.push(p.ino);
                            invalidations.push((p.ino, n.name.clone()));
                        }
                    }
                }
                Change::Delete { id } => {
                    if let Ok(record) = store.record(id) {
                        invalidated_inodes.push(record.ino);
                    }
                }
            }
        }
        drop(store);
        drop(receipt);
        self.initialization
            .pulled(true, event_count, changed_notification);
        for ino in invalidated_inodes {
            self.enqueue_invalidation(Invalidation::Inode(ino));
        }
        for (p, name) in invalidations {
            self.enqueue_invalidation(Invalidation::Entry(p, name));
        }
        self.signal_upload();
        self.finish_session_catchup(&rpc)?;
        Ok(true)
    }
    fn rebuild_snapshot(&self, reason: crate::initialization::RecoveryReason) -> io::Result<bool> {
        if self.initialization_task.lock().unwrap().is_some() {
            return self.run_initialization_owner(reason);
        }
        use std::io::Read;
        self.initialization.recovery(reason);
        let _mutations = self.recovery_gate.write().unwrap();
        let cursor = self
            .store
            .lock()
            .unwrap()
            .cursor()
            .map_err(io::Error::other)?;
        let rpc = self.connection();
        let progress = self.initialization.clone();
        let response = rpc.call_index(
            Duration::from_secs(3600),
            Arc::new(move |p| progress.index(p)),
        )?;
        let (size, hash) = match response.body {
            Response::RecoverySnapshot { size, sha256 } if size <= 512 * 1024 * 1024 => {
                (size, sha256)
            }
            body => return Err(operation_error(body)),
        };
        self.initialization
            .phase(crate::initialization::Phase::Importing);
        let descriptor = response
            .descriptor
            .ok_or_else(|| io::Error::other("snapshot descriptor absent"))?;
        let file = std::fs::File::from(descriptor);
        let metadata = file.metadata()?;
        if !metadata.is_file() || metadata.len() != size {
            return Err(io::Error::other("invalid snapshot file"));
        }
        let mut bytes = Vec::new();
        file.take(size + 1).read_to_end(&mut bytes)?;
        if bytes.len() as u64 != size || drive_core::sha256(&bytes) != hash {
            return Err(io::Error::other("snapshot integrity mismatch"));
        }
        let snapshot: drive_core::native::RemoteSnapshot = serde_json::from_slice(&bytes)?;
        if !rpc.is_connected() || !Arc::ptr_eq(&rpc, &self.connection()) {
            return Err(io::ErrorKind::NotConnected.into());
        }
        let invalidations = {
            let mut store = self.store.lock().unwrap();
            let mut invalidations = store.snapshot_invalidations().map_err(io::Error::other)?;
            store
                .install_native_snapshot(&snapshot, &cursor)
                .map_err(io::Error::other)?;
            invalidations.extend(store.snapshot_invalidations().map_err(io::Error::other)?);
            invalidations
        };
        self.enqueue_snapshot_invalidations(invalidations);
        self.signal_upload();
        self.initialization.activity();
        Ok(true)
    }
    pub fn refresh_cached(&self) {
        #[cfg(test)]
        self.refresh_runs.fetch_add(1, Ordering::Relaxed);
        let tickets = match self.store.lock().unwrap().downloads_needed() {
            Ok(t) => t,
            Err(_) => return,
        };
        for ticket in tickets {
            let now = unix_seconds();
            let node = {
                let store = self.store.lock().unwrap();
                if !self.download_ready(&store, &ticket, now).unwrap_or(false) {
                    continue;
                }
                store.node(&ticket.id)
            };
            if let Ok(node) = node {
                let _ = self.ensure_mode(&node, false);
            }
        }
    }
    pub fn event_loop(&self) -> io::Result<()> {
        self.event_loop_observed(|_| {})
    }
    pub(crate) fn event_loop_observed(
        &self,
        mut stage: impl FnMut(crate::connection_diagnostics::Stage),
    ) -> io::Result<()> {
        use crate::connection_diagnostics::Stage;
        let mut digest = String::new();
        let mut changed_notification = false;
        loop {
            stage(Stage::PullEvents);
            self.pull_after_notification(changed_notification)?;
            // The persistent transfer scheduler owns hydration. Waiting for a
            // body here stalls the native event cursor while a large/limited
            // download is running. A wake only inspects local queued work.
            self.signal_upload();
            let cursor = self
                .store
                .lock()
                .unwrap()
                .cursor()
                .map_err(io::Error::other)?;
            stage(Stage::WaitNotification);
            self.initialization
                .phase(crate::initialization::Phase::Waiting);
            let response = self.connection().call(
                Command::Wait { cursor, digest },
                crate::wire::NOTIFICATION_IDLE,
            )?;
            digest = match response.body {
                Response::Notification { digest, changed, profile_changed } => {
                    if profile_changed {
                        // Profile/ACL changes need not produce a file event.
                        // Discard stale evidence, never the user's read-only
                        // choice or an unresolved mutation's durable intent.
                        let store = self.store.lock().unwrap();
                        if let Some(root) = drive_core::write_permissions::filesystem_root(&store)
                            .map_err(io::Error::other)?
                        {
                            drive_core::write_permissions::invalidate_remote_permissions(&store, &root)
                                .map_err(io::Error::other)?;
                        }
                    }
                    changed_notification = changed;
                    digest
                }
                body => return Err(operation_error(body)),
            };
        }
    }
    fn transfer(&self, node: &Node, flight: &Flight) -> Result<(), i32> {
        let ticket = DownloadTicket {
            id: node.id.clone(),
            version: node.version.clone(),
        };
        let permission_revision = {
            let mut store = self.store.lock().unwrap();
            Self::check_download_permission(&store, &node.id)?;
            store.begin_download(&ticket).map_err(|_| libc::EIO)?;
            drive_core::write_permissions::session_revision(&store).map_err(|_| libc::EIO)?
        };
        match self.transfer_once(node, flight) {
            Ok(()) => {
                let mut store = self.store.lock().unwrap();
                if drive_core::write_permissions::session_revision(&store).map_err(|_|libc::EIO)? == permission_revision {
                    drive_core::write_permissions::observe_targets(&store,&[(node.id.clone(),drive_core::write_permissions::Action::Read)],true).map_err(|_|libc::EIO)?;
                }
                store.finish_download(&ticket).map_err(|_|libc::EIO)?;
                self.download_retry_clock.lock().unwrap().clear(&ticket);
                self.retry_wake.1.notify_all();
                Ok(())
            }
            Err((errno, _, _)) if errno == libc::EINTR => {
                // Cancellation is not a failed download or a persistent retry
                // request. A later pin/update remains independently scheduled.
                self.store
                    .lock()
                    .unwrap()
                    .finish_download(&ticket)
                    .map_err(|_| libc::EIO)?;
                Ok(())
            }
            Err((errno, kind, code)) => {
                self.download_failures_seen.fetch_add(1, Ordering::Relaxed);
                *self.last_download_failure.lock().unwrap() = Some((kind.clone(), code));
                let persisted = {
                    let mut store = self.store.lock().unwrap();
                    let failed_at = unix_seconds();
                    let failed_instant = std::time::Instant::now();
                    // Publish the learned refusal and its retry gate under one
                    // Store guard. A profile notification cannot clear the ACL
                    // between them and leave a stale terminal download behind.
                    let result = if errno == libc::EACCES && kind == DownloadFailureKind::ServerRejected && code.is_none() {
                        if drive_core::write_permissions::session_revision(&store).map_err(|_|libc::EIO)? != permission_revision {
                            drive_core::write_permissions::defer_stale_read_refusal(&store, &ticket)
                        } else {
                            drive_core::write_permissions::observe_targets(&store,
                                &[(node.id.clone(), drive_core::write_permissions::Action::Read)], false)
                                .and_then(|_| store.refuse_download(&ticket, failed_at))
                        }
                    } else { store.fail_download(&ticket, kind, code, failed_at) };
                    result.and_then(|_| self.publish_download_retry(&store, &ticket, failed_at, failed_instant))
                };
                if persisted.is_err() {
                    self.note_failure(BackgroundSource::DownloadPublication);
                }
                Err(errno)
            }
        }
    }
    fn transfer_once(
        &self,
        node: &Node,
        flight: &Flight,
    ) -> Result<(), (i32, DownloadFailureKind, Option<u32>)> {
        use DownloadFailureKind::*;
        self.downloads.fetch_add(1, Ordering::Relaxed);
        let rpc = self.connection();
        let response = rpc
            .call_cancellable(
                Command::Download { node: node.clone() },
                Duration::from_secs(300),
                &|| self.download_abandoned(node, flight),
            )
            .map_err(|e| {
                (
                    if e.kind() == io::ErrorKind::Interrupted {
                        libc::EINTR
                    } else {
                        libc::ENETDOWN
                    },
                    Transport,
                    None,
                )
            })?;
        if let Response::PermissionDenied{node:refused,action:drive_core::write_permissions::Action::Read}=&response.body {
            if refused!=&node.id || response.descriptor.is_some(){return Err((libc::EIO,InvalidContent,None));}
            return Err((libc::EACCES,ServerRejected,None));
        }
        if let Response::Failed { category } = &response.body {
            let (kind, code) = download_failure_category(category);
            if kind == SessionExpired {
                rpc.close();
            }
            return Err((libc::EIO, kind, code));
        }
        let Response::Content {
            identity,
            version,
            size,
            sha256,
        } = response.body
        else {
            return Err((libc::EIO, InvalidContent, None));
        };
        if identity != node.id || version != node.version {
            return Err((libc::ESTALE, InvalidContent, None));
        }
        let fd = response
            .descriptor
            .ok_or((libc::EIO, InvalidContent, None))?;
        let mut file = std::fs::File::from(fd);
        self.publish_content(node, flight, &rpc, &mut file, size, &sha256)
    }
    fn publish_content<R: io::Read>(
        &self,
        node: &Node,
        flight: &Flight,
        rpc: &Arc<Rpc>,
        file: &mut R,
        size: u64,
        sha256: &str,
    ) -> Result<(), (i32, DownloadFailureKind, Option<u32>)> {
        use DownloadFailureKind::*;
        let ticket = DownloadTicket {
            id: node.id.clone(),
            version: node.version.clone(),
        };
        let preparation = self
            .store
            .lock()
            .unwrap()
            .prepare_download(&ticket)
            .map_err(download_publication_error)?;
        // Body copying, hash validation and all body/directory fsyncs happen
        // without Store. The preparation's reservation protects it from GC.
        let prepared = preparation
            .prepare(file, size, sha256, || {
                !Arc::ptr_eq(rpc, &self.connection()) || self.download_abandoned(node, flight)
            })
            .map_err(|error| match error {
                drive_core::Error::Io(ref e) if e.kind() == io::ErrorKind::Interrupted => {
                    (libc::EINTR, Transport, None)
                }
                drive_core::Error::Io(_) | drive_core::Error::Db(_) => {
                    (libc::EIO, LocalStorage, None)
                }
                _ => (libc::EIO, InvalidContent, None),
            })?;
        // Demand inspection takes Store itself; keep it outside the commit
        // guard. Old-session responses cannot publish through a new RPC.
        if self.download_abandoned(node, flight) {
            return Err((libc::EINTR, Transport, None));
        }
        let mut store = self.store.lock().unwrap();
        if !Arc::ptr_eq(rpc, &self.connection()) {
            return Err((libc::ENETDOWN, Transport, None));
        }
        store
            .publish_prepared_download(prepared)
            .map_err(download_publication_error)?;
        let ino = store
            .record(&node.id)
            .map_err(|_| (libc::EIO, LocalStorage, None))?
            .ino;
        drop(store);
        self.enqueue_invalidation(Invalidation::Inode(ino));
        Ok(())
    }
    pub fn ensure(&self, node: &Node) -> Result<(), i32> {
        self.ensure_cancellable(node, None)
    }
    fn check_download_permission(store: &drive_core::Store, id: &str) -> Result<(), i32> {
        if store.download_permission_denied(id).map_err(|_| libc::EIO)? {
            Err(libc::EACCES)
        } else {
            Ok(())
        }
    }
    fn reject_download_flight(&self, node: &Node, flight: &Flight, ticket: &DownloadTicket, error: i32) -> Result<(), i32> {
        // Cancellation can race with the Store permission check. Re-evaluate
        // demand after that guard is released, including pins/cache/other users,
        // before leaving a refusal with this flight's durable retry intent.
        if self.download_abandoned(node, flight) {
            self.store.lock().unwrap().finish_download(ticket).map_err(|_| libc::EIO)?;
            Ok(())
        } else {
            Err(error)
        }
    }
    fn retry_ticket_current(store: &drive_core::Store, ticket: &DownloadTicket) -> drive_core::Result<bool> {
        match store.node(&ticket.id) {
            Ok(node) => Ok(node.version == ticket.version),
            Err(error) if error.is_not_found() => Ok(false),
            Err(error) => Err(error),
        }
    }
    fn publish_download_retry(&self, store: &drive_core::Store, ticket: &DownloadTicket, wall: u64, now: std::time::Instant) -> drive_core::Result<()> {
        // fail_download deliberately no-ops for an obsolete ticket. Never
        // mislabel the current version's failure as that old request's result.
        if !Self::retry_ticket_current(store, ticket)? {
            return Ok(());
        }
        let failure = store.download_failure(&ticket.id)?;
        self.download_retry_clock.lock().unwrap().publish(ticket, failure.as_ref(), wall, now);
        self.retry_wake.1.notify_all();
        Ok(())
    }
    fn retry_clock_ready(&self, store: &drive_core::Store, ticket: &DownloadTicket, now: u64) -> drive_core::Result<bool> {
        if !Self::retry_ticket_current(store, ticket)? {
            return Ok(false);
        }
        let failure = store.download_failure(&ticket.id)?;
        Ok(self.download_retry_clock.lock().unwrap().ready(
            ticket, failure.as_ref(), now, std::time::Instant::now()))
    }
    fn download_ready(&self, store: &drive_core::Store, ticket: &DownloadTicket, now: u64) -> drive_core::Result<bool> {
        // Seed even before the wall deadline is due, so a subsequent clock
        // correction cannot bypass the same gate on another flight or worker.
        let retry_ready = self.retry_clock_ready(store, ticket, now)?;
        Ok(store.download_ready(ticket, now)? && retry_ready)
    }
    fn ensure_cancellable(
        &self,
        node: &Node,
        cancel: Option<&drive_fs::ReadCancellation>,
    ) -> Result<(), i32> {
        self.ensure_mode_controlled(node, true, cancel)
    }

    /// Evaluate actual users of a flight, not the durable retry row created by
    /// that very flight. Hold the demand lock through the decision so a reader
    /// cannot join a transfer once its cancellation has been decided.
    fn download_abandoned(&self, node: &Node, flight: &Flight) -> bool {
        let mut demand = flight.demand.lock().unwrap();
        if demand.keep || demand.readers.iter().any(|r| !r.is_cancelled()) {
            return false;
        }
        let store = self.store.lock().unwrap();
        if store.is_pinned(&node.id).unwrap_or(true) || store.open_cached(&node.id).is_ok() {
            return false;
        }
        demand.closed = true;
        true
    }

    /// One shared flight owns the complete foreground retry interval. Keeping
    /// the same demand set through backoff prevents the scheduler (or the next
    /// attempt) from promoting its own retry record into independent demand.
    fn run_download_flight(&self, node: &Node, flight: &Flight) -> Result<(), i32> {
        let ticket = DownloadTicket {
            id: node.id.clone(),
            version: node.version.clone(),
        };
        let retry_deadline = std::time::Instant::now() + Duration::from_secs(35);
        loop {
            if self.download_abandoned(node, flight) {
                self.store
                    .lock()
                    .unwrap()
                    .finish_download(&ticket)
                    .map_err(|_| libc::EIO)?;
                return Ok(());
            }
            let permission = Self::check_download_permission(&self.store.lock().unwrap(), &node.id);
            if let Err(error) = permission {
                return self.reject_download_flight(node, flight, &ticket, error);
            }
            let result = if self
                .store
                .lock()
                .unwrap()
                .open_cached(&node.id)
                .is_ok_and(|h| h.version() == node.version)
            {
                Ok(())
            } else if !self.connection().is_connected() {
                Err(libc::ENETDOWN)
            } else {
                let ready = self.download_ready(&self.store.lock().unwrap(), &ticket, unix_seconds());
                match ready {
                    Ok(true) => self.transfer(node, flight),
                    Ok(false) => Err(libc::EAGAIN),
                    Err(_) => Err(libc::EIO),
                }
            };
            let error = match result {
                Ok(()) => return Ok(()),
                Err(error) => error,
            };
            // Cancellation can arrive alongside a failed response. Persisted
            // failure history is not itself a reason to keep fetching a body.
            if self.download_abandoned(node, flight) {
                self.store
                    .lock()
                    .unwrap()
                    .finish_download(&ticket)
                    .map_err(|_| libc::EIO)?;
                return Ok(());
            }
            {
                let mut demand = flight.demand.lock().unwrap();
                if !demand.foreground {
                    // Seal a background-only terminal attempt atomically with
                    // checking its readers. A late foreground join starts a
                    // fresh flight which observes the same durable backoff.
                    demand.closed = true;
                    return Err(error);
                }
            }
            if !self.connection().is_connected() {
                return Err(error);
            }
            let retry_at = {
                let store = self.store.lock().unwrap();
                let same_version = store
                    .node(&node.id)
                    .is_ok_and(|n| n.version == node.version);
                if !same_version || store.dirty_draft(&node.id).unwrap_or(true) {
                    return Err(error);
                }
                store
                    .download_failure(&node.id)
                    .ok()
                    .flatten()
                    .and_then(|failure| {
                        if failure.kind == DownloadFailureKind::Transport
                            || (failure.kind == DownloadFailureKind::ServerRejected
                                && failure.server_code == Some(32772))
                        {
                            failure.retry_at
                        } else {
                            None
                        }
                    })
            };
            let Some(_retry_at) = retry_at else {
                return Err(error);
            };
            while !self.retry_clock_ready(&self.store.lock().unwrap(), &ticket, unix_seconds()).map_err(|_|libc::EIO)? {
                if !Self::retry_ticket_current(&self.store.lock().unwrap(), &ticket).map_err(|_|libc::EIO)? {
                    return Err(libc::EAGAIN);
                }
                if self.download_abandoned(node, flight) {
                    self.store
                        .lock()
                        .unwrap()
                        .finish_download(&ticket)
                        .map_err(|_| libc::EIO)?;
                    return Ok(());
                }
                let permission = Self::check_download_permission(&self.store.lock().unwrap(), &node.id);
                if let Err(error) = permission {
                    return self.reject_download_flight(node, flight, &ticket, error);
                }
                let remaining =
                    retry_deadline.saturating_duration_since(std::time::Instant::now());
                if remaining.is_zero() {
                    return Err(libc::EAGAIN);
                }
                if !self.connection().is_connected() {
                    return Err(libc::ENETDOWN);
                }
                // No flight-map/database locks while waiting. A bounded slice
                // (≤250 ms) keeps cancellation observable, and another flight
                // publishing or clearing this ticket wakes us early, instead of
                // taking three Store locks and several SQL reads every 25 ms.
                let wait = retry_wait_slice(remaining);
                if let Ok(guard) = self.retry_wake.0.lock() {
                    let _ = self.retry_wake.1.wait_timeout(guard, wait);
                }
            }
            if std::time::Instant::now() >= retry_deadline {
                return Err(libc::EAGAIN);
            }
        }
    }
    fn ensure_mode(&self, node: &Node, foreground: bool) -> Result<(), i32> {
        self.ensure_mode_controlled(node, foreground, None)
    }
    fn ensure_mode_controlled(
        &self,
        node: &Node,
        foreground: bool,
        cancel: Option<&drive_fs::ReadCancellation>,
    ) -> Result<(), i32> {
        if cancel.is_some_and(|c| c.is_cancelled()) {
            return Err(libc::EINTR);
        }
        {
            let store = self.store.lock().unwrap();
            Self::check_download_permission(&store, &node.id)?;
            if store.open_cached(&node.id).is_ok_and(|h| h.version() == node.version) {
                return Ok(());
            }
        }
        let ticket = DownloadTicket {
            id: node.id.clone(),
            version: node.version.clone(),
        };
        let key = (node.id.clone(), node.version.clone());
        let (flight, owner) = loop {
            let mut flights = self.flights.lock().unwrap();
            // The scheduler observing a foreground transfer's durable inflight
            // row does not create new demand for that body. Pins and existing
            // cache are checked independently before cancellation.
            if !foreground && flights.contains_key(&key) {
                return Ok(());
            }
            let (flight, owner) = match flights.get(&key) {
                Some(f) => (f.clone(), false),
                None => {
                    let f = Arc::new(Flight {
                        demand: Mutex::new(Demand::default()),
                        done: Mutex::new(None),
                        changed: Condvar::new(),
                    });
                    flights.insert(key.clone(), f.clone());
                    (f, true)
                }
            };
            let mut demand = flight.demand.lock().unwrap();
            if !demand.closed {
                demand.foreground |= foreground;
                demand.readers.retain(|reader| !reader.is_cancelled());
                if let Some(cancel) = cancel {
                    demand.readers.push(cancel.clone());
                } else {
                    demand.keep = true;
                }
                drop(demand);
                break (flight, owner);
            }
            drop(demand);
            drop(flights);
            // A new reader must not join an already cancelled or completing
            // flight. Wait for removal before inspecting its durable state.
            let done = flight.done.lock().unwrap();
            let _ = flight
                .changed
                .wait_timeout(done, Duration::from_millis(25))
                .unwrap();
            if cancel.is_some_and(|c| c.is_cancelled()) {
                return Err(libc::EINTR);
            }
        };
        if owner {
            // The flight lock now protects release. Recheck user intent after
            // acquiring it, so an earlier queued pin cannot undo a later release.
            let background = self
                .store
                .lock()
                .unwrap()
                .wants_background_download(&ticket)
                .unwrap_or(false);
            if background {
                flight.demand.lock().unwrap().keep = true;
            }
            let wanted = foreground || background;
            let result = if wanted {
                self.run_download_flight(node, &flight)
            } else {
                Ok(())
            };
            *flight.done.lock().unwrap() = Some(result);
            flight.changed.notify_all();
            self.flights.lock().unwrap().remove(&key);
            if result == Err(libc::EACCES)
                && self.store.lock().unwrap().download_ready(&ticket, unix_seconds()).unwrap_or(false)
            {
                // A late refusal invalidated by a profile change stays queued.
                // Wake only after releasing the old flight's deduplication key.
                self.signal_upload();
            }
            if cancel.is_some_and(|c| c.is_cancelled()) {
                Err(libc::EINTR)
            } else {
                result
            }
        } else {
            let mut done = flight.done.lock().unwrap();
            while done.is_none() {
                if cancel.is_some_and(|c| c.is_cancelled()) {
                    return Err(libc::EINTR);
                }
                done = flight
                    .changed
                    .wait_timeout(done, Duration::from_millis(25))
                    .unwrap()
                    .0
            }
            if cancel.is_some_and(|c| c.is_cancelled()) {
                Err(libc::EINTR)
            } else {
                done.unwrap()
            }
        }
    }
}
fn download_publication_error(error: drive_core::Error) -> (i32, DownloadFailureKind, Option<u32>) {
    match error {
        drive_core::Error::Db(_) if error.is_not_found() => {
            (libc::ESTALE, DownloadFailureKind::InvalidContent, None)
        }
        drive_core::Error::Io(_) | drive_core::Error::Db(_) => {
            (libc::EIO, DownloadFailureKind::LocalStorage, None)
        }
        _ => (libc::ESTALE, DownloadFailureKind::InvalidContent, None),
    }
}
impl drive_fs::Hydrator for Bridge {
    fn local_saved(&self) {
        self.signal_upload();
    }
    fn hydrate_cancellable(
        &self,
        node: &Node,
        cancel: &drive_fs::ReadCancellation,
    ) -> Result<Arc<ReadHandle>, i32> {
        self.ensure_cancellable(node, Some(cancel))?;
        let h = self
            .store
            .lock()
            .unwrap()
            .open_cached(&node.id)
            .map_err(|_| libc::EIO)?;
        if h.version() != node.version {
            return Err(libc::ESTALE);
        }
        Ok(Arc::new(h))
    }
    fn hydrate(&self, node: &Node) -> Result<Arc<ReadHandle>, i32> {
        self.ensure(node)?;
        let h = self
            .store
            .lock()
            .unwrap()
            .open_cached(&node.id)
            .map_err(|_| libc::EIO)?;
        if h.version() != node.version {
            return Err(libc::ESTALE);
        }
        Ok(Arc::new(h))
    }
}

impl Bridge {
    pub fn keep_directory_conflict_both(
        &self,
        root: &str,
        id: &str,
        token: &str,
        name: &str,
    ) -> io::Result<drive_core::directory_conflicts::KeptDirectory> {
        let mut store = self.store.lock().unwrap();
        let result = store
            .keep_directory_both(root, id, token, name)
            .map_err(io::Error::other)?;
        if let Ok(record) = store.record(id) {
            self.enqueue_invalidation(Invalidation::Inode(record.ino));
        }
        if let Some(parent) = result.node.parent.as_deref() {
            if let Ok(record) = store.record(parent) {
                self.enqueue_invalidation(Invalidation::Inode(record.ino));
                self.enqueue_invalidation(Invalidation::Entry(record.ino, result.old_name.clone()));
                self.enqueue_invalidation(Invalidation::Entry(record.ino, result.node.name.clone()));
            }
        }
        drop(store);
        self.signal_upload();
        Ok(result)
    }
    pub fn keep_conflict_both(
        &self,
        root: &str,
        id: &str,
        token: &str,
        name: &str,
    ) -> io::Result<drive_core::conflicts::KeptBoth> {
        let mut store = self.store.lock().unwrap();
        let result = store
            .keep_both(root, id, token, name)
            .map_err(io::Error::other)?;
        if let Ok(record) = store.record_for_handle(id) {
            self.enqueue_invalidation(Invalidation::Inode(record.ino));
            if let Some(parent) = record.node.parent.as_deref() {
                if let Ok(parent) = store.record(parent) {
                    // Resolving an unbound creation exposes a different object
                    // at the old name. Invalidate that dentry as well as the copy.
                    self.enqueue_invalidation(Invalidation::Entry(parent.ino, record.node.name.clone()));
                }
            }
        }
        if let Some(parent) = result.copy.parent.as_deref() {
            if let Ok(record) = store.record(parent) {
                self.enqueue_invalidation(Invalidation::Inode(record.ino));
                self.enqueue_invalidation(Invalidation::Entry(record.ino, result.copy.name.clone()));
            }
        }
        drop(store);
        self.signal_upload();
        Ok(result)
    }
    pub fn signal_upload(&self) {
        *self.upload_signal.0.lock().unwrap() = true;
        self.upload_signal.1.notify_all();
    }
    /// Enqueue a bounded local wake; no Store or network wait on the UI path.
    pub fn request_issue_recheck(&self) -> bool {
        let fresh = !self.recheck_requested.swap(true, Ordering::AcqRel);
        self.signal_upload();
        fresh
    }
    /// Last background failure message, for callers that only render text.
    pub fn last_error(&self) -> Option<String> {
        self.failure
            .lock()
            .unwrap()
            .as_ref()
            .map(|failure| failure.message.to_string())
    }
    pub fn background_failure(&self) -> Option<BackgroundFailure> {
        self.failure.lock().unwrap().clone()
    }
    fn note_failure(&self, source: BackgroundSource) {
        let mut failure = self.failure.lock().unwrap();
        let since_ms = failure
            .as_ref()
            .filter(|current| current.source == source)
            .map(|current| current.since_ms)
            .unwrap_or_else(crate::background::now_ms);
        *failure = Some(BackgroundFailure {
            source,
            message: source.message(),
            since_ms,
        });
    }
    fn clear_failure(&self, source: BackgroundSource) {
        let mut failure = self.failure.lock().unwrap();
        if failure
            .as_ref()
            .is_some_and(|current| current.source == source)
        {
            *failure = None;
        }
    }
    pub fn clear_all_failures(&self) {
        *self.failure.lock().unwrap() = None;
    }
    /// Consume a user-requested recheck while the caller holds the Store.
    fn handle_recheck_request(&self, store: &mut drive_core::Store) {
        store.request_namespace_recheck();
        if drive_core::write_permissions::reset_session(store).is_err() {
            self.note_failure(BackgroundSource::PermissionRecheck);
        } else {
            // Re-arm the bounded read-only reconciliation for uncertain
            // uploads; operation states are untouched.
            let _ = store.reset_upload_reconcile_budget();
            self.clear_failure(BackgroundSource::PermissionRecheck);
        }
    }
    /// Run one cycle and record/clear only the transient failures it observed.
    fn run_upload_cycle(&self) -> CycleOutcome {
        match self.upload_cycle() {
            Ok(outcome) => {
                if outcome.ran {
                    self.clear_failure(BackgroundSource::UploadQueueStopped);
                }
                outcome
            }
            Err(error) if error.kind() != io::ErrorKind::NotConnected => {
                self.note_failure(BackgroundSource::UploadQueueStopped);
                CycleOutcome {
                    ran: false,
                    confirmed: 0,
                }
            }
            Err(_) => CycleOutcome {
                ran: false,
                confirmed: 0,
            },
        }
    }
    pub fn upload_loop(&self, stop: &std::sync::atomic::AtomicBool) {
        self.upload_loop_with(stop, UPLOAD_WAIT_INTERVAL, UPLOAD_REFRESH_MAX_INTERVAL);
    }
    /// Upload worker loop.
    ///
    /// A signal (pin/retry/recheck, remote event batch, snapshot install) or a
    /// due retry deadline wakes the loop immediately instead of waiting for the
    /// idle poll, and a cycle that confirmed uploads runs the next cycle
    /// back-to-back: operations appended by the previous cycle are drained
    /// instead of sitting until the next 1s poll.
    ///
    /// Hydration refreshes are charged against upload progress, because
    /// `refresh_cached()` enumerates durable download intent. A wake that
    /// produced no upload work refreshes immediately (pins, retries, events and
    /// startup must never wait for the poll), while a productive cycle only
    /// marks the refresh pending and runs it on the next idle tick or after
    /// `refresh_max_interval`. Deferral therefore never drops pinned, stale or
    /// retryable download intent; it only avoids paying the enumeration once per
    /// uploaded operation.
    fn upload_loop_with(
        &self,
        stop: &std::sync::atomic::AtomicBool,
        wait_interval: Duration,
        refresh_max_interval: Duration,
    ) {
        self.signal_upload();
        let mut refresh_pending = false;
        let mut pending_since = Instant::now();
        // Set once a cycle confirmed an upload: the next iteration runs it
        // back-to-back without waiting for a wake or the poll timeout.
        let mut drain = false;
        let mut last_storage_prune = Instant::now();
        while !stop.load(Ordering::Acquire) {
            let mut wake = self.upload_signal.0.lock().unwrap();
            if !*wake && !drain && !stop.load(Ordering::Acquire) {
                wake = self
                    .upload_signal
                    .1
                    .wait_timeout(wake, wait_interval)
                    .unwrap()
                    .0;
            }
            if stop.load(Ordering::Acquire) {
                break;
            }
            let signaled = *wake;
            *wake = false;
            drop(wake);
            // A back-to-back cycle is not a fresh signal: it may only refresh
            // when a retry deadline is due or the deferred interval elapsed.
            let auto = drain;
            drain = false;
            // Download deadlines, bounded namespace observation deadlines and
            // the uncertain-upload reconciliation budget. Namespace intent
            // remains durable; its backoff is local to this process. This check
            // does not enumerate the NAS.
            let retry_due = {
                let mut store = self.store.lock().unwrap();
                if self.recheck_requested.swap(false, Ordering::AcqRel) {
                    self.handle_recheck_request(&mut store);
                }
                store
                    .next_download_retry()
                    .ok()
                    .flatten()
                    .is_some_and(|at| at <= unix_seconds())
                    || store.namespace_recovery_due().unwrap_or(false)
                    || store
                        .upload_reconcile_due(unix_seconds() as i64)
                        .unwrap_or(false)
            };
            if !signaled && !retry_due && !auto {
                // Idle poll: settle a deferred hydration refresh now that the
                // queue is quiet, without competing with upload progress.
                if last_storage_prune.elapsed() >= STORAGE_PRUNE_INTERVAL {
                    last_storage_prune = Instant::now();
                    // Bounded bookkeeping cleanup; failures keep the old rows
                    // and are retried on the next interval.
                    let mut store = self.store.lock().unwrap();
                    let _ = store.prune_storage();
                }
                if refresh_pending && self.connection().is_connected() {
                    refresh_pending = false;
                    self.refresh_cached();
                }
                continue;
            }
            if !self.connection().is_connected() {
                continue;
            }
            // A gated cycle observed nothing; it must not clear a failure that
            // this cycle never re-verified.
            let outcome = self.run_upload_cycle();
            if outcome.confirmed > 0 {
                if !refresh_pending {
                    refresh_pending = true;
                    pending_since = Instant::now();
                }
                drain = true;
                // Retry deadlines and the deferred-hydration cap still refresh
                // while uploads drain.
                if !retry_due && pending_since.elapsed() < refresh_max_interval {
                    continue;
                }
            } else if auto && !retry_due {
                // The back-to-back cycle found no more upload work. Leave the
                // deferred refresh to the idle tick instead of paying it here.
                continue;
            }
            refresh_pending = false;
            // Pin actions must not wait for a remote event to wake hydration.
            // A wake that produced no upload work is evidence the queue is
            // idle, so refresh without waiting for the poll timeout.
            if self.connection().is_connected() {
                self.refresh_cached();
            }
        }
    }
    /// Reconciliation sends only read queries. Unknown outcomes never re-upload.
    pub fn upload_pending(&self, reconcile: bool) -> io::Result<usize> {
        match self.upload_pending_pass(reconcile)? {
            UploadPass::Gated => Ok(0),
            UploadPass::Ran(confirmed) => Ok(confirmed),
        }
    }
    /// One queue cycle: reconcile uncertain commits, drain namespace intents,
    /// then submit pending uploads. `ran` is false when every pass was gated.
    fn upload_cycle(&self) -> io::Result<CycleOutcome> {
        let mut ran = false;
        let mut confirmed = 0;
        if let UploadPass::Ran(count) = self.upload_pending_pass(true)? {
            ran = true;
            confirmed += count;
        }
        self.namespace_pending()?;
        if let UploadPass::Ran(count) = self.upload_pending_pass(false)? {
            ran = true;
            confirmed += count;
        }
        Ok(CycleOutcome { ran, confirmed })
    }
    /// Pass body shared by `upload_pending` and one `upload_cycle` step.
    fn upload_pending_pass(&self, reconcile: bool) -> io::Result<UploadPass> {
        let _mutation = self.recovery_gate.read().unwrap();
        if self.session_pending.load(Ordering::Acquire) {
            return Ok(UploadPass::Gated);
        }
        // A restored process has a fresh in-memory lock. Only the durable gate
        // prevents it from reconciling or submitting against an incomplete tree.
        // Reconciliation is read-only remotely but changes local operation state.
        if self
            .store
            .lock()
            .unwrap()
            .initialization_writeback_blocked()
            .map_err(io::Error::other)?
        {
            return Ok(UploadPass::Gated);
        }
        use sha2::Digest;
        use std::io::{Read, Seek, SeekFrom};
        let rpc = self.connection();
        if !rpc.is_connected() {
            return Err(io::Error::new(
                io::ErrorKind::NotConnected,
                "worker offline; queue retained",
            ));
        }
        let operations = {
            let store = self.store.lock().unwrap();
            if reconcile {
                store
                    .upload_reconcile_operations(unix_seconds() as i64)
                    .map_err(io::Error::other)?
            } else {
                store
                    .upload_operations("pending")
                    .map_err(io::Error::other)?
            }
        };
        let mut confirmed = 0;
        let mut blocked: Option<String> = None;
        for op in operations {
            let (mut job, scope) = {
                let mut s = self.store.lock().unwrap();
                let job = match s.upload_job(&op) {
                    Ok(job) => job,
                    Err(error) => {
                        if !reconcile {
                            s.record_upload_preparation_failure(&op, "local_storage").map_err(io::Error::other)?;
                        }
                        return Err(io::Error::other(error));
                    }
                };
                (
                    job,
                    s.scope_path().map_err(io::Error::other)?,
                )
            };
            let permission_targets=drive_core::write_permissions::upload_targets(&job.node);
            if !reconcile && drive_core::write_permissions::targets_denied(&self.store.lock().unwrap(),&permission_targets).map_err(io::Error::other)? {
                blocked=Some("permission_denied".into());continue;
            }
            // Preparation is read-only remotely. A failed attempt is history,
            // not an uncertain mutation, and its raw error is never persisted.
            let record_preparation_failure = |kind: &str| -> io::Result<()> {
                if !reconcile {
                    self.store.lock().unwrap().record_upload_preparation_failure(&op, kind)
                        .map_err(io::Error::other)?;
                }
                Ok(())
            };
            let mut h = sha2::Sha256::new();
            let mut buf = [0; 65536];
            loop {
                let n = match job.file.read(&mut buf) {
                    Ok(n) => n,
                    Err(error) => { record_preparation_failure("local_storage")?; return Err(error); }
                };
                if n == 0 {
                    break;
                }
                h.update(&buf[..n]);
            }
            if let Err(error) = job.file.seek(SeekFrom::Start(0)) {
                record_preparation_failure("local_storage")?;
                return Err(error);
            }
            let sha256 = format!("{:x}", h.finalize());
            if !reconcile && job.node.version == drive_core::creations::CREATE_VERSION {
                let (parent, cursor) = {
                    let s = self.store.lock().unwrap();
                    (
                        s.node(
                            job.node
                                .parent
                                .as_deref()
                                .ok_or_else(|| io::Error::other("creation parent absent"))?,
                        )
                        .map_err(io::Error::other)?,
                        s.cursor().map_err(io::Error::other)?,
                    )
                };
                if parent.version == drive_core::creations::CREATE_VERSION {
                    continue;
                }
                let path = format!("{}/{}", scope.trim_end_matches('/'), job.relative_path);
                let prepared = match rpc.call(
                    Command::PrepareUpload {
                        node: job.node.clone(),
                        parent,
                        path,
                        sha256: sha256.clone(),
                        cursor,
                    },
                    Duration::from_secs(120),
                ) {
                    Ok(reply) => reply,
                    Err(error) => {
                        record_preparation_failure(if error.kind() == io::ErrorKind::InvalidData { "protocol" } else { "network" })?;
                        return Err(error);
                    }
                };
                match prepared.body {
                    Response::UploadPrepared { origin } => {
                        // The operation is still pending: a lost preparation reply
                        // is safe to repeat because no mutation was sent.
                        let saved_origin = self
                            .store
                            .lock()
                            .unwrap()
                            .prepare_upload_origin(&op, &origin, &sha256);
                        if saved_origin.is_err() {
                            record_preparation_failure("protocol")?;
                            continue;
                        }
                    }
                    Response::Conflict => {
                        let mut s = self.store.lock().unwrap();
                        if s.mark_upload_started(&op).is_ok() {
                            s.conflict_upload(&op).map_err(io::Error::other)?;
                        }
                        continue;
                    }
                    Response::Failed { .. } => {
                        record_preparation_failure("server_rejected")?;
                        continue;
                    }
                    _ => {
                        record_preparation_failure("protocol")?;
                        continue;
                    }
                }
            }
            let (node, path, origin, mtime) = {
                let mut s = self.store.lock().unwrap();
                let fresh = s.upload_job(&op).map_err(io::Error::other)?;
                if !reconcile {
                    if let Err(error) = s.mark_upload_started(&op) {
                        // Surface the concrete dependency; a silent continue made
                        // the whole queue look idle in the transfer list.
                        blocked = Some(error.to_string());
                        continue;
                    }
                }
                (
                    fresh.node,
                    format!("{}/{}", scope.trim_end_matches('/'), fresh.relative_path),
                    s.upload_origin(&op).map_err(io::Error::other)?,
                    fresh.mtime,
                )
            };
            let node_id = node.id.clone();
            let outcome = rpc.call_with_descriptor(
                Command::Upload {
                    origin,
                    node,
                    path,
                    sha256,
                    reconcile,
                    mtime,
                },
                Duration::from_secs(1800),
                Some(job.file.into()),
            );
            let mut s = self.store.lock().unwrap();
            match outcome {
                Ok(reply) => match reply.body {
                    Response::Created { node, remote } => {
                        if let Err(e) = s.validate_upload_mtime(&op, &node) {
                            if !reconcile {
                                s.mark_upload_uncertain(&op).map_err(io::Error::other)?;
                            } else {
                                s.note_upload_reconcile_attempt(&op, unix_seconds() as i64)
                                    .map_err(io::Error::other)?;
                            }
                            return Err(io::Error::other(e));
                        }
                        s.confirm_upload_identity(&op, &node.version, Some(&remote))
                            .map_err(io::Error::other)?;
                        drive_core::write_permissions::observe_targets(&s,&permission_targets,true).map_err(io::Error::other)?;
                        confirmed += 1;
                    }
                    Response::Committed { node } => {
                        if let Err(e) = s.validate_upload_mtime(&op, &node) {
                            if !reconcile {
                                s.mark_upload_uncertain(&op).map_err(io::Error::other)?;
                            } else {
                                s.note_upload_reconcile_attempt(&op, unix_seconds() as i64)
                                    .map_err(io::Error::other)?;
                            }
                            return Err(io::Error::other(e));
                        }
                        s.confirm_upload(&op, &node.version)
                            .map_err(io::Error::other)?;
                        drive_core::write_permissions::observe_targets(&s,&permission_targets,true).map_err(io::Error::other)?;
                        confirmed += 1;
                    }
                    Response::PermissionDenied{node,action} => {
                        if !reconcile {
                            drive_core::write_permissions::refuse_operation(&s,&op,false,&permission_targets,&node,action).map_err(io::Error::other)?;
                            blocked=Some("permission_denied".into());
                        } else {
                            // The server still has not acted. A read-only
                            // reconciliation spends one bounded attempt and
                            // writes no denial or ACL evidence.
                            s.note_upload_reconcile_attempt(&op, unix_seconds() as i64)
                                .map_err(io::Error::other)?;
                            if s.is_locally_deleted(&node).map_err(io::Error::other)? {
                                s.retire_unconfirmed_creation(&op).map_err(io::Error::other)?;
                            }
                        }
                    }
                    Response::MutationPermissionUncertain { code } => {
                        if !reconcile {
                            drive_core::write_permissions::mutation_permission_uncertain(&s,&op,false,code).map_err(io::Error::other)?;
                            blocked=Some("permission_commit_uncertain".into());
                        } else {
                            s.note_upload_reconcile_attempt(&op, unix_seconds() as i64)
                                .map_err(io::Error::other)?;
                            if s.is_locally_deleted(&node_id).map_err(io::Error::other)? {
                                s.retire_unconfirmed_creation(&op).map_err(io::Error::other)?;
                            }
                        }
                    }
                    Response::Conflict => {
                        if !reconcile {
                            s.conflict_upload(&op).map_err(io::Error::other)?;
                        } else if s
                            .is_locally_deleted(&node_id)
                            .map_err(io::Error::other)?
                        {
                            // The local creation is gone. Spend one bounded
                            // attempt and retire the intent instead of
                            // parking the operation behind a conflict that
                            // can never be resolved.
                            s.note_upload_reconcile_attempt(&op, unix_seconds() as i64)
                                .map_err(io::Error::other)?;
                            s.retire_unconfirmed_creation(&op).map_err(io::Error::other)?;
                        } else {
                            s.conflict_upload(&op).map_err(io::Error::other)?;
                        }
                    }
                    _ => {
                        if !reconcile {
                            s.mark_upload_uncertain(&op).map_err(io::Error::other)?;
                        } else {
                            // The server replied, but not with a confirmation.
                            // This is still an unconfirmed observation, so it
                            // spends one bounded attempt and never fabricates
                            // a terminal state. A locally deleted creation is
                            // retired instead of waiting for more replies.
                            s.note_upload_reconcile_attempt(&op, unix_seconds() as i64)
                                .map_err(io::Error::other)?;
                            if s
                                .is_locally_deleted(&node_id)
                                .map_err(io::Error::other)?
                            {
                                s.retire_unconfirmed_creation(&op).map_err(io::Error::other)?;
                            }
                        }
                    }
                },
                Err(error) => {
                    if !reconcile {
                        s.mark_upload_uncertain(&op).map_err(io::Error::other)?;
                    } else if !transient_reconcile_error(&error) {
                        // Transport loss does not spend the visible retry
                        // budget; reconnect recovery keeps the original
                        // bounded allowance intact.
                        s.note_upload_reconcile_attempt(&op, unix_seconds() as i64)
                            .map_err(io::Error::other)?;
                    }
                    return Err(io::Error::other("commit unknown; reconciliation required"));
                }
            }
        }
        if !reconcile {
            *self.upload_block.lock().unwrap() = blocked;
        }
        Ok(UploadPass::Ran(confirmed))
    }
}

#[cfg(test)]
#[path = "upload_tests.rs"]
mod upload_tests;

#[cfg(test)]
#[path = "background_error_tests.rs"]
mod background_error_tests;

#[cfg(test)]
#[path = "loop_tests.rs"]
mod loop_tests;

#[cfg(test)]
#[path = "download_publication_tests.rs"]
mod download_publication_tests;

#[path = "namespace.rs"]
mod namespace;

#[derive(Debug)]
struct RootUnavailable;
impl std::fmt::Display for RootUnavailable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("selected native root unavailable; local contents retained")
    }
}
impl std::error::Error for RootUnavailable {}
pub(crate) fn is_root_unavailable(error: &io::Error) -> bool {
    error.get_ref().is_some_and(|e| e.is::<RootUnavailable>())
}
pub(crate) fn capacity_error(error: &io::Error) -> Option<&'static str> {
    match error.to_string().as_str() {
        "native_index_capacity" => Some("native_index_capacity"),
        "native_response_capacity" => Some("native_response_capacity"),
        _ => None,
    }
}
fn operation_error(body: Response) -> io::Error {
    use crate::connection_diagnostics::Cause;
    if let Response::Failed { category } = &body {
        if matches!(
            category.as_str(),
            "native_index_capacity" | "native_response_capacity"
        ) {
            return io::Error::new(io::ErrorKind::InvalidData, category.clone());
        }
    }
    if matches!(&body, Response::Failed { category } if category == "native_root_unavailable") {
        return io::Error::new(io::ErrorKind::InvalidData, RootUnavailable);
    }
    if matches!(&body, Response::Failed { category } if category == crate::worker::WORKER_OVERLOADED_CATEGORY) {
        // The bounded worker queue refused this command. It is retryable and
        // session-preserving; it must not masquerade as a transport loss.
        return io::Error::new(io::ErrorKind::TimedOut, Cause::WorkerOverloaded);
    }
    if let Response::Failed { category } = &body {
        let timeout = match category.as_str() {
            "native_receive_idle" => Some(Cause::NativeReceiveIdle),
            "native_notification_setup_timeout" => Some(Cause::NativeSetupTimeout),
            "native_send_timeout" => Some(Cause::NativeSendTimeout),
            "native_response_timeout" => Some(Cause::NativeResponseTimeout),
            _ => None,
        };
        if let Some(cause) = timeout {
            return io::Error::new(io::ErrorKind::TimedOut, cause);
        }
        if let Some(code) = category
            .strip_prefix("native_server_code:")
            .and_then(|code| code.parse::<u32>().ok())
        {
            return io::Error::new(
                io::ErrorKind::InvalidData,
                Cause::NativeServerRejected(code),
            );
        }
        if category == "session_expired" {
            return io::Error::new(io::ErrorKind::NotConnected, Cause::SessionExpired);
        }
        if category == "native_cursor_reset" {
            return io::Error::new(io::ErrorKind::InvalidData, Cause::NativeCursorReset);
        }
    }
    let kind = match body {
        Response::Failed { category }
            if matches!(category.as_str(), "transport" | "session_expired") =>
        {
            io::ErrorKind::NotConnected
        }
        _ => io::ErrorKind::InvalidData,
    };
    io::Error::new(
        kind,
        "native event stream unavailable; recovery may be required",
    )
}

fn unix_seconds() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
#[path = "overload_mapping_tests.rs"]
mod overload_mapping_tests;

/// A waiting reader re-checks the retry gate at most this often, so a
/// cancellation or a republished retry deadline is observed without a busy
/// 25 ms loop. The 35 s foreground window therefore needs ≤ 140 checks.
const RETRY_WAIT_SLICE: Duration = Duration::from_millis(250);

fn retry_wait_slice(remaining: Duration) -> Duration {
    remaining.min(RETRY_WAIT_SLICE)
}

/// Transport loss during a read-only reconciliation attempt is not evidence
/// that the server rejected the change. It must not spend the bounded retry
/// budget that the user can see.
fn transient_reconcile_error(error: &io::Error) -> bool {
    matches!(
        error.kind(),
        io::ErrorKind::NotConnected
            | io::ErrorKind::TimedOut
            | io::ErrorKind::Interrupted
            | io::ErrorKind::BrokenPipe
            | io::ErrorKind::ConnectionAborted
            | io::ErrorKind::ConnectionReset
            | io::ErrorKind::UnexpectedEof
    )
}

#[cfg(test)]
mod connection_diagnostic_categories {
    use super::*;
    use crate::connection_diagnostics::{Cause, retryable};
    #[test]
    fn staged_timeouts_preserve_event_and_download_retry_semantics() {
        for (category, expected) in [
            ("native_receive_idle", Cause::NativeReceiveIdle),
            (
                "native_notification_setup_timeout",
                Cause::NativeSetupTimeout,
            ),
            ("native_send_timeout", Cause::NativeSendTimeout),
            ("native_response_timeout", Cause::NativeResponseTimeout),
        ] {
            let error = operation_error(Response::Failed {
                category: category.into(),
            });
            assert!(retryable(&error));
            assert_eq!(Cause::from_io(&error), expected);
            assert_eq!(
                download_failure_category(category),
                (DownloadFailureKind::Transport, None)
            );
        }
        let error = operation_error(Response::Failed {
            category: "native_server_code:32772".into(),
        });
        assert!(!retryable(&error));
        assert_eq!(Cause::from_io(&error), Cause::NativeServerRejected(32772));
        let error = operation_error(Response::Failed {
            category: "synthetic-private-input".into(),
        });
        assert!(!error.to_string().contains("synthetic-private-input"));
        assert_eq!(Cause::from_io(&error), Cause::ProtocolOrLocal);
    }
}
fn download_failure_category(category: &str) -> (DownloadFailureKind, Option<u32>) {
    use DownloadFailureKind::*;
    if let Some(code) = category
        .strip_prefix("native_server_code:")
        .and_then(|n| n.parse::<u32>().ok())
    {
        return (ServerRejected, Some(code));
    }
    (
        match category {
            "transport"
            | "native_receive_idle"
            | "native_notification_setup_timeout"
            | "native_send_timeout"
            | "native_response_timeout" => Transport,
            "session_expired" => SessionExpired,
            _ => InvalidContent,
        },
        None,
    )
}

#[cfg(test)]
mod download_scheduler_tests {
    use super::*;
    use crate::transport;
    use std::{
        io::{Seek, SeekFrom, Write},
        sync::atomic::AtomicBool,
        time::Instant,
    };
    #[test]
    fn obsolete_ticket_cannot_displace_current_version_retry_gate() {
        let directory = tempfile::tempdir().unwrap();
        let mut store = Store::open(directory.path()).unwrap();
        let root = Node { id: "root".into(), parent: None, name: "root".into(), version: "v1".into(), directory: true, metadata: None };
        let file = Node { id: "file".into(), parent: Some("root".into()), name: "file".into(), version: "v2".into(), directory: false, metadata: None };
        store.install_remote_snapshot(&[root, file], "1", "server").unwrap();
        store.ensure_fs_root("root").unwrap();
        let current = DownloadTicket { id: "file".into(), version: "v2".into() };
        let old = DownloadTicket { id: "file".into(), version: "v1".into() };
        store.begin_download(&current).unwrap();
        store.fail_download(&current, DownloadFailureKind::Transport, None, 100).unwrap();
        let failure = store.download_failure("file").unwrap().unwrap();
        let bridge = Bridge::offline(Arc::new(drive_core::SharedStore::new(store))).unwrap();
        let start = Instant::now();
        let mut store = bridge.store.lock().unwrap();
        bridge.publish_download_retry(&store, &current, 100, start).unwrap();
        assert!(!bridge.retry_clock_ready(&store, &old, 500).unwrap());
        // This no-op is the real late-result path: it must not publish v2's
        // failure under v1 and thereby evict v2's original monotonic deadline.
        store.fail_download(&old, DownloadFailureKind::Transport, None, 500).unwrap();
        bridge.publish_download_retry(&store, &old, 500, start).unwrap();
        assert_eq!(store.download_failure("file").unwrap(), Some(failure.clone()));
        assert!(!bridge.download_retry_clock.lock().unwrap().ready(
            &current, Some(&failure), 500, start + Duration::from_secs(1)));
        assert!(bridge.download_retry_clock.lock().unwrap().ready(
            &current, Some(&failure), 500, start + Duration::from_secs(2)));
    }
    #[test]
    fn durable_retry_wakes_without_remote_event_and_clears_failure() {
        retry_roundtrip(false, "native_server_code:32772");
    }
    #[test]
    fn concurrent_foreground_reads_wait_for_shared_retry_without_reopen() {
        retry_roundtrip(true, "native_server_code:32772");
    }
    #[test]
    fn foreground_reads_respect_terminal_failure_and_worker_disconnect() {
        for terminal in [false, true] {
            let directory = tempfile::tempdir().unwrap();
            let mut store = Store::open(directory.path()).unwrap();
            let root = Node {
                id: "root".into(),
                parent: None,
                name: "root".into(),
                version: "v1".into(),
                directory: true,
                metadata: None,
            };
            let file = Node {
                id: "file".into(),
                parent: Some("root".into()),
                name: "A.doc".into(),
                version: "v1".into(),
                directory: false,
                metadata: None,
            };
            store
                .install_remote_snapshot(&[root, file.clone()], "1", "server")
                .unwrap();
            store.ensure_fs_root("root").unwrap();
            let ticket = DownloadTicket {
                id: file.id.clone(),
                version: file.version.clone(),
            };
            for _ in 0..if terminal { 5 } else { 1 } {
                store.begin_download(&ticket).unwrap();
                store
                    .fail_download(
                        &ticket,
                        DownloadFailureKind::ServerRejected,
                        Some(32772),
                        unix_seconds() + 60,
                    )
                    .unwrap();
            }
            let store = Arc::new(drive_core::SharedStore::new(store));
            let (socket, peer) = UnixStream::pair().unwrap();
            let bridge =
                Bridge::from_connection(store.clone(), Arc::new(Rpc::new(socket).unwrap()))
                    .unwrap();
            let mut peer = Some(peer);
            if !terminal {
                drop(peer.take());
                let deadline = Instant::now() + Duration::from_secs(2);
                while bridge.connection().is_connected() {
                    assert!(Instant::now() < deadline);
                    std::thread::sleep(Duration::from_millis(5));
                }
            }
            let started = Instant::now();
            assert_eq!(
                bridge.ensure(&file),
                Err(if terminal {
                    libc::EAGAIN
                } else {
                    libc::ENETDOWN
                })
            );
            assert!(started.elapsed() < Duration::from_secs(1));
            assert_eq!(bridge.downloads.load(Ordering::Relaxed), 0);
            let failure = store
                .lock()
                .unwrap()
                .download_failure("file")
                .unwrap()
                .unwrap();
            assert_eq!(failure.attempts, if terminal { 5 } else { 1 });
            drop(peer);
        }
    }
    #[test]
    fn concurrent_foreground_reads_wait_for_transport_backoff_after_reconnect() {
        retry_roundtrip(true, "transport");
    }
    #[test]
    fn concurrent_retry_observer_can_resume_after_request_is_queued() {
        retry_roundtrip_observed(true, "native_server_code:32772", true);
    }
    fn retry_roundtrip(foreground: bool, failure_category: &'static str) {
        retry_roundtrip_observed(foreground, failure_category, false);
    }
    fn retry_roundtrip_observed(
        foreground: bool,
        failure_category: &'static str,
        queued_observer: bool,
    ) {
        let d = tempfile::tempdir().unwrap();
        let mut store = Store::open(d.path()).unwrap();
        let root = Node {
            id: "root".into(),
            parent: None,
            name: "root".into(),
            version: "v1".into(),
            directory: true,
            metadata: None,
        };
        let file = Node {
            id: "file".into(),
            parent: Some("root".into()),
            name: "A.doc".into(),
            version: "v1".into(),
            directory: false,
            metadata: None,
        };
        store
            .install_remote_snapshot(&[root, file], "1", "server")
            .unwrap();
        store.ensure_fs_root("root").unwrap();
        if !foreground {
            store.pin("file", true).unwrap();
        }
        let store = Arc::new(drive_core::SharedStore::new(store));
        let (a, mut b) = UnixStream::pair().unwrap();
        b.set_read_timeout(Some(Duration::from_secs(10))).unwrap();
        let bridge = Arc::new(
            Bridge::from_connection(store.clone(), Arc::new(Rpc::new(a).unwrap())).unwrap(),
        );
        let peer_store = store.clone();
        let peer = std::thread::spawn(move || {
            let (request, _) = transport::receive::<Envelope<Command>>(&mut b).unwrap();
            assert!(matches!(request.body, Command::Download { .. }));
            // Start before sending: once Failed is sent, the client can already
            // persist the deadline and retry while this observer is descheduled.
            let first = Instant::now();
            let failed_at = unix_seconds();
            transport::send(
                &mut b,
                &Envelope {
                    version: VERSION,
                    id: request.id,
                    body: Response::Failed {
                        category: failure_category.into(),
                    },
                },
                None,
            )
            .unwrap();
            if queued_observer {
                // Deterministically cover an observer resuming only after the
                // retry is queued. No extra retry delay or product clock change.
                let mut poll = libc::pollfd {
                    fd: std::os::fd::AsRawFd::as_raw_fd(&b),
                    events: libc::POLLIN,
                    revents: 0,
                };
                assert_eq!(unsafe { libc::poll(&mut poll, 1, 5_000) }, 1);
                assert_ne!(poll.revents & libc::POLLIN, 0);
            }
            let (request, _) = transport::receive::<Envelope<Command>>(&mut b).unwrap();
            // Capture both clocks before taking the Store lock, so diagnostic
            // work cannot make an early request satisfy the original bound.
            let elapsed = first.elapsed();
            let received_at = unix_seconds();
            let failure_at_receive = peer_store.lock().unwrap().download_failure("file").unwrap();
            assert!(elapsed >= Duration::from_millis(900),
                "early retry: elapsed={elapsed:?}, failed_at={failed_at}, received_at={received_at}, download={}, failure={failure_at_receive:?}",
                matches!(&request.body, Command::Download { .. }));
            assert!(matches!(request.body, Command::Download { .. }));
            {
                let store = peer_store.lock().unwrap();
                let failure = store.download_failure("file").unwrap().unwrap();
                assert_eq!(failure.attempts, 1);
                let retry_at = failure.retry_at.unwrap();
                assert!(retry_at >= failed_at + 2);
                assert!(unix_seconds() >= retry_at);
                assert!(
                    !store
                        .download_ready(
                            &DownloadTicket {
                                id: "file".into(),
                                version: "v1".into(),
                            },
                            retry_at - 1
                        )
                        .unwrap()
                );
            }
            let mut content = tempfile::tempfile().unwrap();
            content.write_all(b"new").unwrap();
            content.seek(SeekFrom::Start(0)).unwrap();
            let fd: OwnedFd = content.into();
            transport::send(
                &mut b,
                &Envelope {
                    version: VERSION,
                    id: request.id,
                    body: Response::Content {
                        identity: "file".into(),
                        version: "v1".into(),
                        size: 3,
                        sha256: drive_core::sha256(b"new"),
                    },
                },
                Some(&fd),
            )
            .unwrap();
        });
        let stop = Arc::new(AtomicBool::new(false));
        let running = bridge.clone();
        let flag = stop.clone();
        let scheduler = std::thread::spawn(move || {
            if foreground {
                let barrier = Arc::new(std::sync::Barrier::new(8));
                let readers: Vec<_> = (0..8)
                    .map(|_| {
                        let bridge = running.clone();
                        let barrier = barrier.clone();
                        std::thread::spawn(move || {
                            let node = bridge.store.lock().unwrap().node("file").unwrap();
                            barrier.wait();
                            bridge.ensure(&node).unwrap();
                        })
                    })
                    .collect();
                for reader in readers {
                    reader.join().unwrap();
                }
            } else {
                running.upload_loop(&flag);
            }
        });
        let deadline = Instant::now() + Duration::from_secs(9);
        while store.lock().unwrap().open_cached("file").is_err() {
            if Instant::now() > deadline {
                stop.store(true, Ordering::Release);
                bridge.signal_upload();
                scheduler.join().unwrap();
                panic!("durable retry did not hydrate");
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        stop.store(true, Ordering::Release);
        bridge.signal_upload();
        scheduler.join().unwrap();
        peer.join().unwrap();
        assert_eq!(bridge.downloads.load(Ordering::Relaxed), 2);
        assert!(
            store
                .lock()
                .unwrap()
                .download_failure("file")
                .unwrap()
                .is_none()
        );
    }
}

#[cfg(test)]
#[path = "snapshot_bridge_tests.rs"]
mod snapshot_bridge_tests;

#[cfg(test)]
#[path = "initialization_gate_tests.rs"]
mod initialization_gate_tests;

#[path = "initialization_owner.rs"]
mod initialization_owner;

#[cfg(test)]
#[path = "event_transfer_tests.rs"]
mod event_transfer_tests;

#[cfg(test)]
#[path = "cancel_bridge_tests.rs"]
mod cancel_bridge_tests;

#[cfg(test)]
mod root_diagnostics_tests {
    use super::*;
    #[test]
    fn scoped_root_failure_has_a_typed_non_retryable_diagnostic() {
        let error = operation_error(Response::Failed {
            category: "native_root_unavailable".into(),
        });
        assert!(is_root_unavailable(&error));
        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
        let ordinary = operation_error(Response::Failed {
            category: "native_server_code:12291".into(),
        });
        assert!(!is_root_unavailable(&ordinary));
        let network = operation_error(Response::Failed {
            category: "transport".into(),
        });
        assert!(!is_root_unavailable(&network));
        assert_eq!(network.kind(), io::ErrorKind::NotConnected);
    }
}

/// Non-blocking on Store and kernel notification delivery; returns whether the
/// bounded queue accepted the message. A rejected committed invalidation
/// suspends the mount's cacheable publication for one quiet lease window (the
/// hub recovers by itself), never a permanent fail-closed (#27).
fn enqueue_invalidation(
    queue: &std::sync::mpsc::SyncSender<Invalidation>,
    hub: &Mutex<Option<drive_fs::NotifyHub>>,
    invalidation: Invalidation,
) -> bool {
    let accepted = queue.try_send(invalidation).is_ok();
    if !accepted {
        let hub = hub.lock().unwrap().clone();
        if let Some(hub) = hub {
            hub.invalidate_delivery_failed();
        }
    }
    accepted
}

#[cfg(test)]
#[path = "notification_queue_tests.rs"]
mod notification_queue_tests;
