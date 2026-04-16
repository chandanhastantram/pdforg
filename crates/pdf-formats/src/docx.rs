//! DOCX parser and writer — reads/writes Word XML format.

use crate::FormatError;
use pdf_core::document::*;
use std::io::{Read, Write, Cursor, Seek};
use std::collections::HashMap;
use zip::{ZipArchive, ZipWriter, write::FileOptions};
use quick_xml::events::Event;
use quick_xml::Reader;
use uuid::Uuid;

/// Parse a .docx file from bytes
pub fn parse_docx(bytes: &[u8]) -> Result<Document, FormatError> {
    let cursor = Cursor::new(bytes);
    let mut archive = ZipArchive::new(cursor)?;

    // Read document.xml
    let doc_xml = read_zip_entry(&mut archive, "word/document.xml")?;
    let styles_xml = read_zip_entry(&mut archive, "word/styles.xml").unwrap_or_default();
    let rels_xml = read_zip_entry(&mut archive, "word/_rels/document.xml.rels").unwrap_or_default();

    let mut doc = Document::default();
    doc.body = parse_document_xml(&doc_xml)?;

    // Parse styles if available
    if !styles_xml.is_empty() {
        doc.styles = parse_styles_xml(&styles_xml)?;
    }

    Ok(doc)
}

fn read_zip_entry<R: Read + Seek>(archive: &mut ZipArchive<R>, name: &str) -> Result<String, FormatError> {
    let mut entry = archive.by_name(name).map_err(FormatError::Zip)?;
    let mut content = String::new();
    entry.read_to_string(&mut content).map_err(|e| FormatError::Io(e.into()))?;
    Ok(content)
}

