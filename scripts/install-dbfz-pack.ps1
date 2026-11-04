#requires -Version 5.1
<#
.SYNOPSIS
    Instala os personagens e cenarios do pacote local "Dragon Ball FighterZ Mugen Jus Edition".

.DESCRIPTION
    Extrai o .rar, copia as 40 pastas de personagens para <InstallDir>/mugen/chars
    com o sufixo " DBFZ" (evita colidir com chars ja instalados, ex.: "android16")
    e os cenarios [DBFZ]*.def/.sff para <InstallDir>/mugen/stages.

.PARAMETER Archive
    Caminho do .rar. Padrao: %USERPROFILE%\Downloads\Dragon Ball FighterZ Mugen Jus Edition.rar

.PARAMETER InstallDir
    Padrao: %LOCALAPPDATA%\Programs\UltimateMUGEN

.PARAMETER Tag
    Sufixo das pastas de personagem. Padrao: DBFZ

.EXAMPLE
    powershell -ExecutionPolicy Bypass -File scripts/install-dbfz-pack.ps1
#>
[CmdletBinding()]
param(
    [string]$Archive = (Join-Path $env:USERPROFILE 'Downloads\Dragon Ball FighterZ Mugen Jus Edition.rar'),
    [string]$InstallDir = (Join-Path $env:LOCALAPPDATA 'Programs\UltimateMUGEN'),
    [string]$Tag = 'DBFZ',
    [string]$SourceDir
)

$ErrorActionPreference = 'Stop'
if (-not (Test-Path -LiteralPath $Archive)) { throw "arquivo nao encontrado: $Archive" }
$sevenZip = @('C:\Program Files\7-Zip\7z.exe', 'C:\Program Files (x86)\7-Zip\7z.exe') | Where-Object { Test-Path -LiteralPath $_ } | Select-Object -First 1
if (-not $sevenZip) { throw '7-Zip nao encontrado.' }

$work = Join-Path $env:TEMP 'ume-dbfz'
$charsDir = Join-Path $InstallDir 'mugen\chars'
$stagesDir = Join-Path $InstallDir 'mugen\stages'

Write-Host "== Extraindo $Archive =="
if ($SourceDir -and (Test-Path -LiteralPath $SourceDir)) {
    $root = Get-Item -LiteralPath $SourceDir
    Write-Host "  usando pasta ja extraida: $($root.FullName)"
} else {
    if (Test-Path -LiteralPath $work) { Remove-Item -LiteralPath $work -Recurse -Force }
    New-Item -ItemType Directory -Force -Path $work | Out-Null
    & $sevenZip x -y -o"$work" $Archive | Out-Null
    if ($LASTEXITCODE -ne 0) { throw "7z falhou ($LASTEXITCODE)" }
    $root = Get-ChildItem -LiteralPath $work -Directory | Select-Object -First 1
}
New-Item -ItemType Directory -Force -Path $charsDir, $stagesDir | Out-Null

$srcChars = Join-Path $root.FullName 'chars'
$srcStages = Join-Path $root.FullName 'stages'

$n = 0
foreach ($dir in (Get-ChildItem -LiteralPath $srcChars -Directory)) {
    $dst = Join-Path $charsDir ("{0} {1}" -f $dir.Name, $Tag)
    if (Test-Path -LiteralPath $dst) { Write-Host "[skip] $($dir.Name)"; continue }
    Copy-Item -LiteralPath $dir.FullName -Destination $dst -Recurse -Force
    $n++
}
Write-Host "[chars] $n pastas instaladas com sufixo '$Tag'" -ForegroundColor Green

$s = 0
foreach ($f in (Get-ChildItem -LiteralPath $srcStages -File)) {
    Copy-Item -LiteralPath $f.FullName -Destination (Join-Path $stagesDir $f.Name) -Force
    if ($f.Extension -eq '.def') { $s++ }
}
Write-Host "[stages] $s cenarios copiados para $stagesDir" -ForegroundColor Green

Remove-Item -LiteralPath $work -Recurse -Force -ErrorAction SilentlyContinue

Write-Host "`n== Resumo ==" -ForegroundColor Green
Get-ChildItem -LiteralPath $charsDir -Directory | Where-Object Name -like "* $Tag" |
    ForEach-Object { "{0,-30} {1,7:N1} MB" -f $_.Name, ((Get-ChildItem -LiteralPath $_.FullName -Recurse -File | Measure-Object Length -Sum).Sum / 1MB) }
Get-ChildItem -LiteralPath $stagesDir -File -Filter '*.def' | Where-Object Name -like "[[]DBFZ[]]*" | ForEach-Object { "stage: $($_.Name)" }
