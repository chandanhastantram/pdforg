//! pdf-formats — DOCX, XLSX, PPTX, ODF, RTF parsers and writers.

pub mod docx;
pub mod xlsx;
pub mod pptx;
pub mod odf;
pub mod rtf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum FormatError {
    #[error("ZIP error: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("XML error: {0}")]
    Xml(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("UTF-8 error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("Unsupported format: {0}")]
    Unsupported(String),
}

/// Auto-detect file format from bytes and parse
pub fn detect_format(bytes: &[u8]) -> Option<&'static str> {
    if bytes.starts_with(b"PK") {
        // ZIP-based formats — peek inside
        Some("zip-based") // caller should check extension
    } else if bytes.starts_with(b"{\\rtf") {
        Some("rtf")
    } else if bytes.starts_with(b"%PDF") {
        Some("pdf")
    } else {
        None
    }
}
