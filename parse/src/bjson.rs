//! BJSON parser (bjson.org spec v0.5).
//!
//! Binary JSON format, little-endian.

use crate::error::ParseError;
use std::collections::BTreeMap;

/// BJSON value (JSON-compatible).
#[derive(Clone, Debug, PartialEq)]
pub enum BjsonValue {
    /// Null.
    Null,
    /// Boolean.
    Bool(bool),
    /// Number.
    Number(f64),
    /// String.
    String(String),
    /// Binary data.
    Binary(Vec<u8>),
    /// Array.
    Array(Vec<BjsonValue>),
    /// Object.
    Object(BTreeMap<String, BjsonValue>),
}

/// Parses BJSON from bytes.
///
/// # Errors
///
/// Returns `ParseError` on invalid BJSON.
pub fn parse(data: &[u8]) -> Result<BjsonValue, ParseError> {
    let mut offset = 0;
    parse_value(data, &mut offset)
}

fn ensure_len(data: &[u8], offset: &mut usize, n: usize) -> Result<(), ParseError> {
    if *offset + n > data.len() {
        return Err(ParseError::Eof);
    }
    Ok(())
}

fn read_u8(data: &[u8], offset: &mut usize) -> Result<u8, ParseError> {
    ensure_len(data, offset, 1)?;
    let v = data[*offset];
    *offset += 1;
    Ok(v)
}

fn read_u16_le(data: &[u8], offset: &mut usize) -> Result<u16, ParseError> {
    ensure_len(data, offset, 2)?;
    let v = u16::from_le_bytes([data[*offset], data[*offset + 1]]);
    *offset += 2;
    Ok(v)
}

fn read_u32_le(data: &[u8], offset: &mut usize) -> Result<u32, ParseError> {
    ensure_len(data, offset, 4)?;
    let v = u32::from_le_bytes([
        data[*offset],
        data[*offset + 1],
        data[*offset + 2],
        data[*offset + 3],
    ]);
    *offset += 4;
    Ok(v)
}

fn read_u64_le(data: &[u8], offset: &mut usize) -> Result<u64, ParseError> {
    ensure_len(data, offset, 8)?;
    let v = u64::from_le_bytes([
        data[*offset],
        data[*offset + 1],
        data[*offset + 2],
        data[*offset + 3],
        data[*offset + 4],
        data[*offset + 5],
        data[*offset + 6],
        data[*offset + 7],
    ]);
    *offset += 8;
    Ok(v)
}

fn read_f32_le(data: &[u8], offset: &mut usize) -> Result<f32, ParseError> {
    ensure_len(data, offset, 4)?;
    let v = f32::from_le_bytes([
        data[*offset],
        data[*offset + 1],
        data[*offset + 2],
        data[*offset + 3],
    ]);
    *offset += 4;
    Ok(v)
}

fn read_f64_le(data: &[u8], offset: &mut usize) -> Result<f64, ParseError> {
    ensure_len(data, offset, 8)?;
    let v = f64::from_le_bytes([
        data[*offset],
        data[*offset + 1],
        data[*offset + 2],
        data[*offset + 3],
        data[*offset + 4],
        data[*offset + 5],
        data[*offset + 6],
        data[*offset + 7],
    ]);
    *offset += 8;
    Ok(v)
}

fn read_bytes(data: &[u8], offset: &mut usize, n: usize) -> Result<Vec<u8>, ParseError> {
    ensure_len(data, offset, n)?;
    let v = data[*offset..*offset + n].to_vec();
    *offset += n;
    Ok(v)
}

fn read_utf8(data: &[u8], offset: &mut usize, n: usize) -> Result<String, ParseError> {
    let bytes = read_bytes(data, offset, n)?;
    String::from_utf8(bytes).map_err(|e| ParseError::Io(e.to_string()))
}

