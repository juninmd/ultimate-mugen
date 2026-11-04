#requires -Version 5.1
<#
.SYNOPSIS
    Instala personagens extraidos dos fullgames M.U.G.E.N mais famosos.

.DESCRIPTION
    Baixa os jogos completos (Archive.org), extrai e copia 30 lutadores:
      * Mortal Kombat Project 4.9.3 ......... 10 (Scorpion, Sub-Zero, Liu Kang, Raiden, ...)
      * Super Marvel vs. Capcom: EoH ......... 10 (Ryu, Ken, Hulk, Homem-Aranha, ...)
      * The King of Fighters Zillion SE 2022 . 10 (Kyo, Iori, Terry, Mai, ...)

    Os chars do KOF Zillion dependem de dois arquivos compartilhados do fullgame
    (`data/command.cmd` e `data/common1.cns`); o script copia ambos para dentro de
    cada pasta e reescreve `cmd =` no .def, deixando o char autocontido.

.PARAMETER InstallDir
    Padrao: %LOCALAPPDATA%\Programs\UltimateMUGEN

.PARAMETER Game
    Instala apenas um pacote: mkp, mvc ou kof.

.PARAMETER KeepArchives
    Mantem os .zip/.rar baixados em %TEMP%\ume-famous (para reexecutar sem baixar).

.EXAMPLE
    powershell -ExecutionPolicy Bypass -File scripts/fetch-famous-chars.ps1

.EXAMPLE
    powershell -ExecutionPolicy Bypass -File scripts/fetch-famous-chars.ps1 -Game kof
#>
[CmdletBinding()]
param(
    [string]$InstallDir = (Join-Path $env:LOCALAPPDATA 'Programs\UltimateMUGEN'),
    [ValidateSet('all', 'mkp', 'mvc', 'kof')][string]$Game = 'all',
    [switch]$KeepArchives
)

$ErrorActionPreference = 'Stop'
$work = Join-Path $env:TEMP 'ume-famous'
$charsDir = Join-Path $InstallDir 'mugen\chars'
$sevenZip = @('C:\Program Files\7-Zip\7z.exe', 'C:\Program Files (x86)\7-Zip\7z.exe') | Where-Object { Test-Path -LiteralPath $_ } | Select-Object -First 1
if (-not $sevenZip) { throw '7-Zip nao encontrado.' }

# pasta de destino  <-  pasta dentro do fullgame
$packs = @(
    @{
        id = 'mkp'; file = 'mkp.zip'; dir = 'mkp'
        url = 'https://archive.org/download/mortal-kombat-project-4.9.3/mortal%20kombat%20project%204.9.3.zip'
        root = 'Mortal Kombat Project 4.9.3'
        picks = @(
            @{ name = 'Scorpion UMKIII'; dst = 'Scorpion MKP' },
            @{ name = 'Sub-Zero umk3'; dst = 'Sub-Zero MKP' },
            @{ name = 'liu_kang'; dst = 'Liu Kang MKP' },
            @{ name = 'Raiden MK3'; dst = 'Raiden MKP' },
            @{ name = 'sonya'; dst = 'Sonya Blade MKP' },
            @{ name = 'Jax Briggs'; dst = 'Jax Briggs MKP' },
            @{ name = 'Johnny Cage'; dst = 'Johnny Cage MKP' },
            @{ name = 'Mileena MKII'; dst = 'Mileena MKP' },
            @{ name = 'Shang Tsung MKII'; dst = 'Shang Tsung MKP' },
            @{ name = 'Shao Kahn'; dst = 'Shao Kahn MKP' }
        )
    },
    @{
        id = 'mvc'; file = 'smvceoh.rar'; dir = 'mvc'
        url = 'https://archive.org/download/SMVCEOH/SMVCEOH.rar'
        root = ''
        picks = @(
            @{ name = 'Ryu_MvC'; dst = 'Ryu MvC' },
            @{ name = 'Ken_MvC'; dst = 'Ken MvC' },
            @{ name = 'IncredibleHulk'; dst = 'Hulk MvC' },
            @{ name = 'AmazingSpiderman'; dst = 'Homem-Aranha MvC' },
            @{ name = 'CaptainAmerica_MvC'; dst = 'Capitao America MvC' },
            @{ name = 'MVC2_IronMan'; dst = 'Homem de Ferro MvC' },
            @{ name = 'xMvCWolverinex'; dst = 'Wolverine MvC' },
            @{ name = 'MVC2_Magneto'; dst = 'Magneto MvC' },
            @{ name = 'MorriganE'; dst = 'Morrigan MvC' },
            @{ name = 'Deadpool'; dst = 'Deadpool MvC' }
        )
    },
    @{
        id = 'kof'; file = 'kofzillion.rar'; dir = 'kof'
        url = 'https://archive.org/download/the-king-of-fighters-zillion-super-edition-mugen-2022/THE%20KING%20OF%20FIGHTERS%20ZILLION%20SUPER%20EDITION%20MUGEN%202022.rar'
        root = 'THE KING OF FIGHTERS ZILLION SUPER EDITION MUGEN 2022'
        picks = @(
            @{ name = 'kyo'; dst = 'Kyo KOF' },
            @{ name = 'iori'; dst = 'Iori KOF' },
            @{ name = 'terry'; dst = 'Terry KOF' },
            @{ name = 'mai'; dst = 'Mai KOF' },
            @{ name = 'Kula'; dst = 'Kula KOF' },
            @{ name = 'rugal'; dst = 'Rugal KOF' },
            @{ name = 'Ryo'; dst = 'Ryo KOF' },
            @{ name = 'robert'; dst = 'Robert KOF' },
            @{ name = 'leona'; dst = 'Leona KOF' },
            @{ name = 'athena'; dst = 'Athena KOF' }
        )
    }
)

