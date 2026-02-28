#!/bin/bash
# =============================================================================
# wrtc Installation Script for Linux/Mac
# =============================================================================
# 安装真实wrtc模块，包含平台检测和编译工具安装
# 失败时直接退出，不提供Mock fallback
# =============================================================================

set -e

echo "[install-wrtc] ============================================"
echo "[install-wrtc] wrtc Installation Script (Linux/Mac)"
echo "[install-wrtc] ============================================"

# 检测平台
PLATFORM=$(uname -s)
ARCH=$(uname -m)
echo "[install-wrtc] Detected platform: $PLATFORM ($ARCH)"

# 检查Node.js版本
NODE_VERSION=$(node --version 2>/dev/null || echo "unknown")
echo "[install-wrtc] Node.js version: $NODE_VERSION"

# 安装编译依赖（根据平台）
install_deps() {
  case "$PLATFORM" in
    Linux)
      echo "[install-wrtc] Installing Linux build dependencies..."
      if command -v apt-get &> /dev/null; then
        # Debian/Ubuntu
        sudo apt-get update
        sudo apt-get install -y python3 make g++ gcc libssl-dev pkg-config
      elif command -v yum &> /dev/null; then
        # RHEL/CentOS
        sudo yum groupinstall -y "Development Tools"
        sudo yum install -y python3 openssl-devel
      elif command -v pacman &> /dev/null; then
        # Arch
        sudo pacman -S --noconfirm base-devel python3 openssl
      else
        echo "[install-wrtc] WARNING: Unknown package manager, please install build tools manually"
      fi
      ;;
    Darwin)
      echo "[install-wrtc] Installing macOS build dependencies..."
      # 检查Xcode Command Line Tools
      if ! xcode-select -p &> /dev/null; then
        echo "[install-wrtc] Installing Xcode Command Line Tools..."
        xcode-select --install
      fi
      # 检查Homebrew
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

# 检查Python
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

# 配置node-gyp
configure_node_gyp() {
  echo "[install-wrtc] Configuring node-gyp..."
  npm config set python "$PYTHON" 2>/dev/null || true
  # 设置国内镜像（可选）
  # npm config set registry https://registry.npmmirror.com
}

# 主安装流程
main() {
  echo "[install-wrtc] Step 1/4: Installing system dependencies..."
  install_deps
  
  echo "[install-wrtc] Step 2/4: Checking Python..."
  check_python
  
  echo "[install-wrtc] Step 3/4: Configuring node-gyp..."
  configure_node_gyp
  
  echo "[install-wrtc] Step 4/4: Installing wrtc package..."
  echo "[install-wrtc] This may take several minutes (native compilation)..."
  
  # 执行安装
  if npm install wrtc@^0.4.7; then
    echo "[install-wrtc] ============================================"
    echo "[install-wrtc] SUCCESS: wrtc installed successfully!"
    echo "[install-wrtc] ============================================"
    node -e "const w = require('wrtc'); console.log('[install-wrtc] wrtc version:', w.RTCPeerConnection ? 'OK' : 'FAILED')"
  else
    echo "[install-wrtc] ============================================"
    echo "[install-wrtc] ERROR: wrtc installation failed!"
    echo "[install-wrtc] Fallback strategy: Try @koush/wrtc alternative"
    echo "[install-wrtc]   npm install @koush/wrtc"
    echo "[install-wrtc] Or check: https://github.com/node-webrtc/node-webrtc"
    echo "[install-wrtc] ============================================"
    exit 1
  fi
}

# 执行主函数
main "$@"
