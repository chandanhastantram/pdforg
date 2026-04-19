//! PDF metadata read/write and document sanitization.

use lopdf::{Document, Object, Dictionary};
use super::PdfError;

/// Document metadata
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct PdfMetadata {
    pub title:    Option<String>,
    pub author:   Option<String>,
    pub subject:  Option<String>,
    pub keywords: Option<String>,
    pub creator:  Option<String>,
    pub producer: Option<String>,
    pub created:  Option<String>,
    pub modified: Option<String>,
}

fn obj_to_str(obj: &Object) -> Option<String> {
    match obj {
        Object::String(bytes, _) => String::from_utf8(bytes.clone()).ok(),
        _ => None,
    }
}

/// Extract document info metadata from a PDF
pub fn get_metadata(input: &[u8]) -> Result<PdfMetadata, PdfError> {
    let doc = Document::load_mem(input)?;
    let mut meta = PdfMetadata::default();

    if let Ok(Object::Reference(info_id)) = doc.trailer.get(b"Info") {
        if let Some(Object::Dictionary(ref dict)) = doc.objects.get(info_id) {
            macro_rules! field {
                ($key:expr, $field:expr) => {
                    if let Ok(v) = dict.get($key.as_bytes()) { $field = obj_to_str(v); }
                };
            }
            field!("Title",    meta.title);
            field!("Author",   meta.author);
            field!("Subject",  meta.subject);
            field!("Keywords", meta.keywords);
            field!("Creator",  meta.creator);
            field!("Producer", meta.producer);
            field!("CreationDate", meta.created);
            field!("ModDate",  meta.modified);
        }
    }
    Ok(meta)
}

fn str_obj(s: &str) -> Object {
    Object::String(s.as_bytes().to_vec(), lopdf::StringFormat::Literal)
}

/// Write document info metadata to a PDF
pub fn set_metadata(input: &[u8], meta: &PdfMetadata) -> Result<Vec<u8>, PdfError> {
    let mut doc = Document::load_mem(input)?;

    // Find or create /Info dictionary
    let now = chrono::Local::now().format("D:%Y%m%d%H%M%S").to_string();

    let mut info = Dictionary::new();
    if let Some(ref v) = meta.title    { info.set("Title",    str_obj(v)); }
    if let Some(ref v) = meta.author   { info.set("Author",   str_obj(v)); }
    if let Some(ref v) = meta.subject  { info.set("Subject",  str_obj(v)); }
    if let Some(ref v) = meta.keywords { info.set("Keywords", str_obj(v)); }
    info.set("Creator",  str_obj(meta.creator.as_deref().unwrap_or("PDF Office")));
    info.set("Producer", str_obj("PDF Office — https://github.com/chandanhastantram/pdforg"));
    info.set("ModDate",  str_obj(&now));
    if meta.created.is_none() {
        info.set("CreationDate", str_obj(&now));
    }

    let info_id = if let Ok(Object::Reference(id)) = doc.trailer.get(b"Info") {
        // Update existing
        let id = *id;
        doc.objects.insert(id, Object::Dictionary(info));
        id
    } else {
        doc.add_object(Object::Dictionary(info))
    };

    doc.trailer.set("Info", Object::Reference(info_id));

    let mut buf = Vec::new();
    doc.save_to(&mut buf)?;
    Ok(buf)
}

/// Sanitization options — what to strip from the PDF
#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct SanitizeOptions {
    pub strip_metadata:      bool,
    pub strip_attachments:   bool,
    pub strip_javascript:    bool,
    pub strip_hidden_layers: bool,
    pub strip_search_index:  bool,
}

impl SanitizeOptions {
    pub fn all() -> Self {
        SanitizeOptions {
            strip_metadata: true,
            strip_attachments: true,
            strip_javascript: true,
            strip_hidden_layers: true,
            strip_search_index: true,
        }
    }
}

/// Strip hidden data from a PDF
pub fn sanitize_pdf(input: &[u8], opts: &SanitizeOptions) -> Result<Vec<u8>, PdfError> {
    let mut doc = Document::load_mem(input)?;

    if opts.strip_metadata {
        // Remove /Info dictionary
        if let Ok(Object::Reference(info_id)) = doc.trailer.get(b"Info") {
            let id = *info_id;
            doc.objects.remove(&id);
        }
        doc.trailer.remove(b"Info");
        // Remove XMP metadata stream
        remove_key_from_catalog(&mut doc, "Metadata");
    }

    if opts.strip_javascript {
        // Remove /JS and /JavaScript entries from catalog
        remove_key_from_catalog(&mut doc, "JavaScript");
        remove_key_from_catalog(&mut doc, "Names");
        // Remove AA (additional actions) from all objects
        let ids: Vec<lopdf::ObjectId> = doc.objects.keys().copied().collect();
        for id in ids {
            if let Some(Object::Dictionary(ref mut dict)) = doc.objects.get_mut(&id) {
                dict.remove(b"AA");
                dict.remove(b"JS");
            }
        }
    }

    if opts.strip_attachments {
        // Remove /EmbeddedFiles from Names tree
        remove_key_from_catalog(&mut doc, "EmbeddedFiles");
    }

    if opts.strip_search_index {
        let ids: Vec<lopdf::ObjectId> = doc.objects.keys().copied().collect();
        for id in ids {
            if let Some(Object::Dictionary(ref mut dict)) = doc.objects.get_mut(&id) {
                if let Ok(Object::Name(ref n)) = dict.get(b"Type") {
                    if n == b"EmbeddedFile" {
                        doc.objects.remove(&id);
                    }
                }
            }
        }
    }

    let mut buf = Vec::new();
    doc.save_to(&mut buf)?;
    Ok(buf)
}

fn remove_key_from_catalog(doc: &mut Document, key: &str) {
    let catalog_id = if let Ok(Object::Reference(id)) = doc.trailer.get(b"Root") {
        *id
    } else { return; };
    if let Some(Object::Dictionary(ref mut dict)) = doc.objects.get_mut(&catalog_id) {
        dict.remove(key.as_bytes());
    }
}
