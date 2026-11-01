//! Real local RPC/SCM_RIGHTS owner tests: no worker process, NAS or credentials.
use super::*;
use crate::transport;
use drive_core::{NodeMetadata, native::RemoteSnapshot};
use std::{
    io::{Seek, SeekFrom, Write},
    os::fd::OwnedFd,
    sync::mpsc,
};

const TASK: &str = "12ac1b65-6419-4071-a0b0-853e9d85f8a4";

fn snapshot(restore: &str, cursor: u64) -> RemoteSnapshot {
    let scope = Scope {
        view_id: 1,
        node_id: 1,
        file_id: "100".into(),
        path: "/Lab".into(),
    };
    let nodes = [("100", None, "Lab", true), ("101", Some("100"), "A.txt", false),
        ("102", Some("100"), "B.txt", false), ("200", Some("100"), "Sub", true),
        ("201", Some("200"), "C.txt", false)].into_iter().map(|(id,parent,name,directory)| Node {
        id: id.into(), parent: parent.map(str::to_owned), name: name.into(), directory,
        version: serde_json::json!({"schema":1,"restore":restore,"view":1,"root":"100",
            "file":id,"sync":cursor,"max":cursor,"size":if directory {0} else {3},"hash":"","directory":directory}).to_string(),
        metadata: Some(NodeMetadata { size: if directory {0} else {3}, mtime: 0, mode: if directory {0o755} else {0o644} }),
    }).collect();
    RemoteSnapshot {
        scope,
        restore: restore.into(),
        cursor: format!("native-v1:{cursor}"),
        nodes,
        inaccessible: vec![],
        skipped_symlinks: 0,
    }
}

fn complete_store(path: &std::path::Path, cursor: u64, dirty: bool) -> Store {
    let mut store = Store::open(path).unwrap();
    let data = snapshot("server", cursor);
    store
        .bind_native_scope(&data.restore, &serde_json::to_string(&data.scope).unwrap())
        .unwrap();
    store.install_native_snapshot(&data, "").unwrap();
    store.ensure_fs_root("100").unwrap();
    for node in data.nodes.iter().filter(|node| !node.directory) {
        store
            .publish_download(
                &DownloadTicket {
                    id: node.id.clone(),
                    version: node.version.clone(),
                },
                b"old",
                &drive_core::sha256(b"old"),
            )
            .unwrap();
    }
    if dirty {
        let node = store.node("101").unwrap();
        let source = store.open_cached("101").unwrap();
        let draft = store
            .begin_draft("101", &node.version, Some(&source))
            .unwrap();
        store.draft_write(&draft, 0, b"new", false).unwrap();
        store.checkpoint_draft(&draft).unwrap().unwrap();
        drop(draft);
        drop(source);
        store
            .rename_local("100", "B.txt", "100", "Renamed.txt", false)
            .unwrap();
    }
    store
}

fn protected(store: &Store) -> serde_json::Value {
    let draft = store.open_dirty_draft("101").unwrap().unwrap();
    let mut body = [0; 3];
    assert_eq!(draft.read_at(&mut body, 0).unwrap(), 3);
    assert_eq!(&body, b"new");
    let cache = store.open_cached("102").unwrap();
    assert_eq!(cache.read_at(&mut body, 0).unwrap(), 3);
    assert_eq!(&body, b"old");
    let rename = store.next_rename().unwrap().unwrap();
    serde_json::json!({"cursor":store.cursor().unwrap(),"draft":draft.token,
        "pending":store.upload_operations("pending").unwrap(),"rename":rename.job.operation,
        "renamed":store.lookup_node("100","Renamed.txt").unwrap().id})
}

