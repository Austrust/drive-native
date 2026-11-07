//! Background failure state reported to the desktop as `background_error`.
//!
//! A background pass can fail transiently and recover on the next attempt:
//! a transport hiccup, a busy local store, or a permission recheck that could
//! not run. Failures are tracked with their source so a cycle that actually ran
//! and succeeded clears only the failures it observed, while genuinely
//! unresolved work (a download failure that could not be persisted, a namespace
//! intent whose commit is still unconfirmed) stays visible.
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackgroundSource {
    /// The upload cycle stopped before finishing its queue.
    UploadQueueStopped,
    /// A user-requested permission recheck could not reset the session.
    PermissionRecheck,
    /// A download failure could not be persisted; local state needs inspection.
    DownloadPublication,
    /// A namespace mutation result is still unconfirmed.
    NamespaceUnconfirmed,
}

impl BackgroundSource {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::UploadQueueStopped => "upload_queue_stopped",
            Self::PermissionRecheck => "permission_recheck",
            Self::DownloadPublication => "download_publication",
            Self::NamespaceUnconfirmed => "namespace_unconfirmed",
        }
    }

    pub fn message(self) -> &'static str {
        match self {
            Self::UploadQueueStopped => "upload stopped; local snapshots retained",
            Self::PermissionRecheck => "permission recheck unavailable",
            Self::DownloadPublication => {
                "cannot persist download failure; local state needs inspection"
            }
            Self::NamespaceUnconfirmed => {
                "namespace commit result not confirmed; local intent retained"
            }
        }
    }
}

/// One unresolved background failure. `since_ms` is the first instant this
/// source was observed, so repeated failures do not reset the timestamp.
#[derive(Debug, Clone)]
pub struct BackgroundFailure {
    pub source: BackgroundSource,
    pub message: &'static str,
    pub since_ms: i64,
}

pub(crate) fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| i64::try_from(elapsed.as_millis()).unwrap_or(i64::MAX))
        .unwrap_or(0)
}
