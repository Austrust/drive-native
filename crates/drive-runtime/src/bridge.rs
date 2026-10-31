use crate::{rpc::Rpc, wire::*};
use drive_core::download_jobs::DownloadFailureKind;
use drive_core::{Change, DownloadTicket, Node, ReadHandle, Store};
use std::{
    collections::HashMap,
    io,
    os::{fd::OwnedFd, unix::net::UnixStream},
    path::Path,
    process::{Child, Command as Process, Stdio},
    sync::{
        Arc, Condvar, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
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
            Ok(rpc) => Ok(Self { rpc: Arc::new(rpc), child }),
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
            Ok(Some(status)) => ProcessState::Exited { code: status.code(), signal: status.signal() },
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
pub struct Bridge {
    pub store: Arc<Mutex<Store>>,
    recovery_gate: std::sync::RwLock<()>,
    event_gate: Mutex<()>,
    connection: Mutex<Arc<Rpc>>,
    flights: Mutex<HashMap<(String, String), Arc<Flight>>>,
    notifier: Arc<Mutex<Option<fuser::Notifier>>>,
    notifications: std::sync::mpsc::SyncSender<Invalidation>,
    pub downloads: AtomicUsize,
    pub download_failures_seen: AtomicUsize,
    pub last_download_failure: Mutex<Option<(DownloadFailureKind, Option<u32>)>>,
    upload_signal: (Mutex<bool>, Condvar),
    pub last_error: Mutex<Option<String>>,
    pub cache_error: Mutex<Option<String>>,
    pub initialization: Arc<crate::initialization::Tracker>,
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
    /// SQLite writer survive. A different server restore/root is rejected.
    pub fn reconnect(&self, rpc: Arc<Rpc>, login: Login) -> io::Result<()> {
        let scope = serde_json::to_string(&login.scope)?;
        let restore = Self::authenticate(&rpc, login)?;
        self.attach(rpc, &restore, &scope)
    }
    /// The supervisor serializes this short commit with pause/stop. Network
    /// authentication happens outside that lock and can be interrupted.
    pub fn attach(&self, rpc: Arc<Rpc>, restore: &str, scope: &str) -> io::Result<()> {
        self.store
            .lock()
            .unwrap()
            .bind_native_scope(restore, scope)
            .map_err(io::Error::other)?;
        let old = std::mem::replace(&mut *self.connection.lock().unwrap(), rpc);
        old.close();
        *self.last_error.lock().unwrap() = None;
        self.signal_upload();
        Ok(())
    }

    pub fn connect(store: Arc<Mutex<Store>>, rpc: Arc<Rpc>, login: Login) -> io::Result<Self> {
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
    pub fn offline(store: Arc<Mutex<Store>>) -> io::Result<Self> {
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
            let _ = self.notifications.send(Invalidation::Inode(ino));
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
        let _ = self.notifications.try_send(Invalidation::Inode(ino));
        Ok(())
    }
    fn from_connection(store: Arc<Mutex<Store>>, rpc: Arc<Rpc>) -> io::Result<Self> {
        let notifier: Arc<Mutex<Option<fuser::Notifier>>> = Arc::new(Mutex::new(None));
        let target = notifier.clone();
        let (notifications, rx) = std::sync::mpsc::sync_channel(1024);
        std::thread::spawn(move || {
            for message in rx {
                if let Some(n) = target.lock().unwrap().as_ref() {
                    match message {
                        Invalidation::Inode(ino) => {
                            let _ = n.inval_inode(ino, 0, 0);
                        }
                        Invalidation::Entry(parent, name) => {
                            let _ = n.inval_entry(parent, std::ffi::OsStr::new(&name));
                        }
                    }
                }
            }
        });
        Ok(Self {
            store,
            recovery_gate: std::sync::RwLock::new(()),
            event_gate: Mutex::new(()),
            connection: Mutex::new(rpc),
            flights: Mutex::new(HashMap::new()),
            notifier,
            notifications,
            downloads: AtomicUsize::new(0),
            download_failures_seen: AtomicUsize::new(0),
            last_download_failure: Mutex::new(None),
            upload_signal: (Mutex::new(false), Condvar::new()),
            last_error: Mutex::new(None),
            cache_error: Mutex::new(None),
            initialization: Arc::new(crate::initialization::Tracker::default()),
        })
    }
    pub fn set_notifier(&self, notifier: fuser::Notifier) {
        *self.notifier.lock().unwrap() = Some(notifier);
    }
    /// Initial cursor zero is restricted to the verified lab bootstrap in the live
    /// test. Complete snapshot/permission recovery remains a production gate.
    /// Retry quarantined permissions on an explicit session boundary. The service
    /// uses cursor-based pull for transport retries within that session.
    pub fn initial_pull(&self) -> io::Result<bool> {
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
        let cursor = self
            .store
            .lock()
            .unwrap()
            .cursor()
            .map_err(io::Error::other)?;
        if cursor.is_empty() {
            return self.rebuild_snapshot(crate::initialization::RecoveryReason::InitialIndex);
        }
        let response = self
            .connection()
            .call(Command::Pull { cursor }, Duration::from_secs(60))?;
        let batch = match response.body {
            Response::Events(batch) => batch,
            Response::Failed { category }
                if matches!(
                    category.as_str(),
                    "native_cursor_reset" | "native_filtered_type_changed"
                ) =>
            {
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
            return Ok(false);
        };
        let event_count = batch.events.len();
        let mut invalidations = Vec::new();
        let mut store = self.store.lock().unwrap();
        for event in &batch.events {
            let id = match &event.change {
                Change::Upsert(n) => &n.id,
                Change::Delete { id } => id,
            };
            if let Ok(old) = store.local_identity(id).and_then(|id| store.record(&id)) {
                if let Some(p) = old.node.parent.as_deref() {
                    if let Ok(p) = store.local_identity(p).and_then(|p| store.record(&p)) {
                        invalidations.push((p.ino, old.node.name))
                    }
                }
            }
        }
        store.receive(&batch).map_err(io::Error::other)?;
        store.drain().map_err(io::Error::other)?;
        for event in &batch.events {
            if let Change::Upsert(n) = &event.change {
                if let Some(p) = n.parent.as_deref() {
                    if let Ok(p) = store.local_identity(p).and_then(|p| store.record(&p)) {
                        invalidations.push((p.ino, n.name.clone()))
                    }
                }
            }
        }
        drop(store);
        self.initialization
            .pulled(true, event_count, changed_notification);
        for (p, name) in invalidations {
            let _ = self.notifications.try_send(Invalidation::Entry(p, name));
        }
        self.signal_upload();
        Ok(true)
    }
    fn rebuild_snapshot(&self, reason: crate::initialization::RecoveryReason) -> io::Result<bool> {
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
        for (inode, parent, name) in invalidations {
            if let Some(ino) = inode {
                let _ = self.notifications.send(Invalidation::Inode(ino));
            }
            if let Some(parent) = parent {
                let _ = self.notifications.send(Invalidation::Entry(parent, name));
            }
        }
        self.signal_upload();
        self.initialization.activity();
        Ok(true)
    }
    pub fn refresh_cached(&self) {
        let tickets = match self.store.lock().unwrap().downloads_needed() {
            Ok(t) => t,
            Err(_) => return,
        };
        for ticket in tickets {
            let now = unix_seconds();
            let node = {
                let store = self.store.lock().unwrap();
                if !store.download_ready(&ticket, now).unwrap_or(false) {
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
                Response::Notification { digest, changed } => {
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
        self.store
            .lock()
            .unwrap()
            .begin_download(&ticket)
            .map_err(|_| libc::EIO)?;
        match self.transfer_once(node, flight) {
            Ok(()) => {
                self.store
                    .lock()
                    .unwrap()
                    .finish_download(&ticket)
                    .map_err(|_| libc::EIO)?;
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
                if self
                    .store
                    .lock()
                    .unwrap()
                    .fail_download(&ticket, kind, code, unix_seconds())
                    .is_err()
                {
                    *self.last_error.lock().unwrap() = Some(
                        "cannot persist download failure; local state needs inspection".into(),
                    );
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
        let mut store = self.store.lock().unwrap();
        store
            .publish_download_reader(
                &DownloadTicket {
                    id: identity.clone(),
                    version,
                },
                &mut file,
                size,
                &sha256,
            )
            .map_err(|error| match error {
                drive_core::Error::Io(_) | drive_core::Error::Db(_) => {
                    (libc::EIO, LocalStorage, None)
                }
                _ => (libc::EIO, InvalidContent, None),
            })?;
        let ino = store
            .record(&identity)
            .map_err(|_| (libc::EIO, LocalStorage, None))?
            .ino;
        drop(store);
        let _ = self.notifications.try_send(Invalidation::Inode(ino));
        Ok(())
    }
    pub fn ensure(&self, node: &Node) -> Result<(), i32> {
        self.ensure_cancellable(node, None)
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
                let ready = self
                    .store
                    .lock()
                    .unwrap()
                    .download_ready(&ticket, unix_seconds());
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
            let Some(retry_at) = retry_at else {
                return Err(error);
            };
            while unix_seconds() < retry_at {
                if self.download_abandoned(node, flight) {
                    self.store
                        .lock()
                        .unwrap()
                        .finish_download(&ticket)
                        .map_err(|_| libc::EIO)?;
                    return Ok(());
                }
                if std::time::Instant::now() >= retry_deadline {
                    return Err(libc::EAGAIN);
                }
                if !self.connection().is_connected() {
                    return Err(libc::ENETDOWN);
                }
                // No flight-map/database locks while waiting. Other readers
                // still attach to this flight and can cancel independently.
                std::thread::sleep(Duration::from_millis(25));
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
        if self
            .store
            .lock()
            .unwrap()
            .open_cached(&node.id)
            .is_ok_and(|h| h.version() == node.version)
        {
            return Ok(());
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
            let _ = self.notifications.try_send(Invalidation::Inode(record.ino));
        }
        if let Some(parent) = result.node.parent.as_deref() {
            if let Ok(record) = store.record(parent) {
                let _ = self.notifications.try_send(Invalidation::Inode(record.ino));
                let _ = self
                    .notifications
                    .try_send(Invalidation::Entry(record.ino, result.old_name.clone()));
                let _ = self
                    .notifications
                    .try_send(Invalidation::Entry(record.ino, result.node.name.clone()));
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
            let _ = self.notifications.try_send(Invalidation::Inode(record.ino));
            if let Some(parent) = record.node.parent.as_deref() {
                if let Ok(parent) = store.record(parent) {
                    // Resolving an unbound creation exposes a different object
                    // at the old name. Invalidate that dentry as well as the copy.
                    let _ = self
                        .notifications
                        .try_send(Invalidation::Entry(parent.ino, record.node.name.clone()));
                }
            }
        }
        if let Some(parent) = result.copy.parent.as_deref() {
            if let Ok(record) = store.record(parent) {
                let _ = self.notifications.try_send(Invalidation::Inode(record.ino));
                let _ = self
                    .notifications
                    .try_send(Invalidation::Entry(record.ino, result.copy.name.clone()));
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
    pub fn upload_loop(&self, stop: &std::sync::atomic::AtomicBool) {
        self.signal_upload();
        while !stop.load(Ordering::Acquire) {
            let mut wake = self.upload_signal.0.lock().unwrap();
            if !*wake && !stop.load(Ordering::Acquire) {
                wake = self
                    .upload_signal
                    .1
                    .wait_timeout(wake, Duration::from_secs(1))
                    .unwrap()
                    .0;
            }
            if stop.load(Ordering::Acquire) {
                break;
            }
            let signaled = *wake;
            *wake = false;
            drop(wake);
            // Only durable retry deadlines are checked here. No NAS enumeration.
            let retry_due = self
                .store
                .lock()
                .unwrap()
                .next_download_retry()
                .ok()
                .flatten()
                .is_some_and(|at| at <= unix_seconds());
            if !signaled && !retry_due {
                continue;
            }
            if !self.connection().is_connected() {
                continue;
            }
            if let Err(error) = self
                .upload_pending(true)
                .and_then(|_| self.namespace_pending())
                .and_then(|_| self.upload_pending(false))
            {
                if error.kind() != io::ErrorKind::NotConnected {
                    *self.last_error.lock().unwrap() =
                        Some("upload stopped; local snapshots retained".into());
                }
            }
            // Pin actions must not wait for a remote event to wake hydration.
            if self.connection().is_connected() {
                self.refresh_cached();
            }
        }
    }
    /// Reconciliation sends only read queries. Unknown outcomes never re-upload.
    pub fn upload_pending(&self, reconcile: bool) -> io::Result<usize> {
        let _mutation = self.recovery_gate.read().unwrap();
        use sha2::Digest;
        use std::io::{Read, Seek, SeekFrom};
        let rpc = self.connection();
        if !rpc.is_connected() {
            return Err(io::Error::new(
                io::ErrorKind::NotConnected,
                "worker offline; queue retained",
            ));
        }
        let operations = self
            .store
            .lock()
            .unwrap()
            .upload_operations(if reconcile { "uncertain" } else { "pending" })
            .map_err(io::Error::other)?;
        let mut confirmed = 0;
        for op in operations {
            let (mut job, scope) = {
                let s = self.store.lock().unwrap();
                (
                    s.upload_job(&op).map_err(io::Error::other)?,
                    s.scope_path().map_err(io::Error::other)?,
                )
            };
            let mut h = sha2::Sha256::new();
            let mut buf = [0; 65536];
            loop {
                let n = job.file.read(&mut buf)?;
                if n == 0 {
                    break;
                }
                h.update(&buf[..n]);
            }
            job.file.seek(SeekFrom::Start(0))?;
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
                let prepared = rpc.call(
                    Command::PrepareUpload {
                        node: job.node.clone(),
                        parent,
                        path,
                        sha256: sha256.clone(),
                        cursor,
                    },
                    Duration::from_secs(120),
                )?;
                match prepared.body {
                    Response::UploadPrepared { origin } => {
                        // The operation is still pending: a lost preparation reply
                        // is safe to repeat because no mutation was sent.
                        if self
                            .store
                            .lock()
                            .unwrap()
                            .prepare_upload_origin(&op, &origin, &sha256)
                            .is_err()
                        {
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
                    _ => continue,
                }
            }
            let (node, path, origin) = {
                let mut s = self.store.lock().unwrap();
                let fresh = s.upload_job(&op).map_err(io::Error::other)?;
                if !reconcile && s.mark_upload_started(&op).is_err() {
                    continue;
                }
                (
                    fresh.node,
                    format!("{}/{}", scope.trim_end_matches('/'), fresh.relative_path),
                    s.upload_origin(&op).map_err(io::Error::other)?,
                )
            };
            let outcome = rpc.call_with_descriptor(
                Command::Upload {
                    origin,
                    node,
                    path,
                    sha256,
                    reconcile,
                },
                Duration::from_secs(1800),
                Some(job.file.into()),
            );
            let mut s = self.store.lock().unwrap();
            match outcome {
                Ok(reply) => match reply.body {
                    Response::Created { node, remote } => {
                        s.confirm_upload_identity(&op, &node.version, Some(&remote))
                            .map_err(io::Error::other)?;
                        confirmed += 1;
                    }
                    Response::Committed { node } => {
                        s.confirm_upload(&op, &node.version)
                            .map_err(io::Error::other)?;
                        confirmed += 1;
                    }
                    Response::Conflict => {
                        s.conflict_upload(&op).map_err(io::Error::other)?;
                    }
                    _ => {
                        if !reconcile {
                            s.mark_upload_uncertain(&op).map_err(io::Error::other)?;
                        }
                    }
                },
                Err(_) => {
                    if !reconcile {
                        s.mark_upload_uncertain(&op).map_err(io::Error::other)?;
                    }
                    return Err(io::Error::other("commit unknown; reconciliation required"));
                }
            }
        }
        Ok(confirmed)
    }
}

#[cfg(test)]
#[path = "upload_tests.rs"]
mod upload_tests;

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
        if matches!(category.as_str(), "native_index_capacity" | "native_response_capacity") {
            return io::Error::new(io::ErrorKind::InvalidData, category.clone());
        }
    }
    if matches!(&body, Response::Failed { category } if category == "native_root_unavailable") {
        return io::Error::new(io::ErrorKind::InvalidData, RootUnavailable);
    }
    if let Response::Failed { category } = &body {
        let timeout = match category.as_str() {
            "native_receive_idle" => Some(Cause::NativeReceiveIdle),
            "native_notification_setup_timeout" => Some(Cause::NativeSetupTimeout),
            "native_send_timeout" => Some(Cause::NativeSendTimeout),
            "native_response_timeout" => Some(Cause::NativeResponseTimeout),
            _ => None,
        };
        if let Some(cause) = timeout { return io::Error::new(io::ErrorKind::TimedOut, cause); }
        if let Some(code) = category.strip_prefix("native_server_code:").and_then(|code| code.parse::<u32>().ok()) {
            return io::Error::new(io::ErrorKind::InvalidData, Cause::NativeServerRejected(code));
        }
        if category == "session_expired" { return io::Error::new(io::ErrorKind::NotConnected, Cause::SessionExpired); }
        if category == "native_cursor_reset" { return io::Error::new(io::ErrorKind::InvalidData, Cause::NativeCursorReset); }
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
mod connection_diagnostic_categories {
    use super::*;
    use crate::connection_diagnostics::{Cause, retryable};
    #[test]
    fn staged_timeouts_preserve_event_and_download_retry_semantics() {
        for (category, expected) in [
            ("native_receive_idle", Cause::NativeReceiveIdle),
            ("native_notification_setup_timeout", Cause::NativeSetupTimeout),
            ("native_send_timeout", Cause::NativeSendTimeout),
            ("native_response_timeout", Cause::NativeResponseTimeout),
        ] {
            let error = operation_error(Response::Failed { category: category.into() });
            assert!(retryable(&error));
            assert_eq!(Cause::from_io(&error), expected);
            assert_eq!(download_failure_category(category), (DownloadFailureKind::Transport, None));
        }
        let error = operation_error(Response::Failed { category: "native_server_code:32772".into() });
        assert!(!retryable(&error));
        assert_eq!(Cause::from_io(&error), Cause::NativeServerRejected(32772));
        let error = operation_error(Response::Failed { category: "synthetic-private-input".into() });
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
            "transport" | "native_receive_idle" | "native_notification_setup_timeout"
                | "native_send_timeout" | "native_response_timeout" => Transport,
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
            let store = Arc::new(Mutex::new(store));
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
    fn retry_roundtrip(foreground: bool, failure_category: &'static str) {
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
        let store = Arc::new(Mutex::new(store));
        let (a, mut b) = UnixStream::pair().unwrap();
        b.set_read_timeout(Some(Duration::from_secs(10))).unwrap();
        let bridge = Arc::new(
            Bridge::from_connection(store.clone(), Arc::new(Rpc::new(a).unwrap())).unwrap(),
        );
        let peer = std::thread::spawn(move || {
            let (request, _) = transport::receive::<Envelope<Command>>(&mut b).unwrap();
            assert!(matches!(request.body, Command::Download { .. }));
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
            let first = Instant::now();
            let (request, _) = transport::receive::<Envelope<Command>>(&mut b).unwrap();
            assert!(first.elapsed() >= Duration::from_millis(900));
            assert!(matches!(request.body, Command::Download { .. }));
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
