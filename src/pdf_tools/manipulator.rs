//! Real PDF manipulation — merge, split, rotate, delete, extract, insert, flatten.
//!
//! Design principles (to prevent the "blank pages" bug):
//!
//! 1. **Never decompress streams you don't need to read.**
//!    Page-reorganisation operations (merge, split, rotate, delete, extract)
//!    do not touch stream content, so they must NOT decompress.
//!
//! 2. **Use a globally-unique, monotonically-increasing ID allocator** when copying objects
//!    between documents.
//!
//! 3. **Materialise inherited page attributes BEFORE re-parenting** — otherwise a page
//!    that relied on its parent Pages node for Resources/MediaBox loses those after the
//!    node is replaced.
//!
//! 4. **Deep-clone every indirect object a page transitively references** — fonts, images,
//!    colour spaces, XObjects, etc.
//!
//! 5. **Disable allows_compression on copied streams** to prevent double-compression.

use lopdf::{Document, Object, ObjectId, Dictionary, Stream};
use std::collections::{BTreeMap, BTreeSet};
use super::PdfError;

// ─── Global ID allocator ────────────────────────────────────────────────────
//
// Each call to `next_id()` returns a unique (generation-0) object ID.
// Starts at 1; 0 is reserved for the null object.

struct IdAlloc(u32);

impl IdAlloc {
    fn new(start: u32) -> Self { IdAlloc(start) }
    fn next(&mut self) -> ObjectId {
        let id = (self.0, 0);
        self.0 += 1;
        id
    }
}

// ─── Merge ──────────────────────────────────────────────────────────────────

/// Merge multiple PDFs into a single document in the given order.
///
/// Implementation notes:
/// * We do NOT decompress — stream bytes are copied as-is.
/// * Each source PDF gets its own ID namespace via a per-document id_map.
/// * Inherited page attributes (MediaBox, Resources, …) are materialised
///   directly onto each page dict before the page is re-parented.
pub fn merge_pdfs(inputs: &[&[u8]]) -> Result<Vec<u8>, PdfError> {
    let mut result   = Document::with_version("1.7");
    let catalog_id   = result.new_object_id();
    let pages_id     = result.new_object_id();

    let mut alloc = IdAlloc::new(1000);
    let mut all_page_ids: Vec<ObjectId> = vec![];

    for pdf_bytes in inputs {
        let doc = Document::load_mem(pdf_bytes)?;
        // No decompress — we copy compressed streams verbatim.

        // Build a per-document ID remap: old_id → new_id (from our alloc).
        let mut id_map: BTreeMap<ObjectId, ObjectId> = BTreeMap::new();
        for &old_id in doc.objects.keys() {
            id_map.insert(old_id, alloc.next());
        }

        // Copy every object, remapping all internal references.
        for (&old_id, obj) in &doc.objects {
            let new_id  = id_map[&old_id];
            let new_obj = remap_obj(obj, &id_map);
            result.objects.insert(new_id, new_obj);
        }

        // Collect page IDs in page-number order.
        let page_map = doc.get_pages(); // BTreeMap<u32 page_num, ObjectId>
        let mut sorted_pages: Vec<(u32, ObjectId)> = page_map.into_iter().collect();
        sorted_pages.sort_by_key(|&(n, _)| n);

        for (_, old_page_id) in sorted_pages {
            let new_page_id = id_map[&old_page_id];

            // Materialise inherited attributes on the *copied* page dict.
            materialise_for_page(&doc, old_page_id, &mut result, new_page_id, &id_map);

            // Re-parent to our new Pages node.
            if let Some(Object::Dictionary(ref mut d)) = result.objects.get_mut(&new_page_id) {
                d.set("Parent", Object::Reference(pages_id));
            }
            all_page_ids.push(new_page_id);
        }
    }

    // Build the Pages node.
    let pages_dict = Dictionary::from_iter(vec![
        ("Type",  Object::Name(b"Pages".to_vec())),
        ("Kids",  Object::Array(all_page_ids.iter().map(|&id| Object::Reference(id)).collect())),
        ("Count", Object::Integer(all_page_ids.len() as i64)),
    ]);
    result.objects.insert(pages_id, Object::Dictionary(pages_dict));

    // Build the Catalog.
    let catalog = Dictionary::from_iter(vec![
        ("Type",  Object::Name(b"Catalog".to_vec())),
        ("Pages", Object::Reference(pages_id)),
    ]);
    result.objects.insert(catalog_id, Object::Dictionary(catalog));
    result.trailer.set("Root", Object::Reference(catalog_id));

    crate::pdf_tools::update_max_id(&mut result);
    let mut buf = Vec::new();
    result.save_to(&mut buf)?;
    Ok(buf)
}

