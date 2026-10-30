use rustls::pki_types::ServerName;
use std::{
    io,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
    time::{Duration, Instant},
};
use tokio::{
    io::{AsyncRead, AsyncWrite, BufReader, ReadBuf, ReadHalf, WriteHalf},
    net::TcpStream,
};
use tokio_rustls::TlsConnector;

use crate::{
    error::{self, Error, Result},
    frame,
    pstream::{self, PObject},
};

/// TLS configuration for the channel.
#[derive(Debug, Clone, Copy, Default)]
pub enum TlsMode {
    /// No TLS — plain TCP.
    None,
    /// TLS with certificate verification.
    #[default]
    Verified,
    /// TLS accepting any certificate (for self-signed NAS certs).
    Insecure,
}

// Wrapper to unify plain TCP and TLS streams behind AsyncRead + AsyncWrite.
struct DynStream(Box<dyn DynStreamTrait>);

trait DynStreamTrait: AsyncRead + AsyncWrite + Unpin + Send {}
impl<T: AsyncRead + AsyncWrite + Unpin + Send> DynStreamTrait for T {}

impl AsyncRead for DynStream {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        Pin::new(&mut *self.get_mut().0).poll_read(cx, buf)
    }
}

impl AsyncWrite for DynStream {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut *self.get_mut().0).poll_write(cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut *self.get_mut().0).poll_flush(cx)
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut *self.get_mut().0).poll_shutdown(cx)
    }
}

pub struct Channel {
    writer: WriteHalf<DynStream>,
    pub alive_interval: Option<u64>,
    last_active: std::time::Instant,
    reader: BufReader<ReadHalf<DynStream>>,
}

impl Channel {
    /// Sends one immutable snapshot through a bounded buffer. A failed/cancelled
    /// call leaves the channel unusable; its pool guard must not mark it reusable.
    pub(crate) async fn send_binary<R: AsyncRead + Unpin>(
        &mut self,
        obj: &PObject,
        source: &mut R,
    ) -> Result<()> {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        let (prefix, suffix, length) = pstream::binary_parts(obj)?;
        let mut header = Vec::from(frame::MAGIC.to_be_bytes());
        header.extend_from_slice(&[frame::VERSION, frame::SCMD_REQUEST, 0, 0]);
        self.writer.write_all(&header).await?;
        self.writer.write_all(&prefix).await?;
        let mut remaining = length;
        let mut buffer = vec![0u8; 65536];
        while remaining > 0 {
            let take = remaining.min(buffer.len() as u64) as usize;
            let take = crate::flow::allowance(crate::flow::Direction::Upload, take).await;
            let n = source.read(&mut buffer[..take]).await?;
            if n == 0 {
                return Err(Error::Decode("upload snapshot truncated".into()));
            }
            self.writer.write_all(&buffer[..n]).await?;
            crate::flow::advance(crate::flow::Direction::Upload, n);
            remaining -= n as u64;
        }
        let mut extra = [0u8; 1];
        if source.read(&mut extra).await? != 0 {
            return Err(Error::Decode("upload snapshot grew".into()));
        }
        self.writer.write_all(&suffix).await?;
        self.writer.flush().await?;
        self.last_active = Instant::now();
        Ok(())
    }
    /// Connect to a Synology Drive server.
    /// If TLS is requested, performs the `encrypt_channel` upgrade before the TLS handshake.
    pub async fn connect(
        host: &str,
        port: u16,
        tls_server_name: Option<&str>,
        tls: TlsMode,
        tls_config: Option<&Arc<rustls::ClientConfig>>,
    ) -> Result<Self> {
        tracing::debug!(host, port, ?tls, "tcp connect");
        let tcp = tokio::time::timeout(Duration::from_secs(12), TcpStream::connect((host, port)))
            .await
            .map_err(|_| Error::Timeout("TCP connect"))??;
        tcp.set_nodelay(true)?;

        match tls {
            TlsMode::None => Ok(Self::from_stream(tcp)),
            TlsMode::Verified | TlsMode::Insecure => {
                let (read_half, mut write_half) = tokio::io::split(tcp);
                let mut read_half = BufReader::new(read_half);

                tracing::debug!("sending encrypt_channel");
                let req = pmap! { "_action" => "encrypt_channel" };
                frame::send_message(&mut write_half, frame::SCMD_SSL_UPGRADE, &req).await?;
                let resp = tokio::time::timeout(
                    Duration::from_secs(12),
                    pstream::decode_from(&mut read_half, None),
                )
                .await
                .map_err(|_| Error::Timeout("TLS upgrade"))??;
                error::check_server_error(&resp)?;
                tracing::debug!("encrypt_channel accepted");

                let tcp = read_half.into_inner().unsplit(write_half);

                let config = match tls_config {
                    Some(c) => Arc::clone(c),
                    None => Arc::new(build_tls_config(tls)?),
                };
                let identity = tls_server_name.unwrap_or(host);
                let server_name = ServerName::try_from(identity.to_string()).or_else(|_| {
                    identity
                        .parse::<std::net::IpAddr>()
                        .map(|ip| ServerName::IpAddress(ip.into()))
                        .map_err(|_| {
                            Error::InvalidConfig(format!(
                                "'{identity}' is not a valid TLS server name or IP address"
                            ))
                        })
                })?;
                let connector = TlsConnector::from(config);
                tracing::debug!("tls handshake");
                let tls_stream = tokio::time::timeout(
                    Duration::from_secs(12),
                    connector.connect(server_name, tcp),
                )
                .await
                .map_err(|_| Error::Timeout("TLS handshake"))??;
                tracing::debug!("tls complete");

                Ok(Self::from_stream(tls_stream))
            }
        }
    }

