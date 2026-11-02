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

// Explicit synthetic envelope for RPC fixtures; never infer production hints here.
fn fixture_page(data: &RemoteSnapshot, batch: Batch) -> InitializationEventPage {
    use drive_core::initialization_events::{InitializationEventHint, InitializationEventKind};
    let hints = batch
        .events
        .iter()
        .enumerate()
        .map(|(ordinal, event)| {
            let (id, name, parent, removed) = match &event.change {
                drive_core::Change::Upsert(node) => {
                    (&node.id, node.name.clone(), node.parent.clone(), false)
                }
                drive_core::Change::Delete { id } => (id, "removed-fixture".into(), None, true),
            };
            let mut names = vec![name];
            let mut next = parent.clone();
            while let Some(id) = next {
                if id == data.scope.file_id {
                    break;
                }
                if let Some(node) = data.nodes.iter().find(|node| node.id == id) {
                    names.push(node.name.clone());
                    next = node.parent.clone();
                } else {
                    names.push(format!("missing-parent-{id}"));
                    break;
                }
            }
            names.reverse();
            let path = if id == &data.scope.file_id {
                data.scope.path.clone()
            } else {
                format!("{}/{}", data.scope.path, names.join("/"))
            };
            InitializationEventHint {
                ordinal: ordinal as u64,
                event_id: event.id.clone(),
                event_sync_id: number(&batch.to).unwrap(),
                file_id: id.clone(),
                kind: if removed {
                    InitializationEventKind::Remove
                } else {
                    InitializationEventKind::Modify
                },
                event_path: path,
                destination_path: None,
                metadata_parent_id: if removed { None } else { parent },
                destination_parent_id: None,
                metadata_removed: removed,
            }
        })
        .collect();
    let page = InitializationEventPage {
        batch,
        scope: serde_json::to_value(&data.scope).unwrap(),
        restore: data.restore.clone(),
        hints,
    };
    page.validate().unwrap();
    page
}

