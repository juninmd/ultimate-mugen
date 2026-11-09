# Bleach-inspired kits

Three native fighters borrow ideas from Bleach characters. Everything is data in `assets/chars/*.json`; no sprites, stages or engine code changed, and the new moves reuse each fighter's existing animations.

## Kaname Fuyutsuki (Byakuya Kuchiki)

Noble, precise swordsman: fast steps, thin cuts, petals.

| Move | Input | Idea |
|---|---|---|
| Senbonzakura: Petal Scatter (`frostarc`) | `~D, DF, F, x` | Blade scattered into petals |
| Hado 4: Byakurai (`byakurai`) | `~D, DF, F, y` | Very fast thin beam |
| Hado 31: Shakkaho (`shakkaho`) | `~D, DB, B, x` | Heavy red fireball, two hits |
| Shunpo: Silent Step (`blinkcut`) | `~D, DB, B, a` | Forward flash step cut |
| Shunpo: Hirenkyaku (`shunpo_behind`) | `D, D, a` | Flash step that ends in a cut |
| Bankai: Senbonzakura Kageyoshi (`kageyoshi`) | `~D, DF, F, D, DF, F, y` | Super: six-lane petal storm |

The awakened form is renamed Bankai: Senbonzakura and meter regeneration rises to 0.5.

## Suzume Kagerou (Yoruichi Shihoin)

Speed fighter: `Shunpo: Flash Goddess` (`shadow`), `Shunko: Lightning Palm` (`shunko`, `~D, DB, B, y`, three-hit teleport strike) and `Flash Cat Burst` (`flash_cat`, `D, D, b`). Stack speed gain rises to 0.04 per stack.

## Shion Kurogane (Kenpachi Zaraki)

Brute with a greatsword: `Reckless Reiatsu Swing` (`reckless`, `D, D, x`, two-hit armor), `Kenpachi Cleave` (renamed `cleave`) and a rage passive from 50% life (up to 1.5x) on top of the existing time enrage.
