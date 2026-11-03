//! Offline-testable state engine. Server versions/cursors are opaque strings;
//! these types are internal contracts, not invented Synology wire fields.
pub mod backend;
pub mod filesystem;
#[cfg(feature = "native")]
pub mod native;
pub mod preview;
pub mod state;
pub use state::*;
pub mod shared_store;
pub use shared_store::SharedStore;

pub mod drafts;
pub mod timestamps;

pub mod upload_jobs;

pub mod creations;
pub mod directories;

pub mod namespace;
pub mod namespace_schedule;

pub mod removals;

pub mod write_permissions;

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

pub mod desktop_status;
pub mod status_aggregate;
pub mod status_changes;
pub mod status_index;
pub mod status_projection;
pub mod status_snapshot;
pub mod status_transient;

pub mod initialization_events;
pub mod initialization_store;
pub mod snapshot_progress;
pub mod task_control;
