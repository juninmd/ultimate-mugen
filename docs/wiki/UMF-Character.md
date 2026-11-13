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
  "awakeningPalette": { "hair": "#f6c83e" },
  "anims": {},
  "moves": []
}
```

| Field | Required | Description |
|---|---|---|
| `format` | yes | Always `umf/1` |
| `id`, `name`, `series`, `author`, `color` | yes | Identity; `color` is the select card color |
| `boss` | no | `true` hides it from select and makes it the final Arcade boss |
| `npc` | no | `true` hides it from select and every random pool; used by Story mode thugs, no boss empowerment |
| `speed` | no | Animation speed multiplier (>1 = faster moves) |
| `stats` | no | Attributes, table below |
| `art` | yes | Art, see [Rig](Rig.md) |
| `palettes` | no | Alternate colors; each item overrides keys of `art.style.colors` |
| `awakeningPalette` | no | Color overrides applied on top of the selected palette while awakened; effect sprites are unchanged |
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

#### How an attack frame becomes a hit box

Frames drawn from pixel sprites take their strike box from the rig geometry of the frame's `pose` and `hit` (`part` + `radius`), or from the explicit `box` when given, so every move keeps its own reach. The compiler then reshapes it to the proportions measured on 605 M.U.G.E.N characters (`native/pixel_hit.rs`, `H` = standing height, 96 units for `height` 100):

- like a M.U.G.E.N `Clsn1`, the box spans from the body to the tip: it starts at most `0.10 H` in front of the origin and reaches at least `0.60 H` (the M.U.G.E.N median for punches is `0.72 H`, kicks `0.80 H`);
- at least `0.36 H` tall and never above `0.95 H` or below the floor (M.U.G.E.N punches sit between `-0.75 H` and `-0.33 H`, kicks down to `-0.05 H`);
- the standing hurtbox is `0.46 H` wide (M.U.G.E.N median `0.46 H`) and the crouching one `0.67 H` tall (M.U.G.E.N median `0.67 H`);
- an explicit `box` is used as written, so keep it inside those proportions; the audit fails otherwise.

`cargo test --release -p ume-engine --test native_hitbox_audit` checks every strike frame of the roster (height, reach, start, width, above the head, under the floor), and `native_normal_reach` prints and compares the reach and damage of the four stand normals of every fighter.

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
| `voice` | `1000, 0` | Character sound group and index played at move start, when that sample is bundled |
| `airPhysics` | `true` | Keeps air physics |
| `cancels` | list of move ids | This move may be cancelled **into** those moves on contact (hit, guard or projectile contact). Declaring it replaces the automatic gatling out of this move |
| `throw` | `{ "damage", "bind", "launch", "at" }` | Grab: see [Combat traits](#combat-traits) |
| `armor` | `{ "hits", "threshold" }` | Absorbs hits while the move is active |
| `parry` | `{ "window": [from, to], "spark" }` | Returns attacks during the animation-element window |
| `timestop` | ticks | Freezes everyone else |
| `burst` | `true` | Combo breaker usable while being hit |
| `requires` | `{ "life": {"max": 0.5}, "enemyLife": {"min": 0.1, "max": 0.5} }` | Life condition: the move only starts while the fighter's life (`life`) and/or the rival's (`enemyLife`) is inside `min`..`max`, as a fraction of max life (0 to 1). Either bound may be omitted. Compiles to `life <= lifemax * max` and `enemy,life <= enemy,lifemax * max` triggers |
| `flight` | `true` | 8-way hover while the move lasts |

**Finisher.** A move with `"kind": "finisher"` (use `throw` or `hits` for the effect) is only available after a KO (`RoundState = 3` and `WinKO`). When the final round ends by KO and the winner has a finisher, the round waits up to 6 seconds on "FINISH HIM" for the winner to perform it; performing it (or the timeout) ends the wait. Rosters with no finisher are unaffected.

**Ultimate.** A move whose `id` starts with `ult_` is the fighter's ultimate, and every native fighter declares one (the command is `~D, DF, F, D, DF, F, x+y`). The compiler spends the whole meter on it: `cost` defaults to `3000` (the three bars drawn by the HUD), the move gets a 70 % damage bonus where a normal `super` gets 30 %, and its super freeze lasts 75 ticks instead of 55. `cargo test --release -p ume-engine --test native_ultimates` fails when a fighter has no ultimate, when the cost is not three bars, when the ultimate does not start from its command or drains the meter, or when the measured damage is below 380 or not above every one of the fighter's supers.

### Combat traits

- **Automatic gatling.** A move without `cancels` follows the genre rules: a normal cancels into any *stronger* normal (`light` < `medium` < `heavy` by the strongest hit level, ties by declaration order, so strings can never loop), ground normals link into crouching ones, air normals chain with air normals, and every normal cancels into every special and super on contact. Specials cancel into other specials only through explicit `cancels`; supers cancel out of any move.
- **Explicit routes.** `"cancels": ["strong", "rush"]` on `jab` makes `strong` and `rush` accept `jab` on contact. A projectile special can be cancelled once its projectile touches the opponent. An `air` special may start from a ground launcher that lists it (uppercut, then dive).
- **`throw`** (`damage`, `bind` `[x, y]` in front of the thrower, `launch` `[vx, vy]`, `at` = grab frame, default 1). The grab HitDef fires on animation element `at` (give that frame a `hit` box), the victim is held on `at + 1`, takes the damage on `at + 2` and is launched, so the animation needs at least `at + 2` frames. Throws ignore guarding and whiff when the target is out of reach.
- **`armor`** (`hits` absorbed, optional `threshold` = damage above which the armor breaks). From move start until the last hit frame the move takes 35 % chip damage and is not interrupted; once `hits` are spent the next hit staggers normally.
- **Multi-hit glue.** Moves with three or more `hits` push the target away only a little on every hit except the last (so the whole string connects) and piercing projectiles (4+ hits) do not shove the target out of the beam. Set `knockback` explicitly to opt out.

Validate with `cargo test --release --test native_roster_lab -- --nocapture` (every attack state must connect; prints damage and reach per move), `--test native_combo_routes` (every declared cancel chains end to end) and `python scripts/audit_native_moves.py`.

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

### Elements and stun

`hits[]` and `projectile` accept `element` (`fire`, `ice`, `lightning`, `earth`, `wind`, `water`) and `stun` (extra hit-stun ticks). The compiler adds the element's hit-stun and shake and tints the victim for a moment (`PalFX`): `ice` freezes longest (+16), `lightning` shocks (+22 and a 8-tick shake), `fire` burns red, `earth` shakes the screen, `wind` and `water` add a short tint. `stun` stacks on top and adds a 5-tick shake when none is set. Unknown elements are ignored.

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

## Power normals (Punch 3 and Kick 3)

Every native fighter that does not declare a move bound to `z` or `c` gets four extra normals at compile time (`native/power_normals.rs`): `power_punch` and `crouch_power_punch` on `z` (default key **E**, shown as *Punch 3*), `power_kick` and `crouch_power_kick` on `c` (default key **D**, shown as *Kick 3*). Each one reuses the animation, timing, guard height and `cancels` of the fighter's heaviest ground normal of that family (`y` then `x` for punches, `b` then `a` for kicks, crouching variants take the crouching normal), named `<base name> Max`. Hits deal 130 % damage, are promoted to `heavy`, gain 4 hit-stun and 2 hit-stop ticks when those are set, and push 25 % harder. Moves that cancel into the base normal also cancel into its power version, and the new moves appear in the pause move list. Declare your own `z` or `c` move to opt out. Air versions are not generated here.

All native fighters also receive two down-light-kick normals at compile time (`native/low_normals.rs`): `lowkick` on `D+a` while crouching and `airlowkick` on `D+a` in the air. They use the character's existing crouch-light-kick and air-kick animations, respectively, keep their pixel-art frames, and use low guard. The aerial version moves forward and downward to close short gaps. A fighter that already declares a move with the same id or `D+a` state/input keeps its authored move.

## Energy charge (every fighter)

Every native fighter can charge its meter: hold **R** (the `s` / Start input; the pause key in a fight is **Esc** or **P**). The fighter enters state `3920`, plays its `powerUp` animation on a loop with a blue glow and gains 4 power per tick (a 1000-power super in about four seconds) until the button is released (after at least 6 ticks) or the meter is full. The state is `movetype = I`, so a hit interrupts it, and a form-capable fighter still transforms with `T` once the meter reaches the form cost. The command is `charge_cmd` (`/s`) and shows as `Charge Energy` in the move list. The built-in AI charges only at level 3 or above, with less than 700 power, when the rival is more than 190 units away and is not attacking.

## Air abilities (every special and super)

Every `special` and `super` with `when` unset or `"stand"` also gets an air version (state `20000 + ground state`, label `State -1, air <name>`), entered with the same command while the fighter is airborne. The fighter hovers at the start of the move (drift kept at 40 %), gravity returns at 45 % after 12 ticks, the move ends into the jump fall state (`50`) with control, and touching the ground interrupts it with the landing state (`52`). Air versions deal 70 % of the ground damage, projectiles leave angled down (slope 0.85) so they reach a grounded rival, and each jump allows one air special (`var(57)`, cleared on landing). Throws, flight moves and moves that declare `when: "crouch"` or `"air"` keep their own rules.

## Pixel-art sprites (optional)

If `assets/chars/sprites/<id>/` exists, the fighter is drawn from PNG files instead of the rig. Use transparent PNGs facing right, with the feet on the bottom edge and the same canvas size for every file of one fighter. A pose without a PNG falls back to `01_idle.png`; a fighter without the folder keeps its rig. After extending the rig, run `cargo run --release -p ume-engine --example generate_missing_sprites` to render a dedicated PNG for every pose the loader would otherwise take from the idle canvas.

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

When `awakeningPalette` overrides `hair`, each sprite's `frames.<stem>.awakeningHair` supplies `[left, top, right, bottom]` in source pixels. Hair recoloring is confined to that rectangle; sprites without one retain their original hair. Web packing rescales these coordinates with the sprite.

Palette entries also apply to these PNGs: each pixel is matched to the shading line of a colour declared in `art.style.colors`, and the matching palette colour replaces it at the same shade. A palette key with no matching `art.style.colors` entry, or a colour nothing in the art resembles, is left untouched.

The web build does not read this folder from disk. `scripts/build-web.sh` runs `ume-tools pack-sprites` (`crates/devtools`), which writes one `assets/chars/sprites/<id>.pack` per fighter (images capped at 768 px, 256 colours, `meta.json` anchors rescaled) into `web/dist`; the browser fetches a pack when the fighter is focused on the select screen or loaded for the fight, and the engine reads it through the same virtual folder. A failed download keeps the rig.

## Validate

```bash
cargo test --workspace --release
cargo run --release -p ume-engine --example rig_sheet
```

`rig_sheet` generates a sheet with every pose so you can review the art without opening the game.

Browser sprite packs load asynchronously. A newly downloaded or replaced pack invalidates the affected native fighter’s cached previews and all palettes; other fighters keep their caches. Match loading waits for the chosen built-in fighters’ packs and reports download failures rather than starting with a cached procedural fallback. Imported native definitions without a bundled pack remain supported.
