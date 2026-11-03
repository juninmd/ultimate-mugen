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
| `[BG ...]` | `type` (`normal`, `anim`), `spriteno`, `actionno`, `layerno` (0 atrás, 1 na frente), `start`, `delta`, `tile`, `tilespacing`, `velocity`, `trans`, `mask` |

Planejado: `[BGCtrlDef]`/`[BGCtrl]`, `window`, `[Music]`, sombras e reflexos configuráveis, `type = parallax`.

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
