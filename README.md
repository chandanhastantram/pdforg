# 📄 PDF Office (`pdforg`)

> **Local-first, private, Rust-native office suite — no cloud, no subscriptions, no telemetry.**

PDF Office is an all-in-one office application that includes a word processor (Writer), spreadsheet editor (Sheets), presentation maker (Slides), and PDF tools — all running completely on your own computer. 

Available globally on crates.io as `pdforg`.

---

## 🚀 Quick Start

### Method 1 — Install Globally via Crates.io (Recommended)

**Requirements:** Rust + MSVC Build Tools (Windows) / build-essential (Linux/Mac).

You can easily install PDF Office directly from the central Rust package registry. Once installed, you can launch the app from anywhere.

```powershell
# 1. Install via cargo
cargo install pdforg

# 2. Launch the application!
pdf
```
Your browser will automatically open at `http://localhost:3847` to show the application UI.

### Method 2 — Run from Source

```powershell
# 1. Clone the repository
git clone https://github.com/chandanhastantram/pdforg.git
cd pdforg

# 2. Run the app
cargo run --bin pdf
```

### Method 3 — Build a Release Binary

```powershell
cargo build --release --bin pdf

# Your app is now at:
# target\release\pdf.exe
```

---

## 📦 Share With Friends (No Tech Skills Required)

### Method 1: Share the `.exe` File (Easiest)

1. Build the release binary (`cargo build --release --bin pdf`)
2. Share the generated `pdf.exe` file with your friend.
3. They simply double-click it. Nothing else to install, no setup required!

### Method 2: Share a Single Document (`.pdfo`)

1. Open a document in Writer.
2. Click the **"📤 Share"** button in the toolbar.
3. Send the `document.pdfo` file to your friend.
4. Your friend opens PDF Office and drags the `.pdfo` file onto the home screen.
5. The document appears instantly — no account needed!

### Method 3: Share via Local Network (LAN)

You can host PDF Office for everyone on your Wi-Fi network!

```powershell
# Start the server explicitly on your local network
pdf serve --port 3847

# Your friend on the same WiFi opens:
# http://YOUR_IP:3847
# (Find your IP using `ipconfig` or `ifconfig`)
```

---

## 🖥️ CLI Commands

PDF office ships with a rich CLI (`pdf`) for automated conversions, exports, and spell checking.

| Command | What it does |
|---------|-------------|
| `pdf` | Launch the app and open it in your browser |
| `pdf serve` | Start the server (browser does NOT auto-open) |
| `pdf open myfile.docx` | Open a Word document |
| `pdf open myfile.xlsx` | Open an Excel spreadsheet |
| `pdf export myfile.pdfo --format docx` | Convert to Word format |
| `pdf export myfile.pdfo --format pdf` | Convert to PDF |
| `pdf convert input.docx output.pdf` | Convert between supported formats headlessly |
| `pdf spell myfile.docx` | Check spelling in a document |
| `pdf version` | Show version information |
| `pdf update` | Check for updates |

*Run `pdf --help` for deeper sub-command options.*

---

## 🎯 Using the App (For Everyone)

### Home Screen
When you open PDF Office, you'll see the **Home** screen:
- **📝 New Document** — Creates a blank Word-like document
- **📊 New Spreadsheet** — Creates a blank Excel-like spreadsheet  
- **📽️ New Presentation** — Creates a blank slide presentation
- **Drag & Drop** — Drag any `.docx`, `.xlsx`, `.pptx`, or `.pdf` file directly onto the home screen to open it

---

### 📝 Writer (Word Processor)
1. Click **"📝 New Document"** or **Writer** in the sidebar.
2. Start typing! The document auto-saves locally every 2 seconds.
3. Use the **rich ribbon toolbar** to format text, add headers, align content, and export instantly to `.docx` or `.pdf`.
4. Run the integrated spell check via the **✓ Spell** button.
5. Hit `Ctrl+P` to print your document natively.

### 📊 Sheets (Spreadsheet)
1. Click **"📊 New Spreadsheet"** or **Sheets** in the sidebar.
2. Formats and layouts are supported exactly like Excel.
3. Type formulas starting with `=` (e.g. `=SUM(A1:A10)`). Over 130 native formulas are supported covering Math, Logic, Date, Text, and Lookups (`VLOOKUP`).
4. Auto-completion prompts will help you build your formulas.

### 📽️ Slides (Presentation)
1. Click **"📽️ New Presentation"** or **Slides** in the sidebar.
2. Manage multiple slides via the left-side thumbnail panel.
3. Click elements natively in the canvas to adjust and refine styles.

### 📄 PDF Tools
1. Click **PDF** in the sidebar.
2. **Merge:** Combine multiple PDFs.
3. **Split:** Break large PDFs apart.
4. **Watermark:** Instantly stamp your documents with transparency.

---

## 📁 Data and Privacy

All your documents are stored natively in a highly efficient internal **SQLite** database resting entirely on your filesystem:

```
Windows: C:\Users\YOUR_NAME\AppData\Local\pdf-office\data.db
Linux/Mac: ~/.local/share/pdf-office/data.db
```

**Your data safely stays on your computer.** No account required, no telemetry payloads, and absolutely no internet connection needed.

---

## 📋 Supported File Formats

| Format | Read | Write | Notes |
|--------|------|-------|-------|
| `.pdfo` | ✅ | ✅ | Native PDF Office SQLite bundle |
| `.docx` | ✅ | ✅ | Microsoft Word |
| `.odt` | ✅ | ✅ | OpenDocument Text |
| `.xlsx` | ✅ | ✅ | Microsoft Excel |
| `.pptx` | ✅ | ✅ | Microsoft PowerPoint |
| `.pdf` | ✅ | ✅ | Portable Document Format |
| `.rtf` | ✅ | ❌ | Rich Text Format |

---

## 🛠️ Architecture

To bypass extremely strict distribution limitations, PDF Office was massively refactored into a **monolithic unified crate architecture**. 

Everything from the `axum` local webserver, to the `tiny-skia` SVG rasterizers, to the `.docx` xml parsing pipelines, to the 130+ Sheets function evaluators are all bundled into the highly modular internal `src/` directory tree under the `pdforg` identity.

| Internal Module | Purpose |
|----------------|---------|
| `core` | Core structural data types (Document, Cell, Slide) |
| `writer` | Word processor text block layout and calculation |
| `sheets` | Spreadsheet engine, dependency graph, formula evaluator |
| `slides` | Presentation engine and slide rendering |
| `formats` | File format XML parsers (docx, xlsx, pptx, odt) |
| `pdf_tools` | Native PDF generation (via `lopdf`) |
| `render` | SVG/raster renderer integrations |
| `spell` | Hunspell-compatible spell checking engine |
| `storage` | SQLite file and state layer |
| `server` | Axum HTTP router + live WebSocket handlers |

### Building and Testing

```powershell
# Build binaries
cargo build --release

# Run comprehensive test suite
cargo test

# Apply lints and automatic fixes
cargo fix --allow-dirty
```

---

*Built with ❤️ in Rust · Local-first · Private by design*
