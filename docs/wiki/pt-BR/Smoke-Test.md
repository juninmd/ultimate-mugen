# Smoke test (modo debug)

Um runner dentro do motor valida as principais features em todas as plataformas e deixa evidências: um print por etapa e um relatório que diz qual feature quebrou e por quê.

## Como rodar

| Alvo | Comando | Saída |
|---|---|---|
| Desktop (Linux, macOS, Windows) | `ultimate-mugen --smoke <dir>` (Linux sem tela: `xvfb-run -a ultimate-mugen --smoke <dir> --res 1280x720`) | `<dir>/report.json`, `<dir>/report.md`, `<dir>/shots/*.png` |
| Web | `node scripts/smoke-web.js web/dist <dir>` | mesmos arquivos |
| Web mobile | `SMOKE_MOBILE=1 node scripts/smoke-web.js web/dist <dir>` (viewport Pixel 7, toque) | mesmos arquivos |

O processo sai com código 1 se qualquer checagem falhar. `UME_SMOKE_ONLY=boot,fight_1v1` roda só essas features (o driver web também respeita); a página aceita `?smoke=1&smoke_only=boot`. `UME_SMOKE_FRAME_BUDGET_MS` muda o orçamento médio de quadro (500 ms).

## O que cobre

| Feature | Checagens |
|---|---|
| `boot` | lutadores e cenários registrados, nada em quarentena, sem erro de importação, título renderiza |
| `screens` | menu, ajustes, controles, importação MUGEN, mercado, replays, ficha e ajuste de personagem renderizam |
| `select` | seleção de lutador (1P e 2P) e de cenário renderizam |
| `fight_1v1` | luta carrega, intro termina, combate causa dano, KO encerra a partida, tela de resultado aparece |
| `training` | treino carrega e a simulação avança |
| `team_5v5` | dez lutadores carregam e renderizam |
| `story` | introdução da história e a primeira onda carregam e renderizam |

Cada feature também checa o tempo médio de quadro, e cada print é checado contra quadro preto ou chapado.

## Lendo o relatório

`report.md` tem uma linha por feature. Feature com falha expande as checagens que falharam, cada uma com link para o print. Um panic é gravado no relatório com a feature que estava rodando, então a área do problema é conhecida mesmo quando o jogo morre.

## Adicionando uma checagem

Crie uma função em `src/smoke/features/` (arquivo novo perto de 250 linhas), liste em `src/smoke/features.rs` e use `ctx.check(nome, ok, detalhe)` ou `ctx.shot(app, nome, quadros_de_aquecimento)`. Conduza o jogo pelo estado de `App`, como os tours de screenshot; mantenha cada feature em poucos segundos.

## Ainda não coberto

Netplay, atualização automática, importação de zip e controles de toque na tela não são exercitados pelo runner.

## Varreduras noturnas

Simulações do elenco inteiro que levam de 10 a 30 s cada (`native_roster_lab`, `native_air_abilities`, `native_match_invariants`, `native_input_moves`, `native_normal_reach`, `native_no_stuck_states`, `native_attack_keyframes`, `pixel_states_test` e a varredura de golpes aéreos) estão marcadas com `#[ignore = "nightly: full-roster sweep"]`. O gate do PR as pula; `.github/workflows/nightly.yml` roda todas por dia (`cargo test --workspace --release -- --ignored`), e localmente dá para rodar do mesmo jeito.
