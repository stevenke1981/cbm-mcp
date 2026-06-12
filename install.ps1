# Install codebase-memory-mcp from this checkout (Windows).
#
# Usage:
#   .\install.ps1
#   .\install.ps1 -SkipBuild -AllAgents

param(
    [switch]$SkipBuild,
    [switch]$AllAgents
)

$ErrorActionPreference = "Stop"
$Script = Join-Path $PSScriptRoot "scripts\install.ps1"
& $Script -SkipBuild:$SkipBuild -AllAgents:$AllAgents
