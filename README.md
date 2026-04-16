# PDF Office — Local-First, Rust-Native Office Suite

> **100% private. No telemetry. No cloud. Your documents stay on your machine.**

A complete office suite built entirely in Rust — Writer (word processor), Sheets (spreadsheet), Slides (presentations), and PDF Tools — served as a local web app via an embedded Axum server.

## ✨ Features

| Module | Status | Description |
|--------|--------|-------------|
| **Writer** | ✅ Phase 1 | Word processor with OT editing, rich text, DOCX/ODT/RTF import & export |
| **Sheets** | ✅ Phase 1 | Spreadsheet with 130+ functions, formula engine, XLSX import/export |
| **Slides** | ✅ Phase 1 | Presentation editor with SVG rendering, PPTX support |
| **PDF Tools** | ✅ Phase 1 | PDF view, creation from documents, merge/split (Phase 2) |
| **Spell Check** | ✅ Phase 1 | Hunspell-compatible, ~2500 en_US words with affix stripping |
| **Real-Time WS** | ✅ Phase 1 | WebSocket-based OT sync, formula eval, spell on the wire |
| **Storage** | ✅ Phase 1 | SQLite-backed, versioned documents, comments, preferences |
| **CLI** | ✅ Phase 1 | Full `pdf` binary: open, serve, convert, export, spell, version |

## 🏗️ Project Structure

```
pdf office/
├── Cargo.toml                  # Workspace root
└── crates/
    ├── pdf-core/               # Shared document/spreadsheet/presentation models
    ├── pdf-writer/             # OT engine, layout engine, tracked changes
    ├── pdf-sheets/             # Formula lexer → parser → evaluator, 130+ functions
    ├── pdf-slides/             # SVG slideshow renderer
    ├── pdf-pdf/                # PDF creation (lopdf) + manipulation
    ├── pdf-spell/              # Hunspell-compatible spell checker
    ├── pdf-render/             # tiny-skia rasterizer for print-quality output
    ├── pdf-formats/            # DOCX, XLSX, PPTX, ODT, RTF readers/writers
    ├── pdf-storage/            # SQLite persistence (docs, versions, comments, prefs)
    ├── pdf-server/             # Axum HTTP + WebSocket server (embedded HTML UI)
    ├── pdf-cli/                # `pdf` binary entry point
    └── pdf-ui/                 # Placeholder WASM frontend (Phase 2: Leptos)
```

## 🚀 Quick Start

### Prerequisites

- [Rust toolchain](https://rustup.rs/) (stable, 1.75+)
- Windows: Visual Studio C++ Build Tools (for SQLite bundled feature)

### Build & Run

```powershell
# Clone or open the directory
cd "pdf office"

# Check for errors (first run downloads deps — ~2 min)
cargo check --workspace

# Run the local server
cargo run --bin pdf-cli -- serve --port 3847

# Or just run (the default command opens browser automatically)
cargo run --bin pdf-cli
```

Then open [http://localhost:3847](http://localhost:3847) in your browser.

### Release Build

```powershell
cargo build --release --bin pdf-cli
# Binary: target/release/pdf-cli.exe (Windows) or pdf-cli (*nix)
```

## 💻 CLI Usage

```
pdf                              # Start server, open browser
pdf serve --port 3847            # Start server on specified port
pdf serve --no-browser           # Start without opening browser
pdf open document.docx           # Open a document
pdf open spreadsheet.xlsx        # Open a spreadsheet (opens Sheets UI)
pdf open presentation.pptx       # Open a presentation (opens Slides UI)
pdf convert input.docx out.pdf   # Headless format conversion
pdf export file.docx --format pdf      # Export Writer document
pdf export file.docx --format txt      # Export as plain text
pdf spell document.docx          # Spell check a document
pdf version                      # Show version & build info
```

## 🌐 REST API

The server exposes a REST API at `http://localhost:3847/api/`:

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/documents` | List recent documents |
| POST | `/api/documents` | Create new document |
| GET | `/api/documents/:id` | Get document JSON |
| POST | `/api/documents/:id` | Save/update document |
| DELETE | `/api/documents/:id` | Delete document |
| POST | `/api/documents/:id/export` | Export (format: docx, pdf, pdf, xlsx) |
| GET | `/api/documents/:id/versions` | List version history |
| GET | `/api/documents/:id/versions/:vid` | Get specific version |
| GET | `/api/documents/:id/comments` | List comments |
| POST | `/api/documents/:id/comments` | Add comment |
| POST | `/api/spell` | Spell check text |
| POST | `/api/formula` | Evaluate a formula |
| POST | `/api/convert` | Convert uploaded file (multipart) |
| GET | `/api/preferences/:key` | Get preference value |
| PUT | `/api/preferences/:key` | Set preference value |

## 🔌 WebSocket Protocol

Connect to `ws://localhost:3847/ws/<doc-id>` for real-time collaboration.

Messages use `{"type": "...", "payload": {...}}` format. Key types:

- **Client → Server**: `ApplyOp`, `SpellCheck`, `FormulaEval`, `SetViewport`, `RequestPage`, `Ping`
- **Server → Client**: `OpAck`, `DocumentState`, `SpellResults`, `FormulaResult`, `ViewportData`, `CanvasCommands`, `Pong`

## 🧪 Running Tests

```powershell
# All tests
cargo test --workspace

# Specific crate
cargo test -p pdf-sheets
cargo test -p pdf-spell
cargo test -p pdf-storage
```

## 📦 Supported Formats

| Format | Read | Write |
|--------|------|-------|
| `.docx` | ✅ | ✅ |
| `.xlsx` | ✅ | ✅ |
| `.pptx` | ✅ | ✅ |
| `.odt` (ODT) | ✅ | — |
| `.rtf` | ✅ | — |
| `.pdf` | — | ✅ (creation) |
| `.pdfo` | ✅ | ✅ (native) |
| `.txt` | — | ✅ (export) |

## 🔒 Privacy Guarantee

- Binds **exclusively** to `127.0.0.1` — never reachable from the network
- **Zero telemetry** — no analytics, no crash reporting, no usage metrics
- All data stored in `~/.pdfo/data/` (SQLite + local files)
- No login, no account, no cloud sync required

## 🛠️ Development Notes

### Crate Dependencies

```
pdf-cli
  └── pdf-server ──► pdf-storage, pdf-core, pdf-formats, pdf-spell, pdf-sheets, pdf-pdf, pdf-render, pdf-writer
pdf-sheets ──► pdf-core
pdf-slides ──► pdf-core
pdf-pdf    ──► pdf-core
pdf-render ──► pdf-writer, pdf-core
pdf-formats ──► pdf-core
pdf-spell  (standalone)
pdf-core   (standalone)
```

### Adding Functions to the Formula Engine

1. Open `crates/pdf-sheets/src/functions.rs`
2. Add your function to the `FUNCTIONS` `LazyLock<HashMap<...>>` near the bottom
3. Implement it as `fn my_func(args: &[CellValue]) -> Result<CellValue, String>`

### Adding File Format Support

1. Create a new module in `crates/pdf-formats/src/`
2. Implement `parse_X(bytes: &[u8]) -> Result<Document, FormatError>`
3. Export it from `crates/pdf-formats/src/lib.rs`
4. Wire it into `pdf-cli/src/main.rs` and `pdf-server/src/api.rs`

## 📄 License

MIT — see [LICENSE](LICENSE) for details.

---

*Built with ❤️ and 🦀 Rust.*
