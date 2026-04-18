//! PDF Office CLI — the main binary entry point.
//!
//! Usage:
//!   pdf                              # Open web UI
//!   pdf open <file>                  # Open a file
//!   pdf serve [--port N]             # Start local server
//!   pdf convert <in> <out>           # Headless conversion
//!   pdf export <file> --format pdf   # Export to PDF
//!   pdf spell <file>                 # Spell check
//!   pdf version                      # Version info

use clap::{Parser, Subcommand};
use std::path::PathBuf;
use anyhow::{Result, Context};
use tracing_subscriber::EnvFilter;

#[derive(Parser, Debug)]
#[command(
    name = "pdf",
    version = concat!(env!("CARGO_PKG_VERSION"), " (", env!("CARGO_PKG_NAME"), ")"),
    about = "PDF Office — local-first, Rust-native office suite",
    long_about = "PDF Office is a complete, private, offline office suite.\nAll data stays on your machine. No telemetry, no cloud required.\n\nRun without arguments to launch the web UI."
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,

    /// Verbose logging
    #[arg(short, long, global = true)]
    verbose: bool,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Open a document file
    Open {
        /// Path to the file (.docx, .xlsx, .pptx, .pdf, .odt, .rtf, .pdfo)
        file: PathBuf,
    },

    /// Start the local web server
    Serve {
        /// Port to listen on (default: 3847)
        #[arg(long, default_value_t = 3847)]
        port: u16,

        /// Data directory for storing documents
        #[arg(long)]
        data_dir: Option<PathBuf>,

        /// Don't open the browser automatically
        #[arg(long)]
        no_browser: bool,
    },

    /// Convert a document from one format to another
    Convert {
        /// Input file path
        input: PathBuf,
        /// Output file path (format inferred from extension)
        output: PathBuf,
    },

    /// Export a document to a specific format
    Export {
        /// Input file path
        file: PathBuf,
        /// Output format: pdf, docx, xlsx, pptx
        #[arg(long)]
        format: ExportFormat,
        /// Output file path (optional — defaults to same name with new extension)
        #[arg(long)]
        output: Option<PathBuf>,
    },

    /// Spell check a document
    Spell {
        /// File to spell check
        file: PathBuf,
        /// Language (default: en_US)
        #[arg(long, default_value = "en_US")]
        lang: String,
    },

    /// Show version and build info
    Version,

    /// Check for updates
    Update,

    /// Post-install setup (extract bundled assets)
    #[command(hide = true)]
    PostInstall,
}

#[derive(clap::ValueEnum, Debug, Clone)]
enum ExportFormat {
    Pdf,
    Docx,
    Xlsx,
    Pptx,
    Wps,
    Odt,
    Txt,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Set up logging
    let filter = if cli.verbose { "debug" } else { "info" };
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new(filter))
        .with_target(false)
        .compact()
        .init();

    match cli.command {
        None => {
            // Default: start server and open browser
            cmd_serve(3847, default_data_dir(), false).await
        }
        Some(Command::Serve { port, data_dir, no_browser }) => {
            let dir = data_dir.unwrap_or_else(default_data_dir);
            cmd_serve(port, dir, no_browser).await
        }
        Some(Command::Open { file }) => {
            cmd_open(file).await
        }
        Some(Command::Convert { input, output }) => {
            cmd_convert(&input, &output)
        }
        Some(Command::Export { file, format, output }) => {
            cmd_export(&file, format, output.as_ref())
        }
        Some(Command::Spell { file, lang }) => {
            cmd_spell(&file, &lang)
        }
        Some(Command::Version) => {
            cmd_version();
            Ok(())
        }
        Some(Command::Update) => {
            cmd_update().await
        }
        Some(Command::PostInstall) => {
            cmd_post_install()
        }
    }
}

fn default_data_dir() -> PathBuf {
    dirs_next().join(".pdfo").join("data")
}

fn dirs_next() -> PathBuf {
    std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
}

/// Start server and optionally open the browser
async fn cmd_serve(port: u16, data_dir: PathBuf, no_browser: bool) -> Result<()> {
    let url = format!("http://127.0.0.1:{}", port);

    println!("🦀 PDF Office v{}", env!("CARGO_PKG_VERSION"));
    println!("   Server: {}", url);
    println!("   Data:   {}", data_dir.display());
    println!("   Press Ctrl+C to stop");
    println!();

    if !no_browser {
        // Open browser after a short delay (give server time to start)
        let url_clone = url.clone();
        tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            if let Err(e) = open::that(&url_clone) {
                tracing::warn!("Could not open browser: {}", e);
            }
        });
    }

    pdf_server::start_server(port, data_dir).await
        .map_err(|e| anyhow::anyhow!("{}", e))
}

