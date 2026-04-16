$rootDir = "c:\Users\chand\.gemini\pdf office"

# Rename directories
Get-ChildItem -Path $rootDir -Directory -Recurse | Where-Object {$_.Name -match "pdf"} | Sort-Object -Property @{Expression={$_.FullName.Length};Descending=$true} | ForEach-Object {
    $newName = $_.Name -replace "pdf", "pdf"
    Rename-Item -Path $_.FullName -NewName $newName -PassThru
}

# Rename pdf to pdf in files
$files = Get-ChildItem -Path $rootDir -File -Recurse -Exclude *.exe,*.zip,*.tar.gz,*.png,*.pack,*.idx
foreach ($file in $files) {
    if ($file.FullName -match "\\\.git\\") { continue }
    if ($file.FullName -match "\\target\\") { continue }
    if ($file.FullName -match "\\assets\\") { continue } # avoid replacing in binary assets if any
    
    $content = Get-Content -Path $file.FullName -Raw
    $newContent = $content -creplace "PDF Office", "PDF Office"
    $newContent = $newContent -creplace "PDF", "PDF"
    $newContent = $newContent -creplace "pdf_", "pdf_"
    $newContent = $newContent -creplace "pdf-", "pdf-"
    $newContent = $newContent -creplace "\.pdfo", ".pdfo"
    $newContent = $newContent -creplace "export_pdfo", "export_pdfo"
    $newContent = $newContent -creplace "import_pdfo", "import_pdfo"
    $newContent = $newContent -creplace "pdf", "pdf"
    
    if ($content -cne $newContent) {
        Set-Content -Path $file.FullName -Value $newContent -NoNewline
        Write-Host "Updated $($file.FullName)"
    }
}
