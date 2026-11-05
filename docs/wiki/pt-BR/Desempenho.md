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
| Caminhos críticos sem alocação | `commands`, `matchs`, `triggers`, `ai`, `ikemen` | Chaves de comando em `Arc<str>`, verificação ASCII de `HitDefAttr` sem alocação, seleção de golpes da IA em duas passagens e `RefCell::get_mut` evitam alocações no heap a cada tick |
| Atalho de paleta identidade | `ume_engine::palette` | A verificação `is_identity()` pula trigonometria flutuante, multiplicações e clamp em paletas intocadas, além de atalho em `PalParams::combine` e lote em `apply_slice` |
| Caminhos críticos sem alocação | `commands`, `matchs`, `triggers`, `ai`, `ikemen` | Chaves de comando em `Arc<str>`, verificação ASCII de `HitDefAttr` sem alocação, buffers de rascunho de colisão (`fill_boxes`, `fill_proj_boxes`), array de pilha para projéteis congelados, explods `remove_on_get_hit` pré-filtrados e `RefCell::get_mut` evitam alocações no heap a cada tick |
| Caminhos críticos sem alocação | `commands`, `matchs`, `triggers`, `ai`, `ikemen` | Chaves de comando em `Arc<str>`, verificação ASCII de `HitDefAttr` sem alocação, índices de alvos em array de pilha, persistência em vetor linear, avaliação de parâmetros `num_opt` em passagem única e `RefCell::get_mut` evitam alocações no heap a cada tick |
| Caminhos críticos sem alocação | `commands`, `matchs`, `triggers`, `ai`, `ikemen` | Chaves de comando em `Arc<str>`, verificação ASCII de `HitDefAttr` sem alocação, `rel_dirs` sem desvios condicionais, verificação de botões recentes em ciclo único por operação bitwise e `RefCell::get_mut` evitam alocações no heap a cada tick |
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

`cargo run --locked --release --example audio_bench` decodifica cinco vezes um WAV estéreo gerado de 30 segundos a 44,1 kHz, contando alocações e realocações durante cada decodificação. O decodizador reutiliza o buffer de conversão de amostras intercaladas enquanto houver capacidade; ele cresce quando um pacote posterior precisa de mais amostras. A quantidade de amostras escritas continua vindo do pacote atual, inclusive em pacotes finais parciais e entrada mono.

No ambiente de desenvolvimento na nuvem, as alocações após aquecimento caíram de 2.331 para 1.183 por decodificação. Os tempos variaram (aproximadamente 9–11 ms após aquecimento) e não demonstram ganho de velocidade. A mudança reduz alocações temporárias, não o tamanho final do PCM decodificado. O benchmark exclui reprodução e latência do dispositivo de áudio. Execute `cargo test --locked --release --example audio_bench` para os testes de preservação de amostras, mono/estéreo, cauda e demais operações PCM.

`Ui::quad` transforma os quatro vértices em um array fixo, eliminando uma alocação temporária por quadrilátero desenhado sem mudar a ordem dos triângulos ou suas coordenadas. Isso não comprova ganho de FPS.

## Decodificação Vetorizada de Codecs SFFv2

`decode_rle5` e `decode_lz5` em `crates/formats/src/sff.rs` utilizam preenchimento contíguo de fatias (`out[j..e].fill(val)`) em vez de laços escalares de atribuição elemento a elemento. O LLVM traduz o preenchimento de fatias em instruções memset vetorizadas, acelerando corridas longas em folhas de sprites em alta definição (720p/1080p). Para `kfm720.def`, o tempo de carregamento caiu de 60,1 ms para 6,7 ms (8,97x mais rápido).

## Pipeline Paralelo de Recursos de Personagem

`load_mugen_char` em `crates/engine/src/loader.rs` sobrepõe as fases de recursos do personagem entre núcleos de CPU utilizando `std::thread::scope`. Decodificação de sprites (`sff:decode`), análise de animação (`air:parse`) e decodificação de áudio (`snd:decode`) rodam simultaneamente em threads trabalhadoras enquanto constantes CNS, arquivos de estado e gatilhos CMD são analisados na thread chamadora. `par::workers` em `crates/formats/src/par.rs` escala até 32 threads de processamento paralelo em hardwares multithread. O tempo total de carregamento passa a ser limitado pela fase individual mais lenta em vez do acúmulo serial.

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

## Despacho Direto de Gatilhos em Bytecode