    pub(crate) fn from_stream<S: AsyncRead + AsyncWrite + Unpin + Send + 'static>(
        stream: S,
    ) -> Self {
        let dyn_stream = DynStream(Box::new(stream));
        let (read_half, write_half) = tokio::io::split(dyn_stream);
        Self {
            writer: write_half,
            alive_interval: None,
            last_active: Instant::now(),
            reader: BufReader::new(read_half),
        }
    }

    pub async fn send(&mut self, scmd: u8, obj: &PObject) -> Result<()> {
        self.last_active = Instant::now();
        frame::send_message(&mut self.writer, scmd, obj).await
    }

    /// Receive a server response, skipping keep-alives.
    ///
    /// If the response uses `@proto.body-continue` framing, all continuation
    /// frames are consumed and their array fields merged into the first frame
    /// before returning.
    pub async fn recv(&mut self) -> Result<PObject> {
        let mut obj = self.recv_single().await?;
        error::check_server_error(&obj)?;
        let mut frames = 1usize;
        let mut metadata_bytes = metadata_weight(&obj);

        // Reassemble body-continue frames: the server splits large arrays
        // across multiple PObject frames, each with @proto.body-continue = true
        // except the final frame which has body-continue = false.
        while has_body_continue(&obj) {
            frames += 1;
            if frames > 256 {
                return Err(Error::Decode("too many continuation frames".into()));
            }
            let cont = self.recv_single().await?;
            error::check_server_error(&cont)?;
            metadata_bytes += metadata_weight(&cont);
            if metadata_bytes > 32 * 1024 * 1024 {
                return Err(Error::Decode(
                    "continuation metadata budget exceeded".into(),
                ));
            }
            let more = has_body_continue(&cont);
            merge_continuation(&mut obj, cont)?;
            if !more {
                break;
            }
        }

        self.last_active = Instant::now();
        Ok(obj)
    }

    /// Receive a single `PObject` frame, skipping keep-alives.
    async fn recv_single(&mut self) -> Result<PObject> {
        loop {
            let obj = pstream::decode_from(&mut self.reader, None).await?;
            if !pstream::is_keep_alive(&obj) {
                return Ok(obj);
            }
        }
    }

    /// Returns true if this channel has been idle longer than its `alive_interval`.
    #[must_use]
    pub fn is_expired(&self) -> bool {
        self.alive_interval
            .is_some_and(|secs| self.last_active.elapsed() >= Duration::from_secs(secs))
    }

    pub async fn request(&mut self, scmd: u8, obj: &PObject) -> Result<PObject> {
        self.request_with_timeout(scmd, obj, Duration::from_secs(30))
            .await
    }

    pub async fn request_with_timeout(
        &mut self,
        scmd: u8,
        obj: &PObject,
        deadline: Duration,
    ) -> Result<PObject> {
        tokio::time::timeout(deadline, self.exchange(scmd, obj))
            .await
            .map_err(|_| Error::Timeout("native response"))?
    }

    async fn exchange(&mut self, scmd: u8, obj: &PObject) -> Result<PObject> {
        self.send(scmd, obj).await?;
        let response = self.recv().await?;
        error::check_server_error(&response)?;
        Ok(response)
    }

    /// Receive a download response, streaming any inline file data to `dest`.
    ///
    /// The returned `PObject` contains the complete response including any
    /// post-binary metadata (file hash, size, etc.).
    pub async fn recv_download<W: AsyncWrite + Unpin + Send>(
        &mut self,
        dest: &mut W,
    ) -> Result<PObject> {
        let obj = loop {
            let obj = pstream::decode_from(&mut self.reader, Some(dest)).await?;
            if !pstream::is_keep_alive(&obj) {
                break obj;
            }
        };
        error::check_server_error(&obj)?;
        // Download continuation framing needs its own version/offset contract.
        // Do not reuse a connection with unconsumed frames or publish a prefix.
        if has_body_continue(&obj) {
            return Err(Error::Decode(
                "download continuation is not yet supported".into(),
            ));
        }
        self.last_active = Instant::now();
        Ok(obj)
    }
}

