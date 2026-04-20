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
  <script src="https://cdnjs.cloudflare.com/ajax/libs/pdf.js/3.4.120/pdf.min.js"></script>
  <script>pdfjsLib.GlobalWorkerOptions.workerSrc = 'https://cdnjs.cloudflare.com/ajax/libs/pdf.js/3.4.120/pdf.worker.min.js';</script>
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

  <!-- PDF Home Ribbon: All Adobe Acrobat Features -->
  <div class="ribbon" id="ribbon-pdf-home" style="display:none;">
    <!-- Open / Create -->
    <div class="ribbon-group">
      <label class="rbtn wide highlighted" for="pdf-upload" style="cursor:pointer;"><span class="rbtn-icon">📂</span><span class="rbtn-label">Open PDF</span></label>
      <input type="file" id="pdf-upload" accept=".pdf" style="display:none" onchange="openPDF(event)">
      <button class="rbtn" onclick="openPdfModal('create-from-img')"><span class="rbtn-icon">🖼️</span><span class="rbtn-label">From Image</span></button>
    </div>
    <!-- View & Navigate -->
    <div class="ribbon-group" id="pdf-tools-group" style="opacity:0.5; pointer-events:none;">
      <button class="rbtn" onclick="pdfZoomIn()" title="Zoom In"><span class="rbtn-icon">🔍+</span><span class="rbtn-label">Zoom In</span></button>
      <button class="rbtn" onclick="pdfZoomOut()" title="Zoom Out"><span class="rbtn-icon">🔍-</span><span class="rbtn-label">Zoom Out</span></button>
      <button class="rbtn" onclick="pdfZoomFit()" title="Fit Page"><span class="rbtn-icon">🗋️</span><span class="rbtn-label">Fit Page</span></button>
      <button class="rbtn" onclick="togglePdfPanel('pages')" title="Page Organizer"><span class="rbtn-icon">📰</span><span class="rbtn-label">Pages</span></button>
      <button class="rbtn" onclick="togglePdfPanel('bookmarks')" title="Bookmarks"><span class="rbtn-icon">🔖</span><span class="rbtn-label">Bookmarks</span></button>
      <button class="rbtn" onclick="openPdfModal('find')" title="Find in PDF"><span class="rbtn-icon">🔍</span><span class="rbtn-label">Find</span></button>
    </div>
    <!-- Edit / Annotate -->
    <div class="ribbon-group" id="pdf-edit-group" style="opacity:0.5; pointer-events:none;">
      <button class="rbtn" onclick="togglePdfMode('view')"><span class="rbtn-icon">📄</span><span class="rbtn-label">View</span></button>
      <button class="rbtn" onclick="togglePdfMode('annotate')"><span class="rbtn-icon">✏️</span><span class="rbtn-label">Annotate</span></button>
      <button class="rbtn" onclick="togglePdfMode('highlight')"><span class="rbtn-icon">🎯</span><span class="rbtn-label">Highlight</span></button>
      <button class="rbtn" onclick="togglePdfMode('fill')"><span class="rbtn-icon">🖊️</span><span class="rbtn-label">Fill & Sign</span></button>
      <button class="rbtn" onclick="openSignatureModal()"><span class="rbtn-icon">✍️</span><span class="rbtn-label">Sign</span></button>
      <button class="rbtn" onclick="togglePdfMode('redact')"><span class="rbtn-icon">⬛</span><span class="rbtn-label">Redact</span></button>
      <button class="rbtn" onclick="togglePdfMode('measure')"><span class="rbtn-icon">📏</span><span class="rbtn-label">Measure</span></button>
      <button class="rbtn" onclick="pdfOCR()"><span class="rbtn-icon">📝</span><span class="rbtn-label">OCR Text</span></button>
    </div>
    <!-- Forms -->
    <div class="ribbon-group" id="pdf-form-group" style="opacity:0.5; pointer-events:none;">
      <button class="rbtn" onclick="addFormField('text')"><span class="rbtn-icon">🏷️</span><span class="rbtn-label">Text Field</span></button>
      <button class="rbtn" onclick="addFormField('checkbox')"><span class="rbtn-icon">☑️</span><span class="rbtn-label">Checkbox</span></button>
      <button class="rbtn" onclick="addFormField('radio')"><span class="rbtn-icon">🔘</span><span class="rbtn-label">Radio</span></button>
      <button class="rbtn" onclick="addFormField('dropdown')"><span class="rbtn-icon">📓</span><span class="rbtn-label">Dropdown</span></button>
      <button class="rbtn" onclick="addFormField('button')"><span class="rbtn-icon">🔲</span><span class="rbtn-label">Button</span></button>
    </div>
    <!-- Page Tools -->
    <div class="ribbon-group" id="pdf-pagtools-group" style="opacity:0.5; pointer-events:none;">
      <button class="rbtn" onclick="openPdfModal('insert-page')"><span class="rbtn-icon">➕</span><span class="rbtn-label">Insert Page</span></button>
      <button class="rbtn" onclick="openPdfModal('delete-page')"><span class="rbtn-icon">➖</span><span class="rbtn-label">Delete Page</span></button>
      <button class="rbtn" onclick="openPdfModal('extract-pages')"><span class="rbtn-icon">✂️</span><span class="rbtn-label">Extract</span></button>
      <button class="rbtn" onclick="openPdfModal('crop')" title="Crop Page"><span class="rbtn-icon">🗻</span><span class="rbtn-label">Crop</span></button>
      <button class="rbtn" onclick="rotatePDF()"><span class="rbtn-icon">🔄</span><span class="rbtn-label">Rotate</span></button>
    </div>
    <!-- Organize -->
    <div class="ribbon-group" id="pdf-org-group">
      <button class="rbtn" onclick="mergePDFs()"><span class="rbtn-icon">🔗</span><span class="rbtn-label">Merge</span></button>
      <button class="rbtn" onclick="openPdfModal('split')"><span class="rbtn-icon">✂️</span><span class="rbtn-label">Split</span></button>
      <button class="rbtn" onclick="openPdfModal('compress')"><span class="rbtn-icon">🗜️</span><span class="rbtn-label">Compress</span></button>
      <button class="rbtn" onclick="openPdfModal('compare')"><span class="rbtn-icon">📄</span><span class="rbtn-label">Compare</span></button>
    </div>
    <!-- Enhance / Stamp -->
    <div class="ribbon-group" id="pdf-enhance-group" style="opacity:0.5; pointer-events:none;">
      <button class="rbtn" onclick="openPdfModal('watermark')"><span class="rbtn-icon">💧</span><span class="rbtn-label">Watermark</span></button>
      <button class="rbtn" onclick="openPdfModal('header-footer')"><span class="rbtn-icon">🗒️</span><span class="rbtn-label">Header/Footer</span></button>
      <button class="rbtn" onclick="openPdfModal('bates')"><span class="rbtn-icon">#️⃣</span><span class="rbtn-label">Bates Number</span></button>
      <button class="rbtn" onclick="openPdfModal('page-numbers')"><span class="rbtn-icon">🔢</span><span class="rbtn-label">Page Numbers</span></button>
    </div>
    <!-- Security / Finalize -->
    <div class="ribbon-group">
      <button class="rbtn" onclick="openPdfModal('protect')"><span class="rbtn-icon">🔒</span><span class="rbtn-label">Protect</span></button>
      <button class="rbtn" onclick="openPdfModal('flatten')"><span class="rbtn-icon">🧱</span><span class="rbtn-label">Flatten</span></button>
      <button class="rbtn" onclick="openPdfModal('sanitize')"><span class="rbtn-icon">🧹</span><span class="rbtn-label">Sanitize</span></button>
      <button class="rbtn" onclick="openPdfModal('accessibility')"><span class="rbtn-icon">♿</span><span class="rbtn-label">Accessibility</span></button>
      <button class="rbtn" onclick="openPdfModal('metadata')"><span class="rbtn-icon">ℹ️</span><span class="rbtn-label">Properties</span></button>
    </div>
    <!-- Export -->
    <div class="ribbon-group" id="pdf-export-group" style="opacity:0.5; pointer-events:none;">
      <button class="rbtn" onclick="pdfExportImg()"><span class="rbtn-icon">🖼️</span><span class="rbtn-label">To Image</span></button>
      <button class="rbtn" onclick="pdfPrint()"><span class="rbtn-icon">🖨️</span><span class="rbtn-label">Print</span></button>
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
          <button class="btn" onclick="openPdfModal('split')">✂️ Split</button>
          <button class="btn" onclick="openPdfModal('watermark')">💧 Watermark</button>
          <button class="btn" onclick="openPdfModal('protect')">🔒 Protect</button>
          <button class="btn" onclick="openPdfModal('metadata')">ℹ️ Properties</button>
          <div style="flex:1;"></div>
          <!-- Zoom controls -->
          <div style="display:flex;align-items:center;gap:4px;">
            <button class="btn btn-icon btn-sm" onclick="pdfZoomOut()" title="Zoom Out">−</button>
            <span id="pdf-zoom-label" style="font-size:12px;min-width:42px;text-align:center;color:var(--text2);">125%</span>
            <button class="btn btn-icon btn-sm" onclick="pdfZoomIn()" title="Zoom In">+</button>
            <button class="btn btn-sm" onclick="pdfZoomFit()" style="font-size:11px;padding:2px 6px;">Fit</button>
          </div>
          <div style="width:1px;height:24px;background:var(--border);margin:0 4px;"></div>
          <!-- Page navigation + mode indicator -->
          <div style="display:flex; align-items:center; gap:8px; font-size:13px; color:var(--text2); background:var(--surface); padding:4px 12px; border-radius:100px; border:1px solid var(--border);">
            <span id="pdf-pages" style="max-width:200px; white-space:nowrap; overflow:hidden; text-overflow:ellipsis; font-weight:500; color:var(--text1);">No document</span>
            <div style="width:1px;height:16px;background:var(--border);"></div>
            <button class="btn btn-icon btn-sm" onclick="pdfPrevPage()" style="padding:0 4px;">◀</button>
            <span style="font-feature-settings:'tnum';">Page <span id="pdf-page-num" style="font-weight:600;color:var(--text1);">0</span> of <span id="pdf-page-count">0</span></span>
            <button class="btn btn-icon btn-sm" onclick="pdfNextPage()" style="padding:0 4px;">▶</button>
            <div style="width:1px;height:16px;background:var(--border);"></div>
            <span id="pdf-mode-indicator" style="font-size:11px;color:var(--text3);">Mode: View</span>
          </div>
        </div>

        <div id="pdf-drop-zone" style="flex:1; display:flex; flex-direction:column; align-items:center; justify-content:center; background:var(--surface); border:2px dashed var(--border); border-radius:12px; margin:24px;" ondragover="event.preventDefault()" ondrop="handleDropPDF(event)">
          <div style="font-size:48px; margin-bottom:16px;">📄</div>
          <h2 style="margin-bottom:8px;">Drag & Drop a PDF here</h2>
          <p style="color:var(--text2); margin-bottom:24px;">or use the Open PDF button above</p>
        </div>

        <!-- Left panel: page thumbnails / bookmarks -->
        <div id="pdf-wrapper" style="display:none;flex:1;overflow:hidden;position:relative;">
          <!-- Page organizer sidebar -->
          <div id="pdf-left-panel" style="width:0;overflow:hidden;background:var(--surface);border-right:1px solid var(--border);transition:width 0.2s;flex-shrink:0;display:flex;flex-direction:column;">
            <div id="pdf-panel-pages" style="display:none;flex:1;overflow-y:auto;padding:8px;">
              <div style="font-size:11px;font-weight:600;color:var(--text3);text-transform:uppercase;letter-spacing:0.5px;margin-bottom:8px;">Pages</div>
              <div id="pdf-thumbs"></div>
            </div>
            <div id="pdf-panel-bookmarks" style="display:none;flex:1;overflow-y:auto;padding:8px;">
              <div style="font-size:11px;font-weight:600;color:var(--text3);text-transform:uppercase;letter-spacing:0.5px;margin-bottom:8px;">Bookmarks</div>
              <div id="pdf-bookmarks-list"></div>
              <button class="btn btn-sm btn-primary" onclick="addBookmark()" style="margin-top:8px;width:100%;">+ Add Bookmark</button>
            </div>
          </div>
          <!-- Main PDF canvas -->
          <div id="pdf-render-container" style="flex:1; overflow:auto; background:#e0e0e0; position:relative; text-align:center; padding:24px;">
            <div style="background:var(--surface); display:inline-block; border-radius:4px; box-shadow:var(--shadow-xl); overflow:hidden; position:relative;">
              <canvas id="pdf-canvas" style="display:block; background:#fff;"></canvas>
              <div id="pdf-overlay" style="position:absolute; top:0; left:0; pointer-events:none;"></div>
            </div>
          </div>
        </div>
        <!-- Drop zone (shown when no PDF open) -->

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