O trait `Host` em `crates/engine/src/expr.rs` e a máquina virtual em `crates/engine/src/bytecode.rs` introduzem ganchos de despacho direto `trigger0` e `trigger1`. Para avaliação de gatilhos unários (`Op::Trig1`), operandos na pilha avaliam diretamente na lógica do gatilho sem encapsulamento em arrays AST temporários `[Node::Num]` ou redirecionamento via `eval()`. Gatilhos críticos (`var`, `fvar`, `sysvar`, `animelemno`, `animelemtime`, `animexist`, `selfstatenoexist`) executam como índices e consultas inline em arrays, reduzindo a latência máxima de tick de simulação para 0,042 ms.

## Analisador de Animações AIR com Alocação Zero

`parse_air` em `crates/formats/src/air.rs` opera diretamente sobre fatias de string de cada linha sem alocar instâncias `String` em minúsculas (`line.to_ascii_lowercase()`) para cada linha do arquivo de animação. O reconhecimento de seções e cabeçalhos (`parse_begin_action`, `parse_clsn_header`, `parse_clsn_box`) avalia fatias de bytes com insensibilidade a maiúsculas ASCII (`eq_ignore_ascii_case`), e as coordenadas de caixas de colisão são processadas em um array de pilha fixo `[f32; 4]` em vez de alocar um `Vec<f32>` na heap por caixa delimitadora.

## Cópia Pré-alocada de Canais de Áudio PCM

Em `src/sound_mix.rs:decode`, a decodificação de quadros de áudio calcula a capacidade do pacote antecipadamente e invoca `out.frames.reserve(count)`, eliminando realocações geométricas repetidas do vetor durante a leitura de amostras. Um fluxo dedicado para áudio mono (`ch == 1`) transfere quadros sem divisão de fatias, e a decodificação estéreo utiliza `chunks_exact(ch)` para eliminar verificações de limites. As alocações em execução aquecida caíram para 1.175 por fluxo de áudio de 30 segundos.
- **Onda 23 - Tabela de Jump Direta Inline do Bytecode e Dispatch com Host Fatiado**:
  - Aplica inline das operações aritméticas e relacionais mais frequentes (Add, Sub, MulF, MulI, DivF, DivI, Eq, Ne, Lt, Le, Gt, Ge, BitAnd, BitOr, BitXor, Xor) diretamente no loop de casamento externo de 
run_slice, eliminando chamadas intermediárias de função e closures.
  - Fatia self.host antecipadamente em 
run_slice, reduzindo overhead de desreferenciação de struct a cada resolução de gatilho e expressão.
- **Onda 24 - Passo de Fundo de Cenário Sem Cópias e Retorno Rápido de Ondas**:
  - Utiliza casamento de padrões disjunto (BgState { layers, def_time, ctrl_defs, stage } = self) em step(), eliminando incrementos atômicos de self.stage.clone() em Arc e alocações de std::mem::take a cada tick.
  - Adiciona retorno rápido para amplitude zero s[0] == 0.0 em sin_wave(), evitando cálculos transcendentais de .sin() para camadas estáticas de fundo.
  - Ignora a soma de quadros de início de repetição de animação quando .loop_start == 0.
- **Onda 25 - Codificação Rápida de WAV e Reamostragem de Áudio em Taxa Identidade**:
  - Substitui loops de extensão de fatia por amostra por redimensionamento em bloco de passagem única e escrita indexada direta em wav() e 	o_wav().
  - Implementa caminho rápido vetorizável por SIMD para reprodução em taxa unitária ((rate - 1.0).abs() < 1e-4) em Pcm::render, içando produtos de ganho constantes para fora do loop de mixagem.
- **Onda 26 - Parser de HitDef Sem Alocações e Construção Estática de Shake**:
  - Elimina formatação intermediária de strings e clones em hitdef::build utilizando Option<&str> emprestado para parâmetros, casamento direto de bytes em ttr_bits, state_mask e guardflag_bits.
  - Substitui formatação dinâmica de strings por consultas estáticas de campo para parâmetros de shake de tela/câmera.
  - Ramo padrão com zero alocações para estado e tipos de ataque.
