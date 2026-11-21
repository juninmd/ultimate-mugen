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

## Stage-Specific Environmental & Cameo Easter Eggs (20)
21. **Dojo — Master's Paper Crane**: An animated origami crane drifts across the sunset roof.
22. **Dojo — Rooftop Tremor**: Heavy impacts shake the ancient dojo roof tiles.
23. **Neo-Metrópole 2099 — Cyber UFO Sighting**: A blinking neon flying saucer zips across the skyline.
24. **Neo-Metrópole 2099 — System Overload // 2099**: Firing a Super triggers a neon grid surge banner.
25. **Moonlit Bamboo — Moonlit Bamboo Spirits**: Spirit masks drift through the grove during combos.
26. **Rocky Plains — El Viento Salado**: Marlow's flying galleon crosses the distant clouds.
27. **Spirit Gate — Kitsune Apparition**: A glowing kitsune mask floats above the torii gate on Supers or Awakenings.
28. **Volcanic Crater — Magma Geyser Spike**: Heavy hits trigger a seismic caldera roar.
29. **Praça do Interior (Brazil) — Chill Capybara Cameo**: A pixel-art capybara with a tangerine on its head waddles across the plaza.
30. **Praça do Interior (Brazil) — Caramelo Dog on Patrol**: Plaza guardian cameo on Round 2+.
31. **Christ the Redeemer (Corcovado) — Corcovado Toucan Flyby**: A vibrant pixel-art toucan glides across the Rio sky.
32. **Great Wall of China — Sky Dragon Kite**: A paper kite dances along the mountain ridge.
33. **Petra (Al-Khazneh) — Rose Canyon Scarab**: A golden scarab shimmers over the canyon on heavy impacts.
34. **Texas — High Noon in Texas**: A desert tumbleweed rolls down Main Street at high noon.
35. **Chichén Itzá — Feathered Serpent Shadow**: Kukulcán's emerald glow ripples across the temple on Supers.
36. **Colosseum of Rome — Ave Gladiator!**: The Emperor rewards 6+ hit combos with a golden laurel badge.
37. **Taj Mahal — White Doves of Agra**: Doves take flight over the marble palace on Round 2+.
38. **Pyramids of Giza — Pharaoh's Eye Glint**: Reaching full meter awakens the golden scarab of Giza.
39. **Hanging Gardens of Babylon — Hanging Gardens Peacock**: Emerald-gold plumage glides overhead on air combos.
40. **Ancient Wonders Blessing** (Statue of Zeus, Lighthouse of Alexandria, Temple of Artemis, Colossus of Rhodes, Mausoleum of Halicarnassus): Unique mythic phenomenon per classical wonder.
