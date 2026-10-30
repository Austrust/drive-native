use crate::pstream::{
    Error, PObject, Result, TAG_ARRAY, TAG_END, TAG_INTEGER, TAG_MAP, TAG_NULL, TAG_STRING,
};

/// Encode a `PObject` into the `PStream` wire format.
///
/// Returns an error if a string exceeds the protocol's 65 535-byte limit
/// or if the object contains `Binary`/`BinaryEx` (which must be streamed
/// at the channel level).
pub fn encode(obj: &PObject, buf: &mut Vec<u8>) -> Result<()> {
    match obj {
        PObject::Null => {
            buf.push(TAG_NULL);
            buf.push(0x00);
        }
        PObject::Str(s) => encode_string(s, buf)?,
        PObject::Integer(v) => encode_integer(*v, buf),
        PObject::Array(items) => {
            buf.push(TAG_ARRAY);
            for item in items {
                encode(item, buf)?;
            }
            buf.push(TAG_END);
        }
        PObject::Map(map) => {
            buf.push(TAG_MAP);
            for (key, value) in map {
                // Strip leading underscore from keys on the wire
                let wire_key = key.strip_prefix('_').unwrap_or(key);
                encode_string(wire_key, buf)?;
                encode(value, buf)?;
            }
            buf.push(TAG_END);
        }
        PObject::Binary { .. } | PObject::BinaryEx { .. } => {
            return Err(Error::Decode(
                "Binary/BinaryEx must be streamed at the channel level, not encoded via PStream"
                    .into(),
            ));
        }
    }
    Ok(())
}

#[allow(
    clippy::cast_possible_truncation,
    reason = "Truncation is guarded by range checks"
)]
fn encode_integer(v: u64, buf: &mut Vec<u8>) {
    buf.push(TAG_INTEGER);

    if v < 0x100 {
        buf.push(0x01);
        buf.push(v as u8);
    } else if v < 0x10000 {
        buf.push(0x02);
        buf.extend_from_slice(&(v as u16).to_be_bytes());
    } else if v < 0x1_0000_0000 {
        buf.push(0x04);
        buf.extend_from_slice(&(v as u32).to_be_bytes());
    } else {
        buf.push(0x08);
        buf.extend_from_slice(&v.to_be_bytes());
    }
}

/// Metadata surrounding exactly one streamed Binary. Contents remain unbuffered.
pub(crate) fn binary_parts(obj: &PObject) -> Result<(Vec<u8>, Vec<u8>, u64)> {
    fn walk(
        o: &PObject,
        out: &mut Vec<u8>,
        split: &mut Option<(usize, u64)>,
        depth: usize,
    ) -> Result<()> {
        if depth > 32 {
            return Err(Error::Decode("upload metadata depth exceeded".into()));
        }
        match o {
            PObject::Binary { length } => {
                if split.is_some() {
                    return Err(Error::Decode("multiple upload payloads".into()));
                }
                out.push(super::TAG_BINARY);
                out.extend_from_slice(&length.to_be_bytes());
                *split = Some((out.len(), *length));
            }
            PObject::Map(m) => {
                out.push(TAG_MAP);
                for (k, v) in m {
                    encode_string(k.strip_prefix('_').unwrap_or(k), out)?;
                    walk(v, out, split, depth + 1)?;
                }
                out.push(TAG_END);
            }
            PObject::Array(a) => {
                out.push(TAG_ARRAY);
                for v in a {
                    walk(v, out, split, depth + 1)?;
                }
                out.push(TAG_END);
            }
            _ => encode(o, out)?,
        }
        if out.len() > 1024 * 1024 {
            return Err(Error::Decode("upload metadata exceeds budget".into()));
        }
        Ok(())
    }
    let mut metadata = Vec::new();
    let mut split = None;
    walk(obj, &mut metadata, &mut split, 0)?;
    let (at, length) = split.ok_or_else(|| Error::Decode("missing upload payload".into()))?;
    let suffix = metadata.split_off(at);
    Ok((metadata, suffix, length))
}

