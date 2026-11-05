# Creating a UMF character

UMF (*Ultimate Mugen Format*) describes a whole fighter in **one JSON file**. The compiler (`crates/engine/src/native/compile.rs`) generates commands, states, HitDefs, projectiles, super pause, cancels and hitboxes. The result runs on the same state machine as M.U.G.E.N characters.

## Step by step

1. Copy a finished fighter as a base: `cp assets/chars/kaito.json assets/chars/mine.json`.
2. Change `id` (unique, lowercase, no spaces), `name`, `series`, `author` and `color`.
3. Optional: add the `id` to `assets/chars/index.json` to set its order in the select screen. Missing ones follow alphabetically.
4. Run `cargo run --release`. `build.rs` embeds every `.json` in the folder into the executable.
5. Test in **Training** with `F2` on to see hitboxes and hurtboxes.

Loading a UMF fighter from a folder without recompiling is **planned** (the registry already has `add_native_json`; it only needs to be wired to the `mugen/` folder).

## File skeleton

```json
{
  "format": "umf/1",
  "id": "mine",
  "name": "My Fighter",
  "series": "My Series",
  "author": "you",
  "description": "Select screen text",
  "color": "#e85d2a",
  "winQuote": "Victory line",
  "intro": "Line said at round 1 start",
  "taunt": "Reply/provocation",
  "form": "Awakened form name",
  "boss": false,
  "speed": 1.0,
  "stats": {},
  "art": { "type": "rig", "style": {}, "poses": {} },
  "palettes": [],
  "anims": {},
  "moves": []
}
```

| Field | Required | Description |
|---|---|---|
| `format` | yes | Always `umf/1` |
| `id`, `name`, `series`, `author`, `color` | yes | Identity; `color` is the select card color |
| `boss` | no | `true` hides it from select and makes it the final Arcade boss |
| `speed` | no | Animation speed multiplier (>1 = faster moves) |
| `stats` | no | Attributes, table below |
| `art` | yes | Art, see [Rig](Rig.md) |
| `palettes` | no | Alternate colors; each item overrides keys of `art.style.colors` |
| `anims` | yes | Animations by name |
| `moves` | yes | Moves |

## `stats`

| Field | Example | Meaning |
|---|---|---|
| `life` | `1000` | Life |
| `power` | `3000` | Max super meter (1000 per level) |
| `attack` / `defense` | `100` | Multipliers in % |
| `walk` | `[2.4, 2.2]` | Forward / backward walking speed |
| `run` | `[5.2, 4.6]` | Run forward / back hop |
| `jump` | `[-8.4, -2.5, 0, 2.6]` | Vertical speed, horizontal back, neutral and forward |
| `gravity` | — | Vertical acceleration |
| `airJumps` | — | Extra air jumps |
| `height` | `100` | Body height |
| `width` | `[16, 15]` | Front / back width for pushing |

## `anims`

Each animation is a list of frames:

```json
"jab": { "frames": [
  { "pose": "jab0", "ticks": 3 },
  { "pose": "jab1", "ticks": 3, "hit": { "part": "handR", "radius": 10 } },
  { "pose": "jab0", "ticks": 5 }
], "loop": 0 }
```

