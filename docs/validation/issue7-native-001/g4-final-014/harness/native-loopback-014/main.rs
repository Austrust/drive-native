//! Independent synthetic pstream peer; only public product Client APIs are used.
//! No imported product parser/helpers, real credentials, NAS or fixed network port.
use serde_json::{Value, json};
use sproto::{Client, Error, TlsMode, client::{Config, Credentials}};
use sproto::actions::{
    list::FileType,
    native::{EventCursor, NativeVersion, Watch},
    namespace::NamespaceTarget,
    transfer::ContentVersion,
};
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

type HResult<T> = Result<T, String>;
const BODY: &[u8] = b"synthetic-upload-body";
const OLD_BODY: &[u8] = b"existing-destination-must-survive";
const WIDE_CODE: u64 = 0x1_0000_3002;

#[derive(Clone, Copy, Debug)]
enum Action { Download, Poll, Upload, Rename, Remove, Mkdir }
impl Action {
    fn wire(self) -> &'static str {
        match self {
            Self::Download => "download", Self::Poll => "long_poll",
            Self::Upload | Self::Mkdir => "upload", Self::Rename => "rename", Self::Remove => "remove",
        }
    }
    fn label(self) -> &'static str {
        match self { Self::Mkdir => "mkdir", _ => self.wire() }
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
enum Stage { Action, Auth, Connect }
#[derive(Clone)]
enum Expected {
    Error { kind: &'static str, code: Option<u64> },
    Notification { changed: bool, digest: &'static str, views: Vec<u64> },
}
#[derive(Clone)]
struct Case { name: String, action: Action, stage: Stage, reply: Value, expected: Expected }

fn require(ok: bool, message: &str) -> HResult<()> {
    if ok { Ok(()) } else { Err(message.to_owned()) }
}
fn io<T>(result: std::io::Result<T>) -> HResult<T> { result.map_err(|e| e.to_string()) }
fn byte(stream: &mut TcpStream) -> HResult<u8> {
    let mut b = [0]; io(stream.read_exact(&mut b))?; Ok(b[0])
}
fn bytes<const N: usize>(stream: &mut TcpStream) -> HResult<[u8; N]> {
    let mut value = [0; N]; io(stream.read_exact(&mut value))?; Ok(value)
}

// This deliberately small peer codec is independent of the private product codec.
// It decodes only bounded synthetic client requests and consumes upload payloads.
fn decode(stream: &mut TcpStream, first: u8, depth: usize, budget: &mut usize) -> HResult<Value> {
    require(depth <= 24 && *budget > 0, "peer request nesting/value budget")?;
    *budget -= 1;
    match first {
        0x00 => { require(byte(stream)? == 0, "null padding")?; Ok(Value::Null) }
        0x01 => {
            let value = match byte(stream)? {
                1 => u64::from(byte(stream)?),
                2 => u64::from(u16::from_be_bytes(bytes(stream)?)),
                4 => u64::from(u32::from_be_bytes(bytes(stream)?)),
                8 => u64::from_be_bytes(bytes(stream)?),
                _ => return Err("unsupported integer width".into()),
            };
            Ok(json!(value))
        }
        0x10 => {
            let size = usize::from(u16::from_be_bytes(bytes(stream)?));
            require(size <= 16384, "peer string limit")?;
            let mut data = vec![0; size]; io(stream.read_exact(&mut data))?;
            Ok(Value::String(String::from_utf8(data).map_err(|e| e.to_string())?))
        }
        0x30 => {
            let length = u64::from_be_bytes(bytes(stream)?);
            require(length <= 4096, "peer synthetic binary limit")?;
            let mut data = vec![0; length as usize]; io(stream.read_exact(&mut data))?;
            Ok(json!({"synthetic_binary_length": length, "synthetic_binary_matches": data == BODY}))
        }
        0x41 => {
            let mut values = Vec::new();
            loop {
                let tag = byte(stream)?; if tag == 0x40 { break; }
                values.push(decode(stream, tag, depth + 1, budget)?);
            }
            Ok(Value::Array(values))
        }
        0x42 => {
            let mut values = serde_json::Map::new();
            loop {
                let tag = byte(stream)?; if tag == 0x40 { break; }
                require(tag == 0x10, "map key is not a string")?;
                let key = decode(stream, tag, depth + 1, budget)?;
                let key = key.as_str().ok_or("missing map key")?.to_owned();
                let tag = byte(stream)?;
                require(!values.contains_key(&key), "duplicate peer map key")?;
                values.insert(key, decode(stream, tag, depth + 1, budget)?);
            }
            Ok(Value::Object(values))
        }
        _ => Err(format!("unsupported peer request tag {first:#x}")),
    }
}
fn encode(value: &Value, out: &mut Vec<u8>) -> HResult<()> {
    match value {
        Value::Null => out.extend_from_slice(&[0, 0]),
        Value::Bool(value) => out.extend_from_slice(&[1, 1, u8::from(*value)]),
        Value::Number(number) => {
            let value = number.as_u64().ok_or("peer response requires unsigned number")?;
            let width = if value < 0x100 { 1 } else if value < 0x10000 { 2 } else if value < 0x1_0000_0000 { 4 } else { 8 };
            out.extend_from_slice(&[1, width]);
            out.extend_from_slice(&value.to_be_bytes()[8 - usize::from(width)..]);
        }
        Value::String(text) => {
            let size = u16::try_from(text.len()).map_err(|e| e.to_string())?;
            out.push(0x10); out.extend_from_slice(&size.to_be_bytes()); out.extend_from_slice(text.as_bytes());
        }
        Value::Array(values) => {
            out.push(0x41); for value in values { encode(value, out)?; } out.push(0x40);
        }
        Value::Object(values) => {
            out.push(0x42);
            for (key, value) in values { encode(&Value::String(key.clone()), out)?; encode(value, out)?; }
            out.push(0x40);
        }
    }
    Ok(())
}
fn request(stream: &mut TcpStream, command: u8) -> HResult<Value> {
    let header: [u8; 8] = bytes(stream)?;
    require(header[0..4] == [0x25, 0x52, 0x18, 0x14], "request magic")?;
    require(header[4] == 0x46 && header[5] == command && header[6..8] == [0, 0], "request header/command")?;
    let tag = byte(stream)?;
    decode(stream, tag, 0, &mut 4096)
}
fn respond(stream: &mut TcpStream, response: &Value) -> HResult<()> {
    let mut payload = Vec::new(); encode(response, &mut payload)?;
    io(stream.write_all(&payload))?; io(stream.flush())
}
fn peer(listener: TcpListener, case: Case) -> HResult<Value> {
    io(listener.set_nonblocking(true))?;
    let deadline = Instant::now() + Duration::from_secs(4);
    let (mut stream, address) = loop {
        match listener.accept() {
            Ok(pair) => break pair,
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock && Instant::now() < deadline =>
                std::thread::sleep(Duration::from_millis(2)),
            Err(error) => return Err(format!("loopback accept: {error}")),
        }
    };
    require(address.ip().is_loopback(), "non-loopback peer")?;
    io(stream.set_read_timeout(Some(Duration::from_secs(4))))?;
    io(stream.set_write_timeout(Some(Duration::from_secs(4))))?;
    let mut commands = Vec::new();
    for (index, command) in [0x19, 0x12, 0x19, 0x0e].into_iter().enumerate() {
        let message = request(&mut stream, command)?;
        commands.push(command);
        if command == 0x12 {
            require(message["agent"]["restore_id"] == "loopback-restore", "auth restore identity")?;
            require(message["agent"]["device_uuid"] == "synthetic-loopback-device", "auth device identity")?;
            require(message["username"] == "synthetic-loopback-user", "synthetic username")?;
            if case.stage == Stage::Auth {
                respond(&mut stream, &case.reply)?;
                return Ok(json!({"commands": commands, "reply_stage":"auth", "action_requests":0}));
            }
        }
        if command == 0x0e && case.stage == Stage::Connect {
            respond(&mut stream, &case.reply)?;
            return Ok(json!({"commands":commands, "reply_stage":"connect", "action_requests":0}));
        }
        let response = match index {
            0 | 2 => json!({"database_restore_id":"loopback-restore", "package_version":{"build":17892}}),
            1 => json!({"session":"synthetic-loopback-session"}),
            _ => json!({"alive":150}),
        };
        respond(&mut stream, &response)?;
    }
    let message = request(&mut stream, 0x01)?;
    require(message["action"] == case.action.wire(), "action dispatch did not reach intended public API")?;
    match case.action {
        Action::Download => {
            require(message["path"] == "id:42", "download identity addressing")?;
            require(message["target_sync_id"] == 7, "download target revision")?;
        }
        Action::Poll => {
            require(message["profile_digest"] == "prior-digest", "request profile digest")?;
            require(message["subscribe"][0]["view_id"] == 7 && message["subscribe"][0]["sync_id"] == 77,
                    "long_poll subscription identity/cursor")?;
        }
        Action::Upload => {
            require(message["path"] == "id:42", "upload identity addressing")?;
            require(message["file"]["data"]["synthetic_binary_matches"] == true, "upload body bytes")?;
            require(message["file"]["data"]["synthetic_binary_length"] == BODY.len(), "upload body length")?;
        }
        Action::Rename => {
            require(message["src_path"] == "/fixture/source" && message["dst_path"] == "/fixture/destination", "rename operands")?;
        }
        Action::Remove => require(message["path"] == "id:42", "remove identity addressing")?,
        Action::Mkdir => require(message["path"] == "/fixture/new-dir" && message["is_dir"] == 1, "mkdir upload fields")?,
    }
    respond(&mut stream, &case.reply)?;
    Ok(json!({"commands":commands, "reply_stage":"action", "action_requests":1,
              "action":case.action.wire(), "request_fields_checked":true,
              "upload_payload_bytes":if matches!(case.action,Action::Upload) { BODY.len() } else { 0 }}))
}

fn classify(error: Error) -> Value {
    match error {
        Error::DownloadPermissionDenied { code } => json!({"kind":"download_permission_denied", "code":code}),
        Error::MutationPermissionUncertain { code } => json!({"kind":"mutation_permission_uncertain", "code":code}),
        Error::MutationPermissionDenied { code } => json!({"kind":"mutation_permission_denied", "code":code}),
        Error::Server { code, .. } => json!({"kind":"server", "code":code}),
        Error::Decode(message) => json!({"kind":"decode", "message":message}),
        Error::SessionInvalid => json!({"kind":"session_invalid"}),
        other => json!({"kind":"other", "message":other.to_string()}),
    }
}
fn checked<T>(result: sproto::Result<T>) -> Value {
    match result { Err(error) => classify(error), Ok(_) => json!({"kind":"unexpected_success"}) }
}
async fn action(client: &Client, which: Action, directory: &Path) -> HResult<Value> {
    let version = NativeVersion { sync_id:7, max_id:5 };
    let target = NamespaceTarget { view_id:7, path:"/fixture/source".into(), file_id:"42".into(),
                                   base:version, file_type:FileType::File };
    Ok(match which {
        Action::Download => {
            let destination = directory.join("existing.bin");
            io(std::fs::write(&destination, OLD_BODY))?;
            let expected = ContentVersion { file_id:"42".into(), version, size:3, hash:"synthetic-unused-md4".into() };
            let result = checked(client.download_checked(&expected, &destination).await);
            require(io(std::fs::read(&destination))? == OLD_BODY, "download failure changed existing destination")?;
            result
        }
        Action::Poll => {
            let watches = [Watch { view_id:7, node_id:9, path:"/fixture".into(), cursor:EventCursor(77) }];
            match client.wait_changes(&watches, "prior-digest").await {
                Ok(n) => json!({"kind":"notification", "profile_changed":n.profile_changed,
                                "profile_digest":n.profile_digest, "changed_views":n.changed_views}),
                Err(error) => classify(error),
            }
        }
        Action::Upload => {
            let snapshot = directory.join("snapshot.bin"); io(std::fs::write(&snapshot, BODY))?;
            let result = checked(client.upload_snapshot_at_identity("42", None, version, &snapshot).await);
            require(io(std::fs::read(&snapshot))? == BODY, "upload action changed immutable source")?;
            result
        }
        Action::Rename => checked(client.rename_native(&target, "/fixture/destination").await),
        Action::Remove => checked(client.remove_at_identity(&target).await),
        Action::Mkdir => checked(client.mkdir_native(7, "/fixture/new-dir").await),
    })
}
fn matches_expected(observed: &Value, expected: &Expected) -> bool {
    match expected {
        Expected::Error { kind, code } => observed["kind"] == *kind &&
            code.is_none_or(|code| observed["code"].as_u64() == Some(code)),
        Expected::Notification { changed, digest, views } => observed["kind"] == "notification" &&
            observed["profile_changed"] == *changed && observed["profile_digest"] == *digest &&
            observed["changed_views"] == json!(views),
    }
}
fn expectation(expected: &Expected) -> Value {
    match expected {
        Expected::Error { kind, code } => json!({"kind":kind,"code":code}),
        Expected::Notification { changed, digest, views } => json!({"kind":"notification",
            "profile_changed":changed,"profile_digest":digest,"changed_views":views}),
    }
}
async fn run_case(case: Case, work: PathBuf) -> HResult<Value> {
    let directory = tempfile::Builder::new().prefix("loopback-").tempdir_in(work).map_err(|e|e.to_string())?;
    let listener = io(TcpListener::bind("127.0.0.1:0"))?;
    let address = io(listener.local_addr())?;
    let server_case = case.clone();
    let server = tokio::task::spawn_blocking(move || peer(listener, server_case));
    let config = Config::builder().host("127.0.0.1").port(address.port()).tls(TlsMode::None)
        .device_uuid("synthetic-loopback-device".to_owned()).max_channels(1usize)
        .credentials(Credentials::Password {
            username:"synthetic-loopback-user".into(), password:"synthetic-not-a-real-credential".into(), otp:None,
        }).build();
    let connected = tokio::time::timeout(Duration::from_secs(6), Client::connect(config)).await;
    let observed = match connected {
        Ok(Ok(client)) => {
            if case.stage != Stage::Action { json!({"kind":"unexpected_connect_success"}) }
            else {
                match tokio::time::timeout(Duration::from_secs(6), action(&client, case.action, directory.path())).await {
                    Ok(Ok(value)) => value,
                    Ok(Err(error)) => json!({"kind":"harness_action_error", "message":error}),
                    Err(_) => json!({"kind":"action_timeout"}),
                }
            }
        }
        Ok(Err(error)) if case.stage != Stage::Action => classify(error),
        Ok(Err(error)) => json!({"kind":"unexpected_connect_error", "error":classify(error)}),
        Err(_) => json!({"kind":"connect_timeout"}),
    };
    let server_result = match tokio::time::timeout(Duration::from_secs(5), server).await {
        Ok(Ok(Ok(value))) => json!({"passed":true, "trace":value}),
        Ok(Ok(Err(error))) => json!({"passed":false, "error":error}),
        Ok(Err(error)) => json!({"passed":false, "join_error":error.to_string()}),
        Err(_) => json!({"passed":false, "error":"peer completion timeout"}),
    };
    let passed = matches_expected(&observed, &case.expected) && server_result["passed"] == true;
    Ok(json!({"name":case.name, "passed":passed, "status":if passed {"pass"} else {"fail"},
              "bind_address":address.to_string(), "expected":expectation(&case.expected),
              "observed":observed, "server":server_result}))
}
fn refusal(action: Action, code: u64) -> Case {
    let (kind, code_check) = if code > u64::from(u32::MAX) { ("decode", None) }
        else if matches!(action, Action::Download) && matches!(code, 0x3002 | 0x3008) {
            ("download_permission_denied", Some(code))
        } else if matches!(action, Action::Upload | Action::Mkdir) && code == 0x9003 {
            // T1 delta: the only proven pre-commit ACL refusal, typed at the
            // direct upload reply boundary (creation/mkdir is typed at wire
            // level but downgraded to a hint by the runtime worker).
            ("mutation_permission_denied", Some(code))
        } else if (matches!(action, Action::Upload | Action::Mkdir | Action::Rename | Action::Remove) && matches!(code,0x3002 | 0x3004))
            || (matches!(action,Action::Upload | Action::Mkdir) && matches!(code,0x9004))
            || (matches!(action,Action::Remove) && matches!(code,0xa003 | 0xa004)) {
            ("mutation_permission_uncertain", Some(code))
        } else { ("server", Some(code)) };
    Case { name:format!("{}_reply_{code:x}", action.label()), action, stage:Stage::Action,
           reply:json!({"error":{"code":code,"reason":"synthetic refusal"}}),
           expected:Expected::Error { kind, code:code_check } }
}
fn cases() -> Vec<Case> {
    let mut cases = Vec::new();
    for code in [0x3002,0x3008,0x3004,0x8004,0x9005,0xa005,WIDE_CODE] {
        cases.push(refusal(Action::Download,code));
    }
    // T1 adversarial additions: the typed pre-commit upload refusal must
    // not leak into download responses or handshake/connect classification.
    for code in [0x9003u64, 0xa004] {
        cases.push(refusal(Action::Download, code));
    }
    for stage in [Stage::Auth, Stage::Connect] {
        cases.push(Case { name:format!("t1_setup_{stage:?}_9003_is_generic"), action:Action::Download, stage,
            reply:json!({"error":{"code":0x9003u64,"reason":"synthetic setup refusal"}}),
            expected:Expected::Error { kind:"server", code:Some(0x9003) } });
    }
    for stage in [Stage::Auth, Stage::Connect] {
        cases.push(Case { name:format!("download_setup_{stage:?}_3002_is_generic"), action:Action::Download, stage,
            reply:json!({"error":{"code":0x3002u64,"reason":"synthetic setup refusal"}}),
            expected:Expected::Error { kind:"server", code:Some(0x3002) } });
    }
    for action in [Action::Upload,Action::Rename,Action::Remove,Action::Mkdir] {
        for code in [0x3002,0x3004,0x3008,0x8004,0x9003,0x9004,0x9005,0xa003,0xa004,0xa005,WIDE_CODE] {
            cases.push(refusal(action,code));
        }
    }
    let success = [
        ("profile_only_true_empty_views", json!({"changes":[],"profile_changed":1,"profile_digest":"next-digest"}),true,"next-digest",vec![]),
        ("profile_only_true_other_view", json!({"changes":[8],"profile_changed":1,"profile_digest":"next-digest"}),true,"next-digest",vec![8]),
        ("profile_false_explicit_digest", json!({"changes":[],"profile_changed":0,"profile_digest":"same-digest"}),false,"same-digest",vec![]),
        ("profile_false_omitted_digest", json!({"changes":[],"profile_changed":0}),false,"prior-digest",vec![]),
        ("profile_omitted_flag_and_digest", json!({"changes":[7]}),false,"prior-digest",vec![7]),
    ];
    for (name,reply,changed,digest,views) in success {
        cases.push(Case { name:name.into(), action:Action::Poll, stage:Stage::Action, reply,
            expected:Expected::Notification { changed,digest,views } });
    }
    for (name,reply) in [
        ("profile_flag_two_rejected",json!({"changes":[],"profile_changed":2,"profile_digest":"x"})),
        ("profile_flag_string_rejected",json!({"changes":[],"profile_changed":"1","profile_digest":"x"})),
        ("profile_flag_null_rejected",json!({"changes":[],"profile_changed":null,"profile_digest":"x"})),
        ("profile_changed_without_digest_rejected",json!({"changes":[],"profile_changed":1})),
        ("profile_numeric_digest_rejected",json!({"changes":[],"profile_changed":1,"profile_digest":9})),
        ("profile_changes_missing_rejected",json!({"profile_changed":1,"profile_digest":"x"})),
        ("profile_changes_scalar_rejected",json!({"changes":7,"profile_changed":1,"profile_digest":"x"})),
        ("profile_changes_noninteger_rejected",json!({"changes":["7"],"profile_changed":1,"profile_digest":"x"})),
    ] {
        cases.push(Case { name:name.into(), action:Action::Poll, stage:Stage::Action, reply,
            expected:Expected::Error { kind:"decode", code:None } });
    }
    cases
}
async fn main_result() -> HResult<bool> {
    let mut arguments = std::env::args().skip(1);
    let mut output = None; let mut work = None;
    while let Some(flag) = arguments.next() {
        let value = arguments.next().ok_or("option value missing")?;
        match flag.as_str() { "--output" => output=Some(PathBuf::from(value)), "--work-dir" => work=Some(PathBuf::from(value)), _ => return Err(format!("unknown option {flag}")) }
    }
    let output = output.ok_or("--output <new absolute results.json> required")?;
    let work = work.ok_or("--work-dir <private absolute scratch directory> required")?;
    require(output.is_absolute() && work.is_absolute(), "absolute private output/work paths required")?;
    require(!output.exists(), "refuse to overwrite prior result")?;
    io(std::fs::create_dir_all(&work))?;
    let mut rows = Vec::new(); let started = Instant::now();
    for case in cases() {
        let name = case.name.clone(); let at = Instant::now(); let directory = work.clone();
        // Isolate a product panic or timeout to this row; later matrix rows still run.
        let mut job = tokio::spawn(async move { run_case(case,directory).await });
        let mut row = match tokio::time::timeout(Duration::from_secs(18), &mut job).await {
            Ok(Ok(Ok(row))) => row,
            Ok(Ok(Err(error))) => json!({"name":name,"passed":false,"status":"fail","harness_error":error}),
            Ok(Err(error)) => json!({"name":name,"passed":false,"status":"fail","task_join_error":error.to_string()}),
            Err(_) => { job.abort(); let _ = job.await; json!({"name":name,"passed":false,"status":"fail","harness_error":"case timeout (18s)"}) },
        };
        row["elapsed_seconds"] = json!(at.elapsed().as_secs_f64());
        println!("{row}"); rows.push(row);
    }
    let passed = rows.iter().filter(|row|row["passed"]==true).count();
    let report = json!({"scope":"synthetic IPv4 loopback / public sproto Client APIs; no NAS, real credentials, product worker ELF or live ACL confirmation",
        "counts":{"passed":passed,"failed":rows.len()-passed,"ignored":0,"total":rows.len()},
        "complete":true,"all_passed":passed==rows.len(),"elapsed_seconds":started.elapsed().as_secs_f64(),"cases":rows});
    let mut file = std::fs::OpenOptions::new().write(true).create_new(true).open(&output).map_err(|e|e.to_string())?;
    io(file.write_all(serde_json::to_string_pretty(&report).map_err(|e|e.to_string())?.as_bytes()))?;
    io(file.write_all(b"\n"))?; io(file.sync_all())?;
    println!("{}",json!({"summary":report["counts"],"all_passed":report["all_passed"]}));
    Ok(report["all_passed"] == true)
}
#[tokio::main(flavor="multi_thread", worker_threads=2)]
async fn main() {
    let code = match main_result().await { Ok(true)=>0, Ok(false)=>1, Err(error)=>{ eprintln!("harness setup/output error: {error}"); 2 } };
    std::process::exit(code);
}