pub fn encode_string(s: &str, buf: &mut Vec<u8>) -> Result<()> {
    if u16::try_from(s.len()).is_err() {
        return Err(Error::Decode(format!(
            "string too long for PStream encoding: {} bytes (max {})",
            s.len(),
            u16::MAX
        )));
    }

    buf.push(TAG_STRING);
    #[allow(
        clippy::cast_possible_truncation,
        reason = "length checked above to fit in u16"
    )]
    buf.extend_from_slice(&(s.len() as u16).to_be_bytes());
    buf.extend_from_slice(s.as_bytes());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spec_null_wire_format() {
        // Spec: Null = [0x00][0x00] (tag + zero-length indicator)
        let mut buf = Vec::new();
        encode(&PObject::Null, &mut buf).unwrap();
        assert_eq!(buf, [0x00, 0x00]);
    }

    #[test]
    fn spec_integer_minimum_width() {
        // Spec: integers use minimum bytes needed.
        // Boundary values from §3.2 integer encoding table.
        let cases: &[(u64, u8, &[u8])] = &[
            // < 0x100 → 1 byte
            (0, 0x01, &[0x00]),
            (127, 0x01, &[0x7F]),
            (255, 0x01, &[0xFF]),
            // < 0x10000 → 2 bytes
            (256, 0x02, &[0x01, 0x00]),
            (0xFFFF, 0x02, &[0xFF, 0xFF]),
            // < 0x100000000 → 4 bytes
            (0x1_0000, 0x04, &[0x00, 0x01, 0x00, 0x00]),
            (0xFFFF_FFFF, 0x04, &[0xFF, 0xFF, 0xFF, 0xFF]),
            // >= 0x100000000 → 8 bytes
            (
                0x1_0000_0000,
                0x08,
                &[0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00],
            ),
            (
                u64::MAX,
                0x08,
                &[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF],
            ),
        ];

        for &(value, expected_size, expected_bytes) in cases {
            let mut buf = Vec::new();
            encode(&PObject::Integer(value), &mut buf).unwrap();
            assert_eq!(buf[0], TAG_INTEGER, "tag for value {value}");
            assert_eq!(buf[1], expected_size, "size byte for value {value}");
            assert_eq!(&buf[2..], expected_bytes, "value bytes for {value}");
        }
    }

    #[test]
    fn spec_string_wire_format() {
        // Spec: String = [0x10][uint16_be length][utf8_bytes]
        let mut buf = Vec::new();
        encode(&PObject::Str("hello".into()), &mut buf).unwrap();
        assert_eq!(buf[0], 0x10); // tag
        assert_eq!(buf[1..3], [0x00, 0x05]); // u16 BE length = 5
        assert_eq!(&buf[3..], b"hello"); // UTF-8 bytes
    }

    #[test]
    fn spec_underscore_stripping_rule() {
        // Spec §3.2 / §12.3: "String keys starting with `_` have the leading
        // underscore stripped during serialization — the length is decremented
        // by 1 and the pointer advanced by 1."
        let obj = pmap! { "_action" => "download" };
        let mut buf = Vec::new();
        encode(&obj, &mut buf).unwrap();

        // Find the key in the buffer (after MAP tag 0x42)
        assert_eq!(buf[0], TAG_MAP);
        assert_eq!(buf[1], TAG_STRING);
        let key_len = u16::from_be_bytes([buf[2], buf[3]]);
        assert_eq!(key_len, 6); // "action" not "_action"
        assert_eq!(&buf[4..10], b"action");
    }

    #[test]
    fn spec_map_wire_format() {
        // Spec: Map = [0x42][key-value pairs...][0x40]
        // Keys are strings, values are any type.
        let obj = pmap! { "key" => 1u64 };
        let mut buf = Vec::new();
        encode(&obj, &mut buf).unwrap();

        assert_eq!(buf[0], TAG_MAP); // 0x42
        assert_eq!(buf[1], TAG_STRING); // key tag
        // ... key bytes ...
        assert_eq!(*buf.last().unwrap(), TAG_END); // 0x40
    }

    #[test]
    fn spec_array_wire_format() {
        // Spec: Array = [0x41][elements...][0x40]
        let obj = PObject::Array(vec![PObject::Integer(1), PObject::Integer(2)]);
        let mut buf = Vec::new();
        encode(&obj, &mut buf).unwrap();

        assert_eq!(buf[0], TAG_ARRAY); // 0x41
        assert_eq!(*buf.last().unwrap(), TAG_END); // 0x40
    }
}
