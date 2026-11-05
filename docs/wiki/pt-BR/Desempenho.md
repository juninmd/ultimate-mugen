# Desempenho

[English](../Performance.md) · **Português (BR)**

Como a engine mantém o carregamento rápido e os quadros estáveis, e como medir.

## Carregamento

| Etapa | O que acontece | Threads |
|---|---|---|
| Decodificação SFF v1 | Cabeçalhos de subarquivos escaneados; payloads PCX decodificados em paralelo entre núcleos; sprites ligados compartilham pixels de origem (`Arc`) | Todos os núcleos (máx. 32) |
| Decodificação SFF v2 | Os cabeçalhos são lidos em ordem; os dados LZ5, RLE5, RLE8 e PNG são decodificados em paralelo; sprites ligados compartilham os pixels de origem (`Arc`) | Todos os núcleos (máx. 32) |
| Importação e Descompressão Zip | Diretório central escaneado, entradas descompactadas em paralelo com barra visual de progresso `ZipProgress` e indicador de threads de hardware | Todos os núcleos (máx. 32) |
| Sprites do personagem | A conversão paleta para RGBA roda em paralelo, uma vez por buffer de pixels e paleta únicos; sprites ligados compartilham um `ImageData` e, portanto, um único upload na GPU | Todos os núcleos (máx. 8) |
| Sprites do cenário | Mesma conversão RGBA em paralelo; o buffer RGBA é movido para o cenário, nunca copiado | Todos os núcleos (máx. 8) |
| Cache e reuso de cenários | `src::registry` | Instâncias de `MugenStage` são cacheadas entre a prévia de seleção e o início da luta em `StageEntry`, reduzindo o carregamento do cenário de ~30 ms para 0,0 ms | Lazy init (1) |
| Parsing de cenário em passada única | `ume_formats::stage` | O `.def` do cenário é processado uma única vez com `parse_ini_no_lines` e compartilhado com a detecção de modelos 3D, eliminando passadas duplicadas e alocações de linhas | 1 |
| Cache de estados comuns | `ume_engine::common` | `common_def()` cacheado via `OnceLock<Arc<CharDef>>`, eliminando ~4 ms de parsing duplicado de `common1.cns` por partida | 1 (lazy init) |
| Parsing CNS e CMD em passada única | `ume_engine::loader`, `cns` | Seções processadas uma única vez com `parse_ini_no_lines` e compartilhadas entre constantes e estados; evita passadas redundantes de INI | 1 |
| Cache de bytecode de expressões e parâmetros | `ume_engine::chardef` | `safe_expr` e `Param::new` cacheiam AST e bytecode compilados para triggers e parâmetros idênticos via armazenamento thread-local | Por thread |
| Consultas INI sem alocação | `ume_formats::ini` | `Section::get` e `find_section` usam `eq_ignore_ascii_case` evitando alocações no heap; `strip_comment` atalha linhas sem comentário | Thread chamadora |
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
| VM de bytecode | `ume_engine::bytecode` | Expressões CNS/ZSS compilam para uma VM em pilha fixa (`[f64; 32]`) com redirecionamentos inline (`parent`, `root`, `helper`, `enemynear`), intrínsecos matemáticos, triggers de 0/1 argumento, `CmpStr`, `Special` (`animelem`, `timemod`) e atribuições `:=` |
| Caminhos críticos sem alocação | `commands`, `matchs`, `triggers`, `ai`, `ikemen` | Chaves de comando em `Arc<str>`, verificação ASCII de `HitDefAttr` sem alocação, buffers reutilizáveis de colisão Clsn1/Clsn2 (`fill_boxes`), seleção de golpes da IA em duas passagens e `RefCell::get_mut` evitam alocações no heap a cada tick |
| Caminho rápido de batch de quads | `ume_render::Batch`, `ume_shell::quad` | `push_quad_verts` dedicado insere índices contíguos de quad sem mapeamento por iterador; coordenadas de quad sem rotação evitam trigonometria (`sin_cos`) |
| Limites de resiliência | `ume_formats`, `ume_engine::matchs` | Decodificadores de imagem rejeitam cabeçalhos acima de `MAX_IMAGE_PIXELS` (`1 << 26`) antes de alocar; partidas limitam helpers (60), explods (512), projéteis (256) e textos (128) |

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
| `cargo run --release -p ume-engine --example tick_bench -- [zip_ou_dir] [segundos] [por_lado]` | Taxa de simulação (`ticks/s`) e percentis de latência (`p50`, `p99`, `max`, % do orçamento de 16,6 ms) |
| `cargo test -p ume-engine --release --test tick_alloc` | Zero alocações por tick de simulação |
| `cargo test -p ume-formats --release --test fuzz_resilience` | Fuzzing por mutação em decodificadores SFF v1/v2, SND, PCX, PNG, FNT, RLE8/RLE5/LZ5, AIR e INI |
| `cargo test -p ume-engine --release --test stress_resilience` | Expressões CNS hostis, floods recursivos de helper/explod/projétil e estresse 4v4 simul |

Referência (release): `kfm` caiu de 48,3 ms para 3,5 ms (13,8x mais rápido), `kfm720` de 60,1 ms para 7,8 ms (7,7x mais rápido). Taxa de simulação atinge 235.000+ ticks/s em p50 0,004 ms (0,1% do orçamento de frame).

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

## Busca no sistema de arquivos virtual

`cargo run --locked --release -p ume-formats --example vfs_bench` mede normalização de caminhos e busca por nome com 50.000 arquivos não relacionados. Exibe cinco amostras por cenário. O benchmark sintético exclui acesso a disco, decodificação e renderização.

