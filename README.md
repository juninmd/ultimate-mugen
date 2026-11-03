# Ultimate MUGEN

Motor de luta 2D **feito do zero em Rust**, inspirado no M.U.G.E.N: moderno, multiplataforma (Windows, Linux, macOS, **navegador/WASM**, **Android/iOS**), **compatível com personagens M.U.G.E.N** e com um formato novo, muito mais simples, para criar lutadores. Suporte nativo a **DualShock 4 / DualSense com vibração**.

![Luta](screenshots/06-fight-dojo.jpg)

| | |
|---|---|
| ![Super](screenshots/08-fight-super.jpg) | ![Seleção](screenshots/03-select.jpg) |
| ![MUGEN KFM vs original](screenshots/12-mugen-kfm-vs-native.jpg) | ![Cenário MUGEN](screenshots/13-native-on-mugen-stage.jpg) |
| ![Kaname vs Valdric](screenshots/11b-fight-kaname-valdric.jpg) | ![Celular](screenshots/15-mobile-fight.jpg) |
| ![Seleção com muitos lutadores](screenshots/03b-select-many.jpg) | ![Loading no VS](screenshots/05a-vs-loading.jpg) |
| ![Vídeo](screenshots/17-video.jpg) | ![4K](screenshots/19-fight-4k.jpg) |
| ![KO cinematográfico](screenshots/20-fx-ko.jpg) | ![Super](screenshots/08-fight-super.jpg) |
| ![Impacto neon](screenshots/21-fx-neon-hit.jpg) | ![Transição](screenshots/22-transition.jpg) |
| ![Equipe sequência](screenshots/30-team-sequence.jpg) | ![Equipe paralelo](screenshots/31-team-parallel.jpg) |
| ![Seleção de equipe](screenshots/32-team-select.jpg) | ![Quadro de impacto](screenshots/23-impact-frame.jpg) |

## Por que Rust + macroquad
| Requisito | Escolha |
|---|---|
| Um código para desktop, web e celular | **macroquad/miniquad** (OpenGL/GLES/WebGL; Windows, Linux, macOS, WASM, Android, iOS) |
| Desempenho e determinismo (base para rollback/replays) | Rust, passo fixo 60 Hz, PRNG determinístico, sem GC |
| Controles + vibração | **gilrs** + relatórios HID diretos para DualShock 4/DualSense (USB e Bluetooth, com CRC32) + Gamepad API na web |
| Arte procedural nítida | **tiny-skia** (rasterização vetorial com antialias, gradientes) |
| Leve | WASM ~3 MB, binário nativo único, sem runtime |

## Destaques
- **Compatibilidade MUGEN** (`crates/formats` + `crates/engine`): `.def`, `.sff` v1/v2 (PCX, RLE8, RLE5, LZ5, PNG8/24/32), `.air`, `.cmd`, `.cns`, `.snd`, `.act`, cenários com parallax/tiles/animação. Linguagem de triggers completa (inteiro vs float, intervalos, redirecionamentos), ~90 state controllers, HitDef completo, arremessos (custom states), helpers, projéteis, explods, super pause, after images, `localcoord` HD. Validado com o Kung Fu Man oficial (SD e 720p).
- **Formato UMF**: um personagem = 1 JSON com golpes declarativos; hitboxes automáticas pelo esqueleto; botão "especial simples" no celular. Compila para a mesma VM do MUGEN — UMF e MUGEN lutam entre si.
- **15 lutadores originais** + **10 personagens M.U.G.E.N incluídos** (Kung Fu Man SD/HD e cenários da Elecbyte, Suave Dude, 7 originais da comunidade com licença livre) e **14 cenários** (6 originais + 8 MUGEN) procedurais (sem assets de terceiros), música e efeitos sonoros sintetizados.
- **4K de verdade**: tela cheia (F11), janela até 5K, e a arte procedural (lutadores UMF e cenários) é rasterizada em densidade proporcional à tela — *Automática*: 3× até 900 px de altura, 4× até 1300, 4,5× até 1700, **6× em 4K** (menu **Vídeo**; flags `--res 3840x2160`, `--4k`, `--fullscreen`). Texto e UI escalam sem re-rasterizar fontes a cada quadro.
- **Muitos personagens e cenários**: seleção com rolagem, filtro (Tab: todos/originais/M.U.G.E.N), PgUp/PgDn, roda do mouse, `R` aleatório; só as linhas visíveis são desenhadas e só o que está sob o cursor é carregado (cache LRU limita a memória). Pastas/zips importados são registrados sem decodificar tudo (centenas em segundos).
- **Loadings rápidos**: tela de carregamento na inicialização, pré-carga por etapas atrás da tela de VS (barra de progresso), prévia leve (idle/vitória) nos menus, rasterização dos lutadores em paralelo (≈3× mais rápido) e música sintetizada antes da luta.
- **Visual AAA** ([`docs/AAA_FEATURES.md`](docs/AAA_FEATURES.md)): pós-processamento (bloom, aberração cromática, blur radial, vinheta, grade de cor), partículas e anéis de choque em cada golpe, números de dano, *camera punch*, KO em câmera lenta com zoom, super cinematográfico com linhas de velocidade e aura, reflexos no chão, HUD animado, transições de tela e fundo vivo. Opção de acessibilidade para reduzir efeitos (Vídeo → Efeitos de impacto).
- **Modos de equipe**: *Sequência 3v3* (estilo KOF: ordem de entrada, vencedor mantém a vida e o medidor de super compartilhado) e *Paralelo 2v2* (parceiro IA luta ao seu lado; o round acaba quando a equipe inteira cai). Ver [`docs/AAA_FEATURES.md`](docs/AAA_FEATURES.md).
- **Áudio em camadas**: SFX sintetizados com transiente + corpo + crunch, 3 variações por golpe, reverb, stingers de round/KO/vitória, pings de combo e *ducking* da música. **HUD** com retratos reais, barras de parceiro, roster de equipe e alarmes.
- Modos **Arcade, Versus CPU, 2 Jogadores, Treino (hitboxes), CPU vs CPU**; IA compatível com qualquer personagem.
- **Controles**: teclado, gamepads (DualShock/DualSense com vibração em golpes, defesa, super e K.O.; lightbar azul/vermelha por jogador), toque com analógico virtual e SP1/SP2/SP3/SUPER de um toque.

