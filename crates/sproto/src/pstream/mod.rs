use std::collections::BTreeMap;

use crate::error::{Error, Result};

const TAG_END: u8 = 0x40;
const TAG_MAP: u8 = 0x42;
const TAG_NULL: u8 = 0x00;
const TAG_ARRAY: u8 = 0x41;
const TAG_STRING: u8 = 0x10;
const TAG_BINARY: u8 = 0x30;
const TAG_INTEGER: u8 = 0x01;
const TAG_BINARY_EX: u8 = 0x43;

// Bound recursive async polling on normal Tokio thread stacks.
const MAX_DEPTH: usize = 32;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PObject {
    Null,
    Str(String),
    Integer(u64),
    Array(Vec<Self>),
    Map(BTreeMap<String, Self>),
    /// Binary data marker (tag 0x30). The actual bytes are streamed externally.
    Binary {
        length: u64,
    },
    /// Extended binary marker (tag 0x43). Data + `send_hash`, streamed externally.
    BinaryEx {
        length: u64,
        send_hash: String,
    },
}

impl From<&str> for PObject {
    fn from(s: &str) -> Self {
        Self::Str(s.to_string())
    }
}

impl From<String> for PObject {
    fn from(s: String) -> Self {
        Self::Str(s)
    }
}

impl From<u64> for PObject {
    fn from(v: u64) -> Self {
        Self::Integer(v)
    }
}

impl From<bool> for PObject {
    fn from(v: bool) -> Self {
        Self::Integer(u64::from(v))
    }
}

impl From<Vec<Self>> for PObject {
    fn from(v: Vec<Self>) -> Self {
        Self::Array(v)
    }
}

impl From<BTreeMap<String, Self>> for PObject {
    fn from(m: BTreeMap<String, Self>) -> Self {
        Self::Map(m)
    }
}

impl PObject {
    #[must_use]
    pub const fn as_map(&self) -> Option<&BTreeMap<String, Self>> {
        match self {
            Self::Map(m) => Some(m),
            _ => None,
        }
    }

    #[must_use]
    pub(crate) const fn as_map_mut(&mut self) -> Option<&mut BTreeMap<String, Self>> {
        match self {
            Self::Map(m) => Some(m),
            _ => None,
        }
    }

    #[must_use]
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::Str(s) => Some(s),
            _ => None,
        }
    }

    #[must_use]
    pub const fn as_int(&self) -> Option<u64> {
        match self {
            Self::Integer(v) => Some(*v),
            _ => None,
        }
    }

    #[must_use]
    pub fn as_array(&self) -> Option<&[Self]> {
        match self {
            Self::Array(a) => Some(a),
            _ => None,
        }
    }

    #[must_use]
    pub fn get(&self, key: &str) -> Option<&Self> {
        self.as_map()?.get(key)
    }
}

#[cfg(test)]
impl std::ops::Index<&str> for PObject {
    type Output = Self;

    fn index(&self, key: &str) -> &Self {
        self.get(key)
            .unwrap_or_else(|| panic!("PObject: missing key \"{key}\""))
    }
}

/// Build a `PObject::Map` ergonomically.
///
/// ```ignore
/// let obj = pmap! {
///     "view_id" => 1u64,
///     "enabled" => true,
///     "_action" => "download",
/// };
/// ```
macro_rules! pmap {
    ($($key:expr => $val:expr),* $(,)?) => {{
        #[allow(unused_mut, reason = "mut needed when macro is invoked with entries")]
        let mut map = std::collections::BTreeMap::new();
        $(
            map.insert($key.to_string(), $crate::pstream::PObject::from($val));
        )*
        $crate::pstream::PObject::Map(map)
    }};
}

mod decode;
mod encode;

pub use decode::decode_from;
pub(crate) use encode::binary_parts;
pub use encode::encode;

