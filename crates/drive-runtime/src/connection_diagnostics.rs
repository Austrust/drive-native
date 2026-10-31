//! Bounded, content-free connection diagnostics, committed before worker cleanup.
//! No error Display strings, requests, paths, identities or credentials are stored.
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt},
    path::{Path, PathBuf},
    sync::Mutex,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Stage {
    StartWorker,
    Authenticate,
    ConfigureTransfers,
    BindScope,
    InitialIndex,
    ResumeEvents,
    PullEvents,
    WaitNotification,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case", tag = "kind", content = "code")]
pub(crate) enum Cause {
    NativeReceiveIdle,
    NativeSetupTimeout,
    NativeSendTimeout,
    NativeResponseTimeout,
    IpcIdle,
    IpcQueueUnavailable,
    WorkerDisconnected,
    NativeTransport,
    SessionExpired,
    NativeServerRejected(u32),
    NativeCursorReset,
    NativeRootUnavailable,
    NativeIndexCapacity,
    NativeResponseCapacity,
    AuthenticationOrProtocol,
    CertificateOrConfiguration,
    ProtocolOrLocal,
    Interrupted,
}
impl std::fmt::Display for Cause {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Only this closed enum is formatted, never an underlying error string.
        write!(f, "{self:?}")
    }
}
impl std::error::Error for Cause {}
impl Cause {
    pub(crate) fn from_io(error: &io::Error) -> Self {
        if let Some(cause) = error.get_ref().and_then(|e| e.downcast_ref::<Self>()) {
            return *cause;
        }
        if crate::bridge::is_root_unavailable(error) {
            return Self::NativeRootUnavailable;
        }
        match crate::bridge::capacity_error(error) {
            Some("native_index_capacity") => return Self::NativeIndexCapacity,
            Some("native_response_capacity") => return Self::NativeResponseCapacity,
            _ => {}
        }
        match error.kind() {
            io::ErrorKind::PermissionDenied => Self::AuthenticationOrProtocol,
            io::ErrorKind::InvalidInput => Self::CertificateOrConfiguration,
            io::ErrorKind::Interrupted => Self::Interrupted,
            _ if retryable(error) => Self::NativeTransport,
            _ => Self::ProtocolOrLocal,
        }
    }
}