// ─── Page range parsing ─────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct PageRange { pub start: usize, pub end: usize }

impl PageRange {
    pub fn single(page: usize) -> Self { PageRange { start: page, end: page } }
    pub fn range(start: usize, end: usize) -> Self { PageRange { start, end } }

    /// Parse "1-3, 5, 7-9" into a sorted, deduplicated list of 1-indexed page numbers.
    pub fn parse(spec: &str, total: usize) -> Vec<usize> {
        let mut pages = BTreeSet::new();
        for part in spec.split(',') {
            let part = part.trim();
            if let Some((a, b)) = part.split_once('-') {
                let a = a.trim().parse::<usize>().unwrap_or(1).max(1);
                let b = b.trim().parse::<usize>().unwrap_or(total).min(total);
                for p in a..=b { pages.insert(p); }
            } else if let Ok(n) = part.parse::<usize>() {
                if n >= 1 && n <= total { pages.insert(n); }
            }
        }
        pages.into_iter().collect()
    }
}

// ─── Split ──────────────────────────────────────────────────────────────────

pub fn split_pdf(input: &[u8], ranges: &[PageRange]) -> Result<Vec<Vec<u8>>, PdfError> {
    let src = Document::load_mem(input)?;
    let total = src.get_pages().len();
    let mut results = vec![];
    for range in ranges {
        let start = range.start.max(1);
        let end   = range.end.min(total);
        let pages: Vec<usize> = (start..=end).collect();
        results.push(extract_pages_from_doc(&src, &pages)?);
    }
    Ok(results)
}

// ─── Extract / Delete pages ─────────────────────────────────────────────────

pub fn extract_pages(input: &[u8], pages: &[usize]) -> Result<Vec<u8>, PdfError> {
    let src = Document::load_mem(input)?;
    extract_pages_from_doc(&src, pages)
}

pub fn delete_pages(input: &[u8], pages_to_delete: &[usize]) -> Result<Vec<u8>, PdfError> {
    let src = Document::load_mem(input)?;
    let total = src.get_pages().len();
    let del_set: BTreeSet<usize> = pages_to_delete.iter().copied().collect();
    let keep: Vec<usize> = (1..=total).filter(|p| !del_set.contains(p)).collect();
    extract_pages_from_doc(&src, &keep)
}

