# Fighting game research: opportunities

**English** · [Português (BR)](pt-BR/Pesquisa-Jogos-Luta.md)

> 🇧🇷 Pesquisa de recursos públicos de outros jogos de luta e lista priorizada de melhorias. Nada aqui está implementado ainda.

This page compares publicly known features of modern 2D and 2.5D fighters with the current state of Ultimate MUGEN and ranks concrete opportunities. It is a planning document: **no feature listed here is implemented yet**. Only publicly documented features are cited, and no third-party assets are copied.

Effort scale: **S** under 2 days, **M** about 1 week, **L** 2 to 4 weeks, **XL** over a month.

## Games surveyed

| Game | What it is known for |
|---|---|
| Dragon Ball FighterZ | Anime-fidelity visuals, 3v3 assists, simple auto-combo for beginners, ranked lobbies with arcade-style lobby avatars |
| Sparking! Zero | Large roster, destruction and arena feedback, a long tutorial and a battle-rules customizer |
| Melty Blood: Type Lumina | Moon styles, beginner-friendly auto-combos, rollback, lobbies |
| Guilty Gear -Strive- | Rollback with a lobby of "park" avatars, a deep tutorial and mission mode, training mode with full frame data |
| BlazBlue Cross Tag Battle | Simple 2-button layout, assist system, rollback after patching |
| Street Fighter 6 | Modern controls, Drive gauge HUD, in-game frame meter, replay sharing, Battle Hub, tutorial and character guides |
| Mortal Kombat 1 | Kameo assists, Krypt-style extras, training with tutorial for each character |
| Tekken 8 | Practice mode with heat and rage, ghost battle, replay and tips, character-specific guides |
| Them's Fighterz, Skullgirls | Rollback, training mode with strong tools (Skullgirls in particular), colorblind and visual options, in-game tutorial |
| Persona 4 Arena Ultimax | Hit-stop and cinematic supers, story presentation |
| The King of Fighters XV | Team select with order, 3v3 sequence matches, rollback, practice mode |
| Under Night In-Birth II | Compact 6-button layout, GRD system HUD, rollback, deep training |
| Ikemen GO and the M.U.G.E.N community | Open screenpacks, lifebar and select customization, `fight.def` and `system.def` skinning, netplay with rollback, Lua scripting, common training options |

## Current state of Ultimate MUGEN

Checked in the repository on 2026-11-04.

