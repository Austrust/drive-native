//! Upload-worker loop tests for issue #15.
//!
//! The worker must not wait for its idle poll when a signal or a due retry
//! deadline wakes it, must drain operations appended while a cycle was in
//! flight, and must defer hydration refreshes while uploads keep progressing
//! without ever dropping durable download intent. The NAS is mocked with a
//! `UnixStream` peer.
use super::*;
use drive_core::{NodeMetadata, sha256};
use std::{
    io::{Seek, SeekFrom, Write},
    sync::{Arc, mpsc, atomic::Ordering},
    time::{Duration, Instant},
};

const DEADLINE: Duration = Duration::from_secs(10);
/// A back-to-back drain must finish long before the idle poll (set to 5s in
/// that test), so this window proves the operation was not merely picked up by
/// the next timeout.
const DRAIN_DEADLINE: Duration = Duration::from_secs(2);

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

/// Root plus two files. `cached` selects which files have a published body;
/// `pending` queues one local edit on "2" (the pre-existing fixture shape).
fn store_fixture(
    cached: &[&str],
    pending: bool,
) -> (tempfile::TempDir, Arc<drive_core::SharedStore>) {
    let dir = tempfile::tempdir().unwrap();
    let store = Arc::new(drive_core::SharedStore::new(
        Store::open(dir.path()).unwrap(),
    ));
    {
        let mut s = store.lock().unwrap();
        s.install_remote_snapshot(
            &[
                root(),
                file("2", "A.doc"),
                file("3", "B.doc"),
                file("4", "C.doc"),
            ],
            "1",
            "fixture",
        )
        .unwrap();
        s.ensure_fs_root("1").unwrap();
        for id in cached {
            s.publish_download(
                &DownloadTicket {
                    id: (*id).into(),
                    version: "v1".into(),
                },
                b"old",
                &sha256(b"old"),
            )
            .unwrap();
        }
        if pending {
            s.save_local("2", b"new").unwrap();
        }
    }
    (dir, store)
}

fn connect(store: Arc<drive_core::SharedStore>, socket: UnixStream) -> Bridge {
    Bridge::connect(
        store,
        Arc::new(Rpc::new(socket).unwrap()),
        Login {
            host: "fixture.invalid".into(),
            tls_name: "fixture.invalid".into(),
            trusted_ca_pem: Vec::new(),
            username: "fixture".into(),
            password: String::new(),
            device_id: "fixture".into(),
            scope: drive_core::native::Scope {
                view_id: 1,
                node_id: 1,
                file_id: "1".into(),
                path: "/lab".into(),
            },
        },
    )
    .unwrap()
}

fn handshake(socket: &mut UnixStream) {
    let (hello, descriptor) = crate::transport::receive::<Envelope<Command>>(socket).unwrap();
    assert!(descriptor.is_none());
    assert!(matches!(hello.body, Command::Connect(_)));
    send(socket, hello.id, Response::Connected {
        restore: "fixture".into(),
    });
}

fn send(socket: &mut UnixStream, id: u64, body: Response) {
    crate::transport::send(
        socket,
        &Envelope {
            version: VERSION,
            id,
            body,
        },
        None,
    )
    .unwrap();
}

fn send_content(socket: &mut UnixStream, id: u64, identity: String, version: String) {
    let mut body = tempfile::tempfile().unwrap();
    body.write_all(b"new").unwrap();
    body.seek(SeekFrom::Start(0)).unwrap();
    crate::transport::send(
        socket,
        &Envelope {
            version: VERSION,
            id,
            body: Response::Content {
                identity,
                version,
                size: 3,
                sha256: sha256(b"new"),
            },
        },
        Some(&OwnedFd::from(body)),
    )
    .unwrap();
}

/// Mock upload worker: every request must be an ordinary upload. When
/// `append` is set, the first acknowledged cycle gains one more durable local
/// edit for that node *while its upload is still on the wire*; nothing signals
/// the worker, so only the back-to-back drain can observe it.
fn upload_server(
    mut socket: UnixStream,
    store: Arc<drive_core::SharedStore>,
    uploads: Arc<std::sync::atomic::AtomicUsize>,
    append: Option<&'static str>,
) {
    handshake(&mut socket);
    let mut version = 2u64;
    while let Ok((request, descriptor)) = crate::transport::receive::<Envelope<Command>>(&mut socket)
    {
        let Command::Upload {
            mut node,
            mtime,
            reconcile,
            ..
        } = request.body
        else {
            panic!("upload server received a non-upload command");
        };
        assert!(!reconcile, "ordinary uploads must not replay as reconciliation");
        assert!(descriptor.is_some(), "upload must carry the body descriptor");
        let count = uploads.fetch_add(1, Ordering::SeqCst) + 1;
        node.version = format!("v{version}");
        version += 1;
        node.metadata.as_mut().expect("file metadata").mtime =
            u64::from(mtime.expect("upload carries mtime"));
        if count == 1 {
            if let Some(id) = append {
                store.lock().unwrap().save_local(id, b"later").unwrap();
            }
        }
        send(&mut socket, request.id, Response::Committed { node });
    }
}

