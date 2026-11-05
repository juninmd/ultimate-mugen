# Skeleton art (rig)

With `"art": { "type": "rig" }` the fighter is drawn as vectors at load time (`crates/engine/src/native/rig.rs`). It stays sharp at any resolution and every palette is free.

## `style`: appearance

| Key | Values found in the bundled fighters |
|---|---|
| `build`, `muscle`, `arms`, `waist`, `scale` | Proportions (1.0 = default) |
| `female` | `true` / `false` |
| `torsoShape` | `heavy` |
| `top` | `gi` `bare` `tank` `jacket` `coat` `robe` `suit` `qipao` `armor` |
| `sleeves` | `none` `short` `long` `wide` `gi` |
| `pants` | `gi` `baggy` `tight` `hakama` `trunks` |
| `skirt` | `coat` `robe` |
| `belt` | `true` or color: `belt` `cloth` `metal` `accent` `top2` |
| `gloves` | `wrap` `glove` `long` `gauntlet` |
| `boots` | `barefoot` `shoe` `boot` `tall` `tabi` `wrap` `greave` |
| `hair` | Object `{ "type": "spiky", ... }`; types `short` `spiky` `long` `ponytail` `topknot` `mane` `wild` `helmet` |
| `eyes` | `normal` `fierce` `cute` `glow` |
| `weapon` | `katana` `greatsword` `cutlass` `kunai` |
| `emblem` | `{ "type": "bolt", "color": "#f2b233", "where": "back", "size": 1.3 }`; types `bolt` `crest` `glyph` `lotus` `moon` `star` `wave` |
| `weave`, `pantsWeave` | Fabric texture: `weave` `check` `dots` `hatch` `quilt` `rib` `stripes` |
| Accessories | `headband` `scarf` `cape` `sash` `mask` `glasses` `goggles` `hat` `horns` `crown` `earring` `necklace` `pauldron` `shoulderPads` `pouches` `tattoo` `scars` `bandages` `beard` `facePaint` `eyepatch` `talisman` |
| Details | `veins` `knuckles` `stitch` `folds` `faceShade` `brow` `shine` |
| `colors` | Map of named colors, below |

### `colors`

Keys in use: `skin` `hair` `eye` `lips` `top` `top2` `trim` `pants` `belt` `glove` `boot` `wrap` `cloth` `accent` `metal` `weapon` `weapon2` `hilt` `visor` `mask` `hat` `horn` `undershirt` `legs` `beard` `aura` `energy`.

`palettes` overrides any of them:

```json
"palettes": [
  { "top": "#b8433a", "trim": "#6a1e1a", "hair": "#1a1a22" },
  { "top": "#e8e2d2", "skin": "#f0c8a0" }
]
```

## `poses`: joint angles

```json
"poses": {
  "idle0": { "shoulderF": 42, "elbowF": 102, "shoulderB": 22, "elbowB": 120,
             "thighF": 22, "kneeF": 26, "thighB": -18, "kneeB": 12, "lean": 6, "head": -4 }
}
```

`F` = front limb, `B` = back limb. Angles in degrees.

| Key | Description |
|---|---|
| `shoulderF/B`, `elbowF/B`, `wristF/B` | Arms |
| `thighF/B`, `kneeF/B`, `ankleF/B` | Legs |
| `lean`, `head`, `rot` | Torso lean, head angle and whole-body rotation |
| `y`, `hipH` | Hip height |
| `handF/B` | `fist` `open` |
| `face` | `smile` `shout` `hurt` `ko` ... |
| `blade`, `weapon`, `noWeapon` | Weapon angle and visibility |
| `aura`, `glow`, `charge`, `chargeSize` | Energy glows |
| `wind`, `hairWind`, `phase`, `free` | Cloth/hair wind and secondary animation phase |

Parts usable in `hit.part`: `handF` `handB` `footF` `footB` `heelF` `heelB` `elbowF` `elbowB` `kneeF` `kneeB` `head` `shoulder` `body` `hip` `weapon`. Aliases: `handR`/`handL`, `footR`/`footL`, `fist`, `blade`, `sword`, `tip`.

## Effects

A frame with `effect` draws an effect in place of the body. Use it for projectiles and sparks.

```json
{ "effect": { "type": "slash", "size": 26, "color": "#ff3a5c", "color2": "#ffd0d8", "phase": 0 }, "ticks": 3 }
```

| `type` | Shape |
|---|---|
| `orb` (default), `burst` | Energy ball / explosion |
| `dragon` | Forward-facing energy dragon with a trailing body |
| `beam` (`length`) | Horizontal ray |
| `pillar` (`length`) | Vertical column |
| `shockwave` | Ground wave |
| `slash`, `wave`, `water` | Arc cut |
| `kunai`, `shuriken` | Throwables |
| `ice`, `fire`, `darkfire`, `lightning`, `petal` | Elements |

Without `color`, the effect uses the palette's `energy` color. The effect hitbox is generated from its shape.
