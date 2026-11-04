#requires -Version 5.1
<#
.SYNOPSIS
    Baixa, extrai e valida personagens M.U.G.E.N dublados em PT-BR.

.DESCRIPTION
    Curadoria de chars com dublagem brasileira (vozes do anime / redublagem
    da comunidade "Mugen Br", "MFmugen PT BR" e "Golden Mugen Brasil").
    Cada pacote vem de links publicos do MediaFire divulgados pelos autores.
    O script resolve o link direto, extrai em pasta temporaria, localiza a
    pasta do personagem (o .def com [Files] cns= e sprite=) e instala em
    <InstallDir>/mugen/chars/<nome>.

.PARAMETER InstallDir
    Destino. Padrao: %LOCALAPPDATA%\Programs\UltimateMUGEN

.PARAMETER VerifyOnly
    Apenas roda o mugen_check nos chars ja instalados.

.PARAMETER Only
    Instala somente os chars cujo id contenha um destes textos.

.EXAMPLE
    powershell -ExecutionPolicy Bypass -File scripts/fetch-ptbr-chars.ps1

.EXAMPLE
    powershell -ExecutionPolicy Bypass -File scripts/fetch-ptbr-chars.ps1 -Only goku,luffy
#>
[CmdletBinding()]
param(
    [string]$InstallDir = (Join-Path $env:LOCALAPPDATA 'Programs\UltimateMUGEN'),
    [switch]$VerifyOnly,
    [switch]$SkipVerify,
    [string[]]$Only
)

$ErrorActionPreference = 'Stop'
$Only = @($Only | ForEach-Object { $_ -split ',' } | ForEach-Object { $_.Trim() } | Where-Object { $_ })
$root = Split-Path -Parent $PSScriptRoot
$charsDir = Join-Path $InstallDir 'mugen\chars'
$work = Join-Path $env:TEMP 'ume-ptbr'
$url = 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0 Safari/537.36'

$sevenZip = @(
    'C:\Program Files\7-Zip\7z.exe',
    'C:\Program Files (x86)\7-Zip\7z.exe'
) | Where-Object { Test-Path -LiteralPath $_ } | Select-Object -First 1
if (-not $sevenZip) { throw '7-Zip nao encontrado (necessario para .rar/.zip).' }

