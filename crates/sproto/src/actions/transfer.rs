//! Whole-file native transfers. MD4 is the protocol's content checksum, not an
//! authentication primitive; TLS authenticates the server. SHA-256 identifies
//! the immutable local blob. No caller may publish before all checks complete.
use super::native::{NativeVersion, validate_path};
use crate::{Client, Error, Result, frame, pstream::PObject};
use md4::{Digest, Md4};
use sha2::Sha256;
use std::{
    path::Path,
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWrite};

#[cfg(feature = "protocol-research")]
enum UploadReceiptMode<'a> {
    Read,
    Drop,
    DropAfter(Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + 'a>>),
}

#[derive(Clone, Debug)]
pub struct ContentVersion {
    pub file_id: String,
    pub version: NativeVersion,
    pub size: u64,
    pub hash: String,
}
#[derive(Debug)]
pub struct VerifiedContent {
    pub remote: ContentVersion,
    pub sha256: String,
}

struct HashingWriter<W> {
    inner: W,
    md4: Md4,
    sha256: Sha256,
    count: u64,
    limit: u64,
}
impl<W: AsyncWrite + Unpin> AsyncWrite for HashingWriter<W> {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        let s = self.get_mut();
        if buf.len() as u64 > s.limit.saturating_sub(s.count) {
            return Poll::Ready(Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "download exceeds expected content size",
            )));
        }
        match Pin::new(&mut s.inner).poll_write(cx, buf) {
            Poll::Ready(Ok(n)) => {
                s.md4.update(&buf[..n]);
                s.sha256.update(&buf[..n]);
                s.count += n as u64;
                Poll::Ready(Ok(n))
            }
            other => other,
        }
    }
    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.get_mut().inner).poll_flush(cx)
    }
    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.get_mut().inner).poll_shutdown(cx)
    }
}
fn number(o: &PObject, k: &str) -> Result<u64> {
    o.get(k)
        .and_then(PObject::as_int)
        .ok_or_else(|| Error::Decode(format!("missing transfer {k}")))
}
fn text(o: &PObject, k: &str) -> Result<String> {
    o.get(k)
        .and_then(PObject::as_str)
        .map(str::to_owned)
        .ok_or_else(|| Error::Decode(format!("missing transfer {k}")))
}

fn receipt(r: &PObject) -> Result<ContentVersion> {
    let f = r
        .get("file")
        .ok_or_else(|| Error::Decode("missing transfer file block".into()))?;
    Ok(ContentVersion {
        file_id: text(r, "file_id")?,
        version: NativeVersion {
            sync_id: number(r, "sync_id")?,
            max_id: number(r, "max_id")?,
        },
        size: number(f, "size")?,
        hash: text(f, "hash")?,
    })
}

fn check_content(
    r: &PObject,
    expected: &ContentVersion,
    md4: &str,
    count: u64,
) -> Result<ContentVersion> {
    let got = receipt(r)?;
    let f = r.get("file").unwrap();
    match f.get("data") {
        Some(PObject::Binary { length }) if *length == count => {}
        Some(PObject::BinaryEx { length, send_hash }) if *length == count && send_hash == md4 => {}
        _ => {
            return Err(Error::Decode(
                "missing or unverified download payload".into(),
            ));
        }
    }
    if got.file_id != expected.file_id
        || got.version != expected.version
        || got.size != expected.size
        || got.hash != expected.hash
        || got.hash != md4
        || count != expected.size
    {
        return Err(Error::Decode(
            "download identity, version, size or checksum changed".into(),
        ));
    }
    if f.get("is_delta").and_then(PObject::as_int).unwrap_or(0) != 0
        || f.get("refer").and_then(PObject::as_int).unwrap_or(0) != 0
    {
        return Err(Error::Decode("delta/refer is not a complete file".into()));
    }
    Ok(got)
}

impl Client {
    /// Download latest bytes, accept only the exact identity/version observed in
    /// metadata. `sync_id=0` requests contents; nonzero is NOT a version selector.
    /// Old destination survives interruption, stale response, bad hash or full disk.
    pub async fn download_checked(
        &self,
        expected: &ContentVersion,
        dest: &Path,
    ) -> Result<VerifiedContent> {
        self.download_internal(expected, dest, false, false, None)
            .await
    }

