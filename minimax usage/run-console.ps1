#!/usr/bin/env pwsh
# Run the binary in console mode (one-shot usage dump, no tray)
#
# Usage: .\run-console.ps1
#
# Loads config from %APPDATA%\token usage\config.json, hits each enabled
# provider once, prints results, and exits.

$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $MyInvocation.MyCommand.Path
$bin = Join-Path $root 'target\release\token_usage.exe'

if (-not (Test-Path $bin)) {
    Write-Host "Binary not found at $bin. Run .\build.ps1 first." -ForegroundColor Yellow
    exit 1
}

& $bin --console