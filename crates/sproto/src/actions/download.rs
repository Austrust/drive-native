use std::{
    fs::{self, File},
    path::Path,
};
use tokio::{fs as tokio_fs, io::AsyncWrite};

use crate::{channel::Channel, client::Client, error::Result, frame, pstream::PObject};

/// Download a file by its `file_id` to a local path.
///
/// The `file_id` comes from a prior `list_dir` call's `NodeInfo`.
/// Writes to a same-directory temporary file, then atomically replaces the destination.
/// Metadata-only responses are unsupported and never reported as successful downloads.
/// This checks payload presence/length, not the unconfirmed server hash algorithm.
pub async fn download(ch: &mut Channel, client: &Client, file_id: &str, dest: &Path) -> Result<()> {
    let request = build_download_request(client, file_id)?;
    ch.send(frame::SCMD_REQUEST, &request).await?;
    receive_atomic(ch, dest).await
}

async fn receive_atomic(ch: &mut Channel, dest: &Path) -> Result<()> {
    let parent = dest
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    fs::create_dir_all(parent)?;
    let temp = tempfile::NamedTempFile::new_in(parent)?;
    let mut writer = tokio_fs::File::from_std(temp.reopen()?);
    let response = ch.recv_download(&mut writer).await?;
    validate_payload(&response)?;
    writer.sync_all().await?;
    drop(writer);
    temp.persist(dest).map_err(|e| e.error)?;
    // Linux directory fsync makes the replacement durable across a power loss.
    #[cfg(unix)]
    File::open(parent)?.sync_all()?;
    Ok(())
}

fn validate_payload(response: &PObject) -> Result<()> {
    let file = response
        .get("file")
        .ok_or_else(|| crate::Error::Decode("missing download file metadata".into()))?;
    let length = match file.get("data") {
        Some(PObject::Binary { length } | PObject::BinaryEx { length, .. }) => *length,
        _ => {
            return Err(crate::Error::Decode(
                "download has no inline payload; refer/dedup unresolved".into(),
            ));
        }
    };
    if let Some(size) = file.get("size").and_then(PObject::as_int) {
        if length != size {
            return Err(crate::Error::Decode("download size mismatch".into()));
        }
    }
    // Only this single payload is supported by the action. The codec itself can
    // decode multiple binary chunks, but concatenating arbitrary chunks isn't a file contract.
    fn count(obj: &PObject) -> usize {
        match obj {
            PObject::Binary { .. } | PObject::BinaryEx { .. } => 1,
            PObject::Map(m) => m.values().map(count).sum(),
            PObject::Array(a) => a.iter().map(count).sum(),
            _ => 0,
        }
    }
    if count(response) != 1 {
        return Err(crate::Error::Decode("ambiguous download payloads".into()));
    }
    Ok(())
}

/// Download a file by its `file_id` to any `AsyncWrite` sink.
///
/// Returns the full response `PObject` containing file metadata
/// (`sync_id`, `path`, `file.hash`, `file.size`, etc.).
pub async fn download_to<W: AsyncWrite + Unpin + Send>(
    ch: &mut Channel,
    client: &Client,
    file_id: &str,
    dest: &mut W,
) -> Result<PObject> {
    let request = build_download_request(client, file_id)?;

    tracing::debug!(file_id, "sending download request");
    ch.send(frame::SCMD_REQUEST, &request).await?;

    tracing::debug!(file_id, "receiving download response");
    let result = ch.recv_download(dest).await?;
    validate_payload(&result)?;
    tracing::debug!(file_id, "download complete");
    Ok(result)
}

/// Minimum server build that supports the `"id:<file_id>"` path format.
const MIN_FILE_ID_BUILD: u64 = 12001;

fn build_download_request(client: &Client, file_id: &str) -> Result<PObject> {
    if client.server_build < MIN_FILE_ID_BUILD {
        return Err(crate::Error::UnsupportedServer {
            build: client.server_build,
            min_build: MIN_FILE_ID_BUILD,
            reason: "downloads by file ID require id:-style paths, \
			         which are not supported on this server build"
                .into(),
        });
    }

    Ok(client.build_request(
        "download",
        pmap! {
            "sync_id" => 0u64,
            "file" => pmap! { "offset" => 0u64 },
            "path" => PObject::Str(format!("id:{file_id}")),
        },
    ))
}

#[cfg(test)]
mod regression_tests {
    use super::*;
    use crate::pstream;
    use tokio::io::AsyncWriteExt;

    fn wire(data: &[u8], declared: u64) -> Vec<u8> {
        let mut w = vec![0x42];
        pstream::encode(&PObject::from("file"), &mut w).unwrap();
        w.push(0x42);
        pstream::encode(&PObject::from("data"), &mut w).unwrap();
        w.push(0x30);
        w.extend_from_slice(&declared.to_be_bytes());
        w.extend_from_slice(data);
        if declared == data.len() as u64 {
            w.extend_from_slice(&[0x40, 0x40]);
        }
        w
    }
    async fn receive(bytes: Vec<u8>, path: &Path) -> Result<()> {
        let (a, mut b) = tokio::io::duplex(4096);
        tokio::spawn(async move {
            b.write_all(&bytes).await.unwrap();
        });
        receive_atomic(&mut Channel::from_stream(a), path).await
    }
    #[tokio::test]
    async fn empty_file_is_materialized() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("zero");
        receive(wire(b"", 0), &p).await.unwrap();
        assert_eq!(fs::metadata(p).unwrap().len(), 0);
    }
    #[tokio::test]
    async fn interrupted_transfer_keeps_old_file_and_cleans_temporary() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("doc");
        fs::write(&p, b"old complete version").unwrap();
        assert!(receive(wire(b"prefix", 100), &p).await.is_err());
        assert_eq!(fs::read(p).unwrap(), b"old complete version");
        assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 1);
    }
    #[tokio::test]
    async fn metadata_only_is_not_success() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("doc");
        fs::write(&p, b"old").unwrap();
        let mut w = Vec::new();
        pstream::encode(&pmap! { "file" => pmap! { "refer" => true } }, &mut w).unwrap();
        assert!(receive(w, &p).await.is_err());
        assert_eq!(fs::read(p).unwrap(), b"old");
    }
    #[tokio::test]
    async fn successful_transfer_atomically_replaces_destination() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("doc");
        fs::write(&p, b"old").unwrap();
        receive(wire(b"new complete version", 20), &p)
            .await
            .unwrap();
        assert_eq!(fs::read(p).unwrap(), b"new complete version");
    }
    #[tokio::test]
    async fn cancelled_transfer_keeps_old_file() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("doc");
        fs::write(&p, b"old").unwrap();
        let (a, mut b) = tokio::io::duplex(4096);
        b.write_all(&wire(b"partial", 100)).await.unwrap();
        let mut ch = Channel::from_stream(a);
        assert!(
            tokio::time::timeout(
                std::time::Duration::from_millis(30),
                receive_atomic(&mut ch, &p)
            )
            .await
            .is_err()
        );
        assert_eq!(fs::read(p).unwrap(), b"old");
        assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 1);
    }
}
