//! REST API handlers for the PDF Office server.

use axum::{
    extract::{Path, State, Json, Multipart, Query},
    response::{IntoResponse, Response},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::core::document::{Document, Comment};
use crate::spell::{default_checker, SpellResult};
use crate::sheets::parser::parse_formula;
use crate::sheets::evaluator::{eval, EvalContext};
use crate::core::{Workbook, CellAddress, CellValue};
use super::SharedState;

type ApiResult<T> = Result<Json<T>, ApiError>;

#[derive(Debug)]
pub struct ApiError(String, StatusCode);

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.1, self.0).into_response()
    }
}

impl From<crate::storage::StorageError> for ApiError {
    fn from(e: crate::storage::StorageError) -> Self {
        ApiError(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR)
    }
}

// ─── Document endpoints ───────────────────────────────────────────────────────

pub async fn list_documents(State(state): State<SharedState>) -> ApiResult<serde_json::Value> {
    let store = state.store.lock().await;
    let docs = store.list_recent(50).map_err(ApiError::from)?;
    Ok(Json(serde_json::json!({ "documents": docs })))
}

pub async fn create_document(State(state): State<SharedState>) -> ApiResult<serde_json::Value> {
    let doc = Document::default();
    let id = doc.id;
    let title = doc.title.clone();
    let mut store = state.store.lock().await;
    store.save_document(&doc).map_err(ApiError::from)?;
    Ok(Json(serde_json::json!({ "id": id, "title": title })))
}

pub async fn get_document(
    State(state): State<SharedState>,
    Path(id): Path<Uuid>,
) -> ApiResult<Document> {
    let store = state.store.lock().await;
    let doc = store.load_document(id).map_err(ApiError::from)?;
    Ok(Json(doc))
}

pub async fn save_document(
    State(state): State<SharedState>,
    Path(id): Path<Uuid>,
    Json(mut doc): Json<Document>,
) -> ApiResult<serde_json::Value> {
    doc.id = id;
    doc.revision += 1;
    let mut store = state.store.lock().await;
    store.save_document(&doc).map_err(ApiError::from)?;
    Ok(Json(serde_json::json!({ "ok": true, "revision": doc.revision })))
}

