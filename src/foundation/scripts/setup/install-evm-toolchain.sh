#!/bin/bash
#
# EVM Toolchain Installation Script for Linux/Mac
# Installs: Foundry (forge, cast, anvil) + Slither
# Requirements: curl, python3 >= 3.8, pip3
#

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "=========================================="
echo "  EVM Toolchain Installer (Linux/Mac)"
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

# Install Foundry
echo -e "${YELLOW}[3/5] Installing Foundry...${NC}"
if command -v forge &> /dev/null; then
    FORGE_VERSION=$(forge --version 2>&1 | head -1)
    echo -e "${GREEN}✓ Foundry already installed: $FORGE_VERSION${NC}"
else
    echo "Downloading Foundry from official source..."
    curl -L https://foundry.paradigm.xyz | bash
    
    # Source the profile to get foundryup in PATH
    export PATH="$HOME/.foundry/bin:$PATH"
    
    # Run foundryup
    $HOME/.foundry/bin/foundryup
    
    FORGE_VERSION=$(forge --version 2>&1 | head -1)
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
