//! Real PDF stamping — page numbers, headers/footers, Bates numbers, watermarks.
//!
//! All text is injected as real PDF content stream operators so the stamps
//! are embedded permanently in the PDF byte stream.

use lopdf::{Document, Object, Dictionary, Stream};
use super::PdfError;

// ─── Low-level content stream helpers ────────────────────────────────────────

/// Build a PDF content stream that draws a single line of text at (x, y)
/// in a given font size, colour (0..1 grey scale), and optional rotation.
fn text_content_stream(
    text: &str,
    x: f64,
    y: f64,
    font_size: f64,
    grey: f64,           // 0 = black, 1 = white
    rotate_deg: f64,     // counter-clockwise rotation in degrees
) -> Vec<u8> {
    let rad = rotate_deg.to_radians();
    let cos = rad.cos();
    let sin = rad.sin();
    // PDF text matrix: [cos sin -sin cos tx ty]
    let ops = format!(
        "q\n\
         {grey:.3} g\n\
         BT\n\
         /F1 {font_size:.1} Tf\n\
         {cos:.6} {sin:.6} {neg_sin:.6} {cos:.6} {x:.3} {y:.3} Tm\n\
         ({text}) Tj\n\
         ET\n\
         Q\n",
        grey = grey,
        font_size = font_size,
        cos = cos,
        sin = sin,
        neg_sin = -sin,
        x = x,
        y = y,
        text = escape_pdf_string(text),
    );
    ops.into_bytes()
}

/// Escape special characters in a PDF literal string
fn escape_pdf_string(s: &str) -> String {
    s.chars()
        .flat_map(|c| match c {
            '(' => vec!['\\', '('],
            ')' => vec!['\\', ')'],
            '\\' => vec!['\\', '\\'],
            c => vec![c],
        })
        .collect()
}

/// Find the media box (page size) from a page dictionary
fn get_media_box(doc: &Document, page_id: lopdf::ObjectId) -> (f64, f64) {
    // defaults to A4 if not found
    let default = (595.0f64, 842.0f64);
    let obj = match doc.objects.get(&page_id) {
        Some(o) => o,
        None => return default,
    };
    if let Object::Dictionary(ref dict) = obj {
        if let Ok(Object::Array(ref arr)) = dict.get(b"MediaBox") {
            if arr.len() == 4 {
                let w = arr[2].as_float().unwrap_or(595.0) as f64;
                let h = arr[3].as_float().unwrap_or(842.0) as f64;
                return (w, h);
            }
        }
    }
    default
}

/// Inject a helper font resource (/F1 = Helvetica) into a page's Resources dict.
/// Returns true if successful.
fn ensure_font_resource(doc: &mut Document, page_id: lopdf::ObjectId) {
    let font_dict = Dictionary::from_iter(vec![
        ("Type",     Object::Name(b"Font".to_vec())),
        ("Subtype",  Object::Name(b"Type1".to_vec())),
        ("BaseFont", Object::Name(b"Helvetica".to_vec())),
        ("Encoding", Object::Name(b"WinAnsiEncoding".to_vec())),
    ]);
    let font_id = doc.add_object(Object::Dictionary(font_dict));

    if let Some(Object::Dictionary(ref mut page_dict)) = doc.objects.get_mut(&page_id) {
        // Ensure Resources dict exists
        let res_obj = page_dict.get_mut(b"Resources");
        match res_obj {
            Ok(Object::Dictionary(ref mut res)) => {
                // Ensure Fonts dict
                match res.get_mut(b"Font") {
                    Ok(Object::Dictionary(ref mut fonts)) => {
                        fonts.set("F1", Object::Reference(font_id));
                    }
                    _ => {
                        let mut fonts = Dictionary::new();
                        fonts.set("F1", Object::Reference(font_id));
                        res.set("Font", Object::Dictionary(fonts));
                    }
                }
            }
            _ => {
                let mut fonts = Dictionary::new();
                fonts.set("F1", Object::Reference(font_id));
                let mut resources = Dictionary::new();
                resources.set("Font", Object::Dictionary(fonts));
                page_dict.set("Resources", Object::Dictionary(resources));
            }
        }
    }
}

/// Prepend a content stream overlay to a page
fn prepend_content(doc: &mut Document, page_id: lopdf::ObjectId, content: Vec<u8>) {
    let overlay_stream = Stream::new(
        Dictionary::from_iter(vec![
            ("Length", Object::Integer(content.len() as i64)),
        ]),
        content,
    );
    let overlay_id = doc.add_object(Object::Stream(overlay_stream));

    if let Some(Object::Dictionary(ref mut page_dict)) = doc.objects.get_mut(&page_id) {
        match page_dict.get_mut(b"Contents") {
            Ok(Object::Reference(old_id)) => {
                let old_id = *old_id;
                page_dict.set("Contents", Object::Array(vec![
                    Object::Reference(overlay_id),
                    Object::Reference(old_id),
                ]));
            }
            Ok(Object::Array(ref mut arr)) => {
                arr.insert(0, Object::Reference(overlay_id));
            }
            _ => {
                page_dict.set("Contents", Object::Reference(overlay_id));
            }
        }
    }
}

