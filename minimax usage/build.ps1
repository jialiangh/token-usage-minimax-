#!/usr/bin/env pwsh
# Build token_usage: Rust binary + Inno Setup installer
#
# Usage: .\build.ps1
#
# Requirements:
#   - Rust toolchain (stable-x86_64-pc-windows-msvc or -gnu)
#   - Inno Setup 6+ (https://jrsoftware.org/isdownload.php)
#   - For installer: ISCC.exe on PATH (default: C:\Program Files (x86)\Inno Setup 6\ISCC.exe)

$ErrorActionPreference = 'Stop'

$root = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location $root

Write-Host "==> Building Rust binary (release)..."
cargo build --release

$bin = Join-Path $root 'target\release\token_usage.exe'
if (-not (Test-Path $bin)) {
    throw "Build failed: $bin not found"
}
$size = (Get-Item $bin).Length
Write-Host "    -> $bin ($([math]::Round($size/1024/1024, 2)) MB)" -ForegroundColor Green

Write-Host ""
Write-Host "==> Building installer..."
$iscc = (Get-Command ISCC.exe -ErrorAction SilentlyContinue)?.Source
if (-not $iscc) {
    $candidates = @(
        "${env:ProgramFiles(x86)}\Inno Setup 6\ISCC.exe",
        "${env:ProgramFiles}\Inno Setup 6\ISCC.exe",
        "C:\Program Files (x86)\Inno Setup\ISCC.exe"
    )
    foreach ($c in $candidates) { if (Test-Path $c) { $iscc = $c; break } }
}
if (-not $iscc) {
    Write-Warning "ISCC.exe not found; skipping installer. Install Inno Setup 6+ to build the installer."
} else {
    Push-Location (Join-Path $root 'installer')
    & $iscc token_usage.iss | Out-Null
    Pop-Location
    $installer = Join-Path $root 'installer\dist\token_usage_setup_1.0.0.exe'
    if (Test-Path $installer) {
        $isize = (Get-Item $installer).Length
        Write-Host "    -> $installer ($([math]::Round($isize/1024/1024, 2)) MB)" -ForegroundColor Green
    }
}

Write-Host ""
Write-Host "Done." -ForegroundColor Green