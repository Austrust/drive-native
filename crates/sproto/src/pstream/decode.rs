use std::pin::Pin;

use crate::pstream::{
    BTreeMap, Error, MAX_DEPTH, PObject, Result, TAG_ARRAY, TAG_BINARY, TAG_BINARY_EX, TAG_END,
    TAG_INTEGER, TAG_MAP, TAG_NULL, TAG_STRING,
};

use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

// A valid 2000-event response from Drive 4.0.3 contains 100024 values.
// Keep a bounded per-message allocation independently of string/depth limits.
const MAX_METADATA_VALUES: usize = 1_000_000;
type BinaryDest<'w> = &'w mut (dyn AsyncWrite + Unpin + Send);

struct DecodeBudget {
    nodes: usize,
    strings: usize,
}
impl DecodeBudget {
    fn node(&mut self) -> Result<()> {
        self.nodes = self
            .nodes
            .checked_sub(1)
            .ok_or_else(|| Error::Decode("metadata node budget exceeded".into()))?;
        Ok(())
    }
    fn string(&mut self, size: usize) -> Result<()> {
        self.strings = self
            .strings
            .checked_sub(size)
            .ok_or_else(|| Error::Decode("metadata string budget exceeded".into()))?;
        Ok(())
    }
}

/// Decode a `PObject` by reading from an async stream.
///
/// When `binary_dest` is provided, any inline binary data (tags 0x30/0x43)
/// is streamed directly to the writer and decoding continues, returning a
/// complete `PObject` with `Binary`/`BinaryEx` markers in place of the data.
///
/// When `binary_dest` is `None`, encountering binary data is an error —
/// only use this for messages that are known not to contain file data.
pub fn decode_from<'r, 'w: 'r, R: AsyncRead + Unpin + Send + 'r>(
    reader: &'r mut R,
    binary_dest: Option<BinaryDest<'w>>,
) -> Pin<Box<dyn std::future::Future<Output = Result<PObject>> + Send + 'r>> {
    Box::pin(async move {
        let mut dest = binary_dest;
        let mut budget = DecodeBudget {
            nodes: MAX_METADATA_VALUES,
            strings: 16 * 1024 * 1024,
        };
        Box::pin(decode_value(reader, 0, &mut dest, &mut budget)).await
    })
}

// --- Recursive trampolines ---
//
// `decode_value` and `dispatch_tag` are `async fn`s so the compiler can track
// borrows across `.await` points in loops.  The `boxed_*` wrappers add the
// `Pin<Box<dyn Future>>` indirection required for recursion.

fn boxed_decode_value<'a, 'w: 'a, R: AsyncRead + Unpin + Send + 'a>(
    reader: &'a mut R,
    depth: usize,
    binary_dest: &'a mut Option<BinaryDest<'w>>,
    budget: &'a mut DecodeBudget,
) -> Pin<Box<dyn std::future::Future<Output = Result<PObject>> + Send + 'a>> {
    Box::pin(decode_value(reader, depth, binary_dest, budget))
}

fn boxed_dispatch_tag<'a, 'w: 'a, R: AsyncRead + Unpin + Send + 'a>(
    reader: &'a mut R,
    tag: u8,
    depth: usize,
    binary_dest: &'a mut Option<BinaryDest<'w>>,
    budget: &'a mut DecodeBudget,
) -> Pin<Box<dyn std::future::Future<Output = Result<PObject>> + Send + 'a>> {
    Box::pin(dispatch_tag(reader, tag, depth, binary_dest, budget))
}

/// Read a tag byte then dispatch to the appropriate decoder.
async fn decode_value<R: AsyncRead + Unpin + Send>(
    reader: &mut R,
    depth: usize,
    binary_dest: &mut Option<BinaryDest<'_>>,
    budget: &mut DecodeBudget,
) -> Result<PObject> {
    if depth > MAX_DEPTH {
        return Err(Error::Decode("max nesting depth exceeded".into()));
    }
    let tag = reader.read_u8().await?;

    Box::pin(dispatch_tag(reader, tag, depth, binary_dest, budget)).await
}

