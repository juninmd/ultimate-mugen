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
5. Liste o nome em `CNS-Scripts.md` e `pt-BR/Scripts-CNS.md`. `cargo test -p ume-engine --test wiki_sync` falha para um controller implementado que não está listado e para um listado que não está implementado. Devolva `true` no `run` do módulo (ou caia no último ramo) para o diagnóstico de carregamento não apontá-lo como não suportado.

## Adicionar um trigger

Em `crates/engine/src/triggers.rs`, adicione um ramo no `match` de nomes:

```rust
"winko" => b(m.match_over && m.winner_side == f.side as i32 && m.won_by_ko),
```

`f` é o lutador avaliado e `m` a luta. `won_by_ko` seria um campo novo em `Match`. `b(bool)` converte para 0/1. Triggers que retornam float precisam aparecer em `is_float` de `expr.rs`. Liste o novo nome em `CNS-Scripts.md` e `pt-BR/Scripts-CNS.md`; o `wiki_sync` confere nos dois sentidos.

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
- Arquivos ficam abaixo de 250 linhas, um assunto cada, divididos pelo eixo de mudança: veja [Layout de arquivos](Layout-de-Arquivos.md) e `node scripts/file-size.mjs`.
- Mensagens de commit no padrão Conventional Commits (`feat:`, `fix:`), validadas por `scripts/commit-lint.mjs`.

## Decisões de arquitetura

| Tema | Escolha |
|---|---|
| Render | `wgpu` (Vulkan, Metal, DX12, WebGPU, WebGL2), paleta aplicada em shader |
| Scripts | CNS e ZSS para lutadores, UMF JSON para o formato nativo; sem Lua |
| Expressões | Bytecode de pilha |
| Áudio | `kira` + `symphonia` |
| Rede | `ggrs` (rollback) + `matchbox` (WebRTC) |

A migração está descrita em [`docs/ROADMAP.pt-BR.md`](../../ROADMAP.pt-BR.md).

## Dono do estado e controle

Um lutador roda o estado a partir da definição que o forneceu: o `.cns` dele, o `common1.cns` compartilhado, ou a definição de outro personagem (`TargetState`, `p2stateno`, um arremesso). Três regras mantêm o controle fluindo:

- Um número de estado que o personagem define sempre roda a partir da definição dele, então voltar de um estado de engine (face-off, despertar) ou de um estado do `common1` devolve o controle em vez de deixar o lutador preso à definição compartilhada.
- Estados do `common1.cns` são do próprio jogador: os estados `-1` do personagem e as transições de andar/pular/agachar da engine continuam rodando durante um get-hit, deitado ou qualquer estado compartilhado — personagens M.U.G.E.N. cancelam golpes pelo `-1` assim.
- Só um estado customizado dado pelo oponente pausa esses estados, porque o script de quem deu é dono do lutador até devolver o controle.

Um lutador cujo dono do estado não é ele mesmo nem o `common1` está "em estado customizado"; é esse o teste que protege toda transição vinda do input, e perdê-lo faz o personagem ficar parado mesmo parecendo vivo (`ctrl`, estado 0).

## HUD glass e efeitos de impacto

- Os painéis do HUD em luta são "glass": `ume_shell::quad::glass_panel` registra até 16 retângulos arredondados por quadro, e o passe de composição do pós-processamento troca esses pixels por uma cópia desfocada da cena (um quarto da resolução, blur separável, sem alocações por quadro), com tinta, sombra e brilho na borda. Com o pós-processamento desligado (nível `fx` 0) o HUD usa polígonos translúcidos.
- O feedback de impacto é discreto: guard e counter usam o mesmo rótulo pequeno com anel, o super arts só tem uma rajada leve de anéis e um rótulo compacto, e a quantidade simultânea de textos, anéis, luzes e partículas tem limite em `src/vfx.rs`.
