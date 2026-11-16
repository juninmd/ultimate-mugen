# Matriz de cobertura de formatos

> 🇬🇧 English version: [Format coverage matrix](../Format-Coverage.md).

Status por formato de arquivo, conferido com os decodificadores em `crates/formats` (leitura do código, ainda não testado com uma coleção grande de packs).

- **Implementado**: decodificado e usado pelo motor.
- **Parcial**: decodificado, com um limite conhecido listado nas notas.
- **Ausente**: não decodificado; o arquivo é ignorado ou gera um aviso.

## SFF (sprites)

| Recurso | Status | Notas |
|---|---|---|
| SFF v1.01, sprites PCX RLE | Implementado | `sff.rs`, `pcx.rs` |
| SFF v1, linhas PCX sem compressão | Implementado | `bytes_per_line` respeitado |
| SFF v1, PCX de 8 bits (paleta de 256 cores no fim do arquivo) | Implementado | |
| SFF v1, PCX de 1/4 bits ou 24 bits | Ausente | A profundidade do cabeçalho é ignorada; esses arquivos saem errados. Raro em packs reais |
| SFF v1, paleta compartilhada (flag `same pal`) | Implementado | Herda a paleta do sprite anterior |
| SFF v1, sprites ligados (`length = 0`) | Implementado | |
| SFF v2.00 / 2.01, 8 bits sem compressão | Implementado | |
| SFF v2, 24 e 32 bits sem compressão | Implementado | Convertidos para RGBA |
| SFF v2, RLE8 / RLE5 / LZ5 | Implementado | `sff/codecs.rs` |
| SFF v2, PNG 8 / 24 / 32 | Implementado | PNG quebrado vira sprite vazio e transparente |
| SFF v2, paletas (RGB e RGBA, paletas ligadas) | Implementado | |
| SFF v2, escolha de paleta por sprite | Implementado | |

## SND (sons)

| Recurso | Status | Notas |
|---|---|---|
| Contêiner `ElecbyteSnd`, chaves grupo/índice | Implementado | Vence a primeira chave repetida, como no M.U.G.E.N |
| Cadeia de subarquivos com proteção contra links corrompidos | Implementado | |
| WAV PCM 8 / 16 bits (o caso normal) | Implementado | Decodificado pelo `symphonia` na camada de áudio |
| WAV ADPCM, 24 bits e outros codecs WAV | Parcial | Depende do que o `symphonia` aceita; não verificado por codec |

## AIR (animações)

| Recurso | Status | Notas |
|---|---|---|
| `[Begin Action N]`, quadros, `time` | Implementado | Ações repetidas: vence a primeira |
| Espelhamento `H` / `V` | Implementado | |
| Blending `A`, `A1`, `AS<n>D<n>`, `S` | Implementado | |
| `Clsn1` / `Clsn2` e `Clsn1Default` / `Clsn2Default` | Implementado | |
| `xscale`, `yscale`, `angle` por quadro | Implementado | |
| `Loopstart` | Implementado | |
| `Interpolate` | Ausente | A linha é aceita e ignorada (`air.rs`); sem interpolação entre quadros |

## ACT (paletas)

| Recurso | Status | Notas |
|---|---|---|
| Paleta RGB de 768 bytes | Implementado | Índice 0 transparente, ordem invertida como no MUGEN |
| Arquivos curtos (menos de 256 cores) | Implementado | Entradas faltando ficam pretas |

## DEF e CNS (personagens e stages)

| Recurso | Status | Notas |
|---|---|---|
| `.def` de personagem (`[Info]`, `[Files]`, `localcoord`) | Implementado | |
| `localcoord` 320x240, 640x480 e 1280x720 | Implementado | Posições e distâncias convertidas por jogador |
| `.def` de stage (`[Info]`, `[StageInfo]`, `[Camera]`, `[Player]`, `[Bound]`) | Implementado | |
| Camadas `[BG]`: `normal`, `anim`, `parallax`, `trans`, `mask`, `maskwindow` | Implementado | |
| `[BGCtrlDef]` / `[BGCtrl]`: `null`, `visible`, `enable`, `velset`, `veladd`, `posset`, `posadd`, `anim`, `sinx`, `siny` | Implementado | |
| Stage `zoffset`, `autoturn` | Implementado | Usados pelo motor (`stagevar.rs`, `matchs.rs`) |
| Stage `resetbg` | Ausente | Não é lido em nenhum lugar do workspace |

## Screenpacks

| Recurso | Status | Notas |
|---|---|---|
| `system.def` `[Info] localcoord` (packs 4:3 e 16:9) | Implementado | Convertido para o espaço do HUD |
| `system.def` `[Files]`, `[Music]`, `[Title Info]`, `[Select Info]` | Implementado | |
| `system.def` `[<nome>BGdef]` / `[<nome> N]` (fundos) | Implementado | |
| `select.def` `[Characters]`, `[ExtraStages]`, `[Options]` | Implementado | |
| `fight.def` `[Files]`, barras `pN.*`, prefixos de elementos | Implementado | |
| Storyboards (`[SceneDef]`, `[Scene N]`, fades, camadas, `bgm`) | Implementado | |
| `.fnt` v1 (PCX embutido, `[Def]`, `[Map]`) | Implementado | |
| `.fnt` v2 (bitmap e truetype) | Parcial | Lido em `fnt.rs`; renderização truetype não verificada |
| `[Versus Screen]`, `[Victory Screen]`, `[Round Info]` | Parcial | Lidos pelos prefixos genéricos; nem toda chave tem renderizador próprio; não verificado em packs |
| Particularidades de WinMUGEN e MUGEN 1.0 / 1.1 | Parcial | Chaves desconhecidas são ignoradas, não rejeitadas; sem seletor por versão |

## Lacunas conhecidas (próximos PRs)

Pequenos PRs, nesta ordem:

1. AIR `Interpolate` (palavra-chave hoje ignorada).
2. Stage `resetbg`.
3. SFF v1 com profundidade baixa (PCX de 1 e 4 bits). Precisa de um caminho de paleta do cabeçalho; `sff.rs` está na linha de base do tamanho de arquivo, então o código v1 deve ir antes para um módulo próprio.
4. Verificar codecs WAV além de PCM 8/16 bits com packs reais.
5. Chaves de screenpack sem renderizador próprio (telas de versus e de vitória).

Esta lista é o que a auditoria encontrou; ainda não está provado que seja completa numa coleção grande de packs.