/// Core page-extraction engine.
///
/// For each requested page we:
///   1. Walk all indirect references reachable from the page dict (fonts, images, etc.).
///   2. Deep-copy those objects into the result document with fresh IDs.
///   3. Materialise any inherited attributes (MediaBox, Resources, …) directly.
///   4. Wire the page into a new Pages/Catalog structure.
///
/// No decompress — streams are copied byte-for-byte.
fn extract_pages_from_doc(src: &Document, page_nums: &[usize]) -> Result<Vec<u8>, PdfError> {
    let mut result   = Document::with_version("1.7");
    let catalog_id   = result.new_object_id();
    let pages_id     = result.new_object_id();
    let mut alloc    = IdAlloc::new(1000);
    let mut page_ids_out: Vec<ObjectId> = vec![];

    let all_pages = src.get_pages(); // BTreeMap<page_num u32, ObjectId>
    let total     = all_pages.len();

    for &pn in page_nums {
        if pn < 1 || pn > total { continue; }
        let src_page_id = match all_pages.get(&(pn as u32)) {
            Some(&id) => id,
            None      => continue,
        };

        // Collect every object this page transitively references.
        let deps = collect_transitive_deps(src, src_page_id);

        // Assign fresh IDs for each dependency.
        let mut id_map: BTreeMap<ObjectId, ObjectId> = BTreeMap::new();
        for &dep_id in &deps {
            id_map.insert(dep_id, alloc.next());
        }
        // The page itself might not be in deps (collect_transitive_deps excludes root).
        let new_page_id = *id_map.entry(src_page_id).or_insert_with(|| alloc.next());

        // Copy all objects into result.
        for &old_id in &deps {
            if let Some(obj) = src.objects.get(&old_id) {
                let new_id  = id_map[&old_id];
                let new_obj = remap_obj(obj, &id_map);
                result.objects.insert(new_id, new_obj);
            }
        }
        // Also copy the page object itself if not already done.
        if !result.objects.contains_key(&new_page_id) {
            if let Some(obj) = src.objects.get(&src_page_id) {
                let new_obj = remap_obj(obj, &id_map);
                result.objects.insert(new_page_id, new_obj);
            }
        }

        // Materialise inherited attributes before re-parenting.
        materialise_for_page(src, src_page_id, &mut result, new_page_id, &id_map);

        // Re-parent.
        if let Some(Object::Dictionary(ref mut d)) = result.objects.get_mut(&new_page_id) {
            d.set("Parent", Object::Reference(pages_id));
        }
        page_ids_out.push(new_page_id);
    }

    let pages_dict = Dictionary::from_iter(vec![
        ("Type",  Object::Name(b"Pages".to_vec())),
        ("Kids",  Object::Array(page_ids_out.iter().map(|&id| Object::Reference(id)).collect())),
        ("Count", Object::Integer(page_ids_out.len() as i64)),
    ]);
    result.objects.insert(pages_id, Object::Dictionary(pages_dict));

    let catalog = Dictionary::from_iter(vec![
        ("Type",  Object::Name(b"Catalog".to_vec())),
        ("Pages", Object::Reference(pages_id)),
    ]);
    result.objects.insert(catalog_id, Object::Dictionary(catalog));
    result.trailer.set("Root", Object::Reference(catalog_id));

    crate::pdf_tools::update_max_id(&mut result);
    let mut buf = Vec::new();
    result.save_to(&mut buf)?;
    Ok(buf)
}

// ─── Rotate pages ───────────────────────────────────────────────────────────

/// Rotate the specified pages (or all pages if `page_nums` is empty).
/// degrees must be 0, 90, 180, or 270.  We load without decompress and save as-is.
pub fn rotate_pages(input: &[u8], page_nums: &[usize], degrees: i64) -> Result<Vec<u8>, PdfError> {
    let degrees      = ((degrees % 360) + 360) % 360;
    let mut doc      = Document::load_mem(input)?;
    // No decompress needed — we're only modifying dict entries.
    let page_ids     = doc.get_pages();

    let target_ids: Vec<ObjectId> = if page_nums.is_empty() {
        page_ids.values().copied().collect()
    } else {
        page_nums.iter()
            .filter_map(|&n| page_ids.get(&(n as u32)).copied())
            .collect()
    };

    for page_id in target_ids {
        if let Some(Object::Dictionary(ref mut d)) = doc.objects.get_mut(&page_id) {
            d.set("Rotate", Object::Integer(degrees));
        }
    }

    crate::pdf_tools::update_max_id(&mut doc);
    let mut buf = Vec::new();
    doc.save_to(&mut buf)?;
    Ok(buf)
}

// ─── Insert blank page ──────────────────────────────────────────────────────

