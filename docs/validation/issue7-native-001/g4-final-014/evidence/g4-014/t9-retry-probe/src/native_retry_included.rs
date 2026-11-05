use super::*;
use sproto::actions::{list::NodeInfo, native::Watch};
use std::{future::Future, time::Duration};

/// Total attempts per logical read (first try + retries).
pub(super) const MAX_ATTEMPTS: u32 = 5;
const BASE_BACKOFF_MS: u64 = 200;
const MAX_BACKOFF_MS: u64 = 2000;

/// The real NAS closes TLS without close_notify during long initialization
/// sequences. Only those transport-shaped failures are safe to repeat: a
/// `Server` refusal, decode/config/session error or mutation receipt is a
/// definite answer and must not be retried.
pub(super) fn retryable(error: &sproto::Error) -> bool {
    match error {
        sproto::Error::ConnectionClosed | sproto::Error::Timeout(_) => true,
        sproto::Error::Io(io) => matches!(
            io.kind(),
            std::io::ErrorKind::UnexpectedEof
                | std::io::ErrorKind::ConnectionReset
                | std::io::ErrorKind::ConnectionAborted
                | std::io::ErrorKind::BrokenPipe
                | std::io::ErrorKind::NotConnected
                | std::io::ErrorKind::TimedOut
                | std::io::ErrorKind::WouldBlock
        ),
        _ => false,
    }
}

fn backoff(attempt: u32) -> Duration {
    Duration::from_millis(
        BASE_BACKOFF_MS
            .saturating_mul(1u64 << attempt.saturating_sub(1).min(4))
            .min(MAX_BACKOFF_MS),
    )
}

/// Repeat a read-only operation with bounded exponential backoff. `operation`
/// returns a fresh request future each time so `pull_event` (same cursor) and
/// `list_dir` (same path) restart from the last committed position instead of
/// advancing on a failure.
pub(super) async fn read<T, F, Fut>(mut operation: F) -> sproto::Result<T>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = sproto::Result<T>>,
{
    let mut attempt = 1;
    loop {
        match operation().await {
            Ok(value) => return Ok(value),
            Err(error) if attempt < MAX_ATTEMPTS && retryable(&error) => {
                tokio::time::sleep(backoff(attempt)).await;
                attempt += 1;
            }
            Err(error) => return Err(error),
        }
    }
}

/// Event page read used by seed/fence/pull; the caller keeps the cursor.
pub(super) async fn pull_events(
    client: &Client,
    watch: &Watch,
) -> sproto::Result<EventBatch> {
    read(|| client.pull_events(watch)).await
}

/// Directory list used by seed enumeration and initialization directories.
pub(super) async fn list_dir(
    client: &Client,
    view_id: u64,
    path: &str,
) -> sproto::Result<Vec<NodeInfo>> {
    read(|| client.list_dir(view_id, path)).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{cell::{Cell, RefCell}, collections::VecDeque, rc::Rc};

    fn transport() -> sproto::Error {
        sproto::Error::Io(std::io::Error::from(std::io::ErrorKind::UnexpectedEof))
    }

    async fn run_script<T: 'static>(
        results: Vec<sproto::Result<T>>,
    ) -> (sproto::Result<T>, usize) {
        let calls = Rc::new(Cell::new(0));
        let results = Rc::new(RefCell::new(VecDeque::from(results)));
        let calls_for_op = calls.clone();
        let value = read(|| {
            calls_for_op.set(calls_for_op.get() + 1);
            std::future::ready(results.borrow_mut().pop_front().unwrap())
        })
        .await;
        (value, calls.get())
    }

    #[tokio::test]
    async fn transport_disconnects_are_retried_with_same_read() {
        let (value, calls) =
            run_script(vec![Err(transport()), Err(transport()), Ok(7u32)]).await;
        assert_eq!(calls, 3);
        assert_eq!(value.unwrap(), 7);
    }

    #[tokio::test]
    async fn persistent_transport_failure_stops_at_the_attempt_cap() {
        let (value, calls) = run_script::<u32>(vec![
            Err(transport()),
            Err(transport()),
            Err(transport()),
            Err(transport()),
            Err(transport()),
            Ok(9),
        ])
        .await;
        assert_eq!(calls, MAX_ATTEMPTS as usize);
        assert!(matches!(value, Err(sproto::Error::Io(_))));
    }

    #[tokio::test]
    async fn server_protocol_and_mutation_errors_are_never_retried() {
        for error in [
            sproto::Error::Server {
                code: 12290,
                reason: "denied".into(),
            },
            sproto::Error::Server {
                code: 0x2001,
                reason: "rejected".into(),
            },
            sproto::Error::MutationPermissionDenied { code: 0x9003 },
            sproto::Error::MutationPermissionUncertain { code: 0x9004 },
            sproto::Error::Decode("bad".into()),
            sproto::Error::SessionInvalid,
            sproto::Error::InvalidConfig("bad".into()),
            sproto::Error::CursorReset {
                requested: 9,
                available: 3,
            },
        ] {
            let (value, calls) = run_script::<u32>(vec![Err(error), Ok(1)]).await;
            assert_eq!(calls, 1);
            assert!(matches!(value, Err(_)));
        }
    }

    #[tokio::test]
    async fn timeout_and_connection_closed_are_retryable() {
        for error in [
            sproto::Error::Timeout("native response"),
            sproto::Error::ConnectionClosed,
        ] {
            let (value, calls) = run_script::<u32>(vec![Err(error), Ok(3)]).await;
            assert_eq!(calls, 2);
            assert_eq!(value.unwrap(), 3);
        }
    }
}