/// Decode a value given its already-read tag byte.
#[allow(
    clippy::too_many_lines,
    reason = "tag dispatch is a single match with one arm per wire type"
)]
async fn dispatch_tag<R: AsyncRead + Unpin + Send>(
    reader: &mut R,
    tag: u8,
    depth: usize,
    binary_dest: &mut Option<BinaryDest<'_>>,
    budget: &mut DecodeBudget,
) -> Result<PObject> {
    if depth > MAX_DEPTH {
        return Err(Error::Decode("max nesting depth exceeded".into()));
    }
    budget.node()?;
    match tag {
        TAG_NULL => {
            if reader.read_u8().await? != 0 {
                return Err(Error::Decode("invalid null padding".into()));
            }
            Ok(PObject::Null)
        }
        TAG_INTEGER => {
            let size = reader.read_u8().await?;
            let v = match size {
                1 => u64::from(reader.read_u8().await?),
                2 => u64::from(reader.read_u16().await?),
                4 => u64::from(reader.read_u32().await?),
                8 => reader.read_u64().await?,
                _ => return Err(Error::Decode(format!("invalid integer size byte: {size}"))),
            };
            Ok(PObject::Integer(v))
        }
        TAG_STRING => {
            let s = read_string_async(reader, budget).await?;
            Ok(PObject::Str(s))
        }
        TAG_BINARY => {
            let length = reader.read_u64().await?;
            let dest = binary_dest
                .as_deref_mut()
                .ok_or_else(|| Error::Decode("unexpected binary data".into()))?;
            Box::pin(stream_binary(reader, dest, length)).await?;
            Ok(PObject::Binary { length })
        }
        TAG_ARRAY => {
            let mut items = Vec::new();
            loop {
                let next = reader.read_u8().await?;
                if next == TAG_END {
                    break;
                }
                let item = boxed_dispatch_tag(reader, next, depth + 1, binary_dest, budget).await?;
                items.push(item);
            }
            Ok(PObject::Array(items))
        }
        TAG_MAP => {
            let mut map = BTreeMap::new();
            loop {
                let next = reader.read_u8().await?;
                if next == TAG_END {
                    break;
                }
                if next != TAG_STRING {
                    return Err(Error::Decode(format!(
                        "expected string key in map, got {next:#04x}"
                    )));
                }
                let key = read_string_async(reader, budget).await?;
                if map.contains_key(&key) {
                    return Err(Error::Decode("duplicate map key".into()));
                }
                let value = boxed_decode_value(reader, depth + 1, binary_dest, budget).await?;
                map.insert(key, value);
            }
            Ok(PObject::Map(map))
        }
        TAG_BINARY_EX => {
            let mut send_hash = String::new();
            let mut length = 0u64;
            let mut keys = std::collections::BTreeSet::new();
            loop {
                let next = reader.read_u8().await?;
                if next == TAG_END {
                    break;
                }
                if next != TAG_STRING {
                    return Err(Error::Decode(format!(
                        "expected string key in binary_ex, got {next:#04x}"
                    )));
                }
                let key = read_string_async(reader, budget).await?;
                if !keys.insert(key.clone()) {
                    return Err(Error::Decode("duplicate binary_ex key".into()));
                }
                match key.as_str() {
                    "binary" => {
                        let btag = reader.read_u8().await?;
                        if btag != TAG_BINARY {
                            return Err(Error::Decode("expected binary tag in binary_ex".into()));
                        }
                        length = reader.read_u64().await?;
                        let dest = binary_dest
                            .as_deref_mut()
                            .ok_or_else(|| Error::Decode("unexpected binary data".into()))?;
                        Box::pin(stream_binary(reader, dest, length)).await?;
                    }
                    "send_hash" => {
                        let val =
                            boxed_decode_value(reader, depth + 1, binary_dest, budget).await?;
                        if let PObject::Str(h) = val {
                            send_hash = h;
                        } else {
                            return Err(Error::Decode("invalid send_hash type".into()));
                        }
                    }
                    _ => {
                        boxed_decode_value(reader, depth + 1, binary_dest, budget).await?;
                    }
                }
            }
            if !keys.contains("binary") {
                return Err(Error::Decode("binary_ex has no binary payload".into()));
            }
            Ok(PObject::BinaryEx { length, send_hash })
        }
        TAG_END => Err(Error::Decode("unexpected end marker".into())),
        _ => Err(Error::Decode(format!("unknown tag {tag:#04x}"))),
    }
}

