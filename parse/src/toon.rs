//! TOON parser (Token-Oriented Object Notation).
//!
//! Per [TOON spec](https://toonformat.dev/) core profile: key-value pairs with colon,
//! indentation-based hierarchy, arrays with length markers.
//!
//! Supports strings, numbers, booleans, null, arrays, objects.

use crate::error::ParseError;
use crate::parser::Parser;
use std::collections::BTreeMap;

/// TOON value (JSON-compatible model).
#[derive(Clone, Debug, PartialEq)]
pub enum ToonValue {
    /// Null.
    Null,
    /// Boolean.
    Bool(bool),
    /// Number.
    Number(f64),
    /// String.
    String(String),
    /// Array.
    Array(Vec<ToonValue>),
    /// Object.
    Object(BTreeMap<String, ToonValue>),
}

/// Parses TOON from bytes.
///
/// # Errors
///
/// Returns [`ParseError`](crate::ParseError) on invalid TOON.
pub fn parse(data: &[u8]) -> Result<ToonValue, ParseError> {
    let s = std::str::from_utf8(data).map_err(|e| ParseError::Io(e.to_string()))?;
    parse_str(s)
}

/// Parses TOON from string.
pub fn parse_str(s: &str) -> Result<ToonValue, ParseError> {
    let mut p = Parser::new(s.as_bytes(), None);
    skip_whitespace_and_comments(&mut p);
    if p.is_eof() {
        return Ok(ToonValue::Null);
    }
    let v = parse_value_at_indent(&mut p, 0)?;
    skip_whitespace_and_comments(&mut p);
    if !p.is_eof() {
        return Err(p.syntax_err("trailing data after TOON value"));
    }
    Ok(v)
}

fn skip_whitespace(p: &mut Parser<'_>) {
    while p.one_of(b" \t") && !p.match_char(b'\n') {
        p.advance();
    }
}

fn skip_whitespace_and_comments(p: &mut Parser<'_>) {
    loop {
        while p.one_of(b" \t\n\r") {
            p.advance();
        }
        if p.match_char(b'#') {
            while !p.is_eof() && !p.match_char(b'\n') {
                p.advance();
            }
        } else {
            break;
        }
    }
}

fn indent_level(p: &Parser<'_>, line_start: usize) -> usize {
    let mut level = 0;
    let mut i = line_start;
    while i + 1 < p.data.len() && p.data[i] == b' ' && p.data[i + 1] == b' ' {
        level += 1;
        i += 2;
    }
    level
}

fn parse_value_at_indent(
    p: &mut Parser<'_>,
    _expected_indent: usize,
) -> Result<ToonValue, ParseError> {
    skip_whitespace_and_comments(p);
    if p.is_eof() {
        return Err(ParseError::Eof);
    }
    let line_start = p.offset;
    if p.expect_string_caseless("null")? {
        Ok(ToonValue::Null)
    } else if p.match_char(b't') || p.match_char(b'f') {
        parse_bool(p)
    } else if p.match_char(b'"') {
        parse_string(p)
    } else if p.peek().is_ascii_digit() || (p.peek() == b'-' && p.peek_forward(1).is_ascii_digit())
    {
        parse_number(p)
    } else {
        parse_key_value_or_object(p, line_start)
    }
}

fn parse_bool(p: &mut Parser<'_>) -> Result<ToonValue, ParseError> {
    if p.expect_string_caseless("true")? {
        return Ok(ToonValue::Bool(true));
    }
    if p.expect_string_caseless("false")? {
        return Ok(ToonValue::Bool(false));
    }
    Err(p.syntax_err("expected true or false"))
}

fn parse_string(p: &mut Parser<'_>) -> Result<ToonValue, ParseError> {
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
            return Ok(ToonValue::String(s));
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
                b'n' => s.push('\n'),
                b'r' => s.push('\r'),
                b't' => s.push('\t'),
                _ => s.push(esc as char),
            }
        } else {
            s.push(c as char);
            p.advance();
        }
    }
}

