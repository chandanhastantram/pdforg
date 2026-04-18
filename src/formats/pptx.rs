//! PPTX parser and writer.

use super::FormatError;
use crate::core::presentation::*;
use crate::core::common::*;
use std::io::{Read, Write, Cursor, Seek};
use std::collections::HashMap;
use zip::{ZipArchive, ZipWriter, write::FileOptions};
use quick_xml::events::Event;
use quick_xml::Reader;
use uuid::Uuid;

/// Parse a .pptx file from bytes
pub fn parse_pptx(bytes: &[u8]) -> Result<Presentation, FormatError> {
    let cursor = Cursor::new(bytes);
    let mut archive = ZipArchive::new(cursor)?;

    let pptx_xml = read_zip_str(&mut archive, "ppt/presentation.xml").unwrap_or_default();
    let slide_count = count_slides(&archive);

    let mut pres = Presentation::default();
    pres.slides.clear();

    for i in 1..=slide_count {
        let slide_path = format!("ppt/slides/slide{}.xml", i);
        if let Some(slide_xml) = read_zip_str(&mut archive, &slide_path) {
            let slide = parse_slide_xml(&slide_xml)?;
            pres.slides.push(slide);
        }
    }

    if pres.slides.is_empty() {
        pres.slides.push(Slide::default());
    }

    Ok(pres)
}

fn read_zip_str<R: Read + Seek>(archive: &mut ZipArchive<R>, name: &str) -> Option<String> {
    let mut entry = archive.by_name(name).ok()?;
    let mut content = String::new();
    entry.read_to_string(&mut content).ok()?;
    Some(content)
}

fn count_slides<R: Read + Seek>(archive: &ZipArchive<R>) -> usize {
    (1..=500).take_while(|i| archive.index_for_name(&format!("ppt/slides/slide{}.xml", i)).is_some()).count()
}

fn parse_slide_xml(xml: &str) -> Result<Slide, FormatError> {
    let mut slide = Slide::default();
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();
    let mut in_sp = false;
    let mut current_tb: Option<TextBoxEl> = None;
    let mut current_para: Option<SlideParagraph> = None;
    let mut current_run: Option<SlideRun> = None;
    let mut in_t = false;
    let mut current_x = 0.0f32;
    let mut current_y = 0.0f32;
    let mut current_w = 200.0f32;
    let mut current_h = 50.0f32;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                match e.name().as_ref() {
                    b"p:sp" => {
                        in_sp = true;
                        current_tb = Some(TextBoxEl {
                            id: Uuid::new_v4(),
                            transform: Transform::default(),
                            paragraphs: vec![],
                            fill: None,
                            border: None,
                            padding: Padding { left: 5.0, right: 5.0, top: 5.0, bottom: 5.0 },
                            vertical_align: VerticalAlign::Top,
                            text_direction: TextDirection::LeftToRight,
                        });
                    }
                    b"a:p" if in_sp => {
                        current_para = Some(SlideParagraph {
                            runs: vec![], align: TextAlign::Left, space_before: 0.0,
                            space_after: 6.0, line_height: 1.2, level: 0, bullet: None,
                        });
                    }
                    b"a:r" if in_sp && current_para.is_some() => {
                        current_run = Some(SlideRun {
                            text: String::new(), font: None, bold: false,
                            italic: false, underline: false, color: None, link: None,
                        });
                    }
                    b"a:t" if current_run.is_some() => { in_t = true; }
                    b"a:b" => {
                        if let Some(ref mut run) = current_run { run.bold = true; }
                    }
                    b"a:i" => {
                        if let Some(ref mut run) = current_run { run.italic = true; }
                    }
                    _ => {}
                }
            }
            Ok(Event::Text(ref e)) => {
                if in_t {
                    if let Some(ref mut run) = current_run {
                        run.text.push_str(&e.unescape().unwrap_or_default());
                    }
                }
            }
            Ok(Event::End(ref e)) => {
                match e.name().as_ref() {
                    b"a:t" => { in_t = false; }
                    b"a:r" => {
                        if let (Some(run), Some(ref mut para)) = (current_run.take(), &mut current_para) {
                            para.runs.push(run);
                        }
                    }
                    b"a:p" => {
                        if let (Some(para), Some(ref mut tb)) = (current_para.take(), &mut current_tb) {
                            tb.paragraphs.push(para);
                        }
                    }
                    b"p:sp" => {
                        if let Some(tb) = current_tb.take() {
                            if !tb.paragraphs.is_empty() {
                                slide.elements.push(SlideElement::TextBox(tb));
                            }
                        }
                        in_sp = false;
                    }
                    _ => {}
                }
            }
            Ok(Event::Eof) => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(slide)
}