/// Returns true if the `PObject` is a keep-alive message: `{"type": "keep_alive"}`
#[must_use]
pub fn is_keep_alive(obj: &PObject) -> bool {
    obj.get("type")
        .and_then(|v| v.as_str())
        .is_some_and(|s| s == "keep_alive")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Decode a `PObject` from a byte buffer using the production async decoder.
    /// Returns the decoded object and the number of bytes consumed.
    async fn decode(buf: &[u8]) -> Result<(PObject, u64)> {
        let mut cursor = std::io::Cursor::new(buf);
        let obj = decode_from(&mut cursor, None).await?;
        Ok((obj, cursor.position()))
    }

    #[tokio::test]
    async fn roundtrip_null() {
        let obj = PObject::Null;
        let mut buf = Vec::new();
        encode(&obj, &mut buf).unwrap();
        assert_eq!(buf, [0x00, 0x00]);
        let (decoded, consumed) = decode(&buf).await.unwrap();
        assert_eq!(decoded, obj);
        assert_eq!(consumed, 2);
    }

    #[tokio::test]
    async fn roundtrip_integers() {
        // Zero: 1 byte
        let mut buf = Vec::new();
        encode(&PObject::Integer(0), &mut buf).unwrap();
        assert_eq!(buf, [0x01, 0x01, 0x00]);
        assert_eq!(decode(&buf).await.unwrap(), (PObject::Integer(0), 3));

        // 0xFF: 1 byte
        buf.clear();
        encode(&PObject::Integer(0xFF), &mut buf).unwrap();
        assert_eq!(buf, [0x01, 0x01, 0xFF]);
        assert_eq!(decode(&buf).await.unwrap(), (PObject::Integer(0xFF), 3));

        // 0x100: 2 bytes
        buf.clear();
        encode(&PObject::Integer(0x100), &mut buf).unwrap();
        assert_eq!(buf, [0x01, 0x02, 0x01, 0x00]);
        assert_eq!(decode(&buf).await.unwrap(), (PObject::Integer(0x100), 4));

        // 0xFFFF: 2 bytes
        buf.clear();
        encode(&PObject::Integer(0xFFFF), &mut buf).unwrap();
        assert_eq!(buf, [0x01, 0x02, 0xFF, 0xFF]);
        assert_eq!(decode(&buf).await.unwrap(), (PObject::Integer(0xFFFF), 4));

        // 0x10000: 4 bytes
        buf.clear();
        encode(&PObject::Integer(0x10000), &mut buf).unwrap();
        assert_eq!(buf, [0x01, 0x04, 0x00, 0x01, 0x00, 0x00]);
        assert_eq!(decode(&buf).await.unwrap(), (PObject::Integer(0x10000), 6));

        // 0xFFFF_FFFF: 4 bytes
        buf.clear();
        encode(&PObject::Integer(0xFFFF_FFFF), &mut buf).unwrap();
        assert_eq!(buf, [0x01, 0x04, 0xFF, 0xFF, 0xFF, 0xFF]);
        assert_eq!(
            decode(&buf).await.unwrap(),
            (PObject::Integer(0xFFFF_FFFF), 6)
        );

        // 0x1_0000_0000: 8 bytes
        buf.clear();
        encode(&PObject::Integer(0x1_0000_0000), &mut buf).unwrap();
        assert_eq!(
            buf,
            [0x01, 0x08, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00]
        );
        assert_eq!(
            decode(&buf).await.unwrap(),
            (PObject::Integer(0x1_0000_0000), 10)
        );
    }

    #[tokio::test]
    async fn roundtrip_string() {
        let obj = PObject::Str("hello".into());
        let mut buf = Vec::new();
        encode(&obj, &mut buf).unwrap();
        assert_eq!(buf, [0x10, 0x00, 0x05, b'h', b'e', b'l', b'l', b'o']);
        let (decoded, consumed) = decode(&buf).await.unwrap();
        assert_eq!(decoded, obj);
        assert_eq!(consumed, 8);
    }

    #[tokio::test]
    async fn roundtrip_empty_string() {
        let obj = PObject::Str(String::new());
        let mut buf = Vec::new();
        encode(&obj, &mut buf).unwrap();
        assert_eq!(buf, [0x10, 0x00, 0x00]);
        let (decoded, consumed) = decode(&buf).await.unwrap();
        assert_eq!(decoded, obj);
        assert_eq!(consumed, 3);
    }

    #[tokio::test]
    async fn roundtrip_array() {
        let obj = PObject::Array(vec![
            PObject::Integer(1),
            PObject::Str("two".into()),
            PObject::Null,
        ]);
        let mut buf = Vec::new();
        encode(&obj, &mut buf).unwrap();
        let (decoded, consumed) = decode(&buf).await.unwrap();
        assert_eq!(decoded, obj);
        assert_eq!(consumed, buf.len() as u64);
    }

    #[tokio::test]
    async fn roundtrip_map() {
        // Keys without underscore — no stripping
        let mut map = BTreeMap::new();
        map.insert("name".to_string(), PObject::Str("test".into()));
        map.insert("count".to_string(), PObject::Integer(42));
        let obj = PObject::Map(map);

        let mut buf = Vec::new();
        encode(&obj, &mut buf).unwrap();
        let (decoded, _) = decode(&buf).await.unwrap();
        assert_eq!(decoded, obj);
    }

    #[tokio::test]
    async fn underscore_stripping() {
        let obj = pmap! {
            "_action" => "test",
        };
        let mut buf = Vec::new();
        encode(&obj, &mut buf).unwrap();

        // Wire should contain "action" (6 bytes), not "_action" (7 bytes)
        // Map: 0x42, String: 0x10 0x00 0x06 "action", ...
        assert_eq!(buf[0], TAG_MAP);
        assert_eq!(buf[1], TAG_STRING);
        assert_eq!(buf[2..4], [0x00, 0x06]); // length 6
        assert_eq!(&buf[4..10], b"action");

        // Decode: the key comes back as "action" (no underscore)
        let (decoded, _) = decode(&buf).await.unwrap();
        let map = decoded.as_map().unwrap();
        assert!(map.contains_key("action"));
        assert!(!map.contains_key("_action"));
    }

    #[tokio::test]
    async fn nested_structures() {
        let obj = pmap! {
            "outer" => PObject::Array(vec![
                pmap! {
                    "inner_key" => "inner_value",
                },
                PObject::Integer(99),
            ]),
        };
        let mut buf = Vec::new();
        encode(&obj, &mut buf).unwrap();
        let (decoded, consumed) = decode(&buf).await.unwrap();
        assert_eq!(consumed, buf.len() as u64);

        // Verify nested access
        let arr = decoded["outer"].as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0]["inner_key"].as_str().unwrap(), "inner_value");
        assert_eq!(arr[1].as_int().unwrap(), 99);
    }

    #[test]
    fn pmap_macro() {
        let obj = pmap! {
            "action" => "download",
            "view_id" => 1u64,
            "enabled" => true,
        };
        assert_eq!(obj["action"].as_str().unwrap(), "download");
        assert_eq!(obj["view_id"].as_int().unwrap(), 1);
        assert_eq!(obj["enabled"].as_int().unwrap(), 1);
    }

    #[test]
    fn is_keep_alive_detection() {
        let ka = pmap! { "type" => "keep_alive" };
        assert!(is_keep_alive(&ka));

        let not_ka = pmap! { "type" => "response" };
        assert!(!is_keep_alive(&not_ka));

        assert!(!is_keep_alive(&PObject::Null));
    }

    #[tokio::test]
    async fn golden_vector_update_settings() {
        // Build the update_settings request structure from the Pwn2Own research dump.
        // Fields: @proto { type, date, version { major: 7, minor: 0 }, body-continue },
        //         _action = "update_settings"
        let obj = pmap! {
            "@proto" => pmap! {
                "body-continue" => false,
                "date" => 1_711_234_567_u64,
                "type" => "header",
                "version" => pmap! {
                    "major" => 7u64,
                    "minor" => 0u64,
                },
            },
            "_action" => "update_settings",
        };

        let mut buf = Vec::new();
        encode(&obj, &mut buf).unwrap();

        // Verify structural correctness:
        // Outer map starts with 0x42
        assert_eq!(buf[0], TAG_MAP);

        // Decode round-trip
        let (decoded, consumed) = decode(&buf).await.unwrap();
        assert_eq!(consumed, buf.len() as u64);

        // @proto is NOT underscore-stripped (no leading _)
        let proto = decoded.get("@proto").expect("@proto key must exist");
        assert_eq!(proto["type"].as_str().unwrap(), "header");
        assert_eq!(proto["date"].as_int().unwrap(), 1_711_234_567);
        assert_eq!(proto["version"]["major"].as_int().unwrap(), 7);
        assert_eq!(proto["version"]["minor"].as_int().unwrap(), 0);
        // body-continue is boolean false = Integer(0)
        assert_eq!(proto["body-continue"].as_int().unwrap(), 0);

        // _action is stripped to "action" on wire
        assert_eq!(decoded["action"].as_str().unwrap(), "update_settings");
    }

    #[tokio::test]
    async fn roundtrip_u64_max() {
        let obj = PObject::Integer(u64::MAX);
        let mut buf = Vec::new();
        encode(&obj, &mut buf).unwrap();
        assert_eq!(buf[1], 0x08); // 8-byte encoding
        let (decoded, consumed) = decode(&buf).await.unwrap();
        assert_eq!(decoded, obj);
        assert_eq!(consumed, 10);
    }

    #[tokio::test]
    async fn roundtrip_unicode_string() {
        let obj = PObject::Str("hello 世界 🦀".into());
        let mut buf = Vec::new();
        encode(&obj, &mut buf).unwrap();
        let (decoded, _) = decode(&buf).await.unwrap();
        assert_eq!(decoded, obj);
    }

    #[tokio::test]
    async fn roundtrip_empty_array() {
        let obj = PObject::Array(vec![]);
        let mut buf = Vec::new();
        encode(&obj, &mut buf).unwrap();
        assert_eq!(buf, [TAG_ARRAY, TAG_END]);
        let (decoded, consumed) = decode(&buf).await.unwrap();
        assert_eq!(decoded, obj);
        assert_eq!(consumed, 2);
    }

    #[tokio::test]
    async fn roundtrip_empty_map() {
        let obj = PObject::Map(BTreeMap::new());
        let mut buf = Vec::new();
        encode(&obj, &mut buf).unwrap();
        assert_eq!(buf, [TAG_MAP, TAG_END]);
        let (decoded, consumed) = decode(&buf).await.unwrap();
        assert_eq!(decoded, obj);
        assert_eq!(consumed, 2);
    }

    #[tokio::test]
    async fn underscore_not_stripped_from_values() {
        // Only map *keys* starting with _ are stripped, not string values
        let obj = pmap! { "key" => "_still_has_underscore" };
        let mut buf = Vec::new();
        encode(&obj, &mut buf).unwrap();
        let (decoded, _) = decode(&buf).await.unwrap();
        assert_eq!(decoded["key"].as_str().unwrap(), "_still_has_underscore");
    }

    #[tokio::test]
    async fn underscore_stripping_multiple_keys() {
        let obj = pmap! {
            "_action" => "test",
            "_agent" => "bot",
            "session" => "abc",
        };
        let mut buf = Vec::new();
        encode(&obj, &mut buf).unwrap();
        let (decoded, _) = decode(&buf).await.unwrap();
        let map = decoded.as_map().unwrap();

        assert!(map.contains_key("action"));
        assert!(map.contains_key("agent"));
        assert!(map.contains_key("session"));
        assert!(!map.contains_key("_action"));
        assert!(!map.contains_key("_agent"));
    }

    #[tokio::test]
    async fn at_proto_key_not_stripped() {
        // @ prefix is not _ prefix — should be kept as-is
        let obj = pmap! { "@proto" => "header" };
        let mut buf = Vec::new();
        encode(&obj, &mut buf).unwrap();
        let (decoded, _) = decode(&buf).await.unwrap();
        assert!(decoded.get("@proto").is_some());
    }

    #[test]
    fn accessors_return_none_on_wrong_type() {
        assert!(PObject::Null.as_map().is_none());
        assert!(PObject::Null.as_str().is_none());
        assert!(PObject::Null.as_int().is_none());
        assert!(PObject::Null.as_array().is_none());
        assert!(PObject::Null.get("key").is_none());

        assert!(PObject::Integer(1).as_str().is_none());
        assert!(PObject::Str("hi".into()).as_int().is_none());
    }

    #[test]
    fn from_bool_conversion() {
        assert_eq!(PObject::from(true), PObject::Integer(1));
        assert_eq!(PObject::from(false), PObject::Integer(0));
    }

    #[tokio::test]
    async fn spec_proto_envelope_structure() {
        // Spec §3.4: every modern protocol message carries @proto metadata.
        // Verify we can build and round-trip the exact structure from the spec.
        let obj = pmap! {
            "@proto" => pmap! {
                "type" => "header",
                "date" => 1_711_234_567_u64,
                "version" => pmap! {
                    "major" => 7u64,
                    "minor" => 0u64,
                },
                "body-continue" => false,
            },
            "_action" => "download",
            "_agent" => pmap! {
                "platform" => "mac",
                "type" => "drive",
                "device_uuid" => "2e7ef840-test",
                "restore_id" => "df50e0fe-test",
                "version" => pmap! {
                    "major" => 4u64,
                    "minor" => 0u64,
                    "mini" => 0u64,
                    "build" => 17889u64,
                },
            },
            "session" => "04f7ffbd-test",
            "view_id" => 1u64,
        };

        let mut buf = Vec::new();
        encode(&obj, &mut buf).unwrap();
        let (decoded, consumed) = decode(&buf).await.unwrap();
        assert_eq!(consumed, buf.len() as u64);

        // @proto key is NOT underscore-stripped
        assert!(decoded.get("@proto").is_some());
        // _action and _agent ARE stripped
        assert!(decoded.get("action").is_some());
        assert!(decoded.get("agent").is_some());
        assert!(decoded.get("_action").is_none());
        assert!(decoded.get("_agent").is_none());

        // Verify nested values match spec §3.4
        let proto = &decoded["@proto"];
        assert_eq!(proto["type"].as_str().unwrap(), "header");
        assert_eq!(proto["version"]["major"].as_int().unwrap(), 7);
        assert_eq!(proto["version"]["minor"].as_int().unwrap(), 0);
        assert_eq!(proto["body-continue"].as_int().unwrap(), 0);

        let agent = &decoded["agent"];
        assert_eq!(agent["platform"].as_str().unwrap(), "mac");
        assert_eq!(agent["type"].as_str().unwrap(), "drive");
        assert_eq!(agent["version"]["build"].as_int().unwrap(), 17889);
    }

    #[test]
    fn spec_version_byte() {
        // Spec §2.1 / §13: PROTO_VERSION = 0x46 (70 decimal) = protocol 7.0
        assert_eq!(crate::frame::VERSION, 0x46);
        assert_eq!(crate::frame::VERSION, 70);
    }

    #[test]
    fn spec_magic_bytes() {
        // Spec §2.1: magic = 0x25521814, on wire as [0x25, 0x52, 0x18, 0x14]
        let magic_bytes = crate::frame::MAGIC.to_be_bytes();
        assert_eq!(magic_bytes, [0x25, 0x52, 0x18, 0x14]);
    }

    #[test]
    fn spec_keep_alive_message() {
        // Spec §3.6: keep-alive messages have {"type": "keep_alive"}
        let ka = pmap! { "type" => "keep_alive" };
        assert!(is_keep_alive(&ka));

        // Other "type" values are not keep-alive
        let resp = pmap! { "type" => "response" };
        assert!(!is_keep_alive(&resp));

        // Missing "type" field is not keep-alive
        let no_type = pmap! { "action" => "test" };
        assert!(!is_keep_alive(&no_type));
    }
}
