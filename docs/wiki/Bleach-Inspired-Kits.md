# Bleach-inspired kits

Three native fighters borrow abilities from Bleach characters. Names stay as they are: it is inspiration and copied abilities only. Everything is data in `assets/chars/*.json`; no sprites, stages or engine code changed, and new moves reuse each fighter's existing animations.

## Kaname Fuyutsuki (Byakuya-style)

Noble, precise swordsman: fast steps, thin cuts, petals.

| Move | Input | Idea |
|---|---|---|
| Frostmoon Arc (`frostarc`) | `~D, DF, F, x` | Blade scattered into petals |
| Piercing Needle Beam (`byakurai`) | `~D, DF, F, y` | Very fast thin beam |
| Crimson Cannon (`shakkaho`) | `~D, DB, B, x` | Heavy red fireball, two hits |
| Flash Step Behind (`shunpo_behind`) | `D, D, a` | Flash step that ends in a cut |
| Thousand Petal Storm (`kageyoshi`) | `~D, DF, F, D, DF, F, y` | Super: six-lane petal storm |

Meter regeneration rises to 0.5.

## Suzume Kagerou (Yoruichi-style)

Speed fighter: `Lightning Palm` (`shunko`, `~D, DB, B, y`, three-hit teleport strike) and `Flash Cat Burst` (`flash_cat`, `D, D, b`). Stack speed gain rises to 0.04 per stack.

## Shion Kurogane (Zaraki-style)

Brute with a greatsword: `Reckless Reiatsu Swing` (`reckless`, `D, D, x`, two-hit armor) and a rage passive from 50% life (up to 1.5x) on top of the existing time enrage.

## Byakuren Sakuraba (Secret Story Mode Unlock — Full Byakuya-Inspired Fighter)
Unlocked automatically upon clearing Story Mode (`stats.story.clear >= 1` in `assets/routes/unlocks.toml`). Features a dedicated sakura-blossom & black-shihakusho pixel-art sprite set (`assets/chars/sprites/byakuren/`), 6 original synthesized WAV sound effects (`snd_200_0..5.wav`), and exploits the full UMF 2.0 engine:
- **Core Abilities**: Air-Walk (`fly`), *Shunpo* (`teleport`), *Senka* (`superSpeed`), *Reiatsu Charge* (`charge`), and *Utsusemi* autonomous petal clone (`helpers.clones`).
- **Kido & Shikai**: *Chire, Sakuramai* (ground + air), *Hadō #4: Byakurai*, *Hadō #33: Sōkatsui*, *Bakudō #61: Rikujōkōrō*, *Bakudō #81: Dankū*, and *Shunpo: Senka* behind-teleport strike.
- **2-Stage Bankai Transformations**: Form 1 (*Bankai: Mil Pétalas da Sombra* — unlocks *Kageyoshi: Petal Wave* & *Gokei: Blossom Dome*) and Form 2 (*Shukei: Hakuteiken* — unlocks the 2-bar White Emperor ultimate).
