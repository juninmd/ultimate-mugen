#requires -Version 5.1
<#
.SYNOPSIS
    Relatorio dos personagens M.U.G.E.N dublados (PT-BR) instalados.

.DESCRIPTION
    Para cada pasta *PT-BR em <InstallDir>/mugen/chars, mostra o .def, o autor,
    o tamanho do pacote de voz (.snd) e a quantidade de amostras de som, o que
    comprova que o pacote traz um banco de vozes embutido (as falas dubladas).
    Tambem lista creditos/readme que mencionem dublagem.

.PARAMETER InstallDir
    Padrao: %LOCALAPPDATA%\Programs\UltimateMUGEN

.EXAMPLE
    powershell -ExecutionPolicy Bypass -File scripts/verify-ptbr-chars.ps1
#>
[CmdletBinding()]
param([string]$InstallDir = (Join-Path $env:LOCALAPPDATA 'Programs\UltimateMUGEN'))

$ErrorActionPreference = 'Stop'
$charsDir = Join-Path $InstallDir 'mugen\chars'

function Get-SndInfo([string]$path) {
    $fs = [System.IO.File]::OpenRead($path)
    try {
        $head = New-Object byte[] 20
        [void]$fs.Read($head, 0, 20)
        $sig = [System.Text.Encoding]::ASCII.GetString($head, 0, 12)
        if ($sig -notlike 'ElecbyteSnd*') { return @{ sounds = 0; ok = $false } }
        $count = [BitConverter]::ToUInt32($head, 16)
        return @{ sounds = $count; ok = $true }
    }
    finally { $fs.Dispose() }
}

$rows = @()
foreach ($dir in (Get-ChildItem -LiteralPath $charsDir -Directory | Where-Object Name -like '*PT-BR*' | Sort-Object Name)) {
    $defs = Get-ChildItem -LiteralPath $dir.FullName -Recurse -Filter *.def -File | Where-Object {
        $t = (Get-Content -LiteralPath $_.FullName -Raw).ToLowerInvariant()
        $t.Contains('[files]') -and $t -match '(?m)^\s*cns\s*=' -and $t -match '(?m)^\s*sprite\s*='
    }
    foreach ($def in $defs) {
        $body = Get-Content -LiteralPath $def.FullName -Raw
        $name = ([regex]::Match($body, '(?im)^\s*name\s*=\s*"?(.+?)"?\s*$') | ForEach-Object { $_.Groups[1].Value })
        $author = ([regex]::Match($body, '(?im)^\s*author\s*=\s*"?(.+?)"?\s*$') | ForEach-Object { $_.Groups[1].Value })
        $snd = Get-ChildItem -LiteralPath $dir.FullName -Recurse -Filter *.snd -File | Sort-Object Length -Descending | Select-Object -First 1
        $info = if ($snd) { Get-SndInfo $snd.FullName } else { @{ sounds = 0; ok = $false } }
        $credit = Get-ChildItem -LiteralPath $dir.FullName -Recurse -Include *.txt,*.rtf -File -ErrorAction SilentlyContinue |
            Where-Object { (Get-Content -LiteralPath $_.FullName -Raw -ErrorAction SilentlyContinue) -match '(?i)dubl' } |
            Select-Object -First 2 -ExpandProperty Name
        $rows += [pscustomobject]@{
            Pasta      = $dir.Name
            Personagem = $name
            Autor      = $author
            Def        = $def.Name
            VozMB      = if ($snd) { [math]::Round($snd.Length / 1MB, 1) } else { 0 }
            Sons       = $info.sounds
            CreditoDub = ($credit -join ', ')
        }
    }
}

$rows | Format-Table -AutoSize
Write-Host ("`n{0} pastas PT-BR, {1} .def de personagem, {2:N1} MB de dados de voz" -f `
    ($rows | Select-Object -ExpandProperty Pasta -Unique).Count, $rows.Count, (($rows | Measure-Object VozMB -Sum).Sum))
