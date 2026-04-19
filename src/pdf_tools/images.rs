//! Image to PDF conversion. Converts JPG/PNG images to a PDF document.

use lopdf::{Document, Object, Dictionary, Stream};
use super::PdfError;

/// Convert one or more images into a single PDF document.
/// Each image becomes a full page.
pub fn images_to_pdf(images: &[&[u8]]) -> Result<Vec<u8>, PdfError> {
    let mut doc = Document::with_version("1.7");
    let catalog_id = doc.new_object_id();
    let pages_id = doc.new_object_id();
    let mut page_ids = vec![];

    for img_bytes in images {
        // Load the image to get dimensions
        let img = image::load_from_memory(img_bytes)
            .map_err(|e| PdfError::Image(e))?;
        let (width, height) = (img.width() as f64, img.height() as f64);

        // Standardize DPI sizing (assume 72 DPI = 1 PDF point)
        // Feel free to adjust logic if higher-res scaling is preferred
        let pdf_w = width * 72.0 / 150.0; // scale assuming 150 DPI source for reasonable sizing
        let pdf_h = height * 72.0 / 150.0;

        // Create Image XObject
        let img_dict = Dictionary::from_iter(vec![
            ("Type",             Object::Name(b"XObject".to_vec())),
            ("Subtype",          Object::Name(b"Image".to_vec())),
            ("Width",            Object::Integer(width as i64)),
            ("Height",           Object::Integer(height as i64)),
            ("ColorSpace",       Object::Name(b"DeviceRGB".to_vec())),
            ("BitsPerComponent", Object::Integer(8)),
            ("Filter",           Object::Name(b"DCTDecode".to_vec())),
        ]);

        // Re-encode to JPEG to ensure standard DCTDecode compatibility directly into stream
        let mut jpeg_bytes = Vec::new();
        let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut jpeg_bytes, 90);
        encoder.encode_image(&img).map_err(|_| PdfError::Parse("Failed to encode JPEG".into()))?;

        let img_stream = Stream::new(img_dict, jpeg_bytes);
        let img_xobj_id = doc.add_object(Object::Stream(img_stream));

        // Create Content Stream to draw the XObject covering the whole page
        let content_str = format!("q\n{} 0 0 {} 0 0 cm\n/Im1 Do\nQ\n", pdf_w, pdf_h);
        let content_dict = Dictionary::from_iter(vec![
            ("Length", Object::Integer(content_str.len() as i64)),
        ]);
        let content_stream = Stream::new(content_dict, content_str.into_bytes());
        let content_id = doc.add_object(Object::Stream(content_stream));

        // Create Page Resources
        let mut xobj_dict = Dictionary::new();
        xobj_dict.set("Im1", Object::Reference(img_xobj_id));
        let res_dict = Dictionary::from_iter(vec![
            ("XObject", Object::Dictionary(xobj_dict)),
        ]);
        let res_id = doc.add_object(Object::Dictionary(res_dict));

        // Create Page Profile
        let media_box = Object::Array(vec![
            Object::Integer(0), Object::Integer(0),
            Object::Real(pdf_w as f32), Object::Real(pdf_h as f32),
        ]);
        let page_dict = Dictionary::from_iter(vec![
            ("Type",      Object::Name(b"Page".to_vec())),
            ("Parent",    Object::Reference(pages_id)),
            ("MediaBox",  media_box),
            ("Contents",  Object::Reference(content_id)),
            ("Resources", Object::Reference(res_id)),
        ]);
        let page_id = doc.add_object(Object::Dictionary(page_dict));
        page_ids.push(page_id);
    }

    // Build Pages tree
    let pages_dict = Dictionary::from_iter(vec![
        ("Type",  Object::Name(b"Pages".to_vec())),
        ("Kids",  Object::Array(page_ids.iter().map(|&id| Object::Reference(id)).collect())),
        ("Count", Object::Integer(page_ids.len() as i64)),
    ]);
    doc.objects.insert(pages_id, Object::Dictionary(pages_dict));

    // Build Catalog
    let catalog_dict = Dictionary::from_iter(vec![
        ("Type",  Object::Name(b"Catalog".to_vec())),
        ("Pages", Object::Reference(pages_id)),
    ]);
    doc.objects.insert(catalog_id, Object::Dictionary(catalog_dict));
    doc.trailer.set("Root", Object::Reference(catalog_id));

    let mut buf = Vec::new();
    doc.save_to(&mut buf)?;
    Ok(buf)
}
