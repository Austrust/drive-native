//! Long-lived local state owner. Network authentication and event waiting run
//! outside the desktop bus; credentials stay in memory until a keyring unlock.
use crate::{
    bridge::{Bridge, WorkerProcess},
    rpc::Rpc,
    wire::Login,
};
use drive_core::{Store, native::Scope};
use serde::{Deserialize, Serialize};
use std::{
    fs, io,
    os::unix::fs::{DirBuilderExt, MetadataExt, OpenOptionsExt},
    path::{Path, PathBuf},
    sync::{
        Arc, Condvar, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};
#[cfg(test)]
use std::sync::atomic::{AtomicU64, AtomicUsize};

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Profile {
    pub schema: u16,
    pub task_id: String,
    pub host: String,
    pub tls_name: String,
    #[serde(default)]
    pub trusted_ca_pem: Vec<String>,
    pub username: String,
    pub device_id: String,
    pub scope: Scope,
    pub mount: PathBuf,
    pub writable: bool,
    #[serde(default)]
    pub local_policy: Option<drive_core::write_permissions::LocalPolicy>,
    #[serde(default)]
    pub writable_source: Option<String>,
    /// Issue #50: task-scoped exclusion patterns matched against POSIX paths
    /// relative to `scope.path`. Absent in old profiles and therefore empty.
    #[serde(default)]
    pub exclude_patterns: Vec<String>,
}
impl Profile {
    pub fn effective_local_policy(&self) -> drive_core::write_permissions::LocalPolicy {
        self.local_policy.unwrap_or(if self.writable {
            drive_core::write_permissions::LocalPolicy::FollowRemote
        } else { drive_core::write_permissions::LocalPolicy::ExplicitReadOnly })
    }
    /// Pre-compile the profile's exclusion patterns. An empty list is a cheap
    /// constant-time no-op at match time.
    pub fn exclude_rules(&self) -> io::Result<drive_core::exclude::ExcludeRules> {
        drive_core::exclude::ExcludeRules::compile(&self.exclude_patterns)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error.to_string()))
    }
    pub fn read(path: &Path) -> io::Result<Self> {
        use std::io::Read;
        let file = fs::OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NOFOLLOW)
            .open(path)?;
        let metadata = file.metadata()?;
        if !metadata.is_file()
            || metadata.uid() != unsafe { libc::geteuid() }
            || metadata.mode() & 0o077 != 0
            || metadata.len() > 65536
        {
            return Err(io::Error::other(
                "profile must be an owned private regular file (0600), at most 64 KiB",
            ));
        }
        let mut text = String::new();
        file.take(65537).read_to_string(&mut text)?;
        if text.len() > 65536 {
            return Err(io::Error::other("profile exceeds size limit"));
        }
        let profile: Self =
            serde_json::from_str(&text).map_err(|_| io::Error::other("invalid profile format"))?;
        profile.validate()?;
        Ok(profile)
    }
    pub fn validate(&self) -> io::Result<()> {
        if self.schema != 1
            || uuid::Uuid::parse_str(&self.task_id).is_err()
            || uuid::Uuid::parse_str(&self.device_id).is_err()
            || [&self.host, &self.tls_name, &self.username]
                .iter()
                .any(|s| s.is_empty() || s.len() > 1024 || s.chars().any(char::is_control))
            || self.scope.file_id.is_empty()
            || self.scope.file_id.len() > 128
            || !self.scope.path.starts_with('/')
            || self.scope.path.contains('\0')
            || (self.scope.path != "/"
                && self.scope.path[1..]
                    .split('/')
                    .any(|p| matches!(p, "" | "." | "..")))
            || !self.mount.is_absolute()
            || self
                .mount
                .components()
                .any(|c| matches!(c, std::path::Component::ParentDir))
        {
            return Err(io::Error::other(
                "invalid profile identity, address, scope or mount",
            ));
        }
        sproto::trust::parse_trusted_pem(&self.trusted_ca_pem).map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "invalid extra trust certificates",
            )
        })?;
        if self.writable_source.as_deref().is_some_and(|s| !matches!(s,"user"|"legacy")) {
            return Err(io::Error::other("invalid local permission policy source"));
        }
        // Reject invalid regular expressions at the profile boundary so a typo
        // is reported with its pattern and compiler error instead of surfacing
        // as an undefined initialization failure.
        self.exclude_rules()?;
        if self.effective_local_policy() == drive_core::write_permissions::LocalPolicy::FollowRemote && !cfg!(feature = "native-research") {
            return Err(io::Error::other(
                "writable native protocol release gate is closed in this build",
            ));
        }
        Ok(())
    }
    fn login(&self, password: String, trusted_ca_pem: Vec<String>) -> Login {
        Login {
            host: self.host.clone(),
            tls_name: self.tls_name.clone(),
            trusted_ca_pem,
            username: self.username.clone(),
            password,
            device_id: self.device_id.clone(),
            scope: self.scope.clone(),
        }
    }
}

