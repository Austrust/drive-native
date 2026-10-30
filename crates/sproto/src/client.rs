use std::{
    collections::BTreeMap,
    path::Path,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};
use tokio::io::AsyncWrite;
use uuid::Uuid;

use crate::{
    Error, actions,
    channel::{self, Channel, TlsMode},
    error::Result,
    frame,
    pool::{self, Pool},
    pstream::PObject,
};

/// An authenticated client for the Synology Drive protocol.
///
/// Created via [`Client::connect`]. Cloning is cheap (the connection pool is shared).
#[derive(Clone)]
pub struct Client {
    pub(crate) pool: Arc<Pool>,
    agent: PObject,
    session: String,
    pub(crate) server_build: u64,
    restore_identity: String,
}

/// Configuration for connecting to a Synology Drive server.
///
/// Use [`Config::builder()`] to construct. `host` and `credentials` are required;
/// all other fields have sensible defaults.
///
/// ```rust
/// use sproto::{Client, TlsMode};
/// use sproto::client::{Config, Credentials};
///
/// let config = Config::builder()
///     .host("192.168.1.100")
///     .port(6690)
///     .tls(TlsMode::Verified)
///     .credentials(Credentials::Password {
///         username: "admin".to_string(),
///         password: "secret".to_string(),
///         otp: None,
///     })
///     .build();
/// ```
#[derive(macon::Builder)]
pub struct Config {
    /// NAS hostname or IP address.
    #[builder(Default=!)]
    pub host: String,

    /// Certificate identity, when connecting through an IP/VPN address.
    /// Never disables chain or hostname verification.
    pub tls_server_name: Option<String>,

    /// Port number (default: 6690).
    #[builder(Default)]
    pub port: Port,

    /// TLS mode.
    ///
    /// For a private CA, provide `trusted_ca_der`; certificate names remain verified.
    #[builder(Default)]
    pub tls: TlsMode,

    /// Additional trusted CA certificates in DER encoding, scoped to this client.
    #[builder(Default)]
    pub trusted_ca_der: Vec<Vec<u8>>,

    /// Authentication credentials.
    pub credentials: Credentials,

    /// This device's UUID. Auto-generated if `None`.
    pub device_uuid: Option<String>,

    /// Maximum number of concurrent connections in the pool.
    #[builder(Default)]
    pub max_channels: MaxChannels,
}

/// Authentication credentials for a Synology Drive server.
///
/// ```rust
/// use sproto::client::Credentials;
///
/// // Username/password (with optional 2FA)
/// let creds = Credentials::Password {
///     username: "admin".to_string(),
///     password: "secret".to_string(),
///     otp: Some("123456".to_string()),
/// };
///
/// // A session previously established by this client (never read official credentials).
/// let creds = Credentials::Session {
///     session: "your-session-token".to_string(),
///     restore_id: "your-restore-id".to_string(),
/// };
/// ```
pub enum Credentials {
    /// Reuse an existing session token.
    Session { session: String, restore_id: String },
    /// Authenticate with username/password.
    Password {
        username: String,
        password: String,
        otp: Option<String>,
    },
}

/// Default port for the Synology Drive protocol.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Port(pub u16);

impl Default for Port {
    fn default() -> Self {
        Self(6690)
    }
}

impl From<u16> for Port {
    fn from(v: u16) -> Self {
        Self(v)
    }
}

/// Maximum number of concurrent pool channels.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MaxChannels(pub usize);

impl Default for MaxChannels {
    fn default() -> Self {
        Self(5)
    }
}

impl From<usize> for MaxChannels {
    fn from(v: usize) -> Self {
        Self(v)
    }
}

