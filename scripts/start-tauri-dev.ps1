# Hajimi IDE - Tauri Dev 启动脚本 (实机验收专用)
# 用于解决 debt 文档中记录的实机验收阻塞问题

$ErrorActionPreference = "Stop"
$RepoRoot = Split-Path -Parent $PSScriptRoot
$DesktopDir = Join-Path $RepoRoot "src\interface\desktop"
$WebDir = Join-Path $RepoRoot "src\interface\web"
$DevServerScript = Join-Path $RepoRoot "scripts\dev-server.js"
$LogDir = Join-Path $RepoRoot "logs\smoke"

# 创建日志目录
New-Item -ItemType Directory -Force -Path $LogDir | Out-Null

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  Hajimi IDE - 实机验收启动脚本" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# Step 1: 同步最新前端资源到 dist/dist
Write-Host "[Step 1/5] 同步前端资源到 dist/dist..." -ForegroundColor Yellow
$SyncScript = Join-Path $RepoRoot "scripts\sync-web-dist.js"
if (Test-Path $SyncScript) {
    node $SyncScript
    Write-Host "  -> 同步完成" -ForegroundColor Green
} else {
    Write-Warning "sync-web-dist.js 未找到，跳过同步"
}

# Step 2: 杀死残留的前端 dev server 进程
Write-Host "[Step 2/5] 清理残留的前端服务进程..." -ForegroundColor Yellow
$nodeProcs = Get-Process -Name "node" -ErrorAction SilentlyContinue | Where-Object {
    $_.CommandLine -match "dev-server" -or $_.CommandLine -match "serve"
}
if ($nodeProcs) {
    $nodeProcs | Stop-Process -Force
    Write-Host "  -> 已清理 $($nodeProcs.Count) 个残留 node 进程" -ForegroundColor Green
} else {
    Write-Host "  -> 无残留进程" -ForegroundColor Green
}

# Step 3: 清理 Cargo 增量编译缓存（解决 Windows 拒绝访问问题）
Write-Host "[Step 3/5] 清理 Cargo 增量编译缓存..." -ForegroundColor Yellow
$IncrementalDir = Join-Path $DesktopDir "target\debug\incremental"
if (Test-Path $IncrementalDir) {
    try {
        Remove-Item -Recurse -Force $IncrementalDir -ErrorAction Stop
        Write-Host "  -> 增量缓存已清理" -ForegroundColor Green
    } catch {
        Write-Warning "  -> 清理增量缓存失败（可能被占用）: $_"
        Write-Host "  -> 尝试重启后重试，或手动删除 $IncrementalDir" -ForegroundColor Yellow
    }
} else {
    Write-Host "  -> 无增量缓存需清理" -ForegroundColor Green
}

# Step 4: 启动前端 dev server（纯 Node.js，不依赖 npm serve）
Write-Host "[Step 4/5] 启动前端 dev server (http://localhost:3456)..." -ForegroundColor Yellow
$DevServerLog = Join-Path $LogDir "dev-server.log"
$DevServerJob = Start-Job -ScriptBlock {
    param($scriptPath, $logPath)
    node $scriptPath > $logPath 2>&1
} -ArgumentList $DevServerScript, $DevServerLog

# 等待服务启动
$maxWait = 30
$waited = 0
$serverReady = $false
while ($waited -lt $maxWait) {
    Start-Sleep -Seconds 1
    $waited++
    try {
        $response = Invoke-WebRequest -Uri "http://127.0.0.1:3456" -UseBasicParsing -TimeoutSec 2 -ErrorAction Stop
        if ($response.StatusCode -eq 200) {
            $serverReady = $true
            break
        }
    } catch {
        # 继续等待
    }
}

if ($serverReady) {
    Write-Host "  -> 前端服务已就绪 (http://127.0.0.1:3456)" -ForegroundColor Green
} else {
    Write-Error "前端服务启动超时！请检查 $DevServerLog"
    Stop-Job $DevServerJob
    exit 1
}

# Step 5: 启动 Tauri Dev
Write-Host "[Step 5/5] 启动 Tauri Dev..." -ForegroundColor Yellow
Write-Host ""
Write-Host "========================================" -ForegroundColor Green
Write-Host "  实机验收启动成功！" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Green
Write-Host ""
Write-Host "验收检查清单:" -ForegroundColor Cyan
Write-Host "  [ ] 启动后无 '加载文件树失败' toast"
Write-Host "  [ ] 文件树正确渲染 workspace"
Write-Host "  [ ] 右侧 Inspector 有 Agent Trace tab"
Write-Host "  [ ] 输入 /agent <目标> 触发 Agent 任务"
Write-Host "  [ ] Trace 事件非零计数"
Write-Host "  [ ] 高风险操作触发审批弹窗 (Day 6)"
Write-Host "  [ ] Checkpoint badge 在 Edit 步骤显示 (Day 7)"
Write-Host ""
Write-Host "日志位置: $LogDir" -ForegroundColor DarkGray
Write-Host ""

$TauriLog = Join-Path $LogDir "tauri-dev.log"
Push-Location $DesktopDir
try {
    cargo tauri dev 2>&1 | Tee-Object -FilePath $TauriLog
} finally {
    Pop-Location
    Write-Host ""
    Write-Host "[清理] 停止前端 dev server..." -ForegroundColor Yellow
    Stop-Job $DevServerJob -ErrorAction SilentlyContinue
    Remove-Job $DevServerJob -ErrorAction SilentlyContinue
    Write-Host "  -> 已清理" -ForegroundColor Green
}
