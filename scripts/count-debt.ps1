#!/usr/bin/env pwsh
# HAJIMI 技术债务自动化计数脚本

$paths = Get-ChildItem -Recurse src\ -Include *.rs,*.js,*.ts |
    Select-Object -ExpandProperty FullName

$debtCount = (Select-String -Path $paths -Pattern "DEBT-" | Measure-Object).Count
$todoCount = (Select-String -Path $paths -Pattern "TODO|FIXME" | Measure-Object).Count

$total = $debtCount + $todoCount

Write-Output "=== HAJIMI 技术债务统计 ==="
Write-Output "DEBT-  : $debtCount"
Write-Output "TODO   : $todoCount"
Write-Output "TOTAL  : $total"
Write-Output "==========================="

$claimed = 89
$deviation = [math]::Abs($total - $claimed) / $claimed * 100
Write-Output "声称值 : $claimed"
Write-Output "偏差   : $([math]::Round($deviation, 2))%"

if ($deviation -gt 5) {
    Write-Output "WARNING: 偏差超过5%，请同步文档"
}
