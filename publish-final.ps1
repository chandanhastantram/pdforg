# publish-final.ps1 — publishes the last 5 crates
$delay = 40

function Pub {
    param([string]$path, [string]$name)
    Write-Host ""
    Write-Host "Publishing: $name"
    Write-Host "--------"
    Push-Location $path
    cargo publish --no-verify
    Pop-Location
    Write-Host "Done. Waiting ${delay}s..."
    Start-Sleep -Seconds $delay
}

Pub "crates\pdf-formats" "pdforg-formats"
Pub "crates\pdf-render"  "pdforg-render"
Pub "crates\pdf-storage" "pdforg-storage"
Pub "crates\pdf-server"  "pdforg-server"

Write-Host ""
Write-Host "Publishing the FINAL crate: pdforg (the binary!)"
Write-Host "--------"
Push-Location "crates\pdf-cli"
cargo publish --no-verify
Pop-Location

Write-Host ""
Write-Host "DONE! Install with:"
Write-Host "  cargo install pdforg"
