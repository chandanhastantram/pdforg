//! PDF creation using lopdf — pure Rust, no external dependencies.

use lopdf::{Document, Object, Stream, Dictionary, content::{Content, Operation}};
use super::PdfError;
use crate::core::common::{Color, FontSpec};

/// PDF page size presets (pt)
pub struct PageSize;
impl PageSize {
    pub const A4: (f64, f64) = (595.28, 841.89);
    pub const LETTER: (f64, f64) = (612.0, 792.0);
    pub const A3: (f64, f64) = (841.89, 1190.55);
    pub const LEGAL: (f64, f64) = (612.0, 1008.0);
}

/// A page ready for PDF export
#[derive(Debug, Clone)]
pub struct PdfPageContent {
    pub width: f64,
    pub height: f64,
    pub operations: Vec<PdfOperation>,
}

impl PdfPageContent {
    pub fn new(width: f64, height: f64) -> Self {
        PdfPageContent { width, height, operations: vec![] }
    }

    pub fn fill_rect(&mut self, x: f64, y: f64, w: f64, h: f64, color: &Color) {
        self.operations.push(PdfOperation::SetFillColor(color.r as f64 / 255.0, color.g as f64 / 255.0, color.b as f64 / 255.0));
        self.operations.push(PdfOperation::Rect(x, y, w, h));
        self.operations.push(PdfOperation::Fill);
    }

    pub fn stroke_rect(&mut self, x: f64, y: f64, w: f64, h: f64, color: &Color, line_width: f64) {
        self.operations.push(PdfOperation::SetStrokeColor(color.r as f64 / 255.0, color.g as f64 / 255.0, color.b as f64 / 255.0));
        self.operations.push(PdfOperation::SetLineWidth(line_width));
        self.operations.push(PdfOperation::Rect(x, y, w, h));
        self.operations.push(PdfOperation::Stroke);
    }

    pub fn draw_text(&mut self, text: &str, x: f64, y: f64, font_size: f64, color: &Color) {
        self.operations.push(PdfOperation::SetFillColor(color.r as f64 / 255.0, color.g as f64 / 255.0, color.b as f64 / 255.0));
        self.operations.push(PdfOperation::BeginText);
        self.operations.push(PdfOperation::SetFont("F1".into(), font_size));
        self.operations.push(PdfOperation::TextPosition(x, y));
        self.operations.push(PdfOperation::ShowText(text.to_string()));
        self.operations.push(PdfOperation::EndText);
    }

    pub fn draw_line(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, color: &Color, width: f64) {
        self.operations.push(PdfOperation::SetStrokeColor(color.r as f64 / 255.0, color.g as f64 / 255.0, color.b as f64 / 255.0));
        self.operations.push(PdfOperation::SetLineWidth(width));
        self.operations.push(PdfOperation::MoveTo(x1, y1));
        self.operations.push(PdfOperation::LineTo(x2, y2));
        self.operations.push(PdfOperation::Stroke);
    }
}

/// PDF drawing operations
#[derive(Debug, Clone)]
pub enum PdfOperation {
    SetFillColor(f64, f64, f64),
    SetStrokeColor(f64, f64, f64),
    SetLineWidth(f64),
    Rect(f64, f64, f64, f64),
    Fill,
    Stroke,
    FillStroke,
    MoveTo(f64, f64),
    LineTo(f64, f64),
    ClosePath,
    BeginText,
    EndText,
    SetFont(String, f64),
    TextPosition(f64, f64),
    ShowText(String),
    SaveState,
    RestoreState,
}

impl PdfOperation {
    fn to_content_operation(&self) -> Operation {
        match self {
            PdfOperation::SetFillColor(r, g, b) => Operation::new("rg", vec![
                Object::Real(*r as f32), Object::Real(*g as f32), Object::Real(*b as f32)
            ]),
            PdfOperation::SetStrokeColor(r, g, b) => Operation::new("RG", vec![
                Object::Real(*r as f32), Object::Real(*g as f32), Object::Real(*b as f32)
            ]),
            PdfOperation::SetLineWidth(w) => Operation::new("w", vec![Object::Real(*w as f32)]),
            PdfOperation::Rect(x, y, w, h) => Operation::new("re", vec![
                Object::Real(*x as f32), Object::Real(*y as f32),
                Object::Real(*w as f32), Object::Real(*h as f32)
            ]),
            PdfOperation::Fill => Operation::new("f", vec![]),
            PdfOperation::Stroke => Operation::new("S", vec![]),
            PdfOperation::FillStroke => Operation::new("B", vec![]),
            PdfOperation::MoveTo(x, y) => Operation::new("m", vec![
                Object::Real(*x as f32), Object::Real(*y as f32)
            ]),
            PdfOperation::LineTo(x, y) => Operation::new("l", vec![
                Object::Real(*x as f32), Object::Real(*y as f32)
            ]),
            PdfOperation::ClosePath => Operation::new("h", vec![]),
            PdfOperation::BeginText => Operation::new("BT", vec![]),
            PdfOperation::EndText => Operation::new("ET", vec![]),
            PdfOperation::SetFont(name, size) => Operation::new("Tf", vec![
                Object::Name(name.as_bytes().to_vec()),
                Object::Real(*size as f32),
            ]),
            PdfOperation::TextPosition(x, y) => Operation::new("Td", vec![
                Object::Real(*x as f32), Object::Real(*y as f32)
            ]),
            PdfOperation::ShowText(text) => Operation::new("Tj", vec![
                Object::String(text.as_bytes().to_vec(), lopdf::StringFormat::Literal)
            ]),
            PdfOperation::SaveState => Operation::new("q", vec![]),
            PdfOperation::RestoreState => Operation::new("Q", vec![]),
        }
    }
}

