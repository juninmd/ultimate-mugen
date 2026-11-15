# Three-bar ultimates for the native roster

Every UMF fighter now declares exactly one ultimate: a `super` move whose id starts with `ult_`,
mapped to `~D, DF, F, D, DF, F, x+y`. The compiler treats it as a finisher:

- `cost` defaults to `3000`, which is the whole meter and all three HUD cells (`docs/wiki/UMF-Character.md`).
- Hit and projectile damage gets a 70 % bonus instead of the 30 % a normal `super` receives, and
  ultimate throws are scaled the same way (`crates/engine/src/native/throw.rs`).
- The super freeze lasts 75 ticks instead of 55 (`crates/engine/src/native/ultimate.rs`).
- Ultimate states are ranked above regular supers in `[Statedef -1]`.

## Measured damage

`cargo test --release -p ume-engine --test native_ultimates -- --nocapture` drives each ultimate
from its real command on a full meter, at six distances, against a mirror opponent, and compares
it with every one-bar super of the same fighter. Damage below is the best of the six distances;
the test prints the whole spread, so a projectile ultimate reads lower at point blank and reaches
its full value at its intended spacing.

| Fighter | Ultimate | Damage | Best 1-bar super | Ratio |
|---|---|---:|---:|---:|
| akio | Supernova Barrage | 467 | 364 | 1.28 |
| bruno | Bulldozer Apocalypse | 464 | 273 | 1.70 |
| daichi | Mountain Spirit Ultimate | 467 | 347 | 1.35 |
| garan | Gate of Ash Ultimate | 505 | 340 | 1.49 |
| haruki | Unwritten Final Chapter | 477 | 303 | 1.57 |
| jin | Hundred Gale Phantom | 653 | 417 | 1.57 |
| kaito | Storm Eye Ultimate | 600 | 333 | 1.80 |
| kaname | Winter Blade Ultimate | 736 | 468 | 1.57 |
| lian | Jade Lantern Ultimate | 561 | 356 | 1.58 |
| marlow | Leviathan Maelstrom | 769 | 489 | 1.57 |
| npc_brute | Bulldozer Apocalypse | 421 | 219 | 1.92 |
| npc_colossus | Mountain Spirit Ultimate | 490 | 323 | 1.52 |
| npc_phantom | Hundred Gale Phantom | 437 | 340 | 1.29 |
| npc_razor | Lótus do Julgamento: Mil Sombras | 460 | 367 | 1.25 |
| npc_spitfire | Supernova Barrage | 433 | 292 | 1.48 |
| oboro | Susanoo: Eclipse Absoluto | 1241 | 568 | 2.18 |
| shion | Black Moon Ultimate | 507 | 322 | 1.57 |
| suzume | Lótus do Julgamento: Mil Sombras | 650 | 442 | 1.47 |
| sylene | Absolute Zero | 833 | 575 | 1.45 |
| valdric | Imperial Dominion Ultimate | 910 | 364 | 2.50 |
| volta | Core Meltdown | 640 | 410 | 1.56 |

Every ultimate drains all 3000 power when it starts.

## Evidence

- `ultimate-akio.gif`, `ultimate-suzume.gif`, `ultimate-oboro.gif`: the ultimate animation captured
  in game by the clips tour.
- `ultimate-akio.png`, `ultimate-suzume.png`, `ultimate-oboro.png`: the impact frame.
- `move-list-3-bars.png`: the pause menu move list showing the ultimate at the top with `3 BARS`.
- `web-fight.png`: the wasm build running in headless Chromium (`scripts/web-smoke.js`, `WEB SMOKE OK`).

## Reproduce

```sh
cargo test --release -p ume-engine --test native_ultimates -- --nocapture
cargo test --release -p ume-engine --test native_roster_lab -- --nocapture
UME_TOUR=clips UME_CHARS=suzume,oboro,akio UME_CLIP_STRIDE=4 ultimate-mugen --shots <dir> --res 1280x720 --mute
ume-tools ability-reel <dir> <out> 640 64 12 --no-normals
UME_SHOTS_ONLY=pause ultimate-mugen --shots <dir> --res 1280x720 --mute
```