/// Stream `length` raw bytes from reader to writer.
async fn stream_binary<R: AsyncRead + Unpin, W: AsyncWrite + Unpin + ?Sized>(
    reader: &mut R,
    writer: &mut W,
    length: u64,
) -> Result<()> {
    const BUF_SIZE: usize = 64 * 1024;
    let mut buf = vec![0u8; BUF_SIZE];
    let mut remaining = length;
    while remaining > 0 {
        let to_read = BUF_SIZE.min(usize::try_from(remaining).unwrap_or(usize::MAX));
        let to_read = crate::flow::allowance(crate::flow::Direction::Download, to_read).await;
        let n = reader.read(&mut buf[..to_read]).await?;
        if n == 0 {
            return Err(Error::ConnectionClosed);
        }
        writer.write_all(&buf[..n]).await?;
        crate::flow::advance(crate::flow::Direction::Download, n);
        remaining -= n as u64;
    }
    writer.flush().await?;
    Ok(())
}

async fn read_string_async<R: AsyncRead + Unpin>(
    reader: &mut R,
    budget: &mut DecodeBudget,
) -> Result<String> {
    let len = reader.read_u16().await? as usize;
    budget.string(len)?;
    let mut buf = vec![0u8; len];
    reader.read_exact(&mut buf).await?;
    String::from_utf8(buf).map_err(|e| Error::Decode(format!("invalid utf-8: {e}")))
}

#[cfg(test)]
mod tests {
    use crate::pstream::{encode::encode_string, *};

    /// Decode a `PObject` from a byte buffer (no binary data expected).
    async fn decode(buf: &[u8]) -> Result<(PObject, u64)> {
        let mut cursor = std::io::Cursor::new(buf);
        let obj = decode_from(&mut cursor, None).await?;
        Ok((obj, cursor.position()))
    }

    #[tokio::test]
    async fn empty_buffer() {
        assert!(decode(&[]).await.is_err());
    }

    #[tokio::test]
    async fn unknown_tag() {
        assert!(decode(&[0xFF]).await.is_err());
    }

    #[tokio::test]
    async fn truncated_integer() {
        assert!(decode(&[0x01, 0x04, 0x00, 0x01]).await.is_err());
    }

    #[tokio::test]
    async fn invalid_integer_size() {
        assert!(decode(&[0x01, 0x03, 0x00, 0x00, 0x00]).await.is_err());
    }

    #[tokio::test]
    async fn truncated_string() {
        assert!(decode(&[0x10, 0x00, 0x05, b'h', b'i', b'!']).await.is_err());
    }

    #[tokio::test]
    async fn unterminated_array() {
        assert!(decode(&[TAG_ARRAY, 0x01, 0x01, 0x42]).await.is_err());
    }

    #[tokio::test]
    async fn unterminated_map() {
        assert!(
            decode(&[TAG_MAP, TAG_STRING, 0x00, 0x01, b'k'])
                .await
                .is_err()
        );
    }

    #[tokio::test]
    async fn map_non_string_key() {
        assert!(
            decode(&[TAG_MAP, TAG_INTEGER, 0x01, 0x01, TAG_END])
                .await
                .is_err()
        );
    }

