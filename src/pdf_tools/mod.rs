use thiserror::Error;

pub mod creator;
pub mod manipulator;
pub mod protect;
pub mod compress;
pub mod stamp;
pub mod metadata;
pub mod images;

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

pub use creator::*;
pub use manipulator::*;
pub use protect::*;
pub use compress::*;
pub use stamp::*;
pub use metadata::*;
pub use images::*;
