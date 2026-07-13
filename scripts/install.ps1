<#
.SYNOPSIS
    Cross-platform (Windows) installer for the zed-pike dev extension.

.DESCRIPTION
    Strategy: build + verify + drive Zed. We do NOT write into Zed's internal
    extension database (that format is version-specific and fragile). Instead:
      1. Preflight the toolchain Zed needs to compile the extension.
      2. Build the wasm bridge + pike-lsp; run the headless verify harness so a
         broken tree fails HERE, not silently inside Zed.
      3. Hand off to Zed's own supported dev-extension install (which downloads
         wasi-sdk, compiles the grammar, and registers + rebuilds the extension).

.PARAMETER NoOpen
    Build + verify only; print the manual install step instead of opening Zed.

.PARAMETER SkipVerify
    Skip the headless verification harness.

.EXAMPLE
    pwsh scripts/install.ps1
    pwsh scripts/install.ps1 -NoOpen
#>
[CmdletBinding()]
param(
    [switch]$NoOpen,
    [switch]$SkipVerify
)
$ErrorActionPreference = 'Stop'
$Root = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
Set-Location $Root

function Say ($m) { Write-Host "==> $m" -ForegroundColor Cyan }
function Die ($m) { Write-Host "ERROR: $m" -ForegroundColor Red; exit 1 }

# --- 1. preflight -----------------------------------------------------------
Say 'Preflight: toolchain'
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) { Die 'cargo not found - install Rust (https://rustup.rs)' }
if (-not (Get-Command rustc -ErrorAction SilentlyContinue)) { Die 'rustc not found' }
$targets = & rustup target list --installed 2>$null
if ($targets -notcontains 'wasm32-wasip2') { Say 'Adding wasm32-wasip2 target'; & rustup target add wasm32-wasip2 }

$ZedBin = (Get-Command zed -ErrorAction SilentlyContinue).Source
if (-not $ZedBin) {
    $cand = Join-Path $env:LOCALAPPDATA 'Programs\Zed\Zed.exe'
    if (Test-Path $cand) { $ZedBin = $cand }
}
Write-Host "    cargo: $(cargo --version)"
Write-Host "    zed:   $(if ($ZedBin) { $ZedBin } else { '<not found on PATH>' })"

# --- 2. build + verify ------------------------------------------------------
Say 'Build: wasm bridge (exact Zed build command)'
& cargo build --release --target wasm32-wasip2
if ($LASTEXITCODE -ne 0) { Die 'wasm build failed' }
Say 'Build: pike-lsp (native language server)'
& cargo build --release -p pike-lsp
if ($LASTEXITCODE -ne 0) { Die 'pike-lsp build failed' }

if (-not $SkipVerify) {
    Say 'Verify: headless harness'
    # verify.sh is bash; on Windows use Git Bash / WSL if present, else skip LSP/grammar stages.
    $bash = (Get-Command bash -ErrorAction SilentlyContinue).Source
    if ($bash) {
        & $bash ./scripts/verify.sh
        if ($LASTEXITCODE -ne 0) { Die 'verification failed' }
    } else {
        Say 'bash not found; running minimal Python LSP smoke instead'
        & python scripts/lsp_smoke.py target/release/pike-lsp.exe fixtures/syntax/basic.pike --max-rss-mb 80
        if ($LASTEXITCODE -ne 0) { Die 'LSP smoke failed' }
    }
} else {
    Say 'Verify: skipped (-SkipVerify)'
}

# --- 3. drive Zed's supported install --------------------------------------
Write-Host ''
Write-Host 'Artifacts built and verified.' -ForegroundColor Green
Write-Host @"

To install into Zed (one-time, then Zed auto-rebuilds on launch):
  1. Open the command palette (Ctrl-Shift-P)
  2. Run:  zed: install dev extension
  3. Select this directory:  $Root
  4. Open a .pike / .pmod / .cmod file.
"@

if ((-not $NoOpen) -and $ZedBin) {
    Say 'Opening Zed on the project (run the palette command above)'
    Start-Process -FilePath $ZedBin -ArgumentList $Root | Out-Null
}
