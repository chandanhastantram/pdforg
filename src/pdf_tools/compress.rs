//! PDF compression — selectively re-compresses text/content streams using
//! DEFLATE (FlateDecode) without touching image streams.
//!
//! Key design constraint:
//! * We NEVER call `doc.decompress()` on the whole document.  That would
//!   expand binary image streams (DCTDecode/JPEG, JBIG2, LZW, etc.) and
//!   re-saving would corrupt them by wrapping decompressed pixel data in a
//!   FlateDecode layer that viewers cannot decode back to a valid image.
//!
//! Instead we only touch streams that are:
//!   a) Currently uncompressed (no Filter entry), OR
//!   b) Already FlateDecode-compressed and can be recompressed tighter.
//!
//! Streams with any of the following filters are left completely untouched:
//!   DCTDecode, JPXDecode, JBIG2Decode, CCITTFaxDecode, LZWDecode, Crypt.

use lopdf::{Document, Object};
use flate2::{write::ZlibEncoder, read::ZlibDecoder, Compression};
use std::io::{Read, Write};
use super::PdfError;

#[derive(Debug, Clone, Copy)]
pub enum CompressLevel { Low, Medium, High, Extreme }

impl From<&str> for CompressLevel {
    fn from(s: &str) -> Self {
        match s {
            "low"     => CompressLevel::Low,
            "high"    => CompressLevel::High,
            "extreme" => CompressLevel::Extreme,
            _         => CompressLevel::Medium,
        }
    }
}

impl CompressLevel {
    fn flate2_level(self) -> Compression {
        match self {
            CompressLevel::Low     => Compression::fast(),
            CompressLevel::Medium  => Compression::default(),
            CompressLevel::High    => Compression::best(),
            CompressLevel::Extreme => Compression::best(),
        }
    }
}

/// Returns true if this filter is a lossless or lossy image codec that must
/// not be re-encoded with FlateDecode.
fn is_image_filter(name: &[u8]) -> bool {
    matches!(
        name,
        b"DCTDecode"     | b"DCT"  |
        b"JPXDecode"     |
        b"JBIG2Decode"   |
        b"CCITTFaxDecode"| b"CCF"  |
        b"Crypt"
    )
}

/// Returns true if this stream already has a filter that we should not touch.
fn stream_has_protected_filter(stream: &lopdf::Stream) -> bool {
    match stream.dict.get(b"Filter") {
        Ok(Object::Name(ref n))  => is_image_filter(n),
        Ok(Object::Array(ref a)) => a.iter().any(|f| {
            if let Object::Name(ref n) = f { is_image_filter(n) } else { false }
        }),
        _ => false,
    }
}

/// Returns true if this stream is currently FlateDecode-only (single filter).
fn is_plain_flate(stream: &lopdf::Stream) -> bool {
    match stream.dict.get(b"Filter") {
        Ok(Object::Name(ref n)) => n == b"FlateDecode" || n == b"Fl",
        _ => false,
    }
}

/// Compress a PDF by applying FlateDecode to uncompressed content streams.
///
/// Returns `(compressed_bytes, bytes_saved)`.
pub fn compress_pdf(input: &[u8], level: CompressLevel) -> Result<(Vec<u8>, usize), PdfError> {
    let original_size = input.len();
    let mut doc       = Document::load_mem(input)?;
    crate::pdf_tools::safe_decompress(&mut doc);

    let flate_level = level.flate2_level();
    let ids: Vec<lopdf::ObjectId> = doc.objects.keys().copied().collect();

    for id in ids {
        if let Some(Object::Stream(ref mut stream)) = doc.objects.get_mut(&id) {
            // Never touch image or other protected streams.
            if stream_has_protected_filter(stream) { continue; }

            // Skip tiny streams — overhead outweighs saving.
            if stream.content.len() < 128 { continue; }

            if is_plain_flate(stream) {
                // Already compressed with FlateDecode — decompress first, then
                // recompress with a higher-quality level.  Safe because we
                // confirmed there is no chained image codec.
                if let Ok(raw) = deflate_decompress(&stream.content) {
                    if let Ok(better) = deflate_compress(&raw, flate_level) {
                        if better.len() < stream.content.len() {
                            stream.content = better;
                            // Filter stays FlateDecode; Length recalculated on save.
                        }
                    }
                }
            } else {
                // No filter (raw text/content stream) — apply FlateDecode.
                match stream.dict.get(b"Filter") {
                    Err(_) => {
                        // No filter at all — safe to compress.
                        if let Ok(compressed) = deflate_compress(&stream.content, flate_level) {
                            if compressed.len() < stream.content.len() {
                                stream.content = compressed;
                                stream.dict.set("Filter", Object::Name(b"FlateDecode".to_vec()));
                            }
                        }
                    }
                    _ => {
                        // Has some other filter we don't recognise — skip safely.
                    }
                }
            }
        }
    }

    let mut buf = Vec::new();
    doc.save_to(&mut buf)?;
    let new_size  = buf.len();
    let saved     = original_size.saturating_sub(new_size);
    Ok((buf, saved))
}

/// Compress raw bytes using DEFLATE (zlib framing).
fn deflate_compress(input: &[u8], level: Compression) -> Result<Vec<u8>, PdfError> {
    let mut encoder = ZlibEncoder::new(Vec::new(), level);
    encoder.write_all(input).map_err(PdfError::Io)?;
    encoder.finish().map_err(PdfError::Io)
}

/// Decompress DEFLATE-compressed bytes (zlib framing).
fn deflate_decompress(input: &[u8]) -> Result<Vec<u8>, PdfError> {
    let mut decoder = ZlibDecoder::new(input);
    let mut out     = Vec::new();
    decoder.read_to_end(&mut out).map_err(PdfError::Io)?;
    Ok(out)
}

/// Recompress embedded JPEG images in a PDF at a lower quality.
/// Only touches Image XObjects with /DCTDecode filter.
pub fn downscale_images(input: &[u8], max_dpi: u32) -> Result<Vec<u8>, PdfError> {
    let quality = match max_dpi {
        0..=72   => 30u8,
        73..=150 => 60u8,
        _        => 85u8,
    };

    let mut doc = Document::load_mem(input)?;
    crate::pdf_tools::safe_decompress(&mut doc);
    let ids: Vec<lopdf::ObjectId> = doc.objects.keys().copied().collect();

    for id in ids {
        let is_jpeg_image = {
            if let Some(Object::Stream(ref s)) = doc.objects.get(&id) {
                let is_image = matches!(
                    s.dict.get(b"Subtype"),
                    Ok(Object::Name(ref n)) if n == b"Image"
                );
                let is_dct = matches!(
                    s.dict.get(b"Filter"),
                    Ok(Object::Name(ref n)) if n == b"DCTDecode" || n == b"DCT"
                );
                is_image && is_dct
            } else { false }
        };
        if !is_jpeg_image { continue; }

        if let Some(Object::Stream(ref mut stream)) = doc.objects.get_mut(&id) {
            if let Ok(img) = image::load_from_memory_with_format(
                &stream.content,
                image::ImageFormat::Jpeg,
            ) {
                let mut new_jpeg = Vec::new();
                let mut encoder  = image::codecs::jpeg::JpegEncoder::new_with_quality(
                    &mut new_jpeg, quality,
                );
                if encoder.encode_image(&img).is_ok() && new_jpeg.len() < stream.content.len() {
                    stream.content = new_jpeg;
                }
            }
        }
    }

    let mut buf = Vec::new();
    doc.save_to(&mut buf)?;
    Ok(buf)
}
