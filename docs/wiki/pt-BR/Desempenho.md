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

## Medindo

| Comando | Mede |
|---|---|
| `cargo run --release -p ume-engine --example mugen_load_bench [zip] [execuções]` | Tempo de carga de personagem M.U.G.E.N (padrão: `elecbyte.zip` embutido) |
| `cargo run --release -p ume-engine --example load_bench` | Tempo de compilação nativa (UMF) |
| `cargo test -p ume-engine --release --test tick_alloc` | Zero alocações por tick de simulação |

Referência (4 núcleos, release): `kfm720` caiu de 85 ms para 27 ms, `kfm` de 12 ms para 7 ms.
