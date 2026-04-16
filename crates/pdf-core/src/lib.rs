//! pdf-core — Shared document model for PDF Office
//!
//! Defines the canonical in-memory representation for Writer, Sheets, and Slides.
//! All types implement serde Serialize/Deserialize for persistence via JSON.

pub mod document;
pub mod spreadsheet;
pub mod presentation;
pub mod common;

pub use document::*;
pub use spreadsheet::*;
pub use presentation::*;
pub use common::*;