fn parse_value(data: &[u8], offset: &mut usize) -> Result<BjsonValue, ParseError> {
    let tag = read_u8(data, offset)?;
    match tag {
        0 => Ok(BjsonValue::Null),
        1 => Ok(BjsonValue::Number(0.0)), // or false - prefer number per spec
        2 => Ok(BjsonValue::String(String::new())),
        3 => Ok(BjsonValue::Number(1.0)), // or true
        4 => Ok(BjsonValue::Number(read_u8(data, offset)? as f64)),
        5 => Ok(BjsonValue::Number(read_u16_le(data, offset)? as f64)),
        6 => Ok(BjsonValue::Number(read_u32_le(data, offset)? as f64)),
        7 => Ok(BjsonValue::Number(read_u64_le(data, offset)? as f64)),
        8 => Ok(BjsonValue::Number(-(read_u8(data, offset)? as i8) as f64)),
        9 => Ok(BjsonValue::Number(
            -(read_u16_le(data, offset)? as i16) as f64,
        )),
        10 => Ok(BjsonValue::Number(
            -(read_u32_le(data, offset)? as i32) as f64,
        )),
        11 => Ok(BjsonValue::Number(
            -(read_u64_le(data, offset)? as i64) as f64,
        )),
        14 => Ok(BjsonValue::Number(read_f32_le(data, offset)? as f64)),
        15 => Ok(BjsonValue::Number(read_f64_le(data, offset)?)),
        24 => Ok(BjsonValue::Bool(false)),
        25 => Ok(BjsonValue::Bool(true)),
        26 => Ok(BjsonValue::Number(0.0)),
        27 => Ok(BjsonValue::Number(1.0)),
        16 => {
            let n = read_u8(data, offset)? as usize;
            Ok(BjsonValue::String(read_utf8(data, offset, n)?))
        }
        17 => {
            let n = read_u16_le(data, offset)? as usize;
            Ok(BjsonValue::String(read_utf8(data, offset, n)?))
        }
        18 => {
            let n = read_u32_le(data, offset)? as usize;
            Ok(BjsonValue::String(read_utf8(data, offset, n)?))
        }
        19 => {
            let n = read_u64_le(data, offset)? as usize;
            if n > 4 * 1024 * 1024 * 1024 {
                return Err(ParseError::Unsupported("string too large".to_string()));
            }
            Ok(BjsonValue::String(read_utf8(data, offset, n)?))
        }
        20 => {
            let n = read_u8(data, offset)? as usize;
            Ok(BjsonValue::Binary(read_bytes(data, offset, n)?))
        }
        21 => {
            let n = read_u16_le(data, offset)? as usize;
            Ok(BjsonValue::Binary(read_bytes(data, offset, n)?))
        }
        22 => {
            let n = read_u32_le(data, offset)? as usize;
            if n > 256 * 1024 * 1024 {
                return Err(ParseError::Unsupported("binary too large".to_string()));
            }
            Ok(BjsonValue::Binary(read_bytes(data, offset, n)?))
        }
        23 => {
            let n = read_u64_le(data, offset)? as usize;
            if n > 256 * 1024 * 1024 {
                return Err(ParseError::Unsupported("binary too large".to_string()));
            }
            Ok(BjsonValue::Binary(read_bytes(data, offset, n)?))
        }
        32 => {
            let n = read_u8(data, offset)? as usize;
            let mut arr = Vec::with_capacity(n);
            for _ in 0..n {
                arr.push(parse_value(data, offset)?);
            }
            Ok(BjsonValue::Array(arr))
        }
        33 => {
            let n = read_u16_le(data, offset)? as usize;
            let mut arr = Vec::with_capacity(n.min(65536));
            for _ in 0..n {
                arr.push(parse_value(data, offset)?);
            }
            Ok(BjsonValue::Array(arr))
        }
        34 => {
            let n = read_u32_le(data, offset)? as usize;
            if n > 1_000_000 {
                return Err(ParseError::Unsupported("array too large".to_string()));
            }
            let mut arr = Vec::with_capacity(n.min(1_000_000));
            for _ in 0..n {
                arr.push(parse_value(data, offset)?);
            }
            Ok(BjsonValue::Array(arr))
        }
        35 => {
            let n = read_u64_le(data, offset)? as usize;
            if n > 1_000_000 {
                return Err(ParseError::Unsupported("array too large".to_string()));
            }
            let mut arr = Vec::with_capacity(n.min(1_000_000));
            for _ in 0..n {
                arr.push(parse_value(data, offset)?);
            }
            Ok(BjsonValue::Array(arr))
        }
        36 => {
            let n = read_u8(data, offset)? as usize;
            parse_object(data, offset, n)
        }
        37 => {
            let n = read_u16_le(data, offset)? as usize;
            parse_object(data, offset, n)
        }
        38 => {
            let n = read_u32_le(data, offset)? as usize;
            if n > 100_000 {
                return Err(ParseError::Unsupported("object too large".to_string()));
            }
            parse_object(data, offset, n)
        }
        39 => {
            let n = read_u64_le(data, offset)? as usize;
            if n > 100_000 {
                return Err(ParseError::Unsupported("object too large".to_string()));
            }
            parse_object(data, offset, n)
        }
        _ => Err(ParseError::Syntax {
            filename: None,
            row: 0,
            col: *offset,
            msg: format!("unknown BJSON type tag {}", tag),
        }),
    }
}

fn parse_object(data: &[u8], offset: &mut usize, n: usize) -> Result<BjsonValue, ParseError> {
    let mut map = BTreeMap::new();
    for _ in 0..n {
        let key_val = parse_value(data, offset)?;
        let key = match key_val {
            BjsonValue::String(s) => s,
            _ => {
                return Err(ParseError::Syntax {
                    filename: None,
                    row: 0,
                    col: *offset,
                    msg: "object key must be string".to_string(),
                });
            }
        };
        let value = parse_value(data, offset)?;
        map.insert(key, value);
    }
    Ok(BjsonValue::Object(map))
}
