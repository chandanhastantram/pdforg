//! Spreadsheet model — cells, sheets, workbooks.

use super::common::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// A cell address (row, col — both 0-indexed)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CellAddress {
    pub row: u32,
    pub col: u32,
    pub sheet: Option<String>,  // cross-sheet reference
}

impl CellAddress {
    pub fn new(row: u32, col: u32) -> Self {
        CellAddress { row, col, sheet: None }
    }

    /// Parse "A1" → CellAddress(0, 0)
    pub fn from_a1(s: &str) -> Option<Self> {
        let s = s.trim();
        let col_end = s.chars().take_while(|c| c.is_ascii_alphabetic()).count();
        let col_str = &s[..col_end];
        let row_str = &s[col_end..];

        let col = col_str.chars().fold(0u32, |acc, c| {
            acc * 26 + (c.to_ascii_uppercase() as u32 - 'A' as u32 + 1)
        }) - 1;
        let row: u32 = row_str.parse::<u32>().ok()?.saturating_sub(1);
        Some(CellAddress::new(row, col))
    }

    /// Convert to "A1" notation
    pub fn to_a1(&self) -> String {
        let mut col = self.col;
        let mut col_str = String::new();
        loop {
            col_str.insert(0, (b'A' + (col % 26) as u8) as char);
            if col < 26 { break; }
            col = col / 26 - 1;
        }
        format!("{}{}", col_str, self.row + 1)
    }
}

/// A cell range reference
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CellRange {
    pub start: CellAddress,
    pub end: CellAddress,
}

impl CellRange {
    pub fn new(start: CellAddress, end: CellAddress) -> Self {
        CellRange { start, end }
    }

    pub fn contains(&self, addr: &CellAddress) -> bool {
        addr.row >= self.start.row && addr.row <= self.end.row
            && addr.col >= self.start.col && addr.col <= self.end.col
    }

    pub fn cells(&self) -> Vec<CellAddress> {
        let mut result = vec![];
        for row in self.start.row..=self.end.row {
            for col in self.start.col..=self.end.col {
                result.push(CellAddress::new(row, col));
            }
        }
        result
    }
}

/// Cell value
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CellValue {
    Empty,
    Text(String),
    Number(f64),
    Bool(bool),
    Error(CellError),
    Date(chrono::NaiveDate),
    DateTime(chrono::NaiveDateTime),
}

impl Default for CellValue {
    fn default() -> Self { CellValue::Empty }
}

impl CellValue {
    pub fn as_number(&self) -> Option<f64> {
        match self {
            CellValue::Number(n) => Some(*n),
            CellValue::Bool(b) => Some(if *b { 1.0 } else { 0.0 }),
            CellValue::Text(s) => s.parse::<f64>().ok(),
            _ => None,
        }
    }

    pub fn as_text(&self) -> String {
        match self {
            CellValue::Empty => String::new(),
            CellValue::Text(s) => s.clone(),
            CellValue::Number(n) => n.to_string(),
            CellValue::Bool(b) => if *b { "TRUE".into() } else { "FALSE".into() },
            CellValue::Error(e) => e.to_string(),
            CellValue::Date(d) => d.to_string(),
            CellValue::DateTime(dt) => dt.to_string(),
        }
    }

    pub fn is_empty(&self) -> bool { matches!(self, CellValue::Empty) }
    pub fn is_number(&self) -> bool { matches!(self, CellValue::Number(_)) }
    pub fn is_text(&self) -> bool { matches!(self, CellValue::Text(_)) }
    pub fn is_error(&self) -> bool { matches!(self, CellValue::Error(_)) }
}

/// Cell error types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CellError {
    Div0,       // #DIV/0!
    NA,         // #N/A
    Name,       // #NAME?
    Null,       // #NULL!
    Num,        // #NUM!
    Ref,        // #REF!
    Value,      // #VALUE!
    Getting,    // #GETTING_DATA
    Spill,      // #SPILL!
}

impl std::fmt::Display for CellError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            CellError::Div0 => "#DIV/0!",
            CellError::NA => "#N/A",
            CellError::Name => "#NAME?",
            CellError::Null => "#NULL!",
            CellError::Num => "#NUM!",
            CellError::Ref => "#REF!",
            CellError::Value => "#VALUE!",
            CellError::Getting => "#GETTING_DATA",
            CellError::Spill => "#SPILL!",
        };
        write!(f, "{}", s)
    }
}

/// Cell number format
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct CellFormat {
    pub number_format: Option<String>,
    pub text_wrap: bool,
    pub locked: bool,
    pub hidden: bool,
}

/// Cell style reference
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct CellStyle {
    pub font: Option<FontSpec>,
    pub fill: Option<Color>,
    pub border_top: Option<Border>,
    pub border_bottom: Option<Border>,
    pub border_left: Option<Border>,
    pub border_right: Option<Border>,
    pub align: TextAlign,
    pub vertical_align: VerticalAlign,
    pub text_wrap: bool,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub color: Option<Color>,
}

/// A single spreadsheet cell
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cell {
    pub value: CellValue,
    pub formula: Option<String>,   // raw formula text e.g. "=SUM(A1:A10)"
    pub format: CellFormat,
    pub style: CellStyle,
    pub note: Option<String>,
}

impl Default for Cell {
    fn default() -> Self {
        Cell {
            value: CellValue::Empty,
            formula: None,
            format: CellFormat::default(),
            style: CellStyle::default(),
            note: None,
        }
    }
}