    #[tokio::test]
    async fn unexpected_end_marker() {
        assert!(decode(&[TAG_END]).await.is_err());
    }

    #[tokio::test]
    async fn binary_without_writer_is_error() {
        // Binary tag encountered with no writer → error
        let buf = [0x30, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00];
        assert!(decode(&buf).await.is_err());
    }

    #[tokio::test]
    async fn binary_streams_to_writer() {
        let file_data = b"hello binary world";
        let mut wire = Vec::new();
        wire.push(TAG_BINARY);
        wire.extend_from_slice(&(file_data.len() as u64).to_be_bytes());
        wire.extend_from_slice(file_data);

        let mut cursor = std::io::Cursor::new(&wire);
        let mut dest = Vec::new();
        let obj = decode_from(&mut cursor, Some(&mut dest)).await.unwrap();

        assert_eq!(dest, file_data);
        assert_eq!(
            obj,
            PObject::Binary {
                length: file_data.len() as u64
            }
        );
        assert_eq!(cursor.position(), wire.len() as u64);
    }

    #[tokio::test]
    async fn binary_ex_streams_and_captures_send_hash() {
        // Wire order: "send_hash" before "binary" — decoder reads both.
        let file_data = b"hello world";
        let mut wire = Vec::new();

        wire.push(TAG_BINARY_EX);
        encode_string("send_hash", &mut wire).unwrap();
        encode_string("myhash", &mut wire).unwrap();
        encode_string("binary", &mut wire).unwrap();
        wire.push(TAG_BINARY);
        wire.extend_from_slice(&(file_data.len() as u64).to_be_bytes());
        wire.extend_from_slice(file_data);
        wire.push(TAG_END);

        let mut cursor = std::io::Cursor::new(&wire);
        let mut dest = Vec::new();
        let obj = decode_from(&mut cursor, Some(&mut dest)).await.unwrap();

        assert_eq!(dest, file_data);
        match obj {
            PObject::BinaryEx { length, send_hash } => {
                assert_eq!(length, file_data.len() as u64);
                assert_eq!(send_hash, "myhash");
            }
            other => panic!("expected BinaryEx, got {other:?}"),
        }
        // Entire wire consumed (no leftover data)
        assert_eq!(cursor.position(), wire.len() as u64);
    }

