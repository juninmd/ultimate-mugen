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

## Lore Matchup, Mirror & Fighter Easter Eggs (20)
41. **Mirror Match Banter**: Any fighter facing themselves triggers an "impostor" speech bubble.
42. **Suzume vs Oboro**: Kagerou clan vengeance banner with crimson eclipse emblem.
43. **Sylene vs Garan**: Borealia's Arctic ice vs Underworld ashen flames.
44. **Kaito vs Oboro**: Storm of Izu vs the Scarlet Eclipse.
45. **Bruno on Brazil / Corcovado**: Home crowd cheers on the three-time Minas Gerais champion.
46. **Volta on Neo-Metrópole 2099**: Free-will firmware speech bubble back at its birthplace.
47. **Akio Fired Up (<30% HP)**: Starfall Dojo optimism kicks in with a steaming ramen badge.
48. **Haruki's Apology**: Gentle archivist apologizes mid-fight after landing a 5+ hit combo.
49. **Marlow vs Valdric**: Sky pirate playfully eyes the Red Star Empire's crown.
50. **Kaito vs Jin**: Wind-runner rivalry between Izu Cliffs and Wind Valley.
51. **Shion vs Kaname**: Meteorite blade meets Hokkaido's spirit katana (`crossed_blades`).
52. **Lian on Great Wall**: Jade Lantern grandmaster defending the Great Wall.
53. **Bruno vs Daichi**: Heavyweight grappler showdown — Iron Ring vs Altai Wolf.
54. **Garan on Volcanic Crater**: The Ashen Gate opens where 99 monks sealed him.
55. **Valdric vs Oboro**: Two crowns, one throne — Red Star Empire vs Scarlet Eclipse.
56. **Suzume vs Jin**: High-speed ninja vs courier duel.
57. **Volta vs Haruki**: Android asks the gentle archivist what makes a heart courageous.
58. **Akio vs Kaito**: Classic shonen rivalry — "Loser buys ramen!"
59. **Marlow's Pirate Parrot**: *El Viento Salado*'s first mate swoops across the screen on Supers or 6+ hit combos.
60. **Story Thug Bravado (`npc_*`)**: Gang members complain about their paycheck when caught in a 4+ hit combo.

## Custom Synthesized Sound Effects (`src/audio/easter_sfx.rs`)
All 60 Easter Eggs trigger one of 12 bespoke 32 kHz synthesized arcade SFX cues preloaded alongside the built-in sound bank (desktop, web/WASM, Android, iOS):
- `easter_toasty` — Rising retro arcade vocal/brass stinger with slapback echo (*TOASTY!*, *Taunt to Get Bodied*, *Overkill*, *Buzzer Beater*).
- `easter_coin` — Classic 2-note golden arcade coin / 1UP chime (*Max Meter*, *Sync Jump*, *Air Juggle*, *Speedrun K.O.*).
- `easter_crown` — Shimmering major-7th victory arpeggio fanfare (*Flawless Perfect*, *Magic Pixel*, *Double K.O.*, *Limit Break Duel*, *Colosseum*).
- `easter_clash` — Dual detuned metallic blade/energy clash (*Clash of Fates*, *Guard Fortress*, *Double Super Clash*, *Shion vs Kaname*, *Suzume vs Oboro*).
- `easter_tumbleweed` — Spaghetti-western desert whistle with airy wind sweep (*High Noon Staredown*, *Texas*).
- `easter_ufo` — Sci-fi theremin LFO saucer wobble (*Neo-Metrópole UFO*, *System Overload // 2099*, *Volta*).
- `easter_kitsune` — Mystical shrine bell partials & wooden clack (*Spirit Gate Kitsune*, *Dojo Crane*, *Bamboo Spirits*, *Great Wall*, *Lian*).
- `easter_capybara` — Warm tropical marimba/kalimba bounce (*Brazil Capybara*, *Caramelo*, *Bruno Home Crowd*, *Dance Party*).
- `easter_bird` — Upward tropical bird chirp & wing trill (*Corcovado Toucan*, *Taj Mahal Doves*, *Hanging Gardens Peacock*, *Kaito vs Jin*).
- `easter_parrot` — Expressive pirate parrot double-squawk (*Marlow's Parrot*, *El Viento Salado*).
- `easter_wonder` — Ancient mythic pentatonic harp & deep bronze gong (*Pyramids*, *Petra*, *Chichén Itzá*, *Volcano*, *Classical Wonders*, *Garan*).
- `easter_banter` — Snappy comic speech-bubble pop & chime (*Mirror Match*, *Haruki Apology*, *Marlow vs Valdric*, *Akio vs Kaito*, *Thug Complaint*).

## TOML Customization (`assets/interactions.toml`)
Taunts, fighter rivalries, home-turf stage reactions, and in-fight character triggers are all data-driven in `assets/interactions.toml`. Players and creators can place an `interactions.toml` file in their user storage directory to override taunt pools or append custom interactions for both native UMF fighters and imported M.U.G.E.N characters without recompiling:

```toml
[[matchup]]
id = "custom_rivalry"
fighters = ["ryu", "ken"]
speaker_id = "ken"
speaker = "KEN"
style = "speech"
sfx = "easter_banter"
color = [1.0, 0.45, 0.2]
title_pt = "Vamos nessa, Ryu! Sem segurar!"
title_en = "Come on, Ryu! Don't hold back!"
title_es = "¡Vamos, Ryu! ¡Sin contenerse!"
```

Supported sections:
- `[taunts]`: `opening_pt/en/es` and `answer_pt/en/es` round-start taunt pools.
- `[[matchup]]`: `fighters = [a, b]` or `mirror = true`, `style`, `sprite`, `sfx`, `color`, localized `title_*` and `sub_*`.
- `[[stage_interaction]]`: `fighter` + `stage_contains = [...]`.
- `[[fighter_trigger]]`: `fighter` or `fighter_prefix`, `trigger` (`"low_hp"`, `"combo"`, `"opp_combo"`, `"super"`, `"super_or_combo"`, `"taunt"`), `threshold`, `per_round`.

## Two-Part Rival Exchanges (`[[exchange]]`) & Full-Roster Taunts
`assets/interactions.toml` also supports back-and-forth two-part `[[exchange]]` dialogues (Fighter A speaks, and Fighter B automatically replies `delay_ticks` later), plus `first_attack`, `transform`, and `round_win` triggers, and dedicated `taunt` lines for all 16 playable fighters.