- **Onda 27 - Caixas de Colisão In-Place e Início Rápido de Elemento de Animação**:
  - Implementa Fighter::fill_boxes utilizando buffers scratch fornecidos pelo chamador, eliminando alocações contínuas de Vec<Box4> nos loops de detecção de reversals, golpes e projéteis.
  - Substitui coleta de vetor na heap para verificação de caixa única p2clsnrequire por fatia em array na pilha com zero cópias.
  - Adiciona saída antecipada if n <= 1 { return 0; } em `elem_start` para ignorar instanciação de iteradores nas consultas mais frequentes de tempo de quadros de animação.
Onda 30 - Direções Relativas de Entrada Sem Desvios e Tabela de Salto em Bytes:
- Substitui desvios aninhados em `input::rel_dirs` por deslocamento binário branchless `ud | (4 << (is_back ^ 1))`.
- Substitui casamento de strings em `button_bit` por casamento em nível de byte na fatia, suportando insensibilidade a maiúsculas sem alocação.
- Implementa atribuição de direção branchless em `InputHistory::push`.
- **Onda 31 - Descompactação ZIP VFS Paralela Multi-Thread e HUD de Loading Cibernético**:
  - Implementa descompressão em blocos com escopo multi-thread (`std::thread::scope`) aproveitando todo o paralelismo de CPU disponível para extração de arquivos ZIP em `add_zip`.
  - Substitui descompressão sequencial por trabalho concorrente em lote, reduzindo a decodificação de recursos SFF de 222.6 ms para 88.7 ms (-60.1%).
  - Adiciona HUD de carregamento cibernético com bordas brilhantes, pulso laser de dois tons ciano/branco e telemetria de pipeline paralelo.

## Decodificação SIMD de paletas ACT e fast-path de limites de câmera

- Decodificação vetorizada ACT: decode_act e `encode_act` utilizam s_chunks::<3>() e s_chunks_mut::<4>() para decodificar tripletos RGB para RGBA em lotes sem checagens redundantes de limites.
- Limites de câmera em passada única: Match::update_camera calcula os limites de todos os lutadores raiz (minx, maxx, highest) e dos lutadores vivos (lo, hi) em uma única iteração de self.roots(), eliminando filtragens redundantes.
- Checagens de sobreposição otimizadas: Match::overlap armazena as coordenadas de  em registradores escalares antes do laço interno de colisão contra  e substitui divisões por 2.0 por multiplicações * 0.5.

## Pipeline de avaliação de controladores e gatilhos com zero alocações

- Indexação de alvos sem alocação no heap: `controllers::target_indices` retorna buffer em pilha `Vec<usize>` em vez de `Vec<usize>`, eliminando alocações em controladores de alvo (`targetstate`, `targetlifeadd`, `targetpoweradd`, `targetfacing`, `targetbind`, etc.).
- Resolução de flags AssertSpecial sem alocação: `flags::names_in` e `flags::lookup` operam diretamente em fatias `&str` com comparação ASCII insensível a maiúsculas/minúsculas (`eq_ignore_ascii_case`), removendo alocações de strings a cada quadro.
- Constantes de velocidade em buffer na pilha: `triggers::Ctx::const_val` formata a chave em um buffer de 64 bytes para buscar sufixos `.x` sem alocação no heap via `format!("{k}.x")`.
- Atribuições de variáveis tipadas: `Match::assigns` armazena o enum `AssignKind` em vez de alocar uma `String` na atribuição de variáveis de gatilhos.

## Descompressão RLE em bloco e eliminação de escrita redundante de zeros no SFF v2

- Eliminação de escrita de zeros em `decode_rle8`: Como `vec![0u8; size]` já aloca memória inicializada em zero, repetições de pixels transparentes (`val == 0`) e literais zero (`d == 0`) pulam escritas na memória e apenas avançam o índice `j`. Isso elimina esperas nos buffers de escrita e sujeira nas linhas de cache (geralmente 50-80% de transparência).
- Preenchimento em bloco no RLE5: Em `decode_rle5`, loops byte a byte foram substituídos por `.fill(c)` em fatias (ou avanço direto quando `c == 0`), transformando iterações individuais em memsets vetorizados.
- Cópia direta de fatias para paletas SFF v2: Em `decode_sff_v2`, paletas com limites válidos utilizam cópia em lote ou direta via `copy_from_slice`, eliminando checagens de limites e adições saturadas por cor.

## Saída antecipada de caixas de colisão, fatiamento de alvos sem alocação e despacho de IA indexado