/// Do not follow a state-root or mount symlink, mount over existing contents,
/// or allow the mounted tree to cover its own durable database/content files.
pub fn prepare_paths(profile: &Profile, data_base: &Path) -> io::Result<PathBuf> {
    profile.validate()?;
    if unsafe { libc::geteuid() } == 0 {
        return Err(io::Error::other("run the client as an ordinary user"));
    }
    if !data_base.is_absolute() {
        return Err(io::Error::other("XDG data directory must be absolute"));
    }
    let state = data_base.join("drive-native").join(&profile.task_id);
    fs::DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(&state)?;
    let canonical = state.canonicalize()?;
    if canonical != state {
        return Err(io::Error::other(
            "state path must not contain symlink components",
        ));
    }
    let meta = state.metadata()?;
    if meta.uid() != unsafe { libc::geteuid() } || meta.mode() & 0o077 != 0 {
        return Err(io::Error::other(
            "state directory must be owned and private (0700)",
        ));
    }
    if !profile.mount.exists() {
        fs::DirBuilder::new()
            .recursive(true)
            .mode(0o700)
            .create(&profile.mount)?;
    }
    let mount = profile.mount.canonicalize()?;
    if mount != profile.mount
        || mount.starts_with(&state)
        || state.starts_with(&mount)
        || mount.metadata()?.uid() != unsafe { libc::geteuid() }
        || fs::read_dir(&mount)?.next().is_some()
    {
        return Err(io::Error::other(
            "mount must be an owned empty canonical directory, separate from state",
        ));
    }
    Ok(state)
}

#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConnectionState {
    Locked,
    Connecting,
    Online,
    Offline,
    Paused,
    NeedsAttention,
    Stopping,
}
#[derive(Debug, Serialize)]
pub struct RecoveryRequest {
    pub schema: u8,
    pub accepted: bool,
    pub paused_preserved: bool,
    pub message_code: &'static str,
}
/// Issue #61: origin of the current pause, reported additively as
/// `GetOverview.paused_by`. `None` covers a normally running session.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PauseOrigin {
    None,
    /// The user paused in this session.
    User,
    /// A fresh `startup-pause-v1.json` handover intent paused the new owner.
    StartupIntent,
    /// A persisted pause from a previous process lifetime was auto-resumed at
    /// startup; reported for the rest of this session as the last pause event.
    PersistedAutoResumed,
}

impl PauseOrigin {
    pub(crate) fn as_str(self) -> Option<&'static str> {
        match self {
            Self::None => None,
            Self::User => Some("user"),
            Self::StartupIntent => Some("startup_intent"),
            Self::PersistedAutoResumed => Some("persisted_flag_auto_resumed"),
        }
    }
}

pub struct Control {
    pub state: ConnectionState,
    pub error: Option<&'static str>,
    pub revision: u64,
    generation: u64,
    pub(crate) paused: bool,
    /// Issue #61: why `paused` is set (or why the last startup pause was
    /// auto-resumed). Never persisted; a process lifetime concept only.
    pub(crate) pause_origin: PauseOrigin,
    attention: bool,
    password: Option<String>,
    active: Option<Arc<Rpc>>,
    trust: crate::trust::TrustSettings,
    trust_pending_durability: bool,
}
/// Memory-only snapshot of one background release job. It lives only for the
/// lifetime of the service process: a restart has no job to resume or report.
pub(crate) struct ReleaseJob {
    pub(crate) id: String,
    pub(crate) total: usize,
    pub(crate) done: usize,
    pub(crate) released_count: usize,
    pub(crate) skipped_count: usize,
    pub(crate) protected_count: usize,
    pub(crate) released: Vec<String>,
    pub(crate) skipped: Vec<String>,
    pub(crate) protected: Vec<String>,
    pub(crate) reclaimed_bytes: u64,
    pub(crate) finished: bool,
    pub(crate) cancelled: bool,
    pub(crate) error: Option<String>,
}
pub(crate) struct ReleaseState {
    pub(crate) seq: u64,
    pub(crate) job: Option<ReleaseJob>,
    /// Cancellation flag of the running job; cloning it out keeps the worker
    /// independent of the state lock while it reclaims files.
    pub(crate) cancel: Option<Arc<AtomicBool>>,
}
pub struct Service {
    pub profile: Profile,
    pub bridge: Arc<Bridge>,
    /// Committed desktop status projection published off the global lock.
    status_cache: std::sync::OnceLock<Arc<crate::status_cache::StatusCache>>,
    /// Expensive overview counters keyed by the committed Store generation;
    /// only read and written off the Store lock (issue #21).
    overview_cache: Mutex<Option<crate::desktop::OverviewDerived>>,
    pub control: Mutex<Control>,
    changed: Condvar,
    pub stop: AtomicBool,
    /// Single-slot background release job polled through the desktop bus.
    pub(crate) release: Mutex<ReleaseState>,
    /// Test-only spacing between released files, so progress and cancel can be
    /// observed without racing a job that finishes instantly.
    #[cfg(test)]
    pub(crate) release_step_delay: AtomicU64,
    /// Set by `cache_policy_changed()` so a policy edit runs a trim even when
    /// it did not change `local_revision` (the cache limit lives in metadata).
    cache_policy_dirty: AtomicBool,
    /// Test-only observation of how many trim passes ran.
    #[cfg(test)]
    cache_trim_runs: AtomicUsize,
    pub mounted: AtomicBool,
    /// Issue #61: structured startup pause decision, exposed additively in
    /// `GetOverview.startup_pause`. `None` for a store that could not be read.
    startup_pause: Mutex<Option<drive_core::task_control::StartupPauseRecovery>>,
    pub shutdown_diagnostics: crate::shutdown::Diagnostics,
    // Keep the matching worker inode alive across atomic package replacement.
    worker_image: Option<fs::File>,
    pub(crate) diagnostics: crate::connection_diagnostics::Recorder,
}
pub struct Tasks {
    supervisor: std::thread::JoinHandle<()>,
    uploads: std::thread::JoinHandle<()>,
    cache: std::thread::JoinHandle<()>,
    status: std::thread::JoinHandle<()>,
}
impl Tasks {
    pub fn join(self) {
        let _ = self.supervisor.join();
        let _ = self.uploads.join();
        let _ = self.cache.join();
        let _ = self.status.join();
    }
}
/// How often the cache worker wakes when nothing notifies it.
const CACHE_LOOP_INTERVAL: Duration = Duration::from_secs(15);
/// Minimum spacing between trim passes that are only triggered by a new local
/// revision. Every write bumps the revision, so trimming per revision would
/// re-run `cache_candidates()` after each operation; the policy/limit changes
/// and over-limit pressure paths bypass this debounce.
const CACHE_TRIM_MIN_INTERVAL: Duration = Duration::from_secs(60);

