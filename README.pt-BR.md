# Ultimate MUGEN

[English](README.md) · **Português (BR)**

Motor de luta 2D **feito do zero em Rust**, inspirado no M.U.G.E.N: moderno, determinístico, multiplataforma (Windows, Linux, macOS, **navegador/WASM**, **Android/iOS**), **compatível com personagens e cenários clássicos do M.U.G.E.N** e com suporte ao novo formato nativo **UMF (Ultimate Mugen Fighter)**, que é declarativo e muito mais simples. Conta com suporte nativo a **DualShock 4 / DualSense com feedback háptico/vibração e lightbars coloridas**.

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
| ![5v5 paralelo](screenshots/33-team-5v5-parallel.jpg) | ![5v5 sequência](screenshots/34-team-5v5-sequence.jpg) |
| ![4K · HUD 5v5](screenshots/35-4k-5v5-hud.jpg) | ![4K · super](screenshots/36-4k-super.jpg) |

---

## Índice
- [Por que Rust + wgpu?](#por-que-rust--wgpu)
- [Destaques](#destaques)
- [Pré-requisitos](#pré-requisitos)
- [Build e Execução](#build-e-execução)
  - [Build Nativo (Desktop)](#build-nativo-desktop)
  - [Build Web / PWA (WebAssembly)](#build-web--pwa-webassembly)
  - [Build Mobile (Android & iOS)](#build-mobile-android--ios)
- [Controles](#controles)
  - [Tabela de Controles em Luta](#tabela-de-controles-em-luta)
  - [Navegação nos Menus](#navegação-nos-menus)
  - [Feedback Háptico e Luzes (DualShock 4 / DualSense)](#feedback-háptico-e-luzes-dualshock-4--dualsense)
- [Arquitetura dos Crates](#arquitetura-dos-crates)
- [Formato de Assets](#formato-de-assets)
  - [1. Formato MUGEN Clássico](#1-formato-mugen-clássico)
  - [2. Formato Nativo UMF (JSON)](#2-formato-nativo-umf-json)
  - [3. Áudio e Fontes](#3-áudio-e-fontes)
- [Como Contribuir](#como-contribuir)
- [Releases e Atualização Automática](#releases-e-atualização-automática)
- [Documentação Adicional](#documentação-adicional)
- [Aviso Legal](#aviso-legal)

---

## Por que Rust + wgpu?

| Requisito | Escolha Técnica |
|---|---|
| **Base de código unificada** (desktop, web e mobile) | **wgpu + winit** (Vulkan, Metal, DX12, GLES, WebGL2; Windows, Linux, macOS, WASM, Android, iOS). |
| **Desempenho e determinismo** (base para rollback netcode e replays) | Rust com simulação em passo fixo (60 ticks/s), PRNG determinístico e ausência de Garbage Collector. |
| **Controles & Vibração avançada** | **gilrs** para entrada multiplataforma + relatórios HID diretos para DualShock 4/DualSense (USB e Bluetooth, com checksum CRC32) + HTML5 Gamepad API com `vibrationActuator` na web. |
| **Arte procedural nítida** | **tiny-skia** (rasterização vetorial com antialiasing e gradientes) para esqueletos procedurais em qualquer escala. |
| **Binário leve** | WASM compacto (~3 MB descompactado), binário nativo único e sem runtimes pesados externos. |

---

## Destaques
- **4K de verdade**: tela cheia (F11), janela até 5K, e a arte procedural (lutadores UMF e cenários) é rasterizada em densidade proporcional à tela — *Automática*: 3× até 900 px de altura, 4× até 1300, 4,5× até 1700, **6× em 4K** (menu **Vídeo**; flags `--res 3840x2160`, `--4k`, `--fullscreen`). Texto e UI escalam sem re-rasterizar fontes a cada quadro.
- **Muitos personagens e cenários**: seleção com rolagem, filtro (Tab: todos/originais/M.U.G.E.N), PgUp/PgDn, roda do mouse, `R` aleatório; só as linhas visíveis são desenhadas e só o que está sob o cursor é carregado (cache LRU limita a memória). Pastas/zips importados são registrados sem decodificar tudo (centenas em segundos).
- **Loadings rápidos**: tela de carregamento na inicialização, pré-carga por etapas atrás da tela de VS (barra de progresso), prévia leve (idle/vitória) nos menus, rasterização dos lutadores em paralelo (≈3× mais rápido) e música sintetizada antes da luta.
- **Visual AAA** ([`docs/AAA_FEATURES.md`](docs/AAA_FEATURES.md)): pós-processamento (bloom, aberração cromática, blur radial, vinheta, grade de cor), partículas e anéis de choque em cada golpe, números de dano, *camera punch*, KO em câmera lenta com zoom, super cinematográfico com linhas de velocidade e aura, reflexos no chão, HUD animado, transições de tela e fundo vivo. Opção de acessibilidade para reduzir efeitos (Vídeo → Efeitos de impacto).
- **História e cinema**: face-off com balões de fala no início do round 1, cutscenes no Arcade (abertura, confronto e final), **transformação** (`T`), animações de derrota diferentes a cada round, supers maiores, campo com **zoom-out** em equipes grandes e o **chefe final Mikazuki Oboro** (Itachi × Aizen) no fim do Arcade. Ver [`docs/AAA_FEATURES.md`](docs/AAA_FEATURES.md) (rodada 10).
- **Modos de equipe**: *Sequência 3v3* (estilo KOF: ordem de entrada, vencedor mantém a vida e o medidor de super compartilhado) e *Paralelo até **5v5*** (10 lutadores na tela; companheiros por IA, `Tab` troca o lutador que você controla; o round acaba quando a equipe inteira cai). Ver [`docs/AAA_FEATURES.md`](docs/AAA_FEATURES.md).
- **Áudio em camadas**: SFX sintetizados com transiente + corpo + crunch, 3 variações por golpe, reverb, stingers de round/KO/vitória, pings de combo e *ducking* da música. **HUD** com retratos reais, barras de parceiro, roster de equipe e alarmes.
- **Replay** da última luta (tela de resultado → Replay): o motor é determinístico e o replay re-simula as entradas gravadas, com teste. Ver o *roadmap* de features em [`docs/AAA_FEATURES.md`](docs/AAA_FEATURES.md).
- **Motor/jogabilidade**: escala de dano em combos (nativos), empurrão no canto, squash & stretch; **HUD v3** e efeitos construídos com polígonos (gradiente por vértice, chanfros, hexágonos). Ver rodada 9 em [`docs/AAA_FEATURES.md`](docs/AAA_FEATURES.md).

- **Compatibilidade MUGEN completa** (`crates/formats` + `crates/engine`):
  - Suporte a `.def`, `.sff` v1 e v2 (PCX 8-bit, RLE8, RLE5, LZ5, PNG8/24/32), `.air`, `.cmd`, `.cns`, `.snd` e `.act`.
  - Cenários com múltiplas camadas, parallax, repetição de tiles e animações dedicadas.
  - Interpretador e avaliador de expressões MUGEN com tipagem estática (divisão inteira vs float, intervalos `[a, b]`, redirecionamentos `p2`, `parent`, `root`, `helper`, etc.).
  - Mais de 90 *state controllers* implementados, sistema de `HitDef` completo, arremessos (*custom states*), helpers, projéteis, explods, super pause, after images e escalonamento `localcoord` HD.
  - Testado e validado contra personagens de referência como Kung Fu Man (SD e 720p).
- **Formato Nativo UMF**:
  - Cada lutador é definido em 1 único arquivo JSON declarativo.
  - Hitboxes e hurtboxes geradas automaticamente com base no esqueleto e nas partes ativas (`hit: { part: "handR" }`).
  - Compilado na inicialização para a mesma máquina de estados da VM do MUGEN — lutadores UMF e MUGEN se enfrentam sob regras idênticas.
- **Elenco e Conteúdo**:
  - **15 lutadores originais** procedurais + **10 personagens M.U.G.E.N clássicos compatíveis** (Kung Fu Man SD/HD, Suave Dude e personagens livres da comunidade).
  - **14 cenários** (6 originais procedurais + 8 MUGEN).
  - Áudio procedural sintetizado e efeitos WAV embutidos.
- **Modos de Jogo**:
  - Arcade, Versus CPU, 2 Jogadores (local / mesmo teclado ou gamepads), Treino (visualização de caixas de colisão) e CPU vs CPU.
  - IA universal capaz de lutar com qualquer personagem do elenco.

---

## Pré-requisitos

### 1. Ferramentas Rust
- **Rust Toolchain**: Versão estável (1.75+ recomendada).
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```

### 2. Dependências do Sistema (Linux / Debian / Ubuntu)
Para compilar nativamente no Linux, instale as bibliotecas de desenvolvimento de vídeo, áudio e dispositivos HID:
```bash
sudo apt-get update && sudo apt-get install -y \
    pkg-config \
    libudev-dev \
    libasound2-dev \
    libx11-dev \
    libxi-dev \
    libgl1-mesa-dev
```

*No macOS e Windows*, o Cargo utiliza diretamente as APIs nativas do sistema (Cocoa/CoreAudio e Win32/WASAPI), sem necessidade de pacotes adicionais.

### 3. Dependências para WebAssembly (Navegador)
Adicione o alvo de compilação WebAssembly ao seu toolchain Rust:
```bash
rustup target add wasm32-unknown-unknown
```

---

## Build e Execução

### Build Nativo (Desktop)

Compilar e rodar a aplicação em modo release:
```bash
cargo run --release
```

Para rodar a suíte completa de testes unitários e de integração:
```bash
cargo test --workspace --release
```

Modos especiais de inicialização por linha de comando:
```bash
cargo run --release -- --touch          # Força exibição dos controles virtuais de toque na tela
cargo run --release -- --mute           # Inicializa sem áudio
cargo run --release -- --shots out/     # Executa tour automatizado gerando capturas de tela em out/
cargo run --release -- --4k             # Janela 3840×2160 (ou --res 2560x1440, --fullscreen)
cargo run --release -p ume-engine --example load_bench   # Tempo de carga dos lutadores (completo vs. prévia)
node scripts/web-smoke.js web/dist      # Teste de fumaça da versão web (Chromium headless; precisa do pacote playwright)
```

### Build Web / PWA (WebAssembly)

O script `scripts/build-web.sh` compila o binário WASM e empacota os arquivos estáticos (HTML, assets, JS bridges e manifesto PWA) para a pasta `web/dist`:

```bash
# 1. Gerar o build web
./scripts/build-web.sh

# 2. Servir com qualquer servidor HTTP estático
python3 -m http.server -d web/dist 8080
# Ou via npx
npx serve web/dist
```
Acesse `http://localhost:8080` no navegador. Na versão web, você pode arrastar arquivos `.zip` de personagens ou clicar em **"Importar MUGEN (.zip)"**.

### Build Mobile (Android & iOS)

- **Android**:
  Com o Android SDK e NDK instalados (`ANDROID_HOME` e `ANDROID_NDK_ROOT`):
  ```bash
  cargo install cargo-apk --version 0.9.6 --locked
  cargo apk build -p ultimate-mugen --lib --release --target aarch64-linux-android
  ```
- **iOS**:
  No macOS com Xcode: `cargo install cargo-bundle` e `cargo bundle --release --target aarch64-apple-ios` (assine o `.app` com seu time no Xcode ou via `codesign`).

---

## Controles
O motor aceita simultaneamente teclado, múltiplos gamepads e toque na tela.

### Tabela de Controles em Luta

| Ação | Teclado (P1 Solo) | Teclado (P2 / Versus Local) | DualShock 4 / DualSense / XInput | Toque (Mobile / Web) |
|---|---|---|---|---|
| **Mover / Pular / Agachar** | `W` `A` `S` `D` ou Setas | Setas direcionais | D-Pad ou Analógico Esquerdo | Analógico virtual |
| **Soco Leve (LP)** | `U` (ou `V`) | Teclado Numérico `4` | Quadrado (`□`) / `X` (Xbox) | Botão **LP** |
| **Soco Médio/Forte (HP)** | `I` (ou `B`) | Teclado Numérico `5` | Triângulo (`△`) / `Y` (Xbox) | Botão **HP** |
| **Soco Extra (Z)** | `O` (ou `N`) | Teclado Numérico `6` | `R1` / `RB` | — |
| **Chute Leve (LK)** | `J` (ou `Z`) | Teclado Numérico `1` | Xis (`✕`) / `A` (Xbox) | Botão **LK** |
| **Chute Médio/Forte (HK)** | `K` (ou `X`) | Teclado Numérico `2` | Círculo (`○`) / `B` (Xbox) | Botão **HK** |
| **Chute Extra (C)** | `L` (ou `C`) | Teclado Numérico `3` | `R2` / `RT` | — |
| **Especiais Rápidos** | `1`, `2`, `3` | Teclado Numérico `7`, `8`, `9` | `L1`, `L2` | Botões **ESPECIAL**, **SUPER** |
| **Super Movimento** | `1` + `2` | Teclado Numérico `7` + `8` | `L1` + `L2` | Botão **SUPER** |
| **Despertar (transformação)** | `T` | Teclado Numérico `0` | Clique do analógico direito / `SELECT` | — |
| **Pausa / Hitboxes** | `Esc` / `F2` | `Esc` / `F2` | Botão `OPTIONS` / `START` | Botão `❚❚` (canto superior) |
| **Tela cheia / FPS** | `F11` / `F3` | — | — | — |
| **Trocar de lutador (equipe paralela)** | `Tab` | — | — | — |
| **Lista de lutadores** | `Tab` (filtro), `PgUp`/`PgDn`, `R` (aleatório) | — | — | — |

### Navegação nos Menus
- **Navegar**: Setas direcionais / `W` `A` `S` `D` / D-Pad do Gamepad.
- **Confirmar**: `Enter`, `Espaço`, `J`, `U`, ou `✕`/`A` no gamepad.
- **Voltar / Cancelar**: `Esc`, `Backspace`, `K`, ou `○`/`B` no gamepad.
- **Alternar Hitboxes (modo treino)**: Tecla `F2`.

### Feedback Háptico e Luzes (DualShock 4 / DualSense)
- Conexão nativa via USB e Bluetooth usando relatórios diretos HID.
- **Vibração proporcional**: Diferenciada para acerto de golpes leves, golpes pesados, bloqueios na defesa, ativação de Super e K.O.
- **Lightbar Dinâmica**:
  - Jogador 1: Azul vibrante.
  - Jogador 2: Vermelho vibrante.
  - Pisca em branco durante a ativação de Super / tela escura.

---

## Arquitetura dos Crates

O projeto é dividido em um Workspace Cargo com separação estrita entre parsing puro, motor determinístico e apresentação gráfica:

```
ultimate-mugen/
├── crates/
│   ├── formats/      # Parsers e decodificadores puros de formatos MUGEN (zero dependências de tela)
│   ├── engine/       # VM de luta determinística, física, colisões, IA e compilador UMF
│   ├── render/       # Renderer wgpu: atlas, sprites em lote com paletas em WGSL, pós-processamento
│   └── shell/        # Janela winit, entrada, texto, áudio e a API imediata usada pelo app
├── src/              # Camada da aplicação (telas, renderização, áudio, HID/gamepads e HUD); wgpu + winit em todas as plataformas
├── assets/           # Fontes OFL, definições UMF (.json) e pacotes de assets
├── docs/             # Documentação técnica detalhada, roadmap e pesquisa
└── web/              # Shell HTML, PWA, service worker e glue code JS
```

### Detalhamento dos Componentes:

1. **`crates/formats` (Biblioteca `ume-formats`)**:
   - `sff.rs`: Decodificação de arquivos SFF v1 e v2 (sprites PCX, raw, RLE8, RLE5, LZ5 e sub-arquivos PNG).
   - `air.rs`: Parser de especificações de animação `.air`, definição de quadros, caixas de colisão `Clsn1` (ataque) e `Clsn2` (corpo) e loops.
   - `ini.rs`: Parser genérico de arquivos INI flexíveis do MUGEN com suporte a comentários e seções sem chave.
   - `snd.rs` & `act.rs`: Extração de áudios WAV indexados em `.snd` e tabelas de cores `.act` de 256 cores.
   - `stage.rs`: Carregamento de cenários MUGEN (metadados de câmera, zoffset, bounds e camadas `BgLayer`).
   - `vfs.rs`: Virtual File System em memória que normaliza caminhos sem distinção de maiúsculas/minúsculas e lê diretamente de arquivos ZIP ou do disco.

2. **`crates/engine` (Biblioteca `ume-engine`)**:
   - `expr.rs`: AST, parser e avaliador de expressões lógicas e matemáticas com precisão MUGEN.
   - `chardef.rs` & `cns.rs`: Estrutura de dados de lutadores, compilação de `StateDef`s e constantes.
   - `commands.rs` & `input.rs`: Processamento de buffers de entrada, detecção de comandos com tolerância temporal e movimentos especiais (`~D, DF, F, x`).
   - `fighter.rs`: Estado em tempo real de cada lutador ou helper (posição, velocidade, variáveis de hit, caixas de colisão no espaço de mundo).
   - `matchs.rs`: Simulação principal da partida (passo fixo de 60 Hz, fluxo de rounds, física, colisões, projéteis, câmera e explods).
   - `hitdef.rs`: Parser e construtor dos parâmetros de impacto e propriedades de dano.
   - `controllers.rs`: Execução dos mais de 90 controladores de estado (ex: `VelSet`, `PosAdd`, `ChangeState`, `HitDef`, `Projectile`, `Explod`, `Pause`).
   - `native/`: Compilador do formato moderno UMF para a VM do MUGEN e rasterizador procedural por ossos via `tiny-skia`.
   - `ai.rs`: Algoritmo de inteligência artificial que lê dinamicamente comandos válidos e decide ações defensivas e ofensivas.

3. **`src/` (Binário principal `ultimate-mugen`)**:
   - `app.rs`: Gerenciador de ciclo de vida das telas (Menu Principal, Seleção de Personagens, Luta, Treino).
   - `render.rs`: Pipeline de desenho 2D com blend aditivo, sombras projetadas e efeitos de flash/pausa.
   - `gamepad.rs`: Integração de baixo nível com gamepads via `gilrs` e envio de relatórios HID brutos com cálculo de CRC32 para PlayStation DualShock 4 e DualSense.
   - `controls.rs`: Mapeamento de entradas de teclado, joystick e sobreposição visual de analógico virtual na tela.
   - `audio.rs`: Sintetizador de áudio procedural para impactos, super movimentos e reprodução de WAVs.

---

## Formato de Assets

### 1. Formato MUGEN Clássico
Suporta personagens e cenários MUGEN clássicos.
- **Localização dos arquivos no Desktop**: Coloque a pasta ou arquivo `.zip` do personagem dentro do diretório `mugen/chars/` (por exemplo: `mugen/chars/kfm/kfm.def`) ou configure a variável de ambiente `UME_MUGEN_DIR`.
- **Cenários**: Coloque os arquivos `.def` e `.sff` em `mugen/stages/`.
- **Importação na Web**: Arraste um arquivo `.zip` contendo o personagem ou cenário para dentro da janela do navegador.

### 2. Formato Nativo UMF (JSON)
Lutadores UMF ficam armazenados em `assets/chars/<nome>.json`. Em vez de centenas de sprites manuais e arquivos de texto complexos, um personagem UMF é descrito declarativamente:
- **Metadados**: Nome, resolução de coordenadas, atributos base (vida, velocidade, pulo).
- **Paletas**: Cores hexadecimais para roupas, pele, cabelos e detalhes.
- **Golpes e Animações**:
  - Golpes simples: `{"name": "punch", "type": "attack", "damage": 40, "hit": {"part": "handR"}}`.
  - O motor gera automaticamente as caixas `Clsn1` e `Clsn2` a partir dos membros do esqueleto.
Consulte o guia completo em [`docs/CHARACTER_FORMAT.md`](docs/CHARACTER_FORMAT.md).

### 3. Áudio e Fontes
- **Fontes**: Tipografias vetoriais de código aberto *Russo One* e *Rajdhani* sob licença SIL Open Font License (OFL), embutidas em `assets/fonts/`.
- **Efeitos de Som**: Efeitos sintetizados em tempo real e arquivos `.wav` estéreo/mono PCM (8-bit e 16-bit) suportados dentro de containers `.snd`.

---

## Como Contribuir

Contribuições são muito bem-vindas! Para garantir estabilidade e qualidade, siga os passos abaixo:

1. **Testes Unitários e de Integração**:
   Antes de abrir um pull request, garanta que todos os testes passem:
   ```bash
   cargo test --workspace
   ```
2. **Compatibilidade com WebAssembly**:
   Certifique-se de que nenhum código adicionado quebre a compilação para navegadores:
   ```bash
   cargo check --target wasm32-unknown-unknown
   ```
3. **Formatação e Linters**:
   Mantenha o código formatado e livre de advertências severas:
   ```bash
   cargo fmt --check
   cargo clippy --workspace --tests
   ```
4. **Adicionando Novos Lutadores**:
   - Crie o arquivo `<nome>.json` em `assets/chars/`.
   - Adicione o novo lutador na lista de elenco em `src/registry.rs` e no teste de fumaça em `crates/engine/tests/roster.rs`.
5. **Commits semânticos** (geram os releases, veja abaixo): `feat: …`, `fix: …`, `perf: …`; `feat!:` ou rodapé `BREAKING CHANGE:` para quebra de compatibilidade. `docs`, `chore`, `ci`, `test`, `refactor`, `style`, `build` não geram release. O título do PR (squash) **ou** todos os commits devem seguir o padrão — o check *Conventional commits* valida.

---

## Releases e Atualização Automática

A **tag git é a fonte da verdade da versão** (o `version` do `Cargo.toml` é só a base antes da primeira tag). A cada push na `master`, o workflow `Release` (`.github/workflows/release.yml`):

1. **Planeja** (`scripts/release-plan.mjs`): lê os commits desde a última tag `vX.Y.Z` — `feat` → minor, `fix`/`perf` → patch, `!`/`BREAKING CHANGE` → major (antes da 1.0, major vira minor; a 1.0.0 é uma decisão explícita). Só `docs/chore/ci/…` ⇒ nenhum release.
2. **Compila** em Linux, Windows e macOS (`cargo build --release --locked`) com a versão carimbada via `UME_RELEASE_VERSION` e confere `ultimate-mugen --version`.
3. **Publica** como *draft*: sobe os pacotes + `latest.json` (URLs e SHA-256) + `SHA256SUMS.txt`, **verifica** o draft contra o manifesto e só então publica — o que cria a tag `vX.Y.Z` no commit planejado. Reexecutar retoma o draft (idempotente); uma tag publicada nunca é sobrescrita.
4. **Health check**: confirma que `releases/latest/download/latest.json` já expõe a nova versão.

Primeiro release (histórico antigo sem commits semânticos) ou release forçado: *Actions → Release → Run workflow* e escolha `bump`. **Rollback:** `gh release delete vX.Y.Z --cleanup-tag --yes`.

**No jogo (desktop):** ao abrir, uma verificação em segundo plano lê o `latest.json` público (HTTPS apenas, timeout de 8 s, falha silenciosa offline) e, havendo versão mais nova, mostra o aviso na tela de título. É **somente notificação** — nada é baixado nem executado automaticamente; o pacote vem da página de releases e pode ser conferido com o SHA-256 do manifesto. Desative com `--no-update-check` ou `UME_NO_UPDATE_CHECK=1`; builds locais (`…-dev`) não verificam. Diagnóstico sem janela: `ultimate-mugen --version` e `ultimate-mugen --check-update`.

> Instalação automática exigiria assinar os pacotes (minisign/certificado) e embutir a chave pública no jogo; hoje o manifesto garante integridade, não autoria.

---

## Documentação Adicional

- **[Wiki](docs/wiki/pt-BR/Home.md)**: instalar, configurar, criar personagens UMF, scripts CNS, cenários e customizar a engine.

- [`docs/PESQUISA.md`](docs/PESQUISA.md) — Pesquisa aprofundada sobre a engenharia reversa do M.U.G.E.N, comparação com o Ikemen GO e desafios arquiteturais.
- [`docs/CHARACTER_FORMAT.md`](docs/CHARACTER_FORMAT.md) — Manual do formato nativo UMF (criação de golpes, poses e esqueletos vetoriais).
- [`docs/ROSTER.md`](docs/ROSTER.md) — Guia do elenco de lutadores nativos e catálogo de personagens compatíveis.
- [`docs/ROADMAP.pt-BR.md`](docs/ROADMAP.pt-BR.md) — Roadmap de desenvolvimento com as próximas funcionalidades planejadas.
- [`docs/research-mugen-en.md`](docs/research-mugen-en.md) — Relatório técnico completo de especificação de formatos binários em inglês.

---

## Agradecimentos

Obrigado à equipe do [Ikemen GO](https://github.com/ikemen-engine/Ikemen-GO) e à Elecbyte, criadora do M.U.G.E.N, pela inspiração.

## Aviso Legal

Personagens protegidos por direitos autorais pertencentes a terceiros (franquias como Dragon Ball, Naruto, Street Fighter, etc.) **não são incluídos nem distribuídos** com o executável deste motor. O Ultimate MUGEN é um interpretador de motor de jogo de código aberto capaz de carregar e renderizar conteúdos MUGEN importados pelos próprios usuários.
