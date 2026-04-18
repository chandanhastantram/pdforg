//! Virtual viewport renderer — computes what the grid should show for a given scroll position.

use crate::core::{Sheet, Viewport, ViewportData, CellRenderData, CellStyle, CellValue};

/// Render a viewport of the sheet for display in the frontend
pub fn render_viewport(sheet: &Sheet, vp: &Viewport) -> ViewportData {
    let mut cells = vec![];
    let last_row = (vp.first_row + vp.row_count).min(sheet.max_row.max(vp.first_row + vp.row_count));
    let last_col = (vp.first_col + vp.col_count).min(sheet.max_col.max(vp.first_col + vp.col_count));

    for row in vp.first_row..last_row {
        for col in vp.first_col..last_col {
            let cell = sheet.get_cell(row, col);
            let display_value = cell.map(|c| format_cell_value(&c.value)).unwrap_or_default();
            let style = cell.map(|c| c.style.clone()).unwrap_or_default();
            let has_formula = cell.and_then(|c| c.formula.as_ref()).is_some();

            // Check if this cell is part of a merge region
            let merge = sheet.merges.iter().find(|m| m.range.contains(&crate::core::spreadsheet::CellAddress::new(row, col)));

            cells.push(CellRenderData {
                row,
                col,
                display_value,
                style,
                has_formula,
                is_merged: merge.is_some(),
                merge_region: merge.map(|m| m.range.clone()),
            });
        }
    }

    let row_heights: Vec<f32> = (vp.first_row..last_row)
        .map(|r| sheet.row_height(r))
        .collect();
    let col_widths: Vec<f32> = (vp.first_col..last_col)
        .map(|c| sheet.col_width(c))
        .collect();

    ViewportData {
        cells,
        row_heights,
        col_widths,
        frozen_rows: sheet.freeze_pane.rows,
        frozen_cols: sheet.freeze_pane.cols,
        selections: vec![],
    }
}

fn format_cell_value(v: &CellValue) -> String {
    match v {
        CellValue::Empty => String::new(),
        CellValue::Text(s) => s.clone(),
        CellValue::Number(n) => format_number(*n),
        CellValue::Bool(b) => if *b { "TRUE".into() } else { "FALSE".into() },
        CellValue::Error(e) => e.to_string(),
        CellValue::Date(d) => d.format("%Y-%m-%d").to_string(),
        CellValue::DateTime(dt) => dt.format("%Y-%m-%d %H:%M").to_string(),
    }
}

fn format_number(n: f64) -> String {
    if n.fract() == 0.0 && n.abs() < 1e15 {
        format!("{:.0}", n)
    } else {
        // Smart decimal formatting — up to 10 significant digits
        let s = format!("{:.10}", n);
        let trimmed = s.trim_end_matches('0').trim_end_matches('.');
        trimmed.to_string()
    }
}

// Extension trait to add addr-based contains
trait RangeExt {
    fn contains_addr(&self, row: u32, col: u32) -> bool;
}

impl RangeExt for crate::core::spreadsheet::CellRange {
    fn contains_addr(&self, row: u32, col: u32) -> bool {
        row >= self.start.row && row <= self.end.row
            && col >= self.start.col && col <= self.end.col
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{Sheet, Cell, CellValue};

    #[test]
    fn test_render_viewport() {
        let mut sheet = Sheet::new("Sheet1");
        sheet.set_cell(0, 0, Cell { value: CellValue::Number(42.0), ..Default::default() });
        sheet.set_cell(0, 1, Cell { value: CellValue::Text("Hello".into()), ..Default::default() });

        let vp = Viewport { first_row: 0, first_col: 0, row_count: 5, col_count: 5 };
        let data = render_viewport(&sheet, &vp);
        let cell = data.cells.iter().find(|c| c.row == 0 && c.col == 0).unwrap();
        assert_eq!(cell.display_value, "42");
        let cell2 = data.cells.iter().find(|c| c.row == 0 && c.col == 1).unwrap();
        assert_eq!(cell2.display_value, "Hello");
    }
}


