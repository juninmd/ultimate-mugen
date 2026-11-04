#requires -Version 5.1
<#
.SYNOPSIS
    Marca os personagens dublados com o sufixo " (PT-BR)" em name/displayname.

.DESCRIPTION
    Os chars dublados convivem com as versoes originais (ex.: dois Raditz, dois
    Luffy). Este script altera apenas as linhas name/displayname dentro da secao
    [Info] do .def de cada pasta *PT-BR, para o seletor deixar claro qual e o
    dublado. Idempotente: nao duplica o sufixo. Preserva a codificacao do arquivo.

.EXAMPLE
    powershell -ExecutionPolicy Bypass -File scripts/tag-ptbr-names.ps1
#>
[CmdletBinding()]
param([string]$InstallDir = (Join-Path $env:LOCALAPPDATA 'Programs\UltimateMUGEN'))

$ErrorActionPreference = 'Stop'
$charsDir = Join-Path $InstallDir 'mugen\chars'
$utf8 = New-Object System.Text.UTF8Encoding($false)
$changed = 0

foreach ($dir in (Get-ChildItem -LiteralPath $charsDir -Directory | Where-Object Name -like '*PT-BR*')) {
    $defs = Get-ChildItem -LiteralPath $dir.FullName -Filter *.def -File
    foreach ($def in $defs) {
        $text = [System.IO.File]::ReadAllText($def.FullName)
        if ($text -notmatch '(?i)\[files\]') { continue }
        $lines = $text -split "`r?`n"
        $inInfo = $false
        $touched = $false
        for ($i = 0; $i -lt $lines.Count; $i++) {
            $line = $lines[$i]
            if ($line -match '^\s*\[') { $inInfo = ($line -match '(?i)^\s*\[\s*info\s*\]'); continue }
            if (-not $inInfo) { continue }
            if ($line -match '(?i)^(\s*(?:name|displayname)\s*=\s*)"?([^";]*?)"?\s*(;.*)?$') {
                $key = $Matches[1]
                $val = $Matches[2].Trim()
                $comment = if ($Matches[3]) { ' ' + $Matches[3].Trim() } else { '' }
                if ($val -and $val -notmatch '(?i)\(PT-BR\)') {
                    $lines[$i] = "$key`"$val (PT-BR)`"$comment"
                    $touched = $true
                }
            }
        }
        if ($touched) {
            [System.IO.File]::WriteAllText($def.FullName, ($lines -join "`r`n"), $utf8)
            Write-Host "[tag] $($dir.Name)/$($def.Name)"
            $changed++
        }
    }
}
Write-Host "`n$changed arquivo(s) .def marcados com (PT-BR)."
