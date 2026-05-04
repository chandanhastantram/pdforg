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

/// Safely expand Object Streams (ObjStm) without touching content or image
/// streams.
///
/// **Why not `doc.decompress()`?**
///
/// `lopdf::Document::decompress()` decompresses ALL streams including page
/// content streams.  After decompression the raw bytes are stored in the
/// `Stream::content` field, but the `/Filter` entry is removed.  When
/// `save_to()` serialises the document it writes those raw bytes verbatim
/// **without** re-compressing them.  Because the `/Length` in the stream
/// dictionary is recalculated from the (now-larger) raw content, the output
/// is technically valid — but many operations that previously relied on
/// compressed stream sizes now produce vastly larger files, and any operation
/// that later re-reads the Length (e.g. encryption) can corrupt the stream.
///
/// Worse: for image streams encoded with DCTDecode, JPXDecode, etc.,
/// `decompress()` strips the filter and replaces the JPEG/JPEG2000 byte
/// stream with raw pixel data.  Re-saving then wraps those raw pixels in a
/// stream that no viewer can decode → blank or corrupt images.
///
/// Our strategy:
///   - Walk all objects; find ObjStm (object stream) entries.
///   - Ask lopdf to unpack each ObjStm into its constituent objects.
///   - Leave every other stream (content, image, font, ICC, …) untouched.
///
/// This preserves the compressed state of all streams while still allowing
/// us to access objects that were packed inside ObjStm containers.
pub fn safe_decompress(doc: &mut lopdf::Document) {
    // Collect IDs of Object Streams (Type = ObjStm).
    let objstm_ids: Vec<lopdf::ObjectId> = doc.objects.iter()
        .filter_map(|(&id, obj)| {
            if let lopdf::Object::Stream(ref s) = obj {
                if let Ok(lopdf::Object::Name(ref n)) = s.dict.get(b"Type") {
                    if n == b"ObjStm" {
                        return Some(id);
                    }
                }
            }
            None
        })
        .collect();

    if objstm_ids.is_empty() {
        return; // Nothing to do — no packed object streams.
    }

    // For each ObjStm, decompress its content, parse out the objects inside,
    // and insert them directly into the document's object table.
    for stm_id in &objstm_ids {
        // We need to decompress the ObjStm stream itself to read its content.
        let unpacked = {
            let obj = match doc.objects.get(stm_id) {
                Some(o) => o.clone(),
                None => continue,
            };
            if let lopdf::Object::Stream(mut s) = obj {
                // Decompress only this one stream.
                s.decompress();
                let parsed = parse_objstm_content(&s);
                if !parsed.is_empty() {
                    parsed
                } else {
                    vec![]
                }
            } else {
                vec![]
            }
        };

        // Insert the unpacked objects into the document.
        for (obj_id, obj) in unpacked {
            // Only insert if the object doesn't already exist (avoid clobbering).
            doc.objects.entry(obj_id).or_insert(obj);
        }

        // Remove the ObjStm itself — its contents are now unpacked.
        doc.objects.remove(stm_id);
    }
}

/// Parse the content of an Object Stream (ObjStm) into individual objects.
///
/// ObjStm layout (after decompression):
///   - The dict has `/N` (number of objects) and `/First` (byte offset where
///     object data begins, after the header pairs).
///   - The decompressed content starts with N pairs of (object_number offset),
///     followed by the serialised objects.
///
/// We do a best-effort parse; if anything fails we return an empty vec.
fn parse_objstm_content(stream: &lopdf::Stream) -> Vec<(lopdf::ObjectId, lopdf::Object)> {
    let n = match stream.dict.get(b"N") {
        Ok(lopdf::Object::Integer(n)) => *n as usize,
        _ => return vec![],
    };
    let first = match stream.dict.get(b"First") {
        Ok(lopdf::Object::Integer(f)) => *f as usize,
        _ => return vec![],
    };

    let content = &stream.content;
    if content.len() < first || n == 0 {
        return vec![];
    }

    // Parse header: N pairs of "obj_num offset" separated by whitespace.
    let header = match std::str::from_utf8(&content[..first]) {
        Ok(s) => s,
        Err(_) => return vec![],
    };
    let tokens: Vec<&str> = header.split_whitespace().collect();
    if tokens.len() < n * 2 {
        return vec![];
    }

    let mut result = Vec::with_capacity(n);
    for i in 0..n {
        let obj_num: u32 = match tokens[i * 2].parse() {
            Ok(v) => v,
            Err(_) => continue,
        };
        let offset: usize = match tokens[i * 2 + 1].parse() {
            Ok(v) => v,
            Err(_) => continue,
        };

        let start = first + offset;
        // End is either the next object's offset or the end of content.
        let end = if i + 1 < n {
            let next_offset: usize = tokens[(i + 1) * 2 + 1].parse().unwrap_or(content.len() - first);
            first + next_offset
        } else {
            content.len()
        };

        if start >= content.len() || end > content.len() {
            continue;
        }

        // Try to parse the slice as a PDF object using lopdf's parser.
        // Fallback: store as a raw byte string if parsing fails.
        let obj_bytes = &content[start..end];
        let obj = parse_single_object(obj_bytes);
        result.push(((obj_num, 0), obj));
    }

    result
}

/// Attempt to parse a byte slice as a single PDF object.
/// Falls back to a String object if parsing fails.
fn parse_single_object(bytes: &[u8]) -> lopdf::Object {
    // lopdf doesn't expose a great single-object parser, so we do a
    // best-effort conversion.  The content will still be accessible
    // through the document's object table either way.
    let trimmed = bytes.iter()
        .copied()
        .skip_while(|b| b.is_ascii_whitespace())
        .collect::<Vec<u8>>();

    if trimmed.is_empty() {
        return lopdf::Object::Null;
    }

    // Try to wrap it as a full PDF object and parse via lopdf.
    // We construct "obj_num 0 obj\n<content>\nendobj" and parse that.
    let mut fake_pdf = b"%PDF-1.7\n1 0 obj\n".to_vec();
    fake_pdf.extend_from_slice(&trimmed);
    fake_pdf.extend_from_slice(b"\nendobj\nxref\n0 1\n0000000000 65535 f \ntrailer<</Size 2>>\nstartxref\n9\n%%EOF");

    match lopdf::Document::load_mem(&fake_pdf) {
        Ok(doc) => {
            if let Some(obj) = doc.objects.get(&(1, 0)) {
                return obj.clone();
            }
            lopdf::Object::Null
        }
        Err(_) => {
            // Last resort: return raw bytes as a string.
            lopdf::Object::String(trimmed, lopdf::StringFormat::Literal)
        }
    }
}

/// Ensure the Document's max_id and trailer Size are correct before saving.
///
/// `lopdf` uses `max_id` to know how large to make the cross-reference table.
/// If we manually insert objects into `doc.objects`, `max_id` doesn't update,
/// causing `save_to` to silently drop the objects!
///
/// We also set the trailer `/Size` to `max_id + 1` (required by the PDF spec)
/// so that viewers know the total number of entries in the xref table.
pub fn update_max_id(doc: &mut lopdf::Document) {
    let max = doc.objects.keys().map(|k| k.0).max().unwrap_or(0);
    doc.max_id = max;
    doc.trailer.set("Size", lopdf::Object::Integer(max as i64 + 1));
}
