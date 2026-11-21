# In-Fight Easter Eggs

Ultimate MUGEN includes a presentation-only in-fight Easter Egg system (`src/easter.rs` and `src/easter/`) that reacts to classic fighting-game moments, stage phenomena, and fighter lore without touching the deterministic simulation (`crates/engine`).

## Architecture
- **Deterministic safe**: reads `&Match` state on the presentation thread (`src/app/fight_screen.rs`), never mutates simulation state, and keeps rollback/replay checksums unchanged.
- **Localized**: every banner, speech bubble, and cameo caption ships in English, Portuguese, and Spanish via `crate::i18n::tr3`.
- **Pixel-art animated sprites**: compact multi-frame RGBA sprite sheets in `assets/easter/` rendered with `FilterMode::Nearest`.
- **Non-intrusive pacing**: per-round and per-match bitset tracking, global cooldowns, and a 3-item on-screen cap keep fights readable.

## Combat & Arcade Easter Eggs (20)
1. **TOASTY!** — Corner pop-in referee cameo on heavy airborne launches (`damage >= 115`).
2. **DOUBLE K.O. DRAMA!** — Both fighters knocked out simultaneously.
3. **MAGIC PIXEL!** — Winning a round or landing a Super below 5–6% life.
4. **UNTOUCHABLE!** — Flawless 100% life round victory with golden crown badge.
5. **CLASH OF FATES!** — Simultaneous mutual hit trade within 3 ticks.
6. **TAUNT TO GET BODIED!** — Getting hit within 90 ticks of taunting.
7. **HONORABLE DUEL!** — Both fighters taunt within 120 ticks of each other.
8. **SYNC JUMP!** — Both fighters jump on the exact same frames twice in a round.
9. **TACTICAL DANCE PARTY!** — Rapid crouch-spam 4+ times at long range.
10. **SPEEDRUN K.O.!** — Winning a round in under 10 seconds.
11. **BUZZER BEATER!** — Round decided with 3 or fewer seconds remaining.
12. **GRAVITY OFF!** — Landing a 5+ hit air juggle combo.
13. **IRON FORTRESS!** — Blocking 6+ hits cleanly in a single sequence.
14. **GUARD SHATTERED!** — Guard break (`state_no == 5300`).
15. **DOUBLE SUPER CLASH!** — Both sides fire a Super within 90 ticks.
16. **LIMIT BREAK DUEL!** — Both fighters awaken/transform in the same match.
17. **STOP, HE'S ALREADY DOWN!** — 15+ hit overkill combo with referee pop-in.
18. **GREAT CORNER ESCAPE!** — Reversing out of a prolonged corner trap with a heavy hit.
19. **HIGH NOON STAREDOWN...** — Both fighters stand idle facing each other for 3 seconds, rolling a pixel-art tumbleweed across the floor.
20. **INSERT COIN — MAX METER!** — Reaching full super meter drops a spinning 1UP arcade coin flyby.