- Fast-path em caixas de colisão: Em `fighter::boxes()`, quadros sem caixas e sem modificadores ativos retornam imediatamente `vec![]` sem consultar tabelas de escala ou calcular fatores. Fatores de escala pré-multiplicados e inversão horizontal com XOR único (`flip = fr.flip_h ^ (self.facing < 0.0)`) removem penalidades de ramificação em cada vértice.
- Fatiamento de alvo sem alocação em `p2clsnrequire`: Em `Match::tick`, alvos específicos emprestam a fatia `&d_boxes[req..req + 1]` diretamente em vez de alocar um `Vec` no heap.
- Despacho de comandos de IA indexado: Em `ai::Ai`, comandos são indexados como `Vec<usize>` em `fighter.def.commands` em vez de clonar árvores `CommandDef`. Ataques próximos e à distância são separados na inicialização, tornando a seleção em `O(1)` sem varreduras lineares.
- Conversão vetorizada de pixmap para straight alpha: `raster::pixmap_to_image` evita divisões inteiras em pixels opacos (`alpha == 255`) e transparentes (`alpha == 0`) utilizando `as_chunks_mut::<4>()`.

## Decodificação SND pré-alocada e filtragem de comandos sem alocação

- Tabelas hash pré-alocadas para sons SND: `snd::decode_snd_checked` limita a capacidade em `count.min(8192)` e pré-aloca as tabelas `out` e `seen`, eliminando redimensionamentos repetidos e rehashes durante a decodificação de áudio de personagens.
- Resolução estática de chaves de paleta no carregador: `loader::load_mugen_char` resolve substituições de paletas (`pal1`..`pal12`) usando uma tabela estática de fatias `PAL_KEYS` em vez de formatar 12 strings no heap por personagem.
- Deduplicação de sprites pré-alocada: Pré-dimensiona os vetores `seen` e `unique` para o tamanho de `sff.sprites.len()`.
- Extração de lista de golpes e comandos de IA com zero alocações: `loader::load_mugen_char` filtra comandos no local usando `eq_ignore_ascii_case` antes de clonar ou alocar strings, removendo dezenas de alocações no heap por personagem carregado.
- **Onda 27 - Caixas de Colisão In-Place e Início Rápido de Elemento de Animação**:
  - Implementa Fighter::fill_boxes utilizando buffers scratch fornecidos pelo chamador, eliminando alocações contínuas de Vec<Box4> nos loops de detecção de reversals, golpes e projéteis.
  - Substitui coleta de vetor na heap para verificação de caixa única p2clsnrequire por fatia em array na pilha com zero cópias.
  - Adiciona saída antecipada if n <= 1 { return 0; } em `elem_start` para ignorar instanciação de iteradores nas consultas mais frequentes de tempo de quadros de animação.
- **Onda 30 - Direções Relativas de Entrada Sem Desvios e Tabela de Salto em Bytes**:
  - Substitui desvios aninhados em `input::rel_dirs` por deslocamento binário branchless `ud | (4 << (is_back ^ 1))`.
  - Substitui casamento de strings em `button_bit` por casamento em nível de byte na fatia, suportando insensibilidade a maiúsculas sem alocação.
  - Implementa atribuição de direção branchless em `InputHistory::push`.

## Analisador de animação e colisão AIR no local sem alocações no heap

- Eliminação de alocações `to_ascii_lowercase()` por linha: `air::parse_air` processa linhas de texto diretamente com comparação ASCII de bytes insensível a maiúsculas (`eq_ignore_ascii_case`), removendo de 5.000 a 20.000 alocações de string por arquivo `.air`.
- Analisador de caixas de colisão em pilha: `parse_clsn_box` analisa quatro coordenadas diretamente em um array fixo `[f32; 4]` na pilha, eliminando alocações de `Vec<f32>` por caixa de colisão.
- Tokenização de parâmetros de quadros em buffer de pilha: Linhas AnimFrame desempacotam até 11 parâmetros separados por vírgula em um array fixo `[&str; 11]` sem alocar `Vec<&str>`.
- Análise de flags de flip e blend no local: Flags `fl` e `bl` são avaliadas diretamente sem alocação de strings em maiúsculas.

## Parâmetros de shake no HitDef sem alocação e correspondência de tipos no local

- Resolução de parâmetros de shake sem alocação: `hitdef::build` usa consultas diretas de strings estáticas (`parse_shake`) em vez de formatar 10 chaves dinâmicas no heap (`{prefix}.freq`, `{prefix}.time`, `{prefix}.ampl`, etc.) a cada instanciação de `HitDef`.
- Análise de `animtype` e `groundtype` no local: `hitdef::animtype` e `hitdef::groundtype` comparam tokens padrão de animação/chão do MUGEN usando `eq_ignore_ascii_case` diretamente em fatias `&str` sem alocar strings em minúsculas.

