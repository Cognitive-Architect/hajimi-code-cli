$ErrorActionPreference = "Stop"
$repoRoot = Split-Path -Parent $PSScriptRoot
Set-Location $repoRoot
node tests/e2e/phase1-5-regression/full_chain.test.js
