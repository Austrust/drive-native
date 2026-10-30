//! Test-only loopback TLS boundary. Native payloads are relayed in memory, never logged.
use crate::{Error, Result, frame, pstream};
use rustls::pki_types::{CertificateDer, PrivateKeyDer, ServerName, pem::PemObject};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
};
pub struct TlsTestRelay {
    pub accepted: Arc<AtomicUsize>,
    task: tokio::task::JoinHandle<()>,
}
impl Drop for TlsTestRelay {
    fn drop(&mut self) {
        self.task.abort();
    }
}
pub async fn start(listener: TcpListener, certificate: &str, key: &str) -> Result<TlsTestRelay> {
    if !listener.local_addr()?.ip().is_loopback() {
        return Err(Error::InvalidConfig("test relay requires loopback".into()));
    }
    let cert = CertificateDer::from_pem_slice(certificate.as_bytes())
        .map_err(|_| Error::InvalidConfig("test certificate".into()))?;
    let key = PrivateKeyDer::from_pem_slice(key.as_bytes())
        .map_err(|_| Error::InvalidConfig("test key".into()))?;
    let config = rustls::ServerConfig::builder_with_provider(Arc::new(
        rustls::crypto::ring::default_provider(),
    ))
    .with_safe_default_protocol_versions()?
    .with_no_client_auth()
    .with_single_cert(vec![cert], key)?;
    let server = Arc::new(config);
    let upstream = Arc::new(crate::channel::build_tls_config(crate::TlsMode::Verified)?);
    let accepted = Arc::new(AtomicUsize::new(0));
    let counter = accepted.clone();
    let task = tokio::spawn(async move {
        let mut connections = tokio::task::JoinSet::new();
        loop {
            tokio::select! {
                result=listener.accept()=>{
                    let Ok((tcp,_))=result else{break};
                    let server=server.clone();let upstream=upstream.clone();let counter=counter.clone();
                    connections.spawn(async move {if let Err(error)=relay(tcp,server,upstream,counter).await { eprintln!("test TLS relay: {error}"); }});
                }
                _=connections.join_next(), if !connections.is_empty()=>{}
            }
        }
    });
    Ok(TlsTestRelay { accepted, task })
}
async fn relay(
    front: TcpStream,
    server: Arc<rustls::ServerConfig>,
    upstream: Arc<rustls::ClientConfig>,
    counter: Arc<AtomicUsize>,
) -> Result<()> {
    let mut reader = BufReader::new(front);
    let mut header = [0u8; 8];
    reader.read_exact(&mut header).await?;
    let mut expected = frame::MAGIC.to_be_bytes().to_vec();
    expected.extend_from_slice(&[frame::VERSION, frame::SCMD_SSL_UPGRADE, 0, 0]);
    if header != expected.as_slice() {
        return Err(Error::InvalidConfig(
            "test relay requires native TLS upgrade".into(),
        ));
    }
    let request = pstream::decode_from(&mut reader, None).await?;
    if request.get("action").and_then(pstream::PObject::as_str) != Some("encrypt_channel") {
        return Err(Error::InvalidConfig(
            "test relay requires encrypt_channel".into(),
        ));
    }
    if !reader.buffer().is_empty() {
        return Err(Error::InvalidConfig("data before TLS".into()));
    }
    let mut front = reader.into_inner();
    let mut back = TcpStream::connect(("192.0.2.10", 6690)).await?;
    frame::send_message(&mut back, frame::SCMD_SSL_UPGRADE, &request).await?;
    let mut reader = BufReader::new(back);
    let response = pstream::decode_from(&mut reader, None).await?;
    crate::error::check_server_error(&response)?;
    if !reader.buffer().is_empty() {
        return Err(Error::InvalidConfig("unexpected pre-TLS bytes".into()));
    }
    let back = reader.into_inner();
    let mut bytes = Vec::new();
    pstream::encode(&response, &mut bytes)?;
    front.write_all(&bytes).await?;
    front.flush().await?;
    let front = tokio_rustls::TlsAcceptor::from(server).accept(front);
    let back = tokio_rustls::TlsConnector::from(upstream).connect(
        ServerName::try_from("nas.example.invalid").unwrap().to_owned(),
        back,
    );
    let (mut front, mut back) = tokio::try_join!(front, back)?;
    counter.fetch_add(1, Ordering::Relaxed);
    tokio::io::copy_bidirectional(&mut front, &mut back).await?;
    Ok(())
}
