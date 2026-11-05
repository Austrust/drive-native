//! Independent adversarial probe for the T9 retry wrapper.
//!
//! This crate includes the product file `crates/drive-core/src/native_retry.rs`
//! (byte-identical except the stripped leading `//!` module doc lines) inside a
//! parent module that supplies the same `super::*` names the product module has
//! (`sproto`, `Client`, `EventBatch`). Tests therefore execute the actual retry
//! source with the real `sproto::Error` type, without modifying product code and
//! without depending on private crate visibility.
use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::rc::Rc;
use std::time::{Duration, Instant};

pub mod native {
    pub use sproto;
    pub use sproto::actions::native::EventBatch;
    pub use sproto::Client;
    pub mod retry {
        include!("native_retry_included.rs");
    }

    pub const PROBE_MAX_ATTEMPTS: u32 = retry::MAX_ATTEMPTS;

    pub fn probe_retryable(error: &sproto::Error) -> bool {
        retry::retryable(error)
    }

    pub async fn probe_read<T, F, Fut>(operation: F) -> sproto::Result<T>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = sproto::Result<T>>,
    {
        retry::read(operation).await
    }
}

use native::PROBE_MAX_ATTEMPTS;
use sproto::Error;

fn transport() -> Error {
    Error::Io(std::io::Error::from(std::io::ErrorKind::UnexpectedEof))
}

/// Run `probe_read` over a scripted sequence; return the result and call count.
async fn run_script<T: 'static>(script: Vec<sproto::Result<T>>) -> (sproto::Result<T>, usize) {
    let calls = Rc::new(Cell::new(0));
    let results = Rc::new(RefCell::new(VecDeque::from(script)));
    let calls_for_op = calls.clone();
    let value = native::probe_read(|| {
        calls_for_op.set(calls_for_op.get() + 1);
        let next = results.borrow_mut().pop_front().expect("script underrun");
        std::future::ready(next)
    })
    .await;
    (value, calls.get())
}

#[tokio::test]
async fn max_attempts_constant_is_five() {
    assert_eq!(PROBE_MAX_ATTEMPTS, 5);
}

#[tokio::test]
async fn transport_disconnects_then_success_retries_same_operation() {
    for failures in [1usize, 2, 4] {
        let mut script: Vec<sproto::Result<u32>> = (0..failures).map(|_| Err(transport())).collect();
        script.push(Ok(7u32));
        let (value, calls) = run_script(script).await;
        assert_eq!(calls, failures + 1, "attempts must be failures+1 for K={failures}");
        assert_eq!(value.expect("retry must recover"), 7);
    }
}

#[tokio::test]
async fn persistent_transport_failure_stops_at_the_attempt_cap() {
    let mut script: Vec<sproto::Result<u32>> = (0..10).map(|_| Err(transport())).collect();
    script.push(Ok(9));
    let (value, calls) = run_script(script).await;
    assert_eq!(calls, PROBE_MAX_ATTEMPTS as usize, "must stop at the cap, not try the scripted success");
    match value {
        Err(Error::Io(error)) => assert_eq!(error.kind(), std::io::ErrorKind::UnexpectedEof),
        other => panic!("cap exhaustion must surface the original transport error, got {other:?}"),
    }
}

#[tokio::test]
async fn non_retryable_errors_are_attempted_once() {
    let errors = vec![
        Error::Server { code: 12290, reason: "denied".into() },
        Error::Server { code: 0x2001, reason: "rejected".into() },
        Error::MutationPermissionDenied { code: 0x9003 },
        Error::MutationPermissionUncertain { code: 0x9004 },
        Error::DownloadPermissionDenied { code: 0x3002 },
        Error::Decode("bad".into()),
        Error::SessionInvalid,
        Error::InvalidConfig("bad".into()),
        Error::CursorReset { requested: 9, available: 3 },
        Error::BadMagic,
        Error::VersionMismatch { got: 1 },
        Error::UnsupportedServer { build: 1, min_build: 2, reason: "old".into() },
        Error::Io(std::io::Error::from(std::io::ErrorKind::PermissionDenied)),
        Error::Io(std::io::Error::other("local")),
    ];
    for error in errors {
        let label = format!("{error:?}");
        let (value, calls) = run_script::<u32>(vec![Err(error), Ok(1)]).await;
        assert_eq!(calls, 1, "must attempt exactly once for {label}");
        assert!(value.is_err(), "non-retryable error must surface for {label}");
    }
}

