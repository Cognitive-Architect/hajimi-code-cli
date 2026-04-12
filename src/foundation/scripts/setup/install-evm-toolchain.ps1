# EVM Toolchain Installation Script for Windows
# Installs: Foundry (forge, cast, anvil) + Slither
# Requirements: PowerShell 5.1+, Python 3.8+, pip
$ErrorActionPreference = "Stop"
function Write-Color($Text, $Color) {
    switch ($Color) {
        "Red" { Write-Host $Text -ForegroundColor Red }
        "Green" { Write-Host $Text -ForegroundColor Green }
        "Yellow" { Write-Host $Text -ForegroundColor Yellow }
        default { Write-Host $Text }
    }
}

Write-Host "==========================================" 
Write-Host "  EVM Toolchain Installer (Windows)"
Write-Host "=========================================="
Write-Host ""

# Check Python version
Write-Color "[1/5] Checking Python version..." "Yellow"
$pythonCmd = Get-Command python3 -ErrorAction SilentlyContinue
if (-not $pythonCmd) {
    $pythonCmd = Get-Command python -ErrorAction SilentlyContinue
}

if (-not $pythonCmd) {
    Write-Color "Error: Python is not installed" "Red"
    exit 1
}

$pythonVersion = & $pythonCmd.Source --version 2>&1
$versionMatch = $pythonVersion -match '(\d+)\.(\d+)'
if (-not $versionMatch) {
    Write-Color "Error: Could not detect Python version" "Red"
    exit 1
}

$major = [int]$matches[1]
$minor = [int]$matches[2]

if ($major -lt 3 -or ($major -eq 3 -and $minor -lt 8)) {
    Write-Color "Error: Python $major.$minor < 3.8 required" "Red"
    exit 1
}

Write-Color "✓ Python $major.$minor detected" "Green"

# Check pip
Write-Color "[2/5] Checking pip..." "Yellow"
$pipCmd = Get-Command pip3 -ErrorAction SilentlyContinue
if (-not $pipCmd) {
    $pipCmd = Get-Command pip -ErrorAction SilentlyContinue
}

if (-not $pipCmd) {
    Write-Color "Error: pip is not installed" "Red"
    exit 1
}
Write-Color "✓ pip available" "Green"

# Install Foundry
Write-Color "[3/5] Installing Foundry..." "Yellow"
$forgeCmd = Get-Command forge -ErrorAction SilentlyContinue
if ($forgeCmd) {
    $forgeVersion = & forge --version 2>&1 | Select-Object -First 1
    Write-Color "✓ Foundry already installed: $forgeVersion" "Green"
} else {
    Write-Host "Downloading Foundry from official source..."
    
    # Download and run foundry installer
    $tempDir = [System.IO.Path]::GetTempPath()
    $installerPath = Join-Path $tempDir "foundry-installer.ps1"
    
    Invoke-WebRequest -Uri "https://foundry.paradigm.xyz" -OutFile $installerPath
    
    # Run installer
    & $installerPath
    
    # Add to PATH for current session
    $env:PATH = "$env:USERPROFILE\.foundry\bin;$env:PATH"
    
    # Refresh PATH
    $forgePath = Join-Path $env:USERPROFILE ".foundry\bin\forge.exe"
    if (Test-Path $forgePath) {
        $forgeVersion = & $forgePath --version 2>&1 | Select-Object -First 1
        Write-Color "✓ Foundry installed: $forgeVersion" "Green"
    } else {
        Write-Color "Warning: Please restart terminal or add $env:USERPROFILE\.foundry\bin to PATH" "Yellow"
    }
}

# Verify Foundry components
Write-Color "[4/5] Verifying Foundry components..." "Yellow"
$forgeCmd = Get-Command forge -ErrorAction SilentlyContinue
if (-not $forgeCmd) {
    # Try user profile path
    $forgePath = Join-Path $env:USERPROFILE ".foundry\bin\forge.exe"
    if (Test-Path $forgePath) {
        $env:PATH = "$env:USERPROFILE\.foundry\bin;$env:PATH"
    } else {
        Write-Color "Error: forge not found in PATH" "Red"
        Write-Host "Please add $env:USERPROFILE\.foundry\bin to your PATH"
        exit 1
    }
}

$castCmd = Get-Command cast -ErrorAction SilentlyContinue
$anvilCmd = Get-Command anvil -ErrorAction SilentlyContinue

if (-not $castCmd) {
    Write-Color "Error: cast not found" "Red"
    exit 1
}

if (-not $anvilCmd) {
    Write-Color "Error: anvil not found" "Red"
    exit 1
}

Write-Color "✓ forge: $((forge --version 2>&1 | Select-Object -First 1))" "Green"
Write-Color "✓ cast: $((cast --version 2>&1 | Select-Object -First 1))" "Green"
Write-Color "✓ anvil: $((anvil --version 2>&1 | Select-Object -First 1))" "Green"

# Install Slither
Write-Color "[5/5] Installing Slither..." "Yellow"
$slitherCmd = Get-Command slither -ErrorAction SilentlyContinue
if ($slitherCmd) {
    $slitherVersion = & slither --version 2>&1
    Write-Color "✓ Slither already installed: $slitherVersion" "Green"
} else {
    Write-Host "Installing Slither via pip..."
    & $pipCmd.Source install slither-analyzer
    
    # Refresh PATH
    $env:PATH = "$env:USERPROFILE\AppData\Roaming\Python\Python$major$minor\Scripts;$env:PATH"
    $env:PATH = "$env:USERPROFILE\.local\bin;$env:PATH"
    
    $slitherVersion = & slither --version 2>&1
    Write-Color "✓ Slither installed: $slitherVersion" "Green"
}

# Final verification
Write-Host ""
Write-Host "=========================================="
Write-Color "INSTALL_COMPLETE" "Green"
Write-Host "=========================================="
Write-Host ""
Write-Host "Installed tools:"
Write-Host "  - forge: $((forge --version 2>&1 | Select-Object -First 1))"
Write-Host "  - cast: $((cast --version 2>&1 | Select-Object -First 1))"
Write-Host "  - anvil: $((anvil --version 2>&1 | Select-Object -First 1))"
Write-Host "  - slither: $((slither --version 2>&1))"
Write-Host ""
Write-Host "Next steps:"
Write-Host "  1. Run 'node scripts/setup/check-env.mjs' to verify"
Write-Host "  2. Run 'anvil' to start local testnet"
Write-Host ""
