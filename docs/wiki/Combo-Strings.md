# Combo strings

Every native fighter (the 16 roster fighters and the 5 Story NPCs) has a signature three-part string on top of the shared gatling rules. Moves and animations are data only: each `string_*` animation is composed from the fighter's own jab, kick and strong frames (same poses, same hit parts, so sword users swing the weapon and brawlers use fists) with a per-fighter speed, hit count and recovery.

| Input | Move id | Role |
|---|---|---|
| `F+x` | `string_a` | Opener. Also reachable by cancelling `jab` on contact |
| `B+x` | `string_b` | Middle link. Cancel from `string_a` on contact |
| `B+y` | `string_c` | Finisher. Launch, wall bounce or ground bounce depending on the fighter; heavy fighters get one hit of armor |

`string_a` and `string_b` cancel into `string_c` and into the fighter's two signature specials on contact, so every fighter has its own route. Examples:

| Fighter | Style | Route |
|---|---|---|
| Akio, Jin, Suzume, Phantom, Razor | Rushdown: three to four fast light hits, long advance | `string_a` > `string_b` > `string_c` > starburst/chakram/volley |
| Bruno, Brute, Colossus, Valdric | Heavy: two slow hits, wall bounce finisher with armor | `string_a` > `string_c` > quake/ridgeblast/royallance |
| Kaname, Shion, Marlow, Oboro | Blade: weapon strikes, ice/water element where it fits | `string_a` > `string_b` > frost arc/crescent/riptide |
| Garan, Sylene, Kaito, Volta, Spitfire | Elemental: every hit carries fire, ice, wind or lightning stun | `string_b` > `string_c` > element special |
| Garan, Lian, Jin, Suzume, Phantom, Razor | Ground-bounce finisher for a follow-up juggle | `string_c` |

Validated by `cargo test --release -p ume-engine --test native_combo_routes` (every declared cancel chains on contact), `native_roster_lab` and `native_hitbox_audit`.
