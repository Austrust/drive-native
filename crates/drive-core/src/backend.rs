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
