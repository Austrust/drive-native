use drive_core::{Batch, Node, native::Scope};
use serde::{Deserialize, Serialize};
pub const VERSION: u16 = 21;
// 30 s channel acquisition + 30 s send + 600 s receive idle + 30 s IPC margin.
pub const NOTIFICATION_IDLE: std::time::Duration = std::time::Duration::from_secs(690);
// Deliberately no Debug: authentication material must not enter diagnostics.
#[derive(Serialize, Deserialize)]
pub struct Login {
    pub host: String,
    pub tls_name: String,
    pub trusted_ca_pem: Vec<String>,
    pub username: String,
    pub password: String,
    pub device_id: String,
    pub scope: Scope,
}
#[derive(Serialize, Deserialize)]
pub enum Command {
    ConfigureTransfers(drive_core::transfer_policy::TransferLimits),
    RecoverySnapshot,
    Initialization(drive_core::native::InitializationRequest),
    Mkdir {
        job: drive_core::directories::DirectoryJob,
        reconcile: bool,
    },
    Replace {
        job: drive_core::replacements::ReplaceJob,
        sha256: String,
        reconcile: bool,
    },
    Remove {
        job: drive_core::removals::RemoveJob,
        reconcile: bool,
    },
    Rename {
        job: drive_core::namespace::RenameJob,
        reconcile: bool,
    },
    PrepareUpload {
        node: Node,
        parent: Node,
        path: String,
        sha256: String,
        cursor: String,
    },
    Upload {
        origin: Option<drive_core::upload_origin::UploadOrigin>,
        node: Node,
        path: String,
        sha256: String,
        reconcile: bool,
        mtime: Option<u32>,
    },
    Connect(Login),
    Pull {
        cursor: String,
    },
    Wait {
        cursor: String,
        digest: String,
    },
    Download {
        node: Node,
    },
    Cancel {
        request: u64,
    },
}
#[derive(Serialize, Deserialize)]
pub enum Response {
    InitializationDiagnostics(sproto::diagnostics::Snapshot),
    IndexProgress(drive_core::snapshot_progress::Progress),
    /// Actual received bytes only. CLOCK_MONOTONIC is shared by these local
    /// processes; queued notifications cannot manufacture fresh activity.
    WaitProgress {
        bytes: u64,
        received_at_ns: u64,
    },
    TransfersConfigured(drive_core::transfer_policy::TransferLimits),
    TransferProgress(TransferProgress),
    UploadPrepared {
        origin: drive_core::upload_origin::UploadOrigin,
    },
    RecoverySnapshot {
        size: u64,
        sha256: String,
    },
    Initialization {
        size: u64,
        sha256: String,
    },
    DirectoryPrepared {
        origin: drive_core::directories::DirectoryOrigin,
    },
    DirectoryRenameRefresh {
        node: Node,
    },
    Removed {
        identity: String,
    },
    Renamed {
        node: Node,
        same_content: bool,
    },
    Created {
        node: Node,
        remote: String,
    },
    Committed {
        node: Node,
    },
    Conflict,
    /// Only a verified, definite operation refusal may use this response.
    /// Unclassified numeric NAS codes remain Failed/Uncertain; no guessed map.
    PermissionDenied {
        node: String,
        action: drive_core::write_permissions::Action,
    },
    /// Permission-related direct mutation response, not proof of non-commit
    /// and not a precise node/action ACL observation.
    MutationPermissionUncertain { code: u32 },
    Uncertain,
    Connected {
        restore: String,
    },
    Events(Option<Batch>),
    Notification {
        changed: bool,
        /// Native profile changes can invalidate effective permissions even
        /// when no file event was published for this view.
        profile_changed: bool,
        digest: String,
    },
    Content {
        identity: String,
        version: String,
        size: u64,
        sha256: String,
    },
    Failed {
        category: String,
    },
}
#[derive(Serialize, Deserialize)]
pub struct Envelope<T> {
    pub version: u16,
    pub id: u64,
    pub body: T,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TransferProgress {
    pub epoch: u64,
    pub direction: String,
    pub phase: String,
    pub bytes: u64,
    pub total: u64,
}
impl From<sproto::flow::Progress> for TransferProgress {
    fn from(p: sproto::flow::Progress) -> Self {
        Self {
            epoch: p.epoch,
            direction: p.direction.name().into(),
            phase: p.phase.name().into(),
            bytes: p.bytes,
            total: p.total,
        }
    }
}