/// Check if a response has `@proto.body-continue` set to a truthy value.
fn has_body_continue(obj: &PObject) -> bool {
    obj.get("@proto")
        .and_then(|p| p.get("body-continue"))
        .and_then(PObject::as_int)
        .is_some_and(|v| v != 0)
}

/// Merge array fields from a continuation frame into the base response.
///
/// For each array key in `cont`, appends its elements to the matching array
/// in `base`, or inserts the array if `base` doesn't have that key yet.
/// Repeated scalar metadata must agree; the final frame replaces `@proto`.
fn merge_continuation(base: &mut PObject, cont: PObject) -> Result<()> {
    let PObject::Map(cont_map) = cont else {
        return Err(Error::Decode("continuation is not a map".into()));
    };
    let Some(base_map) = base.as_map_mut() else {
        return Err(Error::Decode("continuation header is not a map".into()));
    };

    for (key, cont_val) in cont_map {
        if key == "@proto" {
            base_map.insert(key, cont_val);
            continue;
        }
        match (base_map.get_mut(&key), &cont_val) {
            (Some(PObject::Array(base_items)), PObject::Array(cont_items)) => {
                base_items.extend(cont_items.iter().cloned())
            }
            (Some(existing), _) if existing != &cont_val => {
                return Err(Error::Decode("conflicting continuation metadata".into()));
            }
            (None, _) => {
                base_map.insert(key, cont_val);
            }
            _ => {}
        }
    }
    Ok(())
}

fn metadata_weight(obj: &PObject) -> usize {
    32 + match obj {
        PObject::Str(s) => s.len(),
        PObject::Array(a) => a.iter().map(metadata_weight).sum(),
        PObject::Map(m) => m.iter().map(|(k, v)| k.len() + metadata_weight(v)).sum(),
        _ => 0,
    }
}

pub fn build_tls_config(tls: TlsMode) -> Result<rustls::ClientConfig> {
    build_tls_config_with_ca(tls, &[])
}

pub fn build_tls_config_with_ca(
    tls: TlsMode,
    extra_ca: &[Vec<u8>],
) -> Result<rustls::ClientConfig> {
    let builder = rustls::ClientConfig::builder_with_provider(Arc::new(
        rustls::crypto::ring::default_provider(),
    ))
    .with_safe_default_protocol_versions()
    .map_err(Error::Tls)?;

    match tls {
        TlsMode::Insecure => Ok(builder
            .dangerous()
            .with_custom_certificate_verifier(Arc::new(NoVerifier))
            .with_no_client_auth()),
        TlsMode::Verified => {
            let certs = rustls_native_certs::load_native_certs();
            if certs.certs.is_empty() && extra_ca.is_empty() {
                let err_msg = if certs.errors.is_empty() {
                    "no CA certificates found in system trust store".into()
                } else {
                    format!(
                        "failed to load CA certificates: {}",
                        certs
                            .errors
                            .iter()
                            .map(ToString::to_string)
                            .collect::<Vec<_>>()
                            .join("; ")
                    )
                };
                return Err(Error::InvalidConfig(err_msg));
            }
            let mut root_store = rustls::RootCertStore::empty();
            for cert in certs.certs {
                root_store.add(cert).ok();
            }
            for cert in extra_ca {
                root_store
                    .add(rustls::pki_types::CertificateDer::from(cert.clone()))
                    .map_err(Error::Tls)?;
            }
            Ok(builder
                .with_root_certificates(root_store)
                .with_no_client_auth())
        }
        TlsMode::None => unreachable!(),
    }
}