# id, pasta de destino, fonte (autor da dublagem), link MediaFire, video de origem (prova da dublagem)
$catalog = @(
    @{ id = 'raditz_dublado';          dir = 'Raditz PT-BR';            src = 'Mugen Br';               yt = '5I6OAO7_WmM'; file = 'https://www.mediafire.com/file/w4jzsop433e6jeb/Raditz.rar/file' }
    @{ id = 'goku_ui_dublado';         dir = 'Goku UI PT-BR';           src = 'Mugen Br';               yt = 'uFwChiACdIM'; file = 'https://www.mediafire.com/file/0moo59n98vveo8e/Goku_UI2.rar/file' }
    @{ id = 'goku_mui_dublado';        dir = 'Goku MUI PT-BR';          src = 'Mugen Br';               yt = 'yOt3kn-u1YU'; file = 'https://www.mediafire.com/file/uzupfj8an3ec2ic/Goku_Mastered.zip/file' }
    @{ id = 'itadori_dublado';         dir = 'Itadori PT-BR';           src = 'MFmugen PT BR';          yt = 'De2dQd9S7CA'; file = 'https://www.mediafire.com/file/jziwy02dn9rj902/Itadori+Dublado.rar/file' }
    @{ id = 'luffy_start_dublado';     dir = 'Luffy PT-BR';             src = 'Mugen Br';               yt = 'hGthbs5f-sE'; file = 'https://www.mediafire.com/file/3c2t5akk43m8w7p/Luffy_Start.rar/file' }
    @{ id = 'luffy_gear2_dublado';     dir = 'Luffy Gear 2 PT-BR';      src = 'MFmugen PT BR';          yt = 'e3o4jgCkiFc'; file = 'https://www.mediafire.com/file/6fy8j9j12wawvdb/Luffy_Mid_Gear_2+dublado.rar/file' }
    @{ id = 'ichigo_dublado';          dir = 'Ichigo PT-BR';            src = 'Golden Mugen Brasil';    yt = 'k8xq8R99bo0'; file = 'https://www.mediafire.com/file/k2nspuaxzy57ciw/Ichigo_dublado.rar/file' }
    @{ id = 'naofumi_dublado';         dir = 'Naofumi PT-BR';           src = 'Mugen Br';               yt = 'jKInYwnsyvQ'; file = 'https://www.mediafire.com/file/388w36nv90cg40z/Naofumi+Br.rar/file' }
    @{ id = 'panda_dublado';           dir = 'Panda JJK PT-BR';         src = 'Mugen Br';               yt = '9Q8CsaXcXvU'; file = 'https://www.mediafire.com/file/nfth1bh62znzepq/Panda+Senpai.rar/file' }
    @{ id = 'inumaki_dublado';         dir = 'Inumaki JJK PT-BR';       src = 'Mugen Br';               yt = 'kXQrjEqbJ6E'; file = 'https://www.mediafire.com/file/ig49wp5ill0w49o/inumaki+toge.rar/file' }
    @{ id = 'maki_dublado';            dir = 'Maki JJK PT-BR';          src = 'Mugen Br';               yt = 'b-2j46FYFb8'; file = 'https://www.mediafire.com/file/we3lo6gktjgiqoh/Maki.rar/file' }
    @{ id = 'yuta_dublado';            dir = 'Yuta JJK PT-BR';          src = 'Mugen Br';               yt = 'Qq3ZQFlwrE4'; file = 'https://www.mediafire.com/file/52iaq2t0m4puawo/Yuta+(JT).rar/file' }
    @{ id = 'gojo_dublado';            dir = 'Gojo PT-BR';              src = 'Mugen Br';               yt = 'GBQeSlnSZ6Y'; file = 'https://www.mediafire.com/file/wk5eyz8kqk9wlu2/Gojo_de_muleta.rar/file' }
    @{ id = 'gambit_dublado';          dir = 'Gambit PT-BR';            src = 'Mugen Br';               yt = 'uDvQUNzw45A'; file = 'https://www.mediafire.com/file/o0u1doob174ywmz/Gambit.rar/file' }
    @{ id = 'noturno_dublado';         dir = 'Noturno PT-BR';           src = 'Mugen Br';               yt = 'tyxcqQWnvvM'; file = 'https://www.mediafire.com/file/z4byc5qfyv0vviq/Nightcrawler.rar/file' }
    @{ id = 'jubileu_dublado';         dir = 'Jubileu PT-BR';           src = 'Mugen Br';               yt = 'f3hbSNzuxuU'; file = 'https://www.mediafire.com/file/wbjb9pveohhr4nf/jubilee.rar/file' }
    @{ id = 'mulher_maravilha_dublado'; dir = 'Mulher Maravilha PT-BR'; src = 'Mugen Br';               yt = 'laIsRuA6UtE'; file = 'https://www.mediafire.com/file/k9ki7vy7v3g341l/Wonder_Woman.rar/file' }
    @{ id = 'ravena_dublado';          dir = 'Ravena PT-BR';            src = 'Mugen Br';               yt = 'drydGeD_u8w'; file = 'https://www.mediafire.com/file/ba60cjwsngrpnm5/Raven.rar/file' }
    @{ id = 'mistica_dublado';         dir = 'Mistica PT-BR';           src = 'Mugen Br';               yt = 'YGC6fLj1gpg'; file = 'https://www.mediafire.com/file/sj6im6v1kvblg9t/Mystique.rar/file' }
    @{ id = 'hange_dublado';           dir = 'Hange AOT PT-BR';         src = 'Mugen Br';               yt = 'w1f6-l3Nero'; file = 'https://www.mediafire.com/file/43yarregrcqbiq9/Hange.rar/file' }
    @{ id = 'annie_dublado';           dir = 'Annie AOT PT-BR';         src = 'Mugen Br';               yt = 'ywXxRUNFplg'; file = 'https://www.mediafire.com/file/0u9m5uisjuwkgr3/Annie.rar/file' }
    @{ id = 'tanjiro_dublado';         dir = 'Tanjiro PT-BR';           src = 'MFmugen PT BR';          yt = 'MAvAYsLyZzE'; file = 'https://www.mediafire.com/file/ddliyoto180erce/Tanjiro_dublado.rar/file' }
    @{ id = 'nanami_dublado';          dir = 'Nanami PT-BR';            src = 'MFmugen PT BR';          yt = 'w7FJ0LdXQr4'; file = 'https://www.mediafire.com/file/u4btfxoewhc3ps9/Nanami+Kento+dublado.rar/file' }
    @{ id = 'mikasa_dublado';          dir = 'Mikasa AOT PT-BR';        src = 'Mugen Br';               yt = '5tx_pkwsiVs'; file = 'https://www.mediafire.com/file/w205aeb47c41nqz/Mikasa.rar/file' }
    @{ id = 'all_might_dublado';       dir = 'All Might PT-BR';         src = 'Mugen Br';               yt = 'i3cYX9fwoMU'; file = 'https://www.mediafire.com/file/gmn2q33q2ua5dv2/All+Might.rar/file' }
    @{ id = 'saitama_dublado';         dir = 'Saitama PT-BR';           src = 'Mugen Br';               yt = 'vBfTdtQ52_o'; file = 'https://www.mediafire.com/file/atjajid860ybim7/Saitama.rar/file' }
    @{ id = 'cell_ex_dublado';         dir = 'Cell EX PT-BR';           src = 'comunidade BR';          yt = '36EezohS7O0'; file = 'http://www.mediafire.com/file/dst0imivdpm823h/Cell_EX.rar' }
    @{ id = 'vegeta_ssgss_dublado';    dir = 'Vegeta SSGSS PT-BR';      src = 'comunidade BR';          yt = 'DcNyjVykhX0'; file = 'http://www.mediafire.com/download/l1i772l816mnmg2/SSGSS_Vegeta.rar' }
)