// ─── Public API ───────────────────────────────────────────────────────────────

/// Position for stamps on the page
#[derive(Debug, Clone, Copy)]
pub enum StampPosition {
    TopLeft, TopCenter, TopRight,
    BottomLeft, BottomCenter, BottomRight,
}

impl StampPosition {
    fn coords(self, page_w: f64, page_h: f64, margin: f64) -> (f64, f64) {
        match self {
            StampPosition::TopLeft      => (margin, page_h - margin - 12.0),
            StampPosition::TopCenter    => (page_w / 2.0, page_h - margin - 12.0),
            StampPosition::TopRight     => (page_w - margin - 60.0, page_h - margin - 12.0),
            StampPosition::BottomLeft   => (margin, margin),
            StampPosition::BottomCenter => (page_w / 2.0, margin),
            StampPosition::BottomRight  => (page_w - margin - 60.0, margin),
        }
    }
}

impl From<&str> for StampPosition {
    fn from(s: &str) -> Self {
        match s {
            "top-left"      => StampPosition::TopLeft,
            "top-center"    => StampPosition::TopCenter,
            "top-right"     => StampPosition::TopRight,
            "bottom-left"   => StampPosition::BottomLeft,
            "bottom-right"  => StampPosition::BottomRight,
            _               => StampPosition::BottomCenter,
        }
    }
}

/// Page number format
#[derive(Debug, Clone, Copy)]
pub enum PageNumFormat {
    Arabic,     // 1, 2, 3
    PageOfN,    // Page 1 of N
    Dashes,     // - 1 -
    Roman,      // i, ii, iii  (lowercase)
}

impl From<&str> for PageNumFormat {
    fn from(s: &str) -> Self {
        match s {
            "roman"    => PageNumFormat::Roman,
            "page-of-n"|"Page of N" => PageNumFormat::PageOfN,
            "dashes"   => PageNumFormat::Dashes,
            _          => PageNumFormat::Arabic,
        }
    }
}

fn to_roman(n: usize) -> String {
    const VALS: &[(usize, &str)] = &[
        (1000,"m"),(900,"cm"),(500,"d"),(400,"cd"),
        (100,"c"),(90,"xc"),(50,"l"),(40,"xl"),
        (10,"x"),(9,"ix"),(5,"v"),(4,"iv"),(1,"i"),
    ];
    let mut n = n;
    let mut out = String::new();
    for &(val, sym) in VALS {
        while n >= val { out.push_str(sym); n -= val; }
    }
    out
}

/// Add page numbers to all pages
pub fn add_page_numbers(
    input: &[u8],
    position: StampPosition,
    format: PageNumFormat,
    start_num: usize,
    font_size: f64,
) -> Result<Vec<u8>, PdfError> {
    let mut doc = Document::load_mem(input)?;
    doc.decompress();
    let page_ids: Vec<lopdf::ObjectId> = doc.get_pages().values().copied().collect();
    let total = page_ids.len();

    for (i, &page_id) in page_ids.iter().enumerate() {
        let num = i + start_num;
        let (page_w, page_h) = get_media_box(&doc, page_id);
        let (x, y) = position.coords(page_w, page_h, 36.0);

        let label = match format {
            PageNumFormat::Arabic   => num.to_string(),
            PageNumFormat::PageOfN  => format!("Page {} of {}", num, total + start_num - 1),
            PageNumFormat::Dashes   => format!("- {} -", num),
            PageNumFormat::Roman    => to_roman(num),
        };

        ensure_font_resource(&mut doc, page_id);
        let content = text_content_stream(&label, x, y, font_size, 0.0, 0.0);
        prepend_content(&mut doc, page_id, content);
    }

    let mut buf = Vec::new();
    doc.save_to(&mut buf)?;
    Ok(buf)
}

/// Header/footer configuration
#[derive(Debug, Clone, Default)]
pub struct HeaderFooterConfig {
    pub header_left:   Option<String>,
    pub header_center: Option<String>,
    pub header_right:  Option<String>,
    pub footer_left:   Option<String>,
    pub footer_center: Option<String>,
    pub footer_right:  Option<String>,
    pub font_size:     f64,
    pub margin:        f64,
    /// Pages to apply to: None = all, Some(start_page) = from that page
    pub start_page:    usize,
}

