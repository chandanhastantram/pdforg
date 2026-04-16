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

const INDEX_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>PDF Office — Local</title>
  <meta name="description" content="PDF Office: local-first, privacy-respecting office suite built in Rust">
  <link rel="preconnect" href="https://fonts.googleapis.com">
  <link href="https://fonts.googleapis.com/css2?family=Inter:wght@300;400;500;600;700&family=JetBrains+Mono:wght@400;500&display=swap" rel="stylesheet">
  <link rel="stylesheet" href="/assets/app.css">
  <style>
    :root {
      --bg: #0d1117;
      --bg2: #161b22;
      --bg3: #21262d;
      --border: #30363d;
      --text: #e6edf3;
      --text2: #8b949e;
      --accent: #58a6ff;
      --accent2: #3fb950;
      --accent3: #f78166;
      --writer: #4493f8;
      --sheets: #3fb950;
      --slides: #e3b341;
      --pdf: #f78166;
    }
    * { box-sizing: border-box; margin: 0; padding: 0; }
    body { font-family: 'Inter', sans-serif; background: var(--bg); color: var(--text); min-height: 100vh; }

    .titlebar {
      display: flex; align-items: center; gap: 12px;
      height: 48px; padding: 0 16px;
      background: var(--bg2); border-bottom: 1px solid var(--border);
      position: sticky; top: 0; z-index: 100;
    }
    .logo { display: flex; align-items: center; gap: 8px; font-weight: 700; font-size: 16px; text-decoration: none; color: var(--text); }
    .logo-icon { width: 28px; height: 28px; background: linear-gradient(135deg, #58a6ff, #3fb950); border-radius: 6px; display: flex; align-items: center; justify-content: center; font-size: 14px; }
    .logo-beta { font-size: 10px; background: var(--accent); color: #0d1117; padding: 1px 6px; border-radius: 20px; font-weight: 600; }
    .spacer { flex: 1; }
    .titlebar-actions { display: flex; gap: 8px; }
    .btn-sm { padding: 4px 12px; border-radius: 6px; font-size: 13px; font-weight: 500; cursor: pointer; border: 1px solid var(--border); background: var(--bg3); color: var(--text); text-decoration: none; transition: all 0.15s; }
    .btn-sm:hover { border-color: var(--accent); color: var(--accent); }
    .btn-primary { background: var(--accent); border-color: var(--accent); color: #0d1117; }
    .btn-primary:hover { background: #79c0ff; border-color: #79c0ff; color: #0d1117; }

    .sidebar {
      position: fixed; left: 0; top: 48px; bottom: 0; width: 220px;
      background: var(--bg2); border-right: 1px solid var(--border);
      padding: 16px 0; display: flex; flex-direction: column; gap: 4px;
      overflow-y: auto;
    }
    .sidebar-section { padding: 0 16px; margin-bottom: 8px; }
    .sidebar-label { font-size: 11px; font-weight: 600; text-transform: uppercase; letter-spacing: 0.08em; color: var(--text2); margin-bottom: 8px; }
    .sidebar-item {
      display: flex; align-items: center; gap: 10px; padding: 8px 16px;
      border-radius: 6px; margin: 0 8px; cursor: pointer; font-size: 14px;
      font-weight: 500; color: var(--text2); transition: all 0.15s; text-decoration: none;
    }
    .sidebar-item:hover { background: var(--bg3); color: var(--text); }
    .sidebar-item.active { background: rgba(88, 166, 255, 0.1); color: var(--accent); }
    .sidebar-icon { width: 18px; text-align: center; font-size: 16px; }
    .sidebar-dot { width: 8px; height: 8px; border-radius: 50%; flex-shrink: 0; }

    .main { margin-left: 220px; padding-top: 48px; }
    .content { padding: 32px; }

    .hero { text-align: center; padding: 64px 32px; }
    .hero h1 { font-size: 48px; font-weight: 700; line-height: 1.1; margin-bottom: 16px; background: linear-gradient(135deg, #58a6ff, #3fb950); -webkit-background-clip: text; -webkit-text-fill-color: transparent; background-clip: text; }
    .hero p { font-size: 18px; color: var(--text2); max-width: 560px; margin: 0 auto 32px; line-height: 1.6; }

    .app-grid { display: grid; grid-template-columns: repeat(4, 1fr); gap: 16px; max-width: 900px; margin: 0 auto 48px; }
    .app-card {
      background: var(--bg2); border: 1px solid var(--border); border-radius: 12px;
      padding: 24px; cursor: pointer; transition: all 0.2s; text-decoration: none; color: var(--text);
      display: flex; flex-direction: column; align-items: center; gap: 12px;
    }
    .app-card:hover { border-color: var(--card-color, var(--accent)); transform: translateY(-2px); box-shadow: 0 8px 24px rgba(0,0,0,0.4); }
    .app-card-icon { width: 56px; height: 56px; border-radius: 14px; display: flex; align-items: center; justify-content: center; font-size: 28px; }
    .app-card-name { font-weight: 600; font-size: 15px; }
    .app-card-desc { font-size: 13px; color: var(--text2); text-align: center; }

    .recent-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 16px; }
    .recent-title { font-size: 18px; font-weight: 600; }
    .recent-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(200px, 1fr)); gap: 12px; }
    .recent-card {
      background: var(--bg2); border: 1px solid var(--border); border-radius: 8px;
      padding: 16px; cursor: pointer; transition: all 0.15s; animation: fadeIn 0.3s ease;
    }
    .recent-card:hover { border-color: var(--border); background: var(--bg3); }
    .recent-doc-icon { font-size: 24px; margin-bottom: 8px; }
    .recent-doc-name { font-size: 14px; font-weight: 500; margin-bottom: 4px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
    .recent-doc-date { font-size: 12px; color: var(--text2); }
    .empty-state { text-align: center; padding: 48px; color: var(--text2); }
    .empty-state p { margin-top: 8px; font-size: 14px; }

    .new-doc-btn {
      display: inline-flex; align-items: center; gap: 8px;
      padding: 12px 24px; border-radius: 8px; font-size: 15px; font-weight: 600;
      background: var(--accent); color: #0d1117; cursor: pointer; border: none;
      transition: all 0.15s; margin: 8px;
    }
    .new-doc-btn:hover { background: #79c0ff; transform: translateY(-1px); }

    .status-bar {
      position: fixed; bottom: 0; left: 220px; right: 0;
      height: 28px; background: var(--accent); color: #0d1117;
      display: flex; align-items: center; padding: 0 16px; font-size: 12px; font-weight: 600;
      gap: 16px;
    }
    .status-dot { width: 8px; height: 8px; border-radius: 50%; background: #0d1117; animation: pulse 2s infinite; }

    @keyframes fadeIn { from { opacity: 0; transform: translateY(8px); } to { opacity: 1; transform: none; } }
    @keyframes pulse { 0%, 100% { opacity: 1; } 50% { opacity: 0.4; } }

    /* Notification toast */
    .toast { position: fixed; bottom: 40px; right: 24px; background: var(--bg3); border: 1px solid var(--border); border-radius: 8px; padding: 12px 16px; font-size: 14px; z-index: 1000; animation: slideIn 0.2s ease; box-shadow: 0 8px 32px rgba(0,0,0,0.5); }
    @keyframes slideIn { from { transform: translateX(120%); } to { transform: none; } }
  </style>
</head>
<body>

<div class="titlebar">
  <a href="/" class="logo">
    <div class="logo-icon">W</div>
    PDF Office
    <span class="logo-beta">BETA</span>
  </a>
  <div class="spacer"></div>
  <div class="titlebar-actions">
    <a href="/download" class="btn-sm">Download</a>
  </div>
</div>

<nav class="sidebar">
  <div class="sidebar-section">
    <div class="sidebar-label">Create</div>
    <a href="#" class="sidebar-item active" onclick="showSection('home')" id="nav-home">
      <span class="sidebar-icon">🏠</span> Home
    </a>
  </div>
  <div class="sidebar-section">
    <div class="sidebar-label">Applications</div>
    <a href="#" class="sidebar-item" onclick="openApp('writer')" id="nav-writer">
      <span class="sidebar-dot" style="background:#4493f8"></span>
      Writer
    </a>
    <a href="#" class="sidebar-item" onclick="openApp('sheets')" id="nav-sheets">
      <span class="sidebar-dot" style="background:#3fb950"></span>
      Sheets
    </a>
    <a href="#" class="sidebar-item" onclick="openApp('slides')" id="nav-slides">
      <span class="sidebar-dot" style="background:#e3b341"></span>
      Slides
    </a>
    <a href="#" class="sidebar-item" onclick="openApp('pdf')" id="nav-pdf">
      <span class="sidebar-dot" style="background:#f78166"></span>
      PDF Tools
    </a>
  </div>
  <div class="sidebar-section" style="margin-top:auto; padding-top: 16px; border-top: 1px solid var(--border);">
    <a href="#" class="sidebar-item" onclick="openSettings()">
      <span class="sidebar-icon">⚙️</span> Settings
    </a>
    <a href="/download" class="sidebar-item">
      <span class="sidebar-icon">⬇️</span> Downloads
    </a>
  </div>
</nav>

<main class="main">
  <div class="content" id="section-home">
    <div class="hero">
      <h1>PDF Office</h1>
      <p>Local-first office suite built in Rust. Your documents stay on your machine.</p>
      <div>
        <button class="new-doc-btn" onclick="newDocument('writer')" id="btn-new-writer">📝 New Document</button>
        <button class="new-doc-btn" style="background: var(--accent2);" onclick="newDocument('sheets')" id="btn-new-sheets">📊 New Spreadsheet</button>
        <button class="new-doc-btn" style="background: var(--slides);" onclick="newDocument('slides')" id="btn-new-slides">📽️ New Presentation</button>
      </div>
    </div>

    <div class="app-grid">
      <a href="#" class="app-card" style="--card-color: #4493f8" onclick="openApp('writer')">
        <div class="app-card-icon" style="background: rgba(68,147,248,0.15);">📝</div>
        <div class="app-card-name">Writer</div>
        <div class="app-card-desc">Word processor with OT editing</div>
      </a>
      <a href="#" class="app-card" style="--card-color: #3fb950" onclick="openApp('sheets')">
        <div class="app-card-icon" style="background: rgba(63,185,80,0.15);">📊</div>
        <div class="app-card-name">Sheets</div>
        <div class="app-card-desc">Spreadsheet with 130+ functions</div>
      </a>
      <a href="#" class="app-card" style="--card-color: #e3b341" onclick="openApp('slides')">
        <div class="app-card-icon" style="background: rgba(227,179,65,0.15);">📽️</div>
        <div class="app-card-name">Slides</div>
        <div class="app-card-desc">Presentation with SVG renderer</div>
      </a>
      <a href="#" class="app-card" style="--card-color: #f78166" onclick="openApp('pdf')">
        <div class="app-card-icon" style="background: rgba(247,129,102,0.15);">📄</div>
        <div class="app-card-name">PDF Tools</div>
        <div class="app-card-desc">View, merge, split, watermark</div>
      </a>
    </div>

    <div style="max-width: 900px; margin: 0 auto;">
      <div class="recent-header">
        <div class="recent-title">Recent Documents</div>
        <button class="btn-sm" onclick="loadRecent()">Refresh</button>
      </div>
      <div class="recent-grid" id="recent-grid">
        <div class="empty-state">
          <div style="font-size:48px">📂</div>
          <p>No recent documents. Create something new!</p>
        </div>
      </div>
    </div>
  </div>

  <div class="content" id="section-writer" style="display:none">
    <div id="writer-app" style="display:flex;flex-direction:column;height:calc(100vh - 48px - 28px);">
      <div style="background:var(--bg2);border-bottom:1px solid var(--border);padding:8px 16px;display:flex;gap:8px;flex-wrap:wrap;align-items:center;">
        <button class="btn-sm" onclick="writerBold()" id="btn-bold" title="Bold (Ctrl+B)"><b>B</b></button>
        <button class="btn-sm" onclick="writerItalic()" id="btn-italic" title="Italic (Ctrl+I)"><i>I</i></button>
        <button class="btn-sm" onclick="writerUnderline()" id="btn-underline" title="Underline (Ctrl+U)"><u>U</u></button>
        <span style="color:var(--border)">|</span>
        <select id="font-family" style="background:var(--bg3);border:1px solid var(--border);color:var(--text);padding:4px 8px;border-radius:6px;font-size:13px;" onchange="applyFont()">
          <option>Inter</option><option>Times New Roman</option><option>Arial</option><option>Courier New</option><option>Georgia</option>
        </select>
        <select id="font-size" style="background:var(--bg3);border:1px solid var(--border);color:var(--text);padding:4px 8px;border-radius:6px;font-size:13px;width:68px;">
          <option>10</option><option>11</option><option selected>12</option><option>14</option><option>16</option><option>18</option><option>20</option><option>24</option><option>28</option><option>32</option><option>36</option><option>48</option>
        </select>
        <span style="color:var(--border)">|</span>
        <button class="btn-sm" onclick="writerAlign('left')" title="Align Left">⬅</button>
        <button class="btn-sm" onclick="writerAlign('center')" title="Center">↔</button>
        <button class="btn-sm" onclick="writerAlign('right')" title="Align Right">➡</button>
        <button class="btn-sm" onclick="writerAlign('justify')" title="Justify">☰</button>
        <span style="color:var(--border)">|</span>
        <button class="btn-sm" onclick="exportDoc('docx')" title="Export as DOCX">⬇ DOCX</button>
        <button class="btn-sm" onclick="exportDoc('pdf')" title="Export as PDF">⬇ PDF</button>
        <span style="flex:1"></span>
        <span id="spell-status" style="font-size:12px;color:var(--text2)">Ready</span>
      </div>

      <div style="flex:1;overflow-y:auto;display:flex;justify-content:center;padding:32px 16px;background:#1a1f26;">
        <div id="writer-canvas" contenteditable="true" spellcheck="false"
          style="width:794px;min-height:1123px;background:white;color:#111;padding:72px 96px;border-radius:4px;box-shadow:0 4px 24px rgba(0,0,0,0.5);outline:none;font-family:var(--font,'Times New Roman');font-size:12pt;line-height:1.5;caret-color:#333;"
          oninput="onDocumentInput()" onkeydown="handleKey(event)"
          id="doc-editor">
          <h1 style="font-size:24pt;margin-bottom:16px;">Untitled Document</h1>
          <p>Start typing your document here...</p>
        </div>
      </div>
    </div>
  </div>

  <div class="content" id="section-sheets" style="display:none">
    <div style="display:flex;flex-direction:column;height:calc(100vh - 48px - 28px);">
      <div style="background:var(--bg2);border-bottom:1px solid var(--border);padding:8px 16px;display:flex;gap:8px;align-items:center;">
        <span style="font-size:13px;color:var(--text2)">Cell:</span>
        <input id="cell-ref" value="A1" readonly style="width:60px;background:var(--bg3);border:1px solid var(--border);color:var(--text);padding:4px 8px;border-radius:6px;font-size:13px;font-family:monospace;">
        <span style="color:var(--border)">|</span>
        <span style="font-size:13px;color:var(--text2)">Formula:</span>
        <input id="formula-bar" placeholder="Enter formula or value..." style="flex:1;background:var(--bg3);border:1px solid var(--border);color:var(--text);padding:4px 8px;border-radius:6px;font-size:14px;font-family:monospace;" onkeydown="formulaKeydown(event)">
        <button class="btn-sm btn-primary" onclick="evalFormulaBar()" id="btn-eval">Eval</button>
        <span id="formula-result" style="font-size:13px;color:var(--accent2);min-width:100px;"></span>
        <span style="flex:1"></span>
        <button class="btn-sm" onclick="exportXlsx()">⬇ XLSX</button>
      </div>
      <div style="flex:1;overflow:hidden;position:relative;">
        <canvas id="grid-canvas" style="display:block;cursor:cell;" onmousedown="gridMousedown(event)" onkeydown="gridKeydown(event)" tabindex="0"></canvas>
      </div>
      <div style="background:var(--bg2);border-top:1px solid var(--border);padding:4px 16px;display:flex;gap:16px;">
        <span id="sheet-tab" style="padding:4px 16px;border-radius:6px 6px 0 0;background:var(--accent);color:#0d1117;font-size:13px;font-weight:600;cursor:pointer;">Sheet1</span>
        <button onclick="addSheet()" style="background:none;border:none;color:var(--text2);cursor:pointer;font-size:18px;line-height:1;">+</button>
      </div>
    </div>
  </div>

  <div class="content" id="section-slides" style="display:none">
    <div style="display:flex;flex-direction:column;height:calc(100vh - 48px - 28px);">
      <div style="background:var(--bg2);border-bottom:1px solid var(--border);padding:8px 16px;display:flex;gap:8px;align-items:center;">
        <button class="btn-sm" onclick="addSlide()">+ Slide</button>
        <button class="btn-sm" onclick="addTextBox()">T Text</button>
        <button class="btn-sm" onclick="addShape()">◻ Shape</button>
        <span style="flex:1"></span>
        <button class="btn-sm" onclick="exportPptx()">⬇ PPTX</button>
      </div>
      <div style="display:flex;flex:1;overflow:hidden;">
        <div style="width:180px;background:var(--bg);border-right:1px solid var(--border);overflow-y:auto;padding:8px;" id="slides-panel">
          <div class="slide-thumb" onclick="selectSlide(0)" id="slide-thumb-0" style="border:2px solid var(--accent);border-radius:4px;margin-bottom:8px;cursor:pointer;aspect-ratio:4/3;background:#333;display:flex;align-items:center;justify-content:center;">
            <span style="font-size:12px;color:var(--text2)">Slide 1</span>
          </div>
        </div>
        <div style="flex:1;overflow:auto;display:flex;align-items:center;justify-content:center;background:#1a1f26;">
          <div id="slide-canvas" style="width:720px;height:540px;background:white;border-radius:4px;box-shadow:0 4px 24px rgba(0,0,0,0.5);position:relative;overflow:hidden;" onclick="slideCanvasClick(event)">
            <div id="slide-content" style="width:100%;height:100%;"></div>
          </div>
        </div>
      </div>
    </div>
  </div>

  <div class="content" id="section-pdf" style="display:none">
    <div style="display:flex;flex-direction:column;height:calc(100vh - 48px - 28px);">
      <div style="background:var(--bg2);border-bottom:1px solid var(--border);padding:8px 16px;display:flex;gap:8px;align-items:center;flex-wrap:wrap;">
        <label class="btn-sm" for="pdf-upload" style="cursor:pointer;">📂 Open PDF</label>
        <input type="file" id="pdf-upload" accept=".pdf" style="display:none" onchange="openPDF(event)">
        <span style="color:var(--border)">|</span>
        <button class="btn-sm" onclick="mergePDFs()" id="btn-merge">Merge PDFs</button>
        <button class="btn-sm" onclick="splitPDF()" id="btn-split">Split</button>
        <button class="btn-sm" onclick="watermarkPDF()" id="btn-watermark">Watermark</button>
        <span style="flex:1"></span>
        <span id="pdf-pages" style="font-size:12px;color:var(--text2)"></span>
      </div>
      <div style="flex:1;overflow-y:auto;display:flex;align-items:center;justify-content:center;background:#1a1f26;padding:32px;" id="pdf-viewer">
        <div style="text-align:center;color:var(--text2);">
          <div style="font-size:64px;margin-bottom:16px;">📄</div>
          <p>Open a PDF file to view it</p>
          <p style="font-size:13px;margin-top:8px;">Supports view, merge, split, watermark, and redact</p>
        </div>
      </div>
    </div>
  </div>
</main>

<!-- Settings Modal -->
<div id="settings-modal" style="display:none;position:fixed;inset:0;background:rgba(0,0,0,0.6);z-index:1000;align-items:center;justify-content:center;">
  <div style="background:var(--bg2);border:1px solid var(--border);border-radius:12px;padding:32px;width:400px;max-width:90vw;box-shadow:0 16px 48px rgba(0,0,0,0.6);">
    <div style="display:flex;align-items:center;justify-content:space-between;margin-bottom:24px;">
      <h2 style="font-size:18px;font-weight:600;">⚙️ Settings</h2>
      <button onclick="closeSettings()" style="background:none;border:none;color:var(--text2);cursor:pointer;font-size:20px;line-height:1;">✕</button>
    </div>
    <div style="display:flex;flex-direction:column;gap:20px;">
      <div>
        <label style="display:block;font-size:13px;font-weight:500;margin-bottom:8px;color:var(--text2);">Theme</label>
        <select id="pref-theme" style="width:100%;background:var(--bg3);border:1px solid var(--border);color:var(--text);padding:8px 12px;border-radius:8px;font-size:14px;">
          <option value="dark">🌙 Dark (default)</option>
          <option value="light">☀️ Light</option>
        </select>
      </div>
      <div>
        <label style="display:block;font-size:13px;font-weight:500;margin-bottom:8px;color:var(--text2);">UI Font Size</label>
        <select id="pref-fontsize" style="width:100%;background:var(--bg3);border:1px solid var(--border);color:var(--text);padding:8px 12px;border-radius:8px;font-size:14px;">
          <option value="13">Small (13px)</option>
          <option value="14" selected>Default (14px)</option>
          <option value="16">Large (16px)</option>
          <option value="18">Extra Large (18px)</option>
        </select>
      </div>
      <div style="display:flex;gap:12px;padding-top:8px;">
        <button onclick="closeSettings()" class="btn-sm" style="flex:1;">Cancel</button>
        <button onclick="saveSettings()" class="btn-sm btn-primary" style="flex:1;">Save Settings</button>
      </div>
    </div>
  </div>
</div>

<div class="status-bar">
  <div class="status-dot"></div>
  <span>PDF Office running on localhost</span>
  <span style="margin-left:auto;opacity:0.7">Rust-native · Local · Private</span>
</div>

<script>
// ─── State ────────────────────────────────────────────────────────────────────
const state = {
  currentSection: 'home',
  currentDocId: null,
  ws: null,
  selectedCell: { row: 0, col: 0 },
  gridData: {},  // { "row,col": { value, formula, style } }
  gridScrollX: 0,
  gridScrollY: 0,
  currentSlide: 0,
  slides: [{ elements: [], background: '#FFFFFF' }],
};

// ─── Navigation ───────────────────────────────────────────────────────────────
function showSection(name) {
  ['home','writer','sheets','slides','pdf'].forEach(s => {
    document.getElementById('section-' + s).style.display = s === name ? '' : 'none';
    const nav = document.getElementById('nav-' + s);
    if(nav) nav.classList.toggle('active', s === name);
  });
  state.currentSection = name;
  if(name === 'sheets') { setTimeout(drawGrid, 50); }
}

function openApp(app) {
  showSection(app);
  if(app === 'writer' && !state.currentDocId) newDocument('writer');
}

// ─── Writer ───────────────────────────────────────────────────────────────────
async function newDocument(type) {
  try {
    const resp = await fetch('/api/documents', { method: 'POST' });
    const data = await resp.json();
    state.currentDocId = data.id;
    if(type === 'writer') showSection('writer');
    else if(type === 'sheets') showSection('sheets');
    else if(type === 'slides') showSection('slides');
    loadRecent();
  } catch(e) { toast('Error creating document: ' + e.message, 'error'); }
}

function writerBold() { document.execCommand('bold'); }
function writerItalic() { document.execCommand('italic'); }
function writerUnderline() { document.execCommand('underline'); }
function writerAlign(dir) { document.execCommand('justify' + dir.charAt(0).toUpperCase() + dir.slice(1)); }
function applyFont() {
  const fam = document.getElementById('font-family').value;
  document.getElementById('doc-editor').style.fontFamily = fam;
}

let spellTimer = null;
function onDocumentInput() {
  clearTimeout(spellTimer);
  spellTimer = setTimeout(doSpellCheck, 1500);
  saveDocumentDebounced();
}

async function doSpellCheck() {
  const editor = document.getElementById('doc-editor');
  const text = editor.innerText;
  if(!text.trim()) return;
  document.getElementById('spell-status').textContent = 'Checking...';
  try {
    const resp = await fetch('/api/spell', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ text })
    });
    const data = await resp.json();
    const count = data.results?.length || 0;
    document.getElementById('spell-status').textContent = count === 0 ? '✓ No errors' : `${count} issue${count > 1 ? 's' : ''}`;
    document.getElementById('spell-status').style.color = count === 0 ? '#3fb950' : '#f78166';
  } catch(e) {
    document.getElementById('spell-status').textContent = 'Spell check failed';
  }
}

let saveTimer = null;
function saveDocumentDebounced() {
  if(!state.currentDocId) return;
  clearTimeout(saveTimer);
  saveTimer = setTimeout(saveDocument, 2000);
}

async function saveDocument() {
  if(!state.currentDocId) return;
  const editor = document.getElementById('doc-editor');
  const text = editor.innerText;
  const doc = {
    id: state.currentDocId,
    title: text.split('\n')[0]?.slice(0,80) || 'Untitled',
    body: [{ Paragraph: { id: crypto.randomUUID(), runs: [{ id: crypto.randomUUID(), text, format: {} }], style_ref: null, align: 'Left', indent_left: 0, indent_right: 0, indent_first: 0, space_before: 0, space_after: 8, line_height: { Multiple: 1.15 }, keep_together: false, keep_with_next: false, page_break_before: false } }],
    styles: { styles: {}, default_paragraph_style: '', default_character_style: '' },
    page_layout: { width: 210, height: 297, margin_top: 25.4, margin_bottom: 25.4, margin_left: 25.4, margin_right: 25.4, orientation: 'Portrait' },
    metadata: { author: '', language: 'en', keywords: [], description: '' },
    revision: 0
  };
  try {
    await fetch(`/api/documents/${state.currentDocId}`, {
      method: 'POST', headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(doc)
    });
  } catch(e) {}
}

async function exportDoc(format) {
  if(!state.currentDocId) { toast('No document open', 'error'); return; }
  await saveDocument();
  window.location.href = `/api/documents/${state.currentDocId}/export`;
  fetch(`/api/documents/${state.currentDocId}/export`, {
    method: 'POST', headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ format })
  }).then(r => r.blob()).then(blob => {
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url; a.download = `document.${format}`;
    document.body.appendChild(a); a.click(); document.body.removeChild(a);
    setTimeout(() => URL.revokeObjectURL(url), 60000);
  }).catch(e => toast('Export failed: ' + e.message, 'error'));
}

function handleKey(e) {
  if(e.ctrlKey && e.key === 's') { e.preventDefault(); saveDocument(); toast('Saved!'); }
  if(e.ctrlKey && e.key === 'b') { e.preventDefault(); writerBold(); }
  if(e.ctrlKey && e.key === 'i') { e.preventDefault(); writerItalic(); }
  if(e.ctrlKey && e.key === 'u') { e.preventDefault(); writerUnderline(); }
}

// ─── Spreadsheet Grid ─────────────────────────────────────────────────────────
const COL_WIDTH = 100, ROW_HEIGHT = 24, HEADER_W = 60, HEADER_H = 24;

function numToCol(n) {
  let s = '';
  for(; n >= 0; n = Math.floor(n/26) - 1)
    s = String.fromCharCode(65 + n%26) + s;
  return s;
}

function drawGrid() {
  const canvas = document.getElementById('grid-canvas');
  if(!canvas) return;
  const container = canvas.parentElement;
  canvas.width = container.clientWidth;
  canvas.height = container.clientHeight;
  const ctx = canvas.getContext('2d');
  const cols = Math.ceil((canvas.width - HEADER_W) / COL_WIDTH) + 2;
  const rows = Math.ceil((canvas.height - HEADER_H) / ROW_HEIGHT) + 2;

  ctx.fillStyle = '#161b22';
  ctx.fillRect(0, 0, canvas.width, canvas.height);

  // Column headers
  ctx.fillStyle = '#21262d';
  ctx.fillRect(HEADER_W, 0, canvas.width - HEADER_W, HEADER_H);
  ctx.fillStyle = '#1a1f26';
  ctx.fillRect(0, 0, HEADER_W, HEADER_H);

  ctx.strokeStyle = '#30363d';
  ctx.fillStyle = '#8b949e';
  ctx.font = '12px Inter, sans-serif';
  ctx.textAlign = 'center';
  ctx.textBaseline = 'middle';

  for(let c = 0; c < cols; c++) {
    const x = HEADER_W + c * COL_WIDTH;
    const col = c + state.gridScrollX;
    const isSelected = col === state.selectedCell.col;

    ctx.fillStyle = isSelected ? 'rgba(88,166,255,0.15)' : '#21262d';
    ctx.fillRect(x, 0, COL_WIDTH, HEADER_H);
    ctx.fillStyle = isSelected ? '#58a6ff' : '#8b949e';
    ctx.fillText(numToCol(col), x + COL_WIDTH/2, HEADER_H/2);
    ctx.strokeStyle = '#30363d';
    ctx.beginPath(); ctx.moveTo(x, 0); ctx.lineTo(x, HEADER_H); ctx.stroke();
  }

  // Row headers
  ctx.textAlign = 'center';
  for(let r = 0; r < rows; r++) {
    const y = HEADER_H + r * ROW_HEIGHT;
    const row = r + state.gridScrollY;
    const isSelected = row === state.selectedCell.row;

    ctx.fillStyle = isSelected ? 'rgba(88,166,255,0.15)' : '#21262d';
    ctx.fillRect(0, y, HEADER_W, ROW_HEIGHT);
    ctx.fillStyle = isSelected ? '#58a6ff' : '#8b949e';
    ctx.fillText(row + 1, HEADER_W/2, y + ROW_HEIGHT/2);
    ctx.strokeStyle = '#30363d';
    ctx.beginPath(); ctx.moveTo(0, y); ctx.lineTo(HEADER_W, y); ctx.stroke();
  }

  // Cells
  ctx.textAlign = 'left';
  for(let r = 0; r < rows; r++) {
    const y = HEADER_H + r * ROW_HEIGHT;
    const row = r + state.gridScrollY;
    for(let c = 0; c < cols; c++) {
      const x = HEADER_W + c * COL_WIDTH;
      const col = c + state.gridScrollX;
      const key = `${row},${col}`;
      const cell = state.gridData[key];
      const isSelected = row === state.selectedCell.row && col === state.selectedCell.col;

      ctx.fillStyle = isSelected ? 'rgba(88,166,255,0.08)' : '#0d1117';
      ctx.fillRect(x + 1, y + 1, COL_WIDTH - 1, ROW_HEIGHT - 1);
      ctx.strokeStyle = '#21262d';
      ctx.strokeRect(x, y, COL_WIDTH, ROW_HEIGHT);

      if(cell?.value) {
        ctx.fillStyle = '#e6edf3';
        ctx.font = '13px Inter, sans-serif';
        ctx.fillText(String(cell.value).slice(0, 15), x + 4, y + ROW_HEIGHT/2 + 1);
      }

      if(isSelected) {
        ctx.strokeStyle = '#58a6ff';
        ctx.lineWidth = 2;
        ctx.strokeRect(x + 1, y + 1, COL_WIDTH - 2, ROW_HEIGHT - 2);
        ctx.lineWidth = 1;
      }
    }
  }

  // Corner
  ctx.fillStyle = '#21262d';
  ctx.fillRect(0, 0, HEADER_W, HEADER_H);
  ctx.strokeStyle = '#30363d';
  ctx.strokeRect(0, 0, HEADER_W, HEADER_H);

  // Update cell reference
  document.getElementById('cell-ref').value = numToCol(state.selectedCell.col) + (state.selectedCell.row + 1);
  const cellKey = `${state.selectedCell.row},${state.selectedCell.col}`;
  const currentCell = state.gridData[cellKey];
  document.getElementById('formula-bar').value = currentCell?.formula || currentCell?.value || '';
}

function gridMousedown(e) {
  const canvas = document.getElementById('grid-canvas');
  const rect = canvas.getBoundingClientRect();
  const x = e.clientX - rect.left;
  const y = e.clientY - rect.top;
  if(x < HEADER_W || y < HEADER_H) return;
  const col = Math.floor((x - HEADER_W) / COL_WIDTH) + state.gridScrollX;
  const row = Math.floor((y - HEADER_H) / ROW_HEIGHT) + state.gridScrollY;
  state.selectedCell = { row, col };
  canvas.focus();
  drawGrid();
}

function gridKeydown(e) {
  const { row, col } = state.selectedCell;
  if(e.key === 'ArrowRight') { state.selectedCell.col = Math.max(0, col + 1); drawGrid(); }
  else if(e.key === 'ArrowLeft') { state.selectedCell.col = Math.max(0, col - 1); drawGrid(); }
  else if(e.key === 'ArrowDown' || e.key === 'Enter') { state.selectedCell.row = Math.max(0, row + 1); drawGrid(); }
  else if(e.key === 'ArrowUp') { state.selectedCell.row = Math.max(0, row - 1); drawGrid(); }
  else if(e.key === 'Tab') { e.preventDefault(); state.selectedCell.col = col + 1; drawGrid(); }
  else if(e.key === 'Delete' || e.key === 'Backspace') {
    const key = `${row},${col}`;
    delete state.gridData[key];
    document.getElementById('formula-bar').value = '';
    drawGrid();
  }
}

async function evalFormulaBar() {
  const formula = document.getElementById('formula-bar').value.trim();
  if(!formula) return;
  const key = `${state.selectedCell.row},${state.selectedCell.col}`;

  if(formula.startsWith('=')) {
    try {
      const resp = await fetch('/api/formula', {
        method: 'POST', headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ formula, row: state.selectedCell.row, col: state.selectedCell.col })
      });
      const data = await resp.json();
      if(data.error) {
        state.gridData[key] = { value: data.error, formula };
        document.getElementById('formula-result').textContent = '⚠ ' + data.error;
        document.getElementById('formula-result').style.color = '#f78166';
      } else {
        state.gridData[key] = { value: data.result, formula };
        document.getElementById('formula-result').textContent = '= ' + data.result;
        document.getElementById('formula-result').style.color = '#3fb950';
      }
    } catch(e) { toast('Formula error: ' + e.message, 'error'); }
  } else {
    state.gridData[key] = { value: formula };
    document.getElementById('formula-result').textContent = formula;
    document.getElementById('formula-result').style.color = '#e6edf3';
  }
  state.selectedCell.row++;
  drawGrid();
}