pub async fn delete_document(
    State(state): State<SharedState>,
    Path(id): Path<Uuid>,
) -> ApiResult<serde_json::Value> {
    let mut store = state.store.lock().await;  // needs write lock — deletes rows
    store.delete_document(id).map_err(ApiError::from)?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

pub async fn export_document(
    State(state): State<SharedState>,
    Path(id): Path<Uuid>,
    Json(params): Json<ExportParams>,
) -> impl IntoResponse {
    let store = state.store.lock().await;
    let doc = match store.load_document(id) {
        Ok(d) => d,
        Err(e) => return (StatusCode::NOT_FOUND, e.to_string()).into_response(),
    };

    match params.format.as_str() {
        "docx" => {
            match crate::formats::docx::write_docx(&doc) {
                Ok(bytes) => (
                    StatusCode::OK,
                    [("Content-Type", "application/vnd.openxmlformats-officedocument.wordprocessingml.document"),
                     ("Content-Disposition", "attachment; filename=\"document.docx\"")],
                    bytes,
                ).into_response(),
                Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
            }
        }
        "pdf" => {
            match crate::pdf_tools::creator::create_pdf_from_document(&doc) {
                Ok(bytes) => (
                    StatusCode::OK,
                    [("Content-Type", "application/pdf"),
                     ("Content-Disposition", "attachment; filename=\"document.pdf\"")],
                    bytes,
                ).into_response(),
                Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
            }
        }
        "pdfo" => {
            // Build the .pdfo archive (ZIP + JSON) while the store lock is still held
            match store.export_pdfo(&doc) {
                Ok(bytes) => (
                    StatusCode::OK,
                    [("Content-Type", "application/zip"),
                     ("Content-Disposition", "attachment; filename=\"document.pdfo\"")],
                    bytes,
                ).into_response(),
                Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
            }
        }
        "xlsx" => {
            // Export workbook (creates minimal single-sheet workbook from text)
            let wb = crate::core::Workbook::default();
            match crate::formats::xlsx::write_xlsx(&wb) {
                Ok(bytes) => (
                    StatusCode::OK,
                    [("Content-Type", "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"),
                     ("Content-Disposition", "attachment; filename=\"document.xlsx\"")],
                    bytes,
                ).into_response(),
                Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
            }
        }
        _ => (StatusCode::BAD_REQUEST, "Unsupported format. Supported: docx, pdf, pdfo, xlsx").into_response(),
    }
}

#[derive(Deserialize)]
pub struct ExportParams {
    pub format: String,
}

// ─── Version endpoints ────────────────────────────────────────────────────────

pub async fn list_versions(
    State(state): State<SharedState>,
    Path(id): Path<Uuid>,
) -> ApiResult<serde_json::Value> {
    let store = state.store.lock().await;
    let versions = store.list_versions(id).map_err(ApiError::from)?;
    Ok(Json(serde_json::json!({ "versions": versions })))
}

pub async fn get_version(
    State(state): State<SharedState>,
    Path((_doc_id, version_id)): Path<(Uuid, Uuid)>,
) -> ApiResult<serde_json::Value> {
    let store = state.store.lock().await;
    let snapshot = store.load_version(version_id).map_err(ApiError::from)?;
    Ok(Json(serde_json::json!({
        "id": snapshot.id,
        "label": snapshot.version_label,
        "created_at": snapshot.created_at,
        "created_by": snapshot.created_by,
        "document": snapshot.document,
    })))
}

pub async fn restore_version(
    State(state): State<SharedState>,
    Path((doc_id, version_id)): Path<(Uuid, Uuid)>,
) -> ApiResult<serde_json::Value> {
    // Load the historical snapshot
    let snapshot = {
        let store = state.store.lock().await;
        store.load_version(version_id).map_err(ApiError::from)?
    };
    // Overwrite current document with the snapshot's content
    let mut restored_doc = snapshot.document;
    restored_doc.id = doc_id;
    restored_doc.revision += 1;
    let mut store = state.store.lock().await;
    store.save_document(&restored_doc).map_err(ApiError::from)?;
    Ok(Json(serde_json::json!({
        "ok": true,
        "restored_from_version": version_id,
        "new_revision": restored_doc.revision,
    })))
}

// ─── Comment endpoints ────────────────────────────────────────────────────────

pub async fn list_comments(
    State(state): State<SharedState>,
    Path(id): Path<Uuid>,
) -> ApiResult<Vec<Comment>> {
    let store = state.store.lock().await;
    let comments = store.list_comments(id).map_err(ApiError::from)?;
    Ok(Json(comments))
}

pub async fn add_comment(
    State(state): State<SharedState>,
    Path(id): Path<Uuid>,
    Json(comment): Json<Comment>,
) -> ApiResult<serde_json::Value> {
    let mut store = state.store.lock().await;
    store.save_comment(id, &comment).map_err(ApiError::from)?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

// ─── Preferences endpoints ────────────────────────────────────────────────────

pub async fn get_preference(
    State(state): State<SharedState>,
    Path(key): Path<String>,
) -> ApiResult<serde_json::Value> {
    let store = state.store.lock().await;
    let value: Option<serde_json::Value> = store.get_pref(&key).map_err(ApiError::from)?;
    Ok(Json(serde_json::json!({ "key": key, "value": value })))
}

#[derive(Deserialize)]
pub struct PrefBody {
    pub value: serde_json::Value,
}

pub async fn set_preference(
    State(state): State<SharedState>,
    Path(key): Path<String>,
    Json(body): Json<PrefBody>,
) -> ApiResult<serde_json::Value> {
    let mut store = state.store.lock().await;
    store.set_pref(&key, &body.value).map_err(ApiError::from)?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

// ─── Spell check ─────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct SpellCheckRequest {
    pub text: String,
    pub lang: Option<String>,
}

pub async fn spell_check(
    Json(req): Json<SpellCheckRequest>,
) -> ApiResult<serde_json::Value> {
    let checker = default_checker();
    let results = checker.check_text(&req.text);
    Ok(Json(serde_json::json!({ "results": results })))
}

// ─── Formula evaluation ───────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct FormulaRequest {
    pub formula: String,
    pub row: Option<u32>,
    pub col: Option<u32>,
}

pub async fn eval_formula(
    Json(req): Json<FormulaRequest>,
) -> ApiResult<serde_json::Value> {
    let expr = match parse_formula(&req.formula) {
        Ok(e) => e,
        Err(e) => return Ok(Json(serde_json::json!({ "error": e }))),
    };

    let wb = Workbook::default();
    let addr = CellAddress::new(req.row.unwrap_or(0), req.col.unwrap_or(0));
    let ctx = EvalContext::new(&wb, 0, addr);

    match eval(&expr, &ctx) {
        Ok(v) => Ok(Json(serde_json::json!({ "result": v.as_text() }))),
        Err(e) => Ok(Json(serde_json::json!({ "error": e.to_string() }))),
    }
}

// ─── File conversion ──────────────────────────────────────────────────────────

pub async fn convert_file(mut multipart: Multipart) -> impl IntoResponse {
    let mut file_bytes = None;
    let mut filename = String::new();
    let mut target_format = "docx".to_string();

    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "file" => {
                filename = field.file_name().unwrap_or("upload").to_string();
                file_bytes = Some(field.bytes().await.unwrap_or_default().to_vec());
            }
            "format" => {
                target_format = field.text().await.unwrap_or_default();
            }
            _ => {}
        }
    }

    let bytes = match file_bytes {
        Some(b) => b,
        None => return (StatusCode::BAD_REQUEST, "No file uploaded").into_response(),
    };

    let ext = filename.rsplit('.').next().unwrap_or("").to_lowercase();
    let doc = match ext.as_str() {
        "docx" => crate::formats::docx::parse_docx(&bytes).ok(),
        "rtf"  => crate::formats::rtf::parse_rtf(&bytes).ok(),
        "odt"  => crate::formats::odf::parse_odt(&bytes).ok(),
        _ => None,
    };

    let doc = match doc {
        Some(d) => d,
        None => return (StatusCode::BAD_REQUEST, "Unsupported input format").into_response(),
    };

    match target_format.as_str() {
        "docx" => {
            match crate::formats::docx::write_docx(&doc) {
                Ok(out) => (StatusCode::OK, [(
                    "Content-Type",
                    "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
                )], out).into_response(),
                Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
            }
        }
        "pdf" => {
            match crate::pdf_tools::creator::create_pdf_from_document(&doc) {
                Ok(out) => (StatusCode::OK, [("Content-Type", "application/pdf")], out).into_response(),
                Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
            }
        }
        _ => (StatusCode::BAD_REQUEST, "Unsupported output format. Supported: docx, pdf").into_response(),
    }
}