impl Client {
    /// Connect to a Synology Drive server and authenticate.
    #[tracing::instrument(skip_all, fields(host = %config.host, port = config.port.0))]
    pub async fn connect(config: Config) -> Result<Self> {
        let device_uuid = config
            .device_uuid
            .unwrap_or_else(|| Uuid::new_v4().to_string());

        let tls_config = match config.tls {
            TlsMode::None => None,
            _ => Some(Arc::new(channel::build_tls_config_with_ca(
                config.tls,
                &config.trusted_ca_der,
            )?)),
        };

        tracing::debug!("connecting auth channel");
        let mut ch = Channel::connect(
            &config.host,
            config.port.0,
            config.tls_server_name.as_deref(),
            config.tls,
            tls_config.as_ref(),
        )
        .await?;

        // AuthSession in the official client uses ProtocolFactory's envelope.
        // Without agent.version this server returns 20481 even when
        // client_version is present. Obtain restore identity before login.
        let preauth = ch
            .request(
                frame::SCMD_SERVER_INFO,
                &pmap! {
                    "_action" => "query_server_info", "get_all" => true,
                },
            )
            .await?;
        let initial_restore = preauth
            .get("database_restore_id")
            .and_then(PObject::as_str)
            .filter(|s| !s.is_empty())
            .ok_or_else(|| Error::Decode("missing preauth restore identity".into()))?;
        let auth_agent = crate::pool::agent_map(&device_uuid, initial_restore);

        let (session, supplied_restore_id) = match &config.credentials {
            Credentials::Password {
                username,
                password,
                otp,
            } => {
                let req = pmap! {
                    "@proto" => protocol_header(),
                    "_agent" => auth_agent,
                    "_action" => "auth",
                    "dry_run" => false,
                    "renew_session" => "",
                    "client_type" => "drive",
                    "username" => username.as_str(),
                    "password" => password.as_str(),
                    "client" => "SynologyDriveClient",
                    "otp" => otp.as_deref().unwrap_or(""),
                    "client_version" => crate::frame::CLIENT_VERSION,
                };
                tracing::debug!("authenticating");
                let resp = ch.request(frame::SCMD_AUTH, &req).await?;
                tracing::debug!("authenticated");
                let session = resp
                    .get("session")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| Error::Decode("missing session in auth response".into()))?
                    .to_string();

                (session, None)
            }
            Credentials::Session {
                session,
                restore_id,
            } => (session.clone(), Some(restore_id.as_str())),
        };

        let info_req = pmap! {
            "_action" => "query_server_info",
            "get_all" => true,
            "session" => session.as_str(),
        };
        tracing::debug!("querying server info");
        let info_resp = ch.request(frame::SCMD_SERVER_INFO, &info_req).await?;
        tracing::debug!("server info received");

        let restore_id = info_resp
            .get("database_restore_id")
            .and_then(|v| v.as_str())
            .or(supplied_restore_id)
            .filter(|s| !s.is_empty())
            .ok_or_else(|| Error::Decode("missing database_restore_id".into()))?
            .to_string();

        let server_build = info_resp
            .get("package_version")
            .and_then(|v| v.get("build"))
            .and_then(PObject::as_int)
            .ok_or_else(|| Error::Decode("missing package_version.build".into()))?;

        let agent = crate::pool::agent_map(&device_uuid, &restore_id);

        let pool = Arc::new(Pool::new(pool::Config {
            tls_config,
            restore_id: restore_id.clone(),
            device_uuid,
            tls: config.tls,
            host: config.host,
            tls_server_name: config.tls_server_name,
            port: config.port.0,
            session: session.clone(),
            max_channels: config.max_channels.0,
        })?);

        tracing::debug!("donating auth channel to pool");
        pool.donate(ch).await?;
        tracing::debug!("client ready");

        Ok(Self {
            pool,
            agent,
            session,
            server_build,
            restore_identity: restore_id,
        })
    }

    /// One-way identity for matching server echoes across reconnects. Never persist the token.
    pub fn commit_fingerprint(&self) -> String {
        session_fingerprint(&self.session)
    }

    /// Non-secret server build used for capability selection.
    pub fn server_build(&self) -> u64 {
        self.server_build
    }

    /// Compare with the persisted identity before resuming an event cursor.
    pub fn restore_identity(&self) -> &str {
        &self.restore_identity
    }

    /// Pre-create pool channels in parallel.
    ///
    /// Call this before bulk operations to avoid per-channel connection overhead.
    /// `count` is the number of channels to create (capped at the pool's max).
    pub async fn warm_pool(&self, count: usize) -> Result<()> {
        self.pool.warm(count).await
    }

    #[must_use]
    /// Build a request `PObject` with the standard envelope fields.
    pub(crate) fn build_request(&self, action: &str, fields: PObject) -> PObject {
        let mut map = BTreeMap::new();

        map.insert("@proto".into(), protocol_header());

        map.insert("_agent".into(), self.agent.clone());
        map.insert("_action".into(), PObject::Str(action.to_string()));
        map.insert("session".into(), PObject::Str(self.session.clone()));

        if let PObject::Map(mut extra) = fields {
            map.append(&mut extra);
        }

        PObject::Map(map)
    }

    /// List available shares/views.
    pub async fn list_shares(&self) -> Result<Vec<actions::list::ShareInfo>> {
        let mut guard = self.pool.acquire().await?;
        let result = actions::list::list_team_folder(guard.channel(), self).await;
        guard.poison_on_err(result)
    }

    /// List directory contents using the `list` action (paginated).
    pub async fn list_dir(&self, view_id: u64, path: &str) -> Result<Vec<actions::list::NodeInfo>> {
        let mut guard = self.pool.acquire().await?;
        let result = actions::list::list(guard.channel(), self, view_id, path).await;
        guard.poison_on_err(result)
    }

    /// List directory contents for sync using `list_sync_to_device`.
    /// Only works with team folder view IDs from `list_shares`.
    ///
    /// Pass a `cursor` from a previous call to enable the server's change-detection
    /// fast path. If the directory is unchanged, the server returns the same cursor
    /// with no node list, skipping the full transfer.
    pub async fn list_sync(
        &self,
        view_id: u64,
        path: &str,
        cursor: Option<&str>,
    ) -> Result<actions::list::DirectoryListing> {
        let mut guard = self.pool.acquire().await?;
        let result =
            actions::list::list_sync_to_device(guard.channel(), self, view_id, path, cursor).await;
        guard.poison_on_err(result)
    }

    /// Download a single file by its `file_id` to a local path.
    ///
    /// The `file_id` comes from a prior `list_dir` call's `NodeInfo`.
    pub async fn download(&self, file_id: &str, dest: &Path) -> Result<()> {
        let mut guard = self.pool.acquire().await?;
        let result = actions::download::download(guard.channel(), self, file_id, dest).await;

        guard.poison_on_err(result)
    }

    /// Download a single file by its `file_id` to any `AsyncWrite` sink.
    pub async fn download_to<W: AsyncWrite + Unpin + Send>(
        &self,
        file_id: &str,
        dest: &mut W,
    ) -> Result<()> {
        let mut guard = self.pool.acquire().await?;
        let result = actions::download::download_to(guard.channel(), self, file_id, dest).await;

        guard.poison_on_err(result.map(|_| ()))
    }
}