/// Column definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Column {
    pub width: f32,   // in characters (Excel unit)
    pub hidden: bool,
    pub style: Option<CellStyle>,
}

impl Default for Column {
    fn default() -> Self {
        Column { width: 8.43, hidden: false, style: None }
    }
}

/// Row definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Row {
    pub height: f32,  // pt
    pub hidden: bool,
    pub style: Option<CellStyle>,
}

impl Default for Row {
    fn default() -> Self {
        Row { height: 15.0, hidden: false, style: None }
    }
}

/// Merged cell region
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeRegion {
    pub range: CellRange,
}

/// Freeze panes
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FreezePane {
    pub rows: u32,
    pub cols: u32,
}

/// A single worksheet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sheet {
    pub id: Uuid,
    pub name: String,
    pub cells: HashMap<(u32, u32), Cell>,  // (row, col) → Cell
    pub rows: HashMap<u32, Row>,
    pub cols: HashMap<u32, Column>,
    pub merges: Vec<MergeRegion>,
    pub freeze_pane: FreezePane,
    pub hidden: bool,
    pub tab_color: Option<Color>,
    pub protection: Option<SheetProtection>,
    pub named_ranges: HashMap<String, CellRange>,
    pub charts: Vec<Chart>,
    pub max_row: u32,
    pub max_col: u32,
}

impl Sheet {
    pub fn new(name: impl Into<String>) -> Self {
        Sheet {
            id: Uuid::new_v4(),
            name: name.into(),
            cells: HashMap::new(),
            rows: HashMap::new(),
            cols: HashMap::new(),
            merges: vec![],
            freeze_pane: FreezePane::default(),
            hidden: false,
            tab_color: None,
            protection: None,
            named_ranges: HashMap::new(),
            charts: vec![],
            max_row: 0,
            max_col: 0,
        }
    }

    pub fn get_cell(&self, row: u32, col: u32) -> Option<&Cell> {
        self.cells.get(&(row, col))
    }

    pub fn set_cell(&mut self, row: u32, col: u32, cell: Cell) {
        self.max_row = self.max_row.max(row + 1);
        self.max_col = self.max_col.max(col + 1);
        self.cells.insert((row, col), cell);
    }

    pub fn row_height(&self, row: u32) -> f32 {
        self.rows.get(&row).map(|r| r.height).unwrap_or(15.0)
    }

    pub fn col_width(&self, col: u32) -> f32 {
        self.cols.get(&col).map(|c| c.width).unwrap_or(8.43)
    }
}

/// Sheet protection settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SheetProtection {
    pub password_hash: Option<String>,
    pub lock_cells: bool,
    pub lock_objects: bool,
    pub lock_scenarios: bool,
}

/// Chart model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chart {
    pub id: Uuid,
    pub kind: ChartKind,
    pub title: String,
    pub data_range: CellRange,
    pub position: Transform,
    pub series: Vec<ChartSeries>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ChartKind {
    Bar,
    Column,
    Line,
    Area,
    Pie,
    Donut,
    Scatter,
    Bubble,
    Radar,
    Combo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartSeries {
    pub name: String,
    pub values: CellRange,
    pub categories: Option<CellRange>,
    pub color: Option<Color>,
}

/// A workbook (collection of sheets)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workbook {
    pub id: Uuid,
    pub title: String,
    pub sheets: Vec<Sheet>,
    pub active_sheet: usize,
    pub named_ranges: HashMap<String, CellRange>,
    pub defined_names: HashMap<String, String>,
    pub shared_strings: Vec<String>,
}

impl Default for Workbook {
    fn default() -> Self {
        Workbook {
            id: Uuid::new_v4(),
            title: "Untitled Spreadsheet".into(),
            sheets: vec![Sheet::new("Sheet1")],
            active_sheet: 0,
            named_ranges: HashMap::new(),
            defined_names: HashMap::new(),
            shared_strings: vec![],
        }
    }
}

impl Workbook {
    pub fn active_sheet(&self) -> Option<&Sheet> {
        self.sheets.get(self.active_sheet)
    }
    pub fn active_sheet_mut(&mut self) -> Option<&mut Sheet> {
        let idx = self.active_sheet;
        self.sheets.get_mut(idx)
    }
    pub fn sheet_by_name(&self, name: &str) -> Option<&Sheet> {
        self.sheets.iter().find(|s| s.name == name)
    }
}

/// Viewport for virtual grid rendering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Viewport {
    pub first_row: u32,
    pub first_col: u32,
    pub row_count: u32,
    pub col_count: u32,
}

/// Rendered cell data for front-end display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellRenderData {
    pub row: u32,
    pub col: u32,
    pub display_value: String,
    pub style: CellStyle,
    pub has_formula: bool,
    pub is_merged: bool,
    pub merge_region: Option<CellRange>,
}

/// Selection range
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectionRange {
    pub anchor: CellAddress,
    pub active: CellAddress,
    pub range: CellRange,
    pub is_primary: bool,
}

/// Data sent to front-end for viewport rendering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewportData {
    pub cells: Vec<CellRenderData>,
    pub row_heights: Vec<f32>,
    pub col_widths: Vec<f32>,
    pub frozen_rows: u32,
    pub frozen_cols: u32,
    pub selections: Vec<SelectionRange>,
}