    /// Read-only experiment for the Windows whole-file request shape.
    #[cfg(feature = "protocol-research")]
    pub async fn research_download_file_flag(
        &self,
        expected: &ContentVersion,
        dest: &Path,
    ) -> Result<VerifiedContent> {
        self.download_internal(expected, dest, true, false, None)
            .await
    }

    /// Windows non-FileEvent download branch: current remote contents, still checked against caller metadata.
    #[cfg(feature = "protocol-research")]
    pub async fn research_download_current(
        &self,
        expected: &ContentVersion,
        dest: &Path,
    ) -> Result<VerifiedContent> {
        self.download_internal(expected, dest, true, true, None)
            .await
    }

    /// Read-only probe for the official ProtocolBuilder root and restore envelope.
    #[cfg(feature = "protocol-research")]
    pub async fn research_download_scoped(
        &self,
        expected: &ContentVersion,
        dest: &Path,
        watch: &super::native::Watch,
    ) -> Result<VerifiedContent> {
        validate_path(&watch.path)?;
        self.download_internal(expected, dest, true, true, Some(watch))
            .await
    }

    async fn download_internal(
        &self,
        expected: &ContentVersion,
        dest: &Path,
        file_flag: bool,
        force_current: bool,
        watch: Option<&super::native::Watch>,
    ) -> Result<VerifiedContent> {
        if expected.file_id.is_empty() || !expected.file_id.bytes().all(|b| b.is_ascii_digit()) {
            return Err(Error::InvalidConfig(
                "native file identity must be decimal".into(),
            ));
        }
        let parent = dest
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        std::fs::create_dir_all(parent)?;
        let tmp = tempfile::NamedTempFile::new_in(parent)?;
        crate::flow::begin(
            crate::flow::Direction::Download,
            expected.size,
            crate::flow::Phase::Queued,
        );
        let mut sink = HashingWriter {
            inner: tokio::fs::File::from_std(tmp.reopen()?),
            md4: Md4::new(),
            sha256: Sha256::new(),
            count: 0,
            limit: expected.size,
        };
        // Official PrepareDownloadRequest at 0xc4b03 carries the event's
        // target_sync_id separately from the locally available sync_id/max_id.
        let file = if file_flag {
            PObject::from(true)
        } else {
            pmap! {"offset"=>0u64}
        };
        let mut fields = pmap! {"path"=>format!("id:{}",expected.file_id),
        "sync_id"=>0u64,"max_id"=>0u64,"target_sync_id"=>if force_current {0u64}else{expected.version.sync_id},"file"=>file};
        if file_flag {
            fields
                .as_map_mut()
                .unwrap()
                .insert("force_current_version".into(), PObject::from(force_current));
        }
        if let Some(watch) = watch {
            let map = fields.as_map_mut().unwrap();
            map.insert("root_node_id".into(), PObject::from(watch.node_id));
            map.insert("root_node_path".into(), PObject::from(watch.path.as_str()));
            // Ordinary ID-based official requests remove view_id but retain root identity.
            map.insert(
                "server_id".into(),
                pmap! {"restore_id"=>self.restore_identity()},
            );
        }
        let request = self.build_request("download", fields);
        let mut guard = self.pool.acquire().await?;
        let response = crate::flow::idle_timeout(Duration::from_secs(1800), async {
            let ch = guard.channel();
            ch.send(frame::SCMD_REQUEST, &request).await?;
            crate::flow::phase(crate::flow::Phase::Transferring);
            ch.recv_download(&mut sink).await
        })
        .await
        .map_err(|_| Error::Timeout("download"))?;
        let response = response?;
        crate::flow::phase(crate::flow::Phase::Verifying);
        let checksum = format!("{:x}", sink.md4.finalize());
        let sha256 = format!("{:x}", sink.sha256.finalize());
        let remote =
            guard.poison_on_err(check_content(&response, expected, &checksum, sink.count))?;
        sink.inner.sync_all().await?;
        drop(sink.inner);
        tmp.persist(dest).map_err(|e| e.error)?;
        #[cfg(unix)]
        std::fs::File::open(parent)?.sync_all()?;
        Ok(VerifiedContent { remote, sha256 })
    }

