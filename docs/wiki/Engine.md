# Customizar a engine

## Arquitetura

| Caminho | Responsabilidade |
|---|---|
| `crates/formats` | Leitura de arquivos M.U.G.E.N: INI, SFF, AIR, SND, ACT, PCX, PNG, stages, sistema de arquivos virtual (pastas e zip). Sem dependência de plataforma |
| `crates/engine` | Máquina de estados: expressões (`expr.rs`), triggers (`triggers.rs`), controllers (`controllers.rs`), HitDef (`hitdef.rs`), luta (`matchs.rs`), comandos e entrada, compilador UMF (`native/`) |
| `src/` | Aplicação: janela, render, HUD, áudio, menus, cenários nativos, efeitos visuais, controles e gamepad |
| `web/` | Página, service worker e ponte JS da versão navegador |
| `assets/chars` | Lutadores UMF embutidos |

Fluxo de um tick: entradas → `Match::tick` → para cada lutador, `[Statedef -3/-2/-1]` e estado atual → controllers cujos triggers são verdadeiros → física → colisão e `apply_hit` → câmera. A simulação roda a 60 ticks por segundo, é determinística e não depende do render.

## Adicionar um state controller

1. Em `crates/engine/src/controllers.rs`, dentro de `run`, adicione um ramo com o nome em minúsculas:

```rust
"victoryquote" => { let v = n(m, "value", -1.0) as i32; m.fighters[i].victory_quote = v; }
```

`victory_quote` é um campo novo que você cria em `Fighter`.


2. `n(m, "param", padrão)` lê um parâmetro numérico; `n2(m, "param", padrão, índice)` lê o N-ésimo valor; `sc.has("param", 0)` testa presença; `sc.raw("param")` devolve o texto.
3. Crie o campo em `fighter.rs` se precisar de estado novo.
4. Escreva um teste em `crates/engine/tests/combat_logic.rs` chamando `ume_engine::controllers::run`.

## Adicionar um trigger

Em `crates/engine/src/triggers.rs`, adicione um ramo no `match` de nomes:

```rust
"winko" => b(m.match_over && m.winner_side == f.side as i32 && m.won_by_ko),
```

`f` é o lutador avaliado e `m` a luta. `won_by_ko` seria um campo novo em `Match`. `b(bool)` converte para 0/1. Triggers que retornam float precisam aparecer em `is_float` de `expr.rs`.

## Adicionar um formato de arquivo

Coloque o decoder em `crates/formats/src/<formato>.rs`, exporte em `lib.rs` e teste com arquivos reais em `crates/formats/tests/real_files.rs`. O crate precisa continuar compilando para `wasm32-unknown-unknown`.

## Testes

| Comando | Cobre |
|---|---|
| `cargo test --workspace --release` | Tudo |
| `cargo test -p ume-engine --test kfm` | Kung Fu Man de referência |
| `cargo test -p ume-engine --test combat_logic` | Regras de combate |
| `cargo test -p ume-formats` | Decoders |
| `cargo clippy --workspace -- -D warnings` | Lint |
| `node scripts/web-smoke.js web/dist` | Versão web no Chromium |

## Regras de código

- Sem comentários no código: nomes de funções, tipos e variáveis devem explicar a intenção. Explicações ficam nesta wiki.
- A simulação (`crates/engine`) não usa relógio, threads nem aleatoriedade fora de `Match::random`, para manter replays e rollback.
- Nada de alocação dentro de `Match::tick` em código novo.
- Mensagens de commit no padrão Conventional Commits (`feat:`, `fix:`), validadas por `scripts/commit-lint.mjs`.

## Decisões de arquitetura

| Tema | Escolha |
|---|---|
| Render | `wgpu` (Vulkan, Metal, DX12, WebGPU, WebGL2), paleta aplicada em shader |
| Scripts | CNS e ZSS para lutadores, UMF JSON para o formato nativo; sem Lua |
| Expressões | Bytecode de pilha |
| Áudio | `kira` + `symphonia` |
| Rede | `ggrs` (rollback) + `matchbox` (WebRTC) |

A migração está descrita em [`docs/ROADMAP.md`](../ROADMAP.md).