pub fn insert_blank_page(input: &[u8], after_page: usize, width: f64, height: f64) -> Result<Vec<u8>, PdfError> {
    let mut doc = Document::load_mem(input)?;
    // No decompress needed — we're only adding new objects.

    let media_box = Object::Array(vec![
        Object::Integer(0), Object::Integer(0),
        Object::Real(width as f32), Object::Real(height as f32),
    ]);
    let empty_stream = Stream::new(
        Dictionary::from_iter(vec![("Length", Object::Integer(0))]),
        vec![],
    );
    let content_id = doc.add_object(Object::Stream(empty_stream));

    // Placeholder parent; fixed below.
    let blank_dict = Dictionary::from_iter(vec![
        ("Type",     Object::Name(b"Page".to_vec())),
        ("MediaBox", media_box),
        ("Contents", Object::Reference(content_id)),
        ("Parent",   Object::Reference((0, 0))),
    ]);
    let blank_id = doc.add_object(Object::Dictionary(blank_dict));

    let pages_node_id = find_pages_node(&doc)?;

    if let Some(Object::Dictionary(ref mut p)) = doc.objects.get_mut(&blank_id) {
        p.set("Parent", Object::Reference(pages_node_id));
    }
    if let Some(Object::Dictionary(ref mut pages)) = doc.objects.get_mut(&pages_node_id) {
        if let Ok(Object::Array(ref mut kids)) = pages.get_mut(b"Kids") {
            let pos = after_page.min(kids.len());
            kids.insert(pos, Object::Reference(blank_id));
        }
        let count = pages.get(b"Count")
            .ok()
            .and_then(|o| if let Object::Integer(n) = o { Some(*n) } else { None })
            .unwrap_or(0);
        pages.set("Count", Object::Integer(count + 1));
    }

    crate::pdf_tools::update_max_id(&mut doc);
    let mut buf = Vec::new();
    doc.save_to(&mut buf)?;
    Ok(buf)
}

fn find_pages_node(doc: &Document) -> Result<ObjectId, PdfError> {
    let root_id = match doc.trailer.get(b"Root") {
        Ok(Object::Reference(id)) => *id,
        _ => return Err(PdfError::Parse("No Root in trailer".into())),
    };
    match doc.objects.get(&root_id) {
        Some(Object::Dictionary(ref cat)) => {
            if let Ok(Object::Reference(pages_id)) = cat.get(b"Pages") {
                return Ok(*pages_id);
            }
        }
        _ => {}
    }
    Err(PdfError::Parse("Cannot find Pages node".into()))
}

// ─── Flatten ────────────────────────────────────────────────────────────────

/// Flatten a PDF (remove interactive annotations and form fields).
/// Streams are NOT decompressed; we only remove annotation dicts.
pub fn flatten_pdf(input: &[u8]) -> Result<Vec<u8>, PdfError> {
    let mut doc = Document::load_mem(input)?;
    // No decompress needed — we're only removing dict entries.
    let page_ids: Vec<ObjectId> = doc.get_pages().values().copied().collect();

    for page_id in page_ids {
        let annot_ids: Vec<ObjectId> = {
            if let Some(Object::Dictionary(ref pd)) = doc.objects.get(&page_id) {
                if let Ok(Object::Array(ref annots)) = pd.get(b"Annots") {
                    annots.iter()
                        .filter_map(|o| if let Object::Reference(id) = o { Some(*id) } else { None })
                        .collect()
                } else { vec![] }
            } else { vec![] }
        };

        for annot_id in &annot_ids {
            doc.objects.remove(annot_id);
        }
        if let Some(Object::Dictionary(ref mut pd)) = doc.objects.get_mut(&page_id) {
            pd.remove(b"Annots");
            pd.remove(b"AA");
        }
    }

    // Remove AcroForm from Catalog.
    if let Ok(Object::Reference(cat_id)) = doc.trailer.get(b"Root").map(|o| o.clone()) {
        if let Some(Object::Dictionary(ref mut cat)) = doc.objects.get_mut(&cat_id) {
            cat.remove(b"AcroForm");
            cat.remove(b"Perms");
        }
    }

    crate::pdf_tools::update_max_id(&mut doc);
    let mut buf = Vec::new();
    doc.save_to(&mut buf)?;
    Ok(buf)
}

