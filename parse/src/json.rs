//! JSON parser (hand-rolled recursive descent).
//!
//! Supports objects, arrays, strings, numbers, booleans, null.

use crate::error::ParseError;
use crate::parser::Parser;
use std::collections::BTreeMap;

/// JSON value.
#[derive(Clone, Debug, PartialEq)]
pub enum JsonValue {
    /// Null.
    Null,
    /// Boolean.
    Bool(bool),
    /// Number (f64).
    Number(f64),
    /// String.
    String(String),
    /// Array.
    Array(Vec<JsonValue>),
    /// Object.
    Object(BTreeMap<String, JsonValue>),
}

/// Parses JSON from bytes.
///
/// # Errors
///
/// Returns `ParseError` on invalid JSON.
pub fn parse(data: &[u8]) -> Result<JsonValue, ParseError> {
    let s = std::str::from_utf8(data).map_err(|e| ParseError::Io(e.to_string()))?;
    parse_str(s)
}

/// Parses JSON from string.
pub fn parse_str(s: &str) -> Result<JsonValue, ParseError> {
    let mut p = Parser::new(s.as_bytes(), None);
    skip_whitespace(&mut p);
    let v = parse_value(&mut p)?;
    skip_whitespace(&mut p);
    if !p.is_eof() {
        return Err(p.syntax_err("trailing data after JSON value"));
    }
    Ok(v)
}

fn skip_whitespace(p: &mut Parser<'_>) {
    while p.one_of(b" \t\n\r") {
        p.advance();
    }
}

fn parse_value(p: &mut Parser<'_>) -> Result<JsonValue, ParseError> {
    skip_whitespace(p);
    if p.is_eof() {
        return Err(ParseError::Eof);
    }
    let c = p.peek();
    match c {
        b'n' => parse_null(p),
        b't' | b'f' => parse_bool(p),
        b'"' => parse_string(p),
        b'[' => parse_array(p),
        b'{' => parse_object(p),
        b'-' | b'0'..=b'9' => parse_number(p),
        _ => Err(p.syntax_err(format!("unexpected '{}'", c as char))),
    }
}

fn parse_null(p: &mut Parser<'_>) -> Result<JsonValue, ParseError> {
    p.require_string_caseless("null")?;
    Ok(JsonValue::Null)
}

fn parse_bool(p: &mut Parser<'_>) -> Result<JsonValue, ParseError> {
    if p.expect_string_caseless("true")? {
        return Ok(JsonValue::Bool(true));
    }
    if p.expect_string_caseless("false")? {
        return Ok(JsonValue::Bool(false));
    }
    Err(p.syntax_err("expected true or false"))
}

fn parse_string(p: &mut Parser<'_>) -> Result<JsonValue, ParseError> {
    if p.peek() != b'"' {
        return Err(p.syntax_err("expected '\"'"));
    }
    p.advance();
    let mut s = String::new();
    loop {
        if p.is_eof() {
            return Err(ParseError::Eof);
        }
        let c = p.peek();
        if c == b'"' {
            p.advance();
            return Ok(JsonValue::String(s));
        }
        if c == b'\\' {
            p.advance();
            if p.is_eof() {
                return Err(ParseError::Eof);
            }
            let esc = p.peek();
            p.advance();
            match esc {
                b'"' => s.push('"'),
                b'\\' => s.push('\\'),
                b'/' => s.push('/'),
                b'b' => s.push('\x08'),
                b'f' => s.push('\x0c'),
                b'n' => s.push('\n'),
                b'r' => s.push('\r'),
                b't' => s.push('\t'),
                b'u' => {
                    let mut hex = 0u32;
                    for _ in 0..4 {
                        if p.is_eof() {
                            return Err(ParseError::Eof);
                        }
                        let h = p.peek();
                        let d = hex_digit(h)?;
                        hex = hex * 16 + d as u32;
                        p.advance();
                    }
                    if let Some(ch) = char::from_u32(hex) {
                        s.push(ch);
                    }
                }
                _ => return Err(p.syntax_err("invalid escape")),
            }
        } else if c < 0x20 {
            return Err(p.syntax_err("control character in string"));
        } else {
            s.push(c as char);
            p.advance();
        }
    }
}

