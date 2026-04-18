//! Embedded static assets — served from memory, not from filesystem.

use axum::{
    extract::Path,
    response::{Html, IntoResponse, Response},
    http::{StatusCode, header},
};

/// Main application HTML shell
pub async fn serve_index() -> Html<&'static str> {
    Html(INDEX_HTML)
}

/// Download page for all platforms
pub async fn serve_download() -> Html<&'static str> {
    Html(DOWNLOAD_HTML)
}

/// Static assets (CSS, JS, WASM)
pub async fn serve_asset(Path(path): Path<String>) -> Response {
    match path.as_str() {
        "app.css" => (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "text/css")],
            APP_CSS,
        ).into_response(),
        _ => (StatusCode::NOT_FOUND, "Asset not found").into_response(),
    }
}

// ─── Embedded HTML ────────────────────────────────────────────────────────────

const INDEX_HTML: &str = r##"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>PDF Office — Your Complete Office Suite</title>
  <meta name="description" content="PDF Office: local-first, privacy-respecting office suite. Write documents, spreadsheets, presentations — all offline.">
  <link rel="preconnect" href="https://fonts.googleapis.com">
  <link href="https://fonts.googleapis.com/css2?family=Inter:wght@300;400;500;600;700&family=JetBrains+Mono:wght@400;500&display=swap" rel="stylesheet">
  <style>
    :root {
      --primary: #1e6ffe;
      --primary-dark: #1558d6;
      --primary-light: #e8f0ff;
      --writer-color: #1e6ffe;
      --sheets-color: #10a760;
      --slides-color: #f5a623;
      --pdf-color: #e0443c;
      --bg: #f0f2f5;
      --surface: #ffffff;
      --surface2: #f8f9fa;
      --border: #e1e4e8;
      --border2: #d0d3d9;
      --text: #1a1d23;
      --text2: #6b7280;
      --text3: #9ca3af;
      --ribbon: #ffffff;
      --ribbon-hover: #e8f0ff;
      --ribbon-active: #d1e3ff;
      --titlebar-bg: #1e6ffe;
      --titlebar-text: #ffffff;
      --sidebar-bg: #2c3e50;
      --sidebar-text: #cdd5e0;
      --sidebar-active: #1e6ffe;
      --status-bg: #1e6ffe;
      --shadow: 0 2px 8px rgba(0,0,0,0.12);
      --shadow-lg: 0 8px 32px rgba(0,0,0,0.16);
    }

    [data-theme="dark"] {
      --bg: #1a1d23;
      --surface: #242830;
      --surface2: #2c3140;
      --border: #3a3f4b;
      --border2: #454a58;
      --text: #e8ecf1;
      --text2: #9aa5b4;
      --text3: #6b7585;
      --ribbon: #242830;
      --ribbon-hover: #2f3545;
      --ribbon-active: #374060;
      --titlebar-bg: #1a1d23;
      --sidebar-bg: #1a1d23;
      --sidebar-text: #9aa5b4;
    }

    * { box-sizing: border-box; margin: 0; padding: 0; }
    body { font-family: 'Inter', sans-serif; background: var(--bg); color: var(--text); min-height: 100vh; overflow: hidden; }

    /* ── Title Bar ── */
    .titlebar {
      display: flex; align-items: center; height: 44px;
      background: var(--titlebar-bg); color: var(--titlebar-text);
      padding: 0 16px; gap: 12px; user-select: none;
      position: relative; z-index: 200;
    }
    .app-logo { display: flex; align-items: center; gap: 8px; font-weight: 700; font-size: 15px; cursor: pointer; }
    .logo-icon {
      width: 30px; height: 30px; background: white; border-radius: 6px;
      display: flex; align-items: center; justify-content: center; font-size: 16px;
    }
    .doc-name-bar {
      flex: 1; display: flex; align-items: center; justify-content: center;
    }
    .doc-name-input {
      background: rgba(255,255,255,0.15); border: 1px solid rgba(255,255,255,0.25);
      color: white; padding: 4px 12px; border-radius: 6px; font-size: 14px;
      font-weight: 500; width: 300px; text-align: center; outline: none;
    }
    .doc-name-input:focus { background: rgba(255,255,255,0.25); border-color: rgba(255,255,255,0.5); }
    .titlebar-right { display: flex; align-items: center; gap: 8px; }
    .tb-btn {
      padding: 5px 12px; border-radius: 5px; font-size: 12px; font-weight: 500;
      cursor: pointer; border: 1px solid rgba(255,255,255,0.3);
      background: rgba(255,255,255,0.1); color: white; transition: all 0.15s;
    }
    .tb-btn:hover { background: rgba(255,255,255,0.25); }
    .tb-btn-save { background: rgba(255,255,255,0.2); border-color: rgba(255,255,255,0.5); }

    /* ── Ribbon Tabs ── */
    .ribbon-tabs {
      display: flex; align-items: flex-end; height: 36px;
      background: var(--ribbon); border-bottom: 1px solid var(--border);
      padding: 0 8px; gap: 2px; position: relative; z-index: 100;
    }
    .rtab {
      padding: 8px 16px; font-size: 13px; font-weight: 500; color: var(--text2);
      cursor: pointer; border-radius: 4px 4px 0 0; border: 1px solid transparent;
      border-bottom: none; transition: all 0.15s; position: relative;
      bottom: -1px; background: transparent;
    }
    .rtab:hover { color: var(--primary); background: var(--ribbon-hover); }
    .rtab.active {
      color: var(--primary); background: var(--surface);
      border-color: var(--border); border-bottom-color: var(--surface);
      font-weight: 600;
    }
    .ribbon-spacer { flex: 1; }
    .quick-access {
      display: flex; align-items: center; gap: 4px; padding-bottom: 4px;
    }
    .qa-btn {
      padding: 4px 8px; border-radius: 4px; font-size: 12px; cursor: pointer;
      border: none; background: transparent; color: var(--text2); transition: all 0.15s;
    }
    .qa-btn:hover { background: var(--ribbon-hover); color: var(--text); }

    /* ── Ribbon Content ── */
    .ribbon {
      background: var(--ribbon); border-bottom: 2px solid var(--border);
      padding: 6px 12px; display: flex; align-items: center; gap: 4px;
      flex-wrap: nowrap; overflow-x: auto; min-height: 56px;
    }
    .ribbon::-webkit-scrollbar { height: 3px; }
    .ribbon::-webkit-scrollbar-thumb { background: var(--border2); border-radius: 2px; }
    .ribbon-group {
      display: flex; align-items: center; gap: 2px; padding: 0 8px;
      border-right: 1px solid var(--border); margin-right: 4px;
    }
    .ribbon-group:last-child { border-right: none; }
    .ribbon-group-label {
      font-size: 10px; color: var(--text3); text-align: center;
      margin-top: 2px; white-space: nowrap;
    }
    .ribbon-col { display: flex; flex-direction: column; align-items: center; gap: 1px; }
    .rbtn {
      display: flex; flex-direction: column; align-items: center; justify-content: center;
      padding: 4px 8px; border: 1px solid transparent; border-radius: 4px;
      cursor: pointer; background: transparent; color: var(--text); transition: all 0.12s;
      min-width: 44px; gap: 2px; font-family: inherit; white-space: nowrap;
    }
    .rbtn:hover { background: var(--ribbon-hover); border-color: var(--border); }
    .rbtn:active { background: var(--ribbon-active); }
    .rbtn-icon { font-size: 18px; line-height: 1; }
    .rbtn-label { font-size: 10px; color: var(--text2); }
    .rbtn.wide { flex-direction: row; gap: 6px; padding: 6px 10px; min-width: auto; }
    .rbtn.wide .rbtn-label { font-size: 12px; color: var(--text); }
    .rbtn.highlighted { background: var(--primary-light); border-color: var(--primary); }
    .rbtn.highlighted .rbtn-label { color: var(--primary); }

    /* Format controls in ribbon */
    .ribbon-select {
      padding: 4px 6px; border: 1px solid var(--border); border-radius: 4px;
      background: var(--surface); color: var(--text); font-size: 12px;
      cursor: pointer; outline: none;
    }
    .ribbon-select:focus { border-color: var(--primary); }
    .ribbon-divider { width: 1px; height: 36px; background: var(--border); margin: 0 4px; }

    /* ── Layout ── */
    .app-shell { display: flex; height: calc(100vh - 44px); }

    /* ── Sidebar (App Switcher) ── */
    .sidebar {
      width: 60px; background: var(--sidebar-bg); display: flex;
      flex-direction: column; align-items: center; padding: 12px 0; gap: 4px;
      z-index: 50; flex-shrink: 0;
    }
    .sidebar-btn {
      width: 44px; height: 44px; border-radius: 10px; display: flex;
      flex-direction: column; align-items: center; justify-content: center;
      cursor: pointer; transition: all 0.15s; background: transparent;
      border: none; color: var(--sidebar-text); gap: 2px; padding: 4px;
    }
    .sidebar-btn:hover { background: rgba(255,255,255,0.12); color: white; }
    .sidebar-btn.active { background: var(--primary); color: white; }
    .sidebar-icon { font-size: 18px; }
    .sidebar-label { font-size: 8px; font-weight: 600; text-transform: uppercase; letter-spacing: 0.04em; }
    .sidebar-divider { width: 32px; height: 1px; background: rgba(255,255,255,0.1); margin: 4px 0; }
    .sidebar-spacer { flex: 1; }

    /* ── Content Area ── */
    .content-area { flex: 1; overflow: hidden; display: flex; flex-direction: column; }
    .ribbon-container { background: var(--surface); display: none; }
    .ribbon-container.visible { display: block; }
    .section { display: none; flex: 1; overflow: hidden; }
    .section.active { display: flex; flex-direction: column; }

    /* ── Home / Dashboard ── */
    .home-page {
      flex: 1; overflow-y: auto; background: var(--bg);
      padding: 40px; display: flex; flex-direction: column; gap: 40px;
    }
    .home-welcome { text-align: center; }
    .home-welcome h1 { font-size: 36px; font-weight: 700; color: var(--text); margin-bottom: 8px; }
    .home-welcome p { font-size: 16px; color: var(--text2); }
    .app-cards { display: grid; grid-template-columns: repeat(4, 1fr); gap: 16px; max-width: 900px; margin: 0 auto; }
    .app-card {
      background: var(--surface); border: 2px solid var(--border); border-radius: 16px;
      padding: 28px 20px; cursor: pointer; transition: all 0.2s; text-align: center;
      display: flex; flex-direction: column; align-items: center; gap: 12px;
    }
    .app-card:hover { transform: translateY(-4px); box-shadow: var(--shadow-lg); border-color: var(--card-color, var(--primary)); }
    .app-card-icon { font-size: 40px; }
    .app-card-icon-bg { width: 72px; height: 72px; border-radius: 18px; display: flex; align-items: center; justify-content: center; }
    .app-card-name { font-size: 16px; font-weight: 700; color: var(--text); }
    .app-card-desc { font-size: 12px; color: var(--text2); }

    .new-btns { display: flex; gap: 12px; justify-content: center; flex-wrap: wrap; }
    .new-btn {
      display: inline-flex; align-items: center; gap: 8px; padding: 12px 24px;
      border-radius: 10px; font-size: 14px; font-weight: 600; cursor: pointer;
      border: none; transition: all 0.15s; color: white;
    }
    .new-btn:hover { transform: translateY(-2px); box-shadow: 0 4px 16px rgba(0,0,0,0.2); }

    .recent-section h2 { font-size: 20px; font-weight: 700; margin-bottom: 16px; display: flex; align-items: center; gap: 8px; }
    .recent-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(180px, 1fr)); gap: 12px; }
    .recent-card {
      background: var(--surface); border: 1px solid var(--border); border-radius: 12px;
      padding: 16px; cursor: pointer; transition: all 0.15s; position: relative;
      animation: slideUp 0.3s ease;
    }
    .recent-card:hover { box-shadow: var(--shadow); border-color: var(--border2); transform: translateY(-2px); }
    .recent-card-icon { font-size: 32px; margin-bottom: 10px; }
    .recent-card-name { font-size: 13px; font-weight: 600; margin-bottom: 4px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
    .recent-card-date { font-size: 11px; color: var(--text3); }
    .recent-card-del {
      position: absolute; top: 8px; right: 8px; width: 22px; height: 22px;
      border-radius: 50%; border: none; background: #fee2e2; color: #dc2626;
      font-size: 12px; cursor: pointer; display: none; align-items: center; justify-content: center;
    }
    .recent-card:hover .recent-card-del { display: flex; }
    .empty-state { text-align: center; padding: 48px; color: var(--text3); }

    /* ── Writer ── */
    .writer-page { flex: 1; overflow-y: auto; background: #e8eaed; display: flex; justify-content: center; padding: 32px 16px; }
    .writer-canvas {
      width: 794px; min-height: 1123px; background: white; color: #111;
      padding: 72px 96px; border-radius: 4px; box-shadow: 0 2px 16px rgba(0,0,0,0.15);
      outline: none; font-family: 'Times New Roman', serif; font-size: 12pt;
      line-height: 1.6; caret-color: #1e6ffe; flex-shrink: 0;
    }
    .writer-canvas h1 { font-size: 24pt; margin-bottom: 16px; }
    .writer-canvas:focus { box-shadow: 0 4px 24px rgba(30,111,254,0.2); }

    /* ── Sheets ── */
    .sheets-page { flex: 1; display: flex; flex-direction: column; overflow: hidden; background: var(--surface); }
    .sheets-formula-bar {
      display: flex; align-items: center; gap: 8px; padding: 4px 12px;
      background: var(--surface); border-bottom: 1px solid var(--border);
    }
    .cell-ref-box {
      width: 64px; padding: 4px 8px; border: 1px solid var(--border); border-radius: 4px;
      font-family: 'JetBrains Mono', monospace; font-size: 13px; background: var(--surface2);
      color: var(--text); text-align: center; outline: none;
    }
    .formula-input {
      flex: 1; padding: 4px 8px; border: 1px solid var(--border); border-radius: 4px;
      font-family: 'JetBrains Mono', monospace; font-size: 13px;
      background: var(--surface); color: var(--text); outline: none;
    }
    .formula-input:focus { border-color: var(--primary); }
    .sheets-grid-area { flex: 1; overflow: hidden; position: relative; }
    .sheets-tabs {
      display: flex; align-items: center; border-top: 1px solid var(--border);
      background: var(--surface2); padding: 4px 8px; gap: 4px;
    }
    .sheet-tab {
      padding: 4px 16px; border-radius: 4px 4px 0 0; font-size: 12px;
      font-weight: 600; cursor: pointer; background: var(--primary); color: white;
    }
    .sheet-add-btn { background: none; border: none; color: var(--text2); cursor: pointer; font-size: 18px; padding: 0 4px; }
    .sheet-add-btn:hover { color: var(--primary); }

    /* ── Slides ── */
    .slides-page { flex: 1; display: flex; overflow: hidden; }
    .slides-panel { width: 176px; background: var(--surface2); border-right: 1px solid var(--border); overflow-y: auto; padding: 8px; }
    .slide-thumb {
      aspect-ratio: 4/3; background: white; border: 2px solid var(--border);
      border-radius: 6px; margin-bottom: 8px; cursor: pointer; transition: all 0.15s;
      display: flex; align-items: center; justify-content: center; font-size: 12px;
      color: var(--text2); overflow: hidden; position: relative;
    }
    .slide-thumb.selected { border-color: var(--primary); box-shadow: 0 0 0 2px rgba(30,111,254,0.25); }
    .slide-thumb-num { position: absolute; bottom: 4px; left: 4px; font-size: 10px; color: var(--text3); font-weight: 600; }
    .slides-canvas-area { flex: 1; background: #e8eaed; display: flex; align-items: center; justify-content: center; padding: 32px; }
    .slide-canvas {
      width: 720px; height: 540px; background: white; border-radius: 4px;
      box-shadow: 0 4px 24px rgba(0,0,0,0.15); position: relative; overflow: hidden;
    }

    /* ── PDF Tools ── */
    .pdf-page { flex: 1; display: flex; flex-direction: column; }
    .pdf-toolbar {
      display: flex; align-items: center; gap: 8px; padding: 8px 16px;
      background: var(--surface); border-bottom: 1px solid var(--border); flex-wrap: wrap;
    }
    .pdf-viewer-area {
      flex: 1; overflow-y: auto; background: #e8eaed;
      display: flex; align-items: center; justify-content: center; padding: 32px;
    }

    /* ── Status Bar ── */
    .status-bar {
      height: 26px; background: var(--primary); color: white;
      display: flex; align-items: center; padding: 0 12px; font-size: 11px; gap: 16px;
    }
    .status-dot { width: 7px; height: 7px; border-radius: 50%; background: rgba(255,255,255,0.6); animation: pulse 2s infinite; }
    .status-right { margin-left: auto; opacity: 0.8; }

    /* ── Buttons ── */
    .btn { padding: 7px 14px; border-radius: 6px; font-size: 13px; font-weight: 500; cursor: pointer; border: 1px solid var(--border); background: var(--surface); color: var(--text); font-family: inherit; transition: all 0.15s; }
    .btn:hover { background: var(--surface2); border-color: var(--border2); }
    .btn-primary { background: var(--primary); border-color: var(--primary); color: white; }
    .btn-primary:hover { background: var(--primary-dark); border-color: var(--primary-dark); }
    .btn-danger { background: #ef4444; border-color: #ef4444; color: white; }
    .btn-sm { padding: 4px 10px; font-size: 12px; }
    .btn-icon { padding: 6px; min-width: 30px; text-align: center; }

    /* ── Modals & Overlays ── */
    .modal-overlay {
      display: none; position: fixed; inset: 0; background: rgba(0,0,0,0.5);
      z-index: 2000; align-items: center; justify-content: center;
    }
    .modal-overlay.open { display: flex; }
    .modal {
      background: var(--surface); border: 1px solid var(--border); border-radius: 16px;
      padding: 32px; width: 480px; max-width: 90vw; box-shadow: var(--shadow-lg);
      animation: scaleIn 0.2s ease;
    }
    .modal-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 20px; }
    .modal-title { font-size: 18px; font-weight: 700; }
    .modal-close { background: none; border: none; font-size: 20px; cursor: pointer; color: var(--text2); padding: 4px; border-radius: 4px; }
    .modal-close:hover { background: var(--surface2); }
    .modal-body { display: flex; flex-direction: column; gap: 16px; }
    .modal-footer { display: flex; gap: 8px; justify-content: flex-end; margin-top: 20px; }

    /* ── Panels ── */
    .side-panel {
      position: fixed; right: -320px; top: 0; bottom: 0; width: 300px;
      background: var(--surface); border-left: 1px solid var(--border);
      z-index: 400; padding: 20px; transition: right 0.25s ease;
      overflow-y: auto; box-shadow: -4px 0 24px rgba(0,0,0,0.1);
    }
    .side-panel.open { right: 0; }
    .panel-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 16px; padding-bottom: 12px; border-bottom: 1px solid var(--border); }
    .panel-title { font-size: 15px; font-weight: 700; }
    .version-item { padding: 12px; border: 1px solid var(--border); border-radius: 8px; margin-bottom: 8px; }
    .version-label { font-size: 13px; font-weight: 600; margin-bottom: 4px; }
    .version-date { font-size: 11px; color: var(--text3); margin-bottom: 8px; }

    /* ── Find Bar ── */
    .find-bar {
      display: none; position: fixed; top: 88px; right: 16px; z-index: 500;
      background: var(--surface); border: 1px solid var(--border); border-radius: 10px;
      padding: 10px 14px; display: none; align-items: center; gap: 8px;
      box-shadow: var(--shadow-lg); width: 320px;
    }
    .find-bar.open { display: flex; }
    .find-input {
      flex: 1; padding: 6px 10px; border: 1px solid var(--border); border-radius: 6px;
      font-size: 13px; background: var(--surface2); color: var(--text); outline: none;
    }
    .find-input:focus { border-color: var(--primary); }

    /* ── Toast ── */
    .toast-container { position: fixed; bottom: 40px; right: 24px; z-index: 9999; display: flex; flex-direction: column; gap: 8px; align-items: flex-end; }
    .toast {
      background: var(--text); color: var(--surface); border-radius: 10px;
      padding: 12px 18px; font-size: 13px; max-width: 320px;
      animation: slideInRight 0.25s ease; display: flex; align-items: center; gap: 8px;
      box-shadow: var(--shadow-lg);
    }
    .toast.success { background: #16a34a; }
    .toast.error { background: #dc2626; }
    .toast.info { background: var(--primary); }

    /* ── Form controls ── */
    .form-group { display: flex; flex-direction: column; gap: 6px; }
    .form-label { font-size: 13px; font-weight: 600; color: var(--text); }
    .form-input {
      padding: 8px 12px; border: 1px solid var(--border); border-radius: 8px;
      background: var(--surface); color: var(--text); font-size: 14px; outline: none;
      font-family: inherit; transition: border-color 0.15s;
    }
    .form-input:focus { border-color: var(--primary); }
    .form-input-lg { padding: 10px 14px; font-size: 15px; }
    .form-select {
      padding: 8px 12px; border: 1px solid var(--border); border-radius: 8px;
      background: var(--surface); color: var(--text); font-size: 14px; outline: none;
      cursor: pointer; width: 100%;
    }
    .form-select:focus { border-color: var(--primary); }

    /* ── Drop Zone ── */
    .drop-zone {
      border: 2px dashed var(--border2); border-radius: 12px; padding: 32px;
      text-align: center; color: var(--text2); cursor: pointer; transition: all 0.2s;
    }
    .drop-zone:hover, .drop-zone.dragging { border-color: var(--primary); background: var(--primary-light); color: var(--primary); }
    .drop-zone-icon { font-size: 48px; margin-bottom: 12px; }

    /* ── Animations ── */
    @keyframes slideUp { from { opacity: 0; transform: translateY(12px); } to { opacity: 1; transform: none; } }
    @keyframes scaleIn { from { opacity: 0; transform: scale(0.95); } to { opacity: 1; transform: scale(1); } }
    @keyframes slideInRight { from { opacity: 0; transform: translateX(60px); } to { opacity: 1; transform: none; } }
    @keyframes pulse { 0%, 100% { opacity: 1; } 50% { opacity: 0.4; } }
    @keyframes spin { to { transform: rotate(360deg); } }

    /* ── Print ── */
    @media print {
      .titlebar, .ribbon-tabs, .ribbon, .sidebar, .status-bar, .find-bar, .side-panel, .modal-overlay, .toast-container { display: none !important; }
      .app-shell { height: auto !important; }
      .writer-page { background: white !important; padding: 0 !important; }
      .writer-canvas { box-shadow: none !important; border-radius: 0 !important; width: 100% !important; padding: 32px !important; }
    }

    /* ── Scrollbar ── */
    ::-webkit-scrollbar { width: 6px; height: 6px; }
    ::-webkit-scrollbar-track { background: transparent; }
    ::-webkit-scrollbar-thumb { background: var(--border2); border-radius: 3px; }
    ::-webkit-scrollbar-thumb:hover { background: var(--text3); }
  </style>
</head>
<body>

<!-- TITLE BAR -->
<div class="titlebar" id="titlebar">
  <div class="app-logo" onclick="goHome()">
    <div class="logo-icon">📄</div>
    PDF Office
  </div>
  <div class="doc-name-bar" id="doc-name-bar" style="display:none;">
    <input class="doc-name-input" id="doc-name" value="Untitled Document" onblur="renameDoc(this.value)">
  </div>
  <div class="titlebar-right">
    <button class="tb-btn tb-btn-save" id="btn-tbar-save" onclick="quickSave()" style="display:none;">💾 Save</button>
    <button class="tb-btn" onclick="openSettings()">⚙️ Settings</button>
    <button class="tb-btn" onclick="toggleTheme()" title="Toggle dark/light mode">🌓</button>
  </div>
</div>

<!-- RIBBON TABS -->
<div class="ribbon-tabs" id="ribbon-tabs" style="display:none;">
  <button class="rtab active" id="rtab-home" onclick="switchRibbonTab('home')">Home</button>
  <button class="rtab" id="rtab-insert" onclick="switchRibbonTab('insert')">Insert</button>
  <button class="rtab" id="rtab-format" onclick="switchRibbonTab('format')">Format</button>
  <button class="rtab" id="rtab-review" onclick="switchRibbonTab('review')">Review</button>
  <div class="ribbon-spacer"></div>
  <div class="quick-access">
    <button class="qa-btn" onclick="quickSave()" title="Save (Ctrl+S)">💾</button>
    <button class="qa-btn" onclick="openFindBar()" title="Find (Ctrl+F)" id="qa-find">🔍</button>
    <button class="qa-btn" onclick="window.print()" title="Print (Ctrl+P)">🖨️</button>
  </div>
</div>

<!-- RIBBON PANELS -->
<div class="ribbon-container" id="ribbon-home" style="display:block;">
  <!-- Writer Home Ribbon -->
  <div class="ribbon" id="ribbon-writer-home" style="display:none;">
    <div class="ribbon-group">
      <button class="rbtn wide highlighted" onclick="quickSave()"><span class="rbtn-icon">💾</span><span class="rbtn-label">Save</span></button>
      <button class="rbtn wide" onclick="shareDoc()"><span class="rbtn-icon">📤</span><span class="rbtn-label">Share</span></button>
    </div>
    <div class="ribbon-group">
      <button class="rbtn" id="rbn-bold" onclick="writerBold()" title="Bold (Ctrl+B)"><span class="rbtn-icon"><b>B</b></span><span class="rbtn-label">Bold</span></button>
      <button class="rbtn" id="rbn-italic" onclick="writerItalic()" title="Italic (Ctrl+I)"><span class="rbtn-icon"><i>I</i></span><span class="rbtn-label">Italic</span></button>
      <button class="rbtn" id="rbn-underline" onclick="writerUnderline()" title="Underline (Ctrl+U)"><span class="rbtn-icon"><u>U</u></span><span class="rbtn-label">Underline</span></button>
    </div>
    <div class="ribbon-group">
      <div class="ribbon-col">
        <select class="ribbon-select" id="font-family" onchange="applyFont()" title="Font Family">
          <option>Times New Roman</option><option>Inter</option><option>Arial</option>
          <option>Georgia</option><option>Courier New</option><option>Verdana</option>
        </select>
        <select class="ribbon-select" id="font-size" onchange="applyFontSize()" title="Font Size" style="width:70px;">
          <option>8</option><option>9</option><option>10</option><option>11</option>
          <option selected>12</option><option>14</option><option>16</option>
          <option>18</option><option>20</option><option>24</option><option>28</option>
          <option>32</option><option>36</option><option>48</option><option>72</option>
        </select>
      </div>
    </div>
    <div class="ribbon-group">
      <button class="rbtn" onclick="writerAlign('left')" title="Align Left"><span class="rbtn-icon">⬛</span><span class="rbtn-label">Left</span></button>
      <button class="rbtn" onclick="writerAlign('center')" title="Center"><span class="rbtn-icon">▬</span><span class="rbtn-label">Center</span></button>
      <button class="rbtn" onclick="writerAlign('right')" title="Align Right"><span class="rbtn-icon">⬛</span><span class="rbtn-label">Right</span></button>
      <button class="rbtn" onclick="writerAlign('justify')" title="Justify"><span class="rbtn-icon">≡</span><span class="rbtn-label">Justify</span></button>
    </div>
    <div class="ribbon-group">
      <button class="rbtn" onclick="insertBulletList()"><span class="rbtn-icon">•≡</span><span class="rbtn-label">List</span></button>
      <button class="rbtn" onclick="insertNumberedList()"><span class="rbtn-icon">1≡</span><span class="rbtn-label">Numbered</span></button>
      <button class="rbtn" onclick="insertHR()"><span class="rbtn-icon">─</span><span class="rbtn-label">Line</span></button>
    </div>
    <div class="ribbon-group">
      <button class="rbtn wide" onclick="exportDoc('docx')"><span class="rbtn-icon">⬇️</span><span class="rbtn-label">DOCX</span></button>
      <button class="rbtn wide" onclick="exportDoc('pdf')"><span class="rbtn-icon">⬇️</span><span class="rbtn-label">PDF</span></button>
    </div>
    <div class="ribbon-group">
      <button class="rbtn" onclick="openVersionsPanel()"><span class="rbtn-icon">🕐</span><span class="rbtn-label">History</span></button>
      <button class="rbtn" onclick="doSpellCheck()"><span class="rbtn-icon">✓</span><span class="rbtn-label">Spell</span></button>
    </div>
  </div>

  <!-- Sheets Home Ribbon -->
  <div class="ribbon" id="ribbon-sheets-home" style="display:none;">
    <div class="ribbon-group">
      <button class="rbtn wide highlighted" onclick="quickSave()"><span class="rbtn-icon">💾</span><span class="rbtn-label">Save</span></button>
    </div>
    <div class="ribbon-group">
      <button class="rbtn wide" onclick="evalFormulaBar()"><span class="rbtn-icon">fx</span><span class="rbtn-label">Apply</span></button>
      <button class="rbtn wide" onclick="clearCell()"><span class="rbtn-icon">🗑️</span><span class="rbtn-label">Clear</span></button>
    </div>
    <div class="ribbon-group">
      <button class="rbtn" onclick="insertFormulaFunc('SUM')"><span class="rbtn-icon">Σ</span><span class="rbtn-label">SUM</span></button>
      <button class="rbtn" onclick="insertFormulaFunc('AVERAGE')"><span class="rbtn-icon">x̄</span><span class="rbtn-label">AVG</span></button>
      <button class="rbtn" onclick="insertFormulaFunc('COUNT')"><span class="rbtn-icon">#</span><span class="rbtn-label">COUNT</span></button>
      <button class="rbtn" onclick="insertFormulaFunc('IF')"><span class="rbtn-icon">?</span><span class="rbtn-label">IF</span></button>
    </div>
    <div class="ribbon-group">
      <button class="rbtn wide" onclick="exportXlsx()"><span class="rbtn-icon">⬇️</span><span class="rbtn-label">XLSX</span></button>
    </div>
  </div>

  <!-- Slides Home Ribbon -->
  <div class="ribbon" id="ribbon-slides-home" style="display:none;">
    <div class="ribbon-group">
      <button class="rbtn wide highlighted" onclick="addSlide()"><span class="rbtn-icon">+</span><span class="rbtn-label">New Slide</span></button>
    </div>
    <div class="ribbon-group">
      <button class="rbtn" onclick="addTextBox()"><span class="rbtn-icon">T</span><span class="rbtn-label">Text</span></button>
      <button class="rbtn" onclick="addShape()"><span class="rbtn-icon">◻</span><span class="rbtn-label">Shape</span></button>
      <button class="rbtn" onclick="addImage()"><span class="rbtn-icon">🖼️</span><span class="rbtn-label">Image</span></button>
    </div>
    <div class="ribbon-group">
      <button class="rbtn wide" onclick="exportPptx()"><span class="rbtn-icon">⬇️</span><span class="rbtn-label">PPTX</span></button>
    </div>
  </div>

  <!-- PDF Home Ribbon -->
  <div class="ribbon" id="ribbon-pdf-home" style="display:none;">
    <div class="ribbon-group">
      <label class="rbtn wide highlighted" for="pdf-upload" style="cursor:pointer;"><span class="rbtn-icon">📂</span><span class="rbtn-label">Open PDF</span></label>
      <input type="file" id="pdf-upload" accept=".pdf" style="display:none" onchange="openPDF(event)">
    </div>
    <div class="ribbon-group">
      <button class="rbtn wide" onclick="mergePDFs()"><span class="rbtn-icon">🔗</span><span class="rbtn-label">Merge</span></button>
      <button class="rbtn wide" onclick="splitPDF()"><span class="rbtn-icon">✂️</span><span class="rbtn-label">Split</span></button>
      <button class="rbtn wide" onclick="watermarkPDF()"><span class="rbtn-icon">💧</span><span class="rbtn-label">Watermark</span></button>
    </div>
  </div>
</div>

<!-- APP SHELL -->
<div class="app-shell">

  <!-- SIDEBAR -->
  <nav class="sidebar" id="sidebar">
    <button class="sidebar-btn active" id="nav-home" onclick="goHome()" title="Home">
      <span class="sidebar-icon">🏠</span>
      <span class="sidebar-label">Home</span>
    </button>
    <div class="sidebar-divider"></div>
    <button class="sidebar-btn" id="nav-writer" onclick="openApp('writer')" title="Writer — Word Processor">
      <span class="sidebar-icon" style="color:#1e6ffe;">📝</span>
      <span class="sidebar-label">Writer</span>
    </button>
    <button class="sidebar-btn" id="nav-sheets" onclick="openApp('sheets')" title="Sheets — Spreadsheet">
      <span class="sidebar-icon" style="color:#10a760;">📊</span>
      <span class="sidebar-label">Sheets</span>
    </button>
    <button class="sidebar-btn" id="nav-slides" onclick="openApp('slides')" title="Slides — Presentation">
      <span class="sidebar-icon" style="color:#f5a623;">📽️</span>
      <span class="sidebar-label">Slides</span>
    </button>
    <button class="sidebar-btn" id="nav-pdf" onclick="openApp('pdf')" title="PDF Tools">
      <span class="sidebar-icon" style="color:#e0443c;">📄</span>
      <span class="sidebar-label">PDF</span>
    </button>
    <div class="sidebar-spacer"></div>
    <button class="sidebar-btn" onclick="showHelpModal()" title="Help & Guide">
      <span class="sidebar-icon">❓</span>
      <span class="sidebar-label">Help</span>
    </button>
  </nav>

  <!-- CONTENT AREA -->
  <div class="content-area">

    <!-- HOME SECTION -->
    <div class="section active" id="section-home">
      <div class="home-page" id="drop-zone-main" ondragover="handleDragover(event)" ondrop="handleDrop(event)">
        <div class="home-welcome">
          <h1>👋 Welcome to PDF Office</h1>
          <p>Your all-in-one, private, offline office suite. All documents stay on your computer.</p>
        </div>

        <div class="new-btns">
          <button class="new-btn" style="background:#1e6ffe;" onclick="newDocument('writer')">📝 New Document</button>
          <button class="new-btn" style="background:#10a760;" onclick="newDocument('sheets')">📊 New Spreadsheet</button>
          <button class="new-btn" style="background:#f5a623;" onclick="newDocument('slides')">📽️ New Presentation</button>
        </div>

        <div class="app-cards">
          <div class="app-card" style="--card-color:#1e6ffe;" onclick="openApp('writer')">
            <div class="app-card-icon-bg" style="background:#e8f0ff;"><span class="app-card-icon">📝</span></div>
            <div class="app-card-name">Writer</div>
            <div class="app-card-desc">Create & edit documents, supports .docx, .odt</div>
          </div>
          <div class="app-card" style="--card-color:#10a760;" onclick="openApp('sheets')">
            <div class="app-card-icon-bg" style="background:#e8f5ec;"><span class="app-card-icon">📊</span></div>
            <div class="app-card-name">Sheets</div>
            <div class="app-card-desc">Spreadsheets with 130+ formulas, exports .xlsx</div>
          </div>
          <div class="app-card" style="--card-color:#f5a623;" onclick="openApp('slides')">
            <div class="app-card-icon-bg" style="background:#fef3e2;"><span class="app-card-icon">📽️</span></div>
            <div class="app-card-name">Slides</div>
            <div class="app-card-desc">Create presentations, exports .pptx</div>
          </div>
          <div class="app-card" style="--card-color:#e0443c;" onclick="openApp('pdf')">
            <div class="app-card-icon-bg" style="background:#fde8e8;"><span class="app-card-icon">📄</span></div>
            <div class="app-card-name">PDF Tools</div>
            <div class="app-card-desc">View, merge, split, add watermarks to PDFs</div>
          </div>
        </div>

        <div class="recent-section">
          <h2>📂 Recent Documents <span id="recent-count" style="font-size:14px;font-weight:400;color:var(--text2);"></span></h2>
          <div class="recent-grid" id="recent-grid">
            <div class="empty-state">
              <div style="font-size:48px;">📂</div>
              <p>No documents yet. Create your first one above!</p>
              <p style="font-size:13px;margin-top:8px;">Or drag & drop a file anywhere on this page.</p>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- WRITER SECTION -->
    <div class="section" id="section-writer">
      <div class="writer-page">
        <div class="writer-canvas" id="doc-editor" contenteditable="true" spellcheck="false"
          oninput="onDocumentInput()" onkeydown="handleWriterKey(event)">
          <h1>Untitled Document</h1>
          <p>Start typing here — use the toolbar above to format your text.</p>
        </div>
      </div>
      <div style="position:absolute;bottom:32px;left:50%;transform:translateX(-50%);background:var(--surface);border:1px solid var(--border);border-radius:20px;padding:4px 16px;font-size:12px;color:var(--text2);display:flex;gap:16px;box-shadow:var(--shadow);">
        <span id="word-count">0 words</span>
        <span>·</span>
        <span id="char-count">0 chars</span>
        <span>·</span>
        <span id="spell-status">Ready</span>
      </div>
    </div>

    <!-- SHEETS SECTION -->
    <div class="section" id="section-sheets">
      <div class="sheets-page">
        <div class="sheets-formula-bar">
          <input class="cell-ref-box" id="cell-ref" value="A1" readonly>
          <span style="color:var(--text2);font-size:14px;font-weight:700;">fx</span>
          <input class="formula-input" id="formula-bar" placeholder="Enter a value or formula like =SUM(A1:A10)" onkeydown="formulaKeydown(event)" oninput="formulaInput(event)">
          <div id="formula-autocomplete" style="display:none;position:absolute;z-index:600;background:var(--surface);border:1px solid var(--border);border-radius:8px;box-shadow:var(--shadow-lg);max-height:200px;overflow-y:auto;min-width:180px;"></div>
          <button class="btn btn-primary btn-sm" onclick="evalFormulaBar()">✓ Enter</button>
          <span id="formula-result" style="min-width:100px;font-size:13px;color:var(--sheets-color);font-weight:600;"></span>
        </div>
        <div class="sheets-grid-area">
          <canvas id="grid-canvas" style="display:block;cursor:cell;" onmousedown="gridMousedown(event)" onkeydown="gridKeydown(event)" tabindex="0"></canvas>
        </div>
        <div class="sheets-tabs">
          <span class="sheet-tab" id="sheet-tab">Sheet1</span>
          <button class="sheet-add-btn" onclick="addSheet()" title="Add Sheet">+</button>
        </div>
      </div>
    </div>

    <!-- SLIDES SECTION -->
    <div class="section" id="section-slides">
      <div class="slides-page">
        <div class="slides-panel" id="slides-panel">
          <div class="slide-thumb selected" id="slide-thumb-0" onclick="selectSlide(0)">
            <div id="slide-thumb-content-0" style="width:100%;height:100%;display:flex;align-items:center;justify-content:center;font-size:11px;color:#888;">Slide 1</div>
            <div class="slide-thumb-num">1</div>
          </div>
        </div>
        <div class="slides-canvas-area">
          <div class="slide-canvas" id="slide-canvas" onclick="slideCanvasClick(event)">
            <div id="slide-content" style="width:100%;height:100%;padding:40px;"></div>
          </div>
        </div>
      </div>
    </div>

    <!-- PDF TOOLS SECTION -->
    <div class="section" id="section-pdf">
      <div class="pdf-page">
        <div class="pdf-toolbar">
          <label class="btn btn-primary" for="pdf-upload-alt" style="cursor:pointer;">📂 Open PDF</label>
          <input type="file" id="pdf-upload-alt" accept=".pdf" style="display:none" onchange="openPDF(event)">
          <div style="width:1px;height:24px;background:var(--border);margin:0 4px;"></div>
          <button class="btn" onclick="mergePDFs()">🔗 Merge PDFs</button>
          <button class="btn" onclick="splitPDF()">✂️ Split</button>
          <button class="btn" onclick="watermarkPDF()">💧 Watermark</button>
          <div style="flex:1;"></div>
          <span id="pdf-pages" style="font-size:13px;color:var(--text2);"></span>
        </div>
        <div class="pdf-viewer-area" id="pdf-viewer">
          <div class="drop-zone" style="max-width:500px;" onclick="document.getElementById('pdf-upload-alt').click()" ondragover="handleDragover(event)" ondrop="handleDropPDF(event)">
            <div class="drop-zone-icon">📄</div>
            <p style="font-size:16px;font-weight:600;margin-bottom:8px;">Drop a PDF here or click to open</p>
            <p style="font-size:13px;color:var(--text3);">Supports view, merge, split, and watermark</p>
          </div>
        </div>
      </div>
    </div>

  </div><!-- end content-area -->
</div><!-- end app-shell -->

<!-- STATUS BAR -->
<div class="status-bar">
  <div class="status-dot"></div>
  <span>PDF Office • Running Locally • All data stays on your computer 🔒</span>
  <span class="status-right">Rust-native · Private · No Cloud</span>
</div>

<!-- FIND BAR -->
<div class="find-bar" id="find-bar">
  <span style="font-size:13px;">🔍</span>
  <input class="find-input" id="find-input" placeholder="Find in document…" oninput="findInDoc()" onkeydown="findKeydown(event)">
  <button class="btn btn-sm btn-icon" onclick="findPrev()" title="Previous">↑</button>
  <button class="btn btn-sm btn-icon" onclick="findNext()" title="Next">↓</button>
  <span id="find-count" style="font-size:12px;color:var(--text2);min-width:50px;"></span>
  <button onclick="closeFindBar()" style="background:none;border:none;color:var(--text2);cursor:pointer;font-size:16px;padding:0 2px;">✕</button>
</div>

<!-- VERSION HISTORY PANEL -->
<div class="side-panel" id="versions-panel">
  <div class="panel-header">
    <div class="panel-title">🕐 Version History</div>
    <button class="modal-close" onclick="closeVersionsPanel()">✕</button>
  </div>
  <button class="btn btn-primary" style="width:100%;margin-bottom:16px;" onclick="saveVersion()">💾 Save Version Now</button>
  <div id="versions-list" style="display:flex;flex-direction:column;gap:8px;"></div>
</div>

<!-- SETTINGS MODAL -->
<div class="modal-overlay" id="settings-modal">
  <div class="modal">
    <div class="modal-header">
      <div class="modal-title">⚙️ Settings</div>
      <button class="modal-close" onclick="closeSettings()">✕</button>
    </div>
    <div class="modal-body">
      <div class="form-group">
        <label class="form-label">Theme</label>
        <select class="form-select" id="pref-theme">
          <option value="light">☀️ Light (Default)</option>
          <option value="dark">🌙 Dark</option>
        </select>
      </div>
      <div class="form-group">
        <label class="form-label">UI Font Size</label>
        <select class="form-select" id="pref-fontsize">
          <option value="13">Small (13px)</option>
          <option value="14" selected>Default (14px)</option>
          <option value="16">Large (16px)</option>
          <option value="18">Extra Large (18px)</option>
        </select>
      </div>
      <div class="form-group">
        <label class="form-label">Auto-save Interval</label>
        <select class="form-select" id="pref-autosave">
          <option value="1000">Every second</option>
          <option value="2000" selected>Every 2 seconds</option>
          <option value="5000">Every 5 seconds</option>
          <option value="0">Manual only</option>
        </select>
      </div>
    </div>
    <div class="modal-footer">
      <button class="btn" onclick="closeSettings()">Cancel</button>
      <button class="btn btn-primary" onclick="saveSettings()">Save Settings</button>
    </div>
  </div>
</div>

<!-- HELP MODAL -->
<div class="modal-overlay" id="help-modal">
  <div class="modal" style="width:600px;max-height:80vh;overflow-y:auto;">
    <div class="modal-header">
      <div class="modal-title">❓ Help & Keyboard Shortcuts</div>
      <button class="modal-close" onclick="closeHelp()">✕</button>
    </div>
    <div class="modal-body">
      <div style="display:grid;grid-template-columns:1fr 1fr;gap:20px;">
        <div>
          <p style="font-weight:700;margin-bottom:10px;color:var(--primary);">📝 Writer</p>
          <table style="width:100%;font-size:13px;border-collapse:collapse;">
            <tr><td style="padding:4px 0;"><kbd style="background:var(--surface2);border:1px solid var(--border);padding:2px 6px;border-radius:4px;">Ctrl+S</kbd></td><td style="padding:4px 8px;">Save document</td></tr>
            <tr><td><kbd style="background:var(--surface2);border:1px solid var(--border);padding:2px 6px;border-radius:4px;">Ctrl+B</kbd></td><td style="padding:4px 8px;">Bold text</td></tr>
            <tr><td><kbd style="background:var(--surface2);border:1px solid var(--border);padding:2px 6px;border-radius:4px;">Ctrl+I</kbd></td><td style="padding:4px 8px;">Italic text</td></tr>
            <tr><td><kbd style="background:var(--surface2);border:1px solid var(--border);padding:2px 6px;border-radius:4px;">Ctrl+U</kbd></td><td style="padding:4px 8px;">Underline</td></tr>
            <tr><td><kbd style="background:var(--surface2);border:1px solid var(--border);padding:2px 6px;border-radius:4px;">Ctrl+F</kbd></td><td style="padding:4px 8px;">Find in document</td></tr>
            <tr><td><kbd style="background:var(--surface2);border:1px solid var(--border);padding:2px 6px;border-radius:4px;">Ctrl+P</kbd></td><td style="padding:4px 8px;">Print document</td></tr>
          </table>
        </div>
        <div>
          <p style="font-weight:700;margin-bottom:10px;color:var(--sheets-color);">📊 Sheets</p>
          <table style="width:100%;font-size:13px;border-collapse:collapse;">
            <tr><td style="padding:4px 0;"><kbd style="background:var(--surface2);border:1px solid var(--border);padding:2px 6px;border-radius:4px;">Enter</kbd></td><td style="padding:4px 8px;">Confirm & move down</td></tr>
            <tr><td><kbd style="background:var(--surface2);border:1px solid var(--border);padding:2px 6px;border-radius:4px;">Tab</kbd></td><td style="padding:4px 8px;">Move right</td></tr>
            <tr><td><kbd style="background:var(--surface2);border:1px solid var(--border);padding:2px 6px;border-radius:4px;">Arrow Keys</kbd></td><td style="padding:4px 8px;">Navigate cells</td></tr>
            <tr><td><kbd style="background:var(--surface2);border:1px solid var(--border);padding:2px 6px;border-radius:4px;">Delete</kbd></td><td style="padding:4px 8px;">Clear cell</td></tr>
            <tr><td><kbd style="background:var(--surface2);border:1px solid var(--border);padding:2px 6px;border-radius:4px;">=SUM(…)</kbd></td><td style="padding:4px 8px;">Enter a formula</td></tr>
          </table>
        </div>
      </div>
      <div style="margin-top:16px;padding:12px;background:var(--primary-light);border-radius:8px;">
        <p style="font-size:13px;font-weight:600;color:var(--primary);margin-bottom:6px;">💡 Tips</p>
        <ul style="font-size:13px;padding-left:16px;display:flex;flex-direction:column;gap:4px;">
          <li>Drag & drop .docx, .xlsx, .pptx files onto the home screen to open them instantly</li>
          <li>Click "📤 Share" to export a .pdfo file you can send to a friend</li>
          <li>All your documents auto-save every 2 seconds</li>
          <li>Use "💾 Save Version Now" in History to create a restore point</li>
        </ul>
      </div>
    </div>
    <div class="modal-footer">
      <button class="btn btn-primary" onclick="closeHelp()">Got it!</button>
    </div>
  </div>
</div>

<!-- TOAST CONTAINER -->
<div class="toast-container" id="toast-container"></div>

<script>
// ─── App State ────────────────────────────────────────────────────────────────
const state = {
  section: 'home',
  docId: null,
  ws: null,
  cell: { row: 0, col: 0 },
  grid: {},     // "row,col" → { value, formula }
  scrollX: 0, scrollY: 0,
  slide: 0,
  slides: [{ elements: [], bg: '#FFFFFF' }],
  findMatches: [], findIdx: 0,
  saveTimer: null,
  wordTimer: null,
  spellTimer: null,
  autosaveDelay: 2000,
  theme: 'light',
};

const FORMULA_HINTS = [
  'SUM','AVERAGE','COUNT','COUNTA','MAX','MIN','IF','AND','OR','NOT',
  'VLOOKUP','HLOOKUP','INDEX','MATCH','CONCATENATE','LEN','LEFT','RIGHT','MID',
  'TRIM','UPPER','LOWER','NOW','TODAY','ROUND','ABS','SQRT','POWER','MOD',
  'IFERROR','SUMIF','COUNTIF','AVERAGEIF','DATE','YEAR','MONTH','DAY',
  'TEXT','VALUE','ISBLANK','ISNUMBER','ISTEXT','COERCE',
];

// ─── Theme ────────────────────────────────────────────────────────────────────
function applyTheme(theme) {
  state.theme = theme;
  document.documentElement.setAttribute('data-theme', theme === 'dark' ? 'dark' : '');
  localStorage.setItem('theme', theme);
}

function toggleTheme() {
  applyTheme(state.theme === 'dark' ? 'light' : 'dark');
  toast('Switched to ' + (state.theme === 'dark' ? '🌙 Dark' : '☀️ Light') + ' mode');
}

// ─── Navigation ───────────────────────────────────────────────────────────────
function goHome() {
  setSection('home');
  document.getElementById('ribbon-tabs').style.display = 'none';
  document.getElementById('doc-name-bar').style.display = 'none';
  document.getElementById('btn-tbar-save').style.display = 'none';
  document.getElementById('ribbon-writer-home').style.display = 'none';
  document.getElementById('ribbon-sheets-home').style.display = 'none';
  document.getElementById('ribbon-slides-home').style.display = 'none';
  document.getElementById('ribbon-pdf-home').style.display = 'none';
}

function setSection(name) {
  ['home','writer','sheets','slides','pdf'].forEach(s => {
    const el = document.getElementById('section-' + s);
    if(el) el.classList.toggle('active', s === name);
    const nav = document.getElementById('nav-' + s);
    if(nav) nav.classList.toggle('active', s === name);
  });
  state.section = name;
  if(name === 'sheets') setTimeout(drawGrid, 50);
}

function openApp(app) {
  setSection(app);
  // Show ribbon
  document.getElementById('ribbon-tabs').style.display = 'flex';
  document.getElementById('doc-name-bar').style.display = 'flex';
  document.getElementById('btn-tbar-save').style.display = '';
  // Show appropriate ribbon
  ['writer','sheets','slides','pdf'].forEach(a => {
    document.getElementById('ribbon-' + a + '-home').style.display = a === app ? '' : 'none';
  });
  if(app === 'writer' && !state.docId) newDocument('writer');
  if(app === 'sheets' && !state.docId) newDocument('sheets');
}

// ─── Ribbon Tabs ──────────────────────────────────────────────────────────────
function switchRibbonTab(tab) {
  document.querySelectorAll('.rtab').forEach(el => el.classList.remove('active'));
  document.getElementById('rtab-' + tab).classList.add('active');
  // For now all tabs show same ribbon
}

// ─── Documents ────────────────────────────────────────────────────────────────
async function newDocument(type) {
  try {
    const resp = await fetch('/api/documents', { method: 'POST' });
    const data = await resp.json();
    state.docId = data.id;
    document.getElementById('doc-name').value = 'Untitled ' + (type === 'writer' ? 'Document' : type === 'sheets' ? 'Spreadsheet' : 'Presentation');
    openApp(type);
    loadRecent();
    toast('New ' + (type === 'writer' ? 'document' : type === 'sheets' ? 'spreadsheet' : 'presentation') + ' created ✓', 'success');
  } catch(e) { toast('Error creating document: ' + e.message, 'error'); }
}

async function renameDoc(name) {
  // title is updated on next save
}

function quickSave() {
  if(state.section === 'writer') saveDocument();
  else toast('Document auto-saved ✓', 'success');
}

async function loadRecent() {
  try {
    const resp = await fetch('/api/documents');
    const docs = await resp.json();
    const grid = document.getElementById('recent-grid');
    const count = document.getElementById('recent-count');
    if(!docs.length) {
      grid.innerHTML = '<div class="empty-state"><div style="font-size:48px;">📂</div><p>No documents yet. Create your first one above!</p><p style="font-size:13px;margin-top:8px;">Or drag &amp; drop a file anywhere on this page.</p></div>';
      count.textContent = '';
      return;
    }
    count.textContent = '(' + docs.length + ')';
    grid.innerHTML = docs.map(d => {
      const icon = d.title?.includes('Sheet') ? '📊' : d.title?.includes('Slide') || d.title?.includes('Present') ? '📽️' : '📝';
      const date = new Date(d.updated_at || d.created_at || Date.now()).toLocaleDateString();
      return `<div class="recent-card" onclick="openDoc('${d.id}')">
        <div class="recent-card-icon">${icon}</div>
        <div class="recent-card-name" title="${d.title}">${d.title || 'Untitled'}</div>
        <div class="recent-card-date">${date}</div>
        <button class="recent-card-del" onclick="event.stopPropagation();deleteDoc('${d.id}')" title="Delete">✕</button>
      </div>`;
    }).join('');
  } catch(e) { console.error('loadRecent', e); }
}

async function openDoc(id) {
  state.docId = id;
  openApp('writer');
  try {
    const resp = await fetch(`/api/documents/${id}`);
    const doc = await resp.json();
    document.getElementById('doc-name').value = doc.title || 'Untitled';
    const text = doc.body?.map(b => b.Paragraph?.runs?.map(r => r.text).join('') || '').join('\n') || '';
    document.getElementById('doc-editor').innerText = text;
  } catch(e) { toast('Error opening document', 'error'); }
}

async function deleteDoc(id) {
  if(!confirm('Delete this document? This cannot be undone.')) return;
  try {
    await fetch(`/api/documents/${id}`, { method: 'DELETE' });
    toast('Document deleted', 'info');
    loadRecent();
  } catch(e) { toast('Error deleting document', 'error'); }
}

// ─── Writer ───────────────────────────────────────────────────────────────────
function writerBold() { document.execCommand('bold'); }
function writerItalic() { document.execCommand('italic'); }
function writerUnderline() { document.execCommand('underline'); }
function writerAlign(dir) { document.execCommand('justify' + dir.charAt(0).toUpperCase() + dir.slice(1)); }
function insertBulletList() { document.execCommand('insertUnorderedList'); }
function insertNumberedList() { document.execCommand('insertOrderedList'); }
function insertHR() { document.execCommand('insertHorizontalRule'); }
function applyFont() {
  const fam = document.getElementById('font-family').value;
  document.execCommand('fontName', false, fam);
}
function applyFontSize() {
  const sz = document.getElementById('font-size').value;
  document.getElementById('doc-editor').style.fontSize = sz + 'pt';
}

function onDocumentInput() {
  clearTimeout(state.saveTimer);
  state.saveTimer = setTimeout(saveDocument, state.autosaveDelay);
  clearTimeout(state.spellTimer);
  state.spellTimer = setTimeout(doSpellCheck, 2000);
  clearTimeout(state.wordTimer);
  state.wordTimer = setTimeout(updateWordCount, 300);
}

function updateWordCount() {
  const text = document.getElementById('doc-editor')?.innerText || '';
  const words = text.trim() === '' ? 0 : text.trim().split(/\s+/).length;
  const chars = text.length;
  const wcEl = document.getElementById('word-count');
  const ccEl = document.getElementById('char-count');
  if(wcEl) wcEl.textContent = words + ' words';
  if(ccEl) ccEl.textContent = chars + ' chars';
}

async function doSpellCheck() {
  const editor = document.getElementById('doc-editor');
  if(!editor) return;
  const text = editor.innerText;
  if(!text.trim()) return;
  const statusEl = document.getElementById('spell-status');
  if(statusEl) statusEl.textContent = 'Checking...';
  try {
    const resp = await fetch('/api/spell', {
      method: 'POST', headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ text })
    });
    const data = await resp.json();
    const count = data.results?.length || 0;
    if(statusEl) {
      statusEl.textContent = count === 0 ? '✓ No errors' : count + ' issue' + (count > 1 ? 's' : '');
      statusEl.style.color = count === 0 ? 'var(--sheets-color)' : 'var(--pdf-color)';
    }
  } catch(e) {
    if(statusEl) statusEl.textContent = 'Spell check unavailable';
  }
}

async function saveDocument() {
  if(!state.docId) return;
  const editor = document.getElementById('doc-editor');
  const text = editor.innerText;
  const title = document.getElementById('doc-name').value || text.split('\n')[0]?.slice(0,80) || 'Untitled';
  const doc = {
    id: state.docId, title,
    body: [{ Paragraph: { id: crypto.randomUUID(), runs: [{ id: crypto.randomUUID(), text, format: {} }], style_ref: null, align: 'Left', indent_left: 0, indent_right: 0, indent_first: 0, space_before: 0, space_after: 8, line_height: { Multiple: 1.15 }, keep_together: false, keep_with_next: false, page_break_before: false } }],
    styles: { styles: {}, default_paragraph_style: '', default_character_style: '' },
    page_layout: { width: 210, height: 297, margin_top: 25.4, margin_bottom: 25.4, margin_left: 25.4, margin_right: 25.4, orientation: 'Portrait' },
    metadata: { author: '', language: 'en', keywords: [], description: '' },
    revision: 0
  };
  try {
    await fetch(`/api/documents/${state.docId}`, {
      method: 'POST', headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(doc)
    });
  } catch(e) {}
}

async function exportDoc(format) {
  if(!state.docId) { toast('No document open', 'error'); return; }
  await saveDocument();
  toast('Exporting as ' + format.toUpperCase() + '...');
  try {
    const resp = await fetch(`/api/documents/${state.docId}/export`, {
      method: 'POST', headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ format })
    });
    if(!resp.ok) { toast('Export failed: ' + await resp.text(), 'error'); return; }
    const blob = await resp.blob();
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url; a.download = 'document.' + format;
    document.body.appendChild(a); a.click(); document.body.removeChild(a);
    setTimeout(() => URL.revokeObjectURL(url), 60000);
    toast(format.toUpperCase() + ' exported ✓', 'success');
  } catch(e) { toast('Export failed: ' + e.message, 'error'); }
}