| Area | Today |
|---|---|
| Modes | Arcade, Versus CPU, 2 players, Training, CPU vs CPU, Team sequence, Team simul (`Mode` in `src/app.rs`) |
| Training | Dummy pilot (`Pilot::Dummy`), infinite meter, hitbox toggle in the pause menu |
| Pause menu | Continue, Hitboxes, Restart, Change fighters, Settings, Main menu |
| HUD | Glass panels, big portraits, combo counter, low-life pulse, MAX meter, urgent timer (PR #156) |
| Impact feedback | Particles, shake, hit-stop, KO slow motion, super cut-in, chromatic aberration, effect intensity setting |
| Settings | Tabs for Video, Audio, Controls, Gameplay, Language, About |
| Netplay | Rollback with GGRS over UDP through CLI flags only (`--netplay`, `--peer`); `.umr` replay files; no in-game lobby, no browser or mobile transport yet |
| Replays | Re-simulation of the last finished fight from recorded inputs; replay file format exists |
| Content | Marketplace from GitHub repos, `.def`/`.zip` import, native UMF fighters |
| Tutorial and trials | None. Combo trials, survival and boss rush are on the roadmap (section 5) |

## Prioritized opportunities

Priority: **P0** do first, **P1** next, **P2** later, **P3** nice to have.

### P0: biggest gaps against what players expect today

| # | Opportunity | Rationale | Effort | Touches |
|---|---|---|---|---|
| 1 | **Training mode toolkit v2**: dummy behaviors (stand, crouch, jump, block all, block after first hit), dummy recording and playback loop, reset-position hotkey with saved start positions, life and meter refill modes | Practice modes in SF6, Tekken 8, Guilty Gear -Strive- and Skullgirls all offer record and replay of the dummy plus state resets. Without them training is just a hitbox view | M | `Pilot::Dummy` in `src/session.rs`, pause menu in `src/app.rs`, new HUD overlay in `src/hud.rs` |
| 2 | **Input display and frame data overlay**: scrolling input history with frame counts, plus last-move startup, active, recovery, advantage and damage | SF6 and Strive show input history and frame advantage in training; this is the main way players learn matchups. The engine already runs a deterministic 60 Hz simulation so the numbers are cheap to derive | M | `src/hud.rs`, `crates/engine` (read-only hooks on `Match`), `src/session.rs` |
| 3 | **In-game online screen**: host or join by code, direct IP, input delay and ping shown before the match, rematch button | Rollback exists but only through CLI flags. Every console fighter exposes lobbies and a rematch loop. Start with code-based rooms using the `matchbox` signalling the architecture decisions already chose | L | `src/netlink.rs`, `crates/netplay`, new `Screen::Online` in `src/app.rs` |
| 4 | **Netcode quality UI**: connection graph (ping, jitter, rolled-back frames), per-match stats and a "poor connection" warning before accepting a match | Guilty Gear -Strive- and SF6 expose connection quality bars; the ROADMAP section 1 already lists this | S | `src/hud.rs`, `crates/netplay` (expose stats) |
| 5 | **Guided tutorial**: basics (move, block, special, super), then per-character move list with live input display | SF6, Tekken 8, Strive and Skullgirls ship tutorials; MUGEN-style engines normally have none, so newcomers have no onboarding | L | New `Screen::Tutorial`, `src/app/cutscene.rs` reuse for text panels |

### P1: strong value, moderate cost

| # | Opportunity | Rationale | Effort | Touches |
|---|---|---|---|---|
| 6 | **Combo trials** from a per-character JSON list, with on-screen next-input prompts and success feedback | Guilty Gear, SF6, DBFZ and Melty Blood have trials; the ROADMAP already lists it. Fits UMF, where creators can ship trials with the fighter | M | New trial runner on top of `Session`, UMF schema in `docs/CHARACTER_FORMAT.md` |
| 7 | **Simple (modern) control scheme**: optional one-button specials and auto-combo, toggled per player, with a matching HUD marker | SF6 Modern, DBFZ, Melty Blood and BlazBlue Cross Tag all have simplified inputs, which matters for touch screens in particular | M | `src/controls.rs`, `src/keybinds.rs`, command layer in `crates/engine` |
| 8 | **Accessibility pass**: colorblind-safe HUD palettes, flash and shake reduction as separate toggles, larger HUD text, subtitle sizes, high-contrast hitbox colors, hold-to-toggle for held inputs | Several current fighters offer colorblind modes and reduced flash. The effect intensity setting exists but is a single knob | M | `src/settings.rs`, `src/app/settings_screen.rs`, `src/vfx.rs`, `src/hud.rs` |
| 9 | **Replay viewer upgrade**: replay list screen, playback speed, pause, frame step, camera control, "take over" a character mid-replay, save and share `.umr` | SF6, Tekken 8 and Strive allow replay browsing and sharing; the file format and re-simulation are already deterministic | M | `src/app.rs` (`Replay`), `crates/netplay` (`.umr`), new `Screen::Replays` |
| 10 | **Ranked and matchmaking queue** (server needed): rating, anonymous queue, rematch, win streak | Standard in every console fighter. Requires a small service, so it comes after lobbies prove stable | XL | New service plus `Screen::Online` |
| 11 | **Select screen upgrades**: character info card (moves, difficulty), stage preview, palette and color preview, random-select roll animation, shuffle favorites, filter by tag and source | KOF XV and DBFZ preview palettes and show team order; Ikemen GO screenpacks support slots, random and stage select. Already partly in `crates/formats/src/screenpack.rs` | M | `src/app.rs` select code, `src/menu_art.rs` |
| 12 | **Pause and result screens**: move list, command list, key-mapping summary, rematch and "change character" shortcuts, post-fight stats (hits, damage, max combo, meter use) | Move list in the pause menu is a long-standing fighter norm (Tekken, SF6, MK). Results screen already exists | S | `src/app.rs` pause menu and `Screen::Results` |

### P2: depth and polish

| # | Opportunity | Rationale | Effort | Touches |
|---|---|---|---|---|
| 13 | **Training options page**: infinite or regenerating life, meter modes, damage readout, hit-stun display, combo damage and scaling readout | Available in practically every training mode | S | Pause menu, `src/hud.rs` |
| 14 | **Survival, boss rush and tower modes** with rewards | Present in Mortal Kombat 1 (towers), Sparking! Zero and Tekken; ROADMAP section 5 | M | New `Mode` variants, `Team` reuse |
| 15 | **Arcade ladder presentation**: rival matches, character-specific intros and endings from storyboard files | Arcade ladders with rival encounters are standard in Tekken, MK and KOF; `Cutscene` screen already supports story scenes | M | `src/app/cutscene.rs`, storyboard loader |
| 16 | **Spectator mode** for rollback matches (watch a live room) | Tournament and community play on Strive and SF6 use this heavily | L | `crates/netplay`, `Screen::Online` |
| 17 | **Per-match battle rules**: round count, timer, damage scale, gravity, handicap | Sparking! Zero's custom rules, MK modifiers, and MUGEN community configs | S | Rules struct (`Rules` already exists in `Replay`) |
| 18 | **Frame-meter and hit-box color legend** drawn in real time (strike, hurt, throw, projectile) with a legend | Matches the frame meter in SF6 and the hitbox modes in Skullgirls training | M | `src/render.rs` debug draw |
| 19 | **Controller UX**: input latency test, gamepad button test, per-device profiles, dead-zone and SOCD choice | Needed for stick and pad players; a latency test page is common in fighters | S | `src/controls.rs`, `src/gamepad.rs` |
| 20 | **Menu feel**: sound and animation for every cursor change, hold-to-repeat navigation, mouse and touch parity, back-button behaviour on mobile | Polish seen across all surveyed games | S | `src/ui.rs`, `src/app.rs` |

### P3: ecosystem and long term

| # | Opportunity | Rationale | Effort | Touches |
|---|---|---|---|---|
| 21 | **Screenpack skin support** for lifebars and select screens (`fight.def`, `system.def`) rendered on wgpu | The biggest reuse of the M.U.G.E.N community's work; loaders exist in `crates/formats` and rendering was waiting on the wgpu migration, which is now done | XL | `crates/formats/src/screenpack.rs`, `src/hud.rs`, `src/menu_art.rs` |
| 22 | **Mod and marketplace discovery**: ratings, tags, screenshots, update notifications | Mod-friendly games (Ikemen GO, Skullgirls community tools) rely on discoverability; marketplace already ships | M | `crates/market`, `src/app/market.rs` |
| 23 | **Character-specific tips and move guides** authored by creators in UMF | Same idea as the guides in SF6 and Tekken 8 | S | UMF schema, tutorial screen |
| 24 | **Tournament tools**: bracket from `.umr` replays, offline match recorder, overlay for streamers | Fits the roadmap section 2 | L | New tooling crate |
| 25 | **Localization expansion** (Spanish, Japanese) and screen-reader labels | ROADMAP section 5 | M | `src/i18n.rs` |

## Suggested sequence

1. Training toolkit (1, 2, 13), because every other feature is easier to verify with it.
2. Netcode UI and online screen (4, 3), then replays (9).
3. Tutorial and trials (5, 6), then accessibility and modern controls (8, 7).
4. Select and results polish (11, 12), then modes (14, 15).
5. Ranked service and spectator (10, 16) once lobbies are stable.
6. Screenpack rendering (21) as a separate track.

## Notes and caveats

- Ikemen GO is credited as inspiration only, like in the Home page.
- Feature lists reflect public information about each game as of the research date and may change with patches. Treat them as reference points, not requirements.
- Input history and frame data tools (2) must read the match state only; they must not change the simulation, to keep rollback determinism.
- Anything that adds fields to `Match` should also be included in `ume_netplay::checksum`.
