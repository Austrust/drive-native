//! Explicit extra trust anchors, scoped to one client. Never disables name checks.
use crate::{Error, Result};
use rustls::pki_types::{CertificateDer, pem::PemObject};
use std::collections::HashSet;
pub const MAX_TRUST_PEM_BYTES: usize = 32768;
pub fn parse_trusted_pem(certificates: &[String]) -> Result<Vec<Vec<u8>>> {
    if certificates.len() > 8
        || certificates.iter().map(String::len).sum::<usize>() > MAX_TRUST_PEM_BYTES
    {
        return Err(Error::InvalidConfig(
            "extra trust store exceeds limit".into(),
        ));
    }
    let mut result = Vec::new();
    let mut seen = HashSet::new();
    let mut roots = rustls::RootCertStore::empty();
    for pem in certificates {
        let text = pem.trim();
        if !text.starts_with("-----BEGIN CERTIFICATE-----")
            || !text.ends_with("-----END CERTIFICATE-----")
            || text.matches("-----BEGIN").count() != 1
            || text.matches("-----END").count() != 1
        {
            return Err(Error::InvalidConfig(
                "expected one public PEM certificate per entry".into(),
            ));
        }
        let mut parsed = CertificateDer::pem_slice_iter(text.as_bytes());
        let cert = parsed
            .next()
            .ok_or_else(|| Error::InvalidConfig("certificate absent".into()))?
            .map_err(|_| Error::InvalidConfig("invalid PEM certificate".into()))?;
        if parsed.next().is_some() {
            return Err(Error::InvalidConfig(
                "multiple certificates in entry".into(),
            ));
        }
        roots
            .add(cert.clone())
            .map_err(|_| Error::InvalidConfig("invalid X.509 certificate".into()))?;
        let der = cert.as_ref().to_vec();
        if !seen.insert(der.clone()) {
            return Err(Error::InvalidConfig("duplicate trust certificate".into()));
        }
        result.push(der);
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    const CA: &str = include_str!("../tests/fixtures/trust/ca.pem");
    const SERVER: &str = include_str!("../tests/fixtures/trust/server.pem");
    const KEY: &str = include_str!("../tests/fixtures/trust/server-key.pem");
    #[test]
    fn extra_trust_parser_accepts_public_cert_and_rejects_private_keys_or_junk() {
        assert!(parse_trusted_pem(&[]).unwrap().is_empty());
        assert_eq!(parse_trusted_pem(&[CA.into()]).unwrap().len(), 1);
        for entries in [
            vec![CA.into(), CA.into()],
            vec![KEY.into()],
            vec![format!("{CA}{KEY}")],
            vec![format!("{CA}{SERVER}")],
            vec!["-----BEGIN CERTIFICATE-----\nYWJj\n-----END CERTIFICATE-----".into()],
            vec!["x".repeat(MAX_TRUST_PEM_BYTES + 1)],
            vec![CA.into(); 9],
        ] {
            assert!(parse_trusted_pem(&entries).is_err());
        }
    }
    async fn handshake(extra: &[String], name: &str) -> bool {
        handshake_at(extra, name, false).await
    }
    async fn handshake_at(extra: &[String], name: &str, expired: bool) -> bool {
        use rustls::pki_types::{PrivateKeyDer, ServerName};
        use std::sync::Arc;
        let cert = CertificateDer::from_pem_slice(SERVER.as_bytes()).unwrap();
        let key = PrivateKeyDer::from_pem_slice(KEY.as_bytes()).unwrap();
        let config = rustls::ServerConfig::builder_with_provider(Arc::new(
            rustls::crypto::ring::default_provider(),
        ))
        .with_safe_default_protocol_versions()
        .unwrap()
        .with_no_client_auth()
        .with_single_cert(vec![cert], key)
        .unwrap();
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            let (tcp, _) = listener.accept().await.unwrap();
            let _ = tokio_rustls::TlsAcceptor::from(Arc::new(config))
                .accept(tcp)
                .await;
        });
        let mut config = crate::channel::build_tls_config_with_ca(
            crate::TlsMode::Verified,
            &parse_trusted_pem(extra).unwrap(),
        )
        .unwrap();
        if expired {
            #[derive(Debug)]
            struct FutureClock;
            impl rustls::time_provider::TimeProvider for FutureClock {
                fn current_time(&self) -> Option<rustls::pki_types::UnixTime> {
                    Some(rustls::pki_types::UnixTime::since_unix_epoch(
                        std::time::Duration::from_secs(6_000_000_000),
                    ))
                }
            }
            config.time_provider = Arc::new(FutureClock);
        }
        let tcp = tokio::net::TcpStream::connect(addr).await.unwrap();
        let result = tokio_rustls::TlsConnector::from(Arc::new(config))
            .connect(ServerName::try_from(name.to_owned()).unwrap(), tcp)
            .await;
        let success = result.is_ok();
        drop(result);
        tokio::time::timeout(std::time::Duration::from_secs(5), server)
            .await
            .unwrap()
            .unwrap();
        success
    }
    #[tokio::test]
    async fn private_chain_is_rejected_without_explicit_task_trust() {
        assert!(!handshake(&[], "localhost").await);
    }
    #[tokio::test]
    async fn explicit_trust_enables_private_chain() {
        assert!(handshake(&[CA.into()], "localhost").await);
    }
    #[tokio::test]
    async fn explicit_trust_never_bypasses_hostname_check() {
        assert!(!handshake(&[CA.into()], "wrong.invalid").await);
    }
    #[tokio::test]
    async fn explicit_trust_never_bypasses_expired_server_certificate() {
        assert!(!handshake_at(&[CA.into()], "localhost", true).await);
    }
}
