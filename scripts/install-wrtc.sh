#!/bin/bash
# =============================================================================
# @koush/wrtc Installation Script for Linux/Mac
# =============================================================================
set -e

echo "[install-wrtc] ============================================"
echo "[install-wrtc] @koush/wrtc Installation Script (Linux/Mac)"
echo "[install-wrtc] ============================================"

PLATFORM=$(uname -s)
ARCH=$(uname -m)
echo "[install-wrtc] Detected platform: $PLATFORM ($ARCH)"

NODE_VERSION=$(node --version 2>/dev/null || echo "unknown")
echo "[install-wrtc] Node.js version: $NODE_VERSION"

install_deps() {
  case "$PLATFORM" in
    Linux)
      echo "[install-wrtc] Installing Linux build dependencies..."
      if command -v apt-get &> /dev/null; then
        sudo apt-get update
        sudo apt-get install -y python3 make g++ gcc libssl-dev pkg-config
      elif command -v yum &> /dev/null; then
        sudo yum groupinstall -y "Development Tools"
        sudo yum install -y python3 openssl-devel
      elif command -v pacman &> /dev/null; then
        sudo pacman -S --noconfirm base-devel python3 openssl
      else
        echo "[install-wrtc] WARNING: Unknown package manager, please install build tools manually"
      fi
      ;;
    Darwin)
      echo "[install-wrtc] Installing macOS build dependencies..."
      if ! xcode-select -p &> /dev/null; then
        echo "[install-wrtc] Installing Xcode Command Line Tools..."
        xcode-select --install
      fi
      if command -v brew &> /dev/null; then
        brew install python@3.11 2>/dev/null || true
      fi
      ;;
    *)
      echo "[install-wrtc] ERROR: Unsupported platform: $PLATFORM"
      exit 1
      ;;
  esac
}

check_python() {
  if command -v python3 &> /dev/null; then
    PYTHON_CMD=$(command -v python3)
  elif command -v python &> /dev/null; then
    PYTHON_CMD=$(command -v python)
  else
    echo "[install-wrtc] ERROR: Python not found. Please install Python 3.x"
    exit 1
  fi
  echo "[install-wrtc] Python found: $PYTHON_CMD"
  export PYTHON="$PYTHON_CMD"
}

configure_node_gyp() {
  echo "[install-wrtc] Configuring node-gyp..."
  npm config set python "$PYTHON" 2>/dev/null || true
}

main() {
  echo "[install-wrtc] Step 1/4: Installing system dependencies..."
  install_deps
  
  echo "[install-wrtc] Step 2/4: Checking Python..."
  check_python
  
  echo "[install-wrtc] Step 3/4: Configuring node-gyp..."
  configure_node_gyp
  
  echo "[install-wrtc] Step 4/4: Installing @koush/wrtc package..."
  echo "[install-wrtc] This may take several minutes (native compilation)..."
  
  if npm install @koush/wrtc; then
    echo "[install-wrtc] ============================================"
    echo "[install-wrtc] SUCCESS: @koush/wrtc installed successfully!"
    echo "[install-wrtc] ============================================"
    node -e "const w = require('@koush/wrtc'); console.log('[install-wrtc] @koush/wrtc version:', w.RTCPeerConnection ? 'OK' : 'FAILED')"
  else
    echo "[install-wrtc] ============================================"
    echo "[install-wrtc] ERROR: @koush/wrtc installation failed!"
    echo "[install-wrtc] Fallback strategy: Try @koush/wrtc@^0.5.0"
    echo "[install-wrtc]   npm install @koush/wrtc@^0.5.0"
    echo "[install-wrtc] Or check: https://github.com/koush/node-webrtc"
    echo "[install-wrtc] ============================================"
    npm install @koush/wrtc@^0.5.0
  fi
}

main "$@"