/// Mock hydration worker: answers each Download with a verified body descriptor
/// and reports the requested node id on `seen`.
fn download_server(
    mut socket: UnixStream,
    downloads: Arc<std::sync::atomic::AtomicUsize>,
    seen: mpsc::Sender<String>,
) {
    handshake(&mut socket);
    while let Ok((request, descriptor)) = crate::transport::receive::<Envelope<Command>>(&mut socket)
    {
        let Command::Download { node } = request.body else {
            panic!("download server received a non-download command");
        };
        assert!(descriptor.is_none());
        downloads.fetch_add(1, Ordering::SeqCst);
        send_content(&mut socket, request.id, node.id.clone(), node.version.clone());
        seen.send(node.id).ok();
    }
}

fn wait_for(mut condition: impl FnMut() -> bool, deadline: Duration) {
    let until = Instant::now() + deadline;
    while !condition() {
        assert!(Instant::now() < until, "condition not met within {deadline:?}");
        std::thread::sleep(Duration::from_millis(5));
    }
}

fn stop_worker(bridge: &Arc<Bridge>, stop: &Arc<AtomicBool>, worker: std::thread::JoinHandle<()>) {
    stop.store(true, Ordering::Release);
    bridge.signal_upload();
    worker.join().unwrap();
}

#[test]
fn confirmed_upload_drains_an_operation_created_mid_cycle() {
    let (_dir, store) = store_fixture(&["2", "3"], true);
    let (client, peer) = UnixStream::pair().unwrap();
    let uploads = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let server = {
        let store = store.clone();
        let uploads = uploads.clone();
        std::thread::spawn(move || upload_server(peer, store, uploads, Some("3")))
    };
    let bridge = Arc::new(connect(store.clone(), client));
    let stop = Arc::new(AtomicBool::new(false));
    let worker = {
        let bridge = bridge.clone();
        let stop = stop.clone();
        // The idle poll is deliberately far longer than the drain window: the
        // appended operation can only be uploaded by a back-to-back cycle.
        std::thread::spawn(move || {
            bridge.upload_loop_with(
                &stop,
                Duration::from_secs(5),
                UPLOAD_REFRESH_MAX_INTERVAL,
            )
        })
    };
    wait_for(
        || store.lock().unwrap().pending_operations().unwrap() == 0,
        DRAIN_DEADLINE,
    );
    assert_eq!(
        uploads.load(Ordering::SeqCst),
        2,
        "the operation appended mid-cycle must be drained without a signal"
    );
    stop_worker(&bridge, &stop, worker);
    bridge.disconnect();
    server.join().unwrap();
}

#[test]
fn productive_uploads_defer_hydration_until_the_queue_is_quiet() {
    let (_dir, store) = store_fixture(&["2"], false);
    let (client, peer) = UnixStream::pair().unwrap();
    let uploads = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let server = {
        let store = store.clone();
        let uploads = uploads.clone();
        std::thread::spawn(move || upload_server(peer, store, uploads, None))
    };
    let bridge = Arc::new(connect(store.clone(), client));
    let stop = Arc::new(AtomicBool::new(false));
    let worker = {
        let bridge = bridge.clone();
        let stop = stop.clone();
        // Production-like idle poll, so the deferred hydration can settle on
        // the next quiet tick.
        std::thread::spawn(move || {
            bridge.upload_loop_with(
                &stop,
                Duration::from_secs(1),
                UPLOAD_REFRESH_MAX_INTERVAL,
            )
        })
    };
    // Startup hydration runs once on the initial idle signal.
    wait_for(
        || bridge.refresh_runs.load(Ordering::Relaxed) >= 1,
        DEADLINE,
    );
    // Three distinct nodes queued back-to-back without waiting for their
    // confirmations: every cycle confirms an upload and must defer hydration
    // instead of rescanning downloads_needed once per operation.
    for id in ["2", "3", "4"] {
        store.lock().unwrap().save_local(id, b"new").unwrap();
        bridge.signal_upload();
    }
    wait_for(
        || {
            uploads.load(Ordering::SeqCst) == 3
                && store.lock().unwrap().pending_operations().unwrap() == 0
        },
        DEADLINE,
    );
    // Exactly one more refresh settles the deferred hydration after the burst;
    // per-upload refreshes (which would pay the whole downloads_needed scan on
    // every write) would make this grow with the number of operations.
    wait_for(
        || bridge.refresh_runs.load(Ordering::Relaxed) >= 2,
        DEADLINE,
    );
    std::thread::sleep(Duration::from_millis(300));
    assert_eq!(
        bridge.refresh_runs.load(Ordering::Relaxed),
        2,
        "hydration must not be re-enumerated for every confirmed upload"
    );
    stop_worker(&bridge, &stop, worker);
    bridge.disconnect();
    server.join().unwrap();
}

