# Desempenho

[English](../Performance.md) · **Português (BR)**

Como a engine mantém o carregamento rápido e os quadros estáveis, e como medir.

## Carregamento

| Etapa | O que acontece | Threads |
|---|---|---|
| Decodificação SFF v2 | Os cabeçalhos são lidos em ordem; os dados LZ5, RLE5, RLE8 e PNG são decodificados em paralelo; sprites ligados compartilham os pixels de origem (`Arc`) | Todos os núcleos (máx. 8) |
| Sprites do personagem | A conversão paleta para RGBA roda em paralelo, uma vez por buffer de pixels e paleta únicos; sprites ligados compartilham um `ImageData` e, portanto, um único upload na GPU | Todos os núcleos (máx. 8) |
| Sprites do cenário | Mesma conversão RGBA em paralelo; o buffer RGBA é movido para o cenário, nunca copiado | Todos os núcleos (máx. 8) |
| Retratos da tela de seleção | `load_mugen_portrait` decodifica só os sprites 9000,0, 9000,1 e o primeiro quadro parado (mais as origens dos links) via `decode_sff_only` | 1 |
| Rig nativo (UMF) | Cada quadro único é rasterizado em paralelo | Todos os núcleos (máx. 8) |
| Personagem inteiro | Carregado numa thread `char-load` enquanto a tela de VS aparece | 1 thread em segundo plano |

O auxiliar é `ume_formats::par::map`. Na web (`wasm32`) ele roda na própria thread, pois o build do navegador não tem threads.

## Quadros estáveis

| Técnica | Onde | Efeito |
|---|---|---|
| Ritmo de quadros | `ume_shell::Pacer` | Tempos de quadro a até 4% de uma taxa de atualização comum (30 a 240 Hz) viram o período exato; a sobra é devolvida quando soma um período inteiro, então o tempo de jogo segue o tempo real. Com vsync, isso dá exatamente um tick de 60 Hz por quadro, sem alternar entre 0 e 2 |
| Passo fixo | `Session::update` | Simulação a 60 Hz com tolerância mínima e no máximo 5 ticks por quadro (sem espiral de atraso) |
| Pré-carga de texturas | `Gfx::prefetch`, `Texture2D::prefetch` | Os sprites dos lutadores sobem para o atlas no início da luta (até 50% do atlas), então o primeiro especial não faz upload no meio da partida |
| Pré-aquecimento de sons | `Audio::preload_match` | Todos os sons dos lutadores são decodificados em paralelo na tela de VS; sons de personagens que saíram da partida são liberados |

## Memória de GPU

| Técnica | Onde | Efeito |
|---|---|---|
| Paleta no shader | `Gfx::sprite_tex`, `IndexedTexture`, `Palette` | Sprites M.U.G.E.N com pixels indexados vão para o atlas de índices de 8 bits e são coloridos por uma linha de paleta no shader: 4x menos upload e espaço de atlas que RGBA. `RemapPal` troca a linha de paleta em vez de gerar uma nova imagem RGBA |
| Reinício separado dos atlas | `ume_shell::quad` | O atlas RGBA (UI, texto, sprites true-color) e o atlas de índices (+ linhas de paleta) reiniciam cada um por conta própria quando enchem, então encher um nunca obriga o outro a refazer uploads |

## Medindo

| Comando | Mede |
|---|---|
| `cargo run --release -p ume-engine --example mugen_load_bench [zip] [execuções]` | Tempo de carga de personagem M.U.G.E.N (padrão: `elecbyte.zip` embutido) |
| `cargo run --release -p ume-engine --example load_bench` | Tempo de compilação nativa (UMF) |
| `cargo test -p ume-engine --release --test tick_alloc` | Zero alocações por tick de simulação |

Referência (4 núcleos, release): `kfm720` caiu de 85 ms para 27 ms, `kfm` de 12 ms para 7 ms.

## Web (wasm)

