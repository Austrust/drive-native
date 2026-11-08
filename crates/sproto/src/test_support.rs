//! In-process protocol fixture for unit tests.
//!
//! It speaks the real framed/pstream wire protocol but serves a synthetic
//! filesystem, so tests that need to observe RPC shape and pagination counts do
//! not require a NAS. Only compiled into the test harness.
#![cfg(test)]

use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, Mutex};

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use crate::{
    Client, TlsMode,
    client::{Config, Credentials},
    frame,
    pstream::{self, PObject},
};

/// Server-side counters used to assert request/pagination behaviour.
///
/// `list_calls` counts new paginated enumerations (`offset == 0`); `list_pages`
/// counts every `SCMD_LIST` page request.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct ServerStats {
    pub(crate) connections: usize,
    pub(crate) list_calls: usize,
    pub(crate) list_pages: usize,
    pub(crate) other_requests: usize,
}

type Directories = Arc<Mutex<HashMap<String, Vec<PObject>>>>;

pub(crate) struct Fixture {
    pub(crate) port: u16,
    stats: Arc<Mutex<ServerStats>>,
    directories: Directories,
}

impl Fixture {
    pub(crate) fn stats(&self) -> ServerStats {
        *self.stats.lock().unwrap()
    }

    pub(crate) fn reset_stats(&self) {
        *self.stats.lock().unwrap() = ServerStats::default();
    }

    /// Replace the synthetic children of `path`.
    pub(crate) fn set_directory(&self, path: &str, nodes: Vec<PObject>) {
        self.directories
            .lock()
            .unwrap()
            .insert(path.to_owned(), nodes);
    }

    /// Populate `path` with `count` synthetic files named `f00000`, `f00001`, ...
    pub(crate) fn set_files(&self, path: &str, count: u64) {
        self.set_directory(path, (0..count).map(|i| file_node(i, "file")).collect());
    }

    /// Connect a real [`Client`] through the auth/connect handshake.
    pub(crate) async fn connect(&self, max_channels: usize) -> Client {
        Client::connect(
            Config::builder()
                .host("127.0.0.1")
                .port(self.port)
                .tls(TlsMode::None)
                .max_channels(crate::client::MaxChannels(max_channels))
                .device_uuid("fixture-device".to_owned())
                .credentials(Credentials::Password {
                    username: "fixture".into(),
                    password: "fixture".into(),
                    otp: None,
                })
                .build(),
        )
        .await
        .expect("fixture handshake")
    }
}

/// Build a synthetic native node row as returned by the `list` action.
///
/// `index` is unique per directory and drives identity/version fields.
pub(crate) fn file_node(index: u64, file_type: &str) -> PObject {
    pmap! {
        "name" => format!("f{index:05}"),
        "file_hash" => "0123456789abcdef0123456789abcdef",
        "mtime" => 1_700_000_000u64,
        "node_id" => index + 2,
        "sync_id" => 7u64,
        "max_id" => 7u64,
        "file_id" => (index + 100).to_string(),
        "file_size" => 3u64,
        "is_removed" => 0u64,
        "file_type" => file_type,
    }
}

/// Start the fixture listener on an ephemeral loopback port.
pub(crate) async fn start() -> Fixture {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let stats = Arc::new(Mutex::new(ServerStats::default()));
    let directories: Directories = Arc::new(Mutex::new(HashMap::new()));

    let accept_stats = Arc::clone(&stats);
    let accept_directories = Arc::clone(&directories);
    tokio::spawn(async move {
        loop {
            let Ok((socket, _)) = listener.accept().await else {
                break;
            };
            accept_stats.lock().unwrap().connections += 1;
            let stats = Arc::clone(&accept_stats);
            let directories = Arc::clone(&accept_directories);
            tokio::spawn(async move {
                serve_connection(socket, stats, directories).await;
            });
        }
    });

    Fixture {
        port,
        stats,
        directories,
    }
}

async fn serve_connection(
    mut socket: TcpStream,
    stats: Arc<Mutex<ServerStats>>,
    directories: Directories,
) {
    loop {
        let mut header = [0u8; 8];
        if socket.read_exact(&mut header).await.is_err() {
            return;
        }
        let command = header[5];
        let Ok(request) = pstream::decode_from(&mut socket, None).await else {
            return;
        };
        let Some(response) = dispatch(command, &request, &stats, &directories).await else {
            return;
        };
        let mut wire = Vec::new();
        if pstream::encode(&response, &mut wire).is_err() {
            return;
        }
        if socket.write_all(&wire).await.is_err() {
            return;
        }
    }
}

async fn dispatch(
    command: u8,
    request: &PObject,
    stats: &Arc<Mutex<ServerStats>>,
    directories: &Directories,
) -> Option<PObject> {
    match command {
        frame::SCMD_SERVER_INFO => Some(pmap! {
            "database_restore_id" => "fixture-restore",
            "package_version" => pmap! {"build" => 17892u64},
        }),
        frame::SCMD_AUTH => Some(pmap! {"session" => "fixture-session"}),
        frame::SCMD_CONNECT => Some(pmap! {"alive" => 150u64}),
        frame::SCMD_LIST => Some(list_response(request, stats, directories)),
        frame::SCMD_REQUEST => {
            stats.lock().unwrap().other_requests += 1;
            let action = request.get("_action").and_then(PObject::as_str);
            if action == Some("long_poll") {
                // Simulate a blocking long poll: never answer within a test.
                tokio::time::sleep(std::time::Duration::from_secs(3600)).await;
                return None;
            }
            None
        }
        _ => None,
    }
}

fn list_response(
    request: &PObject,
    stats: &Arc<Mutex<ServerStats>>,
    directories: &Directories,
) -> PObject {
    let path = request
        .get("path")
        .and_then(PObject::as_str)
        .unwrap_or("/");
    let criteria = request
        .get("search_criteria")
        .and_then(PObject::as_map)
        .cloned()
        .unwrap_or_default();
    let offset = criteria.get("offset").and_then(PObject::as_int).unwrap_or(0);
    let limit = criteria
        .get("limit")
        .and_then(PObject::as_int)
        .unwrap_or(1000);
    {
        let mut stats = stats.lock().unwrap();
        stats.list_pages += 1;
        if offset == 0 {
            stats.list_calls += 1;
        }
    }
    let entries: Vec<PObject> = directories
        .lock()
        .unwrap()
        .get(path)
        .cloned()
        .unwrap_or_default();
    let total = entries.len() as u64;
    let start = offset.min(total) as usize;
    let end = offset.saturating_add(limit).min(total) as usize;
    let page: Vec<PObject> = entries[start..end].to_vec();
    pmap! {"node_list" => page, "total_count" => total}
}

/// Convenience helper for tests that only need a directory map.
#[allow(dead_code)]
pub(crate) fn map(pairs: Vec<(String, Vec<PObject>)>) -> BTreeMap<String, Vec<PObject>> {
    pairs.into_iter().collect()
}
