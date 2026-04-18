pub mod api;
pub mod assets;
pub mod protocol;
pub mod ws;

use axum::{Router, routing::{get, post, put, delete}};
use tower_http::cors::CorsLayer;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::storage::Store;

pub use protocol::*;

pub struct AppState {
    pub store: Mutex<Store>,
    pub data_dir: std::path::PathBuf,
}

impl AppState {
    pub fn new(store: Store, data_dir: std::path::PathBuf) -> Self {
        AppState { store: Mutex::new(store), data_dir }
    }
}

pub type SharedState = Arc<AppState>;

pub async fn start_server(port: u16, data_dir: std::path::PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let store = Store::open(&data_dir)?;
    let state: SharedState = Arc::new(AppState::new(store, data_dir));
    let app = build_router(state);
    let addr = format!("127.0.0.1:{}", port);
    tracing::info!("PDF Office server listening on http://{}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

pub fn build_router(state: SharedState) -> Router {
    Router::new()
        .route("/", get(assets::serve_index))
        .route("/download", get(assets::serve_download))
        .route("/assets/*path", get(assets::serve_asset))
        .route("/api/documents", get(api::list_documents))
        .route("/api/documents", post(api::create_document))
        .route("/api/documents/:id", get(api::get_document))
        .route("/api/documents/:id", post(api::save_document))
        .route("/api/documents/:id", delete(api::delete_document))
        .route("/api/documents/:id/export", post(api::export_document))
        .route("/api/documents/:id/versions", get(api::list_versions))
        .route("/api/documents/:id/versions/:vid", get(api::get_version))
        .route("/api/documents/:id/versions/:vid/restore", post(api::restore_version))
        .route("/api/documents/:id/comments", get(api::list_comments))
        .route("/api/documents/:id/comments", post(api::add_comment))
        .route("/api/spell", post(api::spell_check))
        .route("/api/formula", post(api::eval_formula))
        .route("/api/convert", post(api::convert_file))
        .route("/api/preferences/:key", get(api::get_preference))
        .route("/api/preferences/:key", put(api::set_preference))
        .route("/ws/:doc_id", get(ws::ws_handler))
        .layer(CorsLayer::permissive())
        .with_state(state)
}

pub fn create_router(state: SharedState) -> Router {
    build_router(state)
}