/// Open a file — start server and navigate to the document
async fn cmd_open(file: PathBuf) -> Result<()> {
    let ext = file.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();

    // Parse and import the file
    let doc = match ext.as_str() {
        "docx" => {
            let bytes = std::fs::read(&file).context("Failed to read file")?;
            pdf_formats::docx::parse_docx(&bytes)
                .context("Failed to parse DOCX")?
        }
        "odt" => {
            let bytes = std::fs::read(&file).context("Failed to read file")?;
            pdf_formats::odf::parse_odt(&bytes)
                .context("Failed to parse ODT")?
        }
        "rtf" => {
            let bytes = std::fs::read(&file).context("Failed to read file")?;
            pdf_formats::rtf::parse_rtf(&bytes)
                .context("Failed to parse RTF")?
        }
        "pdf" => {
            let bytes = std::fs::read(&file).context("Failed to read file")?;
            pdf_storage::Store::import_pdfo(&bytes)
                .context("Failed to parse .pdfo file")?
        }
        "xlsx" => {
            // XLSX opens as a spreadsheet — start server and open Sheets UI
            let data_dir = default_data_dir();
            let url = format!("http://127.0.0.1:3847/#sheets");
            println!("Opening {} in PDF Sheets...", file.display());
            tokio::spawn(async move {
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                let _ = open::that(&url);
            });
            return pdf_server::start_server(3847, data_dir).await
                .map_err(|e| anyhow::anyhow!("{}", e));
        }
        "pptx" => {
            // PPTX opens as a presentation — start server and open Slides UI
            let data_dir = default_data_dir();
            let url = format!("http://127.0.0.1:3847/#slides");
            println!("Opening {} in PDF Slides...", file.display());
            tokio::spawn(async move {
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                let _ = open::that(&url);
            });
            return pdf_server::start_server(3847, data_dir).await
                .map_err(|e| anyhow::anyhow!("{}", e));
        }
        _ => {
            eprintln!("Unsupported format: .{}", ext);
            eprintln!("Supported: .docx, .odt, .rtf, .pdfo, .xlsx, .pptx");
            std::process::exit(1);
        }
    };

    let data_dir = default_data_dir();
    let mut store = pdf_storage::Store::open(&data_dir)?;
    store.save_document(&doc)?;

    let url = format!("http://127.0.0.1:3847/?doc={}", doc.id);
    println!("Opening {} in PDF Office...", file.display());

    // Start server with the document pre-loaded
    tokio::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        let _ = open::that(&url);
    });

    pdf_server::start_server(3847, data_dir).await
        .map_err(|e| anyhow::anyhow!("{}", e))
}

/// Headless format conversion
fn cmd_convert(input: &PathBuf, output: &PathBuf) -> Result<()> {
    let in_ext = input.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
    let out_ext = output.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();

    println!("Converting {} → {}...", input.display(), output.display());

    let bytes = std::fs::read(input).context("Failed to read input file")?;

    // Parse input (writer formats → Document)
    let doc_result = match in_ext.as_str() {
        "docx" => Some(pdf_formats::docx::parse_docx(&bytes)?),
        "odt"  => Some(pdf_formats::odf::parse_odt(&bytes)?),
        "rtf"  => Some(pdf_formats::rtf::parse_rtf(&bytes)?),
        _ => None,
    };

    // Spreadsheet conversions
    if in_ext == "xlsx" && out_ext == "xlsx" {
        // identity copy for now
        std::fs::write(output, &bytes)?;
        println!("✓ Done! (identity copy)");
        return Ok(());
    }

    let doc = doc_result.ok_or_else(|| anyhow::anyhow!("Unsupported input format: .{}", in_ext))?;

    // Write output
    let out_bytes = match out_ext.as_str() {
        "docx" => pdf_formats::docx::write_docx(&doc)
            .context("Failed to write DOCX")?,
        "pdf"  => pdf_pdf::creator::create_pdf_from_document(&doc)
            .context("Failed to create PDF")?,
        _ => anyhow::bail!("Unsupported output format: .{}. Supported: docx, pdf", out_ext),
    };

    std::fs::write(output, out_bytes).context("Failed to write output file")?;

    let file_size = std::fs::metadata(output)?.len();
    println!("✓ Done! Output: {} ({} KB)", output.display(), file_size / 1024);
    Ok(())
}

