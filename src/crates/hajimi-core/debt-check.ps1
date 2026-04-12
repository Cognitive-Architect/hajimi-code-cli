# HAJIMI Core - Debt Check Script
# 债务检查 PowerShell 脚本

Write-Host "=== Debt Check ===" -ForegroundColor Cyan

# 检查 unwrap 在生产代码中的使用
Write-Host "`nScanning for unwrap in production code..." -ForegroundColor Yellow
$unwrapFiles = Get-ChildItem -Path "src" -Recurse -Filter "*.rs" | 
    Select-String -Pattern "unwrap\(\)" | 
    Where-Object { $_.FileName -notmatch "test" -and $_.Line -notmatch "//.*unwrap" }

$unwrapCount = ($unwrapFiles | Measure-Object).Count
if ($unwrapCount -eq 0) {
    Write-Host "✅ No unwrap in production code" -ForegroundColor Green
} else {
    Write-Host "❌ Found $unwrapCount unwrap() in production code:" -ForegroundColor Red
    $unwrapFiles | ForEach-Object { Write-Host "  $($_.FileName):$($_.LineNumber) $($_.Line.Trim())" }
    exit 1
}

# 检查 panic! 在生产代码中的使用
Write-Host "`nScanning for panic!..." -ForegroundColor Yellow
$panicFiles = Get-ChildItem -Path "src" -Recurse -Filter "*.rs" | 
    Select-String -Pattern "panic!" | 
    Where-Object { $_.FileName -notmatch "test" }

$panicCount = ($panicFiles | Measure-Object).Count
if ($panicCount -eq 0) {
    Write-Host "✅ No panic! in production code" -ForegroundColor Green
} else {
    Write-Host "❌ Found $panicCount panic! in production code:" -ForegroundColor Red
    $panicFiles | ForEach-Object { Write-Host "  $($_.FileName):$($_.LineNumber)" }
    exit 1
}

# 检查 expect 在生产代码中的使用（允许但需记录）
Write-Host "`nScanning for expect (informational)..." -ForegroundColor Yellow
$expectFiles = Get-ChildItem -Path "src" -Recurse -Filter "*.rs" | 
    Select-String -Pattern "expect\(" | 
    Where-Object { $_.FileName -notmatch "test" }

$expectCount = ($expectFiles | Measure-Object).Count
if ($expectCount -eq 0) {
    Write-Host "ℹ️ No expect in production code" -ForegroundColor Gray
} else {
    Write-Host "ℹ️ Found $expectCount expect() in production code (review required):" -ForegroundColor Yellow
    $expectFiles | ForEach-Object { Write-Host "  $($_.FileName):$($_.LineNumber) $($_.Line.Trim())" }
}

# 运行 clippy
Write-Host "`nRunning clippy..." -ForegroundColor Yellow
$clippyOutput = cargo clippy -- -D warnings 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Clippy found issues:" -ForegroundColor Red
    Write-Host $clippyOutput
    exit 1
} else {
    Write-Host "✅ Clippy clean" -ForegroundColor Green
}

# 验证债务状态文档
Write-Host "`nVerifying debt documentation..." -ForegroundColor Yellow
$libContent = Get-Content "src/lib.rs" -Raw
$clearedCount = ([regex]::Matches($libContent, "\[CLEARED\]")).Count
$hasNextAudit = $libContent -match "NEXT-AUDIT"
$hasLastCleared = $libContent -match "LAST-CLEARED"

if ($clearedCount -ge 4 -and $hasNextAudit -and $hasLastCleared) {
    Write-Host "✅ Debt documentation up to date ($clearedCount debts cleared)" -ForegroundColor Green
} else {
    Write-Host "❌ Debt documentation incomplete" -ForegroundColor Red
    exit 1
}

Write-Host "`n=== Debt Check Complete ===" -ForegroundColor Cyan
Write-Host "All checks passed! Debt status: SATURATED-CLEARED" -ForegroundColor Green
