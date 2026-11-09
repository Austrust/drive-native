//! Mount-free coverage for the hard-link rejection contract (#18).
//!
//! The FUSE handler itself cannot be called from a unit test: `fuser::Request`
//! has no public constructor and `ReplyEntry` needs a reply sender that the
//! vendored crate does not export. The errno contract is covered here, and the
//! end-to-end handler behaviour by the kernel test
//! `kernel_hard_link_is_rejected_with_operation_not_supported` in
//! `tests/mounted.rs`.

use std::io::ErrorKind;

use super::HARD_LINK_UNSUPPORTED;

#[test]
fn hard_link_rejection_uses_operation_not_supported() {
    assert_eq!(HARD_LINK_UNSUPPORTED, libc::EOPNOTSUPP);
    // Linux exposes both names for errno 95; a portable probe must reach the
    // same "not supported" verdict through either spelling.
    assert_eq!(libc::EOPNOTSUPP, libc::ENOTSUP);
    // EPERM is the vendored fuser default this override replaces; confusing
    // "not permitted" with "not supported" hides the documented fallback.
    assert_ne!(HARD_LINK_UNSUPPORTED, libc::EPERM);
    // Rust tools see `ErrorKind::Unsupported`, which is the documented signal
    // for tools that degrade to copy/rename.
    assert_eq!(
        std::io::Error::from_raw_os_error(HARD_LINK_UNSUPPORTED).kind(),
        ErrorKind::Unsupported
    );
}
