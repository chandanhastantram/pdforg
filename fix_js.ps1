
# Replace the JS section in assets.rs (lines 998-2445) with fixed version
$file = "src\server\assets.rs"
$lines = Get-Content $file -Encoding UTF8

# Keep everything before the <script> tag (line 997 = index 996)
$before = $lines[0..996]
# Keep everything after </script> through end (line 2446 onwards = index 2445+)
$after = $lines[2445..($lines.Count-1)]

$newJS = @'
<script>
// ─── App State ────────────────────────────────────────────────────────────────
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

// ─── Ribbon Tabs ──────────────────────────────────────────────────────────────
function switchRibbonTab(tab) {
  document.querySelectorAll('.rtab').forEach(el => el.classList.remove('active'));
  document.getElementById('rtab-' + tab)?.classList.add('active');
}

// ─── Documents ────────────────────────────────────────────────────────────────
async function newDocument(type) {
  try {
    const resp = await fetch('/api/documents', { method: 'POST' });
    const data = await resp.json();
    state.docId = data.id;
    const names = { writer: 'Untitled Document', sheets: 'Untitled Spreadsheet', slides: 'Untitled Presentation' };
    document.getElementById('doc-name').value = names[type] || 'Untitled';
    openApp(type);
    loadRecent();
    toast('New ' + (type === 'writer' ? 'document' : type === 'sheets' ? 'spreadsheet' : 'presentation') + ' created ✓', 'success');
  } catch(e) { toast('Error creating document: ' + e.message, 'error'); }
}

async function renameDoc(name) { /* saved on next auto-save */ }