/// No-op TLS certificate verifier for self-signed NAS certs.
#[derive(Debug)]
struct NoVerifier;

static NO_VERIFY_SCHEMES: std::sync::LazyLock<Vec<rustls::SignatureScheme>> =
    std::sync::LazyLock::new(|| {
        rustls::crypto::ring::default_provider()
            .signature_verification_algorithms
            .supported_schemes()
    });

impl rustls::client::danger::ServerCertVerifier for NoVerifier {
    fn verify_server_cert(
        &self,
        _end_entity: &rustls::pki_types::CertificateDer<'_>,
        _intermediates: &[rustls::pki_types::CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp_response: &[u8],
        _now: rustls::pki_types::UnixTime,
    ) -> std::result::Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::danger::ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &rustls::pki_types::CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> std::result::Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &rustls::pki_types::CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> std::result::Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        NO_VERIFY_SCHEMES.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build wire bytes for a download response with embedded binary data.
    fn build_download_response(file_data: &[u8]) -> Vec<u8> {
        let mut buf = Vec::new();

        // Outer map start
        buf.push(0x42);

        // "type" => "response"
        pstream::encode(&PObject::Str("type".into()), &mut buf).unwrap();
        pstream::encode(&PObject::Str("response".into()), &mut buf).unwrap();

        // "sync_id" => 100
        pstream::encode(&PObject::Str("sync_id".into()), &mut buf).unwrap();
        pstream::encode(&PObject::Integer(100), &mut buf).unwrap();

        // "file" => { "data" => <binary>, "hash" => "abc123", "size" => N }
        pstream::encode(&PObject::Str("file".into()), &mut buf).unwrap();
        buf.push(0x42); // inner map start

        pstream::encode(&PObject::Str("data".into()), &mut buf).unwrap();
        // Binary tag + u64 BE length + raw data
        buf.push(0x30);
        buf.extend_from_slice(&(file_data.len() as u64).to_be_bytes());
        buf.extend_from_slice(file_data);

        pstream::encode(&PObject::Str("hash".into()), &mut buf).unwrap();
        pstream::encode(&PObject::Str("abc123".into()), &mut buf).unwrap();

        pstream::encode(&PObject::Str("size".into()), &mut buf).unwrap();
        pstream::encode(&PObject::Integer(file_data.len() as u64), &mut buf).unwrap();

        buf.push(0x40); // inner map end
        buf.push(0x40); // outer map end

        buf
    }

    #[tokio::test]
    async fn recv_download_streams_binary_data() {
        let file_data = b"hello this is file content for testing";
        let response_bytes = build_download_response(file_data);

        let (client, mut server) = tokio::io::duplex(65536);
        tokio::spawn(async move {
            use tokio::io::AsyncWriteExt;
            server.write_all(&response_bytes).await.unwrap();
            server.shutdown().await.unwrap();
        });

        let mut ch = Channel::from_stream(client);
        let mut dest = Vec::new();
        let obj = ch.recv_download(&mut dest).await.unwrap();

        assert_eq!(dest, file_data);
        assert_eq!(obj.get("sync_id").and_then(PObject::as_int), Some(100));
        // Post-binary fields are preserved
        let file = obj.get("file").unwrap();
        assert_eq!(file.get("hash").and_then(PObject::as_str), Some("abc123"));
        assert_eq!(
            file.get("size").and_then(PObject::as_int),
            Some(file_data.len() as u64)
        );
    }

    #[tokio::test]
    async fn recv_download_skips_keepalives() {
        let file_data = b"the real file data after keepalives";

        let mut stream_bytes = Vec::new();
        let ka = pmap! { "type" => "keep_alive" };
        pstream::encode(&ka, &mut stream_bytes).unwrap();
        pstream::encode(&ka, &mut stream_bytes).unwrap();
        stream_bytes.extend_from_slice(&build_download_response(file_data));

        let (client, mut server) = tokio::io::duplex(65536);
        tokio::spawn(async move {
            use tokio::io::AsyncWriteExt;
            server.write_all(&stream_bytes).await.unwrap();
            server.shutdown().await.unwrap();
        });

        let mut ch = Channel::from_stream(client);
        let mut dest = Vec::new();
        let obj = ch.recv_download(&mut dest).await.unwrap();

        assert_eq!(dest, file_data);
        assert_eq!(obj.get("sync_id").and_then(PObject::as_int), Some(100));
    }

    #[tokio::test]
    async fn recv_download_no_binary_data() {
        let response = pmap! {
            "type" => "response",
            "sync_id" => 100u64,
            "file" => pmap! {
                "refer" => true,
                "hash" => "abc123",
            },
        };
        let mut response_bytes = Vec::new();
        pstream::encode(&response, &mut response_bytes).unwrap();

        let (client, mut server) = tokio::io::duplex(65536);
        tokio::spawn(async move {
            use tokio::io::AsyncWriteExt;
            server.write_all(&response_bytes).await.unwrap();
            server.shutdown().await.unwrap();
        });

        let mut ch = Channel::from_stream(client);
        let mut dest = Vec::new();
        let obj = ch.recv_download(&mut dest).await.unwrap();

        assert!(dest.is_empty());
        assert_eq!(obj.get("sync_id").and_then(PObject::as_int), Some(100));
    }

    /// Build a download response using `BinaryEx` (0x43) instead of `Binary` (0x30).
    /// `BinaryEx` wraps the data in an extra map: `{ "binary": <0x30 data>, "send_hash": "..." }`
    fn build_binary_ex_download_response(file_data: &[u8]) -> Vec<u8> {
        let mut buf = Vec::new();

        buf.push(0x42); // outer map

        pstream::encode(&PObject::Str("sync_id".into()), &mut buf).unwrap();
        pstream::encode(&PObject::Integer(200), &mut buf).unwrap();

        pstream::encode(&PObject::Str("file".into()), &mut buf).unwrap();
        buf.push(0x42); // file map

        pstream::encode(&PObject::Str("data".into()), &mut buf).unwrap();
        // BinaryEx: 0x43 { "binary": 0x30 <len> <data>, "send_hash": "abc" } 0x40
        buf.push(0x43);
        pstream::encode(&PObject::Str("binary".into()), &mut buf).unwrap();
        buf.push(0x30);
        buf.extend_from_slice(&(file_data.len() as u64).to_be_bytes());
        buf.extend_from_slice(file_data);
        pstream::encode(&PObject::Str("send_hash".into()), &mut buf).unwrap();
        pstream::encode(&PObject::Str("deadbeef".into()), &mut buf).unwrap();
        buf.push(0x40); // end binary_ex

        pstream::encode(&PObject::Str("hash".into()), &mut buf).unwrap();
        pstream::encode(&PObject::Str("filehash".into()), &mut buf).unwrap();

        buf.push(0x40); // end file map
        buf.push(0x40); // end outer map

        buf
    }

    #[tokio::test]
    async fn recv_download_binary_ex_drains_correctly() {
        let file_data = b"binary_ex file content";
        let mut stream = build_binary_ex_download_response(file_data);

        // Append a second PObject right after the download response.
        // If the drain depth is wrong, this will be corrupted or unreadable.
        let followup = pmap! { "type" => "followup", "value" => 42u64 };
        pstream::encode(&followup, &mut stream).unwrap();

        let (client, mut server) = tokio::io::duplex(65536);
        tokio::spawn(async move {
            use tokio::io::AsyncWriteExt;
            server.write_all(&stream).await.unwrap();
            server.shutdown().await.unwrap();
        });

        let mut ch = Channel::from_stream(client);

        // First: download with BinaryEx
        let mut dest = Vec::new();
        let obj = ch.recv_download(&mut dest).await.unwrap();
        assert_eq!(dest, file_data);
        assert_eq!(obj.get("sync_id").and_then(PObject::as_int), Some(200));
        // Post-binary fields are preserved
        let file = obj.get("file").unwrap();
        assert_eq!(file.get("hash").and_then(PObject::as_str), Some("filehash"));
        // BinaryEx send_hash is captured even though it appears after "binary" on the wire
        match file.get("data").unwrap() {
            PObject::BinaryEx { send_hash, .. } => assert_eq!(send_hash, "deadbeef"),
            other => panic!("expected BinaryEx, got {other:?}"),
        }

        // Second: read the followup message on the same channel.
        // This fails if BinaryEx left a trailing TAG_END unread.
        let next = pstream::decode_from(&mut ch.reader, None).await.unwrap();
        assert_eq!(next.get("type").and_then(PObject::as_str), Some("followup"));
        assert_eq!(next.get("value").and_then(PObject::as_int), Some(42));
    }

    #[tokio::test]
    async fn recv_download_binary_drains_correctly() {
        let file_data = b"plain binary content";
        let mut stream = build_download_response(file_data);

        // Append a followup message
        let followup = pmap! { "type" => "followup", "value" => 99u64 };
        pstream::encode(&followup, &mut stream).unwrap();

        let (client, mut server) = tokio::io::duplex(65536);
        tokio::spawn(async move {
            use tokio::io::AsyncWriteExt;
            server.write_all(&stream).await.unwrap();
            server.shutdown().await.unwrap();
        });

        let mut ch = Channel::from_stream(client);

        let mut dest = Vec::new();
        let obj = ch.recv_download(&mut dest).await.unwrap();
        assert_eq!(dest, file_data);
        assert_eq!(obj.get("sync_id").and_then(PObject::as_int), Some(100));

        // Channel should be clean — followup readable
        let next = pstream::decode_from(&mut ch.reader, None).await.unwrap();
        assert_eq!(next.get("type").and_then(PObject::as_str), Some("followup"));
        assert_eq!(next.get("value").and_then(PObject::as_int), Some(99));
    }

    #[tokio::test]
    async fn recv_reassembles_body_continue_frames() {
        // Frame 1: header with body-continue=true and first batch of items
        let frame1 = pmap! {
            "@proto" => pmap! {
                "type" => "header",
                "body-continue" => true,
            },
            "node_list" => PObject::Array(vec![
                pmap! { "name" => "file1.txt" },
                pmap! { "name" => "file2.txt" },
            ]),
        };
        // Frame 2: continuation with body-continue=true
        let frame2 = pmap! {
            "@proto" => pmap! {
                "type" => "header",
                "body-continue" => true,
            },
            "node_list" => PObject::Array(vec![
                pmap! { "name" => "file3.txt" },
            ]),
        };
        // Frame 3: final with body-continue=false
        let frame3 = pmap! {
            "@proto" => pmap! {
                "type" => "header",
                "body-continue" => false,
            },
            "node_list" => PObject::Array(vec![
                pmap! { "name" => "file4.txt" },
            ]),
        };

        let mut wire = Vec::new();
        pstream::encode(&frame1, &mut wire).unwrap();
        pstream::encode(&frame2, &mut wire).unwrap();
        pstream::encode(&frame3, &mut wire).unwrap();

        let (client, mut server) = tokio::io::duplex(65536);
        tokio::spawn(async move {
            use tokio::io::AsyncWriteExt;
            server.write_all(&wire).await.unwrap();
            server.shutdown().await.unwrap();
        });

        let mut ch = Channel::from_stream(client);
        let result = ch.recv().await.unwrap();

        let node_list = result.get("node_list").and_then(PObject::as_array).unwrap();
        assert_eq!(node_list.len(), 4);
        assert_eq!(
            node_list[0].get("name").and_then(PObject::as_str),
            Some("file1.txt")
        );
        assert_eq!(
            node_list[1].get("name").and_then(PObject::as_str),
            Some("file2.txt")
        );
        assert_eq!(
            node_list[2].get("name").and_then(PObject::as_str),
            Some("file3.txt")
        );
        assert_eq!(
            node_list[3].get("name").and_then(PObject::as_str),
            Some("file4.txt")
        );
    }

    #[tokio::test]
    async fn recv_returns_single_frame_without_body_continue() {
        let frame = pmap! {
            "@proto" => pmap! {
                "type" => "header",
                "body-continue" => false,
            },
            "node_list" => PObject::Array(vec![
                pmap! { "name" => "only.txt" },
            ]),
        };

        let mut wire = Vec::new();
        pstream::encode(&frame, &mut wire).unwrap();

        let (client, mut server) = tokio::io::duplex(65536);
        tokio::spawn(async move {
            use tokio::io::AsyncWriteExt;
            server.write_all(&wire).await.unwrap();
            server.shutdown().await.unwrap();
        });

        let mut ch = Channel::from_stream(client);
        let result = ch.recv().await.unwrap();

        let node_list = result.get("node_list").and_then(PObject::as_array).unwrap();
        assert_eq!(node_list.len(), 1);
        assert_eq!(
            node_list[0].get("name").and_then(PObject::as_str),
            Some("only.txt")
        );
    }

    #[tokio::test]
    async fn recv_merges_arrays_absent_from_header_frame() {
        // Header frame has body-continue but no node_list.
        let header = pmap! {
            "@proto" => pmap! {
                "type" => "header",
                "body-continue" => true,
            },
            "action" => "list_sync_to_device",
        };
        // Continuation introduces node_list for the first time.
        let body1 = pmap! {
            "@proto" => pmap! { "body-continue" => true },
            "node_list" => PObject::Array(vec![
                pmap! { "name" => "a.txt" },
            ]),
        };
        let body2 = pmap! {
            "@proto" => pmap! { "body-continue" => false },
            "node_list" => PObject::Array(vec![
                pmap! { "name" => "b.txt" },
            ]),
        };

        let mut wire = Vec::new();
        pstream::encode(&header, &mut wire).unwrap();
        pstream::encode(&body1, &mut wire).unwrap();
        pstream::encode(&body2, &mut wire).unwrap();

        let (client, mut server) = tokio::io::duplex(65536);
        tokio::spawn(async move {
            use tokio::io::AsyncWriteExt;
            server.write_all(&wire).await.unwrap();
            server.shutdown().await.unwrap();
        });

        let mut ch = Channel::from_stream(client);
        let result = ch.recv().await.unwrap();

        let node_list = result.get("node_list").and_then(PObject::as_array).unwrap();
        assert_eq!(node_list.len(), 2);
        assert_eq!(
            node_list[0].get("name").and_then(PObject::as_str),
            Some("a.txt")
        );
        assert_eq!(
            node_list[1].get("name").and_then(PObject::as_str),
            Some("b.txt")
        );
        // Header-only fields should still be present.
        assert_eq!(
            result.get("action").and_then(PObject::as_str),
            Some("list_sync_to_device")
        );
    }
}

#[cfg(test)]
mod continuation_regressions {
    use super::*;
    use tokio::io::AsyncWriteExt;
    async fn channel(objects: &[PObject]) -> Channel {
        let mut wire = Vec::new();
        for o in objects {
            pstream::encode(o, &mut wire).unwrap();
        }
        let (a, mut b) = tokio::io::duplex(65536);
        tokio::spawn(async move {
            b.write_all(&wire).await.unwrap();
        });
        Channel::from_stream(a)
    }
    #[tokio::test]
    async fn error_in_final_continuation_is_not_discarded() {
        let mut ch = channel(&[
            pmap! {"@proto"=>pmap!{"body-continue"=>true},"node_list"=>PObject::Array(vec![])},
            pmap! {"error"=>pmap!{"code"=>0x4001u64,"reason"=>"expired"}},
        ])
        .await;
        assert!(matches!(ch.recv().await, Err(Error::SessionInvalid)));
    }
    #[tokio::test]
    async fn changed_metadata_across_continuations_is_rejected() {
        let mut ch = channel(&[
            pmap! {"@proto"=>pmap!{"body-continue"=>true},"cursor"=>"a"},
            pmap! {"@proto"=>pmap!{"body-continue"=>false},"cursor"=>"b"},
        ])
        .await;
        assert!(ch.recv().await.is_err());
    }
    #[tokio::test]
    async fn unsupported_download_continuation_fails_closed() {
        let mut ch = channel(&[pmap! {"@proto"=>pmap!{"body-continue"=>true}}]).await;
        assert!(ch.recv_download(&mut Vec::new()).await.is_err());
    }
    #[test]
    fn invalid_custom_ca_is_rejected() {
        assert!(build_tls_config_with_ca(TlsMode::Verified, &[vec![0, 1, 2]]).is_err());
    }
}
