# 📄 PDF Office

> **Local-first, private, Rust-native office suite — no cloud, no subscriptions, no telemetry.**

PDF Office is an all-in-one office application that includes a word processor (Writer), spreadsheet editor (Sheets), presentation maker (Slides), and PDF tools — all running completely on your own computer.

---

## 🚀 Quick Start

### Option 1 — Run from Source (Recommended)

**Requirements:** Rust + MSVC Build Tools installed

```powershell
# 1. Open the project folder in a terminal
cd "c:\Users\chand\.gemini\pdf office"

# 2. Run the app
cargo run --bin pdf

# 3. Your browser will automatically open at http://localhost:3847
```

### Option 2 — Install Globally with Cargo

After this, you can type `pdf` from **anywhere** in your terminal — just like a normal installed app:

```powershell
# From the project folder:
cargo install --path . --bin pdf

# Now from ANY folder, just type:
pdf
```

### Option 3 — Build Release Binary (Fastest to run)

```powershell
cargo build --release --bin pdf

# Your app is now at:
# target\release\pdf.exe
```

---

## 📦 Share With Friends (No GitHub Required)

### Method 1: Share the .exe File (Easiest — Friend needs nothing installed)

```powershell
# Build the release binary
cargo build --release --bin pdf

# Share this file with your friend:
#   target\release\pdf.exe
# They just double-click it — no setup needed!
```

### Method 2: Share the Source Code