async function shareDoc() {
  if(!state.docId) { toast('No document open', 'error'); return; }
  await saveDocument();
  toast('Preparing .pdfo share file…');
  try {
    const resp = await fetch(`/api/documents/${state.docId}/export`, {
      method: 'POST', headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ format: 'pdfo' })
    });
    if(!resp.ok) { toast('Share failed: ' + await resp.text(), 'error'); return; }
    const blob = await resp.blob();
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url; a.download = 'document.pdfo';
    document.body.appendChild(a); a.click(); document.body.removeChild(a);
    setTimeout(() => URL.revokeObjectURL(url), 60000);
    toast('Share file (.pdfo) ready! Send it to your friend 📤', 'success');
  } catch(e) { toast('Share failed: ' + e.message, 'error'); }
}

function handleWriterKey(e) {
  if(e.ctrlKey && e.key === 's') { e.preventDefault(); saveDocument(); toast('Saved! ✓', 'success'); }
  if(e.ctrlKey && e.key === 'b') { e.preventDefault(); writerBold(); }
  if(e.ctrlKey && e.key === 'i') { e.preventDefault(); writerItalic(); }
  if(e.ctrlKey && e.key === 'u') { e.preventDefault(); writerUnderline(); }
  if(e.ctrlKey && e.key === 'f') { e.preventDefault(); openFindBar(); }
  if(e.ctrlKey && e.key === 'p') { e.preventDefault(); window.print(); }
}

