//! Writer document model — the canonical in-memory representation for word processor documents.

use crate::common::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Top-level document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: Uuid,
    pub title: String,
    pub body: Vec<Block>,
    pub styles: StyleMap,
    pub page_layout: PageLayout,
    pub metadata: DocumentMetadata,
    pub revision: u64,
}

impl Default for Document {
    fn default() -> Self {
        Document {
            id: Uuid::new_v4(),
            title: "Untitled Document".into(),
            body: vec![Block::Paragraph(Paragraph::default())],
            styles: StyleMap::default(),
            page_layout: PageLayout::default(),
            metadata: DocumentMetadata::default(),
            revision: 0,
        }
    }
}

/// Document metadata
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DocumentMetadata {
    pub author: String,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub modified_at: Option<chrono::DateTime<chrono::Utc>>,
    pub description: String,
    pub keywords: Vec<String>,
    pub language: String,
}

/// Block-level elements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Block {
    Paragraph(Paragraph),
    Heading(Heading),
    Table(Table),
    List(List),
    Image(ImageBlock),
    CodeBlock(CodeBlock),
    BlockQuote(Vec<Block>),
    PageBreak,
    HorizontalRule,
}

/// A paragraph with runs of text
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Paragraph {
    pub id: Uuid,
    pub runs: Vec<Run>,
    pub style_ref: Option<StyleRef>,
    pub align: TextAlign,
    pub indent_left: f32,   // mm
    pub indent_right: f32,  // mm
    pub indent_first: f32,  // mm
    pub space_before: f32,  // pt
    pub space_after: f32,   // pt
    pub line_height: LineHeight,
    pub keep_together: bool,
    pub keep_with_next: bool,
    pub page_break_before: bool,
}

impl Default for Paragraph {
    fn default() -> Self {
        Paragraph {
            id: Uuid::new_v4(),
            runs: vec![],
            style_ref: None,
            align: TextAlign::Left,
            indent_left: 0.0,
            indent_right: 0.0,
            indent_first: 0.0,
            space_before: 0.0,
            space_after: 8.0,
            line_height: LineHeight::Multiple(1.15),
            keep_together: false,
            keep_with_next: false,
            page_break_before: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LineHeight {
    Auto,
    Exact(f32),      // pt
    AtLeast(f32),    // pt
    Multiple(f32),   // multiplier
}

/// A heading with level
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Heading {
    pub id: Uuid,
    pub level: u8,  // 1–6
    pub runs: Vec<Run>,
    pub numbering: Option<String>,
}

/// A run of text with consistent formatting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Run {
    pub id: Uuid,
    pub text: String,
    pub format: RunFormat,
    pub comment_refs: Vec<Uuid>,
    pub tracked_change: Option<TrackedChangeRef>,
}

impl Run {
    pub fn new(text: impl Into<String>) -> Self {
        Run {
            id: Uuid::new_v4(),
            text: text.into(),
            format: RunFormat::default(),
            comment_refs: vec![],
            tracked_change: None,
        }
    }

    pub fn bold(mut self) -> Self { self.format.bold = true; self }
    pub fn italic(mut self) -> Self { self.format.italic = true; self }
    pub fn underline(mut self) -> Self { self.format.underline = true; self }
}

/// Text run formatting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunFormat {
    pub font: Option<FontSpec>,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strikethrough: bool,
    pub subscript: bool,
    pub superscript: bool,
    pub color: Option<Color>,
    pub background: Option<Color>,
    pub link: Option<String>,
    pub style_ref: Option<StyleRef>,
}

impl Default for RunFormat {
    fn default() -> Self {
        RunFormat {
            font: None,
            bold: false,
            italic: false,
            underline: false,
            strikethrough: false,
            subscript: false,
            superscript: false,
            color: None,
            background: None,
            link: None,
            style_ref: None,
        }
    }
}

/// Tracked change reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackedChangeRef {
    pub change_id: Uuid,
    pub kind: ChangeKind,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ChangeKind {
    Insertion,
    Deletion,
    FormatChange,
    Move,
}

/// Image block in writer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageBlock {
    pub id: Uuid,
    pub data: Vec<u8>,
    pub mime_type: String,
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub alt: String,
    pub caption: Option<String>,
    pub wrap: ImageWrap,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum ImageWrap {
    #[default]
    Inline,
    FloatLeft,
    FloatRight,
    Behind,
    InFront,
}

/// Code block
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeBlock {
    pub id: Uuid,
    pub language: String,
    pub code: String,
}

/// Table
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Table {
    pub id: Uuid,
    pub rows: Vec<TableRow>,
    pub col_widths: Vec<f32>,
    pub style_ref: Option<StyleRef>,
    pub border: Option<Border>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableRow {
    pub id: Uuid,
    pub cells: Vec<TableCell>,
    pub height: Option<f32>,
    pub is_header: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableCell {
    pub id: Uuid,
    pub content: Vec<Block>,
    pub rowspan: u32,
    pub colspan: u32,
    pub background: Option<Color>,
    pub border: Option<Border>,
    pub vertical_align: VerticalAlign,
}

impl Default for TableCell {
    fn default() -> Self {
        TableCell {
            id: Uuid::new_v4(),
            content: vec![Block::Paragraph(Paragraph::default())],
            rowspan: 1,
            colspan: 1,
            background: None,
            border: None,
            vertical_align: VerticalAlign::Top,
        }
    }
}

/// List (ordered or unordered)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct List {
    pub id: Uuid,
    pub kind: ListKind,
    pub items: Vec<ListItem>,
    pub level: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ListKind {
    Bullet,
    Ordered(OrderedStyle),
    CheckList,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OrderedStyle {
    Decimal,
    LowerAlpha,
    UpperAlpha,
    LowerRoman,
    UpperRoman,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListItem {
    pub id: Uuid,
    pub content: Vec<Block>,
    pub checked: Option<bool>,
}

/// Style definitions
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StyleMap {
    pub styles: HashMap<StyleRef, Style>,
    pub default_paragraph_style: StyleRef,
    pub default_character_style: StyleRef,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Style {
    pub id: StyleRef,
    pub name: String,
    pub kind: StyleKind,
    pub parent: Option<StyleRef>,
    pub font: Option<FontSpec>,
    pub paragraph: Option<ParagraphStyleProps>,
    pub run_format: Option<RunFormat>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StyleKind {
    Paragraph,
    Character,
    Table,
    List,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParagraphStyleProps {
    pub align: Option<TextAlign>,
    pub indent_left: Option<f32>,
    pub indent_right: Option<f32>,
    pub indent_first: Option<f32>,
    pub space_before: Option<f32>,
    pub space_after: Option<f32>,
    pub line_height: Option<LineHeight>,
}

/// Comment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    pub id: Uuid,
    pub author: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub content: String,
    pub resolved: bool,
    pub replies: Vec<CommentReply>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommentReply {
    pub id: Uuid,
    pub author: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub content: String,
}

/// Document snapshot for versioning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentSnapshot {
    pub id: Uuid,
    pub document_id: Uuid,
    pub version_label: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub created_by: String,
    pub document: Document,
}