#[derive(Default)]
struct CacheLoopState {
    last_revision: Option<u64>,
    last_limit: Option<u64>,
    last_trim: Option<Instant>,
    pressure: bool,
}

enum CacheTick {
    Skipped,
    Ran(Option<&'static str>),
}

impl Service {
    pub fn new(profile: Profile, store: Store, worker: PathBuf) -> io::Result<Arc<Self>> {
        Self::new_with_trust_sync(
            profile,
            store,
            worker,
            crate::trust::TrustSettings::confirm_durable,
        )
    }
    fn new_with_trust_sync(
        profile: Profile,
        mut store: Store,
        worker: PathBuf,
        sync: impl Fn(&Path) -> io::Result<()>,
    ) -> io::Result<Arc<Self>> {
        profile.validate()?;
        // Issue #61: a pause is a session control, not a durable preference.
        // Consume a fresh one-shot handover intent exactly once; otherwise
        // auto-resume a pause persisted by a previous process lifetime (stop,
        // crash, kill -9) and report the decision. An explicit pause issued
        // later in this session is never touched here.
        let startup_pause = store
            .reconcile_startup_pause(&profile.task_id)
            .map_err(io::Error::other)?;
        let paused = startup_pause.paused;
        let pause_origin = if paused && startup_pause.maintenance_intent {
            PauseOrigin::StartupIntent
        } else if startup_pause.auto_resumed {
            PauseOrigin::PersistedAutoResumed
        } else {
            PauseOrigin::None
        };
        let trust = crate::trust::TrustSettings::read(
            store.storage_directory(),
            &profile.task_id,
            &profile.trusted_ca_pem,
        )?;
        let worker_image = fs::File::open(&worker).ok();
        drive_core::write_permissions::initialize_policy(&store, &profile.scope.file_id,
            profile.effective_local_policy(), profile.writable_source.as_deref().unwrap_or("legacy"))
            .map_err(io::Error::other)?;
        let trust_pending_durability = sync(store.storage_directory()).is_err();
        let diagnostics = crate::connection_diagnostics::Recorder::open(store.storage_directory());
        let bridge = Arc::new(Bridge::offline(Arc::new(drive_core::SharedStore::new(
            store,
        )))?);
        bridge.enable_initialization_owner(&profile.task_id)?;
        // Issue #50: compile the exclusion rules once and share the state with
        // both the initialization owner (import pruning) and the mount (read
        // path filtering). Empty rules never build or consult an index.
        let exclusion = Arc::new(drive_core::exclude::ExclusionState::new(
            profile.exclude_rules()?,
        ));
        bridge.set_exclusion_state(exclusion);
        Ok(Arc::new(Self {
            profile,
            bridge,
            status_cache: std::sync::OnceLock::new(),
            overview_cache: Mutex::new(None),
            control: Mutex::new(Control {
                state: if paused {
                    ConnectionState::Paused
                } else if trust_pending_durability {
                    ConnectionState::NeedsAttention
                } else {
                    ConnectionState::Locked
                },
                error: trust_pending_durability.then_some("trust_settings_not_durable"),
                revision: 0,
                generation: 0,
                paused,
                pause_origin,
                attention: trust_pending_durability,
                password: None,
                active: None,
                trust,
                trust_pending_durability,
            }),
            changed: Condvar::new(),
            stop: AtomicBool::new(false),
            release: Mutex::new(ReleaseState {
                seq: 0,
                job: None,
                cancel: None,
            }),
            #[cfg(test)]
            release_step_delay: AtomicU64::new(0),
            cache_policy_dirty: AtomicBool::new(false),
            #[cfg(test)]
            cache_trim_runs: AtomicUsize::new(0),
            mounted: AtomicBool::new(false),
            shutdown_diagnostics: crate::shutdown::Diagnostics::default(),
            worker_image,
            startup_pause: Mutex::new(Some(startup_pause)),
            diagnostics,
        }))
    }
    fn worker_executable(&self) -> io::Result<PathBuf> {
        use std::os::fd::AsRawFd;
        self.worker_image
            .as_ref()
            .map(|image| PathBuf::from(format!("/proc/self/fd/{}", image.as_raw_fd())))
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "matching worker image unavailable; restart after repairing installation",
                )
            })
    }
    fn configure_transfers(
        rpc: &Arc<Rpc>,
        limits: drive_core::transfer_policy::TransferLimits,
    ) -> io::Result<()> {
        let reply = rpc.call(
            crate::wire::Command::ConfigureTransfers(limits),
            Duration::from_secs(2),
        )?;
        match reply.body {
            crate::wire::Response::TransfersConfigured(applied) if applied == limits => Ok(()),
            _ => Err(io::Error::other("transfer policy was not acknowledged")),
        }
    }
    pub fn set_transfer_limits(
        &self,
        expected: &str,
        limits: drive_core::transfer_policy::TransferLimits,
    ) -> io::Result<()> {
        limits.validate().map_err(io::Error::other)?;
        let mut c = self.control.lock().unwrap();
        if self.stop.load(Ordering::Acquire) {
            return Err(io::Error::other("mount is stopping"));
        }
        let mut store = self.bridge.store.lock().unwrap();
        if store
            .transfer_limits()
            .map_err(io::Error::other)?
            .fingerprint()
            != expected
        {
            return Err(io::Error::other("transfer settings changed"));
        }
        store
            .set_transfer_limits(limits)
            .map_err(io::Error::other)?;
        drop(store);
        if c.state == ConnectionState::Online {
            let result = c
                .active
                .as_ref()
                .ok_or_else(|| io::Error::other("worker absent"))
                .and_then(|rpc| Self::configure_transfers(rpc, limits));
            if result.is_err() {
                c.state = ConnectionState::Offline;
                c.error = Some("network_unavailable");
                self.interrupt(&mut c);
                return Err(io::Error::other(
                    "settings saved; worker application pending reconnection",
                ));
            }
        } else if matches!(
            c.state,
            ConnectionState::Connecting | ConnectionState::Offline
        ) {
            // No online transfer is interrupted; the next authenticated worker
            // must receive the latest persisted settings before becoming active.
            self.interrupt(&mut c);
        }
        c.revision += 1;
        self.changed.notify_all();
        Ok(())
    }
    pub fn trust_settings(&self) -> crate::trust::TrustSettings {
        self.control.lock().unwrap().trust.clone()
    }
    pub fn set_trust_settings(
        &self,
        expected: &str,
        settings: crate::trust::TrustSettings,
    ) -> io::Result<()> {
        self.set_trust_settings_with(expected, settings, |settings, directory| {
            settings.publish(directory)
        })
    }
    fn set_trust_settings_with(
        &self,
        expected: &str,
        settings: crate::trust::TrustSettings,
        publish: impl FnOnce(
            &crate::trust::TrustSettings,
            &Path,
        ) -> io::Result<crate::trust::Publication>,
    ) -> io::Result<()> {
        settings.validate(&self.profile.task_id)?;
        let mut c = self.control.lock().unwrap();
        if self.stop.load(Ordering::Acquire) || c.trust.fingerprint() != expected {
            return Err(io::Error::other(
                "trust settings changed or service stopping",
            ));
        }
        let store = self.bridge.store.lock().unwrap();
        let publication = publish(&settings, store.storage_directory())?;
        drop(store);
        c.trust = settings;
        c.trust_pending_durability = publication == crate::trust::Publication::VisibleNotDurable;
        if c.trust_pending_durability {
            c.attention = true;
            c.error = Some("trust_settings_not_durable");
            c.state = if c.paused {
                ConnectionState::Paused
            } else {
                ConnectionState::NeedsAttention
            };
            self.interrupt(&mut c);
            return Err(io::Error::other(
                "trust settings visible but durability unconfirmed; reload and save again",
            ));
        }
        c.attention = false;
        c.error = None;
        c.state = if c.paused {
            ConnectionState::Paused
        } else if c.password.is_some() {
            ConnectionState::Connecting
        } else {
            ConnectionState::Locked
        };
        self.interrupt(&mut c);
        Ok(())
    }
    pub fn start(self: &Arc<Self>) -> Tasks {
        let a = self.clone();
        let b = self.clone();
        let c = self.clone();
        let (status_cache, status_task) = crate::status_cache::StatusCache::start(self.clone());
        let _ = self.status_cache.set(status_cache);
        Tasks {
            supervisor: std::thread::spawn(move || a.supervise()),
            uploads: std::thread::spawn(move || b.bridge.upload_loop(&b.stop)),
            cache: std::thread::spawn(move || c.cache_loop()),
            status: status_task,
        }
    }
    /// Latest committed desktop status projection, if the cache is ready.
    ///
    /// Callers must treat `None` as "use the authoritative SQLite path"; the
    /// projection is only published after a complete committed read.
    pub(crate) fn status_projection(
        &self,
    ) -> Option<Arc<crate::status_cache::StatusProjection>> {
        self.status_cache.get().and_then(|cache| cache.snapshot())
    }
    /// Derived overview counters captured for one committed Store generation.
    pub(crate) fn overview_derived(
        &self,
        key: drive_core::shared_store::CommitState,
    ) -> Option<crate::desktop::OverviewDerived> {
        self.overview_cache
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .filter(|derived| derived.key == key)
    }
    /// Publish derived counters for a generation this projection just read.
    pub(crate) fn cache_overview_derived(&self, derived: crate::desktop::OverviewDerived) {
        *self
            .overview_cache
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(derived);
    }
    pub fn cache_policy_changed(&self) {
        self.cache_policy_dirty.store(true, Ordering::Release);
        self.control.lock().unwrap().revision += 1;
        self.changed.notify_all();
    }
    /// Slow down each released file so tests can observe a running job.
    #[cfg(test)]
    pub(crate) fn set_release_step_delay(&self, ms: u64) {
        self.release_step_delay.store(ms, Ordering::Release);
    }
    /// Issue #61: structured startup pause decision for this process. Exposed
    /// additively as `GetOverview.startup_pause`; `None` only if this service
    /// was built without a readable store decision (tests/legacy fixtures).
    pub fn startup_pause_recovery(&self) -> Option<drive_core::task_control::StartupPauseRecovery> {
        self.startup_pause.lock().unwrap().clone()
    }

    fn cache_loop(&self) {
        self.cache_loop_with(CACHE_LOOP_INTERVAL);
    }
    fn cache_loop_with(&self, interval: Duration) {
        let mut state = CacheLoopState::default();
        loop {
            let control = self.control.lock().unwrap();
            if self.stop.load(Ordering::Acquire) {
                return;
            }
            let control = self.changed.wait_timeout(control, interval).unwrap().0;
            if self.stop.load(Ordering::Acquire) {
                return;
            }
            let online = control.state == ConnectionState::Online;
            drop(control);
            if !online {
                continue;
            }
            let outcome = self.cache_tick(&mut state);
            let error = match outcome {
                CacheTick::Skipped => continue,
                CacheTick::Ran(error) => error.map(str::to_owned),
            };
            let changed = {
                let mut current = self.bridge.cache_error.lock().unwrap();
                let changed = *current != error;
                *current = error;
                changed
            };
            if changed {
                self.control.lock().unwrap().revision += 1;
            }
        }
    }
    /// One cache decision. Trimming is skipped unless the policy changed, the
    /// limit changed, the cache is over target, or a new local revision arrived
    /// outside the debounce window. `cache_policy_changed()` must take effect
    /// immediately, so it is never debounced.
    fn cache_tick(&self, state: &mut CacheLoopState) -> CacheTick {
        let policy_dirty = self.cache_policy_dirty.swap(false, Ordering::AcqRel);
        let now = Instant::now();
        let (policy, revision) = {
            let st = self.bridge.store.lock().unwrap();
            (st.cache_limit(), st.local_revision())
        };
        let limit = match policy {
            Ok(0) => {
                state.last_revision = None;
                state.last_limit = Some(0);
                state.pressure = false;
                return CacheTick::Skipped;
            }
            Ok(limit) => limit,
            Err(_) => {
                state.last_limit = None;
                return CacheTick::Ran(Some("cache_maintenance_failed"));
            }
        };
        let limit_changed = state.last_limit != Some(limit);
        let revision_changed = state.last_revision != Some(revision);
        let debounced = state
            .last_trim
            .is_some_and(|last| now.duration_since(last) < CACHE_TRIM_MIN_INTERVAL);
        if !policy_dirty && !limit_changed && !state.pressure && !(revision_changed && !debounced) {
            return CacheTick::Skipped;
        }
        #[cfg(test)]
        self.cache_trim_runs.fetch_add(1, Ordering::Relaxed);
        match self.bridge.maintain_cache() {
            Ok(result) => {
                state.last_revision = Some(result.revision);
                state.last_limit = Some(limit);
                state.pressure = result.usage.over_target_bytes > 0;
                state.last_trim = Some(now);
                CacheTick::Ran(None)
            }
            Err(_) => CacheTick::Ran(Some("cache_maintenance_failed")),
        }
    }
    fn interrupt(&self, c: &mut Control) {
        c.generation += 1;
        c.revision += 1;
        if let Some(rpc) = &c.active {
            rpc.close();
        }
        self.bridge.disconnect();
        self.changed.notify_all();
    }
    pub fn unlock(&self, password: String) -> io::Result<()> {
        if password.is_empty() || password.len() > 16384 {
            return Err(io::Error::other("invalid credential length"));
        }
        let mut c = self.control.lock().unwrap();
        if self.stop.load(Ordering::Acquire) {
            return Err(io::Error::other("mount is stopping"));
        }
        if c.trust_pending_durability {
            return Err(io::Error::other(
                "trust settings require a successful save before unlocking",
            ));
        }
        c.password = Some(password);
        c.attention = false;
        c.error = None;
        c.state = if c.paused {
            ConnectionState::Paused
        } else {
            ConnectionState::Connecting
        };
        self.interrupt(&mut c);
        Ok(())
    }
    pub fn pause(&self) -> io::Result<()> {
        let mut c = self.control.lock().unwrap();
        if self.stop.load(Ordering::Acquire) {
            return Err(io::Error::other("mount is stopping"));
        }
        self.bridge
            .store
            .lock()
            .unwrap()
            .set_sync_paused(true)
            .map_err(io::Error::other)?;
        c.paused = true;
        c.pause_origin = PauseOrigin::User;
        c.state = ConnectionState::Paused;
        self.interrupt(&mut c);
        Ok(())
    }
    pub fn resume(&self) -> io::Result<()> {
        let mut c = self.control.lock().unwrap();
        if self.stop.load(Ordering::Acquire) {
            return Err(io::Error::other("mount is stopping"));
        }
        self.bridge
            .store
            .lock()
            .unwrap()
            .set_sync_paused(false)
            .map_err(io::Error::other)?;
        c.paused = false;
        c.pause_origin = PauseOrigin::None;
        drive_core::write_permissions::reset_session(&self.bridge.store.lock().unwrap()).map_err(io::Error::other)?;
        // Authentication/certificate/recovery failures require a fresh explicit
        // unlock; pause/resume must not silently retry rejected credentials.
        c.state = if c.attention {
            ConnectionState::NeedsAttention
        } else if c.password.is_none() {
            ConnectionState::Locked
        } else {
            ConnectionState::Connecting
        };
        self.interrupt(&mut c);
        Ok(())
    }
    /// Explicit recovery request: preserve pause/trust/authentication gates and
    /// all durable intents. Online recovery wakes the usual reconciliation
    /// path; it never resets uncertain operations to pending. Independent
    /// pending writes may proceed normally after that read-only reconciliation.
    pub fn recheck_issues(&self) -> io::Result<RecoveryRequest> {
        let result = |code, accepted| RecoveryRequest {
            schema: 1,
            accepted,
            paused_preserved: true,
            message_code: code,
        };
        // Maintenance and durable pause may temporarily own Control while
        // waiting for Store. A recovery button must not queue behind that
        // work: report busy without changing any gate or persistent intent.
        let mut c = match self.control.try_lock() {
            Ok(control) => control,
            Err(std::sync::TryLockError::WouldBlock) => return Ok(result("busy", false)),
            Err(std::sync::TryLockError::Poisoned(_)) => {
                return Err(io::Error::other("local control state unavailable"));
            }
        };
        if self.stop.load(Ordering::Acquire) {
            return Ok(result("stopping", false));
        }
        if c.paused {
            return Ok(result("paused", false));
        }
        if c.trust_pending_durability
            || matches!(c.error, Some("certificate_or_configuration" | "worker_image_unavailable"))
        {
            return Ok(result("settings_required", false));
        }
        if c.password.is_none() || c.error == Some("authentication_or_protocol_rejected") {
            return Ok(result("login_required", false));
        }
        if c.attention && !matches!(c.error, Some("native_state_recovery_required" | "native_root_unavailable")) {
            return Ok(result("settings_required", false));
        }
        if matches!(c.state, ConnectionState::Online | ConnectionState::Connecting) && !c.attention {
            let fresh = self.bridge.request_issue_recheck();
            if fresh { c.revision += 1; }
            return Ok(result(if fresh { "queued" } else { "already_queued" }, true));
        }
        // Existing credentials only; supervisor performs authentication outside
        // this method. It resumes persisted candidates/cursors, not a forced
        // clear-cache or unconditional full re-index.
        c.attention = false;
        c.error = None;
        c.state = ConnectionState::Connecting;
        self.interrupt(&mut c);
        self.bridge.request_issue_recheck();
        Ok(result("reconnecting", true))
    }
    pub fn shutdown(&self) {
        self.shutdown_diagnostics
            .phase(crate::shutdown::Phase::Requested);
        let mut c = self.control.lock().unwrap();
        self.stop.store(true, Ordering::Release);
        c.password = None;
        c.state = ConnectionState::Stopping;
        self.interrupt(&mut c);
        self.bridge.signal_upload();
    }
    pub fn mount_if_ready(&self) -> io::Result<Option<fuser::BackgroundSession>> {
        if self.stop.load(Ordering::Acquire) {
            return Ok(None);
        }
        // A preview-only root is mountable before the committed namespace
        // exists (initialization preview). The exclusion index is built from the
        // committed `nodes` table, so it cannot be built in that phase; the
        // bridge rebuilds it after initialization commits or a recovery
        // snapshot installs.
        let mut committed_root = true;
        {
            let store = self.bridge.store.lock().unwrap();
            match store.node(&self.profile.scope.file_id) {
                Ok(_) => {}
                Err(e) if e.is_not_found() => {
                    match store.preview_mount_root(&self.profile.scope.file_id) {
                        Ok(Some(_)) => committed_root = false,
                        Ok(None) => return Ok(None),
                        Err(drive_core::Error::Io(e))
                            if matches!(e.raw_os_error(), Some(libc::EACCES | libc::EAGAIN)) =>
                        {
                            return Ok(None);
                        }
                        Err(_) => {
                            return Err(io::Error::other(
                                "initialization preview root cannot be loaded",
                            ));
                        }
                    }
                }
                Err(_) => return Err(io::Error::other("cached root cannot be loaded")),
            }
        }
        // Issue #50 (phase 4): reconcile the committed namespace with the active
        // exclusion rules before the mount becomes visible, so the FUSE inode
        // map is backfilled from the reconciled namespace. With rules, excluded
        // subtrees are hidden and their local cache/scheduling state is dropped
        // (never a remote delete); without rules, rows hidden by a previous rule
        // set come back. Preview mounts have no committed rows yet.
        let exclusion = self.bridge.exclusion_state();
        if committed_root {
            {
                let store = self.bridge.store.lock().unwrap();
                refresh_exclusion_before_mount(
                    &exclusion,
                    &store,
                    &self.profile.scope.file_id,
                    true,
                )?;
            }
            if exclusion.is_active() {
                self.bridge.purge_excluded_from_store()?;
                // The purge changed the committed namespace; rebuild before the
                // mount and the inode backfill observe it.
                let store = self.bridge.store.lock().unwrap();
                refresh_exclusion_before_mount(
                    &exclusion,
                    &store,
                    &self.profile.scope.file_id,
                    true,
                )?;
            } else {
                self.bridge.restore_excluded_from_store()?;
            }
        }
        // One hub owns the publication gate and invalidation delivery for both
        // the FUSE replies and the runtime's state-change notifications.
        let notify = drive_fs::NotifyHub::from_env();
        let mut fs = match drive_fs::CacheFilesystem::new(
            self.bridge.store.clone(),
            &self.profile.scope.file_id,
            unsafe { libc::getuid() },
            unsafe { libc::getgid() },
        ) {
            Ok(fs) => fs,
            // A candidate can be revoked between the readiness check and
            // constructor. Keep FSD alive to display/resolve the native error.
            Err(drive_core::Error::Io(e))
                if matches!(e.raw_os_error(), Some(libc::EACCES | libc::EAGAIN)) =>
            {
                return Ok(None);
            }
            Err(e) => return Err(io::Error::other(e)),
        }
        .with_hydrator(self.bridge.clone())
        .with_notify_hub(notify.clone())
        .with_exclusion_state(self.bridge.exclusion_state());
        // `new` has now backfilled inodes. Index construction is local metadata
        // only; previews, budget exhaustion and any load error keep the SQL path.
        if drive_fs::namespace_index_enabled() {
            let timing = drive_fs::diagnostics::span(drive_fs::diagnostics::Stage::IndexBuildUs);
            let index = {
                let store = self.bridge.store.lock().unwrap();
                drive_core::namespace_index::NamespaceIndex::build(&store)
            };
            drop(timing);
            match index {
                Ok(index) => fs = fs.with_namespace_index(Arc::new(index)),
                Err(_) => {
                    drive_fs::diagnostics::count(drive_fs::diagnostics::Stage::IndexDegraded);
                    fs = fs.with_namespace_index(Arc::new(drive_core::namespace_index::NamespaceIndex::degraded()));
                }
            }
        }
        let writable = drive_core::write_permissions::mount_writable(
            &self.bridge.store.lock().unwrap(), &self.profile.scope.file_id).map_err(io::Error::other)?;
        if writable && !cfg!(feature="native-research"){return Err(io::Error::other("writable native protocol release gate is closed"));}
        if writable {
            fs = fs.with_writes_enabled();
        }
        // The exclusion index was built (and rebuilt after the phase-4 purge)
        // above, before the FUSE inode map was backfilled; later namespace swaps
        // rebuild it from the bridge.
        let session = fuser::spawn_mount2(
            fs,
            &self.profile.mount,
            &[
                if writable {
                    fuser::MountOption::RW
                } else {
                    fuser::MountOption::RO
                },
                fuser::MountOption::DefaultPermissions,
                fuser::MountOption::NoDev,
                fuser::MountOption::NoSuid,
                fuser::MountOption::FSName("drive-native".into()),
            ],
        )?;
        notify.install(session.notifier());
        self.bridge.set_notify_hub(notify);
        self.mounted.store(true, Ordering::Release);
        self.control.lock().unwrap().revision += 1;
        Ok(Some(session))
    }
    fn supervise(&self) {
        use crate::connection_diagnostics::{ProcessState, Record, Stage};
        let mut delay = 1;
        // Explicit unlock/resume/trust changes create a new generation and
        // refresh quarantined permissions. A transport reconnect in the same
        // generation resumes the cursor instead of re-enumerating every denied
        // subtree. Cursor/restore invalidation still invokes snapshot recovery.
        let mut permissions_generation = None;
        loop {
            let (generation, password, trusted_ca_pem) = {
                let mut c = self.control.lock().unwrap();
                while !self.stop.load(Ordering::Acquire)
                    && (c.paused || c.attention || c.password.is_none())
                {
                    c = self.changed.wait(c).unwrap();
                }
                if self.stop.load(Ordering::Acquire) {
                    break;
                }
                c.state = ConnectionState::Connecting;
                c.revision += 1;
                (
                    c.generation,
                    c.password.as_ref().unwrap().clone(),
                    c.trust.trusted_ca_pem.clone(),
                )
            };
            let mut worker = None;
            self.bridge.initialization.begin();
            let mut stage = Stage::StartWorker;
            let started = std::time::Instant::now();
            let result = (|| -> io::Result<()> {
                worker = Some(
                    WorkerProcess::start(&self.worker_executable()?).map_err(|_| {
                        io::Error::new(io::ErrorKind::InvalidInput, "worker executable unavailable")
                    })?,
                );
                let worker = worker.as_ref().unwrap();
                {
                    let mut c = self.control.lock().unwrap();
                    if c.generation != generation {
                        return Ok(());
                    }
                    c.active = Some(worker.rpc.clone());
                }
                stage = Stage::Authenticate;
                self.bridge
                    .initialization
                    .phase(crate::initialization::Phase::Authenticating);
                let restore = Bridge::authenticate(
                    &worker.rpc,
                    self.profile.login(password, trusted_ca_pem),
                )?;
                self.bridge.initialization.activity();
                {
                    let c = self.control.lock().unwrap();
                    if c.generation != generation {
                        return Ok(());
                    }
                    stage = Stage::ConfigureTransfers;
                    self.bridge
                        .initialization
                        .phase(crate::initialization::Phase::Configuring);
                    let limits = self
                        .bridge
                        .store
                        .lock()
                        .unwrap()
                        .transfer_limits()
                        .map_err(io::Error::other)?;
                    Self::configure_transfers(&worker.rpc, limits)?;
                    self.bridge.initialization.activity();
                    stage = Stage::BindScope;
                    self.bridge
                        .initialization
                        .phase(crate::initialization::Phase::Binding);
                }
                // Acquiring the recovery writer may wait for an interrupted
                // upload to drain. Keep Control free so Pause/Stop can close RPC.
                self.bridge.attach(
                    worker.rpc.clone(),
                    &restore,
                    &serde_json::to_string(&self.profile.scope)?,
                )?;
                self.bridge.initialization.activity();
                if self.control.lock().unwrap().generation != generation {
                    self.bridge.disconnect();
                    return Ok(());
                }
                if permissions_generation == Some(generation) {
                    stage = Stage::ResumeEvents;
                    self.bridge.pull()?;
                } else {
                    stage = Stage::InitialIndex;
                    self.bridge.initial_pull()?;
                    permissions_generation = Some(generation);
                }
                {
                    let mut c = self.control.lock().unwrap();
                    if c.generation != generation {
                        return Ok(());
                    }
                    c.state = ConnectionState::Online;
                    c.error = None;
                    c.revision += 1;
                }
                delay = 1;
                self.bridge.event_loop_observed(|current| stage = current)
            })();
            self.bridge.initialization.end();
            let requested = self.stop.load(Ordering::Acquire)
                || self.control.lock().unwrap().generation != generation;
            let worker_pid = worker.as_ref().map(WorkerProcess::pid);
            let before_cleanup = worker
                .as_mut()
                .map(WorkerProcess::before_cleanup)
                .unwrap_or(ProcessState::NotStarted);
            // Save the reason and child state before close/kill/wait can obscure
            // the original failure. No control or Store lock spans diagnostic IO.
            self.diagnostics.record(Record::new(
                stage,
                &result,
                requested,
                started.elapsed(),
                worker_pid,
                before_cleanup,
            ));
            if let Some(mut worker) = worker {
                worker.stop();
            }
            let mut c = self.control.lock().unwrap();
            if c.generation != generation {
                continue;
            }
            c.active = None;
            self.bridge.disconnect();
            if let Err(error) = result {
                let retry = crate::connection_diagnostics::retryable(&error)
                    || crate::connection_diagnostics::recovery_cause(&error).is_some();
                c.attention = !retry;
                c.state = if retry {
                    ConnectionState::Offline
                } else {
                    ConnectionState::NeedsAttention
                };
                c.error = Some(if self.worker_image.is_none() {
                    "worker_image_unavailable"
                } else if let Some(category) = crate::bridge::capacity_error(&error) {
                    category
                } else if crate::bridge::is_root_unavailable(&error) {
                    "native_root_unavailable"
                } else {
                    match error.kind() {
                        _ if crate::connection_diagnostics::recovery_cause(&error).is_some() => {
                            // The runtime rebuilds this session automatically;
                            // it is a retry, never a request for human action.
                            "network_unavailable"
                        }
                        io::ErrorKind::PermissionDenied => "authentication_or_protocol_rejected",
                        io::ErrorKind::InvalidInput => "certificate_or_configuration",
                        io::ErrorKind::InvalidData => "native_state_recovery_required",
                        _ if retry => "network_unavailable",
                        _ => "local_or_protocol_error",
                    }
                });
                c.revision += 1;
            }
            if !c.attention {
                let _ = self
                    .changed
                    .wait_timeout_while(c, Duration::from_secs(delay), |c| {
                        c.generation == generation && !self.stop.load(Ordering::Acquire)
                    })
                    .unwrap();
                delay = (delay * 2).min(30);
            }
        }
    }
}

/// Build the exclusion index before the mount becomes visible.
///
/// `committed` is false while the mount is a preview-only root: the exclusion
/// index is derived from the committed `nodes` table, which does not exist yet,
/// so the preview keeps an empty index (the bridge refresh paths build it after
/// owner initialization, a recovery snapshot install, or a pull). In the
/// committed phase a failure stays fatal: swallowing it would leak excluded
/// paths into the mount view.
fn refresh_exclusion_before_mount(
    exclusion: &drive_core::exclude::ExclusionState,
    store: &drive_core::Store,
    root: &str,
    committed: bool,
) -> io::Result<()> {
    if !exclusion.is_active() || !committed {
        return Ok(());
    }
    exclusion.refresh(store, root).map_err(io::Error::other)
}

#[cfg(test)]
#[path = "trust_service_tests.rs"]
mod trust_service_tests;

#[cfg(test)]
#[path = "cache_loop_tests.rs"]
mod cache_loop_tests;

#[cfg(test)]
#[path = "mount_exclusion_tests.rs"]
mod mount_exclusion_tests;

#[cfg(test)]
#[path = "startup_pause_tests.rs"]
mod startup_pause_tests;