fn hex_digit(b: u8) -> Result<u8, ParseError> {
    match b {
        b'0'..=b'9' => Ok(b - b'0'),
        b'a'..=b'f' => Ok(b - b'a' + 10),
        b'A'..=b'F' => Ok(b - b'A' + 10),
        _ => Err(ParseError::Syntax {
            filename: None,
            row: 0,
            col: 0,
            msg: "invalid hex digit".to_string(),
        }),
    }
}

fn parse_number(p: &mut Parser<'_>) -> Result<JsonValue, ParseError> {
    let start = p.offset;
    if p.peek() == b'-' {
        p.advance();
    }
    if p.peek() == b'0' && p.peek_forward(1) != b'.' && !matches!(p.peek_forward(1), b'e' | b'E') {
        p.advance();
    } else {
        parse_digits(p)?;
    }
    if p.peek() == b'.' {
        p.advance();
        parse_digits(p)?;
    }
    if p.peek() == b'e' || p.peek() == b'E' {
        p.advance();
        if p.peek() == b'+' || p.peek() == b'-' {
            p.advance();
        }
        parse_digits(p)?;
    }
    let num_str =
        std::str::from_utf8(&p.data[start..p.offset]).map_err(|_| p.syntax_err("invalid UTF-8"))?;
    let n: f64 = num_str
        .parse()
        .map_err(|_| p.syntax_err("invalid number"))?;
    Ok(JsonValue::Number(n))
}

fn parse_digits(p: &mut Parser<'_>) -> Result<(), ParseError> {
    if !p.peek().is_ascii_digit() {
        return Err(p.syntax_err("expected digits"));
    }
    while p.peek().is_ascii_digit() {
        p.advance();
    }
    Ok(())
}

fn parse_array(p: &mut Parser<'_>) -> Result<JsonValue, ParseError> {
    if p.peek() != b'[' {
        return Err(p.syntax_err("expected '['"));
    }
    p.advance();
    skip_whitespace(p);
    let mut arr = Vec::new();
    if p.peek() == b']' {
        p.advance();
        return Ok(JsonValue::Array(arr));
    }
    loop {
        arr.push(parse_value(p)?);
        skip_whitespace(p);
        if p.peek() == b']' {
            p.advance();
            return Ok(JsonValue::Array(arr));
        }
        if p.peek() != b',' {
            return Err(p.syntax_err("expected ',' or ']'"));
        }
        p.advance();
        skip_whitespace(p);
    }
}

fn parse_object(p: &mut Parser<'_>) -> Result<JsonValue, ParseError> {
    if p.peek() != b'{' {
        return Err(p.syntax_err("expected '{'"));
    }
    p.advance();
    skip_whitespace(p);
    let mut obj = BTreeMap::new();
    if p.peek() == b'}' {
        p.advance();
        return Ok(JsonValue::Object(obj));
    }
    loop {
        let key = match parse_value(p)? {
            JsonValue::String(s) => s,
            _ => return Err(p.syntax_err("object key must be string")),
        };
        skip_whitespace(p);
        if p.peek() != b':' {
            return Err(p.syntax_err("expected ':'"));
        }
        p.advance();
        skip_whitespace(p);
        let val = parse_value(p)?;
        obj.insert(key, val);
        skip_whitespace(p);
        if p.peek() == b'}' {
            p.advance();
            return Ok(JsonValue::Object(obj));
        }
        if p.peek() != b',' {
            return Err(p.syntax_err("expected ',' or '}'"));
        }
        p.advance();
        skip_whitespace(p);
    }
}

trait IsAsciiDigit {
    fn is_ascii_digit(self) -> bool;
}
impl IsAsciiDigit for u8 {
    fn is_ascii_digit(self) -> bool {
        (b'0'..=b'9').contains(&self)
    }
}
