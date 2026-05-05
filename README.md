<<<<<< Updated upstream
#  PDF Office (`pdforg`)
=======
# PDF Office
>>>>>> Stashed changes

A lightning-fast, privacy-first, purely local PDF manipulation tool. Built with Rust and React.

![PDF Office](frontend/public/favicon.svg)

# Features

- **Privacy-First Architecture**: 100% of PDF processing happens locally on your machine. No cloud uploads, no external APIs, no tracking.
- **20+ PDF Tools**: Merge, split, compress, protect, watermark, rotate, delete pages, extract pages, insert pages, add page numbers, add headers/footers, bates numbering, edit metadata, sanitize, flatten, compare, unlock, and more.
- **Lightning Fast**: Powered by a robust Rust backend utilizing the `lopdf` engine, handling multi-megabyte files in milliseconds.
- **Modern UI**: A sleek, dark-mode first interface built with React, Vite, and Tailwind CSS.
- **No Size Limits**: Process massive 500MB+ PDF documents directly from your local filesystem without arbitrary browser memory limits.

## Architecture

<<<<<<< Updated upstream
##  Quick Download & Install
=======
PDF Office represents a modern decoupled architecture:
>>>>>>> Stashed changes

- **Backend (`Rust / Axum`)**: Pure REST API. Handles routing, file parsing, and complex PDF stream manipulations. Run natively for best performance.
- **Frontend (`React / TypeScript / Vite`)**: A single-page application focused solely on user experience, API communication, and displaying status feedback.

## Getting Started

### Prerequisites
- [Rust](https://rustup.rs/) (latest stable)
- [Node.js](https://nodejs.org/) (v18+)

### 1. Start the Rust Backend
Open a terminal in the root directory:
```bash
# Build and run the server in release mode (for maximum performance)
cargo run --release -- serve --port 3847 --no-browser
```
*The backend will be available at `http://127.0.0.1:3847`*

<<<<<<< Updated upstream
Your browser will automatically open a local port (e.g. `http://127.0.0.1:3847`) to show the ultra-responsive application UI.

---

##  Groundbreaking PDF Features

Unlike traditional PDF tools that rely on the cloud (like iLovePDF) or charge massive subscriptions (like Adobe Acrobat Pro), our PDF suite operates natively on your machine via direct bytestream manipulation using `lopdf`.

- ** Cryptography & Protection:** Apply authentic AES-128 and RC4-128 bit security. Lock files from printing, editing, or copying natively.
- ** True Compression:** Flate2 `DEFLATE` stream recompression dynamically iterates through your PDF objects to massively reduce file sizes without data loss. Detects and downscales `DCTDecode` JPEGs on the fly.
- ** Total Organization:** Split, merge, rotate, extract, or delete pages natively. The engine dynamically reconstructs the internal PDF Catalog and Page Tree without corruption.
- ** PostScript Stamping:** Inject Bates numbering, Headers, Footers, and Watermarks directly into the PDF rendering layout.
- ** Sanitization:** Remove hidden Metadata, JavaScript triggers, Embedded Files, and flatten AcroForm layers permanently.
- ** Deep Conversions:** Supports transforming JPEG/PNG into perfectly sized PDF XObjects.

---

##  The Office Suite

### Writer (Word Processor)

- Clean, distraction-free environment that auto-saves locally every 2 seconds.
- Rich ribbon toolbar for complete formatting control.
- Exports instantly to `.docx` or directly rasterizes to `.pdf`.
- Native hunspell-powered Spell check integration.

###  Sheets (Spreadsheet)

- Fully featured Grid engine parsing `xlsx` and `csv`.
- Powerful Rust evaluator that resolves mathematical formulas (`=SUM(A1:A10)`, `VLOOKUP`, etc) instantly.
- Supports Date, Text, Logic, Math, and Lookup function blocks.

###  Slides (Presentation)

- Intuitive slide manager.
- Directly import `.pptx` and adjust internal layouts.

---

##  Architecture & Privacy by Design

**Privacy Guarantee:** We believe documents belong to you. The application **will not and cannot** connect to cloud translation, API microservices, or analytics payloads.

All your documents are stored natively inside a highly optimized **SQLite** database bundle resting entirely on your filesystem:

```
Windows: C:\Users\YOUR_NAME\.pdfo\data.db
Linux/Mac: ~/.pdfo/data.db
=======
### 2. Start the Frontend Dev Server
Open a second terminal:
```bash
cd frontend
npm install
npm run dev
>>>>>>> Stashed changes
```
*The frontend will launch at `http://localhost:5173`*

## Recent Improvements

- **Fully Decoupled Stack**: Transitioned from a monolithic server to a distinct Rust API backend and Vite/React frontend.
- **Large File Support**: Overhauled the multipart form data handlers to support payloads up to 500MB, removing old bottleneck limits.
- **Memory Safety & Stability**: Addressed edge-case panics in the splitting mechanisms (handling single-page PDFs gracefully) and improved runtime stability by utilizing `panic="unwind"`.
- **Attribute Preservation**: Added intelligent inheritance resolution ensuring that manipulated PDFs retain their `MediaBox`, `CropBox`, and `Resources` attributes, preventing blank page rendering issues during merges and extractions.

## License

This project is intended for personal and local use.
