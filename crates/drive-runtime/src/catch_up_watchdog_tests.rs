//! Issue #58: deterministic event-loop supervision tests.
//!
//! The NAS is replaced by a local `UnixStream` peer. Catch-up/attempt deadlines
//! come from the documented `DRIVE_NATIVE_*` environment configuration, so the
//! tests never sleep for production durations: the runner sets sub-second
//! values (see `notes/work/issue-58.md`). The assertions read the serialized
//! initialization projection, which keeps the tests meaningful against the
//! unfixed tree (the new keys and behavior simply do not exist there).
use super::*;
use crate::transport;
use drive_core::NodeMetadata;
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

const DEADLINE: Duration = Duration::from_secs(10);

fn root() -> Node {
    Node {
        id: "1".into(),
        parent: None,
        name: "lab".into(),
        version: "root".into(),
        directory: true,
        metadata: None,
    }
}

fn file(id: &str, name: &str) -> Node {
    Node {
        id: id.into(),
        parent: Some("1".into()),
        name: name.into(),
        version: "v1".into(),
        directory: false,
        metadata: Some(NodeMetadata {
            size: 3,
            mtime: 0,
            mode: 0o644,
        }),
    }
}

/// Root plus one cached body: cached reads must keep working while the runtime
/// leaves `catching_up` and while the attempt is being supervised.
fn store_fixture() -> (tempfile::TempDir, Arc<drive_core::SharedStore>) {
    let dir = tempfile::tempdir().unwrap();
    let store = Arc::new(drive_core::SharedStore::new(
        Store::open(dir.path()).unwrap(),
    ));
    {
        let mut s = store.lock().unwrap();
        s.install_remote_snapshot(&[root(), file("101", "A.doc")], "1", "fixture")
            .unwrap();
        s.ensure_fs_root("1").unwrap();
        s.publish_download(
            &DownloadTicket {
                id: "101".into(),
                version: "v1".into(),
            },
            b"old",
            &drive_core::sha256(b"old"),
        )
        .unwrap();
    }
    (dir, store)
}

fn notification(changed: bool) -> Response {
    crate::worker::notification_response(
        1,
        sproto::actions::native::Notification {
            changed_views: if changed { vec![1] } else { vec![] },
            profile_changed: false,
            profile_digest: "next".into(),
        },
    )
}

/// Mock event worker: claims a change on every long poll while applying
/// nothing; production keeps `catching_up` alive forever on this stream. Once
/// `apply_after_budget` is set it delivers one real advancing batch, so the
/// test can prove incremental catch-up still works after the budget was spent.
fn changed_notification_server(
    mut socket: UnixStream,
    apply_after_budget: Arc<AtomicBool>,
    advanced: Arc<AtomicBool>,
) {
    while let Ok((request, _)) = transport::receive::<Envelope<Command>>(&mut socket) {
        let body = match &request.body {
            Command::Pull { cursor } => {
                if apply_after_budget.load(Ordering::Acquire)
                    && !advanced.swap(true, Ordering::AcqRel)
                {
                    Response::Events(Some(drive_core::Batch {
                        from: cursor.clone(),
                        to: format!("{cursor}-advance"),
                        events: vec![drive_core::Event {
                            id: "incremental".into(),
                            change: drive_core::Change::Upsert(file("102", "B.doc")),
                        }],
                    }))
                } else {
                    Response::Events(None)
                }
            }
            Command::Wait { .. } => notification(true),
            _ => panic!("changed-notification server received an unexpected command"),
        };
        if transport::send(
            &mut socket,
            &Envelope {
                version: VERSION,
                id: request.id,
                body,
            },
            None,
        )
        .is_err()
        {
            break;
        }
    }
}