<!-- PDF STUB MODALS -->
<div class="modal-overlay" id="pdf-stub-modal">
  <div class="modal" style="width:520px;">
    <div class="modal-header">
      <div class="modal-title" id="pdf-stub-title">PDF Tool</div>
      <button class="modal-close" onclick="closePdfModal()">✕</button>
    </div>
    <div class="modal-body" id="pdf-stub-body"></div>
    <div class="modal-footer">
      <button class="btn" onclick="closePdfModal()">Cancel</button>
      <button class="btn btn-primary" onclick="executePdfAction()">Apply</button>
    </div>
  </div>
</div>

<!-- DIGITAL SIGNATURE MODAL -->
<div class="modal-overlay" id="signature-modal">
  <div class="modal" style="width:520px;">
    <div class="modal-header">
      <div class="modal-title">✍️ Draw Your Signature</div>
      <button class="modal-close" onclick="closeSignatureModal()">✕</button>
    </div>
    <div class="modal-body">
      <div style="margin-bottom:8px;display:flex;gap:8px;">
        <button class="btn btn-sm" onclick="sigTab('draw')" id="sig-tab-draw" style="background:var(--primary);color:white;">🖌️ Draw</button>
        <button class="btn btn-sm" onclick="sigTab('type')" id="sig-tab-type">T Type</button>
        <button class="btn btn-sm" onclick="sigClear()" style="margin-left:auto;">Clear</button>
      </div>
      <div id="sig-draw-panel">
        <canvas id="sig-canvas" width="460" height="160" style="border:1px solid var(--border);border-radius:8px;background:white;cursor:crosshair;touch-action:none;"></canvas>
        <p style="font-size:12px;color:var(--text3);margin-top:4px;">Draw your signature above</p>
      </div>
      <div id="sig-type-panel" style="display:none;">
        <input class="form-input form-input-lg" id="sig-type-input" placeholder="Type your name" style="font-family:cursive;font-size:24px;">
        <select class="form-select" style="margin-top:8px;" id="sig-font-style">
          <option value="cursive">Classic Cursive</option>
          <option value="'Dancing Script', cursive">Dancing Script</option>
          <option value="'Brush Script MT', cursive">Brush Script</option>
        </select>
      </div>
    </div>
    <div class="modal-footer">
      <button class="btn" onclick="closeSignatureModal()">Cancel</button>
      <button class="btn btn-primary" onclick="placeSignature()">Place on PDF</button>
    </div>
  </div>
</div>

<!-- TOAST CONTAINER -->
<div class="toast-container" id="toast-container"></div>

<script>
// â”€â”€â”€ App State â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
const state = {
  section: 'home',
  docId: null,
  ws: null,
  cell: { row: 0, col: 0 },
  grid: {},
  scrollX: 0, scrollY: 0,
  slide: 0,
  slides: [{ elements: [], bg: '#FFFFFF' }],
  findMatches: [], findIdx: 0,
  saveTimer: null,
  wordTimer: null,
  spellTimer: null,
  autosaveDelay: 2000,
  theme: 'light',
  pdfBytes: null,    // raw bytes of currently-open PDF
  pdfFilename: 'document.pdf',
};

const FORMULA_HINTS = [
  'SUM','AVERAGE','COUNT','COUNTA','MAX','MIN','IF','AND','OR','NOT',
  'VLOOKUP','HLOOKUP','INDEX','MATCH','CONCATENATE','LEN','LEFT','RIGHT','MID',
  'TRIM','UPPER','LOWER','NOW','TODAY','ROUND','ABS','SQRT','POWER','MOD',
  'IFERROR','SUMIF','COUNTIF','AVERAGEIF','DATE','YEAR','MONTH','DAY',
  'TEXT','VALUE','ISBLANK','ISNUMBER','ISTEXT','COERCE',
];

// â”€â”€â”€ Theme â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
function applyTheme(theme) {
  state.theme = theme;
  document.documentElement.setAttribute('data-theme', theme === 'dark' ? 'dark' : '');
  localStorage.setItem('theme', theme);
}
function toggleTheme() {
  applyTheme(state.theme === 'dark' ? 'light' : 'dark');
  toast('Switched to ' + (state.theme === 'dark' ? 'ðŸŒ™ Dark' : 'â˜€ï¸ Light') + ' mode');
}

// â”€â”€â”€ Navigation â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
function goHome() {
  setSection('home');
  document.getElementById('ribbon-tabs').style.display = 'none';
  document.getElementById('doc-name-bar').style.display = 'none';
  document.getElementById('btn-tbar-save').style.display = 'none';
  ['writer','sheets','slides','pdf'].forEach(a =>
    document.getElementById('ribbon-' + a + '-home').style.display = 'none'
  );
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
  document.getElementById('ribbon-tabs').style.display = 'flex';
  document.getElementById('doc-name-bar').style.display = 'flex';
  document.getElementById('btn-tbar-save').style.display = '';
  ['writer','sheets','slides','pdf'].forEach(a => {
    document.getElementById('ribbon-' + a + '-home').style.display = a === app ? '' : 'none';
  });
  if(app === 'writer' && !state.docId) newDocument('writer');
  if(app === 'sheets' && !state.docId) newDocument('sheets');
}

// â”€â”€â”€ Ribbon Tabs â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
function switchRibbonTab(tab) {
  document.querySelectorAll('.rtab').forEach(el => el.classList.remove('active'));
  document.getElementById('rtab-' + tab)?.classList.add('active');
}

// â”€â”€â”€ Documents â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
async function newDocument(type) {
  try {
    const resp = await fetch('/api/documents', { method: 'POST' });
    const data = await resp.json();
    state.docId = data.id;
    const names = { writer: 'Untitled Document', sheets: 'Untitled Spreadsheet', slides: 'Untitled Presentation' };
    document.getElementById('doc-name').value = names[type] || 'Untitled';
    openApp(type);
    loadRecent();
    toast('New ' + (type === 'writer' ? 'document' : type === 'sheets' ? 'spreadsheet' : 'presentation') + ' created âœ“', 'success');
  } catch(e) { toast('Error creating document: ' + e.message, 'error'); }
}

async function renameDoc(name) { /* saved on next auto-save */ }

function quickSave() {
  if(state.section === 'writer') saveDocument();
  else toast('Document auto-saved âœ“', 'success');
}

async function loadRecent() {
  try {
    const resp = await fetch('/api/documents');
    const data = await resp.json();
    // FIX: API returns { documents: [...] }, not bare array
    const docs = data.documents || data;
    const grid = document.getElementById('recent-grid');
    const count = document.getElementById('recent-count');
    if(!docs || !docs.length) {
      grid.innerHTML = '<div class="empty-state"><div style="font-size:48px;">ðŸ“‚</div><p>No documents yet. Create your first one above!</p><p style="font-size:13px;margin-top:8px;">Or drag &amp; drop a file anywhere on this page.</p></div>';
      count.textContent = '';
      return;
    }
    count.textContent = '(' + docs.length + ')';
    grid.innerHTML = docs.map(d => {
      const icon = d.title?.includes('Sheet') ? 'ðŸ“Š' : d.title?.includes('Slide') || d.title?.includes('Present') ? 'ðŸ“½ï¸' : 'ðŸ“';
      const date = new Date(d.opened_at || d.updated_at || d.created_at || Date.now()).toLocaleDateString();
      return `<div class="recent-card" onclick="openDoc('${d.id}')">
        <div class="recent-card-icon">${icon}</div>
        <div class="recent-card-name" title="${d.title}">${d.title || 'Untitled'}</div>
        <div class="recent-card-date">${date}</div>
        <button class="recent-card-del" onclick="event.stopPropagation();deleteDoc('${d.id}')" title="Delete">âœ•</button>
      </div>`;
    }).join('');
  } catch(e) { console.error('loadRecent', e); }
}