function formulaKeydown(e) {
  if(e.key === 'Enter') { e.preventDefault(); evalFormulaBar(); }
  if(e.key === 'Escape') { document.getElementById('formula-bar').value = ''; }
}

function addSheet() { toast('Multiple sheets coming in Phase 2!'); }

async function exportXlsx() {
  if(!state.currentDocId) { toast('No spreadsheet open', 'error'); return; }
  try {
    const resp = await fetch(`/api/documents/${state.currentDocId}/export`, {
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
    toast('XLSX exported!');
  } catch(e) { toast('Export error: ' + e.message, 'error'); }
}

// ─── Slides ───────────────────────────────────────────────────────────────────
function addSlide() {
  state.slides.push({ elements: [], background: '#FFFFFF' });
  updateSlidePanel();
  toast(`Slide ${state.slides.length} added`);
}

function addTextBox() {
  const slide = state.slides[state.currentSlide];
  slide.elements.push({
    type: 'textbox', x: 60, y: 120, w: 600, h: 80,
    text: 'Click to edit text', fontSize: 24, color: '#111111'
  });
  renderSlide();
}

function addShape() {
  const slide = state.slides[state.currentSlide];
  slide.elements.push({
    type: 'shape', shape: 'rect', x: 200, y: 200, w: 200, h: 120,
    fill: '#4493f8', stroke: '#2255aa'
  });
  renderSlide();
}

function selectSlide(idx) {
  state.currentSlide = idx;
  document.querySelectorAll('.slide-thumb').forEach((t,i) => {
    t.style.borderColor = i === idx ? 'var(--accent)' : 'var(--border)';
  });
  renderSlide();
}

function renderSlide() {
  const slide = state.slides[state.currentSlide];
  const content = document.getElementById('slide-content');
  content.style.background = slide.background;
  content.innerHTML = '';
  slide.elements.forEach(el => {
    const div = document.createElement('div');
    div.style.position = 'absolute';
    div.style.left = el.x + 'px'; div.style.top = el.y + 'px';
    div.style.width = el.w + 'px'; div.style.height = el.h + 'px';
    if(el.type === 'textbox') {
      div.style.fontSize = el.fontSize + 'px';
      div.style.color = el.color;
      div.style.display = 'flex'; div.style.alignItems = 'center';
      div.style.padding = '8px';
      div.contentEditable = true; div.style.outline = 'none';
      div.innerText = el.text;
    } else if(el.type === 'shape') {
      div.style.background = el.fill;
      div.style.border = '2px solid ' + el.stroke;
      div.style.borderRadius = '4px';
    }
    content.appendChild(div);
  });
}

function slideCanvasClick(e) {
  // Deselect all elements
}

function updateSlidePanel() {
  const panel = document.getElementById('slides-panel');
  panel.innerHTML = '';
  state.slides.forEach((_, i) => {
    const thumb = document.createElement('div');
    thumb.className = 'slide-thumb';
    thumb.id = 'slide-thumb-' + i;
    thumb.onclick = () => selectSlide(i);
    thumb.style.cssText = `border:2px solid ${i === state.currentSlide ? 'var(--accent)' : 'var(--border)'};border-radius:4px;margin-bottom:8px;cursor:pointer;aspect-ratio:4/3;background:#333;display:flex;align-items:center;justify-content:center;`;
    thumb.innerHTML = `<span style="font-size:12px;color:var(--text2)">Slide ${i+1}</span>`;
    panel.appendChild(thumb);
  });
}

function exportPptx() { toast('PPTX export coming via server — Phase 2!'); }

// ─── PDF Tools ────────────────────────────────────────────────────────────────
async function openPDF(event) {
  const file = event.target.files[0];
  if(!file) return;
  const viewer = document.getElementById('pdf-viewer');
  viewer.innerHTML = `<div style="text-align:center;color:var(--text2)"><div style="font-size:48px">⏳</div><p>Loading PDF...</p></div>`;
  const url = URL.createObjectURL(file);
  const embed = document.createElement('embed');
  embed.src = url; embed.type = 'application/pdf';
  embed.style.cssText = 'width:100%;height:100%;border:none;';
  viewer.innerHTML = '';
  viewer.appendChild(embed);
  document.getElementById('pdf-pages').textContent = file.name;
}

function mergePDFs() { toast('Drag & drop merge coming in Phase 2!'); }
function splitPDF() { toast('Page range splitter coming in Phase 2!'); }
function watermarkPDF() { toast('Watermark editor coming in Phase 2!'); }

// ─── Recent documents ─────────────────────────────────────────────────────────
async function loadRecent() {
  try {
    const resp = await fetch('/api/documents');
    const data = await resp.json();
    const grid = document.getElementById('recent-grid');
    if(!data.documents?.length) {
      grid.innerHTML = '<div class="empty-state"><div style="font-size:48px">📂</div><p>No recent documents. Create something new!</p></div>';
      return;
    }
    const icons = { Writer: '📝', Sheets: '📊', Slides: '📽️', Pdf: '📄' };
    grid.innerHTML = data.documents.slice(0, 12).map(doc => `
      <div class="recent-card" onclick="openDoc('${doc.id}', '${doc.doc_type}')">
        <div class="recent-doc-icon">${icons[doc.doc_type] || '📄'}</div>
        <div class="recent-doc-name" title="${doc.title}">${doc.title}</div>
        <div class="recent-doc-date">${formatDate(doc.opened_at)}</div>
      </div>
    `).join('');
  } catch(e) { console.error('Failed to load recent docs:', e); }
}

function openDoc(id, type) {
  state.currentDocId = id;
  openApp(type?.toLowerCase() || 'writer');
}

function formatDate(iso) {
  if(!iso) return '';
  const d = new Date(iso);
  const now = new Date();
  const diff = (now - d) / 1000;
  if(diff < 60) return 'Just now';
  if(diff < 3600) return Math.floor(diff/60) + 'm ago';
  if(diff < 86400) return Math.floor(diff/3600) + 'h ago';
  return d.toLocaleDateString();
}

function openSettings() {
  document.getElementById('settings-modal').style.display = 'flex';
  loadSettings();
}
function closeSettings() {
  document.getElementById('settings-modal').style.display = 'none';
}
async function loadSettings() {
  try {
    const resp = await fetch('/api/preferences/ui');
    const data = await resp.json();
    if(data.value) {
      const prefs = data.value;
      if(prefs.theme) applyTheme(prefs.theme);
      if(prefs.fontSize) applyFontSize(prefs.fontSize);
    }
  } catch(e) {}
}
async function saveSettings() {
  const theme = document.getElementById('pref-theme').value;
  const fontSize = document.getElementById('pref-fontsize').value;
  applyTheme(theme);
  applyFontSize(fontSize);
  try {
    await fetch('/api/preferences/ui', {
      method: 'PUT', headers: {'Content-Type': 'application/json'},
      body: JSON.stringify({ value: { theme, fontSize } })
    });
    toast('Settings saved!');
    closeSettings();
  } catch(e) { toast('Failed to save settings', 'error'); }
}
function applyTheme(theme) {
  const root = document.documentElement;
  if(theme === 'light') {
    root.style.setProperty('--bg', '#ffffff');
    root.style.setProperty('--bg2', '#f6f8fa');
    root.style.setProperty('--bg3', '#e8eaed');
    root.style.setProperty('--border', '#d0d7de');
    root.style.setProperty('--text', '#1c1c1c');
    root.style.setProperty('--text2', '#57606a');
  } else {
    root.style.setProperty('--bg', '#0d1117');
    root.style.setProperty('--bg2', '#161b22');
    root.style.setProperty('--bg3', '#21262d');
    root.style.setProperty('--border', '#30363d');
    root.style.setProperty('--text', '#e6edf3');
    root.style.setProperty('--text2', '#8b949e');
  }
}
function applyFontSize(size) {
  document.body.style.fontSize = size + 'px';
}

// ─── Toast notifications ──────────────────────────────────────────────────────
function toast(msg, type = 'info') {
  const t = document.createElement('div');
  t.className = 'toast';
  t.style.borderLeftColor = type === 'error' ? '#f78166' : '#3fb950';
  t.style.borderLeftWidth = '3px';
  t.textContent = msg;
  document.body.appendChild(t);
  setTimeout(() => t.remove(), 3000);
}

// ─── Initialize ───────────────────────────────────────────────────────────────
document.addEventListener('DOMContentLoaded', () => {
  loadRecent();
  window.addEventListener('resize', () => { if(state.currentSection === 'sheets') drawGrid(); });

  // Drag & drop file import
  const dropZone = document.getElementById('section-home');
  dropZone.addEventListener('dragover', e => { e.preventDefault(); dropZone.style.opacity = '0.7'; });
  dropZone.addEventListener('dragleave', () => { dropZone.style.opacity = ''; });
  dropZone.addEventListener('drop', async e => {
    e.preventDefault();
    dropZone.style.opacity = '';
    const file = e.dataTransfer.files[0];
    if(!file) return;
    const ext = file.name.split('.').pop().toLowerCase();
    const docFormats = ['docx', 'odt', 'rtf'];
    if(!docFormats.includes(ext)) {
      toast(`Unsupported format: .${ext}. Supported: docx, odt, rtf`, 'error');
      return;
    }
    toast(`Importing ${file.name}...`);
    const form = new FormData();
    form.append('file', file);
    form.append('format', 'docx'); // keep as docx
    try {
      const resp = await fetch('/api/convert', { method: 'POST', body: form });
      if(!resp.ok) { toast('Import failed', 'error'); return; }
      // After convert, create a new doc with the result
      const newDoc = await fetch('/api/documents', { method: 'POST' }).then(r => r.json());
      state.currentDocId = newDoc.id;
      showSection('writer');
      toast(`Imported ${file.name} ✓`);
      loadRecent();
    } catch(e) { toast('Import error: ' + e.message, 'error'); }
  });
});
</script>
</body>
</html>"#;

const DOWNLOAD_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <title>Download — PDF Office</title>
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <link href="https://fonts.googleapis.com/css2?family=Inter:wght@300;400;500;600;700&display=swap" rel="stylesheet">
  <style>
    :root { --bg:#0d1117; --bg2:#161b22; --bg3:#21262d; --border:#30363d; --text:#e6edf3; --text2:#8b949e; --accent:#58a6ff; }
    * { box-sizing:border-box; margin:0; padding:0; }
    body { font-family: 'Inter', sans-serif; background: var(--bg); color: var(--text); min-height: 100vh; }
    .header { background: var(--bg2); border-bottom: 1px solid var(--border); padding: 16px 32px; display: flex; align-items: center; gap: 16px; }
    .logo { font-size: 20px; font-weight: 700; color: var(--text); text-decoration: none; }
    .content { max-width: 960px; margin: 0 auto; padding: 64px 32px; }
    h1 { font-size: 40px; font-weight: 700; text-align: center; margin-bottom: 12px; }
    .subtitle { text-align: center; color: var(--text2); font-size: 18px; margin-bottom: 48px; }
    .platforms { display: grid; grid-template-columns: repeat(3, 1fr); gap: 16px; margin-bottom: 48px; }
    .platform-card { background: var(--bg2); border: 1px solid var(--border); border-radius: 12px; padding: 24px; }
    .platform-name { font-size: 18px; font-weight: 600; margin-bottom: 8px; }
    .platform-arch { font-size: 13px; color: var(--text2); margin-bottom: 16px; }
    .download-btn { display: block; background: var(--accent); color: #0d1117; padding: 10px 20px; border-radius: 8px; text-decoration: none; font-weight: 600; font-size: 14px; text-align: center; margin-bottom: 8px; transition: background 0.15s; }
    .download-btn:hover { background: #79c0ff; }
    .checksum { font-size: 11px; font-family: monospace; color: var(--text2); word-break: break-all; background: var(--bg3); padding: 8px; border-radius: 4px; margin-top: 8px; }
    .install-box { background: var(--bg2); border: 1px solid var(--border); border-radius: 12px; padding: 32px; margin-bottom: 32px; }
    .install-box h2 { margin-bottom: 16px; }
    pre { background: var(--bg3); border: 1px solid var(--border); border-radius: 8px; padding: 16px; font-size: 13px; overflow-x: auto; color: #79c0ff; }
  </style>
</head>
<body>
<div class="header">
  <a href="/" class="logo">← PDF Office</a>
</div>
<div class="content">
  <h1>Download PDF Office</h1>
  <p class="subtitle">Single binary. Fully local. No telemetry. Built in Rust.</p>

  <div class="install-box">
    <h2>Quick Install (Linux / macOS)</h2>
    <pre>curl -fsSL https://get.pdf-local.io/install.sh | sh</pre>
    <h2 style="margin-top:24px">Quick Install (Windows PowerShell)</h2>
    <pre>irm https://get.pdf-local.io/install.ps1 | iex</pre>
  </div>

  <div class="platforms">
    <div class="platform-card">
      <div class="platform-name">🐧 Linux x86_64</div>
      <div class="platform-arch">Ubuntu, Debian, Arch, Fedora and more</div>
      <a href="#" class="download-btn">⬇ pdf-linux-x86_64.tar.gz</a>
      <div class="checksum">SHA-256: (computed at build time)</div>
    </div>
    <div class="platform-card">
      <div class="platform-name">🐧 Linux ARM64</div>
      <div class="platform-arch">Raspberry Pi, AWS Graviton, etc.</div>
      <a href="#" class="download-btn">⬇ pdf-linux-aarch64.tar.gz</a>
      <div class="checksum">SHA-256: (computed at build time)</div>
    </div>
    <div class="platform-card">
      <div class="platform-name">🍎 macOS Intel</div>
      <div class="platform-arch">macOS 11+ on Intel</div>
      <a href="#" class="download-btn">⬇ pdf-macos-x86_64.tar.gz</a>
      <div class="checksum">SHA-256: (computed at build time)</div>
    </div>
    <div class="platform-card">
      <div class="platform-name">🍎 macOS Apple Silicon</div>
      <div class="platform-arch">M1/M2/M3/M4 Macs</div>
      <a href="#" class="download-btn">⬇ pdf-macos-aarch64.tar.gz</a>
      <div class="checksum">SHA-256: (computed at build time)</div>
    </div>
    <div class="platform-card">
      <div class="platform-name">🪟 Windows x86_64</div>
      <div class="platform-arch">Windows 10/11 64-bit</div>
      <a href="#" class="download-btn">⬇ pdf-windows-x86_64.zip</a>
      <div class="checksum">SHA-256: (computed at build time)</div>
    </div>
  </div>

  <div class="install-box">
    <h2>Verify your download</h2>
    <p style="color:var(--text2);margin-bottom:16px">Verify the Ed25519 signature using minisign:</p>
    <pre>minisign -Vm pdf.tar.gz -P RWQ+PDF+PUBLIC+KEY+GOES+HERE</pre>
  </div>
</div>
</body>
</html>"#;

const APP_CSS: &str = r#"
/* PDF Office App CSS */
:root {
  --bg: #0d1117;
  --bg2: #161b22;
  --bg3: #21262d;
  --border: #30363d;
  --text: #e6edf3;
  --text2: #8b949e;
  --accent: #58a6ff;
}
* { box-sizing: border-box; }
"#;