// ─── Find Bar ────────────────────────────────────────────────────────────────
function openFindBar() {
  const bar = document.getElementById('find-bar');
  bar.classList.add('open');
  bar.style.display = 'flex';
  document.getElementById('find-input').focus();
}
function closeFindBar() {
  const bar = document.getElementById('find-bar');
  bar.classList.remove('open');
  bar.style.display = 'none';
}
function findInDoc() {
  const q = document.getElementById('find-input').value;
  state.findMatches = [];
  state.findIdx = 0;
  const count = document.getElementById('find-count');
  if(!q) { count.textContent = ''; return; }
  const found = document.body.innerText.split(q).length - 1;
  count.textContent = found + ' found';
}
function findNext() { window.find(document.getElementById('find-input').value, false, false, true); }
function findPrev() { window.find(document.getElementById('find-input').value, false, true, true); }
function findKeydown(e) {
  if(e.key === 'Enter') { e.shiftKey ? findPrev() : findNext(); }
  if(e.key === 'Escape') closeFindBar();
}

// ─── Spreadsheet Grid ─────────────────────────────────────────────────────────
const COL_W = 100, ROW_H = 26, HEAD_W = 52, HEAD_H = 26;

function col2Letter(n) {
  let s = '';
  for(; n >= 0; n = Math.floor(n/26) - 1) s = String.fromCharCode(65 + n % 26) + s;
  return s;
}