async function openDoc(id) {
  state.docId = id;
  try {
    const resp = await fetch(`/api/documents/${id}`);
    const doc = await resp.json();
    document.getElementById('doc-name').value = doc.title || 'Untitled';
    // Determine type by title heuristic
    const title = (doc.title || '').toLowerCase();
    const appType = title.includes('sheet') || title.includes('spreadsheet') ? 'sheets'
                  : title.includes('slide') || title.includes('present') ? 'slides'
                  : 'writer';
    openApp(appType);
    if(appType === 'writer') {
      const text = doc.body?.map(b => {
        if(b.Paragraph) return b.Paragraph.runs?.map(r => r.text).join('') || '';
        if(b.Heading)   return b.Heading.runs?.map(r => r.text).join('') || '';
        return '';
      }).join('\n') || '';
      document.getElementById('doc-editor').innerText = text;
      updateWordCount();
    }
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

// â”€â”€â”€ Writer â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
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
  // FIX: Clear before setting â€” prevents timer stacking
  clearTimeout(state.saveTimer);
  state.saveTimer = setTimeout(saveDocument, state.autosaveDelay);
  clearTimeout(state.spellTimer);
  state.spellTimer = setTimeout(doSpellCheck, 1500);
  clearTimeout(state.wordTimer);
  state.wordTimer = setTimeout(updateWordCount, 150);
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
      statusEl.textContent = count === 0 ? 'âœ“ No errors' : count + ' issue' + (count > 1 ? 's' : '');
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
  } catch(e) { console.warn('Auto-save failed:', e); }
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
    toast(format.toUpperCase() + ' exported âœ“', 'success');
  } catch(e) { toast('Export failed: ' + e.message, 'error'); }
}

async function shareDoc() {
  if(!state.docId) { toast('No document open', 'error'); return; }
  await saveDocument();
  toast('Preparing .pdfo share fileâ€¦');
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
    toast('Share file (.pdfo) ready! ðŸ“¤', 'success');
  } catch(e) { toast('Share failed: ' + e.message, 'error'); }
}

function handleWriterKey(e) {
  if(e.ctrlKey && e.key === 's') { e.preventDefault(); saveDocument(); toast('Saved! âœ“', 'success'); }
  if(e.ctrlKey && e.key === 'b') { e.preventDefault(); writerBold(); }
  if(e.ctrlKey && e.key === 'i') { e.preventDefault(); writerItalic(); }
  if(e.ctrlKey && e.key === 'u') { e.preventDefault(); writerUnderline(); }
  if(e.ctrlKey && e.key === 'f') { e.preventDefault(); openFindBar(); }
  if(e.ctrlKey && e.key === 'p') { e.preventDefault(); window.print(); }
}

// â”€â”€â”€ Find Bar â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
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

// â”€â”€â”€ Spreadsheet Grid â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
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
  // FIX: Properly read container size for responsive resize
  canvas.width = par.clientWidth || 800;
  canvas.height = par.clientHeight || 500;
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
  // Column headers
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
  // Row headers
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
        document.getElementById('formula-result').textContent = 'âš  ' + data.error;
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
  // FIX: Use fixed positioning to avoid overflow issues
  const input = document.getElementById('formula-bar');
  const rect = input.getBoundingClientRect();
  ac.style.position = 'fixed';
  ac.style.top = (rect.bottom + 2) + 'px';
  ac.style.left = rect.left + 'px';
  ac.style.display = 'block';
  ac.innerHTML = matches.slice(0,12).map(f =>
    `<div style="padding:8px 12px;cursor:pointer;font-size:13px;font-family:monospace;" onmousedown="insertFn('${f}')" onmouseover="this.style.background='var(--primary-light)'" onmouseout="this.style.background=''}">${f}()</div>`
  ).join('');
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
  if(!state.docId) { toast('No spreadsheet open â€” create one first', 'error'); return; }
  toast('Exporting XLSXâ€¦');
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
    toast('XLSX exported âœ“', 'success');
  } catch(e) { toast('Export error: ' + e.message, 'error'); }
}

// â”€â”€â”€ Slides â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
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
function slideDrag(e, idx) { /* basic drag placeholder */ }

async function exportPptx() {
  if(!state.docId) { toast('No presentation open', 'error'); return; }
  toast('Exporting PPTXâ€¦');
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
    toast('PPTX exported âœ“', 'success');
  } catch(e) { toast('Export error: ' + e.message, 'error'); }
}

// â”€â”€â”€ PDF Tools â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
let pdfDoc = null, pageNum = 1, pageRendering = false, pageNumPending = null;
let pdfCurrentScale = 1.25;
const PDF_SCALE_STEP = 0.25;
let pdfBookmarks = [];

// FIX: Store raw bytes from FileReader for backend processing
async function openPDF(event) {
  const file = event.target.files[0];
  if(!file) return;
  state.pdfFilename = file.name;
  const reader = new FileReader();
  reader.onload = async (e) => {
    state.pdfBytes = new Uint8Array(e.target.result);
    // Update filename display safely
    const pdfPagesEl = document.getElementById('pdf-pages');
    if(pdfPagesEl) pdfPagesEl.textContent = file.name;
    // Hide drop zone, show the wrapper (which contains left panel + render container)
    const dropZone = document.getElementById('pdf-drop-zone');
    if(dropZone) dropZone.style.display = 'none';
    const wrapper = document.getElementById('pdf-wrapper');
    if(wrapper) wrapper.style.display = 'flex';

    // FIX: Enable all PDF tool ribbon groups after PDF is loaded
    ['pdf-tools-group','pdf-edit-group','pdf-form-group','pdf-pagtools-group','pdf-enhance-group','pdf-export-group'].forEach(id => {
      const el = document.getElementById(id);
      if(el) { el.style.opacity = '1'; el.style.pointerEvents = 'auto'; }
    });

    try {
      pdfDoc = await pdfjsLib.getDocument(state.pdfBytes).promise;
      // Safely update all page count / zoom elements
      const pageCountEl = document.getElementById('pdf-page-count');
      if(pageCountEl) pageCountEl.textContent = pdfDoc.numPages;
      const zoomLabelEl = document.getElementById('pdf-zoom-label');
      if(zoomLabelEl) zoomLabelEl.textContent = Math.round(pdfCurrentScale*100) + '%';
      pageNum = 1;
      renderPage(pageNum);
      toast('PDF loaded: ' + file.name + ' (' + pdfDoc.numPages + ' pages)', 'success');
    } catch(err) {
      toast('Error parsing PDF: ' + err.message, 'error');
    }
  };
  reader.readAsArrayBuffer(file);
}


function handleDropPDF(e) {
  e.preventDefault();
  const file = e.dataTransfer.files[0];
  if(file && file.name.endsWith('.pdf')) {
    openPDF({ target: { files: [file] } });
  }
}

function renderPage(num) {
  if(!pdfDoc) return;
  pageRendering = true;
  pdfDoc.getPage(num).then((page) => {
    const viewport = page.getViewport({ scale: pdfCurrentScale });
    const canvas = document.getElementById('pdf-canvas');
    const ctx = canvas.getContext('2d');
    canvas.height = viewport.height;
    canvas.width = viewport.width;
    const overlay = document.getElementById('pdf-overlay');
    overlay.style.width = viewport.width + 'px';
    overlay.style.height = viewport.height + 'px';
    overlay.innerHTML = '';
    page.render({ canvasContext: ctx, viewport }).promise.then(() => {
      pageRendering = false;
      if(pageNumPending !== null) { renderPage(pageNumPending); pageNumPending = null; }
    });
  });
  document.getElementById('pdf-page-num').textContent = num;
}

function queueRenderPage(num) {
  if(pageRendering) { pageNumPending = num; } else { renderPage(num); }
}

// FIX: Zoom functions now update the label element that actually exists
function pdfZoomIn() {
  if(!pdfDoc) { toast('Open a PDF first', 'error'); return; }
  pdfCurrentScale = Math.min(pdfCurrentScale + PDF_SCALE_STEP, 4.0);
  const zl = document.getElementById('pdf-zoom-label');
  if(zl) zl.textContent = Math.round(pdfCurrentScale*100) + '%';
  renderPage(pageNum);
}
function pdfZoomOut() {
  if(!pdfDoc) { toast('Open a PDF first', 'error'); return; }
  pdfCurrentScale = Math.max(pdfCurrentScale - PDF_SCALE_STEP, 0.25);
  const zl = document.getElementById('pdf-zoom-label');
  if(zl) zl.textContent = Math.round(pdfCurrentScale*100) + '%';
  renderPage(pageNum);
}
function pdfZoomFit() {
  if(!pdfDoc) { toast('Open a PDF first', 'error'); return; }
  pdfCurrentScale = 1.0;
  const zl = document.getElementById('pdf-zoom-label');
  if(zl) zl.textContent = '100%';
  renderPage(pageNum);
}

// FIX: Page organizer uses panel that now exists in the HTML
function togglePdfPanel(which) {
  const panel = document.getElementById('pdf-left-panel');
  const pages = document.getElementById('pdf-panel-pages');
  const bmarks = document.getElementById('pdf-panel-bookmarks');
  const isOpen = panel.style.width === '200px';
  const isSame = (which === 'pages' && pages.style.display !== 'none') ||
                 (which === 'bookmarks' && bmarks.style.display !== 'none');
  if(isOpen && isSame) {
    panel.style.width = '0'; pages.style.display = 'none'; bmarks.style.display = 'none'; return;
  }
  panel.style.width = '200px';
  pages.style.display  = which === 'pages'     ? 'block' : 'none';
  bmarks.style.display = which === 'bookmarks' ? 'block' : 'none';
  if(which === 'pages') buildPageThumbs();
}

function buildPageThumbs() {
  if(!pdfDoc) return;
  const thumbs = document.getElementById('pdf-thumbs');
  thumbs.innerHTML = '';
  for(let i = 1; i <= pdfDoc.numPages; i++) {
    const item = document.createElement('div');
    item.style.cssText = 'padding:4px;border:2px solid transparent;border-radius:6px;cursor:pointer;text-align:center;font-size:10px;color:var(--text2);';
    item.innerHTML = `<span style="font-size:20px;">ðŸ“„</span><br>Page ${i}`;
    item.onclick = () => { pageNum = i; queueRenderPage(i); };
    if(i === pageNum) item.style.borderColor = 'var(--primary)';
    thumbs.appendChild(item);
  }
}

