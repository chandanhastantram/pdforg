//! pdf-pdf — PDF creation, manipulation, and parsing (pure Rust via lopdf).

pub mod creator;
pub mod manipulator;

pub use creator::*;
pub use manipulator::*;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum PdfError {
    #[error("PDF parse error: {0}")]
    Parse(String),
    #[error("PDF encoding error: {0}")]
    Encode(String),
    #[error("Lopdf error: {0}")]
    Lopdf(#[from] lopdf::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Image error: {0}")]
    Image(#[from] image::ImageError),
}