function drawGrid() {
  const canvas = document.getElementById('grid-canvas');
  if(!canvas) return;
  const par = canvas.parentElement;
  canvas.width = par.clientWidth;
  canvas.height = par.clientHeight;
  const ctx = canvas.getContext('2d');
  const isDark = state.theme === 'dark';
  const bgColor = isDark ? '#242830' : '#ffffff';
  const headerBg = isDark ? '#2c3140' : '#f8f9fa';
  const gridLine = isDark ? '#3a3f4b' : '#e1e4e8';
  const headerText = isDark ? '#9aa5b4' : '#6b7280';
  const cellText = isDark ? '#e8ecf1' : '#1a1d23';
  const selColor = 'rgba(30,111,254,0.12)';
  const selBorder = '#1e6ffe';

  const cols = Math.ceil((canvas.width - HEAD_W) / COL_W) + 2;
  const rows = Math.ceil((canvas.height - HEAD_H) / ROW_H) + 2;

  ctx.fillStyle = bgColor; ctx.fillRect(0, 0, canvas.width, canvas.height);

  // Headers
  for(let c = 0; c < cols; c++) {
    const x = HEAD_W + c * COL_W;
    const col = c + state.scrollX;
    const sel = col === state.cell.col;
    ctx.fillStyle = sel ? 'rgba(30,111,254,0.15)' : headerBg;
    ctx.fillRect(x, 0, COL_W, HEAD_H);
    ctx.fillStyle = sel ? '#1e6ffe' : headerText;
    ctx.font = '600 12px Inter, sans-serif';
    ctx.textAlign = 'center'; ctx.textBaseline = 'middle';
    ctx.fillText(col2Letter(col), x + COL_W/2, HEAD_H/2);
    ctx.strokeStyle = gridLine; ctx.lineWidth = 1;
    ctx.beginPath(); ctx.moveTo(x, 0); ctx.lineTo(x, HEAD_H); ctx.stroke();
  }
  for(let r = 0; r < rows; r++) {
    const y = HEAD_H + r * ROW_H;
    const row = r + state.scrollY;
    const sel = row === state.cell.row;
    ctx.fillStyle = sel ? 'rgba(30,111,254,0.15)' : headerBg;
    ctx.fillRect(0, y, HEAD_W, ROW_H);
    ctx.fillStyle = sel ? '#1e6ffe' : headerText;
    ctx.font = '12px Inter, sans-serif';
    ctx.textAlign = 'center'; ctx.textBaseline = 'middle';
    ctx.fillText(row + 1, HEAD_W/2, y + ROW_H/2);
    ctx.strokeStyle = gridLine;
    ctx.beginPath(); ctx.moveTo(0, y); ctx.lineTo(HEAD_W, y); ctx.stroke();
  }

  // Cells
  for(let r = 0; r < rows; r++) {
    const y = HEAD_H + r * ROW_H;
    const row = r + state.scrollY;
    for(let c = 0; c < cols; c++) {
      const x = HEAD_W + c * COL_W;
      const col = c + state.scrollX;
      const key = row + ',' + col;
      const cell = state.grid[key];
      const isSel = row === state.cell.row && col === state.cell.col;
      ctx.fillStyle = isSel ? selColor : bgColor;
      ctx.fillRect(x+1, y+1, COL_W-1, ROW_H-1);
      ctx.strokeStyle = gridLine; ctx.lineWidth = 1;
      ctx.strokeRect(x, y, COL_W, ROW_H);
      if(cell?.value !== undefined && cell.value !== '') {
        ctx.fillStyle = cellText;
        ctx.font = '13px Inter, sans-serif';
        ctx.textAlign = 'left'; ctx.textBaseline = 'middle';
        ctx.fillText(String(cell.value).slice(0,14), x + 5, y + ROW_H/2);
      }
      if(isSel) {
        ctx.strokeStyle = selBorder; ctx.lineWidth = 2;
        ctx.strokeRect(x+1, y+1, COL_W-2, ROW_H-2);
        ctx.lineWidth = 1;
      }
    }
  }

  // Corner
  ctx.fillStyle = headerBg; ctx.fillRect(0, 0, HEAD_W, HEAD_H);
  ctx.strokeStyle = gridLine; ctx.strokeRect(0, 0, HEAD_W, HEAD_H);

  document.getElementById('cell-ref').value = col2Letter(state.cell.col) + (state.cell.row+1);
  const ck = state.cell.row + ',' + state.cell.col;
  document.getElementById('formula-bar').value = state.grid[ck]?.formula || state.grid[ck]?.value || '';
}