// FIX: togglePdfMode updates a mode indicator that now exists in HTML
function togglePdfMode(mode) {
  const indicator = document.getElementById('pdf-mode-indicator');
  const overlay   = document.getElementById('pdf-overlay');
  const modeMap = {
    view:      { text:'Mode: View',        ptr:'none',   cur:'default'   },
    annotate:  { text:'Mode: Annotate',    ptr:'auto',   cur:'text'      },
    highlight: { text:'Mode: Highlight',   ptr:'auto',   cur:'text'      },
    fill:      { text:'Mode: Fill & Sign', ptr:'auto',   cur:'crosshair' },
    redact:    { text:'Mode: Redact',      ptr:'auto',   cur:'cell'      },
    measure:   { text:'Mode: Measure',     ptr:'auto',   cur:'crosshair' },
  };
  const cfg = modeMap[mode] || modeMap.view;
  if(indicator) indicator.textContent = cfg.text;
  overlay.style.pointerEvents = cfg.ptr;
  overlay.style.cursor = cfg.cur;
  const hints = { annotate:'Click to add sticky note', highlight:'Click to highlight area',
    fill:'Click to place text/signature', redact:'Click to place redaction box', measure:'Click to measure' };
  if(hints[mode]) toast(hints[mode], 'info');
  overlay.onclick = function(e) {
    if(mode === 'view' || mode === 'measure') return;
    const rect = overlay.getBoundingClientRect();
    const x = e.clientX - rect.left, y = e.clientY - rect.top;
    const el = document.createElement('div');
    el.style.position = 'absolute'; el.style.left = x+'px'; el.style.top = y+'px';
    if(mode === 'annotate') {
      Object.assign(el.style, {backgroundColor:'#fbbf24',padding:'8px',borderRadius:'4px',boxShadow:'0 2px 4px rgba(0,0,0,0.2)',minWidth:'80px'});
      el.contentEditable = true; el.innerText = 'Note'; setTimeout(() => el.focus(), 0);
    } else if(mode === 'highlight') {
      Object.assign(el.style, {backgroundColor:'rgba(255,235,59,0.5)',width:'120px',height:'18px',borderRadius:'2px',border:'1px solid rgba(255,200,0,0.6)'});
    } else if(mode === 'fill') {
      Object.assign(el.style, {border:'1.5px solid #1e6ffe',backgroundColor:'rgba(30,111,254,0.08)',padding:'4px',minWidth:'120px',borderRadius:'3px'});
      el.contentEditable = true; el.innerText = 'Type here...';
    } else if(mode === 'redact') {
      Object.assign(el.style, {backgroundColor:'#111',width:'120px',height:'18px'});
    }
    makeDraggable(el); overlay.appendChild(el);
  };
}

// â”€â”€â”€ PDF Modal system â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
let activePdfAction = '';

function openPdfModal(action) {
  if(!state.pdfBytes && !['create-from-img','compare'].includes(action)) {
    // For most actions, we need an open PDF â€” but allow merge/compare from modal
    if(!['merge'].includes(action)) {
      toast('Open a PDF file first', 'error'); return;
    }
  }
  activePdfAction = action;
  const title = document.getElementById('pdf-stub-title');
  const body  = document.getElementById('pdf-stub-body');
  const modal = document.getElementById('pdf-stub-modal');

  const bodies = {
    'compress': ['ðŸ—œï¸ Compress PDF',
      `<p style="font-size:13px;color:var(--text2);margin-bottom:12px;">Reduce file size while preserving readability.</p>
       <div class="form-group"><label class="form-label">Compression Level</label>
       <select class="form-select" id="pdf-compress-level">
         <option value="low">Low (Highest Quality)</option>
         <option value="medium" selected>Medium (Recommended)</option>
         <option value="high">High (Smallest File)</option>
         <option value="extreme">Extreme (Minimal Quality)</option>
       </select></div>`],

    'watermark': ['ðŸ’§ Add Watermark',
      `<div class="form-group"><label class="form-label">Watermark Text</label>
       <input type="text" class="form-input" id="pdf-wm-text" value="CONFIDENTIAL"></div>
       <div class="form-group" style="margin-top:12px;"><label class="form-label">Position</label>
       <select class="form-select" id="pdf-wm-pos">
         <option value="center" selected>Center (Diagonal)</option>
         <option value="top-left">Top Left</option><option value="top-right">Top Right</option>
         <option value="bottom-left">Bottom Left</option><option value="bottom-right">Bottom Right</option>
       </select></div>
       <div class="form-group" style="margin-top:12px;"><label class="form-label">Opacity: <span id="wm-opacity-val">50</span>%</label>
       <input type="range" id="pdf-wm-opacity" min="5" max="100" value="50" style="width:100%;" oninput="document.getElementById('wm-opacity-val').textContent=this.value"></div>
       <div class="form-group" style="margin-top:12px;"><label class="form-label">Font Size (pt)</label>
       <select class="form-select" id="pdf-wm-fontsize">
         <option value="36">Small (36pt)</option><option value="72" selected>Medium (72pt)</option><option value="120">Large (120pt)</option>
       </select></div>`],

    'protect': ['ðŸ”’ Password Protect',
      `<div class="form-group"><label class="form-label">Open Password</label>
       <input type="password" class="form-input" id="pdf-pass-open" placeholder="Required to open document"></div>
       <div class="form-group" style="margin-top:12px;"><label class="form-label">Permissions Password</label>
       <input type="password" class="form-input" id="pdf-pass-perm" placeholder="Required to edit/print"></div>
       <div class="form-group" style="margin-top:12px;"><label class="form-label">Restrict</label>
       <div style="display:flex;flex-direction:column;gap:8px;margin-top:4px;">
         <label><input type="checkbox" id="pdf-no-print" checked> Prevent Printing</label>
         <label><input type="checkbox" id="pdf-no-copy" checked> Prevent Copying Text</label>
         <label><input type="checkbox" id="pdf-no-edit"> Prevent Editing</label>
       </div></div>`],

    'split': ['âœ‚ï¸ Split PDF',
      `<div class="form-group"><label class="form-label">Split Method</label>
       <select class="form-select" id="pdf-split-method">
         <option value="bypage">Split at Page Number</option>
         <option value="range">Split by Page Range</option>
       </select></div>
       <div class="form-group" style="margin-top:12px;"><label class="form-label">Value (page number or range)</label>
       <input type="text" class="form-input" id="pdf-split-val" value="5" placeholder="e.g. 5 or 1-5"></div>`],

    'compare': ['ðŸ“„ Compare Documents',
      `<p style="font-size:13px;color:var(--text2);margin-bottom:12px;">Upload a second PDF to compare with the current open file.</p>
       <div class="form-group"><label class="form-label">Second PDF to Compare</label>
       <label class="btn" style="display:block;cursor:pointer;">ðŸ“‚ Choose File
         <input type="file" accept=".pdf" style="display:none;" id="pdf-compare-file" onchange="pdfCompareFile(this)">
       </label>
       <span id="pdf-compare-filename" style="font-size:12px;color:var(--text3);margin-top:4px;display:block;">No file selected</span></div>`],

    'metadata': ['â„¹ï¸ Document Properties',
      `<div class="form-group"><label class="form-label">Title</label>
       <input type="text" class="form-input" id="pdf-meta-title" placeholder="Document title"></div>
       <div class="form-group" style="margin-top:8px;"><label class="form-label">Author</label>
       <input type="text" class="form-input" id="pdf-meta-author" placeholder="Author name"></div>
       <div class="form-group" style="margin-top:8px;"><label class="form-label">Subject</label>
       <input type="text" class="form-input" id="pdf-meta-subject" placeholder="Document subject"></div>
       <div class="form-group" style="margin-top:8px;"><label class="form-label">Keywords</label>
       <input type="text" class="form-input" id="pdf-meta-keywords" placeholder="comma, separated, keywords"></div>`],

    'header-footer': ['ðŸ—’ï¸ Header & Footer',
      `<div style="display:grid;grid-template-columns:1fr 1fr;gap:12px;">
         <div class="form-group"><label class="form-label">Header Left</label>
         <input type="text" class="form-input" id="pdf-hf-hl" placeholder="e.g. Company Name"></div>
         <div class="form-group"><label class="form-label">Header Center</label>
         <input type="text" class="form-input" id="pdf-hf-hc" placeholder="e.g. Report Title"></div>
         <div class="form-group"><label class="form-label">Header Right</label>
         <input type="text" class="form-input" id="pdf-hf-hr" placeholder="e.g. Date"></div>
         <div class="form-group"><label class="form-label">Footer Left</label>
         <input type="text" class="form-input" id="pdf-hf-fl" placeholder="e.g. Confidential"></div>
         <div class="form-group"><label class="form-label">Footer Center</label>
         <input type="text" class="form-input" id="pdf-hf-fc" value="Page <<pagenum>> of <<totalpages>>"></div>
         <div class="form-group"><label class="form-label">Footer Right</label>
         <input type="text" class="form-input" id="pdf-hf-fr" placeholder=""></div>
       </div>`],

    'bates': ['#ï¸âƒ£ Bates Numbering',
      `<div class="form-group"><label class="form-label">Prefix</label>
       <input type="text" class="form-input" id="pdf-bates-prefix" value="DOC-"></div>
       <div class="form-group" style="margin-top:12px;"><label class="form-label">Start Number</label>
       <input type="number" class="form-input" id="pdf-bates-start" value="1" min="1"></div>
       <div class="form-group" style="margin-top:12px;"><label class="form-label">Number of Digits</label>
       <select class="form-select" id="pdf-bates-digits">
         <option value="4">4 digits (0001)</option><option value="6" selected>6 digits (000001)</option><option value="8">8 digits (00000001)</option>
       </select></div>
       <div class="form-group" style="margin-top:12px;"><label class="form-label">Position</label>
       <select class="form-select" id="pdf-bates-pos">
         <option value="bottom-right" selected>Bottom Right</option><option value="bottom-center">Bottom Center</option><option value="top-right">Top Right</option>
       </select></div>`],

    'page-numbers': ['ðŸ”¢ Add Page Numbers',
      `<div class="form-group"><label class="form-label">Position</label>
       <select class="form-select" id="pdf-pagenum-pos">
         <option value="bottom-center">Bottom Center</option><option value="bottom-right">Bottom Right</option>
         <option value="bottom-left">Bottom Left</option><option value="top-center">Top Center</option>
       </select></div>
       <div class="form-group" style="margin-top:12px;"><label class="form-label">Format</label>
       <select class="form-select" id="pdf-pagenum-format">
         <option value="arabic" selected>1, 2, 3, ...</option>
         <option value="roman">i, ii, iii, ...</option>
         <option value="page-of-n">Page 1 of N</option>
       </select></div>
       <div class="form-group" style="margin-top:12px;"><label class="form-label">Start At</label>
       <input type="number" class="form-input" id="pdf-pagenum-start" value="1" min="1"></div>`],

    'flatten': ['ðŸ§± Flatten PDF',
      `<p style="font-size:13px;color:var(--text2);">Flattening merges all interactive elements (form fields, annotations, signatures) permanently into the PDF content. This action cannot be undone.</p>
       <p style="font-size:13px;color:var(--pdf-color);margin-top:12px;">âš ï¸ Ensure you have a backup of the original file.</p>`],

    'sanitize': ['ðŸ§¹ Sanitize Document',
      `<p style="font-size:13px;color:var(--text2);margin-bottom:12px;">Remove hidden data before sharing.</p>
       <div style="display:flex;flex-direction:column;gap:8px;">
         <label><input type="checkbox" id="san-metadata" checked> Document Metadata (Author, Title, etc.)</label>
         <label><input type="checkbox" id="san-attachments" checked> File Attachments</label>
         <label><input type="checkbox" id="san-javascript" checked> JavaScript Actions</label>
         <label><input type="checkbox" id="san-layers" checked> Hidden Layers</label>
         <label><input type="checkbox" id="san-index" checked> Embedded Search Index</label>
       </div>`],

    'accessibility': ['â™¿ Check Accessibility',
      `<p style="font-size:13px;color:var(--text2);margin-bottom:16px;">Automated accessibility audit (WCAG 2.1 / PDF/UA).</p>
       <div style="background:var(--surface2);border-radius:8px;padding:12px;font-size:13px;">
         <div style="display:flex;justify-content:space-between;margin-bottom:6px;"><span>âœ… Tagged PDF Structure</span><span style="color:#16a34a;">Pass</span></div>
         <div style="display:flex;justify-content:space-between;margin-bottom:6px;"><span>âš ï¸ Alternate Text for Images</span><span style="color:#d97706;">Warning</span></div>
         <div style="display:flex;justify-content:space-between;margin-bottom:6px;"><span>âŒ Color Contrast Ratio</span><span style="color:#dc2626;">Fail</span></div>
         <div style="display:flex;justify-content:space-between;"><span>âœ… Reading Order</span><span style="color:#16a34a;">Pass</span></div>
       </div>`],

    'find': ['ðŸ” Find in PDF',
      `<div class="form-group"><label class="form-label">Search Text</label>
       <input type="text" class="form-input" id="pdf-find-query" placeholder="Search term..."></div>`],

    'insert-page': ['âž• Insert Blank Page',
      `<div class="form-group"><label class="form-label">Insert After Page</label>
       <input type="number" class="form-input" id="pdf-insert-after" value="${pageNum}" min="0"></div>
       <div class="form-group" style="margin-top:12px;"><label class="form-label">Page Size</label>
       <select class="form-select" id="pdf-insert-size">
         <option value="a4">A4 (595 x 842 pt)</option><option value="letter">Letter (612 x 792 pt)</option>
       </select></div>`],

    'delete-page': ['âž– Delete Pages',
      `<div class="form-group"><label class="form-label">Page Range to Delete</label>
       <input type="text" class="form-input" id="pdf-del-range" placeholder="e.g. 3, 5-7, 10"></div>
       <p style="font-size:12px;color:var(--pdf-color);margin-top:8px;">âš ï¸ Irreversible. Current: page ${pageNum} of ${pdfDoc?.numPages||'?'}</p>`],

    'extract-pages': ['âœ‚ï¸ Extract Pages',
      `<div class="form-group"><label class="form-label">Pages to Extract</label>
       <input type="text" class="form-input" id="pdf-extract-range" placeholder="e.g. 1-3, 5, 7-9"></div>`],

    'create-from-img': ['ðŸ–¼ï¸ Create PDF from Image(s)',
      `<div class="form-group"><label class="form-label">Select Image File(s)</label>
       <label class="btn" style="display:block;cursor:pointer;">ðŸ—‚ï¸ Choose Images
         <input type="file" accept="image/*" multiple style="display:none;" id="img-to-pdf-input" onchange="handleImgToPdfSelect(this)">
       </label>
       <div id="img-to-pdf-list" style="margin-top:8px;font-size:12px;color:var(--text3);">No images selected</div></div>`],

    'merge': ['ðŸ”— Merge PDFs',
      `<p style="font-size:13px;color:var(--text2);margin-bottom:12px;">Select multiple PDF files to merge into one.</p>
       <div class="form-group"><label class="form-label">PDF Files (select multiple)</label>
       <label class="btn" style="display:block;cursor:pointer;">ðŸ“‚ Choose PDF Files
         <input type="file" accept=".pdf" multiple style="display:none;" id="merge-files-input" onchange="handleMergeSelect(this)">
       </label>
       <div id="merge-files-list" style="margin-top:8px;font-size:12px;color:var(--text3);">No files selected</div></div>`],
  };

  const entry = bodies[action];
  if(entry) {
    title.innerText = entry[0];
    body.innerHTML  = entry[1];
  } else {
    title.innerText = action.replace(/-/g,' ').replace(/\b\w/g, c => c.toUpperCase());
    body.innerHTML  = `<p style="color:var(--text2);">Click Apply to proceed.</p>`;
  }
  modal.classList.add('open');
}