/// Create a PDF document from a list of pages
pub fn create_pdf(pages: &[PdfPageContent]) -> Result<Vec<u8>, PdfError> {
    let mut doc = Document::with_version("1.7");
    let pages_id = doc.new_object_id();
    let font_id = doc.add_object(Dictionary::from_iter(vec![
        ("Type", Object::Name(b"Font".to_vec())),
        ("Subtype", Object::Name(b"Type1".to_vec())),
        ("BaseFont", Object::Name(b"Helvetica".to_vec())),
    ]));

    let mut page_ids = vec![];

    for page_content in pages {
        // Build content stream
        let operations: Vec<Operation> = page_content.operations.iter()
            .map(|op| op.to_content_operation())
            .collect();
        let content = Content { operations };
        let content_bytes = content.encode().map_err(|e| PdfError::Encode(e.to_string()))?;
        let content_stream = Stream::new(Dictionary::new(), content_bytes);
        let content_id = doc.add_object(content_stream);

        // Build font dict
        let font_dict = Dictionary::from_iter(vec![
            ("F1", Object::Reference(font_id)),
        ]);
        let resources_dict = Dictionary::from_iter(vec![
            ("Font", Object::Dictionary(font_dict)),
        ]);

        // Build page object
        let page_dict = Dictionary::from_iter(vec![
            ("Type", Object::Name(b"Page".to_vec())),
            ("Parent", Object::Reference(pages_id)),
            ("Resources", Object::Dictionary(resources_dict)),
            ("MediaBox", Object::Array(vec![
                Object::Integer(0), Object::Integer(0),
                Object::Real(page_content.width as f32),
                Object::Real(page_content.height as f32),
            ])),
            ("Contents", Object::Reference(content_id)),
        ]);

        let page_id = doc.add_object(page_dict);
        page_ids.push(page_id);
    }

    let pages_dict = Dictionary::from_iter(vec![
        ("Type", Object::Name(b"Pages".to_vec())),
        ("Kids", Object::Array(page_ids.iter().map(|&id| Object::Reference(id)).collect())),
        ("Count", Object::Integer(page_ids.len() as i64)),
    ]);
    doc.objects.insert(pages_id, Object::Dictionary(pages_dict));

    // Update parent references
    for &page_id in &page_ids {
        if let Some(Object::Dictionary(ref mut d)) = doc.objects.get_mut(&page_id) {
            d.set("Parent", Object::Reference(pages_id));
        }
    }

    let catalog_id = doc.add_object(Dictionary::from_iter(vec![
        ("Type", Object::Name(b"Catalog".to_vec())),
        ("Pages", Object::Reference(pages_id)),
    ]));
    doc.trailer.set("Root", Object::Reference(catalog_id));

    let mut buf = Vec::new();
    doc.save_to(&mut buf)?;
    Ok(buf)
}

/// Create a simple text PDF document from a Writer Document
pub fn create_pdf_from_document(doc: &crate::core::Document) -> Result<Vec<u8>, PdfError> {
    let (page_w, page_h) = PageSize::A4;
    let margin = 56.0; // ~2cm
    let content_w = page_w - 2.0 * margin;

    let mut pages = vec![];
    let mut current_page = PdfPageContent::new(page_w, page_h);
    let mut y = page_h - margin; // start from top (PDF coords: 0 = bottom)

    // White background
    current_page.fill_rect(0.0, 0.0, page_w, page_h, &Color::WHITE);

    for block in &doc.body {
        use crate::core::document::Block;
        match block {
            Block::Paragraph(para) => {
                let text: String = para.runs.iter().map(|r| r.text.as_str()).collect::<Vec<_>>().join("");
                if !text.is_empty() {
                    y -= 14.0;
                    if y < margin {
                        pages.push(current_page);
                        current_page = PdfPageContent::new(page_w, page_h);
                        current_page.fill_rect(0.0, 0.0, page_w, page_h, &Color::WHITE);
                        y = page_h - margin;
                    }
                    current_page.draw_text(&text, margin, y, 12.0, &Color::BLACK);
                    y -= 6.0; // space after
                }
            }
            Block::Heading(h) => {
                let text: String = h.runs.iter().map(|r| r.text.as_str()).collect::<Vec<_>>().join("");
                let sizes = [24.0, 20.0, 16.0, 14.0, 12.0, 11.0];
                let size = sizes.get(h.level.saturating_sub(1) as usize).copied().unwrap_or(16.0);
                y -= size + 4.0;
                if y < margin + size {
                    pages.push(current_page);
                    current_page = PdfPageContent::new(page_w, page_h);
                    current_page.fill_rect(0.0, 0.0, page_w, page_h, &Color::WHITE);
                    y = page_h - margin;
                }
                current_page.draw_text(&text, margin, y, size, &Color::BLACK);
                y -= 8.0;
            }
            Block::HorizontalRule => {
                y -= 8.0;
                current_page.stroke_rect(margin, y, content_w, 0.5, &Color::from_hex("#AAAAAA").unwrap(), 0.5);
                y -= 8.0;
            }
            _ => {}
        }
    }

    pages.push(current_page);
    create_pdf(&pages)
}
