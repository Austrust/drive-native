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

    #[allow(
        clippy::cast_possible_truncation,
        reason = "server error codes are small integers, always fit in u32"
    )]
    let code = err.get("code").and_then(PObject::as_int).unwrap_or(0) as u32;
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
}
