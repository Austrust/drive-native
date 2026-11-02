//! Isolated native validation writer. stdin/stdout are private controller pipes.
//! No cleanup, retries, arbitrary remote paths, environment secrets, or raw errors.
//! Build explicitly with --features native-research; this is not a product API.
#[cfg(not(feature = "native-research"))]
fn main() {
    eprintln!("native-research is required");
    std::process::exit(2);
}

#[cfg(feature = "native-research")]
mod peer {
    use serde::Deserialize;
    use serde_json::{Value, json};
    use sproto::{
        Client,
        actions::{
            list::{FileType, NodeInfo},
            native::{EventCursor, NativeVersion, Watch},
            transfer::ContentVersion,
        },
        client::{Config, Credentials},
    };
    use std::{
        fs::{File, OpenOptions},
        io::{BufRead, Write},
        os::unix::fs::{MetadataExt, OpenOptionsExt},
        path::PathBuf,
    };
    type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
    const LAB: &str = "/DriveNative-Lab";
    pub struct Diagnostics {
        stage: &'static str,
        log: Option<File>,
    }
    impl Default for Diagnostics {
        fn default() -> Self {
            Self {
                stage: "preflight",
                log: None,
            }
        }
    }
    fn classification(error: &(dyn std::error::Error + 'static)) -> (&'static str, Option<u32>) {
        if let Some(native) = error.downcast_ref::<sproto::Error>() {
            return match native {
                sproto::Error::Server { code, .. } => ("native_server", Some(*code)),
                sproto::Error::Timeout(_) => ("timeout", None),
                sproto::Error::Io(_) | sproto::Error::ConnectionClosed => ("transport", None),
                sproto::Error::Tls(_) => ("tls", None),
                sproto::Error::SessionInvalid => ("session_invalid", None),
                sproto::Error::InvalidConfig(_) => ("configuration", None),
                sproto::Error::UnsupportedServer { .. } => ("unsupported_server", None),
                sproto::Error::CursorReset { .. } => ("cursor_reset", None),
                _ => ("protocol", None),
            };
        }
        if error.downcast_ref::<serde_json::Error>().is_some() {
            return ("input", None);
        }
        if error.downcast_ref::<std::io::Error>().is_some() {
            return ("io", None);
        }
        ("invariant", None)
    }
    impl Diagnostics {
        fn stage(&mut self, stage: &'static str) -> Result<()> {
            self.stage = stage;
            if let Some(log) = &mut self.log {
                journal(log, stage, json!({"started":true}))?;
            }
            Ok(())
        }
        fn frame(&self, error: &(dyn std::error::Error + 'static)) -> Value {
            let (category, code) = classification(error);
            json!({"failed":true,"stage":self.stage,"category":category,"native_code":code})
        }
        pub fn report(&mut self, error: &(dyn std::error::Error + 'static)) {
            // Neither Display nor Debug of an upstream error may cross this boundary.
            let frame = self.frame(error);
            if let Some(log) = &mut self.log {
                let _ = journal(log, "failure", frame.clone());
            }
            let _ = send(frame.clone());
            eprintln!("{}", frame);
        }
    }
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Input {
        host: String,
        tls_name: String,
        username: String,
        password: String,
        #[serde(default)]
        trusted_ca_pem: Vec<String>,
    }
    #[derive(Deserialize)]
    #[serde(rename_all = "snake_case")]
    enum Command {
        Create,
        Update,
        Events,
        Stat,
        StatCopy,
        Quit,
    }
    fn read<T: serde::de::DeserializeOwned>(reader: &mut impl BufRead) -> Result<T> {
        let mut line = Vec::new();
        std::io::Read::take(reader, 131_073).read_until(b'\n', &mut line)?;
        if line.len() > 131_072 || line.last() != Some(&b'\n') {
            return Err("bounded input required".into());
        }
        Ok(serde_json::from_slice(&line)?)
    }
    fn send(v: Value) -> Result<()> {
        let mut out = std::io::stdout().lock();
        serde_json::to_writer(&mut out, &v)?;
        writeln!(out)?;
        out.flush()?;
        Ok(())
    }
    fn journal(f: &mut File, stage: &str, value: Value) -> Result<()> {
        serde_json::to_writer(&mut *f, &json!({"stage":stage,"value":value}))?;
        writeln!(f)?;
        f.sync_all()?;
        Ok(())
    }
    fn content(n: &NodeInfo) -> ContentVersion {
        ContentVersion {
            file_id: n.file_id.clone(),
            version: NativeVersion {
                sync_id: n.sync_id,
                max_id: n.max_id,
            },
            size: n.file_size,
            hash: n.hash.clone(),
        }
    }
    fn live(n: &NodeInfo, name: &str, ty: FileType) -> bool {
        n.name == name
            && n.file_type == ty
            && !n.is_removed
            && n.node_id != 0
            && !n.file_id.is_empty()
    }
    async fn root(client: &Client, view: u64, path: &str, id: &str, nid: u64) -> Result<()> {
        if client
            .current_selected_node(view, path, id, nid)
            .await?
            .is_none()
        {
            return Err("root changed".into());
        }
        Ok(())
    }
    async fn listing(client: &Client, watch: &Watch) -> Result<Vec<NodeInfo>> {
        Ok(client.list_dir(watch.view_id, &watch.path).await?)
    }
    pub async fn run(trace: &mut Diagnostics) -> Result<()> {
        if unsafe { libc::geteuid() } == 0 || unsafe { libc::isatty(libc::STDIN_FILENO) } != 0 {
            return Err("non-root private stdin pipe required".into());
        }
        let mut input_stat = std::mem::MaybeUninit::<libc::stat>::uninit();
        if unsafe { libc::fstat(libc::STDIN_FILENO, input_stat.as_mut_ptr()) } != 0
            || unsafe { input_stat.assume_init() }.st_mode & libc::S_IFMT != libc::S_IFIFO
        {
            return Err("anonymous stdin pipe required".into());
        }
        drive_runtime::disable_core_dumps()?;
        unsafe {
            libc::umask(0o077);
        }
        let mut args = std::env::args().skip(1);
        let out = PathBuf::from(args.next().ok_or("owned new output directory required")?);
        if args.next().is_some() || !out.is_absolute() {
            return Err("invalid arguments".into());
        }
        let m = std::fs::symlink_metadata(&out)?;
        if !m.is_dir() || m.uid() != unsafe { libc::geteuid() } || m.mode() & 0o077 != 0 {
            return Err("private owned output required".into());
        }
        let mut log = OpenOptions::new()
            .create_new(true)
            .write(true)
            .mode(0o600)
            .custom_flags(libc::O_NOFOLLOW)
            .open(out.join("peer-journal.jsonl"))?;
        trace.log = Some(log.try_clone()?);
        trace.stage("input_read")?;
        let mut input = std::io::BufReader::new(std::io::stdin());
        let login: Input = read(&mut input)?;
        trace.stage("ca_parse")?;
        let ca = sproto::trust::parse_trusted_pem(&login.trusted_ca_pem)?;
        trace.stage("connect")?;
        let client = Client::connect(
            Config::builder()
                .host(login.host)
                .tls_server_name(login.tls_name)
                .trusted_ca_der(ca)
                .max_channels(2usize)
                .credentials(Credentials::Password {
                    username: login.username,
                    password: login.password,
                    otp: None,
                })
                .build(),
        )
        .await?;
        trace.stage("personal_root")?;
        let home = client.personal_root().await?;
        trace.stage("personal_root_validate")?;
        if !home.writable || !home.download_allowed || home.view_id == 0 || home.node_id == 0 {
            return Err("personal root unavailable".into());
        }
        // '/' is the protocol path relative to the server-returned personal view.
        // Never use a homes share or accept an arbitrary caller-supplied root.
        trace.stage("personal_root_list")?;
        let listed = client.list_dir(home.view_id, "/").await?;
        trace.stage("lab_select")?;
        let lab = listed
            .into_iter()
            .find(|n| live(n, "DriveNative-Lab", FileType::Dir))
            .ok_or("authorized Lab must already exist")?;
        trace.stage("lab_verify")?;
        root(&client, home.view_id, LAB, &lab.file_id, lab.node_id).await?;
        let leaf = format!("native-validation-{}", uuid::Uuid::new_v4());
        let path = format!("{LAB}/{leaf}");
        trace.stage("scope_preflight")?;
        if client
            .list_dir(home.view_id, LAB)
            .await?
            .iter()
            .any(|n| n.name == leaf && !n.is_removed)
        {
            return Err("fresh name collision".into());
        }
        // Intent is durable before the sole mkdir attempt. Unknown results stop.
        trace.stage("mkdir_intent")?;
        journal(
            &mut log,
            "mkdir_intent",
            json!({"path":path,"parent":lab.file_id}),
        )?;
        trace.stage("mkdir_request")?;
        let made = match client.mkdir_at_identity(&lab.file_id, &leaf, &path).await {
            Ok(v) => v,
            Err(error) => {
                trace.stage("mkdir_reconcile")?;
                let observed = client
                    .list_dir(home.view_id, LAB)
                    .await
                    .ok()
                    .map(|v| v.iter().any(|n| live(n, &leaf, FileType::Dir)));
                journal(
                    &mut log,
                    "mkdir_uncertain",
                    json!({"desired_name_observed":observed,"retry":false}),
                )?;
                trace.stage("mkdir_uncertain")?;
                return Err(error.into());
            }
        };
        trace.stage("lab_reverify")?;
        root(&client, home.view_id, LAB, &lab.file_id, lab.node_id).await?;
        trace.stage("scope_verify")?;
        root(&client, home.view_id, &path, &made.file_id, made.node_id).await?;
        let mut watch = Watch {
            view_id: home.view_id,
            node_id: made.node_id,
            path: path.clone(),
            cursor: EventCursor(0),
        };
        trace.stage("watermark")?;
        watch.cursor = client.current_sync_watermark(&watch).await?;
        journal(
            &mut log,
            "scope_verified",
            json!({"view_id":home.view_id,"node_id":made.node_id,"file_id":made.file_id,"path":path,"cursor":watch.cursor.0}),
        )?;
        trace.stage("ready_reply")?;
        send(
            json!({"ready":true,"scope":{"view_id":home.view_id,"node_id":made.node_id,"file_id":made.file_id,"path":path},"protocol_ipc":drive_runtime::wire::VERSION}),
        )?;
        let mut previous: Option<ContentVersion> = None;
        let mut generation = 0u8;
        loop {
            trace.stage("await_command")?;
            let command: Command = read(&mut input)?;
            if matches!(command, Command::Quit) {
                send(json!({"stopped":true,"nas_cleanup":false}))?;
                break;
            }
            trace.stage("command_lab_verify")?;
            root(&client, home.view_id, LAB, &lab.file_id, lab.node_id).await?;
            trace.stage("command_scope_verify")?;
            root(&client, home.view_id, &path, &made.file_id, made.node_id).await?;
            if matches!(command, Command::Stat | Command::StatCopy) {
                trace.stage("stat_list")?;
                let name = if matches!(command, Command::Stat) {
                    "A.bin"
                } else {
                    "Copy.bin"
                };
                let entries = listing(&client, &watch).await?;
                let mut selected = entries.iter().filter(|n| live(n, name, FileType::File));
                let node = selected.next().ok_or("stat fixture absent")?;
                if selected.next().is_some() {
                    return Err("ambiguous stat fixture".into());
                }
                if matches!(command, Command::Stat)
                    && !previous.as_ref().is_some_and(|p| p.file_id == node.file_id)
                {
                    return Err("stat fixture identity changed".into());
                }
                trace.stage("stat_reply")?;
                // Private controller response. No durable write or change of
                // the peer's mutation baseline: a later Update still requires
                // the exact version that peer originally produced.
                send(
                    json!({"observed":true,"node_id":node.node_id,"file_id":node.file_id,
                    "sync_id":node.sync_id,"max_id":node.max_id,"size":node.file_size,"mtime":node.mtime,"hash":node.hash}),
                )?;
                continue;
            }
            if matches!(command, Command::Events) {
                trace.stage("pull_events")?;
                let batch = client.pull_events(&watch).await?;
                trace.stage("events_validate")?;
                if batch.recovery_required || batch.next.0 < watch.cursor.0 {
                    return Err("event recovery required".into());
                }
                let matching = batch
                    .events
                    .iter()
                    .filter(|e| previous.as_ref().is_some_and(|p| p.file_id == e.file_id))
                    .count();
                let advanced = batch.next.0 > watch.cursor.0;
                watch.cursor = batch.next;
                send(
                    json!({"events":batch.events.len(),"file_events":matching,"advanced":advanced,"recovery_required":false}),
                )?;
                continue;
            }
            trace.stage("command_validate")?;
            match (&command, generation) {
                (Command::Create, 0) | (Command::Update, 1) => {}
                _ => return Err("invalid write order; no retries".into()),
            }
            trace.stage("pre_upload_list")?;
            let current = listing(&client, &watch)
                .await?
                .into_iter()
                .find(|n| n.name == "A.bin" && !n.is_removed);
            trace.stage("pre_upload_validate")?;
            match (&previous, &current) {
                (None, None) => {}
                (Some(p), Some(n))
                    if live(n, "A.bin", FileType::File)
                        && content(n).file_id == p.file_id
                        && content(n).version == p.version
                        && content(n).size == p.size
                        && content(n).hash == p.hash => {}
                _ => return Err("fixture changed independently".into()),
            }
            trace.stage("body_prepare")?;
            let bytes = vec![b'A' + generation; 2 * 1024 * 1024];
            let expected = drive_core::sha256(&bytes);
            let source = out.join(format!("body-{generation}.bin"));
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .mode(0o600)
                .custom_flags(libc::O_NOFOLLOW)
                .open(&source)?;
            file.write_all(&bytes)?;
            file.sync_all()?;
            drop(file);
            trace.stage("upload_intent")?;
            journal(
                &mut log,
                "upload_intent",
                json!({"generation":generation,"sha256":expected,"size":bytes.len()}),
            )?;
            let identity = previous
                .as_ref()
                .map(|p| p.file_id.as_str())
                .unwrap_or(&made.file_id);
            let base = previous
                .as_ref()
                .map(|p| p.version)
                .unwrap_or(NativeVersion {
                    sync_id: 0,
                    max_id: 0,
                });
            trace.stage("upload_request")?;
            let receipt = client
                .upload_snapshot_at_identity(
                    identity,
                    previous.is_none().then_some("A.bin"),
                    base,
                    &source,
                )
                .await;
            trace.stage("upload_observe_list")?;
            let observed = listing(&client, &watch)
                .await?
                .into_iter()
                .find(|n| live(n, "A.bin", FileType::File));
            trace.stage("upload_observe_download")?;
            let verified = if let Some(n) = observed.as_ref() {
                client
                    .download_checked(&content(n), &out.join(format!("verified-{generation}.bin")))
                    .await
                    .ok()
                    .map(|v| v.sha256 == expected)
            } else {
                Some(false)
            };
            let acknowledged = receipt.is_ok();
            journal(
                &mut log,
                "upload_observed",
                json!({"generation":generation,"acknowledged":acknowledged,"desired_content_observed":verified,"retry":false}),
            )?;
            trace.stage("upload_validate_receipt")?;
            // Retain the typed native class, never its error text; an observed
            // desired body still does not attribute an uncertain acknowledgement.
            let accepted = receipt?;
            let observed = observed.ok_or("uploaded file absent")?;
            if verified != Some(true)
                || content(&observed).file_id != accepted.file_id
                || content(&observed).version != accepted.version
            {
                return Err("receipt/content mismatch".into());
            }
            trace.stage("upload_scope_reverify")?;
            root(&client, home.view_id, &path, &made.file_id, made.node_id).await?;
            previous = Some(accepted);
            generation += 1;
            trace.stage("upload_reply")?;
            send(
                json!({"committed":true,"generation":generation,"sha256":expected,"size":bytes.len(),"file_id":observed.file_id,"sync_id":observed.sync_id,"max_id":observed.max_id}),
            )?;
        }
        Ok(())
    }
    #[cfg(test)]
    mod tests {
        use super::*;
        #[test]
        fn command_set_has_no_delete_or_path() {
            assert!(serde_json::from_str::<Command>("\"remove\"").is_err());
            assert!(serde_json::from_str::<Command>("{\"update\":\"/other\"}").is_err());
            assert!(matches!(
                serde_json::from_str::<Command>("\"stat\"").unwrap(),
                Command::Stat
            ));
            assert!(matches!(
                serde_json::from_str::<Command>("\"stat_copy\"").unwrap(),
                Command::StatCopy
            ));
            assert!(serde_json::from_str::<Command>("{\"stat\":\"/other\"}").is_err());
        }
        #[test]
        fn bounded_pipe_frames() {
            let mut input = std::io::Cursor::new(b"\"events\"\n");
            assert!(matches!(
                read::<Command>(&mut input).unwrap(),
                Command::Events
            ));
            assert!(read::<Command>(&mut std::io::Cursor::new(b"\"events\"")).is_err());
        }
        #[test]
        fn diagnostics_never_serialize_native_reason_or_error_text() {
            let mut trace = Diagnostics::default();
            trace.stage("connect").unwrap();
            let error = sproto::Error::Server {
                code: 402,
                reason: "SECRET user/path/password".into(),
            };
            let frame = trace.frame(&error);
            assert_eq!(frame["stage"], "connect");
            assert_eq!(frame["category"], "native_server");
            assert_eq!(frame["native_code"], 402);
            assert!(!frame.to_string().contains("SECRET"));
            let io = std::io::Error::other("SECRET path");
            assert_eq!(trace.frame(&io)["category"], "io");
            assert!(!trace.frame(&io).to_string().contains("SECRET"));
        }
        #[test]
        fn prewrite_stages_are_durable_without_identity_fields() {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("trace");
            let file = OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(&path)
                .unwrap();
            let mut trace = Diagnostics {
                stage: "preflight",
                log: Some(file),
            };
            for stage in [
                "input_read",
                "connect",
                "personal_root",
                "personal_root_list",
                "lab_select",
                "lab_verify",
            ] {
                trace.stage(stage).unwrap();
            }
            let text = std::fs::read_to_string(path).unwrap();
            let rows: Vec<Value> = text
                .lines()
                .map(|s| serde_json::from_str(s).unwrap())
                .collect();
            assert_eq!(rows.len(), 6);
            assert_eq!(rows.last().unwrap()["stage"], "lab_verify");
            assert!(!text.contains("path") && !text.contains("password"));
        }
    }
}
#[cfg(feature = "native-research")]
#[tokio::main]
async fn main() {
    let mut trace = peer::Diagnostics::default();
    if let Err(error) = peer::run(&mut trace).await {
        trace.report(error.as_ref());
        std::process::exit(1);
    }
}