## Rodando
```bash
cargo run --release                      # desktop
cargo test --workspace --release         # testes (parsers, VM, KFM headless se disponível, elenco)
./scripts/build-web.sh                   # web/dist → sirva com qualquer servidor HTTP estático (PWA)
cargo run --release -- --shots out/      # tour automático de capturas de tela
cargo run --release -- --4k              # janela 3840×2160 (ou --res 2560x1440, --fullscreen)
cargo run --release -p ume-engine --example load_bench   # tempo de carga dos lutadores (completo vs. prévia)
```
Linux precisa de: `libudev-dev libasound2-dev libx11-dev libxi-dev libgl1-mesa-dev`.

**Android**: `cargo quad-apk build --release` (imagem docker `notfl3/cargo-apk`). **iOS**: ver [guia do macroquad](https://macroquad.rs/articles/ios/).

### Personagens MUGEN
Desktop: coloque pastas ou `.zip` em `mugen/` ao lado do executável (ex.: `mugen/chars/kfm/kfm.def`, `mugen/stages/…`) ou defina `UME_MUGEN_DIR`. Web: botão **Importar MUGEN (.zip)** ou arraste o arquivo para a página.

## Controles
| | Teclado | DualShock/DualSense | Toque |
|---|---|---|---|
| Mover | WASD / setas | D-pad / analógico | Analógico virtual |
| Socos | U · I · O | □ · △ · R1 | LP · HP |
| Chutes | J · K · L | ✕ · ○ · R2 | LK · HK |
| Especiais simples | 1 · 2 · 3 (Super: 1+2) | L1 · L2 | SP1–SP3 · SUPER |
| Pausa / Hitboxes | Esc / F2 | OPTIONS | ❚❚ |
| Tela cheia / FPS | F11 / F3 | — | — |
| Lista de lutadores | Tab (filtro) · PgUp/PgDn · R (aleatório) | — | — |

## Estrutura
```
crates/formats   formatos MUGEN (SFF/AIR/CMD/INI/SND/ACT/PCX/PNG/zip/stages) — puro Rust, roda em WASM
crates/engine    VM de estados, expressões, hit system, match, IA, efeitos, UMF (compilador + rig procedural)
src/             app macroquad: telas, render, HUD, áudio, controles/vibração, toque, cenários
assets/          fontes OFL, personagens UMF (JSON)
web/             página, plugin JS (gamepad/vibração/importação), PWA
docs/            pesquisa, formato UMF, elenco
```

## Documentação
- [`docs/PESQUISA.md`](docs/PESQUISA.md) — como o MUGEN funciona, sucessores (Ikemen GO), dores e arquitetura.
- [`docs/CHARACTER_FORMAT.md`](docs/CHARACTER_FORMAT.md) — como criar um personagem UMF.
- [`docs/ROSTER.md`](docs/ROSTER.md) — elenco original + catálogo de personagens MUGEN populares (Dragon Ball, Bleach, Naruto…) para importar.

## Aviso
Personagens de anime/jogos (Goku, Ichigo, Naruto…) pertencem a seus detentores e **não são distribuídos**. O motor executa os pacotes MUGEN feitos por fãs que o próprio jogador importar. Fontes: Russo One e Rajdhani (SIL Open Font License).
