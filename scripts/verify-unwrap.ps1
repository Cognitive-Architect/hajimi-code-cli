#!/usr/bin/env pwsh
# 生产代码unwrap计数（排除test/benches目录）
$prodFiles = Get-ChildItem src -Recurse -Filter "*.rs" | Where-Object { $_.FullName -notmatch 'test|bench' }
$prodUnwrap = 0
foreach ($file in $prodFiles) {
    $content = Get-Content $file.FullName -Raw
    # 只匹配unwrap()，不匹配unwrap_or等
    $count = ([regex]::Matches($content, '(?<!//.*)unwrap\(\)')).Count
    $prodUnwrap += $count
}
Write-Host "生产代码unwrap: $prodUnwrap (目标≤5)"
if ($prodUnwrap -le 5) { Write-Host "✅ 达标" } else { Write-Host "❌ 超标" }
