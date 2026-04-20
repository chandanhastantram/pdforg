# 📄 PDF Office (`pdforg`)

[![Website](https://img.shields.io/website?url=https%3A%2F%2Fpdforg.vercel.app&label=Live%20Website&logo=vercel)](https://pdforg.vercel.app/)
[![Crates.io](https://img.shields.io/crates/v/pdforg.svg)](https://crates.io/crates/pdforg)

> **Local-first, private, Rust-native office suite — no cloud, no subscriptions, no telemetry.**

PDF Office is an enterprise-grade suite including a word processor (Writer), spreadsheet editor (Sheets), presentation maker (Slides), and an incredibly powerful, native PDF manipulation backend — **all running completely on your own computer.**

You can find the official download site here: **[pdforg.app](https://pdforg.vercel.app/)**

---

## 🚀 Quick Download & Install

### Option 1 — Download Pre-Compiled Binaries

Visit **[pdforg.vercel.app](https://pdforg.vercel.app/)** to immediately download the standalone executable.
There are no installation scripts or massive dependencies. A single 30MB executable contains the entire Rust backend, the frontend UI, and all dependencies. Just double-click and run.

### Option 2 — Install via Cargo (For Developers)

You can easily compile and install PDF Office directly from source or the central Rust registry.

```powershell
# 1. Install directly from this Github Repository
cargo install --git https://github.com/chandanhastantram/pdforg.git

# 2. Or from Crates.io
cargo install pdforg

# 3. Launch the application!
pdf
```

Your browser will automatically open a local port (e.g. `http://127.0.0.1:3847`) to show the ultra-responsive application UI.

---

## 🎯 Groundbreaking PDF Features

Unlike traditional PDF tools that rely on the cloud (like iLovePDF) or charge massive subscriptions (like Adobe Acrobat Pro), our PDF suite operates natively on your machine via direct bytestream manipulation using `lopdf`.

- **🔐 Cryptography & Protection:** Apply authentic AES-128 and RC4-128 bit security. Lock files from printing, editing, or copying natively.
- **🗜️ True Compression:** Flate2 `DEFLATE` stream recompression dynamically iterates through your PDF objects to massively reduce file sizes without data loss. Detects and downscales `DCTDecode` JPEGs on the fly.
- **📑 Total Organization:** Split, merge, rotate, extract, or delete pages natively. The engine dynamically reconstructs the internal PDF Catalog and Page Tree without corruption.
- **🖊️ PostScript Stamping:** Inject Bates numbering, Headers, Footers, and Watermarks directly into the PDF rendering layout.
- **🧹 Sanitization:** Remove hidden Metadata, JavaScript triggers, Embedded Files, and flatten AcroForm layers permanently.
- **🖼️ Deep Conversions:** Supports transforming JPEG/PNG into perfectly sized PDF XObjects.

---

## 📝 The Office Suite

### Writer (Word Processor)

- Clean, distraction-free environment that auto-saves locally every 2 seconds.
- Rich ribbon toolbar for complete formatting control.
- Exports instantly to `.docx` or directly rasterizes to `.pdf`.
- Native hunspell-powered Spell check integration.

### 📊 Sheets (Spreadsheet)

- Fully featured Grid engine parsing `xlsx` and `csv`.
- Powerful Rust evaluator that resolves mathematical formulas (`=SUM(A1:A10)`, `VLOOKUP`, etc) instantly.
- Supports Date, Text, Logic, Math, and Lookup function blocks.

### 📽️ Slides (Presentation)

- Intuitive slide manager.
- Directly import `.pptx` and adjust internal layouts.

---

## 🗄️ Architecture & Privacy by Design

**Privacy Guarantee:** We believe documents belong to you. The application **will not and cannot** connect to cloud translation, API microservices, or analytics payloads.

All your documents are stored natively inside a highly optimized **SQLite** database bundle resting entirely on your filesystem:

```
Windows: C:\Users\YOUR_NAME\.pdfo\data.db
Linux/Mac: ~/.pdfo/data.db
```

To bypass extremely strict distribution limitations, PDF Office was massively refactored into a **monolithic unified crate architecture**. Everything from the `axum` local webserver, to the `tiny-skia` SVG rasterizers, to the XML parsing pipelines, to the Rust mathematical evaluators are bundled together under the `pdforg` identity.

| Internal Module            | Architecture Layer                                                      |
| -------------------------- | ----------------------------------------------------------------------- |
| `pdf_tools`                | Native PDF compilation (`protect`, `compress`, `manipulator`, `images`) |
| `writer / sheets / slides` | Application engines and UI logic bridges                                |
| `formats`                  | Complex native XML parsers (`docx`, `xlsx`, `pptx`, `odt`)              |
| `spell`                    | Hunspell-compatible offline spell checking                              |
| `server`                   | Axum HTTP router + live WebSocket handlers                              |

---

_Open Source. Private by design. Engineered in Rust._