// The synthetic native list has numeric IDs in a distinct fixture namespace;
// no production code derives a native node ID from a file ID.
fn fixture_children(
    data: &RemoteSnapshot,
    parent: &str,
) -> Vec<drive_core::initialization_events::InitializationDirectoryEntry> {
    data.nodes
        .iter()
        .enumerate()
        .filter(|(_, node)| node.parent.as_deref() == Some(parent))
        .map(
            |(index, node)| drive_core::initialization_events::InitializationDirectoryEntry {
                node: node.clone(),
                native_node_id: 10_000 + index as u64,
            },
        )
        .collect()
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
                children: fixture_children(&data, &node.id),
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

// A read loop independent of response delivery: one response can stay held while
// siblings complete over the same real framed Unix socket. This deliberately
// avoids the serial Peer::new callback hiding a batch-join regression.
struct WindowPeer {
    peer: Peer,
    release: Arc<(Mutex<bool>, std::sync::Condvar)>,
    requests: mpsc::Receiver<String>,
    peak: Arc<std::sync::atomic::AtomicUsize>,
}
impl WindowPeer {
    fn release(&self) {
        *self.release.0.lock().unwrap() = true;
        self.release.1.notify_all();
    }
    fn new(data: RemoteSnapshot, hold: bool) -> Self {
        let (socket, mut remote) = UnixStream::pair().unwrap();
        let rpc = Arc::new(Rpc::new(socket).unwrap());
        let writer = Arc::new(Mutex::new(remote.try_clone().unwrap()));
        let log = Arc::new(Mutex::new(Vec::new()));
        let requests_log = log.clone();
        let release = Arc::new((Mutex::new(!hold), std::sync::Condvar::new()));
        let gate = release.clone();
        let (request_tx, requests) = mpsc::channel();
        let active = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let peak = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let peak_thread = peak.clone();
        let thread = std::thread::spawn(move || {
            std::thread::scope(|threads| {
                while let Ok((request, _)) = transport::receive::<Envelope<Command>>(&mut remote) {
                    if matches!(request.body, Command::Cancel { .. }) { continue; }
                    let Command::Initialization(Request::Directory { node, path }) = request.body else {
                        panic!("window fixture received non-directory command");
                    };
                    let mut names=vec![node.name.clone()];
                    let mut parent=node.parent.clone().unwrap();
                    while parent!=data.scope.file_id {
                        let ancestor=data.nodes.iter().find(|n|n.id==parent).unwrap();
                        names.push(ancestor.name.clone());
                        parent=ancestor.parent.clone().unwrap();
                    }
                    names.reverse();
                    assert_eq!(path, format!("/Lab/{}", names.join("/")));
                    requests_log.lock().unwrap().push(format!("directory:{}", node.id));
                    let outstanding = active.fetch_add(1, Ordering::SeqCst) + 1;
                    peak_thread.fetch_max(outstanding, Ordering::SeqCst);
                    request_tx.send(node.id.clone()).unwrap();
                    let writer = writer.clone();
                    let gate = gate.clone();
                    let active = active.clone();
                    let reply = InitializationReply {
                        scope: data.scope.clone(), restore: data.restore.clone(),
                        body: Body::Directory { identity: node.id.clone(), denied:false, children:fixture_children(&data,&node.id) },
                    };
                    threads.spawn(move || {
                        if node.id == "200" {
                            let open = gate.0.lock().unwrap();
                            let (open, timeout) = gate.1.wait_timeout_while(open, Duration::from_secs(15), |open| !*open).unwrap();
                            assert!(*open && !timeout.timed_out(), "test failed to release held reply");
                        }
                        let bytes = serde_json::to_vec(&reply).unwrap();
                        let mut file = tempfile::tempfile().unwrap();
                        file.write_all(&bytes).unwrap();
                        file.seek(SeekFrom::Start(0)).unwrap();
                        let descriptor = OwnedFd::from(file);
                        let mut writer = writer.lock().unwrap();
                        active.fetch_sub(1, Ordering::SeqCst);
                        // A timed-out request's late reply is expected and must
                        // not be confused with an owner receipt/checkpoint.
                        let _ = transport::send(&mut writer, &Envelope {
                            version:VERSION, id:request.id,
                            body:Response::Initialization { size:bytes.len() as u64, sha256:drive_core::sha256(&bytes) },
                        }, Some(&descriptor));
                    });
                }
            });
        });
        Self { peer:Peer { rpc,log,thread:Some(thread) }, release,requests,peak }
    }
}
impl Drop for WindowPeer {
    fn drop(&mut self) { self.release(); }
}

fn window_candidate(path: &std::path::Path, siblings: usize) -> (Store, RemoteSnapshot, InitializationRun, serde_json::Value) {
    let mut store = complete_store(path, 7, true);
    let protected_before = protected(&store);
    let mut data = snapshot("server", 8);
    let prototype = data.nodes[3].clone();
    data.nodes.truncate(3);
    for index in 0..siblings {
        let mut node = prototype.clone();
        node.id = (200 + index).to_string();
        node.name = format!("Directory{index:02}");
        let mut version: serde_json::Value = serde_json::from_str(&node.version).unwrap();
        version["file"] = node.id.clone().into();
        node.version = version.to_string();
        data.nodes.push(node);
    }
    let binding = InitializationBinding {
        task_id:TASK.into(), scope:serde_json::to_value(&data.scope).unwrap(), restore:data.restore.clone(),
        start_cursor:data.cursor.clone(), expected_cursor:"native-v1:7".into(),
    };
    let run = store.begin_initialization(&binding,&data.nodes[0],InitializationBudget::default()).unwrap();
    let ticket = store.begin_initialization_directory(&run,"100").unwrap();
    store.store_initialization_native_directory(&ticket,1,&fixture_children(&data,"100")).unwrap();
    (store,data,run,protected_before)
}

#[test]
fn directory_window_commits_and_refills_while_one_independent_response_is_held() {
    let dir = tempfile::tempdir().unwrap();
    let (store,data,run,before) = window_candidate(dir.path(),24);
    let peer = WindowPeer::new(data,true);
    let bridge = bridge(store,&peer.peer,"server");
    let owner = bridge.clone();
    let run_for_owner = run.clone();
    let work = std::thread::spawn(move || {
        let mut completed = 0;
        let result = owner.initialization_directory_window(&owner.connection(),&owner.session().unwrap(),
            &run_for_owner,Instant::now()+Duration::from_secs(10),false,1,&mut completed);
        (result,completed)
    });
    let mut requests = Vec::new();
    while requests.len() < 5 {
        match peer.requests.recv_timeout(Duration::from_secs(3)) {
            Ok(id) => requests.push(id), Err(_) => break,
        }
    }
    let coverage_before_release = bridge.store.lock().unwrap().initialization_coverage(&run).unwrap();
    let protected_before_release = protected(&bridge.store.lock().unwrap());
    peer.release();
    let (result,completed) = work.join().unwrap();
    assert_eq!(requests.len(),5,"a held first response blocked completed siblings and refill");
    assert!(coverage_before_release.iter().any(|c| c.directory!="100" && c.directory!="200" && c.state=="received"));
    assert_eq!(protected_before_release,before);
    assert!(result.is_ok(),"{result:?}");
    assert_eq!(completed,16,"one event window must not admit more than sixteen directories");
    assert_eq!(peer.peer.log.lock().unwrap().len(),16);
    assert!(peer.peak.load(Ordering::SeqCst)<=4,"network concurrency exceeded existing bound");
    let store = bridge.store.lock().unwrap();
    assert_eq!(protected(&store),before);
    assert!(store.initialization_writeback_blocked().unwrap());
    assert_ne!(store.initialization_status().unwrap().unwrap().state,"ready");
}

#[test]
fn directory_window_refills_past_eighty_descendants_of_a_held_ancestor() {
    let dir=tempfile::tempdir().unwrap();
    let (mut store,mut data,run,before)=window_candidate(dir.path(),2);
    let independent=data.nodes.iter_mut().find(|n|n.id=="201").unwrap();
    independent.id="900".into();
    let mut version:serde_json::Value=serde_json::from_str(&independent.version).unwrap();
    version["file"]="900".into(); independent.version=version.to_string();
    let template=data.nodes[3].clone();
    for index in 0..80 {
        let mut child=template.clone();
        child.id=(3000+index).to_string(); child.parent=Some("200".into()); child.name=format!("Child{index:03}");
        let mut version:serde_json::Value=serde_json::from_str(&child.version).unwrap();
        version["file"]=child.id.clone().into(); child.version=version.to_string();
        data.nodes.push(child);
    }
    for id in ["100","200"] {
        let ticket=store.begin_initialization_directory(&run,id).unwrap();
        store.store_initialization_native_directory(&ticket,1,&fixture_children(&data,id)).unwrap();
    }
    // Model a crashed re-list: reopening makes the ancestor pending without
    // deleting its already discovered (also pending) descendants.
    store.begin_initialization_directory(&run,"200").unwrap();
    drop(store);
    let store=Store::open(dir.path()).unwrap();
    let run=store.initialization_status().unwrap().unwrap().run;
    let peer=WindowPeer::new(data,true);
    let bridge=bridge(store,&peer.peer,"server");
    let owner=bridge.clone();
    let run_for_owner=run.clone();
    let work=std::thread::spawn(move || {
        let mut completed=0;
        let result=owner.initialization_directory_window(&owner.connection(),&owner.session().unwrap(),
            &run_for_owner,Instant::now()+Duration::from_secs(10),false,1,&mut completed);
        (result,completed)
    });
    let mut saw_independent=false;
    for _ in 0..2 {
        match peer.requests.recv_timeout(Duration::from_secs(3)) {
            Ok(id) => saw_independent |= id=="900", Err(_) => break,
        }
    }
    // Release even on the failing implementation so its threads terminate and
    // the assertion reports the scheduling regression rather than hanging.
    peer.release();
    let (result,completed)=work.join().unwrap();
    assert!(saw_independent,"a 64-key page/coalescing prefix left independent work behind a held ancestor");
    assert!(result.is_ok(),"{result:?}");
    assert_eq!(completed,16);
    assert!(peer.peak.load(Ordering::SeqCst)<=4);
    let store=bridge.store.lock().unwrap();
    assert_eq!(protected(&store),before);
    assert!(store.initialization_writeback_blocked().unwrap());
}

#[test]
fn directory_window_timeout_keeps_other_receipts_and_reopen_reuses_them() {
    let dir = tempfile::tempdir().unwrap();
    let (store,data,run,before) = window_candidate(dir.path(),8);
    let peer = WindowPeer::new(data.clone(),true);
    let bridge = bridge(store,&peer.peer,"server");
    let mut completed = 0;
    let result = bridge.initialization_directory_window(&peer.peer.rpc,&bridge.session().unwrap(),
        &run,Instant::now()+Duration::from_secs(5),false,1,&mut completed);
    assert!(result.is_err(),"held request must actually time out");
    assert_eq!(result.unwrap_err().kind(),io::ErrorKind::TimedOut);
    let received = {
        let store = bridge.store.lock().unwrap();
        assert_eq!(protected(&store),before);
        assert!(store.initialization_writeback_blocked().unwrap());
        store.initialization_coverage(&run).unwrap().into_iter()
            .filter(|c|c.directory!="100" && c.state=="received").map(|c|c.directory).collect::<Vec<_>>()
    };
    peer.release();
    assert_eq!(received.len(),7,"one timeout discarded other successful directory receipts");
    assert_eq!(completed,7);
    drop(bridge);
    drop(peer);
    let mut reopened = Store::open(dir.path()).unwrap();
    let recovered = reopened.initialization_status().unwrap().unwrap();
    let next_run = reopened.resume_initialization(&recovered.run,&recovered.binding).unwrap();
    assert!(next_run.epoch>run.epoch);
    let peer = WindowPeer::new(data,false);
    let bridge = self::bridge(reopened,&peer.peer,"server");
    let mut completed = 0;
    assert!(!bridge.initialization_directory_window(&peer.peer.rpc,&bridge.session().unwrap(),
        &next_run,Instant::now()+Duration::from_secs(5),false,1,&mut completed).unwrap());
    assert_eq!(completed,1);
    assert_eq!(*peer.peer.log.lock().unwrap(),vec!["directory:200"]);
    let store = bridge.store.lock().unwrap();
    assert_eq!(protected(&store),before);
    assert!(store.initialization_writeback_blocked().unwrap());
    assert_ne!(store.initialization_status().unwrap().unwrap().state,"ready");
}

#[test]
fn delayed_directory_race_cannot_reticket_old_path_after_sibling_moves_identity() {
    let dir = tempfile::tempdir().unwrap();
    let mut store = complete_store(dir.path(), 7, true);
    let before = protected(&store);
    let mut data = snapshot("server", 8);
    let prototype = data.nodes[3].clone();
    data.nodes.truncate(3);
    for (id,parent,name) in [("200","100","A"),("300","100","B"),("400","300","D")] {
        let mut node = prototype.clone();
        node.id=id.into(); node.parent=Some(parent.into()); node.name=name.into();
        let mut version: serde_json::Value=serde_json::from_str(&node.version).unwrap();
        version["file"]=id.into(); node.version=version.to_string();
        data.nodes.push(node);
    }
    let binding=InitializationBinding{task_id:TASK.into(),scope:serde_json::to_value(&data.scope).unwrap(),
        restore:data.restore.clone(),start_cursor:data.cursor.clone(),expected_cursor:"native-v1:7".into()};
    let run=store.begin_initialization(&binding,&data.nodes[0],InitializationBudget::default()).unwrap();
    for id in ["100","300"] {
        let ticket=store.begin_initialization_directory(&run,id).unwrap();
        store.store_initialization_native_directory(&ticket,1,&fixture_children(&data,id)).unwrap();
    }
    // Remote D has moved into A. The first D request still carries /Lab/B/D.
    data.nodes.iter_mut().find(|n|n.id=="400").unwrap().parent=Some("200".into());
    let (socket,mut remote)=UnixStream::pair().unwrap();
    let rpc=Arc::new(Rpc::new(socket).unwrap());
    let log=Arc::new(Mutex::new(Vec::new()));
    let requests=log.clone();
    let thread=std::thread::spawn(move || {
        let send=|remote:&mut UnixStream,id:u64,body:Option<Body>| {
            let (response,descriptor)=if let Some(body)=body {
                let bytes=serde_json::to_vec(&InitializationReply{scope:data.scope.clone(),restore:data.restore.clone(),body}).unwrap();
                let mut file=tempfile::tempfile().unwrap(); file.write_all(&bytes).unwrap(); file.seek(SeekFrom::Start(0)).unwrap();
                (Response::Initialization{size:bytes.len() as u64,sha256:drive_core::sha256(&bytes)},Some(OwnedFd::from(file)))
            } else { (Response::Failed{category:"native_snapshot_changed".into()},None) };
            transport::send(remote,&Envelope{version:VERSION,id,body:response},descriptor.as_ref()).unwrap();
        };
        let mut parent_request=None;
        let mut old_request=None;
        let mut parent_sent=false;
        while let Ok((request,_))=transport::receive::<Envelope<Command>>(&mut remote) {
            if matches!(request.body,Command::Cancel{..}) { continue; }
            let Command::Initialization(Request::Directory{node,path})=request.body else { panic!("unexpected window command"); };
            requests.lock().unwrap().push(path.clone());
            assert!(requests.lock().unwrap().len()<16,"stale-path retry loop");
            match (node.id.as_str(),path.as_str()) {
                ("200","/Lab/A") => parent_request=Some(request.id),
                ("400","/Lab/B/D") if old_request.is_none() => old_request=Some(request.id),
                ("300","/Lab/B") => send(&mut remote,request.id,Some(Body::Directory{identity:"300".into(),denied:false,children:vec![]})),
                ("400","/Lab/A/D") => {
                    // This request can only be scheduled after A's move receipt
                    // committed. Now return the old request's typed race.
                    send(&mut remote,old_request.unwrap(),None);
                    send(&mut remote,request.id,Some(Body::Directory{identity:"400".into(),denied:false,children:vec![]}));
                }
                ("400","/Lab/B/D") => {
                    // Make the unsafe implementation terminate normally; the
                    // assertion below must detect this forbidden second query.
                    send(&mut remote,request.id,Some(Body::Directory{identity:"400".into(),denied:false,children:vec![]}));
                }
                _ => panic!("unexpected candidate path"),
            }
            if !parent_sent && parent_request.is_some() && old_request.is_some() {
                send(&mut remote,parent_request.unwrap(),Some(Body::Directory{
                    identity:"200".into(),denied:false,children:fixture_children(&data,"200"),
                }));
                parent_sent=true;
            }
        }
    });
    let peer=Peer{rpc,log,thread:Some(thread)};
    let bridge=bridge(store,&peer,"server");
    let mut completed=0;
    let result=bridge.initialization_directory_window(&peer.rpc,&bridge.session().unwrap(),&run,
        Instant::now()+Duration::from_secs(5),false,1,&mut completed);
    let requests=peer.log.lock().unwrap().clone();
    assert!(result.is_ok(),"{result:?}; requests={requests:?}");
    assert_eq!(requests.iter().filter(|p|p.as_str()=="/Lab/B/D").count(),1,
        "an invalidated old identity/path was granted a new generation");
    assert!(requests.contains(&"/Lab/A/D".into()));
    let store=bridge.store.lock().unwrap();
    assert_eq!(store.initialization_node(&run,"400").unwrap().parent.as_deref(),Some("200"));
    assert_eq!(protected(&store),before);
    assert!(store.initialization_writeback_blocked().unwrap());
    assert_eq!(store.initialization_coverage(&run).unwrap().iter().find(|c|c.directory=="400").unwrap().state,"received");
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
            Answer::Body(Body::Events(Some(fixture_page(
                &replies,
                Batch {
                    from: "native-v1:7".into(),
                    to: "native-v1:8".into(),
                    events: vec![drive_core::Event {
                        id: "missing-parent".into(),
                        change: drive_core::Change::Upsert(older),
                    }],
                },
            ))))
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
                children: fixture_children(&replies, &node.id),
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
            Answer::Body(Body::Events(Some(fixture_page(
                &replies,
                Batch {
                    from: "native-v1:7".into(),
                    to: "native-v1:8".into(),
                    events: vec![drive_core::Event {
                        id: "unresolved".into(),
                        change: drive_core::Change::Upsert(unknown),
                    }],
                },
            ))))
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
            Answer::Body(Body::Events(Some(fixture_page(
                &replies,
                Batch {
                    from: cursor.clone(),
                    to: "native-v1:8".into(),
                    events: vec![drive_core::Event {
                        id: "unknown-parent-page".into(),
                        change: drive_core::Change::Upsert(replies.nodes[4].clone()),
                    }],
                },
            ))))
        }
        Command::Initialization(Request::Pull { cursor }) if cursor == "native-v1:8" => {
            Answer::Body(Body::Events(Some(fixture_page(
                &replies,
                Batch {
                    from: cursor.clone(),
                    to: "native-v1:9".into(),
                    events: vec![drive_core::Event {
                        id: "directory-moved-next-page".into(),
                        change: drive_core::Change::Upsert(replies.nodes[3].clone()),
                    }],
                },
            ))))
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
                children: fixture_children(&replies, &node.id),
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
            Answer::Body(Body::Events(Some(fixture_page(
                &replies,
                Batch {
                    from: cursor.clone(),
                    to: format!("native-v1:{}", n + 1),
                    events: vec![drive_core::Event {
                        id: format!("missing-{n}"),
                        change: drive_core::Change::Upsert(file),
                    }],
                },
            ))))
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
            Answer::Body(Body::Events(Some(fixture_page(
                &replies,
                Batch {
                    from: cursor.clone(),
                    to: format!("native-v1:{}", current + 1),
                    events: vec![],
                },
            ))))
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
                            children: fixture_children(&data, &data.scope.file_id),
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

#[test]
fn mismatched_native_hint_page_header_is_rejected_before_persistent_receipt() {
    for wrong_restore in [false, true] {
        let dir = tempfile::tempdir().unwrap();
        let data = snapshot("server", 7);
        let replies = data.clone();
        let peer = Peer::new(data, move |request| match request {
            Command::Initialization(Request::Pull { cursor }) => {
                let mut page = fixture_page(
                    &replies,
                    Batch {
                        from: cursor.clone(),
                        to: "native-v1:8".into(),
                        events: vec![],
                    },
                );
                if wrong_restore {
                    page.restore = "another-server".into();
                } else {
                    page.scope["view_id"] = 2.into();
                }
                Answer::Body(Body::Events(Some(page)))
            }
            _ => standard(&replies, request),
        });
        let bridge = bridge(Store::open(dir.path()).unwrap(), &peer, "server");
        assert!(bridge.initial_pull().is_err());
        let store = bridge.store.lock().unwrap();
        assert_eq!(store.cursor().unwrap(), "");
        let state = store.initialization_status().unwrap().unwrap();
        assert_eq!(state.received_sequence, 0);
        assert!(store.initialization_writeback_blocked().unwrap());
        drop(store);
        peer.finish();
    }
}

// An authenticated root rejection is different from loss of transport. The
// former revokes candidate metadata; the latter retains confirmed offline lists.
#[test]
fn cold_preview_root_rejection_revokes_but_transport_and_generic_failure_preserve_lists() {
    for (category, restore, revoked) in [
        ("native_root_unavailable", "server", true),
        ("native_root_unavailable", "new-server", true),
        ("native_response_timeout", "server", false),
        ("synthetic_bad_payload", "server", false),
    ] {
        let dir = tempfile::tempdir().unwrap();
        let mut store = Store::open(dir.path()).unwrap();
        let data = snapshot("server", 7);
        store
            .bind_native_scope("server", &serde_json::to_string(&data.scope).unwrap())
            .unwrap();
        let binding = InitializationBinding {
            task_id: TASK.into(),
            scope: serde_json::to_value(&data.scope).unwrap(),
            restore: "server".into(),
            start_cursor: data.cursor.clone(),
            expected_cursor: "".into(),
        };
        let run = store
            .begin_initialization(&binding, &data.nodes[0], InitializationBudget::default())
            .unwrap();
        store.preview_mount_root("100").unwrap().unwrap();
        let ticket = store.begin_initialization_directory(&run, "100").unwrap();
        store
            .store_initialization_native_directory(&ticket, 1, &fixture_children(&data, "100"))
            .unwrap();
        assert!(store.preview_directory_snapshot(1, 1024 * 1024).is_ok());
        let peer = Peer::new(snapshot(restore, 7), move |request| {
            assert!(matches!(
                request,
                Command::Initialization(Request::Root | Request::Seed)
            ));
            Answer::Fail(category)
        });
        let bridge = bridge(store, &peer, restore);
        assert!(bridge.initial_pull().is_err());
        {
            let store = bridge.store.lock().unwrap();
            let result = store.preview_directory_snapshot(1, 1024 * 1024);
            if revoked {
                assert!(
                    matches!(result,Err(drive_core::Error::Io(ref e)) if e.raw_os_error()==Some(libc::EACCES)),
                    "root rejection must revoke preview: {result:?}"
                );
            } else {
                assert!(
                    result.is_ok(),
                    "transport/non-root failure must retain confirmed lists: {result:?}"
                );
            }
            assert_eq!(store.cursor().unwrap(), "");
            assert_eq!(store.pending_operations().unwrap(), 0);
            assert!(store.initialization_writeback_blocked().unwrap());
        }
        let log = peer.finish();
        assert_eq!(log.len(), 1);
    }
}
