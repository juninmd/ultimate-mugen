# Combo strings

Every native fighter (the 16 roster fighters and the 5 Story NPCs) has a signature three-part string on top of the shared gatling rules. Moves and animations are data only: each `string_*` animation is composed from the fighter's own jab, kick and strong frames (same poses, same hit parts, so sword users swing the weapon and brawlers use fists) with a per-fighter speed, hit count and recovery.

| Input | Move id | Role |
|---|---|---|
| `B+x` | `string_a` | Opener. Also reachable by cancelling `jab` on contact |
| `B+a` | `string_b` | Middle link. Cancel from `string_a` on contact |
| `B+y` | `string_c` | Finisher. Launch, wall bounce or ground bounce depending on the fighter; heavy fighters get one hit of armor |

`string_a` and `string_b` cancel into `string_c` and into the fighter's two signature specials on contact, so every fighter has its own route. Examples:

| Fighter | Style | Route |
|---|---|---|
| Akio, Jin, Suzume, Phantom, Razor | Rushdown: three to four fast light hits, long advance | `string_a` > `string_b` > `string_c` > starburst/chakram/volley |
| Bruno, Brute, Colossus, Valdric | Heavy: two slow hits, wall bounce finisher with armor | `string_a` > `string_c` > quake/ridgeblast/royallance |
| Kaname, Shion, Marlow, Oboro | Blade: weapon strikes, ice/water element where it fits | `string_a` > `string_b` > frost arc/crescent/riptide |
| Garan, Sylene, Kaito, Volta, Spitfire | Elemental: every hit carries fire, ice, wind or lightning stun | `string_b` > `string_c` > element special |
| Garan, Lian, Jin, Suzume, Phantom, Razor | Ground-bounce finisher for a follow-up juggle | `string_c` |

Validated by `cargo test --release -p ume-engine --test it native_combo_routes::` (every declared cancel chains on contact), `native_roster_lab` and `native_hitbox_audit`.

## Same-button mini combos

Pressing the same attack button again after a hit chains into the next move, with no extra direction needed. Each chain move is a `normal` with a `rekka` gate (`after`, `window`, `contact: true`), so it only fires from its source on contact and never from neutral. The chain animations reuse the fighter's existing sprites (same pixel art) in new orders and faster timings.

| Press | Chain |
|---|---|
| `x` | `jab` > `jab2` > `jab3` (finisher with small pushback, cancels into the two signature specials) |
| `a` | `lkick` > `lkick2` > `lkick3` |
| `y` | `strong` > `strong2` |
| crouch `x` | `cjab` > `cjab2` |
| air `x` | `ajab` > `ajab2` |

Fighter `speed` (> 1 shortens every timed frame) and forward walk/run velocity were raised roster-wide; the final boss is scaled with it so it stays faster. Back-walk speeds are unchanged. Validated by `cargo test --release -p ume-engine --test it native_same_button_chains::`.

## Projectile variants

Every roster fighter has a slow heavy and a fast low variant of its signature projectile, reusing the same effects: `~D, DB, B, y` (slow, two hits, long range) and `~D, DB, B, x` (fast, low, hits crouching foes). Fighters whose input is already taken use the first free of `~D, DF, F, y` and `~D, DF, F, a`.
