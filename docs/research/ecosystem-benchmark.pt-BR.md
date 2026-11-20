# Ultimate MUGEN — Benchmark do Motor e Ecossistema de Terceiros

Data: `2026-11-20` · Plataforma: Windows `x86_64` · Perfil: `--release`

## 1. Benchmarks Reais do Motor (Baseline)

### Gate Zero-Allocation (`tick_alloc`)
- `cargo test -p ume-engine --release --test tick_alloc`: **PASS** (`2 passed, 0 failed`, `0.05s`)
- Confirma que `Match::tick` executa **zero alocações de heap**, preservando determinismo para rollback `ggrs` e replays.

### Vazão da Simulação (`native_tick_bench`)
- Comando: `cargo run --release -p ume-engine --example native_tick_bench -- assets/chars`
- Resultado: **21 lutadores · 252.000 ticks · 62.280 ticks/s**
- Tempo médio por tick: **0,0161 ms** (1.038× mais rápido que o orçamento de 16,67 ms a 60 Hz)
- Pior tick: **5,310 ms**

### Carregamento de Lutadores (`load_bench`)

| Lutador | Carga Completa (ms) | Sprites | Preview (ms) | Observações |
|---|---:|---:|---:|---|
| `jin` | 118 | 280 | 47 | Mais rápido do elenco principal |
| `haruki` | 119 | 270 | 36 | — |
| `garan` | 137 | 278 | 43 | — |
| `bruno` | 141 | 278 | 36 | — |
| `akio` | 146 | 281 | 40 | — |
| `daichi` | 153 | 287 | 46 | — |
| `volta` | 153 | 291 | 42 | — |
| `marlow` | 155 | 284 | 40 | — |
| `lian` | 162 | 280 | 41 | — |
| `shion` | 166 | 295 | 47 | — |
| `valdric` | 173 | 277 | 55 | — |
| `kaname` | 185 | 311 | 45 | — |
| `suzume` | 191 | 416 | 63 | — |
| `npc_spitfire` | 197 | 144 | 11 | — |
| `sylene` | 204 | 270 | 59 | — |
| `kaito` | 212 | 422 | 52 | — |
| `oboro` | 281 | 559 | 68 | Maior pacote de sprites (chefe) |
| `npc_razor` | 302 | 142 | 11 | **Anomalia: 2× mais lento por sprite** |
| `npc_phantom` | 436 | 140 | 11 | **Anomalia: 3× mais lento por sprite** |
| `npc_colossus` | 584 | 148 | 10 | **Anomalia: 4× mais lento por sprite** |
| `npc_brute` | 748 | 136 | 10 | **Anomalia: 5× mais lento por sprite** |
| **Total (21 lutadores)** | **4.962 ms** | — | — | — |

> **Descoberta Acionável:** `npc_brute`, `npc_colossus`, `npc_phantom` e `npc_razor` levam 302–748 ms mesmo tendo ~140 sprites (~metade de um lutador padrão). Otimizar a compressão/dimensão dos PNGs em `assets/chars/sprites/npc_*/` pode reduzir o tempo total de carregamento em **~35% (~1,7 s)**.

---

## 2. Avaliação de Crates Rust de Terceiros

| Domínio | Stack Atual | Crate Candidata (Estável) | Seguro em WASM | Impacto no Determinismo | Veredito |
|---|---|---|---|---|---|
| **Netplay Online (Web)** | `ggrs 0.13.0` (apenas UDP desktop) | **`matchbox_socket` `0.4.0`** (WebRTC P2P) | Sim (nativo) | Nenhum (apenas transporte) | **ADOTAR (P0)** — Conecta o rollback `ggrs` existente a partidas PvP no navegador sem instalação. |
| **Layout de UI** | Coordenadas virtuais manuais (`1280×720`) | **`taffy` `0.15.1`** (Flexbox / Grid) | Sim (`no_std`) | Nenhum | **ADOTAR SELETIVAMENTE (P1)** — Ideal para telas responsivas (Menus, Configurações, Torre, Seleção) mantendo o render `ume_shell::quad`. |
| **UI de Debug / Devtools** | Quads imediatos customizados | **`egui` `0.37.0`** | Sim | Nenhum | **APENAS DEV (P2)** — Pesado para UI de jogador, mas excelente sob feature flag `dev-ui` para editor de hitboxes e frame data. |
| **Áudio e BGM Dinâmica** | `symphonia 0.5.5` + `quad-snd` | **`kira` `0.12.6`** | Parcial (requer backend web) | Nenhum | **AVALIAR (P2)** — Permite transições sincronizadas por batida e crossfade dinâmico (awakening, vida baixa). |
| **Scripting de Modos/História** | `toml 1.2.1` (História declarativa) | **`rhai` `1.23.4`** (Rust puro, sandbox) | Sim (Rust puro) | Seguro fora de `Match::tick` | **ADOTAR PARA MODS (P1)** — Scripting sandboxed para eventos de História, regras da Torre e gatilhos customizados sem FFI C. |
| **Lua legado (IKEMEN)** | Parser ZSS próprio (`crates/engine/src/zss`) | **`mlua` `0.12.x`** (Lua 5.4 / Luau) | Complexo em WASM | Alto risco no tick | **ADIAR (P3)** — Priorizar TOML + `rhai` para modos nativos e manter ZSS em Rust puro. |

---

## 3. Ferramentas do Ecossistema de Luta (Importação e Adaptação)

| Ferramenta / Projeto | Licença / Ecossistema | O que Importar / Adaptar | Caminho de Integração |
|---|---|---|---|
| **IKEMEN GO** | MIT (Go) | Barra de frame data no HUD, padrões ZSS, convenções Tag/Simul, gravação do boneco de treino | Referência para `crates/engine/src/zss` e barra de vantagem de frames no Modo Treino. |
| **Fighter Factory Studio (FF3/FFS)** | IDE padrão da comunidade | Fluxo `.def` / `.sff` / `.air` / `.cns` / `.snd` | Adicionar diagnósticos no `ume-devtools` (`mugen_check --lint`) voltados para exportações do FFS. |
| **Aseprite JSON Export** | Padrão de pixel art | Tags de animação, duração de quadros e pivôs | Subcomando `ume-devtools import-aseprite` gerando `meta.json` e animações UMF diretamente. |

---

## 4. Ordem Recomendada de Adoção

1. **P0 — Netplay Rollback no Navegador (`matchbox_socket` + `ggrs`):** PvP online direto via WebRTC em `juninmd.github.io/ultimate-mugen/`.
2. **P0 — Otimização dos PNGs de NPCs da História:** Redução imediata de ~1,7 s no tempo de carga total.
3. **P1 — Layout Responsivo (`taffy`) em Configurações/Torre/Seleção:** Elimina cálculo manual de coordenadas e melhora suporte a telas mobile/ultrawide.
4. **P1 — Scripting Sandboxed (`rhai`):** Estende o novo sistema TOML de História e Torre Diária com fases dinâmicas de chefes e gatilhos customizados.