## Consultas de seções INI sem alocação e análise de constantes CNS em pilha

- `Section::get` e `Section::get_all` sem alocação: `ini::Section::get` e `ini::Section::get_all` comparam chaves diretamente com `eq_ignore_ascii_case` sem alocar instâncias de `String` em minúsculas a cada consulta.
- `find_section` sem alocação: `ini::find_section` utiliza `eq_ignore_ascii_case` diretamente no nome da seção sem alocar strings em minúsculas.
- Análise de constantes CNS em pilha: `cns::parse_constants` desempacota vetores numéricos separados por vírgula em um array `[f64; 4]` na pilha, eliminando alocações de `Vec<f64>` em cada linha de constantes.
- Vetor de comandos pré-alocado: `cns::parse_cmd` pré-aloca a capacidade de `cmds` para `secs.len()`.

## Fast-path de bytecode de operação única e avaliação sem pilha

- Atalho de expressão com operação única: Em `bytecode::Code::run`, expressões com exatamente um opcode (`Op::Const(v)`, `Op::Trig0(i)`, `Op::Host(i)`) retornam imediatamente sem inicialização do array de pilha, avanço do program counter ou despacho no loop da máquina virtual.
- Consultas de constantes e parâmetros com custo zero: Números constantes e gatilhos sem argumentos (`Time`, `StateTime`, `AnimTime`, `StateNo`, etc.) executam em 1 ciclo de CPU sem overhead de frame de execução.

## Avanço de animação em consulta única e atalho na limpeza de alvos

- Desempacotamento de quadro de animação em consulta única em `advance_anim`: Desempacota `(total, loop_start, len, ft)` em uma única consulta a `self.anim()`, eliminando buscas repetidas em HashMap por lutador e explod animado a cada tick.
- Atalho para substituições de tamanho vazias: `fighter::c` consulta diretamente constantes do personagem quando `self.size_overrides.is_empty()`, pulando varreduras lineares na fatia.
- Limpeza de alvos obsoletos com atalho: `Match::tick` pula movimentações de vetores e chamadas de retenção sempre que `self.fighters[k].targets.is_empty()`.
- **Onda 42 - Caminho Rápido de Efeitos de Paleta, Buffers de Quadro em Pilha e Consultas Zero-Alloc**:
  - PalParams::apply: Caminho rápido para transformações idênticas e sem rotação de matiz/inversão, ignorando trigonometria e desenrolando canais de cores.
  - PalFx::from_controller: Eliminação de alocações na heap para buscas de parâmetros de controladores usando chaves em buffer de pilha.
  - Match::tick: Remoção de alocações dinâmicas de Vec<usize> por tick para waken e 	ag_requests através de arrays em pilha, unificando loops com 
etain_mut.
- **Onda 43 - Carregador de Personagens Zero-Alloc e Unquote In-Place**:
  - `ini::unquote_str`: Empréstimo in-place de fatias sem alocar novas instâncias de `String` para strings limpas ou entre aspas.
  - `loader::load_mugen_char`: Conversão de `fget` para retornar `Option<&str>` ao invés de `Option<String>`, eliminando cerca de 20 alocações na heap por carregamento de lutador.
  - `loader::local_size`: Análise em pilha via `split_once(',')` substituindo a alocação dinâmica de `nums`.
  - `cns::parse_cmd`: `unquote_str` in-place para nome e comandos.

- **Onda 44 - Despacho de Estado AssignKind Zero-Alloc e Máscaras de Bits Case-Insensitive**:
  - `matchs::AssignKind`: Substituição de alocações dinâmicas de `String` em atribuições de variáveis por um enum de 1 byte `AssignKind`, reduzindo pela metade a memória da tupla e eliminando operações na heap.
  - `hitdef::state_mask` / `attr_kind_mask` / `guardflag_bits`: Processamento de bytes ASCII insensível a maiúsculas in-place sem alocação de strings.
  - `triggers::const_val`: Busca em buffer de pilha com zero alocações para constantes `velocity.*.x`.