function Resolve-MediaFire([string]$link) {
    $page = & curl.exe -sS -A $url -L $link
    if ($page -match 'Invalid or Deleted File') { throw 'arquivo removido do MediaFire' }
    $m = [regex]::Match(($page -join "`n"), 'https?://download[^"'']+mediafire\.com[^"'']+')
    if (-not $m.Success) { throw 'link direto nao encontrado na pagina do MediaFire' }
    return $m.Value
}

function Find-CharDef([string]$dir) {
    Get-ChildItem -LiteralPath $dir -Recurse -Filter *.def -File | Where-Object {
        $t = (Get-Content -LiteralPath $_.FullName -Raw).ToLowerInvariant()
        $t.Contains('[files]') -and $t -match '(?m)^\s*cns\s*=' -and $t -match '(?m)^\s*sprite\s*='
    } | Select-Object -First 1
}

New-Item -ItemType Directory -Force -Path $charsDir | Out-Null

if ($VerifyOnly) {
    Push-Location $root
    try { & cargo run --release -p ume-engine --example mugen_check -- $charsDir } finally { Pop-Location }
    return
}

New-Item -ItemType Directory -Force -Path $work | Out-Null
Write-Host "== Chars dublados PT-BR -> $charsDir =="

$report = @()
foreach ($c in $catalog) {
    if ($Only -and -not ($Only | Where-Object { $c.id -like "*$_*" })) { continue }
    $dest = Join-Path $charsDir $c.dir
    if (Test-Path -LiteralPath $dest) { Write-Host "[skip] $($c.dir) ja instalado"; $report += [pscustomobject]@{Char=$c.dir;Fonte=$c.src;Video="https://youtu.be/$($c.yt)";Status='ja instalado'}; continue }

    $archive = Join-Path $work ($c.id + '.bin')
    $extract = Join-Path $work $c.id
    try {
        Write-Host "[down] $($c.dir) <- $($c.src)" -ForegroundColor Yellow
        $direct = Resolve-MediaFire $c.file
        & curl.exe -f -sS -A $url -L --retry 2 -o $archive $direct
        if ($LASTEXITCODE -ne 0) { throw "curl exit $LASTEXITCODE" }
        if (Test-Path -LiteralPath $extract) { Remove-Item -LiteralPath $extract -Recurse -Force }
        & $sevenZip x -y -o"$extract" $archive | Out-Null
        if ($LASTEXITCODE -ne 0) { throw "7z exit $LASTEXITCODE" }

        $def = Find-CharDef $extract
        if (-not $def) { throw 'nenhum .def de personagem no pacote' }

        # pasta do personagem = pasta que contem o .def; sobe enquanto houver um unico filho
        $charRoot = $def.Directory
        $top = Get-ChildItem -LiteralPath $extract | Select-Object -First 2
        if ($top.Count -eq 1 -and $top[0].PSIsContainer) { $charRoot = $top[0] }

        New-Item -ItemType Directory -Force -Path $dest | Out-Null
        Copy-Item -Path (Join-Path $charRoot.FullName '*') -Destination $dest -Recurse -Force
        $mb = [math]::Round(((Get-ChildItem -LiteralPath $dest -Recurse -File | Measure-Object Length -Sum).Sum / 1MB), 1)
        Write-Host "[ok]   $($c.dir) ($mb MB) def=$($def.Name)" -ForegroundColor Green
        $report += [pscustomobject]@{Char=$c.dir;Fonte=$c.src;Video="https://youtu.be/$($c.yt)";Status="instalado ($mb MB)"}
    }
    catch {
        Write-Host "[falha] $($c.dir): $($_.Exception.Message)" -ForegroundColor Red
        $report += [pscustomobject]@{Char=$c.dir;Fonte=$c.src;Video="https://youtu.be/$($c.yt)";Status="falha: $($_.Exception.Message)"}
    }
    finally {
        if (Test-Path -LiteralPath $archive) { Remove-Item -LiteralPath $archive -Force -ErrorAction SilentlyContinue }
        if (Test-Path -LiteralPath $extract) { Remove-Item -LiteralPath $extract -Recurse -Force -ErrorAction SilentlyContinue }
    }
}

$report | Format-Table -AutoSize
$report | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $work 'report.json') -Encoding UTF8

Write-Host "`n== Validando no motor (mugen_check) =="
if ($SkipVerify) { Write-Host "(validacao adiada: -SkipVerify)" -ForegroundColor DarkGray; return }
Push-Location $root
try { & cargo run --release -p ume-engine --example mugen_check -- $charsDir } finally { Pop-Location }
