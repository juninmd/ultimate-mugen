# Engenharia reversa de um port de M.U.G.E.N para o navegador

**Português (BR)** · [English](../Ikemen-Web-Reverse-Engineering.md)

O que <https://w8.snokido.com/games/html5/dbz-vs-marvel-vs-capcom/> faz para rodar um elenco de 78 lutadores de M.U.G.E.N no navegador, como seu sistema de animação funciona, por que ele segura 60 fps, e o que disso devemos copiar — ou não — no Ultimate MUGEN.

Tudo abaixo foi medido ou lido dos artefatos publicados. Os scripts que produziram os números estão em [`docs/research/ikemen-web/`](../../research/ikemen-web/), então cada medição pode ser refeita.

## 1. O que a página é

A página pública é uma casca de 11 KB: menus, seleção de personagem, tela de versus e opções são DOM comum (essa parte é a camada de UI `runtime-ui.js` da snokido). A luta em si é um documento separado, `frame.html`, dentro de um iframe — e o motor ali **não** é JavaScript: é um programa Go compilado para WebAssembly.

| Artefato | Tamanho | Papel |
|---|---|---|
| `index.html` | 11 KB | casca: boot, menu, seleção, opções |
| `local-runtime.js` | 342 KB | bootstrap ofuscado: guarda o glue `wasm_exec` do Go e o módulo do frame **como strings**, além do catálogo de conteúdo |
| `runtime-ui.js` | 420 KB | a UI em DOM (ofuscada, protegida por digest) |
| `frame.html` | 2,2 KB | hospeda o motor: um `<canvas id="ikemen-canvas">` |
| `runtime-engine.bin` | 5.887.387 B | gzip de um módulo WebAssembly de **25.343.508 B** |
| `content/catalog.shvd.json` | 265 KB | 78 personagens, 74 cenários, URLs dos pacotes, sha256, tamanhos |
| `content/packs/*.zip` | 1–24 MB cada | arquivos M.U.G.E.N sem modificação, um zip por personagem/cenário |

O bootstrap envia `{frameJs, wasmExec, descriptor, gzipBytes}` ao `frame.html`, que instala o `wasm_exec.js` a partir de uma string, importa o módulo do motor de uma Blob URL e o condiciona a um SHA-256 dos valores recebidos. Isso é proteção contra cópia, não projeto de motor.

```console
$ node docs/research/ikemen-web/wasm-artifacts.js runtime-engine.bin
module bytes: 25343508
[imports "gojs"] 25
  runtime.scheduleTimeoutEvent:function
  runtime.wasmExit:function
  syscall/js.valueGet:function
  syscall/js.copyBytesToJS:function
  mem:memory
[exports] run:function resume:function getsp:function mem:memory
[build info] ikemen-GO v0.0.0-20260927200105-cae6cd63ba4a+dirty
[go source files named by the symbol table] ... render_webgl.go render_webgl_cmdbuf.go system_js.go audio_js.go input_js.go loadyield_js.go ...
```

Imports `gojs` mais as exportações `run`/`resume`/`getsp` são a ABI WebAssembly do runtime Go, e a tabela de símbolos Go preserva o caminho de cada arquivo-fonte. O motor é o **Ikemen-GO** (a reimplementação em Go do M.U.G.E.N) com um backend WebGL2 adicionado para o navegador: `render_webgl.go`, `render_webgl_cmdbuf.go`, `system_js.go`, `audio_js.go`, `input_js.go`, `pause_js.go`, `platform_js.go`, `loadyield_js.go`, `font_webgl.go`, `netplay_js.go`.

O catálogo declara a própria proveniência, e foi assim que o código do backend foi localizado e conferido contra o binário:

```json
"source": { "components": [
  { "name": "ikemen-go-web",      "url": "https://github.com/energyjp/ikemen-go-web",             "revision": "54305ec917c3e995f433004e2e8f7c8902bff96a" },
  { "name": "Ikemen-GO-Screenpack","url": "https://github.com/ikemen-engine/Ikemen-GO-Screenpack", "revision": "87f348a5f6e01e878a62ae865c6ee7d3281f8c98" } ] }
```

O `ikemen-engine/Ikemen-GO` original não tem backend WebGL; o `energyjp/ikemen-go-web` tem. O código conferido bate com o binário exatamente (tabela de opcodes, GLSL, nomes de símbolos, id do canvas), então o "como" abaixo está verificado nas duas fontes.

## 2. Conteúdo: nenhuma recodificação, só zips