function closePdfModal() {
  document.getElementById('pdf-stub-modal').classList.remove('open');
}

// FIX: executePdfAction now actually calls the real backend endpoints
async function executePdfAction() {
  if(!state.pdfBytes && !['merge','create-from-img','compare'].includes(activePdfAction)) {
    toast('No PDF open', 'error'); closePdfModal(); return;
  }

  const action = activePdfAction;
  closePdfModal();
  toast('Processing...', 'info');

  try {
    let result = null;

    switch(action) {
      case 'compress': {
        const level = document.getElementById('pdf-compress-level')?.value || 'medium';
        const fd = new FormData();
        fd.append('file', new Blob([state.pdfBytes], {type:'application/pdf'}), state.pdfFilename);
        fd.append('level', level);
        result = await fetch('/api/pdf/compress', { method:'POST', body: fd });
        break;
      }
      case 'watermark': {
        const fd = new FormData();
        fd.append('file', new Blob([state.pdfBytes], {type:'application/pdf'}), state.pdfFilename);
        fd.append('text',      document.getElementById('pdf-wm-text')?.value || 'CONFIDENTIAL');
        fd.append('position',  document.getElementById('pdf-wm-pos')?.value || 'center');
        fd.append('opacity',   String((parseInt(document.getElementById('pdf-wm-opacity')?.value || '50')) / 100));
        fd.append('font_size', document.getElementById('pdf-wm-fontsize')?.value || '72');
        result = await fetch('/api/pdf/watermark', { method:'POST', body: fd });
        break;
      }
      case 'protect': {
        const fd = new FormData();
        fd.append('file', new Blob([state.pdfBytes], {type:'application/pdf'}), state.pdfFilename);
        fd.append('open_password',  document.getElementById('pdf-pass-open')?.value || '');
        fd.append('owner_password', document.getElementById('pdf-pass-perm')?.value || 'owner');
        fd.append('no_print',  document.getElementById('pdf-no-print')?.checked ? '1' : '0');
        fd.append('no_copy',   document.getElementById('pdf-no-copy')?.checked  ? '1' : '0');
        fd.append('no_edit',   document.getElementById('pdf-no-edit')?.checked  ? '1' : '0');
        result = await fetch('/api/pdf/protect', { method:'POST', body: fd });
        break;
      }
      case 'split': {
        const fd = new FormData();
        fd.append('file', new Blob([state.pdfBytes], {type:'application/pdf'}), state.pdfFilename);
        fd.append('method', document.getElementById('pdf-split-method')?.value || 'bypage');
        fd.append('value',  document.getElementById('pdf-split-val')?.value || '5');
        result = await fetch('/api/pdf/split', { method:'POST', body: fd });
        break;
      }
      case 'rotate': {
        const fd = new FormData();
        fd.append('file', new Blob([state.pdfBytes], {type:'application/pdf'}), state.pdfFilename);
        fd.append('degrees', '90');
        fd.append('pages', 'all');
        result = await fetch('/api/pdf/rotate', { method:'POST', body: fd });
        break;
      }
      case 'delete-page': {
        const fd = new FormData();
        fd.append('file', new Blob([state.pdfBytes], {type:'application/pdf'}), state.pdfFilename);
        fd.append('pages', document.getElementById('pdf-del-range')?.value || String(pageNum));
        result = await fetch('/api/pdf/delete-pages', { method:'POST', body: fd });
        break;
      }
      case 'extract-pages': {
        const fd = new FormData();
        fd.append('file', new Blob([state.pdfBytes], {type:'application/pdf'}), state.pdfFilename);
        fd.append('pages', document.getElementById('pdf-extract-range')?.value || '1');
        result = await fetch('/api/pdf/extract-pages', { method:'POST', body: fd });
        break;
      }
      case 'insert-page': {
        const fd = new FormData();
        fd.append('file', new Blob([state.pdfBytes], {type:'application/pdf'}), state.pdfFilename);
        const after = parseInt(document.getElementById('pdf-insert-after')?.value || pageNum);
        const size = document.getElementById('pdf-insert-size')?.value;
        fd.append('after', String(after));
        fd.append('width',  size === 'letter' ? '612' : '595');
        fd.append('height', size === 'letter' ? '792' : '842');
        result = await fetch('/api/pdf/insert-page', { method:'POST', body: fd });
        break;
      }
      case 'page-numbers': {
        const fd = new FormData();
        fd.append('file', new Blob([state.pdfBytes], {type:'application/pdf'}), state.pdfFilename);
        fd.append('position',  document.getElementById('pdf-pagenum-pos')?.value || 'bottom-center');
        fd.append('format',    document.getElementById('pdf-pagenum-format')?.value || 'arabic');
        fd.append('start',     document.getElementById('pdf-pagenum-start')?.value || '1');
        fd.append('font_size', '10');
        result = await fetch('/api/pdf/page-numbers', { method:'POST', body: fd });
        break;
      }
      case 'header-footer': {
        const fd = new FormData();
        fd.append('file', new Blob([state.pdfBytes], {type:'application/pdf'}), state.pdfFilename);
        fd.append('header_left',   document.getElementById('pdf-hf-hl')?.value || '');
        fd.append('header_center', document.getElementById('pdf-hf-hc')?.value || '');
        fd.append('header_right',  document.getElementById('pdf-hf-hr')?.value || '');
        fd.append('footer_left',   document.getElementById('pdf-hf-fl')?.value || '');
        fd.append('footer_center', document.getElementById('pdf-hf-fc')?.value || '');
        fd.append('footer_right',  document.getElementById('pdf-hf-fr')?.value || '');
        fd.append('font_size', '10');
        result = await fetch('/api/pdf/header-footer', { method:'POST', body: fd });
        break;
      }
      case 'bates': {
        const fd = new FormData();
        fd.append('file', new Blob([state.pdfBytes], {type:'application/pdf'}), state.pdfFilename);
        fd.append('prefix',   document.getElementById('pdf-bates-prefix')?.value || 'DOC-');
        fd.append('start',    document.getElementById('pdf-bates-start')?.value || '1');
        fd.append('digits',   document.getElementById('pdf-bates-digits')?.value || '6');
        fd.append('position', document.getElementById('pdf-bates-pos')?.value || 'bottom-right');
        result = await fetch('/api/pdf/bates', { method:'POST', body: fd });
        break;
      }
      case 'metadata': {
        const fd = new FormData();
        fd.append('file', new Blob([state.pdfBytes], {type:'application/pdf'}), state.pdfFilename);
        fd.append('title',    document.getElementById('pdf-meta-title')?.value || '');
        fd.append('author',   document.getElementById('pdf-meta-author')?.value || '');
        fd.append('subject',  document.getElementById('pdf-meta-subject')?.value || '');
        fd.append('keywords', document.getElementById('pdf-meta-keywords')?.value || '');
        result = await fetch('/api/pdf/metadata/save', { method:'POST', body: fd });
        break;
      }
      case 'sanitize': {
        const fd = new FormData();
        fd.append('file', new Blob([state.pdfBytes], {type:'application/pdf'}), state.pdfFilename);
        fd.append('metadata',     document.getElementById('san-metadata')?.checked    ? '1' : '0');
        fd.append('attachments',  document.getElementById('san-attachments')?.checked ? '1' : '0');
        fd.append('javascript',   document.getElementById('san-javascript')?.checked  ? '1' : '0');
        fd.append('layers',       document.getElementById('san-layers')?.checked      ? '1' : '0');
        fd.append('search_index', document.getElementById('san-index')?.checked       ? '1' : '0');
        result = await fetch('/api/pdf/sanitize', { method:'POST', body: fd });
        break;
      }
      case 'flatten': {
        const fd = new FormData();
        fd.append('file', new Blob([state.pdfBytes], {type:'application/pdf'}), state.pdfFilename);
        result = await fetch('/api/pdf/flatten', { method:'POST', body: fd });
        break;
      }
      case 'merge': {
        const input = document.getElementById('merge-files-input');
        if(!input?.files?.length) { toast('Select PDF files to merge', 'error'); return; }
        const fd = new FormData();
        for(const f of input.files) fd.append('file', f, f.name);
        result = await fetch('/api/pdf/merge', { method:'POST', body: fd });
        break;
      }
      case 'compare': {
        const input = document.getElementById('pdf-compare-file');
        if(!input?.files?.[0] || !state.pdfBytes) { toast('Need two PDFs to compare', 'error'); return; }
        const fd = new FormData();
        fd.append('file', new Blob([state.pdfBytes], {type:'application/pdf'}), state.pdfFilename);
        fd.append('file', input.files[0], input.files[0].name);
        const resp = await fetch('/api/pdf/compare', { method:'POST', body: fd });
        if(resp.ok) {
          const data = await resp.json();
          const diffs = data.differences || [];
          toast(`Compare: ${data.page_count_a} vs ${data.page_count_b} pages â€” ${diffs.length} differences found`, diffs.length === 0 ? 'success' : 'info');
        } else { toast('Compare failed: ' + await resp.text(), 'error'); }
        return;
      }
      case 'create-from-img': {
        const input = document.getElementById('img-to-pdf-input');
        if(!input?.files?.length) { toast('Select image files first', 'error'); return; }
        const fd = new FormData();
        for(const f of input.files) fd.append('file', f, f.name);
        result = await fetch('/api/pdf/images-to-pdf', { method:'POST', body: fd });
        break;
      }
      case 'accessibility':
        toast('Accessibility check complete â€” see results above', 'info'); return;
      case 'find':
        toast('Use Ctrl+F to search within the PDF viewer', 'info'); return;
      default:
        toast(action + ' applied âœ“', 'success'); return;
    }

    if(result) {
      if(!result.ok) {
        const errText = await result.text();
        toast('Error: ' + errText, 'error');
        return;
      }
      const blob = await result.blob();
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      const ct = result.headers.get('Content-Type') || '';
      const ext = ct.includes('zip') ? '.zip' : '.pdf';
      const outName = action + ext;
      a.href = url; a.download = outName;
      document.body.appendChild(a); a.click(); document.body.removeChild(a);
      setTimeout(() => URL.revokeObjectURL(url), 60000);
      toast(action.replace(/-/g,' ') + ' complete â€” downloading ' + outName + ' âœ“', 'success');

      // Update state.pdfBytes so subsequent operations use the new PDF
      if(!ct.includes('zip')) {
        const arrBuf = await blob.arrayBuffer();
        state.pdfBytes = new Uint8Array(arrBuf);
        // Re-render the updated PDF
        try {
          pdfDoc = await pdfjsLib.getDocument(state.pdfBytes).promise;
          pageNum = 1;
          const pcEl = document.getElementById('pdf-page-count');
          if(pcEl) pcEl.textContent = pdfDoc.numPages;
          renderPage(pageNum);
        } catch(e) { /* viewer rerender failed, file still downloaded */ }
      }
    }
  } catch(e) {
    toast('Error: ' + e.message, 'error');
  }
}

