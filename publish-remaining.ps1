# publish-remaining.ps1 — publishes the remaining 6 crates (after rate limit reset)
$delay = 35

function Publish-Crate {
    param([string]$CratePath, [string]$CrateName)
    Write-Host ""
    Write-Host "========================================"
    Write-Host "  Publishing: $CrateName"
    Write-Host "========================================"
    Push-Location $CratePath
    cargo publish --no-verify
    if ($LASTEXITCODE -ne 0) {
        Write-Host "  FAILED - waiting 10 min and retrying..."
        Start-Sleep -Seconds 600
        cargo publish --no-verify
    }
    Pop-Location
    Write-Host "  Waiting ${delay}s for crates.io to index..."
    Start-Sleep -Seconds $delay
}

Write-Host "PDF Office - Publishing remaining crates (rate limit reset)"
Write-Host "Waiting 5 minutes for rate limit to fully clear..."
Start-Sleep -Seconds 300

Publish-Crate "crates\pdf-pdf"     "pdforg-pdf"
Publish-Crate "crates\pdf-formats" "pdforg-formats"
Publish-Crate "crates\pdf-render"  "pdforg-render"
Publish-Crate "crates\pdf-storage" "pdforg-storage"
Publish-Crate "crates\pdf-server"  "pdforg-server"

Write-Host ""
Write-Host "========================================"
Write-Host "Publishing: pdforg (the final binary!)"
Write-Host "========================================"
Push-Location "crates\pdf-cli"
cargo publish --no-verify
Pop-Location

Write-Host ""
Write-Host "======================================== COMPLETE!"
Write-Host ""
Write-Host "Anyone can now install PDF Office with:"
Write-Host "  cargo install pdforg"
Write-Host ""
Write-Host "And run it with:"
Write-Host "  pdf"
