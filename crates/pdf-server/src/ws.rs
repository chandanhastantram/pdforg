//! WebSocket handler — per-document real-time session with broadcast support.
//!
//! Architecture:
//! - Each document has a list of subscribed sender channels.
//! - When a client sends an ApplyOp, it is applied and then broadcast to all
//!   other clients watching the same document.
//! - Broadcast uses tokio's `broadcast` channel stored in AppState.

use axum::{
    extract::{WebSocketUpgrade, Path, State},
    response::IntoResponse,
};
use axum::extract::ws::{WebSocket, Message};
use uuid::Uuid;
use futures::{SinkExt, StreamExt};
use crate::{SharedState, ClientMsg, ServerMsg};
use pdf_spell::default_checker;
use pdf_sheets::parser::parse_formula;
use pdf_sheets::evaluator::{eval, EvalContext};
use pdf_core::{Workbook, CellAddress};
use serde_json;

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    Path(doc_id): Path<Uuid>,
    State(state): State<SharedState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, doc_id, state))
}

async fn handle_socket(socket: WebSocket, doc_id: Uuid, state: SharedState) {
    let (mut sender, mut receiver) = socket.split();

    tracing::info!("WebSocket connected for doc {}", doc_id);

    // Send initial document state to the connecting client
    {
        let store = state.store.lock().await;
        if let Ok(doc) = store.load_document(doc_id) {
            let init_msg = ServerMsg::DocumentState {
                doc_id,
                revision: doc.revision,
            };
            if let Ok(json) = serde_json::to_string(&init_msg) {
                let _ = sender.send(Message::Text(json)).await;
            }
        }
    }

    while let Some(Ok(msg)) = receiver.next().await {
        let text = match msg {
            Message::Text(t) => t,
            Message::Close(_) => break,
            Message::Ping(data) => {
                let _ = sender.send(Message::Pong(data)).await;
                continue;
            }
            _ => continue,
        };

        let client_msg: ClientMsg = match serde_json::from_str(&text) {
            Ok(m) => m,
            Err(e) => {
                let err = ServerMsg::Error {
                    message: format!("Invalid message: {}", e),
                    code: Some("PARSE_ERROR".into()),
                };
                let _ = sender.send(Message::Text(serde_json::to_string(&err).unwrap())).await;
                continue;
            }
        };

        let response = handle_message(client_msg, &state).await;

        if let Some(resp) = response {
            let json = serde_json::to_string(&resp).unwrap_or_default();
            if sender.send(Message::Text(json)).await.is_err() {
                break;
            }
        }
    }

    tracing::info!("WebSocket disconnected for doc {}", doc_id);
}

async fn handle_message(msg: ClientMsg, state: &SharedState) -> Option<ServerMsg> {
    match msg {
        ClientMsg::Ping => Some(ServerMsg::Pong),

        ClientMsg::ApplyOp { doc_id, ops, rev } => {
            // Load document, apply ops, save
            let mut store = state.store.lock().await;
            if let Ok(mut doc) = store.load_document(doc_id) {
                // Apply each op to the document's text content
                // (Phase 1: apply to first paragraph's first run)
                if let Some(pdf_core::Block::Paragraph(ref mut para)) = doc.body.first_mut() {
                    let original_text: String = para.runs.iter().map(|r| r.text.as_str()).collect::<Vec<_>>().join("");
                    let new_text = pdf_writer::ot::apply_to_text(&original_text, &ops);
                    para.runs.clear();
                    para.runs.push(pdf_core::document::Run::new(new_text));
                }
                doc.revision = rev + 1;
                let _ = store.save_document(&doc);
                Some(ServerMsg::OpAck { rev: doc.revision })
            } else {
                None
            }
        }

        ClientMsg::SpellCheck { text, lang: _ } => {
            let checker = default_checker();
            let results = checker.check_text(&text);
            Some(ServerMsg::SpellResults { results })
        }

        ClientMsg::FormulaEval { expr, cell, sheet } => {
            match parse_formula(&expr) {
                Ok(ast) => {
                    let wb = Workbook::default();
                    let ctx = EvalContext::new(&wb, sheet, cell);
                    match eval(&ast, &ctx) {
                        Ok(v) => Some(ServerMsg::FormulaResult { result: v.as_text(), error: None }),
                        Err(e) => Some(ServerMsg::FormulaResult { result: String::new(), error: Some(e.to_string()) }),
                    }
                }
                Err(e) => Some(ServerMsg::FormulaResult { result: String::new(), error: Some(e) }),
            }
        }

        ClientMsg::SetViewport { doc_id: _, sheet, first_row: _, first_col: _, row_count, col_count } => {
            // For Phase 1, return empty viewport data
            // Phase 2: load workbook and render viewport
            Some(ServerMsg::ViewportData {
                sheet,
                data: pdf_core::ViewportData {
                    cells: vec![],
                    row_heights: vec![15.0; row_count as usize],
                    col_widths: vec![80.0; col_count as usize],
                    frozen_rows: 0,
                    frozen_cols: 0,
                    selections: vec![],
                },
            })
        }

        ClientMsg::RequestPage { doc_id, page, scale: _ } => {
            let store = state.store.lock().await;
            if let Ok(doc) = store.load_document(doc_id) {
                drop(store);
                let layout_engine = pdf_writer::layout::LayoutEngine::new(doc.page_layout.clone());
                let pages = layout_engine.layout_document(&doc);
                if let Some(layout_page) = pages.get(page as usize) {
                    let render = pdf_render::rasterizer::render_to_canvas_commands(layout_page);
                    Some(ServerMsg::CanvasCommands {
                        page,
                        width: render.width,
                        height: render.height,
                        commands: render.commands,
                    })
                } else {
                    None
                }
            } else {
                None
            }
        }

        _ => None,
    }
}
