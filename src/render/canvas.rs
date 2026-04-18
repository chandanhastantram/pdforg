//! Canvas command stream — render instructions sent from server to WASM client.

use serde::{Deserialize, Serialize};
use crate::core::common::{Color, FontSpec};
use std::sync::Arc;

/// A single canvas drawing command
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum CanvasCmd {
    /// Clear with background color
    Clear { color: Color },
    /// Fill a rectangle
    FillRect { x: f32, y: f32, w: f32, h: f32, color: Color },
    /// Stroke a rectangle
    StrokeRect { x: f32, y: f32, w: f32, h: f32, color: Color, line_width: f32 },
    /// Fill text
    FillText { text: String, x: f32, y: f32, font: FontSpec, color: Color, max_width: Option<f32> },
    /// Draw an image (PNG bytes as base64)
    DrawImage { data_b64: String, x: f32, y: f32, w: f32, h: f32 },
    /// Path commands
    BeginPath,
    MoveTo { x: f32, y: f32 },
    LineTo { x: f32, y: f32 },
    Arc { x: f32, y: f32, radius: f32, start_angle: f32, end_angle: f32 },
    QuadraticCurveTo { cpx: f32, cpy: f32, x: f32, y: f32 },
    BezierCurveTo { cp1x: f32, cp1y: f32, cp2x: f32, cp2y: f32, x: f32, y: f32 },
    ClosePath,
    Fill { rule: FillRule },
    Stroke,
    /// State management
    Save,
    Restore,
    Clip,
    /// Transforms
    Translate { x: f32, y: f32 },
    Rotate { angle: f32 },
    Scale { x: f32, y: f32 },
    SetTransform { a: f32, b: f32, c: f32, d: f32, e: f32, f: f32 },
    ResetTransform,
    /// Styling
    SetLineWidth { w: f32 },
    SetLineDash { segments: Vec<f32> },
    SetLineCap { cap: LineCap },
    SetLineJoin { join: LineJoin },
    SetGlobalAlpha { alpha: f32 },
    SetShadow { offset_x: f32, offset_y: f32, blur: f32, color: Color },
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum FillRule {
    #[default]
    NonZero,
    EvenOdd,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum LineCap {
    #[default]
    Butt,
    Round,
    Square,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum LineJoin {
    Miter,
    #[default]
    Round,
    Bevel,
}

/// A complete set of canvas commands for one page
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageRender {
    pub page_num: u32,
    pub width: f32,
    pub height: f32,
    pub scale: f32,
    pub commands: Vec<CanvasCmd>,
}

impl PageRender {
    pub fn new(page_num: u32, width: f32, height: f32) -> Self {
        PageRender { page_num, width, height, scale: 1.0, commands: vec![] }
    }

    pub fn push(&mut self, cmd: CanvasCmd) {
        self.commands.push(cmd);
    }

    pub fn fill_rect(&mut self, x: f32, y: f32, w: f32, h: f32, color: Color) {
        self.push(CanvasCmd::FillRect { x, y, w, h, color });
    }

    pub fn stroke_rect(&mut self, x: f32, y: f32, w: f32, h: f32, color: Color, line_width: f32) {
        self.push(CanvasCmd::StrokeRect { x, y, w, h, color, line_width });
    }

    pub fn fill_text(&mut self, text: String, x: f32, y: f32, font: FontSpec, color: Color) {
        self.push(CanvasCmd::FillText { text, x, y, font, color, max_width: None });
    }

    pub fn begin_path(&mut self) { self.push(CanvasCmd::BeginPath); }
    pub fn move_to(&mut self, x: f32, y: f32) { self.push(CanvasCmd::MoveTo { x, y }); }
    pub fn line_to(&mut self, x: f32, y: f32) { self.push(CanvasCmd::LineTo { x, y }); }
    pub fn close_path(&mut self) { self.push(CanvasCmd::ClosePath); }
    pub fn fill(&mut self) { self.push(CanvasCmd::Fill { rule: FillRule::NonZero }); }
    pub fn stroke(&mut self) { self.push(CanvasCmd::Stroke); }
    pub fn save(&mut self) { self.push(CanvasCmd::Save); }
    pub fn restore(&mut self) { self.push(CanvasCmd::Restore); }
    pub fn set_line_width(&mut self, w: f32) { self.push(CanvasCmd::SetLineWidth { w }); }
    pub fn set_global_alpha(&mut self, alpha: f32) { self.push(CanvasCmd::SetGlobalAlpha { alpha }); }
}