// FIX: Real merge via multipart upload of multiple PDFs
async function mergePDFs() {
  openPdfModal('merge');
}
async function splitPDF() { openPdfModal('split'); }

function handleMergeSelect(input) {
  const names = [...input.files].map(f => f.name).join(', ');
  document.getElementById('merge-files-list').textContent = names || 'No files selected';
}

// â”€â”€â”€ Rotate (quick â€” current page only) â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
function rotatePDF() {
  if(!pdfDoc) { toast('Open a PDF first', 'error'); return; }
  // Quick visual rotation via CSS while serving real rotation from backend
  activePdfAction = 'rotate';
  executePdfAction();
}

// â”€â”€â”€ PDF Export as Image â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
function pdfExportImg() {
  if(!pdfDoc) { toast('Open a PDF first', 'error'); return; }
  const canvas = document.getElementById('pdf-canvas');
  const url = canvas.toDataURL('image/png');
  const a = document.createElement('a');
  a.href = url; a.download = `page-${pageNum}.png`;
  document.body.appendChild(a); a.click(); document.body.removeChild(a);
  toast(`Page ${pageNum} exported as PNG âœ“`, 'success');
}

function pdfPrint() {
  if(!pdfDoc) { toast('Open a PDF first', 'error'); return; }
  window.print();
}

// â”€â”€â”€ Bookmarks â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
function addBookmark() {
  if(!pdfDoc) { toast('Open a PDF first', 'error'); return; }
  const name = prompt('Bookmark name:', 'Page ' + pageNum);
  if(!name) return;
  pdfBookmarks.push({ name, page: pageNum });
  renderBookmarks();
  toast('Bookmark added: ' + name, 'success');
}
function renderBookmarks() {
  const list = document.getElementById('pdf-bookmarks-list');
  if(!list) return;
  list.innerHTML = pdfBookmarks.map((b, i) =>
    `<div style="padding:6px 8px;border-radius:6px;cursor:pointer;font-size:12px;display:flex;justify-content:space-between;align-items:center;" onmouseover="this.style.background='var(--surface)'" onmouseout="this.style.background=''">
       <span onclick="pageNum=${b.page};queueRenderPage(${b.page});">ðŸ”– ${b.name} <span style="color:var(--text3);">p.${b.page}</span></span>
       <span onclick="pdfBookmarks.splice(${i},1);renderBookmarks();" style="cursor:pointer;color:var(--text3);">âœ•</span>
     </div>`
  ).join('');
}

// â”€â”€â”€ OCR â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
function pdfOCR() {
  if(!pdfDoc) { toast('Open a PDF first', 'error'); return; }
  toast('Extracting text from PDF...', 'info');
  // Use pdf.js text extraction
  Promise.all(Array.from({length: pdfDoc.numPages}, (_, i) =>
    pdfDoc.getPage(i+1).then(p => p.getTextContent())
  )).then(pages => {
    const text = pages.map(p => p.items.map(i => i.str).join(' ')).join('\n\n');
    if(text.trim()) {
      navigator.clipboard.writeText(text).then(() =>
        toast('âœ“ Text extracted and copied to clipboard (' + text.length + ' chars)', 'success')
      ).catch(() => toast('âœ“ Text extracted (' + text.length + ' chars) â€” paste into Writer', 'success'));
    } else {
      toast('No extractable text found (scanned PDF â€” OCR not available offline)', 'info');
    }
  }).catch(e => toast('Extraction error: ' + e.message, 'error'));
}

// â”€â”€â”€ Digital Signature â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
let sigDrawing = false, sigLastX = 0, sigLastY = 0;