// ─── Direct Browser Download ──────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct BinaryQuery {
    pub platform: Option<String>,
}

pub async fn download_binary(
    Query(_query): Query<BinaryQuery>,
) -> impl IntoResponse {
    match std::env::current_exe() {
        Ok(exe_path) => {
            match std::fs::read(&exe_path) {
                Ok(bytes) => {
                    let filename = exe_path.file_name()
                        .map(|n| n.to_string_lossy().into_owned())
                        .unwrap_or_else(|| "pdf_office_binary".to_string());
                    (
                        StatusCode::OK,
                        [
                            ("Content-Type", "application/octet-stream"),
                            ("Content-Disposition", &format!("attachment; filename=\"{}\"", filename)),
                        ],
                        bytes,
                    ).into_response()
                }
                Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to read executable: {}", e)).into_response(),
            }
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Could not find current executable: {}", e)).into_response(),
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// REAL PDF PROCESSING ENDPOINTS
// Every handler: (1) reads multipart PDF upload, (2) calls real backend,
// (3) streams the processed PDF back as a download.
// ═══════════════════════════════════════════════════════════════════════════════

/// Helper — extract all multipart fields into a map
async fn collect_multipart(mut mp: Multipart) -> (Option<Vec<u8>>, std::collections::HashMap<String, String>) {
    let mut file_bytes: Option<Vec<u8>> = None;
    let mut fields = std::collections::HashMap::new();
    while let Ok(Some(field)) = mp.next_field().await {
        let name = field.name().unwrap_or("").to_string();
        if name == "file" || name == "pdf" {
            if let Ok(bytes) = field.bytes().await {
                file_bytes = Some(bytes.to_vec());
            }
        } else {
            if let Ok(text) = field.text().await {
                fields.insert(name, text);
            }
        }
    }
    (file_bytes, fields)
}

fn pdf_response(bytes: Vec<u8>, filename: &str) -> Response {
    let cd = format!("attachment; filename=\"{}\"", filename);
    (
        StatusCode::OK,
        [
            ("Content-Type".to_string(), "application/pdf".to_string()),
            ("Content-Disposition".to_string(), cd),
        ],
        bytes,
    ).into_response()
}

fn err500(msg: impl std::fmt::Display) -> Response {
    (StatusCode::INTERNAL_SERVER_ERROR, msg.to_string()).into_response()
}

fn err400(msg: impl std::fmt::Display) -> Response {
    (StatusCode::BAD_REQUEST, msg.to_string()).into_response()
}

// ─── Merge ────────────────────────────────────────────────────────────────────

pub async fn pdf_merge(mut multipart: Multipart) -> impl IntoResponse {
    let mut pdfs: Vec<Vec<u8>> = vec![];
    while let Ok(Some(field)) = multipart.next_field().await {
        if let Ok(bytes) = field.bytes().await {
            if !bytes.is_empty() { pdfs.push(bytes.to_vec()); }
        }
    }
    if pdfs.len() < 2 {
        return err400("Upload at least 2 PDF files to merge");
    }
    let refs: Vec<&[u8]> = pdfs.iter().map(|v| v.as_slice()).collect();
    match crate::pdf_tools::merge_pdfs(&refs) {
        Ok(out) => pdf_response(out, "merged.pdf"),
        Err(e)  => err500(e),
    }
}

// ─── Split ────────────────────────────────────────────────────────────────────

pub async fn pdf_split(multipart: Multipart) -> impl IntoResponse {
    let (bytes, fields) = collect_multipart(multipart).await;
    let bytes = match bytes { Some(b) => b, None => return err400("No PDF uploaded") };

    let method = fields.get("method").map(|s| s.as_str()).unwrap_or("bypage");
    let value  = fields.get("value").map(|s| s.as_str()).unwrap_or("1");

    let doc = match lopdf::Document::load_mem(&bytes) {
        Ok(d) => d, Err(e) => return err500(e),
    };
    let total = doc.get_pages().len();

    let ranges: Vec<crate::pdf_tools::PageRange> = match method {
        "bypage" => {
            let n: usize = value.parse().unwrap_or(1).clamp(1, total - 1);
            vec![
                crate::pdf_tools::PageRange::range(1, n),
                crate::pdf_tools::PageRange::range(n + 1, total),
            ]
        }
        "range" => {
            // "value" = page range spec per split part, comma-separated parts
            let pages = crate::pdf_tools::PageRange::parse(value, total);
            if pages.is_empty() {
                return err400("Invalid page range");
            }
            let &first = pages.first().unwrap();
            let &last  = pages.last().unwrap();
            vec![crate::pdf_tools::PageRange::range(first, last)]
        }
        _ => {
            vec![crate::pdf_tools::PageRange::range(1, total / 2),
                 crate::pdf_tools::PageRange::range(total / 2 + 1, total)]
        }
    };

    match crate::pdf_tools::split_pdf(&bytes, &ranges) {
        Err(e) => err500(e),
        Ok(parts) => {
            if parts.len() == 1 {
                return pdf_response(parts.into_iter().next().unwrap(), "split.pdf");
            }
            // Return as ZIP
            let mut zip_buf = Vec::new();
            {
                let mut zip = zip::ZipWriter::new(std::io::Cursor::new(&mut zip_buf));
                let opts: zip::write::FileOptions<'_, ()> = zip::write::FileOptions::default()
                    .compression_method(zip::CompressionMethod::Deflated);
                for (i, part) in parts.iter().enumerate() {
                    zip.start_file(format!("part-{}.pdf", i + 1), opts).ok();
                    use std::io::Write;
                    zip.write_all(part).ok();
                }
                zip.finish().ok();
            }
            (StatusCode::OK,
             [("Content-Type", "application/zip"),
              ("Content-Disposition", "attachment; filename=\"split.zip\"")],
             zip_buf
            ).into_response()
        }
    }
}

// ─── Compress ─────────────────────────────────────────────────────────────────

pub async fn pdf_compress(multipart: Multipart) -> impl IntoResponse {
    let (bytes, fields) = collect_multipart(multipart).await;
    let bytes = match bytes { Some(b) => b, None => return err400("No PDF uploaded") };
    let level = crate::pdf_tools::CompressLevel::from(
        fields.get("level").map(|s| s.as_str()).unwrap_or("medium")
    );
    match crate::pdf_tools::compress_pdf(&bytes, level) {
        Ok((out, saved)) => {
            tracing::info!("Compressed PDF: saved {} bytes", saved);
            pdf_response(out, "compressed.pdf")
        }
        Err(e) => err500(e),
    }
}

// ─── Protect / Encrypt ────────────────────────────────────────────────────────

pub async fn pdf_protect(multipart: Multipart) -> impl IntoResponse {
    let (bytes, fields) = collect_multipart(multipart).await;
    let bytes = match bytes { Some(b) => b, None => return err400("No PDF uploaded") };

    let user_pass  = fields.get("open_password").map(|s| s.as_str()).unwrap_or("");
    let owner_pass = fields.get("owner_password").map(|s| s.as_str()).unwrap_or("owner");

    // Build permission flags
    let mut perms = crate::pdf_tools::Permissions::all();
    if fields.get("no_print").map(|v| v == "1").unwrap_or(false)  { perms &= !crate::pdf_tools::Permissions::PRINT; }
    if fields.get("no_copy") .map(|v| v == "1").unwrap_or(false)  { perms &= !crate::pdf_tools::Permissions::COPY; }
    if fields.get("no_edit") .map(|v| v == "1").unwrap_or(false)  { perms &= !crate::pdf_tools::Permissions::MODIFY; }

    match crate::pdf_tools::encrypt_pdf(&bytes, user_pass, owner_pass, perms) {
        Ok(out) => pdf_response(out, "protected.pdf"),
        Err(e)  => err500(e),
    }
}

// ─── Watermark ────────────────────────────────────────────────────────────────

pub async fn pdf_watermark(multipart: Multipart) -> impl IntoResponse {
    let (bytes, fields) = collect_multipart(multipart).await;
    let bytes = match bytes { Some(b) => b, None => return err400("No PDF uploaded") };

    let config = crate::pdf_tools::WatermarkConfig {
        text:      fields.get("text").cloned().unwrap_or_else(|| "CONFIDENTIAL".into()),
        opacity:   fields.get("opacity").and_then(|v| v.parse().ok()).unwrap_or(0.3),
        font_size: fields.get("font_size").and_then(|v| v.parse().ok()).unwrap_or(72.0),
        position:  fields.get("position").cloned().unwrap_or_else(|| "center".into()),
    };

    match crate::pdf_tools::stamp::add_watermark(&bytes, &config) {
        Ok(out) => pdf_response(out, "watermarked.pdf"),
        Err(e)  => err500(e),
    }
}

// ─── Rotate ───────────────────────────────────────────────────────────────────

pub async fn pdf_rotate(multipart: Multipart) -> impl IntoResponse {
    let (bytes, fields) = collect_multipart(multipart).await;
    let bytes   = match bytes { Some(b) => b, None => return err400("No PDF uploaded") };
    let degrees: i64 = fields.get("degrees").and_then(|v| v.parse().ok()).unwrap_or(90);
    let pages_str = fields.get("pages").map(|s| s.as_str()).unwrap_or("all");

    let page_nums: Vec<usize> = if pages_str == "all" {
        vec![]  // empty = all pages
    } else {
        let doc = match lopdf::Document::load_mem(&bytes) {
            Ok(d) => d, Err(e) => return err500(e),
        };
        let total = doc.get_pages().len();
        crate::pdf_tools::PageRange::parse(pages_str, total)
    };

    match crate::pdf_tools::rotate_pages(&bytes, &page_nums, degrees) {
        Ok(out) => pdf_response(out, "rotated.pdf"),
        Err(e)  => err500(e),
    }
}

// ─── Delete pages ─────────────────────────────────────────────────────────────

pub async fn pdf_delete_pages(multipart: Multipart) -> impl IntoResponse {
    let (bytes, fields) = collect_multipart(multipart).await;
    let bytes = match bytes { Some(b) => b, None => return err400("No PDF uploaded") };
    let range = fields.get("pages").map(|s| s.as_str()).unwrap_or("");

    let doc = match lopdf::Document::load_mem(&bytes) {
        Ok(d) => d, Err(e) => return err500(e),
    };
    let total = doc.get_pages().len();
    let pages = crate::pdf_tools::PageRange::parse(range, total);

    match crate::pdf_tools::delete_pages(&bytes, &pages) {
        Ok(out) => pdf_response(out, "deleted.pdf"),
        Err(e)  => err500(e),
    }
}

// ─── Extract pages ────────────────────────────────────────────────────────────

pub async fn pdf_extract_pages(multipart: Multipart) -> impl IntoResponse {
    let (bytes, fields) = collect_multipart(multipart).await;
    let bytes = match bytes { Some(b) => b, None => return err400("No PDF uploaded") };
    let range = fields.get("pages").map(|s| s.as_str()).unwrap_or("1");

    let doc = match lopdf::Document::load_mem(&bytes) {
        Ok(d) => d, Err(e) => return err500(e),
    };
    let total = doc.get_pages().len();
    let pages = crate::pdf_tools::PageRange::parse(range, total);

    match crate::pdf_tools::extract_pages(&bytes, &pages) {
        Ok(out) => pdf_response(out, "extracted.pdf"),
        Err(e)  => err500(e),
    }
}

// ─── Insert blank page ────────────────────────────────────────────────────────

pub async fn pdf_insert_page(multipart: Multipart) -> impl IntoResponse {
    let (bytes, fields) = collect_multipart(multipart).await;
    let bytes      = match bytes { Some(b) => b, None => return err400("No PDF uploaded") };
    let after_page: usize = fields.get("after").and_then(|v| v.parse().ok()).unwrap_or(0);
    let width:  f64 = fields.get("width") .and_then(|v| v.parse().ok()).unwrap_or(595.0);
    let height: f64 = fields.get("height").and_then(|v| v.parse().ok()).unwrap_or(842.0);

    match crate::pdf_tools::insert_blank_page(&bytes, after_page, width, height) {
        Ok(out) => pdf_response(out, "page_inserted.pdf"),
        Err(e)  => err500(e),
    }
}

// ─── Page numbers ─────────────────────────────────────────────────────────────

pub async fn pdf_page_numbers(multipart: Multipart) -> impl IntoResponse {
    let (bytes, fields) = collect_multipart(multipart).await;
    let bytes = match bytes { Some(b) => b, None => return err400("No PDF uploaded") };

    let position = crate::pdf_tools::StampPosition::from(
        fields.get("position").map(|s| s.as_str()).unwrap_or("bottom-center")
    );
    let format = crate::pdf_tools::PageNumFormat::from(
        fields.get("format").map(|s| s.as_str()).unwrap_or("arabic")
    );
    let start: usize    = fields.get("start").and_then(|v| v.parse().ok()).unwrap_or(1);
    let font_size: f64  = fields.get("font_size").and_then(|v| v.parse().ok()).unwrap_or(10.0);

    match crate::pdf_tools::add_page_numbers(&bytes, position, format, start, font_size) {
        Ok(out) => pdf_response(out, "numbered.pdf"),
        Err(e)  => err500(e),
    }
}

// ─── Header / Footer ─────────────────────────────────────────────────────────

pub async fn pdf_header_footer(multipart: Multipart) -> impl IntoResponse {
    let (bytes, fields) = collect_multipart(multipart).await;
    let bytes = match bytes { Some(b) => b, None => return err400("No PDF uploaded") };

    let config = crate::pdf_tools::HeaderFooterConfig {
        header_left:   fields.get("header_left")  .filter(|s| !s.is_empty()).cloned(),
        header_center: fields.get("header_center").filter(|s| !s.is_empty()).cloned(),
        header_right:  fields.get("header_right") .filter(|s| !s.is_empty()).cloned(),
        footer_left:   fields.get("footer_left")  .filter(|s| !s.is_empty()).cloned(),
        footer_center: fields.get("footer_center").filter(|s| !s.is_empty()).cloned(),
        footer_right:  fields.get("footer_right") .filter(|s| !s.is_empty()).cloned(),
        font_size:     fields.get("font_size").and_then(|v| v.parse().ok()).unwrap_or(10.0),
        margin:        36.0,
        start_page:    fields.get("start_page").and_then(|v| v.parse().ok()).unwrap_or(1),
    };

    match crate::pdf_tools::add_header_footer(&bytes, &config) {
        Ok(out) => pdf_response(out, "header_footer.pdf"),
        Err(e)  => err500(e),
    }
}

// ─── Bates Numbering ─────────────────────────────────────────────────────────

pub async fn pdf_bates(multipart: Multipart) -> impl IntoResponse {
    let (bytes, fields) = collect_multipart(multipart).await;
    let bytes = match bytes { Some(b) => b, None => return err400("No PDF uploaded") };

    let config = crate::pdf_tools::BatesConfig {
        prefix:    fields.get("prefix").cloned().unwrap_or_else(|| "DOC-".into()),
        suffix:    fields.get("suffix").cloned().unwrap_or_default(),
        start:     fields.get("start") .and_then(|v| v.parse().ok()).unwrap_or(1),
        digits:    fields.get("digits").and_then(|v| v.parse().ok()).unwrap_or(6),
        position:  crate::pdf_tools::StampPosition::from(
                       fields.get("position").map(|s| s.as_str()).unwrap_or("bottom-right")),
        font_size: fields.get("font_size").and_then(|v| v.parse().ok()).unwrap_or(9.0),
    };

    match crate::pdf_tools::add_bates_numbers(&bytes, &config) {
        Ok(out) => pdf_response(out, "bates.pdf"),
        Err(e)  => err500(e),
    }
}

// ─── Metadata editor ─────────────────────────────────────────────────────────

pub async fn pdf_get_metadata(mut multipart: Multipart) -> impl IntoResponse {
    let mut bytes = vec![];
    while let Ok(Some(field)) = multipart.next_field().await {
        if let Ok(b) = field.bytes().await { bytes = b.to_vec(); break; }
    }
    if bytes.is_empty() { return (StatusCode::BAD_REQUEST, "No PDF").into_response(); }
    match crate::pdf_tools::get_metadata(&bytes) {
        Ok(meta) => Json(meta).into_response(),
        Err(e)   => err500(e),
    }
}

pub async fn pdf_set_metadata(multipart: Multipart) -> impl IntoResponse {
    let (bytes, fields) = collect_multipart(multipart).await;
    let bytes = match bytes { Some(b) => b, None => return err400("No PDF uploaded") };

    let meta = crate::pdf_tools::PdfMetadata {
        title:    fields.get("title")   .filter(|s| !s.is_empty()).cloned(),
        author:   fields.get("author")  .filter(|s| !s.is_empty()).cloned(),
        subject:  fields.get("subject") .filter(|s| !s.is_empty()).cloned(),
        keywords: fields.get("keywords").filter(|s| !s.is_empty()).cloned(),
        creator:  Some("PDF Office".into()),
        ..Default::default()
    };

    match crate::pdf_tools::set_metadata(&bytes, &meta) {
        Ok(out) => pdf_response(out, "metadata.pdf"),
        Err(e)  => err500(e),
    }
}

// ─── Sanitize ────────────────────────────────────────────────────────────────

pub async fn pdf_sanitize(multipart: Multipart) -> impl IntoResponse {
    let (bytes, fields) = collect_multipart(multipart).await;
    let bytes = match bytes { Some(b) => b, None => return err400("No PDF uploaded") };

    let opts = crate::pdf_tools::SanitizeOptions {
        strip_metadata:      fields.get("metadata")    .map(|v| v != "0").unwrap_or(true),
        strip_attachments:   fields.get("attachments") .map(|v| v != "0").unwrap_or(true),
        strip_javascript:    fields.get("javascript")  .map(|v| v != "0").unwrap_or(true),
        strip_hidden_layers: fields.get("layers")      .map(|v| v != "0").unwrap_or(true),
        strip_search_index:  fields.get("search_index").map(|v| v != "0").unwrap_or(true),
    };

    match crate::pdf_tools::sanitize_pdf(&bytes, &opts) {
        Ok(out) => pdf_response(out, "sanitized.pdf"),
        Err(e)  => err500(e),
    }
}

// ─── Flatten ─────────────────────────────────────────────────────────────────

pub async fn pdf_flatten(multipart: Multipart) -> impl IntoResponse {
    let (bytes, _) = collect_multipart(multipart).await;
    let bytes = match bytes { Some(b) => b, None => return err400("No PDF uploaded") };

    match crate::pdf_tools::flatten_pdf(&bytes) {
        Ok(out) => pdf_response(out, "flattened.pdf"),
        Err(e)  => err500(e),
    }
}

// ─── Compare ─────────────────────────────────────────────────────────────────

#[derive(serde::Serialize)]
pub struct CompareResult {
    pub page_count_a:  usize,
    pub page_count_b:  usize,
    pub differences:   Vec<CompareDiff>,
}

#[derive(serde::Serialize)]
pub struct CompareDiff {
    pub page:    usize,
    pub kind:    String,
    pub detail:  String,
}

pub async fn pdf_compare(mut multipart: Multipart) -> impl IntoResponse {
    let mut pdfs: Vec<Vec<u8>> = vec![];
    while let Ok(Some(field)) = multipart.next_field().await {
        if let Ok(b) = field.bytes().await {
            if !b.is_empty() { pdfs.push(b.to_vec()); }
        }
        if pdfs.len() == 2 { break; }
    }
    if pdfs.len() < 2 {
        return (StatusCode::BAD_REQUEST, "Upload 2 PDFs to compare").into_response();
    }
    let doc_a = match lopdf::Document::load_mem(&pdfs[0]) { Ok(d) => d, Err(e) => return err500(e) };
    let doc_b = match lopdf::Document::load_mem(&pdfs[1]) { Ok(d) => d, Err(e) => return err500(e) };

    let count_a = doc_a.get_pages().len();
    let count_b = doc_b.get_pages().len();
    let mut diffs = vec![];

    if count_a != count_b {
        diffs.push(CompareDiff {
            page: 0,
            kind: "page-count".into(),
            detail: format!("File A has {} pages; File B has {} pages", count_a, count_b),
        });
    }

    // Simple text extraction comparison per page
    let min_pages = count_a.min(count_b);
    for p in 1..=min_pages {
        let text_a = extract_page_text(&doc_a, p);
        let text_b = extract_page_text(&doc_b, p);
        if text_a != text_b {
            diffs.push(CompareDiff {
                page:   p,
                kind:   "content".into(),
                detail: format!("Page {} has different text content", p),
            });
        }
    }

    Json(CompareResult {
        page_count_a: count_a,
        page_count_b: count_b,
        differences:  diffs,
    }).into_response()
}

fn extract_page_text(doc: &lopdf::Document, page_num: usize) -> String {
    // Basic text extraction from content streams (not full CMap decoding)
    if let Ok(contents) = doc.get_page_content(
        *doc.get_pages().get(&(page_num as u32)).unwrap_or(&(0, 0))
    ) {
        String::from_utf8_lossy(&contents).to_string()
    } else {
        String::new()
    }
}

// ─── Unlock PDF ──────────────────────────────────────────────────────────────

pub async fn pdf_unlock(multipart: Multipart) -> impl IntoResponse {
    let (bytes, fields) = collect_multipart(multipart).await;
    let bytes = match bytes { Some(b) => b, None => return err400("No PDF uploaded") };
    let pass = fields.get("password").map(|s| s.as_str()).unwrap_or("");

    match crate::pdf_tools::protect::decrypt_pdf(&bytes, pass) {
        Ok(out) => pdf_response(out, "unlocked.pdf"),
        Err(e)  => err500(e),
    }
}


// --- Images to PDF ---

pub async fn pdf_images_to_pdf(mut multipart: Multipart) -> impl IntoResponse {
    let mut images: Vec<Vec<u8>> = vec![];
    while let Ok(Some(field)) = multipart.next_field().await {
        if let Ok(b) = field.bytes().await {
            if !b.is_empty() { images.push(b.to_vec()); }
        }
    }
    if images.is_empty() {
        return err400("No images uploaded");
    }
    let refs: Vec<&[u8]> = images.iter().map(|v| v.as_slice()).collect();
    match crate::pdf_tools::images_to_pdf(&refs) {
        Ok(out) => pdf_response(out, "images.pdf"),
        Err(e)  => err500(e),
    }
}

// --- Create Version ---

#[derive(Deserialize)]
pub struct VersionCreate {
    pub label: Option<String>,
    pub created_by: Option<String>,
}

pub async fn create_version(
    State(state): State<SharedState>,
    Path(doc_id): Path<Uuid>,
    Json(body): Json<VersionCreate>,
) -> ApiResult<serde_json::Value> {
    let doc = {
        let store = state.store.lock().await;
        store.load_document(doc_id).map_err(ApiError::from)?
    };
    let snapshot = crate::core::document::DocumentSnapshot {
        id: uuid::Uuid::new_v4(),
        document_id: doc_id,
        version_label: body.label.unwrap_or_else(|| "Untitled Version".into()),
        created_at: chrono::Utc::now(),
        created_by: body.created_by.unwrap_or_else(|| "me".into()),
        document: doc,
    };
    let snap_id = snapshot.id;
    let snap_label = snapshot.version_label.clone();
    let mut store = state.store.lock().await;
    store.save_version(&snapshot).map_err(ApiError::from)?;
    Ok(Json(serde_json::json!({
        "ok": true,
        "id": snap_id,
        "label": snap_label,
    })))
}

// --- Import File ---

pub async fn import_file(
    State(state): State<SharedState>,
    mut multipart: axum::extract::Multipart,
) -> impl IntoResponse {
    let mut file_bytes: Option<Vec<u8>> = None;
    let mut filename = String::from("upload");

    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap_or("").to_string();
        if name == "file" {
            filename = field.file_name().unwrap_or("upload").to_string();
            if let Ok(b) = field.bytes().await {
                file_bytes = Some(b.to_vec());
            }
        }
    }

    let bytes = match file_bytes {
        Some(b) => b,
        None => return (StatusCode::BAD_REQUEST, "No file uploaded").into_response(),
    };

    let ext = filename.rsplit('.').next().unwrap_or("").to_lowercase();
    let mut doc = match ext.as_str() {
        "docx" => match crate::formats::docx::parse_docx(&bytes) {
            Ok(d) => d,
            Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("DOCX parse error: {}", e)).into_response(),
        },
        "odt" => match crate::formats::odf::parse_odt(&bytes) {
            Ok(d) => d,
            Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("ODT parse error: {}", e)).into_response(),
        },
        "rtf" => match crate::formats::rtf::parse_rtf(&bytes) {
            Ok(d) => d,
            Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("RTF parse error: {}", e)).into_response(),
        },
        _ => return (StatusCode::BAD_REQUEST, format!("Unsupported format: .{}", ext)).into_response(),
    };

    if doc.title.is_empty() || doc.title == "Untitled" {
        doc.title = std::path::Path::new(&filename)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Imported Document")
            .to_string();
    }

    let id = doc.id;
    let title = doc.title.clone();
    let mut store = state.store.lock().await;
    if let Err(e) = store.save_document(&doc) {
        return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
    }
    Json(serde_json::json!({ "id": id, "title": title })).into_response()
}

