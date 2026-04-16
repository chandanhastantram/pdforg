//! pdf-spell — Hunspell-compatible spell checker in pure Rust.

mod aff;
mod checker;
pub use checker::*;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum SpellError {
    #[error("Failed to parse .aff file: {0}")]
    AffParse(String),
    #[error("Failed to parse .dic file: {0}")]
    DicParse(String),
}

/// Built-in en_US dictionary (minimal for Phase 1)
/// In production, this is loaded from bundled Hunspell dictionaries via include_bytes!
pub fn default_checker() -> SpellChecker {
    SpellChecker::new_minimal_english()
}
