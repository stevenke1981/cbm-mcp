# Install cbm-mcp from GitHub Release (Windows x64).
#
# Usage:
#   irm https://raw.githubusercontent.com/stevenke1981/cbm-mcp/main/packaging/windows/install.ps1 | iex
#   $env:CBM_VERSION = "v0.2.1"; .\packaging\windows\install.ps1

param(
    [string]$Version = $(if ($env:CBM_VERSION) { $env:CBM_VERSION } elseif ($env:CBRLM_VERSION) { $env:CBRLM_VERSION } else { "latest" }),
    [string]$Repo = $(if ($env:CBM_REPO) { $env:CBM_REPO } elseif ($env:CBRLM_REPO) { $env:CBRLM_REPO } else { "stevenke1981/cbm-mcp" })
)

$ErrorActionPreference = "Stop"

$InstallDir = if ($env:CBM_INSTALL_DIR) { $env:CBM_INSTALL_DIR } elseif ($env:CBRLM_INSTALL_DIR) { $env:CBRLM_INSTALL_DIR } else { "$env:USERPROFILE\.config\cbm-mcp\bin" }
$Artifact = "cbm-mcp-windows-x64"
$Archive = "$Artifact.zip"

if ($Version -eq "latest") {
    $rel = Invoke-RestMethod "https://api.github.com/repos/$Repo/releases/latest"
    $Version = $rel.tag_name
}

$Url = "https://github.com/$Repo/releases/download/$Version/$Archive"
$Tmp = Join-Path $env:TEMP "cbm-mcp-install"
New-Item -ItemType Directory -Force -Path $Tmp, $InstallDir | Out-Null

$ChecksumsUrl = "https://github.com/$Repo/releases/download/$Version/SHA256SUMS.txt"
$ArchivePath = Join-Path $Tmp $Archive

Write-Host "Downloading $Url ..."
Invoke-WebRequest -Uri $Url -OutFile $ArchivePath

Write-Host "Verifying checksum ..."
$sums = Invoke-WebRequest -Uri $ChecksumsUrl -UseBasicParsing
$checksumText = if ($sums.Content -is [byte[]]) {
    [Text.Encoding]::UTF8.GetString($sums.Content)
} else {
    [string]$sums.Content
}
$expected = ($checksumText -split "`r?`n" | Where-Object { $_ -match "\s+$([regex]::Escape($Archive))`$" } | ForEach-Object { ($_ -split '\s+')[0] } | Select-Object -First 1)
if (-not $expected) {
    throw "checksum for $Archive not found in SHA256SUMS.txt"
}
$actual = (Get-FileHash -Path $ArchivePath -Algorithm SHA256).Hash.ToLower()
if ($actual -ne $expected.ToLower()) {
    throw "checksum mismatch for $Archive (expected $expected, got $actual)"
}

Expand-Archive -Path $ArchivePath -DestinationPath $Tmp -Force
$Extracted = Get-ChildItem -Path $Tmp -Filter "cbm.exe" -Recurse | Select-Object -First 1
if (-not $Extracted) { throw "cbm.exe not found in archive" }
$bin = Join-Path $InstallDir "cbm.exe"

$userPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($userPath -notlike "*$InstallDir*") {
    [Environment]::SetEnvironmentVariable("Path", "$InstallDir;$userPath", "User")
    $env:Path = "$InstallDir;$env:Path"
    Write-Host "Added $InstallDir to user PATH"
}

& $Extracted.FullName install --yes --all
if ($LASTEXITCODE -ne 0) { throw "cbm OpenCode configuration failed" }
if (-not (Test-Path -LiteralPath $bin)) { throw "cbm.exe was not installed to $bin" }

Write-Host ""
Write-Host "Installed cbm $Version -> $bin" -ForegroundColor Green
