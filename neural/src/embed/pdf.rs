//! PDF text extraction via pdf-extract. Gated by feature `pdf` and native only (not wasm32).

/// Extract text from PDF bytes. Returns concatenated text.
/// Requires feature `pdf`. Not built for wasm32 (pdf-extract may not support it).
pub fn extract_text(bytes: &[u8]) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let text = pdf_extract::extract_text_from_mem(bytes)?;
    Ok(text)
}

/// Extract text from PDF bytes, returning one string per page if the backend supports it.
/// Fallback: returns a single element with full text.
pub fn extract_text_pages(
    bytes: &[u8],
) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
    let full = extract_text(bytes)?;
    if full.trim().is_empty() {
        return Ok(vec![]);
    }
    // pdf_extract does not expose per-page API; return single segment.
    Ok(vec![full])
}