Os pacotes contêm os arquivos M.U.G.E.N originais — `.def`, `.sff`, `.snd`, `.air`, `.cns`, `.zss`, `.act` — intactos. Verificado no pacote do Goku (24.319.368 B, sha256 igual ao do catálogo): `files/goku.sff` 9,6 MB, `files/goku-jap-en-bl.snd` 20,7 MB, `files/goku.air` 287 KB, 21 arquivos de paleta `.png`/`.act`, além do movelist HTML do próprio personagem.

Ou seja: o navegador baixa **conteúdo M.U.G.E.N cru** e o motor em WASM interpreta SFF/SND sozinho. Nada é pré-renderizado em spritesheets, e só a mídia do *front-end* (retratos, ícones, miniaturas de cenário) vem pré-codificada em WebP com nome endereçado por conteúdo (`collections/shvd/media/<sha256>.webp`).

O `core` (7,0 MB zip → 9,7 MB) é o screenpack e os dados de sistema, e todo outro pacote declara `"dependencies": ["core"]`. Uma luta carrega: `core` + um pacote por lutador + um por cenário.

## 3. Animação: semântica de M.U.G.E.N, sem mudanças

As animações são arquivos `.air` interpretados nas mesmas estruturas que o M.U.G.E.N usa (`src/anim.go`):

| Estrutura | Campos que importam |
|---|---|
| `Animation` | `frames`, `loopstart`, `curtime`, `curelem`, `totaltime`, `looptime`, `interpolate_offset/scale/angle/blend`, `mask`, `transType`, `srcAlpha`/`dstAlpha` |
| `AnimFrame` | `Group`, `Number`, `Xoffset`, `Yoffset`, `Time` (ticks), `Xscale`/`Yscale`, `Angle`, `TransType`, `Clsn1`/`Clsn2` |
| `Anim` (instância) | `anim`, `x`/`y`, `xscl`/`yscl`, `window`, `xshear`, `rot`, `palfx`, `layerno`, `localcoord` |

O caminho por tick é deliberadamente barato:

1. `Animation.Action()` avança `curtime`; em `totaltime` salta para `loopstart` e marca `newframe`.
2. `Animation.UpdateSprite()` roda em todo tick mas só faz trabalho quando `newframe` está marcado — é ali que o sprite é resolvido, com uma consulta de mapa: `a.spr = a.sff.GetSprite(uint16(group), uint16(number))` (`anim.go:702`), depois de aplicar os presets `Remap` do personagem.
3. O desenho lê `frames[drawidx]` para a transformação e o `spr` em cache para a textura. Nada é reconsultado por tick e nada é interpolado.

Portanto **não há tweening nem interpolação sub-tick**: o quadro muda no tick de 60 Hz, exatamente como no M.U.G.E.N. A suavidade percebida vem da taxa de ticks, dos blocos `interpolate offset/scale/angle/blend` (que animam transformações *entre* quadros do AIR) e do shader fazer todo o trabalho de cor em uma passada.

### Cor é recurso de shader, não bake

Duas coisas que normalmente custariam upload vão para o fragment shader (`src/shaders/sprite.frag.glsl`, que também está embutido no WASM como string):

```glsl
c = COMPAT_TEXTURE(tex, uv);
if (isRgba) { ... } else {
    c = COMPAT_TEXTURE(pal, vec2(c.r*0.9966, 0.5));   // paleta lida do canal vermelho
}
if (hue  != 0.0) c.rgb = hue_shift(c.rgb, hue);
if (neg)         c.rgb = neg_base - c.rgb;
c.rgb = mix(vec3((c.r+g.c+b)/3.0), c.rgb, 1.0 - gray);
c.rgb += final_add;
c *= final_mul;
if (!isFlat) c.rgb = mix(c.rgb, tint.rgb * c.a, tint.a);
```

* **Paletas**: um sprite indexado mantém os pixels de 8 bits como textura e é colorido por uma textura de paleta `256×1` (`newPaletteTexture()` → `newTexture(256, 1, 32, false)`). Trocar `pal1…pal12` ou `RemapPal` reenvia **1 KB**, não o sprite. `Sprite::CachePalTex` (`image.go:1336`) compara as 256 palavras de cor e só reenvia quando muda, então uma troca de paleta custa um `texSubImage2D` por sprite que realmente mudou.
* **`palFX`** (`add`, `mul`, `color`, `hue`, `invertall`) são uniforms por sprite no mesmo draw — sem segunda passada, sem troca de shader por `RenderMode`.
* **Distorção trapezoidal** (sombras de chão / skew de `xscale`) é calculada no mesmo shader a partir de `gl_FragCoord`, não reprojetando vértices.