function gridMousedown(e) {
  const canvas = document.getElementById('grid-canvas');
  const rect = canvas.getBoundingClientRect();
  const x = e.clientX - rect.left, y = e.clientY - rect.top;
  if(x < HEAD_W || y < HEAD_H) return;
  state.cell.col = Math.floor((x - HEAD_W) / COL_W) + state.scrollX;
  state.cell.row = Math.floor((y - HEAD_H) / ROW_H) + state.scrollY;
  canvas.focus(); drawGrid();
}

function gridKeydown(e) {
  const { row, col } = state.cell;
  if(e.key === 'ArrowRight') { state.cell.col = col+1; drawGrid(); }
  else if(e.key === 'ArrowLeft') { state.cell.col = Math.max(0, col-1); drawGrid(); }
  else if(e.key === 'ArrowDown' || e.key === 'Enter') { state.cell.row = row+1; drawGrid(); }
  else if(e.key === 'ArrowUp') { state.cell.row = Math.max(0, row-1); drawGrid(); }
  else if(e.key === 'Tab') { e.preventDefault(); state.cell.col = col+1; drawGrid(); }
  else if(e.key === 'Delete' || e.key === 'Backspace') {
    delete state.grid[row + ',' + col];
    document.getElementById('formula-bar').value = '';
    drawGrid();
  } else if(e.key.length === 1 && !e.ctrlKey && !e.metaKey) {
    // Start typing in formula bar
    const fb = document.getElementById('formula-bar');
    fb.value = e.key;
    fb.focus();
  }
}

