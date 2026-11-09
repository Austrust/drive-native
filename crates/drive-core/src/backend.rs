use crate::commit_outcome::{CommitCategory, CommitDiagnostic, CommitOutcome};
use crate::{Batch, Node, Result};
use std::{fs::File, future::Future, pin::Pin};

pub type BackendFuture<'a, T> = Pin<Box<dyn Future<Output = Result<T>> + Send + 'a>>;

/// Validated native adapter output. Contents are passed as a descriptor, never a
/// body-sized Vec or a D-Bus message. The fsd verifies size/hash while importing.
pub struct DownloadedFile {
    pub file: File,
    pub identity: String,
    pub version: String,
    pub size: u64,
    pub sha256: String,
}

/// An immutable journaled snapshot owned by the filesystem service.
pub struct UploadSnapshot {
    pub file: File,
    pub size: u64,
    pub sha256: String,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Capabilities {
    pub events: bool,
    pub conditional_writes: bool,
    pub reconcile_uncertain_commit: bool,
    pub version_bound_download: bool,
}
impl Capabilities {
    pub fn writable_release_ready(self) -> bool {
        self.events
            && self.conditional_writes
            && self.reconcile_uncertain_commit
            && self.version_bound_download
    }
}

/// A future native adapter must map these operations to evidence-backed native messages.
/// There is deliberately no WebDAV implementation or scanning fallback.
///
/// **Deprecated scaffold / conclusion (TB1 of #9b).** Nothing in this workspace
/// implements `DriveBackend`, and no call site uses it: the real remote path is
/// `sproto` + `drive_core::native` + the runtime worker RPC, and commit
/// uncertainty is journaled per subsystem. The trait is therefore not a
/// portable asset. TB1 only keeps it as a compatibility alias and maps
/// [`Commit`] onto the unified [`CommitOutcome`] vocabulary; TB3 decides whether
/// to wire it (behind a real adapter) or delete it. Do not add new
/// implementations or call sites in the meantime.
#[deprecated(
    note = "unimplemented legacy scaffold; the native path is sproto + drive_core::native + worker RPC. Use CommitOutcome/CommitDiagnostic; wiring or removal is decided in #9b TB3"
)]
pub trait DriveBackend: Send + Sync {
    fn capabilities(&self) -> Capabilities;
    fn list<'a>(&'a self, parent: &'a str) -> BackendFuture<'a, Vec<Node>>;
    fn changes<'a>(&'a self, cursor: &'a str) -> BackendFuture<'a, Batch>;
    fn download<'a>(&'a self, id: &'a str, version: &'a str) -> BackendFuture<'a, DownloadedFile>;
    fn upload<'a>(
        &'a self,
        operation_id: &'a str,
        id: &'a str,
        base: &'a str,
        snapshot: UploadSnapshot,
    ) -> BackendFuture<'a, Commit>;
    fn rename<'a>(
        &'a self,
        operation_id: &'a str,
        id: &'a str,
        base: &'a str,
        parent: &'a str,
        name: &'a str,
    ) -> BackendFuture<'a, Commit>;
    fn remove<'a>(
        &'a self,
        operation_id: &'a str,
        id: &'a str,
        base: &'a str,
    ) -> BackendFuture<'a, Commit>;
    fn reconcile<'a>(&'a self, operation_id: &'a str) -> BackendFuture<'a, Commit>;
}

/// Legacy three-state backend result kept only for the deprecated
/// [`DriveBackend`] scaffold.
///
/// New code must speak [`CommitOutcome`]: [`CommitOutcome::UnknownRetryable`]
/// and [`CommitOutcome::UnknownFatal`] replace the old catch-all `Unknown`, so a
/// caller can never be left with a state that has neither a retry plan nor an
/// attention reason. Use [`Commit::diagnostic`] when a full durable diagnosis
/// (category, last error, attempts, next retry) is needed.
#[deprecated(note = "legacy scaffold result; map onto CommitOutcome/CommitDiagnostic instead")]
#[derive(Debug)]
pub enum Commit {
    Confirmed {
        version: String,
    },
    Conflict {
        remote_version: String,
    },
    /// Must reconcile before replaying a mutation. Never blindly retry.
    Unknown,
}

#[allow(deprecated)]
impl Commit {
    /// Collapse the legacy three-state result onto the unified four-state
    /// vocabulary. `Unknown` always maps to retryable: the caller must first
    /// reconcile read-only, and only the reconcile budget/response may turn it
    /// terminal (see [`retry_delay`](crate::commit_outcome::retry_delay)).
    pub fn outcome(&self) -> CommitOutcome {
        match self {
            Self::Confirmed { .. } => CommitOutcome::Confirmed,
            Self::Conflict { .. } => CommitOutcome::Conflict,
            Self::Unknown => CommitOutcome::UnknownRetryable,
        }
    }

    /// Full diagnostic row for the same result. `next_retry_at` is left at `0`
    /// because only the runtime knows the current clock; TB2 fills it from
    /// [`retry_delay`](crate::commit_outcome::retry_delay).
    pub fn diagnostic(&self) -> CommitDiagnostic {
        match self {
            Self::Confirmed { .. } => CommitDiagnostic::confirmed(),
            Self::Conflict { remote_version } => CommitDiagnostic::conflict(Some(format!(
                "remote changed to version {remote_version}"
            ))),
            Self::Unknown => CommitDiagnostic::unknown_retryable(
                CommitCategory::Interrupted,
                Some("backend commit unknown; reconcile before replaying".into()),
                0,
                0,
            ),
        }
    }
}