Um shader cobre quadrados chapados, sprites RGBA, sprites indexados, trapézios e toda a cadeia de `palFX`. O problema de "mais um shader por efeito" do M.U.G.E.N simplesmente não existe ali.

## 4. Como ele se mantém rápido

A parte interessante. Medido na build publicada (Chromium headless, ANGLE/SwiftShader, ou seja, números de GPU renderizados em software, 1280×720 CSS, duas execuções numa máquina carregada):

| Métrica | Valor |
|---|---|
| Backing store do canvas | **640×480** (CSS 1280×720, `object-fit: contain`) |
| `drawArrays` por rAF | p50 **82–105**, p95 113–163, máx 172–224 |
| `bufferData` por rAF | 82–125 (um upload pequeno por sprite; sequências de texto compartilham um) |
| Chamadas de uniform por rAF | 322–397 |
| `useProgram` por segundo | 30–76 (poucas por quadro) |
| `bindTexture` por segundo | 1,4 mil–6,2 mil |
| Quadros do motor por rAF | **0,78–0,89** (60 Hz fixo, com portão) |
| Heap JS em uma luta | 9–74 MB |
| Boot → menu | 1,9 s |
| Menu → seleção | 4,0 s |
| Seleção → primeira luta | **37 s** (core + 2 pacotes de lutador + cenário pela rede, depois decode SFF) |
| Long tasks durante a luta | 50–90 ms, recorrentes (decode de assets / GC na thread principal) |

### 4.1 A cena é renderizada em 640×480 numa FBO e depois sofre um blit

`Init()` cria uma textura RGBA + um renderbuffer `DEPTH_COMPONENT24` no tamanho `sys.scrrect` (640×480) e desenha a cena inteira ali; `EndFrame()` roda os shaders de pós-processamento externos em duas FBOs ping-pong e só então faz **um** `drawArrays(TRIANGLE_STRIP, 0, 4)` para o framebuffer padrão, com viewport letterboxed e `NEAREST` (ou `LINEAR`, se o usuário escolheu escala suavizada). Todo sprite é rasterizado em 640×480, independente do tamanho da janela.

### 4.2 Uma travessia JS por quadro em vez de uma por chamada GL

É o truque principal (`render_webgl_cmdbuf.go`). As chamadas GL não saem do Go; elas são **codificadas num fluxo de bytes**:

```go
// GL command buffer: batches the browser build's per-frame WebGL commands into a shared ArrayBuffer
// that a JS-side interpreter drains in ONE syscall/js crossing per flush, instead of one crossing
// per GL command.
//
// Why: profiling the shipped (v0.9) build showed hundreds of `gl.Call` crossings per frame; each
// crossing boxes its arguments through syscall/js.makeValue (~450MB/30s of JS garbage) and pays the
// wasm->JS context switch.
```

* Buffer de staging de 256 KB (`wglCmdBufBytes = 1 << 18`), 23 opcodes (`useProgram`, `uniform*`, `activeTexture+bindTexture`, `bufferData`, `drawArrays`, `enable/disable`, `blendFunc`, `scissor`, `depthMask`, `bindVertexArray`, `texParameteri`, …).
* Objetos GL (texturas, programas, locations de uniform, VAOs, buffers) são registrados uma vez num array JS e referenciados por handle inteiro.
* `flushCmds()` faz um `js.CopyBytesToJS` + um `Invoke` de um interpretador JS criado com o construtor `Function`.
* Toda chamada GL direta passa por `glc()`, que descarrega o buffer antes, então a ordem dos comandos é preservada por construção.

### 4.3 Deduplicação de uniforms e de estado no lado Go

`SetUniformISub`/`SetUniformFSub` mantêm caches por programa com chave `(programID << 16) | uniformIndex` e não emitem nada quando o valor não mudou. `EnableBlending`, `SetDepthTest`, `SetDepthMask`, `EnableScissor` e `ChangeProgram` fazem o mesmo para estado GL. É por isso que ~14 uniforms por sprite viram ~4 na média, e por que `useProgram` aparece só 30–50 vezes *por segundo*.

### 4.4 Zero alocação no caminho de desenho