fn parse_document_xml(xml: &str) -> Result<Vec<Block>, FormatError> {
    let mut reader = Reader::from_str(xml);
    reader.trim_text(true);

    let mut blocks = vec![];
    let mut current_para: Option<Paragraph> = None;
    let mut current_run: Option<Run> = None;
    let mut in_run_props = false;
    let mut bold = false;
    let mut italic = false;
    let mut underline = false;
    let mut buf = Vec::new();
    let mut depth_stack: Vec<&str> = vec![];

    loop {
        match reader.read_event_into(&mut buf) {
            Err(e) => return Err(FormatError::Xml(e.to_string())),
            Ok(Event::Eof) => break,
            Ok(Event::Start(ref e)) => {
                match e.name().as_ref() {
                    b"w:p" => {
                        current_para = Some(Paragraph::default());
                        bold = false;
                        italic = false;
                        underline = false;
                    }
                    b"w:r" => {
                        if current_para.is_some() {
                            current_run = Some(Run::new(""));
                            if bold || italic || underline {
                                let run = current_run.as_mut().unwrap();
                                run.format.bold = bold;
                                run.format.italic = italic;
                                run.format.underline = underline;
                            }
                        }
                    }
                    b"w:rPr" => { in_run_props = true; }
                    b"w:b" if in_run_props => { bold = true; }
                    b"w:i" if in_run_props => { italic = true; }
                    b"w:u" if in_run_props => { underline = true; }
                    b"w:pStyle" => {
                        if let Some(ref mut para) = current_para {
                            for attr in e.attributes().flatten() {
                                if attr.key.as_ref() == b"w:val" {
                                    let val = String::from_utf8_lossy(&attr.value).to_string();
                                    // Check if it's a heading style
                                    if val.starts_with("Heading") || val.starts_with("heading") {
                                        para.style_ref = Some(val);
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::Empty(ref e)) => {
                match e.name().as_ref() {
                    b"w:br" => {
                        // Line break
                        if let Some(ref mut run) = current_run {
                            run.text.push('\n');
                        }
                    }
                    b"w:b" if in_run_props => { bold = true; }
                    b"w:i" if in_run_props => { italic = true; }
                    b"w:u" if in_run_props => { underline = true; }
                    _ => {}
                }
            }
            Ok(Event::End(ref e)) => {
                match e.name().as_ref() {
                    b"w:p" => {
                        if let Some(para) = current_para.take() {
                            // Check if it's a heading based on style
                            let style = para.style_ref.clone().unwrap_or_default();
                            let heading_level = style.strip_prefix("Heading")
                                .or_else(|| style.strip_prefix("heading"))
                                .and_then(|s| s.trim().parse::<u8>().ok());

                            if let Some(level) = heading_level {
                                blocks.push(Block::Heading(Heading {
                                    id: Uuid::new_v4(),
                                    level,
                                    runs: para.runs,
                                    numbering: None,
                                }));
                            } else {
                                blocks.push(Block::Paragraph(para));
                            }
                        }
                        bold = false; italic = false; underline = false;
                    }
                    b"w:r" => {
                        if let (Some(run), Some(ref mut para)) = (current_run.take(), &mut current_para) {
                            if !run.text.is_empty() || !para.runs.is_empty() {
                                para.runs.push(run);
                            }
                        }
                    }
                    b"w:rPr" => {
                        in_run_props = false;
                        // Apply accumulated formatting to current run
                        if let Some(ref mut run) = current_run {
                            run.format.bold = bold;
                            run.format.italic = italic;
                            run.format.underline = underline;
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::Text(ref e)) => {
                if let Some(ref mut run) = current_run {
                    match e.unescape() {
                        Ok(text) => run.text.push_str(&text),
                        Err(_) => {}
                    }
                }
            }
            _ => {}
        }
        buf.clear();
    }

    Ok(blocks)
}

fn parse_styles_xml(xml: &str) -> Result<StyleMap, FormatError> {
    // Returns a default StyleMap for Phase 1
    // Full style parsing (with inheritance resolution) is Phase 2
    Ok(StyleMap::default())
}

/// Write a Document to .docx bytes
pub fn write_docx(doc: &Document) -> Result<Vec<u8>, FormatError> {
    let buf = Vec::new();
    let mut cursor = Cursor::new(buf);
    let mut zip = ZipWriter::new(&mut cursor);
    let options = FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    // [Content_Types].xml
    zip.start_file("[Content_Types].xml", options)?;
    zip.write_all(CONTENT_TYPES_XML.as_bytes())?;

    // _rels/.rels
    zip.start_file("_rels/.rels", options)?;
    zip.write_all(ROOT_RELS_XML.as_bytes())?;

    // word/_rels/document.xml.rels
    zip.start_file("word/_rels/document.xml.rels", options)?;
    zip.write_all(DOC_RELS_XML.as_bytes())?;

    // word/document.xml
    zip.start_file("word/document.xml", options)?;
    let doc_xml = write_document_xml(doc)?;
    zip.write_all(doc_xml.as_bytes())?;

    // word/styles.xml
    zip.start_file("word/styles.xml", options)?;
    zip.write_all(DEFAULT_STYLES_XML.as_bytes())?;

    zip.finish()?;
    drop(zip);
    Ok(cursor.into_inner())
}

fn write_document_xml(doc: &Document) -> Result<String, FormatError> {
    let mut xml = String::new();
    xml.push_str(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#);
    xml.push_str(r#"<w:document xmlns:wpc="http://schemas.microsoft.com/office/word/2010/wordprocessingCanvas" xmlns:mo="http://schemas.microsoft.com/office/mac/office/2008/main" xmlns:mc="http://schemas.openxmlformats.org/markup-compatibility/2006" xmlns:mv="urn:schemas-microsoft-com:mac:vml" xmlns:o="urn:schemas-microsoft-com:office:office" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:m="http://schemas.openxmlformats.org/officeDocument/2006/math" xmlns:v="urn:schemas-microsoft-com:vml" xmlns:wp14="http://schemas.microsoft.com/office/word/2010/wordprocessingDrawing" xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing" xmlns:w10="urn:schemas-microsoft-com:office:word" xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" xmlns:w14="http://schemas.microsoft.com/office/word/2010/wordml" xmlns:wpg="http://schemas.microsoft.com/office/word/2010/wordprocessingGroup" xmlns:wpi="http://schemas.microsoft.com/office/word/2010/wordprocessingInk" xmlns:wne="http://schemas.microsoft.com/office/word/2006/wordml" xmlns:pdf="http://schemas.microsoft.com/office/word/2010/wordprocessingShape" mc:Ignorable="mv mo w14 wp14"><w:body>"#);

    for block in &doc.body {
        xml.push_str(&write_block(block));
    }

    // Sectional properties
    xml.push_str(r#"<w:sectPr><w:pgSz w:w="12240" w:h="15840"/><w:pgMar w:top="1440" w:right="1800" w:bottom="1440" w:left="1800" w:header="720" w:footer="720" w:gutter="0"/></w:sectPr>"#);
    xml.push_str("</w:body></w:document>");
    Ok(xml)
}

fn write_block(block: &Block) -> String {
    match block {
        Block::Paragraph(para) => write_paragraph(para, None),
        Block::Heading(h) => write_paragraph(
            &Paragraph { runs: h.runs.clone(), ..Paragraph::default() },
            Some(&format!("Heading{}", h.level))
        ),
        Block::HorizontalRule => write_paragraph(&Paragraph::default(), Some("Normal")),
        _ => String::new(),
    }
}

fn write_paragraph(para: &Paragraph, style_override: Option<&str>) -> String {
    let mut xml = String::from("<w:p>");

    // Paragraph properties
    xml.push_str("<w:pPr>");
    let style = style_override.or(para.style_ref.as_deref()).unwrap_or("Normal");
    xml.push_str(&format!("<w:pStyle w:val=\"{}\"/>", escape_xml(style)));
    if para.space_after > 0.0 {
        let spacing_pt = (para.space_after * 20.0) as i32; // twentieths of a point
        xml.push_str(&format!("<w:spacing w:after=\"{}\"/>", spacing_pt));
    }
    xml.push_str("</w:pPr>");

    // Runs
    for run in &para.runs {
        xml.push_str("<w:r>");
        if run.format.bold || run.format.italic || run.format.underline {
            xml.push_str("<w:rPr>");
            if run.format.bold { xml.push_str("<w:b/>"); }
            if run.format.italic { xml.push_str("<w:i/>"); }
            if run.format.underline { xml.push_str("<w:u w:val=\"single\"/>"); }
            xml.push_str("</w:rPr>");
        }
        xml.push_str("<w:t xml:space=\"preserve\">");
        xml.push_str(&escape_xml(&run.text));
        xml.push_str("</w:t>");
        xml.push_str("</w:r>");
    }

    xml.push_str("</w:p>");
    xml
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
     .replace('<', "&lt;")
     .replace('>', "&gt;")
     .replace('"', "&quot;")
     .replace('\'', "&apos;")
}

// ─── Static XML templates ────────────────────────────────────────────────────

const CONTENT_TYPES_XML: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
  <Override PartName="/word/styles.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml"/>
</Types>"#;

const ROOT_RELS_XML: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>
</Relationships>"#;

const DOC_RELS_XML: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/>
</Relationships>"#;

const DEFAULT_STYLES_XML: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:styles xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:style w:type="paragraph" w:styleId="Normal"><w:name w:val="Normal"/></w:style>
  <w:style w:type="paragraph" w:styleId="Heading1"><w:name w:val="heading 1"/>
    <w:basedOn w:val="Normal"/>
    <w:rPr><w:b/><w:sz w:val="48"/></w:rPr>
  </w:style>
  <w:style w:type="paragraph" w:styleId="Heading2"><w:name w:val="heading 2"/>
    <w:basedOn w:val="Normal"/>
    <w:rPr><w:b/><w:sz w:val="36"/></w:rPr>
  </w:style>
</w:styles>"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_docx_round_trip() {
        let mut doc = Document::default();
        doc.body = vec![
            Block::Heading(Heading {
                id: Uuid::new_v4(), level: 1,
                runs: vec![Run::new("Test Document")],
                numbering: None,
            }),
            Block::Paragraph(Paragraph {
                runs: vec![Run::new("Hello, World!")],
                ..Default::default()
            }),
        ];

        let bytes = write_docx(&doc).unwrap();
        let parsed = parse_docx(&bytes).unwrap();

        // We should have our content back
        assert!(!parsed.body.is_empty());
    }
}