function clearCell() {
  const key = state.cell.row + ',' + state.cell.col;
  delete state.grid[key];
  document.getElementById('formula-bar').value = '';
  drawGrid();
}

async function evalFormulaBar() {
  const formula = document.getElementById('formula-bar').value.trim();
  if(!formula) return;
  const key = state.cell.row + ',' + state.cell.col;
  if(formula.startsWith('=')) {
    try {
      const resp = await fetch('/api/formula', {
        method: 'POST', headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ formula, row: state.cell.row, col: state.cell.col })
      });
      const data = await resp.json();
      if(data.error) {
        state.grid[key] = { value: '#ERR', formula };
        document.getElementById('formula-result').textContent = '⚠ ' + data.error;
        document.getElementById('formula-result').style.color = 'var(--pdf-color)';
      } else {
        state.grid[key] = { value: data.result, formula };
        document.getElementById('formula-result').textContent = '= ' + data.result;
        document.getElementById('formula-result').style.color = 'var(--sheets-color)';
      }
    } catch(e) { toast('Formula error: ' + e.message, 'error'); }
  } else {
    state.grid[key] = { value: formula };
    document.getElementById('formula-result').textContent = formula;
  }
  state.cell.row++;
  drawGrid();
  document.getElementById('grid-canvas').focus();
}

