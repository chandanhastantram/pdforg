//! pdf-writer — Word processor engine
//!
//! Implements the Operational Transform (OT) engine for conflict-free concurrent editing,
//! tracked changes, and the document layout model.

pub mod ot;
pub mod tracked_changes;
pub mod layout;

pub use ot::*;
pub use tracked_changes::*;
pub use layout::*;
