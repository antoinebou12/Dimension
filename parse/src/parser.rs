//! Shared parser helper for text-based formats (BVH, OBJ, MTL).
//!
//! Provides offset, row, col tracking with peek/advance operations.

use crate::error::ParseError;

/// Parser state for text formats.
#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct Parser<'a> {
    /// Optional filename for error messages.
    pub filename: Option<String>,
    /// Current byte offset.
    pub offset: usize,
    /// Current 1-based row.
    pub row: usize,
    /// Current 1-based column.
    pub col: usize,
    /// Input data.
    pub(crate) data: &'a [u8],
}

#[allow(dead_code)]
impl<'a> Parser<'a> {
    /// Creates a new parser.
    pub fn new(data: &'a [u8], filename: Option<String>) -> Self {
        Self {
            filename,
            offset: 0,
            row: 1,
            col: 1,
            data,
        }
    }

    /// Peeks at the current character.
    pub fn peek(&self) -> u8 {
        self.data.get(self.offset).copied().unwrap_or(0)
    }

    /// Peeks forward `steps` bytes (no bounds check).
    pub fn peek_forward(&self, steps: usize) -> u8 {
        self.data.get(self.offset + steps).copied().unwrap_or(0)
    }

    /// Returns true if at end of input.
    pub fn is_eof(&self) -> bool {
        self.offset >= self.data.len()
    }

    /// Advances by one character, updating row/col.
    pub fn advance(&mut self) {
        if self.offset < self.data.len() {
            if self.data[self.offset] == b'\n' {
                self.row += 1;
                self.col = 1;
            } else {
                self.col += 1;
            }
            self.offset += 1;
        }
    }

    /// Advances by `n` characters.
    pub fn advance_n(&mut self, n: usize) {
        for _ in 0..n {
            if self.is_eof() {
                break;
            }
            self.advance();
        }
    }

    /// Returns true if current char matches.
    pub fn match_char(&self, c: u8) -> bool {
        self.peek() == c
    }

    /// Returns true if current char is in the set.
    pub fn one_of(&self, chars: &[u8]) -> bool {
        let p = self.peek();
        chars.iter().any(|&c| c == p)
    }

    /// Parses whitespace (space, tab, CR, vertical tab). Stops at newline or non-whitespace.
    pub fn parse_whitespace(&mut self) {
        while self.one_of(b" \t\r\x0b") {
            self.advance();
        }
    }

    /// Parses optional whitespace, then a newline, then optional whitespace.
    /// Returns true if successful.
    pub fn parse_newline(&mut self) -> Result<(), ParseError> {
        self.parse_whitespace();
        if self.match_char(b'\n') {
            self.advance();
            self.parse_whitespace();
            Ok(())
        } else {
            Err(self.syntax_err(format!("expected newline at '{}'", char_name(self.peek()))))
        }
    }

    /// Checks that the input starts with the given string (case-insensitive).
    /// Advances past it if so. Returns true if matched.
    pub fn expect_string_caseless(&mut self, s: &str) -> Result<bool, ParseError> {
        let s = s.as_bytes();
        if self.offset + s.len() > self.data.len() {
            return Ok(false);
        }
        for (i, &b) in s.iter().enumerate() {
            let c = self.data[self.offset + i];
            if c.to_ascii_lowercase() != b.to_ascii_lowercase() {
                return Ok(false);
            }
        }
        self.advance_n(s.len());
        Ok(true)
    }

    /// Expects the string (case-insensitive). Returns error if not found.
    pub fn require_string_caseless(&mut self, s: &str) -> Result<(), ParseError> {
        if !self.expect_string_caseless(s)? {
            return Err(self.syntax_err(format!(
                "expected '{}' at '{}'",
                s,
                char_name(self.peek())
            )));
        }
        Ok(())
    }

    /// Creates a syntax error at current position.
    pub fn syntax_err(&self, msg: impl Into<String>) -> ParseError {
        ParseError::Syntax {
            filename: self.filename.clone(),
            row: self.row,
            col: self.col,
            msg: msg.into(),
        }
    }

    /// Returns remaining slice from current offset.
    pub fn remaining(&self) -> &'a [u8] {
        &self.data[self.offset..]
    }
}

#[allow(dead_code)]
fn char_name(c: u8) -> String {
    match c {
        0 => "end of file".to_string(),
        b'\r' | b'\n' => "newline".to_string(),
        b'\t' => "tab".to_string(),
        b'\x0b' => "vertical tab".to_string(),
        b'\x08' => "backspace".to_string(),
        b'\x0c' => "form feed".to_string(),
        _ => format!("'{}'", c as char),
    }
}

#[allow(dead_code)]
trait ToAsciiLowercase {
    fn to_ascii_lowercase(self) -> u8;
}

impl ToAsciiLowercase for u8 {
    fn to_ascii_lowercase(self) -> u8 {
        if (b'A'..=b'Z').contains(&self) {
            self + 32
        } else {
            self
        }
    }
}