function formulaKeydown(e) {
  if(e.key === 'Enter') { e.preventDefault(); evalFormulaBar(); }
  if(e.key === 'Escape') { document.getElementById('formula-bar').value = ''; document.getElementById('grid-canvas').focus(); }
  if(e.key === 'Tab') { e.preventDefault(); evalFormulaBar(); }
}

function formulaInput(e) {
  const val = document.getElementById('formula-bar').value;
  const ac = document.getElementById('formula-autocomplete');
  if(!val.startsWith('=')) { ac.style.display = 'none'; return; }
  const query = val.slice(1).toUpperCase().replace(/[^A-Z]/,'');
  if(!query) { ac.style.display = 'none'; return; }
  const matches = FORMULA_HINTS.filter(f => f.startsWith(query));
  if(!matches.length) { ac.style.display = 'none'; return; }
  ac.style.display = 'block';
  ac.innerHTML = matches.slice(0,12).map(f =>
    `<div style="padding:8px 12px;cursor:pointer;font-size:13px;font-family:monospace;" onmousedown="insertFn('${f}')" onmouseover="this.style.background='var(--primary-light)'" onmouseout="this.style.background=''">${f}()</div>`
  ).join('');
  const input = document.getElementById('formula-bar');
  const rect = input.getBoundingClientRect();
  ac.style.top = (rect.bottom + 2) + 'px';
  ac.style.left = rect.left + 'px';
}

function insertFn(fn) {
  document.getElementById('formula-bar').value = '=' + fn + '(';
  document.getElementById('formula-autocomplete').style.display = 'none';
  document.getElementById('formula-bar').focus();
}

function insertFormulaFunc(fn) {
  document.getElementById('formula-bar').value = '=' + fn + '(';
  document.getElementById('formula-bar').focus();
}

function addSheet() { toast('Multiple sheets will be available in a future update!'); }

async function exportXlsx() {
  if(!state.docId) { toast('No spreadsheet open — create one first', 'error'); return; }
  toast('Exporting XLSX…');
  try {
    const resp = await fetch(`/api/documents/${state.docId}/export`, {
      method: 'POST', headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ format: 'xlsx' })
    });
    if(!resp.ok) { toast('Export failed: ' + await resp.text(), 'error'); return; }
    const blob = await resp.blob();
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url; a.download = 'spreadsheet.xlsx';
    document.body.appendChild(a); a.click(); document.body.removeChild(a);
    setTimeout(() => URL.revokeObjectURL(url), 60000);
    toast('XLSX exported ✓', 'success');
  } catch(e) { toast('Export error: ' + e.message, 'error'); }
}

// ─── Slides ───────────────────────────────────────────────────────────────────
function selectSlide(idx) {
  state.slide = idx;
  document.querySelectorAll('.slide-thumb').forEach((el, i) => el.classList.toggle('selected', i === idx));
  renderSlide();
}

function renderSlide() {
  const slide = state.slides[state.slide];
  const content = document.getElementById('slide-content');
  if(!slide) return;
  content.style.background = slide.bg || '#fff';
  content.innerHTML = (slide.elements || []).map((el, i) => {
    if(el.type === 'text') {
      return `<div contenteditable="true" style="position:absolute;left:${el.x||40}px;top:${el.y||40}px;width:${el.w||640}px;min-height:40px;font-size:${el.size||24}px;font-weight:${el.bold?'700':'400'};outline:none;border:1px dashed transparent;padding:4px;" onmousedown="slideDrag(event,${i})" onfocus="this.style.borderColor='#1e6ffe'" onblur="this.style.borderColor='transparent';saveSlideElement(${i},this)">${el.text||'Click to edit text'}</div>`;
    }
    return '';
  }).join('');
}

function saveSlideElement(idx, el) {
  if(state.slides[state.slide]?.elements[idx]) {
    state.slides[state.slide].elements[idx].text = el.innerText;
  }
}

function addSlide() {
  state.slides.push({ elements: [], bg: '#ffffff' });
  const panel = document.getElementById('slides-panel');
  const idx = state.slides.length - 1;
  const thumb = document.createElement('div');
  thumb.className = 'slide-thumb'; thumb.id = 'slide-thumb-' + idx;
  thumb.onclick = () => selectSlide(idx);
  thumb.innerHTML = `<div style="font-size:11px;color:#888;">Slide ${idx+1}</div><div class="slide-thumb-num">${idx+1}</div>`;
  panel.appendChild(thumb);
  selectSlide(idx);
  toast('New slide added');
}

function addTextBox() {
  if(!state.slides[state.slide]) return;
  state.slides[state.slide].elements.push({ type:'text', text:'Click to edit', x:40, y:40, w:640, size:28, bold:false });
  renderSlide();
}

function addShape() { toast('Shape tools coming soon!'); }
function addImage() { toast('Image insert coming soon!'); }
function slideCanvasClick(e) {}

async function exportPptx() {
  if(!state.docId) { toast('No presentation open', 'error'); return; }
  toast('Exporting PPTX…');
  try {
    const resp = await fetch(`/api/documents/${state.docId}/export`, {
      method: 'POST', headers: { 'Content-Type': 'application/json'},
      body: JSON.stringify({ format: 'pptx' })
    });
    if(!resp.ok) { toast('Export failed: ' + await resp.text(), 'error'); return; }
    const blob = await resp.blob();
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url; a.download = 'presentation.pptx';
    document.body.appendChild(a); a.click(); document.body.removeChild(a);
    setTimeout(() => URL.revokeObjectURL(url), 60000);
    toast('PPTX exported ✓', 'success');
  } catch(e) { toast('Export error: ' + e.message, 'error'); }
}

// ─── PDF Tools ────────────────────────────────────────────────────────────────
function openPDF(event) {
  const file = event.target.files[0];
  if(!file) return;
  const reader = new FileReader();
  reader.onload = (e) => {
    const viewer = document.getElementById('pdf-viewer');
    viewer.innerHTML = `<div style="text-align:center;padding:32px;color:var(--text2);"><div style="font-size:64px;">📄</div><p style="font-size:18px;font-weight:600;margin:12px 0;">${file.name}</p><p style="font-size:13px;">${(file.size/1024).toFixed(1)} KB</p><p style="margin-top:16px;font-size:13px;">PDF rendering via native viewer. Use tools in the toolbar to merge, split, or watermark.</p></div>`;
    document.getElementById('pdf-pages').textContent = file.name;
  };
  reader.readAsArrayBuffer(file);
}

function handleDropPDF(e) {
  e.preventDefault();
  const file = e.dataTransfer.files[0];
  if(file && file.name.endsWith('.pdf')) {
    const fakeEvent = { target: { files: [file] } };
    openPDF(fakeEvent);
  }
}

async function mergePDFs() { toast('Select two PDF files to merge', 'info'); }
async function splitPDF() { toast('Split feature coming in next update!'); }
async function watermarkPDF() { toast('Watermark feature coming in next update!'); }

// ─── Drag & Drop ──────────────────────────────────────────────────────────────
function handleDragover(e) {
  e.preventDefault();
  e.dataTransfer.dropEffect = 'copy';
}

async function handleDrop(e) {
  e.preventDefault();
  const files = [...e.dataTransfer.files];
  for(const file of files) {
    const ext = file.name.split('.').pop().toLowerCase();
    if(['docx','odt','rtf'].includes(ext)) await importFile(file, 'writer');
    else if(['xlsx','csv'].includes(ext)) await importFile(file, 'sheets');
    else if(['pptx'].includes(ext)) await importFile(file, 'slides');
    else if(ext === 'pdfo') await importPdfo(file);
    else if(ext === 'pdf') {
      const fakeEvent = { target: { files: [file] } };
      openApp('pdf'); openPDF(fakeEvent);
    } else toast('Unsupported file type: .' + ext, 'error');
  }
}

async function importFile(file, app) {
  toast('Importing ' + file.name + '…');
  try {
    const form = new FormData();
    form.append('file', file);
    const resp = await fetch('/api/import', { method: 'POST', body: form });
    if(resp.ok) {
      const doc = await resp.json();
      state.docId = doc.id;
      document.getElementById('doc-name').value = doc.title || file.name;
      openApp(app);
      if(app === 'writer') {
        const text = doc.body?.map(b => b.Paragraph?.runs?.map(r => r.text).join('')||'').join('\n')||'';
        document.getElementById('doc-editor').innerText = text;
      }
      toast(file.name + ' imported ✓', 'success');
      loadRecent();
    } else { toast('Import failed: ' + await resp.text(), 'error'); }
  } catch(e) { toast('Import error: ' + e.message, 'error'); }
}

async function importPdfo(file) {
  toast('Importing .pdfo file…');
  try {
    const form = new FormData();
    form.append('file', file);
    const resp = await fetch('/api/import', { method: 'POST', body: form });
    if(resp.ok) {
      const doc = await resp.json();
      state.docId = doc.id;
      openApp('writer');
      toast('.pdfo imported ✓', 'success');
      loadRecent();
    } else toast('Import failed', 'error');
  } catch(e) { toast('Import error: ' + e.message, 'error'); }
}

// ─── Version History ──────────────────────────────────────────────────────────
function openVersionsPanel() {
  document.getElementById('versions-panel').classList.add('open');
  loadVersions();
}
function closeVersionsPanel() {
  document.getElementById('versions-panel').classList.remove('open');
}