enum Answer {
    Body(Body),
    Ordinary,
    OrdinaryBatch(Option<Batch>),
    Fail(&'static str),
    Disconnect,
}

fn standard(data: &RemoteSnapshot, command: &Command) -> Answer {
    match command {
        Command::Initialization(Request::Seed) => Answer::Body(Body::Seed {
            cursor: data.cursor.clone(),
            root: data.nodes[0].clone(),
        }),
        Command::Initialization(Request::Root) => Answer::Body(Body::Root {
            node: data.nodes[0].clone(),
        }),
        Command::Initialization(Request::Pull { .. }) => Answer::Body(Body::Events(None)),
        Command::Initialization(Request::Directory { node, path }) => {
            assert_eq!(path, if node.id == "100" { "/Lab" } else { "/Lab/Sub" });
            Answer::Body(Body::Directory {
                identity: node.id.clone(),
                denied: false,
                children: data
                    .nodes
                    .iter()
                    .filter(|child| child.parent.as_deref() == Some(node.id.as_str()))
                    .cloned()
                    .collect(),
            })
        }
        Command::Pull { .. } => Answer::Ordinary,
        _ => Answer::Fail("synthetic_unexpected_command"),
    }
}

fn label(command: &Command) -> String {
    match command {
        Command::Initialization(Request::Seed) => "seed".into(),
        Command::Initialization(Request::Root) => "root".into(),
        Command::Initialization(Request::Directory { node, .. }) => {
            format!("directory:{}", node.id)
        }
        Command::Initialization(Request::Pull { cursor }) => format!("candidate-pull:{cursor}"),
        Command::Pull { cursor } => format!("ordinary-pull:{cursor}"),
        Command::RecoverySnapshot => "legacy-snapshot".into(),
        _ => "unexpected-write-or-control".into(),
    }
}

struct Peer {
    rpc: Arc<Rpc>,
    log: Arc<Mutex<Vec<String>>>,
    thread: Option<std::thread::JoinHandle<()>>,
}
impl Drop for Peer {
    fn drop(&mut self) {
        self.rpc.close();
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}
impl Peer {
    fn new(
        data: RemoteSnapshot,
        mut reply: impl FnMut(&Command) -> Answer + Send + 'static,
    ) -> Self {
        let (socket, mut remote) = UnixStream::pair().unwrap();
        let rpc = Arc::new(Rpc::new(socket).unwrap());
        let log = Arc::new(Mutex::new(Vec::new()));
        let requests = log.clone();
        let thread = std::thread::spawn(move || {
            while let Ok((request, _)) = transport::receive::<Envelope<Command>>(&mut remote) {
                let count = {
                    let mut log = requests.lock().unwrap();
                    log.push(label(&request.body));
                    log.len()
                };
                assert!(count < 128, "unexpected request loop");
                let answer = reply(&request.body);
                let (body, descriptor) = match answer {
                    Answer::Disconnect => break,
                    Answer::Ordinary => (Response::Events(None), None),
                    Answer::OrdinaryBatch(batch) => (Response::Events(batch), None),
                    Answer::Fail(category) => (
                        Response::Failed {
                            category: category.into(),
                        },
                        None,
                    ),
                    Answer::Body(body) => {
                        let bytes = serde_json::to_vec(&InitializationReply {
                            scope: data.scope.clone(),
                            restore: data.restore.clone(),
                            body,
                        })
                        .unwrap();
                        let mut file = tempfile::tempfile().unwrap();
                        file.write_all(&bytes).unwrap();
                        file.seek(SeekFrom::Start(0)).unwrap();
                        (
                            Response::Initialization {
                                size: bytes.len() as u64,
                                sha256: drive_core::sha256(&bytes),
                            },
                            Some(OwnedFd::from(file)),
                        )
                    }
                };
                if transport::send(
                    &mut remote,
                    &Envelope {
                        version: VERSION,
                        id: request.id,
                        body,
                    },
                    descriptor.as_ref(),
                )
                .is_err()
                {
                    break;
                }
                // Keep the socket open until the owner disconnects. Sending the
                // final response is not proof that FSD has committed it yet.
            }
        });
        Self {
            rpc,
            log,
            thread: Some(thread),
        }
    }
    fn standard(data: RemoteSnapshot) -> Self {
        let replies = data.clone();
        Self::new(data, move |request| standard(&replies, request))
    }
    fn finish(mut self) -> Vec<String> {
        self.rpc.close();
        self.thread.take().unwrap().join().unwrap();
        let log = self.log.lock().unwrap().clone();
        log
    }
}

fn bridge(store: Store, peer: &Peer, restore: &str) -> Arc<Bridge> {
    let bridge = Arc::new(Bridge::offline(Arc::new(drive_core::SharedStore::new(store))).unwrap());
    bridge.enable_initialization_owner(TASK).unwrap();
    bridge
        .attach(
            peer.rpc.clone(),
            restore,
            &serde_json::to_string(&snapshot(restore, 7).scope).unwrap(),
        )
        .unwrap();
    bridge
}

#[test]
fn cold_owner_uses_reliable_steps_and_commits_only_after_separate_fence() {
    let dir = tempfile::tempdir().unwrap();
    let data = snapshot("server", 7);
    let peer = Peer::standard(data.clone());
    let bridge = bridge(Store::open(dir.path()).unwrap(), &peer, "server");
    assert!(bridge.initial_pull().unwrap());
    let store = bridge.store.lock().unwrap();
    assert_eq!(store.cursor().unwrap(), data.cursor);
    assert_eq!(
        store.initialization_status().unwrap().unwrap().state,
        "ready"
    );
    assert!(!store.initialization_writeback_blocked().unwrap());
    assert_eq!(store.node("201").unwrap().name, "C.txt");
    drop(store);
    assert!(!bridge.session_pending.load(Ordering::Acquire));
    let log = peer.finish();
    assert_eq!(
        log,
        vec![
            "seed",
            "candidate-pull:native-v1:7",
            "directory:100",
            "directory:200",
            "candidate-pull:native-v1:7"
        ]
    );
}

#[test]
fn deferred_file_parent_pauses_pull_while_multiple_directory_rounds_make_progress() {
    let dir = tempfile::tempdir().unwrap();
    let mut data = snapshot("server", 7);
    let template = data.nodes[3].clone();
    let mut leaf = data.nodes[4].clone();
    data.nodes.truncate(1);
    // More than one owner directory quantum: a repeated Pull before discovering
    // the parent would consume another page and reproduce the original churn.
    for depth in 0..20 {
        let mut node = template.clone();
        node.id = (200 + depth).to_string();
        node.parent = Some(if depth == 0 {
            "100".into()
        } else {
            (199 + depth).to_string()
        });
        node.name = format!("D{depth}");
        let mut token: serde_json::Value = serde_json::from_str(&node.version).unwrap();
        token["file"] = node.id.clone().into();
        node.version = token.to_string();
        data.nodes.push(node);
    }
    leaf.id = "999".into();
    leaf.parent = Some("219".into());
    let mut token: serde_json::Value = serde_json::from_str(&leaf.version).unwrap();
    token["file"] = leaf.id.clone().into();
    token["sync"] = 9.into();
    token["max"] = 9.into();
    leaf.version = token.to_string();
    data.nodes.push(leaf.clone());
    let replies = data.clone();
    let mut sent = false;
    let mut final_parent_discovered = false;
    let peer = Peer::new(data, move |request| match request {
        Command::Initialization(Request::Pull { cursor }) if !sent => {
            assert_eq!(cursor, "native-v1:7");
            sent = true;
            let mut older = replies.nodes.last().unwrap().clone();
            let mut token: serde_json::Value = serde_json::from_str(&older.version).unwrap();
            token["sync"] = 8.into();
            token["max"] = 8.into();
            older.version = token.to_string();
            Answer::Body(Body::Events(Some(Batch {
                from: "native-v1:7".into(),
                to: "native-v1:8".into(),
                events: vec![drive_core::Event {
                    id: "missing-parent".into(),
                    change: drive_core::Change::Upsert(older),
                }],
            })))
        }
        Command::Initialization(Request::Pull { cursor }) => {
            assert!(
                final_parent_discovered,
                "Pull resumed before the dependency resolved"
            );
            assert_eq!(cursor, "native-v1:8");
            Answer::Body(Body::Events(None))
        }
        Command::Initialization(Request::Directory { node, .. }) => {
            final_parent_discovered |= node.id == "218";
            Answer::Body(Body::Directory {
                identity: node.id.clone(),
                denied: false,
                children: replies
                    .nodes
                    .iter()
                    .filter(|child| child.parent.as_deref() == Some(node.id.as_str()))
                    .cloned()
                    .collect(),
            })
        }
        _ => standard(&replies, request),
    });
    let bridge = bridge(Store::open(dir.path()).unwrap(), &peer, "server");
    assert!(bridge.initial_pull().unwrap());
    let store = bridge.store.lock().unwrap();
    assert_eq!(store.node("999").unwrap(), leaf);
    assert_eq!(store.cursor().unwrap(), "native-v1:8");
    assert_eq!(
        store.initialization_status().unwrap().unwrap().state,
        "ready"
    );
    drop(store);
    let log = peer.finish();
    assert_eq!(
        log.iter()
            .filter(|item| item.as_str() == "directory:100")
            .count(),
        1
    );
    assert_eq!(
        log.iter()
            .filter(|item| item.starts_with("candidate-pull:"))
            .count(),
        3
    );
}

#[test]
fn permanently_absent_parent_gets_one_conservative_recheck_not_an_owner_spin() {
    let dir = tempfile::tempdir().unwrap();
    let data = snapshot("server", 7);
    let replies = data.clone();
    let mut sent = false;
    let peer = Peer::new(data, move |request| {
        if matches!(request, Command::Initialization(Request::Pull { .. })) && !sent {
            sent = true;
            let mut unknown = replies.nodes[4].clone();
            unknown.parent = Some("999".into());
            Answer::Body(Body::Events(Some(Batch {
                from: "native-v1:7".into(),
                to: "native-v1:8".into(),
                events: vec![drive_core::Event {
                    id: "unresolved".into(),
                    change: drive_core::Change::Upsert(unknown),
                }],
            })))
        } else {
            standard(&replies, request)
        }
    });
    let bridge = bridge(Store::open(dir.path()).unwrap(), &peer, "server");
    assert!(bridge.initial_pull().unwrap());
    assert_eq!(
        bridge.store.lock().unwrap().cursor().unwrap(),
        "native-v1:8"
    );
    let log = peer.finish();
    assert_eq!(
        log.iter()
            .filter(|item| item.as_str() == "directory:100")
            .count(),
        2
    );
    assert_eq!(
        log.iter()
            .filter(|item| item.as_str() == "directory:200")
            .count(),
        2
    );
}

#[test]
fn deferred_page_directory_race_resumes_pull_before_relisting_changed_path() {
    run_deferred_page_directory_race("native_snapshot_changed", true);
}

#[test]
fn deferred_page_does_not_turn_transport_permission_or_root_errors_into_rechecks() {
    for category in [
        "transport",
        "native_response_timeout",
        "native_server_code:12291",
        "native_root_unavailable",
    ] {
        run_deferred_page_directory_race(category, false);
    }
}

fn run_deferred_page_directory_race(category: &'static str, expect_success: bool) {
    let dir = tempfile::tempdir().unwrap();
    let mut store = complete_store(dir.path(), 7, true);
    let before = protected(&store);
    let old = snapshot("server", 7);
    let binding = InitializationBinding {
        task_id: TASK.into(),
        scope: serde_json::to_value(&old.scope).unwrap(),
        restore: "server".into(),
        start_cursor: old.cursor.clone(),
        expected_cursor: old.cursor.clone(),
    };
    let run = store
        .begin_initialization(&binding, &old.nodes[0], InitializationBudget::default())
        .unwrap();
    let ticket = store.begin_initialization_directory(&run, "100").unwrap();
    store
        .store_initialization_directory(&ticket, 1, &old.nodes[1..4])
        .unwrap();
    let mut data = snapshot("server", 9);
    data.nodes[3].name = "MovedSub".into();
    data.nodes[4].parent = Some("300".into());
    let mut nested = data.nodes[3].clone();
    nested.id = "300".into();
    nested.parent = Some("200".into());
    nested.name = "Nested".into();
    let mut token: serde_json::Value = serde_json::from_str(&nested.version).unwrap();
    token["file"] = "300".into();
    nested.version = token.to_string();
    data.nodes.push(nested);
    let replies = data.clone();
    let paths = Arc::new(Mutex::new(Vec::new()));
    let seen = paths.clone();
    let peer = Peer::new(data, move |request| match request {
        Command::Initialization(Request::Pull { cursor }) if cursor == "native-v1:7" => {
            Answer::Body(Body::Events(Some(Batch {
                from: cursor.clone(),
                to: "native-v1:8".into(),
                events: vec![drive_core::Event {
                    id: "unknown-parent-page".into(),
                    change: drive_core::Change::Upsert(replies.nodes[4].clone()),
                }],
            })))
        }
        Command::Initialization(Request::Pull { cursor }) if cursor == "native-v1:8" => {
            Answer::Body(Body::Events(Some(Batch {
                from: cursor.clone(),
                to: "native-v1:9".into(),
                events: vec![drive_core::Event {
                    id: "directory-moved-next-page".into(),
                    change: drive_core::Change::Upsert(replies.nodes[3].clone()),
                }],
            })))
        }
        Command::Initialization(Request::Directory { node, path }) => {
            seen.lock().unwrap().push(path.clone());
            if path == "/Lab/Sub" {
                return Answer::Fail(category);
            }
            assert_eq!(
                path,
                match node.id.as_str() {
                    "100" => "/Lab",
                    "200" => "/Lab/MovedSub",
                    "300" => "/Lab/MovedSub/Nested",
                    _ => panic!("unexpected directory"),
                }
            );
            Answer::Body(Body::Directory {
                identity: node.id.clone(),
                denied: false,
                children: replies
                    .nodes
                    .iter()
                    .filter(|child| child.parent.as_deref() == Some(node.id.as_str()))
                    .cloned()
                    .collect(),
            })
        }
        _ => standard(&replies, request),
    });
    let bridge = bridge(store, &peer, "server");
    let result = bridge.initial_pull();
    if result.is_err() {
        let store = bridge.store.lock().unwrap();
        assert_eq!(protected(&store), before);
        assert!(store.initialization_writeback_blocked().unwrap());
    }
    if !expect_success {
        assert!(result.is_err());
        let log = peer.finish();
        assert!(!log.iter().any(|s| s == "candidate-pull:native-v1:8"));
        return;
    }
    assert!(
        result.is_ok(),
        "coverage-race recovery failed: {result:?}; requests={:?}; paths={:?}",
        peer.log.lock().unwrap(),
        paths.lock().unwrap()
    );
    let store = bridge.store.lock().unwrap();
    assert_eq!(store.cursor().unwrap(), "native-v1:9");
    assert_eq!(store.node("200").unwrap().name, "MovedSub");
    assert_eq!(store.node("201").unwrap().parent.as_deref(), Some("300"));
    let mut after = protected(&store);
    after["cursor"] = before["cursor"].clone();
    assert_eq!(after, before);
    drop(store);
    let log = peer.finish();
    let pull = log
        .iter()
        .position(|s| s == "candidate-pull:native-v1:8")
        .unwrap();
    let root_list = log.iter().position(|s| s == "directory:100").unwrap();
    assert!(pull < root_list);
    let paths = paths.lock().unwrap();
    assert!(paths.iter().filter(|s| s.as_str() == "/Lab/Sub").count() <= 3);
    assert!(paths.iter().any(|s| s == "/Lab/MovedSub/Nested"));
}

#[test]
fn repeated_coverage_races_exhaust_budget_without_publishing_or_losing_dirty_state() {
    let dir = tempfile::tempdir().unwrap();
    let mut store = complete_store(dir.path(), 7, true);
    let before = protected(&store);
    let data = snapshot("server", 7);
    let binding = InitializationBinding {
        task_id: TASK.into(),
        scope: serde_json::to_value(&data.scope).unwrap(),
        restore: "server".into(),
        start_cursor: data.cursor.clone(),
        expected_cursor: data.cursor.clone(),
    };
    let run = store
        .begin_initialization(&binding, &data.nodes[0], InitializationBudget::default())
        .unwrap();
    let ticket = store.begin_initialization_directory(&run, "100").unwrap();
    store
        .store_initialization_directory(&ticket, 1, &data.nodes[1..4])
        .unwrap();
    let replies = data.clone();
    let peer = Peer::new(data, move |request| match request {
        Command::Initialization(Request::Pull { cursor }) => {
            let n = number(cursor).unwrap();
            let mut file = replies.nodes[4].clone();
            file.parent = Some("300".into());
            Answer::Body(Body::Events(Some(Batch {
                from: cursor.clone(),
                to: format!("native-v1:{}", n + 1),
                events: vec![drive_core::Event {
                    id: format!("missing-{n}"),
                    change: drive_core::Change::Upsert(file),
                }],
            })))
        }
        Command::Initialization(Request::Directory { node, .. }) if node.id == "200" => {
            Answer::Fail("native_snapshot_changed")
        }
        _ => standard(&replies, request),
    });
    let bridge = bridge(store, &peer, "server");
    let error = bridge.initial_pull().unwrap_err();
    assert!(
        error.to_string().contains("coverage race budget exceeded"),
        "{error}"
    );
    let store = bridge.store.lock().unwrap();
    assert_eq!(protected(&store), before);
    let state = store.initialization_status().unwrap().unwrap();
    assert_eq!((state.applied_sequence, state.received_sequence), (16, 17));
    assert!(store.initialization_writeback_blocked().unwrap());
    drop(store);
    let log = peer.finish();
    assert_eq!(
        log.iter()
            .filter(|s| s.starts_with("candidate-pull:"))
            .count(),
        17
    );
    assert!(log.iter().filter(|s| s.as_str() == "directory:200").count() <= 51);
}

#[test]
fn interrupted_active_and_failed_candidates_resume_received_root_after_store_reopen() {
    for failed in [false, true] {
        let dir = tempfile::tempdir().unwrap();
        let data = snapshot("server", 7);
        let replies = data.clone();
        let peer = Peer::new(data, move |request| {
            if matches!(request,Command::Initialization(Request::Directory{node,..}) if node.id=="200")
            {
                Answer::Disconnect
            } else {
                standard(&replies, request)
            }
        });
        let first = bridge(Store::open(dir.path()).unwrap(), &peer, "server");
        assert!(first.initial_pull().is_err());
        let old_run = {
            let mut store = first.store.lock().unwrap();
            let state = store.initialization_status().unwrap().unwrap();
            assert_eq!(
                store
                    .initialization_coverage(&state.run)
                    .unwrap()
                    .iter()
                    .find(|row| row.directory == "100")
                    .unwrap()
                    .state,
                "received"
            );
            if failed {
                store
                    .fail_initialization(&state.run, "test_interrupted")
                    .unwrap();
            }
            state.run
        };
        let first_log = peer.finish();
        assert!(first_log.contains(&"directory:100".into()));
        drop(first);
        let reopened = Store::open(dir.path()).unwrap();
        assert!(reopened.initialization_status().unwrap().unwrap().run.epoch > old_run.epoch);
        // Current root metadata may be newer; resume never seeds again and
        // retains its original candidate cursor instead of skipping history.
        let data = snapshot("server", 99);
        let peer = Peer::standard(data);
        let second = bridge(reopened, &peer, "server");
        assert!(second.initial_pull().unwrap());
        let store = second.store.lock().unwrap();
        let state = store.initialization_status().unwrap().unwrap();
        assert_eq!(state.run.run_id, old_run.run_id);
        assert_eq!(state.binding.start_cursor, "native-v1:7");
        assert_eq!(store.cursor().unwrap(), "native-v1:7");
        drop(store);
        let log = peer.finish();
        assert!(log.contains(&"root".into()) && log.contains(&"directory:200".into()));
        assert!(!log.contains(&"directory:100".into()));
        assert!(!log.contains(&"seed".into()));
        assert!(!log.iter().any(|item| item.starts_with("ordinary-")
            || item == "legacy-snapshot"
            || item.contains(":99")));
    }
}

#[test]
fn complete_store_reconnect_uses_ordinary_cursor_pull_without_seed() {
    let dir = tempfile::tempdir().unwrap();
    let peer = Peer::standard(snapshot("server", 7));
    let bridge = bridge(complete_store(dir.path(), 7, false), &peer, "server");
    assert!(!bridge.initial_pull().unwrap());
    assert!(
        bridge
            .store
            .lock()
            .unwrap()
            .initialization_status()
            .unwrap()
            .is_none()
    );
    assert!(!bridge.session_pending.load(Ordering::Acquire));
    assert_eq!(peer.finish(), vec!["ordinary-pull:native-v1:7"]);
}

#[test]
fn changed_restore_accepts_its_own_lower_cursor_without_pulling_old_domain() {
    let dir = tempfile::tempdir().unwrap();
    let peer = Peer::standard(snapshot("new-server", 3));
    let bridge = bridge(complete_store(dir.path(), 20, false), &peer, "new-server");
    assert!(bridge.initial_pull().unwrap());
    let store = bridge.store.lock().unwrap();
    assert_eq!(store.cursor().unwrap(), "native-v1:3");
    assert_eq!(
        store.native_scope_binding().unwrap().unwrap().0,
        "new-server"
    );
    drop(store);
    let log = peer.finish();
    assert!(
        !log.iter()
            .any(|item| item == "candidate-pull:native-v1:20" || item.starts_with("ordinary-")),
        "unexpected old-domain request: {log:?}"
    );
}

#[test]
fn same_restore_lower_watermark_requires_explicit_reset_response() {
    for reset in [false, true] {
        let dir = tempfile::tempdir().unwrap();
        let data = snapshot("server", 3);
        let replies = data.clone();
        let peer = Peer::new(data, move |request| {
            if reset
                && matches!(request,Command::Initialization(Request::Pull{cursor}) if cursor=="native-v1:20")
            {
                Answer::Fail("native_cursor_reset")
            } else {
                standard(&replies, request)
            }
        });
        let bridge = bridge(complete_store(dir.path(), 20, false), &peer, "server");
        let result = bridge
            .run_initialization_owner(crate::initialization::RecoveryReason::CursorInvalidated);
        assert_eq!(result.is_ok(), reset);
        let store = bridge.store.lock().unwrap();
        assert_eq!(
            store.cursor().unwrap(),
            if reset { "native-v1:3" } else { "native-v1:20" }
        );
        assert_eq!(store.initialization_writeback_blocked().unwrap(), !reset);
        drop(store);
        let log = peer.finish();
        assert!(log.contains(&"candidate-pull:native-v1:20".into()));
        assert!(!log.iter().any(|item| item.starts_with("ordinary-")));
    }
}

#[test]
fn cancelled_connection_during_directory_receipt_preserves_cache_draft_and_writeback_gate() {
    let dir = tempfile::tempdir().unwrap();
    let store = complete_store(dir.path(), 7, true);
    let before = protected(&store);
    let data = snapshot("server", 8);
    let replies = data.clone();
    let (reached_tx, reached) = mpsc::channel();
    let (release_tx, release) = mpsc::channel();
    let peer = Peer::new(data, move |request| {
        if matches!(request, Command::Initialization(Request::Directory { .. })) {
            reached_tx.send(()).unwrap();
            release.recv_timeout(Duration::from_secs(10)).unwrap();
        }
        standard(&replies, request)
    });
    let bridge = bridge(store, &peer, "server");
    // The authenticated-but-not-yet-seeded window is also closed: at this
    // instant no durable candidate has been created yet.
    assert!(
        bridge
            .store
            .lock()
            .unwrap()
            .initialization_status()
            .unwrap()
            .is_none()
    );
    assert_eq!(bridge.upload_pending(false).unwrap(), 0);
    assert_eq!(bridge.namespace_pending().unwrap(), 0);
    assert!(peer.log.lock().unwrap().is_empty());
    let owner = bridge.clone();
    let work = std::thread::spawn(move || {
        owner.run_initialization_owner(crate::initialization::RecoveryReason::PermissionRecheck)
    });
    reached.recv_timeout(Duration::from_secs(10)).unwrap();
    // Mirrors Pause's current-RPC cancellation. No GUI-only flag can be used
    // to permit writeback while the persisted candidate remains incomplete.
    assert_eq!(bridge.upload_pending(false).unwrap(), 0);
    assert_eq!(bridge.upload_pending(true).unwrap(), 0);
    assert_eq!(bridge.namespace_pending().unwrap(), 0);
    assert_eq!(bridge.rename_pending().unwrap(), 0);
    bridge.disconnect();
    release_tx.send(()).unwrap();
    assert!(work.join().unwrap().is_err());
    let store = bridge.store.lock().unwrap();
    assert_eq!(protected(&store), before);
    assert!(store.initialization_writeback_blocked().unwrap());
    let state = store.initialization_status().unwrap().unwrap();
    assert_ne!(state.state, "ready");
    assert_eq!(
        store.initialization_coverage(&state.run).unwrap()[0].state,
        "loading"
    );
    drop(store);
    let log = peer.finish();
    assert!(!log.contains(&"unexpected-write-or-control".into()));
}

#[test]
fn replaced_rpc_cannot_commit_ordinary_pull_or_finish_new_sessions_catchup() {
    for with_batch in [false, true] {
        let dir = tempfile::tempdir().unwrap();
        let store = complete_store(dir.path(), 7, true);
        let before = protected(&store);
        let (received_tx, received) = mpsc::channel();
        let (release_tx, release) = mpsc::channel();
        let old_peer = Peer::new(snapshot("server", 7), move |request| {
            assert!(matches!(request,Command::Pull{cursor} if cursor=="native-v1:7"));
            received_tx.send(()).unwrap();
            release.recv_timeout(Duration::from_secs(10)).unwrap();
            Answer::OrdinaryBatch(with_batch.then(|| Batch {
                from: "native-v1:7".into(),
                to: "native-v1:8".into(),
                events: vec![drive_core::Event {
                    id: "late-old-session-delete".into(),
                    change: Change::Delete { id: "102".into() },
                }],
            }))
        });
        let bridge = bridge(store, &old_peer, "server");
        let old_rpc = old_peer.rpc.clone();
        let old_owner = bridge.clone();
        let work = std::thread::spawn(move || old_owner.pull());
        received.recv_timeout(Duration::from_secs(10)).unwrap();
        let new_peer = Peer::standard(snapshot("server", 7));
        bridge
            .attach(
                new_peer.rpc.clone(),
                "server",
                &serde_json::to_string(&snapshot("server", 7).scope).unwrap(),
            )
            .unwrap();
        assert!(Arc::ptr_eq(&bridge.connection(), &new_peer.rpc));
        // New attach has its own catchup barrier. An old response, including a
        // stationary None, must not release that barrier or publish old events.
        assert!(bridge.finish_session_catchup(&old_rpc).is_err());
        release_tx.send(()).unwrap();
        assert!(work.join().unwrap().is_err());
        assert!(bridge.session_pending.load(Ordering::Acquire));
        assert_eq!(protected(&bridge.store.lock().unwrap()), before);
        assert!(new_peer.log.lock().unwrap().is_empty());
        assert!(!bridge.pull().unwrap());
        assert!(!bridge.session_pending.load(Ordering::Acquire));
        assert_eq!(protected(&bridge.store.lock().unwrap()), before);
        assert_eq!(old_peer.finish(), vec!["ordinary-pull:native-v1:7"]);
        assert_eq!(new_peer.finish(), vec!["ordinary-pull:native-v1:7"]);
    }
}

#[test]
fn advancing_empty_event_pages_yield_to_directory_work_after_sixteen_pages() {
    let dir = tempfile::tempdir().unwrap();
    let data = snapshot("server", 7);
    let replies = data.clone();
    let mut pages = 0usize;
    let mut directory_seen = false;
    let peer = Peer::new(data, move |request| match request {
        Command::Initialization(Request::Pull { cursor }) if !directory_seen => {
            pages += 1;
            assert!(
                pages <= 16,
                "continuous events starved directory enumeration"
            );
            let current = number(cursor).unwrap();
            Answer::Body(Body::Events(Some(Batch {
                from: cursor.clone(),
                to: format!("native-v1:{}", current + 1),
                events: vec![],
            })))
        }
        Command::Initialization(Request::Directory { .. }) => {
            assert_eq!(pages, 16);
            directory_seen = true;
            standard(&replies, request)
        }
        _ => standard(&replies, request),
    });
    let bridge = bridge(Store::open(dir.path()).unwrap(), &peer, "server");
    assert!(bridge.initial_pull().unwrap());
    assert_eq!(
        bridge.store.lock().unwrap().cursor().unwrap(),
        "native-v1:23"
    );
    let log = peer.finish();
    let first_directory = log
        .iter()
        .position(|item| item.starts_with("directory:"))
        .unwrap();
    assert_eq!(
        log[..first_directory]
            .iter()
            .filter(|item| item.starts_with("candidate-pull:"))
            .count(),
        16
    );
    assert_eq!(log.last().unwrap(), "candidate-pull:native-v1:23");
}

fn send_initialization_body(socket: &mut UnixStream, id: u64, data: &RemoteSnapshot, body: Body) {
    let bytes = serde_json::to_vec(&InitializationReply {
        scope: data.scope.clone(),
        restore: data.restore.clone(),
        body,
    })
    .unwrap();
    let mut file = tempfile::tempfile().unwrap();
    file.write_all(&bytes).unwrap();
    file.seek(SeekFrom::Start(0)).unwrap();
    let fd = OwnedFd::from(file);
    transport::send(
        socket,
        &Envelope {
            version: VERSION,
            id,
            body: Response::Initialization {
                size: bytes.len() as u64,
                sha256: drive_core::sha256(&bytes),
            },
        },
        Some(&fd),
    )
    .unwrap();
}

#[test]
fn four_directory_requests_are_in_flight_before_any_child_reply_and_fence_is_separate() {
    let dir = tempfile::tempdir().unwrap();
    let mut data = snapshot("server", 7);
    let template = data
        .nodes
        .iter()
        .find(|node| node.id == "200")
        .unwrap()
        .clone();
    data.nodes.truncate(1);
    for offset in 0..4 {
        let mut child = template.clone();
        child.id = (200 + offset).to_string();
        child.name = format!("D{offset}");
        let mut version: serde_json::Value = serde_json::from_str(&child.version).unwrap();
        version["file"] = child.id.clone().into();
        child.version = version.to_string();
        data.nodes.push(child);
    }
    let (socket, mut remote) = UnixStream::pair().unwrap();
    let rpc = Arc::new(Rpc::new(socket).unwrap());
    let log = Arc::new(Mutex::new(Vec::new()));
    let requests = log.clone();
    let thread = std::thread::spawn(move || {
        let mut pending: Vec<(u64, String)> = Vec::new();
        let mut replied_children = false;
        let mut fenced = false;
        while let Ok((request, _)) = transport::receive::<Envelope<Command>>(&mut remote) {
            requests.lock().unwrap().push(label(&request.body));
            match request.body {
                Command::Initialization(Request::Seed) => send_initialization_body(
                    &mut remote,
                    request.id,
                    &data,
                    Body::Seed {
                        cursor: data.cursor.clone(),
                        root: data.nodes[0].clone(),
                    },
                ),
                Command::Initialization(Request::Pull { cursor }) => {
                    assert_eq!(cursor, data.cursor);
                    assert!(
                        pending.is_empty(),
                        "fence overlapped unfinished directory receipts"
                    );
                    if replied_children {
                        fenced = true;
                    }
                    send_initialization_body(&mut remote, request.id, &data, Body::Events(None));
                }
                Command::Initialization(Request::Directory { node, path }) if node.id == "100" => {
                    assert_eq!(path, "/Lab");
                    send_initialization_body(
                        &mut remote,
                        request.id,
                        &data,
                        Body::Directory {
                            identity: node.id,
                            children: data.nodes[1..].to_vec(),
                            denied: false,
                        },
                    );
                }
                Command::Initialization(Request::Directory { node, path }) => {
                    assert!(!replied_children);
                    assert_eq!(path, format!("/Lab/{}", node.name));
                    assert!(data.nodes[1..].iter().any(|child| child.id == node.id));
                    assert!(!pending.iter().any(|(_, id)| id == &node.id));
                    pending.push((request.id, node.id));
                    assert!(pending.len() <= 4);
                    // A serial owner stalls here and fails within five seconds,
                    // rather than hanging for the production RPC timeout.
                    remote
                        .set_read_timeout(Some(Duration::from_secs(5)))
                        .unwrap();
                    if pending.len() == 4 {
                        remote.set_read_timeout(None).unwrap();
                        // Reverse reply order also exercises request-ID routing.
                        for (id, identity) in pending.drain(..).rev() {
                            send_initialization_body(
                                &mut remote,
                                id,
                                &data,
                                Body::Directory {
                                    identity,
                                    children: vec![],
                                    denied: false,
                                },
                            );
                        }
                        replied_children = true;
                    }
                }
                _ => panic!("unexpected command in four-way directory fixture"),
            }
        }
        assert!(replied_children && fenced && pending.is_empty());
    });
    let peer = Peer {
        rpc,
        log,
        thread: Some(thread),
    };
    let bridge = bridge(Store::open(dir.path()).unwrap(), &peer, "server");
    assert!(bridge.initial_pull().unwrap());
    let store = bridge.store.lock().unwrap();
    assert_eq!(
        store.initialization_status().unwrap().unwrap().state,
        "ready"
    );
    for id in ["200", "201", "202", "203"] {
        assert!(store.node(id).unwrap().directory);
    }
    drop(store);
    let log = peer.finish();
    assert_eq!(
        log.iter()
            .filter(|item| item.starts_with("directory:"))
            .count(),
        5
    );
    assert_eq!(log.last().unwrap(), "candidate-pull:native-v1:7");
}