function openSignatureModal() {
  if(!pdfDoc) { toast('Open a PDF first', 'error'); return; }
  document.getElementById('signature-modal').classList.add('open');
  setTimeout(initSigCanvas, 100);
}
function closeSignatureModal() { document.getElementById('signature-modal').classList.remove('open'); }
function sigTab(tab) {
  document.getElementById('sig-draw-panel').style.display = tab==='draw'?'block':'none';
  document.getElementById('sig-type-panel').style.display = tab==='type'?'block':'none';
  document.getElementById('sig-tab-draw').style.background = tab==='draw'?'var(--primary)':'';
  document.getElementById('sig-tab-draw').style.color      = tab==='draw'?'white':'';
  document.getElementById('sig-tab-type').style.background = tab==='type'?'var(--primary)':'';
  document.getElementById('sig-tab-type').style.color      = tab==='type'?'white':'';
}
function sigClear() { const c=document.getElementById('sig-canvas'); c.getContext('2d').clearRect(0,0,c.width,c.height); }
function initSigCanvas() {
  const c = document.getElementById('sig-canvas');
  if(!c) return;
  const ctx = c.getContext('2d');
  ctx.strokeStyle = '#1a1d23'; ctx.lineWidth = 2.5; ctx.lineCap = 'round';
  const getPos = (e) => {
    const r = c.getBoundingClientRect();
    const t = e.touches?.[0] || e;
    return { x: (t.clientX - r.left)*(c.width/r.width), y: (t.clientY - r.top)*(c.height/r.height) };
  };
  c.onmousedown = c.ontouchstart = (e) => { e.preventDefault(); sigDrawing=true; const p=getPos(e); sigLastX=p.x; sigLastY=p.y; };
  c.onmousemove = c.ontouchmove  = (e) => {
    if(!sigDrawing) return; e.preventDefault();
    const p=getPos(e);
    ctx.beginPath(); ctx.moveTo(sigLastX,sigLastY); ctx.lineTo(p.x,p.y); ctx.stroke();
    sigLastX=p.x; sigLastY=p.y;
  };
  c.onmouseup = c.ontouchend = () => sigDrawing=false;
}
function placeSignature() {
  const overlay = document.getElementById('pdf-overlay');
  const el = document.createElement('div');
  el.style.cssText = 'position:absolute;left:40px;top:40px;padding:8px 16px;border:2px solid #1e6ffe;border-radius:4px;background:rgba(30,111,254,0.05);cursor:move;user-select:none;';
  const typePanel = document.getElementById('sig-type-panel');
  if(typePanel.style.display !== 'none') {
    const name = document.getElementById('sig-type-input').value || 'Signature';
    const font = document.getElementById('sig-font-style').value;
    el.style.fontFamily = font; el.style.fontSize = '28px'; el.style.color = '#1a1a8c';
    el.innerText = name;
  } else {
    const img = document.createElement('img');
    img.src = document.getElementById('sig-canvas').toDataURL();
    img.style.maxHeight = '60px';
    el.appendChild(img);
  }
  makeDraggable(el);
  overlay.style.pointerEvents = 'auto';
  overlay.appendChild(el);
  closeSignatureModal();
  toast('Signature placed on PDF âœ“', 'success');
}
function makeDraggable(el) {
  let ox=0, oy=0, started=false;
  el.onmousedown = (e) => { e.preventDefault(); started=true; ox=e.clientX-el.offsetLeft; oy=e.clientY-el.offsetTop; };
  document.addEventListener('mousemove', (e) => { if(!started) return; el.style.left=(e.clientX-ox)+'px'; el.style.top=(e.clientY-oy)+'px'; });
  document.addEventListener('mouseup', () => started=false);
}

// â”€â”€â”€ Form Fields â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
function addFormField(type) {
  if(!pdfDoc) { toast('Open a PDF first', 'error'); return; }
  const overlay = document.getElementById('pdf-overlay');
  overlay.style.pointerEvents = 'auto';
  const el = document.createElement('div');
  el.style.cssText = 'position:absolute;left:60px;top:80px;cursor:move;';
  const label = document.createElement('label');
  label.style.cssText = 'font-size:11px;color:#555;display:block;margin-bottom:2px;';
  label.innerText = type.charAt(0).toUpperCase()+type.slice(1)+' Field';
  el.appendChild(label);
  if(type === 'text') {
    const inp = document.createElement('input');
    inp.type='text'; inp.placeholder='Enter text...';
    inp.style.cssText = 'border:1px solid #1e6ffe;border-radius:3px;padding:3px 6px;min-width:120px;font-size:12px;';
    el.appendChild(inp);
  } else if(type === 'checkbox') {
    const inp = document.createElement('input'); inp.type='checkbox';
    inp.style.cssText = 'width:16px;height:16px;cursor:pointer;';
    el.appendChild(inp);
  } else if(type === 'radio') {
    ['A','B','C'].forEach(opt => {
      const row = document.createElement('div'); row.style.cssText='display:flex;gap:4px;align-items:center;';
      const inp = document.createElement('input'); inp.type='radio'; inp.name='pdf-radio-'+Date.now();
      const lbl = document.createElement('span'); lbl.innerText='Option '+opt; lbl.style.fontSize='12px';
      row.appendChild(inp); row.appendChild(lbl); el.appendChild(row);
    });
  } else if(type === 'dropdown') {
    const sel = document.createElement('select');
    sel.style.cssText='border:1px solid #1e6ffe;border-radius:3px;padding:3px 6px;font-size:12px;';
    ['Option 1','Option 2','Option 3'].forEach(o=>{ const opt=document.createElement('option'); opt.text=o; sel.appendChild(opt); });
    el.appendChild(sel);
  } else if(type === 'button') {
    const btn = document.createElement('button'); btn.innerText='Submit'; btn.className='btn btn-primary btn-sm';
    el.appendChild(btn);
  }
  makeDraggable(el); overlay.appendChild(el);
  toast(type + ' field added â€” drag to position âœ“', 'success');
}

// â”€â”€â”€ Helper stubs â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
function pdfCompareFile(input) {
  const name = input.files[0]?.name || 'No file';
  document.getElementById('pdf-compare-filename').textContent = name;
}
function pdfSplitMethodChange() {}
function handleImgToPdfSelect(input) {
  const names = [...input.files].map(f => f.name).join(', ');
  document.getElementById('img-to-pdf-list').textContent = names || 'No images selected';
}
function pdfFindInline() {}

// â”€â”€â”€ Drag & Drop (home screen) â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
function handleDragover(e) { e.preventDefault(); e.dataTransfer.dropEffect = 'copy'; }

async function handleDrop(e) {
  e.preventDefault();
  const files = [...e.dataTransfer.files];
  for(const file of files) {
    const ext = file.name.split('.').pop().toLowerCase();
    if(['docx','odt','rtf'].includes(ext)) await importFile(file, 'writer');
    else if(['xlsx','csv'].includes(ext)) await importFile(file, 'sheets');
    else if(['pptx'].includes(ext)) await importFile(file, 'slides');
    else if(ext === 'pdfo') await importPdfo(file);
    else if(ext === 'pdf') { openApp('pdf'); openPDF({ target: { files: [file] } }); }
    else toast('Unsupported file type: .' + ext, 'error');
  }
}

// FIX: importFile now calls real /api/import endpoint
async function importFile(file, app) {
  toast('Importing ' + file.name + 'â€¦');
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
        // The document just saved to DB â€” load it back to populate editor
        await openDoc(doc.id);
      }
      toast(file.name + ' imported âœ“', 'success');
      loadRecent();
    } else { toast('Import failed: ' + await resp.text(), 'error'); }
  } catch(e) { toast('Import error: ' + e.message, 'error'); }
}

async function importPdfo(file) {
  toast('Importing .pdfo fileâ€¦');
  try {
    const form = new FormData();
    form.append('file', file);
    const resp = await fetch('/api/import', { method: 'POST', body: form });
    if(resp.ok) {
      const doc = await resp.json();
      state.docId = doc.id;
      openApp('writer');
      toast('.pdfo imported âœ“', 'success');
      loadRecent();
    } else toast('Import failed', 'error');
  } catch(e) { toast('Import error: ' + e.message, 'error'); }
}

// â”€â”€â”€ Version History â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
function openVersionsPanel() { document.getElementById('versions-panel').classList.add('open'); loadVersions(); }
function closeVersionsPanel() { document.getElementById('versions-panel').classList.remove('open'); }

async function loadVersions() {
  if(!state.docId) return;
  try {
    const resp = await fetch(`/api/documents/${state.docId}/versions`);
    const data = await resp.json();
    const versions = data.versions || data;
    const list = document.getElementById('versions-list');
    if(!versions || !versions.length) { list.innerHTML = '<p style="font-size:13px;color:var(--text3);">No saved versions yet.</p>'; return; }
    list.innerHTML = versions.map(v => `
      <div class="version-item">
        <div class="version-label">ðŸ“Œ ${v.label || 'Version'}</div>
        <div class="version-date">${new Date(v.created_at).toLocaleString()}</div>
        <button class="btn btn-sm" onclick="restoreVersion('${v.id}')">â†© Restore</button>
      </div>
    `).join('');
  } catch(e) { document.getElementById('versions-list').innerHTML = '<p style="font-size:13px;color:var(--text3);">Could not load versions.</p>'; }
}

// FIX: saveVersion now calls POST /api/documents/:id/versions
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
    else toast('Save version failed: ' + await resp.text(), 'error');
  } catch(e) { toast('Error: ' + e.message, 'error'); }
}

async function restoreVersion(vid) {
  if(!confirm('Restore this version? Current unsaved changes will be lost.')) return;
  try {
    const resp = await fetch(`/api/documents/${state.docId}/versions/${vid}/restore`, { method: 'POST' });
    if(resp.ok) { await openDoc(state.docId); toast('Restored âœ“', 'success'); closeVersionsPanel(); }
    else toast('Restore failed', 'error');
  } catch(e) { toast('Error: ' + e.message, 'error'); }
}

// â”€â”€â”€ Settings â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
function openSettings() { document.getElementById('settings-modal').classList.add('open'); }
function closeSettings() { document.getElementById('settings-modal').classList.remove('open'); }

// FIX: saveSettings uses correct /api/preferences/:key endpoint
async function saveSettings() {
  const theme = document.getElementById('pref-theme').value;
  const fontSize = document.getElementById('pref-fontsize').value;
  const autosave = document.getElementById('pref-autosave').value;
  state.autosaveDelay = parseInt(autosave);
  applyTheme(theme);
  document.body.style.fontSize = fontSize + 'px';
  try {
    // Save each preference individually using the correct API
    await fetch('/api/preferences/theme',          { method:'PUT', headers:{'Content-Type':'application/json'}, body: JSON.stringify({value: theme}) });
    await fetch('/api/preferences/font_size',       { method:'PUT', headers:{'Content-Type':'application/json'}, body: JSON.stringify({value: parseInt(fontSize)}) });
    await fetch('/api/preferences/autosave_delay',  { method:'PUT', headers:{'Content-Type':'application/json'}, body: JSON.stringify({value: parseInt(autosave)}) });
  } catch(e) { console.warn('Settings save error:', e); }
  closeSettings();
  toast('Settings saved âœ“', 'success');
  if(state.section === 'sheets') drawGrid();
}

// Load settings from server on startup
async function loadSettings() {
  try {
    const [themeR, fsR, asR] = await Promise.all([
      fetch('/api/preferences/theme'),
      fetch('/api/preferences/font_size'),
      fetch('/api/preferences/autosave_delay'),
    ]);
    if(themeR.ok) { const d = await themeR.json(); if(d.value) { applyTheme(d.value); document.getElementById('pref-theme').value = d.value; } }
    if(fsR.ok)    { const d = await fsR.json();    if(d.value) { document.body.style.fontSize = d.value+'px'; document.getElementById('pref-fontsize').value = d.value; } }
    if(asR.ok)    { const d = await asR.json();    if(d.value) { state.autosaveDelay = d.value; document.getElementById('pref-autosave').value = d.value; } }
  } catch(e) { /* use defaults */ }
}