O mesmo arquivo documenta o custo medido da versão ingênua: passar um `[16]float32` local para um método de interface fazia o array escapar, alocando vértices + uniform + modelview **por quad**, e só `drawQuads` alocava **>1 GB a cada 30 s** de heap. A correção são arrays de scratch no escopo do pacote, reusados por chamada (o caminho de render é single-thread e cada buffer é consumido de forma síncrona).

### 4.5 Portão de rAF na taxa do jogo

`system_js.go` não roda um quadro por `requestAnimationFrame`; roda no máximo um a cada `1000/Video.Framerate`:

```go
// Browsers fire rAF at the DISPLAY's refresh rate - 120/144/240Hz on high-refresh screens. ...
// Releasing a frame on every rAF therefore runs the whole loop 2-4x more often than the game is
// authored for, multiplying allocation and forcing far more frequent GC pauses (the 200-500ms
// mark stalls high-refresh users saw) for zero visual benefit.
```

`SwapBuffers()` então estaciona a goroutine principal num canal até o próximo quadro liberado — o que também devolve o controle ao event loop JS para input, rede e áudio.

### 4.6 Sprites não vão para atlas; fontes vão

Cada sprite mantém a própria textura. Não existe atlas de sprite nem política de despejo, então uma luta nunca bate num "atlas cheio". As 16 unidades de textura são usadas como **cache LRU** (`texCacheTexSerial`/`texCacheLastUsed`), para que sprites consecutivos da mesma folha não re-vincullem.

Só **texto** usa `TextureAtlas`: 256×256, empacotamento skyline com extrusão de borda, e quads de glifo agrupados por atlas, então um trecho de texto é um único draw.

### 4.7 Um draw call por sprite, por projeto

`SetVertexData(...16 floats...)` + `RenderQuad()` → `bufferData(STATIC_DRAW)` seguido de `drawArrays(TRIANGLE_STRIP, 0, 4)`. Por sprite: 1 `bufferData`, 1 `drawArrays`, ~4 uniforms, ≤1 bind de textura — tudo dentro do command buffer, então o lado Go paga só alguns `emitI`/`emitF` por sprite. Com 82–172 sprites por quadro isso dá 5–10 mil chamadas GL/s, que é exatamente o que um rasterizador em software no Chromium headless aguenta.

## 5. O que já fazemos melhor

O Ultimate MUGEN não precisa do command buffer: nossa shell fala wgpu, que já empacota um quadro em poucas submissões. E nosso caminho de sprite está arquiteturalmente à frente do deles:

| Aspecto | Build web do Ikemen | Ultimate MUGEN |
|---|---|---|
| Draw calls | 1 por sprite (82–172/quadro) | agrupado: quads fundidos por estado de blend + scissor num único buffer de vértices/índices |
| Estado de bind | bind por sprite, LRU de 16 unidades | um bind group; atlas + textura de paleta vinculados uma vez |
| Paleta | textura `256×1` por sprite, upload por sprite quando muda | 1024 linhas de paleta numa textura, índice de linha por vértice, `RemapPal` troca a linha |
| Atlas | nenhum para sprites (textura por sprite) | atlas RGBA + atlas de índices de 8 bits, com reset independente |
| Simulação | máquina de estados inteira em WASM | `hotpath_bench` (abaixo) |
| Ritmo de quadros | portão de rAF em `Video.Framerate` | `ume_shell::Pacer` + passo fixo de 60 Hz (≤5 ticks/quadro) |

Nosso custo de simulação, para deixar claro que o orçamento de quadro não é o problema:

```console
$ cargo run --release -p ume-engine --example hotpath_bench
sff decode                   mean      829.6 us
air parse                    mean      493.5 us
cns parse+compile            mean     3680.8 us
char load (no images)        mean     7388.2 us
match tick (2 fighters)      mean        4.2 us   p99  9.0
match tick (6 fighters)      mean       11.3 us   p99 17.6
match tick (10 fighters)     mean       20.1 us   p99 33.5
```

4–20 µs por tick contra um quadro de 16,7 ms: 0,03–0,12 %. A luta é gasta no renderizador e no carregamento — exatamente onde o port para navegador também gastou o esforço dele.

## 6. O que mudamos

Os relatos de bug do próprio port (`>1 GB/30 s` de GC por alocação por quad, `450 MB/30 s` de boxing por chamada, stalls de 200–500 ms por simular demais a 144 Hz) têm todos a mesma forma: **o trabalho de uma luta inteira pago dentro de um quadro só**. Tínhamos a mesma forma num lugar: o aquecimento de sprites.

`App::finish_fight` subia os sprites dos lutadores no único quadro que trocava para a luta:

```rust
self.g.trim(4000);
for f in &sess.m.fighters { if !self.g.prefetch(&f.def, 0.5) { break; } }
```

Quanta arte um lutador carrega (exemplo novo `sprite_mem`):

```console
$ cargo run --release -p ume-engine --example sprite_mem -- mugen/chars/BuuZ2 mugen/chars/cell mugen/chars/frieza
mugen:c/buuz2.def   sprites=2510  indexed=2510 | index atlas  33.71 MB | total  33.86 MB | largest spr 5202 KB
mugen:c/cell.def    sprites=631   indexed=631  | index atlas   9.91 MB | total   9.99 MB | largest spr  818 KB
mugen:c/frieza.def  sprites=673   indexed=673  | index atlas   8.67 MB | total   8.74 MB | largest spr  682 KB
```

Um par de DBZ é 43,6 MB de folhas indexadas em 3.141 sprites, e o maior sprite sozinho tem 1,3 MB.

### 6.1 Pagar o aquecimento por quadro, não por partida

`App::warm_step` + `Gfx::prefetch_step`: um passo de carregamento próprio que copia no máximo `WARM_BUDGET` = 512 KiB por chamada. Atrás da tela de VS é um pedaço por quadro (a splash dura no mínimo 1,8 s), e o caminho bloqueante de `start_fight` (revanche pelo menu de pausa ou pela tela de resultados, início de netplay, tours roteirizados) agora cede um quadro entre os pedaços em vez de congelar.

Medido na mesma máquina, mesmo tour, mesmos lutadores, com `UME_WATCHDOG=1`:

| | Quadro danoso |
|---|---|
| antes | `212.8 ms em 'fight:prefetch'` (um quadro) |
| depois | `2.3 / 2.4 / 2.5 / 2.1 / 8.8 ms em 'load:sprites'` (um pedaço por quadro) |

Uma revanche com o mesmo elenco não limpa mais o cache de texturas (ver 6.3), então o aquecimento dela não copia nada.

### 6.2 Linhas de paleta são compartilhadas por conteúdo

`ume_render::PaletteRows` mantém uma linha por paleta de 256 cores *distinta* em vez de uma por objeto de paleta: espelho, `RemapPal` que cai numa paleta existente e um novo aquecimento depois de um reset do atlas reusam a linha e pulam o upload. Testado em `crates/render/src/palette.rs` (50 consultas às mesmas cores → uma linha; atlas cheio reporta falha em vez de sobrescrever uma linha em uso).

### 6.3 Revanche mantém os atlas já preenchidos

O aquecimento tem uma chave de elenco (`def.id` dos dois lutadores). Se a chave é a mesma do último aquecimento, `Gfx::trim` é pulado, a geração do atlas não muda e `prefetch_step` encontra tudo residente: a segunda luta de uma maratona começa com zero uploads.

### 6.4 Orçamento de draw calls que quebra o build

`ume_render::DRAW_CALL_BUDGET = 24` com `batch::tests::a_fight_screen_stays_within_the_draw_call_budget`: um quadro de luta sintético (camadas de cenário, passada subtrativa de reflexo, dois lutadores, faíscas aditivas, HUD de duas regiões) precisa ficar abaixo do teto, e `a_flat_run_of_quads_is_one_call` fixa a propriedade "300 quads em uma chamada". Quebre o agrupamento e o CI acusa.

### 6.5 Rótulos de hitch apontando para a fase certa

`ume_shell::watchdog::label` agora fica preso à primeira fase nomeada no quadro. Antes, o `label("import")` no fim de `App::frame` sobrescrevia tudo, então um passo de carga lento aparecia como importação; é por isso que os números de `load:sprites` acima são visíveis. Testado.

### 6.6 O que os números também mostraram

`prefetch_step` registra onde parou: para um lutador nativo (RGBA em alta resolução) o teto de 50 % do atlas cabe **8 dos 116 sprites**, e então o aquecimento desiste e o resto entra sob demanda durante a luta — um upload no momento do desenho, que é o que o port faz com todos os sprites. Para arte indexada de DBZ o mesmo teto cabe algumas centenas. Ou seja, o aquecimento em bloco é limitado pelo atlas, não pelo elenco, e o próximo passo (backlog 5) é dar um orçamento de upload por quadro ao caminho sob demanda em vez de tentar pré-carregar mais.

## 7. Backlog, por prioridade