#[tokio::test]
async fn retryable_transport_errors_are_retried() {
    let errors = vec![
        Error::Timeout("native response"),
        Error::ConnectionClosed,
    ];
    for error in errors {
        let label = format!("{error:?}");
        let (value, calls) = run_script::<u32>(vec![Err(error), Ok(3)]).await;
        assert_eq!(calls, 2, "must retry once for {label}");
        assert_eq!(value.expect("retry must recover"), 3);
    }
    for kind in [
        std::io::ErrorKind::UnexpectedEof,
        std::io::ErrorKind::ConnectionReset,
        std::io::ErrorKind::ConnectionAborted,
        std::io::ErrorKind::BrokenPipe,
        std::io::ErrorKind::NotConnected,
        std::io::ErrorKind::TimedOut,
        std::io::ErrorKind::WouldBlock,
    ] {
        let error = Error::Io(std::io::Error::from(kind));
        assert!(native::probe_retryable(&error), "Io {kind:?} must be retryable");
        let (value, calls) = run_script::<u32>(vec![Err(error), Ok(3)]).await;
        assert_eq!(calls, 2);
        assert_eq!(value.unwrap(), 3);
    }
}

#[tokio::test]
async fn immediate_success_has_no_backoff() {
    let started = Instant::now();
    let (value, calls) = run_script::<u32>(vec![Ok(11)]).await;
    assert_eq!(calls, 1);
    assert_eq!(value.unwrap(), 11);
    assert!(started.elapsed() < Duration::from_millis(150), "first success must not sleep");
}

#[tokio::test]
async fn exhausted_backoff_is_bounded_and_nonzero() {
    // Four sleeps: 200 + 400 + 800 + 1600 ms = 3.0 s between five attempts.
    let started = Instant::now();
    let (value, calls) = run_script::<u32>((0..10).map(|_| Err(transport())).collect()).await;
    let elapsed = started.elapsed();
    assert_eq!(calls, 5);
    assert!(value.is_err());
    assert!(elapsed >= Duration::from_millis(2900), "expected bounded exponential backoff, got {elapsed:?}");
    assert!(elapsed <= Duration::from_secs(8), "backoff must stay bounded, got {elapsed:?}");
}

#[tokio::test]
async fn cancelled_future_during_backoff_does_not_start_another_attempt() {
    let calls = Rc::new(Cell::new(0));
    let calls_for_op = calls.clone();
    let future = native::probe_read(|| {
        calls_for_op.set(calls_for_op.get() + 1);
        std::future::ready(Err::<u32, _>(transport()))
    });
    // The first backoff is 200 ms; a 100 ms outer timeout cancels the future while it
    // sleeps, so no further attempt may start.
    let result = tokio::time::timeout(Duration::from_millis(100), future).await;
    assert!(result.is_err(), "outer timeout must cancel the retry future");
    assert_eq!(calls.get(), 1, "cancellation during the first backoff must not start another attempt");
}

#[tokio::test]
async fn operation_constructs_a_fresh_future_for_each_attempt() {
    let calls = Rc::new(Cell::new(0));
    let created = Rc::new(Cell::new(0));
    let calls_for_op = calls.clone();
    let created_for_op = created.clone();
    let calls_final = calls.clone();
    let mut failing = 0;
    let value = native::probe_read(move || {
        calls_for_op.set(calls_for_op.get() + 1);
        created_for_op.set(created_for_op.get() + 1);
        failing += 1;
        let attempt = failing;
        std::future::ready(if attempt < 3 { Err(transport()) } else { Ok(5u32) })
    })
    .await;
    assert_eq!(value.unwrap(), 5);
    assert_eq!(calls.get(), calls_final.get());
    assert_eq!(created.get(), calls.get(), "each attempt must construct its own future");
}