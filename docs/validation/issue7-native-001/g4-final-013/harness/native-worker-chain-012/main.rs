//! Synthetic native TLS peer -> unmodified worker ELF -> Rpc -> public Bridge.
//! run.py owns a bounded process group for each invocation of this executable.
mod codec;
use drive_core::{Node, SharedStore, Store, write_permissions::{self as permissions, Action, Decision, LocalPolicy}};
use drive_runtime::{bridge::{Bridge, WorkerProcess}, wire::{Command, Login, Response, VERSION}};
use rustls::pki_types::{CertificateDer, PrivateKeyDer, pem::PemObject};
use serde_json::{Value, json};
use std::{
    fs, io, net::{Shutdown, TcpListener, TcpStream}, path::{Path, PathBuf},
    sync::{Arc, Mutex, Condvar, atomic::{AtomicBool, AtomicUsize, Ordering}},
    thread::{self, JoinHandle}, time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
type HResult<T> = Result<T, String>;
fn require(condition: bool, message: &str) -> HResult<()> { if condition { Ok(()) } else { Err(message.into()) } }
fn checked<T, E: std::fmt::Display>(value: Result<T, E>) -> HResult<T> { value.map_err(|e| e.to_string()) }
const ROOT: &str = "9";
const FILE: &str = "42";
const SIBLING: &str = "43";
const RESTORE: &str = "synthetic-chain-restore";
const USER: &str = "synthetic-chain-user";
const PASSWORD: &str = "synthetic-chain-password";
const DEVICE: &str = "synthetic-chain-device";
const DIGEST: &str = "synthetic-next-digest";

#[derive(Clone, Copy, Debug)]
enum Case { Denied(u32), Profile { changed: bool, other: bool }, LateRead, GenericRpc(u32) }
impl Case {
    fn parse(name: &str) -> HResult<Self> {
        Ok(match name {
            "bridge_download_denied_3002" => Self::Denied(0x3002),
            "bridge_download_denied_3008" => Self::Denied(0x3008),
            "bridge_profile_false_empty" => Self::Profile { changed: false, other: false },
            "bridge_profile_true_empty" => Self::Profile { changed: true, other: false },
            "bridge_profile_true_other_view" => Self::Profile { changed: true, other: true },
            "bridge_late_read_after_profile" => Self::LateRead,
            "worker_rpc_generic_3004" => Self::GenericRpc(0x3004),
            "worker_rpc_generic_8004" => Self::GenericRpc(0x8004),
            "worker_rpc_generic_9005" => Self::GenericRpc(0x9005),
            "worker_rpc_generic_a005" => Self::GenericRpc(0xa005),
            _ => return Err("unknown case name".into()),
        })
    }
    fn boundary(self) -> &'static str {
        match self { Self::GenericRpc(_) => "native TLS -> real worker ELF -> IPC/Rpc (direct Download; no Bridge download application)",
            _ => "native TLS -> real worker ELF -> IPC/Rpc -> public Bridge -> Store" }
    }
}
#[derive(Default)]
struct Signals { download_received: bool, second_wait: bool, release_download: bool }
struct Shared {
    case: Case, stop: AtomicBool, connections: AtomicUsize, pulls: AtomicUsize,
    polls: AtomicUsize, downloads: AtomicUsize, sockets: Mutex<Vec<TcpStream>>,
    ledger: Mutex<Vec<Value>>, errors: Mutex<Vec<String>>, signals: Mutex<Signals>, changed: Condvar,
}
impl Shared {
    fn log(&self, value: Value) { self.ledger.lock().unwrap().push(value); }
    fn wait_for(&self, predicate: impl Fn(&Signals) -> bool, label: &str) -> HResult<()> {
        let deadline = Instant::now() + Duration::from_secs(10);
        let mut state = self.signals.lock().unwrap();
        while !predicate(&state) && !self.stop.load(Ordering::SeqCst) {
            let remaining = deadline.saturating_duration_since(Instant::now());
            require(!remaining.is_zero(), label)?;
            state = self.changed.wait_timeout(state, remaining.min(Duration::from_millis(100))).unwrap().0;
        }
        require(predicate(&state), label)
    }
    fn signal(&self, change: impl FnOnce(&mut Signals)) {
        change(&mut self.signals.lock().unwrap()); self.changed.notify_all();
    }
}
struct Server { shared: Arc<Shared>, accept: Option<JoinHandle<()>>, handlers: Arc<Mutex<Vec<JoinHandle<()>>>> }
impl Server {
    fn new(listener: TcpListener, case: Case, certificates: &Path) -> HResult<Self> {
        let certificate = checked(fs::read(certificates.join("server.pem")))?;
        let key = checked(fs::read(certificates.join("server-key.pem")))?;
        let config = checked(checked(rustls::ServerConfig::builder_with_provider(Arc::new(rustls::crypto::ring::default_provider()))
            .with_safe_default_protocol_versions())?.with_no_client_auth()
            .with_single_cert(vec![checked(CertificateDer::from_pem_slice(&certificate))?], checked(PrivateKeyDer::from_pem_slice(&key))?))?;
        let config = Arc::new(config);
        checked(listener.set_nonblocking(true))?;
        let shared = Arc::new(Shared { case, stop: AtomicBool::new(false), connections: AtomicUsize::new(0),
            pulls: AtomicUsize::new(0), polls: AtomicUsize::new(0), downloads: AtomicUsize::new(0),
            sockets: Mutex::new(Vec::new()), ledger: Mutex::new(Vec::new()), errors: Mutex::new(Vec::new()),
            signals: Mutex::new(Signals::default()), changed: Condvar::new() });
        let handlers = Arc::new(Mutex::new(Vec::new()));
        let state = shared.clone(); let handles = handlers.clone();
        let accept = thread::spawn(move || {
            while !state.stop.load(Ordering::SeqCst) {
                match listener.accept() {
                    Ok((socket, address)) => {
                        if !address.ip().is_loopback() { state.errors.lock().unwrap().push("non-loopback connection".into()); continue; }
                        let serial = state.connections.fetch_add(1, Ordering::SeqCst) + 1;
                        if serial > 12 { state.errors.lock().unwrap().push("connection budget exceeded".into()); break; }
                        let clone = match socket.try_clone() { Ok(socket) => socket, Err(e) => { state.errors.lock().unwrap().push(e.to_string()); break; } };
                        state.sockets.lock().unwrap().push(clone);
                        let state = state.clone(); let config = config.clone();
                        handles.lock().unwrap().push(thread::spawn(move || {
                            if let Err(error) = connection(socket, serial, config, &state) {
                                if !state.stop.load(Ordering::SeqCst) { state.errors.lock().unwrap().push(error); }
                            }
                        }));
                    }
                    Err(e) if e.kind() == io::ErrorKind::WouldBlock => thread::sleep(Duration::from_millis(2)),
                    Err(e) => { state.errors.lock().unwrap().push(e.to_string()); break; }
                }
            }
        });
        Ok(Self { shared, accept: Some(accept), handlers })
    }
    fn stop(&mut self) -> bool {
        self.shared.stop.store(true, Ordering::SeqCst); self.shared.changed.notify_all();
        if let Some(handle) = self.accept.take() { if handle.join().is_err() { return false; } }
        for socket in self.shared.sockets.lock().unwrap().iter() { let _ = socket.shutdown(Shutdown::Both); }
        let mut joined = true;
        for handle in self.handlers.lock().unwrap().drain(..) { joined &= handle.join().is_ok(); }
        self.shared.sockets.lock().unwrap().clear(); joined
    }
    fn evidence(&self) -> Value {
        json!({"listener":"127.0.0.1:6690", "connections":self.shared.connections.load(Ordering::SeqCst),
            "pull_event_requests":self.shared.pulls.load(Ordering::SeqCst), "long_poll_requests":self.shared.polls.load(Ordering::SeqCst),
            "download_requests":self.shared.downloads.load(Ordering::SeqCst), "download_content_bytes_sent":0,
            "actions":*self.shared.ledger.lock().unwrap(), "errors":*self.shared.errors.lock().unwrap()})
    }
}
impl Drop for Server { fn drop(&mut self) { self.stop(); } }
fn connection(socket: TcpStream, serial: usize, config: Arc<rustls::ServerConfig>, shared: &Shared) -> HResult<()> {
    checked(socket.set_read_timeout(Some(Duration::from_secs(8))))?;
    checked(socket.set_write_timeout(Some(Duration::from_secs(8))))?;
    checked(socket.set_nodelay(true))?;
    let mut socket = socket;
    let (command, message) = codec::request(&mut socket)?;
    require(command == 0x16 && message["action"] == "encrypt_channel", "native TLS upgrade request")?;
    codec::respond(&mut socket, &json!({"error":null}))?;
    let mut stream = rustls::StreamOwned::new(checked(rustls::ServerConnection::new(config))?, socket);
    let mut connected = false; let mut auth_step = 0usize;
    loop {
        if shared.stop.load(Ordering::SeqCst) { return Ok(()); }
        let (command, message) = match codec::request(&mut stream) {
            Ok(message) => message,
            Err(error) => {
                // Error replies poison pooled channels, so normal EOF is expected.
                shared.log(json!({"connection":serial,"event":"connection_end","after_connect":connected,"io_detail":error}));
                let normal_close=["io:UnexpectedEof:","io:ConnectionReset:","io:ConnectionAborted:","io:BrokenPipe:"]
                    .iter().any(|prefix|error.starts_with(prefix));
                return if connected && normal_close { Ok(()) } else { Err(format!("native channel read/decode failed: {error}")) };
            }
        };
        if !connected {
            let response = match (auth_step, command) {
                (0, 0x19) => { auth_step=1; json!({"database_restore_id":RESTORE,"package_version":{"build":17892}}) }
                (1, 0x12) => {
                    require(message["username"] == USER && message["password"] == PASSWORD, "synthetic login fields")?;
                    require(message["agent"]["restore_id"] == RESTORE && message["agent"]["device_uuid"] == DEVICE, "synthetic auth identity")?;
                    auth_step=2; json!({"session":"synthetic-chain-session"})
                }
                (2, 0x19) => { auth_step=3; json!({"database_restore_id":RESTORE,"package_version":{"build":17892}}) }
                (0 | 3, 0x0e) => {
                    require(message["session"] == "synthetic-chain-session" && message["restore_id"] == RESTORE, "pool connection identity")?;
                    connected=true; json!({"alive":150})
                }
                _ => return Err(format!("unexpected handshake step {auth_step}, command {command}")),
            };
            shared.log(json!({"connection":serial,"event":"handshake","scmd":command,"fields_checked":true}));
            codec::respond(&mut stream, &response)?; continue;
        }
        require(command == 1, "action SCMD")?;
        let action = message["action"].as_str().ok_or("missing action")?;
        match action {
            "pull_event" => {
                let count = shared.pulls.fetch_add(1, Ordering::SeqCst) + 1;
                require(count <= 5, "pull request budget")?;
                require(message["view_id"] == 7 && message["node_id"] == 9 && message["path"] == "/fixture", "pull scope")?;
                let cursor = message["sync_id"].as_u64().ok_or("pull cursor")?;
                require(cursor == if count == 1 {77} else {78}, "pull cursor continuity")?;
                let events = if count == 1 { json!([{"path":"/fixture/file","type":"modify","sync_id":78,"file_id":FILE,
                    "file":{"parent_id":ROOT,"file_type":"file","is_removed":0,"max_id":78,"file_size":3,
                        "file_hash":"00000000000000000000000000000000","mtime":1234}}]) } else { json!([]) };
                shared.log(json!({"connection":serial,"action":action,"cursor":cursor,"events":if count==1 {1} else {0}}));
                codec::respond(&mut stream, &json!({"next_sync_id":78,"event_list":events}))?;
            }
            "download" => {
                let count = shared.downloads.fetch_add(1, Ordering::SeqCst) + 1;
                require(count <= if matches!(shared.case,Case::Denied(_)) {3} else {1}, "download attempt budget")?;
                require(message["path"] == "id:42" && message["target_sync_id"] == 78, "identity-addressed download version")?;
                shared.log(json!({"connection":serial,"action":action,"identity_checked":true,"version_checked":true}));
                shared.signal(|s| s.download_received=true);
                if matches!(shared.case, Case::LateRead) { shared.wait_for(|s| s.release_download, "late download release gate")?; }
                let code = match shared.case { Case::Denied(code) | Case::GenericRpc(code) => code, Case::LateRead => 0x3002,
                    _ => return Err("profile-only case unexpectedly requested a body".into()) };
                codec::respond(&mut stream, &json!({"error":{"code":code,"reason":"synthetic refusal"}}))?;
                shared.log(json!({"connection":serial,"action":"download_reply","code":code}));
            }
            "long_poll" => {
                let count = shared.polls.fetch_add(1, Ordering::SeqCst) + 1;
                require(count <= 2, "long-poll request budget")?;
                require(message["subscribe"][0]["view_id"] == 7 && message["subscribe"][0]["node_id"] == 9
                    && message["subscribe"][0]["sync_id"] == 78 && message["subscribe"][0]["path"] == "/fixture", "notification subscription")?;
                require(message["profile_digest"] == if count==1 {""} else {DIGEST}, "notification digest continuity")?;
                shared.log(json!({"connection":serial,"action":action,"ordinal":count,"cursor":78,"digest_checked":true}));
                if count==2 {
                    shared.signal(|s| s.second_wait=true);
                    while !shared.stop.load(Ordering::SeqCst) { thread::sleep(Duration::from_millis(5)); }
                    return Ok(());
                }
                let (changed, other) = match shared.case { Case::Profile{changed,other} => (changed,other), Case::LateRead => (true,false),
                    _ => return Err("download-only case unexpectedly subscribed".into()) };
                codec::respond(&mut stream, &json!({"changes":if other {vec![8u64]} else {vec![]},
                    "profile_changed":u8::from(changed),"profile_digest":DIGEST}))?;
                shared.log(json!({"connection":serial,"action":"profile_reply","profile_changed":changed,"changed_views":if other {vec![8u64]} else {vec![]}}));
            }
            _ => return Err(format!("unexpected action {action}")),
        }
    }
}
fn fixture(directory: &Path) -> HResult<Arc<SharedStore>> {
    let mut store = checked(Store::open(directory))?;
    let root = Node { id:ROOT.into(), parent:None, name:"fixture".into(), version:"synthetic-root".into(), directory:true, metadata:None };
    let sibling = Node { id:SIBLING.into(), parent:Some(ROOT.into()), name:"sibling".into(), version:"synthetic-sibling".into(), directory:false, metadata:None };
    checked(store.install_remote_snapshot(&[root,sibling], "native-v1:77", RESTORE))?;
    checked(store.ensure_fs_root(ROOT))?;
    Ok(Arc::new(SharedStore::new(store)))
}
fn login(certificates: &Path) -> HResult<Login> {
    Ok(Login { host:"127.0.0.1".into(), tls_name:"localhost".into(), trusted_ca_pem:vec![checked(fs::read_to_string(certificates.join("ca.pem")))?],
        username:USER.into(), password:PASSWORD.into(), device_id:DEVICE.into(),
        scope:drive_core::native::Scope { view_id:7, node_id:9, file_id:ROOT.into(), path:"/fixture".into() } })
}
fn permissions_snapshot(bridge: &Bridge) -> HResult<Value> {
    let store = bridge.store.lock().unwrap();
    Ok(json!({"target_read":checked(permissions::decide(&store,ROOT,FILE,Action::Read))?,
        "sibling_read":checked(permissions::decide(&store,ROOT,SIBLING,Action::Read))?,
        "root_list":checked(permissions::decide(&store,ROOT,ROOT,Action::List))?,
        "target_write_content":checked(permissions::decide(&store,ROOT,FILE,Action::WriteContent))?,
        "policy":checked(permissions::policy(&store,ROOT))?,
        "download_failure":checked(store.download_failure(FILE))?, "has_cached_body":store.open_cached(FILE).is_ok(),
        "cursor":checked(store.cursor())?}))
}
struct Threads { event:Option<JoinHandle<io::Result<()>>>, download:Option<JoinHandle<Result<(),i32>>> }
fn exercise(case: Case, bridge: &Arc<Bridge>, worker: &WorkerProcess, server: &Server, threads: &mut Threads, observations: &mut Value) -> HResult<Value> {
    require(checked(bridge.pull())?, "initial real worker Pull must advance the fixture")?;
    let node = checked(bridge.store.lock().unwrap().node(FILE))?;
    require(node.id==FILE && node.metadata.as_ref().is_some_and(|m|m.size==3 && m.mtime==1234), "real native projection metadata")?;
    let version: Value = checked(serde_json::from_str(&node.version))?;
    require(version["restore"]==RESTORE && version["view"]==7 && version["root"]==ROOT && version["file"]==FILE && version["sync"]==78,
        "Node version produced by real native projection")?;
    {
        let store = bridge.store.lock().unwrap();
        checked(permissions::record_observed(&store,ROOT,SIBLING,Action::Read,true))?;
        if matches!(case, Case::Profile{..}|Case::LateRead) {
            checked(permissions::set_local_policy(&store,ROOT,LocalPolicy::ExplicitReadOnly))?;
            checked(permissions::record_observed(&store,ROOT,ROOT,Action::List,false))?;
            if !matches!(case,Case::LateRead) { checked(permissions::record_observed(&store,ROOT,FILE,Action::Read,false))?; }
        }
    }
    let before = permissions_snapshot(bridge)?;
    *observations = json!({"before":before,"node_from_actual_worker_pull":true});
    let details = observations;
    match case {
        Case::Denied(code) => {
            let result = bridge.ensure(&node); details["ensure_errno"] = json!(result.err());
            require(result==Err(libc::EACCES), "Bridge ensure must return EACCES")?;
            let store = bridge.store.lock().unwrap();
            require(checked(permissions::decide(&store,ROOT,FILE,Action::Read))?==Decision::Deny, "target Read not denied")?;
            require(checked(permissions::decide(&store,ROOT,SIBLING,Action::Read))?==Decision::Allow, "sibling Read was polluted")?;
            require(checked(permissions::decide(&store,ROOT,ROOT,Action::Read))?==Decision::Unknown, "parent Read was polluted")?;
            require(checked(permissions::decide(&store,ROOT,FILE,Action::WriteContent))?==Decision::Unknown, "unrelated target action was polluted")?;
            let failure = checked(store.download_failure(FILE))?.ok_or("missing refusal failure record")?;
            require(failure.kind==drive_core::download_jobs::DownloadFailureKind::ServerRejected
                && failure.server_code.is_none() && failure.attempts==1, "typed refusal failure classification")?;
            details["retry_at_recorded_not_assumed_terminal"] = json!(failure.retry_at);
            details["native_refusal_code"] = json!(code);
            drop(store);
            let repeat=bridge.ensure(&node);
            details["repeat_ensure_errno"] = json!(repeat.err());
            let after_repeat=server.shared.downloads.load(Ordering::SeqCst);
            details["download_count_after_repeat"] = json!(after_repeat);
            // Collect the background result even when the immediate repeat is
            // already wrong; neither assertion is relaxed by collecting both.
            let wait_started=Instant::now();
            let wall_now=checked(SystemTime::now().duration_since(UNIX_EPOCH))?.as_secs();
            let due=failure.retry_at.unwrap_or(wall_now.saturating_add(3));
            loop {
                let wall=checked(SystemTime::now().duration_since(UNIX_EPOCH))?.as_secs();
                if wall>=due && wait_started.elapsed()>=Duration::from_secs(3) {break;}
                require(wait_started.elapsed()<Duration::from_secs(8),"could not observe retry deadline within bounded diagnostic wait")?;
                thread::sleep(Duration::from_millis(10));
            }
            details["background_probe_wait_ms"] = json!(wait_started.elapsed().as_millis());
            bridge.refresh_cached();
            let after_background=server.shared.downloads.load(Ordering::SeqCst);
            details["download_count_after_refresh_cached"] = json!(after_background);
            details["after_refresh_cached"] = permissions_snapshot(bridge)?;
            require(repeat==Err(libc::EACCES) && after_repeat==1 && after_background==1,
                &format!("ReadDeny gates: repeated ensure={repeat:?} (expected EACCES), downloads after repeat={after_repeat}, after due refresh_cached={after_background} (both expected 1)"))?;
        }
        Case::GenericRpc(code) => {
            let response = checked(worker.rpc.call(Command::Download { node },Duration::from_secs(12)))?;
            let Response::Failed{category} = response.body else { return Err("generic code became a typed/success response".into()); };
            require(category==format!("native_server_code:{code}"), "generic error category")?;
            require(response.descriptor.is_none(), "generic failure has descriptor")?;
            details["rpc_category"] = json!(category);
            details["bridge_download_application_exercised"] = json!(false);
        }
        Case::Profile{changed,..} => {
            let task_bridge=bridge.clone(); threads.event=Some(thread::spawn(move || task_bridge.event_loop()));
            server.shared.wait_for(|s|s.second_wait,"second subscription was not observed")?;
            let after=permissions_snapshot(bridge)?;
            let expected=if changed {"unknown"} else {"deny"};
            require(after["target_read"]==expected && after["root_list"]==expected,"profile bit did not control observed ACL invalidation")?;
            require(after["sibling_read"]==if changed {"unknown"} else {"allow"},"profile invalidation scope")?;
            require(after["policy"]["write_policy_revision"].as_u64()==before["policy"]["write_policy_revision"].as_u64().map(|r|r+u64::from(changed)),"profile revision delta")?;
            require(after["policy"]["local_policy"]=="explicit_read_only","profile changed explicit local RO")?;
            require(server.shared.downloads.load(Ordering::SeqCst)==0,"profile-only notification hydrated a body")?;
            details["after_notification"] = after;
        }
        Case::LateRead => {
            let task_bridge=bridge.clone(); threads.download=Some(thread::spawn(move ||task_bridge.ensure(&node)));
            server.shared.wait_for(|s|s.download_received,"in-flight download was not observed")?;
            let task_bridge=bridge.clone(); threads.event=Some(thread::spawn(move ||task_bridge.event_loop()));
            server.shared.wait_for(|s|s.second_wait,"profile application/resubscription was not observed")?;
            let applied=permissions_snapshot(bridge)?;
            require(applied["policy"]["write_policy_revision"].as_u64()==before["policy"]["write_policy_revision"].as_u64().map(|r|r+1),"profile did not advance in-flight revision")?;
            require(applied["root_list"]=="unknown","profile had not cleared ACL before refusal")?;
            server.shared.signal(|s|s.release_download=true);
            let result=threads.download.take().unwrap().join().map_err(|_|"download thread panicked")?;
            require(result==Err(libc::EACCES),"late refusal call result")?;
            let after=permissions_snapshot(bridge)?;
            require(after["target_read"]=="unknown" && after["download_failure"].is_null(),"late refusal reintroduced stale ACL or retry gate")?;
            require(server.shared.connections.load(Ordering::SeqCst)>=2,"late case did not use independent native channels")?;
            details["after_notification_before_refusal"]=applied;
            details["late_ensure_errno"]=json!(libc::EACCES);
        }
    }
    let after=permissions_snapshot(bridge)?;
    require(after["has_cached_body"]==false,"refusal/profile case published body")?;
    require(after["cursor"]=="native-v1:78","unexpected event cursor mutation")?;
    if matches!(case,Case::GenericRpc(_)) { require(after==before,"direct worker RPC changed fsd Store")?; }
    if !matches!(case,Case::Profile{..}) { require(server.shared.downloads.load(Ordering::SeqCst)==1,"download count differs from one")?; }
    details["after"]=after; Ok(details.clone())
}
fn run_case(name: &str, executable: &Path, certificates: &Path, directory: &Path) -> Value {
    let started=Instant::now();
    let case=match Case::parse(name) {Ok(case)=>case,Err(error)=>return json!({"name":name,"passed":false,"status":"failed","error":error})};
    let listener=match TcpListener::bind("127.0.0.1:6690") {
        Ok(listener)=>listener,
        Err(error)=>return json!({"name":name,"passed":false,"status":"blocked","ignored":true,"error":format!("exclusive loopback bind failed; no worker started: {error}")}),
    };
    let mut server=match Server::new(listener,case,certificates) {Ok(server)=>server,Err(error)=>return json!({"name":name,"passed":false,"status":"failed","error":error})};
    let mut worker:Option<WorkerProcess>=None;
    let mut bridge:Option<Arc<Bridge>>=None;
    let mut threads=Threads{event:None,download:None};
    let mut observations=json!({});
    let result=std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| -> HResult<Value> {
        require(unsafe{libc::geteuid()}!=0,"run as ordinary isolated validation user")?;
        worker=Some(checked(WorkerProcess::start(executable))?);
        let active=worker.as_ref().unwrap();
        bridge=Some(Arc::new(checked(Bridge::connect(fixture(&directory.join("state"))?,active.rpc.clone(),login(certificates)?))?));
        exercise(case,bridge.as_ref().unwrap(),active,&server,&mut threads,&mut observations)
    })).unwrap_or_else(|_|Err("harness/product panic (see captured stderr)".into()));
    // Cleanup is outside assertion/result handling; the supervisor also owns the
    // process group if a product call or join fails to return by the outer bound.
    server.shared.signal(|s|s.release_download=true);
    if let Some(bridge)=&bridge { bridge.disconnect(); }
    let pid=worker.as_ref().map(WorkerProcess::pid);
    if let Some(worker)=worker.as_mut() { worker.stop(); }
    let event_joined=threads.event.take().is_none_or(|h|h.join().is_ok());
    let download_joined=threads.download.take().is_none_or(|h|h.join().is_ok());
    let server_joined=server.stop();
    let failure_snapshot=if result.is_err() { bridge.as_ref().map(|bridge| {
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(||permissions_snapshot(bridge))) {
            Ok(Ok(value))=>value,
            Ok(Err(error))=>json!({"snapshot_error":error}),
            Err(_)=>json!({"snapshot_error":"snapshot panicked; original error retained"}),
        }
    }) } else {None};
    drop(bridge); drop(worker);
    let worker_absent=pid.is_none_or(|pid|unsafe{libc::kill(pid as libc::pid_t,0)}==-1 && io::Error::last_os_error().raw_os_error()==Some(libc::ESRCH));
    let server_evidence=server.evidence();
    let cleanup_ok=event_joined&&download_joined&&server_joined&&worker_absent;
    let server_ok=server.shared.errors.lock().unwrap().is_empty();
    let passed=result.is_ok()&&cleanup_ok&&server_ok;
    json!({"name":name,"passed":passed,"status":if passed {"passed"} else {"failed"},"ignored":false,
        "boundary":case.boundary(),"ipc_version":VERSION,"worker_pid":pid,"elapsed_seconds":started.elapsed().as_secs_f64(),
        "checks":observations,"error":result.err(),"failure_snapshot_after_cleanup":failure_snapshot,"native_server":server_evidence,
        "cleanup":{"event_thread_joined":event_joined,"download_thread_joined":download_joined,
            "server_threads_joined_and_sockets_closed":server_joined,"worker_pid_absent_after_stop":worker_absent,"passed":cleanup_ok},
        "scope":"synthetic local TLS/service/Store; no NAS, FUSE, GUI or real ACL source"})
}
fn main() {
    let args:Vec<String>=std::env::args().collect();
    if args.len()!=5 { eprintln!("usage: native-worker-chain-probe CASE WORKER_ELF CERTIFICATE_DIR CASE_DIR"); std::process::exit(2); }
    let executable=PathBuf::from(&args[2]); let certificates=PathBuf::from(&args[3]); let directory=PathBuf::from(&args[4]);
    if !executable.is_absolute() || !certificates.is_absolute() || !directory.is_absolute() {
        eprintln!("absolute isolated paths required"); std::process::exit(2);
    }
    let row=run_case(&args[1],&executable,&certificates,&directory);
    let encoded=serde_json::to_string_pretty(&row).unwrap();
    if let Err(error)=fs::OpenOptions::new().write(true).create_new(true).open(directory.join("case.json"))
        .and_then(|mut file|std::io::Write::write_all(&mut file,encoded.as_bytes())) { eprintln!("cannot preserve result: {error}"); std::process::exit(2); }
    println!("{}",serde_json::to_string(&row).unwrap());
    std::process::exit(if row["passed"]==true {0} else if row["status"]=="blocked" {2} else {1});
}