// ─── Redact regions ─────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct RedactRegion {
    pub page:   usize,
    pub x:      f64,
    pub y:      f64,
    pub width:  f64,
    pub height: f64,
}

pub fn redact_regions(input: &[u8], regions: &[RedactRegion]) -> Result<Vec<u8>, PdfError> {
    let mut doc      = Document::load_mem(input)?;
    // No decompress needed — we're only adding new overlay streams.
    let page_ids     = doc.get_pages();
    let mut by_page: BTreeMap<usize, Vec<&RedactRegion>> = BTreeMap::new();
    for r in regions { by_page.entry(r.page).or_default().push(r); }

    for (page_num, rects) in by_page {
        let page_id = match page_ids.get(&(page_num as u32)) {
            Some(&id) => id,
            None      => continue,
        };
        let mut ops = String::from("q\n0 0 0 rg\n");
        for r in &rects {
            ops.push_str(&format!("{} {} {} {} re f\n", r.x, r.y, r.width, r.height));
        }
        ops.push_str("Q\n");
        let stream = Stream::new(
            Dictionary::from_iter(vec![("Length", Object::Integer(ops.len() as i64))]),
            ops.into_bytes(),
        );
        let overlay_id = doc.add_object(Object::Stream(stream));

        if let Some(Object::Dictionary(ref mut pd)) = doc.objects.get_mut(&page_id) {
            match pd.get_mut(b"Contents") {
                Ok(Object::Reference(old_id)) => {
                    let old = *old_id;
                    pd.set("Contents", Object::Array(vec![
                        Object::Reference(overlay_id),
                        Object::Reference(old),
                    ]));
                }
                Ok(Object::Array(ref mut arr)) => {
                    arr.insert(0, Object::Reference(overlay_id));
                }
                _ => { pd.set("Contents", Object::Reference(overlay_id)); }
            }
        }
    }

    crate::pdf_tools::update_max_id(&mut doc);
    let mut buf = Vec::new();
    doc.save_to(&mut buf)?;
    Ok(buf)
}

// ─── Dependency walker ───────────────────────────────────────────────────────
//
// Starting from a page object ID, walk every indirect reference reachable
// through that object graph and collect all IDs.  This ensures we copy
// fonts, images, colour profiles, XObjects, etc. along with the page.

fn collect_transitive_deps(doc: &Document, root_id: ObjectId) -> BTreeSet<ObjectId> {
    let mut visited: BTreeSet<ObjectId> = BTreeSet::new();
    let mut queue: Vec<ObjectId>        = vec![root_id];
    while let Some(id) = queue.pop() {
        if !visited.insert(id) { continue; }
        if let Some(obj) = doc.objects.get(&id) {
            push_refs_from_object(obj, &mut queue);
        }
    }
    visited
}

fn push_refs_from_object(obj: &Object, queue: &mut Vec<ObjectId>) {
    match obj {
        Object::Reference(id)    => queue.push(*id),
        Object::Array(arr)       => arr.iter().for_each(|o| push_refs_from_object(o, queue)),
        Object::Dictionary(dict) => dict.iter().for_each(|(_, v)| push_refs_from_object(v, queue)),
        Object::Stream(s)        => s.dict.iter().for_each(|(_, v)| push_refs_from_object(v, queue)),
        _                        => {}
    }
}