1. Zip the project folder (skip the `target\` folder — it's huge)
2. Send the zip to your friend
3. Friend installs Rust from https://rustup.rs
4. Friend runs:
   ```powershell
   cargo install --path . --bin pdf
   # Then just type:
   pdf
   ```

### Method 3: Share a Single Document (.pdfo)

1. Open your document in Writer
2. Click **"📤 Share"** button in the toolbar
3. Send the `document.pdfo` file to your friend
4. Friend opens PDF Office, drags the `.pdfo` file onto the home screen
5. Document appears instantly — no account needed!

### Method 4: Share via Local Network (LAN)

```powershell
# Start the server accessible on your local network
pdf serve --port 3847

# Your friend on the same WiFi opens:
# http://YOUR_IP:3847
# (Find your IP with: ipconfig)
```

---

## 🖥️ All Commands

### Basic Commands

| Command | What it does |
|---------|-------------|
| `pdf` | Launch the app and open it in your browser |
| `pdf serve` | Start the server (browser does NOT auto-open) |
| `pdf serve --port 8080` | Start on a specific port |
| `pdf version` | Show version information |
| `pdf update` | Check for updates |

### Document Commands

| Command | What it does |
|---------|-------------|
| `pdf open myfile.docx` | Open a Word document |
| `pdf open myfile.xlsx` | Open an Excel spreadsheet |
| `pdf open myfile.pptx` | Open a PowerPoint presentation |
| `pdf open myfile.pdfo` | Open a PDF Office document |
| `pdf open myfile.pdf` | Open a PDF file |
| `pdf open myfile.odt` | Open an OpenDocument text file |

### Export Commands

| Command | What it does |
|---------|-------------|
| `pdf export myfile.pdfo --format docx` | Convert to Word format |
| `pdf export myfile.pdfo --format pdf` | Convert to PDF |
| `pdf export myfile.pdfo --format xlsx` | Convert to Excel format |
| `pdf export myfile.pdfo --format pptx` | Convert to PowerPoint |
| `pdf export myfile.pdfo --format odt` | Convert to OpenDocument |
| `pdf export myfile.pdfo --format pdf --output out.pdf` | Export with custom filename |

### Conversion Commands

| Command | What it does |
|---------|-------------|
| `pdf convert input.docx output.pdf` | Convert any format to any other |
| `pdf convert report.xlsx report.csv` | Convert spreadsheet formats |

### Spell Check

| Command | What it does |
|---------|-------------|
| `pdf spell myfile.pdfo` | Check spelling in a document |
| `pdf spell myfile.pdfo --lang en` | Check with specific language |

### Advanced / Server Options

| Command | What it does |
|---------|-------------|
| `pdf serve --port 3847 --no-browser` | Start without opening browser |
| `pdf serve --data-dir C:\MyDocs` | Store documents in a specific folder |
| `pdf --verbose serve` | Show detailed log output |

---

## 🎯 Using the App (For Everyone — No Tech Skills Needed)

### Home Screen

When you open PDF Office, you'll see the **Home** screen:

- **📝 New Document** — Creates a blank Word-like document
- **📊 New Spreadsheet** — Creates a blank Excel-like spreadsheet  
- **📽️ New Presentation** — Creates a blank slide presentation
- **Recent Documents** — Your last-edited files appear here automatically
- **Drag & Drop** — Drag any `.docx`, `.xlsx`, `.pptx`, or `.pdf` file directly onto the home screen to open it

---

### 📝 Writer (Word Processor)

**How to use:**
1. Click **"📝 New Document"** or **Writer** in the sidebar
2. Start typing! The document auto-saves every 2 seconds
3. Use the **ribbon toolbar** at the top to format your text

**Ribbon toolbar features:**
| Button | What it does |
|--------|-------------|
| 💾 Save | Save manually right now |
| 📤 Share | Export a `.pdfo` file to send to a friend |
| **B** | Make text **bold** |
| *I* | Make text *italic* |
| <u>U</u> | Underline text |
| Font dropdown | Change the font (Times New Roman, Arial, etc.) |
| Size dropdown | Change text size (8pt to 72pt) |
| ⬛ Left | Align text to the left |
| ▬ Center | Center the text |
| ⬛ Right | Align text to the right |
| ≡ Justify | Justify text to both edges |
| •≡ List | Add a bullet point list |
| 1≡ Numbered | Add a numbered list |
| ─ Line | Add a horizontal line |
| ⬇️ DOCX | Download as Word file |
| ⬇️ PDF | Download as PDF |
| 🕐 History | View and restore old versions |
| ✓ Spell | Run spell check |

**Keyboard shortcuts:**
| Keys | Action |
|------|--------|
| `Ctrl+S` | Save |
| `Ctrl+B` | Bold |
| `Ctrl+I` | Italic |
| `Ctrl+U` | Underline |
| `Ctrl+F` | Find text |
| `Ctrl+P` | Print |

---

### 📊 Sheets (Spreadsheet)

**How to use:**
1. Click **"📊 New Spreadsheet"** or **Sheets** in the sidebar
2. Click any cell to select it
3. Type a value or formula in the **formula bar** at the top
4. Press **Enter** to confirm and move to the next row

**Formula bar:**
- Shows the current cell reference (e.g. `A1`)
- Type values like `25` or `Hello`
- Type formulas starting with `=` like `=SUM(A1:A10)`
- As you type `=`, a dropdown of available functions appears automatically!

**Common formulas:**

| Formula | What it does |
|---------|-------------|
| `=SUM(A1:A10)` | Add up cells A1 through A10 |
| `=AVERAGE(B1:B5)` | Average of B1 to B5 |
| `=MAX(C1:C100)` | Largest value in column C |
| `=MIN(C1:C100)` | Smallest value in column C |
| `=COUNT(A:A)` | Count how many numbers are in column A |
| `=IF(A1>100, "Big", "Small")` | Show "Big" if A1 > 100, else "Small" |
| `=VLOOKUP(A1, B:C, 2, FALSE)` | Look up A1 in column B, return column C |
| `=CONCATENATE(A1, " ", B1)` | Join two cells together |
| `=TODAY()` | Today's date |
| `=LEN(A1)` | Number of characters in A1 |

**Ribbon shortcuts:**
| Button | What it does |
|--------|-------------|
| Σ SUM | Insert SUM formula |
| x̄ AVG | Insert AVERAGE formula |
| # COUNT | Insert COUNT formula |
| ? IF | Insert IF formula |
| ⬇️ XLSX | Download as Excel file |

**Keyboard navigation:**
| Keys | Action |
|------|--------|
| Arrow keys | Move between cells |
| `Enter` | Confirm and go down |
| `Tab` | Confirm and go right |
| `Delete` | Clear the selected cell |

---

### 📽️ Slides (Presentation)

**How to use:**
1. Click **"📽️ New Presentation"** or **Slides** in the sidebar
2. Use the ribbon toolbar:
   - **+ New Slide** — Add another slide
   - **T Text** — Add a text box to the slide
   - **◻ Shape** — Add a shape (coming soon)
3. Click the slide thumbnail panel on the left to switch between slides
4. Click on any text box on the slide canvas to edit it
5. When done, click **⬇️ PPTX** to download as PowerPoint

---

### 📄 PDF Tools

**How to use:**
1. Click **PDF** in the sidebar
2. Drag a PDF file onto the page, or click **"📂 Open PDF"**
3. Use the toolbar to:
   - **🔗 Merge** — Combine two PDFs into one
   - **✂️ Split** — Split one PDF into multiple files
   - **💧 Watermark** — Add a watermark to a PDF

---

## ⌨️ Global Keyboard Shortcuts

These work anywhere in the app:

| Keys | Action |
|------|--------|
| `Ctrl+S` | Quick save current document |
| `Ctrl+Shift+T` | Toggle dark/light theme |
| `F1` | Open Help & Shortcuts guide |

---

## 🌙 Themes

PDF Office supports **Light** and **Dark** modes:

- Click the **🌓** button (top right) to toggle instantly
- Or go to **⚙️ Settings → Theme** to set your preference

---

## 📁 Where Are My Documents Stored?

All your documents are stored locally in a SQLite database on your computer:

```
Windows: C:\Users\YOUR_NAME\AppData\Local\pdf-office\data.db
Linux/Mac: ~/.local/share/pdf-office/data.db
```

**Your data never leaves your computer.** No account required, no internet connection needed (except loading Google Fonts on first launch).

---

## 🔧 Troubleshooting

### App won't start / "linker error"
You need Microsoft Visual C++ build tools:
1. Download from: https://visualstudio.microsoft.com/visual-cpp-build-tools/
2. Install with "Desktop development with C++" workload
3. Restart your terminal
4. Run `cargo run --bin pdf` again

### Port already in use
```powershell
pdf serve --port 8080   # Use a different port
```

### App opens but shows blank page
Try refreshing with `Ctrl+Shift+R` (hard refresh).

### Can't open .docx / .xlsx files
Drag and drop the file directly onto the PDF Office home screen. The import dialog should appear.

### Build takes too long the first time
That's normal! Rust downloads and compiles all dependencies the first time. After that, it's fast. ☕

---

## 🏗️ Building a Release Version

```powershell
# Build optimized release (smaller, faster)
cargo build --release --bin pdf

# The executable is at:
target\release\pdf.exe   # Windows
target/release/pdf        # Linux/Mac

# Install globally
cargo install --path . --bin pdf
pdf   # Run from anywhere!
```

---

## 📋 Supported File Formats

| Format | Read | Write | Notes |
|--------|------|-------|-------|
| `.pdfo` | ✅ | ✅ | Native PDF Office format |
| `.docx` | ✅ | ✅ | Microsoft Word |
| `.odt` | ✅ | ✅ | OpenDocument Text |
| `.xlsx` | ✅ | ✅ | Microsoft Excel |
| `.pptx` | ✅ | ✅ | Microsoft PowerPoint |
| `.pdf` | ✅ | ✅ | Portable Document Format |
| `.rtf` | ✅ | ❌ | Rich Text Format |
| `.csv` | ✅ | ❌ | Comma-Separated Values |

---

## 🤔 Frequently Asked Questions

**Q: Does it need internet?**
> No. Once installed, it works 100% offline. (Google Fonts loads on first launch only if you have internet — it falls back to system fonts without it.)

**Q: Can two people edit the same document at the same time?**
> On a local network, yes — share the server with `pdf serve` and others connect via browser. Real-time collaboration is supported via WebSocket.

**Q: Is my data safe?**
> Yes. Everything is stored locally in a SQLite database. Nothing is sent anywhere. No telemetry.

**Q: What's the difference between .pdfo and .docx?**
> `.pdfo` is PDF Office's native format — faster, smaller, preserves everything. `.docx` is for compatibility with Microsoft Word. Use `.docx` when sharing with Word users.

**Q: How do I update the app?**
> If you installed from source: `git pull` then `cargo install --path . --bin pdf --force`
> If you have the exe: replace it with the new one.

---

## 🛠️ For Developers

### Workspace Crates

| Crate | Purpose |
|-------|---------|
| `pdf-core` | Core data types (Document, Cell, Slide) |
| `pdf-writer` | Word processor logic |
| `pdf-sheets` | Spreadsheet engine with 130+ functions |
| `pdf-slides` | Presentation engine |
| `pdf-formats` | File format parsers (docx, xlsx, pptx, odt) |
| `pdf-pdf` | PDF manipulation (lopdf) |
| `pdf-render` | SVG/raster renderer (tiny-skia) |
| `pdf-spell` | Spell checker |
| `pdf-storage` | SQLite storage layer |
| `pdf-server` | Axum HTTP + WebSocket server |
| `pdf-cli` | CLI entry point |

### Run Tests

```powershell
cargo test --workspace
```

### Apply All Lint Fixes

```powershell
cargo fix --workspace --allow-dirty
```

---

*Built with ❤️ in Rust · Local-first · Private by design*
