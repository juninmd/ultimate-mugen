# Arte do esqueleto (rig)

Com `"art": { "type": "rig" }` o lutador é desenhado por vetor em tempo de carga (`crates/engine/src/native/rig.rs`). Fica nítido em qualquer resolução e cada paleta é gratuita.

## `style`: aparência

| Chave | Valores encontrados nos lutadores inclusos |
|---|---|
| `build`, `muscle`, `arms`, `waist`, `scale` | Proporções (1.0 = padrão) |
| `female` | `true` / `false` |
| `torsoShape` | `heavy` |
| `top` | `gi` `bare` `tank` `jacket` `coat` `robe` `suit` `qipao` `armor` |
| `sleeves` | `none` `short` `long` `wide` `gi` |
| `pants` | `gi` `baggy` `tight` `hakama` `trunks` |
| `skirt` | `coat` `robe` |
| `belt` | `true` ou cor: `belt` `cloth` `metal` `accent` `top2` |
| `gloves` | `wrap` `glove` `long` `gauntlet` |
| `boots` | `barefoot` `shoe` `boot` `tall` `tabi` `wrap` `greave` |
| `hair` | Objeto `{ "type": "spiky", ... }`; tipos `short` `spiky` `long` `ponytail` `topknot` `mane` `wild` `helmet` |
| `eyes` | `normal` `fierce` `cute` `glow` |
| `weapon` | `katana` `greatsword` `cutlass` `kunai` |
| `emblem` | `{ "type": "bolt", "color": "#f2b233", "where": "back", "size": 1.3 }`; tipos `bolt` `crest` `glyph` `lotus` `moon` `star` `wave` |
| `weave`, `pantsWeave` | Textura do tecido: `weave` `check` `dots` `hatch` `quilt` `rib` `stripes` |
| Acessórios | `headband` `scarf` `cape` `sash` `mask` `glasses` `goggles` `hat` `horns` `crown` `earring` `necklace` `pauldron` `shoulderPads` `pouches` `tattoo` `scars` `bandages` `beard` `facePaint` `eyepatch` `talisman` |
| Detalhes | `veins` `knuckles` `stitch` `folds` `faceShade` `brow` `shine` |
| `colors` | Mapa de cores nomeadas, abaixo |

### `colors`

Chaves usadas: `skin` `hair` `eye` `lips` `top` `top2` `trim` `pants` `belt` `glove` `boot` `wrap` `cloth` `accent` `metal` `weapon` `weapon2` `hilt` `visor` `mask` `hat` `horn` `undershirt` `legs` `beard` `aura` `energy`.

`palettes` troca qualquer uma delas:

```json
"palettes": [
  { "top": "#b8433a", "trim": "#6a1e1a", "hair": "#1a1a22" },
  { "top": "#e8e2d2", "skin": "#f0c8a0" }
]
```

## `poses`: ângulos das articulações

```json
"poses": {
  "idle0": { "shoulderF": 42, "elbowF": 102, "shoulderB": 22, "elbowB": 120,
             "thighF": 22, "kneeF": 26, "thighB": -18, "kneeB": 12, "lean": 6, "head": -4 }
}
```

`F` = membro da frente, `B` = de trás. Ângulos em graus.

| Chave | Descrição |
|---|---|
| `shoulderF/B`, `elbowF/B`, `wristF/B` | Braços |
| `thighF/B`, `kneeF/B`, `ankleF/B` | Pernas |
| `lean`, `head`, `rot` | Inclinação do tronco, da cabeça e rotação do corpo inteiro |
| `y`, `hipH` | Altura do quadril |
| `handF/B` | `fist` `open` |
| `face` | `smile` `shout` `hurt` `ko` ... |
| `blade`, `weapon`, `noWeapon` | Ângulo e visibilidade da arma |
| `aura`, `glow`, `charge`, `chargeSize` | Brilhos de energia |
| `wind`, `hairWind`, `phase`, `free` | Vento na roupa/cabelo e fase da animação secundária |

Partes usáveis em `hit.part`: `handF` `handB` `footF` `footB` `heelF` `heelB` `elbowF` `elbowB` `kneeF` `kneeB` `head` `shoulder` `body` `hip` `weapon`. Apelidos: `handR`/`handL`, `footR`/`footL`, `fist`, `blade`, `sword`, `tip`.

## Efeitos

Um quadro com `effect` desenha um efeito no lugar do corpo. Use em projéteis e faíscas.

```json
{ "effect": { "type": "slash", "size": 26, "color": "#ff3a5c", "color2": "#ffd0d8", "phase": 0 }, "ticks": 3 }
```

| `type` | Formato |
|---|---|
| `orb` (padrão), `burst` | Bola de energia / explosão |
| `dragon` | Dragão de energia voltado para frente, com corpo em movimento |
| `beam` (`length`) | Raio horizontal |
| `pillar` (`length`) | Coluna vertical |
| `shockwave` | Onda no chão |
| `slash`, `wave`, `water` | Corte em arco |
| `kunai`, `shuriken` | Arremessáveis |
| `ice`, `fire`, `darkfire`, `lightning`, `petal` | Elementos |

Sem `color`, o efeito usa a cor `energy` da paleta. A caixa de acerto do efeito é gerada sozinha a partir do formato.
