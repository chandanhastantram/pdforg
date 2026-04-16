//! pdf-ui — placeholder for future Leptos WASM frontend.
//! In Phase 2, this crate will be compiled to WASM via trunk
//! and served as the interactive document editing canvas.

/// Re-export message types used by the WASM client
pub mod protocol {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct CanvasSize {
        pub width: f32,
        pub height: f32,
        pub dpr: f32,
    }
}