| Frame field | Description |
|---|---|
| `pose` | Name of a pose in `art.poses`, or an inline pose object |
| `ticks` | Duration in ticks (60 per second); `-1` = forever |
| `hit` | Active hitbox: `part` (limb or weapon) + `radius`, or `box` `[x1, y1, x2, y2]` |
| `hurt` | `"none"` removes the hurtbox (invincible frame) |
| `effect` | Draws an effect instead of the body (projectiles, sparks), see [Rig](Rig.md#effects) |
| `offset` | Drawing offset `[x, y]` |
| `loop` | Frame index where the loop restarts |

### Standard names

Animations with these names become the reserved M.U.G.E.N actions, so the common states work with no extra code.

| Name | Action | Name | Action |
|---|---|---|---|
| `idle` | 0 | `guardHit` | 150 |
| `turn` | 5 | `crouchGuardHit` | 152 |
| `crouchTurn` | 6 | `airGuardHit` | 154 |
| `crouchStart` / `crouch` / `crouchEnd` | 10 / 11 / 12 | `lose` / `draw` / `win` | 170 / 175 / 181 |
| `walk` / `walkBack` | 20 / 21 | `intro` | 190 |
| `jumpStart` / `jumpUp` / `jumpFwd` / `jumpBack` / `jumpDown` | 40–44 | `taunt` / `powerUp` | 195 / 196 |
| `land` | 47 | `defeat1`…`defeat4` | 197–200 |
| `run` / `hop` | 100 / 105 | `hitLight` / `hitMedium` / `hitHeavy` | 5000–5002 |
| `guardStart` / `crouchGuardStart` / `airGuardStart` | 120–122 | `hitLowLight` / `hitLowMedium` / `hitLowHeavy` | 5010–5012 |
| `guard` / `crouchGuard` / `airGuard` | 130–132 | `hitCrouch` / `hitAir` | 5020 / 5030 |
| `guardEnd` | 140 | `airRecover` / `fall` / `trip` | 5040 / 5050 / 5070 |
| | | `bounce` / `bounceAir` / `lieDown` / `getUp` / `dead` | 5100 / 5101 / 5110 / 5120 / 5150 |

`taunt`, `powerUp` and `defeat*` have default versions if you do not define them. Other names get free numbers in file order.

## `moves`

```json
{ "id": "gale", "name": "Wind Wave", "input": "~D, DF, F, x", "simple": "sp1",
  "kind": "special", "anim": "gale", "invuln": [1, 3],
  "moves": [{ "at": 2, "vel": [2, -9] }],
  "hits": [{ "at": 2, "damage": 90, "level": "heavy", "launch": true }],
  "projectile": { "at": 3, "anim": "galeOrb", "hitAnim": "galeHit", "speed": [5, 0], "offset": [40, -60], "damage": 70 } }
```

| Field | Values | Effect |
|---|---|---|
| `id`, `name` | text | Identifier and name in the move list |
| `input` | command syntax, below | Command |
| `simple` | `sp1` `sp2` `sp3` `super` | One-button shortcut (keys 1/2/3, touch) |
| `kind` | `normal` `special` `super` | Priority and cancels: normal → special → super on hit or guard |
| `when` | `stand` (default) `crouch` `air` | State the move starts from |
| `anim` | name in `anims` | Move animation |
| `hits` | list | One HitDef per item |
| `moves` | `[{ "at": frame, "vel": [x, y] }]` | `VelSet` on the frame; use `null` to keep an axis |
| `projectile` | object or list | Projectiles |
| `cost` | number | Meter spent; `super` defaults to 1000 |
| `invuln` | `[start, end]` | Invincible frames |
| `teleport` | frame | Reappears behind the opponent |
| `fx` | `aura` | Extra visual effect |
| `airPhysics` | `true` | Keeps air physics |

### `hits[]`

| Field | Default | Description |
|---|---|---|
| `at` | — | Frame (1 = first) where the HitDef activates |
| `damage` | — | Damage |
| `guardDamage` | `damage / 8` | Damage on block |
| `level` | `medium` | `light` (hitstop 9, hitstun 12), `medium` (11, 16), `heavy` (13, 20) |
| `guard` | `mid` | `high`, `low`, `mid` or `unblockable` |
| `knockback` / `airKnockback` | by level | Velocity `[x, y]` applied to the opponent |
| `launch` | `false` | Knocks down |
| `hitstop`, `hitstun`, `spark`, `shake` | by level | Fine tuning |

### `projectile`

| Field | Description |
|---|---|
| `at` | Frame it is fired on |
| `anim` / `hitAnim` | Flight / impact animation |
| `speed` / `accel` | Velocity and acceleration `[x, y]` |
| `offset` | Spawn position `[x, y]` relative to the fighter |
| `damage`, `level`, `launch`, `knockback` | As in `hits` |
| `hits` | Number of hits |
| `removeTime` | Ticks until it disappears |
| `priority` | Priority against other projectiles |

### Command syntax

| Symbol | Meaning | Example |
|---|---|---|
| `F B U D` / `DF DB UF UB` | Directions (relative to facing) | `D, DF, F, x` |
| `x y z a b c` | Buttons (`s` = start) | `x` |
| `,` | In sequence | `F, F` |
| `+` | At the same time | `x+y` |
| `~` | Release | `~D, DF, F, x` |
| `/` | Hold | `/B, F, x` |
| `$` | Inclusive direction (`$D` accepts DB, D, DF) | `$D, x` |
| `>` | No other input between steps | `F, >F` |
| `N` | Neutral direction (no direction held) | `N, x` |

## Pixel-art sprites (optional)

If `assets/chars/sprites/<id>/` exists, the fighter is drawn from PNG files instead of the rig. Use transparent PNGs facing right, with the feet on the bottom edge and the same canvas size for every file of one fighter. A pose without a PNG falls back to `01_idle.png`; a fighter without the folder keeps its rig.

| File | Poses that use it |
|---|---|
| `01_idle` | `idle0`, walk, run, crouch, guard, jump and every pose with no art of its own |
| `02_attack1` … `06_victory` | jab / punch, kick, special, projectile and victory / taunt poses |
| `07_idle1`, `08_idle2` | the other breathing frames of `idle` (`idle3` reuses `07_idle1`) |
| `09_hit1`, `10_hit2`, `11_hit3` | `hitH1`, `hitH2`, `hitH3` (damage) |
| `12_fall1`, `13_fall2` | `hitAir1`/`fall1`/`trip` and `hitAir2`/`fall2`/`bounce` (knocked into the air) |
| `14_lie` | `lie`, `dead`, `lose` and every `df*` pose (knocked out, with a low and wide hurt box) |
| `15_power0`, `16_power1`, `17_power2` | `powerup0`, `powerup1`, `powerup2` (transformation, key `T`) |

`meta.json` holds `{"refHeight": N}`: the height in pixels of the standing body in `01_idle.png`. Every pose of the fighter is scaled so that body is 100 units tall, so a lying or crouching sprite keeps the same pixel size. Without it each PNG is scaled to 100 units by its own height.

Palette entries also apply to these PNGs: each pixel is matched to the shading line of a colour declared in `art.style.colors`, and the matching palette colour replaces it at the same shade. A palette key with no matching `art.style.colors` entry, or a colour nothing in the art resembles, is left untouched.

The web build does not read this folder from disk. `scripts/build-web.sh` runs `scripts/pack_sprites.py`, which writes one `assets/chars/sprites/<id>.pack` per fighter (images capped at 768 px, 256 colours, `meta.json` anchors rescaled) into `web/dist`; the browser fetches a pack when the fighter is focused on the select screen or loaded for the fight, and the engine reads it through the same virtual folder. A failed download keeps the rig.

## Validate

```bash
cargo test --workspace --release
cargo run --release -p ume-engine --example rig_sheet
```

`rig_sheet` generates a sheet with every pose so you can review the art without opening the game.