A busca alternativa na pasta usa o intervalo do mapa ordenado da VFS correspondente ao diretório do personagem, evitando percorrer outros personagens. Os limites de diretório importam: `chars/kfm-extra` não pode substituir um arquivo de `chars/kfm`. A busca global por nome continua como última alternativa de compatibilidade. A normalização ASCII cria uma única string de saída; caminhos não ASCII mantêm as regras Unicode de minúsculas, inclusive sigma grego contextual.

Na máquina de desenvolvimento na nuvem, a mediana da normalização caiu de 60,8 para 48,0 ns por operação, e a da busca na pasta de 278,2 para 1,0 µs. São resultados de microbenchmark, não medidas de FPS ou de carregamento completo do jogo. Arquivos ausentes que exigem busca global ainda podem percorrer todo o mapa.

## Alocações de áudio e interface

`cargo run --locked --release --example audio_bench` decodifica cinco vezes um WAV estéreo gerado de 30 segundos a 44,1 kHz, contando alocações e realocações durante cada decodificação. O decodificador reutiliza o buffer de conversão de amostras intercaladas enquanto houver capacidade; ele cresce quando um pacote posterior precisa de mais amostras. A quantidade de amostras escritas continua vindo do pacote atual, inclusive em pacotes finais parciais e entrada mono.

No ambiente de desenvolvimento na nuvem, as alocações após aquecimento caíram de 2.331 para 1.183 por decodificação. Os tempos variaram (aproximadamente 9–11 ms após aquecimento) e não demonstram ganho de velocidade. A mudança reduz alocações temporárias, não o tamanho final do PCM decodificado. O benchmark exclui reprodução e latência do dispositivo de áudio. Execute `cargo test --locked --release --example audio_bench` para os testes de preservação de amostras, mono/estéreo, cauda e demais operações PCM.

`Ui::quad` transforma os quatro vértices em um array fixo, eliminando uma alocação temporária por quadrilátero desenhado sem mudar a ordem dos triângulos ou suas coordenadas. Isso não comprova ganho de FPS.

## Decodificação Vetorizada de Codecs SFFv2

`decode_rle5` e `decode_lz5` em `crates/formats/src/sff.rs` utilizam preenchimento contíguo de fatias (`out[j..e].fill(val)`) em vez de laços escalares de atribuição elemento a elemento. O LLVM traduz o preenchimento de fatias em instruções memset vetorizadas, acelerando corridas longas em folhas de sprites em alta definição (720p/1080p). Para `kfm720.def`, o tempo de carregamento caiu de 60,1 ms para 6,7 ms (8,97x mais rápido).

## Pipeline Paralelo de Recursos de Personagem

`load_char_def` em `crates/engine/src/loader.rs` sobrepõe as fases de recursos do personagem entre núcleos de CPU utilizando `std::thread::scope`. Decodificação de sprites (`sff:decode`), análise de animação (`air:parse`) e decodificação de áudio (`snd:decode`) rodam simultaneamente em threads trabalhadoras enquanto constantes CNS, arquivos de estado e gatilhos CMD são analisados na thread chamadora. `par::workers` em `crates/formats/src/par.rs` escala até 32 threads de processamento paralelo em hardwares multithread. O tempo total de carregamento passa a ser limitado pela fase individual mais lenta em vez do acúmulo serial.

## Gatilhos e Atribuições de Variáveis com Alocação Zero

A avaliação de partidas em `crates/engine/src/matchs.rs`, `crates/engine/src/triggers.rs` e `crates/engine/src/hitdef.rs` elimina alocações na heap durante os ticks de estado:
- `Match::assigns` registra mutações de variáveis usando o enum compacto `VarKind` (`Var`, `FVar`, `SysVar`, `SysFVar`) em vez de alocar chaves `String` dinâmicas por atribuição.
- `state_mask`, `attr_kind_mask` e `guardflag_bits` processam fatias de string byte a byte com `to_ascii_uppercase()` em linha, eliminando alocações temporárias de `String` maiúscula em avaliações de `hitbyattr`.
- A taxa de simulação atingiu mais de 262.000 ticks/segundo com latência máxima por tick inferior a 0,053 ms.

## Conversão de Sprites para RGBA sem Ramificação via LUT em Cache L1

`sprite_to_rgba` em `crates/formats/src/sff.rs` pré-popula uma tabela de busca de 256 entradas `lut: [[u8; 4]; 256]` em cache L1, mapeando índices de paleta de 8 bits diretamente para quads RGBA. O laço de pixels usa cópias de 4 bytes sem desvio condicional (`chunk.copy_from_slice(&lut[c])`), eliminando desvios por pixel para transparência (`c != 0`) e reduzindo falhas de predição de desvio durante a decodificação de sprites. A fase de decodificação SFF executa em ~0,8 ms para folhas de sprites padrão de lutadores.

## Fila de Comandos de Entrada da IA com Alocação Zero

`crates/engine/src/ai.rs` reutiliza a memória do buffer circular em `Ai.queue: VecDeque<u32>` por meio de `append_seq(&mut queue, ...)` em vez de criar e descartar vetores temporários na heap `Vec<u32>` a cada decisão de movimento da IA. A latência máxima por tick caiu para 0,046 ms sob carga.

## Lotes de Vértices Contíguos e Rasterização sem Multiplicação Prévia

`Batch::push` em `crates/render/src/batch.rs` substitui chamadas repetidas a `self.vertices.push()` por uma única cópia de fatia contígua (`self.vertices.extend_from_slice(&[...])`), agrupando os quatro vértices do quadrilátero em uma única operação de memória. Em `crates/engine/src/raster.rs`, `pixmap_to_image` pré-dimensiona os buffers de pixel (`vec![0u8; pixels.len() * 4]`) e desmultiplica os canais straight-alpha usando `chunks_exact_mut(4)`, viabilizando auto-vetorização sobre superfícies e efeitos procedurais de cenários.
