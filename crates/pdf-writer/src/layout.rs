//! Document layout model — converts the document model into positioned layout boxes.

use pdf_core::common::*;
use serde::{Deserialize, Serialize};

/// A positioned layout box with absolute coordinates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutBox {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub kind: LayoutBoxKind,
    pub children: Vec<LayoutBox>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LayoutBoxKind {
    Page { page_num: u32 },
    Block,
    Line,
    GlyphRun { text: String, font: FontSpec, color: Color },
    Image { data: Vec<u8>, mime_type: String },
    HorizontalRule,
    TableCell { row: u32, col: u32 },
    TableRow { row: u32 },
    Table,
}

/// A laid-out page ready for rendering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutPage {
    pub page_num: u32,
    pub width: f32,
    pub height: f32,
    pub boxes: Vec<LayoutBox>,
    pub background: Option<Color>,
}

impl LayoutPage {
    pub fn new(page_num: u32, width: f32, height: f32) -> Self {
        LayoutPage { page_num, width, height, boxes: vec![], background: None }
    }
}

/// Simplified layout engine — produces LayoutPages from a Document.
/// For Phase 1, this performs basic paragraph-by-paragraph layout
/// without full Knuth-Plass line breaking (which requires font metrics from rustybuzz).
pub struct LayoutEngine {
    pub page_layout: pdf_core::PageLayout,
    pub dpi: f32,
}

impl LayoutEngine {
    pub fn new(page_layout: pdf_core::PageLayout) -> Self {
        LayoutEngine { page_layout, dpi: 96.0 }
    }

    /// Convert mm to pixels at current DPI
    fn mm_to_px(&self, mm: f32) -> f32 { mm * self.dpi / 25.4 }
    /// Convert pt to pixels at current DPI
    fn pt_to_px(&self, pt: f32) -> f32 { pt * self.dpi / 72.0 }

    pub fn layout_document(&self, doc: &pdf_core::Document) -> Vec<LayoutPage> {
        let page_w = self.mm_to_px(self.page_layout.width);
        let page_h = self.mm_to_px(self.page_layout.height);
        let margin_t = self.mm_to_px(self.page_layout.margin_top);
        let margin_b = self.mm_to_px(self.page_layout.margin_bottom);
        let margin_l = self.mm_to_px(self.page_layout.margin_left);
        let margin_r = self.mm_to_px(self.page_layout.margin_right);
        let content_w = page_w - margin_l - margin_r;
        let _content_h = page_h - margin_t - margin_b;

        let mut pages = vec![];
        let mut current_page = LayoutPage::new(1, page_w, page_h);
        let mut y = margin_t;

        for block in &doc.body {
            let boxes = self.layout_block(block, margin_l, y, content_w, &mut y);
            for b in boxes {
                if y > page_h - margin_b {
                    pages.push(current_page);
                    current_page = LayoutPage::new(pages.len() as u32 + 1, page_w, page_h);
                    y = margin_t;
                }
                current_page.boxes.push(b);
            }
        }

        pages.push(current_page);
        pages
    }

    fn layout_block(
        &self,
        block: &pdf_core::Block,
        x: f32,
        _y: f32,
        width: f32,
        cursor_y: &mut f32,
    ) -> Vec<LayoutBox> {
        use pdf_core::Block;

        match block {
            Block::Paragraph(para) => self.layout_paragraph(para, x, cursor_y, width),
            Block::Heading(h) => self.layout_heading(h, x, cursor_y, width),
            Block::HorizontalRule => {
                let b = LayoutBox {
                    x,
                    y: *cursor_y,
                    width,
                    height: 1.0,
                    kind: LayoutBoxKind::HorizontalRule,
                    children: vec![],
                };
                *cursor_y += 16.0;
                vec![b]
            }
            Block::PageBreak => {
                *cursor_y = f32::MAX; // signals page break
                vec![]
            }
            Block::List(list) => {
                // Phase 1: flatten list items as paragraphs
                let mut boxes = vec![];
                for item in &list.items {
                    for inner in &item.content {
                        let inner_boxes = self.layout_block(inner, x + 20.0, 0.0, width - 20.0, cursor_y);
                        boxes.extend(inner_boxes);
                    }
                }
                boxes
            }
            _ => vec![],
        }
    }

    fn layout_paragraph(
        &self,
        para: &pdf_core::Paragraph,
        x: f32,
        cursor_y: &mut f32,
        width: f32,
    ) -> Vec<LayoutBox> {
        let line_height = 20.0; // default 20px line height
        let mut children = vec![];
        let line_x = x;

        // Render all runs as glyph runs
        for run in &para.runs {
            let font = run.format.font.clone().unwrap_or_default();
            let color = run.format.color.clone().unwrap_or(Color::BLACK);
            children.push(LayoutBox {
                x: line_x,
                y: *cursor_y,
                width: width,
                height: line_height,
                kind: LayoutBoxKind::GlyphRun {
                    text: run.text.clone(),
                    font,
                    color,
                },
                children: vec![],
            });
        }

        let block = LayoutBox {
            x,
            y: *cursor_y,
            width,
            height: line_height,
            kind: LayoutBoxKind::Block,
            children,
        };

        *cursor_y += line_height + self.pt_to_px(para.space_after);
        vec![block]
    }

    fn layout_heading(
        &self,
        heading: &pdf_core::Heading,
        x: f32,
        cursor_y: &mut f32,
        width: f32,
    ) -> Vec<LayoutBox> {
        let sizes = [32.0, 28.0, 24.0, 20.0, 18.0, 16.0];
        let size = sizes.get(heading.level.saturating_sub(1) as usize).copied().unwrap_or(20.0);
        let y = *cursor_y;

        let block = LayoutBox {
            x,
            y,
            width,
            height: size * 1.5,
            kind: LayoutBoxKind::Block,
            children: heading.runs.iter().map(|run| LayoutBox {
                x,
                y,
                width,
                height: size * 1.5,
                kind: LayoutBoxKind::GlyphRun {
                    text: run.text.clone(),
                    font: FontSpec {
                        family: "Liberation Sans".into(),
                        size: size / 1.333,
                        weight: FontWeight::Bold,
                        style: FontStyle::Normal,
                    },
                    color: Color::BLACK,
                },
                children: vec![],
            }).collect(),
        };
        *cursor_y += size * 1.5 + 8.0;
        vec![block]
    }
}