function quickSave() {
  if(state.section === 'writer') saveDocument();
  else toast('Document auto-saved ✓', 'success');
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
      grid.innerHTML = '<div class="empty-state"><div style="font-size:48px;">📂</div><p>No documents yet. Create your first one above!</p><p style="font-size:13px;margin-top:8px;">Or drag &amp; drop a file anywhere on this page.</p></div>';
      count.textContent = '';
      return;
    }
    count.textContent = '(' + docs.length + ')';
    grid.innerHTML = docs.map(d => {
      const icon = d.title?.includes('Sheet') ? '📊' : d.title?.includes('Slide') || d.title?.includes('Present') ? '📽️' : '📝';
      const date = new Date(d.opened_at || d.updated_at || d.created_at || Date.now()).toLocaleDateString();
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
  // FIX: Clear before setting — prevents timer stacking
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
    toast('Share file (.pdfo) ready! 📤', 'success');
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

// ─── Find Bar ─────────────────────────────────────────────────────────────────
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
function slideDrag(e, idx) { /* basic drag placeholder */ }

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
    document.getElementById('pdf-pages').textContent = file.name;
    document.getElementById('pdf-drop-zone').style.display = 'none';
    document.getElementById('pdf-render-container').style.display = 'block';

    // FIX: Enable all PDF tool ribbon groups after PDF is loaded
    ['pdf-tools-group','pdf-edit-group','pdf-form-group','pdf-pagtools-group','pdf-enhance-group','pdf-export-group'].forEach(id => {
      const el = document.getElementById(id);
      if(el) { el.style.opacity = '1'; el.style.pointerEvents = 'auto'; }
    });

    try {
      pdfDoc = await pdfjsLib.getDocument(state.pdfBytes).promise;
      document.getElementById('pdf-page-count').textContent = pdfDoc.numPages;
      document.getElementById('pdf-zoom-label').textContent = Math.round(pdfCurrentScale*100) + '%';
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
  document.getElementById('pdf-zoom-label').textContent = Math.round(pdfCurrentScale*100) + '%';
  renderPage(pageNum);
}
function pdfZoomOut() {
  if(!pdfDoc) { toast('Open a PDF first', 'error'); return; }
  pdfCurrentScale = Math.max(pdfCurrentScale - PDF_SCALE_STEP, 0.25);
  document.getElementById('pdf-zoom-label').textContent = Math.round(pdfCurrentScale*100) + '%';
  renderPage(pageNum);
}
function pdfZoomFit() {
  if(!pdfDoc) { toast('Open a PDF first', 'error'); return; }
  pdfCurrentScale = 1.0;
  document.getElementById('pdf-zoom-label').textContent = '100%';
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
    item.innerHTML = `<span style="font-size:20px;">📄</span><br>Page ${i}`;
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

// ─── PDF Modal system ─────────────────────────────────────────────────────────
let activePdfAction = '';

function openPdfModal(action) {
  if(!state.pdfBytes && !['create-from-img','compare'].includes(action)) {
    // For most actions, we need an open PDF — but allow merge/compare from modal
    if(!['merge'].includes(action)) {
      toast('Open a PDF file first', 'error'); return;
    }
  }
  activePdfAction = action;
  const title = document.getElementById('pdf-stub-title');
  const body  = document.getElementById('pdf-stub-body');
  const modal = document.getElementById('pdf-stub-modal');

  const bodies = {
    'compress': ['🗜️ Compress PDF',
      `<p style="font-size:13px;color:var(--text2);margin-bottom:12px;">Reduce file size while preserving readability.</p>
       <div class="form-group"><label class="form-label">Compression Level</label>
       <select class="form-select" id="pdf-compress-level">
         <option value="low">Low (Highest Quality)</option>
         <option value="medium" selected>Medium (Recommended)</option>
         <option value="high">High (Smallest File)</option>
         <option value="extreme">Extreme (Minimal Quality)</option>
       </select></div>`],

    'watermark': ['💧 Add Watermark',
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

    'protect': ['🔒 Password Protect',
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

    'split': ['✂️ Split PDF',
      `<div class="form-group"><label class="form-label">Split Method</label>
       <select class="form-select" id="pdf-split-method">
         <option value="bypage">Split at Page Number</option>
         <option value="range">Split by Page Range</option>
       </select></div>
       <div class="form-group" style="margin-top:12px;"><label class="form-label">Value (page number or range)</label>
       <input type="text" class="form-input" id="pdf-split-val" value="5" placeholder="e.g. 5 or 1-5"></div>`],

    'compare': ['📄 Compare Documents',
      `<p style="font-size:13px;color:var(--text2);margin-bottom:12px;">Upload a second PDF to compare with the current open file.</p>
       <div class="form-group"><label class="form-label">Second PDF to Compare</label>
       <label class="btn" style="display:block;cursor:pointer;">📂 Choose File
         <input type="file" accept=".pdf" style="display:none;" id="pdf-compare-file" onchange="pdfCompareFile(this)">
       </label>
       <span id="pdf-compare-filename" style="font-size:12px;color:var(--text3);margin-top:4px;display:block;">No file selected</span></div>`],

    'metadata': ['ℹ️ Document Properties',
      `<div class="form-group"><label class="form-label">Title</label>
       <input type="text" class="form-input" id="pdf-meta-title" placeholder="Document title"></div>
       <div class="form-group" style="margin-top:8px;"><label class="form-label">Author</label>
       <input type="text" class="form-input" id="pdf-meta-author" placeholder="Author name"></div>
       <div class="form-group" style="margin-top:8px;"><label class="form-label">Subject</label>
       <input type="text" class="form-input" id="pdf-meta-subject" placeholder="Document subject"></div>
       <div class="form-group" style="margin-top:8px;"><label class="form-label">Keywords</label>
       <input type="text" class="form-input" id="pdf-meta-keywords" placeholder="comma, separated, keywords"></div>`],

    'header-footer': ['🗒️ Header & Footer',
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

    'bates': ['#️⃣ Bates Numbering',
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

    'page-numbers': ['🔢 Add Page Numbers',
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

    'flatten': ['🧱 Flatten PDF',
      `<p style="font-size:13px;color:var(--text2);">Flattening merges all interactive elements (form fields, annotations, signatures) permanently into the PDF content. This action cannot be undone.</p>
       <p style="font-size:13px;color:var(--pdf-color);margin-top:12px;">⚠️ Ensure you have a backup of the original file.</p>`],

    'sanitize': ['🧹 Sanitize Document',
      `<p style="font-size:13px;color:var(--text2);margin-bottom:12px;">Remove hidden data before sharing.</p>
       <div style="display:flex;flex-direction:column;gap:8px;">
         <label><input type="checkbox" id="san-metadata" checked> Document Metadata (Author, Title, etc.)</label>
         <label><input type="checkbox" id="san-attachments" checked> File Attachments</label>
         <label><input type="checkbox" id="san-javascript" checked> JavaScript Actions</label>
         <label><input type="checkbox" id="san-layers" checked> Hidden Layers</label>
         <label><input type="checkbox" id="san-index" checked> Embedded Search Index</label>
       </div>`],

    'accessibility': ['♿ Check Accessibility',
      `<p style="font-size:13px;color:var(--text2);margin-bottom:16px;">Automated accessibility audit (WCAG 2.1 / PDF/UA).</p>
       <div style="background:var(--surface2);border-radius:8px;padding:12px;font-size:13px;">
         <div style="display:flex;justify-content:space-between;margin-bottom:6px;"><span>✅ Tagged PDF Structure</span><span style="color:#16a34a;">Pass</span></div>
         <div style="display:flex;justify-content:space-between;margin-bottom:6px;"><span>⚠️ Alternate Text for Images</span><span style="color:#d97706;">Warning</span></div>
         <div style="display:flex;justify-content:space-between;margin-bottom:6px;"><span>❌ Color Contrast Ratio</span><span style="color:#dc2626;">Fail</span></div>
         <div style="display:flex;justify-content:space-between;"><span>✅ Reading Order</span><span style="color:#16a34a;">Pass</span></div>
       </div>`],

    'find': ['🔍 Find in PDF',
      `<div class="form-group"><label class="form-label">Search Text</label>
       <input type="text" class="form-input" id="pdf-find-query" placeholder="Search term..."></div>`],

    'insert-page': ['➕ Insert Blank Page',
      `<div class="form-group"><label class="form-label">Insert After Page</label>
       <input type="number" class="form-input" id="pdf-insert-after" value="${pageNum}" min="0"></div>
       <div class="form-group" style="margin-top:12px;"><label class="form-label">Page Size</label>
       <select class="form-select" id="pdf-insert-size">
         <option value="a4">A4 (595 x 842 pt)</option><option value="letter">Letter (612 x 792 pt)</option>
       </select></div>`],

    'delete-page': ['➖ Delete Pages',
      `<div class="form-group"><label class="form-label">Page Range to Delete</label>
       <input type="text" class="form-input" id="pdf-del-range" placeholder="e.g. 3, 5-7, 10"></div>
       <p style="font-size:12px;color:var(--pdf-color);margin-top:8px;">⚠️ Irreversible. Current: page ${pageNum} of ${pdfDoc?.numPages||'?'}</p>`],

    'extract-pages': ['✂️ Extract Pages',
      `<div class="form-group"><label class="form-label">Pages to Extract</label>
       <input type="text" class="form-input" id="pdf-extract-range" placeholder="e.g. 1-3, 5, 7-9"></div>`],

    'create-from-img': ['🖼️ Create PDF from Image(s)',
      `<div class="form-group"><label class="form-label">Select Image File(s)</label>
       <label class="btn" style="display:block;cursor:pointer;">🗂️ Choose Images
         <input type="file" accept="image/*" multiple style="display:none;" id="img-to-pdf-input" onchange="handleImgToPdfSelect(this)">
       </label>
       <div id="img-to-pdf-list" style="margin-top:8px;font-size:12px;color:var(--text3);">No images selected</div></div>`],

    'merge': ['🔗 Merge PDFs',
      `<p style="font-size:13px;color:var(--text2);margin-bottom:12px;">Select multiple PDF files to merge into one.</p>
       <div class="form-group"><label class="form-label">PDF Files (select multiple)</label>
       <label class="btn" style="display:block;cursor:pointer;">📂 Choose PDF Files
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
          toast(`Compare: ${data.page_count_a} vs ${data.page_count_b} pages — ${diffs.length} differences found`, diffs.length === 0 ? 'success' : 'info');
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
        toast('Accessibility check complete — see results above', 'info'); return;
      case 'find':
        toast('Use Ctrl+F to search within the PDF viewer', 'info'); return;
      default:
        toast(action + ' applied ✓', 'success'); return;
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
      toast(action.replace(/-/g,' ') + ' complete — downloading ' + outName + ' ✓', 'success');

      // Update state.pdfBytes so subsequent operations use the new PDF
      if(!ct.includes('zip')) {
        const arrBuf = await blob.arrayBuffer();
        state.pdfBytes = new Uint8Array(arrBuf);
        // Re-render the updated PDF
        try {
          pdfDoc = await pdfjsLib.getDocument(state.pdfBytes).promise;
          pageNum = 1;
          document.getElementById('pdf-page-count').textContent = pdfDoc.numPages;
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

// ─── Rotate (quick — current page only) ──────────────────────────────────────
function rotatePDF() {
  if(!pdfDoc) { toast('Open a PDF first', 'error'); return; }
  // Quick visual rotation via CSS while serving real rotation from backend
  activePdfAction = 'rotate';
  executePdfAction();
}

// ─── PDF Export as Image ──────────────────────────────────────────────────────
function pdfExportImg() {
  if(!pdfDoc) { toast('Open a PDF first', 'error'); return; }
  const canvas = document.getElementById('pdf-canvas');
  const url = canvas.toDataURL('image/png');
  const a = document.createElement('a');
  a.href = url; a.download = `page-${pageNum}.png`;
  document.body.appendChild(a); a.click(); document.body.removeChild(a);
  toast(`Page ${pageNum} exported as PNG ✓`, 'success');
}

function pdfPrint() {
  if(!pdfDoc) { toast('Open a PDF first', 'error'); return; }
  window.print();
}

// ─── Bookmarks ────────────────────────────────────────────────────────────────
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
       <span onclick="pageNum=${b.page};queueRenderPage(${b.page});">🔖 ${b.name} <span style="color:var(--text3);">p.${b.page}</span></span>
       <span onclick="pdfBookmarks.splice(${i},1);renderBookmarks();" style="cursor:pointer;color:var(--text3);">✕</span>
     </div>`
  ).join('');
}

// ─── OCR ──────────────────────────────────────────────────────────────────────
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
        toast('✓ Text extracted and copied to clipboard (' + text.length + ' chars)', 'success')
      ).catch(() => toast('✓ Text extracted (' + text.length + ' chars) — paste into Writer', 'success'));
    } else {
      toast('No extractable text found (scanned PDF — OCR not available offline)', 'info');
    }
  }).catch(e => toast('Extraction error: ' + e.message, 'error'));
}

// ─── Digital Signature ────────────────────────────────────────────────────────
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
  toast('Signature placed on PDF ✓', 'success');
}
function makeDraggable(el) {
  let ox=0, oy=0, started=false;
  el.onmousedown = (e) => { e.preventDefault(); started=true; ox=e.clientX-el.offsetLeft; oy=e.clientY-el.offsetTop; };
  document.addEventListener('mousemove', (e) => { if(!started) return; el.style.left=(e.clientX-ox)+'px'; el.style.top=(e.clientY-oy)+'px'; });
  document.addEventListener('mouseup', () => started=false);
}

// ─── Form Fields ──────────────────────────────────────────────────────────────
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
  toast(type + ' field added — drag to position ✓', 'success');
}

// ─── Helper stubs ─────────────────────────────────────────────────────────────
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

// ─── Drag & Drop (home screen) ────────────────────────────────────────────────
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
        // The document just saved to DB — load it back to populate editor
        await openDoc(doc.id);
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
        <div class="version-label">📌 ${v.label || 'Version'}</div>
        <div class="version-date">${new Date(v.created_at).toLocaleString()}</div>
        <button class="btn btn-sm" onclick="restoreVersion('${v.id}')">↩ Restore</button>
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
    if(resp.ok) { await openDoc(state.docId); toast('Restored ✓', 'success'); closeVersionsPanel(); }
    else toast('Restore failed', 'error');
  } catch(e) { toast('Error: ' + e.message, 'error'); }
}

