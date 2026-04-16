//! ODF (OpenDocument Format) basic reader for .odt, .ods, .odp

use crate::FormatError;
use pdf_core::document::{Document, Block, Paragraph, Run};
use std::io::{Read, Cursor, Seek};
use zip::ZipArchive;
use quick_xml::events::Event;
use quick_xml::Reader;

pub fn parse_odt(bytes: &[u8]) -> Result<Document, FormatError> {
    let cursor = Cursor::new(bytes);
    let mut archive = ZipArchive::new(cursor)?;

    let content_xml = {
        let mut entry = archive.by_name("content.xml").map_err(FormatError::Zip)?;
        let mut s = String::new();
        entry.read_to_string(&mut s)?;
        s
    };

    let body = parse_odt_content(&content_xml)?;
    Ok(Document { body, ..Default::default() })
}

fn parse_odt_content(xml: &str) -> Result<Vec<Block>, FormatError> {
    let mut reader = Reader::from_str(xml);
    reader.trim_text(true);
    let mut blocks = vec![];
    let mut current_para: Option<Paragraph> = None;
    let mut current_run: Option<Run> = None;
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                match name.as_str() {
                    "text:p" | "text:h" => {
                        current_para = Some(Paragraph::default());
                    }
                    "text:span" => {
                        if current_para.is_some() {
                            current_run = Some(Run::new(""));
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::Text(ref e)) => {
                let text = e.unescape().unwrap_or_default().to_string();
                if let Some(ref mut run) = current_run {
                    run.text.push_str(&text);
                } else if let Some(ref mut para) = current_para {
                    // Direct text in paragraph (no span)
                    para.runs.push(Run::new(text));
                }
            }
            Ok(Event::End(ref e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                match name.as_str() {
                    "text:span" => {
                        if let (Some(run), Some(ref mut para)) = (current_run.take(), &mut current_para) {
                            if !run.text.is_empty() {
                                para.runs.push(run);
                            }
                        }
                    }
                    "text:p" | "text:h" => {
                        if let Some(para) = current_para.take() {
                            blocks.push(Block::Paragraph(para));
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

    Ok(blocks)
}