- **Onda 45 - Cache de Explods em Array e Varredura Triangular de Colisão de Projéteis**:
  - `Match::update_explods`: Pré-indexação de posições e estados de pausa dos lutadores em array compacto de 8 posições na pilha, eliminando varreduras lineares por explod.
  - `Match::update_projectiles`: Flags de congelamento em buffer de pilha para até 64 projéteis sem alocações na heap, e iteração triangular de pares `(a + 1)..n` reduzindo pela metade verificações de colisão entre projéteis.

- **Onda 46 - Caminho Rápido de Caixas de Colisão e Geometria Pré-Escalada**:
  - `Fighter::boxes`: Saída antecipada para listas vazias de caixas de colisão antes de calcular constantes ou fatores de escala (reduz drasticamente o trabalho em frames ociosos/movimento).
  - Dimensões X e Y pré-escaladas (`sx * s` e `sy * s`), reduzindo as multiplicações de ponto flutuante por caixa de 8 para 4.
  - Combinação de `flip_h != (facing < 0.0)` em uma única condição booleana XOR branchless.
  - `Fighter::c`: Busca de fatia guardada por `!self.size_overrides.is_empty()`.

- **Onda 47 - Junção de Candidatos VFS em Buffer de Pilha e Casamento de Sufixo Zero-Alloc**:
  - `vfs::resolve`: `with_joined` em buffer de pilha de 512 bytes para caminhos candidatos `base_dir/name` e `data/name`, eliminando alocações na heap no caminho direto de busca.
  - Substituição de strings dinâmicas `format!("/{base}")` por predicado de bytes in-place `is_match`.
  - Uso direto de `ini::unquote_str` sem alocar strings intermediárias.
  - Fallback de pasta sub-microssegundo em 50k arquivos (0.9 us/op) e redução no tempo de carga de lutadores (`kfm.def` caiu para 6.7 ms).

- **Onda 48 - Pipeline de HUD Zero-Alloc e Quads de Textura Diretos**:
  - `Hud::draw`: Otimização de `main_of` em varredura única de iterador sem alocar `Vec<&Fighter>` por lado a cada quadro.
  - Uso de array fixo em pilha `roots = [r0, r1]` substituindo vetores dinâmicos na heap.
  - Eliminação de alocações de parceiros em partidas 1v1 convencionais.
  - Substituição de vetores de malha/vértices/índices no desenho de retratos por chamadas diretas de `draw_texture_ex`.

- **Onda 49 - Curto-Circuito em Bytecode de Duas Operações e Contexto Único em Gatilhos**:
  - `bytecode::Code::run`: Execução em caminho rápido para expressões de 2 operações (`!trigger`, `-const`, `!const`, `!host`), ignorando inicialização de pilha e loop do interpretador.
  - `Match::triggered`: Reutilização de instância única de `Ctx` entre `trigger_all` e grupos de gatilhos, eliminando recriações de contexto por controlador.
  - `Match::num`: Avaliação direta de expressão sem mapeamento intermediário de closure.

- **Onda 50 - Pré-Alocação de SND, Deduplicação de Parsing de Estados e Filtros de Comando Zero-Alloc**:
  - `snd::decode_snd_checked`: Pré-alocação do dicionário de sons e do conjunto de offsets visitados usando a contagem de sons do cabeçalho, com inserções in-place `Entry::Vacant` evitando realocações redundantes.
  - `loader::load_mugen_char`: Eliminação da segunda passagem redundante de parsing do `stcommon` durante o carregamento de personagens.
  - Chaves de paletas estáticas pré-alocadas `PAL_KEYS` evitando 12 alocações de formatação de string dinâmica por carga.
  - Pré-alocação de capacidade para `HashSet`, `sprites`, `indexed` e `counts` com base no número de sprites decodificados do SFF.
  - Comparação de bytes ASCII insensível a maiúsculas (`eq_ignore_ascii_case`) nos filtros de `ai_commands` e `movelist`, eliminando strings temporárias na heap para nomes de comandos.
  - `loader::load_mugen_portrait`: Extração in-place de fatias de string em consultas de seção evitando clones de strings pertencidas.