/// This is the existing supervisor policy, shared by the diagnostic classification.
pub(crate) fn retryable(error: &io::Error) -> bool {
    matches!(
        error.kind(),
        io::ErrorKind::NotConnected
            | io::ErrorKind::TimedOut
            | io::ErrorKind::ConnectionAborted
            | io::ErrorKind::ConnectionRefused
            | io::ErrorKind::ConnectionReset
            | io::ErrorKind::BrokenPipe
            | io::ErrorKind::UnexpectedEof
    )
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case", tag = "state", deny_unknown_fields)]
pub(crate) enum ProcessState {
    NotStarted,
    Running,
    Exited {
        code: Option<i32>,
        signal: Option<i32>,
    },
    QueryFailed,
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Recovery {
    Reconnect,
    NeedsAttention,
    RequestedInterruption,
    Finished,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Record {
    pub unix_ms: u64,
    pub attempt_elapsed_ms: u64,
    pub worker_pid: Option<u32>,
    pub stage: Stage,
    pub cause: Option<Cause>,
    pub recovery: Recovery,
    pub before_cleanup: ProcessState,
}
impl Record {
    pub(crate) fn new(
        stage: Stage,
        result: &io::Result<()>,
        requested: bool,
        elapsed: Duration,
        worker_pid: Option<u32>,
        before_cleanup: ProcessState,
    ) -> Self {
        Self {
            unix_ms: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis()
                .min(u64::MAX as u128) as u64,
            attempt_elapsed_ms: elapsed.as_millis().min(u64::MAX as u128) as u64,
            worker_pid,
            stage,
            cause: result.as_ref().err().map(Cause::from_io),
            recovery: if requested {
                Recovery::RequestedInterruption
            } else {
                match result {
                    Ok(()) => Recovery::Finished,
                    Err(e) if retryable(e) => Recovery::Reconnect,
                    Err(_) => Recovery::NeedsAttention,
                }
            },
            before_cleanup,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Snapshot {
    pub schema: u8,
    pub last_transition: Option<Record>,
    pub last_failure: Option<Record>,
    pub load_failed: bool,
    pub persistence_failed: bool,
}
impl Default for Snapshot {
    fn default() -> Self {
        Self {
            schema: 1,
            last_transition: None,
            last_failure: None,
            load_failed: false,
            persistence_failed: false,
        }
    }
}
pub(crate) struct Recorder {
    path: PathBuf,
    state: Mutex<Snapshot>,
}
impl Recorder {
    pub(crate) fn open(directory: &Path) -> Self {
        let path = directory.join("connection-diagnostics.json");
        let state = Self::read(&path).unwrap_or_else(|_| Snapshot {
            load_failed: true,
            ..Snapshot::default()
        });
        Self {
            path,
            state: Mutex::new(state),
        }
    }
    fn read(path: &Path) -> io::Result<Snapshot> {
        let file = match OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
            .open(path)
        {
            Ok(f) => f,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(Snapshot::default()),
            Err(e) => return Err(e),
        };
        let metadata = file.metadata()?;
        if !metadata.is_file()
            || metadata.len() > 16_384
            || metadata.uid() != unsafe { libc::geteuid() }
        {
            return Err(io::Error::other("invalid diagnostic file"));
        }
        let mut bytes = Vec::new();
        file.take(16_385).read_to_end(&mut bytes)?;
        if bytes.len() > 16_384 {
            return Err(io::Error::other("diagnostic file too large"));
        }
        let snapshot: Snapshot = serde_json::from_slice(&bytes)?;
        if snapshot.schema != 1 {
            return Err(io::Error::other("unknown diagnostic schema"));
        }
        Ok(snapshot)
    }
    pub(crate) fn snapshot(&self) -> Snapshot {
        self.state.lock().unwrap().clone()
    }
    pub(crate) fn record(&self, record: Record) {
        self.record_with(record, |snapshot| self.persist(snapshot));
    }
    fn record_with(&self, record: Record, persist: impl FnOnce(&Snapshot) -> io::Result<()>) {
        // Supervisor is the sole writer. No Store/control mutex is held during IO.
        let mut snapshot = self.snapshot();
        if matches!(
            record.recovery,
            Recovery::Reconnect | Recovery::NeedsAttention
        ) {
            snapshot.last_failure = Some(record.clone());
        }
        snapshot.last_transition = Some(record);
        snapshot.persistence_failed = false;
        let persisted = persist(&snapshot).is_ok();
        snapshot.persistence_failed = !persisted;
        *self.state.lock().unwrap() = snapshot.clone();
        // Safe fallback also lets the event probe capture every transition;
        // the bounded on-disk snapshot is not the long-term acceptance archive.
        eprintln!(
            "{}",
            serde_json::json!({"event":"connection_transition","persisted":persisted,"record":snapshot.last_transition})
        );
    }
    fn persist(&self, snapshot: &Snapshot) -> io::Result<()> {
        let directory = self
            .path
            .parent()
            .ok_or_else(|| io::Error::other("diagnostic directory absent"))?;
        let mut temp = tempfile::NamedTempFile::new_in(directory)?;
        temp.as_file()
            .set_permissions(fs::Permissions::from_mode(0o600))?;
        serde_json::to_writer(&mut temp, snapshot)?;
        temp.write_all(b"\n")?;
        temp.as_file().sync_all()?;
        temp.persist(&self.path).map_err(|e| e.error)?;
        File::open(directory)?.sync_all()
    }
}

#[cfg(test)]
#[path = "connection_diagnostics_tests.rs"]
mod tests;
