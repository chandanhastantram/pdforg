//! XLSX parser and writer.

use crate::FormatError;
use pdf_core::spreadsheet::*;
use pdf_core::common::Color;
use std::io::{Read, Write, Cursor, Seek};
use std::collections::HashMap;
use zip::{ZipArchive, ZipWriter, write::FileOptions};
use quick_xml::events::Event;
use quick_xml::Reader;
use uuid::Uuid;

/// Parse a .xlsx file from bytes
pub fn parse_xlsx(bytes: &[u8]) -> Result<Workbook, FormatError> {
    let cursor = Cursor::new(bytes);
    let mut archive = ZipArchive::new(cursor)?;

    let workbook_xml = read_zip_str(&mut archive, "xl/workbook.xml").unwrap_or_default();
    let shared_strings_xml = read_zip_str(&mut archive, "xl/sharedStrings.xml").unwrap_or_default();

    let shared_strings = parse_shared_strings(&shared_strings_xml);
    let sheet_names = parse_sheet_names(&workbook_xml);

    let mut workbook = Workbook {
        id: Uuid::new_v4(),
        title: "Workbook".into(),
        sheets: vec![],
        active_sheet: 0,
        named_ranges: HashMap::new(),
        defined_names: HashMap::new(),
        shared_strings: shared_strings.clone(),
    };

    for (idx, name) in sheet_names.iter().enumerate() {
        let sheet_path = format!("xl/worksheets/sheet{}.xml", idx + 1);
        let sheet_xml = read_zip_str(&mut archive, &sheet_path).unwrap_or_default();
        if !sheet_xml.is_empty() {
            let sheet = parse_sheet_xml(&sheet_xml, name, &shared_strings)?;
            workbook.sheets.push(sheet);
        }
    }

    if workbook.sheets.is_empty() {
        workbook.sheets.push(Sheet::new("Sheet1"));
    }

    Ok(workbook)
}

fn read_zip_str<R: Read + Seek>(archive: &mut ZipArchive<R>, name: &str) -> Option<String> {
    let mut entry = archive.by_name(name).ok()?;
    let mut content = String::new();
    entry.read_to_string(&mut content).ok()?;
    Some(content)
}

fn parse_shared_strings(xml: &str) -> Vec<String> {
    if xml.is_empty() { return vec![]; }
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);
    let mut strings = vec![];
    let mut current = String::new();
    let mut in_t = false;
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) if e.name().as_ref() == b"t" => { in_t = true; }
            Ok(Event::Text(ref e)) if in_t => {
                current.push_str(&e.unescape().unwrap_or_default());
            }
            Ok(Event::End(ref e)) if e.name().as_ref() == b"t" => { in_t = false; }
            Ok(Event::End(ref e)) if e.name().as_ref() == b"si" => {
                strings.push(current.clone());
                current.clear();
            }
            Ok(Event::Eof) => break,
            _ => {}
        }
        buf.clear();
    }
    strings
}

fn parse_sheet_names(xml: &str) -> Vec<String> {
    if xml.is_empty() { return vec!["Sheet1".into()]; }
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);
    let mut names = vec![];
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Empty(ref e)) if e.name().as_ref() == b"sheet" => {
                let name = e.attributes().flatten()
                    .find(|a| a.key.as_ref() == b"name")
                    .and_then(|a| String::from_utf8(a.value.to_vec()).ok())
                    .unwrap_or_else(|| format!("Sheet{}", names.len() + 1));
                names.push(name);
            }
            Ok(Event::Eof) => break,
            _ => {}
        }
        buf.clear();
    }

    if names.is_empty() { vec!["Sheet1".into()] } else { names }
}

