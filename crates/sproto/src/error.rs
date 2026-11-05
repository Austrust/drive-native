use crate::pstream::PObject;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The server no longer accepts the durable cursor; rebuild, never rewind
    /// and continue applying deltas to the existing index.
    #[error("event cursor {requested} exceeds server cursor {available}; recovery required")]
    CursorReset { requested: u64, available: u64 },
    #[error("operation timed out: {0}")]
    Timeout(&'static str),
    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// TLS error
    #[error("TLS error: {0}")]
    Tls(#[from] rustls::Error),

    /// Protocol error: bad magic
    #[error("Protocol error: bad magic")]
    BadMagic,

    /// Protocol error: version mismatch (expected 70-79)
    #[error("Protocol error: version mismatch (got {got}, expected 70-79)")]
    VersionMismatch { got: u8 },

    /// Server error
    #[error("Server error {code:#06x}: {reason}")]
    Server { code: u32, reason: String },

    /// Classified only while receiving the identity-addressed download reply,
    /// never from authentication, channel acquisition or mutation preflight.
    #[error("native download permission denied ({code:#06x})")]
    DownloadPermissionDenied { code: u32 },

    /// An ACL/read-only response to a mutation, without proof that no partial
    /// commit occurred. Callers must reconcile, never turn this into a retry.
    /// The one proven pre-commit subset (upload `0x9003` on an existing node) is
    /// reported as [`Error::MutationPermissionDenied`] instead; everything else
    /// (filter-ambiguous, unreachable, rename/compound, creation) stays here.
    #[error("native mutation permission response ({code:#06x}); commit unconfirmed")]
    MutationPermissionUncertain { code: u32 },

    /// A direct `upload` reply refused by the server ACL middleware after the
    /// header was read and before `ReadBody`, the business handler and commit
    /// (docs/validation/issue7-native-001/server-static/REPORT.md). Only
    /// `0x9003` has one proven producer: the Create ACL test rejecting the
    /// request path because `can_read` is absent (`CreateACLTest::PermissionCheck`
    /// 0x223710 `-3`; `CreateACLTest::HandleError` 0x223140 emits `0x9003`). The
    /// code alone does not name a node: the caller must bind the current request
    /// identity, which is only sound for an identity-addressed upload of an
    /// existing node. Creations are addressed to a child that does not exist
    /// yet, so their capability check is not a CreateFile/CreateDir primitive.
    #[error("native mutation permission denial ({code:#06x}); refused before commit")]
    MutationPermissionDenied { code: u32 },

    /// `PStream` decode error
    #[error("PStream decode error: {0}")]
    Decode(String),

    /// Session expired or invalid
    #[error("Session expired or invalid")]
    SessionInvalid,

    /// Connection closed
    #[error("Connection closed")]
    ConnectionClosed,

    /// Invalid configuration
    #[error("invalid configuration: {0}")]
    InvalidConfig(String),

    /// Server build too old for requested operation
    #[error("server build {build} is too old (need {min_build}+): {reason}")]
    UnsupportedServer {
        build: u64,
        min_build: u64,
        reason: String,
    },
}

/// The result type used throughout the library.
pub type Result<T> = std::result::Result<T, Error>;

pub(crate) fn mutation_reply_error(action: &str, error: Error) -> Error {
    match error {
        // The only mutation reply this evidence proves both non-committed and
        // attributable is the upload Create ACL test failing its read
        // prerequisite. Everything else stays an action-scoped hint: 0x9004 /
        // 0xa004 are also produced by profile read-only filtering, 0xa003 has
        // no proven producer in the remove path, and rename/compound requests
        // do not identify the denied participant.
        Error::Server { code: 0x9003, .. } if action == "upload" => {
            Error::MutationPermissionDenied { code: 0x9003 }
        }
        Error::Server { code, .. }
            if matches!(
                (action, code),
                ("upload" | "remove" | "rename", 0x3002 | 0x3004)
                    | ("upload", 0x9004)
                    | ("remove", 0xa003 | 0xa004)
            ) =>
        {
            Error::MutationPermissionUncertain { code }
        }
        error => error,
    }
}

/// Inspect a response `PObject` for an `"error"` field.
/// Maps session error codes (0x4001-0x4003) to `SessionInvalid`,
/// all other non-null errors to `Error::Server`.
pub fn check_server_error(obj: &PObject) -> Result<()> {
    let Some(err) = obj.get("error") else {
        return Ok(());
    };

    if matches!(err, PObject::Null) {
        return Ok(());
    }

    // A malformed wide integer must not alias a recognized permission code.
    let code = u32::try_from(err.get("code").and_then(PObject::as_int).unwrap_or(0))
        .map_err(|_| Error::Decode("server error code exceeds u32".into()))?;
    let reason = err
        .get("reason")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();

    if (0x4001..=0x4003).contains(&code) {
        return Err(Error::SessionInvalid);
    }

    Err(Error::Server { code, reason })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_error_field() {
        let obj = pmap! { "type" => "response" };
        assert!(check_server_error(&obj).is_ok());
    }

    #[test]
    fn null_error() {
        let obj = pmap! { "error" => PObject::Null };
        assert!(check_server_error(&obj).is_ok());
    }

    #[test]
    fn session_invalid_errors() {
        for code in [0x4001u64, 0x4002, 0x4003] {
            let obj = pmap! {
                "error" => pmap! {
                    "code" => code,
                    "reason" => "session gone",
                },
            };
            let err = check_server_error(&obj).unwrap_err();
            assert!(
                matches!(err, Error::SessionInvalid),
                "code {code:#x} should be SessionInvalid"
            );
        }
    }

    #[test]
    fn server_error() {
        let obj = pmap! {
            "error" => pmap! {
                "code" => 0x3002u64,
                "reason" => "Invalid view ID",
            },
        };
        let err = check_server_error(&obj).unwrap_err();
        match err {
            Error::Server { code, reason } => {
                assert_eq!(code, 0x3002);
                assert_eq!(reason, "Invalid view ID");
            }
            _ => panic!("expected Error::Server, got {err:?}"),
        }
    }

    #[test]
    fn error_missing_fields() {
        // Error object with no code/reason — should default gracefully
        let obj = pmap! { "error" => pmap! {} };
        let err = check_server_error(&obj).unwrap_err();
        match err {
            Error::Server { code, reason } => {
                assert_eq!(code, 0);
                assert_eq!(reason, "unknown");
            }
            _ => panic!("expected Error::Server, got {err:?}"),
        }
    }
    #[test]
    fn oversized_error_code_cannot_alias_a_permission_refusal() {
        let obj = pmap! { "error" => pmap! {
            "code" => 0x1_0000_3002u64, "reason" => "no permission",
        }};
        assert!(matches!(check_server_error(&obj), Err(Error::Decode(_))));
    }
    #[test]
    fn only_the_upload_create_acl_code_is_a_definite_mutation_refusal() {
        // 0x9003 on `upload` is the one code proven to be produced only by the
        // pre-commit ACL middleware (Create test, can_read absent). It still
        // needs request-identity binding; the wire layer only types it.
        match mutation_reply_error(
            "upload",
            Error::Server {
                code: 0x9003,
                reason: "synthetic".into(),
            },
        ) {
            Error::MutationPermissionDenied { code } => assert_eq!(code, 0x9003),
            other => panic!("expected definite refusal, got {other:?}"),
        }
        // Every ambiguous or unattributable code must stay an action-scoped
        // hint or a plain server error; none may become a definite refusal.
        for (action, code) in [
            ("upload", 0x9004u32),
            ("upload", 0x9005),
            ("upload", 0x3002),
            ("upload", 0x3004),
            ("remove", 0xa003),
            ("remove", 0xa004),
            ("remove", 0xa005),
            ("remove", 0x9003),
            ("rename", 0x9003),
            ("rename", 0x9004),
            ("rename", 0x3002),
            ("mkdir", 0x9003),
            ("list", 0x9003),
        ] {
            let error = mutation_reply_error(
                action,
                Error::Server {
                    code,
                    reason: "permission denied".into(),
                },
            );
            assert!(
                !matches!(error, Error::MutationPermissionDenied { .. }),
                "{action} {code:#06x} must not be a definite refusal"
            );
        }
    }
    #[test]
    fn mutation_permission_hints_are_action_scoped_and_never_definite_refusals() {
        for (action, code, permission) in [
            ("upload", 0x9004, true),
            ("upload", 0x9005, false), ("remove", 0xa003, true),
            ("remove", 0xa004, true), ("remove", 0xa005, false),
            ("rename", 0x3002, true), ("rename", 0x9004, false),
            ("upload", 0x3004, true), ("list", 0x3002, false),
            ("upload", 0x6003, false), ("remove", 0xa002, false),
        ] {
            let error = mutation_reply_error(action, Error::Server { code, reason: "permission denied".into() });
            assert_eq!(matches!(error, Error::MutationPermissionUncertain { .. }), permission);
            assert!(!matches!(error, Error::MutationPermissionDenied { .. }));
        }
    }
}