    /// Experimental write primitive. Missing acknowledgements are uncertain,
    /// never permission to resend. The service must journal before calling.
    /// Only protocol-research builds expose mutations until recovery is validated.
    #[cfg(feature = "protocol-research")]
    pub async fn upload_snapshot(
        &self,
        view_id: u64,
        path: &str,
        base: NativeVersion,
        snapshot: &Path,
    ) -> Result<ContentVersion> {
        self.upload_internal(view_id, path, base, snapshot, UploadReceiptMode::Read)
            .await
    }

    /// Fault-injection only: send the complete request, then close before reading
    /// any acknowledgement. The caller must journal and reconcile, never retry.
    #[cfg(feature = "protocol-research")]
    pub async fn research_upload_drop_receipt(
        &self,
        view_id: u64,
        path: &str,
        base: NativeVersion,
        snapshot: &Path,
    ) -> Result<ContentVersion> {
        self.upload_internal(view_id, path, base, snapshot, UploadReceiptMode::Drop)
            .await
    }

    /// Send the complete upload but never read its terminal response. The
    /// independent observer must see the new remote revision before this socket
    /// is dropped. This fault injector never authorizes replay or confirmation.
    #[cfg(feature = "protocol-research")]
    pub async fn research_upload_drop_receipt_after<F>(
        &self,
        view_id: u64,
        path: &str,
        base: NativeVersion,
        snapshot: &Path,
        independently_observed: F,
    ) -> Result<ContentVersion>
    where
        F: std::future::Future<Output = Result<()>> + Send,
    {
        self.upload_internal(
            view_id,
            path,
            base,
            snapshot,
            UploadReceiptMode::DropAfter(Box::pin(independently_observed)),
        )
        .await
    }

    /// Identity addressing from Windows PrepareUploadRequest, verified on Drive
    /// 4.0.3. The caller still must verify live parent/root and commit identity:
    /// a deleted parent can accept a creation without becoming a live directory.
    #[cfg(feature = "protocol-research")]
    pub async fn upload_snapshot_at_identity(
        &self,
        identity: &str,
        leaf: Option<&str>,
        base: NativeVersion,
        snapshot: &Path,
    ) -> Result<ContentVersion> {
        validate_identity_base(leaf, base)?;
        let wire_path = identity_upload_path(identity, leaf)?;
        self.upload_wire(None, &wire_path, base, snapshot, UploadReceiptMode::Read)
            .await
    }

    /// Explicit unread-reply injection using the same identity wire shape.
    #[cfg(feature = "protocol-research")]
    pub async fn research_upload_identity_drop_receipt_after<F>(
        &self,
        identity: &str,
        leaf: Option<&str>,
        base: NativeVersion,
        snapshot: &Path,
        observed: F,
    ) -> Result<ContentVersion>
    where
        F: std::future::Future<Output = Result<()>> + Send,
    {
        validate_identity_base(leaf, base)?;
        let wire_path = identity_upload_path(identity, leaf)?;
        self.upload_wire(
            None,
            &wire_path,
            base,
            snapshot,
            UploadReceiptMode::DropAfter(Box::pin(observed)),
        )
        .await
    }

