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
    time::Duration,
};

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
}
impl Profile {
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
        if self.writable && !cfg!(feature = "native-research") {
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
pub struct Control {
    pub state: ConnectionState,
    pub error: Option<&'static str>,
    pub revision: u64,
    generation: u64,
    paused: bool,
    attention: bool,
    password: Option<String>,
    active: Option<Arc<Rpc>>,
    trust: crate::trust::TrustSettings,
    trust_pending_durability: bool,
}
pub struct Service {
    pub profile: Profile,
    pub bridge: Arc<Bridge>,
    pub control: Mutex<Control>,
    changed: Condvar,
    pub stop: AtomicBool,
    pub mounted: AtomicBool,
    worker: PathBuf,
}
pub struct Tasks {
    supervisor: std::thread::JoinHandle<()>,
    uploads: std::thread::JoinHandle<()>,
    cache: std::thread::JoinHandle<()>,
}
impl Tasks {
    pub fn join(self) {
        let _ = self.supervisor.join();
        let _ = self.uploads.join();
        let _ = self.cache.join();
    }
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
        store: Store,
        worker: PathBuf,
        sync: impl Fn(&Path) -> io::Result<()>,
    ) -> io::Result<Arc<Self>> {
        profile.validate()?;
        let trust = crate::trust::TrustSettings::read(
            store.storage_directory(),
            &profile.task_id,
            &profile.trusted_ca_pem,
        )?;
        let trust_pending_durability = sync(store.storage_directory()).is_err();
        Ok(Arc::new(Self {
            profile,
            bridge: Arc::new(Bridge::offline(Arc::new(Mutex::new(store)))?),
            control: Mutex::new(Control {
                state: if trust_pending_durability {
                    ConnectionState::NeedsAttention
                } else {
                    ConnectionState::Locked
                },
                error: trust_pending_durability.then_some("trust_settings_not_durable"),
                revision: 0,
                generation: 0,
                paused: false,
                attention: trust_pending_durability,
                password: None,
                active: None,
                trust,
                trust_pending_durability,
            }),
            changed: Condvar::new(),
            stop: AtomicBool::new(false),
            mounted: AtomicBool::new(false),
            worker,
        }))
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
        Tasks {
            supervisor: std::thread::spawn(move || a.supervise()),
            uploads: std::thread::spawn(move || b.bridge.upload_loop(&b.stop)),
            cache: std::thread::spawn(move || c.cache_loop()),
        }
    }
    pub fn cache_policy_changed(&self) {
        self.control.lock().unwrap().revision += 1;
        self.changed.notify_all();
    }
    fn cache_loop(&self) {
        let mut last_revision = None;
        let mut pressure = false;
        loop {
            let control = self.control.lock().unwrap();
            if self.stop.load(Ordering::Acquire) {
                return;
            }
            let control = self
                .changed
                .wait_timeout(control, Duration::from_secs(15))
                .unwrap()
                .0;
            if self.stop.load(Ordering::Acquire) {
                return;
            }
            let online = control.state == ConnectionState::Online;
            if !online {
                continue;
            }
            let (policy, revision) = {
                let st = self.bridge.store.lock().unwrap();
                (st.cache_limit(), st.local_revision())
            };
            if matches!(policy, Ok(0)) {
                last_revision = None;
                pressure = false;
                continue;
            }
            if policy.is_ok() && !pressure && last_revision == Some(revision) {
                continue;
            }
            let error = match policy {
                Ok(_) => match self.bridge.maintain_cache() {
                    Ok(result) => {
                        last_revision = Some(result.revision);
                        pressure = result.usage.over_target_bytes > 0;
                        None
                    }
                    Err(_) => Some("cache_maintenance_failed".to_owned()),
                },
                Err(_) => Some("cache_maintenance_failed".to_owned()),
            };
            drop(control);
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
    pub fn pause(&self) {
        let mut c = self.control.lock().unwrap();
        if self.stop.load(Ordering::Acquire) {
            return;
        }
        c.paused = true;
        c.state = ConnectionState::Paused;
        self.interrupt(&mut c);
    }
    pub fn resume(&self) {
        let mut c = self.control.lock().unwrap();
        if self.stop.load(Ordering::Acquire) {
            return;
        }
        c.paused = false;
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
    }
    pub fn shutdown(&self) {
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
        {
            let store = self.bridge.store.lock().unwrap();
            match store.node(&self.profile.scope.file_id) {
                Ok(_) => {}
                Err(e) if e.is_not_found() => return Ok(None),
                Err(_) => return Err(io::Error::other("cached root cannot be loaded")),
            }
        }
        let mut fs = drive_fs::CacheFilesystem::new(
            self.bridge.store.clone(),
            &self.profile.scope.file_id,
            unsafe { libc::getuid() },
            unsafe { libc::getgid() },
        )
        .map_err(io::Error::other)?
        .with_hydrator(self.bridge.clone());
        if self.profile.writable {
            fs = fs.with_writes_enabled();
        }
        let session = fuser::spawn_mount2(
            fs,
            &self.profile.mount,
            &[
                if self.profile.writable {
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
        self.bridge.set_notifier(session.notifier());
        self.mounted.store(true, Ordering::Release);
        self.control.lock().unwrap().revision += 1;
        Ok(Some(session))
    }
    fn supervise(&self) {
        let mut delay = 1;
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
            let result = (|| -> io::Result<()> {
                let mut worker = WorkerProcess::start(&self.worker).map_err(|_| {
                    io::Error::new(io::ErrorKind::InvalidInput, "worker executable unavailable")
                })?;
                {
                    let mut c = self.control.lock().unwrap();
                    if c.generation != generation {
                        return Ok(());
                    }
                    c.active = Some(worker.rpc.clone());
                }
                let restore = Bridge::authenticate(
                    &worker.rpc,
                    self.profile.login(password, trusted_ca_pem),
                )?;
                {
                    let c = self.control.lock().unwrap();
                    if c.generation != generation {
                        return Ok(());
                    }
                    let limits = self
                        .bridge
                        .store
                        .lock()
                        .unwrap()
                        .transfer_limits()
                        .map_err(io::Error::other)?;
                    Self::configure_transfers(&worker.rpc, limits)?;
                    self.bridge.attach(
                        worker.rpc.clone(),
                        &restore,
                        &serde_json::to_string(&self.profile.scope)?,
                    )?;
                }
                self.bridge.initial_pull()?;
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
                let result = self.bridge.event_loop();
                worker.stop();
                result
            })();
            let mut c = self.control.lock().unwrap();
            if c.generation != generation {
                continue;
            }
            c.active = None;
            self.bridge.disconnect();
            if let Err(error) = result {
                let retry = matches!(
                    error.kind(),
                    io::ErrorKind::NotConnected
                        | io::ErrorKind::TimedOut
                        | io::ErrorKind::ConnectionAborted
                        | io::ErrorKind::ConnectionRefused
                        | io::ErrorKind::ConnectionReset
                        | io::ErrorKind::BrokenPipe
                        | io::ErrorKind::UnexpectedEof
                );
                c.attention = !retry;
                c.state = if retry {
                    ConnectionState::Offline
                } else {
                    ConnectionState::NeedsAttention
                };
                c.error = Some(if let Some(category) = crate::bridge::capacity_error(&error) {
                    category
                } else if crate::bridge::is_root_unavailable(&error) {
                    "native_root_unavailable"
                } else {
                    match error.kind() {
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

#[cfg(test)]
#[path = "trust_service_tests.rs"]
mod trust_service_tests;
