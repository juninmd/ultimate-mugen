# Criar cenários

Há dois caminhos: cenário M.U.G.E.N (arquivo `.def` + `.sff`, sem programar) ou cenário nativo (código Rust, com camadas procedurais e efeitos reativos).

## Cenário M.U.G.E.N

Coloque `meu_cenario.def` e `meu_cenario.sff` em `mugen/stages/`.

```ini
[Info]
name = "Meu Cenário"
displayname = "Meu Cenário"

[Camera]
startx = 0
starty = 0
boundleft = -150
boundright = 150
boundhigh = -25
boundlow = 0
verticalfollow = .2
tension = 50

[PlayerInfo]
p1startx = -70
p2startx = 70

[StageInfo]
zoffset = 210
localcoord = 320, 240

[BGdef]
spr = meu_cenario.sff

[BG Céu]
type = normal
spriteno = 0, 0
layerno = 0
start = 0, 0
delta = .2, .2
tile = 1, 0
velocity = .1, 0

[BG Chão]
type = normal
spriteno = 1, 0
start = 0, 160
delta = 1, 1
mask = 1

[BG Bandeira]
type = anim
actionno = 10
start = 80, 60
delta = .8, .8

[Begin Action 10]
2,0, 0,0, 6
2,1, 0,0, 6
```

| Seção | Chaves suportadas |
|---|---|
| `[Info]` | `name`, `displayname` |
| `[Camera]` | `startx`, `boundleft`, `boundright`, `boundhigh`, `boundlow`, `verticalfollow`, `tension` |
| `[PlayerInfo]` | `p1startx`, `p2startx` |
| `[StageInfo]` | `zoffset`, `localcoord` |
| `[BGdef]` | `spr` |
| `[BG ...]` | `type` (`normal`, `anim`), `spriteno`, `actionno`, `layerno` (0 atrás, 1 na frente), `start`, `delta`, `tile`, `tilespacing`, `velocity`, `trans`, `mask`, `scalestart`, `scaledelta`, `angle`, `xshear`, `window`, `maskwindow`, `windowdelta`, `spacing` |

Parâmetros de transformação da camada (Ikemen / M.U.G.E.N 1.1), aplicados tanto pelo renderizador quanto pela pré-visualização na seleção de cenário:

| Chave | Significado |
|---|---|
| `scalestart = x, y` | Escala base do sprite, ancorada no eixo dele (padrão `1, 1`). |
| `scaledelta = x, y` | Variação de escala por unidade de movimento da câmera, multiplicada por `delta`: `escala = scalestart + scaledelta * câmera * delta`; a camada cresce ou encolhe conforme a câmera se move (padrão `0, 0`). |
| `angle` | Rotação em graus em torno do eixo do sprite, anti-horária para valores positivos. |
| `xshear` | Cisalhamento horizontal: cada linha é deslocada por `xshear` vezes a distância dela ao eixo. |
| `window = x1, y1, x2, y2` | Retângulo de recorte em unidades do cenário. |
| `maskwindow = x1, y1, x2, y2` | Segundo retângulo de recorte; com os dois presentes a camada é recortada pela interseção. |
| `windowdelta = x, y` | Quão rápido a janela de recorte acompanha a câmera (padrão: o `delta` da camada). |
| `spacing = x, y` | Espaço entre os ladrilhos; usado quando `tilespacing` não existe (`tilespacing` vence se ambos existirem). |

Planejado: `[Music]`, sombras e reflexos configuráveis, `type = parallax`.

## Cenário nativo

Cenários nativos ficam em `src/stages.rs` e os efeitos animados em `src/stagefx.rs`.

1. Escreva uma função `fn meu() -> Stage` montando as camadas com `lay(...)` e as primitivas de `ume_engine::raster` (`rect`, `lin`, gradientes) e `texgen` (`blobs`, `grain`, `haze`). Use `dojo()` como modelo.
2. Registre em `builtin()`:

```rust
StageEntry { id: "meu".into(), name: "Meu Cenário".into(), thumb: [[10, 10, 30], [40, 40, 90], [200, 120, 60]], build: Box::new(meu) },
```

3. Para efeitos reativos (poeira, multidão, água), adicione uma variante em `StageKind` e trate em `stagefx.rs`.
4. Gere capturas para revisar: `UME_SHOTS_ONLY=stages UME_STAGE=meu cargo run --release -- --shots out/`.

Cada `Layer` tem `delta` (parallax), `y`, `front` (desenhada na frente dos lutadores) e `dynamic` (sem bitmap, desenhada por `stagefx`).

## Pixel art dos cenários nativos

As sete arenas nativas usam imagens PNG coloridas incorporadas de `assets/stages/`: `dojo.png`, `neon.png`, `bamboo.png`, `plains.png`, `spirit.png`, `volcano.png` e `brazil.png`. Os prompts e a ferramenta de geração estão registrados em `assets/stages/prompts.json`. O jogo não precisa de serviço externo de imagens nem de acesso a esses arquivos em execução.

`StageArt::Illustrated` desenha o fundo com filtro nearest-neighbor, alinha o piso a 85% da altura da imagem e limita o movimento horizontal da câmera às bordas da imagem. O zoom de equipes mantém a proporção nativa do chão. Sombras, reflexos dos lutadores e efeitos de combate continuam ativos; as antigas animações de objetos ficam exclusivas da arte procedural.

O seletor exibe as mesmas imagens, recortadas nos cartões sem alterar sua proporção. A seleção aleatória e os cenários M.U.G.E.N importados mantêm os cartões com gradiente. Os cenários importados preservam sua própria arte durante a luta.

Use `UME_STAGE_ART=procedural` para comparar os cenários procedurais originais. O seletor continua mostrando as ilustrações em pixel art padrão. Capture o seletor com `UME_SHOTS_ONLY=stage-select ultimate-mugen --shots out/ --res 1280x720 --mute`, ou as sete lutas com `UME_SHOTS_ONLY=stages`.

`brazil` (Praça do Interior — Brasil) acrescenta uma praça ao entardecer com casario colonial, igreja, coreto, ipês e jacarandás floridos. Usa a música do planalto e não possui versão procedural anterior.