// --- AI Config ---

pub async fn get_ai_config(
    State(state): State<SharedState>,
) -> ApiResult<serde_json::Value> {
    let store = state.store.lock().await;
    let provider: Option<String> = store.get_pref("ai_provider").map_err(ApiError::from)?;
    let groq_key: Option<String> = store.get_pref("ai_groq_key").map_err(ApiError::from)?;
    let ollama_url: Option<String> = store.get_pref("ai_ollama_url").map_err(ApiError::from)?;
    let groq_model: Option<String> = store.get_pref("ai_groq_model").map_err(ApiError::from)?;
    let ollama_model: Option<String> = store.get_pref("ai_ollama_model").map_err(ApiError::from)?;
    Ok(Json(serde_json::json!({
        "provider":     provider.unwrap_or_else(|| "ollama".into()),
        "groq_key":     groq_key.unwrap_or_default(),
        "ollama_url":   ollama_url.unwrap_or_else(|| "http://localhost:11434".into()),
        "groq_model":   groq_model.unwrap_or_else(|| "llama3-8b-8192".into()),
        "ollama_model": ollama_model.unwrap_or_else(|| "llama3.2".into()),
    })))
}

#[derive(Deserialize)]
pub struct AiConfigBody {
    pub provider: Option<String>,
    pub groq_key: Option<String>,
    pub ollama_url: Option<String>,
    pub groq_model: Option<String>,
    pub ollama_model: Option<String>,
}

pub async fn set_ai_config(
    State(state): State<SharedState>,
    Json(body): Json<AiConfigBody>,
) -> ApiResult<serde_json::Value> {
    let mut store = state.store.lock().await;
    if let Some(v) = &body.provider     { store.set_pref("ai_provider",     v).map_err(ApiError::from)?; }
    if let Some(v) = &body.groq_key     { store.set_pref("ai_groq_key",     v).map_err(ApiError::from)?; }
    if let Some(v) = &body.ollama_url   { store.set_pref("ai_ollama_url",   v).map_err(ApiError::from)?; }
    if let Some(v) = &body.groq_model   { store.set_pref("ai_groq_model",   v).map_err(ApiError::from)?; }
    if let Some(v) = &body.ollama_model { store.set_pref("ai_ollama_model", v).map_err(ApiError::from)?; }
    Ok(Json(serde_json::json!({ "ok": true })))
}
