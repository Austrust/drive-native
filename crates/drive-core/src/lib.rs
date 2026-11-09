//! Offline-testable state engine. Server versions/cursors are opaque strings;
//! these types are internal contracts, not invented Synology wire fields.
// Kept for compatibility only: `DriveBackend`/`Commit` are an unimplemented
// scaffold (see backend.rs) and every use inside is a deprecated-path use we
// deliberately tolerate until TB3 decides to wire or remove it.
#[allow(deprecated)]
pub mod backend;
pub mod commit_outcome;
pub use commit_outcome::*;
pub mod commit_diagnostics;
pub mod filesystem;
#[cfg(feature = "native")]
pub mod native;
pub mod preview;
pub mod state;
pub use state::*;
pub mod exclude;
pub mod shared_store;
pub mod write_permissions;
pub use shared_store::SharedStore;

pub mod drafts;
pub mod timestamps;

pub mod upload_jobs;

pub mod creations;
pub mod directories;

pub mod namespace;
pub mod namespace_schedule;

pub mod removals;

pub mod replacements;

pub mod download_jobs;
pub mod download_publication;

pub mod conflicts;

pub mod cache_policy;
pub mod cache_release;
pub mod engine_bridge;
pub mod engine_upload;

pub mod upload_origin;

pub mod commit_origin;

pub mod directory_conflicts;

pub mod transfer_policy;

/// Bounded retention for operation/namespace/inbox bookkeeping. Deleting
/// unobservable rows must never publish a semantic journal change.
pub mod retention;

pub mod desktop_status;
pub mod status_aggregate;
pub mod status_changes;
/// The namespace semantic watermark/invalidation primitive: compare it instead
/// of `total_changes` before serving or rebuilding a namespace projection.
pub use status_changes::NamespaceWatermark;
pub mod status_index;
pub mod status_projection;
pub mod status_snapshot;
pub mod status_transient;

pub mod initialization_events;
pub mod initialization_store;
pub mod snapshot_progress;
pub mod task_control;
pub mod namespace_index;
pub use namespace_index::NamespaceIndex;