#[test]
fn pin_signal_starts_hydration_without_waiting_for_the_idle_poll() {
    let (_dir, store) = store_fixture(&[], false);
    let (client, peer) = UnixStream::pair().unwrap();
    let downloads = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let (tx, rx) = mpsc::channel();
    let server = {
        let downloads = downloads.clone();
        std::thread::spawn(move || download_server(peer, downloads, tx))
    };
    let bridge = Arc::new(connect(store.clone(), client));
    let stop = Arc::new(AtomicBool::new(false));
    let worker = {
        let bridge = bridge.clone();
        let stop = stop.clone();
        // A 5s idle poll makes the 900ms assertion below meaningful: only the
        // signal path can hydrate that fast.
        std::thread::spawn(move || {
            bridge.upload_loop_with(
                &stop,
                Duration::from_secs(5),
                UPLOAD_REFRESH_MAX_INTERVAL,
            )
        })
    };
    // Let the startup hydration settle so the timed pin is the only wake.
    wait_for(
        || bridge.refresh_runs.load(Ordering::Relaxed) >= 1,
        DEADLINE,
    );
    std::thread::sleep(Duration::from_millis(80));
    let started = Instant::now();
    store.lock().unwrap().pin("3", true).unwrap();
    bridge.signal_upload();
    let seen = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("a pin must wake hydration instead of waiting for the idle poll");
    assert_eq!(seen, "3");
    assert!(
        started.elapsed() < Duration::from_millis(900),
        "hydration started after the idle poll ({:?})",
        started.elapsed()
    );
    assert_eq!(downloads.load(Ordering::SeqCst), 1);
    stop_worker(&bridge, &stop, worker);
    bridge.disconnect();
    server.join().unwrap();
}

#[test]
fn due_download_retry_refreshes_without_a_signal() {
    let (_dir, store) = store_fixture(&[], false);
    let (client, peer) = UnixStream::pair().unwrap();
    let downloads = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let (tx, rx) = mpsc::channel();
    let server = {
        let downloads = downloads.clone();
        std::thread::spawn(move || download_server(peer, downloads, tx))
    };
    let bridge = Arc::new(connect(store.clone(), client));
    let stop = Arc::new(AtomicBool::new(false));
    let worker = {
        let bridge = bridge.clone();
        let stop = stop.clone();
        // Short idle poll so the retry deadline is observed promptly.
        std::thread::spawn(move || {
            bridge.upload_loop_with(
                &stop,
                Duration::from_millis(200),
                UPLOAD_REFRESH_MAX_INTERVAL,
            )
        })
    };
    wait_for(
        || bridge.refresh_runs.load(Ordering::Relaxed) >= 1,
        DEADLINE,
    );
    // A durable retry record whose deadline is already due: the idle poll must
    // observe `next_download_retry()` and refresh without any signal.
    store
        .lock()
        .unwrap()
        .fail_download(
            &DownloadTicket {
                id: "3".into(),
                version: "v1".into(),
            },
            DownloadFailureKind::Transport,
            None,
            unix_seconds().saturating_sub(2),
        )
        .unwrap();
    let seen = rx
        .recv_timeout(Duration::from_secs(4))
        .expect("a due retry must wake hydration without a signal");
    assert_eq!(seen, "3");
    assert!(bridge.refresh_runs.load(Ordering::Relaxed) >= 2);
    stop_worker(&bridge, &stop, worker);
    bridge.disconnect();
    server.join().unwrap();
}

#[test]
fn offline_and_paused_worker_retains_the_queue_without_hydrating() {
    let (_dir, store) = store_fixture(&[], true);
    let (client, peer) = UnixStream::pair().unwrap();
    drop(peer);
    let bridge = Arc::new(
        Bridge::from_connection(store.clone(), Arc::new(Rpc::new(client).unwrap())).unwrap(),
    );
    store.lock().unwrap().set_sync_paused(true).unwrap();
    bridge.disconnect();
    let stop = Arc::new(AtomicBool::new(false));
    let worker = {
        let bridge = bridge.clone();
        let stop = stop.clone();
        // Short poll: the loop keeps waking, but must skip every cycle while
        // the worker is disconnected and the queue is paused.
        std::thread::spawn(move || {
            bridge.upload_loop_with(
                &stop,
                Duration::from_millis(100),
                UPLOAD_REFRESH_MAX_INTERVAL,
            )
        })
    };
    std::thread::sleep(Duration::from_millis(300));
    assert_eq!(
        store.lock().unwrap().pending_operations().unwrap(),
        1,
        "a paused/offline queue must retain pending uploads"
    );
    assert_eq!(
        bridge.refresh_runs.load(Ordering::Relaxed),
        0,
        "an offline worker must not enumerate download intent"
    );
    stop_worker(&bridge, &stop, worker);
}