    #[cfg(feature = "protocol-research")]
    async fn upload_internal(
        &self,
        view_id: u64,
        path: &str,
        base: NativeVersion,
        snapshot: &Path,
        receipt_mode: UploadReceiptMode<'_>,
    ) -> Result<ContentVersion> {
        validate_path(path)?;
        if path == "/" {
            return Err(Error::InvalidConfig("cannot upload over root".into()));
        }
        self.upload_wire(Some(view_id), path, base, snapshot, receipt_mode)
            .await
    }
    #[cfg(feature = "protocol-research")]
    async fn upload_wire(
        &self,
        view_id: Option<u64>,
        path: &str,
        base: NativeVersion,
        snapshot: &Path,
        receipt_mode: UploadReceiptMode<'_>,
    ) -> Result<ContentVersion> {
        let mut file = tokio::fs::File::open(snapshot).await?;
        let stat = file.metadata().await?;
        if !stat.is_file() {
            return Err(Error::InvalidConfig(
                "snapshot is not a regular file".into(),
            ));
        }
        crate::flow::begin(
            crate::flow::Direction::Upload,
            stat.len(),
            crate::flow::Phase::Preparing,
        );
        let mut hash = Md4::new();
        let mut buffer = vec![0u8; 65536];
        let mut size = 0u64;
        loop {
            let n = file.read(&mut buffer).await?;
            if n == 0 {
                break;
            }
            hash.update(&buffer[..n]);
            size += n as u64;
        }
        if size != stat.len() {
            return Err(Error::Decode("snapshot changed during hash".into()));
        }
        let hash = format!("{:x}", hash.finalize());
        file.seek(std::io::SeekFrom::Start(0)).await?;
        let mut fields = pmap! {
            "path"=>path,"sync_id"=>base.sync_id,"max_id"=>base.max_id,
            "conflict_policy"=>"overwrite_client","is_dir"=>false,"file_type"=>"file",
            "file"=>pmap!{"data"=>PObject::Binary{length:size},"size"=>size,"hash"=>hash.as_str(),"refer"=>false,"is_delta"=>false},
        };
        // Official PrepareUploadRequest erases view_id for id:-style paths.
        // Sending both was rejected with 0x7000 on Drive 4.0.3.
        if let Some(view) = view_id {
            fields
                .as_map_mut()
                .unwrap()
                .insert("view_id".into(), view.into());
        }
        let request = self.build_request("upload", fields);
        super::upload_observer::before_send(super::upload_observer::UploadBoundary {
            view: view_id,
            path: path.into(),
        })
        .await?;
        let mut guard = self.pool.acquire().await?;
        let result = crate::flow::idle_timeout(Duration::from_secs(1800), async {
            let ch = guard.channel();
            crate::flow::phase(crate::flow::Phase::Transferring);
            ch.send_binary(&request, &mut file).await?;
            crate::flow::phase(crate::flow::Phase::AwaitingConfirmation);
            match receipt_mode {
                UploadReceiptMode::Read => {}
                UploadReceiptMode::Drop => {
                    return Err(Error::Timeout(
                        "injected lost upload reply; reconcile before retry",
                    ));
                }
                UploadReceiptMode::DropAfter(observed) => {
                    tokio::time::timeout(Duration::from_secs(30), observed)
                        .await
                        .map_err(|_| {
                            Error::Timeout("independent upload observation timed out")
                        })??;
                    return Err(Error::Timeout(
                        "injected unread upload reply after independent commit observation",
                    ));
                }
            }
            let response = ch.recv().await?;
            crate::error::check_server_error(&response)?;
            let r = receipt(&response)?;
            if r.hash != hash || r.size != size {
                return Err(Error::Decode(
                    "upload acknowledgement mismatch; reconcile before retry".into(),
                ));
            }
            Ok(r)
        })
        .await
        .map_err(|_| Error::Timeout("upload; commit uncertain"))?;
        guard.poison_on_err(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::AsyncWriteExt;
    fn expected() -> ContentVersion {
        ContentVersion {
            file_id: "99".into(),
            version: NativeVersion {
                sync_id: 7,
                max_id: 7,
            },
            size: 3,
            hash: "a448017aaf21d8525fc10ae87aa6729d".into(),
        }
    }
    fn response() -> PObject {
        pmap! {"file_id"=>"99","sync_id"=>7u64,"max_id"=>7u64,"file"=>pmap!{"size"=>3u64,"hash"=>expected().hash,"data"=>PObject::Binary{length:3}}}
    }
    #[test]
    fn stale_version_and_bad_checksum_never_publish() {
        let e = expected();
        check_content(&response(), &e, &e.hash, 3).unwrap();
        assert!(check_content(&response(), &e, "bad", 3).is_err());
        let mut stale = e.clone();
        stale.version.sync_id = 8;
        assert!(check_content(&response(), &stale, &e.hash, 3).is_err());
        let mut replacement = e.clone();
        replacement.file_id = "100".into();
        assert!(check_content(&response(), &replacement, &e.hash, 3).is_err());
    }
    #[tokio::test]
    async fn hashing_writer_checks_size_and_standard_md4_vector() {
        let mut s = HashingWriter {
            inner: Vec::new(),
            md4: Md4::new(),
            sha256: Sha256::new(),
            count: 0,
            limit: 3,
        };
        s.write_all(b"abc").await.unwrap();
        assert!(s.write_all(b"x").await.is_err());
        assert_eq!(format!("{:x}", s.md4.finalize()), expected().hash);
        assert_eq!(s.inner, b"abc");
    }
    #[tokio::test]
    async fn streaming_encoder_roundtrips_empty_and_nonempty() {
        for data in [b"".as_slice(), b"abc".as_slice()] {
            let obj = pmap! {"file"=>pmap!{"data"=>PObject::Binary{length:data.len() as u64},"size"=>data.len() as u64},"sync_id"=>8u64};
            let (mut prefix, suffix, n) = crate::pstream::binary_parts(&obj).unwrap();
            assert_eq!(n, data.len() as u64);
            prefix.extend_from_slice(data);
            prefix.extend_from_slice(&suffix);
            let mut sink = Vec::new();
            let got =
                crate::pstream::decode_from(&mut std::io::Cursor::new(prefix), Some(&mut sink))
                    .await
                    .unwrap();
            assert_eq!(got, obj);
            assert_eq!(sink, data);
        }
    }
}

#[cfg(any(feature = "protocol-research", test))]
fn validate_identity_base(leaf: Option<&str>, base: NativeVersion) -> Result<()> {
    let valid = if leaf.is_some() {
        base.sync_id == 0 && base.max_id == 0
    } else {
        base.sync_id > 0 && base.max_id > 0
    };
    if !valid {
        return Err(Error::InvalidConfig(
            "identity upload version does not match creation/update".into(),
        ));
    }
    Ok(())
}

#[cfg(any(feature = "protocol-research", test))]
pub(super) fn identity_upload_path(identity: &str, leaf: Option<&str>) -> Result<String> {
    if identity.is_empty()
        || !identity.bytes().all(|v| v.is_ascii_digit())
        || identity.parse::<u64>().ok().filter(|v| *v > 0).is_none()
    {
        return Err(Error::InvalidConfig("invalid upload identity".into()));
    }
    match leaf {
        None => Ok(format!("id:{identity}")),
        Some(name)
            if !name.is_empty() && !matches!(name, "." | "..") && !name.contains(['/', '\0']) =>
        {
            Ok(format!("id:{identity}/{name}"))
        }
        _ => Err(Error::InvalidConfig("invalid upload identity leaf".into())),
    }
}
#[cfg(test)]
mod identity_path_tests {
    use super::*;
    #[test]
    fn identity_upload_address_cannot_escape_to_another_path() {
        assert!(
            validate_identity_base(
                Some("A"),
                NativeVersion {
                    sync_id: 0,
                    max_id: 0
                }
            )
            .is_ok()
        );
        assert!(
            validate_identity_base(
                None,
                NativeVersion {
                    sync_id: 1,
                    max_id: 1
                }
            )
            .is_ok()
        );
        assert!(
            validate_identity_base(
                None,
                NativeVersion {
                    sync_id: 0,
                    max_id: 0
                }
            )
            .is_err()
        );
        assert!(
            validate_identity_base(
                Some("A"),
                NativeVersion {
                    sync_id: 1,
                    max_id: 1
                }
            )
            .is_err()
        );

        assert_eq!(identity_upload_path("123", None).unwrap(), "id:123");
        assert_eq!(
            identity_upload_path("123", Some("离线 A.txt")).unwrap(),
            "id:123/离线 A.txt"
        );
        for id in ["", "0", "-1", "1/2", "id:1", "18446744073709551616"] {
            assert!(identity_upload_path(id, None).is_err());
        }
        for name in ["", ".", "..", "../x", "/x", "x\0y"] {
            assert!(identity_upload_path("123", Some(name)).is_err());
        }
    }
}
