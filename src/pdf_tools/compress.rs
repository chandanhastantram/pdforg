//! Real PDF compression — recompresses streams using flate2 deflate.

use lopdf::{Document, Object};
use flate2::{write::ZlibEncoder, read::ZlibDecoder, Compression};
use std::io::{Read, Write};
use super::PdfError;

/// Compression quality levels
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
    fn image_quality(self) -> u8 {
        match self {
            CompressLevel::Low     => 90,
            CompressLevel::Medium  => 75,
            CompressLevel::High    => 50,
            CompressLevel::Extreme => 25,
        }
    }
}

/// Compress all uncompressed or inflate-compressed streams in a PDF.
/// Returns the recompressed PDF and the size reduction in bytes.
pub fn compress_pdf(input: &[u8], level: CompressLevel) -> Result<(Vec<u8>, usize), PdfError> {
    let original_size = input.len();
    let mut doc = Document::load_mem(input)?;
    doc.decompress(); // expand all contents first

    let flate_level = level.flate2_level();
    let ids: Vec<lopdf::ObjectId> = doc.objects.keys().copied().collect();

    for id in ids {
        if let Some(Object::Stream(ref mut stream)) = doc.objects.get_mut(&id) {
            // Skip already tiny streams (< 64 bytes) — not worth compressing
            if stream.content.len() < 64 { continue; }

            // Re-encode content with flate2
            let compressed = deflate_compress(&stream.content, flate_level)?;

            // Only use if it actually shrank
            if compressed.len() < stream.content.len() {
                stream.content = compressed;
                // Set the compression filter
                stream.dict.set("Filter", Object::Name(b"FlateDecode".to_vec()));
                // lopdf will recalculate Length on save automatically
            }
        }
    }

    let mut buf = Vec::new();
    doc.save_to(&mut buf)?;
    let new_size = buf.len();
    let saved = original_size.saturating_sub(new_size);
    Ok((buf, saved))
}

/// Compress raw bytes using DEFLATE (zlib framing)
fn deflate_compress(input: &[u8], level: Compression) -> Result<Vec<u8>, PdfError> {
    let mut encoder = ZlibEncoder::new(Vec::new(), level);
    encoder.write_all(input).map_err(|e| PdfError::Io(e))?;
    encoder.finish().map_err(|e| PdfError::Io(e))
}

/// Decompress DEFLATE-compressed bytes
#[allow(dead_code)]
fn deflate_decompress(input: &[u8]) -> Result<Vec<u8>, PdfError> {
    let mut decoder = ZlibDecoder::new(input);
    let mut out = Vec::new();
    decoder.read_to_end(&mut out).map_err(|e| PdfError::Io(e))?;
    Ok(out)
}

/// Recompress embedded JPEG images in a PDF at a lower quality.
/// Finds Image XObjects with /DCTDecode filter and re-encodes them.
pub fn downscale_images(input: &[u8], max_dpi: u32) -> Result<Vec<u8>, PdfError> {
    let quality = match max_dpi {
        0..=72   => 30u8,
        73..=150 => 60u8,
        _        => 85u8,
    };

    let mut doc = Document::load_mem(input)?;
    let ids: Vec<lopdf::ObjectId> = doc.objects.keys().copied().collect();

    for id in ids {
        let is_image = {
            if let Some(Object::Stream(ref s)) = doc.objects.get(&id) {
                matches!(
                    s.dict.get(b"Subtype"),
                    Ok(Object::Name(ref n)) if n == b"Image"
                )
            } else { false }
        };

        if !is_image { continue; }

        if let Some(Object::Stream(ref mut stream)) = doc.objects.get_mut(&id) {
            let filter = stream.dict.get(b"Filter")
                .ok()
                .and_then(|f| if let Object::Name(n) = f { Some(n.clone()) } else { None });

            if filter.as_deref() == Some(b"DCTDecode") {
                // Re-encode JPEG at lower quality
                if let Ok(img) = image::load_from_memory_with_format(
                    &stream.content,
                    image::ImageFormat::Jpeg,
                ) {
                    let mut new_jpeg = Vec::new();
                    let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(
                        &mut new_jpeg, quality
                    );
                    if encoder.encode_image(&img).is_ok() && new_jpeg.len() < stream.content.len() {
                        stream.content = new_jpeg;
                        // Update dimensions if changed
                    }
                }
            }
        }
    }

    let mut buf = Vec::new();
    doc.save_to(&mut buf)?;
    Ok(buf)
}
