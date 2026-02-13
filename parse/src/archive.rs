//! ZIP and TAR archive parsers.
//!
//! List entries and read file contents.

use crate::error::ParseError;
use std::io::{Cursor, Read};

/// ZIP archive entry metadata.
#[derive(Clone, Debug)]
pub struct ZipEntry {
    /// Entry path/name.
    pub name: String,
    /// Uncompressed size if known.
    pub size: Option<u64>,
    /// True if directory.
    pub is_dir: bool,
}

/// Lists ZIP entries without extracting.
///
/// # Errors
///
/// Returns [`ParseError`](crate::ParseError) on invalid ZIP.
pub fn list_zip(data: &[u8]) -> Result<Vec<ZipEntry>, ParseError> {
    let cursor = Cursor::new(data);
    let mut archive = zip::ZipArchive::new(cursor).map_err(|e| ParseError::Io(e.to_string()))?;

    let mut entries = Vec::new();
    for i in 0..archive.len() {
        let file = archive
            .by_index(i)
            .map_err(|e| ParseError::Io(e.to_string()))?;
        let name = file.name().to_string();
        let is_dir = file.is_dir();
        let size = if file.size() > 0 {
            Some(file.size())
        } else {
            None
        };
        entries.push(ZipEntry { name, size, is_dir });
    }
    Ok(entries)
}

/// Reads a file from ZIP by name.
///
/// # Errors
///
/// Returns [`ParseError`](crate::ParseError) on invalid ZIP or missing file.
pub fn read_zip_file(data: &[u8], name: &str) -> Result<Vec<u8>, ParseError> {
    let cursor = Cursor::new(data);
    let mut archive = zip::ZipArchive::new(cursor).map_err(|e| ParseError::Io(e.to_string()))?;
    let mut file = archive
        .by_name(name)
        .map_err(|e| ParseError::Io(e.to_string()))?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)
        .map_err(|e| ParseError::Io(e.to_string()))?;
    Ok(buf)
}

/// TAR archive entry metadata.
#[derive(Clone, Debug)]
pub struct TarEntry {
    /// Entry path/name.
    pub name: String,
    /// File size.
    pub size: u64,
    /// True if directory.
    pub is_dir: bool,
}

/// Lists TAR entries without extracting.
///
/// # Errors
///
/// Returns [`ParseError`](crate::ParseError) on invalid TAR.
pub fn list_tar(data: &[u8]) -> Result<Vec<TarEntry>, ParseError> {
    let cursor = Cursor::new(data);
    let mut archive = tar::Archive::new(cursor);
    let mut entries = Vec::new();

    for entry in archive
        .entries()
        .map_err(|e| ParseError::Io(e.to_string()))?
    {
        let entry = entry.map_err(|e| ParseError::Io(e.to_string()))?;
        let path = entry.path().map_err(|e| ParseError::Io(e.to_string()))?;
        let name = path.to_string_lossy().to_string();
        let header = entry.header();
        let size = header.size().unwrap_or(0);
        let is_dir = header.entry_type().is_dir();
        entries.push(TarEntry { name, size, is_dir });
    }
    Ok(entries)
}

/// Reads a file from TAR by name.
///
/// # Errors
///
/// Returns [`ParseError`](crate::ParseError) on invalid TAR or missing file.
pub fn read_tar_file(data: &[u8], name: &str) -> Result<Vec<u8>, ParseError> {
    let cursor = Cursor::new(data);
    let mut archive = tar::Archive::new(cursor);

    for entry in archive
        .entries()
        .map_err(|e| ParseError::Io(e.to_string()))?
    {
        let mut entry = entry.map_err(|e| ParseError::Io(e.to_string()))?;
        let path = entry.path().map_err(|e| ParseError::Io(e.to_string()))?;
        if path.to_string_lossy() == name {
            let mut buf = Vec::new();
            entry
                .read_to_end(&mut buf)
                .map_err(|e| ParseError::Io(e.to_string()))?;
            return Ok(buf);
        }
    }
    Err(ParseError::Io(format!("file not found: {}", name)))
}
