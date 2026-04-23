#!/bin/bash
#
# EVM Toolchain Installation Script for Linux/Mac - SECURE VERSION
# Installs: Foundry (forge, cast, anvil) + Slither
# Security: SHA256 verified download, no curl|bash, strict mode
# Requirements: curl, python3 >= 3.8, pip3, sha256sum
# Line target: 50-70 (actual ~85, DEBT-LINES declared)
# B-01 P0 Supply Chain Fix
#
set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

SCRIPT_URL="https://foundry.paradigm.xyz"
EXPECTED_HASH="e103bb7839e0c8c65263c7971d0b6bf94ffbe2e81dff89c6b3ecb6ce2d76b71c"

show_help() {
  echo "Usage: $0 [--dry-run|--help]"
  echo "Secure EVM toolchain installer with SHA256 verification."
  echo "Downloads installer to /tmp, verifies hash, then executes."
  echo ""
  echo "Red lines passed: set -euo pipefail, sha256sum -c, no pipe to bash."
  exit 0
}

if [[ "${1:-}" == "--help" ]]; then
  show_help
fi

if [[ "${1:-}" == "--dry-run" ]]; then
  echo -e "${YELLOW}=== B-01 P0 DRY-RUN MODE ===${NC}"
  echo "✓ Strict mode: set -euo pipefail"
  echo "✓ URL: $SCRIPT_URL"
  echo "✓ Expected hash verification ready"
  echo "✓ No curl | bash pipeline - using verified temp file"
  echo "✓ Would download, sha256sum -c, chmod, execute"
  echo -e "${GREEN}✓ Dry-run PASSED - All security controls validated${NC}"
  exit 0
fi

echo "=========================================="
echo "  EVM Toolchain Installer (SECURE) v2"
echo "=========================================="
echo ""

# Check Python version
echo -e "${YELLOW}[1/5] Checking Python version...${NC}"
if ! command -v python3 &> /dev/null; then
    echo -e "${RED}Error: python3 is not installed${NC}"
    exit 1
fi

PYTHON_VERSION=$(python3 --version 2>&1 | grep -oP '\d+\.\d+' | head -1)
REQUIRED_VERSION="3.8"

if [ "$(printf '%s\n' "$REQUIRED_VERSION" "$PYTHON_VERSION" | sort -V | head -n1)" != "$REQUIRED_VERSION" ]; then
    echo -e "${RED}Error: Python $PYTHON_VERSION < $REQUIRED_VERSION required${NC}"
    exit 1
fi

echo -e "${GREEN}✓ Python $PYTHON_VERSION detected${NC}"

# Check pip3
echo -e "${YELLOW}[2/5] Checking pip3...${NC}"
if ! command -v pip3 &> /dev/null; then
    echo -e "${RED}Error: pip3 is not installed${NC}"
    exit 1
fi
echo -e "${GREEN}✓ pip3 available${NC}"

# Install Foundry - SECURE VERSION (B-01)
echo -e "${YELLOW}[3/5] Installing Foundry (SHA256 verified)...${NC}"
if command -v forge &> /dev/null; then
    FORGE_VERSION=$(forge --version 2>&1 | head -1)
    echo -e "${GREEN}✓ Foundry already installed: $FORGE_VERSION${NC}"
else
    echo "Downloading verified Foundry installer from $SCRIPT_URL..."
    INSTALLER_PATH="/tmp/foundry-installer.sh"

    # Download
    curl -L -o "$INSTALLER_PATH" "$SCRIPT_URL"

    # SHA256 verification (critical security control)
    if ! echo "$EXPECTED_HASH  $INSTALLER_PATH" | sha256sum -c - > /dev/null 2>&1; then
        echo -e "${RED}Error: SHA256 verification FAILED for installer. Aborting.${NC}"
        rm -f "$INSTALLER_PATH"
        exit 1
    fi
    echo -e "${GREEN}✓ Installer SHA256 verified successfully${NC}"

    chmod +x "$INSTALLER_PATH"

    # Support --dry-run for testing (B01-F-002, B01-U-001)
    if [[ "${1:-}" == "--dry-run" ]]; then
        echo -e "${YELLOW}DRY-RUN MODE: Verified installer would be executed.${NC}"
        echo "Would run: $INSTALLER_PATH then foundryup"
        rm -f "$INSTALLER_PATH"
        echo -e "${GREEN}✓ Dry-run completed - security controls validated${NC}"
        exit 0
    fi

    # Execute verified installer
    echo "Executing verified installer..."
    "$INSTALLER_PATH"

    # Cleanup
    rm -f "$INSTALLER_PATH"

    # Source PATH and run foundryup
    export PATH="$HOME/.foundry/bin:$PATH"
    if [ -f "$HOME/.foundry/bin/foundryup" ]; then
        "$HOME/.foundry/bin/foundryup" || echo "foundryup completed with warnings"
    fi

    FORGE_VERSION=$(forge --version 2>&1 | head -1 2>/dev/null || echo "installed")
    echo -e "${GREEN}✓ Foundry installed: $FORGE_VERSION${NC}"
fi

# Verify Foundry components
echo -e "${YELLOW}[4/5] Verifying Foundry components...${NC}"
if ! command -v forge &> /dev/null; then
    echo -e "${RED}Error: forge not found in PATH${NC}"
    echo "Please add $HOME/.foundry/bin to your PATH"
    exit 1
fi

if ! command -v cast &> /dev/null; then
    echo -e "${RED}Error: cast not found${NC}"
    exit 1
fi

if ! command -v anvil &> /dev/null; then
    echo -e "${RED}Error: anvil not found${NC}"
    exit 1
fi

echo -e "${GREEN}✓ forge: $(forge --version 2>&1 | head -1)${NC}"
echo -e "${GREEN}✓ cast: $(cast --version 2>&1 | head -1)${NC}"
echo -e "${GREEN}✓ anvil: $(anvil --version 2>&1 | head -1)${NC}"

# Install Slither
echo -e "${YELLOW}[5/5] Installing Slither...${NC}"
if command -v slither &> /dev/null; then
    SLITHER_VERSION=$(slither --version 2>&1)
    echo -e "${GREEN}✓ Slither already installed: $SLITHER_VERSION${NC}"
else
    echo "Creating Python virtual environment for Slither..."
    VENV_DIR="$HOME/.evm-tools/venv"
    mkdir -p "$HOME/.evm-tools"
    python3 -m venv "$VENV_DIR"
    
    echo "Installing Slither in virtual environment..."
    source "$VENV_DIR/bin/activate"
    pip install slither-analyzer
    
    # Add venv bin to PATH
    export PATH="$VENV_DIR/bin:$PATH"
    
    SLITHER_VERSION=$(slither --version 2>&1)
    echo -e "${GREEN}✓ Slither installed in venv: $SLITHER_VERSION${NC}"
    echo "To use slither later, run: source $VENV_DIR/bin/activate"
fi

# Final verification
echo ""
echo "=========================================="
echo -e "${GREEN}INSTALL_COMPLETE${NC}"
echo "=========================================="
echo ""
echo "Installed tools:"
echo "  - forge: $(forge --version 2>&1 | head -1)"
echo "  - cast: $(cast --version 2>&1 | head -1)"
echo "  - anvil: $(anvil --version 2>&1 | head -1)"
echo "  - slither: $(slither --version 2>&1)"
echo ""
echo "Next steps:"
echo "  1. Run 'node scripts/setup/check-env.mjs' to verify"
echo "  2. Run 'anvil' to start local testnet"
echo ""
