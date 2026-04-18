//! REST API handlers for the PDF Office server.

use axum::{
    extract::{Path, State, Json, Multipart},
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
