#requires -Version 5.1
<#
.SYNOPSIS
    Instala o Ultimate MUGEN localmente (por usuario, sem admin) no Windows.

.DESCRIPTION
    Compila o binario release, copia o executavel e os assets lidos em runtime
    (pacote da comunidade + licencas), cria as pastas mugen/chars e mugen/stages,
    e adiciona atalhos no Menu Iniciar e na Area de Trabalho.

.PARAMETER InstallDir
    Destino da instalacao. Padrao: %LOCALAPPDATA%\Programs\UltimateMUGEN

.PARAMETER Uninstall
    Remove atalhos e a pasta de instalacao.

.EXAMPLE
    powershell -ExecutionPolicy Bypass -File scripts/install-local.ps1

.EXAMPLE
    powershell -ExecutionPolicy Bypass -File scripts/install-local.ps1 -Uninstall
#>
[CmdletBinding()]
param(
    [string]$InstallDir = (Join-Path $env:LOCALAPPDATA 'Programs\UltimateMUGEN'),
    [switch]$Uninstall,
    [switch]$NoDesktopShortcut,
    [switch]$NoBuild
)

$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$exeName = 'ultimate-mugen.exe'
$startMenuLnk = Join-Path $env:APPDATA 'Microsoft\Windows\Start Menu\Programs\Ultimate MUGEN.lnk'
$desktopLnk = Join-Path ([Environment]::GetFolderPath('Desktop')) 'Ultimate MUGEN.lnk'

function Remove-Shortcut([string]$Path) {
    if (Test-Path -LiteralPath $Path) {
        Remove-Item -LiteralPath $Path -Force
        Write-Host "  atalho removido: $Path"
    }
}

if ($Uninstall) {
    Write-Host "== Desinstalando Ultimate MUGEN =="
    Remove-Shortcut $startMenuLnk
    Remove-Shortcut $desktopLnk
    if (Test-Path -LiteralPath $InstallDir) {
        Remove-Item -LiteralPath $InstallDir -Recurse -Force
        Write-Host "  pasta removida: $InstallDir"
    }
    Write-Host "Desinstalacao concluida."
    return
}

Write-Host "== Instalando Ultimate MUGEN =="
Write-Host "  origem : $root"
Write-Host "  destino: $InstallDir"

if (-not $NoBuild) {
    Write-Host "== Compilando release =="
    Push-Location $root
    try {
        & cargo build --release --bin ultimate-mugen
        if ($LASTEXITCODE -ne 0) { throw "cargo build falhou (exit $LASTEXITCODE)" }
    } finally {
        Pop-Location
    }
}

$srcExe = Join-Path $root "target\release\$exeName"
if (-not (Test-Path -LiteralPath $srcExe)) { throw "Binario nao encontrado: $srcExe" }

New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
Copy-Item -LiteralPath $srcExe -Destination (Join-Path $InstallDir $exeName) -Force
Write-Host "  binario copiado"

# Assets lidos do disco em runtime (o pacote Elecbyte e as fontes sao embutidos no exe).
$mugenDst = Join-Path $InstallDir 'assets\mugen'
New-Item -ItemType Directory -Force -Path $mugenDst | Out-Null
foreach ($f in 'LICENSE-ELECBYTE.txt') {
    $src = Join-Path $root "assets\mugen\$f"
    if (Test-Path -LiteralPath $src) { Copy-Item -LiteralPath $src -Destination (Join-Path $mugenDst $f) -Force }
}

# Pastas de conteudo do usuario (personagens e cenarios MUGEN).
New-Item -ItemType Directory -Force -Path (Join-Path $InstallDir 'mugen\chars') | Out-Null
New-Item -ItemType Directory -Force -Path (Join-Path $InstallDir 'mugen\stages') | Out-Null

function New-AppShortcut([string]$Path) {
    $shell = New-Object -ComObject WScript.Shell
    $lnk = $shell.CreateShortcut($Path)
    $lnk.TargetPath = Join-Path $InstallDir $exeName
    $lnk.WorkingDirectory = $InstallDir
    $lnk.IconLocation = Join-Path $InstallDir $exeName
    $lnk.Description = 'Ultimate MUGEN - motor de luta 2D'
    $lnk.Save()
    Write-Host "  atalho criado: $Path"
}

New-AppShortcut $startMenuLnk
if (-not $NoDesktopShortcut) { New-AppShortcut $desktopLnk }

$total = (Get-ChildItem -LiteralPath $InstallDir -Recurse -File | Measure-Object -Property Length -Sum).Sum
Write-Host ("== Instalado ({0:N1} MB) ==" -f ($total / 1MB))
Write-Host "  Execute: $(Join-Path $InstallDir $exeName)"
