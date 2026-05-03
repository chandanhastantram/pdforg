use thiserror::Error;

pub mod creator;
pub mod manipulator;
pub mod protect;
pub mod compress;
pub mod stamp;
pub mod metadata;
pub mod images;

#[derive(Debug, Error)]
pub enum PdfError {
    #[error("PDF parse error: {0}")]
    Parse(String),
    #[error("PDF encoding error: {0}")]
    Encode(String),
    #[error("Lopdf error: {0}")]
    Lopdf(#[from] lopdf::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Image error: {0}")]
    Image(#[from] image::ImageError),
}

pub use creator::*;
pub use manipulator::*;
pub use protect::*;
pub use compress::*;
pub use stamp::*;
pub use metadata::*;
pub use images::*;

/// Image-codec filter names that must never be decompressed.
const PROTECTED_FILTERS: &[&[u8]] = &[
    b"DCTDecode", b"DCT",
    b"JPXDecode",
    b"JBIG2Decode",
    b"CCITTFaxDecode", b"CCF",
];

fn is_protected_filter(name: &[u8]) -> bool {
    PROTECTED_FILTERS.iter().any(|&f| f == name)
}

/// Decompress a PDF safely by expanding Object Streams (ObjStm) and
/// FlateDecode-only content streams **without** corrupting image streams.
///
/// `lopdf::Document::decompress()` blindly decompresses ALL streams including
/// JPEG (DCTDecode), JPEG2000 (JPXDecode), JBIG2, and CCITT fax images.
/// Re-saving those raw pixel bytes with no filter produces a corrupt file.
///
/// Strategy:
///   1. Walk all streams; for any with a protected image filter, temporarily
///      rename the filter to a sentinel value that `decompress()` won't touch.
///   2. Call `decompress()`.
///   3. Restore original filter names from a saved map.
pub fn safe_decompress(doc: &mut lopdf::Document) {
    use std::collections::BTreeMap;

    // Map from ObjectId → original Filter value (so we can restore exactly).
    let mut saved: BTreeMap<lopdf::ObjectId, lopdf::Object> = BTreeMap::new();

    // 1. Protect image streams.
    let ids: Vec<lopdf::ObjectId> = doc.objects.keys().copied().collect();
    for id in &ids {
        let dominated = if let Some(lopdf::Object::Stream(ref s)) = doc.objects.get(id) {
            match s.dict.get(b"Filter") {
                Ok(lopdf::Object::Name(ref n)) => is_protected_filter(n),
                Ok(lopdf::Object::Array(ref arr)) => arr.iter().any(|f| {
                    if let lopdf::Object::Name(ref n) = f { is_protected_filter(n) } else { false }
                }),
                _ => false,
            }
        } else { false };

        if dominated {
            if let Some(lopdf::Object::Stream(ref mut s)) = doc.objects.get_mut(id) {
                let original = s.dict.get(b"Filter").unwrap().clone();
                saved.insert(*id, original);
                // Sentinel that lopdf's decompress() won't recognise → skips.
                s.dict.set("Filter", lopdf::Object::Name(b"_Protected_".to_vec()));
            }
        }
    }

    // 2. Decompress: unpacks ObjStm + FlateDecode, skips our sentinel.
    doc.decompress();

    // 3. Restore original filters.
    for (id, original_filter) in saved {
        if let Some(lopdf::Object::Stream(ref mut s)) = doc.objects.get_mut(&id) {
            s.dict.set("Filter", original_filter);
        }
    }
}

/// Ensure the Document's max_id is correct before saving.
/// `lopdf` uses `max_id` to know how large to make the cross-reference table.
/// If we manually insert objects into `doc.objects`, `max_id` doesn't update,
/// causing `save_to` to silently drop the objects!
pub fn update_max_id(doc: &mut lopdf::Document) {
    doc.max_id = doc.objects.keys().map(|k| k.0).max().unwrap_or(0);
}