fn parse_sheet_xml(xml: &str, name: &str, shared_strings: &[String]) -> Result<Sheet, FormatError> {
    let mut sheet = Sheet::new(name);
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();
    let mut current_cell_ref = String::new();
    let mut current_cell_type = String::new();
    let mut current_value = String::new();
    let mut in_v = false;
    let mut in_f = false;
    let mut current_formula = String::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                match e.name().as_ref() {
                    b"c" => {
                        current_cell_ref.clear();
                        current_cell_type.clear();
                        current_value.clear();
                        current_formula.clear();
                        in_v = false;
                        in_f = false;

                        for attr in e.attributes().flatten() {
                            match attr.key.as_ref() {
                                b"r" => current_cell_ref = String::from_utf8_lossy(&attr.value).to_string(),
                                b"t" => current_cell_type = String::from_utf8_lossy(&attr.value).to_string(),
                                _ => {}
                            }
                        }
                    }
                    b"v" => { in_v = true; }
                    b"f" => { in_f = true; }
                    _ => {}
                }
            }
            Ok(Event::Text(ref e)) => {
                let text = e.unescape().unwrap_or_default().to_string();
                if in_v { current_value = text; }
                else if in_f { current_formula = text; }
            }
            Ok(Event::End(ref e)) => {
                match e.name().as_ref() {
                    b"v" => { in_v = false; }
                    b"f" => { in_f = false; }
                    b"c" => {
                        if let Some(addr) = CellAddress::from_a1(&current_cell_ref) {
                            let value = match current_cell_type.as_str() {
                                "s" => {
                                    // Shared string index
                                    let idx: usize = current_value.parse().unwrap_or(0);
                                    CellValue::Text(shared_strings.get(idx).cloned().unwrap_or_default())
                                }
                                "b" => CellValue::Bool(current_value == "1"),
                                "e" => CellValue::Error(pdf_core::CellError::Value),
                                _ => {
                                    if current_value.is_empty() {
                                        CellValue::Empty
                                    } else {
                                        current_value.parse::<f64>()
                                            .map(CellValue::Number)
                                            .unwrap_or(CellValue::Text(current_value.clone()))
                                    }
                                }
                            };

                            let formula = if current_formula.is_empty() { None } else {
                                Some(format!("={}", current_formula))
                            };

                            sheet.set_cell(addr.row, addr.col, Cell {
                                value,
                                formula,
                                ..Default::default()
                            });
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::Eof) => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(sheet)
}

/// Write Workbook to .xlsx bytes
pub fn write_xlsx(wb: &Workbook) -> Result<Vec<u8>, FormatError> {
    let buf = Vec::new();
    let mut cursor = Cursor::new(buf);
    let mut zip = ZipWriter::new(&mut cursor);
    let opts = FileOptions::<'_, ()>::default().compression_method(zip::CompressionMethod::Deflated);

    zip.start_file("[Content_Types].xml", opts)?;
    zip.write_all(XLSX_CONTENT_TYPES.as_bytes())?;

    zip.start_file("_rels/.rels", opts)?;
    zip.write_all(XLSX_ROOT_RELS.as_bytes())?;

    zip.start_file("xl/_rels/workbook.xml.rels", opts)?;
    let sheet_rels: String = wb.sheets.iter().enumerate().map(|(i, _)| {
        format!(r#"<Relationship Id="rId{}" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet{}.xml"/>"#, i+1, i+1)
    }).collect();
    let rels_content = format!(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">{}</Relationships>"#, sheet_rels);
    zip.write_all(rels_content.as_bytes())?;

    // workbook.xml
    zip.start_file("xl/workbook.xml", opts)?;
    let sheets_xml: String = wb.sheets.iter().enumerate().map(|(i, s)| {
        format!(r#"<sheet name="{}" sheetId="{}" r:id="rId{}"/>"#, s.name, i+1, i+1)
    }).collect();
    let wb_xml = format!(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"><sheets>{}</sheets></workbook>"#, sheets_xml);
    zip.write_all(wb_xml.as_bytes())?;

    // Build shared strings
    let mut all_strings: Vec<String> = vec![];
    let mut string_map: HashMap<String, usize> = HashMap::new();
    for sheet in &wb.sheets {
        for (_, cell) in &sheet.cells {
            if let CellValue::Text(s) = &cell.value {
                if !string_map.contains_key(s) {
                    string_map.insert(s.clone(), all_strings.len());
                    all_strings.push(s.clone());
                }
            }
        }
    }

    // sharedStrings.xml
    if !all_strings.is_empty() {
        zip.start_file("xl/sharedStrings.xml", opts)?;
        let si: String = all_strings.iter().map(|s| format!("<si><t>{}</t></si>", escape_xml(s))).collect();
        let ss_xml = format!(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><sst xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" count="{0}" uniqueCount="{0}">{1}</sst>"#, all_strings.len(), si);
        zip.write_all(ss_xml.as_bytes())?;
    }

    // Sheet files
    for (idx, sheet) in wb.sheets.iter().enumerate() {
        zip.start_file(&format!("xl/worksheets/sheet{}.xml", idx + 1), opts)?;
        let sheet_xml = write_sheet_xml(sheet, &string_map)?;
        zip.write_all(sheet_xml.as_bytes())?;
    }

    zip.finish()?;
    Ok(cursor.into_inner())
}

fn write_sheet_xml(sheet: &Sheet, string_map: &HashMap<String, usize>) -> Result<String, FormatError> {
    let mut xml = String::from(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"><sheetData>"#);

    // Group cells by row
    let mut rows: HashMap<u32, Vec<(u32, &Cell)>> = HashMap::new();
    for ((row, col), cell) in &sheet.cells {
        rows.entry(*row).or_default().push((*col, cell));
    }
    let mut sorted_rows: Vec<u32> = rows.keys().copied().collect();
    sorted_rows.sort();

    for row_num in sorted_rows {
        if let Some(cells) = rows.get(&row_num) {
            xml.push_str(&format!(r#"<row r="{}">"#, row_num + 1));
            let mut sorted_cells = cells.clone();
            sorted_cells.sort_by_key(|(col, _)| *col);

            for (col, cell) in sorted_cells {
                let addr = CellAddress::new(row_num, col).to_a1();
                let (type_attr, val_content) = match &cell.value {
                    CellValue::Empty => { continue; }
                    CellValue::Text(s) => {
                        let idx = string_map.get(s).copied().unwrap_or(0);
                        ("s", idx.to_string())
                    }
                    CellValue::Number(n) => ("n", n.to_string()),
                    CellValue::Bool(b) => ("b", if *b { "1".into() } else { "0".into() }),
                    CellValue::Error(e) => ("e", e.to_string()),
                    _ => ("n", "0".into()),
                };

                if let Some(formula) = &cell.formula {
                    xml.push_str(&format!(r#"<c r="{}" t="{}"><f>{}</f><v>{}</v></c>"#,
                        addr, type_attr,
                        escape_xml(formula.trim_start_matches('=')),
                        escape_xml(&val_content)));
                } else {
                    xml.push_str(&format!(r#"<c r="{}" t="{}"><v>{}</v></c>"#, addr, type_attr, escape_xml(&val_content)));
                }
            }
            xml.push_str("</row>");
        }
    }

    xml.push_str("</sheetData></worksheet>");
    Ok(xml)
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
     .replace('<', "&lt;")
     .replace('>', "&gt;")
     .replace('"', "&quot;")
}

const XLSX_CONTENT_TYPES: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Override PartName="/xl/workbook.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml"/>
  <Override PartName="/xl/worksheets/sheet1.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml"/>
  <Override PartName="/xl/sharedStrings.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sharedStrings+xml"/>
</Types>"#;

const XLSX_ROOT_RELS: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="xl/workbook.xml"/>
</Relationships>"#;
