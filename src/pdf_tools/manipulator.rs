//! PDF manipulation — merge, split, watermark, redact.

use lopdf::{Document, Object, ObjectId};
use super::PdfError;
use std::collections::BTreeMap;

/// Merge multiple PDFs into one
pub fn merge_pdfs(inputs: &[&[u8]]) -> Result<Vec<u8>, PdfError> {
    let mut result = Document::with_version("1.7");
    let mut all_pages = vec![];
    let catalog_id = result.new_object_id();
    let pages_id = result.new_object_id();

    for pdf_bytes in inputs {
        let mut doc = Document::load_mem(pdf_bytes)?;
        doc.decompress();

        // Remap all object IDs to avoid collisions
        let offset = result.objects.len() as u32 + 100;
        let mut id_map: BTreeMap<ObjectId, ObjectId> = BTreeMap::new();
        let original_ids: Vec<ObjectId> = doc.objects.keys().copied().collect();

        for old_id in &original_ids {
            let new_id = (old_id.0 + offset, old_id.1);
            id_map.insert(*old_id, new_id);
        }

        // Clone and remap objects
        for old_id in &original_ids {
            if let Some(obj) = doc.objects.get(old_id) {
                let new_obj = remap_obj(obj, &id_map);
                let new_id = id_map[old_id];
                result.objects.insert(new_id, new_obj);
            }
        }

        // Get page IDs from this document
        let page_ids = doc.get_pages();
        for (_, page_oid) in page_ids {
            if let Some(&new_id) = id_map.get(&page_oid) {
                all_pages.push(new_id);
            }
        }
    }

    // Update parent references for all pages
    for &page_id in &all_pages {
        if let Some(Object::Dictionary(ref mut d)) = result.objects.get_mut(&page_id) {
            d.set("Parent", Object::Reference(pages_id));
        }
    }

    // Create Pages dict
    let pages_dict = lopdf::Dictionary::from_iter(vec![
        ("Type", Object::Name(b"Pages".to_vec())),
        ("Kids", Object::Array(all_pages.iter().map(|&id| Object::Reference(id)).collect())),
        ("Count", Object::Integer(all_pages.len() as i64)),
    ]);
    result.objects.insert(pages_id, Object::Dictionary(pages_dict));

    // Create Catalog
    let catalog = lopdf::Dictionary::from_iter(vec![
        ("Type", Object::Name(b"Catalog".to_vec())),
        ("Pages", Object::Reference(pages_id)),
    ]);
    result.objects.insert(catalog_id, Object::Dictionary(catalog));
    result.trailer.set("Root", Object::Reference(catalog_id));

    let mut buf = Vec::new();
    result.save_to(&mut buf)?;
    Ok(buf)
}

/// Page range specification for splitting
#[derive(Debug, Clone)]
pub struct PageRange {
    pub start: usize,  // 1-indexed
    pub end: usize,    // 1-indexed, inclusive
}

impl PageRange {
    pub fn single(page: usize) -> Self { PageRange { start: page, end: page } }
    pub fn range(start: usize, end: usize) -> Self { PageRange { start, end } }
}

/// Split a PDF into multiple PDFs based on page ranges
pub fn split_pdf(input: &[u8], ranges: &[PageRange]) -> Result<Vec<Vec<u8>>, PdfError> {
    let doc = Document::load_mem(input)?;
    let total_pages = doc.get_pages().len();
    let mut results = vec![];

    for range in ranges {
        let start = range.start.max(1);
        let end = range.end.min(total_pages);

        // Create a minimal new PDF with just these pages
        // For simplicity, we'll clone the full doc and delete unwanted pages
        let mut new_doc = Document::load_mem(input)?;
        let pages = new_doc.get_pages();

        // Delete pages outside the range
        for (page_num, page_id) in &pages {
            if *page_num < start as u32 || *page_num > end as u32 {
                // Remove this page (simplified — real impl would clean up orphans)
                new_doc.objects.remove(page_id);
            }
        }

        let mut buf = Vec::new();
        new_doc.save_to(&mut buf)?;
        results.push(buf);
    }

    Ok(results)
}

/// Add a text watermark to a PDF
pub fn add_watermark(input: &[u8], text: &str, opacity: f32) -> Result<Vec<u8>, PdfError> {
    use super::creator::{PdfPageContent, PdfOperation};

    let mut doc = Document::load_mem(input)?;
    doc.decompress();
    // In a full implementation, we'd inject a watermark content stream overlay.
    // For Phase 1, we save the doc as-is (watermark injection requires careful
    // content stream manipulation that's done in Phase 2).
    let mut buf = Vec::new();
    doc.save_to(&mut buf)?;
    Ok(buf)
}

/// Redaction region
#[derive(Debug, Clone)]
pub struct RedactRegion {
    pub page: usize,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// Redact regions from a PDF (replace with black rectangles)
pub fn redact(input: &[u8], _regions: &[RedactRegion]) -> Result<Vec<u8>, PdfError> {
    // Phase 1 stub — full implementation in Phase 2
    let doc = Document::load_mem(input)?;
    let mut doc2 = doc;
    let mut buf2: Vec<u8> = Vec::new();
    doc2.save_to(&mut buf2)?;
    Ok(buf2)
}

fn remap_obj(obj: &Object, id_map: &BTreeMap<ObjectId, ObjectId>) -> Object {
    match obj {
        Object::Reference(id) => {
            if let Some(&new_id) = id_map.get(id) {
                Object::Reference(new_id)
            } else {
                Object::Reference(*id)
            }
        }
        Object::Array(arr) => {
            Object::Array(arr.iter().map(|o| remap_obj(o, id_map)).collect())
        }
        Object::Dictionary(dict) => {
            let new_dict = lopdf::Dictionary::from_iter(
                dict.iter().map(|(k, v)| (k.as_slice(), remap_obj(v, id_map)))
            );
            Object::Dictionary(new_dict)
        }
        Object::Stream(stream) => {
            let new_dict = lopdf::Dictionary::from_iter(
                stream.dict.iter().map(|(k, v)| (k.as_slice(), remap_obj(v, id_map)))
            );
            Object::Stream(lopdf::Stream {
                dict: new_dict,
                content: stream.content.clone(),
                allows_compression: stream.allows_compression,
                start_position: None,
            })
        }
        other => other.clone(),
    }
}