| # | Ideia tirada do port | Estado aqui | Como verificar |
|---|---|---|---|
| 1 | Nunca pagar uma luta inteira dentro de um quadro | feito: `App::warm_step` + `Gfx::prefetch_step`, 512 KiB por quadro, tanto no passo de VS quanto em `start_fight` | `UME_WATCHDOG=1` rótulo `load:sprites`; F3 `up`/`KB` |
| 2 | Manter o quadro de transição livre nos caminhos sem tela de VS | feito: `start_fight` cede um quadro por pedaço | pior quadro do `UME_WATCHDOG=1` na transição |
| 3 | Orçamento de draw calls é número de primeira classe | feito: `DRAW_CALL_BUDGET` + teste do batch | `cargo test -p ume-render` |
| 4 | Linhas de paleta são recurso escasso | feito para duplicatas: `PaletteRows` compartilha por conteúdo | `cargo test -p ume-render` |
| 5 | Sprites nunca vão para atlas na build web | **não copiado** (nosso atlas é o motivo de desenharmos em poucos draws). Próximo passo: orçamento de upload por quadro no caminho sob demanda, porque um teto de 50 % cabe só 8 sprites nativos | `atlas {}` no overlay F3 |
| 6 | Um shader para todo efeito de sprite | já é verdade (`sprite.wgsl`: flat/RGBA/indexado/palFX/tint numa passada) | `cargo test -p ume-render --test shader` |
| 7 | Ceder controle ao decodificar | `load:character` já roda um personagem por passo; manter decoders novos dentro desse orçamento | pior quadro de carregamento no `UME_WATCHDOG=1` |

## 7b. Somos melhores que eles?

Nas coisas que esta investigação mediu, sim — e nas que não mediu, não. Ser preciso importa mais que o título.

Melhores:

| | Port para navegador | Ultimate MUGEN |
|---|---|---|
| Draw calls no mesmo tipo de cena | um por sprite, 82–105 por quadro | fundidos por blend + scissor, teto 24, garantido por teste |
| Troca de paleta | upload de 1 KB por sprite que muda de paleta | linhas de paleta deduplicadas por conteúdo, compartilhadas entre lutadores |
| Custo de simulação | máquina de estados inteira numa thread WASM | 4–20 µs por tick (0,03–0,12 % de um quadro) |
| Aquecimento / transição | uploads espalhados por sorte (textura por sprite) | orçamento explícito por quadro, revanche não copia nada |
| Observabilidade de hitch | nenhuma | `UME_WATCHDOG=1` com rótulo por fase, F3 com uploads/KB e ocupação do atlas |
| Licenciamento/portabilidade do conteúdo | empacota personagens M.U.G.E.N de terceiros | carrega a pasta `mugen/` do jogador, UMF, ou o mercado |

Não melhores (ainda):

| | Port para navegador | Ultimate MUGEN |
|---|---|---|
| Entregue no navegador | sim, 78 lutadores, 74 cenários, netplay com rollback GGPO | a build wasm existe, mas o elenco vem em pacotes; sem catálogo equivalente |
| Volume de conteúdo | 78 lutadores, 74 cenários, packs sob demanda por luta | o que o jogador instalar |
| Alcance de dispositivos | celulares, tablets, desktop | desktop, build Android, build de navegador |

Resumo: um *motor* melhor nos eixos medidos e um *produto* menor que o deles. O PR a seguir implementa todos os itens da seção 7 que ainda estavam abertos.

## 8. Reproduzindo

```console
# artefatos, ordem das requisições, tamanhos, lista de frames
NODE_PATH="$(npm root -g)" node docs/research/ikemen-web/capture.js capture

# o que o módulo do motor importa/exporta e de quais arquivos Go ele foi construído
NODE_PATH="$(npm root -g)" node docs/research/ikemen-web/wasm-artifacts.js capture/runtime-engine.bin

# contadores WebGL por quadro, de dentro do frame.html
NODE_PATH="$(npm root -g)" node docs/research/ikemen-web/frame-stats.js
```

O diretório `capture/` passa a conter o motor gzipado, os bundles de UI, o catálogo e os pacotes de conteúdo usados acima. São necessários Playwright e Chromium; nada além disso vem de registro de pacotes de terceiros.

Duas ressalvas de medição: os números vêm do **Chromium headless com SwiftShader** (rasterizador em software), então trate os tempos de quadro como piso, não como resultado de GPU; e os 37 s de seleção→luta são dominados pelo download de ~50 MB de conteúdo nesse sandbox, não pela interpretação do motor.