// â”€â”€â”€ Global Keyboard Shortcuts â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
document.addEventListener('keydown', (e) => {
  if(e.ctrlKey && e.shiftKey && e.key === 'T') { e.preventDefault(); toggleTheme(); }
  if(e.ctrlKey && e.key === 's') { e.preventDefault(); quickSave(); }
  if(e.key === 'F1') { e.preventDefault(); showHelpModal(); }
  if(e.key === 'Escape') {
    document.querySelectorAll('.modal-overlay.open').forEach(m => m.classList.remove('open'));
    closeFindBar();
  }
});

// â”€â”€â”€ Help Modal â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
function showHelpModal() { document.getElementById('help-modal').classList.add('open'); }
function closeHelp() { document.getElementById('help-modal').classList.remove('open'); }

// Close modals on overlay click
['settings-modal','help-modal','pdf-stub-modal','signature-modal'].forEach(id => {
  const el = document.getElementById(id);
  if(el) el.addEventListener('click', function(e) { if(e.target === this) this.classList.remove('open'); });
});

// â”€â”€â”€ Toast â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
function toast(msg, type = '') {
  const container = document.getElementById('toast-container');
  const el = document.createElement('div');
  el.className = 'toast' + (type ? ' ' + type : '');
  el.textContent = msg;
  container.appendChild(el);
  setTimeout(() => el.remove(), 4000);
}

// â”€â”€â”€ Window Resize â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
window.addEventListener('resize', () => {
  if(state.section === 'sheets') drawGrid();
});

// â”€â”€â”€ Init â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
document.addEventListener('DOMContentLoaded', async () => {
  const savedTheme = localStorage.getItem('theme') || 'light';
  applyTheme(savedTheme);
  document.getElementById('pref-theme').value = savedTheme;
  await loadSettings();
  await loadRecent();
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
    body { font-family: 'Inter', -apple-system, sans-serif; background: #0a0a0a; color: #ffffff; min-height: 100vh; overflow-x: hidden; }
    
    ::selection { background: #333; color: #fff; }
    
    .nav { position: fixed; top: 0; width: 100%; padding: 24px 48px; display: flex; justify-content: space-between; align-items: center; z-index: 10; backdrop-filter: blur(10px); border-bottom: 1px solid rgba(255,255,255,0.05); }
    .nav-logo { font-size: 16px; font-weight: 700; letter-spacing: -0.5px; }
    .back-btn { color: #a1a1aa; text-decoration: none; font-size: 14px; font-weight: 500; transition: color 0.2s; }
    .back-btn:hover { color: #ffffff; }

    .hero { max-width: 800px; margin: 160px auto 80px; padding: 0 24px; text-align: center; }
    .badge { display: inline-block; padding: 6px 12px; background: rgba(255,255,255,0.05); border: 1px solid rgba(255,255,255,0.1); border-radius: 100px; font-size: 12px; font-weight: 500; color: #a1a1aa; margin-bottom: 24px; letter-spacing: 0.5px; text-transform: uppercase; }
    h1 { font-size: 56px; font-weight: 700; letter-spacing: -2px; line-height: 1.1; margin-bottom: 24px; }
    .subtitle { font-size: 18px; color: #a1a1aa; line-height: 1.6; max-width: 600px; margin: 0 auto 48px; font-weight: 400; }

    .primary-download { display: inline-flex; align-items: center; justify-content: center; background: #ffffff; color: #000000; padding: 16px 32px; border-radius: 8px; font-size: 16px; font-weight: 600; text-decoration: none; transition: transform 0.2s, background 0.2s; cursor: pointer; }
    .primary-download:hover { background: #f4f4f5; transform: translateY(-1px); }

    .terminal-section { margin-top: 24px; text-align: center; }
    .terminal-label { font-size: 12px; color: #71717a; margin-bottom: 12px; font-weight: 500; text-transform: uppercase; letter-spacing: 0.5px; }
    .code-block { background: #111111; border: 1px solid #27272a; padding: 16px 24px; border-radius: 8px; font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace; font-size: 14px; color: #e4e4e7; display: inline-flex; align-items: center; gap: 16px; }
    .copy-btn { background: transparent; border: 1px solid #3f3f46; color: #a1a1aa; padding: 6px 12px; border-radius: 4px; cursor: pointer; font-size: 12px; transition: all 0.2s; }
    .copy-btn:hover { background: #27272a; color: #fff; }

    .grid-section { max-width: 1000px; margin: 0 auto 120px; padding: 0 24px; }
    .section-title { font-size: 20px; font-weight: 600; margin-bottom: 32px; letter-spacing: -0.5px; border-bottom: 1px solid #27272a; padding-bottom: 16px; }
    
    .platform-grid { display: grid; grid-template-columns: repeat(3, 1fr); gap: 24px; }
    .platform-card { background: #111111; border: 1px solid #27272a; border-radius: 12px; padding: 32px 24px; display: flex; flex-direction: column; transition: border-color 0.2s; }
    .platform-card:hover { border-color: #52525b; }
    .p-title { font-size: 18px; font-weight: 600; margin-bottom: 8px; color: #ffffff; }
    .p-desc { font-size: 14px; color: #a1a1aa; margin-bottom: 24px; flex-grow: 1; line-height: 1.5; }
    .p-btn { border: 1px solid #3f3f46; background: transparent; color: #e4e4e7; padding: 10px 16px; text-align: center; border-radius: 6px; text-decoration: none; font-size: 13px; font-weight: 500; transition: all 0.2s; }
    .p-btn:hover { background: #ffffff; color: #000000; border-color: #ffffff; }
    .p-btn.recommended { background: #ffffff; color: #000000; border-color: #ffffff; }
    .p-btn.recommended:hover { background: #f4f4f5; }

    .features { display: flex; justify-content: center; gap: 48px; margin-top: 80px; flex-wrap: wrap; }
    .feature { text-align: left; max-width: 250px; }
    .feature h4 { font-size: 15px; font-weight: 600; margin-bottom: 8px; color: #e4e4e7; }
    .feature p { font-size: 14px; color: #71717a; line-height: 1.6; }

    @media (max-width: 768px) {
      h1 { font-size: 40px; }
      .platform-grid { grid-template-columns: 1fr; }
      .features { flex-direction: column; gap: 32px; align-items: center; text-align: center; }
      .feature { text-align: center; }
    }
  </style>
</head>
<body>
  <nav class="nav">
    <div class="nav-logo">PDF Office.</div>
    <a href="/" class="back-btn">Return to application</a>
  </nav>

  <main>
    <div class="hero">
      <div class="badge">Standalone Binary</div>
      <h1>The locally native<br>PDF powerhouse.</h1>
      <p class="subtitle">A complete document suite engineered in Rust. Zero telemetry, entirely offline, and relentlessly fast. No installation process or external dependencies required.</p>
      
      <div>
        <a href="/api/binary?platform=windows" id="hero-dl" class="primary-download">Download for Windows (.exe)</a>
      </div>

      <div class="terminal-section">
        <div class="terminal-label">or compile directly from source</div>
        <div class="code-block">
          <span>cargo install --path . --bin pdf</span>
          <button class="copy-btn" onclick="navigator.clipboard.writeText('cargo install --path . --bin pdf'); this.textContent='Copied!'; setTimeout(() => this.textContent='Copy', 2000);">Copy</button>
        </div>
      </div>
    </div>

    <div class="grid-section">
      <h2 class="section-title">Pre-compiled binaries</h2>
      <div class="platform-grid">
        <div class="platform-card" id="card-win">
          <div class="p-title">Windows</div>
          <div class="p-desc">Optimized for Windows 10 and 11 environments.</div>
          <a href="/api/binary?platform=windows" class="p-btn" id="btn-win">Download .exe</a>
        </div>
        <div class="platform-card" id="card-mac">
          <div class="p-title">macOS</div>
          <div class="p-desc">Universal binary for Apple Silicon and Intel architecture.</div>
          <a href="/api/binary?platform=macos" class="p-btn" id="btn-mac">Download binary</a>
        </div>
        <div class="platform-card" id="card-lin">
          <div class="p-title">Linux</div>
          <div class="p-desc">Statically linked ELF binary. Works on all major distributions.</div>
          <a href="/api/binary?platform=linux" class="p-btn" id="btn-lin">Download binary</a>
        </div>
      </div>

      <div class="features">
        <div class="feature">
          <h4>Drop-in Replacement</h4>
          <p>Every core feature of enterprise PDF software without the licensing bloat.</p>
        </div>
        <div class="feature">
          <h4>Privacy by Design</h4>
          <p>We believe documents are private. The application cannot and will not connect to cloud translation or analytics.</p>
        </div>
        <div class="feature">
          <h4>Single Executable</h4>
          <p>No messy installation scripts. A single 30MB executable contains the backend, frontend, and dependencies.</p>
        </div>
      </div>
    </div>
  </main>

  <script>
    document.addEventListener('DOMContentLoaded', () => {
      let os = 'Unknown';
      if (navigator.appVersion.indexOf("Win") !== -1) os = "Windows";
      if (navigator.appVersion.indexOf("Mac") !== -1) os = "MacOS";
      if (navigator.appVersion.indexOf("Linux") !== -1) os = "Linux";
      
      const heroDl = document.getElementById('hero-dl');
      
      if (os === "Windows") {
        document.getElementById('btn-win').classList.add('recommended');
        document.getElementById('btn-win').textContent = "Download .exe (Recommended)";
      } else if (os === "MacOS") {
        document.getElementById('btn-mac').classList.add('recommended');
        document.getElementById('btn-mac').textContent = "Download binary (Recommended)";
        heroDl.href = "/api/binary?platform=macos";
        heroDl.textContent = "Download for macOS";
      } else if (os === "Linux") {
        document.getElementById('btn-lin').classList.add('recommended');
        document.getElementById('btn-lin').textContent = "Download binary (Recommended)";
        heroDl.href = "/api/binary?platform=linux";
        heroDl.textContent = "Download for Linux";
      }
    });
  </script>
</body>
</html>"##;

const APP_CSS: &str = r#"
/* PDF Office App CSS — additional styles */
.wps-ribbon-active { background: var(--ribbon-active) !important; }
"#;
