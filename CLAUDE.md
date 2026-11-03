# Ultimate MUGEN — Claude Code

@AGENTS.md

## Comandos rápidos
- Build: `cargo build --release`
- Testes: `cargo test --workspace --release`
- Lint: `cargo clippy --workspace --release` (deve ficar sem `error`; CI roda testes + build wasm32)
- Scripts de release: `node --test scripts/*.test.mjs`
- Smoke headless de chars MUGEN: `cargo run --release -p ume-engine --example mugen_check -- <dir>`
- Smoke do app completo (sem GPU): `xvfb-run -a env LIBGL_ALWAYS_SOFTWARE=1 ./target/release/ultimate-mugen --shots <dir> --mute`
- Slash command: `/smoke` roda o checklist completo.

## Estrutura
- `crates/formats` (`ume-formats`): parsers SFF/AIR/CMD/CNS/SND/DEF, VFS, stages.
- `crates/engine` (`ume-engine`): simulação, IA, loader MUGEN, chars nativos (`native/`).
- `src/`: app macroquad (UI, render, vfx, stagefx, áudio, input).
- `assets/chars/*.json`: roster nativo (índice em `index.json`).

## Convenções
- Responder em português, de forma curta.
- Desenvolver na branch designada; não abrir PR sem pedido explícito.
- Não versionar `mugen/` (chars locais) nem `target/`.
- Clippy deny-by-default quebra o build de lint: use `std::f32::consts::{PI,TAU}` em vez de literais e evite `* 0`.
- Panic de ALSA (`quad-snd`) em ambiente sem áudio é esperado; não é bug do jogo.
