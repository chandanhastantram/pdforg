# publish.ps1 — publishes all pdforg crates to crates.io in dependency order
# Run this from the project root: .\publish.ps1

$ErrorActionPreference = "Stop"
$delay = 35  # seconds between publishes (crates.io rate limit)

function Publish-Crate {
    param([string]$CratePath, [string]$CrateName)
    Write-Host ""
    Write-Host "========================================" -ForegroundColor Cyan
    Write-Host "  Publishing: $CrateName" -ForegroundColor Cyan
    Write-Host "========================================" -ForegroundColor Cyan
    Push-Location $CratePath
    cargo publish --no-verify 2>&1
    Pop-Location
    Write-Host "  Waiting ${delay}s for crates.io to index..." -ForegroundColor Yellow
    Start-Sleep -Seconds $delay
}

Write-Host "PDF Office — Publishing all crates to crates.io" -ForegroundColor Green
Write-Host "This will take about 7 minutes total." -ForegroundColor Green
Write-Host ""

# Layer 1: no internal dependencies
Publish-Crate "crates\pdf-core"    "pdforg-core"

# Layer 2: depend only on pdforg-core
Publish-Crate "crates\pdf-spell"   "pdforg-spell"
Publish-Crate "crates\pdf-writer"  "pdforg-writer"
Publish-Crate "crates\pdf-sheets"  "pdforg-sheets"
Publish-Crate "crates\pdf-slides"  "pdforg-slides"
Publish-Crate "crates\pdf-pdf"     "pdforg-pdf"
Publish-Crate "crates\pdf-formats" "pdforg-formats"

# Layer 3: depend on core + writer
Publish-Crate "crates\pdf-render"  "pdforg-render"

# Layer 4: depend on all layer 2 crates
Publish-Crate "crates\pdf-storage" "pdforg-storage"

# Layer 5: full server
Publish-Crate "crates\pdf-server"  "pdforg-server"

# Layer 6: the CLI binary (the main install target)
Push-Location "crates\pdf-cli"
Write-Host ""
Write-Host "========================================" -ForegroundColor Green
Write-Host "  Publishing: pdforg (the binary!)" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Green
cargo publish --no-verify 2>&1
Pop-Location

Write-Host ""
Write-Host "========================================" -ForegroundColor Green
Write-Host "  ALL DONE!" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Green
Write-Host ""
Write-Host "Anyone in the world can now install PDF Office with:" -ForegroundColor White
Write-Host ""
Write-Host "  cargo install pdforg" -ForegroundColor Yellow
Write-Host ""
Write-Host "And run it with:" -ForegroundColor White
Write-Host ""
Write-Host "  pdf" -ForegroundColor Yellow
Write-Host ""