New-Item -ItemType Directory -Force -Path $charsDir, $work | Out-Null

foreach ($p in $packs) {
    if ($Game -ne 'all' -and $Game -ne $p.id) { continue }
    $archive = Join-Path $work $p.file
    $extract = Join-Path $work $p.dir

    if (-not (Test-Path -LiteralPath $archive)) {
        Write-Host "[down] $($p.file) (Archive.org)" -ForegroundColor Yellow
        & curl.exe -f -L -C - --retry 3 -o $archive $p.url
        if ($LASTEXITCODE -ne 0) { throw "download falhou: $($p.url)" }
    }
    if (-not (Test-Path -LiteralPath $extract)) {
        Write-Host "[unzip] $($p.file)"
        & $sevenZip x -y -o"$extract" $archive | Out-Null
        if ($LASTEXITCODE -ne 0) { throw "7z falhou em $($p.file)" }
    }

    $gameRoot = if ($p.root) { Join-Path $extract $p.root } else { $extract }
    $gameChars = Join-Path $gameRoot 'chars'
    if (-not (Test-Path -LiteralPath $gameChars)) { throw "pasta chars nao encontrada em $gameRoot" }

    foreach ($pick in $p.picks) {
        $src = Join-Path $gameChars $pick.name
        $dst = Join-Path $charsDir $pick.dst
        if (-not (Test-Path -LiteralPath $src)) { Write-Host "[miss] $($pick.name)" -ForegroundColor Red; continue }
        if (Test-Path -LiteralPath $dst) { Write-Host "[skip] $($pick.dst)"; continue }
        Copy-Item -LiteralPath $src -Destination $dst -Recurse -Force
        Write-Host "[ok]   $($pick.dst)" -ForegroundColor Green
    }

    # KOF Zillion: deixa os chars autocontidos (cmd + estados comuns)
    if ($p.id -eq 'kof') {
        $dataDir = Join-Path $gameRoot 'data'
        $cmdSrc = Join-Path $dataDir 'command.cmd'
        $cnsSrc = Join-Path $dataDir 'common1.cns'
        foreach ($pick in $p.picks) {
            $dst = Join-Path $charsDir $pick.dst
            if (-not (Test-Path -LiteralPath $dst)) { continue }
            Copy-Item -LiteralPath $cmdSrc -Destination (Join-Path $dst 'command.cmd') -Force
            Copy-Item -LiteralPath $cnsSrc -Destination (Join-Path $dst 'common1.cns') -Force
            $def = Get-ChildItem -LiteralPath $dst -Filter *.def -File | Select-Object -First 1
            if ($def) {
                $t = [IO.File]::ReadAllText($def.FullName)
                if ($t -match '(?im)^\s*cmd\s*=\s*[^\r\n]*command\.cmd') {
                    [IO.File]::WriteAllText($def.FullName, ($t -replace '(?im)^(\s*cmd\s*=\s*)[^\r\n]*', '${1}command.cmd'))
                }
            }
        }
    }

    if (-not $KeepArchives) {
        Remove-Item -LiteralPath $archive -Force -ErrorAction SilentlyContinue
        Remove-Item -LiteralPath $extract -Recurse -Force -ErrorAction SilentlyContinue
    }
}

Write-Host "`n== Instalados ==" -ForegroundColor Green
Get-ChildItem -LiteralPath $charsDir -Directory | Where-Object { $_.Name -match '(MKP|KOF|MvC)$|^Capitao America MvC$' } |
    ForEach-Object { "{0,-26} {1,7:N1} MB" -f $_.Name, ((Get-ChildItem -LiteralPath $_.FullName -Recurse -File | Measure-Object Length -Sum).Sum / 1MB) }
