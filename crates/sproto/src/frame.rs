use tokio::io::{AsyncWrite, AsyncWriteExt};

use crate::{
    error::Result,
    pstream::{self, PObject},
};

pub const VERSION: u8 = 0x46; // 70 decimal = protocol 7.0
pub const MAGIC: u32 = 0x2552_1814;

// SCMD command types (spec §2.2 / §3.3)
pub const SCMD_REQUEST: u8 = 0x01;
pub const SCMD_CONNECT: u8 = 0x0E;
pub const SCMD_AUTH: u8 = 0x12;
pub const SCMD_SSL_UPGRADE: u8 = 0x16;
pub const SCMD_SERVER_INFO: u8 = 0x19;
pub const SCMD_LIST: u8 = 0x1a;

/// Client version string sent during auth and connect handshake.
pub const CLIENT_VERSION: &str = "4.0.0-17889";
/// Client version build number used in the `_agent` block.
pub const CLIENT_BUILD: u64 = 17889;

/// Write a framed message: header + `PObject` payload in a single write.
pub async fn send_message<W: AsyncWrite + Unpin>(
    writer: &mut W,
    scmd: u8,
    obj: &PObject,
) -> Result<()> {
    let mut buf = Vec::new();
    buf.extend_from_slice(&MAGIC.to_be_bytes());
    buf.push(VERSION);
    buf.push(scmd);
    buf.extend_from_slice(&0u16.to_be_bytes()); // pkt_len always 0
    pstream::encode(obj, &mut buf)?;
    writer.write_all(&buf).await?;
    writer.flush().await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;
    use tokio::io::{AsyncRead, AsyncReadExt};

    use super::*;
    use crate::error::Error;

    /// Read and validate the 8-byte wire header, returning (version, scmd).
    async fn read_header<R: AsyncRead + Unpin>(reader: &mut R) -> Result<(u8, u8)> {
        let magic = reader.read_u32().await?;
        if magic != MAGIC {
            return Err(Error::BadMagic);
        }

        let version = reader.read_u8().await?;
        if !(70..80).contains(&version) {
            return Err(Error::VersionMismatch { got: version });
        }

        let scmd = reader.read_u8().await?;
        reader.read_u16().await?; // pkt_len (always 0, discard)

        Ok((version, scmd))
    }

    async fn recv_message<R: AsyncRead + Unpin + Send>(reader: &mut R) -> Result<(u8, PObject)> {
        let (_version, scmd) = read_header(reader).await?;
        let obj = pstream::decode_from(reader, None).await?;
        Ok((scmd, obj))
    }

    #[tokio::test]
    async fn header_byte_layout() {
        let mut buf = Vec::new();
        send_message(&mut buf, 0x12, &pmap! {}).await.unwrap();
        assert_eq!(&buf[..8], &[0x25, 0x52, 0x18, 0x14, 0x46, 0x12, 0x00, 0x00]);
    }

    #[tokio::test]
    async fn roundtrip_via_duplex() {
        let (mut client, mut server) = tokio::io::duplex(4096);

        let obj = pmap! {
            "action" => "test",
            "value" => 42u64,
        };

        let obj_clone = obj.clone();
        let write_handle = tokio::spawn(async move {
            send_message(&mut client, 0x01, &obj_clone).await.unwrap();
        });

        let (scmd, decoded) = recv_message(&mut server).await.unwrap();
        write_handle.await.unwrap();

        assert_eq!(scmd, 0x01);
        assert_eq!(decoded, obj);
    }

    #[tokio::test]
    async fn bad_magic() {
        let bad = [0xFF, 0xFF, 0xFF, 0xFF, 0x46, 0x01, 0x00, 0x00];
        let mut cursor = Cursor::new(bad);
        let err = read_header(&mut cursor).await.unwrap_err();
        assert!(matches!(err, Error::BadMagic));
    }

    #[tokio::test]
    async fn version_boundaries() {
        let mut buf = Vec::new();
        buf.extend_from_slice(&MAGIC.to_be_bytes());
        buf.push(69);
        buf.push(0x01);
        buf.extend_from_slice(&0u16.to_be_bytes());

        // Version 69: too old
        let err = read_header(&mut Cursor::new(&buf)).await.unwrap_err();
        assert!(matches!(err, Error::VersionMismatch { got: 69 }));

        // Version 70: ok
        buf[4] = 70;
        let (version, _) = read_header(&mut Cursor::new(&buf)).await.unwrap();
        assert_eq!(version, 70);

        // Version 79: ok
        buf[4] = 79;
        let (version, _) = read_header(&mut Cursor::new(&buf)).await.unwrap();
        assert_eq!(version, 79);

        // Version 80: too new
        buf[4] = 80;
        let err = read_header(&mut Cursor::new(&buf)).await.unwrap_err();
        assert!(matches!(err, Error::VersionMismatch { got: 80 }));
    }

    #[tokio::test]
    async fn recv_response_reads_raw_pobject() {
        let obj = pmap! { "alive" => 300u64, "type" => "response" };
        let mut buf = Vec::new();
        pstream::encode(&obj, &mut buf).unwrap();

        let mut cursor = Cursor::new(buf);
        let decoded = pstream::decode_from(&mut cursor, None).await.unwrap();
        assert_eq!(decoded["alive"].as_int().unwrap(), 300);
        assert_eq!(decoded["type"].as_str().unwrap(), "response");
    }

    #[tokio::test]
    async fn send_message_all_scmd_types() {
        for scmd in [0x01u8, 0x0E, 0x12, 0x16, 0x18, 0x19, 0x1a] {
            let obj = pmap! { "_action" => "test" };
            let mut buf = Vec::new();
            send_message(&mut buf, scmd, &obj).await.unwrap();

            assert_eq!(&buf[..4], &MAGIC.to_be_bytes());
            assert_eq!(buf[4], VERSION);
            assert_eq!(buf[5], scmd);
            assert_eq!(&buf[6..8], &[0x00, 0x00]);
        }
    }
}
