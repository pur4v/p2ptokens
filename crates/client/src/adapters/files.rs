//! File-attachment helpers: decode inline base64 payloads and extract text from
//! documents. Used by the provider-side adapters to fold `File` parts into a form
//! the backend understands — PDFs become extracted text for backends without a
//! native document API (Ollama, generic OpenAI-compatible), while the Claude
//! adapter uses these only for non-PDF files.

use anyhow::{anyhow, Context, Result};
use base64::Engine;

use p2ptokens_shared::types::FileData;

/// Max decoded attachment size we will process (guards the pdf parser / memory).
pub const MAX_FILE_BYTES: usize = 12 * 1024 * 1024;

/// Decode the raw bytes of a `data:` URI or a bare base64 string.
pub fn decode_data(data: &str) -> Result<Vec<u8>> {
    // Accept both `data:<mime>;base64,<payload>` and a bare base64 payload.
    let payload = match data.find(";base64,") {
        Some(i) => &data[i + ";base64,".len()..],
        None => data.strip_prefix("data:").unwrap_or(data),
    };
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(payload.trim())
        .context("decode base64 attachment")?;
    if bytes.len() > MAX_FILE_BYTES {
        return Err(anyhow!("attachment too large ({} bytes)", bytes.len()));
    }
    Ok(bytes)
}

/// Parse a `data:<mime>;base64,<payload>` URI into `(mime, base64_payload)`.
/// A non-data URL (remote http/https) returns `("", url)` so callers can pass it
/// through untouched. A `data:` URI without an explicit mime yields an empty mime.
pub fn parse_data_uri(url: &str) -> (String, String) {
    if let Some(rest) = url.strip_prefix("data:") {
        if let Some(i) = rest.find(";base64,") {
            return (
                rest[..i].to_string(),
                rest[i + ";base64,".len()..].to_string(),
            );
        }
        if let Some(i) = rest.find(',') {
            return (rest[..i].to_string(), rest[i + 1..].to_string());
        }
    }
    (String::new(), url.to_string())
}

/// True if a file looks like a PDF (by declared mime or filename).
pub fn is_pdf(file: &FileData) -> bool {
    file.mime.eq_ignore_ascii_case("application/pdf")
        || file.filename.to_ascii_lowercase().ends_with(".pdf")
}

/// Extract readable text from a file part. PDFs go through `pdf-extract`;
/// everything else is treated as UTF-8 text (best-effort, lossy).
pub fn extract_text(file: &FileData) -> Result<String> {
    let bytes = decode_data(&file.data)?;
    let text = if is_pdf(file) {
        pdf_extract::extract_text_from_mem(&bytes)
            .map_err(|e| anyhow!("pdf extract failed: {e}"))?
    } else {
        String::from_utf8_lossy(&bytes).into_owned()
    };
    Ok(text.trim().to_string())
}

/// Render a file part as a labelled text block for prompt injection.
pub fn as_prompt_block(file: &FileData) -> String {
    let name = if file.filename.is_empty() {
        "attachment"
    } else {
        &file.filename
    };
    match extract_text(file) {
        Ok(t) if !t.is_empty() => format!("[file: {name}]\n{t}"),
        Ok(_) => format!("[file: {name} — empty or no extractable text]"),
        Err(e) => format!("[file: {name} — could not read: {e}]"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_data_uri_and_bare_base64() {
        // "hello" base64 = aGVsbG8=
        let via_uri = decode_data("data:text/plain;base64,aGVsbG8=").unwrap();
        assert_eq!(via_uri, b"hello");
        let via_bare = decode_data("aGVsbG8=").unwrap();
        assert_eq!(via_bare, b"hello");
    }

    #[test]
    fn extracts_plain_text_file() {
        let f = FileData {
            filename: "notes.txt".into(),
            mime: "text/plain".into(),
            data: "data:text/plain;base64,aGVsbG8=".into(),
        };
        assert_eq!(extract_text(&f).unwrap(), "hello");
        assert!(as_prompt_block(&f).contains("notes.txt"));
        assert!(as_prompt_block(&f).contains("hello"));
    }

    #[test]
    fn detects_pdf_by_mime_and_name() {
        let by_mime = FileData {
            filename: "x".into(),
            mime: "application/pdf".into(),
            data: String::new(),
        };
        let by_name = FileData {
            filename: "report.PDF".into(),
            mime: String::new(),
            data: String::new(),
        };
        assert!(is_pdf(&by_mime));
        assert!(is_pdf(&by_name));
    }
}
