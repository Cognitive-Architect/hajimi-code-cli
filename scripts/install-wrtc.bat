@echo off
chcp 65001 >nul
REM =============================================================================
REM wrtc Installation Script for Windows
REM =============================================================================
REM 安装真实wrtc模块，包含平台检测和编译工具安装
REM 失败时直接退出，不提供Mock fallback
REM =============================================================================

echo [install-wrtc] ============================================
echo [install-wrtc] wrtc Installation Script (Windows)
echo [install-wrtc] ============================================

REM 检测平台
set "PLATFORM=win32"
for /f "tokens=2 delims=[]" %%a in ('ver') do set "OS_VERSION=%%a"
echo [install-wrtc] Detected platform: Windows %OS_VERSION%

REM 检查Node.js版本
node --version >nul 2>&1
if %errorlevel% neq 0 (
  echo [install-wrtc] ERROR: Node.js not found. Please install Node.js first.
  exit /b 1
)
for /f "tokens=*" %%a in ('node --version') do set "NODE_VERSION=%%a"
echo [install-wrtc] Node.js version: %NODE_VERSION%

REM 检查Python
echo [install-wrtc] Step 1/4: Checking Python...
python --version >nul 2>&1
if %errorlevel% equ 0 (
  set "PYTHON_CMD=python"
  for /f "tokens=*" %%a in ('python --version') do echo [install-wrtc] Found: %%a
) else (
  python3 --version >nul 2>&1
  if %errorlevel% equ 0 (
    set "PYTHON_CMD=python3"
    for /f "tokens=*" %%a in ('python3 --version') do echo [install-wrtc] Found: %%a
  ) else (
    echo [install-wrtc] WARNING: Python not found. Attempting to use node-gyp bundled python...
    set "PYTHON_CMD=python"
  )
)

REM 安装windows-build-tools (如果需要)
echo [install-wrtc] Step 2/4: Checking build tools...
npm config get msvs_version >nul 2>&1
if %errorlevel% neq 0 (
  echo [install-wrtc] Installing windows-build-tools (requires admin privileges)...
  echo [install-wrtc] Please run: npm install --global windows-build-tools
  echo [install-wrtc] Or install Visual Studio Build Tools manually.
  echo [install-wrtc] Download from: https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2022
)

REM 配置node-gyp
echo [install-wrtc] Step 3/4: Configuring node-gyp...
if defined PYTHON_CMD (
  call npm config set python %PYTHON_CMD% 2>nul
)
call npm config set msvs_version 2022 2>nul || call npm config set msvs_version 2019 2>nul || echo [install-wrtc] Using default msvs_version

REM 安装wrtc
echo [install-wrtc] Step 4/4: Installing wrtc package...
echo [install-wrtc] This may take several minutes (native compilation)...
echo.

call npm install wrtc@^0.4.7
if %errorlevel% equ 0 (
  echo.
  echo [install-wrtc] ============================================
  echo [install-wrtc] SUCCESS: wrtc installed successfully!
  echo [install-wrtc] ============================================
  node -e "const w = require('wrtc'); console.log('[install-wrtc] wrtc loaded:', w.RTCPeerConnection ? 'OK' : 'FAILED');"
  exit /b 0
) else (
  echo.
  echo [install-wrtc] ============================================
  echo [install-wrtc] ERROR: wrtc installation failed!
  echo [install-wrtc].
  echo [install-wrtc] Fallback strategies:
  echo [install-wrtc]   1. Try alternative: npm install @koush/wrtc
  echo [install-wrtc]   2. Install Visual Studio Build Tools manually
  echo [install-wrtc]   3. Use prebuilt binary: npm install wrtc --build-from-source=false
  echo [install-wrtc].
  echo [install-wrtc] Reference: https://github.com/node-webrtc/node-webrtc
  echo [install-wrtc] ============================================
  exit /b 1
)