    #[tokio::test]
    async fn binary_ex_send_hash_after_binary() {
        // Wire order: "binary" before "send_hash" — decoder must continue
        // reading after streaming to capture send_hash.
        let file_data = b"data";
        let mut wire = Vec::new();

        wire.push(TAG_BINARY_EX);
        encode_string("binary", &mut wire).unwrap();
        wire.push(TAG_BINARY);
        wire.extend_from_slice(&(file_data.len() as u64).to_be_bytes());
        wire.extend_from_slice(file_data);
        encode_string("send_hash", &mut wire).unwrap();
        encode_string("posthash", &mut wire).unwrap();
        wire.push(TAG_END);

        let mut cursor = std::io::Cursor::new(&wire);
        let mut dest = Vec::new();
        let obj = decode_from(&mut cursor, Some(&mut dest)).await.unwrap();

        assert_eq!(dest, file_data);
        match obj {
            PObject::BinaryEx { length, send_hash } => {
                assert_eq!(length, file_data.len() as u64);
                assert_eq!(send_hash, "posthash");
            }
            other => panic!("expected BinaryEx, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn map_with_binary_preserves_trailing_fields() {
        // Simulates a download response: { "data" => Binary, "hash" => "abc" }
        let file_data = b"file content";
        let mut wire = Vec::new();

        wire.push(TAG_MAP);
        encode_string("data", &mut wire).unwrap();
        wire.push(TAG_BINARY);
        wire.extend_from_slice(&(file_data.len() as u64).to_be_bytes());
        wire.extend_from_slice(file_data);
        encode_string("hash", &mut wire).unwrap();
        encode_string("abc123", &mut wire).unwrap();
        encode_string("size", &mut wire).unwrap();
        encode(&PObject::Integer(file_data.len() as u64), &mut wire).unwrap();
        wire.push(TAG_END);

        let mut cursor = std::io::Cursor::new(&wire);
        let mut dest = Vec::new();
        let obj = decode_from(&mut cursor, Some(&mut dest)).await.unwrap();

        assert_eq!(dest, file_data);
        // Trailing fields are preserved
        assert_eq!(obj.get("hash").and_then(PObject::as_str), Some("abc123"));
        assert_eq!(
            obj.get("size").and_then(PObject::as_int),
            Some(file_data.len() as u64)
        );
        assert_eq!(
            obj.get("data"),
            Some(&PObject::Binary {
                length: file_data.len() as u64
            })
        );
    }
}

#[cfg(test)]
mod regression_tests {
    use super::*;
    use crate::pstream::encode;
    #[tokio::test]
    async fn multiple_binary_chunks_and_followup_remain_decodable() {
        let mut wire = vec![TAG_ARRAY];
        for bytes in [b"one", b"two"] {
            wire.push(TAG_BINARY);
            wire.extend_from_slice(&3u64.to_be_bytes());
            wire.extend_from_slice(bytes);
        }
        wire.push(TAG_END);
        encode(&PObject::Integer(17), &mut wire).unwrap();
        let mut reader = std::io::Cursor::new(wire);
        let mut out = Vec::new();
        decode_from(&mut reader, Some(&mut out)).await.unwrap();
        assert_eq!(out, b"onetwo");
        assert_eq!(
            decode_from(&mut reader, None).await.unwrap(),
            PObject::Integer(17)
        );
    }
    #[tokio::test]
    async fn nested_arrays_respect_depth_limit() {
        let wire = vec![TAG_ARRAY; MAX_DEPTH + 2];
        let mut r = std::io::Cursor::new(wire);
        assert!(
            matches!(decode_from(&mut r,None).await,Err(Error::Decode(s)) if s.contains("depth"))
        );
    }
    #[tokio::test]
    async fn duplicate_map_key_rejected() {
        let mut wire = vec![TAG_MAP];
        for _ in 0..2 {
            encode(&PObject::from("x"), &mut wire).unwrap();
            encode(&PObject::Integer(1), &mut wire).unwrap();
        }
        wire.push(TAG_END);
        assert!(
            decode_from(&mut std::io::Cursor::new(wire), None)
                .await
                .is_err()
        );
    }
    #[tokio::test]
    async fn binary_ex_without_payload_is_rejected() {
        let mut sink = Vec::new();
        assert!(
            decode_from(
                &mut std::io::Cursor::new([TAG_BINARY_EX, TAG_END]),
                Some(&mut sink)
            )
            .await
            .is_err()
        );
    }
}

#[cfg(test)]
mod resource_limit_tests {
    use super::*;
    #[tokio::test]
    async fn flat_collection_cannot_exhaust_memory() {
        let mut wire = vec![TAG_ARRAY];
        for _ in 0..=MAX_METADATA_VALUES {
            wire.extend_from_slice(&[TAG_NULL, 0]);
        }
        wire.push(TAG_END);
        assert!(
            matches!(decode_from(&mut std::io::Cursor::new(wire),None).await,Err(Error::Decode(s)) if s.contains("budget"))
        );
    }
    #[tokio::test]
    async fn valid_large_event_sized_response_decodes() {
        let mut wire = vec![TAG_ARRAY];
        for _ in 0..100_024 { wire.extend_from_slice(&[TAG_NULL, 0]); }
        wire.push(TAG_END);
        assert!(decode_from(&mut std::io::Cursor::new(wire), None).await.is_ok());
    }
    #[tokio::test]
    async fn bad_null_padding_is_rejected() {
        assert!(
            decode_from(&mut std::io::Cursor::new([TAG_NULL, 9]), None)
                .await
                .is_err()
        );
    }
}