async function loadVersions() {
  if(!state.docId) return;
  try {
    const resp = await fetch(`/api/documents/${state.docId}/versions`);
    const versions = await resp.json();
    const list = document.getElementById('versions-list');
    if(!versions.length) { list.innerHTML = '<p style="font-size:13px;color:var(--text3);">No saved versions yet.</p>'; return; }
    list.innerHTML = versions.map(v => `
      <div class="version-item">
        <div class="version-label">📌 ${v.label || 'Version'}</div>
        <div class="version-date">${new Date(v.created_at).toLocaleString()}</div>
        <button class="btn btn-sm" onclick="restoreVersion('${v.id}')">↩ Restore</button>
      </div>
    `).join('');
  } catch(e) { document.getElementById('versions-list').innerHTML = '<p style="font-size:13px;color:var(--text3);">Could not load versions.</p>'; }
}

async function saveVersion() {
  if(!state.docId) { toast('Open a document first', 'error'); return; }
  const label = prompt('Name this version (e.g. "Draft v1"):', 'Version ' + new Date().toLocaleTimeString());
  if(!label) return;
  await saveDocument();
  try {
    const resp = await fetch(`/api/documents/${state.docId}/versions`, {
      method: 'POST', headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ label, created_by: 'me' })
    });
    if(resp.ok) { toast('Version saved: ' + label, 'success'); loadVersions(); }
    else toast('Save version failed', 'error');
  } catch(e) { toast('Error: ' + e.message, 'error'); }
}

async function restoreVersion(vid) {
  if(!confirm('Restore this version? Current unsaved changes will be lost.')) return;
  try {
    const resp = await fetch(`/api/documents/${state.docId}/versions/${vid}/restore`, { method: 'POST' });
    if(resp.ok) {
      await openDoc(state.docId);
      toast('Restored ✓', 'success');
      closeVersionsPanel();
    } else toast('Restore failed', 'error');
  } catch(e) { toast('Error: ' + e.message, 'error'); }
}

// ─── Settings ─────────────────────────────────────────────────────────────────
function openSettings() { document.getElementById('settings-modal').classList.add('open'); }
function closeSettings() { document.getElementById('settings-modal').classList.remove('open'); }

async function saveSettings() {
  const theme = document.getElementById('pref-theme').value;
  const fontSize = document.getElementById('pref-fontsize').value;
  const autosave = document.getElementById('pref-autosave').value;
  state.autosaveDelay = parseInt(autosave);
  applyTheme(theme);
  document.body.style.fontSize = fontSize + 'px';
  try {
    await fetch('/api/preferences', {
      method: 'PUT', headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ theme, font_size: parseInt(fontSize), autosave_delay: parseInt(autosave) })
    });
  } catch(e) {}
  closeSettings();
  toast('Settings saved ✓', 'success');
  if(typeof drawGrid !== 'undefined') drawGrid();
}

// Global key handler
document.addEventListener('keydown', (e) => {
  if(e.ctrlKey && e.shiftKey && e.key === 'T') { e.preventDefault(); toggleTheme(); }
  if(e.ctrlKey && e.key === 's') { e.preventDefault(); quickSave(); }
  if(e.key === 'F1') { e.preventDefault(); showHelpModal(); }
});

// ─── Help Modal ───────────────────────────────────────────────────────────────
function showHelpModal() { document.getElementById('help-modal').classList.add('open'); }
function closeHelp() { document.getElementById('help-modal').classList.remove('open'); }

// Close modals on overlay click
['settings-modal','help-modal'].forEach(id => {
  document.getElementById(id).addEventListener('click', function(e) {
    if(e.target === this) this.classList.remove('open');
  });
});

// ─── Toast ────────────────────────────────────────────────────────────────────
function toast(msg, type = '') {
  const container = document.getElementById('toast-container');
  const el = document.createElement('div');
  el.className = 'toast' + (type ? ' ' + type : '');
  el.textContent = msg;
  container.appendChild(el);
  setTimeout(() => el.remove(), 3500);
}

// ─── Window resize ────────────────────────────────────────────────────────────
window.addEventListener('resize', () => {
  if(state.section === 'sheets') drawGrid();
});

// ─── Init ─────────────────────────────────────────────────────────────────────
document.addEventListener('DOMContentLoaded', () => {
  const savedTheme = localStorage.getItem('theme') || 'light';
  applyTheme(savedTheme);
  document.getElementById('pref-theme').value = savedTheme;
  loadRecent();
  updateWordCount();
});
</script>
</body>
</html>"##;

const DOWNLOAD_HTML: &str = r##"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Download PDF Office</title>
  <link rel="preconnect" href="https://fonts.googleapis.com">
  <link href="https://fonts.googleapis.com/css2?family=Inter:wght@300;400;500;600;700&display=swap" rel="stylesheet">
  <style>
    * { box-sizing: border-box; margin: 0; padding: 0; }
    body { font-family: 'Inter', sans-serif; background: #f0f2f5; color: #1a1d23; min-height: 100vh; }
    .header { background: #1e6ffe; color: white; padding: 24px 48px; display: flex; align-items: center; gap: 12px; }
    .header h1 { font-size: 24px; font-weight: 700; }
    .content { max-width: 900px; margin: 48px auto; padding: 0 24px; }
    h2 { font-size: 28px; font-weight: 700; text-align: center; margin-bottom: 8px; }
    .subtitle { text-align: center; color: #6b7280; margin-bottom: 48px; font-size: 16px; }
    .platforms { display: grid; grid-template-columns: repeat(3, 1fr); gap: 24px; margin-bottom: 48px; }
    .platform-card { background: white; border: 1px solid #e1e4e8; border-radius: 16px; padding: 32px 24px; text-align: center; }
    .platform-icon { font-size: 56px; margin-bottom: 16px; }
    .platform-name { font-size: 20px; font-weight: 700; margin-bottom: 8px; }
    .platform-desc { font-size: 14px; color: #6b7280; margin-bottom: 20px; }
    .download-btn { display: inline-flex; align-items: center; gap: 8px; padding: 12px 24px; background: #1e6ffe; color: white; border-radius: 10px; font-weight: 600; font-size: 14px; text-decoration: none; border: none; cursor: pointer; transition: all 0.15s; }
    .download-btn:hover { background: #1558d6; transform: translateY(-2px); }
    .section { background: white; border: 1px solid #e1e4e8; border-radius: 16px; padding: 32px; margin-bottom: 24px; }
    .section h3 { font-size: 18px; font-weight: 700; margin-bottom: 16px; display: flex; align-items: center; gap: 8px; }
    code { background: #f0f2f5; padding: 2px 8px; border-radius: 4px; font-family: monospace; font-size: 14px; }
    pre { background: #1a1d23; color: #e8ecf1; padding: 16px 20px; border-radius: 8px; overflow-x: auto; font-family: monospace; font-size: 14px; line-height: 1.6; }
    .back-btn { display: inline-flex; align-items: center; gap: 6px; color: rgba(255,255,255,0.8); text-decoration: none; font-size: 14px; transition: color 0.15s; }
    .back-btn:hover { color: white; }
  </style>
</head>
<body>
  <div class="header">
    <a href="/" class="back-btn">← Back to App</a>
    <div style="width:32px;height:32px;background:white;border-radius:8px;display:flex;align-items:center;justify-content:center;font-size:18px;">📄</div>
    <h1>PDF Office Downloads</h1>
  </div>
  <div class="content">
    <h2>Get PDF Office on Any Platform</h2>
    <p class="subtitle">Local-first • Private • No internet required after install</p>

    <div class="platforms">
      <div class="platform-card">
        <div class="platform-icon">🪟</div>
        <div class="platform-name">Windows</div>
        <div class="platform-desc">Works on Windows 10 & 11</div>
        <a href="#" class="download-btn">⬇ pdf-windows-x64.zip</a>
      </div>
      <div class="platform-card">
        <div class="platform-icon">🐧</div>
        <div class="platform-name">Linux</div>
        <div class="platform-desc">Ubuntu, Debian, Fedora, Arch</div>
        <a href="#" class="download-btn">⬇ pdf-linux-x64.tar.gz</a>
      </div>
      <div class="platform-card">
        <div class="platform-icon">🍎</div>
        <div class="platform-name">macOS</div>
        <div class="platform-desc">macOS 11 Big Sur and later</div>
        <a href="#" class="download-btn">⬇ pdf-macos-x64.tar.gz</a>
      </div>
    </div>

    <div class="section">
      <h3>🦀 Install via Cargo (Developers)</h3>
      <p style="color:#6b7280;margin-bottom:12px;">If you have Rust installed, this is the easiest way:</p>
      <pre>cargo install --path . --bin pdf</pre>
      <p style="margin-top:12px;color:#6b7280;font-size:13px;">Then just type <code>pdf</code> anywhere in your terminal to launch the app.</p>
    </div>

    <div class="section">
      <h3>👤 Share with a Friend (No GitHub Needed)</h3>
      <p style="color:#6b7280;margin-bottom:16px;">Three easy ways to share PDF Office:</p>
      <div style="display:flex;flex-direction:column;gap:12px;">
        <div style="padding:16px;background:#f0f2f5;border-radius:8px;">
          <strong>Option 1 — Share the .exe (Easiest)</strong>
          <p style="font-size:13px;color:#6b7280;margin-top:4px;">Build the app, then send the <code>pdf.exe</code> file. Your friend just double-clicks it. No Rust needed!</p>
          <pre style="margin-top:8px;">cargo build --release --bin pdf
# Then send: target\release\pdf.exe</pre>
        </div>
        <div style="padding:16px;background:#f0f2f5;border-radius:8px;">
          <strong>Option 2 — Share Source Code</strong>
          <p style="font-size:13px;color:#6b7280;margin-top:4px;">Zip the project (excluding the <code>target/</code> folder), send it. Friend unzips and runs:</p>
          <pre style="margin-top:8px;">cargo install --path . --bin pdf</pre>
        </div>
        <div style="padding:16px;background:#f0f2f5;border-radius:8px;">
          <strong>Option 3 — Share a Document (.pdfo)</strong>
          <p style="font-size:13px;color:#6b7280;margin-top:4px;">Click "📤 Share" in Writer to export a <code>.pdfo</code> file. Your friend imports it by dropping it on the home page.</p>
        </div>
      </div>
    </div>

    <div class="section">
      <h3>📋 System Requirements</h3>
      <table style="width:100%;border-collapse:collapse;font-size:14px;">
        <tr style="border-bottom:1px solid #e1e4e8;"><td style="padding:10px 0;font-weight:600;">OS</td><td style="padding:10px;">Windows 10+, Ubuntu 20.04+, macOS 11+</td></tr>
        <tr style="border-bottom:1px solid #e1e4e8;"><td style="padding:10px 0;font-weight:600;">RAM</td><td style="padding:10px;">256 MB minimum, 512 MB recommended</td></tr>
        <tr style="border-bottom:1px solid #e1e4e8;"><td style="padding:10px 0;font-weight:600;">Disk</td><td style="padding:10px;">~30 MB for the binary + documents</td></tr>
        <tr><td style="padding:10px 0;font-weight:600;">Browser</td><td style="padding:10px;">Any modern browser (Chrome, Firefox, Edge, Safari)</td></tr>
      </table>
    </div>
  </div>
</body>
</html>"##;

const APP_CSS: &str = r#"
/* PDF Office App CSS — additional styles */
.wps-ribbon-active { background: var(--ribbon-active) !important; }
"#;
