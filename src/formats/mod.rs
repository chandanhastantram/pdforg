use thiserror::Error;

pub mod docx;
pub mod xlsx;
pub mod pptx;
pub mod odf;
pub mod rtf;

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

pub fn detect_format(bytes: &[u8]) -> Option<&'static str> {
    if bytes.starts_with(b"PK") {
        Some("zip-based")
    } else if bytes.starts_with(b"{\\rtf") {
        Some("rtf")
    } else if bytes.starts_with(b"%PDF") {
        Some("pdf")
    } else {
        None
    }
}
