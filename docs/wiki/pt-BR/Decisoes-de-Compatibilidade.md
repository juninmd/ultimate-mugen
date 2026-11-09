# Decisões de compatibilidade

Esta página registra o que a engine deliberadamente não implementa, para que autores de personagens e cenários tenham uma resposta clara em vez de uma falha silenciosa.

## State controllers de módulo

A camada de módulos/motif do Ikemen é triada em grupo. Nenhum destes está implementado; o verificador de compatibilidade os reporta como controllers não suportados.

| Controller | Decisão | Motivo |
|---|---|---|
| `Storyboard` | Planejado | Depende do runtime de screenpack/storyboard; anda junto com o trabalho de screenpacks |
| `Dialogue` | Planejado | Mesmo runtime do `Storyboard` |
| `MatchRestart` | Planejado | Depende das opções de regras e de um reinício determinístico |
| `LifebarAction` | Planejado | Depende do renderizador de lifebars do `fight.def` |
| `ChangeMovelist` | Planejado | Precisa de uma move list por luta; a lista de debug é somente leitura hoje |
| `LoadFile` | Rejeitado por decisão | Personagens rodam dentro de uma luta determinística; leitura de disco por script não é permitida |
| `SaveFile` | Rejeitado por decisão | Mesma regra de sandbox do `LoadFile` |
| `SaveState` | Rejeitado por decisão | A engine não tem snapshot de rollback; reproduzir o stream de inputs é o caminho suportado |
| `LoadState` | Rejeitado por decisão | Igual ao `SaveState` |
| `ShaderSet` | Rejeitado por decisão | Shaders fazem parte do renderizador, não do conteúdo do personagem |

A lista `REJECTED_CONTROLLERS` em `crates/engine/src/diagnostics.rs` é usada pelo relatório de carga: esses nomes aparecem como `(rejected by design)` no detalhe, enquanto os planejados aparecem apenas como não suportados.

## Cenários 3D

Cenários glTF são **rejeitados por decisão**. O loader de cenário detecta o pedido de modelo (`[Model]`, `model = *.glb`, `[StageInfo] scene = ...`), avisa uma vez por cenário e renderiza as camadas 2D normais. Não existe fallback para cenários que só tenham modelo 3D.

## Música de tracker

Música `xm`, `it`, `mod` e `s3m` é **rejeitada por decisão** por enquanto. O backend de áudio decodifica ogg/vorbis, mp3, flac e wav; uma faixa não suportada gera aviso e a luta continua com a música embutida em vez de ficar muda.

## Desvios de física intencionais

| Comportamento | M.U.G.E.N | Ultimate MUGEN | Motivo |
|---|---|---|---|
| Estado de hit aéreo com `physics = N` | Sem gravidade: um lutador atingido por golpes repetidos com `yvel = 0` fica suspenso no ar | O lutador cai, como se o estado tivesse gravidade | Impede que lutadores flutuem enquanto um projétil continua acertando. Estados de ataque com `physics = N` (golpes de voo e pairar) mantêm a altura programada. |