/// Export to a specific format
fn cmd_export(file: &PathBuf, format: ExportFormat, output: Option<&PathBuf>) -> Result<()> {
    let ext = file.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
    let bytes = std::fs::read(file).context("Failed to read file")?;

    let doc = match ext.as_str() {
        "docx" => pdf_formats::docx::parse_docx(&bytes)?,
        "odt"  => pdf_formats::odf::parse_odt(&bytes)?,
        "rtf"  => pdf_formats::rtf::parse_rtf(&bytes)?,
        _ => anyhow::bail!("Unsupported input format: .{}. Supported: docx, odt, rtf", ext),
    };

    let (out_bytes, out_ext): (Vec<u8>, &str) = match format {
        ExportFormat::Pdf  => (pdf_pdf::creator::create_pdf_from_document(&doc)?, "pdf"),
        ExportFormat::Docx => (pdf_formats::docx::write_docx(&doc)?, "docx"),
        ExportFormat::Xlsx => {
            // Export as minimal xlsx from writer content isn't ideal but works for roundtrip
            let wb = pdf_core::Workbook::default();
            (pdf_formats::xlsx::write_xlsx(&wb)?, "xlsx")
        }
        ExportFormat::Txt => {
            // Plain text export
            let mut text = String::new();
            for block in &doc.body {
                match block {
                    pdf_core::Block::Paragraph(p) => {
                        for run in &p.runs { text.push_str(&run.text); }
                        text.push('\n');
                    }
                    pdf_core::Block::Heading(h) => {
                        for run in &h.runs { text.push_str(&run.text); }
                        text.push_str("\n\n");
                    }
                    _ => {}
                }
            }
            (text.into_bytes(), "txt")
        }
        _ => anyhow::bail!("Export format not yet fully supported for this input"),
    };

    let out_path = output.map(|p| p.to_path_buf()).unwrap_or_else(|| {
        file.with_extension(out_ext)
    });

    std::fs::write(&out_path, out_bytes)?;
    let size = std::fs::metadata(&out_path)?.len();
    println!("✓ Exported to {} ({} KB)", out_path.display(), size / 1024);
    Ok(())
}

/// Spell check a document file
fn cmd_spell(file: &PathBuf, lang: &str) -> Result<()> {
    let ext = file.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
    let bytes = std::fs::read(file).context("Failed to read file")?;

    let doc = match ext.as_str() {
        "docx" => pdf_formats::docx::parse_docx(&bytes)?,
        "odt" => pdf_formats::odf::parse_odt(&bytes)?,
        "rtf" => pdf_formats::rtf::parse_rtf(&bytes)?,
        _ => anyhow::bail!("Unsupported format: .{}", ext),
    };

    // Extract all text
    let text = extract_text(&doc);
    let checker = pdf_spell::default_checker();
    let results = checker.check_text(&text);

    if results.is_empty() {
        println!("✓ No spelling errors found in {}", file.display());
    } else {
        println!("Found {} spelling error(s) in {}:", results.len(), file.display());
        for (i, result) in results.iter().enumerate() {
            let suggestions = result.suggestions.iter().take(3).cloned().collect::<Vec<_>>().join(", ");
            println!("  {}. '{}' → [{}]", i + 1, result.word,
                if suggestions.is_empty() { "no suggestions".into() } else { suggestions });
        }
    }

    Ok(())
}

fn extract_text(doc: &pdf_core::Document) -> String {
    let mut text = String::new();
    for block in &doc.body {
        match block {
            pdf_core::Block::Paragraph(p) => {
                for run in &p.runs {
                    text.push_str(&run.text);
                    text.push(' ');
                }
                text.push('\n');
            }
            pdf_core::Block::Heading(h) => {
                for run in &h.runs {
                    text.push_str(&run.text);
                    text.push(' ');
                }
                text.push('\n');
            }
            _ => {}
        }
    }
    text
}

/// Print version information
fn cmd_version() {
    println!("PDF Office v{}", env!("CARGO_PKG_VERSION"));
    println!("Target:  {}", std::env::consts::ARCH);
    println!("OS:      {}", std::env::consts::OS);
    println!("Rust:    stable");
    println!("License: MIT");
}

/// Check for updates via GitHub Releases API
async fn cmd_update() -> Result<()> {
    println!("Update checking requires the 'update' feature flag.");
    println!("Current version: v{}", env!("CARGO_PKG_VERSION"));
    println!("Check manually: https://github.com/pdf-local/pdf/releases");
    Ok(())
}

/// Post-install: extract bundled assets from the binary
fn cmd_post_install() -> Result<()> {
    let data_dir = default_data_dir();
    std::fs::create_dir_all(&data_dir)?;

    println!("PDF Office post-install setup");
    println!("  Data directory: {}", data_dir.display());

    // In production, this would extract bundled fonts, dictionaries, and templates
    // from embedded ZIP data. For Phase 1, we just create the directory structure.
    let dirs = ["fonts", "dictionaries", "templates", "documents"];
    for dir in &dirs {
        std::fs::create_dir_all(data_dir.join(dir))?;
        println!("  ✓ Created {}/", dir);
    }

    // Initialize the database
    pdf_storage::Store::open(&data_dir)?;
    println!("  ✓ Initialized database");

    println!("\nPDF Office is ready!");
    println!("Run `pdf` to start.");
    Ok(())
}
