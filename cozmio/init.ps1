#Requires -Version 5.1
Write-Host "Initializing Cozmio development environment..." -Foreground Cyan

# Check required tools
$required = @()

# Node/pnpm
if (Test-Path "package.json") {
    $required += "pnpm"
}

# Rust/Cargo
if (Test-Path "Cargo.toml") {
    $required += "cargo"
}

# Tauri CLI
if (Test-Path "src-tauri/Cargo.toml") {
    $required += "tauri"
}

Write-Host "Environment check complete." -Foreground Green
Write-Host "Ready for: /discover-task, /plan-task, /implement-task"