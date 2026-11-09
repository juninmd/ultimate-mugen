# Ultimate MUGEN Wiki

**English** · [Português (BR)](pt-BR/Home.md)

A guide for players and creators who want to customize the engine: install, configure, create fighters, stages and scripts.

| Page | Audience | Covers |
|---|---|---|
| [Installation](Installation.md) | Players | Download, build, run on desktop, browser and phone |
| [Automatic updates](Auto-Update.md) | Players | Self-update from GitHub releases, opt-outs and CLI flags |
| [Configuration and controls](Configuration.md) | Players | `.cfg` file, command-line flags, environment variables, controls |
| [Adding M.U.G.E.N content](MUGEN-Content.md) | Players | Install `.def`/`.zip` characters and stages, and validate them |
| [GitHub marketplace](Marketplace.md) | Players | Install characters, stages and music from any GitHub repository |
| [Story mode](Story-Mode.md) | Players | Five scrolling stages, waves, bosses, cutscenes and health carry-over |
| [Audio](Audio.md) | Players & Creators | Character sounds, music per stage, formats and panning |
| [Character select](Character-Select.md) | Players | Select screen, portraits, teams and shortcuts |
| [Game modes](Game-Modes.md) | Players | Arcade, teams (Sequence, Parallel, Tag) and their rules |
| [Universe and Lore](Lore.md) | Players & Creators | Cosmology, factions, energy systems and the Great Convergence |
| [Character Dossiers](Characters.md) | Players & Creators | Roster bios, portraits, moves, rivalries and home stages |
| [Compatibility decisions](Compatibility-Decisions.md) | Creators | Features the engine rejects by design and why |
| [Creating a UMF character](UMF-Character.md) | Creators | A full fighter in a single JSON |
| [Passives](Passives.md) | Creators | Rage, ability stacks and regeneration, data-driven |
| [Skeleton art (rig)](Rig.md) | Creators | Style, poses, joints and palettes |
| [CNS scripts](CNS-Scripts.md) | Creators | States, controllers, triggers and commands |
| [AssertSpecial flags](AssertSpecial.md) | Creators | All 77 Ikemen flags and what each one does |
| [Red life, dizzy and guard points](Life-Systems.md) | Creators | RedLife, DizzyPoints, GuardPoints, states 5300/5400 and HitDef defaults |
| [Screenpacks](Screenpacks.md) | Creators | Menu and HUD screenpack support |
| [Sprite atlas import](Sprite-Atlas-Import.md) | Creators | Turn a sprite sheet into a fighter |
| [Dev tools](Dev-Tools.md) | Contributors | Rust build and audit tools (`ume-tools`) |
| [Sprite toolkit](Sprite-Toolkit.md) | Creators | Slice sheets, palette variants, auto collision boxes |
| [Move generator](Move-Generator.md) | Creators | TOML to AIR, CNS, CMD and Statedef -1 |
| [ZSS scripts](ZSS.md) | Creators | Ikemen ZSS state scripts |
| [Ikemen extensions](Ikemen-Extensions.md) | Creators | Ikemen GO controllers and triggers the engine accepts |
| [Creating stages](Stages.md) | Creators | M.U.G.E.N `.def` stages and native stages |
| [Customizing the engine](Engine.md) | Developers | Architecture, new controller, new trigger, tests |
| [File layout](File-Layout.md) | Developers | The 250-line rule, one responsibility per file, splitting a file, parallel PRs |
| [Performance](Performance.md) | Developers | Parallel loading, frame pacing, benchmarks |
| [Reverse engineering the browser port](Ikemen-Web-Reverse-Engineering.md) | Developers | How a 78-fighter M.U.G.E.N build runs in a browser, and what we took from it |
| [Replays](Netplay-and-Replays.md#saved-replays-in-the-game-replays) | Players | Saved fights, replay browser, playback speed |
| [Research: fighting games](Research-Fighting-Games.md) | Developers | Survey of other fighters and prioritized improvement list |
| [Stress and resilience](Stress-and-Resilience.md) | Developers | Fuzzing, spawner caps, hot path numbers |
| [Native animation audit](Native-Animation-Audit.md) | Developers | Pose coverage of the native roster |
| [Research: story mode](Research-Story-Mode.md) | Developers | Survey behind the story mode design |
| [Media capture](Media-Capture.md) | Developers | Screenshots and GIFs from the web build in headless Chromium |
| [Troubleshooting](Troubleshooting.md) | Everyone | Common errors and diagnostic tools |

## Accepted formats

| Format | Use | Files |
|---|---|---|
| UMF | Native declarative fighter | `assets/chars/<id>.json` |
| M.U.G.E.N | Classic fighters and stages | `.def`, `.sff` v1/v2, `.air`, `.cmd`, `.cns`, `.snd`, `.act` |

UMF and M.U.G.E.N fighters run on the same state machine and can fight each other.

## Roadmap

Compatibility plans and architecture decisions are in [`docs/ROADMAP.md`](../ROADMAP.md). Items marked **planned** in this wiki do not exist in the current version yet.

## Compatibility status

The progress toward Ikemen GO v1.0.0 parity is tracked in [issue #135](https://github.com/juninmd/ultimate-mugen/issues/135), with one issue per missing controller, trigger or feature. The lists of supported controllers, triggers and redirections in [CNS scripts](CNS-Scripts.md) are checked against the engine by `cargo test -p ume-engine --test wiki_sync`: a listed name the engine does not handle, an implemented name that is not listed, or a mismatch between the English and Portuguese pages fails the test. Every character load also reports the controllers, triggers and parameters it uses that the engine ignores (see "Load diagnostics" in the same page).

## Acknowledgments

Thanks to the [Ikemen GO](https://github.com/ikemen-engine/Ikemen-GO) team and to Elecbyte, creator of M.U.G.E.N, for the inspiration.