fn wait_until(deadline: Duration, mut condition: impl FnMut() -> bool) -> bool {
    let end = Instant::now() + deadline;
    while Instant::now() < end {
        if condition() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    condition()
}

/// AC #58-1: the mock returns `changed=true` notifications and empty pulls
/// forever. The runtime must leave `catching_up` when the configured catch-up
/// budget is spent, emit the structured no-progress event, expose the observed
/// backlog and keep cached reads available; incremental pulls keep running.
///
/// Run with the short limits (`--ignored`; see the issue report for the exact
/// command): `DRIVE_NATIVE_CATCHUP_BUDGET_MS=300`
/// `DRIVE_NATIVE_NO_PROGRESS_TIMEOUT_MS=60000` `DRIVE_NATIVE_NO_PROGRESS_STREAK=2`.
#[test]
#[ignore = "needs DRIVE_NATIVE_* short limits; see notes/work/issue-58.md"]
fn catch_up_budget_is_bounded_and_reports_no_progress() {
    let (_dir, store) = store_fixture();
    let (client, peer) = UnixStream::pair().unwrap();
    let apply_after_budget = Arc::new(AtomicBool::new(false));
    let advanced = Arc::new(AtomicBool::new(false));
    let server = {
        let apply_after_budget = apply_after_budget.clone();
        let advanced = advanced.clone();
        std::thread::spawn(move || changed_notification_server(peer, apply_after_budget, advanced))
    };
    let bridge = Arc::new(
        Bridge::from_connection(store.clone(), Arc::new(Rpc::new(client).unwrap())).unwrap(),
    );
    let runner = {
        let bridge = bridge.clone();
        std::thread::spawn(move || bridge.event_loop())
    };
    let projection =
        |bridge: &Bridge| serde_json::to_value(bridge.initialization.snapshot()).unwrap();
    let left_catching_up = wait_until(DEADLINE, || {
        let snapshot = projection(&bridge);
        snapshot["catching_up"].as_bool() == Some(false)
            && snapshot["budget_remaining_ms"].as_u64() == Some(0)
    });
    let snapshot = projection(&bridge);
    assert!(
        left_catching_up,
        "catching_up never left the unbounded catch-up loop; projection={snapshot}"
    );
    assert_eq!(
        snapshot["budget_remaining_ms"].as_u64(),
        Some(0),
        "the total catch-up budget must be reported as spent; projection={snapshot}"
    );
    assert_ne!(
        snapshot["phase"], "catching_up",
        "the phase must not keep claiming catching_up after the budget; projection={snapshot}"
    );
    let backlog = snapshot["backlog_estimate"]
        .as_u64()
        .expect("the observed backlog must be exposed after leaving catching_up");
    assert!(backlog >= 2, "backlog_estimate={backlog}");
    let no_progress = &snapshot["no_progress"];
    assert!(
        no_progress["streak"].as_u64().unwrap_or(0) >= 2,
        "a structured initialization.no_progress event must be recorded; projection={snapshot}"
    );
    assert!(
        no_progress["last_progress_age_ms"].as_u64().is_some(),
        "the no-progress event must carry last_progress_age; projection={snapshot}"
    );
    assert_eq!(
        snapshot["events_applied"].as_u64(),
        Some(0),
        "the mock applied no events"
    );
    // Incremental catch-up continues in the explicit degraded state: a real
    // advancing batch is still applied while `catching_up` stays false.
    apply_after_budget.store(true, Ordering::Release);
    assert!(
        wait_until(DEADLINE, || {
            let snapshot = projection(&bridge);
            snapshot["events_applied"].as_u64().unwrap_or(0) >= 1
        }),
        "incremental catch-up stopped after the budget was spent; projection={}",
        projection(&bridge)
    );
    assert_eq!(
        projection(&bridge)["catching_up"].as_bool(),
        Some(false),
        "the degraded state is sticky until the session settles"
    );
    // Cached data stays readable: leaving catching_up is a degraded state, not
    // a mount failure.
    let mut body = [0; 3];
    let handle = bridge.store.lock().unwrap().open_cached("101").unwrap();
    assert_eq!(handle.read_at(&mut body, 0).unwrap(), 3);
    assert_eq!(&body, b"old");
    bridge.disconnect();
    assert!(runner.join().unwrap().is_err());
    server.join().unwrap();
}

/// Build a server that returns `batches` monotonic cursor advances of
/// `batch_events` synthetic events each, then settles to an empty pull.
fn monotonic_events_server(mut socket: UnixStream, batches: usize, batch_events: usize) {
    let mut cursor = "1".to_string();
    let mut served = 0usize;
    while let Ok((request, _)) = transport::receive::<Envelope<Command>>(&mut socket) {
        let body = match &request.body {
            Command::Pull { .. } => {
                if served < batches {
                    let next = format!("p{}", served + 1);
                    let events = (0..batch_events)
                        .map(|index| drive_core::Event {
                            id: format!("e{served}-{index}"),
                            change: drive_core::Change::Upsert(Node {
                                id: format!("e{served}-{index}"),
                                parent: Some("1".into()),
                                name: format!("e{served}-{index}.txt"),
                                version: "v1".into(),
                                directory: false,
                                metadata: Some(NodeMetadata {
                                    size: 3,
                                    mtime: 0,
                                    mode: 0o644,
                                }),
                            }),
                        })
                        .collect();
                    let batch = drive_core::Batch {
                        from: std::mem::replace(&mut cursor, next.clone()),
                        to: next,
                        events,
                    };
                    served += 1;
                    Response::Events(Some(batch))
                } else {
                    Response::Events(None)
                }
            }
            Command::Wait { .. } => notification(false),
            _ => panic!("monotonic server received an unexpected command"),
        };
        if transport::send(
            &mut socket,
            &Envelope {
                version: VERSION,
                id: request.id,
                body,
            },
            None,
        )
        .is_err()
        {
            break;
        }
    }
}

/// AC #58-4 (event-loop half): a legitimate monotonic catch-up keeps applying
/// real store batches and is never reported as a no-progress stall.
///
/// Run with short limits (`--ignored`):
/// `DRIVE_NATIVE_CATCHUP_BUDGET_MS=60000` `DRIVE_NATIVE_NO_PROGRESS_TIMEOUT_MS=2000`
/// `DRIVE_NATIVE_NO_PROGRESS_STREAK=2`.
#[test]
#[ignore = "needs DRIVE_NATIVE_* short limits; see notes/work/issue-58.md"]
fn monotonic_catch_up_completes_without_watchdog_interference() {
    const BATCHES: usize = 4;
    const EVENTS: usize = 50;
    let (_dir, store) = store_fixture();
    let (client, peer) = UnixStream::pair().unwrap();
    let server = std::thread::spawn(move || monotonic_events_server(peer, BATCHES, EVENTS));
    let bridge = Arc::new(
        Bridge::from_connection(store.clone(), Arc::new(Rpc::new(client).unwrap())).unwrap(),
    );
    let runner = {
        let bridge = bridge.clone();
        std::thread::spawn(move || bridge.event_loop())
    };
    let drained = wait_until(Duration::from_secs(30), || {
        bridge.store.lock().unwrap().cursor().unwrap() == format!("p{BATCHES}")
    });
    let cursor = bridge.store.lock().unwrap().cursor().unwrap();
    let snapshot: serde_json::Value =
        serde_json::to_value(bridge.initialization.snapshot()).unwrap();
    assert!(
        drained,
        "monotonic catch-up did not drain (cursor={cursor}); projection={snapshot}"
    );
    assert_eq!(
        snapshot["events_applied"].as_u64(),
        Some((BATCHES * EVENTS) as u64)
    );
    assert_eq!(snapshot["cursor_advances"].as_u64(), Some(BATCHES as u64));
    assert!(
        snapshot["no_progress"].is_null(),
        "progress-based supervision must not fire on monotonic advances; projection={snapshot}"
    );
    if let Some(remaining) = snapshot["budget_remaining_ms"].as_u64() {
        assert!(
            remaining > 0,
            "a progressing catch-up must not burn the budget to zero; projection={snapshot}"
        );
    }
    bridge.disconnect();
    assert!(runner.join().unwrap().is_err());
    server.join().unwrap();
}
