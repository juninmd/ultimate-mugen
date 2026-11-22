# Ability library

Reusable blocks for UMF fighters. A converted or hand-written fighter should name an ability and tune it, not re-write the controllers. The blocks are engine features (data fields) plus a built-in kit (`abilities`) with ready-made moves.

## Gap map: what a M.U.G.E.N pack does and where UMF covers it

| Pattern in the pack | UMF block | Status |
|---|---|---|
| Fireball helper, `Projectile` | `projectile` | existing |
| Ki volley (several `Projectile` controllers, one per frame) | `projectile.repeat` | new |
| Teleport behind or to the rival (`PosAdd p2dist`, `Turn`) | `blink` (`behind`, `front`, `above`, `back`) and the older `teleport` | new |
| Charge flash, aura burst, `Explod` of an animation | `vfx` | new |
| `EnvShake` outside a hit | `quake` | new |
| Beam with many hits (`projhits`) | `projectile.hits` | existing |
| Flight (`physics = N`), clone helper, form (`var` flag) | `abilities.fly`, `helpers.clones`, `abilities.forms` | existing |
| Rush with several `HitDef`s | `hits[]` plus `moves[].vel` | existing |
| Helper with its own AI, `Statedef -2/-3` logic, Lua/ZSS | `scripts`, or stay in the legacy format | not migrated |

## Move fields

```json
{ "id": "salvo", "kind": "special", "anim": "arcbolt", "cost": 250,
  "blink": { "at": 2, "to": "behind", "gap": 36 },
  "vfx": [{ "at": 1, "anim": "boltHit", "offset": [0, -50], "time": 12, "follow": true, "scale": 1.5 }],
  "quake": [{ "at": 3, "time": 12, "amp": 2 }],
  "projectile": { "at": 3, "anim": "boltFx", "speed": [6, 0], "offset": [44, -78], "damage": 22,
                  "repeat": { "count": 5, "every": 1, "offsetStep": [0, 8], "speedStep": [0.4, 0], "damageStep": 0 } } }
```

| Field | Meaning |
|---|---|
| `blink` (object or list) | `at` frame, `to` = `behind` (turns around, default), `front`, `above`, `back`; `gap` distance in units (default 40); `y` vertical offset |
| `vfx[]` | An `Explod` of an animation of the fighter on frame `at`: `offset` (default `[0, -50]`), `time` ticks (default: until the animation ends), `follow` binds it to the fighter, `scale`, `behind` draws it under the fighter |
| `quake[]` | `EnvShake` on frame `at`: `time` (8), `amp` (4) |
| `projectile.repeat` | Fans one projectile into `count` (max 24): the k-th fires on frame `at + k * every` (default 2), shifted by `k * offsetStep`, `k * speedStep` and `k * damageStep`. A volley longer than the animation fires its tail on the last frame, so a fan needs no extra frames |

`vfx` reuses animations the fighter already has (PNG frames). The engine never draws a frame in code.

## Built-in kit `abilities`

A library kit: `"extends": ["abilities"]` adds nothing by itself; only the moves you list (by id or with `use`) enter the fighter. Moves: `ki_blast`, `ki_barrage`, `beam`, `instant_transmission`, `blink_strike`, `power_surge`. Each ships with sane numbers; change only what differs. `projectile` and `blink` merge field by field:

```json
{ "extends": ["abilities"],
  "moves": [
    { "id": "volt_blink", "use": "instant_transmission", "name": "Volt Blink", "anim": "overclock", "input": "B, F, x", "blink": { "gap": 36 } },
    { "id": "arc_salvo", "use": "ki_barrage", "input": "~D, DF, F, a", "anim": "arcbolt",
      "projectile": { "anim": "boltFx", "hitAnim": "boltHit", "element": "lightning", "damage": 22 } }
  ] }
```

Always override `anim` (and `projectile.anim`) with the fighter's own animations: the template defaults use `strong` so the kit stays valid for every roster fighter. Tools that read a roster file without `parse_char` (some tests) do not understand `extends`; the roster fighters write the blocks out, and kits fit packs and mods.

[Volta](Characters.md) shows the data fields in `assets/chars/volta.json` (`volt_blink`, `arc_salvo`).

## Converter

`tools/mugen2umf` maps `Explod` to `vfx` and `EnvShake` to `quake` (at most 3 and 2 per move), and matches `command` names case-insensitively, which MUGEN does and many packs rely on (`SPECIAL 1` in the `.cmd`, `"special 1"` in the trigger). Before that fix a pack with such names kept only its normals: Goku SS4 went from 5 to 15 moves.

## Rules of thumb

- Repeating the same controllers in two moves is a missing block: add a field or a kit move instead.
- A new block needs a unit test of the expansion and an engine test that runs it (`crates/engine/tests/native_ability_blocks.rs`).
- Check every `at` against the animation length; the compiler does not stretch an animation.