fn parse_number(p: &mut Parser<'_>) -> Result<ToonValue, ParseError> {
    let start = p.offset;
    if p.peek() == b'-' {
        p.advance();
    }
    parse_digits(p)?;
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
    Ok(ToonValue::Number(n))
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

fn parse_identifier(p: &mut Parser<'_>) -> Result<String, ParseError> {
    let mut s = String::new();
    let c = p.peek();
    if !c.is_ascii_alphabetic() && c != b'_' {
        return Err(p.syntax_err("expected identifier"));
    }
    while p.peek().is_ascii_alphanumeric() || p.peek() == b'_' || p.peek() == b'-' {
        s.push(p.peek() as char);
        p.advance();
    }
    Ok(s)
}

fn parse_key_value_or_object(
    p: &mut Parser<'_>,
    line_start: usize,
) -> Result<ToonValue, ParseError> {
    let key = parse_identifier(p)?;
    skip_whitespace(p);
    if p.peek() != b':' {
        return Err(p.syntax_err("expected ':' after key"));
    }
    p.advance();
    skip_whitespace_and_comments(p);

    if p.peek() == b'[' {
        p.advance();
        let n_str = parse_identifier(p)?;
        let n: usize = n_str
            .parse()
            .map_err(|_| p.syntax_err("expected array length"))?;
        if p.peek() != b']' {
            return Err(p.syntax_err("expected ']'"));
        }
        p.advance();
        skip_whitespace(p);
        if p.peek() != b':' {
            return Err(p.syntax_err("expected ':' after array declaration"));
        }
        p.advance();
        skip_whitespace_and_comments(p);

        let mut arr = Vec::with_capacity(n);
        for i in 0..n {
            if i > 0 {
                skip_whitespace(p);
                if p.peek() == b',' {
                    p.advance();
                    skip_whitespace(p);
                }
            }
            arr.push(parse_value_at_indent(p, 0)?);
        }
        return Ok(ToonValue::Object(
            [(key, ToonValue::Array(arr))].into_iter().collect(),
        ));
    }

    let val = parse_value_at_indent(p, 0)?;
    let mut map = BTreeMap::new();
    map.insert(key, val);

    let base_indent = indent_level(p, line_start);
    loop {
        skip_whitespace_and_comments(p);
        if p.is_eof() {
            break;
        }
        let ls = p.offset;
        let ind = indent_level(p, ls);
        if ind <= base_indent && (ind > 0 || ls < p.data.len()) {
            let ch = p.data.get(ls).copied().unwrap_or(0);
            if ch == b' ' || ch == b'\t' {
                if ind <= base_indent && ind < (base_indent + 1) {
                    break;
                }
            } else {
                break;
            }
        }
        if ind != base_indent + 1 {
            break;
        }
        p.advance_n(ind * 2);
        let k = parse_identifier(p)?;
        skip_whitespace(p);
        if p.peek() != b':' {
            break;
        }
        p.advance();
        skip_whitespace_and_comments(p);
        let v = parse_value_at_indent(p, ind)?;
        map.insert(k, v);
    }
    Ok(ToonValue::Object(map))
}

trait IsAsciiDigit {
    fn is_ascii_digit(self) -> bool;
}
impl IsAsciiDigit for u8 {
    fn is_ascii_digit(self) -> bool {
        (b'0'..=b'9').contains(&self)
    }
}

trait IsAsciiAlphabetic {
    fn is_ascii_alphabetic(self) -> bool;
}
impl IsAsciiAlphabetic for u8 {
    fn is_ascii_alphabetic(self) -> bool {
        (b'a'..=b'z').contains(&self) || (b'A'..=b'Z').contains(&self)
    }
}

trait IsAsciiAlphanumeric {
    fn is_ascii_alphanumeric(self) -> bool;
}
impl IsAsciiAlphanumeric for u8 {
    fn is_ascii_alphanumeric(self) -> bool {
        self.is_ascii_digit() || self.is_ascii_alphabetic()
    }
}