fn protocol_header() -> PObject {
    pmap! {
        "type" => "header", "body-continue" => false,
        "date" => unix_timestamp(),
        "version" => pmap! { "major" => 7u64, "minor" => 0u64 },
    }
}

#[cfg(test)]
mod auth_regression {
    use super::*;
    use tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        net::TcpListener,
    };
    #[tokio::test]
    async fn auth_uses_preauth_restore_and_complete_agent_envelope() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let server = tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            for command in [
                frame::SCMD_SERVER_INFO,
                frame::SCMD_AUTH,
                frame::SCMD_SERVER_INFO,
                frame::SCMD_CONNECT,
            ] {
                let mut header = [0u8; 8];
                socket.read_exact(&mut header).await.unwrap();
                assert_eq!(header[5], command);
                let req = crate::pstream::decode_from(&mut socket, None)
                    .await
                    .unwrap();
                let response = if command == frame::SCMD_AUTH {
                    let agent = req.get("agent").unwrap();
                    assert_eq!(
                        agent.get("platform").and_then(PObject::as_str),
                        Some("linux")
                    );
                    assert_eq!(
                        agent.get("restore_id").and_then(PObject::as_str),
                        Some("fixture-restore")
                    );
                    assert_eq!(
                        agent.get("device_uuid").and_then(PObject::as_str),
                        Some("fixture-device")
                    );
                    assert_eq!(
                        agent
                            .get("version")
                            .unwrap()
                            .get("build")
                            .and_then(PObject::as_int),
                        Some(frame::CLIENT_BUILD)
                    );
                    assert!(req.get("@proto").is_some());
                    pmap! {"session"=>"fixture-session"}
                } else if command == frame::SCMD_SERVER_INFO {
                    pmap! {"database_restore_id"=>"fixture-restore","package_version"=>pmap!{"build"=>17892u64}}
                } else {
                    pmap! {"alive"=>150u64}
                };
                let mut wire = Vec::new();
                crate::pstream::encode(&response, &mut wire).unwrap();
                socket.write_all(&wire).await.unwrap();
            }
        });
        let client = Client::connect(
            Config::builder()
                .host("127.0.0.1")
                .port(port)
                .tls(TlsMode::None)
                .device_uuid("fixture-device".to_owned())
                .credentials(Credentials::Password {
                    username: "fixture".into(),
                    password: "fixture".into(),
                    otp: None,
                })
                .build(),
        )
        .await
        .unwrap();
        assert_eq!(client.restore_identity(), "fixture-restore");
        server.await.unwrap();
    }
}

fn unix_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

/// Domain-separated digest; not usable to authenticate a connection.
pub(crate) fn session_fingerprint(value: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hash = Sha256::new();
    hash.update(b"drive-native-committer-v1\0");
    hash.update(value.as_bytes());
    format!("{:x}", hash.finalize())
}
