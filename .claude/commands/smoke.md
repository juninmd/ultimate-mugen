---
description: Smoke test completo do Ultimate MUGEN (testes, chars MUGEN, app headless)
---
Rode o smoke test e reporte de forma curta:

1. `cargo test --workspace --release` — todos devem passar.
2. `cargo clippy --workspace --release` — nenhum `error`.
3. Extraia `test-assets/kfm.zip` e `assets/mugen/{elecbyte,community}.zip` em pastas novas no scratchpad e rode
   `cargo run --release -p ume-engine --example mugen_check -- <dir>/chars` em cada; sem `FAIL`/`PANIC`.
4. `cargo build --release` e rode o app sob Xvfb com `--shots <scratchpad>/shots --mute`; exit 0, sem panic do jogo
   (ignore ALSA). Olhe 2–3 screenshots.
5. Corrija bugs encontrados, commite na branch designada e só abra PR se pedido.