// ─── Object remapping ───────────────────────────────────────────────────────
//
// Deep-clone an Object, rewriting every Reference through `id_map`.
// References not in the map are kept as-is (they point to objects that
// will already exist in the destination, e.g. page-tree nodes).
//
// CRITICAL: For streams, we set `allows_compression = false` to prevent
// `lopdf::save_to()` from double-compressing streams that are already
// compressed (e.g. FlateDecode content streams, DCTDecode images).

pub fn remap_obj(obj: &Object, id_map: &BTreeMap<ObjectId, ObjectId>) -> Object {
    match obj {
        Object::Reference(id) => {
            Object::Reference(*id_map.get(id).unwrap_or(id))
        }
        Object::Array(arr) => {
            Object::Array(arr.iter().map(|o| remap_obj(o, id_map)).collect())
        }
        Object::Dictionary(dict) => {
            Object::Dictionary(Dictionary::from_iter(
                dict.iter().map(|(k, v)| (k.as_slice(), remap_obj(v, id_map)))
            ))
        }
        Object::Stream(s) => {
            let new_dict = Dictionary::from_iter(
                s.dict.iter().map(|(k, v)| (k.as_slice(), remap_obj(v, id_map)))
            );
            Object::Stream(Stream {
                dict:               new_dict,
                content:            s.content.clone(),
                allows_compression: false,  // Prevent double-compression!
                start_position:     None,
            })
        }
        other => other.clone(),
    }
}

// ─── Inherited attribute materialisation ────────────────────────────────────
//
// PDF pages can inherit MediaBox, CropBox, Resources, and Rotate from
// ancestor Pages nodes in the page tree.  When we extract a page and
// give it a new parent, those inherited values would be lost.
//
// This function:
//   1. Walks the source document's page-tree ancestry for the given page.
//   2. Finds any inherited attributes not already present on the page.
//   3. Copies them inline onto the *destination* page dict.
//
// Important: if the inherited value was a Reference in the source doc, we
// look it up and inline its resolved value, because the referent object
// may not have been copied into the destination.
//
// Call this AFTER copying all objects into the result document (so that
// objects pointed to by References from Resources are present) but BEFORE
// setting the new Parent.

fn materialise_for_page(
    src:         &Document,
    src_page_id: ObjectId,
    dst:         &mut Document,
    dst_page_id: ObjectId,
    id_map:      &BTreeMap<ObjectId, ObjectId>,
) {
    // Walk the source page-tree ancestors collecting inherited values.
    let keys: &[&[u8]] = &[b"MediaBox", b"CropBox", b"Resources", b"Rotate"];
    let mut inherited: BTreeMap<Vec<u8>, Object> = BTreeMap::new();

    let mut cur = src_page_id;
    let mut visited_ancestors: BTreeSet<ObjectId> = BTreeSet::new();
    loop {
        if !visited_ancestors.insert(cur) { break; } // cycle guard
        let dict = match src.objects.get(&cur) {
            Some(Object::Dictionary(d)) => d,
            _                          => break,
        };
        for &key in keys {
            if inherited.contains_key(key) { continue; } // already found closer
            if let Ok(val) = dict.get(key) {
                // Resolve one level of indirection inside the source doc.
                let resolved = match val {
                    Object::Reference(ref_id) => {
                        src.objects.get(ref_id).cloned().unwrap_or_else(|| val.clone())
                    }
                    other => other.clone(),
                };
                inherited.insert(key.to_vec(), resolved);
            }
        }
        match dict.get(b"Parent") {
            Ok(Object::Reference(parent_id)) => cur = *parent_id,
            _                                => break,
        }
    }

    // Now apply any inherited keys that are absent from the dst page.
    if let Some(Object::Dictionary(ref mut dst_dict)) = dst.objects.get_mut(&dst_page_id) {
        for (key, val) in inherited {
            if !dst_dict.has(&key) {
                // If the inherited value is a dict/stream that references objects,
                // those refs were remapped into dst — remap them here too.
                let remapped = remap_obj(&val, id_map);
                dst_dict.set(key, remapped);
            }
        }
    }
}