/// Write a Presentation to .pptx bytes
pub fn write_pptx(pres: &Presentation) -> Result<Vec<u8>, FormatError> {
    let buf = Vec::new();
    let mut cursor = Cursor::new(buf);
    let mut zip = ZipWriter::new(&mut cursor);
    let opts = FileOptions::<'_, ()>::default().compression_method(zip::CompressionMethod::Deflated);

    zip.start_file("[Content_Types].xml", opts)?;
    zip.write_all(build_pptx_content_types(pres.slides.len()).as_bytes())?;

    zip.start_file("_rels/.rels", opts)?;
    zip.write_all(PPTX_ROOT_RELS.as_bytes())?;

    zip.start_file("ppt/_rels/presentation.xml.rels", opts)?;
    let slide_rels: String = (1..=pres.slides.len()).map(|i| {
        format!(r#"<Relationship Id="rId{}" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slide" Target="slides/slide{}.xml"/>"#, i, i)
    }).collect();
    let pres_rels = format!(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">{}</Relationships>"#, slide_rels);
    zip.write_all(pres_rels.as_bytes())?;

    // presentation.xml
    zip.start_file("ppt/presentation.xml", opts)?;
    let slide_ids: String = pres.slides.iter().enumerate().map(|(i, _)| {
        format!(r#"<p:sldId id="{}" r:id="rId{}"/>"#, i + 256, i + 1)
    }).collect();
    let pres_xml = format!(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><p:presentation xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"><p:sldMasterIdLst/><p:sldIdLst>{}</p:sldIdLst><p:sldSz cx="{}" cy="{}"/></p:presentation>"#,
        slide_ids,
        (pres.slide_width * 12700.0) as i64,
        (pres.slide_height * 12700.0) as i64);
    zip.write_all(pres_xml.as_bytes())?;

    // Individual slides
    for (idx, slide) in pres.slides.iter().enumerate() {
        zip.start_file(&format!("ppt/slides/slide{}.xml", idx + 1), opts)?;
        let slide_xml = write_slide_xml(slide, pres.slide_width, pres.slide_height)?;
        zip.write_all(slide_xml.as_bytes())?;

        zip.start_file(&format!("ppt/slides/_rels/slide{}.xml.rels", idx + 1), opts)?;
        zip.write_all(EMPTY_RELS.as_bytes())?;
    }

    zip.finish()?;
    Ok(cursor.into_inner())
}

fn write_slide_xml(slide: &Slide, w: f32, h: f32) -> Result<String, FormatError> {
    let mut xml = String::from(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><p:sld xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"><p:cSld><p:spTree><p:nvGrpSpPr><p:cNvPr id="1" name=""/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr><p:grpSpPr/>"#);

    for (idx, el) in slide.elements.iter().enumerate() {
        if let SlideElement::TextBox(tb) = el {
            let t = &tb.transform;
            let emu = |pt: f32| (pt * 12700.0) as i64;
            xml.push_str(&format!(r#"<p:sp><p:nvSpPr><p:cNvPr id="{}" name="TextBox{}"/><p:cNvSpPr txBox="1"/><p:nvPr/></p:nvSpPr><p:spPr><a:xfrm><a:off x="{}" y="{}"/><a:ext cx="{}" cy="{}"/></a:xfrm></p:spPr><p:txBody><a:bodyPr/><a:lstStyle/>"#,
                idx + 2, idx,
                emu(t.x), emu(t.y), emu(t.width), emu(t.height)));

            for para in &tb.paragraphs {
                xml.push_str("<a:p><a:pPr/>");
                for run in &para.runs {
                    let font_size = run.font.as_ref().map(|f| (f.size * 100.0) as i32).unwrap_or(1400);
                    xml.push_str(&format!(r#"<a:r><a:rPr lang="en-US" sz="{}""#, font_size));
                    if run.bold { xml.push_str(" b=\"1\""); }
                    if run.italic { xml.push_str(" i=\"1\""); }
                    xml.push_str(r#"/><a:t>"#);
                    xml.push_str(&escape_xml(&run.text));
                    xml.push_str("</a:t></a:r>");
                }
                xml.push_str("</a:p>");
            }

            xml.push_str("</p:txBody></p:sp>");
        }
    }

    xml.push_str("</p:spTree></p:cSld></p:sld>");
    Ok(xml)
}

fn build_pptx_content_types(slide_count: usize) -> String {
    let overrides: String = (1..=slide_count).map(|i| {
        format!(r#"<Override PartName="/ppt/slides/slide{}.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.slide+xml"/>"#, i)
    }).collect();
    format!(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Override PartName="/ppt/presentation.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.presentation.main+xml"/>
  {}</Types>"#, overrides)
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

const PPTX_ROOT_RELS: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="ppt/presentation.xml"/>
</Relationships>"#;

const EMPTY_RELS: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"></Relationships>"#;
