#!/usr/bin/env pwsh
# HAJIMI 代码行数自动化统计脚本

$rsFiles = Get-ChildItem -Recurse src\ -Filter *.rs | Where-Object { $_.FullName -notmatch "\\target\\|\\node_modules\\|\\dist\\" }
$jsFiles = Get-ChildItem -Recurse src\ -Filter *.js | Where-Object { $_.FullName -notmatch "\\target\\|\\node_modules\\|\\dist\\" }
$tsFiles = Get-ChildItem -Recurse src\ -Filter *.ts | Where-Object { $_.FullName -notmatch "\\target\\|\\node_modules\\|\\dist\\" }
$htmlFiles = Get-ChildItem -Recurse src\ -Filter *.html | Where-Object { $_.FullName -notmatch "\\target\\|\\node_modules\\|\\dist\\" }
$cssFiles = Get-ChildItem -Recurse src\ -Filter *.css | Where-Object { $_.FullName -notmatch "\\target\\|\\node_modules\\|\\dist\\" }

$rustLines = ($rsFiles | ForEach-Object { Get-Content $_.FullName | Measure-Object -Line } | Measure-Object -Property Lines -Sum).Sum
$jsLines = ($jsFiles | ForEach-Object { Get-Content $_.FullName | Measure-Object -Line } | Measure-Object -Property Lines -Sum).Sum
$tsLines = ($tsFiles | ForEach-Object { Get-Content $_.FullName | Measure-Object -Line } | Measure-Object -Property Lines -Sum).Sum
$htmlLines = ($htmlFiles | ForEach-Object { Get-Content $_.FullName | Measure-Object -Line } | Measure-Object -Property Lines -Sum).Sum
$cssLines = ($cssFiles | ForEach-Object { Get-Content $_.FullName | Measure-Object -Line } | Measure-Object -Property Lines -Sum).Sum

$total = $rustLines + $jsLines + $tsLines + $htmlLines + $cssLines

Write-Output "=== HAJIMI 代码行数统计 ==="
Write-Output "Rust : $rustLines"
Write-Output "JS   : $jsLines"
Write-Output "TS   : $tsLines"
Write-Output "HTML : $htmlLines"
Write-Output "CSS  : $cssLines"
Write-Output "TOTAL: $total"
Write-Output "==========================="