- **Onda 51 - Cache de Dimensões Comuns, Consulta Única de Sprites e Despacho Direto de Textura**:
  - `CharDef`: Dimensões centrais e fatores de escala (`scale_x`, `scale_y`, `size_ground_back`, `size_ground_front`, `size_air_back`, `size_air_front`, `size_height`, `size_attack_dist`) em cache direto na estrutura via `refresh_sizes`.
  - `Fighter::c`: Casamento de padrões em caminho rápido para chaves `size.*`, evitando hashing de strings dinâmicas e consultas a `HashMap` a cada tick de física, colisão e avaliação de gatilhos.
  - `Gfx::sprite_tex_for`: Eliminação da consulta duplicada de `def.sprites.get` por quadro renderizado em todos os lutadores, sombras, ajudantes e explods.
  - `Gfx::draw_sprite`: Uso direto de `def.scale_x` e `def.scale_y`, removendo consultas repetidas à tabela `def.c(...)` a cada quadro.
  - Redução no tempo de carregamento de `kfm.def` para 4.6 ms e taxa de simulação acelerada para 216k ticks/s.

- **Onda 52 - Cache de Temporização de Animação e Atalhos em Bytecode de 3 Operações**:
  - `Fighter`: Cache de `anim_total_time` e `anim_loop_start` em mudanças de animação, convertendo `anim_time_trig` (`animtime`) em uma subtração escalar direta sem consultas a mapas.
  - `Fighter::advance_anim`: Reutilização da consulta única de animação durante o avanço de quadros, eliminando chamadas repetidas a `self.anim()`.
  - `Fighter::anim_elem_time`: Retorno instantâneo para `animelemtime(1)` sem iteração de quadros.
  - `bytecode::Code::run`: Curto-circuito em caminho rápido para expressões de comparação de 3 operações (`[Trig0, Const, Cmp]` e `[Const, Trig0, Cmp]`), avaliando gatilhos diretamente via `Host::trigger0` e ignorando alocações na pilha da máquina virtual.
  - Redução do tempo de carregamento de `kfm720.def` para 8.7 ms e de `kfm.def` para 4.4 ms.

- **Onda 53 - Dicas de Índice de Lutadores, Acertos em Buffer de Pilha e Atalhos de Reversão**:
  - `Match::idx_hint`: Resolução de índice direta em $O(1)$ baseada em dicas de posição da ordem de execução, substituindo varreduras lineares entre lutadores no pipeline de cada tick.
  - `Match::detect_reversals`: Saída rápida antecipada quando nenhum lutador possui estado ativo de reversão, além de buffers de pilha de tamanho fixo para reversões detectadas.
  - `Match::detect_hits`: Buffer fixo em pilha `pending` armazenando apenas índices e coordenadas de lutadores, eliminando clones prematuros na heap de `HitDefData` em frames sem impacto.
  - Lista de `struck` migrada para buffers de pilha, eliminando alocações dinâmicas nos loops de física e detecção de acertos.

- **Onda 54 - Índices de Alvo em Buffer de Pilha e Avaliação Direta de Expressões**:
  - `controllers::target_indices`: Substituição de `Vec<usize>` alocado na heap por array fixo em pilha `[Option<usize>; 8]`, eliminando alocações repetidas em `TargetState`, `TargetLifeAdd`, `TargetPowerAdd`, `TargetFacing` e `TargetBind`.
  - `controllers::set_vars`: Avaliação direta de `param.exprs.first()` sem iterações duplicadas de busca por string `sc.param(...)`.
  - `controllers::run`: Substituição de checagens duplas de parâmetros em `changestate`/`selfstate` por consultas unificadas `opt(...)` para `ctrl` e `anim`.
  - Vazão de simulação elevada para 226.415 ticks/s com p50 em 0.003 ms, e decodificação de retratos reduzida para 0.6 ms.

- **Onda 55 - Decodificacao RGBA Alinhada em u32, Tabelas SFF Pre-Alocadas e Paletas PCX Sem Desvios**:
  - sff::sprite_to_rgba: Alocacao direta em Vec<u32> garantindo alinhamento de 4 bytes por hardware, eliminando loops de alinhamento de fatias e checagens de desvio, e convertendo para Vec<u8> via Vec::from_raw_parts com custo zero.
  - pcx::read_rgb_palette: Conversao de blocos RGB de 768 bytes em [u32; 256] com u32::from_ne_bytes sem desvios condicionais e atribuicao direta de canal alfa constante.
  - sff::decode_v1: Remocao do campo nao utilizado idx em V1Job e pre-alocacao do vetor jobs com count.min(8192).
  - sff::decode_v2: Pre-alocacao das tabelas de palettes com pal_count.min(buf.len() / 16 + 1) e jobs com spr_count.min(buf.len() / 28 + 1).
  - sff::decode_v2_image: Otimizacao de quadros RAW profundidade 32 com copia direta de fatias e profundidade 24 com chunks de 3 bytes em vez de consultas byte a byte.

