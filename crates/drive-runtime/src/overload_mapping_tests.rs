//! #31: worker overload is a distinct, retryable diagnostic, and the download
//! retry wait is sliced so a waiting reader stays cancellable and bounded.
use super::{operation_error, retry_wait_slice};
use crate::connection_diagnostics::{retryable, Cause};
use crate::worker::WORKER_OVERLOADED_CATEGORY;
use crate::wire::{Response, VERSION};
use std::time::Duration;

#[test]
fn overload_refusal_is_retryable_but_still_labeled_worker_overloaded() {
    let error = operation_error(Response::Failed {
        category: WORKER_OVERLOADED_CATEGORY.into(),
    });
    assert!(
        retryable(&error),
        "an overloaded worker must not be treated as a permanent protocol failure"
    );
    assert_eq!(Cause::from_io(&error), Cause::WorkerOverloaded);
    // The message is the fixed closed-enum label, never a server detail.
    assert_eq!(error.to_string(), format!("{:?}", Cause::WorkerOverloaded));
}

#[test]
fn overload_refusal_does_not_look_like_a_transport_loss() {
    let error = operation_error(Response::Failed {
        category: WORKER_OVERLOADED_CATEGORY.into(),
    });
    assert!(!crate::bridge::is_root_unavailable(&error));
    assert_eq!(crate::bridge::capacity_error(&error), None);
}

#[test]
fn retry_waits_are_sliced_to_at_most_four_per_second() {
    let mut remaining = Duration::from_secs(1);
    let mut waits = 0;
    while !remaining.is_zero() {
        let wait = retry_wait_slice(remaining);
        assert!(wait <= Duration::from_millis(250), "slice too long: {wait:?}");
        assert!(!wait.is_zero(), "a wait slice must not spin");
        remaining = remaining.saturating_sub(wait);
        waits += 1;
    }
    assert_eq!(waits, 4, "a 1s retry window needs at most four slices");
}

#[test]
fn retry_wait_slice_never_exceeds_its_input() {
    let short = Duration::from_millis(40);
    assert_eq!(retry_wait_slice(short), short);
    assert!(retry_wait_slice(Duration::from_secs(35)) <= Duration::from_millis(250));
    // Sanity: the wire version is present for the mapping helpers above.
    assert_eq!(VERSION, VERSION);
}