/// Add headers and footers to all pages
pub fn add_header_footer(input: &[u8], config: &HeaderFooterConfig) -> Result<Vec<u8>, PdfError> {
    let mut doc = Document::load_mem(input)?;
    doc.decompress();
    let page_ids: Vec<(u32, lopdf::ObjectId)> = doc.get_pages().into_iter().collect();
    let total = page_ids.len();
    let font_size = if config.font_size > 0.0 { config.font_size } else { 10.0 };
    let margin  = if config.margin  > 0.0 { config.margin  } else { 36.0 };

    for (page_num, page_id) in &page_ids {
        if (*page_num as usize) < config.start_page { continue; }
        let (page_w, page_h) = get_media_box(&doc, *page_id);

        let substituted_text = |tmpl: &str, n: u32| -> String {
            tmpl.replace("<<pagenum>>", &n.to_string())
                .replace("<<totalpages>>", &total.to_string())
                .replace("<<date>>", &chrono::Local::now().format("%Y-%m-%d").to_string())
        };

        let zones: Vec<(&Option<String>, f64, f64)> = vec![
            (&config.header_left,   margin,                   page_h - margin - font_size),
            (&config.header_center, page_w / 2.0,             page_h - margin - font_size),
            (&config.header_right,  page_w - margin - 60.0,   page_h - margin - font_size),
            (&config.footer_left,   margin,                   margin),
            (&config.footer_center, page_w / 2.0,             margin),
            (&config.footer_right,  page_w - margin - 60.0,   margin),
        ];

        ensure_font_resource(&mut doc, *page_id);

        let mut all_content = Vec::new();
        for (text_opt, x, y) in zones {
            if let Some(ref tmpl) = text_opt {
                let text = substituted_text(tmpl, *page_num);
                if !text.is_empty() {
                    all_content.extend(text_content_stream(&text, x, y, font_size, 0.0, 0.0));
                }
            }
        }
        if !all_content.is_empty() {
            prepend_content(&mut doc, *page_id, all_content);
        }
    }

    let mut buf = Vec::new();
    doc.save_to(&mut buf)?;
    Ok(buf)
}

/// Bates numbering configuration
#[derive(Debug, Clone)]
pub struct BatesConfig {
    pub prefix:   String,
    pub suffix:   String,
    pub start:    usize,
    pub digits:   usize,
    pub position: StampPosition,
    pub font_size: f64,
}

/// Add Bates numbers to every page
pub fn add_bates_numbers(input: &[u8], config: &BatesConfig) -> Result<Vec<u8>, PdfError> {
    let mut doc = Document::load_mem(input)?;
    doc.decompress();
    let page_ids: Vec<lopdf::ObjectId> = doc.get_pages().values().copied().collect();

    for (i, &page_id) in page_ids.iter().enumerate() {
        let num    = config.start + i;
        let padded = format!("{:0>width$}", num, width = config.digits);
        let label  = format!("{}{}{}", config.prefix, padded, config.suffix);

        let (page_w, page_h) = get_media_box(&doc, page_id);
        let (x, y) = config.position.coords(page_w, page_h, 36.0);
        let fs = if config.font_size > 0.0 { config.font_size } else { 9.0 };

        ensure_font_resource(&mut doc, page_id);
        let content = text_content_stream(&label, x, y, fs, 0.0, 0.0);
        prepend_content(&mut doc, page_id, content);
    }

    let mut buf = Vec::new();
    doc.save_to(&mut buf)?;
    Ok(buf)
}

/// Watermark configuration
#[derive(Debug, Clone)]
pub struct WatermarkConfig {
    pub text:       String,
    pub opacity:    f64,   // 0.0 – 1.0
    pub font_size:  f64,
    pub position:   String,  // "center", "top-left", etc.
}

/// Inject a real text watermark into every page.
/// For diagonal "center" placement the text is rotated 45°.
pub fn add_watermark(input: &[u8], config: &WatermarkConfig) -> Result<Vec<u8>, PdfError> {
    let mut doc = Document::load_mem(input)?;
    doc.decompress();
    let page_ids: Vec<lopdf::ObjectId> = doc.get_pages().values().copied().collect();
    let grey = 1.0 - config.opacity.clamp(0.0, 1.0) * 0.7; // lighter = more transparent

    for &page_id in &page_ids {
        let (page_w, page_h) = get_media_box(&doc, page_id);
        let fs = if config.font_size > 0.0 { config.font_size } else { 72.0 };

        let (x, y, rot) = if config.position == "center" || config.position.is_empty() {
            // Centre diagonal
            (page_w * 0.2, page_h * 0.35, 45.0)
        } else {
            let pos = StampPosition::from(config.position.as_str());
            let (px, py) = pos.coords(page_w, page_h, 36.0);
            (px, py, 0.0)
        };

        ensure_font_resource(&mut doc, page_id);
        let content = text_content_stream(&config.text, x, y, fs, grey, rot);
        prepend_content(&mut doc, page_id, content);
    }

    let mut buf = Vec::new();
    doc.save_to(&mut buf)?;
    Ok(buf)
}
