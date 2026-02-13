//! Error types for the parse crate.

use std::fmt;

/// Unified parse error.
#[derive(Debug, Clone)]
pub enum ParseError {
    /// Syntax error with optional location.
    Syntax {
        /// Optional filename for user-facing messages.
        filename: Option<String>,
        /// 1-based row.
        row: usize,
        /// 1-based column.
        col: usize,
        /// Error message.
        msg: String,
    },
    /// I/O error.
    Io(String),
    /// Unsupported feature or format.
    Unsupported(String),
    /// Unexpected end of input.
    Eof,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::Syntax {
                filename,
                row,
                col,
                msg,
            } => {
                if let Some(ref name) = filename {
                    write!(f, "{}:{}:{}: error: {}", name, row, col, msg)
                } else {
                    write!(f, "{}:{}: error: {}", row, col, msg)
                }
            }
            ParseError::Io(s) => write!(f, "I/O error: {}", s),
            ParseError::Unsupported(s) => write!(f, "Unsupported: {}", s),
            ParseError::Eof => write!(f, "Unexpected end of input"),
        }
    }
}

impl std::error::Error for ParseError {}
