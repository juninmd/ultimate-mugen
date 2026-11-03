<#
.SYNOPSIS
  Download e validação de personagens M.U.G.E.N da Shonen Jump (Dragon Ball, Naruto, Bleach).
.DESCRIPTION
  Baixa lutadores verificados e compatíveis com a VM do Ultimate MUGEN,
  descompacta em mugen/chars e executa validação automática (mugen_check).
.PARAMETER VerifyOnly
  Apenas valida os personagens já instalados sem baixar novamente.
#>
param(
  [switch]$VerifyOnly
)

$ErrorActionPreference = "Stop"
$MugenDir = Join-Path $PSScriptRoot "..\mugen"
$CharsDir = Join-Path $MugenDir "chars"
$StagesDir = Join-Path $MugenDir "stages"
$TmpDir = Join-Path $MugenDir ".tmp"

New-Item -ItemType Directory -Path $CharsDir -Force | Out-Null
New-Item -ItemType Directory -Path $StagesDir -Force | Out-Null
New-Item -ItemType Directory -Path $TmpDir -Force | Out-Null

function Run-MugenCheck($target) {
  Write-Host "Verificando compatibilidade com mugen_check: $target" -ForegroundColor Cyan
  cargo run --release -p ume-engine --example mugen_check -- $target
}

if ($VerifyOnly) {
  Run-MugenCheck $CharsDir
  exit 0
}

Write-Host "=== Gerenciador de Personagens Shonen Jump (Ultimate MUGEN) ===" -ForegroundColor Green

function Download-ZipAndExtract($name, $url, $destDir, $expectedFile) {
  if (Test-Path (Join-Path $destDir $expectedFile)) {
    Write-Host "[OK] $name já instalado em $destDir." -ForegroundColor Green
    return
  }
  Write-Host "Baixando $name..." -ForegroundColor Yellow
  $zipPath = Join-Path $TmpDir "$name.zip"
  $extractPath = Join-Path $TmpDir "$name-extract"

  try {
    if (Test-Path $extractPath) { Remove-Item $extractPath -Recurse -Force }
    curl.exe -f -sS -L --retry 3 -o $zipPath $url
    if ($LASTEXITCODE -ne 0) {
      throw "Erro no download via curl para $name (exit code: $LASTEXITCODE)"
    }
    Expand-Archive -Path $zipPath -DestinationPath $extractPath -Force
    $innerDirs = Get-ChildItem -Path $extractPath -Directory
    $srcDir = if ($innerDirs.Count -eq 1) { $innerDirs[0].FullName } else { $extractPath }

    New-Item -ItemType Directory -Path $destDir -Force | Out-Null
    Copy-Item -Path "$srcDir\*" -Destination $destDir -Recurse -Force
    Write-Host "[Sucesso] $name instalado em $destDir." -ForegroundColor Green
  }
  finally {
    if (Test-Path $zipPath) { Remove-Item $zipPath -Force -ErrorAction SilentlyContinue }
    if (Test-Path $extractPath) { Remove-Item $extractPath -Recurse -Force -ErrorAction SilentlyContinue }
  }
}

# 1. Kid Goku (Dragon Ball - BillsLasa)
Download-ZipAndExtract -name "kid_goku" `
  -url "https://github.com/BillsLasa/2D-Kid-Goku-Fighter---M.U.G.E.N./archive/refs/heads/BD.zip" `
  -destDir "$CharsDir\kid_goku" `
  -expectedFile "kid_goku.def"

# 2. Naruto Hokage (Naruto - BillsLasa)
Download-ZipAndExtract -name "naruto_hokage" `
  -url "https://github.com/BillsLasa/2D-Naruto-Hokage-Fighter---M.U.G.E.N./archive/refs/heads/BD.zip" `
  -destDir "$CharsDir\naruto_hokage" `
  -expectedFile "naruto nanadaime.def"

# 3. Bills (Dragon Ball Super - BillsLasa)
Download-ZipAndExtract -name "bills" `
  -url "https://github.com/BillsLasa/2D-Bills-Fighter---M.U.G.E.N./archive/refs/heads/BD.zip" `
  -destDir "$CharsDir\bills" `
  -expectedFile "bills.def"

# Limpeza da pasta temporária
if (Test-Path $TmpDir) {
  Remove-Item $TmpDir -Recurse -Force -ErrorAction SilentlyContinue
}

Write-Host "`nValidando todos os personagens instalados..." -ForegroundColor Green
Run-MugenCheck $CharsDir
