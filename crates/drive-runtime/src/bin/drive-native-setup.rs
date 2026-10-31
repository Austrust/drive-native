//! Transient read-only native login/directory selector for the GUI. Credentials
//! enter a private stdin pipe; stdout contains only versioned, bounded metadata.
use serde::Deserialize;
use sproto::{
    Client,
    actions::{
        list::FileType,
        native::{EventCursor, EventKind, Watch},
    },
    client::{Config, Credentials},
};
use std::io::{self, BufRead, Read, Write};
use std::collections::BTreeMap;
#[derive(Deserialize)]
#[serde(tag = "command", rename_all = "snake_case", deny_unknown_fields)]
enum Request {
    Login {
        host: String,
        tls_name: String,
        #[serde(default)]
        trusted_ca_pem: Vec<String>,
        username: String,
        password: String,
        device_id: String,
    },
    List {
        #[serde(default)]
        view_id: Option<u64>,
        path: String,
    },
    Quit,
}
fn valid_path(path: &str) -> bool {
    path == "/"
        || path.starts_with('/')
            && !path.contains('\0')
            && path.len() < 8192
            && path[1..].split('/').all(|s| !matches!(s, "" | "." | ".."))
}
fn category(error: &sproto::Error) -> &'static str {
    match error {
        sproto::Error::Io(e) if e.kind() == io::ErrorKind::InvalidData => "certificate",
        sproto::Error::Tls(_) => "certificate",
        sproto::Error::Io(_) | sproto::Error::Timeout(_) | sproto::Error::ConnectionClosed => {
            "network"
        }
        sproto::Error::InvalidConfig(_) => "configuration",
        sproto::Error::SessionInvalid => "session_expired",
        sproto::Error::BadMagic | sproto::Error::VersionMismatch { .. }
        | sproto::Error::Decode(_) | sproto::Error::UnsupportedServer { .. }
        | sproto::Error::CursorReset { .. } => "protocol",
        sproto::Error::Server { .. } => "server_rejected",
    }
}
// The reviewed Ubuntu branch observed 4100 on rejected credentials. Scope this
// hint to connection/login; do not claim the server proved a wrong password,
// and do not classify an unrelated directory response as a login failure.
fn login_category(error: &sproto::Error) -> &'static str {
    match error {
        sproto::Error::Server { code: 4100, .. } => "login_rejected",
        _ => category(error),
    }
}
#[derive(Clone)]
struct Root {
    name: String,
    view: u64,
    node: Option<u64>,
    file: Option<String>,
    writable: bool,
    allowed: bool,
    personal: bool,
}
impl Root {
    fn scope(&self) -> Option<drive_core::native::Scope> {
        Some(drive_core::native::Scope {view_id:self.view,node_id:self.node?,
            file_id:self.file.clone().filter(|id|!id.is_empty() && id!="0")?,path:"/".into()})
    }
    fn entry(&self) -> serde_json::Value {
        serde_json::json!({"name":self.name,"path":"/","view_id":self.view,
            "scope":self.scope(),"writable":self.writable,"available":self.allowed,
            "personal":self.personal})
    }
}
async fn roots(client:&Client) -> sproto::Result<(BTreeMap<u64,Root>,Vec<&'static str>)> {
    let mut roots=BTreeMap::new();let mut warnings=Vec::new();
    match client.personal_root().await {
        Ok(home)=>{roots.insert(home.view_id,Root{name:"我的文件".into(),view:home.view_id,
            node:Some(home.node_id),file:None,writable:home.writable,
            allowed:home.download_allowed,personal:true});},
        Err(_)=>warnings.push("personal_unavailable"),
    }
    match client.list_shares().await {
        Ok(shares)=>for share in shares {
            if share.name.is_empty() || share.name.contains('/') || share.name.chars().any(char::is_control) {
                return Err(sproto::Error::Decode("invalid Drive space name".into()));
            }
            roots.entry(share.view_id).or_insert(Root{name:share.name,view:share.view_id,
                node:share.node_id,file:Some(share.file_id),writable:share.writable,
                allowed:share.can_download && share.can_sync,personal:false});
        },
        Err(error)=>{if roots.is_empty(){return Err(error);}warnings.push("team_folders_unavailable");},
    }
    Ok((roots,warnings))
}
async fn folders(client:&Client,root:&mut Root,path:&str) -> sproto::Result<serde_json::Value> {
    if !valid_path(path) || !root.allowed {
        return Err(sproto::Error::InvalidConfig("invalid or inaccessible selector scope".into()));
    }
    // Resolve only the expanded space. No full tree/body fetch during login.
    if path=="/" && root.scope().is_none() {
        if let Some(node_id)=root.node {
            if let Ok(batch)=client.pull_events(&Watch{view_id:root.view,node_id,path:"/".into(),cursor:EventCursor(0)}).await {
                if !batch.recovery_required {
                    let mut entries=batch.events.into_iter().filter(|e|e.path=="/" && !e.removed
                        && e.file_type==FileType::Dir && e.kind==EventKind::Modify);
                    if let Some(first)=entries.next() {
                        if entries.next().is_none(){root.file=Some(first.file_id);}
                    }
                }
            }
        }
    }
    let nodes=client.list_dir(root.view,path).await?;
    let dirs:Vec<_>=nodes.into_iter().filter(|n|n.file_type==FileType::Dir&&!n.is_removed).map(|n|{
        let path=format!("{}/{}",path.trim_end_matches('/'),n.name);
        serde_json::json!({"name":n.name,"path":path,"view_id":root.view,"writable":root.writable,
            "available":true,"scope":{"view_id":root.view,"node_id":n.node_id,"file_id":n.file_id,"path":path}})
    }).collect();
    Ok(serde_json::json!({"path":path,"view_id":root.view,"folders":dirs,"root_scope":root.scope(),"writable":root.writable}))
}
#[tokio::main]
async fn main() {
    if run().await.is_err() {
        std::process::exit(1);
    }
}
async fn run() -> io::Result<()> {
    drive_runtime::disable_core_dumps()?;
    if std::env::args().nth(1).as_deref() != Some("--private-stdin") {
        return Err(io::Error::other("private input required"));
    }
    let input = io::stdin();
    let mut input = input.lock();
    let mut output = io::stdout().lock();
    let mut session: Option<(Client, BTreeMap<u64,Root>)> = None;
    loop {
        let mut line = Vec::new();
        input.by_ref().take(65537).read_until(b'\n', &mut line)?;
        if line.is_empty() {
            break;
        }
        if line.len() > 65536 {
            return Err(io::Error::other("request too large"));
        }
        let request = serde_json::from_slice::<Request>(&line);
        let result: Result<serde_json::Value, &'static str> = match request {
            Ok(Request::Quit) => break,
            Ok(Request::Login {
                host,
                tls_name,
                trusted_ca_pem,
                username,
                password,
                device_id,
            }) => {
                session = None;
                let trusted_ca = sproto::trust::parse_trusted_pem(&trusted_ca_pem);
                if [&host, &tls_name, &username]
                    .iter()
                    .any(|s| s.is_empty() || s.len() > 1024 || s.chars().any(char::is_control))
                    || password.len() > 16384
                    || uuid::Uuid::parse_str(&device_id).is_err()
                {
                    Err("invalid_input")
                } else if trusted_ca.is_err() {
                    Err("certificate")
                } else {
                    match Client::connect(
                        Config::builder()
                            .host(host)
                            .tls_server_name(tls_name)
                            .trusted_ca_der(trusted_ca.unwrap())
                            .device_uuid(device_id)
                            .credentials(Credentials::Password {
                                username,
                                password,
                                otp: None,
                            })
                            .build(),
                    )
                    .await
                    {
                        Err(error) => Err(login_category(&error)),
                        Ok(client) => match roots(&client).await {
                            Err(error)=>Err(category(&error)),
                            Ok((spaces,warnings))=>{
                                let entries:Vec<_>=spaces.values().map(Root::entry).collect();
                                session=Some((client,spaces));
                                Ok(serde_json::json!({"spaces":entries,"warnings":warnings}))
                            }
                        },
                    }
                }
            }
            Ok(Request::List {view_id,path}) => match session.as_mut() {
                Some((client,spaces)) => {
                    let selected=view_id.or_else(||spaces.values().find(|r|r.personal).map(|r|r.view));
                    match selected.and_then(|id|spaces.get_mut(&id)) {
                        Some(root) if !root.allowed => Err("download_permission"),
                        Some(root)=>folders(client,root,&path).await.map_err(|e|category(&e)),
                        None=>Err("configuration_or_permission"),
                    }
                },
                None=>Err("login_required"),
            },
            Err(_) => Err("invalid_input"),
        };
        let value = match result {
            Ok(data) => serde_json::json!({"schema":1,"ok":true,"data":data}),
            Err(code) => serde_json::json!({"schema":1,"ok":false,"error":code}),
        };
        serde_json::to_writer(&mut output, &value)?;
        output.write_all(b"\n")?;
        output.flush()?;
    }
    Ok(())
}

#[cfg(test)]
mod classification_tests {
    use super::*;
    #[test]
    fn rejected_login_is_phase_scoped_and_does_not_echo_server_reason() {
        let e = sproto::Error::Server {code:4100,reason:"untrusted server detail".into()};
        assert_eq!(login_category(&e), "login_rejected");
        assert_eq!(category(&e), "server_rejected");
        assert_eq!(login_category(&sproto::Error::Server {code:99999,reason:"unknown".into()}), "server_rejected");
    }
    #[test]
    fn distinct_failure_classes_do_not_suggest_password_retries_for_protocol_errors() {
        assert_eq!(login_category(&sproto::Error::Timeout("connect")), "network");
        assert_eq!(category(&sproto::Error::Io(io::Error::new(io::ErrorKind::ConnectionRefused,"synthetic"))), "network");
        assert_eq!(category(&sproto::Error::Io(io::Error::new(io::ErrorKind::InvalidData,"synthetic TLS failure"))), "certificate");
        assert_eq!(category(&sproto::Error::Decode("bad frame".into())), "protocol");
        assert_eq!(category(&sproto::Error::SessionInvalid), "session_expired");
        assert_eq!(category(&sproto::Error::InvalidConfig("bad config".into())), "configuration");
    }
}