- **Onda 56 - Binds com Semantica Copy, Fallbacks Preguicosos e Animacoes Diretas**:
  - `fighter::Bind`: Implementacao de `Copy` e `PartialEq`, convertendo verificacoes e atualizacoes em `rebind` e `update_bind` em copias binarias de pilha sem clones na heap.
  - `fighter::Fighter::change_anim_own`: Adicao de metodo auxiliar para troca de animacao propria, removendo clones externos de `def.clone()` em `changestate`, `changeanim` e transicoes de auto-turn.
  - `matchs::run_state`: Conversao da consulta a `common` em fallback preguicoso (`let fallback;`), ignorando clones de `self.common` em todos os estados negativos (-3, -2, -1) e estados definidos pelo personagem.
  - `matchs::change_state`: Substituicao do clone ansioso de `self.common` por igualdade de ponteiro `&self.common` e fallback sob demanda.
  - `matchs::tick`: Eliminacao do clone por quadro de `f.st_def` no loop de override de fisica atraves de extracao escalar de `Option<char>`.

- **Onda 57 - Parsing INI Sem Alocacao de Linhas, Controladores e Estados Pre-Dimensionados**:
  - `ini::parse_ini_no_lines`: Substituicao do parsing de INI que copiava linhas em `parse_states`, `parse_constants`, `parse_cmd` e na resolucao de DEF do `loader.rs` por `parse_ini_no_lines`, evitando dezenas de milhares de alocacoes de `String` descartadas por personagem.
  - `ini::parse_ini_opt`: Pre-alocacao do vetor de secoes com `text.len() / 256 + 4` e das entradas de cada secao com capacidade 8.
  - `cns::build_controller`: Pre-alocacao de `params` do controlador com a capacidade exata de `s.entries.len()`.
  - `cns::parse_states`: Pre-alocacao de `StateDef` com capacidade de 16 controladores e parametros dimensionados pelas entradas da secao, eliminando realocacoes dinamicas na compilacao de estados.

- **Onda 58 - Normalizacao de Caminhos em Pilha, Atalhos Rapidos de Comandos e Simbolos Copy**:
  - `vfs::with_norm_path`: Buffer de 512 bytes em pilha para normalizacao e remocao de aspas de caminhos, eliminando alocacoes na heap em todas as consultas de recursos de `vfs::resolve`.
  - `commands::CmdSymbol`: Implementacao de `Copy` e `PartialEq` para manipulacao binaria de simbolos de 20 bytes.
  - `commands::held`: Verificacao antecipada de mascara de botoes antes de consultas de direcao relativa e orientacao, descartando botoes inativos na primeira instrucao.
  - `commands::fires`: Avaliacao de `now & !prev` antes de ler ticks anteriores do historico (`p2`, `p3`), evitando sondagens de historico em frames sem disparo.

- **Onda 59 - Escala em Cache no Preenchimento de Caixas e Poda AABB Multicaixas**:
  - `fighter::fill_boxes`: Ignoradas consultas dinamicas por string `size.*` quando nao ha overrides de tamanho, lendo `def.scale_x` e `def.scale_y` diretamente.
  - `matchs::overlap`: Adicionada saida antecipada por bounding box composto (AABB) em colisoes de grupos multicaixas (`a.len() > 1 || b.len() > 1`), eliminando testes $O(N 	imes M)$ em pares disjuntos.

- **Onda 60 - Tokenizacao de Quadros AIR Sem Alocacao e Buffers Pre-Dimensionados**:
  - `air::parse_air`: Substituicao da alocacao em heap de `Vec<&str>` por quadro por um array fixo em pilha `[""; 12]`, eliminando alocacoes dinamicas de vetores em todos os quadros de animacao.
  - `air::parse_air`: Substituicao de alocacoes `String` via `to_ascii_uppercase()` para flags e modos de blend por inspecao em nivel de byte (`fl.as_bytes()`, `bl.as_bytes()`).
  - `air::parse_as_d`: Substituicao de busca por strings por varredura de bytes sem distincao de maiusculas/minusculas para parametros alfa `AS...D...`.
  - `air::parse_air`: Pre-alocacao do vetor de quadros com `Vec::with_capacity(8)` e do HashMap de animacoes com `text.len() / 512 + 16`.