| Técnica | Onde | Efeito |
|---|---|---|
| `ControlFlow::Wait` na web | `ume_shell::run` | O `Poll` do winit gira um laço `postTask`/`AbortController` ao lado do `requestAnimationFrame`; o profile mostrou cerca de 75% do tempo da main thread (mais GC) gasto aí. Os redraws são auto-solicitados, então `Wait` basta |
| Zerar atlas | `Renderer::new` | Texturas de atlas e paleta são inicializadas uma vez, então `texSubImage2D` não força mais limpeza lazy (aviso do Firefox) |
| Retry da surface | `Shell::redraw` | Falha em `get_current_texture` pede novo frame em vez de travar o laço (tela preta) |
| Loader + cache | `web/index.html`, `web/sw.js` | Progresso do download do wasm, JS e wasm buscados como `?v=<build>` (carimbado pelo `build-web.sh`) e cacheados de forma imutável, então o cache nunca mistura dois builds e recarga automática se o contexto WebGL for perdido |

Sonda: `NODE_PATH=$(npm root -g) node scripts/web-profile.js Enter Enter Enter` (Chromium headless, rode `./scripts/build-web.sh` antes).

## Carregamento de cenários no navegador

O WASM do navegador embute miniaturas de cenários de 240×135, em vez dos sete PNGs completos (21.858.090 bytes). A grade de seleção decodifica apenas as miniaturas solicitadas. Confirmar um cenário baixa `assets/stages/<id>.png` durante o carregamento da tela VS; erros HTTP e um limite de 30 segundos retornam ao menu. O cache em Rust do navegador retém apenas a última ilustração completa preparada; uma luta ativa ainda pode manter sua própria referência. O cache HTTP/service worker do navegador pode reutilizar arquivos já baixados.

A versão nativa mantém as imagens completas embutidas, decodificando cada cenário separadamente no primeiro uso. As definições UMF e o pacote Elecbyte continuam dentro do WASM; esta alteração separa apenas a arte dos cenários. A compilação dos lutadores e a preparação de áudio continuam na thread principal do navegador.

`scripts/build-web.sh` copia os PNGs externos e `assets.js` para `web/dist/`. Publique o diretório inteiro, incluindo `assets/stages/`. Jogar offline exige que o cenário escolhido tenha sido baixado anteriormente. Ao substituir a arte de um cenário, regenere sua miniatura em `assets/stages/thumbs/` como PNG RGB de 240×135 com `python scripts/generate-stage-thumbnails.py` (exige Pillow).

Execute `node scripts/web-smoke.js web/dist` após compilar a versão web (exige Playwright e Chromium). O teste rejeita downloads de cenários completos antes da confirmação e exige exatamente um na primeira luta. Isso verifica o escopo da transferência, não um percentil de latência nem um ganho de FPS.

## Orçamento da thread principal no carregamento

Regra: a thread de render nunca espera disco, decodificação, parsing ou síntese. No nativo esse trabalho roda em threads de trabalho e a UI consulta a conclusão; no wasm (thread única) continua fatiado em um passo por quadro.

| Trabalho | Onde roda agora |
|---|---|
| Preview e retrato MUGEN na seleção | `CharEntry::stream_preview` / `stream_portrait` (máx. 2 workers, coletados por `Registry::reap_streams`). Chamada ~0,1 ms contra 28 ms (KFM) e 84 ms (KFM HD) síncronos |
| Importação de `.zip` (hash + descompressão) | `registry::read_zip` em worker; só `finish_import` (~4 ms) roda na thread principal |
| Decodificação dos sons da luta | `Audio::preload_match` em worker; a barra do VS espera |
| Síntese da música do cenário (17-31 ms) | `Audio::music_ready` em worker |
| Tela de vitória | usa a definição em cache (`peek`), nunca carrega |

Watchdog de quadros: `UME_WATCHDOG=1` registra todo quadro acima de `UME_HITCH_MS` (padrão 50) com o rótulo ativo (`ume_shell::watchdog::label("load:stage")`) e, ao sair, um resumo com o pior quadro.
