//! WebSocket protocol — typed message enums for client↔server communication.

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use pdf_core::{CellAddress, ViewportData};
use pdf_writer::ot::Op;
use pdf_render::canvas::CanvasCmd;
use pdf_spell::SpellResult;

/// Messages sent from the browser client to the server
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum ClientMsg {
    /// Apply an OT operation to a Writer document
    ApplyOp {
        doc_id: Uuid,
        ops: Vec<Op>,
        rev: u64,
    },
    /// Move cursor position
    MoveCursor {
        doc_id: Uuid,
        pos: CursorPosition,
    },
    /// Set spreadsheet viewport (which cells are visible)
    SetViewport {
        doc_id: Uuid,
        sheet: usize,
        first_row: u32,
        first_col: u32,
        row_count: u32,
        col_count: u32,
    },
    /// Request a rendered page from the server
    RequestPage {
        doc_id: Uuid,
        page: u32,
        scale: f32,
    },
    /// Evaluate a single formula expression
    FormulaEval {
        expr: String,
        cell: CellAddress,
        sheet: usize,
    },
    /// Spell check a block of text
    SpellCheck {
        text: String,
        lang: String,
    },
    /// Request presentation slide as SVG
    RequestSlide {
        doc_id: Uuid,
        slide_idx: usize,
    },
    /// Set cell value in spreadsheet
    SetCell {
        doc_id: Uuid,
        sheet: usize,
        row: u32,
        col: u32,
        value: String,
        formula: Option<String>,
    },
    /// Ping for keepalive
    Ping,
}

/// Messages sent from the server to the browser client
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum ServerMsg {
    /// Operation acknowledged and applied at this revision
    OpAck {
        rev: u64,
    },
    /// Remote op to apply to client-side document (for future real-time collab)
    DocUpdate {
        ops: Vec<Op>,
        rev: u64,
        author: String,
    },
    /// Canvas commands to replay for a page
    CanvasCommands {
        page: u32,
        width: f32,
        height: f32,
        commands: Vec<CanvasCmd>,
    },
    /// Spreadsheet viewport data for rendering the grid
    ViewportData {
        sheet: usize,
        data: ViewportData,
    },
    /// Formula evaluation result
    FormulaResult {
        result: String,
        error: Option<String>,
    },
    /// Spell check results
    SpellResults {
        results: Vec<SpellResult>,
    },
    /// Slide rendered as SVG
    SlideData {
        slide_idx: usize,
        svg: String,
    },
    /// Export ready — download URL
    ExportReady {
        doc_id: Uuid,
        format: String,
        url: String,
    },
    /// Error message
    Error {
        message: String,
        code: Option<String>,
    },
    /// Pong keepalive response
    Pong,
    /// Sent on connection: current document state summary
    DocumentState {
        doc_id: Uuid,
        revision: u64,
    },
}

/// Cursor position in a Writer document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CursorPosition {
    pub paragraph_idx: usize,
    pub run_idx: usize,
    pub char_idx: usize,
}