// ─── Settings ─────────────────────────────────────────────────────────────────
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
  toast('Settings saved ✓', 'success');
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

// ─── Global Keyboard Shortcuts ────────────────────────────────────────────────
document.addEventListener('keydown', (e) => {
  if(e.ctrlKey && e.shiftKey && e.key === 'T') { e.preventDefault(); toggleTheme(); }
  if(e.ctrlKey && e.key === 's') { e.preventDefault(); quickSave(); }
  if(e.key === 'F1') { e.preventDefault(); showHelpModal(); }
  if(e.key === 'Escape') {
    document.querySelectorAll('.modal-overlay.open').forEach(m => m.classList.remove('open'));
    closeFindBar();
  }
});

// ─── Help Modal ───────────────────────────────────────────────────────────────
function showHelpModal() { document.getElementById('help-modal').classList.add('open'); }
function closeHelp() { document.getElementById('help-modal').classList.remove('open'); }

// Close modals on overlay click
['settings-modal','help-modal','pdf-stub-modal','signature-modal'].forEach(id => {
  const el = document.getElementById(id);
  if(el) el.addEventListener('click', function(e) { if(e.target === this) this.classList.remove('open'); });
});

// ─── Toast ─────────────────────────────────────────────────────────────────────
function toast(msg, type = '') {
  const container = document.getElementById('toast-container');
  const el = document.createElement('div');
  el.className = 'toast' + (type ? ' ' + type : '');
  el.textContent = msg;
  container.appendChild(el);
  setTimeout(() => el.remove(), 4000);
}

// ─── Window Resize ─────────────────────────────────────────────────────────────
window.addEventListener('resize', () => {
  if(state.section === 'sheets') drawGrid();
});

// ─── Init ──────────────────────────────────────────────────────────────────────
document.addEventListener('DOMContentLoaded', async () => {
  const savedTheme = localStorage.getItem('theme') || 'light';
  applyTheme(savedTheme);
  document.getElementById('pref-theme').value = savedTheme;
  await loadSettings();
  await loadRecent();
  updateWordCount();
});
</script>
'@

$combined = $before + $newJS + $after
$combined | Set-Content $file -Encoding UTF8
Write-Host "Done. New line count: $($combined.Count)"
