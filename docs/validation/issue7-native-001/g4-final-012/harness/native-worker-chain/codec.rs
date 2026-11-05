//! Independent bounded pstream codec, adapted from the sibling Client harness.
use serde_json::{Value, json};
use std::io::{Read, Write};
use crate::{HResult, require};
fn io<T>(result: std::io::Result<T>) -> HResult<T> { result.map_err(|e| format!("io:{:?}:{e}",e.kind())) }
fn byte<R: Read>(stream: &mut R) -> HResult<u8> {
    let mut data = [0]; io(stream.read_exact(&mut data))?; Ok(data[0])
}
fn bytes<const N: usize, R: Read>(stream: &mut R) -> HResult<[u8; N]> {
    let mut data = [0; N]; io(stream.read_exact(&mut data))?; Ok(data)
}
fn decode<R: Read>(stream: &mut R, first: u8, depth: usize, budget: &mut usize) -> HResult<Value> {
    require(depth <= 24 && *budget > 0, "request nesting/value budget")?;
    *budget -= 1;
    match first {
        0x00 => { require(byte(stream)? == 0, "null padding")?; Ok(Value::Null) }
        0x01 => {
            let value = match byte(stream)? {
                1 => u64::from(byte(stream)?),
                2 => u64::from(u16::from_be_bytes(bytes(stream)?)),
                4 => u64::from(u32::from_be_bytes(bytes(stream)?)),
                8 => u64::from_be_bytes(bytes(stream)?),
                _ => return Err("unsupported integer width".into()),
            }; Ok(json!(value))
        }
        0x10 => {
            let length = usize::from(u16::from_be_bytes(bytes(stream)?));
            require(length <= 16384, "request string budget")?;
            let mut data = vec![0; length]; io(stream.read_exact(&mut data))?;
            Ok(Value::String(String::from_utf8(data).map_err(|e| e.to_string())?))
        }
        0x41 => {
            let mut values = Vec::new();
            loop { let tag = byte(stream)?; if tag == 0x40 { break; }
                values.push(decode(stream, tag, depth + 1, budget)?); }
            Ok(Value::Array(values))
        }
        0x42 => {
            let mut values = serde_json::Map::new();
            loop {
                let tag = byte(stream)?; if tag == 0x40 { break; }
                require(tag == 0x10, "map key type")?;
                let key = decode(stream, tag, depth + 1, budget)?;
                let key = key.as_str().ok_or("map key missing")?.to_owned();
                let tag = byte(stream)?;
                require(!values.contains_key(&key), "duplicate map key")?;
                values.insert(key, decode(stream, tag, depth + 1, budget)?);
            } Ok(Value::Object(values))
        }
        _ => Err(format!("unsupported request tag {first:#x}")),
    }
}
fn encode(value: &Value, output: &mut Vec<u8>) -> HResult<()> {
    match value {
        Value::Null => output.extend_from_slice(&[0, 0]),
        Value::Bool(value) => output.extend_from_slice(&[1, 1, u8::from(*value)]),
        Value::Number(number) => {
            let value = number.as_u64().ok_or("response integer is not unsigned")?;
            let width: u8 = if value < 0x100 { 1 } else if value < 0x10000 { 2 } else if value < 0x1_0000_0000 { 4 } else { 8 };
            output.extend_from_slice(&[1, width]);
            output.extend_from_slice(&value.to_be_bytes()[8 - usize::from(width)..]);
        }
        Value::String(value) => {
            let length = u16::try_from(value.len()).map_err(|e| e.to_string())?;
            output.push(0x10); output.extend_from_slice(&length.to_be_bytes()); output.extend_from_slice(value.as_bytes());
        }
        Value::Array(values) => { output.push(0x41); for value in values { encode(value, output)?; } output.push(0x40); }
        Value::Object(values) => {
            output.push(0x42);
            for (key, value) in values { encode(&Value::String(key.clone()), output)?; encode(value, output)?; }
            output.push(0x40);
        }
    } Ok(())
}
pub fn request<R: Read>(stream: &mut R) -> HResult<(u8, Value)> {
    let header: [u8; 8] = bytes(stream)?;
    require(header[..4] == [0x25, 0x52, 0x18, 0x14] && header[4] == 0x46 && header[6..] == [0, 0], "native frame header")?;
    let tag = byte(stream)?;
    Ok((header[5], decode(stream, tag, 0, &mut 4096)?))
}
pub fn respond<W: Write>(stream: &mut W, response: &Value) -> HResult<()> {
    let mut bytes = Vec::new(); encode(response, &mut bytes)?;
    io(stream.write_all(&bytes))?; io(stream.flush())
}
